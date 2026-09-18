//! Parent-side preparation, the `clone3` window and bounded direct-child
//! cleanup (**P3**, plan sections 8.2 and 8.3).
//!
//! # Division of labour
//!
//! Everything that can fail, allocate or block happens **before** a child
//! exists. [`prepare`] turns a consumed `AuthorizedLaunch` into a
//! [`PreparedLaunch`] holding owned descriptors, the argv storage and the pure
//! descriptor-layout plan; a failure there creates **no child**. [`spawn`] then
//! does only what must happen inside the clone window, and the only raw
//! operations in this file are the two `rt_sigprocmask` calls and `clone3`
//! itself. Every other operating-system call goes through a safe `rustix`
//! wrapper: pipe creation, descriptor relocation, status-flag changes,
//! `setpgid`, reads, pidfd signalling and `waitid`.
//!
//! # What is deliberately absent
//!
//! No observation loop, no `poll`, no plan-driven run deadline, no `SIGTERM`,
//! no grace period, no stream drain policy, no receipt and **no process-group
//! sweep**. The parent's `setpgid(child, child)` result is recorded as a
//! boolean and used for nothing: no negative-pid signal exists anywhere in this
//! crate. Those belong to a P4 slice that is not authorised.

use std::cell::Cell;
use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::time::{Duration, Instant};

use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use rustix::io::fcntl_dupfd_cloexec;
use rustix::pipe::{PipeFlags, pipe_with};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, pidfd_send_signal, setpgid, waitid};

use super::{
    BackendError, ChildPlan, CloseSpan, Fault, POLL_INTERVAL_MS, POST_KILL_REAP_MS, PrepareStep,
    child, syscall,
};
use crate::authority::{AuthorizedLaunch, AuthorizedParts};
use crate::layout::{self, Descriptors, FIRST_RELOCATED};
use crate::model::{ChildEnd, ExecutableMeasurement};
use crate::plan::ValidatedLaunchPlan;

/// The lowest descriptor number every child-side descriptor is relocated to.
const FIRST_RELOCATED_RAW: RawFd = 3;
const _: () = assert!(FIRST_RELOCATED == 3);

// ------------------------------------------------------------- preparation

/// Everything the parent built before any child existed.
///
/// The six child-side descriptors are relocated to at least 3 and are
/// close-on-exec; the four parent-side ends are not relocated, because the
/// child's range close removes them from its own table whatever their numbers.
/// Dropping this value closes every descriptor it still owns, which is what
/// makes every early-return path before `clone3` leak-free.
pub(super) struct PreparedLaunch {
    executable: OwnedFd,
    working_directory: OwnedFd,
    stdin_read: OwnedFd,
    stdout_write: OwnedFd,
    stderr_write: OwnedFd,
    status_write: OwnedFd,
    stdin_write: OwnedFd,
    stdout_read: OwnedFd,
    stderr_read: OwnedFd,
    status_read: OwnedFd,
    /// Owns the argv bytes the child's `execveat` reads. Moving this value
    /// moves only the vector headers, never the heap buffers the pointers name.
    argv_storage: Vec<CString>,
    /// The null-terminated `argv` pointer array itself.
    argv_pointers: Vec<*const u8>,
    close_ranges: [CloseSpan; 3],
    measurement: ExecutableMeasurement,
    plan: ValidatedLaunchPlan,
}

/// The parent-retained pipe ends, after the child-side copies are closed.
pub(super) struct ParentEnds {
    /// Kept open only by the test-only pre-exec stall injection; otherwise
    /// already closed, so the child reads immediate end-of-file on 0 (T22).
    pub(super) stdin_write: Option<OwnedFd>,
    pub(super) stdout_read: OwnedFd,
    pub(super) stderr_read: OwnedFd,
    pub(super) status_read: OwnedFd,
}

impl PreparedLaunch {
    /// The pre-execution measurement `admit_executable` recorded, carried
    /// through unchanged. Nothing here re-measures or reopens the object.
    pub(super) const fn measurement(&self) -> ExecutableMeasurement {
        self.measurement
    }

    /// The validated plan this authorisation owns.
    pub(super) const fn plan(&self) -> &ValidatedLaunchPlan {
        &self.plan
    }

    /// Close the parent's copies of the six child-side descriptors, then the
    /// stdin write end, and hand back what the parent keeps (plan section 8.3
    /// step 4). Every pipe can now reach end-of-file.
    pub(super) fn release_child_side(self, retain_stdin_writer: bool) -> ParentEnds {
        let Self {
            executable,
            working_directory,
            stdin_read,
            stdout_write,
            stderr_write,
            status_write,
            stdin_write,
            stdout_read,
            stderr_read,
            status_read,
            argv_storage,
            argv_pointers,
            close_ranges: _,
            measurement: _,
            plan: _,
        } = self;
        drop(executable);
        drop(working_directory);
        drop(stdin_read);
        drop(stdout_write);
        drop(stderr_write);
        drop(status_write);
        // The child has its own copy of the address space, so the parent's argv
        // storage is no longer read by anything.
        drop(argv_pointers);
        drop(argv_storage);
        ParentEnds {
            stdin_write: if retain_stdin_writer {
                Some(stdin_write)
            } else {
                drop(stdin_write);
                None
            },
            stdout_read,
            stderr_read,
            status_read,
        }
    }
}

/// Consume an authorisation and build everything the child will need.
///
/// Plan section 8.2, in order: argv storage and the pointer arrays; four
/// close-on-exec pipes; unconditional relocation of all six child-side
/// descriptors to at least 3 with close-on-exec (T19); the pure close-range
/// plan; non-blocking parent read ends; and the child plan itself, which
/// [`spawn`] fills once the prepared value has stopped moving.
///
/// **A failure here creates no child**, and every descriptor opened so far
/// closes by `OwnedFd` drop.
pub(super) fn prepare(authorized: AuthorizedLaunch) -> Result<PreparedLaunch, BackendError> {
    let AuthorizedParts {
        plan,
        measurement,
        executable,
        working_directory,
    } = authorized.into_parts();

    // 1. argv. Plan validation already refused a NUL in any element, including
    //    the JSON escape, so the conversion cannot fail; if it ever did, that
    //    would be an internal invariant violation and not a caller error.
    let mut argv_storage: Vec<CString> = Vec::with_capacity(plan.argv().len());
    for argument in plan.argv() {
        let owned = CString::new(argument.as_bytes())
            .map_err(|_| BackendError::InternalInvariant("a validated argv element held a NUL"))?;
        argv_storage.push(owned);
    }
    let mut argv_pointers: Vec<*const u8> = argv_storage
        .iter()
        .map(|argument| argument.as_ptr().cast::<u8>())
        .collect();
    argv_pointers.push(std::ptr::null());

    // 2. Four close-on-exec pipes: stdin, stdout, stderr, exec status.
    let (stdin_read, stdin_write) = new_pipe()?;
    let (stdout_read, stdout_write) = new_pipe()?;
    let (stderr_read, stderr_write) = new_pipe()?;
    let (status_read, status_write) = new_pipe()?;

    // 3. Unconditional relocation to at least 3, close-on-exec, in the accepted
    //    order. Each duplicate is taken while its original is still open, so no
    //    relocated number can equal its own original, and no relocated number
    //    is ever closed, so the six are distinct.
    let executable = relocate(executable)?;
    let working_directory = relocate(working_directory)?;
    let stdin_read = relocate(stdin_read)?;
    let stdout_write = relocate(stdout_write)?;
    let stderr_write = relocate(stderr_write)?;
    let status_write = relocate(status_write)?;

    // 4. The pure close-range plan over the relocated numbers.
    let descriptors = Descriptors {
        executable: layout_number(executable.as_fd())?,
        working_directory: layout_number(working_directory.as_fd())?,
        stdin_read: layout_number(stdin_read.as_fd())?,
        stdout_write: layout_number(stdout_write.as_fd())?,
        stderr_write: layout_number(stderr_write.as_fd())?,
        status_write: layout_number(status_write.as_fd())?,
    };
    let child_layout = layout::plan_child_layout(&descriptors).map_err(|_| {
        BackendError::InternalInvariant("the relocated descriptors were not a valid layout")
    })?;
    let mut close_ranges = [CloseSpan::UNUSED; 3];
    for (slot, range) in close_ranges
        .iter_mut()
        .zip(child_layout.close_ranges.iter())
    {
        *slot = CloseSpan {
            first: usize::try_from(range.first).map_err(|_| {
                BackendError::InternalInvariant("a close range exceeded the host width")
            })?,
            last: usize::try_from(range.last).map_err(|_| {
                BackendError::InternalInvariant("a close range exceeded the host width")
            })?,
            used: 1,
        };
    }

    // 5. Non-blocking parent read ends. The fixed pre-exec bound reads the
    //    status channel without blocking; the stream ends are handed to the
    //    caller in the same state.
    set_non_blocking(stdout_read.as_fd())?;
    set_non_blocking(stderr_read.as_fd())?;
    set_non_blocking(status_read.as_fd())?;

    Ok(PreparedLaunch {
        executable,
        working_directory,
        stdin_read,
        stdout_write,
        stderr_write,
        status_write,
        stdin_write,
        stdout_read,
        stderr_read,
        status_read,
        argv_storage,
        argv_pointers,
        close_ranges,
        measurement,
        plan,
    })
}

fn new_pipe() -> Result<(OwnedFd, OwnedFd), BackendError> {
    pipe_with(PipeFlags::CLOEXEC).map_err(|errno| BackendError::Preparation {
        step: PrepareStep::Pipe,
        errno: errno.raw_os_error(),
    })
}

/// Duplicate one descriptor close-on-exec at or above 3 and close the original.
///
/// T19: there is no "already high enough" shortcut. A caller-supplied
/// descriptor of unknown close-on-exec state must never reach the executed
/// image as itself, and no `dup2` source may equal its target later.
fn relocate(original: OwnedFd) -> Result<OwnedFd, BackendError> {
    let relocated = fcntl_dupfd_cloexec(&original, FIRST_RELOCATED_RAW).map_err(|errno| {
        BackendError::Preparation {
            step: PrepareStep::Relocate,
            errno: errno.raw_os_error(),
        }
    })?;
    // The duplicate was taken while `original` was still open, so the two
    // numbers differ; the original closes here.
    drop(original);
    Ok(relocated)
}

fn set_non_blocking(fd: BorrowedFd<'_>) -> Result<(), BackendError> {
    let flags = fcntl_getfl(fd).map_err(|errno| BackendError::Preparation {
        step: PrepareStep::NonBlocking,
        errno: errno.raw_os_error(),
    })?;
    fcntl_setfl(fd, flags | OFlags::NONBLOCK).map_err(|errno| BackendError::Preparation {
        step: PrepareStep::NonBlocking,
        errno: errno.raw_os_error(),
    })
}

fn layout_number(fd: BorrowedFd<'_>) -> Result<u32, BackendError> {
    u32::try_from(fd.as_raw_fd())
        .map_err(|_| BackendError::InternalInvariant("a descriptor number was negative"))
}

fn plan_number(fd: BorrowedFd<'_>) -> Result<usize, BackendError> {
    usize::try_from(fd.as_raw_fd())
        .map_err(|_| BackendError::InternalInvariant("a descriptor number was negative"))
}

/// The address of a value, as the kernel ABI takes it.
///
/// `expose_provenance` is the operation that says "this address leaves Rust":
/// the kernel, not Rust code, dereferences it.
fn address_of<T>(value: &T) -> usize {
    std::ptr::from_ref(value).expose_provenance()
}

fn address_of_mut<T>(value: &mut T) -> usize {
    std::ptr::from_mut(value).expose_provenance()
}

/// The address of a slice's first element, as the kernel ABI takes it.
fn address_of_slice<T>(values: &[T]) -> usize {
    values.as_ptr().expose_provenance()
}

// ------------------------------------------------------------------- spawn

/// What one successful `clone3` produced.
pub(super) struct SpawnedChild {
    /// The direct child, owned by its pidfd. Dropping it cannot leak a child.
    pub(super) child: ChildHandle,
    /// **Only** a successful parent-side `setpgid(child, child)` sets this. It
    /// is recorded for a future P4 sweep and is used for nothing in P3:
    /// no negative-pid signal exists in this crate.
    pub(super) group_authority_established: bool,
}

/// Create the direct child and attempt to execute the admitted descriptor.
///
/// The window is exactly: block every blockable signal on **this thread**;
/// `clone3(CLONE_PIDFD)`; on the child branch enter [`child::child_main`]
/// immediately; on the parent branch issue `setpgid(child, child)` as the first
/// system call; then restore the saved mask.
pub(super) fn spawn(prepared: &PreparedLaunch, fault: Fault) -> Result<SpawnedChild, BackendError> {
    // Constants the child reads through the plan. They are locals of this
    // frame, so their addresses are stable for as long as `clone3` and the
    // child's copy-on-write copy of this frame can reach them.
    let envp: [usize; 1] = [0];
    let empty_path: [u8; 1] = [0];
    let default_action = syscall::KernelSigaction::DEFAULT_DISPOSITION;
    let empty_signal_mask: u64 = syscall::EMPTY_KERNEL_SIGSET;

    let plan = ChildPlan {
        stdin_source: plan_number(prepared.stdin_read.as_fd())?,
        stdout_source: plan_number(prepared.stdout_write.as_fd())?,
        stderr_source: plan_number(prepared.stderr_write.as_fd())?,
        working_directory: plan_number(prepared.working_directory.as_fd())?,
        executable: plan_number(prepared.executable.as_fd())?,
        status_write: plan_number(prepared.status_write.as_fd())?,
        close_ranges: prepared.close_ranges,
        argv: address_of_slice(&prepared.argv_pointers),
        envp: address_of(&envp),
        empty_path: address_of(&empty_path),
        default_action: address_of(&default_action),
        empty_signal_mask: address_of(&empty_signal_mask),
        status_record: [0_u8; 8],
        fault,
    };

    // 1. Block every blockable signal on the calling thread only (T21).
    //    `SIGKILL` and `SIGSTOP` are not blockable: their bits are set in the
    //    value passed, and the kernel removes them from the mask it installs.
    let full_mask: u64 = syscall::FULL_KERNEL_SIGSET;
    let mut saved_mask: u64 = 0;
    // SAFETY: `SIG_SETMASK` replaces the mask. The second argument is the
    // address of a live `u64` kernel signal set and the third the address of a
    // live `u64` the kernel writes the previous mask into; both are locals of
    // this frame. The fourth is the kernel signal-set size, which the call
    // validates against its own `sizeof(sigset_t)`.
    let blocked = unsafe {
        syscall::syscall6(
            syscall::NR_RT_SIGPROCMASK,
            syscall::SIG_SETMASK,
            address_of(&full_mask),
            address_of_mut(&mut saved_mask),
            syscall::KERNEL_SIGSET_BYTES,
            0,
            0,
        )
    };
    if syscall::is_error(blocked) {
        // Nothing has been created; the mask was not changed.
        return Err(BackendError::SignalMaskFailed {
            errno: syscall::errno_of(blocked),
        });
    }

    // 2. The clone itself, and the child's immediate departure into its own
    //    closed sequence.
    let mut pidfd_slot: i32 = -1;
    let pidfd_address = u64::try_from(address_of_mut(&mut pidfd_slot))
        .map_err(|_| BackendError::InternalInvariant("a host address exceeded 64 bits"))?;
    let mut clone_args = syscall::CloneArgs::for_direct_child(pidfd_address);
    // SAFETY: `clone_args` is a live, fully initialised `clone_args` record in
    // the kernel's layout, and `plan` is a live, fully initialised `ChildPlan`
    // of this frame, which the child reaches through its copy-on-write copy of
    // this frame. Both outlive the call: `clone_args` is borrowed mutably for
    // its duration, and the child either replaces its image or ends before this
    // frame can return.
    let cloned = unsafe { clone_and_dispatch(&mut clone_args, &plan) };

    // Only the parent reaches this point: the child branch diverges inside
    // `clone_and_dispatch` and never returns.
    if syscall::is_error(cloned) {
        restore_mask(saved_mask);
        let errno = syscall::errno_of(cloned);
        return Err(if errno == syscall::ENOSYS || errno == syscall::EPERM {
            BackendError::ProcessCreationUnavailable { errno }
        } else {
            BackendError::ProcessCreationFailed { errno }
        });
    }

    // 3. `setpgid(child, child)` is the first system call after `clone3`.
    //    Only its success establishes group-sweep authority; an error
    //    establishes nothing, is not retried and is not interpreted further.
    //    The two conversions before it are pure arithmetic, not system calls.
    let child_pid = i32::try_from(cloned).ok().and_then(Pid::from_raw);
    let group_authority_established = match child_pid {
        Some(pid) => setpgid(Some(pid), Some(pid)).is_ok(),
        None => false,
    };

    // 4. Take ownership of the pidfd. This is bookkeeping, not a system call,
    //    so it does not come between `clone3` and `setpgid`.
    if pidfd_slot < 0 {
        // `CLONE_PIDFD` guarantees a descriptor on success, so this cannot
        // happen; if it ever did there would be no authorised way to reach the
        // child, because no pid-valued signal exists in this crate.
        restore_mask(saved_mask);
        return Err(BackendError::PidfdNotProvided);
    }
    // SAFETY: `clone3` returned success with `CLONE_PIDFD`, so the kernel wrote
    // a freshly created, close-on-exec descriptor for the new process into
    // `pidfd_slot`, and nothing else holds or will close that number. Ownership
    // moves here exactly once.
    let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd_slot) };
    let child = ChildHandle::new(pidfd, cloned);

    // 5. Restore the mask. A failure here cannot be reported as a caller
    //    contract, and it must not leak the child: dropping `child` sends one
    //    `SIGKILL` through the pidfd and reaps within the fixed bound.
    let restored = restore_mask(saved_mask);
    if syscall::is_error(restored) {
        return Err(BackendError::SignalMaskRestoreFailed {
            errno: syscall::errno_of(restored),
        });
    }

    Ok(SpawnedChild {
        child,
        group_authority_established,
    })
}

/// Issue `clone3` and, on the child branch, enter the closed child sequence
/// before any other statement.
///
/// Kept out of line and free of every value with a destructor, so that the
/// child's path from the kernel's return to [`child::child_main`] is the branch
/// and nothing else.
///
/// # Safety
///
/// `args` must be a fully initialised `clone_args` record and `plan` a fully
/// initialised [`ChildPlan`] whose descriptors and addresses are live in the
/// calling frame, as [`child::child_main`] requires.
#[inline(never)]
unsafe fn clone_and_dispatch(args: &mut syscall::CloneArgs, plan: &ChildPlan) -> i64 {
    let args_address = address_of_mut(args);
    let plan_pointer: *const ChildPlan = std::ptr::from_ref(plan);
    // SAFETY: the second argument is the size of the record the first argument
    // points at, which is the `clone3` ABI's own versioning contract; the
    // record is live for the call. The flag word is exactly `CLONE_PIDFD`, so
    // the child gets its own address space, descriptor table and thread group.
    let result = unsafe {
        syscall::syscall6(
            syscall::NR_CLONE3,
            args_address,
            syscall::CLONE_ARGS_SIZE_VER2,
            0,
            0,
            0,
            0,
        )
    };
    if result == 0 {
        // SAFETY: a zero return from `clone3` is the child. `plan_pointer`
        // names this frame's `ChildPlan`, which the child reaches in its own
        // copy-on-write copy of the frame. `child_main` never returns, so no
        // value of any parent frame is dropped in the child.
        unsafe { child::child_main(plan_pointer) }
    }
    result
}

/// Restore a previously saved kernel signal mask on the calling thread.
fn restore_mask(saved: u64) -> i64 {
    // SAFETY: `SIG_SETMASK` replaces the mask with the set the second argument
    // points at, which is the address of a live `u64` of this frame. The third
    // argument is a null old-mask pointer, and the fourth is the kernel
    // signal-set size the call validates.
    unsafe {
        syscall::syscall6(
            syscall::NR_RT_SIGPROCMASK,
            syscall::SIG_SETMASK,
            address_of(&saved),
            0,
            syscall::KERNEL_SIGSET_BYTES,
            0,
            0,
        )
    }
}

// ----------------------------------------------------------- child handle

/// The direct child, addressed **only** by its pidfd.
///
/// No operation here takes a pid: the pid is carried for tracing and for the
/// group identity a future P4 slice would need, and is never the target of a
/// signal or a wait. Dropping the handle cannot leak a child.
pub(crate) struct ChildHandle {
    pidfd: OwnedFd,
    pid: i64,
    end: Cell<Option<ChildEnd>>,
    reaped: Cell<bool>,
}

impl ChildHandle {
    fn new(pidfd: OwnedFd, pid: i64) -> Self {
        Self {
            pidfd,
            pid,
            end: Cell::new(None),
            reaped: Cell::new(false),
        }
    }

    /// The direct child's process id. Recorded, never signalled.
    pub(crate) const fn pid(&self) -> i64 {
        self.pid
    }

    /// Send one `SIGKILL` through the pidfd.
    ///
    /// The only two P3 callers are the fixed pre-exec timeout and bounded
    /// cleanup that must not leak a child. There is no `SIGTERM`, no grace
    /// period and no group signal.
    pub(crate) fn send_sigkill(&self) -> Result<(), BackendError> {
        pidfd_send_signal(self.pidfd.as_fd(), Signal::KILL).map_err(|errno| {
            BackendError::SignalFailed {
                errno: errno.raw_os_error(),
            }
        })
    }

    /// One non-blocking `waitid(P_PIDFD, WEXITED | WNOHANG)`.
    ///
    /// Never blocks, never waits by pid, and never consumes a state change it
    /// does not classify.
    pub(crate) fn reap_once(&self) -> Option<ChildEnd> {
        if let Some(end) = self.end.get() {
            return Some(end);
        }
        let status = waitid(
            WaitId::PidFd(self.pidfd.as_fd()),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
        );
        match status {
            Ok(Some(status)) => {
                self.reaped.set(true);
                let end = classify(&status);
                self.end.set(Some(end));
                Some(end)
            }
            // No state change yet.
            Ok(None) => None,
            // An interrupted non-blocking wait established nothing, and is not
            // an end. The bounded caller retries it.
            Err(rustix::io::Errno::INTR) => None,
            // `ECHILD` and anything else: the child was reaped elsewhere, or
            // the end cannot be classified here. Latched, so a later drop does
            // not signal a process this handle no longer owns.
            Err(_) => {
                self.reaped.set(true);
                self.end.set(Some(ChildEnd::EndUnobservable));
                Some(ChildEnd::EndUnobservable)
            }
        }
    }

    /// Reap within a fixed bound, polling without ever blocking.
    ///
    /// Returns `None` when the bound passes with no end observed. That is not a
    /// claim that the child is still running, nor that it never ended.
    pub(crate) fn reap_within(&self, bound: Duration) -> Option<ChildEnd> {
        let deadline = Instant::now().checked_add(bound);
        loop {
            if let Some(end) = self.reap_once() {
                return Some(end);
            }
            match deadline {
                Some(deadline) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                }
                _ => return None,
            }
        }
    }

    /// The end this handle has already observed, if any.
    pub(crate) fn observed_end(&self) -> Option<ChildEnd> {
        self.end.get()
    }
}

impl Drop for ChildHandle {
    /// Bounded cleanup, so that no P3 code path can leave a child behind.
    ///
    /// This is **P3 cleanup**, not a lifecycle policy: one `SIGKILL` through
    /// the pidfd and a bounded, non-blocking reap. There is no `SIGTERM`, no
    /// grace period, no group signal and no run deadline anywhere in this
    /// crate. A child already reaped is left alone.
    fn drop(&mut self) {
        if self.reaped.get() {
            return;
        }
        let _ = self.send_sigkill();
        let _ = self.reap_within(Duration::from_millis(POST_KILL_REAP_MS));
    }
}

/// Classify one `waitid` result into the accepted child-end vocabulary.
///
/// A signal number is not a sender: no cause is claimed (S-13).
fn classify(status: &rustix::process::WaitIdStatus) -> ChildEnd {
    if status.exited() {
        return ChildEnd::Exited {
            code: status.exit_status().unwrap_or(0),
        };
    }
    if status.dumped() {
        return match status.terminating_signal() {
            Some(signal) => ChildEnd::Signaled {
                signal,
                core_dumped: true,
            },
            None => ChildEnd::EndUnobservable,
        };
    }
    if status.killed() {
        return match status.terminating_signal() {
            Some(signal) => ChildEnd::Signaled {
                signal,
                core_dumped: false,
            },
            None => ChildEnd::EndUnobservable,
        };
    }
    ChildEnd::EndUnobservable
}
