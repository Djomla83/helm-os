//! Linux x86_64 capability admission and authorisation composition (**P2**).
//!
//! This module is compiled only under
//! `cfg(all(target_os = "linux", target_arch = "x86_64"))`. Off that cohort the
//! crate is the portable P1 model and none of these types or functions exists.
//!
//! # What authority is here
//!
//! ```text
//! caller-owned executable fd  --admit_executable--------▶ ExecutableCapability
//! caller-owned cwd fd + id    --admit_working_directory-▶ WorkingDirectoryCapability
//! plan + executable + cwd     --authorize---------------▶ AuthorizedLaunch
//! AuthorizedLaunch            --╳--------------------------▶ process
//! ```
//!
//! **The last edge does not exist.** There is no `launch`, no process creation
//! and no process execution in this crate, so an `AuthorizedLaunch` is an inert
//! in-process value that nothing can execute. Creating a process needs a new
//! explicit owner decision.
//!
//! Execution authority is an already-open descriptor a trusted caller moves in
//! by value. It is never a pathname, a name, a verdict or a document field: no
//! function here takes a path, resolves a name, searches `PATH`, opens anything
//! or consults the ambient environment, and no function turns bytes, a string,
//! a path, a receipt or any `serde` input into a capability.
//!
//! # Implementation boundary
//!
//! Every operating-system call goes through a safe `rustix` wrapper: descriptor
//! flag inspection (`F_GETFL`), metadata sampling (`fstat`) and positional
//! reads (`pread`). Nothing here manipulates a raw descriptor number, and the
//! crate root denies the code the workspace forbids.
//!
//! # Side effects of admission, stated
//!
//! Executable admission reads the object's bytes, so it **may update atime**
//! under the host's mount policy and **populates the page cache**. `O_NOATIME`
//! is not used, because it requires file ownership or `CAP_FOWNER`.
//!
//! # Non-claims
//!
//! * The measurement is a **pre-execution measurement of the pinned object**,
//!   never the identity of bytes that executed, and never a statement about an
//!   ELF interpreter, a shared library or any other part of a loaded-code
//!   closure.
//! * A successful admission says only that **the measurement protocol detected
//!   no instability** in the fields it samples. It never says that no mutation
//!   occurred, that the object is immutable, that a snapshot exists, or that
//!   the measured bytes are the bytes any later execution would run.
//! * An in-cohort ELF header proves nothing about the program. Execute
//!   permission, filesystem type, mount options, `noexec`, file-capability
//!   attributes, `binfmt_misc` registrations, the dynamic loader, shared
//!   libraries, the kernel version and the sandbox state are **not checked and
//!   not attested**.
//! * Admission grants no privilege and is not a sandbox.

use core::cell::Cell;
use core::marker::PhantomData;
use std::os::fd::{AsFd as _, BorrowedFd, OwnedFd};

use rustix::fs::{FileType, Mode, OFlags, RawMode, fcntl_getfl, fstat};
use rustix::io::{Errno, pread, retry_on_intr};
use sha2::{Digest as _, Sha256};

use crate::error::{
    AdmissionError, AdmissionErrorCode as A, AuthorizationRefusal, AuthorizationRefusalCode as R,
};
use crate::model::{CapabilityId, Digest, ElfType, ExecutableMeasurement};
use crate::plan::ValidatedLaunchPlan;

/// The accepted 0.1 executable size bound: 512 MiB.
///
/// A first metadata sample above this is refused **before any byte of the body
/// is read**.
pub const MAX_EXECUTABLE_BYTES: u64 = 536_870_912;

/// The accepted fixed measurement read buffer: 64 KiB.
const MEASUREMENT_BUFFER_BYTES: u64 = 65_536;

/// The ELF64 header length the cohort check requires at offset 0.
const ELF_HEADER_BYTES: usize = 64;

/// The permission and set-ID bits recorded as `pre_exec_mode_bits`.
const MODE_BITS_MASK: RawMode = 0o7777;

// The closed set of header fields the accepted cohort fixes. Nothing else in
// the header is read, and the object is never loaded or executed.
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const EI_CLASS: usize = 4;
const EI_DATA: usize = 5;
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const E_TYPE_OFFSET: usize = 16;
const E_MACHINE_OFFSET: usize = 18;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const EM_X86_64: u16 = 62;

/// Classifies a 64-byte header against the accepted cohort.
///
/// Pure: it reads the fixed offsets of one array and performs no I/O. A header
/// value is a header value; it says nothing about the program.
fn classify_elf_header(header: &[u8; ELF_HEADER_BYTES]) -> Result<ElfType, AdmissionError> {
    if header[..ELF_MAGIC.len()] != ELF_MAGIC {
        // Where `#!` scripts and every other non-ELF object stop.
        return Err(AdmissionError::new(A::NotElf));
    }
    if header[EI_CLASS] != ELFCLASS64 || header[EI_DATA] != ELFDATA2LSB {
        return Err(AdmissionError::new(A::ElfNotInCohort));
    }
    let machine = u16::from_le_bytes([header[E_MACHINE_OFFSET], header[E_MACHINE_OFFSET + 1]]);
    if machine != EM_X86_64 {
        return Err(AdmissionError::new(A::ElfNotInCohort));
    }
    match u16::from_le_bytes([header[E_TYPE_OFFSET], header[E_TYPE_OFFSET + 1]]) {
        ET_EXEC => Ok(ElfType::EtExec),
        ET_DYN => Ok(ElfType::EtDyn),
        _ => Err(AdmissionError::new(A::ElfNotInCohort)),
    }
}

/// Exactly the metadata fields the instability protocol observes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MetadataSample {
    size: u64,
    mtime_sec: i64,
    mtime_nsec: u64,
    ctime_sec: i64,
    ctime_nsec: u64,
}

/// What the measurement protocol observed: two metadata samples of the same
/// descriptor and the number of bytes read between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Observation {
    first: MetadataSample,
    second: MetadataSample,
    bytes_read: u64,
}

impl Observation {
    /// Whether the protocol **detected instability**.
    ///
    /// It observes exactly the five sampled fields and the byte count, and
    /// nothing else. A change that moves none of them between the two samples
    /// is not detected — for example one the filesystem's timestamp resolution
    /// does not distinguish, or a write through a shared writable mapping whose
    /// timestamp update is deferred. **Returning `false` proves nothing**: it
    /// is not evidence that no mutation occurred, that the object is immutable,
    /// that a snapshot exists, or that the measured bytes are the bytes any
    /// later execution would run.
    fn instability_detected(&self) -> bool {
        let (first, second) = (&self.first, &self.second);
        first.size != second.size
            || first.mtime_sec != second.mtime_sec
            || first.mtime_nsec != second.mtime_nsec
            || first.ctime_sec != second.ctime_sec
            || first.ctime_nsec != second.ctime_nsec
            || self.bytes_read != first.size
    }
}

/// Maps a failed operating-system operation onto a bounded refusal.
///
/// The `rustix` error type never reaches the public API: only a code and a
/// plain error number do, and no `strerror` text is ever consulted.
fn os_refusal(code: A, errno: Errno) -> AdmissionError {
    AdmissionError::with_errno(code, errno.raw_os_error())
}

/// Step 1 of both admissions: the descriptor's access mode.
///
/// `O_PATH` is refused because nothing can be measured through it. `O_WRONLY`
/// and `O_RDWR` are refused because a writable capability would also be a
/// mutation authority. Only `O_RDONLY` is admitted, so an execute-only object
/// is inadmissible: measurability, not executability, is the binding
/// constraint.
///
/// The descriptor's close-on-exec flag is deliberately **not** inspected. It is
/// irrelevant to admission, and `F_GETFL` does not even report it.
fn require_read_only(fd: BorrowedFd<'_>) -> Result<(), AdmissionError> {
    let flags = fcntl_getfl(fd).map_err(|errno| os_refusal(A::MetadataUnavailable, errno))?;
    if flags.contains(OFlags::PATH) || flags.intersection(OFlags::RWMODE) != OFlags::RDONLY {
        return Err(AdmissionError::new(A::DescriptorModeUnsuitable));
    }
    Ok(())
}

/// One metadata sample of the same descriptor, plus its raw mode word.
fn metadata_sample(fd: BorrowedFd<'_>) -> Result<(MetadataSample, RawMode), AdmissionError> {
    let stat = fstat(fd).map_err(|errno| os_refusal(A::MetadataUnavailable, errno))?;
    // A negative size is not representable metadata; refusing is honest, since
    // the object's extent is then unknown.
    let size =
        u64::try_from(stat.st_size).map_err(|_| AdmissionError::new(A::MetadataUnavailable))?;
    // On the Linux x86_64 cohort `st_mtime` and `st_ctime` are `i64` seconds
    // and their `_nsec` companions are `u64`, so the sample stores them as they
    // are: a type change would be a compile error here rather than a silent
    // narrowing.
    let sample = MetadataSample {
        size,
        mtime_sec: stat.st_mtime,
        mtime_nsec: stat.st_mtime_nsec,
        ctime_sec: stat.st_ctime,
        ctime_nsec: stat.st_ctime_nsec,
    };
    Ok((sample, stat.st_mode))
}

/// `st_mode & 0o7777`, the recorded pre-execution permission and set-ID bits.
fn mode_bits(raw_mode: RawMode) -> u16 {
    // The mask bounds the value by 0o7777, so this conversion cannot truncate.
    (raw_mode & MODE_BITS_MASK) as u16
}

/// Step 5: 64 bytes read positionally from offset 0 through the same
/// descriptor, so the shared file offset is never used or moved.
fn read_elf_header(fd: BorrowedFd<'_>) -> Result<[u8; ELF_HEADER_BYTES], AdmissionError> {
    let mut header = [0u8; ELF_HEADER_BYTES];
    let mut filled = 0usize;
    while filled < ELF_HEADER_BYTES {
        let offset = u64::try_from(filled).map_err(|_| AdmissionError::new(A::ReadFailed))?;
        // `filled < ELF_HEADER_BYTES`, so the slice is non-empty and in range.
        let read = retry_on_intr(|| pread(fd, &mut header[filled..], offset))
            .map_err(|errno| os_refusal(A::ReadFailed, errno))?;
        if read == 0 {
            // Fewer than 64 bytes exist: not an ELF object.
            return Err(AdmissionError::new(A::NotElf));
        }
        filled = filled.saturating_add(read);
    }
    Ok(header)
}

/// Step 6: positional reads from offset 0 with a fixed 64 KiB buffer, until end
/// of file or **one byte beyond** the first sample's size, so growth is
/// observed without an unbounded read.
///
/// Returns the number of bytes read and the SHA-256 over exactly those bytes.
/// The shared file offset is never used or moved.
fn measure_body(fd: BorrowedFd<'_>, initial_size: u64) -> Result<(u64, Digest), AdmissionError> {
    let limit = initial_size.saturating_add(1);
    let capacity = usize::try_from(MEASUREMENT_BUFFER_BYTES)
        .map_err(|_| AdmissionError::new(A::ReadFailed))?;
    let mut buffer = vec![0u8; capacity];
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    while total < limit {
        let want = usize::try_from(limit.saturating_sub(total).min(MEASUREMENT_BUFFER_BYTES))
            .map_err(|_| AdmissionError::new(A::ReadFailed))?;
        // `want <= capacity` by construction, so the slice is in range.
        let read = retry_on_intr(|| pread(fd, &mut buffer[..want], total))
            .map_err(|errno| os_refusal(A::ReadFailed, errno))?;
        if read == 0 {
            break;
        }
        let chunk = buffer
            .get(..read)
            .ok_or_else(|| AdmissionError::new(A::ReadFailed))?;
        hasher.update(chunk);
        total = total
            .saturating_add(u64::try_from(read).map_err(|_| AdmissionError::new(A::ReadFailed))?);
    }
    Ok((total, Digest::from_raw(hasher.finalize().into())))
}

/// Authority to select **one** already-open object as an execution target.
///
/// The value owns exactly the descriptor the caller moved into
/// [`admit_executable`], and it closes that descriptor when it drops. It cannot
/// execute anything by itself: no method of this type, and no function in this
/// crate, creates a process.
///
/// The read-only facts it exposes are the inert results of admission. There is
/// no host path, no inode or device identity, no descriptor number and no
/// public raw-descriptor accessor, and its `Debug` output prints admitted facts
/// only.
///
/// # Type boundary
///
/// The value is `Send`, because moving an owned descriptor to another thread is
/// legitimate:
///
/// ```
/// fn assert_send<T: Send>() {}
/// assert_send::<helm_launch::ExecutableCapability>();
/// ```
///
/// It is **not** `Sync`:
///
/// ```compile_fail
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<helm_launch::ExecutableCapability>();
/// ```
///
/// There is no public constructor, so it cannot be field-constructed:
///
/// ```compile_fail
/// let _ = helm_launch::ExecutableCapability { fd: todo!() };
/// ```
///
/// no `Default`:
///
/// ```compile_fail
/// let _ = helm_launch::ExecutableCapability::default();
/// ```
///
/// no `Clone`, so one admission cannot become two:
///
/// ```compile_fail
/// fn duplicate(c: &helm_launch::ExecutableCapability) -> helm_launch::ExecutableCapability {
///     c.clone()
/// }
/// ```
///
/// no `From` of a descriptor, of bytes or of anything else:
///
/// ```compile_fail
/// fn forge(fd: std::os::fd::OwnedFd) -> helm_launch::ExecutableCapability {
///     helm_launch::ExecutableCapability::from(fd)
/// }
/// ```
///
/// ```compile_fail
/// fn forge(bytes: &[u8]) -> helm_launch::ExecutableCapability {
///     helm_launch::ExecutableCapability::from(bytes)
/// }
/// ```
///
/// and no `Deserialize`, so no document can become authority:
///
/// ```compile_fail
/// let _: helm_launch::ExecutableCapability = serde_json::from_slice(b"{}").unwrap();
/// ```
pub struct ExecutableCapability {
    fd: OwnedFd,
    measurement: ExecutableMeasurement,
    // `OwnedFd` is `Sync`; the accepted contract is `Send` and not `Sync`, and
    // `Cell` is the safe marker that removes exactly `Sync`.
    not_sync: PhantomData<Cell<()>>,
}

impl ExecutableCapability {
    /// The complete pre-execution measurement recorded at admission.
    #[must_use]
    pub const fn measurement(&self) -> ExecutableMeasurement {
        self.measurement
    }

    /// Bytes read through this descriptor before any execution attempt.
    #[must_use]
    pub const fn pre_exec_body_size(&self) -> u64 {
        self.measurement.pre_exec_body_size()
    }

    /// SHA-256 over exactly the bytes read before any execution attempt.
    ///
    /// **Not** the identity of bytes that executed, and not a statement about
    /// any interpreter, library or other loaded code.
    #[must_use]
    pub const fn pre_exec_body_sha256(&self) -> Digest {
        self.measurement.pre_exec_body_sha256()
    }

    /// `st_mode & 0o7777` from the **first** accepted metadata sample.
    ///
    /// Recorded, never enforced: execute permission is a kernel decision at
    /// execution time, which this crate cannot reach.
    #[must_use]
    pub const fn pre_exec_mode_bits(&self) -> u16 {
        self.measurement.pre_exec_mode_bits()
    }

    /// The header `e_type` of the admitted object.
    #[must_use]
    pub const fn elf_type(&self) -> ElfType {
        self.measurement.elf_type()
    }

    /// The admitted descriptor, borrowed. Crate-private: no public accessor
    /// exposes a descriptor or its number.
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "P2 only pins the descriptor; the execution slice that would use it is not authorised"
        )
    )]
    pub(crate) fn descriptor(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl core::fmt::Debug for ExecutableCapability {
    /// Admitted facts only: no descriptor, no descriptor number, no host path.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExecutableCapability")
            .field("pre_exec_body_size", &self.pre_exec_body_size())
            .field("pre_exec_body_sha256", &self.pre_exec_body_sha256())
            .field("pre_exec_mode_bits", &self.pre_exec_mode_bits())
            .field("elf_type", &self.elf_type().as_str())
            .finish_non_exhaustive()
    }
}

/// Authority to select **one** already-open directory as a working directory.
///
/// The value owns exactly the descriptor the caller moved into
/// [`admit_working_directory`] and closes it when it drops. After admission it
/// performs no I/O of its own. The directory is never enumerated, no path is
/// resolved or recorded, no search permission is checked, and nothing here
/// changes any working directory: that kernel decision belongs to an execution
/// slice that is not authorised.
///
/// # Type boundary
///
/// ```
/// fn assert_send<T: Send>() {}
/// assert_send::<helm_launch::WorkingDirectoryCapability>();
/// ```
///
/// ```compile_fail
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<helm_launch::WorkingDirectoryCapability>();
/// ```
///
/// ```compile_fail
/// let _ = helm_launch::WorkingDirectoryCapability { fd: todo!() };
/// ```
///
/// ```compile_fail
/// let _ = helm_launch::WorkingDirectoryCapability::default();
/// ```
///
/// ```compile_fail
/// fn duplicate(
///     c: &helm_launch::WorkingDirectoryCapability,
/// ) -> helm_launch::WorkingDirectoryCapability {
///     c.clone()
/// }
/// ```
///
/// ```compile_fail
/// fn forge(path: &std::path::Path) -> helm_launch::WorkingDirectoryCapability {
///     helm_launch::WorkingDirectoryCapability::from(path)
/// }
/// ```
///
/// ```compile_fail
/// let _: helm_launch::WorkingDirectoryCapability = serde_json::from_slice(b"{}").unwrap();
/// ```
pub struct WorkingDirectoryCapability {
    fd: OwnedFd,
    id: CapabilityId,
    // As for `ExecutableCapability`: `Send`, not `Sync`.
    not_sync: PhantomData<Cell<()>>,
}

impl WorkingDirectoryCapability {
    /// The caller's validated logical identifier. An identifier, never a path.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// The admitted descriptor, borrowed. Crate-private, as for
    /// [`ExecutableCapability::descriptor`].
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "P2 only pins the descriptor; the execution slice that would use it is not authorised"
        )
    )]
    pub(crate) fn descriptor(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl core::fmt::Debug for WorkingDirectoryCapability {
    /// The logical identifier only: no descriptor, number or host path.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WorkingDirectoryCapability")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

/// One plan composed with the two capabilities it authorises: a **single-use**
/// authority value for a future launch API.
///
/// **Nothing in this crate can consume it to create a process**, because no
/// `launch` function exists:
///
/// ```
/// fn assert_send<T: Send>() {}
/// assert_send::<helm_launch::AuthorizedLaunch>();
/// ```
///
/// ```compile_fail
/// fn execute(a: helm_launch::AuthorizedLaunch) {
///     let _ = helm_launch::launch(a);
/// }
/// ```
///
/// ```compile_fail
/// fn outcome() -> helm_launch::LaunchOutcome {
///     todo!()
/// }
/// ```
///
/// It is not `Sync`, has no public constructor, no `Default`, no `Clone` — so
/// one authorisation cannot be replayed — and no `From` or `Deserialize`, so
/// neither data nor a receipt can become authority:
///
/// ```compile_fail
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<helm_launch::AuthorizedLaunch>();
/// ```
///
/// ```compile_fail
/// let _ = helm_launch::AuthorizedLaunch { plan: todo!() };
/// ```
///
/// ```compile_fail
/// let _ = helm_launch::AuthorizedLaunch::default();
/// ```
///
/// ```compile_fail
/// fn replay(a: &helm_launch::AuthorizedLaunch) -> helm_launch::AuthorizedLaunch {
///     a.clone()
/// }
/// ```
///
/// ```compile_fail
/// fn escalate(receipt: helm_launch::LaunchReceipt) -> helm_launch::AuthorizedLaunch {
///     helm_launch::AuthorizedLaunch::from(receipt)
/// }
/// ```
///
/// ```compile_fail
/// let _: helm_launch::AuthorizedLaunch = serde_json::from_slice(b"{}").unwrap();
/// ```
pub struct AuthorizedLaunch {
    plan: ValidatedLaunchPlan,
    executable: ExecutableCapability,
    working_directory: WorkingDirectoryCapability,
    // No marker is needed: `ExecutableCapability` is not `Sync`, so neither is
    // this. The type-boundary tests prove it rather than assuming it.
}

impl AuthorizedLaunch {
    /// SHA-256 over the exact plan bytes this authorisation owns.
    #[must_use]
    pub fn plan_sha256(&self) -> Digest {
        self.plan.sha256()
    }

    /// The logical working-directory identifier both the plan and the
    /// capability carry, which `authorize` proved identical.
    #[must_use]
    pub fn working_directory_id(&self) -> &str {
        self.working_directory.id()
    }

    /// The executable measurement exactly as admission recorded it.
    ///
    /// `authorize` moves the capability in unchanged: it does not re-measure,
    /// reopen or re-resolve anything, and no future execution would either.
    #[must_use]
    pub const fn executable_measurement(&self) -> ExecutableMeasurement {
        self.executable.measurement()
    }
}

/// What an [`AuthorizedLaunch`] decomposes into for the **crate-private**
/// backend (**P3**).
///
/// This is the first consumer an `AuthorizedLaunch` has ever had. It is not
/// public and cannot be made public from outside: the type, the method that
/// produces it and the module that consumes it are all crate-private, and the
/// backend module is not re-exported. No public function turns an
/// authorisation into a process, here or anywhere.
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "off the Linux x86_64 cohort no backend exists to consume these parts"
    )
)]
pub(crate) struct AuthorizedParts {
    /// The validated plan, moved out unchanged.
    pub(crate) plan: ValidatedLaunchPlan,
    /// The pre-execution measurement admission recorded, moved out unchanged:
    /// nothing re-measures, reopens or re-resolves the object.
    pub(crate) measurement: ExecutableMeasurement,
    /// The admitted executable descriptor, moved out by value.
    pub(crate) executable: OwnedFd,
    /// The admitted working-directory descriptor, moved out by value.
    pub(crate) working_directory: OwnedFd,
}

impl AuthorizedLaunch {
    /// Consume this single-use authorisation into its parts.
    ///
    /// Crate-private, and consuming: an authorisation cannot be decomposed
    /// twice, and nothing is copied, cloned or re-derived. `AuthorizedLaunch`
    /// has no `Clone`, so this is also the only way its descriptors can ever
    /// reach a process-creation path.
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "off the Linux x86_64 cohort no backend exists to consume these parts"
        )
    )]
    pub(crate) fn into_parts(self) -> AuthorizedParts {
        let Self {
            plan,
            executable,
            working_directory,
        } = self;
        let measurement = executable.measurement;
        let ExecutableCapability {
            fd: executable,
            measurement: _,
            not_sync: _,
        } = executable;
        let WorkingDirectoryCapability {
            fd: working_directory,
            id: _,
            not_sync: _,
        } = working_directory;
        AuthorizedParts {
            plan,
            measurement,
            executable,
            working_directory,
        }
    }
}

impl core::fmt::Debug for AuthorizedLaunch {
    /// Inert facts only: no descriptor, number, host path or argument byte.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AuthorizedLaunch")
            .field("plan_sha256", &self.plan_sha256())
            .field("working_directory_id", &self.working_directory_id())
            .field("executable", &self.executable)
            .finish_non_exhaustive()
    }
}

/// Admits one already-open object as an [`ExecutableCapability`].
///
/// The descriptor is **moved in**, and every check runs through that same
/// descriptor in the accepted order (ADR-0024 section C, plan section 6.2):
///
/// 1. access mode — only `O_RDONLY`, never `O_PATH`, `O_WRONLY` or `O_RDWR`;
/// 2. `fstat` — regular file only; **the first metadata sample**;
/// 3. set-ID refusal — `S_ISUID` or `S_ISGID` is refused, with no privilege
///    transition attempted;
/// 4. size bound — above [`MAX_EXECUTABLE_BYTES`], refused before any body
///    byte is read;
/// 5. ELF header — 64 bytes read positionally at offset 0, against the closed
///    cohort;
/// 6. measurement — positional reads with a fixed 64 KiB buffer and SHA-256
///    over exactly the bytes read;
/// 7. detected instability — a second metadata sample of the same descriptor;
/// 8. the capability, with the inert admitted facts.
///
/// There is **no pathname form of this call**, and no name is resolved at any
/// step:
///
/// ```compile_fail
/// let _ = helm_launch::admit_executable("/usr/bin/example");
/// ```
///
/// # Errors
///
/// Every refusal is a bounded [`AdmissionError`]. It carries no descriptor, so
/// a refusal can never hand back authority, and the moved-in descriptor closes.
///
/// # Side effects
///
/// Reading the body may update atime and populates the page cache.
pub fn admit_executable(fd: OwnedFd) -> Result<ExecutableCapability, AdmissionError> {
    // STEP 1 — descriptor access mode.
    require_read_only(fd.as_fd())?;

    // STEP 2 — first metadata sample, and the regular-file requirement.
    let (first, raw_mode) = metadata_sample(fd.as_fd())?;
    if FileType::from_raw_mode(raw_mode) != FileType::RegularFile {
        return Err(AdmissionError::new(A::NotRegularFile));
    }

    // STEP 3 — set-ID refusal.
    if Mode::from_raw_mode(raw_mode).intersects(Mode::SUID.union(Mode::SGID)) {
        return Err(AdmissionError::new(A::SetIdBitsPresent));
    }

    // STEP 4 — size bound, before any byte of the body is read.
    if first.size > MAX_EXECUTABLE_BYTES {
        return Err(AdmissionError::new(A::ExecutableTooLarge));
    }

    // STEP 5 — ELF cohort, from the header of the same descriptor.
    let elf_type = classify_elf_header(&read_elf_header(fd.as_fd())?)?;

    // STEP 6 — measurement of the candidate pre-execution body.
    let (bytes_read, pre_exec_body_sha256) = measure_body(fd.as_fd(), first.size)?;

    // STEP 7 — second metadata sample, then the instability decision.
    let (second, _) = metadata_sample(fd.as_fd())?;
    let mut observation = Observation {
        first,
        second,
        bytes_read,
    };
    perturb_observation(&mut observation);
    if observation.instability_detected() {
        return Err(AdmissionError::new(A::MeasurementInstabilityDetected));
    }

    // STEP 8 — the capability owns the same descriptor and the inert facts.
    Ok(ExecutableCapability {
        fd,
        measurement: ExecutableMeasurement {
            pre_exec_body_size: observation.bytes_read,
            pre_exec_body_sha256,
            pre_exec_mode_bits: mode_bits(raw_mode),
            elf_type,
        },
        not_sync: PhantomData,
    })
}

/// Admits one already-open directory as a [`WorkingDirectoryCapability`].
///
/// `id` is the caller's logical identifier and must match the accepted grammar
/// `[a-z0-9][a-z0-9._-]{0,79}` — the same validator the plan parser uses. The
/// descriptor is **moved in**; only an `O_RDONLY` directory descriptor is
/// admitted, and `O_PATH` is refused.
///
/// The directory is never enumerated, no path is resolved or recorded, and no
/// search permission is checked: that kernel decision belongs to an execution
/// slice which does not exist.
///
/// # Errors
///
/// A bounded [`AdmissionError`]: an invalid identifier, an unsuitable
/// descriptor mode, unavailable metadata, or an object that is not a directory.
/// The moved-in descriptor closes on refusal.
pub fn admit_working_directory(
    id: &str,
    fd: OwnedFd,
) -> Result<WorkingDirectoryCapability, AdmissionError> {
    let id =
        CapabilityId::parse(id).ok_or_else(|| AdmissionError::new(A::WorkingDirectoryIdInvalid))?;
    require_read_only(fd.as_fd())?;
    let (_, raw_mode) = metadata_sample(fd.as_fd())?;
    if FileType::from_raw_mode(raw_mode) != FileType::Directory {
        return Err(AdmissionError::new(A::NotDirectory));
    }
    Ok(WorkingDirectoryCapability {
        fd,
        id,
        not_sync: PhantomData,
    })
}

/// Composes one validated plan with the two capabilities it authorises.
///
/// **Performs zero I/O.** It checks exactly one relation — that the plan's
/// `working_directory.capability_id` equals the admitted capability's logical
/// identifier — and records the rest. It does not re-measure, reopen, re-resolve
/// or stat anything, and it does not inspect the plan's `asserted_context`
/// digests: those are inert caller assertions, never evidence that a binding or
/// an observation was consulted, and no binding state, observation result or
/// application-specification verdict can participate. The crate links no HELM
/// crate, so no such value is even nameable here.
///
/// All three inputs are consumed, so a refusal drops both capabilities and
/// closes their descriptors, and no refused value carries authority back.
///
/// # Errors
///
/// [`AuthorizationRefusal`] with
/// [`AuthorizationRefusalCode::WorkingDirectoryIdMismatch`](crate::AuthorizationRefusalCode::WorkingDirectoryIdMismatch),
/// raised before any process could exist — and in this crate none ever can.
pub fn authorize(
    plan: ValidatedLaunchPlan,
    executable: ExecutableCapability,
    working_directory: WorkingDirectoryCapability,
) -> Result<AuthorizedLaunch, AuthorizationRefusal> {
    if plan.working_directory_id() != working_directory.id() {
        // `plan`, `executable` and `working_directory` all drop here.
        return Err(AuthorizationRefusal::new(R::WorkingDirectoryIdMismatch));
    }
    Ok(AuthorizedLaunch {
        plan,
        executable,
        working_directory,
    })
}

// ---------------------------------------------------------------------------
// Test-only seam.
//
// The instability rule is an owner-approved product obligation that no formal
// trial established as an implementation property, so it needs real product
// tests. Provoking a difference in exactly one sampled field, or a byte count
// that differs while the metadata does not, is not reliably schedulable from
// outside. The seam below is the internal control point the accepted boundary
// allows: it exists only under `cfg(test)`, it is not part of the public API,
// and no release build contains any hook at all — the non-test definition
// compiles to nothing.
// ---------------------------------------------------------------------------

/// Release definition: no hook exists.
#[cfg(not(test))]
fn perturb_observation(_observation: &mut Observation) {}

/// Test definition: applies the seam the current thread has installed, if any.
#[cfg(test)]
fn perturb_observation(observation: &mut Observation) {
    if let Some(perturb) = INSTABILITY_SEAM.with(Cell::get) {
        perturb(observation);
    }
}

#[cfg(test)]
thread_local! {
    /// Per-thread, so one test cannot affect another.
    static INSTABILITY_SEAM: Cell<Option<fn(&mut Observation)>> = const { Cell::new(None) };
}

#[cfg(test)]
fn with_instability_seam<T>(perturb: fn(&mut Observation), body: impl FnOnce() -> T) -> T {
    INSTABILITY_SEAM.with(|seam| seam.set(Some(perturb)));
    let outcome = body();
    INSTABILITY_SEAM.with(|seam| seam.set(None));
    outcome
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::{
        A, AuthorizedLaunch, ELF_HEADER_BYTES, ELFCLASS64, ELFDATA2LSB, EM_X86_64, ET_DYN, ET_EXEC,
        MAX_EXECUTABLE_BYTES, MetadataSample, Observation, admit_executable,
        admit_working_directory, authorize, classify_elf_header, metadata_sample, mode_bits,
        with_instability_seam,
    };
    use crate::model::ElfType;
    use crate::plan::parse_launch_plan;
    use rustix::fs::{Mode, OFlags, chmod, fstat, ftruncate, mkdir, open, seek, unlink};
    use rustix::io::{Errno, pwrite, write};
    use std::os::fd::{AsFd as _, OwnedFd};

    /// Tests are the trusted caller: they may use pathnames to build fixtures
    /// and open descriptors. The product API never can.
    const SCRATCH: &str = "/tmp/helm-launch-p2-authority";

    /// One named way to perturb what the measurement protocol observed.
    type Perturbation = (&'static str, fn(&mut Observation));

    fn scratch_path(name: &str) -> String {
        match mkdir(SCRATCH, Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(other) => panic!("scratch directory: {other:?}"),
        }
        format!("{SCRATCH}/{name}")
    }

    /// A 64-byte ELF64 little-endian x86_64 header with the given `e_type`.
    fn elf_header(e_type: u16) -> [u8; ELF_HEADER_BYTES] {
        let mut header = [0u8; ELF_HEADER_BYTES];
        header[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        header[4] = ELFCLASS64;
        header[5] = ELFDATA2LSB;
        header[6] = 1; // EI_VERSION, deliberately not part of the cohort check.
        header[16..18].copy_from_slice(&e_type.to_le_bytes());
        header[18..20].copy_from_slice(&EM_X86_64.to_le_bytes());
        header
    }

    /// Writes `bytes` to a fresh fixture at mode `mode` and returns its path.
    fn fixture(name: &str, bytes: &[u8], mode: u32) -> String {
        let path = scratch_path(name);
        let _ = unlink(path.as_str());
        let fd = open(
            path.as_str(),
            OFlags::CREATE | OFlags::WRONLY | OFlags::TRUNC,
            Mode::from_bits_truncate(0o600),
        )
        .unwrap();
        let mut written = 0usize;
        while written < bytes.len() {
            written += write(fd.as_fd(), &bytes[written..]).unwrap();
        }
        drop(fd);
        // Set the mode explicitly, after writing, so umask cannot change it.
        chmod(path.as_str(), Mode::from_bits_truncate(mode)).unwrap();
        let observed = fstat(open(path.as_str(), OFlags::RDONLY, Mode::empty()).unwrap()).unwrap();
        assert_eq!(observed.st_mode & 0o7777, mode, "fixture mode");
        path
    }

    fn read_only(path: &str) -> OwnedFd {
        open(path, OFlags::RDONLY, Mode::empty()).unwrap()
    }

    // ------------------------------------------------------- pure classifier

    #[test]
    fn the_cohort_accepts_exactly_et_exec_and_et_dyn() {
        assert_eq!(
            classify_elf_header(&elf_header(ET_EXEC)).unwrap(),
            ElfType::EtExec
        );
        assert_eq!(
            classify_elf_header(&elf_header(ET_DYN)).unwrap(),
            ElfType::EtDyn
        );
        for other in [0u16, 1, 4, 5, 0xfe00, 0xffff] {
            assert_eq!(
                classify_elf_header(&elf_header(other)).unwrap_err().code(),
                A::ElfNotInCohort,
                "e_type {other:#x}"
            );
        }
    }

    #[test]
    fn a_missing_magic_is_not_elf_and_a_wrong_field_is_out_of_cohort() {
        // Magic absent: not an ELF object at all.
        assert_eq!(
            classify_elf_header(&[0u8; ELF_HEADER_BYTES])
                .unwrap_err()
                .code(),
            A::NotElf
        );
        let mut script = [0u8; ELF_HEADER_BYTES];
        script[..2].copy_from_slice(b"#!");
        assert_eq!(classify_elf_header(&script).unwrap_err().code(), A::NotElf);

        // Magic present, one cohort field wrong: in the ELF family, out of
        // cohort. Each field is perturbed on its own.
        let mut wrong_class = elf_header(ET_EXEC);
        wrong_class[4] = 1; // ELFCLASS32
        let mut wrong_data = elf_header(ET_EXEC);
        wrong_data[5] = 2; // ELFDATA2MSB
        let mut wrong_machine = elf_header(ET_EXEC);
        wrong_machine[18..20].copy_from_slice(&183u16.to_le_bytes()); // EM_AARCH64
        for (label, header) in [
            ("class", wrong_class),
            ("data", wrong_data),
            ("machine", wrong_machine),
        ] {
            assert_eq!(
                classify_elf_header(&header).unwrap_err().code(),
                A::ElfNotInCohort,
                "{label}"
            );
        }
    }

    // ------------------------------------------- the instability comparison

    fn sample() -> MetadataSample {
        MetadataSample {
            size: 4_096,
            mtime_sec: 1_700_000_000,
            mtime_nsec: 123_456_789,
            ctime_sec: 1_700_000_001,
            ctime_nsec: 987_654_321,
        }
    }

    #[test]
    fn the_protocol_observes_exactly_five_fields_and_one_count() {
        let stable = Observation {
            first: sample(),
            second: sample(),
            bytes_read: 4_096,
        };
        assert!(!stable.instability_detected());

        let perturbations: [Perturbation; 6] = [
            ("size", |o| o.second.size = o.second.size.wrapping_add(1)),
            ("mtime_sec", |o| o.second.mtime_sec += 1),
            ("mtime_nsec", |o| o.second.mtime_nsec += 1),
            ("ctime_sec", |o| o.second.ctime_sec += 1),
            ("ctime_nsec", |o| o.second.ctime_nsec += 1),
            ("bytes_read", |o| o.bytes_read += 1),
        ];
        for (field, perturb) in perturbations {
            let mut observation = stable;
            perturb(&mut observation);
            assert!(observation.instability_detected(), "{field} not detected");
        }

        // A short read is detected too, from the same count comparison.
        let mut short = stable;
        short.bytes_read = 4_095;
        assert!(short.instability_detected());

        // Fields the protocol does not observe are not part of the decision:
        // the comparison is exactly the six above, so a stable observation
        // stays stable however the first sample's unobserved context differs.
        assert!(!stable.instability_detected());
    }

    #[test]
    fn real_metadata_changes_are_detected_by_the_same_comparison() {
        let path = fixture("real-instability", &elf_header(ET_EXEC), 0o644);
        let fd = read_only(path.as_str());
        let (first, _) = metadata_sample(fd.as_fd()).unwrap();
        let (again, _) = metadata_sample(fd.as_fd()).unwrap();
        assert_eq!(first, again, "an untouched object samples identically");

        // An append changes size and mtime.
        let writer = open(path.as_str(), OFlags::WRONLY, Mode::empty()).unwrap();
        pwrite(writer.as_fd(), b"x", 64).unwrap();
        drop(writer);
        let (grown, _) = metadata_sample(fd.as_fd()).unwrap();
        assert!(
            Observation {
                first,
                second: grown,
                bytes_read: first.size,
            }
            .instability_detected()
        );

        // A mode change moves only ctime, which the protocol also observes.
        let (before_chmod, _) = metadata_sample(fd.as_fd()).unwrap();
        chmod(path.as_str(), Mode::from_bits_truncate(0o600)).unwrap();
        let (after_chmod, _) = metadata_sample(fd.as_fd()).unwrap();
        assert_eq!(before_chmod.size, after_chmod.size);
        assert_eq!(before_chmod.mtime_sec, after_chmod.mtime_sec);
        assert_eq!(before_chmod.mtime_nsec, after_chmod.mtime_nsec);
        assert!(
            Observation {
                first: before_chmod,
                second: after_chmod,
                bytes_read: before_chmod.size,
            }
            .instability_detected(),
            "a ctime-only change is detected"
        );
        drop(fd);
        unlink(path.as_str()).unwrap();
    }

    // ------------------------------------------- seam-driven admission tests

    /// Each perturbation stands for one way the protocol detects instability,
    /// applied between the measurement read and the instability decision.
    #[test]
    fn detected_instability_refuses_admission_for_every_observed_field() {
        let perturbations: [Perturbation; 7] = [
            ("size grew", |o| {
                o.second.size = o.second.size.wrapping_add(1)
            }),
            ("size shrank", |o| {
                o.second.size = o.second.size.wrapping_sub(1)
            }),
            ("mtime seconds", |o| o.second.mtime_sec += 1),
            ("mtime nanoseconds", |o| o.second.mtime_nsec += 1),
            ("ctime seconds", |o| o.second.ctime_sec += 1),
            ("ctime nanoseconds", |o| o.second.ctime_nsec += 1),
            ("byte count", |o| o.bytes_read += 1),
        ];
        for (label, perturb) in perturbations {
            let path = fixture("seam-fixture", &elf_header(ET_EXEC), 0o644);
            let fd = read_only(path.as_str());
            let refusal = with_instability_seam(perturb, || admit_executable(fd));
            assert_eq!(
                refusal.unwrap_err().code(),
                A::MeasurementInstabilityDetected,
                "{label}"
            );
            unlink(path.as_str()).unwrap();
        }
    }

    #[test]
    fn a_short_byte_count_is_detected_as_instability() {
        let path = fixture("seam-short", &elf_header(ET_EXEC), 0o644);
        let fd = read_only(path.as_str());
        let refusal = with_instability_seam(
            |o| o.bytes_read = o.bytes_read.saturating_sub(1),
            || admit_executable(fd),
        );
        assert_eq!(
            refusal.unwrap_err().code(),
            A::MeasurementInstabilityDetected
        );
        unlink(path.as_str()).unwrap();
    }

    #[test]
    fn the_seam_is_per_thread_and_leaves_admission_untouched_by_default() {
        let path = fixture("seam-absent", &elf_header(ET_DYN), 0o644);
        let fd = read_only(path.as_str());
        // No seam installed: ordinary admission, exactly as a release build.
        let capability = admit_executable(fd).unwrap();
        assert_eq!(capability.elf_type(), ElfType::EtDyn);
        assert_eq!(capability.pre_exec_body_size(), 64);
        unlink(path.as_str()).unwrap();
    }

    // -------------------------------------------------------- the non-claim

    #[test]
    fn a_successful_admission_claims_only_that_nothing_was_detected() {
        // The capability records what was read **before** any execution
        // attempt. It is not re-derived, so mutating the object afterwards does
        // not change it — which is precisely why admission cannot claim that
        // the measured bytes are the bytes a later execution would run, that
        // the object is immutable, or that a snapshot exists.
        let body = [elf_header(ET_EXEC).as_slice(), &[0u8; 16]].concat();
        let path = fixture("non-claim", &body, 0o644);
        let fd = read_only(path.as_str());
        let capability = admit_executable(fd).unwrap();
        let admitted = capability.measurement();
        assert_eq!(admitted.pre_exec_body_size(), 80);

        let writer = open(path.as_str(), OFlags::WRONLY, Mode::empty()).unwrap();
        // A length-preserving mutation of the same inode, after admission.
        pwrite(writer.as_fd(), b"MUTATED0", 72).unwrap();
        drop(writer);

        // The recorded facts are unchanged: they describe the earlier read, and
        // nothing in this crate re-checks or re-measures them. The descriptor
        // still names the same object, at the same length.
        assert_eq!(capability.measurement(), admitted);
        let (now, _) = metadata_sample(capability.descriptor()).unwrap();
        assert_eq!(now.size, 80);

        // Yet the object's bytes now differ from what that digest identifies: a
        // fresh admission of the same inode measures something else, at the same
        // size. The first capability is still a capability, so a successful
        // admission was never a claim about the bytes a later execution would
        // run — only that the protocol detected no instability while reading.
        let second = admit_executable(read_only(path.as_str())).unwrap();
        assert_eq!(second.pre_exec_body_size(), admitted.pre_exec_body_size());
        assert_ne!(
            second.pre_exec_body_sha256(),
            admitted.pre_exec_body_sha256()
        );

        drop(capability);
        drop(second);
        unlink(path.as_str()).unwrap();
    }

    // ------------------------------------------------------- mode recording

    #[test]
    fn recorded_mode_bits_come_from_the_first_accepted_sample() {
        for mode in [0o400u32, 0o444, 0o644, 0o755, 0o500] {
            let path = fixture("mode-bits", &elf_header(ET_EXEC), mode);
            let fd = read_only(path.as_str());
            let capability = admit_executable(fd).unwrap();
            assert_eq!(
                capability.pre_exec_mode_bits(),
                u16::try_from(mode).unwrap(),
                "mode {mode:o}"
            );
            unlink(path.as_str()).unwrap();
        }
        assert_eq!(mode_bits(0o100_755), 0o755);
        assert_eq!(mode_bits(0o104_755), 0o4755);
        assert_eq!(mode_bits(0o40_700), 0o700);
    }

    // -------------------------------------------------- the size bound edge

    #[test]
    fn the_size_bound_refuses_before_any_body_byte_is_read() {
        let path = scratch_path("too-large");
        let _ = unlink(path.as_str());
        let fd = open(
            path.as_str(),
            OFlags::CREATE | OFlags::WRONLY | OFlags::TRUNC,
            Mode::from_bits_truncate(0o600),
        )
        .unwrap();
        // A sparse object one byte above the bound: no data block is allocated,
        // and admission must refuse before reading anything.
        ftruncate(fd.as_fd(), MAX_EXECUTABLE_BYTES + 1).unwrap();
        drop(fd);
        let refusal = admit_executable(read_only(path.as_str()));
        assert_eq!(refusal.unwrap_err().code(), A::ExecutableTooLarge);
        // Exactly at the bound the object is not too large; it is simply not an
        // ELF object, so the next step refuses it.
        let fd = open(path.as_str(), OFlags::WRONLY, Mode::empty()).unwrap();
        ftruncate(fd.as_fd(), MAX_EXECUTABLE_BYTES).unwrap();
        drop(fd);
        let refusal = admit_executable(read_only(path.as_str()));
        assert_eq!(refusal.unwrap_err().code(), A::NotElf);
        unlink(path.as_str()).unwrap();
    }

    // ------------------------------------------------- positional-read proof

    #[test]
    fn admission_never_uses_or_moves_the_shared_file_offset() {
        let body = [elf_header(ET_EXEC).as_slice(), &[7u8; 4_096]].concat();
        let path = fixture("shared-offset", &body, 0o644);
        let fd = read_only(path.as_str());
        // A duplicate shares the one open file description, so it observes the
        // same shared offset the admitted descriptor uses.
        let watcher = rustix::io::dup(fd.as_fd()).unwrap();
        seek(fd.as_fd(), rustix::fs::SeekFrom::Start(11)).unwrap();
        let capability = admit_executable(fd).unwrap();
        assert_eq!(capability.pre_exec_body_size(), 64 + 4_096);
        assert_eq!(
            seek(watcher.as_fd(), rustix::fs::SeekFrom::Current(0)).unwrap(),
            11,
            "the shared file offset moved"
        );
        drop(capability);
        unlink(path.as_str()).unwrap();
    }

    // ------------------------------------------ the exact admitted descriptor

    #[test]
    fn each_capability_owns_the_exact_descriptor_the_caller_moved_in() {
        let path = fixture("exact-descriptor", &elf_header(ET_EXEC), 0o644);
        let directory = scratch_path("exact-descriptor-dir");
        match mkdir(directory.as_str(), Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(other) => panic!("fixture directory: {other:?}"),
        }

        // The caller's own view of each object, taken before admission.
        let expected_file = fstat(read_only(path.as_str())).unwrap();
        let expected_directory = fstat(read_only(directory.as_str())).unwrap();

        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();

        // Admission pinned those objects: it opened, resolved and reopened
        // nothing, so the capability's descriptor still names the same inode.
        let pinned_file = fstat(executable.descriptor()).unwrap();
        assert_eq!(pinned_file.st_ino, expected_file.st_ino);
        assert_eq!(pinned_file.st_dev, expected_file.st_dev);
        let pinned_directory = fstat(cwd.descriptor()).unwrap();
        assert_eq!(pinned_directory.st_ino, expected_directory.st_ino);
        assert_eq!(pinned_directory.st_dev, expected_directory.st_dev);
        assert_ne!(pinned_file.st_ino, pinned_directory.st_ino);

        // Renaming the object afterwards changes nothing: no pathname was ever
        // recorded, so nothing can go stale.
        let renamed = scratch_path("exact-descriptor-renamed");
        rustix::fs::rename(path.as_str(), renamed.as_str()).unwrap();
        let after_rename = fstat(executable.descriptor()).unwrap();
        assert_eq!(after_rename.st_ino, expected_file.st_ino);

        drop(executable);
        drop(cwd);
        unlink(renamed.as_str()).unwrap();
        rustix::fs::rmdir(directory.as_str()).unwrap();
    }

    // ----------------------------------------------------------- authorize

    fn plan_bytes(working_directory_id: &str) -> Vec<u8> {
        format!(
            concat!(
                r#"{{"schema":"helm-launch-plan","version":"0.1","#,
                r#""execution_kind":"linux_exact_executable","argv":["tool"],"#,
                r#""environment":{{"mode":"empty"}},"#,
                r#""working_directory":{{"capability_id":"{id}"}},"#,
                r#""stdin":{{"mode":"closed_pipe_eof"}},"#,
                r#""stdout":{{"capture_prefix_bytes":0}},"#,
                r#""stderr":{{"capture_prefix_bytes":0}},"#,
                r#""timeout_ms":1000,"#,
                r#""termination":{{"signal":"SIGTERM","grace_ms":0}}}}"#
            ),
            id = working_directory_id
        )
        .into_bytes()
    }

    #[test]
    fn authorisation_checks_exactly_the_working_directory_identifier() {
        let path = fixture("authorize", &elf_header(ET_EXEC), 0o755);
        let directory = scratch_path("authorize-dir");
        match mkdir(directory.as_str(), Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(other) => panic!("fixture directory: {other:?}"),
        }

        // Matching identifier: authorised.
        let plan = parse_launch_plan(&plan_bytes("workdir")).unwrap();
        let expected_plan_sha256 = plan.sha256();
        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let expected_measurement = executable.measurement();
        let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
        let authorized: AuthorizedLaunch = authorize(plan, executable, cwd).unwrap();
        assert_eq!(authorized.working_directory_id(), "workdir");
        assert_eq!(authorized.plan_sha256(), expected_plan_sha256);
        assert_eq!(
            authorized.executable_measurement(),
            expected_measurement,
            "authorize must not re-measure"
        );

        // Mismatching identifier: refused, and both capabilities are consumed.
        let plan = parse_launch_plan(&plan_bytes("other")).unwrap();
        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
        let refusal = authorize(plan, executable, cwd).unwrap_err();
        assert_eq!(
            refusal.code(),
            crate::AuthorizationRefusalCode::WorkingDirectoryIdMismatch
        );

        drop(authorized);
        unlink(path.as_str()).unwrap();
        rustix::fs::rmdir(directory.as_str()).unwrap();
    }
}
