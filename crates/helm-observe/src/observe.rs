//! The observation loop.
//!
//! Structure matters for review: only the `Regular` arm of the classification
//! `match` can reach [`linux::reopen_and_stream`], so a symlink, FIFO, socket,
//! character device or block device provably cannot reach a data reopen.

use crate::authority::AuthorizedScope;
use crate::linux::{self, Resolved, Streamed};
use crate::model::{
    BudgetReason, DirectoryFacts, Failure, FileFacts, ObjectKind, ObservationArtifact,
    ObservationRecord, Rejection, TargetObservation, TargetOutcome,
};
use crate::plan::{Digest, Observable};

/// Maximum bytes hashed for one file.
pub const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
/// Maximum bytes read across one whole observation, including partial reads.
pub const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
/// Fixed streaming buffer. Never allocated from an observed file length.
pub const READ_BUFFER_BYTES: usize = 64 * 1024;

/// Observe every authorised target, in plan declaration order.
///
/// Sequential per-object capture. This is **not** an atomic file or environment
/// snapshot: one batch may describe a mixture of times, and a digest identifies
/// the bytes actually supplied through the retained descriptor during that read
/// sequence, subject to the documented consistency model.
#[must_use]
pub fn observe(scope: &AuthorizedScope) -> ObservationArtifact {
    let plan = scope.plan();
    let mut buffer = vec![0u8; READ_BUFFER_BYTES];
    let mut spent: u64 = 0;
    let mut exhausted = false;
    let mut targets = Vec::with_capacity(plan.targets().len());

    for target in plan.targets() {
        let outcome = if exhausted {
            TargetOutcome::NotObserved(BudgetReason::NotAttemptedTotalLimit)
        } else {
            let Some(root) = scope.root(target.root()) else {
                // Unreachable while authorize() binds the exact root set by ID.
                targets.push(TargetObservation {
                    target_id: target.id().to_owned(),
                    root_id: target.root().to_owned(),
                    outcome: TargetOutcome::Failed {
                        failure: Failure::IoFailure,
                        observed_kind: None,
                        partial_bytes: None,
                    },
                });
                continue;
            };
            observe_one(scope, root, target, &mut buffer, &mut spent, &mut exhausted)
        };
        targets.push(TargetObservation {
            target_id: target.id().to_owned(),
            root_id: target.root().to_owned(),
            outcome,
        });
    }

    ObservationArtifact::new(ObservationRecord {
        subject_spec_sha256: plan.subject_spec_sha256(),
        plan_sha256: scope.authorized_plan_sha256(),
        // Plan declaration order, not the caller's capability vector order. Roots
        // bind by logical ID, so the order in which the trusted caller happened to
        // hand over descriptors carries no meaning and must not change the
        // artifact's exact-byte identity. `authorize` has already established that
        // every declared root has exactly one capability.
        roots: plan
            .root_ids()
            .iter()
            .map(|id| (id.clone(), scope.root(id).and_then(|r| r.tuple())))
            .collect(),
        targets,
    })
}

fn observe_one(
    scope: &AuthorizedScope,
    root: &crate::authority::RootCapability,
    target: &crate::plan::Target,
    buffer: &mut [u8],
    spent: &mut u64,
    exhausted: &mut bool,
) -> TargetOutcome {
    // A target with no path denotes the supplied root directory object itself.
    let pinned = match target.path() {
        None => match rustix::io::dup(rustix::fd::AsFd::as_fd(root.fd())) {
            Ok(fd) => fd,
            Err(_) => {
                return TargetOutcome::Failed {
                    failure: Failure::IoFailure,
                    observed_kind: None,
                    partial_bytes: None,
                };
            }
        },
        Some(path) => match linux::resolve(root.fd(), path) {
            Resolved::Pinned(fd) => fd,
            Resolved::Absent => return TargetOutcome::Absent,
            Resolved::Rejected(r) => {
                return TargetOutcome::Rejected {
                    rejection: r,
                    observed_kind: None,
                };
            }
            Resolved::Failed(f) => {
                return TargetOutcome::Failed {
                    failure: f,
                    observed_kind: None,
                    partial_bytes: None,
                };
            }
        },
    };

    // Classification happens on the pinned descriptor, before any data access.
    let Ok(meta) = linux::describe(&pinned) else {
        return TargetOutcome::Failed {
            failure: Failure::IoFailure,
            observed_kind: None,
            partial_bytes: None,
        };
    };

    match target.observable() {
        Observable::DirectoryMetadata => {
            if meta.kind == ObjectKind::Directory {
                TargetOutcome::ObservedDirectory(DirectoryFacts { tuple: meta.tuple })
            } else {
                TargetOutcome::Rejected {
                    rejection: reject_for(meta.kind),
                    observed_kind: Some(meta.kind),
                }
            }
        }
        Observable::RegularFileSha256 => match meta.kind {
            // The only arm that may reach a data reopen.
            ObjectKind::Regular => stream_regular(scope, &pinned, &meta, buffer, spent, exhausted),
            other => TargetOutcome::Rejected {
                rejection: reject_for(other),
                observed_kind: Some(other),
            },
        },
    }
}

const fn reject_for(kind: ObjectKind) -> Rejection {
    match kind {
        ObjectKind::Symlink => Rejection::SymlinkForbidden,
        ObjectKind::Fifo
        | ObjectKind::Socket
        | ObjectKind::CharacterDevice
        | ObjectKind::BlockDevice
        | ObjectKind::Other => Rejection::SpecialFile,
        ObjectKind::Regular | ObjectKind::Directory => Rejection::WrongKind,
    }
}

fn stream_regular(
    scope: &AuthorizedScope,
    pinned: &std::os::fd::OwnedFd,
    meta: &linux::Meta,
    buffer: &mut [u8],
    spent: &mut u64,
    exhausted: &mut bool,
) -> TargetOutcome {
    // Budgets are enforced before any data reopen, so an over-limit file is
    // never opened for data at all.
    if meta.size > MAX_FILE_BYTES {
        return TargetOutcome::NotObserved(BudgetReason::FileLimit);
    }
    let need = meta.size.saturating_add(1);
    let remaining = MAX_TOTAL_BYTES.saturating_sub(*spent);
    if need > remaining {
        *exhausted = true;
        return TargetOutcome::NotObserved(BudgetReason::TotalLimit);
    }

    match linux::reopen_and_stream(scope.reopen().fd(), pinned, meta, buffer, remaining) {
        Streamed::Complete {
            bytes,
            sha256,
            link_count,
        } => {
            *spent = spent.saturating_add(bytes);
            TargetOutcome::ObservedFile(FileFacts {
                tuple: meta.tuple,
                link_count,
                bytes_read: bytes,
                sha256: Digest::from_raw(sha256),
            })
        }
        Streamed::Changed { partial } => {
            *spent = spent.saturating_add(partial);
            TargetOutcome::Failed {
                failure: Failure::ChangedDuringRead,
                observed_kind: Some(ObjectKind::Regular),
                partial_bytes: Some(partial),
            }
        }
        Streamed::Failed { failure, partial } => {
            *spent = spent.saturating_add(partial);
            TargetOutcome::Failed {
                failure,
                observed_kind: Some(ObjectKind::Regular),
                partial_bytes: Some(partial),
            }
        }
    }
}
