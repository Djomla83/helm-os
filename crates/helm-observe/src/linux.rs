//! Linux acquisition backend: constrained `openat2`, classification on the
//! pinned descriptor, and a procfs reopen reached only by a classified regular
//! file.
//!
//! Implemented entirely with safe `rustix` wrappers; the crate forbids unsafe.

use std::io::Read as _;
use std::os::fd::{AsFd as _, OwnedFd};

use rustix::fs::{
    AtFlags, FileType, Mode, OFlags, PROC_SUPER_MAGIC, ResolveFlags, StatxFlags, fstatfs, openat,
    openat2, statx,
};
use rustix::io::Errno;

use crate::error::{AdmissionError, AdmissionErrorCode as A};
use crate::model::{Failure, ObjectKind, ObjectTuple, Rejection};

/// Filesystem magic for ext4 (shared with ext2/ext3).
const EXT_SUPER_MAGIC: i64 = 0xEF53;

/// The reviewed constraint set, applied to every target resolution.
///
/// `NO_SYMLINKS` already implies `NO_MAGICLINKS`; both are named so the policy
/// reads explicitly. This set is never weakened, and there is no `openat`
/// fallback for target pathname resolution.
pub(crate) const RESOLVE: ResolveFlags = ResolveFlags::BENEATH
    .union(ResolveFlags::NO_SYMLINKS)
    .union(ResolveFlags::NO_MAGICLINKS)
    .union(ResolveFlags::NO_XDEV);

const STATX_MASK: StatxFlags = StatxFlags::TYPE
    .union(StatxFlags::MODE)
    .union(StatxFlags::NLINK)
    .union(StatxFlags::SIZE)
    .union(StatxFlags::INO)
    .union(StatxFlags::MTIME)
    .union(StatxFlags::CTIME)
    .union(StatxFlags::MNT_ID);

pub(crate) struct Meta {
    pub kind: ObjectKind,
    pub tuple: ObjectTuple,
    pub size: u64,
    pub link_count: u64,
    mtime: (i64, u32),
    ctime: (i64, u32),
}

fn kind_of(mode: u16) -> ObjectKind {
    match FileType::from_raw_mode(u32::from(mode)) {
        FileType::RegularFile => ObjectKind::Regular,
        FileType::Directory => ObjectKind::Directory,
        FileType::Symlink => ObjectKind::Symlink,
        FileType::Fifo => ObjectKind::Fifo,
        FileType::Socket => ObjectKind::Socket,
        FileType::CharacterDevice => ObjectKind::CharacterDevice,
        FileType::BlockDevice => ObjectKind::BlockDevice,
        FileType::Unknown => ObjectKind::Other,
    }
}

/// Metadata of the object a descriptor already pins. Descriptor-relative, never
/// a pathname re-stat, so classification describes exactly the pinned object.
pub(crate) fn describe(fd: &OwnedFd) -> Result<Meta, Errno> {
    let s = statx(fd.as_fd(), "", AtFlags::EMPTY_PATH, STATX_MASK)?;
    let mount_id = (s.stx_mask & StatxFlags::MNT_ID.bits()) != 0;
    Ok(Meta {
        kind: kind_of(s.stx_mode),
        tuple: ObjectTuple {
            device_major: s.stx_dev_major,
            device_minor: s.stx_dev_minor,
            inode: s.stx_ino,
            mount_id: mount_id.then_some(s.stx_mnt_id),
        },
        size: s.stx_size,
        link_count: u64::from(s.stx_nlink),
        mtime: (s.stx_mtime.tv_sec, s.stx_mtime.tv_nsec),
        ctime: (s.stx_ctime.tv_sec, s.stx_ctime.tv_nsec),
    })
}

/// Admit a root descriptor: a directory on the supported local cohort.
/// Metadata only; the directory is never enumerated.
pub(crate) fn admit_root(fd: &OwnedFd) -> Result<ObjectTuple, AdmissionError> {
    let meta = describe(fd).map_err(|_| AdmissionError::new(A::RootMetadataUnavailable))?;
    if meta.kind != ObjectKind::Directory {
        return Err(AdmissionError::new(A::RootNotDirectory));
    }
    let fs = fstatfs(fd.as_fd()).map_err(|_| AdmissionError::new(A::RootMetadataUnavailable))?;
    if fs.f_type != EXT_SUPER_MAGIC {
        return Err(AdmissionError::new(A::RootUnsupportedFilesystem));
    }
    Ok(meta.tuple)
}

/// Admit the procfs descriptor-directory capability.
///
/// Check A, filesystem type. Check B, self-identity: open the decimal name of a
/// descriptor this process already controls and require it to identify that same
/// object. A tmpfs or ext4 decoy fails A; another process's descriptor directory
/// fails B. Neither check probes a plan-controlled path.
pub(crate) fn admit_procfs(fd: &OwnedFd) -> Result<(), AdmissionError> {
    let fs = fstatfs(fd.as_fd()).map_err(|_| AdmissionError::new(A::ProcfsWrongFilesystem))?;
    if fs.f_type != PROC_SUPER_MAGIC {
        return Err(AdmissionError::new(A::ProcfsWrongFilesystem));
    }
    let known = describe(fd).map_err(|_| AdmissionError::new(A::ProcfsForeignOrUnusable))?;
    let number = rustix::fd::AsRawFd::as_raw_fd(&fd.as_fd()).to_string();
    // Deliberately without NO_SYMLINKS/NO_NOFOLLOW: this magic link must be
    // traversed so the probe observes the pinned object rather than the link.
    let probe = openat(
        fd.as_fd(),
        number.as_str(),
        OFlags::PATH | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|_| AdmissionError::new(A::ProcfsForeignOrUnusable))?;
    let seen = describe(&probe).map_err(|_| AdmissionError::new(A::ProcfsForeignOrUnusable))?;
    if seen.tuple.inode != known.tuple.inode
        || seen.tuple.device_major != known.tuple.device_major
        || seen.tuple.device_minor != known.tuple.device_minor
    {
        return Err(AdmissionError::new(A::ProcfsForeignOrUnusable));
    }
    Ok(())
}

/// What a constrained resolution attempt produced.
pub(crate) enum Resolved {
    Pinned(OwnedFd),
    Absent,
    Rejected(Rejection),
    Failed(Failure),
}

/// Pin a target beneath a held root without opening the object for data.
///
/// `O_PATH` means the object is not opened: `read`, `mmap` and `ioctl` on the
/// returned descriptor fail. That is what lets classification precede any data
/// access, including for a FIFO, socket or device node.
pub(crate) fn resolve(root: &OwnedFd, path: &str) -> Resolved {
    match openat2(
        root.as_fd(),
        path,
        OFlags::PATH | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        RESOLVE,
    ) {
        Ok(fd) => Resolved::Pinned(fd),
        Err(Errno::NOENT) => Resolved::Absent,
        Err(Errno::NOTDIR) => Resolved::Rejected(Rejection::WrongKind),
        // One kernel error covers both a mount crossing and a root escape, so no
        // attribution between them is invented.
        Err(Errno::XDEV) => Resolved::Rejected(Rejection::ScopeViolation),
        // A non-final symlink; a trailing one instead yields a pinned link that
        // classification rejects.
        Err(Errno::LOOP) => Resolved::Rejected(Rejection::SymlinkForbidden),
        Err(Errno::ACCESS | Errno::PERM) => Resolved::Failed(Failure::PermissionDenied),
        Err(Errno::AGAIN) => Resolved::Failed(Failure::ResolutionRace),
        Err(Errno::NOSYS | Errno::OPNOTSUPP) => Resolved::Failed(Failure::UnsupportedPlatform),
        Err(_) => Resolved::Failed(Failure::IoFailure),
    }
}

/// Outcome of streaming a classified regular file.
pub(crate) enum Streamed {
    Complete {
        bytes: u64,
        sha256: [u8; 32],
        link_count: u64,
    },
    Changed {
        partial: u64,
    },
    Failed {
        failure: Failure,
        partial: u64,
    },
}

/// Reopen exactly the pinned object through the procfs capability and stream it.
///
/// Only a descriptor already classified as a regular file reaches this function;
/// see [`crate::observe`]. The original pathname is never reopened.
pub(crate) fn reopen_and_stream(
    procfs: &OwnedFd,
    pinned: &OwnedFd,
    before: &Meta,
    read_buffer: &mut [u8],
    remaining_budget: u64,
) -> Streamed {
    let number = rustix::fd::AsRawFd::as_raw_fd(&pinned.as_fd()).to_string();
    let data = match openat(
        procfs.as_fd(),
        number.as_str(),
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOCTTY,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(Errno::ACCESS | Errno::PERM) => {
            return Streamed::Failed {
                failure: Failure::PermissionDenied,
                partial: 0,
            };
        }
        // A procfs-stage failure is never target absence.
        Err(_) => {
            return Streamed::Failed {
                failure: Failure::ReopenUnavailable,
                partial: 0,
            };
        }
    };

    // Verify the reopened descriptor identifies the pinned object before reading.
    let Ok(check) = describe(&data) else {
        return Streamed::Failed {
            failure: Failure::IoFailure,
            partial: 0,
        };
    };
    if check.kind != ObjectKind::Regular
        || check.tuple.inode != before.tuple.inode
        || check.tuple.device_major != before.tuple.device_major
        || check.tuple.device_minor != before.tuple.device_minor
    {
        return Streamed::Failed {
            failure: Failure::IoFailure,
            partial: 0,
        };
    }

    let mut file = std::fs::File::from(data);
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    let mut total: u64 = 0;
    // One extra byte is permitted so growth past the declared length is detected;
    // it is charged to the aggregate budget by the caller.
    let ceiling = before.size.saturating_add(1).min(remaining_budget);
    loop {
        if total >= ceiling {
            break;
        }
        let want = usize::try_from(ceiling - total)
            .unwrap_or(read_buffer.len())
            .min(read_buffer.len());
        match file.read(&mut read_buffer[..want]) {
            Ok(0) => break,
            Ok(n) => {
                let taken = u64::try_from(n).unwrap_or(0);
                // Only bytes within the declared length contribute to the digest;
                // a sentinel byte beyond it signals growth instead.
                if total < before.size {
                    let keep = usize::try_from((before.size - total).min(taken)).unwrap_or(0);
                    sha2::Digest::update(&mut hasher, &read_buffer[..keep]);
                }
                total = total.saturating_add(taken);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                return Streamed::Failed {
                    failure: Failure::IoFailure,
                    partial: total,
                };
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return Streamed::Failed {
                    failure: Failure::PermissionDenied,
                    partial: total,
                };
            }
            Err(_) => {
                return Streamed::Failed {
                    failure: Failure::IoFailure,
                    partial: total,
                };
            }
        }
    }

    let Ok(after) = describe(&OwnedFd::from(file)) else {
        return Streamed::Failed {
            failure: Failure::IoFailure,
            partial: total,
        };
    };
    let changed = after.size != before.size
        || after.link_count != before.link_count
        || after.mtime != before.mtime
        || after.ctime != before.ctime;
    if changed || total != before.size {
        return Streamed::Changed { partial: total };
    }
    Streamed::Complete {
        bytes: total,
        sha256: sha2::Digest::finalize(hasher).into(),
        link_count: after.link_count,
    }
}
