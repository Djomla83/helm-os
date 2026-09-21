# LAUNCH-EXEC-01 — Trial #2 result review

Independent review of the second and only execution authorised under the Trial #2 D-7. The
reviewer did not author the frozen experiment, the dispatcher or the trial workflow. Nothing was
fixed, re-run, re-dispatched or posed; no experimental ELF was executed; no frozen byte was
touched.

**Trial #2 completed. Its frozen aggregate is `MECHANISM_REJECTED`, and that result is sound: it
reproduces exactly, offline, from the preserved evidence and the frozen checker.**

Two things are kept apart throughout, and the reader should keep them apart too:

* **What the frozen trial decided** — determined by the preregistered checker over the preserved
  records. Section D and Section E.
* **Why a case got that result** — engineering diagnosis after the fact. Section F and Section G.

A later finding that a frozen expectation was wrong, or that a harness defect prevented a case
from being posed, does **not** convert a Trial #2 FAIL into a PASS or an INVALID into a valid
observation. Section H states this explicitly.

## A. Execution identity

| | |
|---|---|
| Workflow run | `34640280964` — run number **1**, attempt **1**, `workflow_dispatch` |
| Job | `103397999925` (`trial-002`) → `success` |
| Workflow | id `356056797`, `.github/workflows/launch-exec-01-trial-002.yml` |
| Workflow head | `d8a6887d508be3b4bb5707c40e6433cee21411b5` (`main`) |
| **Experiment bytes executed** | **`ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`** |
| `SOURCE-HASHES.json` SHA-256 | `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94` |
| `SOURCE-HASHES.json` Git blob | `6f000fac9a48625d3c9def18e16ae7ce1b61efa8` |
| Final pretrial review | `5da397c6a79aea7c9a626789480903d50df0b7b4` |
| Owner D-7 authorisation | `900fe5a403eded9f148494f557fb357fea3a207e` |
| Artifact | id `10279663503`, `launch-exec-01-trial-002-evidence`, 96826 bytes |
| Artifact ZIP SHA-256 | `38ee4628eb810ae021257aac78316a6ff8b23f6c6d67c646b4200037e805837c` |
| Started / completed (UTC) | 2026-09-11T19:42:28Z / 2026-09-11T19:44:07Z |
| **Frozen runner exit code** | **`0`** |

`run_number` and `run_attempt` were both confirmed against the Actions API, and workflow
`356056797` has a `total_count` of exactly **1** run, which itself has exactly one attempt
(`attempts/2` returns 404). **No second Trial #2 run or attempt exists.**

The runner's exit code 0 means the frozen runner completed and published its document. It is not
`MECHANISM_ACCEPTED`, and the dispatcher's own last step says so: *"A CI colour is not a verdict;
the durable evidence is."*

### Dispatch and freeze binding — every gate verified from the job log

| Gate | Observed |
|---|---|
| One-shot guard | `event=workflow_dispatch ref=refs/heads/main run_number=1 run_attempt=1` → `one-shot guards passed` |
| Confirmation input | accepted (`RUN-TRIAL-002-ONE-VALID-TRIAL`) |
| Checkout resolved to | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`, tree clean |
| Manifest SHA-256 | `616dc6b3…6c9a94` — matched |
| Manifest blob, file and tree | both `6f000fac…1b61efa8` — matched |
| Final-review binding | commit present, descends from the freeze, carries `TRIAL_2_READY_FOR_NEW_D7_DECISION`, and `git diff` over the experiment directory, the definition, ADR-0024 and the architecture document is empty |
| `--verify-freeze` | `{"freeze_verified": true, "detail": "frozen sources match the manifest"}` |
| Membership | `total 72`, `mandatory 54`, `conditional 11`, `recorded 7` — all ok |
| Traced set | `traced 8 exact` — `E1 E7 F4 F7 M1 M2 M3 M4` |
| Driver completeness | `72 frozen`, `72 handlers`, nothing missing/unknown/duplicated, every setup, parent, rule, check and channel resolves |
| Posability | `72 posable, 0 unposable` |
| Frozen runner | invoked **once**, with `--i-have-owner-authorisation-d7`, exit code `0` |
| Evidence staging | ran **after** the runner step; upload ran after staging |

The trial executed the authorised bytes, not a branch tip. No finding here.

## B. Evidence preservation

Preserved at [`docs/experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/`](../experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/),
byte-for-byte from the published artifact.

**Evidence-preservation commit: `8a9dc7721751d2c94a91f109959b8d6416184843`.**

| File | SHA-256 |
|---|---|
| `preflight.json` | `4669c8490811ef5a3bd0fff812c877ee46e7d7c0a830670456e0bc0b5019a892` |
| `build-identity.json` | `cbf015a64b7b5bf0034fb6e638bbfdaaccbc9110a95236ea58f06efac45bb21b` |
| `journal.jsonl` | `451e471d68ab8bb6509eca3974ab54bcfa08b89ecf37bd99f7541dfd664a6558` |
| `evidence.json` | `c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f` |
| `runner-stdout.json` | `c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f` |

**`evidence.json` and `runner-stdout.json` are byte-identical: YES.** `cmp` reports no difference,
and after staging both resolve to the *same Git blob* `75648268fe7dd30f4beb73905061e7892b8dd630`.
They were produced independently — one is the file the runner wrote to its output directory, the
other is the runner's stdout captured by the workflow — so their agreement is a cross-check that
the publication boundary emitted one document and only one.

Every digest above was recomputed locally from the downloaded bytes; none was taken on trust. The
ZIP digest computed here matches both the `upload-artifact` step's reported digest and the Actions
API `digest` field. The artifact contains exactly these five files: no ZIP, no raw `strace`, no
runner stderr, no build intermediates, no PIDs and no descriptor numbers.

`.gitattributes` pins the five files as byte-exact, following this repository's existing rule for
preserved captures. `PROVENANCE.md` in that directory records the provenance and states that it is
not itself Trial #2 evidence.

## C. Journal

| Event | Count |
|---|---|
| `trial_begin` | 1 |
| `preflight` | 1 |
| `build_identity` | 1 |
| `case_entered` | 72 |
| `case_pose_started` | **68** |
| `case_completed` | 72 |
| `trial_end` | 1 |
| **Total records** | **216** |

* Sequence `n` runs `0 … 215` with no gap and no repeat — strictly monotonic.
* Every record carries `trial: "trial-002"`; there are no records of any other kind.
* 72 distinct cases entered, 72 distinct completed, and the entered set equals the completed set.
  No case was entered and left incomplete; no case was completed twice; nothing disappeared.
* The frozen reader (`journal.replay`) accepts the file with **no tail truncation** and raises none
  of its ordering contradictions.
* `trial_end` is present exactly once and its `aggregate`, `counts`, `detail` and `status` all
  agree with `evidence.json`. `halts` is `null`.
* Partial-journal semantics do **not** apply: `journal.project_status` returns `TRIAL_COMPLETED`,
  not `TRIAL_ABORTED_AFTER_BOUNDARY`.

### The four cases with no `case_pose_started`

Read from the journal itself, not assumed: **`E4`, `E6`, `E6c`, `N3`.**

Each is legitimate. `E4`, `E6` and `E6c` were refused by their own setup before any launcher was
started (`not_posed`, with `invocation.mechanism_invoked: false`). `N3` was blocked at preflight on
its frozen cause. The remaining 68 cases each have exactly one durable `case_pose_started`.

### D-7 consumption

`journal.Journal._append` writes each line, then `flush()`, then `os.fsync()` before returning, and
`case_pose_started` is written *before* the launcher is started. The frozen implementation is
explicit that this record is the immutability boundary and the moment D-7 is consumed, and that the
bias is deliberate: a process that dies between the record and the actual launch still counts as
having consumed its authorisation.

The preserved journal's **first durable `case_pose_started` is `E1`** at `n = 4` — independently
confirmed from the file, not taken from the instruction. `build_identity` is at `n = 2`, so build
identity was durable *before* the boundary was crossed.

**D-7 consumed: YES. `TRIAL_2_D7_CONSUMED`.**

**Valid Trial #2 count: ONE.** The preserved journal is one complete trial: every membership case
has a durable completed record, no case has two, and `trial_end` carries an aggregate the frozen
checker produced from exactly those records.

## D. Frozen result

| Status | Count |
|---|---|
| PASS | **59** |
| FAIL | **6** |
| INVALID | **6** |
| BLOCKED | **1** |
| **Total** | **72** |

| Class | Cases |
|---|---|
| FAIL | `X2b`, `X2c`, `X4`, `T1`, `S4`, `M2` |
| INVALID | `E4`, `E6`, `E6c`, `O6`, `O7`, `R3` |
| BLOCKED | `N3` |

**Aggregate: `MECHANISM_REJECTED`.**

`uncontrolled` and `notes` are both empty. `status` is `RUN`.

### Aggregate validity

The frozen precedence in `checker.verdict` is ordered, total and disjoint:

1. any `FAIL` among mandatory + conditional + recorded → `MECHANISM_REJECTED`;
2. any mandatory `INVALID`/`BLOCKED`, or any conditional/recorded `INVALID` → `MECHANISM_INCONCLUSIVE`;
3. otherwise → `MECHANISM_ACCEPTED`.

Verified against the preserved statuses:

* Rule 1 fires on exactly `['X2b', 'X2c', 'X4', 'T1', 'S4', 'M2']`. **Six FAILs independently
  suffice**; any one of them alone would produce the same aggregate.
* No `INVALID` was converted to `FAIL`. Every one of the six carries `not_posed` and no `outcome`,
  and `score_case` tests invalidity *first*, before any outcome expectation is considered.
* `N3` is conditional, its recorded cause `unprivileged_runner` is exactly its frozen `blocked_if`,
  and a conditional `BLOCKED` with its recorded cause fires neither rule 1 nor rule 2. **N3 did not
  cause `MECHANISM_REJECTED`.**
* No status was silently ignored: `score_all` iterates the full frozen membership, all 72 cases
  received a status, and every status is one of the four frozen values.

**Counterfactual, for calibration only:** had all six FAILs instead PASSed, the aggregate would have
been `MECHANISM_INCONCLUSIVE` on the six unposed cases — **not** `MECHANISM_ACCEPTED`.
`MECHANISM_ACCEPTED` was not reachable in this trial.

**`TRIAL_2_FROZEN_RESULT_SOUND`**

## E. Offline reproduction

**Method.** The frozen checker, case, journal and evidence modules were imported from a
`git archive` extraction of `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` into a scratch directory.
The preserved `journal.jsonl` was read with the frozen `journal.read`, replayed with
`journal.replay`, and the per-case records recovered with `journal.recovered_records` were scored
with `checker.report`. Nothing else ran: no launcher, no helper ELF, no case posed, no syscall
experiment, and no repository source was modified to make replay convenient.

**Frozen checker used.** `checker.py` SHA-256
`6201a2dcaa1be1b48667b1d0ceedfbd2d6619668fb78a1043b408ebfd7a7f9b0` — identical to the value
`evidence.json` itself records under `freeze.sha256`. The same holds for every other module used:
`frozen_cases.py` `ee4b9599…4cbd7`, `journal.py` `63931640…64d6`, `evidence.py` `a91e02a9…1893a`,
`observations.py` `4626814e…1399c`, `driver.py` `844db6e3…701d`. No later checker was used.

**Result.**

| Compared | Reproduced |
|---|---|
| Aggregate | `MECHANISM_REJECTED` — match |
| Counts | `{PASS: 59, FAIL: 6, INVALID: 6, BLOCKED: 1}` — match |
| `detail.failing_cases` | `['X2b','X2c','X4','T1','S4','M2']` — match, same order |
| All 72 case statuses | match |
| All 72 case **reason strings** | match, character for character |

**Exact match: YES.** The published result is not merely plausible; it is recomputable.

The journal's own `trial_end` record independently carries the same aggregate, counts, detail and
status as `evidence.json`, so the two preserved documents corroborate each other.

## F. Non-PASS analysis

Reviewed deeply: the six FAILs, the six INVALIDs and the one BLOCKED. PASS cases were consulted only
where needed to interpret one of these thirteen. No unrelated backlog was reopened.

### Summary table

| Case | Frozen status | Observed fact | Frozen reason | Primary cause class | Confidence | Next action |
|---|---|---|---|---|---|---|
| `X2b` | FAIL | `ExecFailed:EACCES` | not the single frozen prediction `ExecFailed:ENOENT` | FROZEN_EXPECTATION_OR_SPEC_DEFECT | HIGH | Make `make_fixtures` fixtures executable; re-derive X2b/X2c/X4 predictions |
| `X2c` | FAIL | `ExecFailed:EACCES` | not `interpreter_ran_with_devfd` | FROZEN_EXPECTATION_OR_SPEC_DEFECT | HIGH | As `X2b` |
| `X4` | FAIL | `ExecFailed:EACCES` | not `ExecFailed:ENOEXEC` | FROZEN_EXPECTATION_OR_SPEC_DEFECT | HIGH | As `X2b` |
| `T1` | FAIL | `TimedOut:KilledByLauncher:SIGTERM` | not the bare `TimedOut` | FROZEN_EXPECTATION_OR_SPEC_DEFECT | HIGH | Correct T1's frozen prediction to the qualified token its siblings already use |
| `S4` | FAIL | `ExecStatusIndeterminate` | not `Exited:7` | FROZEN_EXPECTATION_OR_SPEC_DEFECT | HIGH | Give S4 a positive exec-evidence channel, or restate what S4 is allowed to conclude |
| `M2` | FAIL | `differs_from_single_threaded_arm` (published opaque) | not `identical_to_single_threaded_arm` | HARNESS_OR_OBSERVATION_DEFECT | HIGH | Anchor the traced child window on the `CLONE_PIDFD` clone3, not any clone3 |
| `E4` | INVALID | binding `bound: false`, `object_sha256: null`, `SYMLINK_TO_BASE` | build identity: the starting bytes are not the artefact this trial hashed | HARNESS_OR_OBSERVATION_DEFECT | HIGH | Resolve the build directory to an absolute path before any fixture is built |
| `E6` | INVALID | setup refused before any launch | the guarded marker region is not uniquely locatable in the built `helper_report` image | HARNESS_OR_OBSERVATION_DEFECT | MEDIUM | Make the guard+marker+guard run one object; report which locator condition failed |
| `E6c` | INVALID | setup refused before any launch | same reason as `E6` | HARNESS_OR_OBSERVATION_DEFECT | MEDIUM | As `E6` (identical root cause) |
| `O6` | INVALID | `fixture_descendant_signalled: false` while arming was correct | the forced state did not materialise | HARNESS_OR_OBSERVATION_DEFECT | HIGH | Resolve the build directory to an absolute path; make a failed FIFO open observable |
| `O7` | INVALID | `fixture_descendant_signalled: false` while arming was correct | same reason as `O6` | HARNESS_OR_OBSERVATION_DEFECT | HIGH | As `O6` (identical root cause) |
| `R3` | INVALID | termination signal not renderable | observation not interpretable: termination signal outside the frozen signal table | HARNESS_OR_OBSERVATION_DEFECT | MEDIUM | Stop discarding `si_status` when `si_code != CLD_KILLED`; record `si_code` |
| `N3` | BLOCKED | `geteuid: 1001`, no privileged identity | no privileged identity exists to attempt a real privilege transition, and none is manufactured | ENVIRONMENT_BLOCK | HIGH | None. Correct and expected |

### F.1 `X2b` / `X2c` / `X4` — a demonstrated common cause

**Frozen class.** All three mandatory. **Frozen predictions:** `X2b` `ExecFailed:ENOENT`, `X2c`
`interpreter_ran_with_devfd`, `X4` `ExecFailed:ENOEXEC`. Each is a single prediction, not a safe set.

**Setup and posing.** `X2b`/`X2c` use `script_fixture.sh`; `X4` uses `unloadable_in_cohort.elf`. All
three pass `--bypass-admission` so that `execveat` is actually reached; `X2c` additionally passes
`--exec-fd-no-cloexec` so the interpreter would receive `/dev/fd/N`. All three were genuinely posed:
`mechanism_invoked: true`, `pose_started: true`, and each object's starting bytes bound to the
trial's own build identity as `DIRECT_BASE`.

**What was observed.** All three: `ExecFailed:EACCES`, `elapsed_ms: 0`, reason *"the child wrote an
explicit pre-exec status record"* — that is, the pre-exec stub reported `EACCES` from `execveat`
before any image ran.

**Evidence sufficiency.** Sufficient. The errno is an explicit record written by the child through
the exec-status channel, not an inference.

**Cause.** Both fixtures are produced by `make_fixtures.write()`, which is
`pathlib.Path.write_bytes(data)` and nothing else. `write_bytes` creates a file with mode `0o666`
masked by the umask — **a mode that contains no execute bit under any umask, because a umask can
only clear bits.** Nothing chmods these files afterwards: `harness.build()` calls
`make_fixtures.write(build)` and returns, and the two setups (`_setup_script`,
`_setup_unloadable`) return the fixture path unchanged.

That the frozen code knows how to do this where it matters makes the omission plain: `_setup_noexec_copy`
explicitly does `os.chmod(target, 0o755)`, `_setup_never_executable` explicitly does
`os.chmod(..., 0o644)`, and the compiled helpers arrive executable from `cc -o`.

**The decisive control is inside this very trial.** `X3` uses `_setup_never_executable`, which
chmods its object to `0o644`, does **not** bypass admission, and **PASSed with the frozen
prediction `ExecFailed:EACCES`**. The launcher's admission check tests `S_ISREG`, the set-id bits
and the ELF cohort header — it does **not** test the execute bit — so `X3` reaches `execveat` with a
non-executable file and the kernel answers `EACCES`. `X8` likewise PASSed on a frozen `EACCES`
prediction from a `noexec` mount. So "a non-executable object reaching `execveat` in this build
directory yields `EACCES`" is not a hypothesis here; it is an observed, frozen-predicted,
in-trial fact.

Filesystem-level alternatives are excluded by the same trial: 59 cases executed helper images from
that same build directory, so it is neither `noexec` nor missing search permission.

*External documentation, kept separate from Trial #2 evidence:* Linux
[`execve(2)`](https://man7.org/linux/man-pages/man2/execve.2.html) lists `EACCES` for
"Execute permission is denied for the file", `ENOEXEC` for an unrecognised format, and `ENOENT`
for a missing script or ELF interpreter; Linux
[`execveat(2)`](https://man7.org/linux/man-pages/man2/execveat.2.html) documents the
`O_CLOEXEC`/script `ENOENT` case the frozen X2b expectation targets. The in-trial controls and
frozen source establish that the missing execute permission prevented the three deeper behaviours
from being reached. **No reproduction executable was run for this review.**

**Conclusion.** A single, demonstrated common cause: the `make_fixtures` fixtures are never made
executable. Under it, all three frozen predictions were unsatisfiable as written, which is why the
cause class is `FROZEN_EXPECTATION_OR_SPEC_DEFECT` — the frozen material predicted an outcome its
own fixture construction forbade. A secondary contributing factor is that `--bypass-admission`
skipped the launcher's own object checks, so nothing upstream reported the unusable fixture.

**These three FAILs say nothing about `execveat` shebang handling, `/dev/fd` interpreter behaviour
or ENOEXEC loading. Trial #2 produced no evidence on any of those questions.**

### F.2 `T1`

**Frozen class.** Mandatory. **Frozen prediction:** the bare token `TimedOut`.

**Setup and posing.** `helper_report --sleep-ms 60000`, no spike flags, total bound 11000 ms.
Genuinely posed; `DIRECT_BASE` binding held.

**What was observed.** `TimedOut:KilledByLauncher:SIGTERM` at `elapsed_ms: 2006`, reason *"the
deadline expired with the child still running"*.

**Evidence sufficiency.** Sufficient. The reason string is produced only after the observation layer
has already established positive exec evidence, so the child demonstrably ran and was still running
at the deadline.

**Cause.** This is a frozen expectation defect, and it is provable rather than arguable.

`observations._timed_out` renders the bare token `TimedOut` **only** when the receipt's
`timeout_disposition` is the empty string. In `launcher_spike.c`, `timeout_disposition` is
initialised to `""` and is assigned in exactly one place — inside the `else if (timed_out)` branch
that also sets `disposition = "TimedOut"` — where all three arms assign a non-empty value
(`TerminationFailed`, `ExitedDuringGrace` or `KilledByLauncher`). Therefore whenever the disposition
is `TimedOut`, the sub-disposition is never empty, and **the bare token `TimedOut` is unreachable
from this launcher.** T1 predicted a token the mechanism cannot emit.

The mechanism behaved exactly as designed: at the deadline it sent `SIGTERM` via `pidfd_send_signal`,
opened the grace window, the child died of that signal, `waitid` reported a non-`CLD_EXITED` status,
and the receipt recorded `KilledByLauncher` with `SIGTERM`.

Its own siblings confirm the vocabulary: `T2` and `T6` predict `TimedOut:KilledByLauncher:SIGKILL`
(fully qualified) and `T2` PASSed with exactly that; `T3` predicts
`TimedOut:ExitedDuringGrace:9`. T1 alone was written without the qualifier.

**Do not collapse the two strings.** `TimedOut` and `TimedOut:KilledByLauncher:SIGTERM` are distinct
tokens in a frozen closed vocabulary, the checker compares them for equality, and the FAIL is
correct as a frozen result. The engineering finding is that T1's prediction was wrong, not that the
launcher misbehaved.

### F.3 `S4`

**Frozen class.** Mandatory, `instant_reject: true`. **Frozen prediction:** `Exited:7`.

**Setup and posing.** `helper_report --exit-immediately 7` under `--post-fork-delay-ms 200`, so the
parent sleeps 200 ms after `clone3` before it polls and the child has certainly execed and exited
first. Declared channels: **`receipt` only** — no `report` channel. Genuinely posed.

**What was observed.** `ExecStatusIndeterminate` at `elapsed_ms: 204`, reason *"clean exec-status EOF
with no positive evidence that the pinned image ever ran; EOF alone never means exec"*.

**Evidence sufficiency.** Sufficient for the status, and it also settles more than it appears to.
That exact reason string is produced at one place only — `rule_process_disposition`'s
`confirmation != EXEC_REACHED` branch — which is reached **after** the branches for `ExecFailed`,
`ExecStatusIndeterminate` and `ExitStatusUnobservable` have already returned. So the launcher's own
recorded disposition was **not** `ExecStatusIndeterminate`. Since the launcher emits that
disposition whenever `exec_confirmed` is false, it follows that **the launcher did establish
`exec_confirmed`**: it observed a clean, record-free EOF on the exec-status pipe, which is what a
successful `execveat` produces through the `O_CLOEXEC` status descriptor. With `elapsed_ms: 204`
against a 200 ms forced delay, and a helper whose first action is `_exit(7)`, the disposition the
launcher recorded was `Exited`.

**Did the mechanism lose an observation it was supposed to retain? No.** The launcher retained the
exit status in its receipt. What happened is that the frozen observation rule — described in its own
source as *"the S5-vs-S1 rule and the single most load-bearing line in this module"* — refuses to
render any `Exited:`, `Signaled:` or `TimedOut` token without positive evidence that the pinned
image ran, because a clean EOF is equally consistent with the pre-exec stub dying silently. That
rule is epistemically correct and was preregistered.

**Cause.** S4's own frozen design withheld the only evidence its own rule accepts. `helper_report.c`
handles `--exit-immediately` as the very first observable action — its comment names R3 and S4
explicitly: *"no report, no output and no setup can precede it"* — and S4 declares no `report`
channel. So `exec_confirmation` could never reach `EXEC_REACHED`, and `Exited:7` was unreachable by
construction. Compare `S1`, whose frozen note already states the principle — *"The report is the
exec evidence; clean EOF alone is not a PASS"* — and which PASSed because its report was present.

A real consequence worth recording: **`S4` and `S5` are indistinguishable under the frozen
vocabulary.** `S5` (die between the last setup stage and `execveat`) predicts
`ExecStatusIndeterminate` and PASSed; `S4` produced the same token. As written, S4 cannot
discriminate "the image ran and exited 7" from "the child died before exec". Whether that is
acceptable — whether the launch mechanism *should* be able to distinguish them without cooperation
from the executed image — is an engineering question for the owner (Section I), not something this
review decides.

*What the preserved public evidence does not contain:* the receipt itself, and therefore the
recorded exit code. That the child exited 7 specifically is a strong inference from the helper's
source and the timing, not a preserved observation.

### F.4 `M2`

**Frozen class.** Conditional (`blocked_if: no_tracer`), traced. **Frozen prediction:**
`identical_to_single_threaded_arm` — exact list equality between the threaded arm's child syscall
window and the single-threaded control arm's.

**Setup and posing.** `helper_report --exit 0` with `--extra-threads 3`; `posed_when:
threaded_parent_observed` **held** (`declared_launcher_threads: 3`, `extra_threads: {control_arm: 0,
threaded_arm: 3}`), and the control arm ran and returned. The case was properly posed and the
comparison really was made against a control arm collected under the same plan.

**What was observed.** The published outcome token is sanitised to `<OPAQUE:661a45e4>`; the frozen
reason is *"the child sequence differed from the single-threaded arm"*, so the rule returned
`differs_from_single_threaded_arm`.

**Evidence sufficiency.** Sufficient for the status — and, unusually, sufficient to diagnose the
cause, because the preserved trace is decisive on its own.

**Cause — a trace acquisition defect, not a launch-semantic divergence.** The seven other traced
cases (`E1`, `E7`, `F4`, `F7`, `M1`, `M3`, `M4`) each recorded the *identical* canonical launcher
child window:

```
dup2 dup2 dup2 fcntl fcntl fcntl fchdir close_range close_range setpgid
rt_sigprocmask rt_sigaction×59 prctl execveat
```

with `stage_sequence` = the full frozen nine (`DUP2 … EXEC`), `clone3_call_count: 1` and
`clone3_flags: ['CLONE_PIDFD']`.

`M2` alone recorded:

```
rseq set_robust_list rt_sigprocmask mmap munmap munmap mprotect rt_sigprocmask madvise exit
```

with `stage_sequence: ['SIGMASK']`, `clone3_call_count: 4`, `clone_pidfd_flag: false`, and
`clone3_flags: [CLONE_VM, CLONE_FS, CLONE_FILES, CLONE_SIGHAND, CLONE_THREAD, CLONE_SYSVSEM,
CLONE_SETTLS, CLONE_PARENT_SETTID, CLONE_CHILD_CLEARTID]`.

That flag set is `pthread_create`, not process creation. The window contains **no** `dup2`, `fchdir`,
`close_range`, `setpgid`, `prctl` or `execveat` — none of the launcher's child-setup path — and it
ends in **`exit`**, the thread-exit syscall, where every other traced case ends in `execveat`, which
the frozen child path documents as one of its only two exits.

With `--extra-threads 3` the launcher makes four `clone3` calls: three `pthread_create` plus the one
`CLONE_PIDFD` process clone. The trace parser anchored the "child window" on one of the **thread**
clones. So M2 compared a launcher worker thread's glibc startup and stack teardown against the
control arm's launched-child window. Those were never going to be equal, whatever the launch
semantics are.

`trace_integrity` reports zero malformed, orphaned, ambiguous or unmatched fragments, so the trace
text was fine; the defect is in *window selection*, not acquisition quality.

**Conclusion.** `HARNESS_OR_OBSERVATION_DEFECT`, HIGH confidence. A secondary factor worth noting
separately: the frozen rule demands exact list equality, which would remain brittle even with
correct window selection — but that is not what failed here.

**The consequence matters more than the case.** M2 exists because *"without this the mechanism is
evidenced only for a single-threaded parent, which no real HELM caller is."* Its FAIL is a frozen
result, but it carries **no information about multithreaded-parent launch semantics**. Trial #2
therefore produced no valid evidence on the very question M2 was written to answer.

*What the preserved evidence does not contain:* the control arm's own `child_syscalls` list, so the
element-wise difference cannot be displayed. It does not need to be: the threaded arm's window is
not a child-process window at all.

### F.5 `E4`

**Frozen class.** Mandatory, `instant_reject: true`. **Frozen prediction:** `Exited:0`, with
assertions `executed_marker` and `measured_starting_identity`.

**Setup and posing.** `symlink_retarget`: a symlink is created at `<build>/E4_link` pointing at
`helper_report`, pinned, and retargeted to a private copy of `helper_alt` after the pin. The case
was **not posed**: `mechanism_invoked: false`, `pose_started: false`, and it is one of the four cases
with no `case_pose_started` in the journal.

**What was observed.** `build_identity_binding`: `classification: SYMLINK_TO_BASE`,
`base_artefact: helper_report`, `base_sha256: 423ac81e…`, `object: E4_link`, **`object_sha256:
null`**, `bound: false`, detail *"the starting bytes are not the artefact this trial hashed"*.

**Evidence sufficiency.** Sufficient, and the `null` digest is the tell.

**Cause — the symlink was dangling, and the reason is a relative path.** `_verified_source` returns
`ctx.build / name`, and `TrialContext.__init__` stores `pathlib.Path(build)` **without resolving
it**. The trial was launched by the dispatcher with `--build-dir target/launch-exec-01`, a
**relative** path (visible in the job log's `BUILD_DIR` environment). So `base` is the relative
string `target/launch-exec-01/helper_report`, and `_setup_symlink_retarget` calls
`link.symlink_to(base)` on a link that itself lives at `target/launch-exec-01/E4_link`.

A relative symlink target resolves against the directory containing the link, so `E4_link` points at
`target/launch-exec-01/target/launch-exec-01/helper_report`, which does not exist. `bind_build_identity`
then calls `os.path.realpath`, which does not raise on a dangling link and returns a path whose
basename is still `helper_report`, so the name check passes; `_digest` then fails to read the file
and returns `None`; `bound` becomes false. That reproduces the preserved record exactly, including
`object_sha256: null`.

**Is the INVALID correctly assigned? Yes.** The binder did precisely what it should: it refused to
let a case run against bytes the trial could not name, and it refused *before* the boundary. The
identity binding is not the defect.

**What must change before a future trial could pose the intended condition:** resolve the build
directory to an absolute path — `pathlib.Path(build).resolve()` in `TrialContext`, or resolution in
the runner before the context is constructed — or have `_setup_symlink_retarget` create an absolute
symlink target. **This weakens no identity binding whatsoever**; it removes an ambiguity in path
resolution that the binding then evaluates correctly. Separately, a dangling symlink should be
reported as such rather than reaching the digest step, so a future failure names its own cause.

### F.6 `E6` / `E6c`

**Frozen class.** `E6` mandatory (`instant_reject: true`, prediction `Exited:0`, documentation gate);
`E6c` recorded (safe set `{ExecFailed:ETXTBSY, Exited:mutated}`, gate
`no_claim_that_measured_bytes_ran`). **Both not posed**, both with `mechanism_invoked: false`, both
among the four cases with no `case_pose_started`.

**Setup and posing.** `E6` mutates the marker region in place, length-preserving; `E6c` writes
through a shared writable mapping that survives the descriptor close. Both call the same
`find_marker_region` on their own copy of `helper_report`, and both refused with the identical
reason.

**Evidence sufficiency.** Sufficient for the INVALID; **not** sufficient to identify which of three
distinct conditions fired.

**Cause.** `helper_report.c` declares three separate file-scope objects:

```c
volatile char g_marker_guard_lo[9] = "HELM-MARK";
volatile char g_marker[16]         = "helper_report\0\0\0";
volatile char g_marker_guard_hi[9] = "KRAM-MLEH";
```

and `find_marker_region` requires that the built image contain `HELM-MARK`, that `KRAM-MLEH` appear
exactly 16 bytes after it, and that `HELM-MARK` occur only once. It returns `None` — with the same
message — for **all three** failures, so the preserved reason does not say which occurred.

The fixture assumption is the weak point: the three declarations are three distinct objects, and
the frozen source supplies no mechanism that guarantees their adjacency or ordering. The compiler
and linker may place or pad them independently; the
[C11 committee draft](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf) does not grant
three separately declared objects the one contiguous representation that the locator assumes. The
source comment asserts the guards "make the offset locatable by an unambiguous byte search", but
nothing in the declaration or build enforces the layout that claim depends on.

**Do `E6` and `E6c` share exactly the same root cause? Yes** — the same function, applied to copies
of the same built image, returning `None` in both.

**Smallest future correction that does not weaken the evidence standard:** put the guard, marker and
guard in **one** object whose layout the language guarantees — a single packed struct, or one
`char[34]` array initialised with the full `HELM-MARK` + 16 bytes + `KRAM-MLEH` pattern — so
adjacency is a property of the declaration rather than a hope about the linker. Independently,
`find_marker_region` should distinguish and report its three failure conditions, so a future refusal
names its own cause instead of leaving this question open. Neither change touches what E6 proves.

*Confidence is MEDIUM, not HIGH:* the built `helper_report` bytes are not part of the preserved
evidence, so which condition fired cannot be settled from Trial #2 alone. A bounded offline
diagnostic — rebuild `helper_report` at the freeze and run `find_marker_region` against it, posing
no case — would settle it without executing anything.

### F.7 `O6` / `O7`

**Frozen class.** Both mandatory; `O6` `instant_reject: true`. **Frozen predictions:** `O6`
`Exited:0` with completeness `WriterRetainedAfterChildExit`; `O7` `Exited:42` with a simultaneous
stderr capture failure. **Both `posed_when: fixture_descendant_signalled`.**

**Setup and posing.** `fork_helper_prearmed`: the harness creates a case-private FIFO, opens its read
end **before** the launcher is spawned, and passes the FIFO path to `helper_fork` so its descendant
can signal that it reached the signalling path before the launcher's group sweep ends it.

**What was observed.** The arming itself was flawless and is recorded as such:
`armed_before_launch: true`, `fresh_fifo: true`, `reader_inheritable: false`,
`reader_passed_to_launcher: false`. But `fixture_descendant_signalled: false`, so
`posed_check.held: false` and neither case was posed. Both returned at `elapsed_ms: 2007`.

**Evidence sufficiency.** Sufficient for the INVALID, and — combined with the frozen source —
sufficient to identify the cause.

**Cause — the same relative-path defect as `E4`, on a different surface.** The full chain:

1. `_setup_fork_helper_prearmed` computes `fifo = ctx.build / (case + ".fixture-signal")`. Because
   `TrialContext` never resolves the build directory and the dispatcher passed
   `--build-dir target/launch-exec-01`, this is the **relative** string
   `target/launch-exec-01/O6.fixture-signal`.
2. It returns `extra_helper_args = ("--liveness-fifo", <that relative path>)`.
3. `_arm_fixture_signal` recreates the node at that path and opens the read end — correct, and from
   the harness's own working directory it resolves fine. This is why every arming fact is true.
4. `spike_argv` appends `--arg --liveness-fifo --arg <relative path>`, so the launcher passes them
   through to the child's argv. That part works.
5. **`launcher_spike.c`'s child calls `fchdir(p->dir_fd)` unconditionally** at the `ST_CHDIR` stage,
   before `execveat`. The work directory is `built["work_dir"] or ctx.work`, and the runner
   constructs the context with `work=build_dir`. So the executed `helper_fork` starts with its
   current directory set to **`<repo>/target/launch-exec-01`**.
6. `helper_fork`'s descendant then calls `open("target/launch-exec-01/O6.fixture-signal",
   O_WRONLY|O_CLOEXEC)` relative to that directory — i.e. against
   `<repo>/target/launch-exec-01/target/launch-exec-01/…`, which does not exist.
7. `helper_fork.c` guards the write with `if (f >= 0)`. **A failed open is silent.** No byte is
   written, nothing is reported, and the descendant proceeds to its sleep as if nothing happened.
8. The harness's bounded read finds an empty FIFO and records
   `fixture_descendant_signalled: false`.

**This is exactly the mismatch the review was asked to look for.** The pretrial Python stand-in that
validated the pre-armed design ran in the harness's own working directory, where the relative path
resolves correctly. Only the real C helper — reached through the launcher's mandatory working-directory
capability, which is itself decision D-5 — ever sees a different current directory. A design proof
that never crossed `fchdir` could not have caught this.

**Do `O6` and `O7` share the same root cause? Yes**, identically; nothing distinguishes them here.

**A consequence that reaches beyond `O6`/`O7`.** The P-series uses the same
`--liveness-fifo` argument through the same path. `P1`, `P2` and `P4` PASSed on outcomes
(`descendant_died`, `descendant_alive_after_launch: false`) that rest on the **absence** of the
signal byte. This defect makes that absence uninformative: the byte would have been absent whether
the descendant died or not. **Their frozen PASS statuses are historical and unchanged** — Section H
— but the launch-containment conclusions drawn from them should not be relied on until the path
defect is corrected. This is recorded as an IMPORTANT finding, not a status change.

**Fix direction (not applied):** resolve the build and work directories to absolute paths, and make
`helper_fork` report a failed FIFO open instead of continuing silently, so an unposable fixture is
distinguishable from a dead descendant.

### F.8 `R3`

**Frozen class.** Mandatory. **Frozen prediction:** `Signaled:SIGSEGV`.

**Setup and posing.** `helper_report --rlimit-core-zero --raise-segv`. The helper sets `RLIMIT_CORE`
to 0 first — the frozen note explains why: *"systemd-coredump is installed on the runner"* — and
then calls `raise(SIGSEGV)`. Genuinely posed: `mechanism_invoked: true`, `pose_started: true`,
`launch_returned: true`, `elapsed_ms: 116`.

**What was observed.** `not_posed: "observation not interpretable: termination signal outside the
frozen signal table"`. That reason originates at exactly one place in the frozen source: the
`Signaled` branch of `rule_process_disposition`, where `_signaled` returned `None` because
`SIGNAL_NAMES.get(term_signal)` was `None`.

**Evidence sufficiency.** Sufficient to prove an observation defect; **not** sufficient to name the
signal, because the receipt is not part of the preserved public evidence.

**Which signal was actually observed?** The sanitised evidence does not permit reading it. But the
frozen source constrains it sharply. `SIGNAL_NAMES` covers 1–31 and includes `SIGSEGV` (11). And
the launcher's receipt emits:

```c
"exit_code":%d,"term_signal":%d,
    info.si_code == CLD_EXITED ? info.si_status : -1,
    info.si_code == CLD_KILLED ? info.si_status : -1,
```

**`term_signal` is the real signal number only when `si_code == CLD_KILLED`; for any other
signal-bearing `si_code` the launcher writes `-1` and throws the observed number away.** Reaching
the `Signaled` branch at all requires `si_code != CLD_EXITED`. So the value the table could not name
was `-1`, and therefore `si_code` was not `CLD_KILLED` — leaving `CLD_DUMPED` as the realistic
value, i.e. the kernel flagged a core dump for this termination despite `RLIMIT_CORE = 0`. Linux
[`waitid(2)`](https://man7.org/linux/man-pages/man2/wait.2.html) specifies that `si_status` carries
the terminating signal for both `CLD_KILLED` and `CLD_DUMPED`; the frozen receipt retains it only
for the former.

**Cause.** `HARNESS_OR_OBSERVATION_DEFECT`: the launcher observed a perfectly good signal number in
`info.si_status` and discarded it because the dump flag was set, leaving the frozen normalisation
table with an uninterpretable `-1`. The signal table is not missing `SIGSEGV`; the receipt never
delivered it.

**Was the case honestly marked INVALID? Yes.** The frozen rule returned no token rather than
guessing one, and the driver recorded that the observation was not interpretable instead of
inventing a disposition. That is the correct behaviour and it is exactly what "fail closed" should
look like.

**Confidence: MEDIUM.** The chain above is tight, but it rests on excluding a real-time signal
(32–64) as `si_status`, which the preserved evidence cannot do. Under that alternative the cause
would instead be a missing vocabulary entry, i.e. a frozen-spec defect. Either way the case was
honestly INVALID; the cause class differs, so this review does not force certainty.

**Next action (not applied):** record `si_code` alongside the signal number and emit `si_status`
whenever the child was signalled, not only under `CLD_KILLED`. **Do not simply add a signal to the
table** — the table was not the failure.

### F.9 `N3`

**Frozen class.** Conditional, `blocked_if: unprivileged_runner`. **Frozen prediction:**
`privilege_transition_suppressed`.

Every §22 condition verified against the preserved evidence:

| Requirement | Observed |
|---|---|
| `N3` is conditional | `cls: conditional` in the frozen manifest |
| This exact block cause was frozen | `blocked_if: 'unprivileged_runner'`; record carries `blocked: 'unprivileged_runner'` |
| No privileged identity existed | `preflight.geteuid: 1001`; `unprivileged_runner` is the **only** entry in `block_reasons` |
| No synthetic privileged identity manufactured | the `privileged_setid` setup was never reached |
| No mechanism invocation occurred | `mechanism_invoked: false`, `pose_started: false`, and no `case_pose_started` for `N3` in the journal |
| BLOCKED allowed by the aggregate algebra | `score_case` permits BLOCKED only for a conditional case whose recorded cause equals its frozen `blocked_if` — both hold |
| `N3` did not cause `MECHANISM_REJECTED` | rule 1 reads FAILs only; `N3` is not in the rule 2 set either |

**Primary cause class: `ENVIRONMENT_BLOCK`, HIGH confidence.** The preserved evidence contradicts
nothing. N3 behaved exactly as the frozen manifest intended, and the privileged transition remains
**UNTESTED**, resting on primary-source kernel semantics rather than on any demonstration.

## G. Cluster analysis

Three shared causes are demonstrated, and one of them spans two clusters the instruction treated
separately.

### G.1 Relative build directory — `E4`, `O6`, `O7` (and the P-series' negative evidence)

**The single largest finding of this review.** `run_launch_exec_01.py` defaults `--build-dir` to the
relative `target/launch-exec-01`, the dispatcher passed exactly that relative value, the runner
constructs `TrialContext(build=build_dir, work=build_dir)`, and `TrialContext.__init__` stores
`pathlib.Path(...)` **without resolving it**. Every path derived from the build directory is
therefore relative, and everything downstream is correct only while the current directory stays the
harness's own.

Two independent surfaces break, and both are in this trial's results:

* **`E4`** — a relative symlink target created *inside* the build directory self-nests, producing a
  dangling link and an unbindable identity.
* **`O6`, `O7`** — a relative FIFO pathname is handed to a child that the launcher `fchdir`s into
  the build directory before `execveat`, so the descendant's `open()` resolves against the wrong
  directory and fails silently.

A third, non-scoring consequence: the P-series' liveness signal travels the same way, so `P1`, `P2`
and `P4` PASSed on the absence of a byte that could not have arrived. Their statuses stand; the
containment conclusions drawn from them do not, until this is corrected.

**One correction addresses all of it:** resolve the build and work directories to absolute paths
before any fixture is constructed. It weakens no gate, no binding and no evidence standard.

### G.2 Non-executable generated fixtures — `X2b`, `X2c`, `X4`

`make_fixtures.write()` creates every non-compiled fixture with `write_bytes`, which cannot set an
execute bit under any umask, and nothing chmods them. Every case that reaches `execveat` on such a
fixture gets `EACCES` before its own predicted behaviour can occur. `X3`, in this same trial, is the
control that demonstrates the mechanism. Section F.1.

### G.3 Traced-window anchoring under a multithreaded parent — `M2`

Single-case, but a distinct root cause with a consequence larger than the case: the trace parser
anchors the child window on a `clone3` call without requiring `CLONE_PIDFD`, so extra parent threads
displace the window onto a `pthread`. Section F.4.

### Not a cluster

`T1`, `S4`, `E6`/`E6c` and `R3` each have their own cause. `E6` and `E6c` share a root cause with
each other and nothing else. `N3` is an environment block and not a defect at all.

## H. Result versus diagnosis

**Trial #2's frozen statuses are immutable and remain exactly as recorded, in full knowledge of
everything in Sections F and G.**

* `X2b`, `X2c`, `X4`, `T1` and `S4` are **FAIL**. The finding that their frozen predictions were
  unsatisfiable as written does not make them PASS. A prediction that could not be met is still a
  prediction that was not met, and the trial recorded that honestly.
* `M2` is **FAIL**. The finding that the comparison was made against the wrong syscall window does
  not make it PASS; it means the FAIL carries no information about multithreaded launch semantics.
* `E4`, `E6`, `E6c`, `O6`, `O7` and `R3` are **INVALID**. The finding that harness defects prevented
  them from being posed does not retroactively make them valid observations. INVALID is precisely
  the correct record of "this was never a test of the mechanism".
* `N3` is **BLOCKED**, and correctly so.
* The aggregate is **`MECHANISM_REJECTED`**, produced by the frozen rule from the preserved records,
  and independently reproduced.

`evidence.json`, `journal.jsonl`, the freeze `ba41a3f`, `SOURCE-HASHES.json`, the frozen checker, the
frozen case definitions, the D-7 record, the dispatcher and the GitHub result are **unchanged by this
review**. No corrected aggregate is manufactured anywhere in this document.

Any future experiment after a source or specification correction requires a **new freeze and a new
trial number**. Nothing that follows may be called "Trial #2 fixed".

## I. Owner decisions required

These are genuinely open, and this review does not decide any of them.

1. **Scope of correction before any future trial.** Whether to correct only the three demonstrated
   root causes (G.1, G.2, G.3) or to also revisit `T1`, `S4`, `E6`/`E6c` and `R3` in the same freeze.
2. **`S4`'s question.** Whether the launch mechanism is *required* to distinguish "the image ran and
   exited N" from "the child died before exec" without cooperation from the executed image, or
   whether S4 should be restated to give itself a positive exec-evidence channel. This determines
   whether S4's FAIL is a specification correction or a mechanism requirement.
3. **`E6c`'s recorded status.** It never reached the kernel question it exists to record
   (`i_mmap_writable` / `i_writecount`). Whether that question remains open in the manifest, or is
   retired to primary sources, is an owner call.
4. **The P-series' standing.** Whether `P1`, `P2` and `P4`'s containment conclusions are treated as
   unevidenced pending G.1's correction, given their reliance on an absence this defect explains.
5. **`N3` and privileged transition.** Whether the privileged transition stays UNTESTED on
   primary-source semantics, or whether a privileged execution environment is ever to be authorised.
   No privileged fixture was or should be manufactured to avoid this decision.
6. **Whether a bounded, non-posing offline diagnostic is authorised** for the two questions this
   review could not close from preserved evidence: which `find_marker_region` condition fired for
   `E6`/`E6c`, and which `si_code`/`si_status` pair `R3` actually produced. Both can be answered
   without posing any case or executing the launcher; both currently sit at MEDIUM confidence
   because they were not run.

## J. Rerun status

> ## TRIAL #2 MUST NOT BE RERUN
>
> The D-7 authorisation for Trial #2 is **consumed**. The first durable `case_pose_started` (`E1`)
> crossed the immutability boundary, and the trial ran to a complete `trial_end`.
>
> **Valid Trial #2 count: ONE.** **No Trial #3 is authorised.**

A rerun is not a retry. The dispatcher's own one-shot guard refuses a second dispatch and a second
attempt, and that guard is correct. Any future execution requires a new freeze, a new trial number
and a new owner authorisation.

**TRIAL #2 D-7 IS CONSUMED**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE**

**TRIAL #2 MUST NOT BE RERUN**

**NO TRIAL #3 IS AUTHORISED**

## Findings

| Severity | Finding |
|---|---|
| — | **No BLOCKER.** The preserved evidence honestly supports the frozen Trial #2 result; it reproduces exactly. |
| IMPORTANT | G.1 — relative build directory defeats `E4`'s symlink and `O6`/`O7`'s fixture signal, and makes the P-series' negative liveness evidence non-probative. |
| IMPORTANT | G.2 — generated fixtures are never executable, so `X2b`/`X2c`/`X4` could not test what they were written to test. |
| IMPORTANT | G.3 — the traced child window is not anchored on the `CLONE_PIDFD` clone3, so `M2` produced no evidence about a multithreaded parent. |
| IMPORTANT | `R3` — the launcher discards `si_status` unless `si_code == CLD_KILLED`, losing an observed signal number. |
| IMPORTANT | `helper_fork` ignores a failed FIFO open, making an unposable fixture indistinguishable from a dead descendant. |
| MINOR | `T1`'s frozen prediction names a token the launcher cannot emit; its siblings `T2`, `T3` and `T6` use the qualified form. |
| MINOR | `S4`'s declared channels and helper arguments contradict its own frozen prediction under the preregistered exec-evidence rule; as written it cannot be distinguished from `S5`. |
| MINOR | `find_marker_region` returns one message for three distinct failure conditions, so `E6`/`E6c`'s refusal does not name its own cause. |
| BACKLOG_NONBLOCKING | `helper_report`'s guard/marker/guard adjacency is assumed of three separate C objects rather than guaranteed by one declaration. |
| BACKLOG_NONBLOCKING | `MECHANISM_ACCEPTED` was unreachable in this trial: even with all six FAILs passing, six unposed cases would have produced `MECHANISM_INCONCLUSIVE`. |

An engineering defect in the mechanism is not a blocker to preserving the result — it is what the
experiment may have discovered. A wrong frozen expectation does not erase Trial #2 either; it
becomes a post-trial engineering finding, recorded above.

## Classification

**`TRIAL_2_RESULT_CONFIRMED_MECHANISM_REJECTED_READY_FOR_OWNER_DECISIONS`**

## Environment of record

| | |
|---|---|
| Platform | Linux `6.17.0-1022-azure`, `x86_64`, Ubuntu 24.04.5 LTS |
| Runner image | `ubuntu-24.04`, version `20260907.300.1` |
| euid | `1001` (unprivileged) |
| `ptrace_scope` | `1`; strace `/usr/bin/strace`, version 6.8 |
| clone3 | `available: true` — implemented and reached, `EINVAL` on a zero-size argument |
| gcc / glibc | 13.3.0 / 2.39 |
| `parent_no_new_privs` | `0`; binfmt_misc enabled |
| Block reasons derived | `unprivileged_runner` only |

Build identity was written durably at journal record `n = 2`, **before** the first
`case_pose_started` at `n = 4`, and covers all 12 objects in the build directory — `launcher_spike`
itself, every helper, every generated fixture, and the static-link probe. The in-memory digests
`harness.build()` returned agree with the on-disk identity hashed afterwards, and every non-PASS
case's `build_identity_binding.base_sha256` matches that identity, so the binaries the trial used
are the binaries this run built and hashed. **No later rebuild is evidenced.** Build 7 identity is
not substituted anywhere: Build 7 was prior compile evidence only, and this trial's own build
identity is what every binding resolves against.

## Review boundary

This review verified the execution, preserved the evidence, reproduced the result offline and
diagnosed the thirteen non-PASS cases. It did **not** re-audit the 59 PASS cases; PASS cases were
consulted only where they interpret a non-PASS case (`X3` and `X8` for the X cluster, `T2`/`T3`/`T6`
for `T1`, `S1`/`S5` for `S4`, the seven other traced cases for `M2`, and the P-series for
`O6`/`O7`). No defect was fixed, no frozen byte changed, no case posed, no experimental ELF executed
and nothing pushed.
