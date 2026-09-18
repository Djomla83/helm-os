//! The Linux x86_64 process-creation backend (**P3**), and the **only** place
//! in this crate where `unsafe` is permitted.
//!
//! This module is compiled only under
//! `cfg(all(target_os = "linux", target_arch = "x86_64"))`, is **private**, and
//! re-exports nothing. No item of it is reachable from outside the crate: there
//! is no public `launch`, no public spawn, no public process handle and no
//! public pidfd, child pid or raw descriptor. An external caller still has no
//! way to make `helm-launch` create a process.
//!
//! # What P3 does
//!
//! It consumes an `AuthorizedLaunch` — the first crate-private consumer that
//! value has ever had — prepares every descriptor, pointer and byte in the
//! parent, creates **one** direct child with `clone3(CLONE_PIDFD)`, and lets
//! that child walk the closed sequence of [`child`] to an `execveat` of the
//! exact descriptor the caller admitted.
//!
//! # What P3 deliberately does not do
//!
//! No public `launch`, no `LaunchOutcome`, no receipt from a real launch, no
//! observation loop, no plan-driven run timeout, no `SIGTERM`, no grace period,
//! no general stream drain policy, **no process-group sweep**, no process-tree
//! containment, no Wine, no orchestration and no sandbox. Those are P4 and P5,
//! which are not authorised.
//!
//! The parent's `setpgid(child, child)` result is recorded as
//! `group_authority_established` and is used for nothing here. **No negative-pid
//! signal exists anywhere in this crate**, and `tests/p3_boundary.rs` fails if
//! one appears.
//!
//! # The unsafe boundary
//!
//! The crate root denies `unsafe_code`. This file carries the single scoped
//! `#![allow(unsafe_code)]`, which covers exactly this module and its
//! descendants, and adds the four backend-only `deny`s. Six operations are
//! authorised, each in its own block with its own `// SAFETY:` comment:
//!
//! | # | Operation | Where |
//! |---|---|---|
//! | 1 | raw `rt_sigprocmask` around `clone3`, and the restore | [`spawn`] |
//! | 2 | raw `clone3` | [`spawn`] |
//! | 3 | `OwnedFd::from_raw_fd(pidfd)` right after a successful `clone3` | [`spawn`] |
//! | 4 | crossing into the child entry with the prepared `ChildPlan` pointer | [`spawn`] |
//! | 5 | the raw child syscalls of the closed post-clone sequence | [`child`] |
//! | 6 | the x86_64 `asm!` syscall shim itself | [`syscall`] |
//!
//! Nothing else is unsafe here. Parent-side work that a safe `rustix` wrapper
//! performs — pipes, descriptor relocation, status flags, `setpgid`, reads,
//! pidfd signalling, `waitid` — uses that wrapper and is not reimplemented raw.

// The one scoped exception the accepted architecture reserves for this slice
// (ADR-0024 section E, plan section 7.4). It is an inner attribute of this
// module, so it covers `syscall`, `spawn`, `child` and nothing else. The crate
// root's `deny` stays in force everywhere else, and `tests/p3_boundary.rs`
// fails if a second `allow` of this lint appears anywhere in the crate.
#![allow(unsafe_code)]
// The backend-only discipline of plan section 7.4. Indexing, unchecked
// arithmetic and `as` casts are all ways to be silently wrong about a
// descriptor number, a range bound or an ABI width, so none of them is
// available in the one module that talks to the kernel directly.
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block,
    unsafe_op_in_unsafe_fn
)]
// P3 adds no public entry point, so outside the test build nothing in the crate
// calls this module. That is the accepted shape of the slice, not dead code
// that should be deleted: P4 owns the public consumer.
#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "P3 is a crate-private backend; the public consumer belongs to a slice that is not authorised"
    )
)]

mod child;
#[cfg(all(feature = "test-fault-injection", debug_assertions))]
mod injection;
mod spawn;
mod syscall;

#[cfg(test)]
mod tests;

use std::os::fd::OwnedFd;
use std::time::{Duration, Instant};

pub(crate) use spawn::ChildHandle;

use crate::authority::AuthorizedLaunch;
use crate::model::{ChildStage, Digest, ExecStatus, ExecutableMeasurement, IndeterminateReason};

// ------------------------------------------------------------- fixed bounds

/// The fixed pre-exec bound (S6). It is **not** a run timeout: `plan.timeout_ms`
/// is not executed anywhere in P3, and no `SIGTERM`, grace period or run
/// deadline exists.
pub(crate) const SPAWN_CONFIRM_TIMEOUT_MS: u64 = 5_000;

/// The fixed bound on every reap that follows a direct-child `SIGKILL` (T40).
pub(crate) const POST_KILL_REAP_MS: u64 = 5_000;

/// How long a bounded wait sleeps between non-blocking attempts.
///
/// P3 has no `poll` and no event loop: the `event` feature of `rustix` is not
/// enabled, because the observation loop that would need it is P4's.
pub(crate) const POLL_INTERVAL_MS: u64 = 1;

/// Whether this build compiled the test-only fault injection at all.
///
/// The gate is deliberately two conditions: the non-default feature **and**
/// debug assertions. A release build therefore contains no injection even if
/// the feature is enabled by accident.
pub(crate) const INJECTION_COMPILED: bool =
    cfg!(all(feature = "test-fault-injection", debug_assertions));

const _: () = assert!(!INJECTION_COMPILED || cfg!(debug_assertions));

// -------------------------------------------------------------- child plan

/// The stage codes of the closed child sequence, as they travel in the failure
/// record's first byte.
pub(crate) mod stage {
    /// No stage. Never written into a record.
    pub(crate) const NONE: u8 = 0;
    /// Final stdio mapping.
    pub(crate) const DUP2: u8 = 1;
    /// Clearing close-on-exec on 0, 1 and 2.
    pub(crate) const CLEAR_CLOEXEC: u8 = 2;
    /// Changing to the working-directory capability.
    pub(crate) const CHDIR: u8 = 3;
    /// Closing every non-preserved descriptor.
    pub(crate) const CLOSE_RANGE: u8 = 4;
    /// Entering the dedicated process group.
    pub(crate) const SETPGID: u8 = 5;
    /// Resetting every signal disposition to the default.
    pub(crate) const SIGACTION: u8 = 6;
    /// Installing the final, empty signal mask.
    pub(crate) const SIGMASK: u8 = 7;
    /// Setting `no_new_privs`.
    pub(crate) const NO_NEW_PRIVS: u8 = 8;
    /// The execution attempt itself.
    pub(crate) const EXEC: u8 = 9;
}

/// The stage sequence, in the one order the child walks it.
pub(crate) const STAGE_ORDER: [u8; 9] = [
    stage::DUP2,
    stage::CLEAR_CLOEXEC,
    stage::CHDIR,
    stage::CLOSE_RANGE,
    stage::SETPGID,
    stage::SIGACTION,
    stage::SIGMASK,
    stage::NO_NEW_PRIVS,
    stage::EXEC,
];

/// Map a record's stage byte onto the public vocabulary.
pub(crate) const fn stage_of(code: u8) -> Option<ChildStage> {
    match code {
        stage::DUP2 => Some(ChildStage::Dup2),
        stage::CLEAR_CLOEXEC => Some(ChildStage::ClearCloexec),
        stage::CHDIR => Some(ChildStage::Chdir),
        stage::CLOSE_RANGE => Some(ChildStage::CloseRange),
        stage::SETPGID => Some(ChildStage::Setpgid),
        stage::SIGACTION => Some(ChildStage::Sigaction),
        stage::SIGMASK => Some(ChildStage::Sigmask),
        stage::NO_NEW_PRIVS => Some(ChildStage::NoNewPrivs),
        stage::EXEC => Some(ChildStage::Exec),
        _ => None,
    }
}

/// One planned descriptor range for the child's `close_range`.
///
/// `used == 0` marks a span the pure layout plan did not produce, which happens
/// exactly when a gap is inverted — the adjacent (F7) and lowest-possible
/// cases. It is skipped rather than issued.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct CloseSpan {
    /// First descriptor number of the inclusive range.
    pub(crate) first: usize,
    /// Last descriptor number of the inclusive range.
    pub(crate) last: usize,
    /// 1 when the span is to be issued, 0 when it is to be skipped.
    pub(crate) used: usize,
}

impl CloseSpan {
    /// A span the child skips.
    pub(crate) const UNUSED: Self = Self {
        first: 0,
        last: 0,
        used: 0,
    };
}

/// A test-only fault request.
///
/// In any build without the `test-fault-injection` feature, and in **every**
/// release build, this type has no fields, is zero-sized and has no reader:
/// the code that would consult it is not compiled.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Fault {
    /// The stage to report as failed without issuing its system call.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    pub(crate) stage: u8,
    /// The `errno` such an injected stage failure reports.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    pub(crate) errno: i32,
    /// What to do immediately before the execution attempt.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    pub(crate) mode: u8,
}

/// Everything the child needs, as one plain-old-data record.
///
/// It holds integers and addresses only: no `OwnedFd`, no `Vec`, no `String`,
/// no `CString`, no `Box` and nothing with a destructor. The child copies it
/// once and computes nothing from anything else. The `needs_drop` assertion
/// below is a **compile-time** proof, not a runtime test.
///
/// Every address in it names memory of the frame that issues `clone3`, or of a
/// heap buffer that frame owns; the child reaches all of it through its
/// copy-on-write copy of the parent's address space.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ChildPlan {
    /// `dup2` source for descriptor 0.
    pub(crate) stdin_source: usize,
    /// `dup2` source for descriptor 1.
    pub(crate) stdout_source: usize,
    /// `dup2` source for descriptor 2.
    pub(crate) stderr_source: usize,
    /// `fchdir` target: the admitted working directory.
    pub(crate) working_directory: usize,
    /// The admitted executable, preserved by the close ranges.
    pub(crate) executable: usize,
    /// The exec-status write end, preserved by the close ranges.
    pub(crate) status_write: usize,
    /// At most three ascending, never inverted ranges.
    pub(crate) close_ranges: [CloseSpan; 3],
    /// Address of the null-terminated `argv` pointer array.
    pub(crate) argv: usize,
    /// Address of the one-element `envp` array, whose only element is null.
    pub(crate) envp: usize,
    /// Address of the one-byte empty pathname `execveat` requires.
    pub(crate) empty_path: usize,
    /// Address of the kernel `sigaction` record that installs `SIG_DFL`.
    pub(crate) default_action: usize,
    /// Address of the zeroed kernel signal set the child ends with.
    pub(crate) empty_signal_mask: usize,
    /// The prepared, all-zero eight-byte failure record.
    pub(crate) status_record: [u8; 8],
    /// Test-only fault request; zero-sized outside an injection build.
    pub(crate) fault: Fault,
}

// The child must not run a destructor, so the value it copies must not have
// one. This is the compile-time proof the accepted plan requires.
const _: () = assert!(!core::mem::needs_drop::<ChildPlan>());
const _: () = assert!(!core::mem::needs_drop::<CloseSpan>());
const _: () = assert!(!core::mem::needs_drop::<Fault>());
const _: () = assert!(core::mem::size_of::<ChildPlan>() > 0);

/// `ChildPlan` is `Copy`, proven by use rather than by a comment.
const fn assert_child_plan_is_copy<T: Copy>() {}
const _: () = assert_child_plan_is_copy::<ChildPlan>();
const _: () = assert_child_plan_is_copy::<CloseSpan>();
const _: () = assert_child_plan_is_copy::<Fault>();

/// The exec-status record is exactly eight bytes, below `PIPE_BUF`, so the
/// child's single write is atomic.
pub(crate) const STATUS_RECORD_BYTES: usize = 8;
const _: () = assert!(STATUS_RECORD_BYTES == 8);

// ------------------------------------------------------------------- errors

/// Which preparation step failed. Crate-private: P3 adds no public error
/// vocabulary, because the public boundary that would need one is P4's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrepareStep {
    /// Creating one of the four close-on-exec pipes.
    Pipe,
    /// Relocating a child-side descriptor to at least 3.
    Relocate,
    /// Making a parent read end non-blocking.
    NonBlocking,
}

/// Why a crate-private launch could not proceed.
///
/// Every variant except the two `SignalMask*` ones is raised **before any child
/// exists**. `SignalMaskRestoreFailed` is raised after one does, and dropping
/// the returned error's sibling handle has already killed and reaped it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackendError {
    /// A preparation step failed; no child exists.
    Preparation {
        /// The step that failed.
        step: PrepareStep,
        /// Raw error number.
        errno: i32,
    },
    /// `clone3` is not available here — `ENOSYS`, or a policy that hides it.
    ProcessCreationUnavailable {
        /// Raw error number.
        errno: i32,
    },
    /// `clone3` failed for another reason; no child exists.
    ProcessCreationFailed {
        /// Raw error number.
        errno: i32,
    },
    /// The calling thread's signal mask could not be replaced; no child exists.
    SignalMaskFailed {
        /// Raw error number.
        errno: i32,
    },
    /// The calling thread's signal mask could not be restored. The child was
    /// killed and reaped within the fixed bound before this was returned.
    SignalMaskRestoreFailed {
        /// Raw error number.
        errno: i32,
    },
    /// Signalling the direct child through its pidfd failed.
    SignalFailed {
        /// Raw error number.
        errno: i32,
    },
    /// `clone3` reported success without providing a pidfd.
    PidfdNotProvided,
    /// A state this crate constructs was not what it constructed.
    InternalInvariant(&'static str),
}

// ----------------------------------------------------------- launch_minimal

/// The crate-private facts one minimal launch established.
///
/// This is **not** a receipt and **not** a `LaunchOutcome`: it has no
/// serialisation, no digest, no verdict, no stream facts and no public
/// existence. It exists so P3's own tests can observe the backend.
pub(crate) struct MinimalLaunch {
    /// What the exec-status channel established. There is no success value.
    pub(crate) exec_status: ExecStatus,
    /// Whether the parent's own `setpgid(child, child)` succeeded. Recorded
    /// for a future P4 sweep; nothing in P3 consumes it.
    pub(crate) group_authority_established: bool,
    /// Whether P3 sent the direct child one `SIGKILL` through its pidfd.
    pub(crate) sigkill_sent: bool,
    /// The direct child. Dropping it kills and reaps within the fixed bound.
    pub(crate) child: ChildHandle,
    /// The parent's stdout read end, non-blocking. P3 implements no drain
    /// policy; tests read bounded fixture reports through it.
    pub(crate) stdout_read: OwnedFd,
    /// The parent's stderr read end, non-blocking.
    pub(crate) stderr_read: OwnedFd,
    /// The stdin write end, retained only by the test-only stall injection.
    pub(crate) stdin_write: Option<OwnedFd>,
    /// The P2 measurement, carried through unchanged for a future P4 receipt.
    pub(crate) measurement: ExecutableMeasurement,
    /// The plan identity, carried through unchanged.
    pub(crate) plan_sha256: Digest,
}

/// Consume an authorisation, create one direct child, and observe only the
/// exec-status channel.
///
/// **Crate-private and non-public.** It is not `launch`: it emits no receipt,
/// runs no observation loop, executes no plan timeout, sends no `SIGTERM`,
/// drains no stream and issues no group signal.
pub(crate) fn launch_minimal(authorized: AuthorizedLaunch) -> Result<MinimalLaunch, BackendError> {
    launch_with(authorized, Fault::default())
}

/// [`launch_minimal`] with a test-only fault request.
#[cfg(all(feature = "test-fault-injection", debug_assertions))]
pub(crate) fn launch_minimal_with_fault(
    authorized: AuthorizedLaunch,
    fault: Fault,
) -> Result<MinimalLaunch, BackendError> {
    launch_with(authorized, fault)
}

fn launch_with(authorized: AuthorizedLaunch, fault: Fault) -> Result<MinimalLaunch, BackendError> {
    let prepared = spawn::prepare(authorized)?;
    let measurement = prepared.measurement();
    let plan_sha256 = prepared.plan().sha256();

    // The stall injection is the one case in which the parent keeps the stdin
    // write end, so that the child's blocking read on 0 cannot reach
    // end-of-file. Outside an injection build this is a constant `false`.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    let retain_stdin_writer = fault.mode == injection::MODE_STALL_BEFORE_EXEC;
    #[cfg(not(all(feature = "test-fault-injection", debug_assertions)))]
    let retain_stdin_writer = false;

    let spawn::SpawnedChild {
        child,
        group_authority_established,
    } = spawn::spawn(&prepared, fault)?;

    // A child exists from here on. `child` owns it: every return path below
    // either reaps it or drops the handle, which kills and reaps it.
    let ends = prepared.release_child_side(retain_stdin_writer);

    let observation = observe_exec_status(&ends.status_read);
    let mut sigkill_sent = false;
    if observation.timed_out {
        // The fixed pre-exec bound passed with neither a record nor
        // end-of-file (S6). One `SIGKILL` through the pidfd, then the bounded,
        // non-blocking reap. No `SIGTERM`, no grace, no group signal.
        if child.send_sigkill().is_ok() {
            sigkill_sent = true;
        }
        let _ = child.reap_within(Duration::from_millis(POST_KILL_REAP_MS));
    }

    Ok(MinimalLaunch {
        exec_status: observation.status,
        group_authority_established,
        sigkill_sent,
        child,
        stdout_read: ends.stdout_read,
        stderr_read: ends.stderr_read,
        stdin_write: ends.stdin_write,
        measurement,
        plan_sha256,
    })
}

/// What the exec-status channel produced within the fixed bound.
struct StatusObservation {
    status: ExecStatus,
    timed_out: bool,
}

/// Read the exec-status channel until a record, end-of-file, a read error or
/// the fixed pre-exec bound.
///
/// **Clean end-of-file with no record is never exec-success evidence**: it is
/// exactly what a child killed after its last setup stage shows (S5). There is
/// no `ExecSucceeded` value to produce.
fn observe_exec_status(status_read: &OwnedFd) -> StatusObservation {
    // Nine bytes, so a record longer than the fixed eight is detectable rather
    // than silently truncated.
    let mut buffer = [0_u8; 9];
    let mut filled: usize = 0;
    let deadline = Instant::now().checked_add(Duration::from_millis(SPAWN_CONFIRM_TIMEOUT_MS));

    loop {
        let (_, rest) = buffer.split_at_mut(filled);
        if rest.is_empty() {
            // More than the fixed record length arrived.
            return StatusObservation {
                status: ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed),
                timed_out: false,
            };
        }
        match rustix::io::read(status_read, &mut *rest) {
            // End-of-file: the write end is gone in every copy.
            Ok(0) => {
                return StatusObservation {
                    status: classify_record(buffer, filled),
                    timed_out: false,
                };
            }
            Ok(read) => filled = filled.saturating_add(read),
            Err(rustix::io::Errno::INTR) => {}
            // On Linux `EWOULDBLOCK` and `EAGAIN` are the same number, so this
            // one arm is the whole "nothing yet" case.
            Err(rustix::io::Errno::AGAIN) => match deadline {
                Some(deadline) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                }
                _ => {
                    return StatusObservation {
                        status: ExecStatus::Indeterminate(
                            IndeterminateReason::PreExecStatusTimeout,
                        ),
                        timed_out: true,
                    };
                }
            },
            // A read error is never treated as end-of-file (T41).
            Err(errno) => {
                return StatusObservation {
                    status: ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed {
                        errno: errno.raw_os_error(),
                    }),
                    timed_out: false,
                };
            }
        }
    }
}

/// Classify what the exec-status channel delivered before end-of-file.
fn classify_record(buffer: [u8; 9], filled: usize) -> ExecStatus {
    match (filled, buffer) {
        // Every ordinary run, and S5: nothing was written and the write end
        // closed. Nothing more can be said about the execution attempt.
        (0, _) => ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
        (STATUS_RECORD_BYTES, [code, 0, 0, 0, e0, e1, e2, e3, _]) => match stage_of(code) {
            Some(stage) => ExecStatus::PreExecFailure {
                stage,
                errno: i32::from_le_bytes([e0, e1, e2, e3]),
            },
            None => ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed),
        },
        // A short record, or eight bytes whose reserved pad is not zero.
        _ => ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed),
    }
}
