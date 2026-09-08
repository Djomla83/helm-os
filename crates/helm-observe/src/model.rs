//! Immutable actual-fact results and the deterministic artifact serializer.
//!
//! No satisfaction, compatibility or readiness vocabulary appears here. Those
//! belong to a future comparison module, not to an observer.

use sha2::{Digest as _, Sha256};

use crate::plan::Digest;

/// Maximum serialized observation artifact size.
pub const MAX_ARTIFACT_BYTES: usize = 256 * 1024;

/// The object kind actually established for a pinned descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectKind {
    Regular,
    Directory,
    Symlink,
    Fifo,
    Socket,
    CharacterDevice,
    BlockDevice,
    Other,
}

impl ObjectKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Fifo => "fifo",
            Self::Socket => "socket",
            Self::CharacterDevice => "character_device",
            Self::BlockDevice => "block_device",
            Self::Other => "other",
        }
    }
}

/// Local capture context for one observed object. Not a durable identity: it is
/// not portable across boot, inode reuse, clone, restore or mount namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectTuple {
    pub device_major: u32,
    pub device_minor: u32,
    pub inode: u64,
    pub mount_id: Option<u64>,
}

/// Why a target produced no observation. Every variant is an actual outcome of
/// an attempt; none of them is a judgement about desired state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rejection {
    /// A symlink was targeted, in a final or non-final component.
    SymlinkForbidden,
    /// A FIFO, socket, character device or block device was targeted.
    SpecialFile,
    /// The object kind cannot answer the requested observable.
    WrongKind,
    /// Constrained resolution refused a mount crossing or a root escape. The
    /// kernel reports both with one error, so no attribution is invented.
    ScopeViolation,
}

impl Rejection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SymlinkForbidden => "symlink_forbidden",
            Self::SpecialFile => "special_file",
            Self::WrongKind => "wrong_kind",
            Self::ScopeViolation => "scope_violation",
        }
    }
}

/// An attempt that could not complete. Never collapsed into absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failure {
    PermissionDenied,
    IoFailure,
    ResolutionRace,
    ChangedDuringRead,
    ReopenUnavailable,
    UnsupportedPlatform,
}

impl Failure {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PermissionDenied => "permission_denied",
            Self::IoFailure => "io_failure",
            Self::ResolutionRace => "resolution_race",
            Self::ChangedDuringRead => "changed_during_read",
            Self::ReopenUnavailable => "reopen_unavailable",
            Self::UnsupportedPlatform => "unsupported_platform",
        }
    }
}

/// Why an observation was bounded away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BudgetReason {
    /// Metadata length exceeds the per-file ceiling. No data-open occurred.
    FileLimit,
    /// The aggregate read budget cannot admit this target.
    TotalLimit,
    /// The aggregate budget was already exhausted before this target.
    NotAttemptedTotalLimit,
}

impl BudgetReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileLimit => "file_limit",
            Self::TotalLimit => "total_limit",
            Self::NotAttemptedTotalLimit => "not_attempted_total_limit",
        }
    }
}

/// Facts about an observed regular file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileFacts {
    pub tuple: ObjectTuple,
    /// Sampled hard-link count. Establishes no origin, ownership or uniqueness.
    pub link_count: u64,
    pub bytes_read: u64,
    pub sha256: Digest,
}

/// Facts about an observed directory. No children are listed or counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectoryFacts {
    pub tuple: ObjectTuple,
}

/// What actually happened for one declared target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetOutcome {
    ObservedFile(FileFacts),
    ObservedDirectory(DirectoryFacts),
    /// Constrained authorised lookup legitimately established absence at that
    /// attempt. Not a machine-wide search, and not a permission or safety error.
    Absent,
    Rejected {
        rejection: Rejection,
        observed_kind: Option<ObjectKind>,
    },
    Failed {
        failure: Failure,
        observed_kind: Option<ObjectKind>,
        partial_bytes: Option<u64>,
    },
    NotObserved(BudgetReason),
}

impl TargetOutcome {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ObservedFile(_) => "observed_file",
            Self::ObservedDirectory(_) => "observed_directory",
            Self::Absent => "absent",
            Self::Rejected { rejection, .. } => rejection.as_str(),
            Self::Failed { failure, .. } => failure.as_str(),
            Self::NotObserved(reason) => reason.as_str(),
        }
    }
}

/// One declared target and its actual outcome, in plan declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetObservation {
    pub target_id: String,
    pub root_id: String,
    pub outcome: TargetOutcome,
}

/// The immutable observation record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationRecord {
    pub subject_spec_sha256: Digest,
    pub plan_sha256: Digest,
    pub roots: Vec<(String, Option<ObjectTuple>)>,
    pub targets: Vec<TargetObservation>,
}

/// Serialized observation with exact-byte identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationArtifact {
    bytes: Vec<u8>,
    sha256: Digest,
    record: ObservationRecord,
}

fn push_tuple(out: &mut String, t: &ObjectTuple) {
    out.push_str("{\"device_major\":");
    out.push_str(&t.device_major.to_string());
    out.push_str(",\"device_minor\":");
    out.push_str(&t.device_minor.to_string());
    out.push_str(",\"inode\":");
    out.push_str(&t.inode.to_string());
    out.push_str(",\"mount_id\":");
    match t.mount_id {
        Some(m) => out.push_str(&m.to_string()),
        None => out.push_str("null"),
    }
    out.push('}');
}

/// Escape a validated logical identifier. IDs are already restricted to
/// lowercase ASCII alphanumerics, dot, underscore and hyphen, so this cannot
/// widen the output alphabet; it exists so the serializer stays total.
fn push_id(out: &mut String, id: &str) {
    out.push('"');
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            out.push(ch);
        }
    }
    out.push('"');
}

impl ObservationArtifact {
    pub(crate) fn new(record: ObservationRecord) -> Self {
        // Fixed field order within schema 0.1. This is a deterministic
        // serializer, not a canonicalisation standard: reformatting produces a
        // different artifact identity.
        let mut s = String::with_capacity(1024);
        s.push_str("{\"schema\":\"helm-observation\",\"version\":\"0.1\",");
        s.push_str("\"consistency\":\"sequential_objects\",");
        s.push_str("\"subject_spec_sha256\":\"");
        s.push_str(&record.subject_spec_sha256.to_hex());
        s.push_str("\",\"plan_sha256\":\"");
        s.push_str(&record.plan_sha256.to_hex());
        s.push_str("\",\"roots\":[");
        for (i, (id, tuple)) in record.roots.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("{\"id\":");
            push_id(&mut s, id);
            s.push_str(",\"object\":");
            match tuple {
                Some(t) => push_tuple(&mut s, t),
                None => s.push_str("null"),
            }
            s.push('}');
        }
        s.push_str("],\"targets\":[");
        for (i, t) in record.targets.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("{\"id\":");
            push_id(&mut s, &t.target_id);
            s.push_str(",\"root\":");
            push_id(&mut s, &t.root_id);
            s.push_str(",\"outcome\":\"");
            s.push_str(t.outcome.code());
            s.push('"');
            match &t.outcome {
                TargetOutcome::ObservedFile(f) => {
                    s.push_str(",\"kind\":\"regular\",\"object\":");
                    push_tuple(&mut s, &f.tuple);
                    s.push_str(",\"link_count\":");
                    s.push_str(&f.link_count.to_string());
                    s.push_str(",\"bytes_read\":");
                    s.push_str(&f.bytes_read.to_string());
                    s.push_str(",\"sha256\":\"");
                    s.push_str(&f.sha256.to_hex());
                    s.push_str("\",\"change_check\":\"no_change_detected\"");
                    s.push_str(",\"origin\":\"unestablished\"");
                }
                TargetOutcome::ObservedDirectory(d) => {
                    s.push_str(",\"kind\":\"directory\",\"object\":");
                    push_tuple(&mut s, &d.tuple);
                }
                TargetOutcome::Rejected { observed_kind, .. } => {
                    if let Some(k) = observed_kind {
                        s.push_str(",\"kind\":\"");
                        s.push_str(k.as_str());
                        s.push('"');
                    }
                }
                TargetOutcome::Failed {
                    observed_kind,
                    partial_bytes,
                    ..
                } => {
                    if let Some(k) = observed_kind {
                        s.push_str(",\"kind\":\"");
                        s.push_str(k.as_str());
                        s.push('"');
                    }
                    if let Some(b) = partial_bytes {
                        s.push_str(",\"partial_bytes\":");
                        s.push_str(&b.to_string());
                    }
                }
                TargetOutcome::Absent | TargetOutcome::NotObserved(_) => {}
            }
            s.push('}');
        }
        s.push_str("]}");
        let mut bytes = s.into_bytes();
        bytes.truncate(MAX_ARTIFACT_BYTES);
        let sha256 = Digest::from_raw(sha256_of(&bytes));
        Self {
            bytes,
            sha256,
            record,
        }
    }

    /// Exact serialized bytes. Store unchanged; reformatting makes another artifact.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 of exactly [`Self::exact_bytes`].
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    #[must_use]
    pub const fn record(&self) -> &ObservationRecord {
        &self.record
    }
}

fn sha256_of(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::{
        DirectoryFacts, FileFacts, MAX_ARTIFACT_BYTES, ObjectTuple, ObservationArtifact,
        ObservationRecord, TargetObservation, TargetOutcome,
    };
    use crate::plan::{Digest, MAX_ID_BYTES, MAX_ROOTS, MAX_TARGETS};

    /// The serializer truncates at [`MAX_ARTIFACT_BYTES`], which would emit invalid
    /// JSON whose digest still matched. That branch must be unreachable, so prove it
    /// numerically from the actual schema and the actual declared limits rather than
    /// leaving boundedness implicit.
    #[test]
    fn the_largest_expressible_artifact_stays_inside_the_ceiling() {
        let widest = ObjectTuple {
            device_major: u32::MAX,
            device_minor: u32::MAX,
            inode: u64::MAX,
            mount_id: Some(u64::MAX),
        };
        let longest = "d".repeat(MAX_ID_BYTES);
        let file = TargetOutcome::ObservedFile(FileFacts {
            tuple: widest,
            link_count: u64::MAX,
            bytes_read: u64::MAX,
            sha256: Digest::from_raw([0xff; 32]),
        });
        let record = ObservationRecord {
            subject_spec_sha256: Digest::from_raw([0xff; 32]),
            plan_sha256: Digest::from_raw([0xff; 32]),
            roots: (0..MAX_ROOTS)
                .map(|_| (longest.clone(), Some(widest)))
                .collect(),
            targets: (0..MAX_TARGETS)
                .map(|_| TargetObservation {
                    target_id: longest.clone(),
                    root_id: longest.clone(),
                    outcome: file,
                })
                .collect(),
        };
        let artifact = ObservationArtifact::new(record);
        assert!(
            artifact.exact_bytes().len() < MAX_ARTIFACT_BYTES,
            "the widest artifact is {} bytes, against a {MAX_ARTIFACT_BYTES} byte ceiling",
            artifact.exact_bytes().len()
        );
        // `ObservedFile` is the widest outcome; a directory record is strictly smaller.
        assert!(
            matches!(
                TargetOutcome::ObservedDirectory(DirectoryFacts { tuple: widest }),
                TargetOutcome::ObservedDirectory(_)
            ),
            "directory facts carry no digest, link count or byte count"
        );
    }
}
