# LAUNCH-EXEC-01 — Trial #2 final bounded independent review of freeze `ba41a3f`

Final bounded independent pre-D7 review of the Trial #2 freeze
`ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`. It supersedes `ab74356` with the AB7 correction
`1db4347`, which answers the [review of `ab74356`](HELM-LAUNCH-EXEC-01-TRIAL-002-AB74356-FINAL-REVIEW.md)
(`898a31c`). The reviewer did not implement the correction.

The scope is bounded to:

* the delta for AB7-B1, AB7-I1, AB7-M1 and AB7-M2;
* the new FIFO descriptor hygiene;
* the consistency of the definition and the manifest;
* the freeze's integrity;
* Build 7's applicability.

No broad 72-case audit was repeated. Nothing was fixed and no case was posed. No
`launcher_spike` or helper ELF ran, no `Authorisation` was constructed and no dispatcher was
created. No D-7 was granted and nothing was pushed.

**Classification: `TRIAL_2_READY_FOR_NEW_D7_DECISION`.**

The freeze is intact. Only `driver.py`, `observations.py`, the definition and the manifest moved,
and no case membership, class, prediction, safe set, traced flag or block cause moved. Build 7
still binds every C byte.

The pre-armed fixture signal closes AB7-B1. The harness opens the read end before the launcher
exists, the reader reaches no other process on any spawn path Python can take, and the byte
survives the launcher's unchanged group sweep. A Linux stand-in driven through the real
`_run_once` shows O7 and O6 PASS with the signal, INVALID without it, and FAIL on
`CompleteAtEof`. The following are sound under 182 fabricated-observation checks on Windows and on
Linux:

* S2/S7's authoritative CHDIR status, report sentinel and contradiction rule;
* O6's separation of posing from result;
* the O7 matrix;
* the 200-repetition reduction.

No BLOCKER, IMPORTANT or MINOR finding remains. Five backlog notes are recorded.

## A. Review target and history

| Item | Verified state |
|---|---|
| Branch / HEAD | `docs/helm-launch-architecture`, clean; HEAD `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` |
| Remote tip | `f41798455662579c3895f7a4af50bac17d47bb43`; local is eight commits ahead, none pushed |
| Ancestry | `f417984` → `2591f35` → `7f97100` → `f9bbf39` → `82b8746` → `ab74356` → `898a31c` → `1db4347` → `ba41a3f`, each the single parent of the next |
| `898a31c` | adds only the `ab74356` review record (480 insertions); the record classifies `TRIAL_2_NEEDS_OWNER_DECISION` |
| `1db4347` | the definition, `SOURCE-HASHES.json`, `driver.py`, `observations.py`, one new and three edited test files |
| `ba41a3f` | `PROJECT_STATE.md` (+28, 0 removed), `BUILD-EVIDENCE.md` (+24, 0 removed) and `SOURCE-HASHES.json`; its hashed sources are the committed blobs of `1db4347` |
| Earlier records | no commit after `898a31c` touches `docs/implementation/`; the reviews at `43371c6`, `7f97100` and `898a31c` and every earlier freeze stay addressable and unchanged |
| Trailers | `898a31c`, `1db4347` and `ba41a3f` carry no agent attribution |

No historical evidence was edited. The definition's superseded S2 clause is struck through in
section 3 rather than deleted, and section 9.6's O7 text points at section 9.7.

## B. Freeze integrity

Every hash was recomputed from the git blobs at `ba41a3f` and from the working tree, independently
of the repository's own verifier.

| Check | Result |
|---|---|
| `SOURCE-HASHES.json` | blob `6f000fac9a48625d3c9def18e16ae7ce1b61efa8`, SHA-256 `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94`, both exact |
| Source hashes | **17 / 17**; only `driver.py` and `observations.py` differ from `ab74356` |
| Definition hashes | **3 / 3**; only `LAUNCH-EXEC-01-DEFINITION.md` differs from `ab74356` |
| Unhashed files | exactly `BUILD-EVIDENCE.md` and `SOURCE-HASHES.json`, both declared |
| Partition | 72 total, 54 mandatory, 11 conditional, 7 recorded |
| Traced | 8: E1 E7 F4 F7 M1 M2 M3 M4 |
| Driver | 72 handlers, complete, 72 posable, 0 unposable; `--verify-freeze` true |
| Status | `NOT_RUN`, `d7_execution_authorised: false`, valid Trial #2 count ZERO |

`frozen_cases.py`, `checker.py`, `harness.py`, `make_fixtures.py`, `evidence.py`, `journal.py`,
`oracles.py`, the runner and `README.md` are byte-identical across `f417984`, `ab74356` and
`ba41a3f`.

Every `CasePlan` slot and registry was dumped from an extracted `ab74356` tree and from `ba41a3f`
and compared. The only differences are these:

* O6's and O7's `setup` (`fork_helper` → `fork_helper_prearmed`), `posed_when` and `note`;
* the new setup and the renamed posed check;
* `no_executed_image`'s declared reads.

Membership, class, prediction, safe set, traced membership and block causes are untouched.

## C. O7 final plan: `O7_FINAL_PLAN_SOUND`

| Field | Value |
|---|---|
| binary | `helper_fork` |
| setup | `fork_helper_prearmed` |
| arguments | `--retain-stdio --parent-exit 42 --lifetime-ms 20000` (`P_DESCENDANT_LIFETIME_MS`); no `--setsid` |
| channels | receipt, liveness; no payload, no stream recipe, no helper report |
| posed check | `fixture_descendant_signalled` |
| rule / assertion | `process_disposition` / `stderr_capture_failure_reported` |
| class and prediction | mandatory, `Exited:42` |

Stdout retention is not an O7 verdict condition. The FIFO path reaches `helper_fork` only as the
setup's appended `--liveness-fifo <path>`, and never enters `helper_args` or the durable record.

## D. The pre-armed FIFO and its descriptor: `O7_FIFO_ISOLATION_SOUND`

**Construction, from the code rather than the prose.** `_setup_fork_helper_prearmed` runs before
the boundary. It only names `<build>/<case>.fixture-signal` and removes a stale node; it opens
nothing.

For each launcher invocation, `_launch_and_observe` calls `_arm_fixture_signal` (driver.py 1426)
at 3251, after the argv is built and before either spawn path (3261 and 3275). That function:

1. unlinks the node and runs `mkfifo(path, 0o600)`; the node was `0600` under the test umask;
2. opens the read end with `O_RDONLY | O_NONBLOCK | O_CLOEXEC`;
3. stores the descriptor in the case's own `built` dict at once, so every later exit can close it;
4. requires `S_ISFIFO`, `get_inheritable() == False`, absence from `applied.pass_fds`, and an empty
   `poll(0)`. Any failure returns `not_posed`, which scores INVALID.

After `communicate()` returns, `_read_fixture_signal` reads the descriptor at 3347. `_run_once`'s
`finally` (3225–3228) calls `disarm_fixture_signal` on every exit, exceptions included, and
`release_setup_resources` calls it again, idempotently.

**Non-inheritance on the real spawn path.** The launcher is spawned by
`subprocess.Popen(argv, stdout=PIPE, stderr=PIPE, env=..., pass_fds=applied.pass_fds,
restore_signals=...)` (3275–3278); barrier cases add their control socket to `pass_fds`. Three
facts keep the reader out of the child:

* `close_fds` is left at its default `True`;
* O6's and O7's parent state is `none`, so `pass_fds` is `()`;
* the reader cannot share a number with a passed descriptor that is still open, and arming refuses
  one that does.

The WSL probe (Python 3.12.3) found `_USE_POSIX_SPAWN` true and `_USE_VFORK` true. The production
call took the `_fork_exec` path, which closes every descriptor at 3 and above that `pass_fds` does
not name. `O_CLOEXEC` closes it again at `execve`.

A child Python process recorded the `(st_dev, st_ino)` of every descriptor it held, and so did its
forked grandchild. The FIFO's identity was absent from both in every variant:

* the production call with `pass_fds=()`;
* the production call with an inheritable regular file in `pass_fds`, which was present, as the
  positive control;
* `close_fds=False`;
* a forced `posix_spawn`, confirmed by counting `os.posix_spawn` calls;
* vfork disabled;
* a harness with descriptor 0 closed, so the reader itself was descriptor 0.

The negative controls show the probe sees a real leak. An inheritable FIFO with `close_fds=False`
was present, and so was the FIFO when placed in `pass_fds`. Arming's `pass_fds` refusal is
therefore load-bearing, not decorative.

**Through the real harness.** A Python stand-in launcher was driven by the frozen `_run_once` and
`_launch_and_observe`, with only `spike_argv` redirected. On purpose, its child does not close
inherited descriptors. The launcher, the "helper" and the descendant each recorded their
descriptor identities, and the reader appeared in none of them, in all six scenarios.

## E. Who can write the positive byte: `O7_FIXTURE_SIGNAL_PROVEN_INDEPENDENT`

| Candidate | Why it cannot produce `fixture_descendant_signalled == true` |
|---|---|
| The Python setup | opens nothing; it removes a stale node |
| The Python harness | `_arm_fixture_signal`, `_read_fixture_signal` and `disarm_fixture_signal` open `O_RDONLY` only, and no call anywhere in `driver.py` writes `FIXTURE_SIGNAL_BYTE` |
| `launcher_spike` | never receives the descriptor (D). Its only `open` calls are the exec object (launcher_spike.c 684) and the work directory (686), so the FIFO path in its argv is forwarded, never opened |
| The direct child, before exec | the launcher's pre-exec code opens no pathname |
| `helper_fork`'s parent | reaches the FIFO only in the `pid == 0` branch after `fork()` (helper_fork.c 85–109) |
| A prior case or repetition | the node is unlinked and recreated per invocation; data dies with the last open end; an old node's writer cannot reach the new node (probe) |
| Another case | the path is per case (`O6.fixture-signal`, `O7.fixture-signal`, and `*.liveness` for the P-series); O6's `L` was invisible to O7's reader (probe) |

The positive fact is exactly one byte `L`. The harness reads at most one byte past it, and anything
else is false. A positive signal therefore needs an executed `helper_fork` whose descendant
reached its signalling path.

## F. The group sweep: `O7_SWEEP_INTERACTION_SOUND`

The correction changes the timing, not the launcher; `launcher_spike.c` is byte-identical. The
sequence is valid:

1. The reader is open before the launcher process exists (D).
2. The descendant runs no `setsid`, so it stays in the direct child's process group. Its
   `open(fifo, O_WRONLY)` returns at once because a reader exists, then it writes `L`, closes the
   FIFO and sleeps holding descriptors 1 and 2. On WSL the whole open-write-close took 2 ms.
   Without a reader, the same open was still blocked after 500 ms, which is AB7-B1's mechanism.
3. The direct child exits 42. The launcher sees the pidfd and drains until
   `child_end_ms + POST_EXIT_DRAIN_MS` (launcher_spike.c 957–964). The drain cannot end early,
   because the descendant holds both pipes, so line 956 never fires.
4. The drain expires. Both streams are reported `WriterRetainedAfterChildExit` (986–989).
5. `kill(-child, SIGKILL)` runs before the reap (967–975). The descendant dies and the launcher
   reaps, prints its receipt and exits.
6. `communicate()` returns, and the harness reads the byte already buffered.

A writer killed by `SIGKILL` after writing leaves `L` readable. That holds whether or not it had
closed the FIFO first (probe). Nothing depends on the descendant surviving past `launch()`: its
`descendant_alive_after_launch` stays `None`. The stand-in ran the full order and gave O7 PASS
with `group_sweep_issued: true`.

The window is at least `POST_EXIT_DRAIN_MS` (2000 ms), and its length is structural. The
descendant must still be scheduled inside it. A stand-in descendant delayed 3.5 s was swept before
signalling and scored INVALID, which is section 9.7's preregistered outcome (BA41-N3).

## G. O7 posing evidence: `O7_POSING_EVIDENCE_SOUND`

`_check_fixture_descendant_signalled` declares and reads only `fixture_descendant_signalled` and
requires the value `True` exactly. It never reads:

* stdout or stderr completeness;
* the exit status;
* the receipt's outcome;
* `WriterRetainedAfterChildExit`.

With a perfect receipt beside a false, `None` or truthy-but-not-`True` signal, the check is false
and the case is INVALID. With no receipt at all it is true, and the receipt is then judged as the
result. `_evaluate_repetition` applies the posed check after the non-return test and before the
rule (2753–2877). A non-return is therefore a FAIL, whatever the signal.

## H. O7 exec evidence: `O7_EXEC_EVIDENCE_SOUND`

`exec_confirmation(..., fixture_signalled=True)` returns `exec_reached` (observations.py 359–414).
The payload's contradiction is still decided first, although O7 declares no payload. An authentic
`L` shows three things:

* `helper_fork` executed;
* its fork path ran;
* its descendant reached the write.

It does not show survival. The code, section 9.7, section 2 and the manifest's `fixture_signal`
block all use that narrower meaning.

`descendant_alive_after_launch` now serves only the P-series rendezvous and P4's
`retention_observed`. It is not O7's positive fact, and the O7 durable record's `measured` block
holds only `fixture_descendant_signalled`.

## I. O7 scoring: `O7_SCORING_SOUND`

Each row was built as `launcher_spike.c` prints receipts, passed through the frozen parser, then
`driver.evaluate`, then the checker. It was run with stdout both `WriterRetainedAfterChildExit` and
`CompleteAtEof`, and stdout never changed a status.

| Signal | Disposition | stderr completeness | Status |
|---|---|---|---|
| true | `Exited:42` | `WriterRetainedAfterChildExit` | PASS eligible |
| true | `Exited:42` | `CompleteAtEof` | FAIL: `capture_failure_not_reported`, `mechanism_outcome` `Exited:42` |
| true | `Exited:0` or `Exited:1` | `WriterRetainedAfterChildExit` | FAIL |
| true | `Exited:0` | `CompleteAtEof` | FAIL |
| true | `Signaled:SIGKILL` | `WriterRetainedAfterChildExit` | FAIL |
| true | `Exited:42` | absent (view) or unparseable (receipt) | INVALID |
| true | `Exited:0` | absent (view) | FAIL: the decisive contradiction controls |
| true or false | launch did not return | — | FAIL |
| false, `None`, or `9` | a perfect receipt | — | INVALID |

## J. O7 durable evidence: `O7_DURABLE_EVIDENCE_SOUND`

The PASS, both FAIL and the INVALID records went through `evidence.public_sanitiser`, an fsynced
journal and `replay`, and came back with every fact intact. The live stand-in records did the same
through the real `_run_once`. Each record separates five things:

| Fact | Where it is kept |
|---|---|
| fixture signal | `posing_evidence.measured.fixture_descendant_signalled`, `posed_check {name, held}`, `fixture.signal_arming {armed_before_launch, reader_inheritable, reader_passed_to_launcher, fresh_fifo}` |
| mechanism invoked | `posing_evidence.invocation` |
| direct-child disposition | `outcome`, or `mechanism_outcome` when an assertion replaced it |
| stderr completeness | the assertion's `detail` |
| assertion result | the assertion's `result` |

The following never appeared in the raw journal:

* the FIFO's absolute path or name;
* the build or temporary directory;
* a descriptor number or inode;
* a pid;
* `capture_prefix_base64`;
* the report sentinel's bytes;
* the stand-in's descriptor table.

`_problem` keeps only the operation and the errno name, so an arming failure cannot carry the path
either. BA41-N4 notes that a not-posed record carries no receipt facts.

## K. S2/S7 exec status: `S2_S7_EXEC_STATUS_CLASSIFICATION_SOUND`

`assert_no_executed_image` now reads `report_sentinel_seen`, `spike` and `report_state`, and the
manifest's reads match. `_explicit_pre_exec_status` accepts only a `process_disposition` of
`ExecFailed` with a stage and errno the frozen table renders.

A full record is authoritative for three reasons:

* the launcher sets `exec_failed` only on a complete record;
* `child_fail` writes the record and `_exit`s;
* after a successful `execveat`, the `CLOEXEC` status write end is gone.

| S2 and S7 repetition, per case | Status |
|---|---|
| forced state not landed | INVALID |
| `ExecFailed:CHDIR:EACCES`, stdout retained (not drained), no sentinel | PASS |
| `ExecFailed:CHDIR:EACCES`, stdout complete and empty | PASS |
| `ExecFailed:CHDIR:EACCES`, non-sentinel bytes, or an undecodable prefix | PASS: there is no positive contradiction |
| `ExecFailed:SETPGID:EPERM` | FAIL |
| `ExecStatusIndeterminate`; clean-EOF `Exited:0` | FAIL |
| `Exited:0` or `Exited:127` with a complete report | FAIL |
| `Exited:0` or `Signaled`, stdout retained, no status, no sentinel | INVALID |
| `ExecFailed:CHDIR` with an errno outside the frozen table | INVALID |

No stdout end-of-file is required. That closes AB7-I1: the stop-before-EOF of launcher_spike.c
955 can no longer turn a correct repetition INVALID.

## L. S2/S7 sentinel: `S2_S7_SENTINEL_CLASSIFICATION_SOUND`

`report_sentinel_seen` (observations.py 333–348) takes three values:

* `True` when the decoded retained prefix contains the frozen sentinel anywhere, with no parse
  required;
* `False` for a decoded prefix without it;
* `None` when no prefix decodes.

The assertion also treats the `complete`, `malformed` and `truncated` report states as seen, and
each of those states exists only with the sentinel. `False` from an incomplete prefix never holds
the assertion.

| Observation | Status |
|---|---|
| sentinel with a malformed report, `Exited:0`, complete stream | FAIL (`executed_image_observed`) |
| sentinel with a truncated report, stream retained | FAIL |
| sentinel with a malformed report, `Signaled` or `TimedOut` | FAIL |
| no status, no sentinel, incomplete stream | INVALID |

The rule renders no token when a report is malformed. `_evaluate_repetition` (2813–2830) then lets
only `DECISIVE_WITHOUT_TOKEN_ASSERTIONS` decide, and that set is `{no_executed_image}`, used only by
S2 and S7. That closes AB7-M1, and no other case gained a no-token FAIL path. The retained bytes
stay in-process: only the boolean, and per-case counts of it, reach the record.

## M. S2/S7 contradiction: `S2_S7_CONTRADICTION_HANDLING_SOUND`

Fabricated: `ExecFailed:CHDIR:EACCES` together with a sentinel. It was run with a complete report,
a malformed report, a truncated report on a retained stream, and a sentinel after other bytes.

In every variant the rule's token equals the prediction, and the case is a FAIL:

* `outcome` is `executed_image_observed`;
* `mechanism_outcome` keeps `ExecFailed:CHDIR:EACCES`;
* the assertion is `violated`, with the reason "contradictory evidence: the child wrote the
  explicit pre-exec status record ExecFailed:CHDIR:EACCES and a helper-report sentinel was observed
  on descriptor 1; a record that contradicts itself cannot pass".

A primary token that matches the prediction cannot rescue the contradiction.

## N. S7 repetitions

Every repetition passed through the frozen `evaluate` and checker, with `_evaluate_repetition`
wrapped to count calls. The count was 200 on every run.

| S7, 200 repetitions | Status | `decided_by_trial` |
|---|---|---|
| all correct: retained-stdout and complete-stdout CHDIR mixed | PASS | — |
| one sentinel FAIL at 0, 100 or 199 | FAIL | 0, 100 or 199 |
| one unlanded repetition at 57, 199 PASS | INVALID | 57; counts 199 / 0 / 1 |
| INVALID at 10 and FAIL at 150 | FAIL | 150 |
| FAIL at 10 and INVALID at 150 | FAIL | 10 |
| a contradiction at 199 | FAIL | 199; `report_sentinel_seen` `{False: 199, True: 1}` |

Every repetition's status and outcome is durable, and the 200-entry list survived the journal.

## O. O6: `O6_CLASSIFICATION_SOUND`

The construction is unchanged, apart from its setup:

* `helper_fork --prewrite 512 --retain-stdio --parent-exit 0 --lifetime-ms 20000`;
* no `--setsid`;
* setup `fork_helper_prearmed`;
* posed check `fixture_descendant_signalled`.

`retention_observed` now poses P4 only. The recipe's digest was recomputed independently and equals
the plan's.

| O6 | Status |
|---|---|
| no signal (`False` or `None`), even beside a retained receipt | INVALID; the old receipt route no longer poses O6 |
| signal, `Exited:0`, stdout `WriterRetainedAfterChildExit`, recipe | PASS eligible |
| signal, `Exited:0`, stdout `CompleteAtEof` | FAIL: `completeness_mismatch`, `mechanism_outcome` `Exited:0` |
| signal, `Exited:1`, `Signaled:SIGKILL`, `TimedOut` or `TimedOut:KilledByLauncher:*` | FAIL |
| signal, a 400-byte payload | FAIL |
| launch did not return | FAIL |

Live, the stand-in's descendant closed its stdout. The receipt then carried stdout `CompleteAtEof`
beside the signal, and O6 scored FAIL. AB7-M2's unreachable FAIL is now reachable.

## P. Shared machinery, separate state

O6 and O7 share one setup and three functions. All runtime state lives in the case's own `built`
dict, under the key `_fixture_signal_fd`, and the FIFO path is per case. The driver has no
module-level cache of any signal, and no `global` statement.

The probes confirmed four things:

* setup holds no descriptor;
* an unarmed case reads `None`;
* O6's byte is invisible to O7's reader;
* re-arming drops the previous invocation's descriptor.

No O6 observation can satisfy O7's check, and no O7 observation O6's.

## Q. Stale S2 note: `STALE_S2_NOTE_NON_NORMATIVE_AND_SUPERSEDED`

`frozen_cases.py` is byte-identical, and its S2 `note` still says "its absence is the independent
proof that the image never ran". Section 9.7 supersedes that clause by name, "in section 3 and in
`frozen_cases.py`'s S2 note; that note is not edited", and section 3 strikes it through. Five
things support this classification:

1. **Not executable.** Nothing reads a `frozen_cases` note. `checker.py`, `driver.py`,
   `evidence.py`, `journal.py` and `oracles.py` never consume the field. The vocabulary sources
   listed in the manifest exclude it, and `driver.py` declares its own `CasePlan.note`
   documentation-only (`CASEPLAN_FIELD_CONSUMERS`).
2. **Precedence is explicit and specific.** Section 9.7 says that where it differs from sections 2,
   3, 9.5 or 9.6 about S2, S7, O6 or O7, section 9.7 governs. It also names the note itself.
   `SOURCE-HASHES.json` `s2_s7.source` records the same supersession inside the freeze.
3. **The general rule does not reach it.** The definition's opening rule makes the manifest
   authoritative over prose for membership, classes, schedules and expectations. S2's frozen
   expectation, `ExecFailed:CHDIR:EACCES`, is unchanged.
4. **No observation separates the readings.** The note's sentence "no helper report may be
   received" still holds: a sentinel is a FAIL under both. Its "absence is the proof" clause
   never demanded a complete stream, so a correct CHDIR record with no sentinel passes under both
   readings. With no explicit status, the rule, not the note, decides the token.
5. **A post-trial reviewer has no legitimate choice.** There is one active contract.

BA41-N1 records the textual tension and a second stale documentation note in `driver.py`.

## R. Preregistration: `TRIAL2_BA41_PREREGISTRATION_COMPLETE`

Before execution, the following are frozen:

* definition sections 2, 3, 9.5, 9.6 and 9.7;
* the manifest's `posing_versus_showing` blocks `fixture_signal`, `o6`, `o7`, `s2_s7`,
  `decisive_without_token` and `posed_check_inputs`;
* the plans' comments.

Together they fix these points.

* **O7.** The pre-armed reader and its flags and checks; the signal's meaning (executed, forked,
  signalling path) and its timing (in the descendant's first actions, before the sweep); no
  `--setsid`; the unchanged group sweep; posing by the signal alone; the signal as exec evidence;
  `WriterRetainedAfterChildExit` as the result; the full PASS/FAIL/INVALID matrix.
* **S2/S7.** The barrier's work-directory landing; the authority of the CHDIR status; the sentinel
  as positive contradiction whatever the parse; no clean-EOF requirement; the contradiction FAIL;
  the 200-repetition reduction.
* **O6.** Posing by the signal; retained completeness as the result; `CompleteAtEof` as a FAIL.

The manifest's assertion reads, violation tokens, posed-check inputs and decisive-without-token set
equal the code's. Every observation built in this review has a determined status without an owner
choice.

## S. Build 7: `BUILD_7_STILL_BINDS_BA41_C_SOURCES`

| Item | Verified |
|---|---|
| C sources `f417984` → `ab74356` → `ba41a3f` | identical git blobs, SHA-256 equal to the manifest: `launcher_spike.c` `0a45447b…0d622`, `helper_report.c` `99770ed8…54711`, `helper_alt.c` `7993aa63…99113`, `helper_dynamic.c` `b3c398d8…350a0`, `helper_fork.c` `a62a6a1e…ba0b0`, `helper_setid.c` `c609755d…6329f` |
| `harness.py`, `make_fixtures.py` | byte-identical, so the build commands and fixtures are unchanged |
| Run `34575558065` | `gh`: attempt 1, `push`, success, head `f41798455662579c3895f7a4af50bac17d47bb43` |
| Build 7 log | `freeze_verified: true`; zero `.c:<line>:<col>: warning` or `error` lines; `Ran 562 tests`; the runner printed `NOT_RUN` twice; the D-7 flag and `Authorisation(True)` appear zero times |
| `BUILD-EVIDENCE.md` note in `ba41a3f` | append-only (24 lines, 0 removed), hashes accurate, "no compiler ran", no claim of a new build |

The changes in `driver.py`, `observations.py` and the definition do not change which C bytes
Build 7 compiled. No Build 8 is needed.

## T. Findings

| ID | Severity | Area | Current reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| BA41-N1 | BACKLOG_NONBLOCKING | preregistration wording | no observation gives a different verdict | the definition's opening rule makes the manifest authoritative over prose, which reads the other way from section 9.7's named supersession of `frozen_cases.py`'s S2 note; `driver.py`'s S2 `CasePlan` note still calls "the report's decisive ABSENCE" the result; both notes are documentation-only and unread | align the wording in a later freeze if ever re-cut; not a pre-D7 change |
| BA41-N2 | BACKLOG_NONBLOCKING | FIFO semantics | fail-closed only | arming's freshness check relies on a Linux rule. A FIFO read end whose writer never connected polls not ready, although a `read()` there returns 0. Verified on WSL 6.6.87.2; a kernel that reported `POLLHUP` early would make every O6/O7 run INVALID, never PASS | the POSIX hygiene tests would show it on the trial host |
| BA41-N3 | BACKLOG_NONBLOCKING | section 9.7 wording | only under ≥ 2 s scheduler starvation of the descendant | "structural rather than a timing hope": the window's length (≥ `POST_EXIT_DRAIN_MS`) is structural, but the descendant must be scheduled inside it; missing it is INVALID, as section 9.7 preregisters (stand-in, 3.5 s delay) | wording only |
| BA41-N4 | BACKLOG_NONBLOCKING | durable evidence | — | a not-posed O6/O7 record keeps the signal, the arming facts and the invocation, but not the receipt's disposition or completeness; that is consistent with the `posing_evidence` contract, which records only what the verdict relied on | backlog |
| BA41-N5 | BACKLOG_NONBLOCKING | test quality | — | `test_the_case_table_is_untouched` re-asserts the manifest total four times instead of checking each case, and no test compares `frozen_cases.py` with an earlier freeze; this review's blob and plan comparison covers both | backlog |

No BLOCKER, IMPORTANT or MINOR finding. AB7-N1 to N3 and every earlier backlog item stay as
recorded and are not reopened.

## U. Executed during the review

**Inspection.** Read-only `git` and `gh`: history, diffs, blob identities, the Build 7 run metadata
and its log.

**Local validation.**

| Command | Result |
|---|---|
| `git diff --check`, on the tree and on `ab74356..ba41a3f` | clean |
| `python tools/validate_docs.py` | PASS |
| `cargo fmt --check` | clean |
| the runner with `--verify-freeze` | `freeze_verified: true` |
| the runner with `--driver-completeness` | complete; nothing unposable |
| the runner with no flag | printed `NOT_RUN` |
| `python -m unittest discover -s tools/tests`, Windows, Python 3.14.3 | 667 tests, OK, 40 Linux-only skipped |
| the same suite, WSL Ubuntu, Python 3.12.3, kernel 6.6.87.2 | 667 tests, OK, none skipped |

`cargo test` was not run: no Rust or Cargo file changed since `f417984`, and the recorded Windows
path issue is left alone.

**Scratchpad probes.** None writes into the repository.

1. **Fabricated observations.** Receipts built as `launcher_spike.c` prints them, run through the
   frozen parser, the driver's `evaluate` and the checker, with journal round-trips. 182 checks on
   Windows and 182 on Linux, all as expected.
2. **FIFO semantics and spawn-path isolation**, on Linux (D, E, F).
3. **The frozen `_run_once` against a Python stand-in launcher and `helper_fork`.** Six scenarios
   and an injected exception after arming (D, F, J, O).
4. **A plan and registry comparison** between extracted `ab74356` and `ba41a3f` trees (B).

All probes used Python processes and inert FIFOs only.

**No `launcher_spike`, no helper, no generated ELF and no preregistered case was executed; no
`Authorisation` was constructed; the D-7 flag was never passed.** Trial #2 ELF executions: ZERO.
Cases posed: ZERO. Valid trial count: ZERO.

## V. Authority and classification

TRIAL #2 D-7 REMAINS NOT AUTHORISED

LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT REMAINS ZERO

TRIAL #2 ba41a3f IS READY FOR A NEW OWNER D-7 DECISION

**`TRIAL_2_READY_FOR_NEW_D7_DECISION`.** Every load-bearing result is sound and the
preregistration is complete. No BLOCKER or IMPORTANT finding is unresolved. This review grants
nothing: a D-7 needs a new explicit owner decision and a new manual dispatch. The following are
preserved unchanged: Trial #1, its freeze and result review, the reviews at `43371c6`, `7f97100`
and `898a31c`, and freezes `ab74356` and `ba41a3f`. This record is not amended by anything that
follows it.
