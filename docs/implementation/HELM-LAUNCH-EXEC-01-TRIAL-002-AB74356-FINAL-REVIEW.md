# LAUNCH-EXEC-01 — Trial #2 final bounded independent review of freeze `ab74356`

Final bounded independent pre-D7 review of the Trial #2 freeze
`ab743567d84bfcb25311688b00496db866799174`. It supersedes `f417984` with the classification
correction `f9bbf39` and the O7 owner correction `82b8746`. The reviewer implemented neither. The
scope is bounded to four things: the `f9bbf39` classification rules, the O7 construction and its
evidence, the freeze's integrity and Build 7's binding. Nothing was fixed and no case was posed.
No `launcher_spike` or helper ELF ran, no `Authorisation` was constructed and no dispatcher was
created. No D-7 was granted and nothing was pushed.

**Classification: `TRIAL_2_NEEDS_OWNER_DECISION`.**

The freeze is intact and Build 7 still binds every C byte. The central classification rule, the
FAIL > INVALID > PASS reduction, O8, F7, the report-based rules, M2's control arm and BLOCKED
posing evidence are sound under 264 fabricated-observation checks on Windows and Linux.

One reachable BLOCKER remains, in the O7 construction itself. O7's `helper_fork` descendant has no
`--setsid`, so it stays in the direct child's process group. The launcher kills that group before
`launch()` returns, while the descendant is still blocked opening the liveness FIFO. The harness
opens the FIFO's read end only after `launch()` returned. `fixture_descendant_alive` can therefore
never hold, and mandatory O7 is INVALID on every honest run. A Linux stand-in that drives the
frozen harness functions reproduces this in 3 of 3 runs.

One IMPORTANT item concerns S2/S7. A correct `ExecFailed:CHDIR:EACCES` repetition scores INVALID
whenever the launcher stops draining before stdout's end-of-file. A stand-in puts the chance of
that in at least one of S7's 200 repetitions at 18–33 %. Both items need an owner decision before
a fix.

## A. Review target and history

| Item | Verified state |
|---|---|
| Branch / HEAD | `docs/helm-launch-architecture`, clean; HEAD `ab743567d84bfcb25311688b00496db866799174` |
| Remote tip | `f41798455662579c3895f7a4af50bac17d47bb43` (`git ls-remote`); local is five commits ahead, none pushed |
| Ancestry | `f417984` → `2591f35` → `7f97100` → `f9bbf39` → `82b8746` → `ab74356`, single-parent and linear |
| `2591f35` | `BUILD-EVIDENCE.md` only, 61 insertions, 0 deletions |
| `7f97100` | adds only the [F417 bounded review](HELM-LAUNCH-EXEC-01-TRIAL-002-F417-BOUNDED-REVIEW.md); no later commit touches `docs/implementation/` |
| `f9bbf39` | the definition, `SOURCE-HASHES.json`, `driver.py`, `observations.py` and three test files |
| `82b8746` | the definition, `SOURCE-HASHES.json`, `driver.py`, `observations.py` and one test file |
| `ab74356` | `PROJECT_STATE.md` (+29), `BUILD-EVIDENCE.md` (+23, append-only), `SOURCE-HASHES.json` and one reworded test assertion; no hashed source |
| `f417984..ab74356` | no C, helper, Rust or Cargo file changed |
| Trial #1 | not reinterpreted; the manifest lineage still reads `TRIAL_ABORTED_AFTER_BOUNDARY`, `AGGREGATE_NOT_DERIVABLE_FROM_FROZEN_EVIDENCE` and `D7_AUTHORIZATION_CONSUMED` |

## B. Freeze integrity

Every hash was recomputed from the git blobs at `ab74356` and from the working tree, independently
of the repository's own verifier.

| Check | Result |
|---|---|
| `SOURCE-HASHES.json` | blob `88977184ff2639f8d0ceef43e07436d42ddaaaa5`, SHA-256 `d871f8d6710eabf43763aa0712490213b42150efe5fcc13c032ddd4875cffbdd`, both exact |
| Source hashes | **17 / 17**; only `driver.py` and `observations.py` differ from `f417984` |
| Definition hashes | **3 / 3**; only `LAUNCH-EXEC-01-DEFINITION.md` differs from `f417984` |
| Unhashed files | exactly `BUILD-EVIDENCE.md` and `SOURCE-HASHES.json`, both declared |
| Partition | 72 total, 54 mandatory, 11 conditional, 7 recorded |
| Traced | 8: E1 E7 F4 F7 M1 M2 M3 M4 |
| Driver | 72 handlers, complete, 72 posable, 0 unposable; `--verify-freeze` true |
| Status | `NOT_RUN`, `d7_execution_authorised: false`, valid Trial #2 count ZERO |

`frozen_cases.py` and `checker.py` are byte-identical to `f417984`. No membership, class,
prediction, safe set, gate, block reason or traced flag moved, and O7 is still mandatory with
prediction `Exited:42`.

## C. Classification algebra: `CLASSIFICATION_ALGEBRA_SOUND`

`_evaluate_repetition` (driver.py 2547–2657) judges each launcher invocation in a fixed order:

1. not posed → INVALID;
2. a non-return → FAIL;
3. M2's control arm;
4. the posed check;
5. the rule;
6. the assertions.

A violated assertion replaces the token with its violation token, which is no case's expectation.
An unobservable assertion suppresses only a token that would otherwise PASS; a non-matching token
still FAILs. `checker.score_case` then decides the status. INVALID comes only from `not_posed`, a
missing outcome or structurally invalid traced evidence. FAIL comes from a non-return, a late
return, or a token, gate or assertion that contradicts the frozen expectation.

Across the probe no decisive failure became INVALID for want of secondary evidence, with one
exception. That exception is case-specific and recorded under F as AB7-M1: S2/S7 with a report
sentinel that does not parse.

## D. Repeated cases: `REPEATED_CASE_REDUCTION_SOUND`

`evaluate` (driver.py 2660–2704) runs `_evaluate_repetition` on every repetition. Each
repetition's view replaces every per-invocation key with that repetition's own
(`TRIAL_OBSERVATION_KEYS`), so no repetition borrows another's evidence. Each is scored by the
frozen checker and reduced by `reduce_repetitions`: FAIL, else INVALID, else PASS, with an empty
list INVALID. A BLOCKED observation returns before any repetition. A counter wrapped around
`_evaluate_repetition` saw exactly 200 calls for a 200-repetition case and none for a BLOCKED one.

| Fabricated through `evaluate` and the checker | Case status |
|---|---|
| 200 PASS | PASS |
| FAIL at repetition 0, 100 or 199 | FAIL; `decided_by_trial` is 0, 100 or 199 |
| one INVALID, 199 PASS | INVALID |
| one INVALID and one FAIL, either order | FAIL |
| S7 BLOCKED on `euid_zero` | BLOCKED, no repetition evaluated |

All 200 repetitions' statuses and outcomes survive the journal, with the reduction counts.

## E. O8: `O8_CLASSIFICATION_SOUND`

`trial_floor_512` is gone and O8 has no posed check. Each repetition is decided by its own
`rule_stream_exact`. That rule returns no token only when there is no parseable receipt, no
declared stream, or no stream block.

| O8 repetition | Case status |
|---|---|
| 512 recipe bytes, all 200 | PASS |
| first = 400, middle = 0, last = 511 bytes | FAIL (`stream_mismatch`) |
| 512 wrong bytes; 512 bytes not `CompleteAtEof` | FAIL |
| a repetition that did not return | FAIL |
| an unparseable receipt | INVALID |

A short but parseable repetition cannot reach INVALID.

## F. S2/S7: `S2_S7_CLASSIFICATION_DEFECT`

**What holds.** A forced state that did not land at the barrier is not posed, which is INVALID.
With the state landed, `ExecFailed:CHDIR:EACCES` on a stdout drained to its end PASSes. Each of
the following is a FAIL, at S7 repetitions 0, 100 and 199 alike:

* an image that ran with a complete report;
* `Exited:127`;
* `ExecStatusIndeterminate`;
* a clean-EOF `Exited:0`;
* `ExecFailed` at another stage.

Every one of S7's 200 repetitions is reduced by the central algebra.

**AB7-I1 (IMPORTANT) — a correct repetition can be INVALID.** The PASS path needs stdout drained
to its end. launcher_spike.c 955 leaves the drain loop the moment `exec_failed` is set. Lines
986–989 then report any stream whose EOF was not yet read as `WriterRetainedAfterChildExit`. The
knock-on is mechanical:

1. `report_state` becomes `stream_incomplete`;
2. `no_executed_image` is unobservable;
3. `ExecFailed:CHDIR:EACCES` is the frozen prediction, so the repetition is not posed: INVALID.

A Linux stand-in reproduced the child's descriptor layout at a CHDIR failure and the launcher's
loop order: status, stdout, stderr, pidfd, break on `exec_failed`. It stopped before stdout's EOF
in 3 and in 6 of 3000 repetitions. That is P ≈ 18–33 % that at least one of S7's 200 repetitions
is INVALID with a correct mechanism, which makes the aggregate `MECHANISM_INCONCLUSIVE`. The
stand-in's parent is Python, slower than the C loop, so the rate is an estimate, not a C
measurement. The child's explicit pre-exec status record already proves that no image ran, and
section 9.6 treats such a record as decisive for the report-based rules. `no_executed_image`
does not read it. The dependency existed at `f417984` through the `no_helper_report` posed check;
`7f97100` did not record it. It never produces a false PASS or FAIL.

**AB7-M1 (MINOR) — a report sentinel that does not parse.** Section 9.6 (definition 962–963)
says: "A report sentinel on descriptor 1 is decisive evidence that an image ran. One such S7
repetition fails the case." Suppose the sentinel arrives but the report does not parse, on an
`Exited`, `Signaled` or `TimedOut` run. Exec confirmation is then uninterpretable, and
`rule_process_disposition` (observations.py 1280–1290) returns no token before
`no_executed_image` runs. The repetition is INVALID, not FAIL. The pinned `helper_report` writes a
complete report, so this path needs a second anomaly.

## G. F7: `F7_ADJACENCY_AND_CLASSIFICATION_SOUND`

The adjacency is followed from live return values, never from `close_range`:

1. `--parent-close-low-fds 1` closes fd 0 before the pin (launcher_spike.c 659–666).
2. The exec object opens as 0 (684), and the barrier proves it (`exec_fd_at_0`, driver.py
   1693–1719). The barrier must land before CONTINUE, so the literal 0 handed to the parser is a
   proven fact.
3. The four `pipe2` calls (767–768) take the lowest free descriptors.
4. `move_above_2(exec_fd)` (775, 193–200) is the only `F_DUPFD_CLOEXEC` after them. It lands on
   the lowest free descriptor above 2, which is `status_w + 1` when nothing is inherited there.

`parse_parent_descriptor_pair` (observations.py 766–838) reads only the parent task's records
before `clone3`:

* it takes the fourth pipe's write end as `status_w`;
* it follows every later `F_DUPFD` of either descriptor to its returned value;
* it requires every one of those returns to be an observed success;
* if the child reaches `execveat`, it requires the child's fd argument to be the followed exec
  descriptor.

The boolean is `|exec_fd − status_w| == 1` over those live return values. The one source
assumption is which pipe carries exec status, and that is fixed by the frozen, build-bound source.

Synthetic strace records of the launcher's sequence gave these results through the frozen parser
and evaluator:

* the success path and the inverted-gap `EINVAL` path both give the pair `{12, 11}`, which is
  adjacent;
* a descriptor already sitting at `status_w + 1` makes the pair not adjacent, which is INVALID;
* three pipes, an unobserved return, an `execveat` mismatch or no `clone3` give no pair, which is
  INVALID;
* adjacency with `close_range = -1 EINVAL` gives `ExecFailed:CLOSE_RANGE:EINVAL`, a FAIL decided
  by `decisive_without_report` before any report state;
* adjacency with fds 0, 1 and 2 is a PASS.

The durable record carries only `exec_status_pair_adjacent: true`.

## H. Report-based rules: `REPORT_BASED_CLASSIFICATION_SOUND`

Derived from the plans, the seven report rules cover exactly A1–A4, A6, V1, F1–F7, X2c and N1–N3,
the set section 9.6 names. Each reads the report through `_report_or_decisive`. For all 17 cases:

* an explicit pre-exec record, an admission refusal, or a complete stdout holding no report is a
  FAIL;
* no receipt, a retained stdout, a malformed report or a truncated report is INVALID.

F6's frozen failure, a child stdout lost through `FD_CLOEXEC`, leaves an empty but complete stdout.
That is a FAIL (`ExecStatusIndeterminate`), not INVALID. A report missing fd 1, showing stdio
`CLOEXEC` or showing an extra descriptor is a FAIL, and a correct report PASSes. Wrong content on
an honestly posed case is a FAIL: A1's argv, V1's environ and N1's `NoNewPrivs: 0`.

## I. M2: `M2_CONTROL_CLASSIFICATION_SOUND`

`_control_arm_verdict` (driver.py 2521–2544) decides the control arm:

| Control arm | Status |
|---|---|
| never run, not posed, or its return not recorded | INVALID |
| `launch_returned: false` | FAIL; the record's own `launch_returned` becomes false |
| returned after `total_bound_ms` | FAIL |
| both arms returned, identical child windows | PASS |
| both arms returned, different windows | FAIL |

A watchdog non-return carries `launch_returned: false` and no `not_posed`, so it cannot be
relabelled as missing control evidence. The durable `control_arm` fact records `run`, `posed` and
`launch_returned`. AB7-N3 records one backlog precedence detail from the unchanged checker.

## J. BLOCKED records: `BLOCKED_POSING_EVIDENCE_SOUND`

All 11 conditional cases score BLOCKED. Each carries `invocation` `{pose_started: false,
mechanism_invoked: false, blocked_cause: <its frozen cause>}`, with no `forced_state` and no
`posed_check`. A pre-setup not-posed record says `{false, false}`. The journal applies P-14 before
writing, and a BLOCKED S2 record came back through the public sanitiser, an fsynced journal and
replay unchanged. AB7-N1 notes the cosmetic extras.

## K. O7 plan: `O7_PLAN_SOUND`

The plan is exactly the owner's decision:

| Field | Value |
|---|---|
| binary | `helper_fork` |
| setup | `fork_helper` |
| arguments | `--retain-stdio --parent-exit 42 --lifetime-ms 20000` (`P_DESCENDANT_LIFETIME_MS`) |
| channels | receipt, liveness |
| not required | payload, stream recipe, helper report |
| posed check | `fixture_descendant_alive` |
| assertion | `stderr_capture_failure_reported` |
| rule | `process_disposition` |
| class and prediction | mandatory, `Exited:42` |

Stdout retention is not scored. The FIFO path reaches the helper only as the setup's appended
`--liveness-fifo <path>`, and it never enters `helper_args` or the durable record. That
construction is the subject of AB7-B1.

## L. O7 posing: `O7_POSING_EVIDENCE_INDEPENDENT`, and never established

**It is independent.** `_check_fixture_descendant_alive` (driver.py 1764–1776) declares and reads
only `descendant_alive_after_launch`. That value comes only from `_descendant_alive` (3223–3249),
which opens the case-private FIFO's read end after `launch()` returned and accepts exactly one
`L`. The check returns false for a perfect receipt without liveness, and true with no receipt at
all. No completeness, exit code or disposition is read, so the evidence is not circular.

**The chain breaks at the descendant's write.** Traced through the frozen sources:

1. The launcher's child calls `setpgid(0, 0)` (launcher_spike.c 307) and the parent calls
   `setpgid(child, child)` (826), so the direct child leads its own process group.
2. The child execs `helper_fork`, which forks (helper_fork.c 80). O7 passes no `--setsid`
   (98–100), so the descendant stays in that group.
3. The descendant calls a blocking `open(fifo, O_WRONLY)` (104). It cannot return until a reader
   exists.
4. The direct child returns 42 (115). The launcher sees the pidfd and drains for at most
   `POST_EXIT_DRAIN_MS` (957–964). Then, unconditionally and before the reap, it calls
   `kill(-(pid_t)child, SIGKILL)` (967–975).
5. The descendant, still blocked in `open`, dies. The launcher reaps, prints the receipt and
   exits.
6. Only now does `_launch_and_observe` call `_descendant_alive` (driver.py 3109–3112). No writer is
   left, the poll times out after 3000 ms, and the answer is false.

The frozen definition already records this mechanism. P2 is "same, with setsid; establishes that
a group sweep is best-effort", and P4's retaining descendant is setsid for the same reason
(definition 445, 447). Every O7 test fabricates `descendant_alive_after_launch=True`
(test_launch_exec_01_final.py 668–691), and no test models the sweep.

**Stand-in, Linux, no ELF.** Python processes replicate `launcher_spike.c`'s process-group
handling, pidfd poll, bounded post-exit drain, pre-reap group sweep and completeness rule, and
`helper_fork.c`'s argument parsing and fork, retention and FIFO behaviour. The harness side is the
frozen driver itself: `SETUPS["fork_helper"]`, `spike_argv`, `_descendant_alive`, `evaluate` and
the checker. The stand-in ran on WSL 6.6.87.2 with Python 3.12.3.

| Variant | Descendant at return | Exit | stderr completeness | liveness | O7 |
|---|---|---|---|---|---|
| frozen O7 arguments, frozen sweep (3 runs) | gone | 42 | `WriterRetainedAfterChildExit` | false | **INVALID**, "the forced state did not materialise: fixture_descendant_alive" |
| counterfactual: plus `--setsid`, as P2/P4 | alive | 42 | `WriterRetainedAfterChildExit` | true | PASS |
| counterfactual: frozen arguments, no sweep | alive | 42 | `WriterRetainedAfterChildExit` | true | PASS |

## M. O7 exec evidence: `O7_LIVENESS_EXEC_EVIDENCE_SOUND`

The inference is valid. Among the frozen artefacts only `helper_fork.c` has a FIFO write path, and
it sits in the branch after `fork()` returns 0 in the child. The FIFO path reaches only the
executed image's argv. The launcher's pre-exec child code resolves no pathname, and the harness
never writes the FIFO. A positive `L` therefore requires all four steps: an exec of
`helper_fork`, its fork path, a created descendant, and that descendant's write.

Stale data cannot fake it. FIFO data lives only while an end is open, and the setup unlinks and
recreates the case-private FIFO. O7 has one repetition, and paths are per case. A payload that
contradicts the recipe is still decided before liveness (observations.py 377–387). AB7-N2 records
that the FIFO is not unlinked at release. The inference is sound; its premise never occurs while
AB7-B1 stands.

## N. O7 result: `O7_RESULT_ASSERTION_SOUND`

`stderr_capture_failure_reported` is an assertion, never a posed check, and it reads only the
receipt's stderr completeness:

| stderr completeness | Assertion |
|---|---|
| absent | unobservable |
| `WriterRetainedAfterChildExit` | holds |
| any other value, e.g. `CompleteAtEof` | violated |

`Exited:42` is derived separately, by `rule_process_disposition` from the process fields. The
durable record keeps the two facts apart. The disposition is `outcome`, or `mechanism_outcome`
once the assertion replaces it. The assertion's result is kept with a detail that names the
completeness.

The real receipt parser refuses any completeness other than those two values. An absent
completeness on a real run is therefore an unparseable receipt, which is INVALID. An unobservable
assertion beside a non-predicted token such as `Exited:0` is a FAIL, so the decisive
contradiction controls.

## O. O7 scoring: `O7_SCORING_SOUND`

| Fixture | Exit | stderr completeness | Status |
|---|---|---|---|
| established | 42 | `WriterRetainedAfterChildExit` | eligible PASS |
| established | 42 | `CompleteAtEof` | FAIL: `capture_failure_not_reported`, `mechanism_outcome` `Exited:42` |
| established | 0 | `WriterRetainedAfterChildExit` | FAIL (`Exited:0`) |
| established | 0 | `CompleteAtEof` | FAIL |
| established | 42 | missing | INVALID |
| not established (false or none) | 42 | `WriterRetainedAfterChildExit` | INVALID |

Stdout's completeness does not change any row. The evaluator is sound, but the "established" rows
cannot occur on a real run while AB7-B1 stands.

## P. O7 retained-writer timing

| Requirement | Result |
|---|---|
| the direct child exits 42 right after creating the descendant | holds (helper_fork.c 80–115) |
| the descendant retains fds 1 and 2 | holds (`--retain-stdio`) |
| it is alive through the launcher's post-exit drain window | holds, and structurally: it blocks in the FIFO `open`, holding both pipes, until something ends it. `WriterRetainedAfterChildExit` is not timing luck and does not even depend on `P_DESCENDANT_LIFETIME_MS` (20000) > `POST_EXIT_DRAIN_MS` (2000) |
| it eventually exits under the bounded fixture lifetime | **does not hold**: the launcher's group sweep kills it at the end of the drain window, before it answers or reaches its lifetime sleep (AB7-B1) |

## Q. O7 durable evidence

The O7 PASS, FAIL and INVALID records went through the public sanitiser, an fsynced journal and
replay, and came back identical. Each record carries:

* `posed_check` `{name, held}` and `measured.descendant_alive_after_launch`, which is both the
  landing fact and the exec evidence;
* `fixture` `{binary, helper_args}` and the public build-identity binding;
* `outcome`, `mechanism_outcome`, and the assertion's result and detail.

No FIFO path, temporary path, pid, descriptor number, inode, device, raw receipt or capture prefix
appears in the raw journal.

## R. Preregistration: `FINAL_TRIAL2_PREREGISTRATION_COMPLETE`

Before execution, definition section 9.6 (877–1016), section 2 (295–298) and the manifest's
`posing_versus_showing` freeze all of the following:

* the central FAIL versus INVALID rule;
* the repetition reduction;
* O8, S2/S7, F7 and the report-based decisive-evidence rule;
* M2's control-arm non-return;
* BLOCKED posing evidence;
* O7's construction, liveness posing evidence and liveness exec evidence;
* the retained-writer capture-failure fact;
* stdout retention as a permitted side effect;
* O7's scoring.

The manifest's assertion registry, violation tokens and posed-check inputs equal the code's, and
no violation token is any case's expectation.

Every observation the frozen C and helpers can produce has a determined status without an owner
choice. Two latent disagreements concern observations the frozen C and helpers cannot practically
produce: AB7-M1 and AB7-M2. The O7 preregistration is complete, but it describes a fixture that
cannot be established (AB7-B1).

## S. Build 7 and the post-freeze note: `BUILD_7_STILL_BINDS_FINAL_C_SOURCES`

| Item | Verified |
|---|---|
| C sources `f417984` → `ab74356` | byte-identical git blobs, SHA-256 equal to the manifest |
| `launcher_spike.c` | `0a45447b627f16dce2fd7193e26993dcc126ae96dce6e6c5b511b41361e0d622` |
| `helper_report.c` | `99770ed8cfdd4049f9bef3624e21850496b6c14a169c3895b54ab7a858d54711` |
| `helper_alt.c` | `7993aa631abdca6796171fe4ffafb2d296f2a00b61ea2322ee8e624731b99113` |
| `helper_dynamic.c` | `b3c398d840c16dfa4e0dbc56593b905b4019bfe2316fc339049b8a69930350a0` |
| `helper_fork.c` | `a62a6a1e2bbb9cc6ef8a346ae9322c7b7435b1c7f0b6c3710b3aadb9cc2ba0b0` |
| `helper_setid.c` | `c609755d105122eea304f5fe12685d8dcc24064d77659295a11a76d42b86329f` |
| `harness.py`, `make_fixtures.py` | byte-identical, so the build commands and fixtures are unchanged |
| Run `34575558065` | attempt 1, `push`, conclusion success, head SHA `f417984`; the checkout printed it twice |
| Build 7 log | `freeze_verified: true`; zero `.c:<line>:<col>: warning` or `error` lines; `Ran 562 tests`; the runner step printed `NOT_RUN` |
| Build 7 executions | the D-7 flag and the three control flags appear zero times; no `set -x` line runs a `build/` path |
| Rust run `34575558173` | attempt 1, success, head SHA `f417984` |
| Post-freeze note | append-only (0 removed lines), accurate hashes, "no compiler ran", no claim of a new build, and says why Build 7 still applies |

The Python and definition changes after `f417984` do not change which C bytes Build 7 compiled.
No Build 8 is needed.

## T. Findings

| ID | Severity | Area | Current reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| AB7-B1 | **BLOCKER** | O7 construction and evidence; aggregate; D-7 one-shot | yes, on every honest run; reproduced by a Linux stand-in through the frozen harness functions | the launcher's own group sweep (launcher_spike.c 967–975) kills the non-setsid `helper_fork` descendant while it is blocked in the FIFO `open`, before `launch()` returns; `_descendant_alive` opens the read end only afterwards, so `fixture_descendant_alive` never holds and mandatory O7 is INVALID on every run; a D-7 would buy a trial whose aggregate is known in advance to be at best `MECHANISM_INCONCLUSIVE` | owner decision on how O7's fixture is established, e.g. a setsid descendant as P2/P4 use, or a rendezvous taken before the sweep; then fix, refreeze and one bounded review |
| AB7-I1 | IMPORTANT | S2/S7 PASS path; aggregate | yes; stand-in 0.1–0.2 % per repetition, P ≈ 18–33 % per S7 case; present since `f417984` via `no_helper_report`, not recorded by `7f97100` | launcher_spike.c 955 ends the drain on `exec_failed` before stdout's EOF is read; stdout is then reported `WriterRetainedAfterChildExit`, `no_executed_image` is unobservable and a correct `ExecFailed:CHDIR:EACCES` repetition is INVALID, so S7 becomes INVALID and the aggregate `MECHANISM_INCONCLUSIVE`; never a false PASS or FAIL | owner decision: e.g. accept the child's explicit CHDIR status record as the no-image proof, as section 9.6 already does for the report rules, or accept the risk; fix with AB7-B1 |
| AB7-M1 | MINOR | S2/S7; section 9.6 consistency | only with an unparseable report from the pinned helper after an image ran | a sentinel with a report that does not parse, on an `Exited`, `Signaled` or `TimedOut` run: `rule_process_disposition` returns no token (observations.py 1280–1290) before `no_executed_image` runs, so the repetition is INVALID where section 9.6 (962–963) makes the sentinel decisive for FAIL | decide the S2/S7 image-ran evidence before the uninterpretable branch, or amend the text; same batch |
| AB7-M2 | MINOR | O6; section 3 consistency | not with the frozen C: a held writer is always reported retained | O6's descendant is also non-setsid, so `retention_observed`'s liveness branch never holds and retention rests only on the receipt's own completeness; O6's section 3 FAIL condition `CompleteAtEof` therefore scores INVALID (probe; test_launch_exec_01_final.py 539 enshrines it), and `f9bbf39`'s "CompleteAtEof beside a live retaining descendant … a FAIL" is vacuous for O6 | resolve with AB7-B1 |
| AB7-N1 | BACKLOG_NONBLOCKING | BLOCKED records | — | S2/S7 BLOCKED records carry `measured.report_states {"None": 1}` and M2's carry null thread counts; `invocation` is correct and no landing fact is invented | cosmetic |
| AB7-N2 | BACKLOG_NONBLOCKING | FIFO hygiene | — | `release_setup_resources` (driver.py 855–889) never unlinks the case-private FIFO; the setup recreates it and FIFO data cannot outlive its last open end | hygiene |
| AB7-N3 | BACKLOG_NONBLOCKING | M2 | only with two anomalies | a control-arm non-return is a FAIL only while the threaded arm's trace is structurally valid; the unchanged checker's traced-evidence gate (checker.py 93–100) scores a missing primary trace INVALID first | backlog |

No previously accepted backlog item is reopened. `2591f35` carries an agent `Co-Authored-By`
trailer, already recorded as F417-N6. `f9bbf39`, `82b8746` and `ab74356` carry none, and neither
does this record's commit.

## U. Executed during the review

**Inspection.** Read-only `git` and `gh`: history, diffs, blob identities, the Build 7 and Rust
run metadata, and the Build 7 log. `git ls-remote` was run.

**Local validation.**

| Command | Result |
|---|---|
| `git diff --check`, on the tree and on `f417984..ab74356` | clean |
| `python tools/validate_docs.py` | PASS |
| `cargo fmt --check` | clean |
| the runner with `--verify-freeze` | `freeze_verified: true` |
| the runner with `--driver-completeness` | complete; nothing unposable |
| the runner with no flag | printed `NOT_RUN` |
| `python -m unittest discover -s tools/tests`, Windows | 628 tests, OK, 26 Linux-only skipped |
| the same suite, WSL Ubuntu, Python 3.12.3 | 628 tests, OK, none skipped |

`cargo test` was not run: no Rust or Cargo file changed, and run `34575558173` is green at
`f417984`.

**Scratchpad probes.** None writes into the repository.

1. A fabricated-observation probe built receipts as `launcher_spike.c` prints them. It fed them
   through `parse_spike_stdout`, `helper_report_state` and `exec_confirmation`, assembled
   observations the way `_launch_and_observe` and `pose()` do, and scored them with the frozen
   `evaluate` and checker. It ran 264 checks on Windows and on Linux, with no unexpected result.
2. A Linux stand-in of the O7 process tree against the frozen harness functions (section L).
3. A Linux stand-in of S2/S7's CHDIR-failure exit and the launcher's drain order (section F).

All three used Python processes only.

**No `launcher_spike`, no helper, no generated ELF and no preregistered case was executed; no
`Authorisation` was constructed; the D-7 flag was never passed.** Trial #2 ELF executions: ZERO.
Cases posed: ZERO. Valid trial count: ZERO.

## V. Authority and classification

TRIAL #2 D-7 REMAINS NOT AUTHORISED

LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT REMAINS ZERO

TRIAL #2 ab74356 IS NOT READY FOR D-7

**`TRIAL_2_NEEDS_OWNER_DECISION`.** The owner must decide two things. First, how O7's fixture is
established given the launcher's pre-reap group sweep; AB7-M2 follows from the same decision.
Second, whether S2/S7 may take the child's explicit CHDIR status record as the no-image proof when
stdout was not drained to its end, fixing AB7-M1 with it. The fixes, a new freeze and one bounded
independent review follow. No C source needs to change for either option named above. The
following are preserved unchanged: Trial #1, its freeze and result review, the reviews at
`43371c6` and `7f97100`, and freeze `ab74356`. This record is not amended by any correction that
follows it.
