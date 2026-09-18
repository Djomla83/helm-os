//! Test-only fault injection for the closed child sequence.
//!
//! # This file does not exist in a default or release build
//!
//! Its module declaration in [`super`] carries
//! `#[cfg(all(feature = "test-fault-injection", debug_assertions))]`, and so
//! does every call site. The gate is deliberately **two** conditions, so that a
//! release build contains no injection even if the non-default feature is
//! enabled by accident. [`PRESENCE_MARKER`] is kept in the artifact by
//! `#[used]`, so "absent from release" is a property of the built object that
//! CI greps for, not a claim about source.
//!
//! # What it is for
//!
//! Two child states cannot be provoked through real host state, and the
//! accepted test plan requires both:
//!
//! * **S5** — a child that ends after its last setup stage and **before**
//!   `execveat`, writing no record. The parent then sees exactly what a
//!   successful exec shows: a clean end-of-file. The product must read that as
//!   indeterminate, never as success.
//! * **S6** — a child that stalls before `execveat`, so the parent's fixed
//!   pre-exec bound, its single pidfd `SIGKILL` and its bounded reap are
//!   exercised.
//!
//! A third mode reports a chosen stage as failed without issuing its system
//! call, which reaches the stage failure paths that no host state can produce.
//! That mode lives inside the child's one `issue` helper, not here.
//!
//! **The default build's trace remains the authority** for the closed child
//! syscall set. Injection may deviate from it, and does: the stall mode blocks
//! in `read` on descriptor 0, and the exit mode calls `exit_group` before the
//! execution attempt.

use super::{ChildPlan, syscall};

/// No injection.
pub(super) const MODE_NONE: u8 = 0;
/// End the process immediately before `execveat`, writing **no** record (S5).
pub(super) const MODE_EXIT_BEFORE_EXEC: u8 = 1;
/// Block immediately before `execveat` until the parent kills the child (S6).
pub(super) const MODE_STALL_BEFORE_EXEC: u8 = 2;

/// The exit status the S5 mode ends with.
///
/// Zero on purpose: a child that never executed anything, exiting 0, is the
/// sharpest possible test that clean end-of-file plus a normal exit status is
/// not read as exec success.
pub(super) const EXIT_BEFORE_EXEC_STATUS: usize = 0;

/// A string that must appear in a debug build with the feature enabled and must
/// **not** appear in any release artifact. `#[used]` keeps it in the object.
pub(super) const PRESENCE_MARKER: &str = "helm_launch_p3_fault_injection_present";

#[used]
static PRESENCE_MARKER_KEPT: &str = PRESENCE_MARKER;

/// The injections that run immediately before the execution attempt.
///
/// # Safety
///
/// The caller must be inside the child window, after stage 8 and before stage
/// 9, with descriptor 0 already bound to the stdin pipe by stage 1. The stall
/// mode additionally requires that the parent has kept the stdin write end
/// open, which `launch_with` does for exactly this mode.
pub(super) unsafe fn before_exec(plan: &ChildPlan) {
    if plan.fault.mode == MODE_EXIT_BEFORE_EXEC {
        loop {
            // SAFETY: `exit_group` ends every thread of this process and takes
            // no pointer. No record is written first: that is the whole point
            // of the S5 shape.
            unsafe {
                syscall::syscall6(
                    syscall::NR_EXIT_GROUP,
                    EXIT_BEFORE_EXEC_STATUS,
                    0,
                    0,
                    0,
                    0,
                    0,
                )
            };
        }
    }

    if plan.fault.mode == MODE_STALL_BEFORE_EXEC {
        let mut byte = [0_u8; 1];
        loop {
            // SAFETY: descriptor 0 is the blocking read end of the stdin pipe,
            // bound by stage 1, and `byte` is a live one-byte stack array whose
            // length matches the third argument exactly.
            let read = unsafe {
                syscall::syscall6(
                    syscall::NR_READ,
                    0,
                    byte.as_mut_ptr().expose_provenance(),
                    1,
                    0,
                    0,
                    0,
                )
            };
            if read == 0 {
                // End-of-file: the parent closed the write end after all, so
                // fall through to the ordinary execution attempt rather than
                // spinning.
                return;
            }
        }
    }
}
