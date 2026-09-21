# HELM-LAUNCH P3 — AUTHOR SELF-REVIEW of the implementation candidate

> # ⚠ AUTHOR SELF-REVIEW — NOT INDEPENDENT
>
> # THIS DOCUMENT DOES NOT SATISFY THE P3 INDEPENDENT REVIEW GATE.
>
> **The same agent session authored the P3 authority commit
> `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60`, the P3 implementation commit
> `afe8922bebd0c85ead7e58d146b738b84141f096`, and this review.** The independence precondition the
> owner set — *"the reviewer MUST NOT have authored"* those commits — was **not met**.
>
> `AGENTS.md` states that the author of a patch is not the sole author of the reference expectation
> and the final approver of its release. Recording a self-review as independent would falsify a
> provenance property that the P3 gate exists to provide, so it is not recorded as one.
>
> **Owner disposition, 2026-09-18** ([P3R-00 accepted](../DECISIONS.md#helm-launch-p3-author-review-disposition)):
> this document is **useful diagnostic evidence only**. A genuinely fresh independent unsafe review
> remains **mandatory** before publication, and must be performed **after** the bounded correction
> the same disposition requires.
>
> **What this document is:** a full adversarial self-audit at the technical depth the review
> instruction specified, with machine-level evidence. It found **two IMPORTANT defects** that the
> implementation handoff did not report, both of which the owner accepted and required to be
> corrected. That makes it useful; it does not make it independent.

> **THE SUBJECT OF THIS REVIEW IS THE HELM-LAUNCH P3 CANDIDATE.**
>
> **P3 IS THE FIRST SLICE WITH INTERNAL PROCESS CREATION, AN INTERNAL EXECUTION ATTEMPT AND SCOPED
> `unsafe`.**
>
> **NO PUBLIC `launch()` EXISTS.**
>
> **P4 AND P5 REMAIN NOT AUTHORISED.**

<a id="independence-not-satisfied"></a>

## 0. Status of this artifact

| Property | Value |
|---|---|
| Review type | **AUTHOR SELF-REVIEW** |
| Independent | **NO** |
| Satisfies the P3 independent-review gate | **NO** |
| Owner disposition | [accepted as diagnostic evidence only](../DECISIONS.md#helm-launch-p3-author-review-disposition) |
| Independent review | **STILL OWED**, after the bounded correction |
| Subject commit | `afe8922bebd0c85ead7e58d146b738b84141f096` |

The technical sections below are unchanged from the form in which they were first recorded, except
for this status block and the disposition note appended at the end.

| Item | Value |
|---|---|
| Accepted P2 base (published) | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| P3 authority commit | `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60` |
| P3 implementation commit | `afe8922bebd0c85ead7e58d146b738b84141f096` |
| Review base | `afe8922bebd0c85ead7e58d146b738b84141f096` |
| Branch | `docs/helm-launch-architecture`, 2 ahead / 0 behind `origin` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc`, untouched |
| Independence | **NOT SATISFIED** — this is an author self-review |
| Review type | source, type, ABI and **machine-level** audit; no runtime Linux execution |
| Findings | **0 BLOCKER**, **2 IMPORTANT**, **6 MINOR**, **2 GATE_PENDING** |

## 1. Starting state

Verified before reading any code:

```text
git status --short          (clean)
git branch --show-current   docs/helm-launch-architecture
git rev-parse HEAD          afe8922bebd0c85ead7e58d146b738b84141f096
origin/docs/helm-launch-architecture  9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b
origin/main                           501a7fa95c4884da4fec9a20a512c2d63f2b30cc
ancestry                    9fb0f8c -> 7bb016f -> afe8922
ahead/behind                2 / 0
```

`origin` was fetched read-only. Nothing was pushed, no CI was dispatched, and no implementation file
was modified by this review.

## 2. Method, and what counts as proof

This review separates three evidence classes and never substitutes one for another.

| Class | What it can establish | Used for |
|---|---|---|
| **SOURCE / TYPE** | what the code says, and what the compiler refuses | confinement, visibility, cfg gating, POD proofs, dependency graph |
| **MACHINE** | what the compiler actually emitted for the cohort | the child window's real call graph, the syscall shim's register contract |
| **RUNTIME / TRACE** | what the kernel actually did | **none of it is available yet** |

The machine-level class is the addition this review makes over the handoff. The candidate's own
closed-world proof is a source-token scan of `child.rs`; a token scan cannot see code the compiler
inserts. To close that gap, the crate was cross-compiled to x86_64 Linux assembly and the child
window's emitted call graph was read directly:

```text
cargo rustc -p helm-launch --target x86_64-unknown-linux-gnu --profile test --lib -- --emit=asm
cargo rustc -p helm-launch --target x86_64-unknown-linux-gnu --lib --features test-fault-injection -- --emit=asm
cargo rustc -p helm-launch --target x86_64-unknown-linux-gnu --profile release --lib --features test-fault-injection -- --emit=asm
```

ABI constants were verified against **`linux-raw-sys` 0.12.1**, which is generated from Linux UAPI
headers and is a *different* source from the `libc` crate the candidate pins itself against. That
answers the instruction's requirement not to accept a value merely because two locally copied
constants agree.

## 3. Authority commit and implementation scope

`7bb016f` precedes the implementation and changes only the four authorised authority and status
documents:

```text
docs/DECISIONS.md                                     +172
docs/PROJECT_STATE.md                                  +59
docs/adr/ADR-0024-launch-authority.md               +40/-11 net
docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md +64
```

It authorises internal Linux x86_64 process creation, an internal execution attempt against the
authorised object, and scoped `unsafe` under `src/backend/`. It authorises **no** public `launch()`,
**no** P4 lifecycle, **no** process-group sweep, **no** Wine, **no** sandbox and **no** Trial #4, and
it states the closed six-item unsafe list and the two fixed internal bounds. It records the accepted
architecture unchanged: ADR-0024 sections A–N are not rewritten, only the current-authority rows are
synced.

`afe8922` changes 16 files, all inside the authorised P3 areas: the crate manifest, README, crate
root, `authority.rs` (crate-private consumption only), `src/backend/**`, `tests/**`, `Cargo.lock`,
the two workflows, and `tools/tests/`. No other HELM crate, no Trial workflow, no LAUNCH-EXEC source
or evidence, no freeze manifest and no `main` content is touched.

**Verdict: `P3_AUTHORITY_AND_COMMIT_SCOPE_SOUND`.**

## 4. The P3 / P4 boundary

Independently inventoried from the diff, not from the handoff.

| Property | State | Evidence |
|---|---|---|
| Internal process creation | **PRESENT** | `clone3` in `src/backend/spawn.rs` |
| Internal execution attempt | **PRESENT** | `execveat` in `src/backend/child.rs` |
| Public `launch()` | **ABSENT** | no such item; `compile_fail` doctest |
| `LaunchOutcome` | **ABSENT** | no such item; `compile_fail` doctest |
| Public backend API | **ABSENT** | `mod backend;` is private; **no `pub use backend`** line exists |
| General run timeout | **ABSENT** | `plan.timeout_ms` is never read by the backend |
| `SIGTERM` / grace lifecycle | **ABSENT** | `Signal::TERM` appears nowhere |
| General stream drain | **ABSENT** | the backend hands read ends out untouched |
| Process-group sweep | **ABSENT** | see section 14 |
| Real execution receipt | **ABSENT** | no receipt is produced by any backend path |
| P4 / P5 implementation | **NONE** | `src/launch.rs` does not exist |

The complete public-API delta of `afe8922` is:

```text
+ mod backend;                      (private, not re-exported)
+ pub(crate) struct AuthorizedParts (crate-private)
+ pub(crate) fn into_parts          (crate-private)
```

**Zero new public items.** Every backend file was scanned for a bare `pub ` item; there are none —
everything is `pub(crate)` or `pub(super)` inside a private module.

**Verdicts: `P3_PUBLIC_API_BOUNDARY_SOUND`, `P3_INTERNAL_EXECUTION_BOUNDARY_SOUND`.**

## 5. Platform boundary

`mod backend;` carries exactly `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]`, the same
predicate as `mod authority;`. Off the cohort neither module is compiled, so no backend symbol can
be public or link-required on Windows, macOS or Linux on another architecture. The crate root's
off-cohort `cfg_attr` doc block still proves the P2 names absent there, and the workflow adds a step
that fails if any `backend::` test exists off the cohort. The portable P1 surface is unchanged.

Local cross-compilation for `x86_64-unknown-linux-gnu` compiles the full backend and its tests; the
Windows host build compiles neither. Both were run at `-D warnings`.

**Verdict: `P3_PLATFORM_BOUNDARY_SOUND`.**

## 6. Dependency graph

| Package | Pin | Gate | Features | Use |
|---|---|---|---|---|
| `rustix` | `=1.1.4` | cohort | `std, fs, process, pipe` | safe parent-side wrappers |
| `libc` | `=0.2.189` | cohort | `default-features = false` | **constants only** |

`process` is required by `setpgid`, `waitid`/`WaitId::PidFd` and `pidfd_send_signal`; `pipe` by
`pipe_with`. **`event` is absent**, and no committed code needs it: the two fixed bounds use
non-blocking reads and non-blocking `waitid` with a sleep, not `poll`. `thread`, `mm`, `net`,
`runtime` and `use-libc` are absent.

Every `libc::` mention in the crate is inside a `const _: () = assert!(…)`. Searched semantically
across all of `src/` and `tests/`: **no `extern` block, no `libc::syscall`, no `libc` function call,
no `dlopen`/`dlsym`, no direct `linux-raw-sys` dependency, no build script.**

`Cargo.lock` delta is exactly one line — `libc` joining `helm-launch`'s dependency list. No package
and no version was added; `libc 0.2.189` and `rustix 1.1.4` each appear exactly once and were
already vetted for this workspace.

**Verdict: `P3_DEPENDENCY_GRAPH_SOUND`.**

## 7. Unsafe confinement — independent census

Counted with a Rust scanner written for this review, not the candidate's lexer: line and block
comments removed, string, raw-string, byte-string and char literals blanked, then tokenised.

| File | `unsafe` tokens | blocks | `unsafe fn` | `allow(unsafe_code)` | `asm!` | `// SAFETY:` | `# Safety` |
|---|---|---|---|---|---|---|---|
| `src/backend/child.rs` | 28 | 24 | 4 | 0 | 0 | 24 | 4 |
| `src/backend/injection.rs` | 3 | 2 | 1 | 0 | 0 | 2 | 1 |
| `src/backend/mod.rs` | 0 | 0 | 0 | **1** | 0 | — | 0 |
| `src/backend/spawn.rs` | 7 | 6 | 1 | 0 | 0 | 6 | 1 |
| `src/backend/syscall.rs` | 2 | 1 | 1 | 0 | **1** | 1 | 1 |
| **every other file in `src/` and `tests/`** | **0** | 0 | 0 | 0 | 0 | — | — |
| **Total** | 40 | **33** | **7** | **1** | **1** | 33 | 7 |

* The token appears as code **only** under `src/backend/`. `src/backend/tests.rs` — which the scoped
  `allow` does cover — contains **zero** unsafe.
* Exactly **one** relaxation, `#![allow(unsafe_code)]` as an inner attribute of `src/backend/mod.rs`,
  covering that module and its descendants. No test source relaxes it.
* Exactly **one** `asm!`, in `src/backend/syscall.rs`.
* `// SAFETY:` comments equal unsafe blocks **1:1** in every file; `# Safety` doc sections equal
  unsafe fns 1:1.
* Raw-descriptor ownership transitions: **exactly one**, `OwnedFd::from_raw_fd(pidfd_slot)` at
  `spawn.rs:432`. The other raw-fd mentions are two read-only `as_raw_fd()` calls used to compute
  descriptor numbers, plus imports.

These counts agree with the implementation handoff. Every site maps to one of the six owner-approved
categories:

| # | Category | Sites | Why safe Rust is insufficient | Invariant, and who established it |
|---|---|---|---|---|
| 1 | raw `rt_sigprocmask` block + restore | `spawn.rs:365`, `spawn.rs:498` | glibc cannot express a mask containing its own internal RT signals | both sets are live `u64` locals of the calling frame; size argument is validated by the kernel; established by `spawn` itself |
| 2 | raw `clone3` | `spawn.rs:471` | no safe wrapper exists | `clone_args` is a live, fully initialised kernel-layout record borrowed mutably across the call; size is the record's own size |
| 3 | `OwnedFd::from_raw_fd` | `spawn.rs:432` | kernel-returned integer | `clone3` returned success with `CLONE_PIDFD`, so the kernel wrote a fresh CLOEXEC descriptor; adopted exactly once, never duplicated |
| 4 | crossing into the child entry | `spawn.rs:487` | the post-clone path is not expressible safely | taken only on `clone3 == 0`; pointer names the calling frame's `ChildPlan`, reached through the child's copy-on-write copy |
| 5 | raw child syscalls | `child.rs` ×24, `injection.rs` ×2 | rustix has no `close_range`; `execveat` is unsafe and `doc(hidden)`; no wrapper is reviewed for the post-clone state | each block names the one descriptor or address it touches and why it is live in the child's copy |
| 6 | the `asm!` shim | `syscall.rs:88` | the whole point | register contract only; see section 8 |

No site falls outside the closed list.

**Verdict: `P3_UNSAFE_CONFINEMENT_SOUND`.**

## 8. Raw syscall shim — machine-level ABI audit

The emitted x86_64 code for `syscall6` in the test profile, comments stripped:

```asm
subq  $88, %rsp
movq  %r9, (%rsp)        ; a5  -> spill
movq  %r8, %r10          ; a4  -> r10
movq  (%rsp), %r8        ; a5  -> r8
movq  %rcx, 8(%rsp)      ; a3  -> spill
movq  %rdx, %rax         ; a2
movq  8(%rsp), %rdx      ; a3  -> rdx
movq  %rax, 16(%rsp)
movq  %rsi, %rax         ; a1
movq  16(%rsp), %rsi     ; a2  -> rsi
movq  %rax, 24(%rsp)
movq  %rdi, %rax         ; nr
movq  24(%rsp), %rdi     ; a1  -> rdi
movq  96(%rsp), %r9      ; a6  -> r9
...
syscall
movq  %rax, 32(%rsp)     ; result
```

Reading the SysV parameter registers (`nr`=rdi, `a1`=rsi, `a2`=rdx, `a3`=rcx, `a4`=r8, `a5`=r9,
`a6`=stack) through to the syscall registers, the mapping the compiler produced is exactly:

```text
rax = nr      rdi = a1   rsi = a2   rdx = a3   r10 = a4   r8 = a5   r9 = a6      result = rax
```

This is the Linux x86_64 syscall ABI. `rcx` and `r11` are declared clobbered, which is exactly what
the `syscall` instruction destroys; the compiler is visibly not keeping a live value in either across
the block (it spilled `a3` out of `rcx` first). **No `asm!` option is requested at all** — `nomem`
and `preserves_flags` are both absent, which is correct for a general syscall, and `nostack` is
absent, which the emitted stack spills show is the right choice. The block has **zero calls** and
**exactly one `syscall` instruction**, confirming that the crate has one raw syscall implementation.

**Return decoding.** `is_error(result) = result < 0 && result >= -4095` — the standard Linux
`-MAX_ERRNO..=-1` window, so a legitimately negative non-error return (none of the twelve syscalls
used here has one, but the shim is general) is not misread. `errno_of` returns 0 for a non-error
value, so an unchecked caller cannot turn success into a failure code. The conversion is
`i32::try_from(result.unsigned_abs()).unwrap_or_default()` — checked, non-panicking, no `as` cast, no
host `errno` and no TLS anywhere on the path.

Each of the twelve syscalls was checked against its own return contract: all return `0`/a
non-negative value on success and `-errno` on failure, and none is in the small family whose success
value can be negative.

**Verdict: `P3_RAW_SYSCALL_ABI_SOUND`.**

## 9. Syscall numbers and ABI constants

Verified against `linux-raw-sys` 0.12.1 `src/x86_64/` — machine-generated from Linux UAPI headers,
and **not** the `libc` crate the candidate asserts against.

| Name | Committed | `linux-raw-sys` | | Name | Committed | `linux-raw-sys` |
|---|---|---|---|---|---|---|
| `read` | 0 | 0 | | `CLONE_PIDFD` | 0x1000 | 4096 |
| `write` | 1 | 1 | | `CLONE_VM` | 0x100 | 256 |
| `rt_sigaction` | 13 | 13 | | `CLONE_FILES` | 0x400 | 1024 |
| `rt_sigprocmask` | 14 | 14 | | `CLONE_VFORK` | 0x4000 | 16384 |
| `dup2` | 33 | 33 | | `CLONE_THREAD` | 0x10000 | 65536 |
| `fcntl` | 72 | 72 | | `SIGCHLD` | 17 | 17 |
| `fchdir` | 81 | 81 | | `SIGKILL` | 9 | 9 |
| `setpgid` | 109 | 109 | | `SIGSTOP` | 19 | 19 |
| `prctl` | 157 | 157 | | `AT_EMPTY_PATH` | 0x1000 | 4096 |
| `exit_group` | 231 | 231 | | `F_SETFD` | 2 | 2 |
| `execveat` | 322 | 322 | | `SIG_SETMASK` | 2 | 2 |
| `clone3` | 435 | 435 | | `PR_SET_NO_NEW_PRIVS` | 38 | 38 |
| `close_range` | 436 | 436 | | `CLONE_ARGS_SIZE_VER0/2` | 64 / 88 | 64 / 88 |

All 26 agree. `sigset_t` in the same source is `c_ulong`, confirming the committed
`KERNEL_SIGSET_BYTES = 8`.

**Verdict: `P3_LINUX_ABI_CONSTANTS_SOUND`.**

## 10. `clone_args` UAPI layout

`linux-raw-sys` `struct clone_args`, x86_64:

```text
flags, pidfd, child_tid, parent_tid, exit_signal, stack, stack_size, tls, set_tid, set_tid_size, cgroup
```

Eleven `__u64` in that order. The committed `CloneArgs` is `#[repr(C)]` with the same eleven `u64`
fields in the same order, with compile-time assertions on size (88 = `CLONE_ARGS_SIZE_VER2`),
alignment (8) and **every field offset** (0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80).

`for_direct_child` is the only constructor. It sets `flags = CLONE_PIDFD`,
`exit_signal = SIGCHLD`, `pidfd = <address>`, and **zero everywhere else** — including `stack` and
`stack_size`, which is the valid "continue on the caller's stack" encoding (`clone3_stack_valid`
accepts `stack == 0` only together with `stack_size == 0`, which holds). Compile-time assertions
prove the flag word has exactly one bit set and contains none of `CLONE_VM`, `CLONE_FILES`,
`CLONE_VFORK`, `CLONE_THREAD`.

The `pidfd` field is the address of a live `i32` local of the calling frame — the correct width for
the kernel's `put_user(pidfd, args->pidfd)` — converted with `expose_provenance()`, which is the
operation that means "this address leaves Rust". The size argument passed to `clone3` is the record's
own size, which is the ABI's versioning contract.

**Verdict: `P3_CLONE3_UAPI_SOUND`.**

## 11. Clone return bifurcation — machine-level

`clone_and_dispatch` is `#[inline(never)]`. Its complete emitted call list in the test profile is:

```text
callq _ZN…spawn14address_of_mut…      (pure; before the syscall)
callq _ZN…syscall8syscall6…           (the clone)
callq _ZN…child10child_main…          (the == 0 branch)
```

**Three calls, no others.** There is no allocation, no `Drop`, no formatting, no logging, no rustix,
no std and no TLS access between the kernel's return and the child's entry into `child_main`; the
function holds no value with a destructor, and `child_main` is `-> !`, so no parent frame is unwound
or dropped in the child. This is the strongest available source of proof for the instruction's
requirement, and it is stronger than the source reading.

On error the parent restores the saved mask and returns without assuming a child or a pidfd. On
`> 0`, the source order is: `i32::try_from` and `Pid::from_raw` (pure arithmetic, no syscall),
`setpgid`, then the pidfd wrap (not a syscall), then the mask restore. So the **first system call
after `clone3` is `setpgid(child, child)`**, and the trace assertion expects exactly that.

**Verdicts: `P3_CLONE_RETURN_PATH_SOUND`, `P3_PARENT_POST_CLONE_ORDER_SOUND`.**

## 12. Pidfd ownership, and finding P3-F6

The single adoption site is guarded by `clone3` success and by an explicit `pidfd_slot < 0` check,
and moves ownership exactly once into an `OwnedFd` that no other value aliases. There is no
close-before-wrap window, no duplication, and no public accessor for the descriptor or its number.

The handoff's residual **P3-F6** — "`clone3` succeeds but the kernel does not populate the pidfd
storage" — was re-derived against this code:

* the kernel writes the descriptor with `put_user` inside `copy_process`; a failing `put_user` fails
  the clone, so the success-without-write state is not reachable through the kernel contract;
* the address handed over is a live `i32` local, so the implementation cannot supply a bad pointer;
* the `u64::try_from` on the address cannot fail on this cohort;
* a stale read of the `-1` initialiser is prevented because the local's address escapes through a
  pointer-to-integer conversion and the `asm!` block declares no `nomem`, so the compiler must treat
  the slot as potentially written.

The implementation correctly declines a numeric-PID fallback, which would expand the accepted
mechanism. The residual stands as a **MINOR defensive** item: if it ever triggered, the child would
be unreachable and would leak, and that consequence is documented in source.

**Verdict: `P3_PIDFD_OWNERSHIP_SOUND`.**

## 13. Parent signal boundary and the kernel `sigaction` ABI

**Mask.** `FULL_KERNEL_SIGSET = u64::MAX` with `sigsetsize = 8`. The committed test walks
`1..=64` and asserts bit `n - 1` for every signal, rather than eyeballing a hex literal, and
separately asserts bits 31 and 32 — glibc's `SIGCANCEL` (32) and `SIGSETXID` (33), the two signals
`sigfillset` omits and `pthread_sigmask` strips, which is the whole reason this call cannot go
through glibc. `SIGKILL` and `SIGSTOP` are described correctly: their bits are present in the value
passed, and the kernel removes them from the mask it installs; nothing claims they are blockable.
The saved mask is a live local of the calling frame, and the source order is block → `clone3` →
`setpgid` attempt → restore, with a restore on the clone-failure path too.

**Restore failure** does not panic, does not invent a public error, does not fall back to a numeric
pid and does not sweep: the crate-private `SignalMaskRestoreFailed` is returned and the child handle
drops, which sends one pidfd `SIGKILL` and reaps within the fixed bound.

**`sigaction` layout.** `linux-raw-sys` x86_64 `struct sigaction`:

```text
sa_handler : __sighandler_t
sa_flags   : c_ulong
sa_restorer: __sigrestore_t
sa_mask    : sigset_t          <- mask LAST
```

The committed `KernelSigaction` is `#[repr(C)] { handler: usize, flags: u64, restorer: usize, mask:
u64 }`, size 32, with every offset asserted (0, 8, 16, 24). That is the kernel record, **not** glibc's
userspace `struct sigaction`, which places `sa_mask` second — the exact confusion the instruction
warns about. Confirmed against an independent source.

**`SA_RESTORER` absent for `SIG_DFL`.** The reasoning is documentation-derived, not verified against
local kernel source: `do_sigaction` validates the signal number only, and the restorer is consumed
solely by the signal-frame construction path, which is reached only for a user handler. A default
disposition is handled entirely in the kernel and never builds a frame.

This is the one ABI conclusion in the candidate that rests on documentation rather than on a local
primary source — but it has a **strong runtime falsifier already committed**: if the layout, the
size argument or the flags were wrong, every one of the 62 `rt_sigaction` calls would fail, the child
would emit a `SIGACTION`-stage failure record, and every normal integration test would observe
`PreExecFailure { stage: Sigaction, … }` instead of a successful run. The pending Linux gate
therefore decides it.

**Verdicts: `P3_PARENT_SIGNAL_BOUNDARY_SOUND`, `P3_KERNEL_SIGACTION_ABI_SOUND` (documentation-derived,
runtime-falsifiable).**

## 14. Group authority without a sweep

Only a successful parent-side `setpgid(Some(pid), Some(pid))` sets
`group_authority_established`; every error leaves it false. It is never retried, never inferred from
the child's own `setpgid(0, 0)`, and **never consumed** — no code reads it to decide anything.

Searched independently across all of `src/`: the **only** signalling site in the entire crate is

```rust
pidfd_send_signal(self.pidfd.as_fd(), Signal::KILL)   // spawn.rs:546
```

one site, `SIGKILL`, by pidfd, to the direct child. There is no `kill`, no `killpg`, no `tgkill`, no
`tkill`, no negative pid and no `Signal::TERM` anywhere. The remaining `SIGTERM` matches in the crate
are P1 plan-vocabulary strings and test-assertion text, not executed signals. The trace test
additionally asserts no `kill` appears at all, and no negative first argument to any signal syscall.

**Verdicts: `P3_GROUP_AUTHORITY_WITHOUT_SWEEP_SOUND`, `P3_NO_PROCESS_GROUP_SWEEP_SOUND`.**

## 15. `ChildPlan` memory and pointer soundness

`#[repr(C)]`, `Copy`, with `const _: () = assert!(!core::mem::needs_drop::<ChildPlan>())` — a
compile-time proof, as the plan requires, for `ChildPlan`, `CloseSpan` and `Fault`. Every field is
`usize`, `[CloseSpan; 3]`, `[u8; 8]` or the zero-sized `Fault`. There is no `OwnedFd`, `Vec`,
`String`, `CString`, `Box` or reference with a destructor.

Pointer provenance was traced field by field:

| Address field | Points at | Built | Stable until |
|---|---|---|---|
| `argv` | `argv_pointers.as_ptr()` — heap buffer of a `Vec<*const u8>` | in `prepare`, before any child exists; pushed to completion **before** the address is taken | the `Vec` is never mutated again; moving `PreparedLaunch` moves only the header |
| — its elements | `argv_storage[i].as_ptr()` — `CString` heap buffers | same | moving the `Vec<CString>` moves headers only, never the buffers |
| `envp` | `[usize; 1] = [0]` | a local of `spawn` | function-body scope, so live across `clone3` |
| `empty_path` | `[u8; 1] = [0]` | a local of `spawn` | same |
| `default_action` | `KernelSigaction` | a local of `spawn` | same |
| `empty_signal_mask` | `u64` | a local of `spawn` | same |
| `status_record` | **not a pointer** — an inline `[u8; 8]` copied with the plan | — | — |

The `ChildPlan` is deliberately constructed **inside `spawn`**, borrowing `&PreparedLaunch`, so the
prepared value cannot move while its addresses are live; and the four inline constants are locals of
the very frame that issues `clone3`, so rustc's scope-based storage liveness keeps them alive across
the call. The child reaches all of it through its copy-on-write copy of that frame, at the same
virtual addresses. The parent's later `release_child_side` frees the argv storage in the **parent's**
address space only; the child's copy is unaffected.

No `Vec` is reallocated after a pointer into it is captured, no temporary's address is stored, and no
reference is converted to a pointer whose owner then moves. `argv_pointers` is null-terminated by an
explicit `push(std::ptr::null())`, and `envp` is the one-element `[NULL]` the contract fixes.

**Verdict: `P3_CHILDPLAN_MEMORY_SOUND`.**

## 16. Child closed-world review — where the candidate's proof falls short

`child.rs` carries `#![no_implicit_prelude]` and, as code, names only `syscall`, `ChildPlan`,
`CloseSpan` and `stage`. The source scan finds no `std`, `alloc`, `rustix`, `libc`, `Vec`, `String`,
`Box`, `CString`, `format`, `print`, `panic`, `unwrap`, `Mutex`, `Once` or `thread_local`, no
indexing (the close ranges are read by array destructuring), and no unchecked arithmetic (the
signal loop uses `wrapping_add`).

**That scan is not sufficient, and this review went past it.** The emitted call graph of the child
window in the profile the tests actually run (`test`, i.e. debug) is:

| Function | Calls emitted |
|---|---|
| `child_main` | `fail`, `syscall6`, `is_error`, `errno_of`, **`memcpy@PLT`**, **`core::panicking::panic_null_pointer_dereference`**, **`core::panicking::panic_misaligned_pointer_dereference`** |
| `fail` | `syscall6`, `is_error`, `errno_of`, `i32::to_le_bytes` |
| `issue`, `close_span` | not emitted — fully inlined |
| `syscall6` | none; one `syscall` instruction |

Two constructs appear that no source-token scan can see, and that `strace` cannot see either because
neither performs a system call. They are finding **P3R-02**, developed in section 22. The rest is
clean: no allocation, no lock, no formatting, no arithmetic-overflow panic, no TLS, and no libc call
other than the `memcpy`.

**Verdict: `P3_CHILD_RUST_CLOSED_WORLD_SOUND` — with IMPORTANT finding P3R-02.**

## 17. Child stage order, descriptors, record, exec, `no_new_privs`

**Stage order** was read from control flow, not from the constant array. `child_main` issues, in
source order: `dup2` ×3 → `fcntl(F_SETFD, 0)` ×3 → `fchdir` → `close_span` ×3 → `setpgid(0,0)` →
`rt_sigaction` for every signal `1..=64` except 9 and 19 → `rt_sigprocmask(SIG_SETMASK, empty)` →
`prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)` → `execveat`. `CHDIR` precedes `CLOSE_RANGE`, which is
required because the range close does not preserve the working-directory descriptor. Every stage
routes failure through one helper that writes a record and ends the process; `fail` is `-> !`, so
there is no fallthrough after a failure, and no helper reorders a stage.

**Descriptor contract.** The child's ranges come from the P1 pure planner, which is property-tested
to cover every number at or above 3 except the two preserved ones, never to invert, and never to
contain a preserved number; the adjacent (F7) and lowest-possible cases are fixed vectors. Unused
spans carry `used == 0` and are skipped rather than issued. Numbers originate as `u32` and are
widened to `usize`, so the kernel's 32-bit `close_range` arguments cannot be polluted by high bits,
and `u32::MAX` reaches the kernel as the intended `~0U`. Descriptors 0, 1 and 2 survive because they
are below the first range; the executable and the status write end survive as the preserved pair and
then disappear at exec because both are close-on-exec; the working-directory descriptor is closed by
the range after `fchdir` has used it. Hosts with 0, 1 or 2 closed are covered by the planner's
simulated-table property tests.

**Exec-status record.** `[stage, pad, pad, pad, errno little-endian]`, eight bytes, built by two
array destructurings — the three pad bytes are taken from the record the parent prepared inside the
plan, so nothing is formatted and nothing is allocated. Eight bytes is below `PIPE_BUF`, so the write
is atomic. The write loop retries on `EINTR` **only** and breaks on every other outcome, then
`exit_group(127)`; there is no path that emits a second record. Errno comes from the raw negative
return through the checked `errno_of`, never from a host `errno` variable.

**`execveat`.** `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)` with the arguments in
`rdi/rsi/rdx/r10/r8` — verified against the register mapping established in section 8. The exec
descriptor is the relocated copy of the admitted object, the pathname is the address of a one-byte
empty C string, `envp` is the one-element `[NULL]`. There is no pathname fallback, no `/proc`, no
`fexecve`, no `execve` and no `PATH` anywhere in the backend; a string-literal scan for a quoted
procfs path finds none. Success does not return; failure produces an `EXEC` record and exit 127.

**`no_new_privs`.** `prctl(38, 1, 0, 0, 0)`; failure is a `NO_NEW_PRIVS` stage record. Nothing claims
N3, nothing attempts a privilege transition, and no setuid or capability fixture is required.

**Verdicts: `P3_CHILD_STAGE_ORDER_SOUND`, `P3_CLOSE_RANGE_CONTRACT_SOUND`,
`P3_EXEC_STATUS_RECORD_SOUND`, `P3_EXECVEAT_CONTRACT_SOUND`, `P3_NO_NEW_PRIVS_CONTRACT_SOUND`.**

## 18. Parent preparation and bounded cleanup

Everything fallible happens before a child exists: argv `CString`s and the pointer array, four
`CLOEXEC` pipes, unconditional `F_DUPFD_CLOEXEC` relocation of all six child-side descriptors to at
least 3, the pure range plan, and `O_NONBLOCK` on the three parent read ends. Relocation takes the
duplicate **while the original is still open** and drops the original afterwards, so no relocated
number can equal its own original, and because no relocated descriptor is ever closed, the six are
distinct — the property the layout planner then requires. A failure at any step returns before
`clone3` and closes everything by `OwnedFd` drop. The P2 measurement and the plan identity are
carried through unchanged; nothing re-measures, reopens or re-resolves the admitted object.

**Cleanup** uses only `pidfd_send_signal(SIGKILL)` to the direct child and
`waitid(P_PIDFD, WEXITED | WNOHANG)`, never `waitpid`, never `wait4`, never a numeric-pid kill and
never a blocking wait. `EINTR` on the wait establishes nothing and is retried by the bounded caller;
`ECHILD` and other errors latch `EndUnobservable` so a later drop cannot signal a process the handle
no longer owns; a bound that passes returns `None`, which is not a claim that the child is still
running. The `Drop` implementation is the backstop that makes leaking a child unreachable, and it is
itself bounded by `POST_KILL_REAP_MS`; a handle already reaped is left alone. The worst case is one
`SPAWN_CONFIRM_TIMEOUT_MS` plus at most two `POST_KILL_REAP_MS` waits — finite on every path.

**Verdicts: `P3_PARENT_PREPARATION_SOUND`, `P3_DIRECT_CHILD_CLEANUP_SOUND`.**

## 19. Fault injection confinement

The gate is `all(feature = "test-fault-injection", debug_assertions)` at **every** site: the module
declaration, the `ChildPlan` field, the `issue` stage hook, the pre-exec hook, the parent's
stdin-retention branch and the test module. The complement branch `cfg(not(all(…)))` exists exactly
once, so the non-injection build has a definition rather than a hole. Outside an injection build
`Fault` has no fields and is zero-sized. A compile-time assertion states
`!INJECTION_COMPILED || cfg!(debug_assertions)`.

Machine-level confirmation:

* debug lib built **with** the feature: the marker string appears once, in
  `.data.rel.ro…PRESENCE_MARKER_KEPT`, so `#[used]` does keep it in the object;
* release lib built **with** the feature: **no backend symbol at all is emitted**, and the marker is
  absent.

The gating is therefore sound. The **proof of it in CI is not** — see finding P3R-01, and note the
second-order point the release measurement exposes: in the release *lib* profile the whole backend is
dead-code-eliminated, so marker absence there would not be specific to the gating even if the file
selection worked.

**Verdict: `P3_FAULT_INJECTION_CONFINEMENT_SOUND` (gating) — with IMPORTANT finding P3R-01 (proof).**

## 20. S5 and S6

**S5.** The injection point is immediately before stage 9, after `NO_NEW_PRIVS`, which is exactly the
accepted shape: a child that completed its setup and ended before `execveat`. It calls `exit_group(0)`
and writes **no** record, so the parent observes a clean end-of-file with zero bytes and classifies
`indeterminate(status_eof_without_record)` beside `child_end = exited { 0 }`. Exit status **zero**
makes this the sharpest available form of the test. The committed assertions check the classification,
that no `SIGKILL` was sent, and that the fixture's stdout is empty — which independently confirms
nothing executed. No path can emit a status record on this branch.

**Verdict: `P3_S5_TEST_PROBATIVE`.**

**S6 — the load-bearing question the instruction raises.** The pipe topology was traced end to end:

1. `prepare` creates the stdin pipe; the read end is relocated and given to the child, the write end
   stays in the parent at its original number.
2. `release_child_side(retain_stdin_writer)` **drops** the write end in every normal build —
   `retain_stdin_writer` is a compile-time `false` constant outside an injection build — and returns
   `Some(fd)` only under `MODE_STALL_BEFORE_EXEC`.
3. The retained descriptor is stored in `MinimalLaunch.stdin_write`, so the parent holds it for the
   whole of `observe_exec_status` and beyond, until the test drops the value.
4. The child inherits its own copy of the write end at clone time, and **closes it at stage 4**: the
   descriptor is not one of the two the range plan preserves.

So at the injection point, after `CLOSE_RANGE` and before `EXEC`, the **only** remaining writer is
the parent's deliberately retained copy. `read(0, …)` on a blocking pipe with a live writer and no
data blocks indefinitely. The stall is **deterministic, not a scheduling race**, and the committed
test asserts the lower bound directly (`elapsed >= SPAWN_CONFIRM_TIMEOUT_MS`) as well as the
`PreExecStatusTimeout` classification, `sigkill_sent`, and `Signaled { signal: 9 }` from the bounded
reap — the last of which is what proves no zombie was left.

The retained writer meets every condition the instruction sets: explicit, bounded, test-only, absent
from normal product behaviour, reliably cleaned by ownership, and documented in both `launch_with`
and `injection.rs`. No group sweep is involved.

**Verdict: `P3_S6_TEST_PROBATIVE`.**

## 21. Test-support contracts

**Report fixture.** `report_fixture_reports_without_the_backend` runs the fixture directly through
`std::process::Command`, parses its report, asserts the versioned marker line
`HELM-LAUNCH-P3-FIXTURE/1`, asserts the fixture-identity marker names the intended fixture, and
asserts the report is non-empty of environment entries — which is what makes the later
empty-environment assertion meaningful rather than vacuous. The parser rejects a missing version
line and a truncated report (`end` sentinel). Every consumer re-checks the identity marker. The
content-addressed cache cannot bypass the self-test: the cache decides only whether `rustc` runs, and
the producer self-test is a `#[test]` in its own right that runs regardless.

**Verdict: `P3_REPORT_FIXTURE_CONTRACT_SOUND`.**

**FD isolation.** The fixture enumerates `/proc/self/fd` and explicitly excludes the scan's own
handle by resolving each link and skipping the one that points at a `/proc/*/fd` directory, so the
observation mechanism cannot invalidate the claim. The test holds two unrelated descriptors open in
the launching process — one close-on-exec and one with the flag explicitly cleared and asserted
cleared — and requires the executed image to report exactly `{0, 1, 2}`, each resolving to a pipe.
The admitted executable descriptor is absent because it is close-on-exec. The test additionally
asserts the reporting image's `Tgid` equals the direct child's pid, so the report cannot have come
from a descendant.

**Verdict: `P3_FD_ISOLATION_TEST_PROBATIVE`.**

**argv / environment / cwd.** argv coverage is spaces, shell metacharacters, a newline, an empty
element and an arbitrary `argv[0]`, compared byte-for-byte after hex round-trip, so no shell
interpretation could hide. An empty element is admissible under the plan parser, which bounds element
length only from above. The environment test asserts the launching process has a non-empty
environment including `PATH` before requiring the executed image to report none. The cwd test compares
`(st_dev, st_ino)` and never a pathname.

**Verdict: `P3_ARGV_ENV_CWD_TESTS_SOUND`.**

**atfork.** The C helper does exactly one thing: register `pthread_atfork` handlers and record, via
`open`/`write` to paths named by environment variables, that registration succeeded and whether any
handler ran. It contains no launcher logic. The preload-timing defect is genuinely fixed: per-run
variables are applied by `env(1)` **inside** the tracer, so `LD_PRELOAD` first takes effect in the
launcher process and not in `strace`, whose own `fork(2)` would otherwise have fired the handlers
under test. The `registered` marker is the control that prevents a vacuous pass. The inference
"registered and not run ⇒ no glibc fork path was taken" is sound given glibc's documented behaviour,
but the test contains no positive control demonstrating that a handler *would* fire — see MINOR
finding P3R-05.

**Verdict: `P3_ATFORK_TEST_PROBATIVE` — with MINOR P3R-05.**

**Multithreaded trace.** The threaded variant spawns three threads that allocate in a loop, and holds
them alive across the measured clone through an `AtomicBool` that is only set after `complete()`
returns, with a 50 ms settle before the launch. The threads cannot terminate early. The trace parser
selects the process clone by requiring `CLONE_PIDFD` **and** the absence of `CLONE_THREAD`, and
asserts that exactly one such record exists — which is the Trial #2 M2 defect handled correctly,
since the Rust test harness itself clones threads with `clone3`. Ambiguity fails the test rather than
being guessed at.

**Verdict: `P3_MULTITHREADED_TRACE_PROBATIVE`.**

**Trace parser.** Handles the realistic forms: pid prefixes on every line (which `-f` with a single
`-o` guarantees), `<unfinished …>` / `<… resumed>` splicing per pid at the resumption point — which
is when the call completed, so per-task ordering stays true — signal (`---`) and exit (`+++`) lines
skipped, and `" = "` located from the right so struct arguments containing `=` do not confuse it.
The child window is defined as the child pid's stream up to and including the first `execveat`, and
every syscall in it must be in the permitted set; an unknown syscall fails rather than being ignored.
Stage multiplicities are checked as ordered groups (`dup2` ×3, `fcntl` ×3, `fchdir` ×1,
`close_range` 1–3, `setpgid` ×1, `rt_sigaction` ×62, `rt_sigprocmask` ×1, `prctl` ×1, `execveat` ×1)
with no trailing calls permitted, and `fchdir` is required to precede the first `close_range`.

**Verdicts: `P3_TRACE_PARSER_SOUND`, `P3_CHILD_SYSCALL_WINDOW_SOUND` (design; the runtime result is
`GATE_PENDING`) — with MINOR P3R-04 and P3R-06.**

**Exec-failure fixtures.** `EACCES` uses a mode-0600 copy, which admission accepts because it records
but never enforces execute permission, so the denial necessarily happens at `execveat` and not at
admission; with no execute bit set the denial holds for root as well. `ENOEXEC` zeroes `e_phentsize`,
which is the **first** field the kernel's ELF loader validates after the fields admission itself
reads, and which makes `load_elf_phdrs` fail with the pre-set `-ENOEXEC`; the test asserts the
fixture's `e_phentsize` really was 56 before zeroing it, so the mutation cannot silently become a
no-op, and the magic, class, data encoding, machine and type the cohort check reads are untouched.
`ETXTBSY` holds a real writer open across the whole launch. Every expectation is a
`PreExecFailure { stage: Exec, errno }` with an exact errno, never an inference from exit 127 — and
each is paired with an independent `Exited { 127 }` assertion and, for two of the three, an assertion
that the fixture produced no output.

**Verdict: `P3_EXEC_FAILURE_TESTS_SOUND`.**

**Exact-object execution.** The descriptor is admitted before the namespace is mutated; the pathname
is then replaced, by rename, with a different object; the executed image still reports the admitted
fixture's marker. The product backend resolves no pathname at any point. The fixture marker is used
as **test** evidence that the expected object ran, and is never converted into a product
exec-success claim: `exec_status` on that path remains `indeterminate`.

**`no_new_privs`.** The parent control asserts the launching process reports `NoNewPrivs: 0` and
fails with a named host precondition otherwise, so the child's `1` is attributable to the child's own
`prctl`. No N3 claim, no privilege transition, no root or setuid fixture.

**Verdicts: `P3_EXACT_OBJECT_TESTS_SOUND`, `P3_NO_NEW_PRIVS_TEST_SOUND`.**

## 22. README and CI

**README.** States P1 accepted, P2 accepted, P3 an implemented candidate not yet product-accepted and
not yet independently reviewed. States that internal process creation and an internal
authorised-object execution attempt exist, and that scoped `unsafe` exists under `src/backend/`.
States the absence of a public `launch`, of the P4 lifecycle, of the group sweep, of a receipt from a
real launch, of a sandbox, of containment, of Wine and of any exec-success claim. **No stale "no
execution backend exists" claim survives** anywhere in the README or the crate root documentation.
The one qualification is the closed-world wording discussed in P3R-02.

**Verdict: `P3_README_SOUND` — subject to P3R-02.**

**CI.** The Ubuntu job checks `strace`, `cc` and `rustc` before anything else, and the P3 Linux tests
are genuinely scheduled: the default test run, a feature-enabled run, a release build, the named
backend and traced-window runs, and the two structural suites, plus the Python confinement suite.
Windows and macOS run the portable surfaces and are additionally required to prove no `backend::`
test exists. There is no trial semantics, no D-7, no privileged runner and no `sudo`. The 30-minute
timeout is realistic for a job that contains one intentional 5-second bound repeated across runs and
three tracer re-executions.

One step does not work — finding **P3R-01**.

**Verdict: `P3_CI_DESIGN_SOUND` in structure — with IMPORTANT finding P3R-01.**

## 23. Candidate hygiene

No binary file in either commit. No generated fixture executable, no `target/` content, no trace
output, no log, no NUL byte, no credential, no private host path, and no unrelated file. No
`Co-Authored-By`, no "Generated with" and no agent signature in either message — verified by scanning
the full commit bodies. The single match for a personal identifier is "Djomla83", the repository
owner's own name in the decision record, matching existing practice in `DECISIONS.md`.

**Verdict: `P3_CANDIDATE_HYGIENE_SOUND`.**

## 24. Local validation performed by this review

| Check | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo test --workspace --locked` | **PASS** |
| `cargo test -p helm-launch --locked` | **PASS** — lib 42, p2_boundary 19, p3_boundary 17, plan_contract 19, doctests 23 + 2 |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — 794 tests, OK (74 skipped) |
| `python tools/validate_docs.py` | **PASS** |
| `git diff --check` | **PASS** |
| Linux cross clippy, default features, `-D warnings` | **PASS** |
| Linux cross clippy, all features, `-D warnings` | **PASS** |
| Linux cross check, release + all features | **PASS** |
| Linux assembly emission and child-window call-graph audit | **PERFORMED** — see sections 8, 11, 16 |
| ABI constants against `linux-raw-sys` | **PERFORMED** — 26/26 agree |

Rust was not installed in WSL for this review.

```text
P3 LINUX RUNTIME VALIDATION:        PENDING PUBLICATION CI
P3 STRACE CHILD-WINDOW VALIDATION:  PENDING PUBLICATION CI
```

Both are **GATE_PENDING**, not passes. Nothing in this review is runtime evidence about the kernel.

## 25. Findings

| ID | Severity | Area | Reachable? | Summary |
|---|---|---|---|---|
| **P3R-00** | **IMPORTANT (process)** | review provenance | yes, now | The independence precondition is not satisfied: the session that authored `7bb016f` and `afe8922` produced this review. It must not be recorded as the independent unsafe review the P3 gate requires. |
| **P3R-01** | **IMPORTANT** | CI proof | yes, first Ubuntu run | The release fault-injection absence step cannot work. Cargo places the rlib at `target/<profile>/libhelm_launch.rlib` (no hash) and the hashed copies under `target/<profile>/deps/`; the step's glob `target/<profile>/libhelm_launch-*.rlib` matches **neither**. Under `set -eux` without `pipefail`, `$(ls … \| head -n 1)` yields an empty string, `grep` on it errors, and the job exits 1 reporting a misleading "TEST ENVIRONMENT FAILURE". Verified by listing the actual artifacts. Two second-order problems would remain after fixing the glob: `deps/` holds one rlib per feature set and `head -n 1` selects arbitrarily; and in the release lib profile the whole backend is dead-code-eliminated, so marker absence there is not specific to the gating. |
| **P3R-02** | **IMPORTANT** | child closed world | `memcpy` yes; panics statically unreachable | In the `test`/debug profile — the only profile in which the backend is compiled and run — `child_main` emits `callq memcpy@PLT` for the 168-byte `ChildPlan` copy, plus `core::panicking::panic_null_pointer_dereference` and `panic_misaligned_pointer_dereference` from the raw-pointer debug assertions. The committed proof is a source-token scan, which structurally cannot see compiler-inserted code, and `strace` cannot see it either because neither performs a syscall. The documented claims "no glibc code after the clone" and "panics never" are therefore stronger than what is established. The underlying safety property still holds in fact: `memcpy` is async-signal-safe, lock-free, allocation-free and non-unwinding, its PLT entry is resolved by the parent long before the clone, and the panic paths cannot fire because the pointer is always a live, aligned `&ChildPlan`. |
| **P3R-03** | MINOR | pidfd residual | no | `PidfdNotProvided` is unreachable under the `CLONE_PIDFD` kernel contract and cannot be produced by the implementation's own handling; the check is defensive. The implementation correctly declines a numeric-pid fallback. If it ever triggered, the child would leak — documented in source. (Supersedes the handoff's P3-F6.) |
| **P3R-04** | MINOR | trace coverage | n/a | No traced exec-failure case. `assert_closed_child_window` requires a successful `execveat`, so the failure-path child window (`write` + `exit_group`) is never trace-verified. The failure path is covered by non-traced integration tests with exact errno. |
| **P3R-05** | MINOR | atfork probity | n/a | The atfork test has a control for registration but no positive control demonstrating that a handler would fire on a libc fork path. The conclusion rests on glibc's documented `fork(2)` behaviour rather than on an in-test demonstration. |
| **P3R-06** | MINOR | trace brittleness | n/a | The trace asserts the mask restore is exactly the second syscall after `clone3` (`parent[at + 2]`), which is tighter than the accepted contract ("restore occurs after the setpgid attempt"). A contract-conformant implementation change would fail the test. |
| **P3R-07** | MINOR | CI cost | n/a | The Ubuntu job runs the backend suite three times (plain, named, feature-enabled), each paying the intentional 5-second S6 bound and re-running the three tracer re-executions. |
| **P3R-08** | MINOR | host assumption | n/a | `require_tool("env", …)` runs `env --version`, which is GNU-coreutils-specific. On the `ubuntu-24.04` cohort runner this is satisfied; elsewhere it surfaces as a loud test-environment failure, which is the correct behaviour. |
| **P3R-G1** | **GATE_PENDING** | runtime | — | Real Linux backend execution: no process has been created or executed by this candidate anywhere yet. |
| **P3R-G2** | **GATE_PENDING** | runtime | — | Real `strace` child-window evidence, including the parent's first-syscall ordering, the permitted child syscall set and the absence of a group signal. |

No BLOCKER was found. In particular, no unsafe exists outside the backend, no public `launch` exists,
no process-group sweep exists in any build, the `clone3` ABI matches the UAPI field by field, no
`ChildPlan` pointer can dangle, the `execveat` target is the authorised descriptor, no unapproved
child syscall is structurally reachable in a default build, the trace parser cannot bless the wrong
process, no parent syscall precedes `setpgid` after the clone, and the child cannot allocate, lock or
take a lock on its normal path.

## 26. Recommendation

**P3 CANDIDATE REQUIRES CORRECTION BEFORE PUBLICATION.**

Two IMPORTANT findings stand. **P3R-01 alone would make the publication run fail** at the very step
that is supposed to evidence the fault-injection confinement, and it would fail with a message that
misdescribes the cause — so the Linux runtime and `strace` gates this publication exists to reach
would not even be interpreted cleanly. **P3R-02** means a documented safety claim is broader than what
the committed evidence establishes; the disposition may be a documentation correction, an additional
machine-level check in the boundary suite, or both, and that choice is the owner's.

**P3R-00 is separate and is not fixed by correcting code.** Whatever is done about the two IMPORTANT
findings, the fresh independent unsafe review that ADR-0024 and the P3 authority decision require has
**not** been performed.

**P4 REMAINS NOT AUTHORISED.**

## 27. Next gate

**OWNER REVIEW OF THESE FINDINGS**, and in particular an owner decision on how to obtain the
independent unsafe review that section P3R-00 records as still owed.

Trial #3 remains `MECHANISM_REJECTED` and must not be rerun. **No Trial #4 is authorised.**


---

<a id="owner-disposition"></a>

## 28. Owner disposition of these findings, 2026-09-18

Recorded after this document was written; the sections above are left as they were.

| Finding | Owner disposition |
|---|---|
| **P3R-00** | **ACCEPTED.** The independence precondition was not met. This document does **not** satisfy the required independent P3 unsafe-review gate; its technical work is diagnostic evidence only. A genuinely fresh independent unsafe review remains mandatory after the correction. |
| **P3R-01** | **ACCEPTED.** The committed release injection-absence CI proof is defective and must be corrected before publication. |
| **P3R-02** | **ACCEPTED.** The actual test/debug child machine code contains `memcpy@PLT` and compiler-emitted panic paths. This does **not** satisfy the intended closed child contract merely because those paths are believed safe or unreachable. **ADR-0024 and the productization contract are not weakened to allow it, and documentation alone does not resolve it**: the implementation is corrected and machine-code evidence is added. |
| **P3R-03** | **ACCEPTED AS OPEN**, MINOR. No numeric-PID fallback is authorised. |
| **P3R-04 … P3R-08** | remain **MINOR / OPEN**, carried forward. |
| **P3R-G1 / P3R-G2** | remain **GATE_PENDING**. |

The owner's required target for P3R-02:

> **NORMAL P3 CHILD MACHINE CODE MUST NOT CALL GLIBC, LIBSTD, ALLOCATOR, PANIC/UNWIND OR OTHER
> EXTERNAL RUNTIME HELPERS BETWEEN CHILD ENTRY AND `execveat` / `exit_group`.**
>
> Internal helm-launch child helpers and the raw syscall shim are acceptable if they themselves
> satisfy the same closed contract.

P3 authority remains valid. The candidate is **not ready for publication**. P4 and P5 remain **not
authorised**, and **no Trial #4 is authorised**.
