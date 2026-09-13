# LAUNCH-EXEC-01 — Trial #3 correction delta: bounded R-I1 / R-M1 re-review

**THIS IS A DELTA RE-REVIEW, NOT A FULL CORRECTION RE-AUDIT.**

**IT FIXES NOTHING, FREEZES NOTHING AND AUTHORISES NOTHING.**

| | |
|---|---|
| Base correction review | `ee8cd9106277a450f2987d3f4c759fd96085d5f3` — findings **R-I1** (IMPORTANT) and **R-M1** (MINOR) |
| Fix under re-review | `53ad8bfef1b66592f2ae2e4e3df5fc847b526834` |
| Reviewed base candidate | `7f98d4dcdb098abf96fe5e15e314194f5df6cb20` (parent `b20c2b0f41ff01ca13b451df34a59e57f7da75fe`) |
| Remote milestone | `1f0b86a13ece63badf28d7cef97d580ebc359a99`, unchanged after a read-only fetch |
| Scope | R-I1, R-M1, and their direct interaction with P1, P2, P3, P4, O6, O7, S4, S5, the freeze-plus-delta accounting and the publication boundary |
| Pushed | NO |

**Reviewer independence.** This re-review was run in the same working session that authored
`53ad8bf`. That is recorded here because [AGENTS.md](../../AGENTS.md) does not let a patch's
author be its only approver. To compensate, every load-bearing verdict below rests on artefacts
produced for this review: a side-by-side scoring matrix of the `7f98d4d` and `53ad8bf` modules, a
process-tree simulation of the gate, and a separate adversarial pass by a fresh reviewer context
that had not seen the implementation (§13). Nothing relies on the fix's own tests alone.

The already-green verdicts of `ee8cd91` are not reopened. Owner decisions carried into this
review: the P1/P2/P4 direct-child gate is authorised as fixture synchronisation if the ten
conditions of the instruction hold (§4), and the S4 precedence is fixed prospectively as
posing prerequisites → decisive mechanism contradiction → missing positive evidence (§8).

## 1. Starting state and exact delta

The worktree was clean, the branch `docs/helm-launch-architecture`, `HEAD` `53ad8bf`, and
`origin/docs/helm-launch-architecture` still `1f0b86a`.

`git diff ee8cd91..53ad8bf` — exactly the nine expected files:

| Path | +/− | Hunks |
|---|---|---|
| `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md` | +57 / −11 | §10.4 (S4) and §10.9 (P1/P2/P4) only |
| `docs/experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json` | +25 / −9 | S4 posed check removed, liveness health, correction review, `requires` |
| `docs/experiments/launch-exec-01/driver.py` | +205 / −24 | setup schema key; `release_setup_resources`; `_setup_fork_helper`; new health functions after `disarm_fixture_signal`; S4 posed check removed; S4 plan; `posing_evidence`; `TRIAL_OBSERVATION_KEYS`; `_run_once`; `_launch_and_observe` |
| `docs/experiments/launch-exec-01/observations.py` | +166 / −29 | health grammar and `rule_descendant_lifecycle`; `_s4_receipt_exit` and `assert_helper_exit_corroborated` |
| `docs/experiments/launch-exec-01/frozen_cases.py` | +7 / −3 | S4 note text only |
| `docs/experiments/launch-exec-01/helper_fork.c` | +109 / −5 | health tokens, `liveness_probe`, gate |
| `tools/tests/test_launch_exec_01_trial3.py` | +617 / −12 | S4 expectations, helper_fork source pin, new R-M1 and R-I1 classes |
| `tools/tests/test_launch_exec_01_driver.py` | +14 / −4 | lifecycle test, setup-key consumer and affected-set subtraction |
| `tools/tests/test_launch_exec_01_final.py` | +5 / −1 | P4 fabricated observation gains health |

`git diff 7f98d4d..53ad8bf` is the same set plus the `ee8cd91` review record. No hunk touches
`_setup_fork_helper_prearmed`, `_arm_fixture_signal`, `_read_fixture_signal`,
`disarm_fixture_signal`, `_check_fixture_descendant_signalled`, `rule_process_disposition`, the S5
plan, any P plan, `launcher_spike.c`, `checker.py` or the case table. The only plan change is S4
(`posed_when` removed, note text). No unrelated semantic delta was found.

## 2. R-I1 — the corrected path, inspected end to end

1. **Setup** (`driver.py` `_setup_fork_helper`, line 1485): a fresh `0600` liveness FIFO at
   `realpath(build)/<case>.liveness`. Only when `plan.rule == LIVENESS_HEALTH_RULE`
   (`descendant_lifecycle`, i.e. P1, P2, P4) does the setup also name
   `<case>.fixture-health`, remove any stale node, and append
   `--fixture-health-fifo <absolute path>` after `--liveness-fifo <absolute path>`. P3 (`sweep`) and
   T5 (`process_disposition`) keep exactly their previous arguments.
2. **Arming** (`_arm_liveness_health`, line 1727; called at line 3627 before either spawn path): the
   node is unlinked and recreated with `0600`, opened `O_RDONLY | O_NONBLOCK | O_CLOEXEC`, and
   refused unless it is a FIFO, has no group/other bits, is non-inheritable, is absent from
   `pass_fds`, is empty, and the helper argument is absolute and the same `(st_dev, st_ino)`. A
   refusal returns `not_posed` before the launcher exists.
3. **Descendant** (`helper_fork.c`): stdio release at line 195 (P1/P2), `setsid` at line 207
   (P2/P4), then `liveness_probe` (line 212 → 127): non-blocking health open (131); non-blocking
   liveness write open (133), where only `ENXIO` — a FIFO with no reader — leads on (137); otherwise
   `LIVENESS_OPEN_FAILED:<errno>`, or `LIVENESS_UNEXPECTED_OPEN` if it opened; `PROBE_REACHED`
   (140); gate write end closed (145). The blocking rendezvous open follows at line 217; its failure
   or a failed write is reported on the already-open health descriptor.
4. **Direct child**: `pipe2(gate, O_CLOEXEC)` only with a health path (183); after `fork` it closes
   its write end and reads the gate until end-of-file (246), then `return parent_exit` (250).
5. **Launcher** (unchanged): direct child in its own process group; drain until pidfd readiness and
   stream end-of-file or the post-exit drain bound; `kill(-child, SIGKILL)` before the reap.
6. **Harness reads** (`_launch_and_observe`): liveness rendezvous at line 3719 (reader opened only
   now), then `_read_liveness_health` at 3723 — at most 1000 ms for a first byte, then only what is
   buffered, at most 65 bytes — normalised by `observations.liveness_fixture_health`.
7. **Rule** (`observations.rule_descendant_lifecycle`, line 1850): `liveness_fixture_validity`
   first; only a reached probe with `channel_failure == "none"` lets the liveness byte render
   `descendant_survived` and its absence `descendant_died`.
8. **Cleanup**: `disarm_liveness_health` in `_run_once`'s `finally` and in
   `release_setup_resources`.

## 3. Health channel isolation

P1/P2/P4 only (selected by rule; P3, T5, O6 and O7 receive no health argument and
`_arm_liveness_health` returns `(None, None)` for them); case-private and invocation-private (new
node per arming); mode `0600` and private-mode checked; absolute and canonical; armed before launch;
empty before launch (poll checked); bounded (1000 ms, 65 bytes); closed grammar; independent of
stderr and of the liveness FIFO; harness reader non-inheritable and refused when present in
`pass_fds`; never passed as an exec descriptor — `helper_fork` opens it by path after exec; removed
after every invocation. Re-arming closes the old reader, so a writer of an earlier node gets `EPIPE`
and reaches nothing. O6/O7 keep their own FIFO, setup key, descriptor slot (`_FIXTURE_FD` versus
`_LIVENESS_HEALTH_FD`), reader and posed check; neither family's fact is read by the other's rule
or check (§9).

**P_HEALTH_CHANNEL_ISOLATION_SOUND**

## 4. PROBE_REACHED and the direct-child gate — load-bearing

### 4.1 What PROBE_REACHED proves

It is written only after the case's stdio release (P1/P2) and `setsid` (P2/P4), after the health
channel opened, and after a non-blocking write open of the exact liveness path failed with `ENXIO`,
which proves that path is a FIFO the descendant may open for writing and that no reader exists yet.
The next action is releasing the gate; the one after that is the blocking rendezvous open. It
therefore proves that the descendant reached the liveness observation point, not merely that it
forked.

**P_PROBE_REACHED_SEMANTICS_SOUND**

### 4.2 The ten owner conditions

| # | Condition | Evidence |
|---|---|---|
| 1 | exists only for the fixture-health observation point | gate created only with `--fixture-health-fifo` (line 183) |
| 2 | waits only for the health token or a bounded failure/termination | every step before `close(gate_write)` is non-blocking (`open … O_NONBLOCK` twice, one write of ≤ 32 bytes to an empty pipe, `setsid`); the direct child's `read` also returns at the descendant's death |
| 3 | never waits for liveness, survival, rendezvous or the byte | `close(gate_write)` (145) precedes the blocking open (217); the direct child never reads the liveness or health FIFO |
| 4 | preparation before PROBE_REACHED | release (195) and `setsid` (207) precede `liveness_probe` (212) |
| 5 | PROBE_REACHED immediately before the rendezvous | 140 → 145 → 217, nothing else between |
| 6 | direct child released before any liveness result | gate closes before the blocking open; the harness opens the liveness reader only after `launch()` returned, which is after the direct child exited |
| 7 | launcher lifecycle and group sweep still decide the result | process group and session membership are set only by the launcher's `setpgid` and the case's own `--setsid`; the gate adds neither |
| 8 | no launcher-visible or inherited descriptor | gate pipe and health descriptor are created after exec, close-on-exec, and closed; the static scratch image links `pipe2` and no `exec*` symbol |
| 9 | no `--setsid` added | P1/P2/P3/P4 helper arguments byte-identical to `7f98d4d` |
| 10 | no timeout or outcome expectation changed | no P plan hunk; P safe sets, gates and classes unchanged |

### 4.3 Process-tree simulation (Python stand-ins, WSL)

A stand-in launcher mirroring `launcher_spike.c`'s order (own process group; wait for the direct
child within a 2 s bound; `killpg(SIGKILL)` before the reap), a stand-in direct child and a stand-in
descendant in `helper_fork.c`'s order; the liveness reader opened only after "launch" returned; the
candidate normaliser and rule scored the result:

| Scenario | Direct child exited in bound | Health | Liveness | Rule |
|---|---|---|---|---|
| A — gate on PROBE_REACHED, P1 shape | yes (2 ms) | `PROBE_REACHED` | none | `descendant_died` |
| A — gate on PROBE_REACHED, P2 shape | yes (2 ms) | `PROBE_REACHED` | byte | `descendant_survived` |
| B — gate on liveness success, P1 shape | **no — held to the 2 s bound** | `PROBE_REACHED` | none | invalid construction |
| B — gate on liveness success, P2 shape | **no — held to the 2 s bound** | `PROBE_REACHED` | byte | invalid construction |
| C — PROBE_REACHED, swept before the reader exists | yes | `PROBE_REACHED` | none | `descendant_died` (valid) |
| D — PROBE_REACHED, survives, rendezvous completes | yes | `PROBE_REACHED` | byte | `descendant_survived` |
| E — never reaches the probe (P1 and P2 shapes) | yes | empty | none | INVALID |
| broken liveness open (P1 and P2 shapes) | yes | `LIVENESS_OPEN_FAILED:2` | none | INVALID |
| no gate, `setsid` 30 ms late (P2 shape) | yes | empty | none | INVALID — swept before `setsid` |
| gate on PROBE_REACHED, `setsid` 30 ms late (P2 shape) | yes (32 ms) | `PROBE_REACHED` | byte | `descendant_survived` |

Construction B — a gate that waits for liveness — is visibly different: the direct child cannot exit
before the post-return reader exists, so the launcher would observe a timeout instead of the
case's exit, which is exactly the construction change the owner excludes. `53ad8bf` implements A.
The last two rows show the gate's one side effect. Without the gate, a P2 descendant releases stdio
before `setsid`. The launcher can then see end-of-file and sweep in that window, so the observed case
would not be the declared setsid descendant, and a `descendant_died` could be a fixture race.
(For P4 the window is negligible, because retained stdio holds the launcher in its 2 s drain before
any sweep.) The gate guarantees that the declared preparation precedes the sweep, as condition 4
requires. The outcome is still decided by whether the launcher's sweep reaches that descendant, both
members stay in the frozen safe set, and no expectation changed. In consequence P2's
`descendant_died` is now reachable only through a mechanism change. The effect is recorded as RR-M1.

**P_DIRECT_CHILD_PROBE_GATE_SOUND**

## 5. R-I1 interpretation algebra and the negative control

`liveness_fixture_validity` refuses, in order: no health observation; `malformed`; `unreadable`;
`unexpected_open`; `open_failed` or `write_failed` (before or after the probe); any unrecognised
failure; `probe_reached` not true. Only then does the rule read the liveness fact; a liveness value
other than true or false is INVALID. An unarmable channel never reaches the rule (not posed).

Side-by-side scoring of identical fabricated observations with the `7f98d4d` and `53ad8bf` modules
(the old modules extracted with `git show`):

| Observation (each for P1, P2 and P4) | `7f98d4d` | `53ad8bf` |
|---|---|---|
| no health, no liveness byte — the `ee8cd91` shape | PASS `descendant_died` | INVALID |
| no health, byte | PASS `descendant_survived` | INVALID |
| health empty (no probe), no byte | PASS `descendant_died` | INVALID |
| `PROBE_REACHED` + `LIVENESS_OPEN_FAILED:2`, no byte | PASS `descendant_died` | INVALID |
| `LIVENESS_OPEN_FAILED:2` before the probe, no byte | PASS `descendant_died` | INVALID |
| `PROBE_REACHED` + `LIVENESS_WRITE_FAILED:32`, no byte | PASS `descendant_died` | INVALID |
| malformed health, no byte | PASS `descendant_died` | INVALID |
| O6/O7-style fixture signal true, no health, no byte (P1) | PASS `descendant_died` | INVALID |
| `PROBE_REACHED`, no byte | PASS `descendant_died` | PASS `descendant_died` |
| `PROBE_REACHED`, byte | PASS `descendant_survived` | PASS `descendant_survived` |

The same rule serves all three cases; P4's posed check `retention_observed` is unchanged and runs
before the rule. The stand-in reproduction of the `ee8cd91` defect (a descendant whose liveness open
fails) yields `LIVENESS_OPEN_FAILED:2` and INVALID for P1, P2 and P4, both in the fix's stand-in
tests run under WSL and in the simulation above.

**Residual timing gap (RR-M2).** The independent pass reproduced one narrow path with a Python
stand-in:
* A surviving descendant whose rendezvous open has completed stalls for more than the 3000 ms
  rendezvous bound before writing its byte.
* `_descendant_alive` times out and closes the reader. The write then raises `SIGPIPE`, whose action
  the launcher reset to default, and the descendant dies before it can write
  `LIVENESS_WRITE_FAILED`.
* With `SIGPIPE` ignored, the token arrives after the single health read.

Either way the record is `PROBE_REACHED` with no byte, so `descendant_died` is recorded for a
process that was alive at the rendezvous. This is not the `ee8cd91` defect: the fixture itself
reached its probe point and opened the rendezvous. It needs a multi-second stall of a runnable
process, and it affects a recorded case whose safe set contains both members. It is recorded MINOR,
and in practice the late write-failure token is unreachable or unread.

**R_I1_FALSE_PASS_CLOSED** — for the `ee8cd91` defect class (a broken or unproven fixture). The
bounded-rendezvous timing residual is RR-M2.

**RI1_OLD_FALSE_PASS_NEW_INVALID_PROVEN**

## 6. P-series non-regression

P1 and P2 still release stdio as their first three descendant actions, and stderr is not the health
authority. P2 and P4 keep `--setsid`; P1 has none. P4 keeps retained stdio and its posed check.
Helper arguments, timeouts, safe sets, gates and classes are unchanged, and the process-tree
questions are unchanged. A broken fixture is now INVALID rather than a mechanism result. P3 receives
no health argument, has no plan hunk, and its `sweep` rule is untouched.

**P_SERIES_QUESTION_NONREGRESSION_SOUND**

## 7. O6/O7 delta interaction

The O6/O7 functions have no hunk; `helper_fork.c` without `--fixture-health-fifo` creates no gate,
attempts the rendezvous exactly as before, and emits the same stderr diagnostic (only a saved `err`
local was introduced). An O7 observation with a valid P health fact, a live descendant and no
fixture signal is INVALID in both versions; a P1 observation with an O-style fixture signal and no
health is INVALID. No descriptor contract, `--setsid` or group-sweep change;
`WriterRetainedAfterChildExit` remains O6's result.

**O6_O7_DELTA_NONREGRESSION_SOUND**

## 8. R-M1 — S4 precedence

**Posing prerequisites.** S4's pre-mechanism prerequisites are its setup, the build-identity binding
of the starting `helper_report` bytes, and the declared spike flag; `prepare()` and
`bind_build_identity` refuse the case before launch when they fail. None of them reads the report,
the receipt or any mechanism result. `53ad8bf` removed the report-reading posed check, so mechanism
evidence no longer sits in the posing step.

**Two concepts that share a spelling.**

* The **EXEC-STATUS CLAIM** is the S4 prediction, produced by `rule_launcher_receipt_claim`: the
  receipt evaluated without any independent exec evidence. For a clean-EOF `Exited` receipt it is
  `ExecStatusIndeterminate`. It is never compared as a contradiction; it is the expected token.
* The **DIRECT-CHILD TERMINATION OBSERVATION** is the receipt's own `process_disposition`,
  `exit_code` and `wait_si_code`, read by `_s4_receipt_exit`. On S4's intended path the launcher
  observes `Exited` with status 7. A receipt whose `process_disposition` is itself
  `ExecStatusIndeterminate` (a short status record or an exec-confirmation bound elapsing, after which
  the launcher kills the child) or `ExitStatusUnobservable` reports no exit 7 at all; neither can
  arise when `helper_report` execs and exits, so each is an explicit mechanism outcome incompatible
  with the path — a FAIL, as definition §10.4 already stated when a report is present.

`assert_helper_exit_corroborated` combines the two sides violated → unobservable → holds.

| Observation | `7f98d4d` | `53ad8bf` |
|---|---|---|
| receipt `Exited` 7 `CLD_EXITED` + complete report declaring 7 | PASS | PASS (`ExecStatusIndeterminate`) |
| receipt `Exited` 7 + no report / truncated / malformed / stream incomplete | INVALID | INVALID |
| receipt `Exited` 7 without `wait_si_code` + no report | INVALID | INVALID |
| receipt `Exited` 7 beside `CLD_KILLED` + good report | INVALID | INVALID |
| admission refusal + no report | INVALID | **FAIL**, `mechanism_outcome` `refused:ElfNotInCohort` |
| explicit `ExecFailed:EACCES` + no report | INVALID | **FAIL**, `mechanism_outcome` `ExecFailed:EACCES` |
| `TimedOut:KilledByLauncher` + no report | INVALID | **FAIL** |
| `Signaled:SIGKILL` + no report | INVALID | **FAIL** |
| `Exited` 3 + no report | INVALID | **FAIL** |
| receipt disposition `ExecStatusIndeterminate` / `ExitStatusUnobservable` + no report | INVALID | **FAIL** |
| timeout, `SIGSEGV`, `Exited` 3, report declaring 3, report naming `helper_alt` — each with a report | FAIL | FAIL (unchanged) |
| explicit `ExecFailed` with an errno outside the frozen table + no report (or with a report) | INVALID | **INVALID — defect RR-I1** |

The claim token for a receipt whose disposition is `ExecStatusIndeterminate` is still
`ExecStatusIndeterminate`; the FAIL comes from the termination observation, not from the claim, so
the two are not conflated. Clean exec-status EOF alone still proves nothing.

**One decisive shape still becomes INVALID (RR-I1).** For an explicit `ExecFailed` record whose
errno is not in the frozen `ERRNO_NAMES` table, `rule_process_disposition` cannot render a token, so
`rule_launcher_receipt_claim` returns none. `_evaluate_repetition` then consults only
`DECISIVE_WITHOUT_TOKEN_ASSERTIONS`, which contains `no_executed_image` alone. So the violation
`_s4_receipt_exit` would report is never read, and S4 is scored "observation not interpretable".
The record is still an explicit pre-exec failure: the image demonstrably did not run, which the owner's
precedence makes a FAIL. The shape is also inconsistent with its neighbour, because an unknown
*stage* with a known errno renders `ExecFailed:<stage>:<errno>` and FAILs. No test covers it.

Reachability is disputed and is recorded as such. The independent pass (§13) rates the finding
IMPORTANT: it is reachable whenever the child fails with an errno outside the table. This session's
own inspection found no concrete errno outside the table that S4's stages can produce on this
construction. `E2BIG`, `EAGAIN`, `ELIBBAD` and `EBUSY` all need conditions S4 does not create, and
the same "unrenderable errno is not interpretable" convention applies to every `ExecFailed`-rendering
case since the Trial #2 freeze. Because this session authored `53ad8bf`, it does not overrule the
independent rating. The finding stands as IMPORTANT, and the owner may downgrade it.

**R_M1_S4_PRECEDENCE_DEFECT**

## 9. S4/S5 delta interaction

S4 keeps `--post-fork-delay-ms 200`, `helper_report --exit 7`, the conservative claim, and the report
as the required positive evidence for the ambiguous success path; its causal question is not S5's.
S5 has no hunk: its plan, `no_helper_report` posed check and `process_disposition` rule are
unchanged, and its died-before-exec, with-report, `ExecFailed` and truncated rows score identically
in both versions.

**S4_S5_DELTA_NONREGRESSION_SOUND**

## 10. Publication boundary

New durable facts: `fixture.liveness_health` with `probe_reached` (boolean), `channel_failure`
(closed set), `errno` (closed table or null) and `errno_number` (at most four digits, or null); and
`fixture.liveness_health_arming` (seven booleans). Malformed channel bytes are never echoed: the
normaliser returns only those four fields. New `not_posed` and reason strings are fixed text plus an
errno name or number. Arming failures go through `_problem`, which keeps only the errno name. FIFO
paths live in `extra_helper_args`, which is neither in `plan.as_dict()` nor in the posing evidence;
no pid or descriptor number is published. The fix's stand-in test asserts that no scratch root,
`fixture-health`, `.liveness` or `launch-exec-01` substring reaches the posing evidence or the
reason.

**RI1_PUBLICATION_BOUNDARY_SOUND**

## 11. Freeze-plus-delta accounting and Trial #2

* `SOURCE-HASHES.json` is byte-identical to `ba41a3f`.
* Byte drift against that manifest: `driver.py`, `frozen_cases.py`, `helper_fork.c`,
  `helper_report.c`, `launcher_spike.c`, `make_fixtures.py`, `observations.py`, and the definition —
  exactly the record's declared lists, unchanged in membership by `53ad8bf`. No new frozen input
  drifted and no declared one reverted. The candidate stays `NOT_FROZEN`.
* `run_launch_exec_01.py --verify-freeze` prints `freeze_verified: false`, lists those seven
  sources, and exits 1.
* Tamper copies: undeclared drift in `evidence.py`, `checker.py`, `helper_alt.c` or `ADR-0024`;
  declared drift reverted in `make_fixtures.py` or `helper_fork.c`; a name declared that has not
  drifted; and a posed check declared but unused — each fails the contract tests. Restoring every
  frozen input to `ba41a3f` makes `verify_freeze` true.
* **Trial #2**, replayed with every Python module extracted from `ba41a3f`: re-scoring the published
  records, replaying the journal, and comparing the journal's `case_completed` lines each give
  59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED and `MECHANISM_REJECTED`, with zero mismatches over all 72
  status/reason pairs; not torn; `d7_consumed` true. The evidence files, result review and
  postmortem diagnostics are byte-identical to `8a9dc77`, `1f0b86a` and `b20c2b0`, and the
  `ee8cd91` record is unchanged.

**RI1_RM1_DELTA_ACCOUNTING_SOUND**

**TRIAL2_HISTORICAL_INTEGRITY_SOUND**

## 12. Regression test quality

* **R-I1 against `7f98d4d`:** `LivenessFixtureHealthGrammar.test_7f98d4d_scored_a_missing_liveness_byte_descendant_died`
  extracts the `7f98d4d` modules, asserts PASS/`descendant_died` there for P1, P2 and P4, and INVALID
  in the candidate. `LivenessFixtureHealthThroughStandIns.test_the_reviewers_broken_liveness_open_is_invalid`
  reproduces the shape through a stand-in.
* **R-M1 against `7f98d4d`:** `S4DecisiveContradictionFirst` refusal, `ExecFailed` and other
  contradiction tests; the matrix in §8 shows each was INVALID under `7f98d4d`.
* Coverage present: no probe; open failure before and after the probe; write failure; malformed;
  stale node; valid died; valid alive; directory change for both paths; stdio released before the
  health write; non-inheritable reader and `pass_fds` refusal; relative or foreign health path;
  unarmable channel; no path in evidence; S4 conservative receipt plus missing or malformed report
  INVALID; S5 unchanged; P3, T5, O6 and O7 receive no health argument; the `helper_fork.c` order of
  release, probe, gate and rendezvous.
* Expectations are literals (tokens, statuses, helper-argument tuples, errno values), not values read
  back from the implementation. Construction B has no dynamic test; the static order test pins
  `close(gate_write)` before the blocking open, and §4.3 demonstrates B.
* Gaps, tied to findings rather than to test weakness in what is covered: no test covers an
  `ExecFailed` record with an out-of-table errno (RR-I1), and none covers the post-rendezvous
  `SIGPIPE` timing path (RR-M2).

**RI1_RM1_REGRESSION_TESTS_SOUND**

## 13. Independent adversarial pass

A fresh reviewer context, given only the instruction's acceptance criteria and pointers to the code
and not this record, reviewed `53ad8bf` read-only. It ran only Python:
* fabricated observations scored with the `53ad8bf` modules and a `git archive` copy of `7f98d4d`;
* stand-in processes on scratch FIFOs;
* the trial3, driver, final and delta suites (642 tests, OK, 1 skipped).

It executed no C binary and changed no repository file. Its conclusions:

* **Direct-child gate: SOUND.** The direct child waits only for end-of-file on the gate. Every step
  before the gate closes is non-blocking. The gate pipe never crosses an exec. No `--setsid`,
  process-group or session change was made. P3, T5, O6 and O7 keep their arguments. It independently
  identified the P2 `setsid`-versus-sweep race the gate removes (RR-M1).
* **R-I1: SOUND at scoring.** Across P1, P2 and P4 × liveness {false, true, none} × 16 health
  variants, only `PROBE_REACHED` with no failure and no byte scores `descendant_died`; everything
  else is INVALID. A never-written channel, a stale writer and P3 with bad health data behave
  correctly. It reproduced the timing gap RR-M2.
* **S4: DEFECT for one shape.** Refusal, `ExecFailed` with a known errno or an unknown stage,
  `Exited 0`, signals, timeout and `ExitStatusUnobservable` FAIL without a report. `Exited 7` with a
  missing, malformed, truncated, stream-incomplete or exit-less report is INVALID, and the matching
  shape PASSes. There is no conflation of the claim with the receipt disposition. An `ExecFailed`
  record with an out-of-table errno is INVALID; it rated this IMPORTANT (RR-I1). S5 is identical on
  all eight shapes.
* **Publication: SOUND.** Whitelisted keys only; `_problem` strips paths; no raw bytes are returned.
* It also noted that a failed `pipe2` silently runs without the gate (RR-B2), which is INVALID at
  worst.

Its findings match this record's matrix. The only disagreement is the severity of RR-I1, which
§8 keeps at the independent rating.

## 14. Validation

| Check | Result |
|---|---|
| `git diff --check` (`7f98d4d..53ad8bf` and working tree) | clean |
| `python tools/validate_docs.py` | PASS |
| `cargo fmt --check` | PASS |
| Windows, Python 3.14.3, `python -m unittest discover -s tools/tests` | 766 OK, 74 skipped (POSIX-only) |
| WSL Ubuntu 24.04, Python 3.12.3, full suite | 766 OK, 1 skipped (the compiled-image E6 test; no `cc` on `PATH`; E6 is untouched by the delta) |
| `helper_fork.c` at `53ad8bf` (SHA-256 `52089697…87cb0`), `x86_64-linux-gnu-gcc-13 13.3.0 -O2 -Wall -Wextra -static`, unpacked unprivileged into a scratch directory | rc 0, zero diagnostic bytes, ELF `EXEC`, no `PT_INTERP`, links `pipe2`, no `exec*` symbol; read as data only, then deleted. **Not Build 8.** |
| Scoring matrix `7f98d4d` versus `53ad8bf`; gate simulation; Trial #2 replay; tamper copies | see §§4.3, 5, 8, 11 |

**Experimental LAUNCH-EXEC ELF executions: ZERO. Cases posed: ZERO.** No `helper_fork`,
`launcher_spike` or other helper ELF was executed, no D-7 path was taken and no workflow was
dispatched. The only processes run were Python stand-ins, the test suites, the compiler and binutils.

## 15. Build

`launcher_spike.c`, `helper_report.c` and `helper_fork.c` still differ from the Trial #2 freeze.

**BUILD_8_REQUIRED_FOR_FUTURE_FREEZE**

## 16. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| RR-I1 | IMPORTANT (independent rating; this session found no concrete producing errno on S4's path) | S4 precedence | Only for an explicit `ExecFailed` record whose errno is outside the frozen `ERRNO_NAMES` table | The claim rule renders no token, and `helper_exit_corroborated` is not a decisive-without-token assertion, so this explicit pre-exec failure is INVALID instead of FAIL. An unknown stage with a known errno FAILs. Untested | Bounded fix before freeze preparation — for example, let S4's launcher-side violation decide without a token, or render an out-of-table errno as a closed decisive token. Not fixed here. The owner may instead downgrade it |
| RR-M1 | MINOR | P2 gate documentation | Yes, by design | Besides buffering the health token, the gate guarantees that the case's own `setsid` precedes the sweep. Without it a P2 descendant could be swept before `setsid` (§4.3), so a `descendant_died` could be a fixture race; now P2 `descendant_died` needs a mechanism change. This is condition 4's declared preparation, not a construction change, but §10.9 does not say so | State it in §10.9 at freeze, noting that earlier P2 `descendant_died` observations were not race-free |
| RR-M2 | MINOR | P2/P4 rendezvous timing | Only with a stall of more than 3 s between the completed rendezvous open and the one-byte write | A survivor then dies of `SIGPIPE`, or its `LIVENESS_WRITE_FAILED` arrives after the single health read, and is recorded `descendant_died` (§5) | Optional hardening at freeze: treat end-of-file without the byte as INVALID, or re-read health after closing the liveness reader |
| RR-B1 | BACKLOG_NONBLOCKING | Vocabulary | n/a | The S4 exec-status claim and the receipt's `process_disposition` share the spelling `ExecStatusIndeterminate`; the code keeps them apart (§8) | Name the two concepts distinctly in the frozen definition |
| RR-B2 | BACKLOG_NONBLOCKING | `helper_fork.c` gate | Only if `pipe2` fails | The helper silently runs without the gate; a lost token is INVALID, never a false PASS | Optional diagnostic at freeze |

No BLOCKER. One IMPORTANT (RR-I1).

## 17. Verdicts

| Area | Verdict |
|---|---|
| R-I1 | **R_I1_FALSE_PASS_CLOSED** |
| Direct-child gate | **P_DIRECT_CHILD_PROBE_GATE_SOUND** |
| Health channel | **P_HEALTH_CHANNEL_ISOLATION_SOUND** |
| PROBE_REACHED | **P_PROBE_REACHED_SEMANTICS_SOUND** |
| `ee8cd91` negative control | **RI1_OLD_FALSE_PASS_NEW_INVALID_PROVEN** |
| P-series | **P_SERIES_QUESTION_NONREGRESSION_SOUND** |
| R-M1 | **R_M1_S4_PRECEDENCE_DEFECT** (one shape, RR-I1) |
| O6/O7 | **O6_O7_DELTA_NONREGRESSION_SOUND** |
| S4/S5 | **S4_S5_DELTA_NONREGRESSION_SOUND** |
| Trial #2 | **TRIAL2_HISTORICAL_INTEGRITY_SOUND** |
| Publication | **RI1_PUBLICATION_BOUNDARY_SOUND** |
| Accounting | **RI1_RM1_DELTA_ACCOUNTING_SOUND** |
| Tests | **RI1_RM1_REGRESSION_TESTS_SOUND** |
| Build | **BUILD_8_REQUIRED_FOR_FUTURE_FREEZE** |

**TRIAL #2 D-7 IS CONSUMED. LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE. TRIAL #2 MUST NOT BE
RERUN. NO TRIAL #3 IS AUTHORISED. NO TRIAL #3 FREEZE EXISTS.**

**TRIAL #3 CORRECTION CANDIDATE IS NOT READY FOR FREEZE.**

The single blocking item is RR-I1. R-I1 is closed, and the direct-child gate is sound under the
owner's ten conditions, so no further gate decision is requested. If the owner downgrades RR-I1 to
MINOR, every load-bearing verdict here is otherwise SOUND. If it is fixed, a re-review of that one
shape suffices.

Classification: **TRIAL_3_RI1_RM1_REREVIEW_NEEDS_FIX**
