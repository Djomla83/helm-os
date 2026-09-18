//! `helm-launch` 0.1 — **P1 and P2 only: the portable model plus Linux x86_64
//! capability admission.**
//!
//! **No execution backend exists. Nothing in this crate can create a process.**
//! There is no `launch` function and no `LaunchOutcome`. P1 and P2 implement,
//! under Accepted [ADR-0024](../../../docs/adr/ADR-0024-launch-authority.md),
//! the owner's P1 authorisation and acceptance of 2026-09-17 and the owner's
//! [P2 authorisation of 2026-09-18](../../../docs/DECISIONS.md#helm-launch-p2-authorised):
//!
//! * **P1, portable.** [`parse_launch_plan`]: untrusted plan bytes to an inert
//!   [`ValidatedLaunchPlan`], with exact-byte SHA-256 identity and no I/O;
//!   [`Digest`], the closed non-verdict fact enums and [`ReceiptRecord`];
//!   [`LaunchReceipt`] and its deterministic serializer, with no public
//!   producer; the plan error vocabulary ([`LaunchPlanErrors`]); and
//!   crate-private pure models of the descriptor layout and of the lifecycle
//!   policy, exercised only by tests.
//! * **P2, Linux x86_64 only.** `admit_executable` and
//!   `admit_working_directory`, which turn a descriptor a trusted caller moves
//!   in into an `ExecutableCapability` or a `WorkingDirectoryCapability`; and
//!   `authorize`, which composes a validated plan with both into a single-use
//!   `AuthorizedLaunch`. Admission performs read-only I/O through the
//!   caller's own descriptor; the admission and refusal vocabularies
//!   ([`AdmissionError`], [`AuthorizationRefusal`]) are portable data.
//!
//! # Authority
//!
//! ```text
//! untrusted bytes            --parse_launch_plan--------▶ ValidatedLaunchPlan   intent only
//! caller-owned executable fd --admit_executable---------▶ ExecutableCapability
//! caller-owned cwd fd + id   --admit_working_directory--▶ WorkingDirectoryCapability
//! plan + executable + cwd    --authorize---------------▶ AuthorizedLaunch
//! AuthorizedLaunch           --╳------------------------▶ process
//! ```
//!
//! **The last edge does not exist.** An `AuthorizedLaunch` is an inert
//! in-process value: no function consumes it to create a process, because
//! `launch` does not exist on any platform.
//!
//! ```compile_fail
//! use helm_launch::launch;
//! ```
//!
//! ```compile_fail
//! use helm_launch::LaunchOutcome;
//! ```
//!
//! Parsing a plan yields caller intent and nothing else. A validated plan holds
//! no descriptor, names no executable and cannot cause execution. No function
//! in this crate turns bytes, a string, a path, a receipt or any `serde` input
//! into authority: only an owned descriptor a trusted caller moves in carries
//! any.
#![cfg_attr(
    all(target_os = "linux", target_arch = "x86_64"),
    doc = "",
    doc = "# On this platform",
    doc = "",
    doc = "This is the Linux x86_64 cohort, so the capability, authorisation and",
    doc = "admission APIs exist:",
    doc = "",
    doc = "```",
    doc = "use helm_launch::{admit_executable, admit_working_directory, authorize};",
    doc = "use helm_launch::{AuthorizedLaunch, ExecutableCapability, WorkingDirectoryCapability};",
    doc = "",
    doc = "fn assert_send<T: Send>() {}",
    doc = "assert_send::<ExecutableCapability>();",
    doc = "assert_send::<WorkingDirectoryCapability>();",
    doc = "assert_send::<AuthorizedLaunch>();",
    doc = "let _ = (admit_executable, admit_working_directory, authorize);",
    doc = "```"
)]
#![cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64")),
    doc = "",
    doc = "# On this platform",
    doc = "",
    doc = "This is **not** the Linux x86_64 cohort. The portable model above is",
    doc = "available, and the capability, authorisation and admission APIs do not",
    doc = "exist in the public API at all — they are not stubbed:",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::admit_executable;",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::admit_working_directory;",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::authorize;",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::ExecutableCapability;",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::WorkingDirectoryCapability;",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "use helm_launch::AuthorizedLaunch;",
    doc = "```",
    doc = "",
    doc = "No support for another platform is advertised, and Linux on another",
    doc = "architecture is not claimed."
)]
//!
//! # No verdicts
//!
//! No fact type offers a success reading. There is no `bool` conversion, no
//! `is_success`, and no exec-success value:
//!
//! ```compile_fail
//! use helm_launch::{ExecStatus, IndeterminateReason};
//! let s = ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord);
//! let _: bool = s.into();
//! ```
//!
//! ```compile_fail
//! use helm_launch::{ExecStatus, IndeterminateReason};
//! let s = ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord);
//! if s {}
//! ```
//!
//! ```compile_fail
//! use helm_launch::ChildEnd;
//! let _ = ChildEnd::Exited { code: 0 }.is_success();
//! ```
//!
//! ```compile_fail
//! let _ = helm_launch::ExecStatus::ExecSucceeded;
//! ```
//!
//! # Non-claims
//!
//! No process creation, no process execution, no sandbox and no containment; no
//! Wine; no `PATH`, shell, command string or pathname launch; no authority from
//! parsing; no receipt authenticity. A digest identifies bytes and nothing
//! more.
//!
//! Executable measurement is a **pre-execution measurement of the pinned
//! object**, never the identity of bytes that executed and never a statement
//! about an interpreter, a library or any other loaded code. A successful
//! admission says only that **the measurement protocol detected no
//! instability** in the fields it samples; it never says that no mutation
//! occurred, that the object is immutable, or that a snapshot exists. Admission
//! reads the caller's object, so it may update atime and populates the page
//! cache.

// The accepted policy of plan section 7.4, restated in source as in the
// manifest: `deny`, not `forbid`, because only `deny` leaves room for the one
// scoped `allow` that a later, separately authorised backend slice may need.
// No such slice is authorised: P1 and P2 contain no code either lint would
// reject and no `allow` of either, and `tests/p2_boundary.rs` fails on any.
#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

// P2 (owner decision 2026-09-18): capability admission and authorisation
// composition, on the Linux x86_64 cohort only. Off the cohort this module is
// not compiled and none of its types or functions exists.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod authority;
mod error;
mod layout;
mod lifecycle;
mod model;
mod plan;
mod receipt;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use authority::{
    AuthorizedLaunch, ExecutableCapability, MAX_EXECUTABLE_BYTES, WorkingDirectoryCapability,
    admit_executable, admit_working_directory, authorize,
};
pub use error::{
    AdmissionError, AdmissionErrorCode, AuthorizationRefusal, AuthorizationRefusalCode,
    LaunchPlanError, LaunchPlanErrorCode, LaunchPlanErrors, MAX_PLAN_ERRORS,
};
pub use model::{
    AssertedContext, Backend, ChildEnd, ChildStage, Completeness, Digest, ElfType, EnvironmentMode,
    ExecStatus, ExecutableMeasurement, GroupSweep, IndeterminateReason, MAX_ID_BYTES,
    ReceiptRecord, Stream, StreamFacts, Termination,
};
pub use plan::{
    ExecutionKind, MAX_ARG_BYTES, MAX_ARGS, MAX_ARGV_TOTAL_BYTES, MAX_CAPTURE_BYTES, MAX_GRACE_MS,
    MAX_JSON_DEPTH, MAX_PLAN_BYTES, MAX_TIMEOUT_MS, MIN_TIMEOUT_MS, StdinMode, TerminationSignal,
    ValidatedLaunchPlan, parse_launch_plan,
};
pub use receipt::{LaunchReceipt, MAX_RECEIPT_BYTES};
