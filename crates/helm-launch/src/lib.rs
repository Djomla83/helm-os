//! `helm-launch` 0.1 — **P1 only: the portable, pure model.**
//!
//! **No execution backend exists. Nothing in this crate can create a process.**
//! There is no executable or working-directory capability, no authorisation and
//! no launch function. P1 implements, under Accepted
//! [ADR-0024](../../../docs/adr/ADR-0024-launch-authority.md) and the owner's P1
//! authorisation of 2026-09-17:
//!
//! * [`parse_launch_plan`]: untrusted plan bytes to an inert
//!   [`ValidatedLaunchPlan`], with exact-byte SHA-256 identity and no I/O;
//! * [`Digest`], the closed non-verdict fact enums and [`ReceiptRecord`];
//! * [`LaunchReceipt`] and its deterministic serializer, with no public producer;
//! * the plan error vocabulary ([`LaunchPlanErrors`]);
//! * crate-private pure models of the descriptor layout and of the lifecycle
//!   policy, exercised only by tests.
//!
//! # Authority
//!
//! ```text
//! untrusted bytes --parse_launch_plan--> ValidatedLaunchPlan   intent only, no authority
//! ```
//!
//! Parsing a plan yields caller intent and nothing else. A validated plan holds
//! no descriptor, names no executable and cannot cause execution. No function in
//! this crate turns bytes, a string, a path or any serde input into authority.
//!
//! The capability, authorisation and launch APIs of the productization plan do
//! not exist in P1 and are not stubbed:
//!
//! ```compile_fail
//! use helm_launch::launch;
//! ```
//!
//! ```compile_fail
//! use helm_launch::authorize;
//! ```
//!
//! ```compile_fail
//! use helm_launch::admit_executable;
//! ```
//!
//! ```compile_fail
//! use helm_launch::admit_working_directory;
//! ```
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
//! No process execution, sandbox or containment; no Wine; no `PATH` or shell;
//! no authority from parsing; no receipt authenticity. A digest identifies bytes
//! and nothing more.

// The accepted policy of plan section 7.4, restated in source as in the
// manifest: `deny`, not `forbid`, because only `deny` leaves room for the one
// scoped `allow` that a later, separately authorised backend slice may need.
// No such slice is authorised: P1 contains no code either lint would reject and
// no `allow` of either, and `tests/p1_boundary.rs` fails on any.
#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

mod error;
mod layout;
mod lifecycle;
mod model;
mod plan;
mod receipt;

pub use error::{LaunchPlanError, LaunchPlanErrorCode, LaunchPlanErrors, MAX_PLAN_ERRORS};
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
