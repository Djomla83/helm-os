//! The closed post-clone child contract (**P3**, plan section 8.4).
//!
//! # The window this file describes
//!
//! Everything here runs between the `clone3` return in the child and a
//! successful `execveat`. In that window the child is a single thread in a
//! copy-on-write copy of the parent's address space, created **without**
//! `CLONE_VM`, `CLONE_VFORK`, `CLONE_FILES` and `CLONE_THREAD`, holding
//! whatever locks the parent's other threads happened to hold at the clone. The
//! contract that keeps that state harmless is:
//!
//! * **Nothing is computed here.** Every descriptor number, pointer, range and
//!   record byte was built by the parent before the child existed and travels
//!   in one plain-old-data [`ChildPlan`], which the child copies once.
//! * **No allocation, no lock, no formatting, no printing, no panic, no
//!   unwinding, no destructor.** This module has `#![no_implicit_prelude]`, and
//!   `tests/p3_boundary.rs` fails the build on `std`, `alloc`, `rustix`, `Vec`,
//!   `String`, `Box`, `format`, `print`, `panic`, `unwrap` or `expect`
//!   appearing in it as code.
//! * **Every system call goes through the one raw shim** of
//!   [`super::syscall`], never through glibc, `rustix` or `std`.
//! * **There are exactly two exits**: a successful `execveat`, which replaces
//!   the image, or `exit_group(127)` after one 8-byte failure record. The entry
//!   point returns `!`, so no parent destructor can run in the child and no
//!   unwinding can begin.
//!
//! # The sequence
//!
//! `DUP2`, `CLEAR_CLOEXEC`, `CHDIR`, `CLOSE_RANGE`, `SETPGID`, `SIGACTION`,
//! `SIGMASK`, `NO_NEW_PRIVS`, `EXEC` — in that order, with no configuration
//! branch. `CHDIR` precedes `CLOSE_RANGE` because the working-directory
//! descriptor is not preserved by the range close. `SIGACTION` precedes
//! `SIGMASK` because the parent blocked every blockable signal across `clone3`,
//! so no host disposition can run in the child before the reset; unblocking is
//! therefore the last signal-related step, and by then every disposition is
//! already the default.

#![no_implicit_prelude]

use super::syscall;
use super::{ChildPlan, CloseSpan, stage};

/// The post-clone child entry point. It never returns and never unwinds.
///
/// Called directly on the `clone3 == 0` branch, before any other statement of
/// the cloning frame, so that no value of the parent's frame is dropped in the
/// child.
///
/// # Safety
///
/// The caller must be the child returned by a successful `clone3`, on the
/// `== 0` branch, with `plan` pointing at a fully initialised [`ChildPlan`]
/// that lives in the parent frame which issued the clone. Every descriptor
/// number in that plan must name a descriptor the parent opened and did not
/// close before the clone, every address in it must be an address of parent
/// memory that is live at the clone, and the close ranges must preserve the
/// executable and the exec-status write end. The caller must not rely on this
/// function returning: it replaces the image or ends the process.
pub(super) unsafe fn child_main(plan: *const ChildPlan) -> ! {
    // SAFETY: `plan` is the address of the caller's `ChildPlan`, taken in the
    // frame that issued `clone3` and therefore live in this child's
    // copy-on-write copy of that frame. `ChildPlan` is `Copy` plain-old data
    // with no padding invariants and no destructor, so this reads one value out
    // and leaves nothing to drop.
    let plan: ChildPlan = unsafe { *plan };

    // ---------------------------------------------------------- 1. DUP2
    //
    // The final stdio mapping. Every source was relocated to at least 3 in the
    // parent (T19), so no source can equal its target and no `dup2` can be the
    // silent no-op that keeps close-on-exec set.

    // SAFETY: `stdin_source` is the parent's relocated read end of the stdin
    // pipe, open in this child's descriptor table, and 0 is a valid target
    // number. `dup2` takes two descriptor numbers and no pointer.
    unsafe {
        issue(
            &plan,
            stage::DUP2,
            syscall::NR_DUP2,
            plan.stdin_source,
            0,
            0,
            0,
            0,
            0,
        )
    };
    // SAFETY: as above, for the relocated stdout write end onto 1.
    unsafe {
        issue(
            &plan,
            stage::DUP2,
            syscall::NR_DUP2,
            plan.stdout_source,
            1,
            0,
            0,
            0,
            0,
        )
    };
    // SAFETY: as above, for the relocated stderr write end onto 2.
    unsafe {
        issue(
            &plan,
            stage::DUP2,
            syscall::NR_DUP2,
            plan.stderr_source,
            2,
            0,
            0,
            0,
            0,
        )
    };

    // -------------------------------------------------- 2. CLEAR_CLOEXEC
    //
    // A backstop. `dup2` to a different number already clears the flag, and the
    // unconditional relocation makes the same-number case unreachable; the
    // explicit clear means the executed image keeps 0, 1 and 2 even if it ever
    // became reachable.

    // SAFETY: 0 is open here, `F_SETFD` is a descriptor-flag command, and its
    // third argument is a flag word, not a pointer.
    unsafe {
        issue(
            &plan,
            stage::CLEAR_CLOEXEC,
            syscall::NR_FCNTL,
            0,
            syscall::F_SETFD,
            0,
            0,
            0,
            0,
        )
    };
    // SAFETY: as above, for 1.
    unsafe {
        issue(
            &plan,
            stage::CLEAR_CLOEXEC,
            syscall::NR_FCNTL,
            1,
            syscall::F_SETFD,
            0,
            0,
            0,
            0,
        )
    };
    // SAFETY: as above, for 2.
    unsafe {
        issue(
            &plan,
            stage::CLEAR_CLOEXEC,
            syscall::NR_FCNTL,
            2,
            syscall::F_SETFD,
            0,
            0,
            0,
            0,
        )
    };

    // --------------------------------------------------------- 3. CHDIR
    //
    // Before the range close, which does not preserve this descriptor.

    // SAFETY: `working_directory` is the parent's relocated copy of the
    // descriptor the caller moved into `admit_working_directory`, open here and
    // known to be a directory. `fchdir` takes one descriptor number.
    unsafe {
        issue(
            &plan,
            stage::CHDIR,
            syscall::NR_FCHDIR,
            plan.working_directory,
            0,
            0,
            0,
            0,
            0,
        )
    };

    // --------------------------------------------------- 4. CLOSE_RANGE
    //
    // Everything at or above 3 except the executable and the exec-status write
    // end. The ranges are the pure `layout.rs` plan, are ascending, are never
    // inverted, and never contain a preserved number.
    let [first, second, third] = plan.close_ranges;
    // SAFETY: a close range is two descriptor numbers and a zero flag word; no
    // pointer is involved, and `close_span` skips an unused span.
    unsafe { close_span(&plan, first) };
    // SAFETY: as above, for the second span.
    unsafe { close_span(&plan, second) };
    // SAFETY: as above, for the third span.
    unsafe { close_span(&plan, third) };

    // ------------------------------------------------------- 5. SETPGID
    //
    // The executed image leads its own process group whichever of this call and
    // the parent's runs first. **Only the parent's own successful
    // `setpgid(child, child)` establishes group-sweep authority**; nothing is
    // inferred from this one, and P3 issues no group signal at all.

    // SAFETY: `setpgid(0, 0)` names this process twice by the kernel's
    // "caller" encoding and takes no pointer.
    unsafe { issue(&plan, stage::SETPGID, syscall::NR_SETPGID, 0, 0, 0, 0, 0, 0) };

    // ----------------------------------------------------- 6. SIGACTION
    //
    // Every signal 1 ..= 64 except `SIGKILL` and `SIGSTOP`, which the kernel
    // refuses to let anyone redirect. Delivery is still fully blocked, so no
    // host handler can run between the clone and this reset. The raw call is
    // what makes signals 32 and 33 reachable at all: glibc's wrapper refuses
    // its own internal real-time signals.
    let mut signal: usize = 1;
    while signal <= syscall::NSIG {
        if signal != syscall::SIGKILL && signal != syscall::SIGSTOP {
            // SAFETY: `signal` is in 1 ..= 64 and is neither `SIGKILL` nor
            // `SIGSTOP`, so the kernel accepts it. `default_action` is the
            // address of the parent's `KernelSigaction`, live in this child's
            // copy of the parent frame, in the kernel's x86_64 layout — handler
            // first, mask last — and the kernel reads exactly
            // `KERNEL_SIGACTION_BYTES` from it. The third argument is a null
            // old-action pointer, and the fourth is the kernel signal-set size,
            // which the call validates.
            unsafe {
                issue(
                    &plan,
                    stage::SIGACTION,
                    syscall::NR_RT_SIGACTION,
                    signal,
                    plan.default_action,
                    0,
                    syscall::KERNEL_SIGSET_BYTES,
                    0,
                    0,
                )
            };
        }
        signal = signal.wrapping_add(1);
    }

    // -------------------------------------------------------- 7. SIGMASK
    //
    // The intended final mask of the executed image. Only now can a blockable
    // signal arrive, and every disposition is already the default.

    // SAFETY: `SIG_SETMASK` replaces the mask. `empty_signal_mask` is the
    // address of the parent's zeroed `u64` kernel signal set, live in this
    // child's copy of the parent frame; the kernel reads exactly
    // `KERNEL_SIGSET_BYTES` from it. The third argument is a null old-mask
    // pointer.
    unsafe {
        issue(
            &plan,
            stage::SIGMASK,
            syscall::NR_RT_SIGPROCMASK,
            syscall::SIG_SETMASK,
            plan.empty_signal_mask,
            0,
            syscall::KERNEL_SIGSET_BYTES,
            0,
            0,
        )
    };

    // --------------------------------------------------- 8. NO_NEW_PRIVS
    //
    // D-11. Set, never claimed to have been exercised: no privilege transition
    // is attempted anywhere in this crate.

    // SAFETY: `PR_SET_NO_NEW_PRIVS` takes the value 1 and three zero arguments,
    // none of which the kernel reads as a pointer.
    unsafe {
        issue(
            &plan,
            stage::NO_NEW_PRIVS,
            syscall::NR_PRCTL,
            syscall::PR_SET_NO_NEW_PRIVS,
            1,
            0,
            0,
            0,
            0,
        )
    };

    // ---------------------------------------------------------- 9. EXEC
    //
    // The exact descriptor the caller admitted, by `AT_EMPTY_PATH`. No
    // pathname is resolved, no `/proc/self/fd` is read, and no `PATH` is
    // searched, here or anywhere in this crate.

    // The test-only injections live immediately before the execution attempt,
    // and only in a build that both enables the feature and has debug
    // assertions on. In every default and release build this line does not
    // exist.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    // SAFETY: the injection reads only the plan's own fault record and, in the
    // stall mode, descriptor 0, which stage 1 bound to the stdin pipe. It
    // either returns or ends this process, and it is compiled only into a
    // debug build with the non-default feature enabled.
    unsafe {
        super::injection::before_exec(&plan)
    };

    // SAFETY: `executable` is the parent's relocated copy of the admitted
    // executable descriptor, preserved by the close ranges and open here.
    // `empty_path` is the address of the parent's one-byte `""`, and
    // `argv`/`envp` are the addresses of the parent's null-terminated pointer
    // arrays, all live in this child's copy of the parent frame and all
    // unchanged since before the clone. `AT_EMPTY_PATH` is what makes the
    // descriptor itself the executed object.
    unsafe {
        issue(
            &plan,
            stage::EXEC,
            syscall::NR_EXECVEAT,
            plan.executable,
            plan.empty_path,
            plan.argv,
            plan.envp,
            syscall::AT_EMPTY_PATH,
            0,
        )
    };

    // `execveat` returned, which for this call means it failed and `issue`
    // already ended the process. The loop is unreachable and exists only to
    // give the function its `!` type without a panic.
    loop {
        // SAFETY: `exit_group` ends every thread of this process and takes no
        // pointer.
        unsafe {
            syscall::syscall6(
                syscall::NR_EXIT_GROUP,
                syscall::CHILD_FAILURE_EXIT,
                0,
                0,
                0,
                0,
                0,
            )
        };
    }
}

/// Issue one child-window system call, ending the process through [`fail`] if
/// the kernel reports an error.
///
/// This is the only place in the child that reaches the syscall shim, so the
/// permitted syscall set of plan section 8.4 is the set of numbers this
/// function is called with.
///
/// # Safety
///
/// The caller must satisfy [`syscall::syscall6`]'s contract for `nr` and its
/// arguments, and must be inside the child window, where ending the process is
/// the correct response to a failure.
#[inline(always)]
#[expect(
    clippy::too_many_arguments,
    reason = "the six syscall argument registers are the ABI; naming them individually is what makes each call site auditable"
)]
unsafe fn issue(
    plan: &ChildPlan,
    stage_code: u8,
    nr: i64,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
) -> i64 {
    // Test-only: report this stage as having failed with a chosen `errno`,
    // without issuing its system call, so that stages whose real failure cannot
    // be provoked through host state are still covered. Compiled only into a
    // debug build with the non-default feature enabled.
    #[cfg(all(feature = "test-fault-injection", debug_assertions))]
    if stage_code != stage::NONE && plan.fault.stage == stage_code {
        // SAFETY: the child window is the only caller, and `fail` ends the
        // process exactly as a real failure of this stage would.
        unsafe { fail(plan, stage_code, plan.fault.errno) }
    }

    // SAFETY: forwarded verbatim from this function's own contract, which the
    // call site established.
    let result = unsafe { syscall::syscall6(nr, a1, a2, a3, a4, a5, a6) };
    if syscall::is_error(result) {
        // SAFETY: as above; the child window is the only caller.
        unsafe { fail(plan, stage_code, syscall::errno_of(result)) }
    }
    result
}

/// Close one planned descriptor range, skipping an unused span.
///
/// # Safety
///
/// As [`issue`].
#[inline(always)]
unsafe fn close_span(plan: &ChildPlan, span: CloseSpan) {
    if span.used == 0 {
        return;
    }
    // SAFETY: `first` and `last` are descriptor numbers from the pure layout
    // plan, with `first <= last`, and the third argument is the zero flag word
    // `close_range` defines. No pointer is involved.
    unsafe {
        issue(
            plan,
            stage::CLOSE_RANGE,
            syscall::NR_CLOSE_RANGE,
            span.first,
            span.last,
            0,
            0,
            0,
            0,
        )
    };
}

/// Write the one structured failure record and end the process.
///
/// The record is exactly eight bytes — `[stage, 0, 0, 0, errno little-endian]`
/// — which is below `PIPE_BUF`, so the write is atomic and a reader sees either
/// nothing or the whole record. Nothing is formatted and nothing is allocated:
/// the three pad bytes come from the record the parent prepared inside the
/// plan, and the rest is two array destructurings.
///
/// A failure is **never** reported by exit status alone. A child that ends
/// without a record is exactly the S5 shape the parent must read as
/// indeterminate.
///
/// # Safety
///
/// The caller must be inside the child window, with `status_write` naming the
/// exec-status pipe's write end.
unsafe fn fail(plan: &ChildPlan, stage: u8, errno: i32) -> ! {
    // The parent prepared an all-zero eight-byte record. The stage byte and the
    // four errno bytes are replaced; the three pad bytes are kept as prepared.
    let [_, pad1, pad2, pad3, _, _, _, _] = plan.status_record;
    let [e0, e1, e2, e3] = errno.to_le_bytes();
    let record: [u8; 8] = [stage, pad1, pad2, pad3, e0, e1, e2, e3];

    loop {
        // SAFETY: `status_write` is the parent's relocated exec-status write
        // end, preserved by the close ranges and open here. `record` is a live
        // eight-byte stack array and the length matches it exactly.
        let written = unsafe {
            syscall::syscall6(
                syscall::NR_WRITE,
                plan.status_write,
                record.as_ptr().expose_provenance(),
                8,
                0,
                0,
                0,
            )
        };
        if !syscall::is_error(written) || syscall::errno_of(written) != syscall::EINTR {
            break;
        }
    }

    loop {
        // SAFETY: `exit_group` ends every thread of this process and takes no
        // pointer. The loop only satisfies the `!` return type.
        unsafe {
            syscall::syscall6(
                syscall::NR_EXIT_GROUP,
                syscall::CHILD_FAILURE_EXIT,
                0,
                0,
                0,
                0,
                0,
            )
        };
    }
}
