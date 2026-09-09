# LAUNCH-EXEC-01 — preregistered execution definition

**Status: PROPOSED — NOT_RUN. No trial has been executed and no result exists.**\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`.\
**Authority:** Proposed [ADR-0024](../adr/ADR-0024-launch-authority.md) and the
[helm-launch architecture](../research/HELM-LAUNCH-ARCHITECTURE.md). Neither is Accepted, and
this definition authorises nothing by itself.

This document **freezes the case set** for the first `helm-launch` mechanism experiment, before
any trial, following the discipline that worked for
[OBS-FS-01](obs-fs-01/README.md): the definition is committed first, and the mechanism is not
edited after the first valid trial.

## 0. What this experiment answers, and what it does not

> **It answers:** can HELM execute **one explicitly authorized synthetic Linux executable** with
> the process-boundary semantics the architecture claims?

> **It does not answer:** whether HELM can safely manage a Wine application lifecycle. That
> needs a separate later experiment and is out of scope here.

**No Wine. No 7-Zip. No proprietary software. No A0 lab. No privileged operation.** Only
synthetic helper executables built from source committed alongside this definition.

## 1. Scope and disposability

All experiment code is **disposable pre-implementation spike code**. It is not
`crates/helm-launch`, is not a Cargo workspace member, defines no product API, and must not be
promoted into one. Its only purpose is to falsify the mechanism described in the architecture.

The experiment may be written in C, Rust or both. A small C spike is acceptable and is expected
to make `fork`/`exec` semantics and the syscall trace clearer; product language preference must
not distort syscall evidence.

## 2. Artefacts to be committed before the first trial

Committed **before** any trial, and hashed in `SOURCE-HASHES.json` as OBS-FS-01 did:

| File | Role |
|---|---|
| `helper_report.c` | Primary synthetic helper. Reports its own observed process boundary as JSON on a descriptor chosen by argv |
| `helper_alt.c` | A second helper with a **different body and different digest**, used to detect executable substitution |
| `helper_fork.c` | Negative control: forks a descendant that outlives its parent |
| `launcher_spike.c` | The mechanism under test: pin → measure → `fork` → child setup → `execveat` |
| `frozen_cases.py` | Frozen case manifest: membership, inputs, schedules, safe outcome sets |
| `oracles.py` | Independent expected values computed without reading spike output |
| `harness.py` | Fixture builder, descriptor pre-loader, tracer driver |
| `checker.py` | Verdict evaluator; links nothing from the spike |
| `run_launch_exec_01.py` | Runner; emits one sanitised report |

`helper_report.c` must report, from inside the executed image:

- its exact `argv`, element by element, with lengths, so quoting and splitting are visible;
- its **complete** `environ` key set and the exact values, so leakage is detectable;
- every open descriptor number and its type, read from `/proc/self/fd`, so inheritance is visible;
- its working-directory identity as `st_dev`/`st_ino` of `.`, never as a pathname;
- a self-identity marker distinguishing it from `helper_alt`;
- requested stdout and stderr byte volumes, exactly as instructed by argv;
- a requested exit code, and an optional sleep.

Helpers are **statically linked** where practical, deliberately: it removes dynamic-loader
activity from the trace so that negative claims — "no other descriptor was present", "no other
file was opened" — rest on a complete trace rather than on filtering. This is the same reasoning
recorded for `spike.c` in OBS-FS-01.

## 3. Preregistered cases

Every case has a frozen expected outcome. Where scheduling makes an outcome genuinely
nondeterministic, the **safe outcome set** is preregistered here and any result outside it is a
FAIL, not a retry.

### E — executable identity and TOCTOU

| # | Case | Expected |
|---|---|---|
| **E1** | Normal pinned executable runs | `helper_report` executes; self-identity marker matches; measured SHA-256 equals an independent `hashlib` digest of the file |
| **E2** | The **pathname is replaced** with `helper_alt` after the capability is pinned, before exec | The **pinned body** runs: marker is `helper_report`. Any other result **rejects the mechanism** |
| **E3** | The original pathname is `rename`d, then `unlink`ed, after pin | Pinned body still runs; exec unaffected |
| **E4** | The same pathname now resolves to a different executable | Pinned body runs; the substituted body never executes |
| **E5** | A writable descriptor is held open on the inode at exec time | `ExecFailed { ETXTBSY }`, establishing the `deny_write_access` behaviour the architecture relies on |
| **E6** | The file is modified in place in the window between hashing and exec | **Outcome recorded, not predicted.** Safe set: `{ETXTBSY refusal, pinned-inode execution with changed bytes}`. The result determines whether [section 32](../research/HELM-LAUNCH-ARCHITECTURE.md#32-executable-identity-and-mutation) must be rewritten before implementation |

### A — argv

| # | Case | Expected |
|---|---|---|
| **A1** | Argument containing spaces: `a b c` | Arrives as **one** element, byte-identical |
| **A2** | Argument containing `; \| & $(id) ' " \`` and a newline | Arrives as **one** literal element, byte-identical; no shell involved; no expansion |
| **A3** | Empty argument `""` | Arrives as one element of length 0 |
| **A4** | Argument of exactly `MAX_ARG_BYTES` | Arrives byte-identical |
| **A5** | Argument containing a NUL byte | **Rejected at plan-parse time**; never reaches exec |
| **A6** | `argv[0]` differs from any pathname | The child observes exactly the declared `argv[0]` |

### V — environment

| # | Case | Expected |
|---|---|---|
| **V1** | Host sets `HELM_LEAK_CANARY`, `LD_LIBRARY_PATH`, `PATH`, `HOME` before launch; plan mode `empty` | Child's `environ` is **exactly empty**. Any inherited name is a FAIL |
| **V2** | Plan mode `explicit` with three entries including UTF-8 and `=` inside a **value** | Exactly those three names exist, values byte-identical |
| **V3** | Plan declares `LD_PRELOAD` | **Rejected at plan-parse time** under the `LD_` rule |
| **V4** | Plan declares `PATH=/nonexistent` | Accepted; child sees it; the executable that ran is still the pinned one |

### F — descriptor inheritance

| # | Case | Expected |
|---|---|---|
| **F1** | Baseline launch | Child's `/proc/self/fd` contains **exactly** 0, 1, 2 (plus the transient descriptor the helper itself opens to read `/proc/self/fd`, which the checker accounts for explicitly) |
| **F2** | Harness deliberately opens an unrelated **non-`CLOEXEC`** descriptor before launch | Child does **not** see it. Under decision D-1 option (ii) this case is **expected to fail**, and the failure is preserved as the documented limitation |
| **F3** | Harness opens a `CLOEXEC` descriptor before launch | Child does not see it |
| **F4** | The exec descriptor and the exec-status pipe | Neither survives into the executed image |

### X — exec failure

| # | Case | Expected |
|---|---|---|
| **X1** | Capability on a non-executable regular file | `ExecFailed { EACCES }`, never an exit status |
| **X2** | Capability on a `#!` script | **Refused at admission** by the ELF rule. A separate observation records what `execveat` *would* have done, as evidence for the future scripts decision |
| **X3** | Executable with mode `0644` | `ExecFailed { EACCES }` |
| **X4** | A truncated or non-ELF binary reaching exec | `ExecFailed { ENOEXEC }` |
| **X5** | Capability on a directory descriptor | **Refused at admission**, `NotRegularFile` |
| **X6** | Capability on an `O_PATH` descriptor | **Refused at admission**, `DescriptorModeUnsuitable` |

Every X case must produce `ExecFailed`, never `Exited { 127 }`. Conflating them is a FAIL.

### O — output

| # | Case | Expected |
|---|---|---|
| **O1** | stdout only, 4 KiB | Exact count and SHA-256 match an independent oracle |
| **O2** | stderr only, 4 KiB | Same, on the correct stream; streams never merged |
| **O3** | **Both** streams emit 8 MiB **concurrently**, far beyond the 64 KiB pipe capacity | **No deadlock**; both drain; both counts exact; completes well inside the timeout |
| **O4** | Output exceeds `MAX_CAPTURE_BYTES` | Defined bounded behaviour: exact drained count, digest over all drained bytes, `truncated: true`, retained prefix exactly at the bound |
| **O5** | Child closes stdout early, keeps writing stderr | Handled; no hang; per-stream counts correct |

### R — exit and signal

| # | Case | Expected |
|---|---|---|
| **R1** | Helper exits 0 | `Exited { code: 0 }` |
| **R2** | Helper exits 42 | `Exited { code: 42 }` |
| **R3** | Helper raises `SIGSEGV` on itself | `Signaled { SIGSEGV, by_launcher: false }` |

### T — timeout

| # | Case | Expected |
|---|---|---|
| **T1** | Helper sleeps well past `timeout_ms` | `TimedOut`, never `Exited` |
| **T2** | Helper ignores `SIGTERM`, `grace_ms` elapses | `TimedOut { KilledByLauncher { SIGKILL } }` |
| **T3** | Helper exits during the grace window | `TimedOut { ExitedDuringGrace { code } }` |
| **T4** | Helper exits just before the deadline | Safe set: `{Exited{code}, TimedOut{ExitedDuringGrace{code}}}`. Both are honest; a bare `TimedOut` with no disposition is a FAIL |

### P — process-tree negative control

| # | Case | Expected |
|---|---|---|
| **P1** | `helper_fork` forks a descendant that outlives its parent; the launcher terminates the direct child | **Outcome recorded, not predicted.** The expectation is that the descendant **survives**. If it survives, that result is preserved as the standing evidence that 0.1 provides **no process-tree containment** |
| **P2** | Same, with the descendant calling `setsid` | Recorded. Establishes that a process-group sweep is best-effort only |

P1 and P2 are **not pass/fail gates on the mechanism**. They exist to establish a limitation
honestly, and their results must appear in the README and the ADR regardless of outcome.

### S — spawn/exec confirmation

| # | Case | Expected |
|---|---|---|
| **S1** | Successful exec | Exec-status pipe reports clean EOF with no record |
| **S2** | Child setup fails before exec (deliberately unopenable working directory) | A record arrives with the correct `stage`; result is `ExecFailed`, not `Exited { 127 }` |
| **S3** | The helper itself exits 127 immediately after a successful exec | Reported as `Exited { 127 }` and **not** confused with exec failure |

S3 is the discriminating case for the whole confirmation design: it is the one place where a
naive implementation would report the wrong fact.

## 4. Instrumentation

Authoritative evidence, in preference order:

1. **The helper's own report**, written to a descriptor the harness controls — the primary
   evidence for argv, environ, descriptors and cwd identity, because it is observed from inside
   the executed image.
2. **`strace`**, if present on the runner, for the syscall-level record of the launcher: that
   `execveat` was the syscall used, with `AT_EMPTY_PATH`, on the pinned descriptor. The
   `helm-observe` review workflow already establishes the practice of inventorying runner tools
   and `ptrace_scope`, and the same inventory step is required here.
3. **A ptrace helper**, if `strace` is unavailable and `ptrace_scope` permits.
4. **`/proc/<pid>/fd` inspection** from the harness, as a cross-check on the helper's own view.
5. **Independent oracles** in Python: `hashlib` digests over the byte recipes, never reading
   spike output.

**No retries.** A case is run once per trial. A result outside its preregistered safe set is a
FAIL, and FAIL, INVALID and BLOCKED results are preserved rather than re-run. **The frozen
mechanism must not be edited after the first valid trial**; a defect discovered afterwards ends
the trial and starts a new, separately recorded one.

## 5. Aggregate verdict rules

Per case: **PASS**, **FAIL**, **INVALID** (the case could not be posed as written, e.g. the
fixture could not be built), or **BLOCKED** (the environment cannot host it, e.g. `ptrace_scope`
forbids tracing).

Aggregate:

| Verdict | Condition |
|---|---|
| **MECHANISM_ACCEPTED** | Every E, A, V, X, O, R, T and S case is PASS; F1, F3, F4 are PASS; F2 is PASS or is a preserved documented FAIL under decision D-1 option (ii); P1 and P2 have recorded outcomes |
| **MECHANISM_REJECTED** | Any E-series FAIL; or any X-series case reported as an exit status; or O3 deadlocks; or S3 is misreported |
| **MECHANISM_INCONCLUSIVE** | Any mandatory case is INVALID or BLOCKED and the environment cannot be corrected without changing the frozen definition |

**A single E2, E4 or O3 failure rejects the mechanism outright**, because each falsifies a claim
the architecture is built on: pinned execution and deadlock-free draining. No aggregate verdict
may be reported as a percentage, and there is no partial credit.

The experiment establishes **mechanism viability only**. It confers no permission to implement,
no compatibility claim, no sandbox claim and no readiness claim, and **A0-7ZIP remains
experimental FAIL** irrespective of its outcome.

## 6. Environment

**Recommended first environment: a GitHub-hosted `ubuntu-24.04` runner.** It is unprivileged,
disposable, already used by this repository's CI, and its kernel is far above the 5.9 floor. The
runner's exact kernel, architecture, glibc and tool inventory must be recorded in the report, as
the `helm-observe` review workflow already does.

**A0 must not be reused or modified for this experiment**, and no preserved A0 evidence is
opened.

If a case turns out to need a specific older kernel, a mount configuration a hosted runner
cannot provide, or a privileged operation, that case is marked **BLOCKED** and a fresh
disposable Hyper-V or WSL lab is proposed in a **separate** owner request. No `sudo`, no
privilege change and no lab modification is authorised by this definition.

## 7. What this definition does not authorise

Running the experiment; creating `crates/helm-launch`; executing Wine or 7-Zip; touching A0;
modifying any lab; changing any existing crate; or treating any result here as acceptance of
ADR-0024.
