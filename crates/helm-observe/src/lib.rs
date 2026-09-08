//! `helm-observe` 0.1 — experimental explicit-target observation of actual
//! Linux filesystem facts, under Accepted
//! [ADR-0022](../../../docs/adr/ADR-0022-observation-authority.md).
//!
//! It answers exactly one question: **what actually exists at explicitly
//! authorised targets?**
//!
//! It does not answer what should exist, whether desired state is satisfied,
//! whether an application is compatible, whether a runtime is correct, whether
//! installation succeeded, whether a prefix is dedicated, whether DLL policy is
//! effective, or whether launch is safe. Comparison belongs to a future
//! `helm-bind`; execution belongs to a future `helm-launch`. Neither exists.
//!
//! It never executes a process, invokes Wine, interprets registry semantics,
//! consults PATH, HOME, cwd or system locations, discovers installations,
//! enumerates or recurses through directories, searches for files, touches the
//! network, or interprets a `helm-app-spec` requirement. It depends on neither
//! HELM crate; artifact identities, not Cargo types, connect the modules.
//!
//! # Authority
//!
//! ```text
//! untrusted plan bytes --parse_plan--> ValidatedPlan          (no authority)
//! ValidatedPlan + caller-opened roots + procfs --authorize--> AuthorizedScope
//! AuthorizedScope --observe--> ObservationArtifact
//! ```
//!
//! `authorize` consumes the plan, so "authorise plan A, observe plan B" cannot
//! be expressed through this API.
//!
//! # Supported cohort
//!
//! Linux x86_64 on local ext4, anchored by the OBS-FS-01 evidence cohort. Other
//! filesystems, architectures and kernels are **not** claimed. Target paths
//! whose correctness claim depends specifically on rejecting a bind mount
//! created as a descendant of an authorised ext4 root are outside the
//! empirically validated 0.1 cohort; `RESOLVE_NO_XDEV` remains mandatory and
//! such crossings are conservatively rejected, but no validation is claimed.
//!
//! # Consistency
//!
//! Sequential per-object capture. No atomic file or environment snapshot. A
//! digest identifies the bytes actually supplied through the retained descriptor
//! during that observed read sequence.

// The Linux backend is the only consumer of a few model and error helpers.
// On a non-Linux developer host those modules are cfg'd out, so the helpers
// are legitimately unreachable there. Linux CI still reports real dead code.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

pub mod error;
pub mod model;
pub mod plan;

#[cfg(target_os = "linux")]
mod authority;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod observe;

pub use error::{
    AdmissionError, AdmissionErrorCode, AdmissionErrors, PlanError, PlanErrorCode, PlanErrors,
};
pub use model::{
    BudgetReason, DirectoryFacts, Failure, FileFacts, ObjectKind, ObjectTuple, ObservationArtifact,
    ObservationRecord, Rejection, TargetObservation, TargetOutcome,
};
pub use plan::{
    Digest, MAX_COMPONENT_BYTES, MAX_ID_BYTES, MAX_JSON_DEPTH, MAX_PATH_BYTES, MAX_PATH_COMPONENTS,
    MAX_PLAN_BYTES, MAX_ROOTS, MAX_TARGETS, Observable, Target, ValidatedPlan, parse_plan,
};

#[cfg(target_os = "linux")]
pub use authority::{
    AuthorizedScope, ProcFdCapability, RootCapability, authorize,
    proc_fd_from_trusted_current_process, root_from_fd,
};
#[cfg(target_os = "linux")]
pub use observe::{MAX_FILE_BYTES, MAX_TOTAL_BYTES, READ_BUFFER_BYTES, observe};
