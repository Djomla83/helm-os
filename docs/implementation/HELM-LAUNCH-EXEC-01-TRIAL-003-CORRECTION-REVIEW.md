# LAUNCH-EXEC-01 — Trial #3 correction candidate: bounded independent review

**THIS REVIEW DOES NOT MODIFY TRIAL #2, FREEZE ANYTHING, OR AUTHORISE TRIAL #3.**

| | |
|---|---|
| Reviewed candidate commit | `7f98d4dcdb098abf96fe5e15e314194f5df6cb20` |
| Parent | `b20c2b0f41ff01ca13b451df34a59e57f7da75fe` |
| Preceding owner/postmortem commits | `a73e159ac4ebfaea3c04cd80ab458ebb402c4f0d`, `b20c2b0f41ff01ca13b451df34a59e57f7da75fe` |
| Remote milestone (`origin/docs/helm-launch-architecture`) | `1f0b86a13ece63badf28d7cef97d580ebc359a99`, unchanged after a read-only fetch |
| Trial #2 freeze | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`, run `34640280964` |
| Reviewer | a fresh review session that did not implement `7f98d4d` |
| Pushed | NO |

This is the one bounded independent correction review the
[candidate record](../experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json) and
[definition section 10](../experiments/LAUNCH-EXEC-01-DEFINITION.md#10-trial-3-correction-candidate--not-frozen-not-authorised-not-run)
require. Its scope is the owner's
[postmortem decisions](../DECISIONS.md#trial-002-postmortem-decisions) and the
[postmortem diagnostics](HELM-LAUNCH-EXEC-01-TRIAL-002-POSTMORTEM-DIAGNOSTICS.md). It is not a
72-case experiment review, not a freeze, not Build 8 and not a D-7. No finding was fixed.

## 1. Starting state

* `git status --short` was clean; the branch was `docs/helm-launch-architecture`; `HEAD` was
  `7f98d4d`.
* After `git fetch origin`, the remote branch was still `1f0b86a`, and `1f0b86a` is an ancestor
  of `HEAD`. Local history is `1f0b86a → a73e159 → b20c2b0 → 7f98d4d`, a normal descendant.

## 2. Exact delta `b20c2b0..7f98d4d`

| Status | Path | +/− |
|---|---|---|
| M | `docs/PROJECT_STATE.md` | +20 |
| M | `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md` | +276 / −2 |
| M | `docs/experiments/launch-exec-01/BUILD-EVIDENCE.md` | +36 (appended) |
| A | `docs/experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json` | +149 |
| M | `docs/experiments/launch-exec-01/driver.py` | +274 / −66 |
| M | `docs/experiments/launch-exec-01/frozen_cases.py` | +19 / −5 |
| M | `docs/experiments/launch-exec-01/helper_fork.c` | +35 / −4 |
| M | `docs/experiments/launch-exec-01/helper_report.c` | +31 / −16 |
| M | `docs/experiments/launch-exec-01/launcher_spike.c` | +24 / −2 |
| M | `docs/experiments/launch-exec-01/make_fixtures.py` | +32 / −4 |
| M | `docs/experiments/launch-exec-01/observations.py` | +249 / −25 |
| M | `tools/tests/test_launch_exec_01_ab7.py` | +22 / −4 |
| M | `tools/tests/test_launch_exec_01_delta.py` | +5 / −2 |
| M | `tools/tests/test_launch_exec_01_driver.py` | +3 / −1 |
| M | `tools/tests/test_launch_exec_01_final.py` | +21 / −4 |
| A | `tools/tests/test_launch_exec_01_trial3.py` | +1388 |

No workflow, no experiment evidence and no other source changed. The same set, relative to
`ba41a3f`, adds nothing under `docs/experiments/launch-exec-01/` beyond these files.

**Semantic movement, derived by importing the `ba41a3f` and candidate modules side by side:** the
case table differs only in `T1.predict` and `S4.predict`; the plan table differs only in `S4`
(`channels`, `helper_args`, `posed_when`, `expected_marker`, `assertions`, `rule`). Added
registry names are exactly `launcher_receipt_claim`, `helper_exit_corroborated` and
`helper_report_complete`; none was removed. The remaining changes are posing machinery and
receipt corroboration bound to X2b/X2c/X4, M2, E4, E6/E6c, O6/O7, P1/P2/P4 and R3. Two shared
effects were checked and move no unrelated semantic: E8 and X2 now also require their generated
fixture's declared mode (E8 keeps Trial #2's effective `0644`; X2 is refused at admission whatever
the mode), and P3 and T5 receive the same absolute `helper_fork` path, which neither case's rule
reads. N3 is untouched.

**CORRECTION_SCOPE_BOUND_SOUND**

## 3. Trial #2 historical integrity

* The five preserved evidence files are byte-identical to `8a9dc77`, the result review to
  `1f0b86a`, and the postmortem diagnostics to `b20c2b0`; no commit after them touches these paths.
* `SOURCE-HASHES.json` is Git blob `6f000fac9a48625d3c9def18e16ae7ce1b61efa8` both at `ba41a3f`
  and in the working tree, and every `sha256` and `definition_sha256` entry of that manifest equals
  the hash of the corresponding blob at `ba41a3f`.
* **Independent replay.** Every Python module the manifest hashes was extracted from `ba41a3f`,
  checked against its manifest hash, and run in an isolated interpreter (`-I -B`). Three
  derivations agree with the published document for all 72 status/reason pairs, with zero
  mismatches: the `ba41a3f` checker re-scoring each published case record; the `ba41a3f` journal
  replay of `journal.jsonl`; and the journal's own `case_completed` lines. Each yields
  **59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED**, `MECHANISM_REJECTED`, failing cases
  `X2b, X2c, X4, T1, S4, M2`. The journal is not torn, `trial_status` is `TRIAL_COMPLETED` and
  `d7_consumed` is true. The frozen predictions read from `ba41a3f` are still `T1 = TimedOut` and
  `S4 = Exited:7`.

**TRIAL2_HISTORICAL_INTEGRITY_SOUND**

## 4. NOT_FROZEN candidate contract

The record says `state: NOT_FROZEN`, `trial_3_authorised: false`, `trial_3_d7: null`,
`trial_3_executed: false`, `build_7_binds_candidate: false`, and carries no manifest hashes of its
own. It claims no D-7, no Trial #3 execution authority, no Trial #3 trial count, no frozen manifest
and no Build 8 evidence. Definition section 10 says the same.

**Drift derived from bytes**, working tree against the `ba41a3f` manifest:

* `sha256` (17 entries): `driver.py`, `frozen_cases.py`, `helper_fork.c`, `helper_report.c`,
  `launcher_spike.c`, `make_fixtures.py`, `observations.py`. `README.md`, `checker.py`,
  `evidence.py`, `harness.py`, `journal.py`, `oracles.py`, `run_launch_exec_01.py`, `helper_alt.c`,
  `helper_dynamic.c` and `helper_setid.c` are unchanged.
* `definition_sha256` (3 entries): only `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md`;
  `ADR-0024` and `HELM-LAUNCH-ARCHITECTURE.md` are unchanged.
* The two changed paths the manifest does not hash are `BUILD-EVIDENCE.md` (declared
  `not_hashed_here`, append-only; the diff only appends) and the new candidate record.

That set equals the record's `sources_differing_from_trial_2_freeze` and
`definition_files_differing_from_trial_2_freeze` exactly. `run_launch_exec_01.py --verify-freeze`
on the candidate prints `freeze_verified: false`, lists exactly the seven drifted sources and exits
1; the default invocation still refuses with `NOT_RUN` and exit 3. The old manifest is therefore not
accepted for this candidate.

**NOT_FROZEN_DELTA_ACCOUNTING_SOUND**

## 5. Freeze-plus-delta test discipline

The changed contract tests compare the code with the Trial #2 manifest **plus** the record's
explicit delta. Their expected hashes come from `SOURCE-HASHES.json`, whose blob is pinned
(`6f000fac`) by `Trial2StaysImmutable`, never from the files under test; the AB7 C-source check
also cross-checks each frozen hash against Build 7's own table in `BUILD-EVIDENCE.md`.

Each tamper below was applied to a disposable copy of `docs/` and `tools/tests/`, and only the
manifest-coupled classes (`CandidateRecord`, `AB7Preregistration`, `PreregistrationAgrees`) were run:

| Tamper | Result |
|---|---|
| none (baseline) | 14 tests OK; `verify_freeze` false |
| undeclared Python drift (`evidence.py`, `checker.py`) | drift test FAILS |
| undeclared C drift (`helper_alt.c`) | drift test and AB7 C test FAIL |
| undeclared definition drift (`ADR-0024`) | drift test FAILS |
| declared drift missing (`make_fixtures.py` reverted) | drift, runner and contract-delta tests FAIL |
| declared C drift missing (`helper_fork.c` reverted) | drift, runner and AB7 C tests FAIL |
| a name declared that has not drifted | drift and runner tests FAIL |
| a new posed check not declared in the record | `PreregistrationAgrees` FAILS |
| every frozen input restored to `ba41a3f` bytes | `verify_freeze` true — the frozen state is distinguishable from the candidate |

Undeclared drift fails, missing declared drift fails, and candidate state is distinguishable from
frozen state. By design the candidate bytes of the declared files are pinned only by commit
`7f98d4d`, not by a hash; a future freeze still needs an explicit new manifest.

**FREEZE_PLUS_DELTA_CONTRACT_SOUND**

## 6. Static global shape

Imported without executing anything: **72** total, **54** mandatory, **11** conditional,
**7** recorded; traced exactly `E1 E7 F4 F7 M1 M2 M3 M4`; `driver.completeness()` complete with 72
driver entries and no missing, duplicate, unknown or unresolved entry; `unposable_cases()` empty,
so 72 are statically posable and 0 unposable. Membership order is identical to `ba41a3f`, and no
class, membership or traced flag moved. `checker.py` is byte-identical to the manifest, so
PASS / FAIL / INVALID / BLOCKED and the aggregate are unchanged. `STATUS_PRECEDENCE` is still
`(FAIL, INVALID, PASS)` and `reduce_repetitions` is unchanged. No result-driven exception was
added.

**CANDIDATE_GLOBAL_SHAPE_SOUND**

## 7. Per-correction verdicts

### 7.1 X2b / X2c / X4 — fixture modes

`make_fixtures.FIXTURE_MODES` declares one mode per fixture and `write()` applies it with one
`os.chmod(path, FIXTURE_MODES[name])` after writing, so the umask cannot defeat it and a rewrite
restores it. Under umask `022` the reviewer observed `script_fixture.sh` and
`unloadable_in_cohort.elf` at `0755` and `helper_foreign.elf` and `magic_only.bin` at `0644`;
`ba41a3f` gave all four `0644`. Digests equal Trial #2's build identity. `_generated_fixture`
refuses to pose E8, X2, X2b, X2c or X4 when the mode differs from the declaration; a `0600`
`unloadable_in_cohort.elf` returned the not-posed reason.

Kernel path, reasoned and not executed: `execveat` checks `MAY_EXEC` on the object before any
binfmt handler, which produced Trial #2's `EACCES`. With `0755`, X2b's `O_CLOEXEC` script reaches
`binfmt_script`, whose `/dev/fd/N` interpreter path is inaccessible, so `ENOENT` is reachable; X2c's
non-CLOEXEC descriptor lets the interpreter receive and read `/dev/fd/N`; X4's `e_phnum = 0` image
reaches `binfmt_elf`, which refuses it with `ENOEXEC`. The fixtures were not executed.

**X_CLUSTER_CORRECTION_SOUND**

### 7.2 T1

In `launcher_spike.c` the deadline sends SIGTERM through the pidfd and starts the grace window;
a timed-out child reaped with any `si_code` other than `CLD_EXITED` becomes `KilledByLauncher`,
with `term_signal` taken from `si_status`. T1's helper keeps SIGTERM's default action, so the
intended token is `TimedOut:KilledByLauncher:SIGTERM`. The checker compares a single prediction by
equality. Scored with fabricated receipts: the SIGTERM path PASSes; bare `TimedOut`,
`KilledByLauncher:SIGKILL`, `ExitedDuringGrace:9` and `TerminationFailed` FAIL; a `wait_si_code`
contradicting the sub-disposition is INVALID, never PASS.

**T1_CORRECTION_SOUND**

### 7.3 S4 — load-bearing

* **The launcher was not taught that EOF proves exec.** The `launcher_spike.c` delta touches only the
  signal/`wait_si_code` receipt fields; the exec-status loop is unchanged, and the receipt has no
  exec-success field.
* **Evidence authority.** `rule_launcher_receipt_claim` evaluates `process_disposition` on a view with
  `report`, `report_state` and `payload_is_recipe` removed and `exec_confirmation(spike, None)`.
  With every positive evidence present (report, sentinel, fixture signal, recipe) the claim is
  still `ExecStatusIndeterminate`. Independent evidence poses S4 (`helper_report_complete`) and is
  asserted beside the claim (`executed_marker`, `helper_exit_corroborated`); it never becomes the
  claim.
* **The race is retained and S4 is not S5.** The parent still sleeps 200 ms after `clone3`, before
  closing its pipe copies and polling. `helper_report --exit 7` writes one small report into an
  empty 64 KiB pipe and exits, so status hangup, stream end-of-file and pidfd readiness still
  arrive together at the first poll. The launcher's handling of that simultaneous readiness is
  still decisively checked: `helper_exit_corroborated` requires the receipt's own `Exited` with
  status 7 and `CLD_EXITED`. S5 keeps `--die-before-exec` and is posed by the report's absence. The
  shared token is not a duplication: the causal condition (post-fork delay versus pre-exec death)
  and the posing evidence differ.
* **Classification**, scored with fabricated real-shaped observations:

| Observation | Candidate |
|---|---|
| `Exited` 7, `CLD_EXITED`, complete report naming `helper_report` and declaring 7 | PASS, `ExecStatusIndeterminate` |
| receipt `Exited` 0; report declaring 3; `Signaled:SIGKILL`; `TimedOut:KilledByLauncher`; receipt `ExecStatusIndeterminate`; `ExecFailed` beside a report | FAIL, `helper_exit_contradicted` |
| report naming `helper_alt` | FAIL, `executed_body_mismatch` |
| no report, truncated report | INVALID (not posed) |
| `Exited` 7 beside `CLD_KILLED` | INVALID (assertion unobservable) |
| admission refusal, or an explicit pre-exec status record with no report | INVALID — see finding **R-M1** |

The receipt's self-assertion alone cannot create a PASS: without a complete report S4 is not posed.

**S4_CONSERVATIVE_RACE_CORRECTION_SOUND**

### 7.4 M2 — load-bearing

`select_direct_child_clone` reads `clone_args.flags` from the braces of the joined `clone3`
record (`_CLONE3_LINE` then `_FLAGS`); symbolic names are kept, and numeric parts are decoded for
`CLONE_PIDFD` (0x1000) and `CLONE_THREAD` (0x10000). Fragments are rejoined by
`join_trace_fragments` before selection, and the structural integrity gate still runs first. The
window follows only the selected child task. Fabricated traces, parsed by both implementations:

| Trace | `ba41a3f` | Candidate |
|---|---|---|
| three thread clones, then the `CLONE_PIDFD` clone | thread 301 | child 222 |
| numeric thread flags `0x3d0f00`, then the process clone | thread 301 | child 222 |
| split `<unfinished>`/`resumed` process clone after a thread | thread 301 | child 222 |
| thread clone returning `EAGAIN`, then the process clone | no window | child 222 |
| a thread clone after the process clone | child 222 | child 222 |
| process clone returning `EPERM` beside a thread | thread 301 | no window |
| two `CLONE_PIDFD` process clones | thread 301 | no window |
| a process clone without `CLONE_PIDFD` among threads | thread 301 | no window |
| undecodable flags (`~CLONE_X`) among several | thread 301 | no window |
| a lone `CLONE_THREAD` clone | window | no window |
| a lone process clone without `CLONE_PIDFD` | child 222 | child 222, so M3 keeps its decisive FAIL |

Selection reads trace facts only, never the expected outcome. `parse_pidfd_acquisition` describes
the selected clone and still counts every `clone3`, so M3's single-clone requirement is unchanged.
The traced single-clone cases use `helper_report` or `helper_dynamic`, which make no `clone3`. No
strace was run.

**M2_PROCESS_CLONE_CORRELATION_SOUND**

### 7.5 E4

In a scratch nested relative build directory (`target/launch-exec-01`, as Trial #2 used), the
candidate link target is absolute and resolves to the build artefact, the binding is
`bound: true`, and the retarget lands only when the link's own bytes equal the replacement's.
`ba41a3f` reproduced the self-nested dangling link and `bound: false`. A link to a byte-identical
copy elsewhere is now refused (`does not resolve to the build artefact helper_report`); `ba41a3f`
bound it. A link given a relative replacement does not land.

Wording clarified: "the binding names a dangling link" means the binding fact states the dangling
condition in its own `detail`, with `bound: false` and `object_sha256: null`, so the case is not
posed with that stated cause. It is not a new forced state. Identity binding remains by digest at
the build artefact's canonical path, as for every other binding. The published landing fact carries
only `target_before`/`target_after` names, digests and fixed detail text.

**E4_PATH_CORRECTION_SOUND**

### 7.6 E6 / E6c

`helper_report.c` declares one `volatile unsigned char g_marker_region[34]` holding `HELM-MARK`,
16 marker bytes and `KRAM-MLEH`, with a `_Static_assert` on its size; the report reads the marker
at `g_marker_region[9 + i]`. No correctness dependency on linker order or padding remains.

A reviewer scratch compile (§9) was inspected as data. Candidate image: one `OBJECT` symbol
`g_marker_region`, size 34, in `.data`; file offset `0xaa0c0 + (0x4ab100 − 0x4ab0c0) = 696576`,
so the marker starts at 696585; `HELM-MARK` and `KRAM-MLEH` each occur exactly once in the whole
file; `main` references `g_marker_region`; `locate_marker_region` returns 696585. The `ba41a3f`
image reproduces the postmortem's reversed layout (`g_marker_guard_hi` 0x4ab0d8, `g_marker`
0x4ab0f0, `g_marker_guard_lo` 0x4ab100) and yields no region.

On a copy of the candidate image, E6's `pwrite_marker` and E6c's `mmap_write` both posed through
the shared `_marker_region_of`, landed, changed only bytes 696585–696597 (within the 16-byte
marker), reported `outside_region_unchanged: true`, and left the marker reading `MUTATED`.
`_outside_region_unchanged` compares whole-file length and every byte outside
`[offset, offset + 16)`. Zero regions, two regions and a stray guard each refuse with the failed
condition. E6c remains RECORDED with its safe set and gate unchanged.

**E6_E6C_CONTIGUOUS_MARKER_CORRECTION_SOUND**

### 7.7 O6 / O7 — load-bearing

Path from setup to posing, inspected and exercised with Python stand-ins only:

* `_setup_fork_helper_prearmed` names `realpath(build)/<case>.fixture-signal`; `_arm_fixture_signal`
  recreates the node with mode `0600` for each invocation, opens the reader
  `O_RDONLY|O_NONBLOCK|O_CLOEXEC`, checks it is a FIFO, non-inheritable, absent from `pass_fds` and
  holds no stale byte, and now also proves before the spawn that the argument after
  `--liveness-fifo` is absolute and the same `(st_dev, st_ino)` as the armed reader. A relative or
  foreign path is refused before the launch.
* A stand-in that `chdir`s into another directory, as the launcher's child `fchdir`s, then opens the
  handed path and writes `L`: signal `True` with the candidate; with `ba41a3f`'s relative path it
  failed `open` with `ENOENT` and the signal was `False` — Trial #2's defect.
* No `--setsid` was introduced, the launcher's group sweep and `{0, 1, 2}` descriptor contract are
  unchanged, and `helper_fork.c` still has exactly two `open` calls (`/dev/null` and the FIFO).
* **Diagnostic.** On open or write failure the descendant writes
  `HELM-LAUNCH-EXEC-01-FIXTURE-SIGNAL-FAILED:<open|write>:<errno number>` on its existing fd 2, with
  no path. `fixture_signal_diagnostic` reduces it to `reported`, `failed_step` (closed set),
  `errno` (closed table) and `errno_number` (at most four digits); an unparseable line keeps no
  captured byte. Open and write failures are distinguishable.
* **Authority.** `fixture_descendant_signalled` still reads only the FIFO byte. The diagnostic is
  used only to extend the not-posed reason after that check has already failed, and as durable
  posing-evidence metadata. That metadata explains why a fixture was not established; it is never a
  fact that the case was posed. It cannot change a status: an O7 with the signal absent is INVALID
  with or without a diagnostic, and an O7 with the signal present scores the same with a diagnostic.
  It cannot contaminate O6's stdout recipe or O7's stderr completeness assertion, and O6's
  `WriterRetainedAfterChildExit` remains a result read by `stream_completeness_as_declared`.

For P1/P2, whose descendant releases stdio first, the diagnostic goes to `/dev/null` and creates no
signal and no PASS for O6/O7. Its consequence for the P-series is recorded in §7.8.

**O6_O7_FIXTURE_CORRECTION_SOUND**

### 7.8 P1 / P2 / P4 — liveness revalidation support

Sound parts: every `fork_helper` setup hands a canonical absolute, case-private `0600` FIFO created
afresh by setup; the path names the same node after a directory change; a stand-in writer that
changes directory reaches the P-series reader, and a stale byte in a replaced node is not read. The
process-tree questions, safe sets and gates are unchanged, and P3 is unchanged.

**Defect.** `_descendant_alive` returns `False` whenever no byte arrives within its bound, and
`rule_descendant_lifecycle` renders `False` as `descendant_died`. Nothing establishes that the
descendant reached its signalling path, so a fixture failure is still a process-tree result.
Demonstrated with the candidate modules and a Python stand-in whose `open` fails, as any failure of
the descendant's liveness open would:

| Scenario | P1 | P2 | P4 |
|---|---|---|---|
| stand-in writer succeeds after changing directory | PASS, `descendant_survived` | PASS, `descendant_survived` | PASS, `descendant_survived` |
| stand-in writer's liveness `open` fails (`ENOENT` diagnosed) | **PASS, `descendant_died`** | **PASS, `descendant_died`** | **PASS, `descendant_died`** |

For P1/P2 the new diagnostic goes to `/dev/null`; for P4 it would reach the launcher's stderr pipe
and the posing evidence, but nothing consults it. Absence of the liveness signal is therefore still
interpreted as demonstrated death, and a fixture failure is not INVALID. This is exactly the
non-probative condition owner decision 4 identified in Trial #2's P1/P2/P4 records, now reachable
through any fixture failure rather than through the relative path alone. See finding **R-I1**.

**P_LIVENESS_REVALIDATION_SUPPORT_DEFECT**

### 7.9 R3

`term_signal` is `si_status` for `CLD_KILLED` and `CLD_DUMPED`, and −1 otherwise; `exit_code` is
still `si_status` for `CLD_EXITED` only. `wait_si_code_name` maps exactly the three classifications
and returns `""` when `waitid` failed or for any other `si_code`. `parse_spike_stdout` rejects any
`wait_si_code` outside `{"", CLD_EXITED, CLD_KILLED, CLD_DUMPED}`. The normaliser renders
`Signaled:<SIG>` only beside a signal classification, and `Exited:<code>` only beside `CLD_EXITED`,
through the closed signal table:

| Receipt | `ba41a3f` | Candidate |
|---|---|---|
| `Signaled`, 11, `CLD_DUMPED` | PASS `Signaled:SIGSEGV` | PASS `Signaled:SIGSEGV` |
| `Signaled`, −1, `CLD_DUMPED` (Trial #2's launcher output) | INVALID | INVALID — no signal invented |
| `Signaled`, 11, `CLD_EXITED` or `""` | PASS | INVALID |
| `Signaled`, 99, `CLD_DUMPED` | INVALID | INVALID |
| `Exited` 0 beside `CLD_DUMPED` | PASS | INVALID |
| `wait_si_code: CLD_STOPPED` | parsed | rejected |

`R2 Exited:42` with `CLD_EXITED` is unchanged. With the candidate launcher the new checks only reject
contradictory receipts, which that launcher cannot print, so no unrelated wait or exit case moves.
The WSL `core_pattern` is a pipe to the crash handler, so no reviewer `CLD_DUMPED` probe was run;
the `si_status` rule for `CLD_DUMPED` rests on `waitid(2)` and the postmortem's isolated probe. The
candidate's own Linux probe of `CLD_EXITED` and `CLD_KILLED` passed in the WSL suite.

**R3_WAITID_CORRECTION_SOUND**

### 7.10 N3

The case entry is unchanged: CONDITIONAL, `blocked_if: unprivileged_runner`, prediction
`privilege_transition_suppressed`. The setup still manufactures no privileged fixture, and no sudo,
set-ID setup or privileged runner was added.

**N3_DEFERRED_STATE_SOUND**

## 8. Publication and privacy boundary

* FIFO paths are absolute internally; `public_sanitiser(build=...)` resolves the build root and
  rendered the candidate's O6/O7 paths as `<BUILD>/O6.fixture-signal` and
  `<BUILD>/O7.fixture-signal`; other absolute paths fall to `<ABSPATH>`. The path is a setup-time
  extra argument and does not enter `plan.as_dict()` or the fixture posing evidence.
* Arming, fixture-mode and marker refusals use fixed text or `_problem`, which keeps only the errno
  name. The E4 landing fact publishes names, digests and fixed text; its `%r` OSError details, as
  before, pass the same sanitiser.
* The diagnostic publishes only closed fields; `wait_si_code` is a closed enumeration;
  `helper_path_absolute`, `helper_path_is_armed_fifo` and `outside_region_unchanged` are booleans.
* Capture prefixes stay withheld by key (`INTERNAL_ONLY_KEYS`), and sanitisation still happens once
  at the publication boundary.

**CORRECTED_PUBLICATION_BOUNDARY_SOUND**

## 9. Validation

| Check | Result |
|---|---|
| `git diff --check` (`b20c2b0..7f98d4d` and working tree) | clean |
| `python tools/validate_docs.py` | PASS on the candidate (251 JSON, 120 Markdown, 1272 link targets) and again with this record added (121 Markdown, 1276 link targets) |
| `cargo fmt --check` (cargo 1.95.0) | PASS |
| Windows, Python 3.14.3: `python -m unittest discover -s tools/tests` | 737 tests OK, 60 skipped (POSIX-only) |
| WSL Ubuntu 24.04, kernel `6.6.87.2-microsoft-standard-WSL2`, Python 3.12.3: full suite | 737 tests OK, 1 skipped (no `cc` on `PATH`) |
| WSL: `E6ContiguousMarkerRegion` with `HELM_LAUNCH_EXEC_01_TEST_CC` set to the scratch compiler | 8 tests OK, including the compiled-image test |
| Independent Trial #2 replay with `ba41a3f` modules | exact; see §3 |
| Tamper scenarios on disposable copies | see §5 |
| `run_launch_exec_01.py --verify-freeze` | `false`, seven drifted sources, exit 1 |
| Scratch compile, `x86_64-linux-gnu-gcc-13 (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`, unpacked unprivileged from official Ubuntu packages, sysroot wrapper, nothing installed | candidate and `ba41a3f`: `helper_report.c`, `helper_fork.c`, `helper_alt.c`, `helper_setid.c` with `-O2 -Wall -Wextra -static`, and `launcher_spike.c` with `-pthread` added — all rc 0, no diagnostic, no `PT_INTERP`. `helper_dynamic.c` is unchanged and was not compiled |

The scratch compile is validation only. It is **not Build 8**, it records no digest as evidence,
its outputs were read only with `nm`, `readelf`, `objdump` and Python byte scans, and they were
deleted after the review.

**Experimental LAUNCH-EXEC ELF executions: ZERO. Cases posed: ZERO.** No `launcher_spike`, no
helper ELF, no generated fixture, no strace, no `driver.Authorisation`, no D-7 invocation and no
workflow dispatch. The only processes run were Python stand-ins and the test suites' own isolated
probes.

## 10. Test quality

Each correction cluster has at least one regression test that distinguishes corrected behaviour
from `ba41a3f` behaviour on the same inputs, confirmed by the side-by-side probe:

| Cluster | Distinguishing test | `ba41a3f` behaviour on its inputs |
|---|---|---|
| X2b/X2c/X4 | `FixtureModes.test_write_applies_each_declared_mode_under_any_umask` | all fixtures `0644` |
| T1 | `T1QualifiedTimeout.test_the_intended_sigterm_sequence_passes` | the SIGTERM path FAILs |
| S4 | `S4ConservativeExecEvidence.test_contradictory_evidence_fails` | a report declaring 3, or naming `helper_alt`, PASSes |
| M2 | `M2DirectChildWindow.test_thread_clones_before_the_process_clone_are_not_the_child` | window anchored on thread 301 |
| E4 | `E4ResolvedSymlinkTarget.test_the_binding_still_names_the_built_artefact`, `test_an_identical_copy_elsewhere_is_not_the_artefact` | unbound; identical copy bound |
| E6/E6c | `E6ContiguousMarkerRegion.test_a_compiled_scratch_helper_has_one_region_in_one_symbol` | the `ba41a3f` image has no region |
| O6/O7 | `HelperFacingFifoPaths.test_a_stand_in_that_changes_directory_still_signals` | `ENOENT`, no signal |
| R3 | `R3SignalPreservation.test_no_signal_is_invented`, `test_the_launcher_expressions_are_these` | `Signaled` beside `CLD_EXITED` PASSes; old C expression present |
| P path | `HelperFacingFifoPaths.test_the_p_series_rendezvous_works_across_the_change` | relative path, no byte |

The changed historical contract tests do not read expectations from the files under test (§5).
Two limits are noted rather than scored as weakness: no test covers the P-series fixture-failure
property, because it is not implemented (R-I1); and the only compiled-layout test skips on a host
without a C compiler, while the source-level layout test always runs.

**CORRECTION_REGRESSION_TESTS_SOUND**

## 11. Build requirement

`launcher_spike.c`, `helper_report.c` and `helper_fork.c` differ from the Trial #2 freeze by bytes;
`helper_alt.c`, `helper_dynamic.c` and `helper_setid.c` do not. Build 7 compiled the Trial #2 bytes
and does not bind this candidate.

**BUILD_8_REQUIRED_FOR_FUTURE_FREEZE**

## 12. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| R-I1 | IMPORTANT | P1/P2/P4 liveness | Yes — any failure of the descendant's liveness `open`/`write` (demonstrated with a stand-in) | Absence of the liveness byte is rendered `descendant_died` and PASSes; no positive evidence that the descendant reached its signalling path; the P1/P2 diagnostic goes to `/dev/null` and P4's is not consulted. A fixture failure remains a process-tree result instead of INVALID — the condition owner decision 4 found non-probative | Fix before freeze. Not fixed here. If the fix changes the P-series questions or construction beyond fixture-validity evidence, it needs an owner decision |
| R-M1 | MINOR | S4 classification | Not S4-specifically: admission and pre-exec setup do not depend on the post-fork delay, so such a failure would also FAIL S1/S3 | An admission refusal, or an explicit pre-exec status record without a report, is INVALID for S4 because `helper_report_complete` is checked first; the owner rule treats a decisive contradiction as FAIL | Recommended before freeze: let S4's posing admit the decisive no-report outcomes (as `decisive_without_report` does) so they FAIL |
| R-M2 | MINOR | Definition section 10 wording | Documentation only | Section 10 says `--verify-freeze` reports the definition as drift; `verify_freeze()` checks only the manifest's `sha256` map, and definition drift is caught by the candidate test | Correct the wording at freeze; consider verifying `definition_sha256` in the next runner |
| R-B1 | BACKLOG_NONBLOCKING | Receipt parser | No — the candidate launcher always prints the field | A receipt without `wait_si_code` is accepted uncorroborated (S4 and R3 still PASS) | Consider requiring the field at freeze |
| R-B2 | BACKLOG_NONBLOCKING | S4 token | n/a | The S4 claim token is `ExecStatusIndeterminate` for every admitted receipt except `ExecFailed`; S4's discrimination lives in its assertions, as §10.4 of the definition states | None required |

No BLOCKER. One IMPORTANT.

## 13. Freeze-step tasks — not performed

Prerequisite: resolve R-I1 (and, as recommended, R-M1) in a bounded fix, followed by a bounded
re-review of that fix. Then the freeze step must:

1. write a new `SOURCE-HASHES.json` over the corrected bytes, binding all 17 source hashes and
   all three definition hashes, keeping the Trial #2 manifest addressable at `ba41a3f` (blob
   `6f000fac`), and updating `trial`, `status`, `classification`, `supersedes`,
   `supersession_reason`, `freeze_lineage`, `requires`, `build_evidence`, `posing_versus_showing`
   (`helper_report_complete`, `helper_exit_corroborated`, `launcher_receipt_claim`,
   `executed_marker` for S4) and the vocabulary for the new tokens;
2. change `run_launch_exec_01.py`'s `TRIAL_ID` from `trial-002` to `trial-003`, with its NOT_RUN
   and D-7 text, and bind the runner's new hash (it is a frozen input);
3. rebase the manifest-coupled tests from "Trial #2 freeze plus declared delta" onto the new
   manifest, retiring `CandidateRecord`'s drift expectations openly while keeping
   `Trial2StaysImmutable` and its `ba41a3f` replay;
4. mark `TRIAL-3-CORRECTION-CANDIDATE.json` superseded by the freeze commit without rewriting its
   history, and change definition section 10 from candidate to frozen language, correcting R-M2;
5. update the experiment README/index and `PROJECT_STATE.md` only as the freeze requires;
6. re-verify 72 / 54 / 11 / 7, the traced set, `completeness()` and `unposable_cases() == {}`;
7. cut the freeze commit and obtain an independent freeze review;
8. only after the freeze, obtain formal Build 8 compile-only evidence from the compile-only
   workflow for the frozen C sources, appended to `BUILD-EVIDENCE.md`;
9. separately: a Trial #3 dispatcher with its own infrastructure review, and a new owner D-7.

The runner still saying `trial-002` and the README not listing the candidate are not defects before
freeze, because the candidate clearly says NOT_FROZEN.

## 14. Verdicts

| Area | Verdict |
|---|---|
| Historical integrity | **TRIAL2_HISTORICAL_INTEGRITY_SOUND** |
| Scope | **CORRECTION_SCOPE_BOUND_SOUND** |
| NOT_FROZEN accounting | **NOT_FROZEN_DELTA_ACCOUNTING_SOUND** |
| Freeze-plus-delta tests | **FREEZE_PLUS_DELTA_CONTRACT_SOUND** |
| X2b/X2c/X4 | **X_CLUSTER_CORRECTION_SOUND** |
| T1 | **T1_CORRECTION_SOUND** |
| S4 | **S4_CONSERVATIVE_RACE_CORRECTION_SOUND** |
| M2 | **M2_PROCESS_CLONE_CORRELATION_SOUND** |
| E4 | **E4_PATH_CORRECTION_SOUND** |
| E6/E6c | **E6_E6C_CONTIGUOUS_MARKER_CORRECTION_SOUND** |
| O6/O7 | **O6_O7_FIXTURE_CORRECTION_SOUND** |
| P1/P2/P4 | **P_LIVENESS_REVALIDATION_SUPPORT_DEFECT** |
| R3 | **R3_WAITID_CORRECTION_SOUND** |
| N3 | **N3_DEFERRED_STATE_SOUND** |
| Global shape | **CANDIDATE_GLOBAL_SHAPE_SOUND** |
| Publication | **CORRECTED_PUBLICATION_BOUNDARY_SOUND** |
| Tests | **CORRECTION_REGRESSION_TESTS_SOUND** |
| Build | **BUILD_8_REQUIRED_FOR_FUTURE_FREEZE** |

**TRIAL #2 D-7 IS CONSUMED. LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE. TRIAL #2 MUST NOT BE
RERUN. NO TRIAL #3 IS AUTHORISED. NO TRIAL #3 FREEZE EXISTS.**

**TRIAL #3 CORRECTION CANDIDATE IS NOT READY FOR FREEZE.**

Classification: **TRIAL_3_CORRECTION_CANDIDATE_NEEDS_FIX**
