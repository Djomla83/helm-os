//! `helm-bind` 0.1 — experimental pure comparison of **desired** application
//! claims with the **actual** observations that were made, under Accepted
//! [ADR-0023](../../../docs/adr/ADR-0023-binding-authority.md).
//!
//! It answers exactly one question: **what follows from comparing explicitly
//! mapped desired claims with the observations that were actually made?**
//!
//! It does not answer whether the application is satisfied, compatible, correctly
//! installed, ready or launchable. There is **no satisfaction, compatibility or
//! readiness verdict** in this crate, and no global success token of any kind.
//! Execution belongs to a future `helm-launch`, which does not exist. Evidence
//! completeness belongs to `helm-evidence`, which this crate does not depend on.
//!
//! # Authority
//!
//! ```text
//! untrusted mapping bytes --parse_binding_plan--> ValidatedBindingPlan   (inert)
//! ValidatedAppSpec + ValidatedBindingPlan + ValidatedPlan + ObservationArtifact
//!                        --bind--> BindingReport
//! ```
//!
//! `bind` is a **pure function**. It performs no filesystem, descriptor, network,
//! process, environment, registry, clock or randomness access, has no OS backend
//! and no product-semantic `cfg` branch, and produces byte-identical output on
//! Linux, Windows and macOS for the same four inputs.
//!
//! # Two axes, and no verdict
//!
//! A report carries one typed outcome per semantic desired claim, plus:
//!
//! - **contradiction** — `Contradicted` if and only if at least one claim is a
//!   `Mismatch` of two known values. Absence, observer rejection, observer
//!   failure, budget omission, unsupported binding, an unmapped claim, an
//!   unspecified desired value and an uninterpretable outcome are never promoted.
//! - **coverage** — deterministic counts over the ten states, plus the claim
//!   classes that have no comparator at all.
//!
//! **Coverage is necessarily incomplete in 0.1.** Every valid specification
//! carries four mandatory semantic requirements that `helm-observe` 0.1 cannot
//! establish — source architecture, runtime family, Windows architecture and
//! prefix role — so a caller cannot map a couple of easy digests, omit the rest
//! and obtain a success token. No such token exists.
//!
//! # What a match does not mean
//!
//! An agreeing body establishes the identity of **that body** and nothing else:
//! not that it was installed, not that it produced the current prefix, not that a
//! loader came from it, not that a process loaded it. An agreeing entry point
//! establishes a path binding, a regular-file kind and, when the specification
//! states one, a digest — never executability, PE validity, Wine loadability,
//! correct installation or launchability. A verification-definition body match is
//! definition-body identity, never verification executed, passed or fresh.
//!
//! # Assertions are not attestations
//!
//! `asserted_prefix_root_id` is a **caller assertion**. It proves neither
//! dedication, nor ownership, nor that the root belongs to this application, and
//! no report upgrades it. Likewise the identity checks are consistency checks
//! between supplied documents; they establish nothing about who supplied them.
#![forbid(unsafe_code)]

pub mod bind;
pub mod error;
pub mod model;
pub mod plan;

pub use bind::bind;
pub use error::{
    BindingPlanError, BindingPlanErrorCode, BindingPlanErrors, BindingRefusal, BindingRefusalCode,
    MAX_PLAN_ERRORS,
};
pub use model::{
    BindingRecord, BindingReport, ClaimBinding, ClaimState, ClaimSubject, Contradiction, Coverage,
    Difference, FailureReason, InputIdentities, MAX_REPORT_BYTES, OmissionReason, RejectionReason,
};
pub use plan::{
    ClaimKind, ClaimMapping, Digest, MAX_CLAIMS, MAX_ID_BYTES, MAX_JSON_DEPTH, MAX_PLAN_BYTES,
    ValidatedBindingPlan, parse_binding_plan,
};
