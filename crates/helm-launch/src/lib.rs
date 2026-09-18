//! `helm-launch` 0.1 — **P1, P2 and P3: the portable model, Linux x86_64
//! capability admission, and a crate-private Linux x86_64 process-creation
//! backend.**
//!
//! **There is no public launch API.** No public function, type or constant of
//! this crate can create a process: there is no `launch`, no `LaunchOutcome`,
//! no process handle, no pidfd, no child pid and no raw descriptor in the
//! public surface, on any platform. P1, P2 and P3 implement, under Accepted
//! [ADR-0024](../../../docs/adr/ADR-0024-launch-authority.md), the owner's P1
//! authorisation and acceptance of 2026-09-17, the owner's
//! [P2 authorisation of 2026-09-18](../../../docs/DECISIONS.md#helm-launch-p2-authorised),
//! and the owner's
//! [P3 authorisation of 2026-09-18](../../../docs/DECISIONS.md#helm-launch-p3-authorised):
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
//! * **P3, Linux x86_64 only, and entirely crate-private.** A `backend` module
//!   that consumes an `AuthorizedLaunch`, creates **one** direct child with
//!   `clone3(CLONE_PIDFD)` and attempts `execveat` on the exact admitted
//!   descriptor. It is the only place in the crate where scoped `unsafe` is
//!   allowed, it is not re-exported, and **no public item reaches it**.
//!
//! # Authority
//!
//! ```text
//! untrusted bytes            --parse_launch_plan--------▶ ValidatedLaunchPlan   intent only
//! caller-owned executable fd --admit_executable---------▶ ExecutableCapability
//! caller-owned cwd fd + id   --admit_working_directory--▶ WorkingDirectoryCapability
//! plan + executable + cwd    --authorize---------------▶ AuthorizedLaunch
//! AuthorizedLaunch           --╳------------------------▶ process   public API
//! AuthorizedLaunch           --crate-private backend----▶ process   P3, internal
//! ```
//!
//! **The public edge does not exist.** An `AuthorizedLaunch` an external caller
//! holds is inert: the only consumer that can turn it into a process is
//! crate-private, and so is every value that consumer produces.
//!
//! ```compile_fail
//! use helm_launch::launch;
//! ```
//!
//! ```compile_fail
//! use helm_launch::LaunchOutcome;
//! ```
//!
//! ```compile_fail
//! use helm_launch::backend;
//! ```
//!
//! ```compile_fail
//! use helm_launch::SpawnedChild;
//! ```
//!
//! ```compile_fail
//! use helm_launch::PreparedLaunch;
//! ```
//!
//! ```compile_fail
//! use helm_launch::ChildHandle;
//! ```
//!
//! ```compile_fail
//! use helm_launch::MinimalLaunch;
//! ```
//!
//! ```compile_fail
//! use helm_launch::Fault;
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
    doc = "```",
    doc = "",
    doc = "The P3 backend exists here too, and an external caller still cannot",
    doc = "reach it. These are the on-point proofs, not an unrelated missing",
    doc = "import: the module is private, and the only method that decomposes an",
    doc = "authorisation, and the only accessor of an admitted descriptor, are",
    doc = "crate-private. An outside caller has no expressible way to turn an",
    doc = "authorisation into a process.",
    doc = "",
    doc = "```compile_fail",
    doc = "fn execute(a: helm_launch::AuthorizedLaunch) {",
    doc = "    let _ = helm_launch::backend::launch_minimal(a);",
    doc = "}",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "fn decompose(a: helm_launch::AuthorizedLaunch) {",
    doc = "    let _ = a.into_parts();",
    doc = "}",
    doc = "```",
    doc = "",
    doc = "```compile_fail",
    doc = "fn descriptor(c: &helm_launch::ExecutableCapability) {",
    doc = "    let _ = c.descriptor();",
    doc = "}",
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
//! **No public process creation and no public process execution.** The P3
//! backend creates one direct child internally and attempts one execution of
//! the admitted descriptor; that is not reachable from outside the crate, emits
//! no receipt, and establishes **no** exec-success fact. A clean exec-status
//! end-of-file is indeterminate, and no `ExecSucceeded` value exists anywhere.
//!
//! No sandbox and no containment; no process-group sweep; no run timeout,
//! `SIGTERM` or grace period; no stream drain policy; no Wine; no `PATH`,
//! shell, command string or pathname launch; no authority from parsing; no
//! receipt authenticity. A digest identifies bytes and nothing more.
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
// scoped `allow` the owner authorised for `src/backend/` in P3. That allow is
// an inner attribute of `src/backend/mod.rs` and covers that module alone.
// `tests/p3_boundary.rs` fails if a second one appears anywhere, if the token
// appears as code outside the backend, or if any test source relaxes the lint.
#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]

// P2 (owner decision 2026-09-18): capability admission and authorisation
// composition, on the Linux x86_64 cohort only. Off the cohort this module is
// not compiled and none of its types or functions exists.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod authority;
// P3 (owner decision 2026-09-18): the unsafe Linux x86_64 process-creation
// backend and the closed post-clone child contract. **Private, and re-exported
// nowhere**: there is no `pub use backend::…` line in this file, so no item of
// it is nameable outside this crate. Off the cohort it is not compiled at all.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod backend;
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
