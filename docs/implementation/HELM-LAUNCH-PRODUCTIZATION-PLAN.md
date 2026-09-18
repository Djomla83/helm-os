# helm-launch 0.1 — productization plan

> **ADR-0024: ACCEPTED 2026-09-17.**
> **PRODUCTIZATION PLAN: OWNER-REVIEWED** (2026-09-16: `HELM_LAUNCH_PRODUCTIZATION_PLAN_OWNER_REVIEW_PASSED_WITH_BOUNDED_AMENDMENTS`).
> **IMPLEMENTATION AUTHORITY: P1 AND P2, BOTH ACCEPTED.**
> **HELM-LAUNCH P1: ACCEPTED 2026-09-17** as the first product slice (portable model only); the complete 0.1 module is **not yet product-accepted**.
> **HELM-LAUNCH P2: ACCEPTED 2026-09-18** as the second product slice — capability admission and authorisation composition only, with no process creation, no process execution and no `unsafe`. Its **stop condition is SATISFIED**: admission, refusal and measurement are **green on hosted Linux x86_64**, the independent review **PASSED with 0 BLOCKER and 0 IMPORTANT**, and portable compatibility is **green on Windows and macOS**.
> **P3, P4, P5: NOT AUTHORISED.**
> **NO TRIAL #4 IS AUTHORISED** (authorised = false).

<a id="current-authority-2026-09-17"></a>

**Current authority, 2026-09-17.** The owner
[accepted revised ADR-0024 and authorised P1 only](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised).
This status block and the notes marked 2026-09-17 in sections 1.1, 16 and 18 are the only changes;
the plan's contract, slices, traceability and evidence classes are unchanged, and acceptance
promotes no row of section 3 to a stronger class. Under the P1 authority `crates/helm-launch` may be
created with a crate skeleton and portable, pure surfaces only — the validated plan model and its
deterministic parsing, the digest value type, the portable receipt model, closed non-verdict enums,
the public error vocabulary, the pure fd-layout planner, a pure lifecycle state model that performs
no OS operation, the serialisation the contract requires, compile-fail and type-boundary tests,
portable unit and property tests, and the crate README. P1 has **no process execution, no `unsafe`,
no host privilege and no experiment execution**: no `clone3`, `execveat`, syscall shim, Linux
backend, pidfd acquisition or signalling, `waitid`, `close_range`, `fchdir`, process-group or signal
manipulation, `PR_SET_NO_NEW_PRIVS` call, executable or working-directory admission, ELF or
measurement I/O, `launch`, anything that can cause a child process to exist, or reuse of the
experimental runner. Where section 16's P1 row and that boundary differ, the owner's boundary
governs. The sections below that say "Proposed" or "not authorised" record the state at 2026-09-16
and are left as written.

<a id="p2-authority-2026-09-18"></a>

**P2 authority, 2026-09-18.** The owner
[authorised HELM-LAUNCH P2](../DECISIONS.md#helm-launch-p2-authorised), the bounded
capability-admission slice of [section 16](#16-implementation-slices). Under that authority the
crate may add the `authority.rs` module of section 7.2 with safe `rustix` only: the executable
admission of [section 6.2](#62-executable-admission-admit_executablefd-ownedfd) in its accepted
order, the working-directory admission of
[section 6.3](#63-working-directory-admission-admit_working_directoryid-fd), the composition of
[section 6.4](#64-composition-authorizeplan-executable-working_directory), the
`ExecutableCapability`, `WorkingDirectoryCapability` and `AuthorizedLaunch` types with the
properties of [section 5.3](#53-per-type-properties), the bounded `AdmissionError` and
`AuthorizationRefusal` vocabularies of [section 13.2](#132-families), the admission and refusal
tests, and the capability and type-boundary tests. The APIs exist only under
`cfg(all(target_os = "linux", target_arch = "x86_64"))`.

**P2 has no process creation, no process execution, no `unsafe`, no host privilege and no
experiment execution**: no `LaunchOutcome`, `launch`, `clone3`, `execveat`, pidfd acquisition or
signalling, `waitid`, `pidfd_send_signal`, `close_range`, `fchdir` execution, signal or
process-group manipulation, `PR_SET_NO_NEW_PRIVS` call, child pipe, polling lifecycle, timeout
execution, `backend/` directory, syscall shim, inline assembly, libc call or `unsafe` operation.
**P2 may perform read-only I/O through caller-supplied descriptors**, and executable measurement
may update atime and populate the page cache. **P2 creates authority-bearing in-process values but
no function that can execute them**, because `launch` does not exist; **P2 does not authorise
process creation.** Where section 16's P2 row and this boundary differ, the owner's boundary
governs. **P3, P4 and P5 remain not authorised**, and each needs a new explicit owner decision.

<a id="p1-acceptance-sync-2026-09-17"></a>

**P1 acceptance sync, 2026-09-17.** The owner
[accepted HELM-LAUNCH P1](../DECISIONS.md#helm-launch-p1-accepted). With that decision this plan's
status block and the T40 wording are synced to the owner's clarification that `EndNotObserved`,
once the post-`SIGKILL` bound expires without an observed end, is latched as the receipt-facing
child-end fact: the section 8.5 bounded-kill-wait rule, Phase B step 4, Phase C steps 2 to 4, and
the T40 row of section 3. This resolves independent-review finding P1-DOC-01. No other contract
rule, slice, evidence class or traceability row changed. P2 and every later slice remain **not
authorised**.

This plan turns the closed LAUNCH-EXEC-01 experiment line into an implementation-ready product
design for `helm-launch` 0.1. It creates no crate, changes no code, accepts no ADR and authorises
no implementation. Trial #3's frozen result stays **`MECHANISM_REJECTED`** (70 PASS / 1 FAIL /
1 BLOCKED). The accepted X2c disposition says only that X2c does not implicate the supported ELF
launch mechanism. It does **not** say Trial #3 was `MECHANISM_ACCEPTED`, and nothing here says so.

Every "must" below is a proposed product-contract rule for owner approval. Section 18 lists what
the owner must approve before `crates/helm-launch` may be created.

**Owner review, 2026-09-16.** The owner reviewed this plan as proposed at
`930ec14b940da9b136c7d2ad024b441d47ceba6c` and
[passed it with bounded amendments](../DECISIONS.md#helm-launch-productization-plan-owner-review):
Q1 approved, Q2 approved with an exact narrowing, Q3 approved with a group-authority guard, and
three required wording amendments on measurement instability, receipt authenticity and "executes
exactly one". They are applied in place, and [section 19](#19-owner-review-amendments-2026-09-16)
lists every amended passage. The approved refinements are product design obligations to be
validated by ordinary product tests. They are not claims that LAUNCH-EXEC-01 validated them, and
approval changes no evidence class in section 3. [ADR-0024](../adr/ADR-0024-launch-authority.md) is
revised to this contract and **stays Proposed**.

**Starting state.** Branch `docs/helm-launch-architecture` at
`174caad1a3980a35b2ea841f49e05555753710b5`, `main` at `501a7fa95c4884da4fec9a20a512c2d63f2b30cc`.
Crates on the branch: `helm-app-spec`, `helm-observe`, `helm-bind`, `helm-evidence`.

## 1. Authority and evidence base

### 1.1 Precedence used by this plan

When two sources conflict, the higher row wins. The conflict is recorded in section 4, and the
older document is **not** edited. ADR-0024 is the one exception: the owner review of 2026-09-16
ordered it revised in place.

| Rank | Source | Examples |
|---|---|---|
| 1 | Current explicit owner decisions | [DECISIONS.md](../DECISIONS.md): D-1 to D-11, the [Trial #2 postmortem decisions](../DECISIONS.md#trial-002-postmortem-decisions) (S4 policy), the [X2c disposition](../DECISIONS.md#trial-003-x2c-postmortem-owner-disposition), the [owner review of this plan](../DECISIONS.md#helm-launch-productization-plan-owner-review) |
| 2 | Accepted Trial #3 result review and X2c disposition | [result review](HELM-LAUNCH-EXEC-01-TRIAL-003-RESULT-REVIEW.md), [X2c postmortem](HELM-LAUNCH-EXEC-01-TRIAL-003-X2C-POSTMORTEM.md) |
| 3 | Preserved Trial #3 evidence and frozen experiment facts | [`evidence.json`](../experiments/evidence/LAUNCH-EXEC-01-TRIAL-003-2026-09-14/evidence.json), [definition](../experiments/LAUNCH-EXEC-01-DEFINITION.md), [`launcher_spike.c`](../experiments/launch-exec-01/launcher_spike.c) at freeze `bebd8a5` |
| 4 | Current public APIs of existing crates | `crates/helm-*/src` on this branch |
| 5 | Accepted ADRs | [ADR-0021](../adr/ADR-0021-second-product-module.md), [ADR-0022](../adr/ADR-0022-observation-authority.md), [ADR-0023](../adr/ADR-0023-binding-authority.md) |
| 6 | ADR-0024 while Proposed | [ADR-0024](../adr/ADR-0024-launch-authority.md), revised to this plan on 2026-09-16; section 4 cites its pre-revision text. **2026-09-17:** revised ADR-0024 is Accepted and now ranks with the accepted ADRs of row 5 |
| 7 | Older architecture prose and pretrial hypotheses | [HELM-LAUNCH-ARCHITECTURE.md](../research/HELM-LAUNCH-ARCHITECTURE.md), proposed [ADR-0005](../adr/ADR-0005-sandbox-boundary.md) |

### 1.2 The Trial #3 environment the evidence comes from

GitHub-hosted `ubuntu-24.04` (image `20260907.300.1`), kernel `6.17.0-1022-azure`, x86_64,
glibc 2.39, gcc 13.3.0, strace 6.8, euid 1001, unprivileged. The mechanism under test was a
**statically linked C spike**, not Rust.

**Transfer gap, stated once and carried everywhere.** The evidence shows that the *mechanism
sequence* works on that kernel. It does not show that a Rust implementation in a dynamically
linked, multi-threaded Rust host reproduces the spike's child window. Product tests must
re-establish that (section 14). No new formal trial is needed to do so.

### 1.3 What the preserved evidence says per series

Read directly from `evidence.json` (`cases.*.status` and `cases.*.record.outcome`):

| Series | Trial #3 | Product relevance |
|---|---|---|
| E1–E8, E5b, E6b, E6c, E6d | all PASS; E6c recorded `ExecFailed:ETXTBSY` | pinning, measurement semantics, cohort admission |
| A1–A4, A6, V1 | PASS | literal argv, empty environment |
| F1–F7 | PASS | exactly `{0,1,2}`, signal reset |
| X1–X8, X2b | PASS | exec failure classes, refusals |
| **X2c** | **FAIL** | non-product control arm; harness defect by accepted disposition |
| O1–O8 | PASS | bounded draining, descendant-held pipes |
| R1–R4 | PASS; R4 recorded `ExitStatusUnobservable` | exit vs signal, foreign reaper |
| T1–T6 | PASS | timeout, grace, observed ordering |
| N1, N2 | PASS | `NoNewPrivs` set, and controlled |
| **N3** | **BLOCKED** `unprivileged_runner` | real privilege transition untested |
| P1–P4 | PASS; P1 `descendant_died`, P2 and P4 `descendant_survived`, P3 `sweep_issued` | no containment; a same-group descendant was recorded `descendant_died` under the spike's every-path sweep, which is no descendant-killed claim |
| S1–S7 | PASS; S4 and S5 `ExecStatusIndeterminate`, S6 `ExecStatusIndeterminate:PreExecTimeout` | conservative exec evidence |
| M1–M5 | PASS; M5 recorded `waitid_echild` | child window, multi-threaded parent, `clone3(CLONE_PIDFD)` |

## 2. Product goal and non-goals

### 2.1 Goal

> `helm-launch` 0.1 authorises and **attempts** execution of **exactly one** already-open,
> admitted, regular ELF64 x86_64 object on Linux x86_64, through the exact authorised descriptor.
> It uses a caller-declared UTF-8 argv, an exactly empty environment, an explicit
> working-directory capability, exactly descriptors 0, 1 and 2 in the executed image, bounded
> stdout/stderr measurement, a monotonic timeout and direct-child lifecycle only. It returns a
> deterministic, non-verdict `LaunchReceipt`, which never states that the attempt succeeded or
> that the measured image ran (section 10).

### 2.2 What the crate owns

* Parsing an inert `LaunchPlan` document and binding its exact-byte identity.
* Admitting an executable descriptor and a working-directory descriptor, including the
  pre-execution measurement.
* Composing plan and capabilities into one single-use `AuthorizedLaunch`.
* Creating, observing, terminating and reaping the one direct child.
* Serialising the receipt.

### 2.3 What the crate explicitly does not own

| Not owned | Where it belongs |
|---|---|
| Finding an executable, `PATH`, names, shells, command strings | nowhere in HELM 0.1 |
| Deciding *whether* to launch, from a spec, observation or binding | a future orchestrator, which does not exist |
| Wine, Proton, prefixes, `WINEPREFIX`, runtime selection | a future adapter under its own ADR and experiment |
| PWA, MicroVM, GUI, portals, sandboxing, namespaces, seccomp, cgroups | separate future modules |
| Process-tree containment or session lifecycle | cgroup-based future work |
| Install, update, snapshot, rollback, recovery | separate lifecycle authorities |
| Compatibility, readiness, success or pass/fail judgement | nowhere; no module emits it |
| Persisting or publishing child output | the caller |
| Evidence-bundle completeness | `helm-evidence` |

### 2.4 Standing non-claims

* **Not a sandbox.** The child runs with the caller's credentials and can do what the caller can.
* **No process-tree containment.** Descendants may outlive the launch (P2, P4).
* **No positive exec proof.** Clean exec-status EOF alone is not positive exec proof.
* **The measurement is not the executed bytes.** It covers the main file body before exec only.
* **No stability proof.** Admission refuses when its measurement protocol *detects* instability.
  Detecting none does not prove the absence of concurrent mutation, immutability or a snapshot
  (T51).
* **No receipt authenticity.** A serialised receipt carries zero authority, is not signed, can be
  copied or fabricated outside the crate, and is not proof of provenance by itself (T52).
* **The group sweep is cleanup, not containment.** Once group authority is established, a
  same-group background process may be terminated when `launch` completes, even after a normal exit
  (T30, T31).
* **An empty environment does not pin loaded code.** `PT_INTERP` and libraries resolve by name (E7).
* **No real privilege transition is validated** (N3 BLOCKED).
* **Exit code 0 means only that the direct child exited with status 0.**
* **Scripts are not supported.** `#!` objects are refused at admission.

## 3. Experiment-to-product traceability

**Classes.** `EXPERIMENTALLY_SUPPORTED`: a Trial #3 case PASSed on it and the X2c disposition does
not touch it. `OWNER_POLICY`: an explicit owner decision. `DOCUMENTATION_DERIVED`: primary Linux
documentation or architecture reasoning, with no repository evidence. `CURRENT_IMPLEMENTATION_PRECEDENT`:
an existing HELM crate already does it. `UNVALIDATED`: nothing in the repository establishes it yet.
Each row has one primary class. An `EXPERIMENTALLY_SUPPORTED` row is evidence about the C spike on
the Trial #3 runner, subject to the transfer gap of section 1.2. An owner approval from the review
of 2026-09-16 is named in the authority column. It makes the rule a binding 0.1 design obligation,
leaves the row's class unchanged, and is never evidence.

Component names refer to section 5 and section 7.

| ID | Product invariant | Authority / evidence | Class | Responsible component | Proposed product test | Residual limitation |
|---|---|---|---|---|---|---|
| T01 | Execution authority is an owned, already-open descriptor; no path, name or document field constructor exists | D-2; ADR-0024 authority boundary; `helm_observe::root_from_fd` pattern | OWNER_POLICY | `admit_executable` | public API has no `&Path`/`&str` executable input; `compile_fail` forging an `ExecutableCapability` | trusts the caller who opened the descriptor |
| T02 | The admitted descriptor is the `execveat` target; later path replacement changes nothing | E2, E3, E4 PASS | EXPERIMENTALLY_SUPPORTED | `ExecutableCapability` → `AuthorizedLaunch` → child plan | rename over, unlink, retarget symlink after admission; executed marker is the admitted body | `PT_INTERP` and `DT_NEEDED` still resolve by name |
| T03 | No `PATH`, no shell, no command string, no pathname exec anywhere | ADR-0024; architecture §9; A1/A2 | OWNER_POLICY | plan schema; child `execveat(fd, "", …, AT_EMPTY_PATH)` | plan with any path/command field refused; source scan finds no `execve`/`execvp`/`Command` | none inside the crate |
| T04 | Admission requires ELF64, little-endian, `EM_X86_64`, `ET_EXEC` or `ET_DYN` | E8 refused; X4 admitted and `ExecFailed:ENOEXEC` | EXPERIMENTALLY_SUPPORTED | admission header check | foreign machine, ELF32, big-endian, `ET_REL`, short file, X4-style header | a valid header proves nothing about the program |
| T05 | `#!` scripts are refused at admission | X2 PASS; X2b `ENOENT` PASS; X2c non-implicating by owner disposition | EXPERIMENTALLY_SUPPORTED | admission (`NotElf`) | script fixture refused; no child, no receipt | script execution is unsupported, not demonstrated impossible |
| T06 | `S_ISUID`/`S_ISGID` objects are refused | X7 PASS; D-9 | EXPERIMENTALLY_SUPPORTED | admission | `u+s` and `g+s` fixtures owned by the test user | file capabilities are not read; see T16/T17 |
| T07 | `O_PATH` executable descriptors are refused | X6 PASS | EXPERIMENTALLY_SUPPORTED | admission access-mode check | `O_PATH` descriptor → `DescriptorModeUnsuitable` | execute-only objects are inadmissible |
| T08 | Writable (`O_WRONLY`/`O_RDWR`) executable descriptors are refused | new product rule; E5 shows a held writer yields `ETXTBSY`; owner-approved (Q2) | UNVALIDATED | admission access-mode check | both writable modes refused before measurement | a writer elsewhere is not detected |
| T09 | Size, SHA-256 and mode bits are measured through the same descriptor before the attempt | E1 digest equals oracle; E6d and X1 record pre-change mode | EXPERIMENTALLY_SUPPORTED | admission measurement | digest equals an independent hash; `fchmod` after admission leaves recorded bits unchanged | reading updates atime and page cache |
| T10 | The measurement is never presented as the executed bytes | E6 mutated body ran; E6b safe set; E5b no `ETXTBSY`; E6c recorded | EXPERIMENTALLY_SUPPORTED | receipt field names `pre_exec_body_*`; docs | length-preserving mutation after admission runs the mutated body; receipt keeps the old digest | the measure-to-exec window is uncovered; memfd sealing not adopted |
| T11 | No `binfmt_misc` entry can route an in-cohort object to a pathname interpreter | architecture §6 | DOCUMENTATION_DERIVED | admission header check | none possible unprivileged | an entry matching native x86_64 ELF still routes; host precondition |
| T12 | Working directory is a capability, `fchdir`ed in the child before `close_range` | S2, S7 `ExecFailed:CHDIR:EACCES`; M4 order | EXPERIMENTALLY_SUPPORTED | `WorkingDirectoryCapability`; child stage `CHDIR` | child reports cwd `(st_dev, st_ino)` equal to the admitted directory; mode `0000` after admission → `CHDIR:EACCES` | search permission is checked by the kernel only at `fchdir` |
| T13 | argv elements arrive byte-identical and literal | A1, A2, A3, A4, A6 PASS | EXPERIMENTALLY_SUPPORTED | argv materialisation | spaces, metacharacters, newline, empty, 4 KiB element, arbitrary `argv[0]` | none |
| T14 | argv is UTF-8, NUL-free and bounded; violations are plan errors | D-6; frozen `OUT_OF_SCOPE` A5 | OWNER_POLICY | `parse_launch_plan` | NUL, invalid UTF-8 escape, count and byte bounds | non-UTF-8 arguments are unrepresentable |
| T15 | The child environment is exactly empty | V1 PASS; D-10 | EXPERIMENTALLY_SUPPORTED | `envp = [NULL]` | host sets canary, `PATH`, `HOME`, `LD_LIBRARY_PATH`; child `environ` empty | hardening, not provenance |
| T16 | `PR_SET_NO_NEW_PRIVS = 1` is set before exec | N1 PASS; N2 control PASS; D-11 | EXPERIMENTALLY_SUPPORTED | child stage `NO_NEW_PRIVS` | child observes `NoNewPrivs: 1` when the test process has 0 | not isolation, not privilege reduction |
| T17 | A real set-ID or file-capability transition is suppressed | N3 BLOCKED `unprivileged_runner` | UNVALIDATED | D-9 + D-11 together | none unprivileged; privileged environment deferred | rests on kernel documentation only |
| T18 | Exactly 0, 1, 2 survive; unrelated host descriptors never do, `CLOEXEC` or not | F1, F2, F3, F4, F6, F7 PASS; D-1 arm (i) | EXPERIMENTALLY_SUPPORTED | parent fd layout + child `close_range` | non-`CLOEXEC` host fd, `CLOEXEC` host fd, host 0–2 closed, adjacent numbers | none claimed beyond the executed image |
| T19 | Every child-side preserved descriptor is relocated `F_DUPFD_CLOEXEC ≥ 3` in the parent before the final stdio mapping, unconditionally | refinement of spike `move_above_2`, which moved only 0–2; owner-approved (Q2) | UNVALIDATED | fd-layout planner | property test over layouts; caller-supplied non-`CLOEXEC` exec fd does not reach the image | none |
| T20 | Child signal mask is emptied and every disposition reset to default | F5, T6 PASS | EXPERIMENTALLY_SUPPORTED | child stages `SIGACTION`, `SIGMASK` | host blocks `SIGTERM`, ignores `SIGPIPE`; child masks clean; T6-shaped timeout | none |
| T21 | The calling thread blocks every blockable signal across `clone3` with a raw full-set `rt_sigprocmask` and restores its saved mask after; the child resets dispositions while delivery is blocked and sets the final mask only then | refinement; not in the frozen stage order; owner-approved (Q2) | UNVALIDATED | parent pre-clone; child stage order | host handler installed; the traced parent mask includes glibc's internal real-time signals; a signal sent in the window does not run host code in the child | `SIGKILL`/`SIGSTOP` are not blockable; controls the calling thread and the child's inheritance only, never process-wide delivery in a multithreaded caller |
| T22 | stdin is a pipe whose write end the parent closes, so the child reads EOF | `pipe(7)`; architecture §18 | DOCUMENTATION_DERIVED | parent pipe setup | fixture reads stdin and reports immediate EOF | no interactive input in 0.1 |
| T23 | stdout and stderr drain concurrently without deadlock, spin or unbounded memory | O1–O5, O8 PASS | EXPERIMENTALLY_SUPPORTED | parent poll loop | 8 MiB both streams; early close; `POLLIN`+`POLLHUP` 200 repetitions; poll-return bound | timing depends on host load |
| T24 | Raw output bytes stay in memory; the receipt holds counts, digests and completeness only | helm-observe/helm-bind artifacts carry no payload; experiment `PRE-D7-B1` kept prefixes internal | CURRENT_IMPLEMENTATION_PRECEDENT | `LaunchOutcome` vs `LaunchReceipt` | receipt bytes never contain a canary the child printed; `Debug` of outcome redacts | hashing does not stop the child reading secrets |
| T25 | Only the direct child's lifecycle is claimed | D-4 as corrected | OWNER_POLICY | lifecycle loop; docs | P1/P2/P4-shaped tests record survival without failing | descendants may outlive the launch |
| T26 | Process identity is a pidfd, polled from creation; exit is observed against the deadline | T4, T5 PASS | EXPERIMENTALLY_SUPPORTED | parent poll set | exit just before deadline with retained stdio → `Exited` | scheduling granularity |
| T27 | The pidfd comes from the same `clone3(CLONE_PIDFD)` that creates the child, from a multi-threaded parent | M3 traced direct form PASS (bounded claim); M2 PASS; M5 recorded `waitid_echild` | EXPERIMENTALLY_SUPPORTED | backend `clone3` | traced test: one process `clone3` with `CLONE_PIDFD`, no `pidfd_open`; run from the multi-threaded test harness | M3 claim covers one traced execution only |
| T28 | The Rust child window issues only the closed syscall set: no allocation, lock, formatting or panic | M1/M4 evidence is for the C spike | UNVALIDATED | unsafe backend child path | traced window vs closed set; source-token scan; `needs_drop` assertion | proof is per build configuration |
| T29 | Timeout sends the plan's `SIGTERM` by pidfd, waits `grace_ms`, then `SIGKILL` | T1, T2, T3, T6 PASS | EXPERIMENTALLY_SUPPORTED | lifecycle loop | sleeper, `SIGTERM`-ignoring child, exit during grace | `waitid` gives a signal number, not a sender |
| T30 | With group authority established (T31), exactly one `SIGKILL` process-group sweep is issued on every completion path, including a normal exit, strictly before the reap | P3 PASS; P1 `descendant_died`; spike line 988; owner-approved (Q3) | EXPERIMENTALLY_SUPPORTED | lifecycle loop | traced order sweep < `waitid`; same-group descendant of a normally exiting child ends; `setsid` descendant survives | best-effort cleanup; `setsid`/`setpgid` escape |
| T31 | No sweep without positively established group authority: only the launcher's own successful `setpgid(child, child)` establishes it, no group id is inferred, and a child observed already reaped elsewhere gets no sweep | architecture §24 "only while unreaped"; owner-approved guard (Q2, Q3) | UNVALIDATED | spawn step `setpgid`; lifecycle loop (`waitid WNOWAIT`) | test-only parent delay past exec → `EACCES`, sweep not issued, no `kill` in the trace; host `SIGCHLD = SIG_IGN` → sweep not issued | a parent `setpgid` that loses the race to exec leaves the image in its dedicated group but grants no sweep authority; a foreign reaper acting between the probe and the sweep |
| T32 | After the direct child ends, draining stops after `POST_EXIT_DRAIN_MS`, recorded `WriterRetainedAfterChildExit` | O6, O7, P4, T5 PASS | EXPERIMENTALLY_SUPPORTED | lifecycle loop | descendant holds 1 and 2 for 30 s; launch returns within bound | closing the read end may `SIGPIPE` the descendant |
| T33 | The run timeout is monotonic and starts at exec-status EOF | T1–T5 PASS; spike `deadline = now + timeout_ms` at EOF | EXPERIMENTALLY_SUPPORTED | lifecycle loop | T4-shaped schedule | EOF is not exec proof (T36) |
| T34 | The pre-exec phase is bounded; on expiry the child is `SIGKILL`ed with the bounded kill wait (T40), swept only under group authority (T31), and reaped once its end is observed | S6 PASS | EXPERIMENTALLY_SUPPORTED | lifecycle loop | test-only stall before exec → `pre_exec_status_timeout`, no zombie | exec may have happened just after the bound |
| T35 | A structured pre-exec failure carries stage and errno and is never an exit status | X1, X3, X4, X8, E5, E6d, S2, S7 PASS; S3 `Exited:127` distinct | EXPERIMENTALLY_SUPPORTED | exec-status record | `EACCES`, `ENOEXEC`, `ETXTBSY`, `CHDIR:EACCES`; helper exiting 127 is `Exited` | errno is host-kernel specific |
| T36 | Clean exec-status EOF alone is not positive exec proof | Trial #2 owner decision 2; X2c disposition | OWNER_POLICY | outcome model | no public API or receipt field maps EOF to exec success | 0.1 cannot say "the image ran" |
| T37 | Death before exec or a short record is `indeterminate`, never success | S5 PASS; S4 launcher claim | EXPERIMENTALLY_SUPPORTED | outcome model | test-only death before exec → `indeterminate`, byte-identical status view to a normal run | — |
| T38 | Exit status and signal termination stay distinct, including `CLD_DUMPED` | R1, R2, R3 PASS; S3 PASS | EXPERIMENTALLY_SUPPORTED | `waitid` classification | exit 0, exit 42, `SIGSEGV` with and without core, `SIGABRT` | none |
| T39 | A child reaped elsewhere yields an unobservable end, never a guessed status | R4 recorded `ExitStatusUnobservable`; M5 `waitid_echild` | EXPERIMENTALLY_SUPPORTED | `waitid` classification | host `SIGCHLD = SIG_IGN` | caller precondition, not enforced |
| T40 | Every `SIGKILL` the launcher sends to the direct child by pidfd starts a wait bounded by `POST_KILL_REAP_MS`; a child whose end is still not observed is recorded as `end_not_observed`, latched against any later reap result or `ECHILD` (8.5), and left unreaped if nothing is collected, and `launch` still returns | architecture falsifier 21; spike `waitid` blocks; owner-approved (Q2) | UNVALIDATED | lifecycle loop | simulated backend only; no unprivileged way to force an unkillable child | an unreaped child is left to the host; no claim that it is still running later |
| T41 | A stream read failure is its own completeness value, never EOF | architecture §30; spike treated read errors as EOF; owner-approved (Q2) | UNVALIDATED | lifecycle loop | simulated backend injects `EIO` | — |
| T42 | A dynamically linked ELF launches, and the empty environment does not pin its loaded code | E7 PASS | EXPERIMENTALLY_SUPPORTED | none beyond T02/T15 | dynamic fixture (every Rust fixture is dynamic) | loaded-code closure unmeasured |
| T43 | The durable receipt has no timestamp and no elapsed duration | D-8 | OWNER_POLICY | receipt serializer | schema test; field list closed | elapsed time exists only in memory |
| T44 | No term the crate emits is a verdict word | helm-bind vocabulary tests; ADR-0023 | CURRENT_IMPLEMENTATION_PRECEDENT | model enums, serializer | exhaustive vocabulary scan | caller-chosen ids are data |
| T45 | Nothing is described as sandboxing or containment | D-4; ADR-0024 non-sandbox boundary; proposed ADR-0005 principle | OWNER_POLICY | README, docs, vocabulary | vocabulary and doc scan | — |
| T46 | A refusal produces no receipt | helm-bind refusal rule | CURRENT_IMPLEMENTATION_PRECEDENT | error model | every refusal path returns `Err` with no receipt value | — |
| T47 | `authorize` consumes plan and capabilities; `launch` consumes the authorisation | `helm_observe::authorize` obligation C | CURRENT_IMPLEMENTATION_PRECEDENT | type signatures | `compile_fail` reuse and substitution tests | — |
| T48 | Any integrity digest meant for independent recomputation commits to published bytes | X2c disposition §7 (R3-M1) | OWNER_POLICY | receipt serializer | `sha256(exact_bytes) == sha256()`; no withheld field exists | — |
| T49 | Kernel floor is 5.9 (`close_range`) | architecture §5 | DOCUMENTATION_DERIVED | cohort statement | none; CI runs one kernel | evidence exists only for `6.17.0-1022-azure` |
| T50 | A test that reads a fixture report must prove the executed fixture can emit it | X2c disposition §8 | OWNER_POLICY | test harness | producer self-test before any consumer assertion | applies to tests, not to the product |
| T51 | Admission is refused when the measurement protocol **detects** instability: the second metadata sample differs from the first in size, `st_mtime` or `st_ctime`, or the byte count read differs from the first sample's size. Detecting none is never reported as stability | owner amendment 1 (2026-09-16); `helm-observe` before/after sampling (`changed_during_read`) | CURRENT_IMPLEMENTATION_PRECEDENT | admission measurement | test-only hook between the read loop and the second sample changes the size, or rewrites content with a timestamp change → refused; no test asserts that every concurrent change is detected | a change that moves no sampled field between the samples is not detected; no immutability, snapshot or measured-equals-executed claim |
| T52 | No receipt-authenticity claim: a serialised receipt carries zero authority, is not signed, can be copied or fabricated outside the crate, and is not proof of provenance by itself | owner amendment 2 (2026-09-16) | OWNER_POLICY | receipt docs, README, vocabulary | no API turns receipt bytes into a `LaunchReceipt` value; documentation and vocabulary scan finds no authenticity, signature or provenance claim | provenance and bundle validation belong above `helm-launch` |

**Counts:** EXPERIMENTALLY_SUPPORTED **26**, OWNER_POLICY **10**, DOCUMENTATION_DERIVED **3**,
CURRENT_IMPLEMENTATION_PRECEDENT **5**, UNVALIDATED **8**. T51 and T52 were added by the owner
review of 2026-09-16; no existing row changed class.

**Load-bearing UNVALIDATED rows.** T28 (Rust child window) and T21 (signal blocking across
`clone3`) carry the safety of the unsafe backend. T19, T31, T40 and T41 are product refinements
that close gaps the spike left open. T08 is a new admission rule. The owner approved T08, T19, T21,
T31, T40 and T41 as 0.1 design obligations on 2026-09-16. Approval does not validate them: product
tests must. T17 (N3) stays a non-claim and is not made load-bearing: no product text may rely on a
demonstrated privilege suppression.

## 4. Superseded or refined pretrial assumptions

None of the documents named here is edited by this plan. The product follows the right-hand
column. "Superseded" means a higher-ranked source contradicts the statement. "Refined" means the
statement still holds but is narrower or more specific than its wording.

**ADR-0024 citations below are to its pre-revision Proposed text.** The owner review of 2026-09-16
had ADR-0024 revised in place to this plan. The cited text is the one the Trial #3 freeze bound by
SHA-256 `c1f3cce88438439aab6632a9c56450ae98d1c64adb31b13c742e2febb1476351`; it remains at freeze
commit `bebd8a5` and is unchanged through `930ec14`.

| # | Pretrial statement and location | Later authority | Status | Product rule |
|---|---|---|---|---|
| S-01 | "`fork()`, async-signal-safe child setup, then `execveat`" — ADR-0024 *Proposed mechanism*; architecture §7 recommendation box; PROJECT_STATE design section | architecture §22 (corrected) makes `clone3(CLONE_PIDFD)` primary; spike uses `clone3`; M3 traced PASS; M5 records the rejected arm | **Superseded** | `clone3` with `CLONE_PIDFD` and `exit_signal = SIGCHLD`; no `fork`, no `pidfd_open` for the child (section 7.3) |
| S-02 | `pidfd_open` preconditions (no `SIGCHLD = SIG_IGN`, no `SA_NOCLDWAIT`, no other reaper) are caller preconditions — ADR-0024 process identity | M3 removes the wrong-process hazard; R4 and M5 show the exit *status* is still lost under a foreign reaper | **Refined** | the pidfd names the right process unconditionally; exit-status observability remains a caller precondition, reported as `end_unobservable` |
| S-03 | glibc `fork()` runs `pthread_atfork` handlers in the child — architecture §7, §8 | raw `clone3` runs none; M2 PASS with a registered handler | **Superseded for 0.1** | no atfork surface. Inherited host *signal handlers* remain a pre-`SIGACTION` hazard, closed by T21 |
| S-04 | "clean EOF, no record, **and the direct child exited normally** → exec succeeded" — architecture §21 table; ADR-0024 "exec is concluded from clean EOF together with the direct child's normal exit" | Trial #2 owner decision 2; Trial #3 S4 predicts and PASSes `ExecStatusIndeterminate` beside `Exited:7`; X2c disposition | **Superseded** | EOF plus any disposition never yields an exec-success claim (section 10) |
| S-05 | Exec success can be evidenced — architecture §3 "`execveat` on that exact object succeeded" | the only positive evidence in Trial #3 came from test fixtures (helper report, recipe payload, FIFO signal), never from the launcher | **Superseded for the product** | the receipt has no exec-success field; test fixtures may supply evidence in tests only |
| S-06 | `Signaled { signal }` from `waitid` — architecture §30 | Trial #2 R3 lost the signal under `CLD_DUMPED`; Trial #3 R3 PASS after the fix | **Refined** | `CLD_KILLED` and `CLD_DUMPED` both carry `si_status` as the signal; `core_dumped` recorded separately |
| S-07 | Relocate preserved descriptors above 2 "if any of them is 0, 1 or 2" — architecture §17 step 0; spike `move_above_2` | F6, F7 PASS for that rule. The spike opened its own exec fd `O_CLOEXEC`; a product receives caller descriptors of unknown `CLOEXEC` state, and X2c shows a non-`CLOEXEC` exec fd reaches the executed image | **Refined** | unconditional `F_DUPFD_CLOEXEC ≥ 3` for every child-needed descriptor (T19) |
| S-08 | Stage order `SIGMASK` then `SIGACTION` — definition §2; M4 PASS | M4 proves the spike matched the frozen order. It does not consider host handlers running in the child before `SIGACTION` | **Refined, owner-approved (Q2)** | parent blocks every blockable signal across `clone3`; child runs `SIGACTION` then `SIGMASK` (section 8.3, section 8.4) |
| S-09 | The group sweep is "a best-effort sweep target" — ADR-0024 lifecycle limit; architecture §24 | spike issues `kill(-child, SIGKILL)` on **every** return path, including a normal exit; P1 records `descendant_died` for a same-group descendant | **Refined, owner-approved with a group-authority guard (Q3)** | with group authority established, the one sweep is `SIGKILL` and runs on every completion path before the reap; a same-group background process left by a normally exiting program may be terminated. Best-effort cleanup, never containment (T30, T31) |
| S-10 | The sweep is safe because it precedes the reap — architecture §24 | true only while the child is an unreaped zombie; under a foreign reaper (R4) the group id may already be free. The spike ignored its own `setpgid` result | **Refined, owner-approved (Q2, Q3)** | sweep authority comes only from the launcher's own successful `setpgid(child, child)`; a non-blocking `WNOWAIT` `waitid(P_PIDFD)` probe runs just before the sweep, and `ECHILD` means no sweep (T31) |
| S-11 | Pre-exec timeout: "termination signal, `grace_ms`, `SIGKILL`" — architecture §23 | spike and S6 use immediate `SIGKILL` by pidfd | **Superseded** | immediate `SIGKILL`, then the bounded kill wait (T40). Before `SIGACTION` the child may still carry host handlers; blocked delivery keeps them from running (T21), and `SIGKILL` involves no handler at all |
| S-12 | The timeout "starts after confirmed exec" — ADR-0024; architecture §23 | it starts at clean exec-status EOF, which is not exec confirmation (S-04) | **Refined** | name the event `exec_status_eof`, never "exec confirmed" |
| S-13 | Timeout sub-disposition `KilledByLauncher { signal }` — architecture §23, §30 | architecture §23 itself says it does not attest causation; falsifier 20 | **Refined, owner-approved (Q2)** | record `deadline_expired`, which signals were sent, and the observed end separately; no causal name (section 10) |
| S-14 | "Admission pins the cohort in the ELF header, so no `binfmt_misc` entry … can route a HELM capability" — ADR-0024; architecture §6 | holds only for entries that do not match in-cohort x86_64 ELF bytes. A root-registered entry matching native x86_64 still routes | **Refined** | host precondition, stated; no `/proc/sys/fs/binfmt_misc` read in 0.1 (T11) |
| S-15 | `execve("/proc/self/fd/N")` retained as a documented fallback "only if LAUNCH-EXEC-01 falsifies the primary" — ADR-0024; architecture §7 | the primary was not implicated (X2c disposition); scripts stay refused | **Superseded** | no procfs fallback exists in 0.1 |
| S-16 | Stream completeness `DeadlineTruncated` and `CaptureFailed { errno_class }` — architecture §30; ADR-0024 | the spike emits neither; a read error became EOF. O7's "capture failure" was operationalised as `WriterRetainedAfterChildExit` (definition §9.6) | **Refined, owner-approved (Q2), UNVALIDATED** | closed product vocabulary in section 11.3; read failure distinct from EOF (T41) |
| S-17 | `environment_mode` from `{empty, explicit}`; `LD_` refusal; environment bounds — architecture §15, §29, §33, §34 | D-10 | **Superseded** | `{empty}` only; no environment bounds or name rules exist in 0.1 |
| S-18 | Per-stream plan modes `discard` / `measure` / `capture_prefix` — architecture §19 | the spike always counted and hashed; prefix retention is the only variable | **Refined** | every stream is always counted and hashed; the plan chooses only `capture_prefix_bytes` |
| S-19 | "Static posability" proves a case can be observed — definition `driver.unposable_cases` | X2c disposition: transport availability is not producer capability | **Superseded for tests** | product tests prove the executed fixture can emit the evidence a test reads (T50) |
| S-20 | Aggregate/summary digests over internal records — definition §9.3 `aggregate_input_digest` | R3-M1 (X2c disposition §7) | **Superseded for future contracts** | a receipt digest is over exact published bytes and no field is ever withheld (T48) |
| S-21 | "`crates/helm-launch` must not be created before [LAUNCH-EXEC-01] has run and been reviewed" — ADR-0024 | Trial #3 ran and was reviewed; the X2c disposition closed the trial line | **Precondition met; authority still absent** | creating the crate still needs the owner gate of section 18 |
| S-22 | Supported cohort "kernel 5.9 or newer" — ADR-0024; architecture §5 | evidence exists for `6.17.0-1022-azure` only | **Refined** | state the floor as documentation-derived and the evidenced kernel separately (T49) |
| S-23 | A C spike's M1 child-window evidence stands in for the product — definition §1 | the definition itself calls the spike disposable and not transferable as code | **Refined** | the Rust child window is re-established by product tests (T28) |
| S-24 | `helm-launch` "executes **exactly one**" authorised object — architecture §4 recommendation box; ADR-0024 *Proposed decision* | owner amendment 3 (2026-09-16); Trial #2 decision 2 (S4 policy) | **Refined** | authorises and **attempts** execution of exactly one admitted object through the exact authorised descriptor; nothing states that the attempt succeeded or that the measured image ran (section 10) |
| S-25 | The receipt is "an exact identity-bearing artifact" — architecture §29; ADR-0024 receipt semantics | owner amendment 2 (2026-09-16) | **Refined** | the digest identifies receipt bytes, not their origin; a receipt carries zero authority and no authenticity claim (T52) |

## 5. Public API boundary

### 5.1 Five layers, five kinds of type

```text
DATA        untrusted plan bytes                     &[u8]
INTENT      parse_launch_plan ─────────────────────▶ ValidatedLaunchPlan           no authority, portable
AUTHORITY   admit_executable(OwnedFd) ─────────────▶ ExecutableCapability          Linux x86_64 only
            admit_working_directory(id, OwnedFd) ──▶ WorkingDirectoryCapability    Linux x86_64 only
AUTHORISED  authorize(plan, exe, cwd) ─────────────▶ AuthorizedLaunch              single use, Linux x86_64 only
EXECUTION   launch(AuthorizedLaunch) ──────────────▶ LaunchOutcome                 Linux x86_64 only
OBSERVED    LaunchOutcome::receipt() ──────────────▶ LaunchReceipt                 portable model
```

> **CAN PARSING UNTRUSTED BYTES EVER PRODUCE EXECUTION AUTHORITY? NO.**
> No function turns bytes, a string, a path, a `ValidatedAppSpec`, a `BindingReport`, an
> `ObservationArtifact` or any `serde` input into an `ExecutableCapability`,
> `WorkingDirectoryCapability` or `AuthorizedLaunch`. The only inputs that carry authority are
> owned descriptors a trusted caller moves in.

### 5.2 Provisional signatures

Names are provisional. They are derived from the current requirements, not kept from the
architecture sketch: `executable_from_fd` becomes `admit_executable` because the call performs
I/O and can refuse, `DirectoryCapability` becomes `WorkingDirectoryCapability` because the
directory has exactly one role, and the single `LaunchRefusal`/`LaunchError` pair is split by
when the failure happens (section 13).

```rust
// ---- portable: compiled and tested on every platform ---------------------------
pub fn parse_launch_plan(bytes: &[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors>;

pub struct Digest([u8; 32]);                       // crate-local, like every HELM crate

pub struct ValidatedLaunchPlan { /* private */ }
impl ValidatedLaunchPlan {
    pub fn exact_bytes(&self) -> &[u8];
    pub fn sha256(&self) -> Digest;
    pub fn argv(&self) -> &[String];               // readable data, no authority
    pub fn working_directory_id(&self) -> &str;
    pub fn timeout_ms(&self) -> u32;
    pub fn grace_ms(&self) -> u32;
    pub fn capture_prefix_bytes(&self, stream: Stream) -> u32;
    pub fn asserted_subject_spec_sha256(&self) -> Option<Digest>;
    pub fn asserted_binding_report_sha256(&self) -> Option<Digest>;
}

pub struct LaunchReceipt { /* private: bytes, sha256, record */ }
impl LaunchReceipt {
    pub fn exact_bytes(&self) -> &[u8];
    pub fn sha256(&self) -> Digest;
    pub fn record(&self) -> &ReceiptRecord;        // closed, #[non_exhaustive] fact enums
}

// ---- Linux x86_64 only: #[cfg(all(target_os = "linux", target_arch = "x86_64"))] ------
pub fn admit_executable(fd: OwnedFd) -> Result<ExecutableCapability, AdmissionError>;
pub fn admit_working_directory(id: &str, fd: OwnedFd)
    -> Result<WorkingDirectoryCapability, AdmissionError>;

pub fn authorize(
    plan: ValidatedLaunchPlan,                     // consumed
    executable: ExecutableCapability,              // consumed
    working_directory: WorkingDirectoryCapability, // consumed
) -> Result<AuthorizedLaunch, AuthorizationRefusal>;

pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError>; // consumed

pub struct LaunchOutcome { /* private */ }
impl LaunchOutcome {
    pub fn receipt(&self) -> &LaunchReceipt;
    pub fn stdout_prefix(&self) -> &[u8];          // in memory only, never serialised
    pub fn stderr_prefix(&self) -> &[u8];
    pub fn prefix_truncated(&self, stream: Stream) -> bool;
    pub fn elapsed(&self) -> core::time::Duration; // monotonic, in memory only
    pub fn into_receipt(self) -> LaunchReceipt;
}
```

`launch` is synchronous and blocks the calling thread for at most the total bound of section 8.6.
It is not async, not re-entrant from a signal handler, and not `Sync`-shared.

### 5.3 Per-type properties

| Type | Who constructs it | I/O at construction | Owns a descriptor | Execution authority | `Clone` | Serialisable | May hold private bytes | Crosses platforms | Can trigger execution |
|---|---|---|---|---|---|---|---|---|---|
| `ValidatedLaunchPlan` | `parse_launch_plan` only | none | no | **none** | yes | exact input bytes only (`exact_bytes`); no `Serialize`/`Deserialize` | caller argv, by the caller's choice | yes | no |
| `Digest` | parser, admission, serializer | none | no | none | `Copy` | hex | no | yes | no |
| `ExecutableCapability` | `admit_executable` only | `fcntl(F_GETFL)`, `fstat`, `pread` + SHA-256 | **yes**, the admitted object | selects the object; cannot run alone | **no** | no | no | Linux x86_64 only | no |
| `WorkingDirectoryCapability` | `admit_working_directory` only | `fcntl(F_GETFL)`, `fstat` | **yes** | selects the cwd; cannot run alone | **no** | no | no | Linux x86_64 only | no |
| `AuthorizedLaunch` | `authorize` only | none | **yes**, both | **the only executable value** | **no** | no | caller argv | Linux x86_64 only | yes, once, through `launch` |
| `LaunchOutcome` | `launch` only | — | no (every descriptor closed before return) | none | **no** | **no**; `Debug` prints prefix lengths, never bytes | **yes**: output prefixes | Linux x86_64 only | no |
| `LaunchReceipt` | `launch` only, as a Rust value; receipt **bytes** are data anyone can write (T52) | — | no | none | yes | exact bytes | **no** by construction | model is portable | no |
| `ReceiptRecord` and fact enums | the serializer's input | — | no | none | yes | via the receipt only | no | yes | no |
| `LaunchPlanErrors`, `AdmissionError`, `AuthorizationRefusal`, `LaunchError` | the crate | — | no | none | yes | fixed codes, no host strings | no | yes (Linux-only variants inert elsewhere) | no |

`ExecutableCapability`, `WorkingDirectoryCapability` and `AuthorizedLaunch` are `Send` (moving
an owned descriptor to another thread is legitimate) and not `Sync`. None has a public
constructor, `Default`, `Deserialize`, `From`, setter or `Clone`. A refused `authorize` drops the
capabilities and closes their descriptors, as `helm_observe::authorize` drops its roots; errors
never carry authority-bearing values.

**Non-constructibility is an in-process API property, not authenticity.** It stops a caller of the
safe API from building an authority-bearing value from data. It says nothing about serialised
receipt bytes. A durable receipt is deterministic data with zero execution authority. It may be
copied, it may be fabricated outside the crate, it is not cryptographically signed, and it is not
proof of provenance by itself. `helm-launch` makes **no receipt-authenticity claim**. Provenance
and bundle validation belong above it (section 11.5, T52).

### 5.4 Misuse made inexpressible

| Misuse | Why it cannot be written | Guard test |
|---|---|---|
| Authorise executable A, execute B | `authorize` consumes the capability; `launch` takes no executable | `compile_fail` |
| Authorise plan A, execute plan B | `launch` takes no plan | `compile_fail` |
| Replay one authorisation | `launch` takes `AuthorizedLaunch` by value; not `Clone` | `compile_fail` |
| Forge a capability or authorisation from data | private fields, no constructor, no `Deserialize` | `compile_fail` per type |
| Obtain an in-memory `LaunchReceipt` from bytes | no constructor, no `Deserialize`. This does **not** make receipt bytes authentic (T52) | `compile_fail` |
| Use a binding or observation as permission | no HELM crate is linked, so no such type is nameable (section 12) | dependency test on `cargo metadata` |
| Obtain authority on an unsupported platform | the types and functions do not exist off Linux x86_64 | cross-platform build |
| Map an outcome to success | no `is_success`, `ok()`, `bool` conversion or verdict enum exists | source scan for verdict names and `-> bool` on outcome types |
| Leak output into an artifact | the receipt serializer has no input carrying bytes | receipt canary test |

## 6. Capability and admission model

### 6.1 The `LaunchPlan` document

Exact-byte SHA-256 identity with no canonicalisation, a strict scanner that rejects duplicate
decoded keys at every depth before any `serde_json::Value` is built, closed fields, and
deterministic ordered errors. This is the pattern `helm-observe`'s `parse_plan` and
`helm-app-spec` already implement.

```json
{
  "schema": "helm-launch-plan",
  "version": "0.1",
  "execution_kind": "linux_exact_executable",
  "argv": ["tool", "--flag", "a b; c"],
  "environment": { "mode": "empty" },
  "working_directory": { "capability_id": "workdir" },
  "stdin": { "mode": "closed_pipe_eof" },
  "stdout": { "capture_prefix_bytes": 0 },
  "stderr": { "capture_prefix_bytes": 4096 },
  "timeout_ms": 30000,
  "termination": { "signal": "SIGTERM", "grace_ms": 5000 },
  "asserted_context": {
    "subject_spec_sha256": "…64 lowercase hex…",
    "binding_report_sha256": "…64 lowercase hex…"
  }
}
```

| Field | Rule |
|---|---|
| `schema`, `version`, `execution_kind` | exact constants; `execution_kind` is a closed set with one 0.1 member |
| `argv` | 1 to `MAX_ARGS` elements; each UTF-8, no NUL, at most `MAX_ARG_BYTES`; total at most `MAX_ARGV_TOTAL_BYTES`; `argv[0]` is caller data with no default |
| `environment.mode` | `empty` only (D-10) |
| `working_directory.capability_id` | identifier grammar `[a-z0-9][a-z0-9._-]{0,79}`, as in `helm-observe` |
| `stdin.mode` | `closed_pipe_eof` only |
| `stdout`/`stderr.capture_prefix_bytes` | 0 to `MAX_CAPTURE_BYTES`; counting and hashing always happen |
| `timeout_ms` | 1 to 600 000 |
| `termination` | `signal` is `SIGTERM` only in 0.1 (the only evidenced first signal); `grace_ms` 0 to 60 000; `SIGKILL` follows, fixed |
| `asserted_context` | optional; each member optional; digest grammar only; never compared, fetched or interpreted |

Absent from the schema on purpose: any executable path or name, any host path, any descriptor
number, any shell or command string, any environment entry, any expected exit code, success
condition or predicate, and any Wine, prefix or runtime field.

### 6.2 Executable admission: `admit_executable(fd: OwnedFd)`

Performed in this order, on the descriptor the caller moved in, with no name resolved at any step:

1. **Access mode.** `fcntl(F_GETFL)`. `O_PATH` → `DescriptorModeUnsuitable` (X6).
   `O_WRONLY` or `O_RDWR` → `DescriptorModeUnsuitable` (T08, owner-approved): a writable
   capability is also a mutation authority, and on the evidenced kernel a writer held at exec time
   yielded `ETXTBSY` (E5), a kernel behaviour the refusal does not rely on.
   Only `O_RDONLY` passes.
2. **Regular file.** `fstat`; not `S_IFREG` → `NotRegularFile` (X5). This is the first metadata
   sample.
3. **Set-ID.** `S_ISUID` or `S_ISGID` → `SetIdBitsPresent` (X7, D-9).
4. **Size bound.** `st_size > MAX_EXECUTABLE_BYTES` (512 MiB, the owner-approved initial 0.1
   bound) → `ExecutableTooLarge`, before any read.
5. **Header.** `pread` 64 bytes at offset 0. Fewer than 64 bytes or wrong magic →
   `NotElf`; this is where `#!` scripts stop (X2). Magic present but not
   `ELFCLASS64`/`ELFDATA2LSB`/`EM_X86_64`/`ET_EXEC|ET_DYN` → `ElfNotInCohort` (E8).
6. **Measurement.** `pread` loop with a fixed 64 KiB buffer from offset 0 until EOF or one byte
   beyond the step 2 size, whichever comes first, so growth is seen without an unbounded read.
   SHA-256 over every byte read. No shared file offset is used.
7. **Detected instability (owner amendment 1, T51).** `fstat` again and compare this second sample
   with the step 2 sample: `st_size`, `st_mtime` and `st_ctime`, with nanoseconds. If any differs,
   or the byte count read differs from the step 2 size, the measurement protocol has **detected
   instability** → `MeasurementInstabilityDetected`. This is `helm-observe`'s before/after
   sampling. It observes those fields and that count, and nothing else. A change that moves none of
   them between the two samples is not detected, for example one the filesystem's timestamp
   resolution does not distinguish, or a write through a shared writable mapping whose timestamp
   update is deferred. **Not detecting instability does not prove** that no concurrent mutation
   occurred, that the object is immutable, that a snapshot was taken, or that the measured bytes
   are the bytes later executed.
8. **Result.** `ExecutableCapability { fd, pre_exec_body_size, pre_exec_body_sha256,
   pre_exec_mode_bits (st_mode & 0o7777), elf_type }`.

**Not checked, deliberately.**

* **Execute permission.** It is recorded, not enforced. The kernel evaluates `MAY_EXEC` at
  `execveat`, and a denial arrives as `ExecFailed { stage: EXEC, errno: EACCES }` (X3, X1, X8).
* **Filesystem, mount or `noexec`.** No attestation is made. `noexec` surfaces as `EACCES` (X8).
* **File capabilities.** No `security.capability` xattr is read. `PR_SET_NO_NEW_PRIVS` covers
  them (D-11). The real transition is untested (T17).
* **`binfmt_misc`.** Not read. An entry matching in-cohort x86_64 ELF is a host precondition (S-14).
* **The descriptor's `CLOEXEC` flag.** Irrelevant, because the launch duplicates it
  `F_DUPFD_CLOEXEC` (T19).

**Execute-only objects are inadmissible.** An object the caller can execute but cannot read cannot
be opened `O_RDONLY`, and `O_PATH` is refused. Measurability, not executability, is the binding
constraint.

**Side effects of admission, stated.** Reading updates atime under `relatime` and fills the page
cache. `O_NOATIME` is not used, because it needs ownership or `CAP_FOWNER`.

**What the capability asserts, verbatim for the README.** *These bytes were read through this
descriptor before the execution attempt, the measurement protocol detected no instability while
reading them, and this descriptor is the exec target.* It never asserts that these bytes are the
bytes the kernel executed (E6, E6b), that the object was stable (T51), nor anything about the ELF
interpreter, shared libraries or loaded-code closure (E7).

### 6.3 Working-directory admission: `admit_working_directory(id, fd)`

1. `id` must match the identifier grammar, otherwise `WorkingDirectoryIdInvalid`.
2. `fcntl(F_GETFL)`: `O_PATH` → `DescriptorModeUnsuitable`. Only the evidenced `O_RDONLY`
   directory descriptor passes (S2 and S7 used one). `O_PATH` directories are a backlog item.
3. `fstat`: not `S_IFDIR` → `NotDirectory`.
4. No search permission check. The kernel decides at `fchdir`, and a denial is
   `ExecFailed { stage: CHDIR, errno: EACCES }` (S2, S7).

The directory is never enumerated and no path is known or recorded.

### 6.4 Composition: `authorize(plan, executable, working_directory)`

`authorize` performs no I/O. It checks exactly one relation and records the rest.

| Relation | Handling |
|---|---|
| `plan.working_directory_id() != working_directory.id()` | **refused** before process creation: `AuthorizationRefusal::WorkingDirectoryIdMismatch`. Nothing is returned, and the descriptors close |
| plan identity | bound: `AuthorizedLaunch` owns the plan, so its `sha256` is the receipt's `plan_sha256` |
| executable measurement | bound: moved into `AuthorizedLaunch` unchanged; not re-measured at launch |
| `asserted_context` digests | **recorded only**, as caller assertions, under that name in the receipt |
| execution kind vs capability kind | nothing to compare: one kind exists, and its capability types exist only on the supported platform |

**Why comparison and observation context cannot become permission.**

* `helm-launch` links neither `helm-bind` nor `helm-observe` (section 12). `Contradiction`,
  `Coverage`, `BindingReport`, `ObservationArtifact` and `RootCapability` are not nameable in the
  crate, so no code path can read them.
* The context digests are 32 opaque bytes, and no function compares them to anything.
* Their presence in a receipt means only that the caller asserted an association. It never means
  a binding was performed, found no contradiction, or gated the launch.
  **`NoClaimContradicted` is not permission to execute**, and the receipt cannot express that it
  was consulted.
* A trusted caller still decides to open the descriptor. If a future orchestrator reads a
  binding report before doing so, that is that orchestrator's accepted policy, not a
  `helm-launch` property.

## 7. Platform and unsafe backend boundary

### 7.1 What compiles where

| Target | Compiles | Exists | Tested |
|---|---|---|---|
| Linux x86_64 | whole crate | plan, model, receipt, errors, pure planners, capabilities, `authorize`, `launch`, backend | Levels 1–4 |
| Linux on another architecture | portable modules | plan, model, receipt, errors, pure planners | Level 1 only |
| Windows, macOS | portable modules | same | Level 1 on CI |

Gate: `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]` on the `authority`, `backend`
and `launch` modules, exactly as `helm-observe` gates `authority`, `linux` and `observe`. Off the
cohort a caller can parse a plan and read a receipt, and cannot obtain a capability or a launcher
at all. No backend portability is advertised. Linux aarch64 is not claimed merely because the
syscalls exist there: their numbers, `clone3` argument layout and the evidence all differ.

### 7.2 Crate layout (provisional)

```text
crates/helm-launch/
  Cargo.toml                 workspace lints NOT inherited; all restated (7.4)
  README.md                  non-claims first
  src/lib.rs                 #![deny(unsafe_code, unsafe_op_in_unsafe_fn)]; re-exports; cfg gates
  src/plan.rs                parse_launch_plan, strict scanner, bounds           portable, safe
  src/model.rs               receipt record, fact enums, vocabulary              portable, safe
  src/receipt.rs             deterministic serializer, size proof                portable, safe
  src/error.rs               four error families                                 portable, safe
  src/layout.rs              pure fd-layout planner (child fd numbers, gaps)     portable, safe
  src/lifecycle.rs           pure event-driven lifecycle state machine           portable, safe
  src/authority.rs           admit_executable, admit_working_directory, authorize  Linux x86_64, safe (rustix)
  src/launch.rs              orchestration: prepare, spawn, poll loop, receipt     Linux x86_64, safe (rustix)
  src/backend/mod.rs         the ONLY place unsafe_code is allowed                 Linux x86_64
  src/backend/spawn.rs       parent side: signal block, clone3, pidfd wrap         unsafe
  src/backend/child.rs       post-clone child path, #[no_implicit_prelude]         unsafe
```

`lifecycle.rs` takes events (status bytes, status EOF, stream bytes, stream EOF, read error,
pidfd readable, deadline) and emits actions (read, send `SIGTERM`, send `SIGKILL`, sweep, reap,
stop). It is a pure function over a state value, so ordering rules such as sweep-before-reap and
"no timeout after an observed exit" are unit-tested on every platform. `launch.rs` only connects
it to real descriptors.

### 7.3 Process-creation mechanism: choice

| Candidate | Pins the object | Pidfd without a race | Hidden userspace in child | Needs procfs | Verdict |
|---|---|---|---|---|---|
| **`clone3(CLONE_PIDFD)` + child setup + `execveat(fd, "", …, AT_EMPTY_PATH)`** | yes (E2–E4) | yes, same syscall (M3) | none: raw syscall, no atfork (M2) | no | **Recommended for 0.1** |
| `fork` + `pidfd_open` | yes | no: three caller preconditions; M5 `waitid_echild` under `SIGCHLD = SIG_IGN` | glibc atfork handlers | no | rejected (S-01) |
| `std::process::Command` | no: path-based `execvp`. Calling `execveat` from `pre_exec` hijacks std's unenumerated child path and its status pipe | only via unstable API | std's own child code before `pre_exec` | no | rejected |
| `posix_spawn` | no: takes a pathname | no | glibc's `clone(CLONE_VM \| CLONE_VFORK)` path | no | rejected |
| helper or trampoline binary | yes, if the helper is found | depends | a second program | discovery or memfd | rejected: reintroduces discovery and a second product language |
| `execve("/proc/self/fd/N")` | yes | — | — | **yes** | rejected (S-15) |

**Why `clone3(CLONE_PIDFD)`, and not convenience.** It is the only candidate that (a) executes the
admitted descriptor, (b) obtains the pidfd in the syscall that creates the child, so no PID is
ever used as authority for signalling or waiting on the direct child individually (the guarded
group sweep of T31 is the one pid-valued signal), and (c) runs no userspace code in the child that
this crate does not write. All three are the mechanism Trial #3 exercised: M3 traced direct form,
M2 multi-threaded parent, E1–E4 pinning.

**Exact use.** `clone_args { flags: CLONE_PIDFD, pidfd: &mut pidfd, exit_signal: SIGCHLD,
stack: 0, … }` through the `raw_syscall6` shim of section 7.6, never `libc::syscall`. There is no
`CLONE_VM`, `CLONE_VFORK`, `CLONE_FILES` or `CLONE_THREAD`, so the child gets a copy-on-write
address space and its own descriptor table, as in the spike.

**No fallback.** `ENOSYS` or `EPERM` from `clone3` (for example a seccomp profile that hides it)
returns `LaunchError::ProcessCreationUnavailable` with no child. Container environments that block
`clone3` are outside the cohort, and the README says so.

### 7.4 Lint policy

Cargo does not allow `[lints] workspace = true` together with local lint tables, and
`unsafe_code = "forbid"` cannot be relaxed by `#[allow]`. The crate therefore restates every
workspace lint and adds the unsafe-specific ones:

```toml
[lints.rust]
unsafe_code            = "deny"     # workspace: "forbid"; relaxed only in src/backend/
unsafe_op_in_unsafe_fn = "deny"

[lints.clippy]
unwrap_used = "deny"                # restated verbatim from [workspace.lints.clippy]
expect_used = "deny"
panic       = "deny"
undocumented_unsafe_blocks    = "deny"
multiple_unsafe_ops_per_block = "deny"
```

* `src/lib.rs` restates `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]` in source.
* `src/backend/mod.rs` is the single `#![allow(unsafe_code)]`. It also denies
  `clippy::indexing_slicing`, `clippy::arithmetic_side_effects`, `clippy::as_conversions` and
  `clippy::missing_safety_doc`.
* **Lint-drift test** (`tools/tests`): parse the workspace `Cargo.toml` and the crate's
  `Cargo.toml`. Fail if any workspace lint is missing or weaker in the crate, or if any other
  crate stops inheriting the workspace table.
* **Unsafe-confinement test**: fail if the token `unsafe` appears in any crate source file
  outside `src/backend/`.

### 7.5 Unsafe operations: allowed and forbidden

**Allowed, each in its own documented block:**

| Operation | Where | Why no safe wrapper suffices |
|---|---|---|
| raw `rt_sigprocmask(SIG_SETMASK, full kernel set)` and the restore, through the syscall shim | parent, around `clone3` | glibc cannot express the mask: `sigfillset` omits, and `pthread_sigmask` strips, its internal signals `SIGCANCEL` (`__SIGRTMIN`) and `SIGSETXID` (`__SIGRTMIN + 1`) (glibc 2.39 `signal/sigfillset.c`, `nptl/pthread_sigmask.c`); rustix 1.1.4 has only the `unsafe`, `doc(hidden)` `runtime::kernel_sigprocmask` |
| raw `clone3` through the syscall shim | parent | no safe wrapper exists |
| `OwnedFd::from_raw_fd(pidfd)` | parent, immediately after `clone3` returns | kernel-returned integer |
| calling the `-> !` child entry with a pointer to the prepared plan | child | crosses into the post-clone path |
| raw syscalls of section 8.4 | child only | rustix has no `close_range`; `execveat` is `unsafe` and `doc(hidden)`; std/rustix wrappers are not reviewed for the post-clone state |

**Forbidden in the backend:** any other FFI; `CLONE_VM`/`CLONE_VFORK`; `mmap`; `transmute`;
unchecked slice or string construction; `static mut`; raw pointer arithmetic other than reading
the prepared argv/envp arrays; and every allocation, lock, format, print, panic, destructor or
std/rustix call on the child path.

**Required `// SAFETY:` content per block:** which invariant the parent established, which
descriptor or pointer is used and why it is valid in the child's copy of the address space, why
the call cannot allocate, lock or unwind, and the only exits: `execveat` success or
`exit_group(127)`.

### 7.6 Post-clone Rust constraints

* **Everything is prepared before `clone3`.** The parent builds each `CString`, the NULL-terminated
  argv pointer array, the one-element `envp = [NULL]`, the 8-byte exec-status record buffer, the
  signal set, and all descriptor numbers and `close_range` gaps. They live in a `PreparedLaunch`
  owned by the parent frame that calls `clone3`.
* **One POD for the child.** `#[repr(C)] #[derive(Clone, Copy)] struct ChildPlan` holding
  integers and raw pointers only, with
  `const _: () = assert!(!core::mem::needs_drop::<ChildPlan>());`.
* **One child entry.** `unsafe fn child_main(plan: *const ChildPlan) -> !`. It is called on the
  `clone3 == 0` branch directly, before any other statement, so no destructor of the parent frame
  can run in the child and no unwinding can begin.
* **No implicit prelude.** `child.rs` uses `#[no_implicit_prelude]`, and a source-token
  allowlist test permits only `::core` paths, the syscall shim and `ChildPlan`. The test fails on
  `Vec`, `String`, `Box`, `format`, `print`, `panic`, `std`, `alloc`, `rustix`, `unwrap`, `expect`,
  `[`-indexing or arithmetic operators.
* **No glibc code after the clone.** `clone3` and every child syscall go through one
  `raw_syscall6` shim written with `core::arch::asm!` for the x86_64 syscall ABI. The kernel returns
  `-errno` in `rax`, and the child reads that value directly. `libc::syscall` is not used on this
  path, because its stub writes glibc's thread-local `errno` and would put glibc code in the child
  window. The `libc` crate supplies constants only. The parent's two `rt_sigprocmask` calls use the
  same shim, because glibc's wrappers cannot block every blockable signal (section 7.5).
* **A fixed syscall sequence.** The stage list of section 8.4 is a `const` array. The child walks
  it in order and has no configuration branch other than the gap arithmetic. Test-only fault
  injection is compiled only under a non-default feature, and a release-build test proves the
  injection symbols are absent (section 14.3).

## 8. Process lifecycle algorithm

### 8.1 Fixed constants (never plan fields)

| Constant | Value | Source |
|---|---|---|
| `SPAWN_CONFIRM_TIMEOUT_MS` | 5 000 | frozen; S6 |
| `POST_EXIT_DRAIN_MS` | 2 000 | frozen; O6, O7, P4, T5 |
| `POST_KILL_REAP_MS` | 5 000 | **new** (T40); bounds the wait after every `SIGKILL`; owner-approved initial 0.1 bound |
| `MAX_CAPTURE_BYTES` per stream | 65 536 | frozen; O4 |
| `MAX_PLAN_BYTES` | 32 768 | architecture §33 |
| `MAX_ARGS` / `MAX_ARG_BYTES` / `MAX_ARGV_TOTAL_BYTES` | 64 / 4 096 / 131 072 | architecture §33; A4 at 4 096 |
| `timeout_ms` range / `grace_ms` range | 1–600 000 / 0–60 000 | architecture §23 |
| `MAX_EXECUTABLE_BYTES` | 536 870 912 (512 MiB) | **new**; mirrors `helm_observe::MAX_FILE_BYTES`; owner-approved initial 0.1 product bound |
| `MAX_RECEIPT_BYTES` | 8 192 | architecture §33; proven by a widest-receipt test |
| `READ_BUFFER_BYTES` | 65 536 | spike and `helm-observe` |

### 8.2 Preparation in the parent, before any child exists

Everything that can fail here returns `Err(LaunchError::PreparationFailed { step, errno })`. No
child exists and no receipt exists, and every descriptor created so far closes by `OwnedFd` drop.

1. Build the argv `CString`s and pointer array, and `envp = [NULL]`. Plan validation makes a NUL
   unreachable. If `CString::new` fails anyway, return `LaunchError::InternalInvariant`.
2. `pipe_with(CLOEXEC)` four times: stdin, stdout, stderr, exec-status.
3. `fcntl_dupfd_cloexec(fd, 3)` for each of the six descriptors the child needs: exec, working
   directory, stdin read end, stdout write end, stderr write end, status write end. Close the
   originals. All six are now distinct, `≥ 3` and `CLOEXEC` (T19).
4. Compute the `close_range` gaps preserving exactly the exec descriptor and the status write end,
   skipping inverted ranges. This is `layout.rs`, a pure function (section 9.1).
5. Set `O_NONBLOCK` on the three parent read ends.
6. Fill `ChildPlan`.

### 8.3 Spawn

1. **Block every blockable signal on the calling thread (T21, owner-approved).** One raw
   `rt_sigprocmask(SIG_SETMASK, full set, &saved)` through the shim saves the current mask and
   blocks every blockable signal. glibc's `sigfillset` and `pthread_sigmask` cannot do this,
   because they exclude glibc's two internal real-time signals from the mask (section 7.5). The kernel leaves `SIGKILL` and
   `SIGSTOP` unblocked: they are not blockable. Only this thread's mask changes, and only until
   step 3. Other host threads keep their masks and may still receive process-directed signals, so
   no process-wide signal control is claimed. A glibc `set*id` call in another host thread, or a
   cancellation of this thread, waits until the mask is restored.
2. `clone3(CLONE_PIDFD, exit_signal = SIGCHLD)` through the shim. The child inherits the blocked
   mask.
   * `== 0`: call `child_main(&plan)`, which never returns (8.4).
   * `< 0`: restore the saved mask. `ENOSYS`/`EPERM` → `ProcessCreationUnavailable`; anything else
     → `ProcessCreationFailed`. No child, no receipt.
   * `> 0`: wrap the pidfd in `OwnedFd` at once. From here on **`launch` always returns `Ok` with a
     receipt** (section 13).
3. **Establish group-sweep authority, or record that it was not established (T31, owner-approved
   guard).** `setpgid(child, child)` is the first system call after `clone3` returns. **A return
   of 0 is the only event that establishes group-sweep authority**: the launcher itself has then
   placed the direct child in a dedicated process group whose id is the child's pid. `EACCES` (the
   child has already exec'd), `ESRCH`, `EPERM` or any other error establishes nothing. It is not
   retried and not interpreted further, and no group id is ever inferred from the child's own
   `setpgid(0, 0)` stage, from a later pre-exec failure record or from the pid value alone. The child's
   stage still puts the executed image in a dedicated group whichever call runs first, but only
   the launcher's own successful call grants sweep authority. Then restore the saved mask.
4. Close the parent's copies of the six child-side descriptors, then the stdin write end. The
   child now reads immediate EOF on 0 (T22). Every pipe can now reach EOF, which fixes the
   pre-execution-review defect in which the parent kept write ends open.

### 8.4 The post-clone child contract

The sequence is closed. Every step maps to exactly one stage, and each stage issues only the
listed syscalls. On any failure the child writes one record and exits; there is no other branch.

| # | Stage | Syscalls (through the shim) | Failure record |
|---|---|---|---|
| 1 | `DUP2` | `dup2(stdin_r, 0)`, `dup2(stdout_w, 1)`, `dup2(stderr_w, 2)` | `DUP2` + errno |
| 2 | `CLEAR_CLOEXEC` | `fcntl(0, F_SETFD, 0)`, `fcntl(1, …)`, `fcntl(2, …)` — a backstop: `dup2(fd, fd)` would be a no-op that keeps the flag, but the unconditional relocation (T19) makes that case unreachable, and a `dup2` to a different number already clears the flag | `CLEAR_CLOEXEC` + errno |
| 3 | `CHDIR` | `fchdir(dir)` — **before** `close_range`, because the directory descriptor is not preserved | `CHDIR` + errno |
| 4 | `CLOSE_RANGE` | up to three `close_range(first, last, 0)` over the gaps around the exec descriptor and status write end, skipping `first > last` | `CLOSE_RANGE` + errno |
| 5 | `SETPGID` | `setpgid(0, 0)` | `SETPGID` + errno |
| 6 | `SIGACTION` | `rt_sigaction(sig, SIG_DFL, NULL, 8)` for every signal 1–64 except `SIGKILL` and `SIGSTOP`, while delivery is still blocked | `SIGACTION` + errno |
| 7 | `SIGMASK` | `rt_sigprocmask(SIG_SETMASK, empty, NULL, 8)`, the intended final mask for the executed image — only now can blockable signals arrive, and every disposition is already default | `SIGMASK` + errno |
| 8 | `NO_NEW_PRIVS` | `prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)` (D-11) | `NO_NEW_PRIVS` + errno |
| 9 | `EXEC` | `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)` | `EXEC` + errno |
| — | failure exit | `write(status_w, record, 8)` retried on `EINTR` only, then `exit_group(127)` | — |

* **Record format.** Exactly 8 bytes: `[stage: u8, 0, 0, 0, errno: i32 little-endian]`, written
  from a buffer prepared in `ChildPlan`. It is below `PIPE_BUF`, so the write is atomic.
* **Permitted child syscall set:** `dup2`, `fcntl`, `fchdir`, `close_range`, `setpgid`,
  `rt_sigaction`, `rt_sigprocmask`, `prctl`, `execveat`, `write`, `exit_group`. Any other syscall
  between the `clone3` return and `execveat` is a test failure. Examples: `brk`, `mmap`, `munmap`,
  `mprotect`, `futex`, `openat`, `set_robust_list`, `getrandom`, `rt_sigreturn`, `clone`, `clone3`,
  `getpid`, `kill`.
* **Deviation from the frozen M4 order, owner-approved (Q2, 2026-09-16).** The frozen order was
  `… SETPGID, SIGMASK, SIGACTION, NO_NEW_PRIVS, EXEC`, and the frozen vocabulary had a child-side
  `RELOCATE`. The product blocks every blockable signal across `clone3` (8.3 step 1), so stages 6
  and 7 swap: no host handler can run in the child between the clone and the reset. `SIGKILL` and
  `SIGSTOP` still act on the child with their default actions, because they can be neither blocked
  nor caught. Relocation is parent-only (8.2 step 3) and never produces a child record. The executed
  image's end state is identical to the evidenced one.
* **Independent testing.** Section 14.2: traced syscall window, stage-order match, source-token
  allowlist, `needs_drop` assertion, release-build injection absence. Section 14.3: every stage's
  failure provoked through real kernel state where one exists (`CHDIR` by mode `0000`; `EXEC` by
  `EACCES`/`ENOEXEC`/`ETXTBSY`), otherwise through the test-only injection feature.

### 8.5 Observation loop

One thread, one `poll` over at most four descriptors: status read end, stdout read end, stderr
read end, pidfd. The timeout comes from a `CLOCK_MONOTONIC` deadline. The rules are validated
spike behaviour except where marked.

**Common rules.**

* **Read before hangup.** A descriptor is read only when `poll` reports it, and `POLLHUP` is never
  interpreted before `read() == 0` (O8, S7).
* **EOF removal.** A descriptor leaves the poll set only at `read() == 0` or a read error. It never
  spins (O5).
* **`EINTR`** is retried without consuming a deadline.
* **Streams.** Every drained byte is counted and hashed. The first `capture_prefix_bytes` are kept
  in a buffer allocated once at that size, and excess bytes are discarded, never withheld (O4).
* **Read error, new (T41, owner-approved).** A read error on a stream sets `ReadFailed { errno }`
  and stops reading that stream. On the status channel it sets `Indeterminate(StatusReadFailed)`. A
  read error is never treated as EOF.
* **Bounded kill wait, new (T40, owner-approved).** Every `SIGKILL` the launcher sends by pidfd sets
  a kill deadline `now + POST_KILL_REAP_MS`. The loop keeps polling the pidfd and draining the
  streams until the pidfd is readable or that deadline passes. If it passes, the child end is
  `EndNotObserved`: no end was observed within the bound. That is not a claim that the child is
  still running at any later moment, nor that it never ended. **`EndNotObserved` is latched at that
  deadline as the receipt-facing child-end fact** (owner clarification, synced at P1 acceptance,
  2026-09-17): no later observation or Phase C event changes it — not a late pidfd readiness, a late
  `Exited`, `Signaled` or core-dumped result, nor an `ECHILD` showing that another actor already
  reaped the child. A later, independent cleanup fact such as `not_issued_child_already_reaped`
  may still be recorded.

**Phase A — exec status**, deadline `spawn + SPAWN_CONFIRM_TIMEOUT_MS`. Streams drain throughout.

| Observation | Exec status | Next |
|---|---|---|
| 8 record bytes, then EOF | `PreExecFailure { stage, errno }` | wait for the pidfd until the Phase A deadline; if it is still not readable, `SIGKILL` and the kill wait; then Phase C |
| 1–7 bytes, then EOF; or more than 8 bytes | `Indeterminate(StatusRecordMalformed)` | as for a record |
| EOF with no byte | `Indeterminate(StatusEofWithoutRecord)` | Phase B; run deadline `= now + timeout_ms` |
| deadline, neither | `Indeterminate(PreExecStatusTimeout)` | immediate `SIGKILL` by pidfd (S-11) and the kill wait, then Phase C |

A pidfd that becomes readable in Phase A is recorded, and the loop keeps reading the status channel
to EOF within the Phase A deadline.

**Phase B — running.** Poll the streams and the pidfd.

1. **Pidfd readable.** The child has ended: `ended_observed = true`, and the drain deadline is
   `now + POST_EXIT_DRAIN_MS`. Stop when both streams are at EOF or the drain deadline passes;
   streams still open become `WriterRetainedAfterChildExit` (O6).
2. **Run deadline passes, pidfd not readable.** Set `deadline_expired`,
   `pidfd_send_signal(SIGTERM)`, grace deadline `now + grace_ms` (T1, T3).
3. **Grace deadline passes, pidfd not readable.** `pidfd_send_signal(SIGKILL)`, kill deadline
   `now + POST_KILL_REAP_MS` (T2, T6).
4. **Kill deadline passes, pidfd not readable (new, T40).** Child end `EndNotObserved`, latched;
   streams still open become `ReadStoppedChildEndNotObserved`.

The classification is from what was observed. A pidfd readable before the run deadline is never
`deadline_expired`, even if a later signal reaches a zombie (T4, T5).

**Phase C — sweep and reap** (P3; T30; T31, owner-approved guard):

1. **No authority, no sweep.** If 8.3 step 3 did not establish group-sweep authority, no sweep is
   issued on any path (`not_issued_group_not_established`). No group id is inferred, and the
   remaining steps run unchanged.
2. **With authority,** run `waitid(P_PIDFD, WEXITED | WNOHANG | WNOWAIT)` immediately before the
   sweep.
   * `ECHILD` → the child was reaped elsewhere (R4). The sweep can no longer precede the reap, so
     it is **not issued** (`not_issued_child_already_reaped`); child end `EndUnobservable`, unless
     `EndNotObserved` is already latched, in which case the child end stays `EndNotObserved` and
     only the sweep disposition is recorded.
   * Otherwise → issue exactly **one** `kill(-child_pid, SIGKILL)` group sweep. The child is still
     unreaped: an ended child is a zombie the probe did not consume, and a child whose end was not
     observed still leads its group.
3. Reap with `waitid(P_PIDFD, WEXITED | WNOHANG)`. It cannot block: the zombie is present, or the
   child is `EndNotObserved`; then the reap is cleanup only, and if it collects nothing the child is
   left unreaped, stated in the receipt.
4. Classify: a latched `EndNotObserved` is kept whatever the reap collects. Otherwise
   `CLD_EXITED` → `Exited { code }`; `CLD_KILLED` → `Signaled { signal, core_dumped: false }`;
   `CLD_DUMPED` → `Signaled { signal, core_dumped: true }` (R3); anything else →
   `EndUnobservable`.
5. Close every remaining descriptor, serialise the receipt and return `Ok(LaunchOutcome)`.

`issued` records that the one `kill` call was made. It records nothing about which processes, if
any, received the signal, so neither a descendant-killed claim nor a containment claim follows.

Phase C runs on **every** path in which a child exists: normal exit, exec failure, indeterminate
status, pre-exec timeout, run timeout and an end that was not observed. With group-sweep authority
established, that makes the `SIGKILL` group sweep an every-path behaviour, **including after a
normal direct-child exit** (S-09, Q3). A same-group background process left by a program that
exited normally may therefore be terminated when `launch` completes. That is intentional 0.1
cleanup policy.

### 8.6 Total bound

`launch()` returns within `SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_KILL_REAP_MS +
POST_EXIT_DRAIN_MS` plus scheduling slack. That is 12 s beyond `timeout_ms + grace_ms`, and it is
a tested property (O6-shaped and T2-shaped tests assert it). The one unbounded wait the spike had,
a blocking `waitid` after `SIGKILL`, does not exist in the product: every `SIGKILL` sent to the
direct child by pidfd, on the pre-exec and the run path alike, is followed only by the bounded kill
wait of 8.5.

### 8.7 Caller preconditions (documented, not attested, not enforced)

* The host does not ignore `SIGCHLD`, does not set `SA_NOCLDWAIT`, and runs no thread that reaps
  children it did not create (for example `waitpid(-1, …)`). Violation makes the exit status
  unobservable (R4) and can suppress the sweep, and a foreign reaper acting between the Phase C
  probe and the sweep can defeat the group-authority guard (T31). It never causes a wrong process
  to be signalled through the pidfd.
* The host's `SIGCHLD` handler, if any, will see this child's exit (`exit_signal = SIGCHLD`).
* No seccomp or LSM policy denies `clone3`, `close_range`, `execveat` or `prctl`.
* No `binfmt_misc` registration matches in-cohort x86_64 ELF.
* Linux x86_64, kernel 5.9 or newer (documentation-derived; evidenced on `6.17.0-1022-azure`).

### 8.8 No process-tree containment

> **NO PROCESS-TREE CONTAINMENT CLAIM.**

The sweep, issued only under established group-sweep authority (T31), reaches only processes still
in the child's group at that instant. A descendant that called `setsid` or `setpgid`, daemonised,
or handed work to an existing service survives (P2, P4). A same-group descendant is in the sweep's
target group even after a normal direct-child exit (P1 recorded `descendant_died`), and that is
best-effort cleanup behaviour, not containment. Closing the read ends after the drain may deliver
`SIGPIPE`/`EPIPE` to a surviving writer. cgroup v2 delegation is the named future path and is out of
scope. **Wine remains blocked on a future multi-process
lifecycle model:** `wineserver` outlives its starter, so one direct child is not an application
session.

## 9. FD, cwd, argv and environment invariants

### 9.1 Descriptor inheritance

> **Invariant.** In the executed image exactly descriptors 0, 1 and 2 are open: stdin reads the
> launcher's pipe to immediate EOF, and stdout and stderr write to the launcher's two pipes. No
> other descriptor of the host survives, whether or not it is `CLOEXEC` and whether or not HELM
> opened it (F1–F4, F6, F7).

The claim rests on the child's `close_range`, **not** on HELM opening its own descriptors
`CLOEXEC`. A host may hold non-`CLOEXEC` descriptors, and another host thread may create one during
the launch; F2 is the evidence that they do not survive.

| Descriptor | Parent | Child before exec | Executed image |
|---|---|---|---|
| host descriptors, any flags | untouched | inherited, then closed by `CLOSE_RANGE` | absent |
| exec capability | `F_DUPFD_CLOEXEC ≥ 3`; the parent copy closes after `clone3` | kept by the gaps; used by `execveat` | absent (`CLOEXEC`; F4) |
| working directory | `F_DUPFD_CLOEXEC ≥ 3`; closes after `clone3` | used by `fchdir`, then closed by `CLOSE_RANGE` | absent |
| stdin pipe read end | `≥ 3`; closes after `clone3` | `dup2` → 0, flag cleared | 0 |
| stdin pipe write end | closes after `clone3` → EOF | closed by `CLOSE_RANGE` | absent |
| stdout / stderr write ends | `≥ 3`; close after `clone3` | `dup2` → 1 / 2, flag cleared | 1 / 2 |
| stdout / stderr read ends | kept, `O_NONBLOCK`, closed at return | closed by `CLOSE_RANGE` | absent |
| status write end | `≥ 3`; closes after `clone3` | kept by the gaps; written only on failure | absent (`CLOEXEC`) |
| status read end | kept, closed at return | closed by `CLOSE_RANGE` | absent |
| pidfd | `OwnedFd` (`CLOEXEC`), closed at return | returned to the parent only; any copy would be closed by `CLOSE_RANGE` | absent |

**Collisions.** A host with 0, 1 or 2 closed makes `pipe2` return low numbers (F6). The
unconditional relocation to `≥ 3` before the final stdio mapping (T19, owner-approved) means no
`dup2` source equals its target. The explicit flag clear of stage 2 still covers any case where one
would. Adjacent exec and status descriptors are covered
by the gap rule, which skips inverted ranges (F7).

**Cleanup order.** (1) child-side copies close in the parent right after `clone3`; (2) the stdin
write end closes next; (3) the read ends and pidfd close after the reap. On every early-return
path before `clone3`, `OwnedFd` drops close everything. On the child path nothing is dropped,
because `child_main` never returns.

**`layout.rs` is pure and property-tested.** Input: six descriptor numbers, all `≥ 3` and distinct.
Output: the `dup2` table and at most three `close_range` ranges. Properties: the ranges cover every
number `≥ 3` except the two preserved ones; no range is inverted; the preserved numbers are never
inside a range.

### 9.2 Working directory

* **Ownership.** The caller moves an `OwnedFd` into `admit_working_directory`. It then moves
  through `authorize` into `AuthorizedLaunch`, and `launch` duplicates it `≥ 3` and closes both the
  original and the duplicate in the parent after `clone3`.
* **Validation.** Directory, not `O_PATH`, valid id (section 6.3). Search permission is left to
  the kernel.
* **When `fchdir` happens.** In the child, stage 3, after stdio is bound and **before**
  `CLOSE_RANGE`. The stage order is a `const` array, and a traced test asserts
  `fchdir` precedes the first `close_range`. That makes the pre-execution-review defect, closing
  the directory descriptor before using it, a test failure rather than a latent bug.
* **Parent vs child lifetime.** The parent's copy outlives `clone3` and nothing else. The child's
  copy lives until `CLOSE_RANGE`. The executed image holds none, only its cwd.
* **Receipt.** `working_directory_id` only: no path, no inode, no device.
* **Errors.** Admission: `NotDirectory`, `DescriptorModeUnsuitable`, `WorkingDirectoryIdInvalid`.
  Composition: `WorkingDirectoryIdMismatch`. Launch: `ExecFailed { stage: CHDIR, errno }` in the
  receipt, because a child exists.
* **Relative paths in argv.** They resolve against this directory. That is the Trial #2 O6/O7
  lesson: the fixture's relative FIFO path pointed at the wrong place after `fchdir`.

### 9.3 argv

* **Validation at parse** (D-6): a JSON array of strings, so UTF-8 is guaranteed by the decoder.
  Each element has no NUL, including the `\u0000` escape, which decodes to a NUL and is refused.
  Element count is 1 to 64, each element at most 4 096 bytes, total at most 131 072 bytes.
  `argv[0]` is caller data with no default and no rewriting.
* **Materialisation before the child exists.** `Vec<CString>` plus
  `Vec<*const c_char>` with a trailing null, both owned by `PreparedLaunch` in the parent frame
  that calls `clone3`. The child reads the same addresses in its copy-on-write address space. No
  quoting, joining, splitting or expansion exists anywhere (A1, A2).
* **Receipt.** `argument_count` only. The values are fixed by `plan_sha256` and are not repeated
  in the artifact.
* **`E2BIG`.** The bounds keep argv far below `ARG_MAX`. If the kernel still refuses, the result is
  `ExecFailed { stage: EXEC, errno: E2BIG }`, never an exit status.

### 9.4 Environment

* **Exactly empty.** `envp` is a one-element array containing NULL, in every launch (V1, D-10).
* **Not inherited, not constructed.** No host environment is read. No `PATH`, `HOME`, `LD_*`,
  `GCONV_PATH`, locale variables (`LANG`, `LC_*`), `DISPLAY`, `WAYLAND_DISPLAY`,
  `XDG_RUNTIME_DIR` or `WINE*`. No name is special-cased and no field anticipates one.
* **Hardening, not provenance.** A dynamic object still resolves its interpreter and libraries by
  name from host state (E7). The receipt records `environment_mode: "empty"` so a future mode
  cannot silently change an old receipt's meaning.

## 10. Exec-confirmation and outcome model

### 10.1 The policy, unchanged

> **CLEAN EXEC-STATUS EOF ALONE IS NOT POSITIVE EXEC PROOF.**

A child killed after its last setup stage and before `execveat` closes the `CLOEXEC` status write
end by dying. The parent then sees exactly what a successful exec shows (S5). A normal exit status
does not change that. S4's frozen claim is `ExecStatusIndeterminate` beside an observed `Exited:7`,
and the S4 policy (Trial #2 decision 2) is not weakened here.

**There is no `ExecSucceeded`, `Launched`, `Started` or `Ran` value** in the crate, and no
function derives one.

### 10.2 Two independent axes plus termination facts

```text
exec_status        what the exec-status channel established
  pre_exec_failure { stage, errno }     explicit structured record: the target was never executed
  indeterminate { reason }              nothing more can be said about exec
     reason: status_eof_without_record   (every normal run, and S5)
           | status_record_malformed      (short or oversized record)
           | pre_exec_status_timeout      (S6)
           | status_read_failed { errno } (T41)

child_end          what the pidfd and waitid established
  exited { code }
  signaled { signal, core_dumped }
  end_unobservable                        reaped elsewhere (R4) or classification unavailable
  end_not_observed                        no end observed within POST_KILL_REAP_MS after SIGKILL (T40)

deadline           run_deadline_expired: bool   (only when the pidfd was not readable first)
termination        sigterm_sent: bool, sigkill_sent: bool,
                   group_sweep: issued
                              | not_issued_group_not_established      (T31)
                              | not_issued_child_already_reaped       (T31, R4)
```

Every requested outcome is expressible without an exec-success claim:

| Requested state | Representation |
|---|---|
| explicit structured pre-exec failure | `exec_status = pre_exec_failure { stage, errno }` (X-series, S2, S7) |
| exec-status indeterminate | `exec_status = indeterminate { reason }` (S4, S5) |
| pre-exec timeout | `indeterminate { pre_exec_status_timeout }`, `sigkill_sent`, `child_end` observed (S6) |
| direct child exited | `child_end = exited { code }` (R1, R2, S3) |
| direct child signaled | `child_end = signaled { signal, core_dumped }` (R3) |
| exit status unobservable | `child_end = end_unobservable` (R4) |
| timed out | `run_deadline_expired = true` plus whatever `child_end` was observed (T1–T3) |
| killed during launcher termination | `sigkill_sent = true` and `child_end = signaled { SIGKILL }`. No causal claim: `waitid` reports a number, not a sender (S-13) |
| no end observed after `SIGKILL` | `child_end = end_not_observed` (T40); no claim that the child is still running |
| group sweep | `group_sweep = issued` records that the one call was made; `not_issued_*` records why not. Neither says which processes received a signal (T30, T31) |

The two axes are orthogonal. `pre_exec_failure` beside `child_end = exited { 127 }` is two true
facts: the child's own failure path exits 127, and the record says why. A reader that wants
"did the program exit 127 by itself" gets **no** such inference from the crate.

### 10.3 Can anything establish positive exec evidence in 0.1?

No launcher-side event does, so 0.1 preserves indeterminacy.

| Candidate | Why it is not 0.1 positive evidence |
|---|---|
| clean EOF plus exit code other than 127 | an inference from EOF; an external signal or a failed record write defeats it; contradicts S4 |
| `/proc/<pid>/exe` identity compared with the capability | ambient procfs authority; racy; gone once the child is reaped |
| `ptrace` `PTRACE_EVENT_EXEC` | the tracer receives wait notifications first, corrupting the lifecycle classification (definition §4) |
| a report written by the executed program | the X2c lesson: evidence from the executed object is a test fixture's property, never the launcher's. Its producer capability must be proven, and interpreting it is caller policy |
| pidfd information interfaces | report exit information, not exec |

A future positive-evidence mechanism needs its own owner decision and evidence (backlog B-04). A
caller that needs to know "the program ran" must obtain that from the program's own observable
effects, under its own policy, outside `helm-launch`.

### 10.4 Vocabulary rules

* Every emitted token is lowercase snake case from closed `#[non_exhaustive]` enums.
* Forbidden anywhere the crate emits: `pass`, `fail`, `ok`, `success`, `succeeded`, `ready`,
  `compatible`, `verified`, `worked`, `launched`, `sandboxed`, `contained`, `safe`, `authentic`,
  `signed` (T52).
* No function maps an outcome to `bool`, `Result<(), _>` or an ordering.
* **Exit code 0 means only that the direct child exited with status 0.**

## 11. Output, privacy and receipt model

### 11.1 Two products of one observation

| | `LaunchReceipt` — public, durable | `LaunchOutcome` — private, in memory |
|---|---|---|
| Purpose | an identity-bearing artifact another layer may reference by digest; the digest identifies these bytes, not their origin (11.5) | what the caller needs right now |
| Stream content | `bytes_drained`, `drained_sha256` over exactly those bytes, `completeness` | the retained prefix bytes, up to `capture_prefix_bytes` (at most 64 KiB per stream), and whether the prefix was truncated |
| Timing | none (D-8) | `elapsed`, monotonic |
| Serialisable | exact bytes plus SHA-256 | **no** `Serialize`; `Debug` prints lengths only |
| Who persists it | whoever chooses to | **only the caller, by explicit decision**; `helm-launch` never writes, logs or publishes it |

**Stated bluntly.** Hashing output does not prevent the child from having read secrets: it ran
with the caller's credentials. The boundary is about not *republishing* output in a HELM artifact.
It is not a confidentiality control.

**Relation to `helm-evidence` discipline.** `helm-evidence` reports use fixed codes and carry no
host paths, timestamps or raw evidence strings. The receipt follows the same rule. `helm-evidence`
gains no permission role: it may later check a receipt's bytes and digest as a declared artifact,
and that confers neither launch authority nor authenticity (sections 11.5 and 12.5).

### 11.2 Receipt contents

Deterministic JSON with fixed field order and no map iteration, hand-serialised as
`ObservationArtifact` and `BindingReport` already are. `receipt_sha256 = SHA-256(exact bytes)`, and
the receipt never contains its own digest.

```json
{
  "schema": "helm-launch-receipt",
  "version": "0.1",
  "backend": "linux_x86_64_clone3_pidfd_execveat",
  "plan_sha256": "…",
  "asserted_context": { "subject_spec_sha256": null, "binding_report_sha256": null },
  "working_directory_id": "workdir",
  "executable": {
    "pre_exec_body_size": 123456,
    "pre_exec_body_sha256": "…",
    "pre_exec_mode_bits": 493,
    "elf_type": "et_dyn"
  },
  "argument_count": 3,
  "environment_mode": "empty",
  "exec_status": { "kind": "indeterminate", "reason": "status_eof_without_record" },
  "child_end": { "kind": "exited", "code": 0 },
  "run_deadline_expired": false,
  "termination": { "sigterm_sent": false, "sigkill_sent": false, "group_sweep": "issued" },
  "stdout": { "bytes_drained": 512, "drained_sha256": "…", "completeness": "complete_at_eof" },
  "stderr": { "bytes_drained": 0, "drained_sha256": "…", "completeness": "complete_at_eof" }
}
```

| Field group | Records | Deliberately not recorded |
|---|---|---|
| plan identity | `plan_sha256` | argv values, timeout values — fixed by the plan digest |
| context | caller-**asserted** digests, unverified | any claim that a binding or observation was consulted |
| executable | pre-exec measurement and ELF type | host path, inode, device, "executed body" |
| admission facts | implied by the receipt's existence: admission passed | refusals, which produce no receipt |
| direct-child creation | implied: a receipt exists only if `clone3` succeeded | numeric pid, pidfd number |
| exec/setup observation | `exec_status` | exec success |
| process disposition | `child_end`, `run_deadline_expired` | cause of death |
| termination | signals sent; sweep issued, or not issued with its reason | which processes the sweep reached |
| streams | count, digest, completeness | bytes, prefix length, truncation flag |
| backend | one closed mechanism identifier | kernel release, glibc, hostname |
| non-claims | carried by schema name and version, documented in the README; not repeated per receipt | — |

**Why no pid or descriptor number.** No consumer need is demonstrated. They are host-local, not
stable identity, and mild host information. The Trial #3 evidence path withheld them too.

### 11.3 Closed stream completeness

| Value | Meaning | Evidence |
|---|---|---|
| `complete_at_eof` | the pipe reached EOF and every byte was drained | O1–O5, O8 |
| `writer_retained_after_child_exit` | the direct child ended, another process still held a write end when `POST_EXIT_DRAIN_MS` expired | O6, O7, P4 |
| `read_stopped_child_end_not_observed` | reading stopped because no end of the direct child was observed within the kill bound (`end_not_observed`) | UNVALIDATED (T40), owner-approved |
| `read_failed` with `errno` | a read failed; bytes before it are counted | UNVALIDATED (T41), owner-approved |

**What `writer_retained_after_child_exit` claims, verbatim.** The direct child ended with the
recorded end. For this stream, `bytes_drained` bytes were read before reading stopped, and
`drained_sha256` is over exactly those bytes. At that moment at least one other process still held
a write end. It claims nothing about the total output, later output, whether any descendant is
still running, or containment.

### 11.4 R3-M1 applied

* The receipt digest is over the exact published bytes.
* No receipt field is withheld, redacted or sanitised after serialisation. Nothing private is ever
  put into the record, so there is no "pre-sanitisation" form to diverge from.
* The receipt carries no aggregate digest over in-memory state. `drained_sha256` is over the
  drained bytes, which the receipt does not publish. It is a commitment the caller can check
  against the prefix only when the stream fit the capture bound, and the documentation says so
  instead of presenting it as independently recomputable.
* A test recomputes `sha256(exact_bytes)` for every receipt the suite produces.

### 11.5 No receipt-authenticity claim (owner amendment 2)

A durable `LaunchReceipt` is deterministic data. It **carries zero execution authority**, it may be
copied, it may be fabricated outside the crate, it is not cryptographically signed, and it is not
proof of provenance by itself. `helm-launch` makes **no receipt-authenticity claim**. Recomputing a
receipt's digest shows only that the bytes are the bytes the digest names, never which process
produced them. The safe API's refusal to construct authority-bearing values (5.3) does not extend
to receipt bytes. Provenance and bundle validation belong above `helm-launch`, and `helm-evidence`
does not become launch authority by checking a receipt (T52).

## 12. Existing-crate integration and dependency graph

### 12.1 Decision

> **`helm-launch` 0.1 has zero HELM crate dependencies.** It composes with the other modules
> through exact-byte identities at an orchestration layer that does not exist yet, never through
> Cargo types.

This is argued from the current public APIs, not only from D-3 or the experiment's own rule.

### 12.2 `helm-app-spec` — no dependency

* `EntryPointRequirement::path()` returns `RelativeEntryPoint`, documented in `src/model.rs` as an
  "inert prefix-relative Windows C-drive entry-point spelling", beginning `drive_c/`. It is not a
  Linux executable, and `helm-launch` must never treat it as one or as executable authority.
* The launch plan names **no** executable at all: the descriptor is the executable. So there is
  nothing to derive from a spec and nothing to cross-check inside the launcher.
* `ValidatedAppSpec` has no serializer. The only useful link is its `spec_sha256`, which a caller
  may place in `asserted_context.subject_spec_sha256`.
* **A future higher orchestration layer**, which does not exist, would turn application intent
  into substrate selection and into explicit Linux execution authority (an opened descriptor). For
  a Windows application that means a Wine adapter with its own ADR and experiment. No Wine adapter
  is designed or implemented here.

### 12.3 `helm-observe` — option C: independent, composed above both

Options considered: **A** depend on `helm-observe`; **B** accept selected observation facts as
inert metadata; **C** stay independent and compose above both. **C is chosen.**

* `RootCapability` is **read** authority. Linking the crate would put a capability type into the
  launcher's graph where it must never be confused with execution authority.
* `helm-observe` exposes no pinned descriptor: `observe` returns an `ObservationArtifact`, and no
  API hands out the object it read. An observation therefore **cannot** be the object a launch
  executes. The two pins are separate opens, and only digests can be compared.
* `helm_observe::Digest` and `helm_launch::Digest` are distinct types. Option B would still need
  the type or a copied hex string, and a copied fact is a caller assertion anyway, which
  `asserted_context` already expresses without implying observation.
* Observation authority never becomes execution authority. An orchestrator may compare an
  observed `regular_file_sha256` with a receipt's `pre_exec_body_sha256`, but equality means two
  reads saw the same bytes at two moments, not that the observed file ran.

### 12.4 `helm-bind` — no dependency

* `helm-bind` depends on `helm-app-spec` and `helm-observe`, so linking it would pull in both,
  including `RootCapability`.
* Everything a launch could use from a report is one digest.
* **`NoClaimContradicted` != permission to execute.** With no dependency, the crate contains no code
  path that can read `Contradiction` or `Coverage`: reading a verdict is unavailable, not merely
  discouraged. A report digest in `asserted_context` records a caller's association and nothing
  more. Any policy that consults a binding before a launch belongs to a future orchestrator under
  its own accepted decision.

### 12.5 `helm-evidence` — no dependency in either direction for 0.1

* `LaunchReceipt` is **not** a `helm-evidence` type. `helm-evidence` is a read-only verifier
  (`verify(&Path) -> Report`) over a closed, `deny_unknown_fields` bundle contract. Making it
  define or link an execution crate would put launch code in a read-only verifier.
* **What works today without code change.** A bundle can already list a receipt file as an
  `artifact` (`id`, `path`, `sha256`), and `helm-evidence` will check its presence and byte
  identity. It will not interpret the receipt. Byte identity is not authenticity: a fabricated
  receipt listed with its own digest passes the same check (11.5).
* **Later, not in 0.1.** Semantic receipt checks in `helm-evidence` would need a new evidence
  contract version and a receipt schema document, and ideally portable receipt test vectors rather
  than a Cargo dependency on `helm-launch`. That is backlog B-02.
* **R3-M1 carried forward.** Any digest a future evidence contract expects to recompute must be
  over bytes published in the bundle. A receipt digest qualifies, and a digest over in-memory
  records or withheld output does not.
* No cycle exists or can arise. `helm-launch` never references an evidence bundle, and a bundle may
  reference a receipt by digest.

### 12.6 Graph

**Rust crate dependencies** (proposed; unchanged crates shown as they are today):

```text
helm-app-spec ──(none)
helm-observe  ──(none)
helm-evidence ──(none)
helm-bind ──────▶ helm-app-spec
          └─────▶ helm-observe
helm-launch ────(no HELM crate)
   external: serde =1.0.228, serde_json =1.0.149, sha2 =0.10.9,
             rustix =1.1.4  (features std, fs, process, pipe, event — Linux only),
             libc  =0.2.189 (constants only — Linux only)
```

`rustix 1.1.4` and `libc 0.2.189` are already in `Cargo.lock`; `libc` is pulled today through
`rustix`, `errno` and `cpufeatures`. No new package version enters the lockfile. Feature
unification enables extra rustix features (`process`, `pipe`, `event`) for the workspace graph.
That is build composition only, recorded in the same spirit as the existing `sha2`/`force-soft`
note. `helm-launch` links no `helm-app-spec`, so it gets no `force-soft` unification of its own.

**Orchestration and data flow** (a future layer; nothing here is a Cargo edge):

```text
app-spec bytes ──parse──▶ ValidatedAppSpec ─┐
observation plan ─observe─▶ ObservationArtifact ─┼─bind─▶ BindingReport
binding plan ────────────────────────────────┘                 │ digest only, as a caller assertion
                                                                ▼
trusted caller opens exe + cwd descriptors ──▶ helm-launch ──▶ LaunchReceipt ──(artifact)──▶ helm-evidence bundle
launch plan bytes ─────────────────────────────▶
```

The arrows into `helm-launch` from the upper half carry **digests**, never types, and never
decide whether a launch occurs.

## 13. Error model

### 13.1 Boundary: `Err` versus a receipt

> **Before a direct child exists, every failure is `Err` and no receipt exists. Once `clone3` has
> returned a child, `launch` returns `Ok(LaunchOutcome)` with a receipt, whatever happened.**

That rule preserves "a refusal produces no receipt" and "an attempted launch always has a
receipt", and it removes the only ambiguous case: a failure after child creation can never hide
behind an error value while a live or unreaped child exists.

### 13.2 Families

| Family | Type | Returned by | Examples | Receipt |
|---|---|---|---|---|
| API misuse / invalid plan | `LaunchPlanErrors` (ordered, capped like `helm-observe`'s `PlanErrors`) | `parse_launch_plan` | `InputTooLarge`, `MalformedJson`, `DuplicateKey`, `NestingTooDeep`, `UnknownField`, `MissingField`, `TypeMismatch`, `UnknownSchema`, `UnknownVersion`, `ExecutionKindUnsupported`, `ArgvEmpty`, `ArgvTooMany`, `ArgTooLong`, `ArgvTooLarge`, `ArgContainsNul`, `EnvironmentModeUnsupported`, `StdinModeUnsupported`, `CaptureBoundOutOfRange`, `TimeoutOutOfRange`, `GraceOutOfRange`, `TerminationSignalUnsupported`, `IdGrammar`, `DigestGrammar` | none |
| capability admission refusal | `AdmissionError { code, errno: Option<Errno> }` | `admit_executable`, `admit_working_directory` | `DescriptorModeUnsuitable`, `NotRegularFile`, `NotDirectory`, `SetIdBitsPresent`, `ExecutableTooLarge`, `NotElf`, `ElfNotInCohort`, `MeasurementInstabilityDetected`, `MetadataUnavailable`, `ReadFailed`, `WorkingDirectoryIdInvalid` | none |
| unsupported platform or cohort | *not a runtime error* | — | off Linux x86_64 the capability and launch APIs do not exist; a kernel without `clone3` is `ProcessCreationUnavailable` below | none |
| authorisation refusal | `AuthorizationRefusal` | `authorize` | `WorkingDirectoryIdMismatch` | none |
| process-creation failure | `LaunchError` | `launch`, before a child exists | `PreparationFailed { step, errno }` (pipe, dup, sigmask), `ProcessCreationFailed { errno }` (`EAGAIN`, `ENOMEM`), `ProcessCreationUnavailable { errno }` (`ENOSYS`, `EPERM`) | none |
| internal invariant violation, before a child | `LaunchError::InternalInvariant { code }` | `launch` | a prepared value that plan validation made impossible, such as an interior NUL | none |
| child setup failure | receipt `exec_status = pre_exec_failure { stage ≠ EXEC, errno }` | `launch` → `Ok` | `CHDIR:EACCES`, `CLOSE_RANGE:EINVAL` | **yes** |
| exec failure | receipt `pre_exec_failure { stage: EXEC, errno }` | `launch` → `Ok` | `EACCES`, `ENOEXEC`, `ETXTBSY`, `E2BIG`, `ENOMEM` | **yes** |
| observation indeterminacy | receipt `exec_status = indeterminate { reason }`; `child_end = end_unobservable`; stream `read_failed` | `launch` → `Ok` | S5, R4, a poll or read error after the child exists | **yes** |
| timeout | receipt `run_deadline_expired`, signals sent | `launch` → `Ok` | T1–T3 | **yes** |
| no end observed after `SIGKILL` | receipt `child_end = end_not_observed` | `launch` → `Ok` | T40 | **yes** |
| internal invariant violation, after a child | receipt fact, never `Err`: the loop stops observing, then kills with the bounded kill wait, sweeps only under established group authority, reaps without blocking, and records the indeterminate state | `launch` → `Ok` | an impossible poll result | **yes** |

### 13.3 Rules shared with the existing crates

* Fixed, bounded, machine-stable codes. Errno values are carried as numbers with a symbolic name
  where known, never as OS message strings.
* No host path, descriptor number, pid or captured byte appears in any error or `Display` output.
* No `anyhow`, no `Box<dyn Error>` and no string errors. Every type implements
  `std::error::Error` and `Display`.
* No panic on any input. `unwrap`, `expect` and `panic` are denied by lint, and out-of-memory
  abort is outside the claim, as in `helm-app-spec`.

## 14. Product test strategy

**The LAUNCH-EXEC-01 harness is not ported.** No Python driver, journal, checker, sanitiser,
frozen case table or D-7 gate enters the product. Ordinary product tests need no formal trial and
no D-7. They run with `cargo test` locally and in CI, and a red run is a defect to fix, never a
trial result.

### 14.1 Level 1 — pure, portable (Linux, Windows, macOS)

| Area | Tests |
|---|---|
| plan parsing | every field rule and bound at its edge; duplicate decoded keys at every depth, including escaped spellings; nesting; unknown fields; `\u0000` in argv; exact-byte identity changes with whitespace |
| error vocabulary | codes stable and ordered; no host strings; error cap |
| receipt serialisation | a fixed record produces a fixed byte string and digest on all three platforms, like `helm-bind`'s determinism anchor; widest possible receipt stays under `MAX_RECEIPT_BYTES`; injectivity over the fact enums; no timestamp or duration field |
| authority from data | `compile_fail` doctests: construct `ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch` or `LaunchReceipt` from fields; clone an `AuthorizedLaunch`; call `launch` twice; pass a plan to `launch` |
| verdict vocabulary | every `as_str` of every enum and every schema key against the forbidden list of section 10.4 |
| no success mapping | source scan: no `fn … -> bool` on outcome types, no `is_success`/`ok`-style helpers |
| fd layout (`layout.rs`) | property test over random distinct descriptor sets `≥ 3`: ranges cover everything except the two preserved numbers, never inverted, never containing a preserved number; the F7 adjacent case and a 0/1/2-closed host case as fixed vectors |
| lifecycle state machine (`lifecycle.rs`) | event scripts for S5 (status EOF, then `SIGKILL` end → `indeterminate`, never success); T4/T5 (pidfd readable before deadline → no `run_deadline_expired`); T1–T3; S6; O6 drain expiry; read error → `read_failed` (T41); no pidfd readiness within the kill wait after any `SIGKILL`, pre-exec or run path → `end_not_observed` (T40); sweep action always precedes reap action; no group authority → no sweep action on any path; with authority, exactly one sweep action on every path, including a normal exit (T30, T31); `ECHILD` on the `WNOWAIT` probe → sweep not issued (T31); POLLIN+POLLHUP in one event → data read before EOF |
| deterministic serialisation | the same record serialises byte-identically across runs and platforms |

### 14.2 Level 2 — Linux backend structure (Linux x86_64)

| Property | Test |
|---|---|
| unsafe confined | source scan: the token `unsafe` appears only under `src/backend/` |
| lint policy intact | lint-drift test (section 7.4); `cargo clippy -p helm-launch -- -D warnings` |
| child state is POD | `const` `needs_drop` assertion compiles; `ChildPlan` is `Copy` |
| closed child vocabulary | source-token allowlist on `child.rs`; the stage array equals section 8.4 |
| no allocation, lock, format or panic in the child | a test launcher binary runs under `strace -f`, and the child window from the `clone3` return to `execveat` contains only the permitted set of 8.4; run once from a single-threaded parent and once from a parent with extra threads, one allocating and one with a registered `pthread_atfork` handler (M1, M2) |
| stage order | the same trace: `dup2 ×3`, `fcntl ×3`, `fchdir`, `close_range ≤3`, `setpgid`, `rt_sigaction …`, `rt_sigprocmask`, `prctl`, `execveat`, with `fchdir` before the first `close_range` (M4 as refined) |
| pidfd acquisition | the trace shows one process `clone3` with `CLONE_PIDFD`, a rendered pidfd, no `pidfd_open`, and `waitid(P_PIDFD)` on that descriptor. The process clone is selected by excluding `CLONE_THREAD` records, because the cargo test harness is multi-threaded (M3; Trial #2 M2 regression) |
| injection absent in release | build without the `test-fault-injection` feature; assert the injection code paths are not compiled (a `cfg` compile test plus a symbol check on the release artifact) |
| signal blocking across clone3 | trace shows the parent's raw `rt_sigprocmask(SIG_SETMASK, …)` with the full set, including glibc's internal real-time signals, immediately before `clone3`; the parent `setpgid` as the first call after it; then the restore (T21, T31) |

A tracer-based test declares `strace` as a requirement and fails with an explicit message if it is
absent, never silently skipping. The GitHub `ubuntu-24.04` image has `strace`, observed in
definition §4.

### 14.3 Level 3 — Linux integration with purpose-built fixtures

**Fixtures.** Small Rust fixture binaries built by the test profile (dynamically linked, which also
exercises E7), plus generated byte fixtures. Tests open them by path in the **test** code, which
plays the trusted caller. Every generated file is written with an explicit mode, and the mode is
asserted after writing (the Trial #2 umask lesson).

**Report fixtures follow the X2c rule (T50).** A fixture that reports its own argv, environ,
descriptors, signal masks, `NoNewPrivs` or cwd identity writes a versioned structured report to
stdout. Before any launcher test consumes that report:

* a **producer self-test** runs the same fixture binary directly through `std::process::Command`
  (not through `helm-launch`) and asserts that the report parses;
* the consumer asserts the report's marker names the fixture that the test admitted;
* a test declaring a report from a non-reporting object is a test-suite build error, enforced by a
  typed fixture registry in which report-capable fixtures carry a marker type.

| Behaviour | Test | Evidence it mirrors |
|---|---|---|
| exact ELF execution | admitted body's marker observed; rename over, unlink and retarget after admission change nothing | E1–E4 |
| measurement semantics | length-preserving mutation after admission runs the mutated marker; receipt keeps the pre-mutation digest; `fchmod 0` after admission → `pre_exec_failure { EXEC, EACCES }` with the old mode bits | E6, E6d, X1 |
| exec failure errno | never-executable file `EACCES`; X4-style header `ENOEXEC`; test-held writer `ETXTBSY`; writer closed before launch runs | X3, X4, E5, E5b |
| cwd capability | report cwd `(st_dev, st_ino)` equals the admitted directory; mode `0000` after admission → `CHDIR:EACCES`; 200-repetition variant | S2, S7 |
| empty environment | canary, `PATH`, `HOME`, `LD_LIBRARY_PATH` set in the test process; report environ empty | V1 |
| argv | spaces, metacharacters, newline, empty element, 4 096-byte element, arbitrary `argv[0]` | A1–A4, A6 |
| no_new_privs | report `NoNewPrivs: 1`; the control precondition asserts the test process has 0, otherwise the test fails with a named host precondition | N1, N2 |
| fd isolation | non-`CLOEXEC` and `CLOEXEC` unrelated descriptors open in the launching process; caller-supplied non-`CLOEXEC` exec descriptor; report shows exactly `{0,1,2}` | F1–F4, T19 |
| closed 0/1/2 | a dedicated test subprocess closes its own 0–2 and launches (cargo's harness stdio must stay intact) | F6 |
| adjacent descriptors | construct adjacency, then launch; no `CLOSE_RANGE:EINVAL` | F7 |
| signals | test process blocks `SIGTERM`/`SIGUSR1` and ignores `SIGPIPE`/`SIGUSR2`; report masks clean; T6-shaped timeout matches the default-parent shape | F5, T6 |
| stdout/stderr | 4 KiB each; 8 MiB concurrently under a 10 s bound; over the capture bound; early stdout close with CPU and poll-return bounds; POLLIN+POLLHUP, 200 repetitions | O1–O5, O8 |
| prefix privacy | the child prints a canary; `LaunchOutcome` prefix holds it; receipt bytes and `Debug` output do not | T24 |
| timeout | sleeper; `SIGTERM`-ignoring child; exit during grace; exit just before the deadline | T1–T5 |
| signal termination | `SIGSEGV` with `RLIMIT_CORE = 0` and with a core allowed where possible; `SIGABRT`; exit 127 after a successful start stays `exited` | R3, S3 |
| pidfd lifecycle | test process sets `SIGCHLD = SIG_IGN` → `end_unobservable`, sweep not issued, `launch` returns | R4, T31 |
| descendant-held stdio | descendant keeps 1 and 2 for 30 s → `writer_retained_after_child_exit`, return within the total bound | O6, O7, P4 |
| sweep and descendants | same-group background process of a normally exiting child ends; `setsid` descendant survives; recorded, not failed. Test-only parent delay past exec → no group authority, sweep not issued | P1, P2, T30, T31 |
| measurement instability | test-only hook between the read loop and the second sample changes the size, or rewrites content with a timestamp change → `MeasurementInstabilityDetected`, no capability; no test asserts that every concurrent change is detected | T51 |
| pre-exec timeout and death before exec | `test-fault-injection` stall → `pre_exec_status_timeout`, no zombie; injection kill before exec → `indeterminate(status_eof_without_record)` | S6, S5 |
| set-ID refusal | `u+s` and `g+s` fixtures owned by the test user → `SetIdBitsPresent`, no child | X7 |
| script refusal | `#!` fixture with mode `0755` → `NotElf`, no child | X2 |
| descriptor modes | `O_PATH` and writable executable descriptors refused; `O_PATH` directory refused | X6, T08 |
| total bound | every integration test asserts `launch` returned within section 8.6's bound | falsifier 21 |

### 14.4 Level 4 — adversarial regressions derived from Trial #1–#3 defects

| Regression | Origin | Product test |
|---|---|---|
| a test helper read back through a write-only descriptor | Trial #1 E5b setup abort | mutation helpers verify through a separate read-only descriptor; a lint-style test forbids reads on descriptors opened write-only in test support |
| setup failure mistaken for a mechanism result | Trial #1, definition §9.6 | the test support returns a typed `NotPosed` error, and a test that could not construct its state fails with that label, never as an assertion about the launcher |
| fixture permission and mode assumptions | Trial #2 X2b/X2c/X4 `0666 & ~umask` | every generated fixture's mode asserted after writing |
| fd collision and layout assumptions | F6/F7, spike `move_above_2` | `layout.rs` property tests plus F6/F7 integration tests |
| `CLD_DUMPED` lost its signal | Trial #2 R3 | `SIGSEGV` classified `signaled { SIGSEGV }` regardless of `core_dumped` |
| clean EOF read as exec success | S5, Trial #2 S4 | state-machine script and injection test: no path yields an exec-success value |
| `clone3` thread-vs-process confusion in traces | Trial #2 M2 | trace parser selects the `CLONE_PIDFD` process clone and rejects ambiguous traces instead of guessing |
| relative fixture paths broken by `fchdir` | Trial #2 O6/O7, E4 | fixture paths handed to children are canonical absolute paths; a test asserts that |
| report producer vs evidence consumer | Trial #3 X2c | producer self-tests and the typed fixture registry of 14.3 |
| digest not recomputable from published bytes | Trial #3 R3-M1 | every receipt: `sha256(exact_bytes) == sha256()`; any digest in a future published test artifact is recomputed from that artifact in CI |
| read error treated as EOF | spike drain loop | simulated `EIO` in the state machine → `read_failed` |
| blocking reap after `SIGKILL` | spike `waitid` | state machine: kill deadline without pidfd readiness → `end_not_observed`, no blocking call |
| sweep after a lost reap | R4 plus architecture §24 | `SIGCHLD = SIG_IGN` integration test: sweep not issued |
| sweep from an unestablished group | spike ignored its `setpgid` result and swept unconditionally | test-only parent delay past exec → sweep not issued, no `kill` in the trace (T31) |
| glibc wrapper leaves signals unblocked | glibc `sigfillset`/`pthread_sigmask` keep internal signals out of any mask | traced parent mask includes glibc's internal real-time signals (T21) |

### 14.5 CI placement (for the implementation slices, not this task)

* A `crates/helm-launch/**` path filter added to the existing Linux workspace workflow
  (`helm-evidence.yml`).
* A three-platform job like `helm-bind.yml` for Level 1.
* Neither workflow gains `workflow_dispatch`-only trial semantics, a D-7 gate, or artifact
  publication.

## 15. Experiment-to-product migration

The experiment is evidence and design input. **It is not the product source tree.** No Python
enters `helm-launch`, and no C file is transliterated into Rust and called productization. Every
experiment file stays frozen where it is.

| Asset | Classification | What carries over, and what does not |
|---|---|---|
| `launcher_spike.c` | **POTENTIAL PRODUCT ALGORITHM REFERENCE** | carries over as a reference only: stage order, gap arithmetic, parent-side closes, drain rules, sweep-before-reap, `CLD_DUMPED` handling. **Do not copy.** Its argument parsing, embedded SHA-256, base64 prefix emission, test flags (`--bypass-admission`, `--exec-fd-no-cloexec`, `--exec-fd-o-path`, `--skip-no-new-privs`, `--die-before-exec`, `--stall-pre-exec-ms`, `--post-fork-delay-ms`, `--extra-threads`, `--rejected-acquisition-arm`, `--post-pin-control-fd`, `--parent-*`), blocking `waitid`, read-error-as-EOF and its unconditional sweep without a group-authority check are not product behaviour |
| `driver.py` | **DO NOT COPY** | the case table, posing machinery, barrier protocol and static posability gate are trial machinery; the posability defect is the X2c lesson, carried over as T50 |
| `checker.py` | **DO NOT COPY** | PASS/FAIL/INVALID/BLOCKED and aggregate verdicts are exactly the vocabulary the product must not have |
| `observations.py` | **CONCEPT TO REIMPLEMENT** | carries over: exec confirmation needs positive evidence; read before hangup; the conservative S4/S5 rule. Reimplemented as the typed outcome model of section 10, not as token strings |
| `frozen_cases.py` | **TEST IDEA TO PORT** | carries over as test ideas: case constructions, permitted and forbidden child syscall sets, `OUT_OF_SCOPE` plan-parse obligations (NUL argv). Not membership, classes, gates or D-7 fields |
| `journal.py` | **TEMPORARY EXPERIMENT-ONLY** | fsynced trial journal and D-7 boundary; no product analogue |
| `evidence.py` | **CONCEPT TO REIMPLEMENT** | carries over: withhold raw output, never publish host paths or ids. Not carried: post-hoc sanitisation. The product never records private data, so it has nothing to sanitise, and R3-M1 cannot recur |
| `harness.py` | **TEMPORARY EXPERIMENT-ONLY** | preflight inventory, static-link gate, descriptor pre-loader |
| `oracles.py` | **TEST IDEA TO PORT** | independent expected digests computed without reading launcher output, for O-series stream tests |
| `run_launch_exec_01.py` | **TEMPORARY EXPERIMENT-ONLY** | the runner and its `--verify-freeze`; stays as history |
| `helper_report.c` | **TEST IDEA TO PORT** | the report contents (argv with lengths, environ, `F_GETFD` scan, `SigBlk/SigIgn/SigCgt`, `NoNewPrivs`, cwd identity), rewritten as a Rust fixture with a producer self-test |
| `helper_alt.c` | **TEST IDEA TO PORT** | substitution detector with a fixed-offset marker, as a Rust fixture |
| `helper_dynamic.c` | **TEST IDEA TO PORT** | a dynamic object; every Rust fixture already is one |
| `helper_fork.c` | **TEST IDEA TO PORT** | descendant shapes: release stdio, `setsid`, retain stdio, pre-armed signal |
| `helper_setid.c` | **TEST IDEA TO PORT** | the `u+s` admission fixture; the product needs only the mode bits, never a privileged transition |
| `make_fixtures.py` | **TEST IDEA TO PORT** | foreign ELF, X4-style unloadable header, magic-only file, script, with explicit modes |
| `SOURCE-HASHES.json`, `BUILD-EVIDENCE.md`, trial workflows | **TEMPORARY EXPERIMENT-ONLY** | freeze and trial authority; never product CI |

## 16. Implementation slices

Six slices. Each is one reviewable unit with its own commit series and a stop condition. A slice
that discovers it needs a policy choice not approved at P0 stops and returns to the owner; it does
not choose.

| Slice | Introduces | Adds | Explicitly absent | Required tests | Safety / privacy impact | Stop condition | ADR-0024 acceptance needed first |
|---|---|---|---|---|---|---|---|
| **P0 — product contract gate** | no code. Owner decisions on Q1–Q3 (recorded 2026-09-16) and ADR-0024 revised in place, still Proposed | the approved contract: this plan as amended by the owner | any crate, Cargo change or workflow change | `validate_docs.py` | none | owner acceptance decision on revised ADR-0024 recorded in `DECISIONS.md` | it **is** the ADR disposition step |
| **P1 — skeleton and portable model** | `crates/helm-launch` with `Cargo.toml` (restated lints), `lib.rs`, `plan.rs`, `model.rs`, `receipt.rs`, `error.rs`, `layout.rs`, `lifecycle.rs`, README; workspace member; CI path filters and a three-platform Level 1 job | parsing, receipt model and serializer, error families, pure fd-layout planner, pure lifecycle state machine | any Linux module, any `unsafe`, any descriptor, any process | Level 1 in full; lint-drift and unsafe-confinement tests (unsafe count is zero) | none: no I/O | Level 1 green on three platforms; independent slice review | **yes** |
| **P2 — capability admission** | `authority.rs` (safe rustix only), `authorize` | `admit_executable` measurement and refusals, `admit_working_directory`, composition | process creation, `unsafe`, `launch` | admission refusals (X2, X5, X6, X7, E8, T08, `NotElf`, `MeasurementInstabilityDetected` (T51), size bound); measurement vs an independent digest; `compile_fail` for capabilities | reads caller-supplied objects: atime and page cache, documented | refusals and measurement green on Linux; no `unsafe` yet | yes (inherited from P1) |
| **P3 — unsafe backend and the child contract** | `backend/mod.rs`, `spawn.rs`, `child.rs`, asm syscall shim; internal non-public `launch_minimal` for tests | preparation (8.2), spawn with signal blocking (8.3), the complete post-clone contract (8.4), exec-status channel, bounded non-blocking reap | public `launch`, timeouts, drain policy, receipt | Level 2 in full (traced window, stage order, pidfd acquisition, `needs_drop`, token allowlist, injection absence); Level 3 F-series, argv, env, `NoNewPrivs`, cwd, exec failure stages, S5/S6 injection | **the unsafe surface**: needs its own independent review focused on sections 7.4–8.4 | Level 2 green; independent unsafe review has no BLOCKER | yes |
| **P4 — lifecycle, termination and receipt** | `launch.rs`, public `launch`, `LaunchOutcome` | observation loop (8.5), deadlines, `SIGTERM`/grace/`SIGKILL`, bounded post-kill reap, drain, guarded sweep, classification, receipt emission, in-memory prefixes | anything outside section 8; any orchestration | Level 3 O, R, S, T, P series; total-bound assertions; prefix privacy canary | output bytes enter memory; receipt proven payload-free | Level 3 green; state-machine scripts agree with the real loop on shared scenarios | yes |
| **P5 — regressions, evidence contract, documentation** | Level 4 suite; receipt schema document; README non-claims; CI hardening | adversarial regressions of 14.4; receipt digest recomputation; published schema and test vectors | `helm-evidence` changes (backlog B-02); any Wine or orchestrator code | Level 4 in full; the whole suite on Linux; Level 1 on three platforms | none new | full suite green, then an **independent review of the whole crate** before any owner merge | yes |

**Authority, 2026-09-17.** P0 is complete: the owner's acceptance of revised ADR-0024 is recorded in
[`DECISIONS.md`](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised). **P1 is authorised**,
within the boundary stated in the [current authority note](#current-authority-2026-09-17). **P2, P3,
P4 and P5 are not authorised**, and each needs a new explicit owner decision.

**Authority, 2026-09-18.** P1 is **accepted** and **P2 is authorised**, within the boundary stated
in the [P2 authority note](#p2-authority-2026-09-18). **P3, P4 and P5 remain not authorised**, and
each needs a new explicit owner decision.

<a id="p2-acceptance-sync-2026-09-18"></a>

**P2 acceptance, 2026-09-18.** The owner
[accepted HELM-LAUNCH P2](../DECISIONS.md#helm-launch-p2-accepted) as the second product
implementation slice. **P1 is accepted, P2 is accepted, and P3, P4 and P5 remain not authorised.**
The P2 row's **stop condition is satisfied**: the admission refusals and the measurement are green
on a hosted Linux x86_64 runner, where `tests/linux_admission.rs` ran **28 tests** and the
`authority.rs` Linux-specific unit tests ran **13**, with the cohort doctest surface executed and
passed; the independent review recorded **0 BLOCKER and 0 IMPORTANT**; and portable compatibility is
green on `windows-2025` and `macos-15`, where the P2 authority API is proven **absent off the
cohort**. Accepted P2 still has **no process creation, no process execution, no `unsafe`, no host
privilege and no experiment execution**, and `AuthorizedLaunch` has no consumer that can create a
process. **This acceptance changes no P3 implementation detail and authorises no part of P3**: the
section 7.4 `unsafe` exception stays reserved and inactive. The next gate is an owner decision on
whether to authorise the **P3 unsafe backend and child contract**.

**After P5.** An independent product review, then an owner acceptance and merge decision, as for
`helm-observe` and `helm-bind`. Merging would be a product-module acceptance, not a verdict about
any application. No slice needs a formal trial or a D-7.

## 17. Open questions

### 17.1 OWNER_DECISION_REQUIRED_BEFORE_IMPLEMENTATION

**Q1–Q3 were decided by the [owner review of 2026-09-16](../DECISIONS.md#helm-launch-productization-plan-owner-review)
and are no longer open.** The questions as proposed are kept verbatim below the dispositions.

| # | Owner disposition, 2026-09-16 | Applied in |
|---|---|---|
| **Q1** | **APPROVED.** ADR-0024 must be revised before it can be considered for acceptance. The revision is prepared, and ADR-0024 **stays Proposed**, with no acceptance date and no approver | ADR-0024; section 16 (P0); 18.1 |
| **Q2** | **APPROVED WITH EXACT NARROWING.** Approved as 0.1 design obligations, to be validated by normal product tests and not by another formal D-7 trial: refuse writable executable capabilities (T08); relocate all child-side preserved descriptors to `≥ 3` before final mapping (T19); block all blockable signals across `clone3` and restore or reset child signal state in the closed child sequence (T21); guard the process-group sweep on positively established group authority (T31); bound the reap after `SIGKILL` and represent a child that still cannot be observed ended honestly (T40, `end_not_observed`); distinguish read failure from EOF (T41); record timeout and termination actions as facts without unsupported causal claims (S-13); `POST_KILL_REAP_MS = 5000` as the initial 0.1 bound; a maximum admitted executable size of 512 MiB as the initial 0.1 product bound. These are product design choices, **not** claims that LAUNCH-EXEC-01 directly validated them. Two items of the proposed set are not in that list: `ChangedDuringMeasurement` is replaced by amendment 1's detected-instability rule (T51), and `capture_prefix_bytes` as the only stream option stays a detail of revised ADR-0024, settled with that ADR under the rule that raw bounded capture is in memory only | section 3 authority columns; 6.2; 7.5; 7.6; 8.1; 8.3; 8.4; 8.5; 10.2; 11.3 |
| **Q3** | **APPROVED WITH A GROUP-AUTHORITY GUARD.** Exactly one `SIGKILL` process-group sweep is the fixed 0.1 cleanup policy on every return or completion path, including a normal direct-child exit, **after** the launcher has positively established the direct child's dedicated process group, and before the direct child is reaped. Without positive establishment the launcher infers or guesses no group id and issues no sweep. Accepted consequence: same-group descendants may be killed when `launch` completes, including after a normal exit, and a descendant that leaves the group may survive. Best-effort cleanup, never process-tree containment | T30; T31; S-09; S-10; 8.3; 8.5 Phase C; 8.8; 10.2; 14 |

The three required amendments are applied as well: instability is refused only as *detected* (T51,
6.2), no receipt-authenticity claim is made (T52, 5.3, 5.4, 11.5), and the goal reads "authorises and
attempts execution of exactly one" (2.1, S-24).

**Remaining OWNER_DECISION_REQUIRED_BEFORE_IMPLEMENTATION: none.** No unresolved product-contract
owner question remains. The next owner decision is whether to accept revised ADR-0024 (section 18).

#### Questions as proposed at `930ec14`

| # | Question | Why it blocks | Recommendation |
|---|---|---|---|
| **Q1** | How is ADR-0024 disposed of: revised and then accepted, or accepted as written with an amendment note? | ADR-0024 as written says `fork()` is the mechanism, that clean EOF plus a normal exit concludes exec, and it retains a procfs fallback (S-01, S-04, S-15). Accepting that text would accept superseded statements | **Revise, then accept.** One bounded ADR revision that adopts `clone3(CLONE_PIDFD)`, the S4 policy, the two-axis outcome model, removal of the procfs fallback, and the S-09/S-14 refinements, followed by an owner acceptance decision |
| **Q2** | Approve the product refinements beyond the evidenced spike and the new constants, to be validated by product tests rather than a formal trial? They are: T08 writable-descriptor refusal; T19 unconditional `F_DUPFD_CLOEXEC ≥ 3`; T21 all signals blocked across `clone3` with `SIGACTION` before `SIGMASK`; T31 guarded sweep; T40 bounded post-kill reap and `not_ended`; T41 `read_failed`; `ChangedDuringMeasurement`; causal-neutral timeout facts (S-13); `POST_KILL_REAP_MS = 5000`; `MAX_EXECUTABLE_BYTES = 512 MiB`; `capture_prefix_bytes` as the only stream option | each one changes the child sequence, the receipt vocabulary or refusal behaviour, relative to what Trial #3 exercised. An implementer must not pick them silently | **Approve as a set.** Each one closes a gap the spike left open, and each has a named product test |
| **Q3** | Confirm that one `SIGKILL` process-group sweep is issued on **every** return path, including a normal exit | it is the evidenced behaviour (P1 `descendant_died`, spike line 988), but the user-visible consequence was never ruled on: a program that leaves a same-group background process has it killed when `launch` returns | **Confirm**, and document it as behaviour, not containment. A timeout-only sweep would leave same-group processes holding launcher pipes and is not evidenced |

At proposal time, no other question blocked implementation.

### 17.2 IMPLEMENTATION_DETAIL_WITHIN_APPROVED_CONTRACT

* How test fixture binaries are packaged: `[[bin]]` targets behind a non-default feature, or a
  dev-only package.
* The `asm!` syscall shim's exact form, provided no glibc code runs in the child window.
* Exact error-code spellings and `Display` text.
* The strace output parser used by Level 2, reusing the lessons of definition §2 (`R-1`, `V-1`,
  `V-2`) without its code.
* The precise rustix feature list and whether `event::poll` or a raw `poll` wrapper is used.
* CI job layout inside the two existing workflows.

### 17.3 BACKLOG_NONBLOCKING

| ID | Item |
|---|---|
| B-01 | N3: a real privilege-transition test in a controlled privileged environment |
| B-02 | `helm-evidence` semantic receipt checks through a new contract version and portable test vectors (R3-M1 rule applies) |
| B-03 | `O_PATH` working-directory capabilities |
| B-04 | any positive exec-evidence mechanism, under its own decision and evidence |
| B-05 | evidence on kernels between 5.9 and 6.17, and on other runner images |
| B-06 | the report-producer / static-posability audit of LAUNCH-EXEC-01 report-declaring plans, as recorded in the X2c disposition, which does not reopen any trial |
| B-07 | a sealed-memfd execution path, the only known route to "measured bytes are executed bytes" |
| B-08 | cgroup v2 delegated containment |
| B-09 | an async or non-blocking `launch` API |
| B-10 | receipt provenance or authenticity (for example signing) at a layer above `helm-launch`, under its own decision; `helm-launch` makes no such claim (T52) |

Wine, PWA, MicroVM, custom shell and GUI change no 0.1 API choice and are not listed.

## 18. Next owner gate

### 18.1 What must the owner approve before `crates/helm-launch` may be created?

1. **ADR-0024 acceptance.** Q1 is decided: revise, then consider for acceptance. The revision is
   prepared (2026-09-16) and ADR-0024 stays Proposed. Accepting the revised ADR is the next owner
   decision.
2. **The unsafe-backend exception as concretely scoped.** D-1 arm (i) was decided in principle.
   The owner should confirm this plan's concrete form: restated lint table, one
   `src/backend/` module, asm syscall shim, `#[no_implicit_prelude]` child, the closed syscall set,
   and the confinement, lint-drift and traced-window tests (sections 7.4–7.6, 8.4, 14.2).
3. **The public API boundary** of section 5: types, consumption rules, platform gating, and the
   rule that parsing never yields authority.
4. **Supported and unsupported 0.1 semantics** of sections 2 and 6–11, including the refinements
   approved under Q2 and the guarded every-path sweep approved under Q3.
5. **Dependency direction** of section 12: zero HELM crate dependencies, and `LaunchReceipt` not a
   `helm-evidence` type.
6. **Implementation authorisation** for P1–P5 as sliced in section 16, including the independent
   review at P3 and at P5 before any merge.

The proposal approved none of these. The owner review of 2026-09-16 decided Q1–Q3 only; items 1–6
remain open.

**Update 2026-09-17.** The owner accepted revised ADR-0024. That decides item 1, settles the
architecture behind items 3, 4 and 5 without stabilising any schema or API, and accepts the scoped
Linux unsafe backend of item 2 as architecture for a later implementation slice. Item 6 is decided
for **P1 only**. Implementing the unsafe backend, and P2–P5 generally, stay **not authorised**.

### 18.2 What must be tested before ADR-0024 could be accepted?

Architecture acceptance and product acceptance are separate, following the `helm-observe` and
`helm-bind` precedent.

* **Before ADR-0024 acceptance: no new execution and no formal trial.** The mechanism's kernel
  semantics rest on the Trial #3 evidence and the accepted X2c disposition. What acceptance needs
  is a **review of the revised ADR text** against section 4, so that no superseded statement is
  accepted and no UNVALIDATED row of section 3 is presented as established. On 2026-09-16 the owner
  set that gate as **owner review of revised ADR-0024**.
* **Before any product merge:** Levels 1–4 green, including the traced child-window test that
  closes T28 for the Rust implementation, then an independent crate review and an owner merge
  decision.

### 18.3 Boundary of this plan

The proposal (`930ec14`) created only this document. It changed no crate, Cargo file, workflow,
ADR, architecture document, decision record, project state, experiment file or evidence. It ran no
launcher, helper ELF, trial, workflow dispatch or case.

The owner-review amendment of 2026-09-16 changed only this document, ADR-0024, `DECISIONS.md` and
`PROJECT_STATE.md`. It changed no crate, Cargo file, workflow, architecture document, experiment
file, freeze manifest or evidence, and it ran no launcher, helper ELF, trial, workflow dispatch or
case.

## 19. Owner review amendments, 2026-09-16

The owner review recorded
**`HELM_LAUNCH_PRODUCTIZATION_PLAN_OWNER_REVIEW_PASSED_WITH_BOUNDED_AMENDMENTS`**. This section is
the change log against the proposal at `930ec14b940da9b136c7d2ad024b441d47ceba6c`. Section 1's
evidence hierarchy is unchanged, no row of section 3 changed class, and T51 and T52 are added rows.
Trial #3's frozen result, every experiment file and every evidence file are unchanged.

| # | Owner amendment | Applied in | How |
|---|---|---|---|
| 1 | Q1 approved: revise ADR-0024 before acceptance | header; 1.1; 4 (intro); 16 (P0); 17.1; 18 | ADR-0024 revised in place, still Proposed; section 4 keeps citing its pre-revision text |
| 2 | Q2 approved with exact narrowing | section 3 intro, counts and load-bearing note; T08, T19, T21, T31, T40, T41; S-08, S-13, S-16; 6.2; 8.1; 8.3; 8.4 (stage 2, deviation note); 8.5; 9.1; 11.3; 17.1 | owner approval named in the authority columns; the evidence class is unchanged; stage 2's close-on-exec clear is described as a backstop, since relocation makes `dup2(fd, fd)` unreachable |
| 3 | Q3 approved with a group-authority guard | 1.3 (P1–P4 row); 2.4; T30; T31; S-09; S-10; 7.3 (pid-valued signal); 8.3 step 3; 8.5 Phase C; 8.7; 8.8; 10.2; 11.2 (termination row); 13.2; 14.1–14.4; 15 | see the two applied specifications below; no text claims that the sweep reached or killed a descendant |
| 4 | Measurement instability: refuse only what the protocol **detects** | 2.4; T51; 6.2 steps 2, 6 and 7; the capability assertion; 13.2; 14.3; 16 (P2) | `ChangedDuringMeasurement` renamed `MeasurementInstabilityDetected`; the protocol is described by what it observes; the measurement read stops one byte past the sampled size |
| 5 | Receipt authenticity: none claimed | 2.4; T52; S-25; 5.1; 5.3; 5.4; 10.4; 11.1; 11.5; 12.5; B-10 | "forge a receipt" split from "forge a capability"; the receipt is data without authenticity |
| 6 | "Executes exactly one" made precise | 2.1; S-24 | "authorises and attempts execution of exactly one admitted object through the exact authorised descriptor" |
| 7 | Signal contract | T21; S-08; S-11; 7.3 (exact use); 7.5; 7.6; 8.3 steps 1–3; 8.4 stages 6–7; 12.6; 14.2; 14.4 | raw full-set `rt_sigprocmask`, because glibc's `sigfillset` and `pthread_sigmask` exclude its two internal real-time signals from the mask (glibc 2.39 source); `clone3` and both mask calls go through the shim, never `libc::syscall`; `SIGKILL`/`SIGSTOP` never described as blockable; no process-wide claim |
| 8 | A child that cannot be observed ended, represented honestly; reap bounded after `SIGKILL` | T34; T40; 8.5 (common rules, Phase A rows, Phase B, Phase C); 8.6; 10.2; 11.3; 13.2; 14.1; 14.4 | `not_ended` renamed `end_not_observed`; every `SIGKILL` sent to the direct child by pidfd, including the pre-exec one, is followed by the bounded kill wait; after a status record the loop waits for the pidfd until the Phase A deadline, then kills if needed, so a child exiting by itself is not misrecorded as `end_not_observed` |

**Two applied specifications for the owner's ADR review.** The review fixed the rule. These are how
this plan and revised ADR-0024 apply it, and neither is presented as a separate owner decision:

* **What "positively established" means.** Only the launcher's own `setpgid(child, child)` returning
  0, issued as the first system call after `clone3` (8.3 step 3). If the child execs first, the
  parent's call fails `EACCES`. The executed image is still in its dedicated group, through the
  child's own stage, but the launcher holds no sweep authority and records
  `not_issued_group_not_established`.
* **Why an established authority can still yield no sweep.** If the probe just before the sweep
  shows that the child was already reaped elsewhere (R4), the owner's rule that the sweep precede
  the reap can no longer be met, and the numeric group id no longer positively denotes the
  dedicated group. The sweep is then recorded as `not_issued_child_already_reaped` (8.5 Phase C).

**Editorial corrections made with the amendment.** Two literal NUL bytes that `930ec14` committed in
place of the JSON escape `\u0000` (9.3, 14.1) are restored as text. Section 8.6's "14 s beyond
`timeout_ms + grace_ms`" is corrected to 12 s, which the section's own constants give (5 000 +
5 000 + 2 000 ms). An unescaped `|` that split a 7.3 table row is escaped. Admission step 1 now
scopes `ETXTBSY` to the evidenced kernel, because the refusal does not rely on that kernel behaviour.

> **OWNER REVIEW OF REVISED ADR-0024 IS REQUIRED BEFORE ACCEPTANCE OR CREATION OF
> `crates/helm-launch`.** *(State at 2026-09-16.)*

> **2026-09-17: ADR-0024 IS ACCEPTED. HELM-LAUNCH P1 IS AUTHORISED; P2+ IS NOT.
> `crates/helm-launch` MAY NOW BE CREATED ONLY WITHIN P1 SCOPE. NO TRIAL #4 IS AUTHORISED.**
