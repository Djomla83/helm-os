# HELM-LAUNCH P3 — GENUINELY INDEPENDENT UNSAFE REVIEW of the corrected candidate

> # ✅ GENUINELY INDEPENDENT REVIEW — THIS IS THE P3 REVIEW GATE ARTIFACT
>
> **The reviewer authored none of the P3 commits under review.** This session did not produce
> `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60` (authority),
> `afe8922bebd0c85ead7e58d146b738b84141f096` (implementation),
> `168fe1339dd20bdecc8d5c0f111ef5f185493e9f` (author self-review),
> `ac823cd87e38ad5af5393ff3436c5c55b1f496df` (owner disposition) or
> `672228b8eeeef95cf72bb07051c0ccdec1ae261f` (bounded correction). It read them, re-derived the
> safety case from source, machine code, artifacts and tests, and reached its own conclusions.
>
> **This is distinct from [`HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md)**
> (`168fe133`, reclassified by `ac823cd8`), which is an **AUTHOR SELF-REVIEW**, is **NOT
> INDEPENDENT**, and is **NOT AN APPROVAL ARTIFACT**. Its findings guided investigation here; none
> of its conclusions was taken on trust. Every verdict below rests on evidence this session
> produced or re-derived.

> **SUBJECT: THE CORRECTED HELM-LAUNCH P3 CANDIDATE, `9fb0f8ca..672228b8` IN FULL — NOT ONLY THE
> CORRECTION.**
>
> **P1 ACCEPTED. P2 ACCEPTED. P3 AUTHORISED, NOT YET ACCEPTED. P4 AND P5 NOT AUTHORISED.**
>
> **NO PUBLIC `launch()` EXISTS. NO PROCESS-GROUP SWEEP EXISTS. NO TRIAL #4 IS AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| BLOCKER | **0** |
| IMPORTANT | **2** — `P3R-10` (new), `P3R-11` (new) |
| MINOR | 8 — `P3R-03` … `P3R-09`, `P3R-12` (new) |
| BACKLOG_NONBLOCKING | 2 — `P3R-13`, `P3R-14` (new) |
| GATE_PENDING | 2 — Linux runtime, strace child window |
| `P3R-01` (artifact selection / injection proof) | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-02` (child closed world at machine level) | **INDEPENDENTLY VERIFIED FIXED** |
| Classification | **`HELM_LAUNCH_P3_INDEPENDENT_UNSAFE_REVIEW_NEEDS_FIX`** |

The two corrections the owner required are genuinely made, and this review confirms both from
independently regenerated evidence rather than from the author's report. The candidate nevertheless
cannot be published as it stands: one load-bearing Linux backend test **cannot pass** because a
fixture report key and its parser key disagree, and the new machine-code gate — the artifact the
owner made load-bearing for the corrected P3R-02 property — **does not fail closed on one class of
control-transfer instruction**, contrary to the explicit fail-closed requirement.

Neither IMPORTANT is a memory-safety, authority or boundary defect. The P3 safety case itself —
unsafe confinement, the clone ABI, the signal ABI, the borrowed `ChildPlan`, the inner pointers, the
closed child sequence, descriptor isolation, exec authority and direct-child cleanup — is sound, and
this review re-established it independently, including at the level of emitted x86_64 machine code.

## 1. Starting state, verified

| Requirement | Observed | Result |
|---|---|---|
| Branch | `docs/helm-launch-architecture` | ✅ |
| Worktree | clean (`git status --porcelain` empty) | ✅ |
| Local `HEAD` | `672228b8eeeef95cf72bb07051c0ccdec1ae261f` | ✅ |
| `origin/docs/helm-launch-architecture` | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` | ✅ |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` | ✅ |
| Branch relation | **5 ahead, 0 behind** | ✅ |
| Exact local chain | `9fb0f8c → 7bb016f → afe8922 → 168fe13 → ac823cd → 672228b` | ✅ |

`origin` was fetched read-only. Nothing was pushed, amended, rebased or dispatched.

## 2. Authority, re-read from source

Read in full: `AGENTS.md`, [ADR-0024](../adr/ADR-0024-launch-authority.md) (750 lines, sections A–N),
the [productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md) P3 authority note,
[`docs/DECISIONS.md`](../DECISIONS.md) (P3 authorisation `7bb016f`, author-review disposition
`ac823cd`), [`docs/PROJECT_STATE.md`](../PROJECT_STATE.md).

| Item | Current authority |
|---|---|
| ADR-0024 | **ACCEPTED** 2026-09-17 |
| HELM-LAUNCH P1 | **ACCEPTED** 2026-09-17 |
| HELM-LAUNCH P2 | **ACCEPTED** 2026-09-18 |
| HELM-LAUNCH P3 | **AUTHORISED** 2026-09-18, **not yet accepted** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #3 | **`MECHANISM_REJECTED`**, historical and immutable, not rerun |
| Trial #4 | **NOT AUTHORISED** |
| helm-launch 0.1 module | **NOT YET PRODUCT-ACCEPTED** |

The authority commit `7bb016f` touches only `docs/DECISIONS.md`, `docs/PROJECT_STATE.md`,
`docs/adr/ADR-0024-launch-authority.md` and the plan — no product code, test, workflow or Cargo
file — and the accepted contract of ADR-0024 sections A to N is unchanged by it.

## 3. Commit and review provenance

| Commit | Content | Crosses into P4? |
|---|---|---|
| `7bb016f` | authority and status documents only (4 files) | no |
| `afe8922` | original bounded P3 implementation (16 files) | no |
| `168fe13` | one new document, the author review (1 file, 816 lines) | no |
| `ac823cd` | owner disposition + **rename** of the review to `…-AUTHOR-UNSAFE-REVIEW.md` (3 files) | no |
| `672228b` | bounded correction: `child.rs`, `spawn.rs`, `p3_boundary.rs`, the workflow, two new tools, one new tool test file (7 files) | no |

The correction adds no public item, no `launch`, no `LaunchOutcome`, no group signal and no P4
lifecycle. The reclassified author review now opens with `# HELM-LAUNCH P3 — AUTHOR SELF-REVIEW`
and `⚠ AUTHOR SELF-REVIEW — NOT INDEPENDENT`, and states `THIS DOCUMENT DOES NOT SATISFY THE P3
INDEPENDENT REVIEW GATE`. It no longer represents itself as independent, and the path
`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md` is free for this artifact.

**`P3_COMMIT_AND_REVIEW_PROVENANCE_SOUND`.**

## 4. Absolute P3 boundary, independently proven

Established by reading [`src/lib.rs`](../../crates/helm-launch/src/lib.rs), the export list at the
accepted P2 base versus `HEAD`, [`src/backend/mod.rs`](../../crates/helm-launch/src/backend/mod.rs),
and by grep over the whole crate.

| Property | Independent result |
|---|---|
| Internal process creation | **PRESENT — Linux x86_64 only** (`cfg(all(target_os = "linux", target_arch = "x86_64"))` on `mod backend`) |
| Internal execution attempt | **PRESENT — authorised object only** (`execveat(plan.executable, "", argv, [NULL], AT_EMPTY_PATH)`) |
| Public `launch()` | **ABSENT** |
| Public process handle | **ABSENT** |
| `LaunchOutcome` | **ABSENT** (appears only in prose and in `compile_fail` doctests) |
| P4 run lifecycle | **ABSENT** |
| `SIGTERM` / grace | **ABSENT** |
| General stream drain | **ABSENT** (`drain_bounded` is test infrastructure inside `#[cfg(test)]`) |
| Process-group sweep | **ABSENT** — no `kill`, `killpg`, `tgkill`, `tkill`, negative pid or `Pgid` anywhere |
| Real execution receipt | **ABSENT** (`MinimalLaunch` has no serialisation, digest or verdict) |
| Wine / sandbox / containment | **ABSENT** |
| P4 / P5 implementation | **NONE** |

The public export list is **byte-identical** to the accepted P2 base: `pub use` of `authority`,
`error`, `model`, `plan` and `receipt` is unchanged, and the only `lib.rs` additions are
`#[cfg(...)] mod backend;` (private, not re-exported) and documentation. `Backend` and `GroupSweep`
in the `model` export list are pre-existing P1 receipt-vocabulary enums, present at `9fb0f8ca`, not
new P3 surface. `AuthorizedParts` and `AuthorizedLaunch::into_parts` are `pub(crate)`. No bare `pub`
item exists in any backend file.

No accidental public execution surface. No negative-pid group kill. **No BLOCKER.**

## 5. Complete unsafe inventory — counted independently

Counted by this session's own scan, not copied from the author review.

| Item | Count | Location |
|---|---|---|
| `unsafe { … }` blocks | **32** | `child.rs` 23, `spawn.rs` 6, `syscall.rs` 1, `injection.rs` 2 (feature+debug gated) |
| `unsafe fn` | **7** | `child_main`, `issue`, `close_span`, `fail`, `clone_and_dispatch`, `syscall6`, `before_exec` |
| `asm!` invocations | **1** | [`syscall.rs:89`](../../crates/helm-launch/src/backend/syscall.rs#L89) |
| `#![allow(unsafe_code)]` as code | **1** | [`backend/mod.rs:57`](../../crates/helm-launch/src/backend/mod.rs#L57), inner attribute |
| `FromRawFd` / raw ownership adoption | **1** | [`spawn.rs:432`](../../crates/helm-launch/src/backend/spawn.rs#L432) |
| Raw pointer / provenance sites | **6** | `spawn.rs` 173 (`CString::as_ptr().cast`), 299, 303, 308; `child.rs` 476; `injection.rs` 96 |
| `// SAFETY:` comments | **32** | one per block, 1:1 |
| `extern` blocks | **0** | — |
| `libc` **function** calls | **0** | 28 code mentions, all inside `const _: () = assert!(libc::…)` |
| `transmute`, `static mut`, `MaybeUninit`, `from_raw_parts`, `mmap`, `fork`, `vfork`, `pidfd_open`, `fexecve`, `posix_spawn`, `Command` in backend product code | **0** | — |

All code-level `unsafe` is under `crates/helm-launch/src/backend/**`. The only other `unsafe` token
in the crate is inside string literals in `tests/p2_boundary.rs` and `tests/p3_boundary.rs`, and
`crates/helm-evidence/src/lib.rs` carries `#![forbid(unsafe_code)]` plus one prose use of the word.
Exactly one scoped relaxation exists, at the backend module boundary, as an inner attribute, with
the four backend-only `deny`s beside it.

Mapping to the owner's **closed list of six**:

| # | Authorised operation | Sites | Result |
|---|---|---|---|
| 1 | raw `rt_sigprocmask` around `clone3`, and the restore | `spawn.rs` 2 | ✅ |
| 2 | raw `clone3` | `spawn.rs` 1 | ✅ |
| 3 | `OwnedFd::from_raw_fd(pidfd)` after a successful `clone3` | `spawn.rs` 1 | ✅ |
| 4 | crossing into the child entry with the prepared `ChildPlan` | `spawn.rs` 1 | ✅ |
| 5 | raw child syscalls of the closed sequence | `child.rs` 23, `injection.rs` 2 | ✅ |
| 6 | the x86_64 `asm!` shim itself | `syscall.rs` 1 | ✅ |

**No new unsafe category.** `injection.rs`'s two blocks are category 5 raw child syscalls
(`exit_group`, `read`) inside the non-default `test-fault-injection` feature the P3 authority
explicitly permits. Parent-side work a safe `rustix` wrapper performs — `pipe2`, `F_DUPFD_CLOEXEC`,
`F_GETFL`/`F_SETFL`, `setpgid`, `read`, `pidfd_send_signal`, `waitid` — is not reimplemented raw.

**`P3_UNSAFE_CONFINEMENT_SOUND`.**

## 6. Raw syscall ABI

[`syscall6`](../../crates/helm-launch/src/backend/syscall.rs#L68) audited as emitted:

```text
inlateout("rax") nr => result, in("rdi") a1, in("rsi") a2, in("rdx") a3,
in("r10") a4, in("r8") a5, in("r9") a6, lateout("rcx") _, lateout("r11") _,
```

Correct Linux x86_64 ABI: number and result in `rax`; arguments in `rdi`, `rsi`, `rdx`, `r10`, `r8`,
`r9`; `rcx` and `r11` destroyed by the `syscall` instruction and declared clobbered. Every other
register is preserved by the kernel, so the clobber list is complete. **No `asm!` option is
requested at all** — no `nomem`, no `readonly`, no `preserves_flags`, no `pure`, no `nostack` —
which is the conservative and sound choice for a general syscall, and `tests/p3_boundary.rs` fails
if any of them appears.

Errno decoding: `is_error(r) = r < 0 && r >= -4095` and
`errno_of(r) = i32::try_from(r.unsigned_abs()).unwrap_or_default()`, returning `0` for a
non-error. Correct for Linux's reserved `-4095 ..= -1` band, and `unsigned_abs()` of that band is
`1 ..= 4095`, which always fits `i32`. Verified in the emitted code as `cmpq $-4095, %rax` /
`cmpq $-4096, %rax` with the matching unsigned branch, and `negl %eax` for the errno.

Every number and constant was checked against the Linux x86_64 UAPI independently of the `libc`
constants the crate pins against:

| Symbol | Crate | Independent | Symbol | Crate | Independent |
|---|---|---|---|---|---|
| `read` | 0 | 0 ✅ | `write` | 1 | 1 ✅ |
| `rt_sigaction` | 13 | 13 ✅ | `rt_sigprocmask` | 14 | 14 ✅ |
| `dup2` | 33 | 33 ✅ | `fcntl` | 72 | 72 ✅ |
| `fchdir` | 81 | 81 ✅ | `setpgid` | 109 | 109 ✅ |
| `prctl` | 157 | 157 ✅ | `exit_group` | 231 | 231 ✅ |
| `execveat` | 322 | 322 ✅ | `clone3` | 435 | 435 ✅ |
| `close_range` | 436 | 436 ✅ | `F_SETFD` | 2 | 2 ✅ |
| `AT_EMPTY_PATH` | 0x1000 | 0x1000 ✅ | `PR_SET_NO_NEW_PRIVS` | 38 | 38 ✅ |
| `SIG_SETMASK` | 2 | 2 ✅ | `CLONE_PIDFD` | 0x1000 | 0x1000 ✅ |
| `SIGCHLD` | 17 | 17 ✅ | `SIGKILL` | 9 | 9 ✅ |
| `SIGSTOP` | 19 | 19 ✅ | `_NSIG` | 64 | 64 ✅ |
| `EINTR` | 4 | 4 ✅ | `ENOSYS` | 38 | 38 ✅ |
| `EPERM` | 1 | 1 ✅ | `CLONE_VM` | 0x100 | 0x100 ✅ |
| `CLONE_FILES` | 0x400 | 0x400 ✅ | `CLONE_VFORK` | 0x4000 | 0x4000 ✅ |
| `CLONE_THREAD` | 0x10000 | 0x10000 ✅ | kernel `sigset_t` | 8 bytes | 8 ✅ |

The `libc` pinning is a second, independent check of the same numbers at compile time, and it is
constants-only: 28 code mentions of `libc::`, all 28 inside `const _: () = assert!(…)`.

**`P3_RAW_SYSCALL_ABI_SOUND`.**

## 7. `clone3`, pidfd and the return path

`CloneArgs` is `repr(C)` with eleven `u64` fields in UAPI order, size 88, align 8, and every offset
pinned by `const` assertion: `0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80`. Independently checked
against `include/uapi/linux/sched.h`: `flags, pidfd, child_tid, parent_tid, exit_signal, stack,
stack_size, tls, set_tid, set_tid_size, cgroup` — **exact match**, and `CLONE_ARGS_SIZE_VER2 = 88`
is the size the call is given. `CLONE_ARGS_SIZE_VER0 = 64` is pinned but unused, correctly.

`for_direct_child` sets `flags = CLONE_PIDFD` (one bit, proven by
`assert!(CLONE_PIDFD.count_ones() == 1)`), `exit_signal = SIGCHLD`, the pidfd output address, and
**zero everywhere else** — `stack = 0` with `stack_size = 0`, which `clone3_stack_valid` accepts and
which makes the child continue on the parent's stack pointer in its own copy-on-write copy
(`copy_thread` leaves `childregs->sp` unchanged when `sp == 0`, and sets `ax = 0`). Compile-time
assertions prove the flag word contains none of `CLONE_VM`, `CLONE_FILES`, `CLONE_VFORK`,
`CLONE_THREAD`.

The pidfd output is `&mut pidfd_slot: i32` in the cloning frame; ownership is adopted immediately
after the success check, guarded by `if pidfd_slot < 0`. There is no `pidfd_open` anywhere. No
fallback: `ENOSYS`/`EPERM` become `ProcessCreationUnavailable`, anything else
`ProcessCreationFailed`, with no child and no receipt.

**Both return paths, verified in the emitted release-codegen assembly of
[`clone_and_dispatch`](../../crates/helm-launch/src/backend/spawn.rs#L471):**

```text
pushq %rbx
movq  %rsi, %rbx          ; the borrowed ChildPlan, into a callee-saved register
movl  $435, %eax          ; clone3
movl  $88,  %esi          ; CLONE_ARGS_SIZE_VER2
xorl  %edx,%edx  /  %r10d / %r8d / %r9d
syscall
testq %rax, %rax
je    .LBB56_2
popq  %rbx  /  retq       ; parent
.LBB56_2:
movq  %rbx, %rdi
callq …backend5child10child_main…   ; child — nothing else in between
```

On the child branch there is **no semantic work whatever** between the kernel's return and the
child entry: one register move and the call. `%rbx` is callee-saved and preserved by the kernel
across `syscall`, and the child inherits the identical register file with `rax = 0`, so the plan
reference is correct in the child.

On the parent branch, `setpgid(child, child)` is the **first system call** after `clone3`: the only
intervening statements are `i32::try_from` and `Pid::from_raw`, both pure arithmetic. The mask
restore follows the `setpgid` attempt. Only the parent's own `setpgid` success sets
`group_authority_established`, which nothing in P3 consumes.

**`P3R-03` re-evaluated independently and NOT promoted.** `PidfdNotProvided` is unreachable: on
success with `CLONE_PIDFD` the kernel always writes the descriptor. If it ever triggered, the
`SpawnedChild` would not be constructed and the child would be left to the host — a documented,
unreachable residual. The accepted kernel contract is sufficient, so **no numeric-pid fallback is
recommended**; adding one would violate the accepted contract. Stays MINOR.

The `SignalMaskRestoreFailed` path returns **after** `ChildHandle` exists, so the handle drops and
kills-and-reaps within the fixed bound. No leak there.

**`P3_CLONE_ABI_AND_RETURN_PATH_SOUND`.**

## 8. Signal ABI

`FULL_KERNEL_SIGSET = u64::MAX`, `sigsetsize = 8`, `SIG_SETMASK = 2`. Independently confirmed
against the kernel: `do_sigprocmask` rejects `sigsetsize != sizeof(sigset_t)` (8 on x86_64) and
itself does `sigdelsetmask(&new_set, sigmask(SIGKILL)|sigmask(SIGSTOP))`, so passing every bit is
accepted and `SIGKILL`/`SIGSTOP` are removed by the kernel, never claimed blockable. Bit numbering
is `sigmask(n) = 1 << (n-1)`, so signal 32 (`SIGCANCEL`) is bit 31 and signal 33 (`SIGSETXID`) is
bit 32 — both set, which is exactly what `sigfillset` omits and `pthread_sigmask` strips, and the
reason this call cannot go through glibc. The saved mask is written into a live `u64` of the cloning
frame and restored on both the success and failure paths.

`KernelSigaction` is `repr(C)`: `handler, flags, restorer, mask` — the mask **last** — size 32,
align 8, offsets `0, 8, 16, 24`, all pinned. Independently confirmed: x86 defines
`__ARCH_HAS_SA_RESTORER`, so `struct sigaction` in `include/linux/signal_types.h` is exactly that
order. glibc's userspace layout puts `sa_mask` second, which would write the mask into the flags and
restorer words — the crate's comment states this correctly and the layout avoids it.

**`SIG_DFL` without `SA_RESTORER` is sound**, verified against the kernel rather than accepted from
the comment: `do_sigaction` validates neither `sa_flags` nor `sa_restorer`; it rejects only an
invalid signal, signal `< 1`, and `sig_kernel_only(sig)` (`SIGKILL`, `SIGSTOP`). A restorer is
consulted only in `__setup_rt_frame`, which is reached only when a signal is delivered to a **user
handler**; a `SIG_DFL` disposition is handled entirely inside `get_signal` and never reaches that
path. The child's loop covers signals `1 ..= 64` skipping 9 and 19 — 62 calls, `valid_signal(sig)`
being `sig <= _NSIG` — which the traced test also pins as exactly 62.

**`P3_SIGNAL_ABI_SOUND`.**

## 9. `P3R-02` — the borrowed `ChildPlan`

This is the load-bearing correction. "The `memcpy` disappeared" was **not** accepted as a proof; the
reference semantics were reviewed from first principles and then confirmed in emitted code.

**Signature and call site.** `unsafe fn child_main(plan: &ChildPlan) -> !`, `#[inline(never)]`,
called as `child_main(plan)` from `clone_and_dispatch(args: &mut CloneArgs, plan: &ChildPlan)`,
which is called as `clone_and_dispatch(&mut clone_args, &plan)` where `plan` is a **named local
`ChildPlan` of `spawn`'s frame**. A shared reference, not a raw pointer, and never dereferenced
through one.

| Question | Independent answer |
|---|---|
| Reference or pointer? | **Shared reference** `&ChildPlan`. `tests/p3_boundary.rs` fails if `*const ChildPlan` reappears or if `let plan: ChildPlan =` (a whole-record copy) appears. |
| Where is it created? | From the named local `plan` in `spawn`, after `PreparedLaunch` has stopped moving. |
| Alignment statically guaranteed? | **Yes.** A reference to a local of type `ChildPlan` is aligned by construction; no raw pointer is cast, so no alignment assumption is asserted at runtime. |
| Pointee live for the whole child window? | **Yes.** `child_main` returns `!` and its only exits are a successful `execveat` or `exit_group`, so `spawn`'s frame is never unwound in the child. |
| Can the compiler assume anything invalid across the returns-twice `clone3`? | **No.** `clone3` without `CLONE_VM` is fork-like: the kernel copies **both** the address space (copy-on-write) and the register file, and the child resumes at the next instruction with `rax = 0`. That is unlike `setjmp`/`vfork`, where the returns-twice hazard comes from a *shared* address space or partially restored registers. LLVM needs no `returns_twice` attribute for this shape, and the emitted code confirms the plan reference survives in a callee-saved register the kernel preserves. |
| Can the parent mutate / move / drop the underlying object in a way that matters? | **No.** `plan` is only ever borrowed immutably, and after the clone the two processes have separate copy-on-write address spaces, so no parent write, move or drop is visible to the child. |
| Why is parent activity irrelevant after the clone? | Because no `CLONE_VM` means the child's page tables are its own copy; the parent's later writes fault in private copies on the parent side only. |
| Does every pointer stored inside `ChildPlan` have equally valid storage? | **Yes** — see section 10. |

**Emitted-code audit around the clone3 return, the child branch and the child entry** — not merely
`child_main` in isolation. The `clone_and_dispatch` listing in section 7 shows the branch is
`testq`/`je` plus one `movq` and the call. Inside `child_main` the plan arrives in `%rdi`, is moved
to `%rbx`, and **every field read is a plain load**:

| Field | Struct offset | Emitted access | Used as |
|---|---|---|---|
| `stdin_source` | 0 | `movq (%rbx), %rdi` | `dup2` source → 0 |
| `stdout_source` | 8 | `movq 8(%rbx), %rdi` | `dup2` source → 1 |
| `stderr_source` | 16 | `movq 16(%rbx), %rdi` | `dup2` source → 2 |
| `working_directory` | 24 | `movq 24(%rbx), %rdi` | `fchdir` |
| `executable` | 32 | `movq 32(%rbx), %rdi` | `execveat` dirfd |
| `close_ranges[n].used` | 64 / 88 / 112 | `cmpq $0, 64(%rbx)` … | skip an unused span |
| `close_ranges[n].first/last` | 48/56, 72/80, 96/104 | `movq 48(%rbx), %rdi` … | `close_range` |
| `argv` | 120 | `movq 120(%rbx), %rdx` | `execveat` argv |
| `envp` | 128 | `movq 128(%rbx), %r10` | `execveat` envp |
| `empty_path` | 136 | `movq 136(%rbx), %rsi` | `execveat` pathname |
| `default_action` | 144 | `movq 144(%rbx), %rsi` | `rt_sigaction` act |
| `empty_signal_mask` | 152 | `movq 152(%rbx), %rsi` | `rt_sigprocmask` nset |

**No `memcpy`, no `memmove`, no null-pointer check, no alignment check, no panic-runtime call and no
external helper of any kind** appears in the child's closure in either profile. The correction did
**not** trade compiler helper calls for unsound reference or lifetime assumptions: it removed a
whole-record copy and a raw-pointer dereference, and both removals strictly narrow the unsafe
surface — the authorised "crossing into the child entry" operation is now the call alone.

The compile-time proofs beside the type remain: `!needs_drop::<ChildPlan>()`,
`!needs_drop::<CloseSpan>()`, `!needs_drop::<Fault>()`, `size_of::<ChildPlan>() > 0`, and
`Copy` proven by use for all three.

**`P3_CORRECTED_CHILDPLAN_REFERENCE_SOUND`. `P3R-02` INDEPENDENTLY VERIFIED FIXED.**

## 10. `ChildPlan` inner pointers

Every address the child hands the kernel was traced to its storage.

| Address | Storage | Prepared before clone | Stable | Alive at clone | Alive in the child's COW image | Alignment | Termination |
|---|---|---|---|---|---|---|---|
| `argv` | heap buffer of `argv_pointers: Vec<*const u8>`, owned by `PreparedLaunch` | yes, in `prepare` | yes — no `push` after the address is taken in `spawn` | yes (`prepared` outlives `spawn`) | yes | 8 (heap alloc for `*const u8`) | `push(null())` after the loop ✅ |
| each `argv[i]` | the `CString`'s own heap buffer inside `argv_storage: Vec<CString>` | yes | yes — pointers are taken **after** every `push`, and a later `Vec` realloc would move only the `CString` headers, never the byte buffers they own | yes | yes | 1 | `CString` NUL ✅ |
| `envp` | `let envp: [usize; 1] = [0];`, a named local of `spawn` | yes | yes | yes | yes | 8 | single NULL element ✅ |
| `empty_path` | `let empty_path: [u8; 1] = [0];`, named local of `spawn` | yes | yes | yes | yes | 1 | one NUL = `""` ✅ |
| `default_action` | `let default_action = KernelSigaction::DEFAULT_DISPOSITION;`, named local | yes | yes | yes | yes | 8 | fixed 32-byte record ✅ |
| `empty_signal_mask` | `let empty_signal_mask: u64 = 0;`, named local | yes | yes | yes | yes | 8 | fixed 8 bytes ✅ |
| `status_record` | a `[u8; 8]` **field of `ChildPlan`**, not a pointer | yes | n/a | n/a | n/a | 1 | n/a |
| failure record write buffer | `let record: [u8; 8]` on the **child's own** stack in `fail` | n/a | yes | n/a | n/a | 1 | length 8 passed explicitly ✅ |
| injection read buffer | `let mut byte = [0u8; 1]` on the child's own stack | n/a | yes | n/a | n/a | 1 | length 1 ✅ |

**No dangling temporary.** Every one of the four constant addresses is taken from a **named `let`
binding** in `spawn`'s body, not from a temporary whose storage would end with the enclosing
statement. Storage lifetime runs to the end of `spawn`, and `spawn` cannot return before `clone3`
returns. The address escape is expressed with `expose_provenance()`, and the `asm!` block requests
no `nomem`/`readonly`, so the compiler must model the syscall as potentially reading any memory the
arguments name — it may not treat those slots as dead across the call.

No descriptor number is computed in the child; all six come from `plan_number(fd)` in the parent,
after unconditional relocation. No post-pointer `Vec` reallocation exists on any path.

**`P3_CHILDPLAN_INNER_POINTERS_SOUND`.**

## 11. Source-level closed-world child (level 1 evidence only)

[`child.rs`](../../crates/helm-launch/src/backend/child.rs) read token by token. It carries
`#![no_implicit_prelude]`, imports only `super::syscall` and `super::{ChildPlan, CloseSpan, stage}`,
and names none of `std`, `alloc`, `rustix`, `libc`, `Vec`, `String`, `Box`, `Rc`, `Arc`, `Mutex`,
`RefCell`, `format`, `print`/`println`/`eprintln`, `panic`, `unwrap`, `expect`, `unreachable`,
`todo`, `collect`, `iter`, `clone`, `drop`/`Drop`, `thread`, `Instant`, `Duration`, `sleep`,
`transmute`, `mmap`, `fork`, `vfork`, `pidfd_open`, `kill`, `Command` or `extern`. No `.unwrap(`,
`.expect(`, `panic!(`, `assert!(` or `unreachable!(` appears. No value in the child path has a
destructor; the entry point returns `!`.

Two constructs deserve explicit note, and both are copies of `Copy` data with no helper call in the
emitted code: `let [first, second, third] = &plan.close_ranges;` borrows rather than copying a
72-byte aggregate, and `let [_, pad1, pad2, pad3, _, _, _, _] = plan.status_record;` copies eight
bytes that lower to plain loads.

This is **level 1 evidence and is not sufficient alone** — precisely the lesson `P3R-02` recorded.
Section 12 is the load-bearing evidence.

## 12. The machine-code gate — reviewing the reviewer

[`tools/helm_launch_child_closure.py`](../../tools/helm_launch_child_closure.py) (452 lines) read in
full. Its 20 unit tests were **not** taken on trust.

**How it works, established by reading:**

| Question | Answer |
|---|---|
| Locating `child_main` | `find_unique("backend5child10child_main")`, anchored to the Itanium mangling hash marker `17h` |
| Symbol resolution | substring match over **defined** function bodies only; missing → `CheckerError` (exit 2); more than one → `CheckerError`. The `17h` anchor rejects compiler-generated `…28_$u7b$$u7b$closure…` siblings, tested |
| Function body extraction | non-`.L` label at column 0 starts a body; `.size` ends it; `.L…` labels are kept inside the current body |
| Direct call targets | `call`/`callq`/`jmp`/`jmpq` followed by a symbol, with `@PLT` / `@GOTPCREL` suffixes stripped; `.L…` targets skipped as local branches |
| Tail calls | `jmp sym` to a defined symbol is walked as an edge; `jmp sym@PLT` to an undefined symbol is an external edge |
| Relocations / PLT / GOT | handled by name before the linker, since the input is `--emit=asm`, not a linked image |
| Indirect calls | `call *…` with a non-symbol operand → counted and reported as a problem |
| Inlined syscall asm | `syscall` instructions counted per function; the shim being inlined is detected and accepted under its own rule |
| Crossing into unrelated functions | prevented by the `.size` boundary |
| Recursion / cycles | `seen` list guards re-entry |
| Transitive walk | breadth-first over every defined target |
| Duplicate symbol names | `setdefault` merges bodies, which over-approximates the edge set — conservative |
| Unknown external symbol | still a failure: `categorise` returns `"unclassified external reference"` and the edge is reported as a problem |
| Vacuous pass | impossible by four independent guards: missing root, empty root body, zero `syscall` instructions, and a failed positive control are each errors |

**Negative-direction capability, independently confirmed by running the suite (20 tests, OK):**
rejection of a `memcpy@PLT` edge in the child, a GOT-relative `core::panicking` edge, an edge
reached **only transitively** through `fail`, an unresolved `call *%rax`, a missing child root
(raises), an ambiguous symbol (raises), a child with no `syscall` at all, and an assembly with no
forbidden-category reference anywhere (positive control fails). A clean closure and an inlined-shim
closure both pass.

**Positive control on the real candidate.** The file-wide witness scan found 4 witness functions
reaching forbidden-category helpers in the debug assembly (`panic_const_div_by_zero`, three
`memcpy`) and 4 in the release-codegen assembly (`memcpy`, two `bcmp`, `_Unwind_Resume`), so a
clean child closure is a real result and not a parser that found nothing. The named control
`backend5spawn7prepare` reaches **143** external helpers in the debug assembly — the child's
sibling in the same module is full of them, and the child's closure has none.

**`P3R-11` (new, IMPORTANT): the checker does not fail closed on an indirect `jmp`.** See
section 19. It is the one place where the gate's own stated contract — "an unresolved indirect call
is an error… so 'zero matches' can never mean 'clean'" — does not hold.

### Debug / test-profile machine code — regenerated independently

Generated fresh in this session (not the author's saved output), and then analysed **twice**: once
with the committed checker and once with a parser written here that shares no code with it and that
deliberately treats every control-transfer instruction, indirect `jmp` included, as an edge or an
unresolved site.

```text
cargo rustc -p helm-launch --lib --locked --target x86_64-unknown-linux-gnu \
  --profile test -- --emit=asm
```

| Item | Result |
|---|---|
| Child root | `_ZN11helm_launch7backend5child10child_main17haee81f8dae299a2eE`, **unique** (1 label among 7 textual occurrences; 5 342 defined symbols in the file) |
| Closure size | **9** functions |
| Closure members | `child_main`, `fail`, `syscall::syscall6`, `syscall::is_error`, `syscall::errno_of`, `core::i64::unsigned_abs`, `core::TryFrom<u64> for i32::try_from`, `core::Result::unwrap_or_default`, `core::i32::to_le_bytes` — every one **defined in the same assembly** |
| Internal call edges | **12**, all listed and all inside the closure |
| External edges | **0** |
| Forbidden external runtime edges | **0** |
| Indirect call sites | **0** |
| Unparsed / other control transfers (independent parser) | **0** |
| `syscall` instructions | 1, in the out-of-line reviewed shim; **no other closure function issues one** |
| Explicit search for `memcpy`, `memmove`, `memset`, `memcmp`, `bcmp`, `strlen`, `malloc`, `calloc`, `realloc`, `free`, `__rust_alloc*`, `handle_alloc_error`, `panic*`, `core::panicking`, `std::panicking`, `unwrap_failed`, `expect_failed`, `_Unwind*`, `rust_eh_personality`, `pthread*`, `futex`, `__tls_get_addr`, `errno_location`, `abort`, `exit`, PLT helpers | **none reachable** |

The four `core` functions in the closure are monomorphised into this crate's own codegen unit and
their bodies contain no external edge; had they been in another unit they would have surfaced as
external edges and failed the gate.

**`P3_DEBUG_CHILD_MACHINE_CLOSED_WORLD_SOUND`.**

### Release-codegen machine code — and whether it is DCE-only

The reviewed question is not "does a release library contain the backend" but "is the real child
implementation instantiated under release codegen and is its closure clean".

First, the DCE fact was confirmed independently: `cargo rustc --lib --profile release` emits **no**
`child_main` at all, and the checker then **fails closed** with
`CHILD CLOSURE CHECK ERROR: no function named 'backend5child10child_main' is defined in the
assembly` (exit 2). A plain release library therefore cannot yield a vacuous pass.

The correction obtains release codegen over the instantiated child by applying release codegen flags
to the profile that does instantiate it:

```text
cargo rustc -p helm-launch --lib --locked --target x86_64-unknown-linux-gnu \
  --profile test -- --emit=asm -Copt-level=3 -Cdebug-assertions=off
```

Verified here from cargo's own verbose invocation: `--profile test` makes cargo pass `--test`, so the
lib is built as its test harness and the backend is reached through `launch_minimal`; cargo passes
**no** `-Copt-level`, `-Cdebug-assertions` or `-Coverflow-checks` of its own for that profile, so the
appended flags are decisive and overflow checks follow `debug-assertions` to **off**. The result is
the product source's own `child_main`, at `opt-level=3` with debug assertions and overflow checks
off. It is not a dummy copy of the algorithm: it is the same function, uniquely identified by
symbol, reached from the same call site.

| Item | Result |
|---|---|
| **RELEASE CHILD BACKEND ACTUALLY INSTANTIATED** | **YES** |
| Child root | the same `…child_main17haee81f8dae299a2eE`, **unique** (674 defined symbols) |
| Closure size | **2** functions — `child_main`, `fail` (the shim, `is_error`, `errno_of` and the `core` helpers are inlined) |
| Internal call edges | **1** (`child_main → fail`) |
| External edges | **0** |
| Forbidden external runtime edges | **0** |
| Indirect call sites | **0** |
| Unparsed / other control transfers (independent parser) | **0** |
| `syscall` instructions | **18** — 16 in `child_main`, 2 in `fail` |
| **RELEASE PROOF NOT DCE-ONLY** | **YES** |

The 18 syscalls are exactly the closed contract and nothing else. Read out of the emitted code in
order: `33, 33, 33` (`dup2` ×3), `72, 72, 72` (`fcntl` `F_SETFD` ×3 — `%rsi` still holding 2 from
the preceding `dup2`, correct), `81` (`fchdir`), `436, 436, 436` (`close_range`, each guarded by
`cmpq $0, used`), `109` (`setpgid(0,0)`), `13` (`rt_sigaction` inside a loop
`%rdi = 1 … 64`, `je` past 9 and 19, `%rsi = default_action`, `%rdx = 0`, `%r10 = 8`), `14`
(`rt_sigprocmask`, `%rdi = 2`, `%rsi = empty_signal_mask`, `%r10 = 8`), `157` (`prctl(38, 1, 0,0,0)`),
`322` (`execveat`, `%rdi = executable`, `%rsi = empty_path`, `%rdx = argv`, `%r10 = envp`,
`%r8 = 4096 = AT_EMPTY_PATH`), and `231` (`exit_group(127)`); `fail` issues `write` and
`exit_group`. **There is no hidden system call in the normal machine closure**, and no stack-protector
or profiling helper.

Caveats stated honestly: this is release **codegen**, not the literal `release` profile, and the
remaining differences are `-Cdebuginfo=2`, incremental compilation, and `cfg(test)`. None of them can
remove a call to an external helper — less inlining produces *more* calls, not fewer — and `child.rs`
contains no `cfg(test)`, so the analysed body is the product body. The gap is a consequence of P3
having no public consumer, which is exactly the situation the owner's instruction anticipated.

### The injection-enabled child — not covered by CI, checked here

CI runs the machine proof only without features, yet also runs
`cargo test --features test-fault-injection`. Regenerated and analysed here:

```text
… --profile test --features test-fault-injection
```

closure **10** functions (the nine above plus `injection::before_exec`), internal edges 14, **external
edges 0**, indirect sites 0. Clean, but unproven by CI — recorded as `P3R-13`
(BACKLOG_NONBLOCKING).

## 13. Machine code versus strace

The two-gate boundary is stated where it is load-bearing: the workflow header states both gates and
that neither replaces the other, and the checker's own module docstring repeats it
(`MACHINE-CODE GATE no forbidden userspace runtime helper in the child closure` /
`STRACE GATE no forbidden syscall in the runtime child window`). Neither is used to substitute for
the other anywhere in the tests. The crate README does **not** mention the machine-code gate at all
— folded into `P3R-09`.

Runtime strace remains **`GATE_PENDING`** before publication.

## 14. Child stage and syscall contract

Reviewed as actual helper control flow, not as a stage enum, in source and in emitted code. The one
order, with no configuration branch: `DUP2` ×3 → `CLEAR_CLOEXEC` ×3 → `CHDIR` → `CLOSE_RANGE` (≤3,
each skipping an unused span) → `SETPGID` → `SIGACTION` ×62 → `SIGMASK` → `NO_NEW_PRIVS` → `EXEC`.
`CHDIR` precedes `CLOSE_RANGE` because the working-directory descriptor is not preserved by the
range close. `SIGACTION` precedes `SIGMASK` so no inherited host disposition can run before the
reset. `close_span` reports the `CLOSE_RANGE` stage and the entry point never issues a range close
directly. On failure: one 8-byte record, then `exit_group(127)`.

## 15. `close_range` and the descriptor contract

`prepare` relocates **all six** child-side descriptors with `F_DUPFD_CLOEXEC` to at least 3 and
closes each original — unconditionally, with no "already high enough" shortcut — so no later `dup2`
source can equal its target even when the host has 0, 1 or 2 closed, and the six are distinct
because each duplicate is taken while its original is still open.

`layout::plan_child_layout` then plans the gaps around the two preserved numbers
(`executable`, `status_write`), refusing non-relocated or non-distinct inputs, skipping inverted
gaps — exactly the adjacent (F7) and lowest-possible cases — and producing at most three ascending,
non-overlapping ranges that never contain a preserved number, with coverage of every number ≥ 3
checked at the interesting values and range boundaries. The working-directory descriptor is
deliberately **not** preserved and is closed by the range after `fchdir`. The original relocated
copies of the three stdio descriptors are also closed by the range, while 0, 1 and 2 are below
`FIRST_RELOCATED = 3` and thus in no range. After exec the image holds only 0, 1 and 2; the
executable and exec-status descriptors survive the range close but are close-on-exec, so a
successful exec closes them. No pathname fallback anywhere.

## 16. Exec status record

Exactly 8 bytes: stage byte, three zero pad bytes kept from the record the parent prepared, and
`errno` as little-endian `i32`. Below `PIPE_BUF`, and the status write end is **blocking** (only the
three parent read ends are made non-blocking), so the single write is atomic and a reader sees
nothing or the whole record. Exactly one semantic attempt, retried on `EINTR` and on nothing else.

`classify_record` reads into a **nine**-byte buffer so an over-long record is detectable rather than
silently truncated, and produces `Indeterminate` for: nothing before EOF, a short record, more than
eight bytes, an unknown stage byte, and a non-zero reserved pad. Clean EOF with no record is
`StatusEofWithoutRecord` — **indeterminate**, never success. There is no `ExecSucceeded`,
`Launched`, `Started` or `Ran` value anywhere in the crate, and the crate root carries
`compile_fail` doctests proving `ExecStatus` has no `bool` conversion and no `is_success`.

## 17. `execveat` authority

The child's only exec is `execveat(plan.executable, empty_path, argv, envp, AT_EMPTY_PATH)` where
`plan.executable` is the relocated duplicate of the descriptor the caller moved into
`admit_executable` and `authorize` consumed, carried through `AuthorizedParts` without re-measuring,
reopening or re-resolving. `envp` is a one-element array whose only element is NULL. Confirmed in
emitted code (section 12). No reopen, no path resolution, no `/proc/self/fd`, no `fexecve`, no
`PATH`, no substitution after authorisation; `tests/p3_boundary.rs` fails on a `"/proc` string
literal, on `execve(` and on `fexecve` in any backend file.

## 18. Direct-child cleanup

`pidfd_send_signal(pidfd, SIGKILL)` at exactly one site; `waitid(WaitId::PidFd, EXITED | NOHANG)`
only; a bounded polling loop with a `CLOCK_MONOTONIC`-based deadline and a 1 ms sleep; `Drop` sends
one `SIGKILL` and reaps within `POST_KILL_REAP_MS` unless the end is already latched. No `SIGTERM`,
no grace period, no `waitpid`/`wait4`, no numeric-pid signal, no group signal, no blocking wait, no
`poll` (the `rustix` `event` feature is deliberately not enabled).

`EINTR` from the non-blocking wait establishes nothing and returns `None` for the bounded caller to
retry. Any other error latches `EndUnobservable` **and** sets `reaped`, so a later `Drop` cannot
signal a process this handle no longer owns — the same latch discipline the owner ruled for P1.

The no-zombie claim is tested, not asserted: the S6 case requires
`child.observed_end() == Signaled { signal: 9, core_dumped: false }`, and `complete()` asserts the
handle latched the end it observed.

## 19. New findings

### `P3R-10` — IMPORTANT — the fixture report key and its parser key disagree, so a load-bearing Linux backend test cannot pass

**Reachable: YES, on every Linux run of the default backend suite.**

The report fixture emits the direct child's thread-group id under the key `pgid_is_self`:

```rust
// crates/helm-launch/src/backend/tests.rs:466
out.push_str(&format!("pgid_is_self={}\n", status_field("Tgid")));
```

`parse_report` has no arm for `pgid_is_self`; it has one for `tgid`, which the fixture never emits:

```rust
// crates/helm-launch/src/backend/tests.rs:537
"tgid" => report.tgid = value.parse().expect("tgid"),
```

`pgid_is_self` therefore falls into the `_ => {}` arm and `report.tgid` keeps its `Default` value of
`0` on every run. The value is then asserted against the real child pid:

```rust
// crates/helm-launch/src/backend/tests.rs:752-757
assert_eq!(
    i64::try_from(report.tgid).unwrap(),          // always 0
    completed.launch.child.pid(),                // the clone3 return value, > 0
    "the reporting image is not the direct child"
);
```

`child.pid()` is the `clone3` return value in the parent, which is non-zero by construction (zero is
the child branch). The assertion compares `0` to a real pid and **fails**.

Consequences:

* `backend::tests::an_authorised_object_executes_with_exactly_the_intended_descriptors` — the
  descriptor-isolation case, the most load-bearing default-backend test — **cannot pass on Linux
  x86_64**. `cargo test -p helm-launch --locked` fails, so both `helm-launch.yml` and
  `helm-evidence.yml` would be red on the first Ubuntu run.
* The claim "the image that reported is the direct child the backend created, not a descendant of
  one" is **not established** by any working assertion. The crate README lists "the direct child's
  identity" among the properties the P3 suite covers; that documentation claim is currently backed
  by a failing test.
* `sig_blk`, `sig_ign`, `sig_cgt`, `argv_count`, `env_count` and `fd_count` are also emitted and
  ignored, but no assertion depends on them, so only `tgid` is defective.

The defect survived local validation because the backend is gated to
`cfg(all(target_os = "linux", target_arch = "x86_64"))`: on Windows and macOS the test is not
compiled, and Linux cross-`clippy` type-checks it without running it. It fails **closed** — it
cannot produce false evidence — but it does block publication.

Not fixed here: this review makes no code change.

### `P3R-11` — IMPORTANT — the child-closure checker silently drops indirect `jmp` control transfers

**Reachable: NO today; latent in a load-bearing gate.**

The owner's requirement is explicit: an unknown or unparseable call instruction must fail closed, an
ambiguous symbol must fail closed, and no edge may be silently dropped. The checker meets this for
`call` but not for `jmp`:

```python
# tools/helm_launch_child_closure.py:99-100
DIRECT_CALL   = re.compile(r"^\s*(?:call|callq|jmp|jmpq)\s+\*?([A-Za-z_$.][\w$.@]*)")
INDIRECT_CALL = re.compile(r"^\s*(?:call|callq)\s+\*(?![A-Za-z_$.])")
```

`INDIRECT_CALL` accepts only `call`/`callq`, so `jmpq *%rax`, `jmp *(%rax)` and
`jmpq *table(,%rax,8)` are not counted as indirect sites. They also fail `DIRECT_CALL`, whose
`[A-Za-z_$.]` class rejects `%` and a digit after the optional `*`. The line matches neither pattern
and is **discarded without record**: not an edge, not an indirect site, not a problem. An indirect
tail call out of the child closure would therefore pass the gate silently, and the checker's own
docstring promise — that "zero matches" can never mean "the parser found nothing" — does not hold
for that instruction class. There is no unit test for it; the indirect-call test uses `callq *%rax`.

Independently bounded: a parser written for this review, which treats every control transfer
including indirect `jmp` as an edge or an unresolved site, reports **0 indirect sites and 0 unparsed
control transfers** in all three analysed profiles (debug, release codegen, debug + injection). The
shape that would exploit the hole is a tail call through a function pointer, and the source contains
no function pointer on the child path. Direct tail calls to external helpers — `jmp memcpy@PLT`, the
realistic escape — **are** caught, because their operand carries no `*`.

Rated IMPORTANT because the gate is the sole evidence for the corrected `P3R-02` property, was
installed precisely because source-level scanning could not see compiler-inserted code, and its
durability against a future compiler or source change depends on failing closed. The owner may
reasonably re-rate it MINOR on the strength of the reachability result above; this review is not
entitled to close it against the explicit fail-closed instruction.

### `P3R-12` — MINOR — cached fixtures and the preloaded helper live in a shared temporary directory

`fixture_root()` is `std::env::temp_dir()/helm-launch-p3-fixtures`. `fixture_binary` and
`atfork_helper` are content-addressed by a SHA-256 prefix of their source text but return the cached
file **without verifying its contents**, and the atfork helper is then `LD_PRELOAD`ed into the traced
launcher process. On a fresh CI runner this is immaterial; on a shared developer host a file placed
at a predictable path in `/tmp` would be executed or preloaded. Test hygiene, not a product defect.

### `P3R-13` — BACKLOG_NONBLOCKING — CI does not machine-prove the injection-enabled child

CI runs `cargo test --features test-fault-injection` but runs the machine-code proof only without
features, so the child closure of the build that the injection tests actually execute is not gated.
Verified clean by this review (section 12): closure 10, external edges 0.

### `P3R-14` — BACKLOG_NONBLOCKING — the release-library backend-absence step reads only the first emitted assembly

```bash
asm=$(find "$out" -name 'helm_launch-*.s' -print -quit)
… if grep -q 'backend5child10child_main' "$asm"
```

`-print -quit` takes one file. The step asserts **absence**, so more than one emitted unit could
make it a false negative. `--emit=asm` currently produces exactly one matching file for that build
— the closure checker refuses to run on more than one, and this review confirmed the count — so the
risk is latent. The closure checker's own `len(emitted) > 1` guard is the stricter pattern.

## 20. Inherited findings, re-evaluated rather than inherited

| ID | Re-evaluated verdict |
|---|---|
| `P3R-01` | **INDEPENDENTLY VERIFIED FIXED** — section 21 |
| `P3R-02` | **INDEPENDENTLY VERIFIED FIXED** — sections 9, 10, 12 |
| `P3R-03` | **MINOR, OPEN, not promoted.** `PidfdNotProvided` confirmed unreachable under the `CLONE_PIDFD` kernel contract; the accepted contract is sufficient and **no numeric-pid fallback is recommended** |
| `P3R-04` | **MINOR, OPEN, not promoted.** `assert_closed_child_window` requires `execveat` with result `0`, so the failure-path child window is never trace-verified. Narrower after the correction: the machine-code gate now covers `fail`'s own closure (`write` + `exit_group`, 2 syscalls, 0 external edges), and four integration cases assert exact `(stage, errno)` records |
| `P3R-05` | **MINOR, OPEN, not promoted** — section 22. The registration control makes the test probative, so the lack of a handler-fire control does not defeat it |
| `P3R-06` | **MINOR, OPEN, not promoted.** The trace pins the restore as `parent[at + 2]`, tighter than "after the setpgid attempt". Confirmed correct for the current implementation: nothing between `setpgid` and `restore_mask` issues a system call |
| `P3R-07` | **MINOR, OPEN.** Confirmed from the workflow: the backend suite runs three times, each paying the intentional 5 s S6 bound and re-running the three tracer executions. The 30-minute job timeout is realistic for it |
| `P3R-08` | **MINOR, OPEN.** `require_tool("env", …)` runs `env --version`; GNU-specific, satisfied on `ubuntu-24.04`, and a loud environment failure elsewhere, which is correct |
| `P3R-09` | **MINOR, OPEN, widened.** [`crates/helm-launch/README.md`](../../crates/helm-launch/README.md) still says "a CI step greps the release artifact for a marker to prove it", which the correction replaced; and the README does not mention the machine-code gate at all. Both understate the evidence rather than overstating it. **Not edited here** |
| `P3R-00` | Satisfied by this document: the independence precondition is met |

## 21. `P3R-01` — artifact selection and the injection proof

[`tools/helm_launch_injection_proof.py`](../../tools/helm_launch_injection_proof.py) (287 lines) read
in full and **run independently** against a cross-compiled Linux target.

**Fresh roots.** `build()` does `shutil.rmtree(target_dir)` before each case, so no artifact from a
different feature set can be inspected. The two cases use separate roots under `--root`.

**Machine-readable selection.** `cargo build … --message-format=json-render-diagnostics`, and the
artifact is taken from the `compiler-artifact` record of **that invocation**, filtered on
`target.name == "helm_launch"` **and** `"lib" in target.kind` **and** `"crates/helm-launch" in
package_id` (path-separator normalised), then on `.rlib`/`.rmeta` suffix. No glob, no `head`, no
`tail`, no directory listing, no path convention. An empty selection raises.

More than one artifact is permitted rather than rejected, but the semantics are conservative in the
safety direction: all selected artifacts are inspected and the negative proof fails if **any** of
them contains the marker.

**Archive parsing.** The `ar` reader requires `!<arch>\n` (so a thin archive or a non-archive fails
closed), validates the backtick-newline terminator of every 60-byte header, parses decimal sizes with
a failure on garbage, honours the even-byte padding, decodes GNU long names through the `//` string
table, skips the `/` and `/SYM64/` index members, and raises if no inspectable member remains. Every
member is scanned, not the first — proven by the committed tests (marker in a late member) and
exercised on the real artifact, whose 128 members carry names far longer than 16 bytes. BSD `#1/N`
names are not decoded, but such a member's body is still scanned, so no content is skipped.

**Independently reproduced result** (`--target x86_64-unknown-linux-gnu`):

| Case | Artifact | Members | Marker |
|---|---|---|---|
| debug + `test-fault-injection` (positive control) | `…/debug/libhelm_launch.rlib`, 4 323 136 bytes | **128** inspected | **PRESENT** in `lib.rmeta` **and** in `helm_launch-67242ca42bb19bcd.3reb1vmpi3flgkc21n7rss7d0.0kwngu9.rcgu.o` |
| debug + feature | `…/debug/deps/libhelm_launch-67242ca42bb19bcd.rmeta` | 1 | PRESENT |
| **release + `--all-features`** (negative proof) | `…/release/libhelm_launch.rlib`, 1 037 276 bytes | **11** inspected | **ABSENT** |
| release + `--all-features` | `…/release/deps/libhelm_launch-b4978278e2e77b87.rmeta` | 1 | ABSENT |

`INJECTION PROOF PASSED`. The positive control hits a real **object** member, not only metadata, so
the marker witnesses compiled code and not merely a string in an `.rmeta` blob.

**The DCE-versus-`cfg` distinction, established with its own experiment.** The marker is
`#[used] static PRESENCE_MARKER_KEPT` inside `injection.rs`, whose `mod injection;` declaration
carries `#[cfg(all(feature = "test-fault-injection", debug_assertions))]`, and `#[used]` defeats
dead-code elimination for anything actually compiled. To test that rather than assume it, this review
built the **release profile** with the feature enabled and `-Cdebug-assertions=on`:

```text
cargo rustc -p helm-launch --lib --locked --release --features test-fault-injection \
  --target x86_64-unknown-linux-gnu -- -Cdebug-assertions=on
```

The marker is **PRESENT** in a real object member of that release rlib
(`helm_launch-63cbd1761ceaa415.helm_launch.b65edb0ff2b5a80b-cgu.07.rcgu.o`, 11 members), even though
the backend is otherwise dead code in a release library. The differential therefore isolates the
cause of the standard release build's absence to `debug_assertions = off` — the `cfg` gate — and
**not** to dead-code elimination. That is the structural claim plus artifact evidence the owner's
instruction required, and it separates claim A (injection absent in release) from claim B (backend
absent from an unused release library) instead of using B as evidence for A.

The structural half is also machine-checked in source: `tests/p3_boundary.rs` requires that **every**
`feature = "test-fault-injection"` condition in the crate is the full two-condition gate, in exactly
the four files that carry one, that `mod injection;` is declared behind it, that the `cfg(not(...))`
complement exists exactly once, that `#[used] static PRESENCE_MARKER_KEPT` is present, that the
marker spelling is the one CI looks for, and that the feature is declared and non-default.

**`P3R-01` INDEPENDENTLY VERIFIED FIXED.**

## 22. S5, S6, fault injection and the atfork test

**S5 reconfirmed.** `MODE_EXIT_BEFORE_EXEC` ends the child with status **0** before `execveat`,
writing no record. The parent observes `Indeterminate(StatusEofWithoutRecord)`, `sigkill_sent ==
false`, `Exited { code: 0 }` and an empty stdout. That is exactly the shape a successful exec shows,
and the product reads it as indeterminate. Probative.

**S6 topology established independently, not inherited.** Writer ownership at the stall point was
traced through the code rather than assumed:

1. `prepare` creates the stdin pipe; the read end is relocated to ≥ 3 and the write end is not.
2. `spawn` clones; the child's descriptor table holds copies of **both** ends.
3. The child's stage 1 binds the read end to 0; **stage 4's `close_range` closes the child's copy of
   the write end**, because it is ≥ 3 and is not one of the two preserved numbers.
4. `release_child_side(retain_stdin_writer)` runs in the parent after the clone and, for this mode
   only, keeps the parent's write end, moving it into `MinimalLaunch.stdin_write`.

So at the stall point there is **exactly one writer, the parent's retained end, and it is held for
the whole pre-exec window**. The child's blocking `read(0)` therefore cannot reach end-of-file:
the stall is **deterministic, not race dependent**. It is dropped afterwards when `MinimalLaunch` is
consumed. In a normal product build the retention is a compile-time `false` constant, and
`tests/p3_boundary.rs` fails if the gated and complement definitions are not both present. The test
asserts `PreExecStatusTimeout`, `sigkill_sent`, `Signaled { signal: 9 }` as the latched end, and
that the elapsed time is **at least** `SPAWN_CONFIRM_TIMEOUT_MS`, so the bound is really waited out.
Probative.

**Stage failures.** `every_child_stage_can_report_its_own_structured_failure` walks all nine stages,
each reporting the injected `errno` 42 as `PreExecFailure { stage, errno }` with `Exited { code: 127 }`.

**The atfork test is probative.** `LD_PRELOAD` is applied **inside** the tracer through `env(1)`,
which execs without forking, so the preload first exists in the launcher process and not in
`strace`, whose own `fork(2)` would otherwise fire the very handlers under test. The helper contains
only `pthread_atfork` registration and a `note()` that appends to a path from the environment — no
launcher code. Registration is proven by a positive control (`registered` must exist), and because
`note()` is the **same function** that would record a handler firing, the control also proves
`getenv` + `open` + `write` work in that process, so the absence of `ran` is not vacuous. If a child
handler had run it would have run immediately after `clone3`, with the environment and descriptor
table still intact, and would have been detected. `P3R-05` is therefore **not promoted**.

The threaded mode keeps three allocating threads alive across `clone3`, synchronised with an
`AtomicBool` and joined afterwards, with a 50 ms warm-up. The atfork case runs in single-threaded
mode only; the combination is untested, which is a coverage nicety, not a defect.

## 23. Exec failure fixtures and the fixture report contract

| Case | Construction | Independent assessment |
|---|---|---|
| `EACCES` (13) | copy of the fixture at mode `0600`, with the mode asserted | Passes P2 admission, which **records** `st_mode & 0o7777` and never enforces the execute bit, so the child reaches `execveat` and the kernel refuses. Correct |
| `ENOEXEC` (8) | `e_phentsize` at byte offset 54 zeroed, with the original value asserted to be 56 first | Offset 54 is `e_phentsize` in `Elf64_Ehdr` and `sizeof(Elf64_Phdr)` is 56 — both confirmed independently. `load_elf_binary` returns `-ENOEXEC` for `e_phentsize != sizeof(struct elf_phdr)`, and the magic, class, encoding, machine and type the cohort check reads are untouched, so admission still accepts it. Deterministic for the intended kernel reason |
| `ETXTBSY` (26) | the test holds a write handle on the copy | `deny_write_access` fails at exec. The product never opens the object for writing; admission opens read-only. Correct |
| `CHDIR: EACCES` (13) | the admitted directory is chmod `000` after admission | Handles the privileged-host case explicitly by requiring `geteuid().is_root()` when the child instead reaches exec, so it is neither flaky nor a false pass |
| exact descriptor | pathname unlinked and replaced after admission | The admitted inode still runs, proven by the report marker |

Every case asserts the exact `PreExecFailure { stage, errno }` record. **Nothing is inferred from
exit status 127** — the exit code is asserted *in addition to* the record, never instead of it.

**Fixture report contract.** `report_fixture_reports_without_the_backend` runs the fixture directly
through `std::process::Command`, parses its output, and checks the versioned marker
(`HELM-LAUNCH-P3-FIXTURE/1`), the `end` completeness line, the fixture name, `argv`, a **non-empty**
inherited environment (so the later empty-environment assertion is not vacuous) and the stderr
marker. That is the X2c rule, and no impossible evidence is expected anywhere.

Two observations. The Rust harness gives no ordering guarantee that the producer self-test runs
before a consumer, so "precedes" holds for the suite as a whole rather than per-execution — a
broken producer still fails the suite, so no false evidence can arise. And a fixture **cache hit
does not bypass producer validation**: the self-test executes the cached binary on every run. The
cache location itself is `P3R-12`.

## 24. Public API

Inventory taken independently and diffed against the accepted P2 base: **no newly public item
exists**. Public surface is P1 and P2 only — `parse_launch_plan`, `ValidatedLaunchPlan`, the plan
limits, the error and refusal vocabularies, the fact enums, `LaunchReceipt`, and on the Linux x86_64
cohort `admit_executable`, `admit_working_directory`, `authorize`, the two capability types,
`AuthorizedLaunch` and `MAX_EXECUTABLE_BYTES`.

The P3 backend is `mod backend;` — private, `cfg`-gated, re-exported nowhere, with no bare `pub`
item in any of its five files. `AuthorizedLaunch` remains **not consumable by an external caller**:
`into_parts` and `AuthorizedParts` are `pub(crate)`, `AuthorizedLaunch` has no `Clone`, and the
crate root carries `compile_fail` doctests for `helm_launch::launch`, `LaunchOutcome`, `backend`,
`SpawnedChild`, `PreparedLaunch`, `ChildHandle`, `MinimalLaunch`, `Fault`,
`helm_launch::backend::launch_minimal`, `AuthorizedLaunch::into_parts` and
`ExecutableCapability::descriptor`.

## 25. CI design

The committed `helm-launch.yml` (30-minute bounded timeout, `permissions: contents: read`,
concurrency cancellation, three-platform matrix, pinned `actions/checkout` by SHA,
`persist-credentials: false`) does run every load-bearing proof on Ubuntu:

| Required | Step | Result |
|---|---|---|
| source confinement | `cargo test --test p3_boundary`, `--test p2_boundary` (all platforms) + `python3 -m unittest tools.tests.test_helm_launch_confinement` | ✅ |
| debug machine-code proof | `helm_launch_child_closure.py --profile test` | ✅ |
| release machine-code proof | same tool with `-Copt-level=3 -Cdebug-assertions=off` | ✅ |
| injection positive control | `helm_launch_injection_proof.py` (debug + feature case) | ✅ |
| release all-features injection absence | same tool (release + `--all-features` case) | ✅ |
| default backend tests | `cargo test -p helm-launch --locked` and `--lib -- backend::tests::` | ✅ runs, ❌ **fails** — `P3R-10` |
| feature S5/S6 tests | `cargo test --features test-fault-injection` (twice) | ✅ |
| strace tests | inside the backend suite; `strace --version` pre-checked | ✅ |
| missing tool | `strace --version`, `cc --version`, `rustc --version` under `set -eux`, plus `require_tool` panicking with `TEST ENVIRONMENT FAILURE` | ✅ never a silent skip |
| off-cohort | admission suite listed empty; a backend test must not exist (independently confirmed: **0** on Windows) | ✅ |
| no Trial workflow | none added; nothing dispatched | ✅ |

No accidental shell false result was found in the added steps: each is a single command under
`bash -e`, the `grep -q` in the release-absence step sits inside an `if` condition so `set -e` does
not abort on a non-match, and the empty-`$asm` case is an explicit environment failure. `P3R-14`
records the one `-print -quit` narrowing. `helm-evidence.yml`'s change is a comment plus the same
commands.

## 26. Local validation performed by this review

Fresh target roots throughout; nothing reused from the author's output.

| Command | Result |
|---|---|
| `cargo fmt --check` | **pass** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **pass** |
| `cargo test --workspace --locked` | **pass** — 20 suites, 0 failures |
| `cargo test -p helm-launch --locked` | **pass** — 42 + 19 + 17 + 19 + 23 + 2 + 0 |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **pass** (identical set off the cohort) |
| `cargo build -p helm-launch --release --locked` | **pass** |
| `cargo clippy -p helm-launch --all-targets --locked --target x86_64-unknown-linux-gnu -- -D warnings` | **pass** |
| `cargo clippy -p helm-launch --all-targets --all-features --locked --target x86_64-unknown-linux-gnu -- -D warnings` | **pass** |
| `python -m unittest discover -s tools/tests` | **pass** — 814 tests, 74 skipped |
| `python -m unittest tools.tests.test_helm_launch_machine_proofs` | **pass** — 20 tests |
| `python -m unittest tools.tests.test_helm_launch_confinement` | **pass** — 10 tests |
| `python tools/validate_docs.py` | **pass** |
| `git diff --check` | clean |
| Linux x86_64 assembly regenerated and both machine proofs re-run (debug, release codegen, debug + injection) | **pass**, and re-analysed with an independent parser |
| Injection proof re-run cross-compiled, plus the release + `-Cdebug-assertions=on` differential | **pass** |
| `--profile release --lib` closure attempt | **fails closed** as designed (exit 2) |

`rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0`, Python 3.14.3, host
`x86_64-pc-windows-msvc`, target `x86_64-unknown-linux-gnu` installed. No Rust toolchain was
installed in WSL.

**Not performed here, and not claimable:**

* **P3 LINUX RUNTIME VALIDATION: `PENDING PUBLICATION CI`.** No Linux process was created or
  executed by this review. `P3R-10` predicts a failure there.
* **P3 STRACE CHILD-WINDOW VALIDATION: `PENDING PUBLICATION CI`.** `strace` was not run.

## 27. Findings

| ID | Severity | Area | Reachable | Summary | Disposition |
|---|---|---|---|---|---|
| `P3R-01` | — | CI proof / artifact selection | — | Exact `compiler-artifact` selection, fresh roots, every archive member inspected, positive control, and a `cfg`-versus-DCE differential that this review reproduced | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-02` | — | child closed world | — | Borrowed `ChildPlan`, no whole-record copy, no raw-pointer dereference; 0 external runtime edges in debug, release codegen and injection builds, re-derived here | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-10` | **IMPORTANT** | Linux backend test | **yes** | The fixture emits `pgid_is_self` and `parse_report` handles only `tgid`, so `report.tgid` is always `0` and `assert_eq!(0, child.pid())` at `tests.rs:754` fails; the descriptor-isolation case cannot pass on Linux and the "direct child identity" claim is unproven | **OPEN — must be corrected before publication** |
| `P3R-11` | **IMPORTANT** | machine-code gate | no (latent) | `INDIRECT_CALL` matches only `call`/`callq`, so an indirect `jmp *…` is neither an edge nor an unresolved site and is discarded without record, against the explicit fail-closed requirement; 0 such instructions exist today, confirmed with an independent parser | **OPEN — owner decision on rating** |
| `P3R-03` | MINOR | pidfd residual | no | `PidfdNotProvided` unreachable under the kernel contract; no numeric-pid fallback recommended | OPEN, carried |
| `P3R-04` | MINOR | trace coverage | n/a | No traced exec-failure window; narrowed by the machine-code gate covering `fail` | OPEN, carried |
| `P3R-05` | MINOR | atfork probity | n/a | No handler-fire positive control; the registration control keeps the test probative | OPEN, **not promoted** |
| `P3R-06` | MINOR | trace brittleness | n/a | The restore is pinned as the second syscall after `clone3`, tighter than the contract | OPEN, carried |
| `P3R-07` | MINOR | CI cost | n/a | The backend suite runs three times, each paying the 5 s S6 bound and the tracer executions | OPEN, carried |
| `P3R-08` | MINOR | host assumption | n/a | `env --version` is GNU-specific; a loud environment failure elsewhere | OPEN, carried |
| `P3R-09` | MINOR | documentation | n/a | The README still describes the grep-based injection proof and does not mention the machine-code gate; both understate the evidence. Not edited here | OPEN, carried |
| `P3R-12` | MINOR | test hygiene | no | Content-addressed fixtures and the `LD_PRELOAD`ed atfork helper are cached in a shared temporary directory and reused without content verification | OPEN, new |
| `P3R-13` | BACKLOG_NONBLOCKING | proof coverage | n/a | CI does not machine-prove the injection-enabled child; verified clean here | OPEN, new |
| `P3R-14` | BACKLOG_NONBLOCKING | CI robustness | no | The release-absence step reads only the first emitted assembly (`-print -quit`) | OPEN, new |
| — | GATE_PENDING | Linux runtime | — | No Linux process created or executed by this review | PENDING PUBLICATION CI |
| — | GATE_PENDING | strace child window | — | `strace` not run | PENDING PUBLICATION CI |

## 28. Core safety verdicts

| Area | Verdict |
|---|---|
| Clone ABI | **SOUND** — `repr(C)`, size 88, every offset pinned and independently checked; `CLONE_PIDFD` alone; `exit_signal = SIGCHLD`; no `VM`/`VFORK`/`FILES`/`THREAD`; `stack = 0` with `stack_size = 0` valid |
| pidfd | **SOUND** — from the creating call, adopted immediately, guarded, no `pidfd_open`, no fallback; pid recorded and never signalled |
| Signal ABI | **SOUND** — full 64-bit kernel set, `sigsetsize = 8`, correct bit numbering, glibc's signals 32/33 included, `SIGKILL`/`SIGSTOP` never claimed blockable; raw `sigaction` layout with the mask **last**; `SIG_DFL` without `SA_RESTORER` verified against the kernel's delivery path |
| `ChildPlan` / reference | **SOUND** — borrowed, aligned by construction, live for the whole child window, unaffected by parent activity across copy-on-write, no invalid compiler assumption across the fork-like clone, plain loads only in emitted code |
| Inner pointers | **SOUND** — all nine traced to named locals or to stable heap buffers, prepared before the clone, correctly terminated, no dangling temporary, no post-pointer reallocation |
| Child stage order | **SOUND** — verified in source and in emitted machine code, 18 syscalls, nothing hidden |
| Descriptor isolation | **SOUND** — unconditional relocation, gap-based range close that preserves exactly two numbers, `fchdir` before the close, only 0/1/2 in the image, no pathname fallback |
| `execveat` authority | **SOUND** — the relocated admitted descriptor, empty path, `AT_EMPTY_PATH`, `envp = [NULL]`, no reopen, no procfs, no `PATH` |
| Cleanup | **SOUND** — pidfd `SIGKILL` only, non-blocking bounded reap, latched end, no `SIGTERM`, no group signal, no blocking wait |
| Fault injection | **SOUND** — two-condition gate machine-checked in source, absence in release proven by `cfg` and confirmed by artifact differential |
| strace parser | **SOUND and probative** — `-o` output with bare pid prefixes, unfinished/resumed splicing, exactly one `CLONE_PIDFD` non-`CLONE_THREAD` clone, exact parent window, permitted-set and exact-count child window (62 `rt_sigaction`), `fchdir` before `close_range`, `AT_EMPTY_PATH` with an empty pathname, no group signal, `P_PIDFD` waits only, and a failure on any unknown syscall in the window |
| Public API | **SOUND** — byte-identical to the accepted P2 base; nothing new is public |
| CI | **SOUND in structure**; it executes every load-bearing proof, and `P3R-10` will make it red |

## 29. Recommendation

**`P3 CANDIDATE REQUIRES OWNER REVIEW / CORRECTION BEFORE PUBLICATION.`**

`P3R-10` must be corrected: the candidate cannot go green on the Ubuntu publication runner, and the
direct-child identity claim is presently unproven. `P3R-11` needs an owner rating decision and, if
upheld, a bounded fix to the gate's instruction coverage plus a test in its negative direction.

The P3 safety case itself is sound and both required corrections are genuinely made. **P4 remains
not authorised. No Trial #4 is authorised. Trial #3 stays `MECHANISM_REJECTED`.**

**Next gate: OWNER REVIEW OF THE INDEPENDENT P3 FINDINGS.**

## 30. Classification

**`HELM_LAUNCH_P3_INDEPENDENT_UNSAFE_REVIEW_NEEDS_FIX`**
