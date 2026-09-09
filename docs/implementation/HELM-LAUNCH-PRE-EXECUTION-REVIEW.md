# helm-launch 0.1 pre-execution review — LAUNCH-EXEC-01 and ADR-0024

**Role:** bounded three-workstream independent review, before any launch experiment is
authorised, by reviewers who are not the architecture author.\
**Reviewed base:** authoritative main `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`.\
**Reviewed tip:** `a5f09f8df8c3868b7ec2f3f6f9d973058c164a44` on `docs/helm-launch-architecture`,
comprising `ae4591e` (architecture), `be0251b` (LAUNCH-EXEC-01, NOT_RUN) and `a5f09f8`
(proposed ADR-0024 and project-state records). All three are preserved unchanged in history.\
**Authority:** **Proposed** [ADR-0024](../adr/ADR-0024-launch-authority.md). It is not Accepted,
and this review does not accept it.

**Classification at the time of review: NEEDS_ARCHITECTURE_OWNER_REVIEW.**

> **Owner disposition, 2026-09-09 — the open decisions are now resolved.** The owner ruled on
> every decision this review recorded as provisional or open, and added one the review's own A-3
> finding implied but did not name:
>
> | # | Ruling | Effect on this review |
> |---|---|---|
> | **D-1** | **arm (i)** — scoped unsafe backend; FD isolation not weakened to keep crate-wide `forbid` | Confirms section 8 and section 12. F2 is mandatory, with no documented-failure path |
> | **D-2, D-3, D-5, D-6** | Accepted as recorded | No change |
> | **D-4** | Accepted **as corrected** by section 9 | The addition this review required is now the decision |
> | **D-8** | Accepted | Its stated condition is met: the pidfd is polled, so the ordering is observed |
> | **D-9** | **Refuse** set-id objects at admission | X7 becomes a deterministic admission-refusal test, not a conditional privilege observation |
> | **D-10** | **Empty environment only** | V2, V4 and V5 leave the experiment as plan-parse properties; section 11's recommendation is taken |
> | **D-11** | **New:** `PR_SET_NO_NEW_PRIVS` before exec | Closes the half of A-3 that D-9 cannot reach — **file capabilities**, which admission metadata does not carry. New cases N1, N2, N3 |
> | **D-7** | **NOT granted** | The experiment is still not authorised to run |
>
> The definition was re-frozen from 71 to **72 cases** and is now freezable; the disposable
> experiment sources exist and are hashed. **LAUNCH-EXEC-01 remains NOT_RUN, `crates/helm-launch`
> was not created, and ADR-0024 remains Proposed.** Nothing in the findings below was edited by
> this disposition: section 18's recommendation describes the state at the time of review, and
> the one bounded reason it gave for withholding freeze — D-9 and D-10 being open — is what the
> owner has now closed.

> **LAUNCH-EXEC-01 was not executed.** No trial ran, no case was posed, no result exists, and
> LAUNCH-EXEC-01 remains **NOT_RUN**. `crates/helm-launch` was not created. No Wine, no 7-Zip,
> no A0 access, no lab change, no privileged operation, no change to main, and no change to any
> existing crate. **ADR-0024 remains Proposed.** **A0-7ZIP remains experimental FAIL.**

The three workstreams returned **sixteen BLOCKER findings** among 53 in total, two of which were reached
independently by two workstreams each. The core mechanism survives — `execveat` on a retained
descriptor does pin the inode, and the rejections of `fexecve`, path-based `Command` and
`posix_spawn` are correct — but the frozen definition could have returned `MECHANISM_ACCEPTED`
while four load-bearing claims were false, and the child-setup sequence as written could not
have executed a single successful launch. The bounded corrections are applied to the
architecture, the preregistration, the ADR and the project records. Two **new owner decisions**
(D-9, D-10) that the review surfaced are unresolved, and they change case membership, so the
definition is **not yet freezable** and **D-7 is not authorised**.

## 1. Method and independence

Three reviewers worked in **isolated contexts and in parallel**, each reading the required
document set independently and none seeing the others' work. None could write to the repository;
each returned findings only, so no reviewer could tidy away another's finding. The parent
session acted as synthesis agent afterwards and resolved disagreements **by technical evidence,
never by majority vote** — section 6 records every place the resolution differs from what any
single workstream proposed.

| Workstream | Scope | Findings |
|---|---|---|
| **A** | executable identity, `execveat`, the unsafe backend | 16 (4 BLOCKER, 9 IMPORTANT, 3 MINOR) |
| **B** | descriptors, environment, I/O, timeout, process lifecycle | 16 (7 BLOCKER, 6 IMPORTANT, 3 MINOR) |
| **C** | preregistration, oracles, verdict logic | 21 (5 BLOCKER, 11 IMPORTANT, 5 MINOR) |
| **Total** | | **53 (16 BLOCKER, 26 IMPORTANT, 11 MINOR)** |

Each workstream read `AGENTS.md`, [project state](../PROJECT_STATE.md),
[decisions](../DECISIONS.md), [ADR-0022](../adr/ADR-0022-observation-authority.md),
[ADR-0023](../adr/ADR-0023-binding-authority.md), ADR-0024, the
[architecture](../research/HELM-LAUNCH-ARCHITECTURE.md), the
[preregistration](../experiments/LAUNCH-EXEC-01-DEFINITION.md), the current `helm-observe` and
`helm-bind` public contracts, and the workspace lint configuration. Syscall claims rest on
primary kernel and man-page sources, cited at the finding. Where a workstream verified a claim
against vendored crate sources rather than accepting the design report's account, it is noted.

## 2. Independently reconstructed state

| Claim | Method | Result |
|---|---|---|
| workspace forbids unsafe | `Cargo.toml` `[workspace.lints.rust]` | `unsafe_code = "forbid"`, plus `unwrap_used`/`expect_used`/`panic = "deny"` under clippy |
| every crate inherits it | `[lints]` in all four crate manifests | all four use `workspace = true` |
| rustix has no `close_range` | grep of vendored `rustix` 1.1.4 `src/` | **zero** occurrences |
| rustix `execveat` is unsafe and unstable | `rustix-1.1.4/src/runtime.rs:456` | `pub unsafe fn` in a `doc(hidden)` module documenting itself as unstable, with safety requirements "not fully documented" |
| the glibc wrappers carry a version floor | vendored `libc` 0.2.189 | `execveat` and `close_range` declared **only** under `linux/gnu/`, `close_range` marked "Added in glibc 2.34"; absent under `musl/` |
| the case count | manual enumeration of the definition's ten tables | **43** — E 6, A 6, V 4, F 4, X 6, O 5, R 3, T 4, P 2, S 3, matching the recorded total |
| no `frozen_cases.py` exists yet | `docs/experiments/` listing | only `obs-fs-01/`; every section 2 artefact is still unwritten |
| `strace` on the recommended runner | `actions/runner-images` Ubuntu 24.04 manifest | **not installed**; neither is `build-essential` |

Section 1.1 of the design report is accurate on both the rustix and the workspace-lint facts.
The design's own account of what it cannot do in safe Rust is confirmed, not merely repeated.

## 3. Workstream A findings — executable identity, `execveat`, the unsafe backend

| # | Severity | Finding |
|---|---|---|
| **A-1** | **BLOCKER** | The receipt's executable digest is a **pre-execution measurement**, and the documents present it as the identity of the body that ran. `execveat` does not execute the caller's open file description: `fs/exec.c:do_open_execat` performs a fresh open with `.acc_mode = MAY_EXEC` and takes the write-deny reference **at exec time**, so `binfmt_elf` maps the inode as of that moment. Section 32's "`ETXTBSY` … means such a writer **usually** breaks the exec instead" is unfounded: a writer that opens, writes and **closes before the exec** returns `i_writecount` to zero and is never refused. That is the shape of every editor, `cp`, `install` and package manager |
| **A-2** | **BLOCKER** | The child's **signal mask and inherited ignored dispositions** are ambient hidden inputs the contract never closes. `signal(7)`: the mask is inherited across `fork` and "preserved across `execve(2)`", and "the dispositions of ignored signals are left unchanged". Rust's runtime sets `SIGPIPE` to `SIG_IGN` before `main`, so every child launched from a Rust host runs with `SIGPIPE` ignored — changing exactly the behaviour case O5 tests. No frozen case would detect it |
| **A-3** | **BLOCKER** | "The child runs with the **caller's own OS credentials**" is unconditionally false for a set-user-ID, set-group-ID or capability-bearing object. `execve(2)` honours those bits unless `no_new_privs`, a `nosuid` mount or ptrace suppresses them; section 28 sets none of the three and section 6 records mode bits without gating on them. The stated upper bound on what a launch can do is wrong |
| **A-4** | **BLOCKER** | Four-byte ELF magic does not establish that `binfmt_elf` handles the object. A `binfmt_misc` entry matching `\x7fELF` with a masked `e_machine` — `qemu-user-static` is the ubiquitous case, and foreign architectures are HELM's own problem domain — is consulted **before** `binfmt_elf` and resolves its interpreter **by pathname** at exec time, violating the design's own falsifier 4. A hosted runner has no such registration, so the frozen definition would return `MECHANISM_ACCEPTED` without ever posing the case |
| **A-5** | IMPORTANT | Section 17's child sequence closes the working-directory descriptor before using it: step 2's `close_range` preserves only the exec fd and the status pipe, then step 3 calls `fchdir(dir_fd)` on a closed descriptor. As written **no launch can succeed** |
| **A-6** | IMPORTANT | `dup2` clears `FD_CLOEXEC` only when `oldfd != newfd`; `dup(2)`: "if *newfd* has the same value as *oldfd*, then dup2() does nothing". On a host with stdio closed, a `CLOEXEC` pipe end allocated at its own target number stays `CLOEXEC`, the kernel closes it at exec, and the launcher records `bytes_drained: 0` with a digest of the empty string |
| **A-7** | IMPORTANT | `pidfd_open` after `fork` is sound only under the three conditions `pidfd_open(2)` NOTES states (no `SIGCHLD = SIG_IGN`, no `SA_NOCLDWAIT`, no other reaper). `helm-launch` is a **library** and can establish none of them. Under `SIGCHLD = SIG_IGN` the child is auto-reaped, the PID is immediately reusable, and `pidfd_open` may open an unrelated process the launcher then signals |
| **A-8** | IMPORTANT | `execveat` on an interpreter-requiring file fails **`ENOENT`** when the exec fd is `O_CLOEXEC` — `execveat(2)` ERRORS and BUGS, and `fs/exec.c` sets `BINPRM_FLAGS_PATH_INACCESSIBLE`. X2's single side-observation would record a CLOEXEC artefact and attribute it to scripts as a class |
| **A-9** | IMPORTANT | The digest does not identify a **dynamic** ELF's executed image: the kernel resolves `PT_INTERP` by pathname and `ld.so` resolves `DT_NEEDED` by name. Every helper is statically linked, so **no preregistered case executes a dynamic object at all**, while every real HELM subject is dynamic |
| **A-10** | IMPORTANT | The cohort floor is stated as kernel 5.9 only. The glibc wrapper route silently adds **glibc >= 2.34** and excludes musl entirely; the raw-syscall route takes no libc floor |
| **A-11** | IMPORTANT | The **multithreaded parent** — which every real HELM caller is — is unanalysed. glibc's `fork()` runs every registered `pthread_atfork` handler in the child before any helm-launch code, outside the "one file, no allocation, no locking" boundary section 8 claims |
| **A-12** | IMPORTANT | The aggregate rule makes E6 simultaneously **ungradeable and mandatory-PASS**: section 5 requires "Every E … case is PASS" while section 3 says E6's outcome is "recorded, not predicted". P1/P2 are carved out of the gate; E6 is not |
| **A-13** | IMPORTANT | "No allocation, no locking, no formatting, no panic path" in the post-fork child is asserted with **no mechanism that could falsify it**. No frozen case would fail if the child allocated, took a futex or ran a `Drop` |
| **A-14** | MINOR | The process-group sweep signals a **numeric pgid** in a design that refuses to expose a numeric PID for exactly that reason |
| **A-15** | MINOR | Read permission is required to measure but not to execute, so 0.1 cannot launch execute-only objects the kernel would run. An unstated narrowing |
| **A-16** | MINOR | `O_PATH` would be accepted by the kernel (`execveat(2)` says so explicitly); the refusal is HELM's measurement requirement. X6 must not be read as a kernel fact |

**A mechanism verdict: SOUND WITH CORRECTIONS.** What the descriptor buys is immunity to rename,
unlink and pathname replacement of the main object. What the documents over-claim is content
immutability (A-1), coverage of a dynamic object's loaded-code closure (A-9), and immunity to a
`binfmt_misc` interpreter (A-4).

## 4. Workstream B findings — descriptors, environment, I/O, timeout, lifecycle

| # | Severity | Finding |
|---|---|---|
| **B-1** | **BLOCKER** | **The drain loop has no termination condition that survives a descendant holding a stdio write end.** `pipe(7)` gives EOF only when *all* write-end descriptors close, and `poll(2)` POLLHUP "merely indicates that the peer closed its end". A descendant forked after exec inherits fds 1 and 2, so after the direct child exits the loop's only exits — both streams at EOF, or deadline expiry — are respectively unreachable and already consumed. `launch()` re-enters `poll()` and never returns. Because P1 and P2 are declared "not pass/fail gates", **a launcher hang is a recordable outcome under the frozen rules and does not block acceptance** |
| **B-2** | **BLOCKER** | The **pidfd is never polled** — the poll set is enumerated exhaustively as three descriptors, none of them the pidfd — although `pidfd_open(2)` documents that it becomes readable when the task becomes a zombie. Direct-child exit is therefore not observed, and section 23's classification is driven by "the launcher issued a signal". The receipt then asserts `TimedOut { ExitedDuringGrace }` for a child that exited **before** the deadline and was never affected by any signal. "The launcher always knows which case applies because it knows whether it issued the signal" is false |
| **B-3** | **BLOCKER** | **The parent never closes its own copies of the pipe endpoints.** `fork(2)` gives the parent a copy of each write end; the child's `CLOEXEC` closure is necessary and not sufficient. Implemented literally, the design fails on the first successful launch: `status_r` never reaches EOF, every run burns `SPAWN_CONFIRM_TIMEOUT_MS` and reports `ExecStatusIndeterminate`, and B-1's hang occurs **with no descendant required** |
| **B-4** | **BLOCKER** | `dup2`/`close_range` ordering omits relocation. If the exec fd or status write end was allocated at 0-2, step 1 destroys it — and a destroyed status pipe reads as EOF, which section 21 defines as "exec succeeded" (falsifier 8 by construction). Source collisions make three `dup2`s order-dependent; the `dup2(fd, fd)` no-op leaves `FD_CLOEXEC` set; and adjacent preserved descriptors make the middle `close_range(a+1, a)` return `EINVAL` inside a region whose only exits are `execveat` and `_exit` |
| **B-5** | **BLOCKER** | The process-group sweep uses a **numeric pgid with no ordering rule against the reap**. POSIX.1 reserves a process-group ID only while the group is non-empty, and a zombie keeps it alive (`kill(2)`). A sweep issued after `waitid` may `SIGKILL` an unrelated recycled group owned by the same user — the launcher acting on processes it never created, which is stronger than the limitation D-4 openly accepts. "**may** additionally be issued" is also not a contract |
| **B-6** | **BLOCKER** | `ExecutionOutcome` is a **sum type where the facts are orthogonal**. A child that exits 42 while a stream read fails has two facts and the enum carries one; whichever is kept, the receipt asserts a falsehood by omission. There is no outcome at all that can express B-1's state. Both accepted modules already solved this shape — `helm-observe`'s `TargetOutcome::Failed { …, partial_bytes }` and ADR-0023's two-axis result — and this design regresses from both |
| **B-7** | IMPORTANT | Per-stream `sha256` and `truncated` over-claim, and `truncated` is overloaded: it means "the retained prefix was cut" in section 19 and will read as "the output was truncated" in the receipt. A deadline-truncated digest is byte-indistinguishable from a complete one |
| **B-8** | IMPORTANT | No rule removes an EOF'd descriptor from the poll set. POLLHUP is "ignored in *events*" and cannot be masked, so a descriptor left in the set makes every `poll()` return immediately — a **100% CPU spin** for the remaining timeout. O3's "completes well inside the timeout" and O5's "no hang" are both satisfied by a spinning launcher |
| **B-9** | **BLOCKER** | POLLHUP must not be interpreted before POLLIN is drained. A child that writes the failure record and `_exit`s sets both in one `poll` return; an implementation reading hangup first applies section 21's "clean EOF, no record" rule and reports **exec success for a child that never execed** |
| **B-10** | IMPORTANT | The pre-exec timeout has a bound but **no defined action, no outcome and no case**: a stalled child leaves a live process, a live pidfd and four open pipe ends in a library caller's address space, with no obligation to terminate or reap |
| **B-11** | IMPORTANT | `helm-launch` reaps children in **someone else's process**. `wait(2)`: under `SIGCHLD = SIG_IGN` or `SA_NOCLDWAIT`, children do not become zombies and `waitid` fails `ECHILD`. Section 22's soundness argument silently assumes the launcher is the only reaper |
| **B-12** | IMPORTANT | Ambient host **signal state** is the fifth hidden input and the contract does not acknowledge it. A host that blocks `SIGTERM` hands the child a blocked `SIGTERM` across exec, so section 23's grace window is guaranteed to be burned and T2/T3 become host-dependent. *(Independently found as A-2.)* |
| **B-13** | IMPORTANT | The `LD_*` rationale over-claims three ways: the property it names is established by `execveat`, not by the environment policy; "the measured process image" is false because no process image is measured; and the `LD_` rule is an **incomplete prefix denylist** — glibc's own secure-execution list also strips `GCONV_PATH`, `LOCPATH`, `NLSPATH`, `TZDIR` and others, none of which 0.1 refuses |
| **B-14** | MINOR | `group_sweep_issued` is promised in section 24 and absent from section 29's "all recorded" inventory and section 30's vocabulary |
| **B-15** | MINOR | X2's counterfactual is left open where `execveat(2)` already fixes it at `ENOENT`. *(Converges with A-8.)* |
| **B-16** | MINOR | S2's fixture is not constructible as written. *(Converges with C-8.)* |

**B lifecycle verdict: UNSOUND** as written. The authority boundary is not what fails; what
fails is that the boundary was drawn around the **process** while the design shares **kernel
objects** across it — pipes, a numeric pgid, the host's signal state and its `SIGCHLD`
disposition. Every finding is correctable inside the current scope: none requires process-tree
containment, none requires broadening the environment, none requires abandoning D-4.

## 5. Workstream C findings — preregistration, oracles, verdict logic

| # | Severity | Finding |
|---|---|---|
| **C-1** | **BLOCKER** | The aggregate precedence is **neither total nor disjoint**. An A3 FAIL with everything else PASS satisfies none of the three rows — no verdict is defined, and the same hole exists for 22 other cases. An E2 FAIL together with a BLOCKED case fires REJECTED and INCONCLUSIVE simultaneously with no precedence. OBS-FS-01 fixed exactly this (`checker.py`, "Frozen precedence: FAIL > BLOCKED > INCONCLUSIVE > PASS"); LAUNCH-EXEC-01 drops the precedent. "Mandatory" is used in the rules and **defined nowhere**, and the INCONCLUSIVE row's "cannot be corrected" contradicts section 4's "No retries" |
| **C-2** | **BLOCKER** | F2 is scored **after** the trial under an undecided D-1. If the owner takes arm (i), an F2 FAIL is falsifier 7 and must reject; as written the experiment can report `MECHANISM_ACCEPTED` while exact descriptor isolation is false. Scoring latitude resolved after the trial is goalpost movement of the class OBS-FS-01 halted for |
| **C-3** | **BLOCKER** | The **primary instrumentation contradicts the invariant it instruments.** Section 2 has the helper report "on a descriptor chosen by argv" and section 4 has it "written to a descriptor the harness controls" — a descriptor above 2 that must survive `close_range` into the executed image, so F1/F4 would score PASS while "exactly 0, 1 and 2 survive" is false |
| **C-4** | **BLOCKER** | **Clean EOF does not prove exec.** A child killed between `fork` and `execveat` closes its `CLOEXEC` write end by dying, so the parent sees a clean EOF byte-identical to success. S1 uses the mechanism under test as its own oracle; nothing in the 43 can falsify this, and falsifier 8 is reachable through the design |
| **C-5** | **BLOCKER** | **E6's safe set is illegitimate.** The two members are not equally acceptable: `ETXTBSY` leaves the identity model true, execution-with-changed-bytes makes the receipt name bytes that did not run. The set is also non-exhaustive — `ENOEXEC`, a post-exec `SIGBUS`, and the writer-already-closed case are all honest kernel behaviour and all score FAIL, hence `MECHANISM_REJECTED`. Both members are individually forceable, so the nondeterminism is an artefact of not specifying a schedule |
| **C-6** | IMPORTANT | E5 does not say **which process** holds the writable descriptor — one held in the child is closed by `close_range` and `ETXTBSY` never fires, failing a correct mechanism — and has **no oracle** proving the writable descriptor refers to the same inode |
| **C-7** | IMPORTANT | **X1 and X3 are the same kernel state**: a "non-executable regular file" *is* a file whose mode lacks an execute bit. Both reach `inode_permission(MAY_EXEC)` and return the same `EACCES` from the same branch. The rejection path is tested twice and counted twice |
| **C-8** | IMPORTANT | **S2 is unrealisable**: the working directory is an already-open capability, so "unopenable" describes a state admission has already passed, and an unopenable directory fails at `directory_from_fd` before any child exists — no child, no record, no receipt |
| **C-9** | IMPORTANT | P1/P2 can hang or contaminate: the descendant inherits the launcher's pipe write ends (converging with B-1 from the protocol side), nothing orders them after the mandatory cases, a `setsid`'d descendant has no lifetime bound, and a P1 whose `fork` failed has no INVALID rule |
| **C-10** | IMPORTANT | **`strace` is not installed** on `ubuntu-24.04` hosted runners, and installing it needs `sudo`, which section 6 forbids — so the preferred oracle for the single most important claim is unobtainable, with no rule saying so. Worse, `ptrace(2)` delivers exit and signal notifications **to the tracer before the real parent**, and the launcher *is* the real parent, so tracing perturbs exactly the R/T/S cases that measure `waitid` classification |
| **C-11** | IMPORTANT | "Statically linked **where practical**" is a silent-substitution licence, and the assumption is unverified: `build-essential` and `libc6-dev` are not in the runner manifest. A dynamic substitution would put loader traffic in the trace and destroy the completeness on which every negative claim rests |
| **C-12** | IMPORTANT | **A5 and V3 cannot be posed**: both are plan-parse properties of a crate that does not exist and that ADR-0024 forbids creating, and a NUL byte cannot be delivered through a C `argv` at all. Both are already assigned to section 41's in-crate obligations |
| **C-13** | IMPORTANT | pidfd/status-pipe **ordering** is untested: S3 tests value discrimination, not ordering, and section 22's soundness claim is documentation-derived and unposed |
| **C-14** | IMPORTANT | The O-series digest oracles are **uncomputable**: a byte *volume* is not a byte *recipe*, and `oracles.py` is specified as hashing "the byte recipes" that no document defines. O1/O2/O4 also never state the per-stream capture mode, and `discard` records no digest at all |
| **C-15** | IMPORTANT | T3 and T4 freeze **no schedule**: "during the grace window" and "just before the deadline" are unquantified races, and a slip in T4 produces an outcome outside its safe set, failing a correct mechanism. T4's safe set is otherwise the only legitimate one in the document |
| **C-16** | IMPORTANT | **No privacy or sanitisation rule.** The helper must report its complete `environ` with exact values, and V1 deliberately populates the host environment; if V1 FAILs, the published report contains the runner's environment verbatim — on GitHub Actions that includes `ACTIONS_RUNTIME_TOKEN` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN`. OBS-FS-01 published an explicit rule; this does not |
| **C-17** | MINOR | X2's "separate observation" is an unlabelled 44th case with no class, prediction, oracle or disposition — and its answer is determinate and is not what section 6 states |
| **C-18** | MINOR | E2 and E4 are not distinguished at the level of detail given, yet the aggregate singles both out as instant-rejection cases |
| **C-19** | MINOR | X4's fixture is ambiguous and half unposable: a non-ELF file cannot reach exec, because the ELF rule refuses it at admission |
| **C-20** | MINOR | Source-freeze and first-valid-trial rules are weaker than the OBS-FS-01 precedent, and the **preflight-halt rule is missing entirely** — the rule under which OBS-FS-01 halted when a frozen expectation was found factually wrong before its first trial |
| **C-21** | MINOR | Hosted-runner assumptions are unstated, and the existing workflows' `cancel-in-progress: true` would truncate a trial mid-run |

**C mechanical audit: verified count 43**, matching the recorded total. The document declares
**no** mandatory/conditional/negative-control partition anywhere; the classes are inferable only
from the aggregate rules, and the inference is self-contradictory (A-12/C-1). Of the 43,
**14 PASS and 29 carry a defect**; E6, A5, V3 and S1 are unfalsifiable or unposable as mandatory
cases, and S2 is unrealisable.

**C: READY TO FREEZE? NO.**

## 6. Cross-workstream conflicts and their resolution

Resolved by evidence. Where the synthesis differs from every workstream, that is stated.

**6.1 Convergence, treated as strengthening rather than duplication.** Two findings were reached
independently by two workstreams from different directions and are recorded at the higher
severity: **inherited signal state** (A-2 BLOCKER, B-12 IMPORTANT, taken as **BLOCKER**) and the
**`dup2(fd, fd)` no-op** (A-6, B-4). S2's unrealisability (B-16, C-8) and X2's `ENOENT` (A-8,
B-15, C-17) converged three ways.

**6.2 The receipt field name — three names, two different defects.** A and C proposed temporal
names (`pre_exec_measured_sha256`, `executable_sha256_at_measurement`); B proposed an extensional
one (`executable_body_sha256`). Neither narrowing subsumes the other: A-1 is *when* the
measurement was taken, B-13/A-9 is *what* it covers. **Resolution: `pre_exec_body_sha256`**, with
siblings `pre_exec_body_size` and `pre_exec_mode_bits` — `pre_exec` carries the temporal fact,
`body` the extensional one. No workstream proposed this name; each carried half the correction.

**6.3 Descendant-held pipes — a direct contradiction.** C-9 requires `helper_fork`'s descendant
to close 0/1/2 and reopen on `/dev/null` "as its first three actions", so it can never hold a
launcher pipe. B-1 requires exactly that retention, because it is the state that hangs the
launcher. **Resolution: both, in different cases.** P1/P2 exist to establish the *lifecycle*
non-claim, for which a retained pipe is a confound that can hang the run — C's isolation applies
there. **O6** and **P4** exist to establish the *liveness* property, for which retention is the
whole point — B's construction applies there, gated on bounded return. Neither workstream had
both halves: C's rule alone would have deleted the only evidence for B's BLOCKER, and B's alone
would have left every mandatory case exposed to a leaked descendant.

**6.4 The unobservable exit status — vocabulary.** A-7 proposed a new
`ExitStatusUnobservable { reason }`; B-11 proposed reusing
`ExecStatusIndeterminate { phase: StatusUnavailable }`. **A is correct, on B's own argument**:
`ExecStatusIndeterminate` is about *exec confirmation*, and overloading it with an *exit-status*
fact is precisely the orthogonal collapse B-6 objects to. Resolution: A's name, placed inside
B's product-type restructure.

**6.5 E6 — one prediction or four cases.** C-5 replaced E6 with a single prediction; A-1 split it
into four. **Resolution: both, partitioned by whether the kernel answer is known.** C's single
prediction becomes **E6** (length-preserving write, writer closed before exec). A's
truncate-and-rewrite becomes **E6b** with a legitimate safe set — all four members are honest
once the field is renamed. A's `fchmod` case becomes **E6d**. A's shared-writable-mapping case
becomes **E6c** and is **carved out of the PASS gate**, because A could not settle from primary
sources whether a surviving `i_mmap_writable` mapping leaves `i_writecount` at zero; C's own rule
forbids a case being both unpredicted and mandatory, so the carve-out is required rather than
optional.

**6.6 The syscall trace — a required oracle that may not exist.** A-13's enforcement of "no
allocation, no locking, no panic path" is case **M1**, which needs a syscall trace; C-10
established that `strace` is absent from the runner and that tracing perturbs the R/T/S cases.
**Resolution: a per-case `traced` declaration** (C's mechanism), with M1/M2/M4 declared
`traced: true` and **conditional** — if no tracer is available they are BLOCKED, which under the
corrected precedence yields `MECHANISM_INCONCLUSIVE`, never `MECHANISM_ACCEPTED`. An unprovable
minimality claim must not be silently accepted.

**6.7 Case-ID collisions.** A and B both proposed an `F5` and an `F6` for different cases; A and
C both proposed an `X7`; B and C both proposed an `S4` and an `S5`. Renumbered in section 13; no
proposed case was dropped to resolve a collision.

**6.8 One workstream overreached.** C-12 deletes A5 and V3 as unposable. That is correct as far
as it goes, but it would leave the properties untested anywhere. The synthesis keeps the deletion
**and** records both explicitly as in-crate obligations with a sentence saying no result here
supports or refutes them, so the deletion cannot later read as "the experiment covered plan
parsing".

## 7. E6 disposition — the receipt identity model

**Determination: the safe set was illegitimate, and the outcome it treated as the unlikely one
is in fact the ordinary one.**

Of E6's two permitted outcomes, `ETXTBSY` leaves the identity model true — nothing executed —
and "pinned-inode execution with changed bytes" makes the receipt name bytes that did not run.
They are therefore not equally acceptable, which is what a safe outcome set requires. Worse, A-1
established from `fs/exec.c:do_open_execat` that the write-deny reference is taken **at exec
time**, so the ordinary open-write-close writer never triggers `ETXTBSY` at all: the arm that
invalidates the model is the one that will happen, and the arm the architecture leaned on covers
only a writer still *holding* a descriptor at the exec instant.

**What the receipt may honestly say.** Only: *these bytes were read through this descriptor
before the execution attempt, and this descriptor was the exec target.* It may **not** be
presented as the executable body that ran. Consequently:

- the field becomes **`pre_exec_body_sha256`** (section 6.2), with `pre_exec_body_size` and
  `pre_exec_mode_bits`;
- section 6's closing sentence, section 32's bullets and block quote, and ADR-0024's receipt
  paragraph state the non-claim in the artifact's own voice;
- falsifier **19** is added: presenting a pre-execution measurement as the identity of the body
  that executed;
- **E6, E6b and E6d are documentation gates as well as mechanism gates** — they cannot be PASSed
  by observing the expected kernel behaviour while any document still presents the digest as the
  executed body;
- `memfd_create(MFD_ALLOW_SEALING)` plus `F_SEAL_WRITE|F_SEAL_SHRINK|F_SEAL_GROW|F_SEAL_SEAL` is
  recorded as the known path to the strong claim — measured bytes provably identical to executed
  bytes — **costed, not adopted**, since it forfeits set-user-ID and on-disk identity semantics
  and interacts with `MFD_NOEXEC_SEAL`.

E6 therefore cannot "PASS while preserving a false identity statement": under the corrected
definition that combination is a FAIL of the case.

## 8. Exact FD-isolation disposition

**Exact isolation is retained and is now provable.** Under owner decision D-1 arm (i), **F2
becomes a mandatory case whose only PASS is that the child does not see the descriptor**, and the
aggregate no longer permits a documented FAIL (C-2). D-1 is recorded as a **frozen input** in the
definition before the first trial, not a scoring choice made afterwards.

Three corrections are required before the invariant is even reachable, and none is cosmetic: the
working-directory descriptor must be used **before** the range close (A-5), `FD_CLOEXEC` must be
cleared explicitly on 0/1/2 because `dup2(fd, fd)` does nothing (A-6/B-4), and the preserved
descriptors must be relocated above 2 in the parent so the gap arithmetic has no special case and
`close_range(a+1, a)` cannot return `EINVAL` (B-4). The instrumentation must also stop violating
the invariant it measures: **no descriptor above 2 is ever passed to a helper**, and the report
travels on descriptor 1 behind a frozen sentinel (C-3).

FD isolation is **not** weakened to retain crate-wide `forbid`, per the owner's D-1.

## 9. Descendant-held-pipe disposition

**The hang is real, it is reachable without any descendant at all (B-3), and the frozen rules
would have recorded it as an acceptable outcome.** The state machine terminates nowhere:
`pipe(7)` withholds EOF while any write-end duplicate lives, POLLHUP is not raised, and the
deadline path leads to termination and reap rather than to loop exit.

The disposition is a **bounded post-exit drain**, fixed by the architecture and deliberately
**not** a plan field, so a plan cannot lengthen the launcher's own tail:

1. after `fork`, the parent closes its copies of `status_w`, `stdout_w`, `stderr_w`, `stdin_r`,
   and closes `stdin_w` (B-3);
2. the poll set is `{pidfd, status_r, stdout_r, stderr_r}` from `fork` onward, each descriptor
   dropped on its terminal event and only after `read()` returns 0 (B-2, B-8, B-9);
3. on child end, the group sweep is issued **strictly before** `waitid` (B-5), then `waitid`;
4. the launcher then drains for at most **`POST_EXIT_DRAIN_MS` = 2000**, and on expiry stops
   reading, closes the read end, and records `WriterRetainedAfterChildExit` for that stream;
5. total bound `SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_EXIT_DRAIN_MS`, and that
   bound is itself a tested property.

This is **not** containment and is not claimed as such: closing the read end may deliver
`SIGPIPE` or `EPIPE` to the retaining descendant, an effect on a process outside the claimed
lifecycle, and it is stated as such rather than described as a sweep that worked. D-4 is accepted
with the addition that **direct-child lifecycle is a claim about what `helm-launch` ends, not
about what it waits for**. A launcher-side non-return becomes an unconditional
`MECHANISM_REJECTED` trigger **including in P1-P4**, which is the single rule that closes the
acceptance path B-1 found.

## 10. pidfd and rapid-exit disposition

Four independent defects, one disposition:

- the pidfd **joins the poll set**, and exit is classified from the **observed ordering** of its
  readability against the monotonic deadline, never from "the launcher issued a signal" (B-2);
- `pidfd_open`-after-`fork` is documented with the **three caller preconditions** `pidfd_open(2)`
  states, in the shape of ADR-0022's cohort clarification — `helm-launch` does not attest and
  cannot enforce them (A-7, B-11) — and `clone3(CLONE_PIDFD)` is elevated from "possible later
  refinement" to the mechanism that removes the precondition, measured as arm (b) of case **M3**
  and adopted if the `fork`+`pidfd_open` arm shows any loss;
- a stolen reap gets an honest outcome, **`ExitStatusUnobservable { reason }`**, distinct from
  `ExecStatusIndeterminate` (section 6.4), so the launcher never reports `Exited { 0 }` for a
  status it did not observe;
- **S4** forces the rapid exec-and-exit schedule with a spike-mode `EXEC_RACE_DELAY_MS = 200` in
  the parent, so the child has certainly execed and exited before `pidfd_open` — a forced
  schedule, not a timing hope — and **S5** kills the child between `fchdir` and `execveat` to
  establish that clean EOF alone does not prove exec (C-4, C-13).

## 11. Environment and `LD_*` wording disposition

**`LD_*` refusal is hardening, not provenance**, and the documents now say so in their own voice.
The measured digest covers only the main executable file body — never the ELF interpreter the
kernel resolves from `PT_INTERP` **by pathname**, never the `DT_NEEDED` objects `ld.so` resolves
by name, never anything later `dlopen`ed. **An empty environment does not close this**; it
removes one way of steering the loaded-code closure, not the closure itself. The `LD_` rule is
additionally recorded as an **incomplete prefix denylist**, since glibc's own secure-execution
list strips `GCONV_PATH`, `LOCPATH`, `NLSPATH`, `TZDIR` and others that 0.1 accepts (B-13).

Case **E7** adds the only dynamically linked helper, precisely because every other helper is
static and the dominant real case would otherwise never be exercised (A-9). Case **V5** records
that a non-`LD_` loader-affecting name **is** accepted, so the denylist's incompleteness is
evidenced rather than asserted.

Whether 0.1 should be **`empty`-only** is a genuine owner decision, recorded as **D-10** and not
taken here. The review's recommendation is `empty`-only: section 15 already concedes it is the
only mode the first synthetic cohort needs, it deletes the weakest reasoning in the document, and
it narrows scope rather than broadening it. If `explicit` is kept, it is acceptable **only** with
all four narrowings of B-13 — the field rename, a recorded `environment_mode`, the denylist
documented as incomplete, and falsifier 19 — and the definition is written to work either way.

## 12. The unsafe backend under D-1

**The review agrees with the owner's D-1 and permits the tiny isolated unsafe backend**, on
evidence rather than preference: option (ii) does not preserve descriptor isolation more cheaply,
it deletes it — leaving a child that inherits whatever non-`CLOEXEC` descriptors the host already
holds, a log file, a listening socket, a credential file, into a process that is explicitly not
sandboxed and runs with the caller's full credentials. Trading a kernel-enforced, mechanically
falsifiable invariant for a lint attribute inverts this repository's evidence discipline.

One correction to how "preserving the workspace lint policy" must be implemented: a crate that
needs `#[allow(unsafe_code)]` anywhere **cannot** inherit `[lints] workspace = true`, because
`forbid` cannot be locally relaxed — and dropping that table silently drops **all four**
workspace lints, not just the one. The three clippy denies must therefore be restated verbatim or
the policy is weakened by accident. The architecture now carries the exact block, the `#[cfg]`
gate to the cohort, `undocumented_unsafe_blocks` as the mechanism that turns "every unsafe block
carries a documented invariant" into a compile error, and a `needs_drop` assertion on the
child-argument type so no destructor can reach the post-fork path by construction.

Two framing corrections: section 8's "~100 lines of unsafe, all syscalls" understates the proof
obligation — the hard parts are the pointer lifetimes spanning `fork`, the `-> !` child contract,
and the `pthread_atfork` surface glibc's `fork()` opens before any helm-launch code runs — and
the unsafe decision now depends on **M1/M2/M4** evidence, not only on E and F results. The
trampoline rejection stands and is not reopened; `memfd` plus sealing is recorded not as a
rejected trampoline but as the costed future path to a stronger receipt (section 7).

**No Rust API is frozen.** A-1's rename, A-3's possible `SetIdBitsPresent`, A-4's
`ElfNotInCohort` and A-7's `ExitStatusUnobservable` all change the public surface, and all four
depend on evidence this experiment has not produced.

## 13. The corrected LAUNCH-EXEC-01 case set

**43 to 71 cases**: two deleted as unposable (A5, V3), thirty added, eleven reformulated in
place. The full table with fixtures, oracles and single predictions is in the
[corrected definition](../experiments/LAUNCH-EXEC-01-DEFINITION.md); this is the membership and
the provenance of each change.

| Series | Members | n | Origin of the additions |
|---|---|---|---|
| **E** | E1-E5, E5b, E6, E6b, E6c, E6d, E7, E8 | 12 | E5b (A-1: write-then-close never yields `ETXTBSY`), E6b/E6c/E6d (section 6.5), E7 (A-9: the only dynamic helper), E8 (A-4: foreign-arch ELF, plus the `binfmt_misc` inventory) |
| **A** | A1-A4, A6 | 5 | A5 deleted (C-12) |
| **V** | V1, V2, V4, V5 | 4 | V3 deleted (C-12); V5 records that `GCONV_PATH` is accepted (B-13) |
| **F** | F1-F7 | 7 | F5 (signal state, A-2/B-12), F6 (host stdio closed, A-6/B-3/B-4), F7 (adjacent preserved descriptors, `close_range` `EINVAL`, B-4) |
| **X** | X1-X8, including X2b and X2c | 10 | X1 reformulated (C-7), X2b/X2c split the counterfactual into its `O_CLOEXEC` and non-`CLOEXEC` arms (A-8), X7 set-user-ID (A-3), X8 `noexec` mount, conditional (C-7) |
| **O** | O1-O8 | 8 | O6 (descendant retains pipes, the B-1 case), O7 (exit status **and** capture failure together, B-6), O8 (POLLIN+POLLHUP in one return, 200 trials, B-9) |
| **R** | R1-R4 | 4 | R4 (host sets `SIGCHLD = SIG_IGN`, A-7/B-11) |
| **T** | T1-T6 | 6 | T5 (exit just before the deadline while a descendant holds the pipes, B-2), T6 (host blocks `SIGTERM`, B-12); T3/T4 given frozen schedules (C-15) |
| **P** | P1-P4 | 4 | P3 (sweep issued strictly before the reap, B-5), P4 (`setsid` descendant retaining pipes, B-1/B-5) |
| **S** | S1-S7 | 7 | S4 (rapid exec-and-exit, C-13), S5 (child killed before exec, C-4), S6 (pre-exec stall, B-10), S7 (record and hangup together, 200 trials, B-9) |
| **M** | M1-M4 | 4 | New series: post-fork child minimality (A-13), multithreaded parent (A-11), pidfd acquisition (A-7), frozen-sequence match (A-5) |

| Class | Members | n |
|---|---|---|
| **Mandatory** — must PASS | E-series except E6c (11); A (5); V1, V2, V4 (3); F1-F7 (7); X1, X2, X2b, X2c, X3, X4, X5, X6 (8); O (8); R1-R3 (3); T (6); S (7) | **58** |
| **Conditional** — may be BLOCKED by the environment | X7, X8, M1, M2, M4 | **5** |
| **Recorded** — outcome not predicted, gated only on named sub-assertions | E6c, M3, P1, P2, P3, P4, R4, V5 | **8** |

The three classes are disjoint, their union is the whole membership, and the counts are recorded
in the definition and must appear in the report. Under D-1 arm (ii) — not the owner's choice — F2
moves from mandatory to conditional and the counts become 57/6/8.

Ten protocol rules land with the membership: the frozen output byte recipe and per-stream modes
(C-14), the fd-1 sentinel report channel and `fcntl(F_GETFD)` enumeration (C-3), the frozen
child-setup `stage` vocabulary (C-8), the per-case `traced` declaration (C-10), the preflight
static-linkability gate with **no silent substitution** (C-11), the privacy and sanitisation rule
including `RLIMIT_CORE = 0` for R3 (C-16), negative-control isolation and last-position ordering
(C-9), the freeze point with built-binary digests (C-20), the **preflight halt** rule OBS-FS-01
established (C-20), and the workflow discipline with `cancel-in-progress: false` (C-21).

## 14. Corrected aggregate verdict

The three-row table is replaced by a **total, disjoint, ordered precedence** — evaluation stops
at the first rule that fires — following OBS-FS-01's frozen precedence of FAIL before BLOCKED
before INCONCLUSIVE before PASS:

| Order | Verdict | Condition |
|---|---|---|
| 1 | `MECHANISM_REJECTED` | any mandatory case is FAIL; **or** `launch()` fails to return within its declared total bound in any case, including P1-P4; **or** the receipt asserts a temporal or causal fact the launcher did not observe |
| 2 | `MECHANISM_INCONCLUSIVE` | no mandatory FAIL, and at least one mandatory case, conditional case or negative control is INVALID or BLOCKED |
| 3 | `MECHANISM_ACCEPTED` | otherwise: every mandatory case PASS, every conditional case PASS or BLOCKED with a recorded cause, and every recorded case carrying a non-INVALID outcome that passes its named sub-assertions |

A case in the frozen membership with no recorded status is **BLOCKED, never absent**. The
instant-rejection list grows from `{E2, E4, O3}` to `{E2, E4, E6, E6b, O3, O6, S4, S5, S7}`, and
**D-1 is a frozen input** recorded before the first trial. The rule that closes B-1's acceptance
path is the unconditional one: a launcher-side non-return is never a recordable outcome.

## 15. Owner decisions recorded provisionally

Recorded as the owner directed. None is an acceptance of ADR-0024, and none authorises execution.

| # | Decision | Status after this review |
|---|---|---|
| **D-1** | Pursue a tiny isolated unsafe backend; do not weaken FD isolation to retain crate-wide `forbid` | **Provisionally recorded.** Agreed on evidence (section 12). Frozen as **arm (i)**; F2 is mandatory |
| **D-2** | Scope B — one explicit synthetic Linux executable substrate before Wine | **Provisionally recorded.** No finding disturbs it |
| **D-3** | Zero HELM crate dependencies in 0.1 | **Provisionally recorded.** No finding disturbs it |
| **D-4** | Direct-child lifecycle only; no process-tree containment | **Provisionally recorded, with a required addition** — it is a claim about what the launcher *ends*, not what it *waits for* (section 9) |
| **D-5** | Mandatory cwd directory capability | **Provisionally recorded.** Agreed without reservation; S2's fixture is corrected (C-8) |
| **D-6** | UTF-8-only argv for experimental 0.1 | **Provisionally recorded**, with the addition that it is a *plan-representation* limit, not a kernel limit |
| **D-7** | Execution authorisation | **NOT AUTHORISED.** See section 18 |
| **D-8** | No wall-clock timestamp or elapsed duration in the receipt | **Provisionally recorded, conditional on B-2** — excluding the duration is only safe once the pidfd is polled and the ordinal fact is *observed*; otherwise the artifact neither records the timing nor observed it |

### New owner decisions this review surfaced

Both change case membership, and neither is the reviewer's to take.

| # | Decision | Recommendation |
|---|---|---|
| **D-9** | Refuse `S_ISUID`/`S_ISGID` objects at admission, or record and permit them? (A-3) | **Refuse in 0.1**, with a new `AdmissionError::SetIdBitsPresent`. The non-elevation claim is load-bearing for the whole non-sandbox section, and 0.1 has no setuid use case. X7's expected outcome flips with this decision |
| **D-10** | Environment `empty`-only for 0.1, or keep `explicit` with the four narrowings? (B-13) | **`empty`-only.** V2 and V5 become plan-rejection cases if taken; both are written to work either way |

## 16. What this review did not do

LAUNCH-EXEC-01 was not executed and no case was posed. `crates/helm-launch` was not created. No
Rust API was frozen — four separate findings would change the public surface, and all depend on
evidence that does not yet exist. No Wine, no 7-Zip, no A0 access, no lab modification, no
privileged operation, no `sudo`. Main was not changed. ADR-0024 was **not** accepted. The three
original commits `ae4591e`, `be0251b` and `a5f09f8` are preserved unchanged; the corrections are
separate commits descending from them, and the findings above were committed **before** the
corrections they justify.

Not verified, and recorded as such: every syscall claim rests on primary documentation and on
kernel source read at review time, **not** on execution — that is what the experiment is for. One
kernel question could not be settled from primary sources at all and is preregistered as an open
observation rather than a prediction: whether a surviving shared writable mapping leaves
`i_writecount` at zero, which is case **E6c**.

## 17. Residual limitations after correction

1. E6c is unpredicted, so one mutation class remains an open kernel question until the trial.
2. M1, M2 and M4 are conditional on a tracer existing on the runner; if none does, the post-fork
   minimality claim is BLOCKED and the aggregate is INCONCLUSIVE, not ACCEPTED.
3. The `LD_` denylist is incomplete by construction, and 0.1 records that rather than fixing it.
4. Under D-1 arm (i) the F-series result transfers to `helm-launch` only if the implementation
   has the same child-setup capability as the C spike; the definition records that explicitly.
5. Nothing here establishes anything about Wine, whose multi-process lifecycle direct-child
   semantics do not describe.

## 18. Recommendation

**NEEDS_ARCHITECTURE_OWNER_REVIEW**, and **D-7 may not yet be authorised.**

The mechanism is not rejected: `execveat` on a retained descriptor is the right primitive, and
after the corrections in this sequence the experiment can falsify what it claims to falsify. But
the definition is not freezable yet, for one bounded reason: **D-9 and D-10 are undecided and
both change case membership** — D-9 flips X7's expected outcome, D-10 converts V2 and V5 from
observation cases into plan-rejection cases. A definition whose membership still depends on an
open decision cannot be frozen, and freezing it afterwards would be the goalpost movement
OBS-FS-01 halted for.

The recommended next decision is therefore a single one: **rule on D-9 and D-10, confirm D-1 at
arm (i) as recorded, and re-read the corrected definition's case table.** With those settled, the
definition can be frozen, `frozen_cases.py` and the section 2 artefacts can be written and
hashed, and D-7 becomes a live question rather than a premature one. ADR-0024 stays **Proposed**
until LAUNCH-EXEC-01 has executed and been reviewed.
