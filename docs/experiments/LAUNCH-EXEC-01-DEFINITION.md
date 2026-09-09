# LAUNCH-EXEC-01 — preregistered execution definition

**Status: PROPOSED — NOT_RUN. No trial has been executed and no result exists.**\
**Not yet freezable:** owner decisions **D-9** and **D-10** both change case membership. See
[section 8](#8-what-this-definition-does-not-authorise).\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`.\
**Authority:** Proposed [ADR-0024](../adr/ADR-0024-launch-authority.md) and the
[helm-launch architecture](../research/HELM-LAUNCH-ARCHITECTURE.md). Neither is Accepted, and
this definition authorises nothing by itself.

This document **freezes the case set** for the first `helm-launch` mechanism experiment, before
any trial, following the discipline that worked for
[OBS-FS-01](obs-fs-01/README.md): the definition is committed first, and the mechanism is not
edited after the first valid trial.

> **Corrected 2026-09-09 under the
> [three-workstream pre-execution review](../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md),
> from 43 cases to 71.** The original preregistration `be0251b` is preserved unchanged in
> history. It could have returned `MECHANISM_ACCEPTED` while four load-bearing claims were
> false: its aggregate precedence was neither total nor disjoint, E6 was simultaneously
> ungradeable and required to PASS, F2 was scored after the trial under an undecided D-1, the
> primary instrumentation contradicted the descriptor invariant it was measuring, and clean EOF
> was read as proof of exec. Two cases were deleted as unposable, thirty added, eleven
> reformulated.

## 0. What this experiment answers, and what it does not

> **It answers:** can HELM execute **one explicitly authorized synthetic Linux executable** with
> the process-boundary semantics the architecture claims?

> **It does not answer:** whether HELM can safely manage a Wine application lifecycle. That
> needs a separate later experiment and is out of scope here.

**No Wine. No 7-Zip. No proprietary software. No A0 lab. No privileged operation.** Only
synthetic helper executables built from source committed alongside this definition.

**Plan-parse properties are not answered here either.** The original A5 (NUL argument) and V3
(`LD_PRELOAD`) are **deleted**: both are properties of `crates/helm-launch`, which does not exist
and which ADR-0024 forbids creating before this experiment runs, and a NUL byte cannot be
delivered through a C spike's `argv` at all. They are recorded as in-crate obligations in
[architecture section 41](../research/HELM-LAUNCH-ARCHITECTURE.md#41-test-and-falsification-strategy-beyond-the-experiment),
and **no result here supports or refutes them.**

## 1. Scope and disposability

All experiment code is **disposable pre-implementation spike code**. It is not
`crates/helm-launch`, is not a Cargo workspace member, defines no product API, and must not be
promoted into one. Its only purpose is to falsify the mechanism described in the architecture.

The experiment may be written in C, Rust or both. A small C spike is acceptable and is expected
to make `fork`/`exec` semantics and the syscall trace clearer; product language preference must
not distort syscall evidence.

**A C spike always has `close_range` available, so under D-1 arm (ii) an F-series PASS would not
transfer to a `forbid(unsafe_code)` Rust implementation.** That non-transferability is recorded
in [section 6](#6-aggregate-verdict-rules) rather than discovered afterwards.

## 2. Artefacts to be committed before the first trial

Committed **before** any trial, and hashed in `SOURCE-HASHES.json` as OBS-FS-01 did. The
definition records that freeze commit by SHA.

| File | Role |
|---|---|
| `helper_report.c` | Primary synthetic helper. Reports its own observed process boundary as JSON **on descriptor 1 only** |
| `helper_alt.c` | A second helper with a **different body and different digest**, used to detect executable substitution |
| `helper_dynamic.c` | Identical to `helper_report` but **dynamically linked**; used only by E7 |
| `helper_fork.c` | Negative control: forks a descendant that outlives its parent |
| `helper_setid.c` | Set-user-ID bit set, owned by the running user; used only by X7 |
| `helper_foreign.elf` | Well-formed ELF64 fixture with `e_machine = EM_AARCH64`; committed, **never executed** |
| `launcher_spike.c` | The mechanism under test: pin → measure → `fork` → child setup → `execveat` |
| `frozen_cases.py` | Frozen case manifest: membership, **class partition**, the **frozen D-1 arm**, inputs, schedules, per-case `traced` declaration, safe outcome sets |
| `oracles.py` | Independent expected values computed without reading spike output |
| `harness.py` | Fixture builder, descriptor pre-loader, tracer driver |
| `checker.py` | Verdict evaluator; links nothing from the spike |
| `run_launch_exec_01.py` | Runner; emits one sanitised report |

`helper_report.c` must report, from inside the executed image:

- its exact `argv`, element by element, with lengths, so quoting and splitting are visible;
- its **complete** `environ` key set and the exact values, so leakage is detectable;
- its full descriptor set, enumerated with `fcntl(i, F_GETFD)` over `[0, RLIMIT_NOFILE)` — which
  **opens nothing** — plus each descriptor's `FD_CLOEXEC` flag;
- its `SigBlk`, `SigIgn`, `SigCgt` and `SigPnd` masks from `/proc/self/status`, so inherited
  signal state is visible;
- its real and effective uid and gid, so a credential transition is visible;
- its working-directory identity as `st_dev`/`st_ino` of `.`, never as a pathname;
- a self-identity marker distinguishing it from `helper_alt`;
- requested stdout and stderr byte volumes, exactly as instructed by argv;
- a requested exit code, and an optional sleep.

**Report channel.** The report is written to **descriptor 1 only**, terminated by the frozen
sentinel `HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n`; all payload bytes requested by argv precede the
sentinel and all report bytes follow it, and the sentinel offset is recorded. O-series counts and
digests are computed over the pre-sentinel segment only. **No descriptor above 2 is ever passed
to a helper, in any case.** The original definition had the report travel on "a descriptor the
harness controls", which is a descriptor above 2 that must survive `close_range` into the
executed image — so F1 and F4 would have scored PASS while "exactly 0, 1 and 2 survive" was
false. `/proc/self/fd` is read **only** in F1 and F4, as a secondary cross-check, and the
descriptor that reading consumes is reported by number and is the only descriptor the checker may
excuse.

**Frozen output recipe.** Stream bytes are generated as `byte[i] = (i * 251 + tag) mod 256`, with
`tag = 1` for stdout and `tag = 2` for stderr, `i` counting from 0 within the stream. `oracles.py`
computes the expected count and SHA-256 from this recipe and the declared volume alone, and never
reads spike output. A byte *volume* is not a byte *recipe*, and without this the O-series digest
oracles are uncomputable.

**Frozen child-setup stage vocabulary.** `{RELOCATE, DUP2, CLEAR_CLOEXEC, CHDIR, CLOSE_RANGE,
SETPGID, SIGMASK, SIGACTION, EXEC}`, declared in `frozen_cases.py`. "The correct stage" is
otherwise unverifiable.

**Static linking is a precondition, not a preference.** A complete trace is what makes the
negative claims "no other descriptor was present" and "no other file was opened" evidence rather
than filtering. Before any trial the harness compiles a two-line probe with
`gcc -O2 -Wall -Wextra -static`, verifies it with `file` and `ldd` (which must report "not a
dynamic executable"), and records the command, the `gcc -v` banner and the SHA-256 of every built
binary. **If the static link fails for any reason, every case is BLOCKED and the aggregate is
`MECHANISM_INCONCLUSIVE`.** A dynamically linked helper must **not** be substituted, in whole or
for any individual case; `musl-gcc` must not be substituted; no package is installed and no
`sudo` is used to obtain a static toolchain. `build-essential` and `libc6-dev` are **not** in the
recommended runner's manifest, so this gate is live rather than theoretical. The single exception
is `helper_dynamic`, which exists precisely to expose the loader activity every other helper
excludes.

## 3. Preregistered cases

**71 cases.** Every case has a **single frozen expected outcome**, except where a safe outcome
set is preregistered here because every member is equally honest; any result outside it is a
FAIL, not a retry. Every case declares its class and its `traced` flag in `frozen_cases.py`.

### E — executable identity and TOCTOU (12)

| # | Case | Expected |
|---|---|---|
| **E1** | Normal pinned executable runs | `helper_report` executes; self-identity marker matches; the measured digest equals an independent `hashlib` digest of the file taken before the run |
| **E2** | The pathname is replaced by `rename(2)` of `helper_alt` **over** it — an atomic inode swap — after the capability is pinned | The **pinned body** runs: marker is `helper_report`. Any other result **rejects the mechanism** |
| **E3** | The original pathname is `rename`d, then `unlink`ed, after pin | Pinned body still runs; exec unaffected |
| **E4** | The pinned leaf was a **symlink** whose target is retargeted after the pin | Pinned body runs; the substituted body never executes |
| **E5** | The **harness process** — the parent of the launcher, never the child — opens a second `O_WRONLY` descriptor on the same inode and **holds it open across `launch`**; `fcntl(F_GETFD)` on it is recorded as succeeding immediately after `launch` returns | `ExecFailed { stage: EXEC, errno: ETXTBSY }`, never `Exited { 127 }`. **Inode oracle, mandatory:** `fstat` of the writable descriptor and of the exec capability are both recorded and their `(st_dev, st_ino)` pairs must be **equal**; pathname equality is not evidence. If the pairs differ the case is **INVALID**, not FAIL. Establishes the exec-time write deny **for a concurrent holder only** |
| **E5b** | A writer opens `O_WRONLY`, writes, and **closes**, all strictly between the measurement and `launch` | **No `ETXTBSY`.** Exec succeeds. The write-deny reference is taken at exec time (`fs/exec.c:do_open_execat`) and `i_writecount` has returned to zero, so the ordinary open-write-close writer is never refused. An `ETXTBSY` here falsifies the primary-source reading and is recorded, not retried |
| **E6** | After measurement and before `execveat`, the harness opens a second `O_WRONLY` descriptor on the same inode, `pwrite`s a **length-preserving, ELF-valid** mutation flipping only the self-identity marker to `helper_alt`'s value, `fsync`s and **closes** it — all sequentially in the parent before `launch` is entered | **Single prediction:** exec **succeeds**; the child's report carries the **mutated** marker; the recorded measurement is the **pre-mutation** digest. Oracle: `fstat` `(st_dev, st_ino)` equality, and a `hashlib` digest of the post-mutation file that differs from the recorded measurement and matches the marker the child reported. `ExecFailed { ETXTBSY }` here is a **FAIL** — no writer is open at exec time. **Documentation gate:** the case PASSes only if the field is named `pre_exec_body_sha256` and no document or receipt text asserts that the measured bytes executed |
| **E6b** | `ftruncate` + rewrite to a **different** length, writer closed before exec | Safe set: `{Exited{…} with the mutated marker, ExecFailed{ENOEXEC}, Signaled{SIGBUS}, Signaled{SIGKILL}}`. All four are honest — a torn image fails the loader's header checks or faults on a page beyond `i_size`. Any outcome **presented as "the measured body ran"** is a FAIL. Also a documentation gate |
| **E6c** | A writer establishes `mmap(MAP_SHARED, PROT_WRITE)`, **closes the fd**, then stores into the mapping, then the launcher execs | **Outcome recorded, not predicted.** Safe set `{ExecFailed{ETXTBSY}, Exited{…} with the mutated marker}`. This is the one genuinely unresolved kernel question in the E series: the review could not settle from primary sources whether a surviving `i_mmap_writable` mapping leaves `i_writecount` at zero. **Explicitly carved out of the `MECHANISM_ACCEPTED` gate**, on the same footing as P1/P2 |
| **E6d** | `fchmod` to `0` between admission and exec | `ExecFailed { EACCES }`. Establishes that `pre_exec_mode_bits` is likewise a pre-execution measurement, because permission is evaluated at exec (`.acc_mode = MAY_EXEC`). **BLOCKED** if preflight reports `geteuid() == 0` |
| **E7** | The pinned object is **dynamically linked** (`helper_dynamic`) | Exec succeeds; the trace shows the kernel resolving `PT_INTERP` and `ld.so` resolving `DT_NEEDED` libraries **by name**. Establishes that `pre_exec_body_sha256` covers the pinned object only. A report presenting E1's static result as covering dynamic objects is a **FAIL**. Without this case no preregistered case executes a dynamic object at all, while every real HELM subject is dynamic |
| **E8** | Capability on `helper_foreign.elf` — valid `\x7fELF` magic, `e_machine = EM_AARCH64` | **Refused at admission** with `ElfNotInCohort`; `execveat` must never be reached. Reaching `execveat` at all **rejects the mechanism**. Additionally the report must record `/proc/sys/fs/binfmt_misc/status` and every registration **before the first trial**, because an entry matching `\x7fELF` with a masked `e_machine` is inserted ahead of `binfmt_elf` and would resolve an interpreter by pathname |

### A — argv (5)

| # | Case | Expected |
|---|---|---|
| **A1** | Argument containing spaces: `a b c` | Arrives as **one** element, byte-identical |
| **A2** | Argument containing `; \| & $(id) ' " \`` and a newline | Arrives as **one** literal element, byte-identical; no shell involved; no expansion |
| **A3** | Empty argument `""` | Arrives as one element of length 0 |
| **A4** | Argument of exactly `MAX_ARG_BYTES` | Arrives byte-identical |
| **A6** | `argv[0]` differs from any pathname | The child observes exactly the declared `argv[0]` |

### V — environment (4)

| # | Case | Expected |
|---|---|---|
| **V1** | Host sets `HELM_LEAK_CANARY`, `LD_LIBRARY_PATH`, `PATH`, `HOME` before launch; plan mode `empty` | Child's `environ` is **exactly empty**. Any inherited name is a FAIL. A leaked name is reported under the sanitisation rule of [section 5](#5-privacy-and-sanitisation), never reproduced verbatim |
| **V2** | Plan mode `explicit` with three entries including UTF-8 and `=` inside a **value** | Exactly those three names exist, values byte-identical. **Under D-10 `empty`-only this becomes a plan-rejection case**: mode `explicit` is refused and no child is created |
| **V4** | Plan declares `PATH=/nonexistent` | Accepted; child sees it; the executable that ran is still the pinned one. **Under D-10 `empty`-only, likewise a plan-rejection case** |
| **V5** | Plan mode `explicit` declaring `GCONV_PATH=/tmp/helm-gconv` — a non-`LD_` name on glibc's secure-execution list | **Recorded, not predicted.** The expectation is that the plan **is accepted** and the child sees the variable. This is the evidence that the `LD_` rule is a prefix denylist and **not a closure**. Under D-10 `empty`-only it is restated as a plan-rejection case and becomes mandatory |

### F — descriptor inheritance (7)

| # | Case | Expected |
|---|---|---|
| **F1** | Baseline launch | The child's descriptor set, enumerated by `fcntl(F_GETFD)`, is **exactly** 0, 1, 2. The checker verifies the set **in the executed image**; the pre-exec set `{0,1,2,exec_fd,status_w}` is not a violation. The single transient descriptor consumed by the secondary `/proc/self/fd` cross-check is reported by number and is the only one excused |
| **F2** | Harness deliberately opens an unrelated **non-`CLOEXEC`** descriptor before launch, verified open in the parent at exec time by `fcntl(F_GETFD)` | **Under the frozen D-1 arm (i) — the owner's recorded choice — the child does not see it, and that is the only PASS.** An F2 FAIL is a FAIL of architecture falsifier 7 and **rejects the mechanism**. Under arm (ii) the child **does** see it, that is the only PASS, and it is preserved verbatim as the standing published limitation. The arm is recorded in `frozen_cases.py` before the first trial and **cannot be selected afterwards** |
| **F3** | Harness opens a `CLOEXEC` descriptor before launch | Child does not see it |
| **F4** | The exec descriptor and the exec-status pipe | Neither survives into the executed image |
| **F5** | Harness blocks `{SIGTERM, SIGUSR1}` with `sigprocmask` **and** sets `SIGPIPE` and `SIGUSR2` to `SIG_IGN` before `launch` | Child's `SigBlk` is **empty** and `SigIgn` contains nothing the launcher did not set. Any inherited blocked signal or inherited ignored disposition is a **FAIL**, on the same footing as V1's environment leak. `signal(7)`: the mask is preserved across `execve` and ignored dispositions are left unchanged |
| **F6** | Harness closes descriptors 0, 1 and 2 before `launch`, so the launcher's own pipes, exec fd and status pipe can land on 0–2 | Child sees exactly 0, 1, 2 bound to the launcher's endpoints, and `fcntl(F_GETFD)` on each returns **0**. A child stdio descriptor with `FD_CLOEXEC` set — the `dup2(fd, fd)` no-op — is a **FAIL**, and so is any per-stream `bytes_drained: 0` produced that way, or an `ExecFailed { EBADF }` from a destroyed exec descriptor |
| **F7** | The exec descriptor and the exec-status write end are arranged at **adjacent** numbers by allocating and closing intervening descriptors in the harness | Exec succeeds; the child's descriptor set is exactly `{0,1,2}`; **no `close_range` returns `EINVAL`**. `close_range(2)` fails `EINVAL` when *first* exceeds *last*, inside a region whose only exits are `execveat` and `_exit` |

### X — exec failure and admission (10)

| # | Case | Expected |
|---|---|---|
| **X1** | Capability pinned on a mode-`0755` file; the harness then clears the execute bits with `fchmod(exec_fd, 0644)` **after** admission recorded `mode_bits` and **before** `launch` | `ExecFailed { EACCES }`. The recorded `pre_exec_mode_bits` are the **pre-`fchmod`** value, proving the mode is a recorded fact and the kernel is the authority. **BLOCKED** if preflight reports `geteuid() == 0` |
| **X2** | Capability on a `#!` script | **Refused at admission** by the ELF cohort rule; `execveat` is never reached |
| **X2b** | The same script, admission bypassed in a frozen experiment-only spike mode, reaching `execveat` with `exec_fd` opened `O_RDONLY \| O_CLOEXEC` **as the architecture mandates** | Single prediction: `ExecFailed { ENOENT }`, per `execveat(2)` ERRORS and BUGS and `fs/exec.c`'s `BINPRM_FLAGS_PATH_INACCESSIBLE`. `ExecFailed { ENOEXEC }` is a **FAIL**. Under the mandated descriptor mode the interpreter is never invoked and no procfs dependency is reached |
| **X2c** | The same script, admission bypassed, with `exec_fd` opened **without** `O_CLOEXEC` | The interpreter runs and receives `/dev/fd/N` in its argv, demonstrating the procfs dependency the ELF rule exists to exclude. **Recording X2b without X2c is a FAIL of the pair**, because it would attribute a CLOEXEC artefact to scripts as a class and poison the future scripts decision |
| **X3** | Capability on a file that has never had an execute bit, mode `0644` | `ExecFailed { EACCES }`. Distinct from X1 only in that no post-admission change occurred; if the owner prefers, X3 may be dropped as redundant with X1 and the count reduced by one |
| **X4** | A 4-byte file whose contents are exactly `7F 45 4C 46`, passing the magic check at admission and reaching `execveat` | `ExecFailed { ENOEXEC }`. A non-ELF file is **not** posable here: the cohort rule refuses it at admission |
| **X5** | Capability on a directory descriptor | **Refused at admission**, `NotRegularFile` |
| **X6** | Capability on an `O_PATH` descriptor | **Refused at admission**, `DescriptorModeUnsuitable` — a **HELM admission rule, not a kernel limitation**: `execveat(2)` states `AT_EMPTY_PATH` works with `O_PATH` descriptors, and the refusal exists because the bytes cannot be measured through one |
| **X7** *(conditional)* | Capability on `helper_setid` (set-user-ID bit set, owned by the running user) | **Under D-9 "refuse at admission" — the review's recommendation — the expected outcome is refusal with `SetIdBitsPresent`.** Under D-9 "record and permit", the helper's reported euid is the file owner's; on a hosted runner where owner equals caller this is observationally identical, so the **privileged** variant is **BLOCKED** and not bypassed. Either way the case records whether the work filesystem is `nosuid`, from `/proc/self/mountinfo` |
| **X8** *(conditional)* | Executable copy placed on a mount preflight reports as `noexec` in `/proc/self/mountinfo` and that the unprivileged user can write | `ExecFailed { EACCES }`. If preflight finds no such mount, X8 is **BLOCKED**; no mount is created and no `sudo` is used |

Every X case and case E5 must produce `ExecFailed`, never `Exited { 127 }`. Conflating them is a
FAIL.

### O — output (8)

| # | Case | Expected |
|---|---|---|
| **O1** | stdout only, 4 KiB, mode `measure` | Exact count and `drained_sha256` match the frozen-recipe oracle; `completeness = CompleteAtEof` |
| **O2** | stderr only, 4 KiB, mode `measure` | Same, on the correct stream; streams never merged |
| **O3** | **Both** streams emit 8 MiB **concurrently**, far beyond the 64 KiB pipe capacity, mode `measure` | **No deadlock**; both drain; both counts exactly 8 MiB; both `completeness = CompleteAtEof`; completes in **under 10 000 ms with `timeout_ms = 60000`**. A `TimedOut` outcome is a FAIL. "No deadlock" alone does not distinguish a complete drain from a spin plus a deadline |
| **O4** | Output exceeds `MAX_CAPTURE_BYTES`, mode `capture_prefix` | Exact drained count, `drained_sha256` over **all drained bytes**, `completeness = CompleteAtEof`, retained in-memory prefix exactly at the bound with the in-memory `truncated` flag set. **`truncated` must not appear in the receipt** |
| **O5** | Child closes stdout early, keeps writing stderr | Handled; per-stream counts correct; **the launcher's total `poll()` return count for the case is at most 10 000 and its CPU time under 200 ms**. A spinning launcher satisfies "no hang" and must not pass |
| **O6** | `helper_fork` forks a descendant that inherits fds 1 and 2, writes 1 KiB to stdout, then the **direct child exits 0 immediately**; the descendant sleeps 30 s **holding both fds**. `timeout_ms = 5000` | `launch()` returns within `5000 + grace_ms + POST_EXIT_DRAIN_MS` and measurably **before** the descendant's 30 s sleep ends. `process_disposition = Exited { code: 0 }`; `stdout.bytes_drained = 1024` with the oracle digest; `stdout.completeness = WriterRetainedAfterChildExit`; stderr likewise with 0 bytes. A non-return, a bare `TimedOut`, or `completeness = CompleteAtEof` is a **FAIL and rejects the mechanism**. This is the case the original definition had no way to pose |
| **O7** | Successful exec; helper writes 4 KiB to each stream and exits 42; the harness induces a read failure on the stderr read end mid-stream | The receipt carries **both** `process_disposition = Exited { code: 42 }` **and** `stderr.completeness = CaptureFailed { errno_class }`, with stdout `CompleteAtEof` at the exact count and digest. A receipt that can report only one of two simultaneously true facts is a **FAIL** |
| **O8** | Helper writes 512 bytes to stdout and exits immediately, so POLLIN and POLLHUP arrive in the same `poll()` return. **200 trials** | Every trial: `stdout.bytes_drained = 512`, digest matches, `completeness = CompleteAtEof`. Any trial reporting fewer than 512 bytes is a **FAIL** — hangup acted on before draining |

### R — exit and signal (4)

| # | Case | Expected |
|---|---|---|
| **R1** | Helper exits 0 | `Exited { code: 0 }` |
| **R2** | Helper exits 42 | `Exited { code: 42 }` |
| **R3** | Helper raises `SIGSEGV` on itself | `Signaled { SIGSEGV, launcher_signal_issued: false }`. The helper sets `RLIMIT_CORE = 0` before raising, so no core dump is produced; `systemd-coredump` is installed on the recommended runner |
| **R4** | Host sets `SIGCHLD` to `SIG_IGN` before `launch`; helper exits 42 | **Recorded, not predicted:** whether the launcher emits `ExitStatusUnobservable` or `Exited { 42 }`. **Gated sub-assertion:** it must **never** emit `Exited { 0 }` and must **never** hang. Establishes the caller precondition that `pidfd_open(2)` NOTES states and that a library cannot enforce |

### T — timeout (6)

| # | Case | Expected |
|---|---|---|
| **T1** | Helper sleeps well past `timeout_ms` | `TimedOut`, never `Exited` |
| **T2** | Helper ignores `SIGTERM`, `grace_ms` elapses | `TimedOut { KilledByLauncher { SIGKILL } }` |
| **T3** | Helper installs a `SIGTERM` handler that sleeps `T3_GRACE_EXIT_DELAY_MS = 500` then `_exit`s 9; `timeout_ms = 2000`, `grace_ms = 5000` | `TimedOut { ExitedDuringGrace { code: 9 } }`. The schedule is frozen because "during the grace window" is otherwise a race |
| **T4** | Helper sleeps `timeout_ms - T4_DELTA_MS` then `_exit`s 9, with `timeout_ms = 5000`, `T4_DELTA_MS = 500`, `grace_ms = 5000` | `Exited { code: 9 }`. With the pidfd in the poll set the exit is **observed** against the monotonic deadline, so there is no honest ambiguity: `TimedOut { ExitedDuringGrace { 9 } }` is now a **FAIL**, and so is a bare `TimedOut` and any `KilledByLauncher` disposition |
| **T5** | Direct child exits 7 at `timeout_ms - 50 ms` while a descendant retains fds 1 and 2, so the launcher cannot see the exit unless it polls the pidfd | `process_disposition = Exited { code: 7 }` — **not** `TimedOut`, and not `TimedOut { ExitedDuringGrace { 7 } }`. Reporting a `TimedOut` disposition for a child observed to have exited before the deadline is a **FAIL** |
| **T6** | The harness parent has `SIGTERM` **blocked** and `SIGPIPE` set to `SIG_IGN`; T2's construction is then run | Identical wall-clock shape to T2 from a default-signal parent, and the child's `SigBlk` is empty. A divergence between the two parents means the signal reset of architecture section 17 is not implemented, and is a **FAIL** |

### P — process-tree negative control (4)

| # | Case | Expected |
|---|---|---|
| **P1** | `helper_fork` forks a descendant that outlives its parent; the launcher terminates the direct child | **Outcome recorded, not predicted.** The expectation is that the descendant **survives**, and that result is preserved as the standing evidence that 0.1 provides **no process-tree containment**. **Gated sub-assertion:** `launch()` returns within the declared total bound, and per-stream `completeness` correctly reports whether end-of-file was reached |
| **P2** | Same, with the descendant calling `setsid` | Recorded. Establishes that a process-group sweep is best-effort only. Same gated sub-assertion |
| **P3** | `helper_fork` with a surviving descendant; the launcher's sweep and reap are traced | **Gated:** the sweep syscall, if issued, appears **strictly before** `waitid`. A sweep issued after the reap is a **FAIL**, because the process-group ID may have been reused and the launcher would be signalling processes it never created. Descendant survival itself remains recorded |
| **P4** | P2's `setsid` descendant, additionally **retaining fds 1 and 2** | Descendant survival and sweep ineffectiveness recorded, not predicted. **Gated:** `launch()` returns within the total bound and both streams carry `completeness = WriterRetainedAfterChildExit`. Establishes that a process-group sweep does not release a pipe, and that the bounded drain — not the sweep — is what makes the launcher terminate |

**Isolation and ordering of the negative controls.** P1 and P2 run **last**, after every mandatory
and conditional case has a recorded status, and are never interleaved with them. In P1 and P2 the
descendant's **first three actions after `fork`** are to close descriptors 0, 1 and 2 and reopen
them on `/dev/null`, so it can never hold a launcher pipe open and can never make a mandatory case
hang or become ambiguous; it signals liveness by writing one byte to a harness-created FIFO, and
`_exit`s unconditionally after `P_DESCENDANT_LIFETIME_MS = 20000`. **O6 and P4 are the deliberate
exceptions**: there the descendant *must* retain the pipes, because that is the state under test,
and both are gated on bounded return rather than left open. The harness confirms before declaring
the run complete that no descendant remains, and records that confirmation. A P-case whose `fork`
did not occur, or whose descendant is still alive at run end, is **INVALID**.

P1–P4 are **not pass/fail gates on descendant survival**. They exist to establish a limitation
honestly, and their results must appear in the README and the ADR regardless of outcome. Their
**liveness sub-assertions are gates**, because a launcher that never returns is not a limitation.

### S — spawn/exec confirmation (7)

| # | Case | Expected |
|---|---|---|
| **S1** | Successful exec | Exec-status pipe reports clean EOF with no record **and** the helper's own report is received on descriptor 1 with a matching self-identity marker **and** the direct child exited normally. **The report is the exec evidence; the EOF alone is not.** Clean EOF with no report is **not** a PASS |
| **S2** | The harness opens the working directory at mode `0755`, obtains the `DirectoryCapability`, then `fchmod`s **that same descriptor** to `0000` after admission and before `launch`, so the child's `fchdir(dir_fd)` fails | A record arrives with `stage = CHDIR` and `errno = EACCES`; the result is `ExecFailed { stage: CHDIR, errno_class: EACCES }`, never `Exited { 127 }` and never an `EXEC`-stage attribution; and **no helper report is received** — the absence of the report is the independent proof that the image never ran. The harness restores `0755` afterwards. **BLOCKED** if preflight reports `geteuid() == 0`. The original "deliberately unopenable working directory" was unposable: the cwd is an already-open capability, and an unopenable directory fails at `directory_from_fd` before any child exists |
| **S3** | The helper itself exits 127 immediately after a successful exec | Reported as `Exited { 127 }` and **not** confused with exec failure |
| **S4** | **Rapid exec-and-exit.** `helper_report` runs in frozen mode `--exit-immediately 7`, whose first statement is `_exit(7)`. The spike runs in frozen mode `--post-fork-delay-ms 200`, in which the **parent** sleeps `EXEC_RACE_DELAY_MS = 200` after `fork()` returns and **before** `pidfd_open` and the poll loop. A forced schedule, not a timing hope. `traced: false` | Single prediction: `pidfd_open` **succeeds** — the child is an unreaped zombie and the launcher has issued no `waitid`; the exec-status pipe reports clean EOF with no record; `waitid(P_PIDFD, WEXITED)` returns `CLD_EXITED` status 7; outcome `Exited { code: 7 }`. `ExecFailed`, `ExecStatusIndeterminate`, `TimedOut`, a lost exit code, or `pidfd_open` returning `ESRCH` are each a **FAIL** |
| **S5** | **Child killed before exec.** Spike frozen mode `--die-before-exec`: the child completes relocation, `dup2`, `fchdir`, `close_range` and `setpgid`, then calls `kill(getpid(), SIGKILL)` **instead of** `execveat`. No record is written. `traced: false` | Single prediction: the parent observes **clean EOF with no record — byte-identical to S1 —** and receives **no helper report**, and `waitid` reports `CLD_KILLED` with `SIGKILL`. The required outcome is **`ExecStatusIndeterminate`**. Reporting exec success, or any assertion that the pinned image ran, is a **FAIL** and a FAIL of falsifier 8. This is the discriminating case for the whole confirmation design |
| **S6** | **Pre-exec stall.** The child is stopped before exec — by S2's construction extended past the bound, or by `SIGSTOP` from the harness — so `SPAWN_CONFIRM_TIMEOUT_MS` expires with no record and no EOF | `launch()` returns within `SPAWN_CONFIRM_TIMEOUT_MS + grace_ms + slack`; `process_disposition = ExecStatusIndeterminate { phase: PreExecTimeout }`; and **no surviving direct child and no zombie attributable to the launcher** after return. A leaked child or zombie is a **FAIL** |
| **S7** | S2's construction, so the record and the hangup arrive in the same `poll()` return. **200 trials** | Every trial: `ExecFailed { stage: CHDIR, errno_class: EACCES }`. Any trial reporting exec success, `Exited { 127 }`, or `ExecStatusIndeterminate` is a **FAIL and rejects the mechanism** |

S3 and S5 together are the discriminating pair for the confirmation design: S3 is the case a
naive implementation reports as exec failure, and S5 is the case it reports as exec success.

### M — mechanism minimality and parent shape (4)

| # | Case | Expected |
|---|---|---|
| **M1** *(conditional)* | Traced child window: from the return of `fork`/`clone` in the child to `execveat`. `traced: true` | The traced syscalls are **exactly** drawn from `{dup2\|dup3, fcntl(F_SETFD), fchdir, close_range, setpgid, rt_sigprocmask, rt_sigaction, write (exec-status record only), execveat, exit_group}`. Any `brk`, `mmap`, `munmap`, `mprotect`, `futex`, `openat`, `set_robust_list`, `getrandom` or `rt_sigreturn` in that window is a **FAIL**. This is the evidence that replaces architecture section 8's assertion of "no allocation, no locking, no formatting, no panic path" |
| **M2** *(conditional)* | Parent has at least three additional live threads, one with an allocation in flight and one with a registered `pthread_atfork` handler. `traced: true` | E1, F1, F5, F6 and M1 give results identical to the single-threaded arm, and no hang. Without this arm the mechanism is evidenced only for a single-threaded parent, which no real HELM caller is |
| **M3** | pidfd acquisition. Arm (a): baseline `fork` + `pidfd_open`. Arm (b): harness sets `SIGCHLD` to `SIG_IGN` in the parent and the helper exits immediately | Arm (a) **PASS**. Arm (b) **recorded, not predicted**; safe set `{pidfd_open fails ESRCH and the launcher emits ExitStatusUnobservable, waitid(P_PIDFD) fails ECHILD and the launcher emits ExitStatusUnobservable}`. A launcher reporting `Exited { code }` in arm (b) is a **FAIL**, because it cannot have observed one. If neither safe outcome is acceptable to the owner, the mechanism moves to `clone3(CLONE_PIDFD)` and M3 is re-posed with that arm |
| **M4** *(conditional)* | The frozen child-setup sequence. `traced: true` | Architecture section 17's ordered sequence — including that **`fchdir` precedes `close_range`** and that `FD_CLOEXEC` is cleared explicitly on 0/1/2 — is matched syscall-for-syscall against the M1 trace. A spike whose child sequence differs from the frozen one is **INVALID**, not PASS, because the accepted mechanism must be the documented one |

## 4. Instrumentation

Authoritative evidence, in preference order:

1. **The helper's own report**, written to **descriptor 1** behind the frozen sentinel — the
   primary evidence for argv, environ, descriptors, signal state, credentials and cwd identity,
   because it is observed from inside the executed image.
2. **A syscall record of the launcher**, collected **only** for the cases whose purpose is
   mechanism identification — E1, F4, M1, M2 and M4 — establishing that
   `execveat(exec_fd, "", …, AT_EMPTY_PATH)` was the syscall used on the pinned descriptor. It is
   collected with `strace` if preflight finds `/usr/bin/strace`, otherwise with a purpose-built
   parent-side `ptrace` tracer if preflight reports `kernel.yama.ptrace_scope <= 1`, otherwise
   **the syscall-record component of those cases is BLOCKED** and their remaining components are
   scored normally. **`strace` is not installed on the recommended `ubuntu-24.04` hosted runner
   and is not installed by this definition**; no `sudo` and no package installation is authorised.
3. **`/proc/<pid>/fd` inspection** from the harness, as a cross-check on the helper's own view.
4. **Independent oracles** in Python: `hashlib` digests over the **frozen byte recipe** of
   [section 2](#2-artefacts-to-be-committed-before-the-first-trial), never reading spike output.

**Tracing authority.** Every case declares `traced: true|false` in `frozen_cases.py`. Only E1, F4,
M1, M2 and M4 declare `traced: true`. **Every R, T, O, P and S case declares `traced: false`**,
because `ptrace(2)` delivers a tracee's exit and signal notifications to the **tracer** before the
real parent, and the launcher *is* the real parent whose `waitid` classification those cases
exist to measure. A case declared `traced: false` that is nevertheless observed under a tracer is
**INVALID**, not FAIL.

**No retries.** A case is run once per trial. A result outside its preregistered prediction or
safe set is a FAIL, and FAIL, INVALID and BLOCKED results are preserved rather than re-run.

**Freeze point.** Every artefact in section 2 is committed and hashed in `SOURCE-HASHES.json`
before any trial, and this document records that freeze commit by SHA. The report additionally
records the SHA-256 of each **built** binary, the exact build command, the compiler banner, and a
byte-identity re-verification of the committed content on the runner. **The first valid trial** is
the first execution of any preregistered case under the frozen definition on the recorded
environment; every earlier run is preparation and is not reported as a result. After it, **no
artefact in section 2 may be edited**: a defect discovered afterwards ends the trial and starts a
new, separately recorded one.

**Preflight halt.** If, before the first valid trial, a frozen expectation is found to be
factually wrong about documented Linux behaviour, execution **halts** and the correction is put to
the owner as a separate decision, exactly as
[OBS-FS-01 did](OBS-FS-01-PREFLIGHT-AND-HALT.md). Correcting an expectation after observing
behaviour is goalpost movement and is forbidden.

## 5. Privacy and sanitisation

The launcher spike is started from a **scrubbed** parent environment containing only the names
declared in `frozen_cases.py` plus the V1 canaries, and the harness records that scrubbing. The
published report replaces the runner work directory with `<WORK>` and the home directory with
`<HOME>`, and contains **no** absolute host path. Any environment name observed in a child that is
not declared in `frozen_cases.py` is reported as `<UNDECLARED:` first eight hex of its SHA-256 `>`
and its value is **never** reproduced, so a V1 leak is detectable without republishing what
leaked — on a hosted runner the inherited environment includes `ACTIONS_RUNTIME_TOKEN` and
`ACTIONS_ID_TOKEN_REQUEST_TOKEN`. Captured stream bytes are never published: only counts, digests
and completeness, following
[architecture section 20](../research/HELM-LAUNCH-ARCHITECTURE.md#20-output-privacy).

## 6. Aggregate verdict rules

**Per-case status**, one of exactly four, assigned once and never revised:

- **PASS** — the case was posed as written, its recorded outcome is its single frozen prediction
  or a member of its frozen safe outcome set, and every frozen trace invariant held.
- **FAIL** — the case was posed as written and its outcome is not that.
- **INVALID** — the case could not be *posed* as written: a fixture could not be built, a required
  forced state did not land, a required oracle could not be computed, or the case was observed
  under an instrumentation its `traced` declaration forbids.
- **BLOCKED** — the case could be posed but the *environment* cannot host it: no unprivileged
  `noexec` mount, no tracer available, `geteuid() == 0` where the case requires an unprivileged
  DAC check, or the static toolchain is absent.

A case in the frozen membership with **no recorded status is BLOCKED, never absent.** No case is
ever re-run to change its status; a re-run is a **new trial** with its own report.

**Case classes are frozen in `frozen_cases.py` before the first trial**, as `MANDATORY_CASES`,
`CONDITIONAL_CASES` and `NEGATIVE_CONTROLS`. Every case is in exactly one class, the three lists
are disjoint, and their union is the whole membership.

| Class | Count under D-1 arm (i) | Members |
|---|---|---|
| **Mandatory** | **58** | E1, E2, E3, E4, E5, E5b, E6, E6b, E6d, E7, E8 (11); A1, A2, A3, A4, A6 (5); V1, V2, V4 (3); F1–F7 (7); X1, X2, X2b, X2c, X3, X4, X5, X6 (8); O1–O8 (8); R1, R2, R3 (3); T1–T6 (6); S1–S7 (7) |
| **Conditional** | **5** | X7, X8, M1, M2, M4 |
| **Negative control / recorded** | **8** | E6c, M3, P1, P2, P3, P4, R4, V5 |
| **Total** | **71** | 11+5+3+7+8+8+3+6+7 = 58, plus 5, plus 8 |

Under D-1 arm (ii) F2 moves from mandatory to conditional and the counts are 57 / 6 / 8.

**D-1 is a frozen input, not a scoring choice.** The owner's D-1 arm is recorded in
`frozen_cases.py` before the first trial and **cannot be selected after any result is known**.
Under **arm (i)** — the owner's recorded choice: scoped `unsafe`, exact FD isolation claimed —
**F2 is mandatory**, its only PASS is that the child does not see the descriptor, and an F2 FAIL
rejects the mechanism. No aggregate verdict may describe an F2 FAIL as a documented limitation
under arm (i). Under **arm (ii)** F2 is conditional, its expected outcome is inheritance, its FAIL
is preserved as the standing published limitation, and the F-series result is explicitly recorded
as **not transferable** to `helm-launch`, because the C spike always has `close_range` available.

**Aggregate precedence.** Applied in this order; evaluation stops at the first rule that fires.
The order is **total**: every assignment of statuses to the frozen membership reaches exactly one
verdict.

| Order | Verdict | Condition |
|---|---|---|
| 1 | **MECHANISM_REJECTED** | Any mandatory case is FAIL; **or** `launch()` fails to return within its declared total bound in **any** case, including P1–P4; **or** the receipt asserts a temporal or causal fact the launcher did not observe; **or** any recorded case fails a gated sub-assertion |
| 2 | **MECHANISM_INCONCLUSIVE** | No mandatory case is FAIL, and at least one mandatory case, conditional case or negative control is INVALID or BLOCKED |
| 3 | **MECHANISM_ACCEPTED** | Otherwise: every mandatory case is PASS; every conditional case is PASS or BLOCKED with a recorded cause; and every recorded case has a non-INVALID outcome that passes its gated sub-assertions |

Precedence 1 over 2 follows OBS-FS-01's frozen precedence — FAIL before BLOCKED before
INCONCLUSIVE before PASS: a falsified claim is a result, and an unposable case never converts it
into an open question.

**A single FAIL of E2, E4, E6, E6b, O3, O6, S4, S5 or S7 rejects the mechanism outright**, because
each falsifies a claim the architecture is built on: pinned execution, the receipt's executable
identity, deadlock-free draining, bounded return, and the exec-confirmation inference.

**A launcher-side non-return is never a recordable outcome.** It is a rejection, in every case
including the negative controls. This is the single rule that closes the acceptance path the
original definition left open.

**E6, E6b and E6d are documentation gates as well as mechanism gates.** They cannot be PASSed by
observing the expected kernel behaviour while the architecture, the ADR or the receipt schema
still presents the pre-execution measurement as the identity of the executed body.

No aggregate verdict may be reported as a percentage, and there is no partial credit.

The experiment establishes **mechanism viability only**. It confers no permission to implement,
no compatibility claim, no sandbox claim and no readiness claim, and **A0-7ZIP remains
experimental FAIL** irrespective of its outcome.

## 7. Environment

**Recommended first environment: a GitHub-hosted `ubuntu-24.04` runner.** It is unprivileged,
disposable, already used by this repository's CI, and its kernel is far above the 5.9 floor.

**Preflight inventory, recorded before the first case:** `uname -a`, `/etc/os-release`,
`uname -m`, the kernel release, the **glibc version**, the target triple, `geteuid`,
`kernel.yama.ptrace_scope`, `command -v strace`, the static-link probe of
[section 2](#2-artefacts-to-be-committed-before-the-first-trial), `getconf PAGESIZE`, the pipe
capacity via `fcntl(F_GETPIPE_SZ)`, **`/proc/sys/fs/binfmt_misc/status` and every registration**,
and the `nosuid`/`noexec` mount survey from `/proc/self/mountinfo`.

**Workflow discipline.** The experiment workflow sets `cancel-in-progress: false` — the existing
workflows set it true, which would truncate a trial mid-run — has no `pull_request` trigger, and
runs only on `workflow_dispatch`. GitHub's "re-run failed jobs" is forbidden; any re-run is a
**new trial** with its own report. Evidence is preserved under `docs/experiments/evidence/`, as
OBS-FS-01 did.

**A0 must not be reused or modified for this experiment**, and no preserved A0 evidence is
opened.

If a case turns out to need a specific older kernel, a mount configuration a hosted runner
cannot provide, or a privileged operation, that case is marked **BLOCKED** and a fresh
disposable Hyper-V or WSL lab is proposed in a **separate** owner request. No `sudo`, no
privilege change and no lab modification is authorised by this definition.

## 8. What this definition does not authorise

Running the experiment; creating `crates/helm-launch`; executing Wine or 7-Zip; touching A0;
modifying any lab; changing any existing crate; or treating any result here as acceptance of
ADR-0024.

**This definition is not yet freezable.** Owner decisions **D-9** (refuse set-user-ID objects at
admission, or record and permit) and **D-10** (`empty`-only environment, or retain `explicit`)
both change case membership: D-9 flips X7's expected outcome, and D-10 converts V2, V4 and V5 from
observation cases into plan-rejection cases. The case table above is written to work either way,
but the membership cannot be committed to `frozen_cases.py` until both are ruled on. Freezing
first and deciding afterwards would be exactly the goalpost movement the
[preflight-halt rule](#4-instrumentation) exists to prevent.
