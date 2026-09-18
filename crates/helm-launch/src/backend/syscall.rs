//! The one raw Linux x86_64 syscall shim, its kernel ABI records and its
//! constants (**P3**, plan section 7.6).
//!
//! # Why this module exists
//!
//! The post-clone child may run no userspace code this crate did not write
//! (ADR-0024 section D, plan section 7.3). `libc::syscall` is therefore
//! excluded: its stub writes glibc's thread-local `errno`, which would put
//! glibc code inside the child window. The parent's two `rt_sigprocmask` calls
//! use the same shim, because glibc's `sigfillset` omits, and `pthread_sigmask`
//! strips, glibc's two internal real-time signals, so neither can express the
//! full kernel mask the accepted contract requires (plan section 7.5).
//!
//! **There is exactly one raw syscall implementation in this crate**:
//! [`syscall6`]. Every raw call in the backend goes through it, and
//! `tests/p3_boundary.rs` fails if a second `asm!` block appears.
//!
//! # What this module is not
//!
//! It is not a general syscall layer. Parent-side operations that safe `rustix`
//! wrappers perform — pipe creation, descriptor relocation, `setpgid`,
//! `waitid`, pidfd signalling, reads — are **not** reimplemented here. The
//! closed list of authorised raw operations is in the module documentation of
//! [`super`].
//!
//! # Constants
//!
//! Every number below is written as the literal the Linux x86_64 UAPI defines
//! and is then pinned, at compile time, against the repository-vetted `libc`
//! constant of the same name. `libc` is a **constants-only** dependency: no
//! `libc` function, and no `extern` declaration of one, exists in this crate.

use core::arch::asm;

// ---------------------------------------------------------------- the shim

/// The most negative value the kernel returns as an encoded error. Linux
/// reserves `-4095 ..= -1` in the return register for `-errno`; every other
/// value is a successful result.
pub(super) const MIN_ERROR_RETURN: i64 = -4095;

/// Issue one Linux x86_64 system call.
///
/// The x86_64 syscall ABI places the syscall number in `rax` and the six
/// arguments in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`, returns the result in
/// `rax`, and clobbers `rcx` and `r11` (the `syscall` instruction stores the
/// return address in `rcx` and `RFLAGS` in `r11`). The result is returned
/// **raw**: negative values in `MIN_ERROR_RETURN ..= -1` encode `-errno`, and
/// no host `errno` variable, TLS slot or glibc state is read or written.
///
/// No `asm!` option is requested. `nomem` and `preserves_flags` would both be
/// wrong for a general syscall — the kernel may read and write memory the
/// arguments point at, and a syscall is not required to leave flags alone —
/// and `nostack` is only an optimisation that would let the compiler keep data
/// in the red zone across the call, which nothing here needs. Omitting all
/// three is the conservative choice and is always sound.
///
/// # Safety
///
/// The caller must guarantee that `nr` together with `a1 ..= a6` forms a
/// complete, valid invocation of that system call under the Linux x86_64 ABI:
/// every argument that the call interprets as a pointer must be a valid
/// address for the access the kernel performs, for the whole duration of the
/// call, and every argument that the call interprets as a descriptor, a signal
/// number or a flag word must be one the call accepts. The caller must also
/// accept the call's effects on this process, which are outside Rust's model.
#[inline]
pub(super) unsafe fn syscall6(
    nr: i64,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
) -> i64 {
    let result: i64;
    // SAFETY: this is the sole raw syscall site. Soundness for the Rust
    // abstract machine rests on the register contract alone: `rax` carries the
    // number in and the result out, `rdi`/`rsi`/`rdx`/`r10`/`r8`/`r9` carry the
    // six arguments, and `rcx` and `r11` are declared clobbered, which is
    // exactly what the `syscall` instruction destroys. No option is requested,
    // so the compiler assumes the block may read and write any memory and may
    // change the flags, and it may not keep live data in the red zone across
    // it. The block touches no memory of its own: any memory the kernel reaches
    // is named by the pointer arguments, whose validity is the caller's
    // documented obligation above.
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") nr => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            in("r10") a4,
            in("r8") a5,
            in("r9") a6,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

/// Whether a raw return value encodes an error.
pub(super) const fn is_error(result: i64) -> bool {
    result < 0 && result >= MIN_ERROR_RETURN
}

/// The `errno` a raw error return encodes.
///
/// Returns 0 for a value that is not an error return, so a caller that has not
/// checked [`is_error`] cannot mistake a successful result for a failure.
pub(super) fn errno_of(result: i64) -> i32 {
    if !is_error(result) {
        return 0;
    }
    // `result` is in `-4095 ..= -1`, so its magnitude is 1 ..= 4095, which
    // always fits an `i32`. The conversion stays checked rather than casting;
    // the fallback is unreachable and is not an error code.
    i32::try_from(result.unsigned_abs()).unwrap_or_default()
}

// ------------------------------------------------------------ syscall numbers
//
// `arch/x86/entry/syscalls/syscall_64.tbl`. Each is pinned against the vetted
// `libc` constant, so a wrong number is a compile error rather than a wrong
// system call.

/// `write(2)`.
pub(super) const NR_WRITE: i64 = 1;
/// `rt_sigaction(2)`.
pub(super) const NR_RT_SIGACTION: i64 = 13;
/// `rt_sigprocmask(2)`.
pub(super) const NR_RT_SIGPROCMASK: i64 = 14;
/// `dup2(2)`.
pub(super) const NR_DUP2: i64 = 33;
/// `fcntl(2)`.
pub(super) const NR_FCNTL: i64 = 72;
/// `fchdir(2)`.
pub(super) const NR_FCHDIR: i64 = 81;
/// `setpgid(2)`.
pub(super) const NR_SETPGID: i64 = 109;
/// `prctl(2)`.
pub(super) const NR_PRCTL: i64 = 157;
/// `exit_group(2)`.
pub(super) const NR_EXIT_GROUP: i64 = 231;
/// `execveat(2)`.
pub(super) const NR_EXECVEAT: i64 = 322;
/// `clone3(2)`.
pub(super) const NR_CLONE3: i64 = 435;
/// `close_range(2)`.
pub(super) const NR_CLOSE_RANGE: i64 = 436;
/// `read(2)`, reached only by the test-only pre-exec stall injection.
pub(super) const NR_READ: i64 = 0;

const _: () = assert!(libc::SYS_write == NR_WRITE);
const _: () = assert!(libc::SYS_rt_sigaction == NR_RT_SIGACTION);
const _: () = assert!(libc::SYS_rt_sigprocmask == NR_RT_SIGPROCMASK);
const _: () = assert!(libc::SYS_dup2 == NR_DUP2);
const _: () = assert!(libc::SYS_fcntl == NR_FCNTL);
const _: () = assert!(libc::SYS_fchdir == NR_FCHDIR);
const _: () = assert!(libc::SYS_setpgid == NR_SETPGID);
const _: () = assert!(libc::SYS_prctl == NR_PRCTL);
const _: () = assert!(libc::SYS_exit_group == NR_EXIT_GROUP);
const _: () = assert!(libc::SYS_execveat == NR_EXECVEAT);
const _: () = assert!(libc::SYS_clone3 == NR_CLONE3);
const _: () = assert!(libc::SYS_close_range == NR_CLOSE_RANGE);
const _: () = assert!(libc::SYS_read == NR_READ);

// ----------------------------------------------------------------- constants

/// `F_SETFD`, the descriptor-flag setter used to clear close-on-exec.
pub(super) const F_SETFD: usize = 2;
/// `AT_EMPTY_PATH`, which makes `execveat` execute the descriptor itself.
pub(super) const AT_EMPTY_PATH: usize = 0x1000;
/// `PR_SET_NO_NEW_PRIVS` (D-11).
pub(super) const PR_SET_NO_NEW_PRIVS: usize = 38;
/// `SIG_SETMASK`: replace the mask rather than adding to or removing from it.
pub(super) const SIG_SETMASK: usize = 2;
/// `CLONE_PIDFD`: the only clone flag this crate ever sets.
pub(super) const CLONE_PIDFD: u64 = 0x0000_1000;
/// `SIGCHLD`, the child's `exit_signal`.
pub(super) const SIGCHLD: u64 = 17;
/// `SIGKILL`, which cannot be blocked, caught or ignored.
pub(super) const SIGKILL: usize = 9;
/// `SIGSTOP`, which cannot be blocked, caught or ignored.
pub(super) const SIGSTOP: usize = 19;
/// The highest signal number the kernel defines, `_NSIG`.
pub(super) const NSIG: usize = 64;
/// `EINTR`, the only `write` error the child's failure path retries.
pub(super) const EINTR: i32 = 4;
/// `ENOSYS`, which `clone3` returns where the kernel does not provide it.
pub(super) const ENOSYS: i32 = 38;
/// `EPERM`, which `clone3` returns where a policy hides it.
pub(super) const EPERM: i32 = 1;
/// The child's fixed failure exit status.
pub(super) const CHILD_FAILURE_EXIT: usize = 127;

const _: () = assert!(libc::F_SETFD == 2);
const _: () = assert!(libc::AT_EMPTY_PATH == 0x1000);
const _: () = assert!(libc::PR_SET_NO_NEW_PRIVS == 38);
const _: () = assert!(libc::SIG_SETMASK == 2);
const _: () = assert!(libc::CLONE_PIDFD == 0x0000_1000);
const _: () = assert!(libc::SIGCHLD == 17);
const _: () = assert!(libc::SIGKILL == 9);
const _: () = assert!(libc::SIGSTOP == 19);
const _: () = assert!(libc::EINTR == 4);
const _: () = assert!(libc::ENOSYS == 38);
const _: () = assert!(libc::EPERM == 1);

// The clone flags the accepted mechanism must never set. They are named here so
// that the `clone3` flag word can be proven, at compile time, to contain none
// of them (plan section 7.3, ADR-0024 section D).
/// `CLONE_VM`: a shared address space. Never set.
pub(super) const CLONE_VM: u64 = 0x0000_0100;
/// `CLONE_FILES`: a shared descriptor table. Never set.
pub(super) const CLONE_FILES: u64 = 0x0000_0400;
/// `CLONE_VFORK`: suspend the parent until exec. Never set.
pub(super) const CLONE_VFORK: u64 = 0x0000_4000;
/// `CLONE_THREAD`: a thread rather than a process. Never set.
pub(super) const CLONE_THREAD: u64 = 0x0001_0000;

const _: () = assert!(libc::CLONE_VM == 0x0000_0100);
const _: () = assert!(libc::CLONE_FILES == 0x0000_0400);
const _: () = assert!(libc::CLONE_VFORK == 0x0000_4000);
const _: () = assert!(libc::CLONE_THREAD == 0x0001_0000);

/// The kernel signal set on x86_64: 64 bits, one per signal, signal `n` at bit
/// `n - 1`. `sigsetsize` is therefore always 8.
pub(super) const KERNEL_SIGSET_BYTES: usize = 8;

/// Every bit of the kernel signal set.
///
/// This is the mask the accepted contract blocks across `clone3` (T21). It
/// includes the bits of glibc's two internal real-time signals, `SIGCANCEL`
/// (`__SIGRTMIN`, signal 32, bit 31) and `SIGSETXID` (`__SIGRTMIN + 1`, signal
/// 33, bit 32), which `sigfillset` omits and `pthread_sigmask` strips, and
/// which is why this call cannot go through glibc.
///
/// **`SIGKILL` and `SIGSTOP` are not claimed blockable.** Their bits are set in
/// the value passed to the kernel, and `do_sigprocmask` removes them from the
/// mask it installs; they keep acting on the thread with their default actions.
pub(super) const FULL_KERNEL_SIGSET: u64 = u64::MAX;

/// The empty kernel signal set: the final mask the executed image receives.
pub(super) const EMPTY_KERNEL_SIGSET: u64 = 0;

// glibc's two internal real-time signal numbers, as bit positions.
const _: () = assert!(FULL_KERNEL_SIGSET & (1_u64 << 31) != 0);
const _: () = assert!(FULL_KERNEL_SIGSET & (1_u64 << 32) != 0);
const _: () = assert!(KERNEL_SIGSET_BYTES == core::mem::size_of::<u64>());

// ------------------------------------------------------------- clone_args

/// `struct clone_args` of `include/uapi/linux/sched.h`.
///
/// Eleven `__aligned_u64` fields in this order. On x86_64 a `u64` is already
/// eight-byte aligned, so `repr(C)` reproduces the kernel layout exactly; the
/// assertions below pin the size and every field offset.
///
/// The whole record is zeroed and only `flags`, `pidfd` and `exit_signal` are
/// set. `stack` stays 0, which means the child continues on the caller's stack
/// in its own copy-on-write copy of the address space.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CloneArgs {
    /// Clone flags. This crate sets exactly `CLONE_PIDFD`.
    pub(super) flags: u64,
    /// Address of a parent-owned `i32` the kernel fills with the pidfd.
    pub(super) pidfd: u64,
    /// `CLONE_CHILD_SETTID` target. Unused, zero.
    pub(super) child_tid: u64,
    /// `CLONE_PARENT_SETTID` target. Unused, zero.
    pub(super) parent_tid: u64,
    /// The signal delivered to the parent when the child ends: `SIGCHLD`.
    pub(super) exit_signal: u64,
    /// Child stack. Zero: the child uses the caller's stack.
    pub(super) stack: u64,
    /// Child stack size. Unused, zero.
    pub(super) stack_size: u64,
    /// `CLONE_SETTLS` value. Unused, zero.
    pub(super) tls: u64,
    /// `set_tid` array. Unused, zero.
    pub(super) set_tid: u64,
    /// `set_tid` array length. Unused, zero.
    pub(super) set_tid_size: u64,
    /// `CLONE_INTO_CGROUP` descriptor. Unused, zero.
    pub(super) cgroup: u64,
}

/// `CLONE_ARGS_SIZE_VER0`: the first published `struct clone_args`.
pub(super) const CLONE_ARGS_SIZE_VER0: usize = 64;
/// `CLONE_ARGS_SIZE_VER2`: the third, and the current size of the record.
pub(super) const CLONE_ARGS_SIZE_VER2: usize = 88;

const _: () = assert!(core::mem::size_of::<CloneArgs>() == CLONE_ARGS_SIZE_VER2);
const _: () = assert!(core::mem::align_of::<CloneArgs>() == 8);
const _: () = assert!(core::mem::offset_of!(CloneArgs, flags) == 0);
const _: () = assert!(core::mem::offset_of!(CloneArgs, pidfd) == 8);
const _: () = assert!(core::mem::offset_of!(CloneArgs, child_tid) == 16);
const _: () = assert!(core::mem::offset_of!(CloneArgs, parent_tid) == 24);
const _: () = assert!(core::mem::offset_of!(CloneArgs, exit_signal) == 32);
const _: () = assert!(core::mem::offset_of!(CloneArgs, stack) == 40);
const _: () = assert!(core::mem::offset_of!(CloneArgs, stack_size) == 48);
const _: () = assert!(core::mem::offset_of!(CloneArgs, tls) == 56);
const _: () = assert!(core::mem::offset_of!(CloneArgs, set_tid) == 64);
const _: () = assert!(core::mem::offset_of!(CloneArgs, set_tid_size) == 72);
const _: () = assert!(core::mem::offset_of!(CloneArgs, cgroup) == 80);
const _: () = assert!(CLONE_ARGS_SIZE_VER0 == 64);

impl CloneArgs {
    /// The only `clone_args` this crate builds: `CLONE_PIDFD` alone,
    /// `exit_signal = SIGCHLD`, the pidfd output address, everything else zero.
    pub(super) const fn for_direct_child(pidfd_out: u64) -> Self {
        Self {
            flags: CLONE_PIDFD,
            pidfd: pidfd_out,
            child_tid: 0,
            parent_tid: 0,
            exit_signal: SIGCHLD,
            stack: 0,
            stack_size: 0,
            tls: 0,
            set_tid: 0,
            set_tid_size: 0,
            cgroup: 0,
        }
    }
}

// The flag word is exactly `CLONE_PIDFD` — one bit — and none of the four
// forbidden flags.
const _: () = assert!(CLONE_PIDFD.count_ones() == 1);
const _: () = assert!(CLONE_PIDFD & CLONE_VM == 0);
const _: () = assert!(CLONE_PIDFD & CLONE_FILES == 0);
const _: () = assert!(CLONE_PIDFD & CLONE_VFORK == 0);
const _: () = assert!(CLONE_PIDFD & CLONE_THREAD == 0);

// --------------------------------------------------------- kernel sigaction

/// `struct sigaction` **as the raw `rt_sigaction` system call reads it on
/// x86_64**, which is not glibc's userspace layout.
///
/// x86 defines `__ARCH_HAS_SA_RESTORER`, so the kernel record of
/// `include/linux/signal_types.h` is, in this order: `sa_handler`, `sa_flags`,
/// `sa_restorer`, `sa_mask` — the mask **last**. glibc's userspace
/// `struct sigaction` puts `sa_mask` second, so passing a glibc value to the
/// raw syscall would write the mask into the flags and restorer words. This is
/// the layout glibc itself uses internally as `struct kernel_sigaction`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct KernelSigaction {
    /// The disposition. `SIG_DFL` is the null handler.
    pub(super) handler: usize,
    /// `SA_*` flags.
    pub(super) flags: u64,
    /// The signal trampoline. Only ever consulted when delivering to a **user
    /// handler**, so it stays null for `SIG_DFL`.
    pub(super) restorer: usize,
    /// The mask applied while a handler runs; the kernel record ends here.
    pub(super) mask: u64,
}

/// The size the raw `rt_sigaction` call copies from user space.
pub(super) const KERNEL_SIGACTION_BYTES: usize = 32;

const _: () = assert!(core::mem::size_of::<KernelSigaction>() == KERNEL_SIGACTION_BYTES);
const _: () = assert!(core::mem::align_of::<KernelSigaction>() == 8);
const _: () = assert!(core::mem::offset_of!(KernelSigaction, handler) == 0);
const _: () = assert!(core::mem::offset_of!(KernelSigaction, flags) == 8);
const _: () = assert!(core::mem::offset_of!(KernelSigaction, restorer) == 16);
const _: () = assert!(core::mem::offset_of!(KernelSigaction, mask) == 24);

impl KernelSigaction {
    /// The one disposition the child installs: `SIG_DFL`, no flags, no
    /// restorer, an empty handler mask.
    ///
    /// `SA_RESTORER` is deliberately absent. The kernel requires a restorer
    /// only when it builds a frame for a **user handler**
    /// (`setup_rt_frame` on x86_64); a signal whose disposition is `SIG_DFL` is
    /// handled entirely in the kernel and never reaches that path.
    /// `do_sigaction` itself validates neither the flags nor the restorer.
    pub(super) const DEFAULT_DISPOSITION: Self = Self {
        handler: 0,
        flags: 0,
        restorer: 0,
        mask: EMPTY_KERNEL_SIGSET,
    };
}

// `SIG_DFL` is the null handler.
const _: () = assert!(KernelSigaction::DEFAULT_DISPOSITION.handler == 0);

// ------------------------------------------------------------ host width

// Every address this module hands the kernel is a host pointer written into a
// 64-bit ABI field. The cohort gate already restricts the module to x86_64;
// this pins the assumption in source.
const _: () = assert!(core::mem::size_of::<usize>() == core::mem::size_of::<u64>());
