# LAUNCH-EXEC-01 — Trial #3 correction: final RR-I1 micro review

**THIS IS THE FINAL RR-I1 MICRO REVIEW, NOT A FULL CORRECTION RE-AUDIT.**

**IT FIXES NOTHING, FREEZES NOTHING AND AUTHORISES NOTHING.**

| | |
|---|---|
| Reviewed fix | `f1e7eead6f604d28d436cfa474c7efc44d1dc886` |
| Parent (bounded re-review) | `db45336684817d95edbb7fcea3cf0191f7eee528` — finding **RR-I1** |
| Earlier fix / review / candidate | `53ad8bf` / `ee8cd91` / `7f98d4d` |
| Remote milestone | `1f0b86a13ece63badf28d7cef97d580ebc359a99`, unchanged after a read-only fetch |
| Pushed | NO |

**Reviewer independence.** This working session authored `f1e7eea`. The load-bearing judgements
below — the structural predicate, the `53ad8bf`-versus-`f1e7eea` regression, the S4 matrix, the
vocabulary bound, unaffected cases and the candidate-record tamper — therefore come from a fresh
reviewer context. It had not seen the implementation, worked read-only on disposable copies, and
executed no C binary (§12). This session ran the mechanical integrity checks (§§8–11) and checked
that the independent results agree with its own before/after scoring matrix. Owner dispositions
applied: the candidate-record metadata change is accepted if §7 holds, and the boolean renderer
quirk is backlog if the predicate rejects booleans.

## 1. Exact micro delta

`git diff db45336..f1e7eea` touches exactly five files:

| Path | +/− | Content |
|---|---|---|
| `docs/experiments/launch-exec-01/observations.py` | +57 / −1 | new `explicit_pre_exec_failure` with `ERRNO_NUMBER_MIN/MAX`; `_s4_receipt_exit` ExecFailed branch; `DECISIVE_WITHOUT_TOKEN_ASSERTIONS` gains `helper_exit_corroborated` |
| `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md` | +15 | one §10.4 paragraph |
| `docs/experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json` | +16 / −1 | `contract_delta.decisive_without_token_added`, `correction_rereview`, `requires` |
| `tools/tests/test_launch_exec_01_trial3.py` | +163 | class `S4StructuredPreExecFailure` |
| `tools/tests/test_launch_exec_01_ab7.py` | +19 / −5 | two decisive-set tests now read "Trial #2 manifest + declared delta" |

`helper_fork.c`, `driver.py`, `checker.py`, `frozen_cases.py`, `evidence.py`, `launcher_spike.c`, the
runner, `SOURCE-HASHES.json`, all evidence and all earlier review records are unchanged. An AST
comparison against `db45336` shows only three top-level items changed in `observations.py`.

**RR_I1_MICRO_SCOPE_SOUND**

## 2. Structural pre-exec predicate

`explicit_pre_exec_failure(spike)` is true only when all of the following hold:
* `spike` is a dict with `admission == "accepted"`;
* `process_disposition == "ExecFailed"`;
* `timeout_disposition == ""`;
* `exec_failed_stage` is in the frozen `STAGES`;
* `exec_failed_errno` passes `_plain_int` (rejecting `bool`) and lies in 1..4095.

It never consults `ERRNO_NAMES`.

Independent direct probes:
* **True:** errno 13 (named), 7 (verified absent from `ERRNO_NAMES`), 1 and 4095.
* **False:** errno `True`, `False`, 0, −7, 4096, 7.0, `"7"`, `None` or missing; stage `""`,
  `UNKNOWN`, missing, free text or a list; `timeout_disposition` `KilledByLauncher` or missing;
  admission refused; disposition `Exited`; any non-dict.

The fields match `launcher_spike.c`:
* `ExecFailed` is set only when a record of exactly `sizeof(struct exec_status)` bytes was read.
* `timeout_disposition` is assigned only in a branch `ExecFailed` precedes.
* The stage is printed by `stage_name()`, which yields a frozen name or `UNKNOWN`.
* The errno is an `int32_t` printed with `%d`, so the integer range check is necessary and sufficient.

**RR_I1_STRUCTURAL_PREEXEC_FACT_SOUND**

## 3. Primary RR-I1 regression

The same fabricated, honestly posed S4 observation was scored in separate processes against the
`53ad8bf` modules (extracted with `git show`) and against `f1e7eea`: accepted, `ExecFailed`, stage
`EXEC`, errno 7, exit 127, no helper report. 7 was first verified absent from `ERRNO_NAMES`.

| Revision | Status | Outcome |
|---|---|---|
| `53ad8bf` | **INVALID** | none — "errno outside the frozen table" |
| `f1e7eea` | **FAIL** | `helper_exit_contradicted` (`helper_exit_corroborated` violated) |

This session's own side-by-side matrix, 64 rows, agrees: exactly two rows changed, both the
unnamed-errno S4 shape.

**RR_I1_OLD_INVALID_NEW_FAIL_PROVEN**

## 4. S4 classification matrix

Independent scoring at `f1e7eea`, with `53ad8bf` in brackets:

| Observation | Status |
|---|---|
| known errno 13 `ExecFailed` | FAIL [FAIL] |
| valid unnamed errno 7, 4095, or 7 at stage `SETPGID` | **FAIL** [INVALID] |
| boolean, non-integer (`"7"`, 7.0), 0, 4096, −7, `None` or missing errno | INVALID [INVALID] |
| stage `UNKNOWN`, `""` or missing, with errno 7; or a timeout sub-disposition with errno 7 | INVALID [INVALID] |
| malformed errno `"13"` + complete report declaring exit 3 (independent decisive side) | FAIL [INVALID] |
| malformed errno `"13"` + report declaring exit 7 | INVALID [INVALID] |
| admission refusal; `TimedOut:KilledByLauncher:SIGTERM`; `Signaled:SIGKILL`; `Exited` 3 | FAIL [FAIL] |
| `Exited` 7 `CLD_EXITED` + missing report / malformed report / report with no declared exit | INVALID [INVALID] |
| `Exited` 7 `CLD_EXITED` + complete report declaring 7 | PASS, `ExecStatusIndeterminate` [same] |

**Exec-status claim versus termination observation.** The harness's exec-status claim for the
accepted path is `ExecStatusIndeterminate`, which equals the S4 prediction and is never a
contradiction. The direct-child termination observation (`process_disposition`, `exit_code`,
`wait_si_code`) is read separately by `_s4_receipt_exit`, and for that path it *holds*. The fix
does not conflate the two.

One row sits against the new §10.4 wording, recorded as RR-M3. An `ExecFailed` receipt with stage
`""`, or with a timeout sub-disposition, *plus a named errno* scores FAIL on the shared rule's own
symbolic token (for example `ExecFailed:EACCES`), not INVALID. The structural predicate correctly
calls the record malformed, and the S4 exit check correctly does not decide on it. But
`rule_process_disposition` still renders a token from it, and that token mismatches the prediction.
`parse_spike_stdout` accepts both receipts, while `launcher_spike.c` cannot emit either. The status
was already FAIL at `53ad8bf` and at `7f98d4d`; `f1e7eea` changed only the outcome token from
`helper_exit_contradicted`. The new tests use errno 7 for these rows, so they do not show it. No
unnamed-errno, accepted-path or conservative-claim row is wrong.

**RR_I1_S4_PRECEDENCE_SOUND**

## 5. Symbolic errno vocabulary

The following are AST-identical to `db45336`:
* `ERRNO_NAMES` and `SIGNAL_NAMES`;
* `_exec_failed`, `_timed_out`, `_signaled` and `_exited`;
* `ASSERTIONS`, `ASSERTION_READS` and `ASSERTION_VIOLATION_TOKENS`;
* both S4 rules.

The only new string literals are the predicate's docstring and two assertion detail strings. No
outcome token was added. Classification uses the structural fact before, and independently of,
symbolic rendering.

**RR_I1_ERRNO_VOCABULARY_BOUND_SOUND**

## 6. Unaffected cases and P/O delta

The independent pass scored S5, S1, S2, S3, S7, X1, X4 and R1 on seven receipt shapes each:
* `ExecFailed` EXEC/13;
* `ExecFailed` CHDIR/13;
* `ExecFailed` errno 7;
* errno `"13"`;
* stage `UNKNOWN` with errno 7;
* `Exited` 0 with a report;
* `Exited` 0 without one.

All 56 results are identical between `53ad8bf` and `f1e7eea`. This session's matrix agrees for S5,
S1, S2, S3, S7, X1, X4, E1 and R1. `helper_exit_corroborated` is used by S4 alone, so adding it to the
decisive set cannot reach another case.

`f1e7eea` has no hunk in `helper_fork.c`, `driver.py` (P health channel, direct-child gate, P and O
setups and plans) or any P/O rule. The earlier P1/P2/P3/P4/O6/O7 verdicts stand unchanged.

**RR_I1_UNAFFECTED_CASE_NONREGRESSION_SOUND**

**RR_I1_P_O_DELTA_NONE**

## 7. Candidate-record deviation

**Before.** The Trial #2 manifest (`posing_versus_showing.decisive_without_token.assertions`) froze
only `no_executed_image`, for S2 and S7.

**Added.** `f1e7eea` adds `helper_exit_corroborated`, for S4, to the checks that decide a repetition
without a rule token. The record's `contract_delta.decisive_without_token_added` names exactly that
one check, with `cases: ["S4"]`, the load-bearing structural fields, and a statement that malformed
records never decide on the launcher side.

**Guards.** The record stays `NOT_FROZEN`, with `trial_3_authorised: false`, `trial_3_d7: null`
and no hash map of its own; the only hash in it is the pre-existing reference to the Trial #2 manifest.
It grants no D-7 or execution authority. `verify_freeze()` does not read it, so it cannot make
arbitrary drift acceptable.

The two AB7 tests now require `DECISIVE_WITHOUT_TOKEN_ASSERTIONS == manifest ∪ declared addition`.
They also check that the addition does not overlap the manifest, that the addition is exactly
`{helper_exit_corroborated}`, and that the users are S2, S4 and S7. Tamper copies, 108 tests each:

| Tamper | Result |
|---|---|
| none | OK |
| undeclared `executed_marker` added to the set | 2 FAIL |
| declaration removed from the record | 2 ERROR |
| an extra name declared in the record | 2 FAIL |

The failing tests in each tamper are `AB7Preregistration.test_the_manifest_records_the_same_contract`
and `S2S7ExecStatusIsAuthoritative.test_only_declared_cases_may_fail_without_a_rule_token`. The
current implementation value is therefore never accepted on its own. The owner's acceptance of the
metadata change holds.

**RR_I1_CANDIDATE_RECORD_DELTA_SOUND**

## 8. NOT_FROZEN accounting

* `SOURCE-HASHES.json` is byte-identical to `ba41a3f`; no new manifest exists.
* The candidate record reads `state: NOT_FROZEN`, `trial_3_authorised: false`,
  `trial_3_d7: null`, `trial_3_executed: false`, and carries no hash map. Its `requires` is this micro
  review, and it states that it grants nothing. It is not read by `verify_freeze()`, which reads
  `SOURCE-HASHES.json` alone.
* Byte drift against the `ba41a3f` manifest: `driver.py`, `frozen_cases.py`, `helper_fork.c`,
  `helper_report.c`, `launcher_spike.c`, `make_fixtures.py`, `observations.py`, and the definition.
  This equals the record's declared lists; `f1e7eea` changed two already-declared frozen inputs
  (`observations.py`, the definition) and added none.
* `run_launch_exec_01.py --verify-freeze` lists exactly those seven sources and exits 1.
* The freeze-plus-delta tamper copies still fail on undeclared drift (`evidence.py`, `checker.py`,
  `helper_alt.c`, `ADR-0024`), on reverted declared drift (`make_fixtures.py`, `helper_fork.c`), on a
  declared name that did not drift, and on an undeclared posed check. Restoring every frozen input to
  `ba41a3f` makes `verify_freeze` true.

**RR_I1_NOT_FROZEN_DELTA_ACCOUNTING_SOUND**

## 9. Historical Trial #2

Replayed with every Python module extracted from `ba41a3f`:
* re-scoring the published records;
* replaying the journal;
* comparing the journal's `case_completed` lines.

Each gives 59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED and `MECHANISM_REJECTED`, with zero mismatches
over all 72 status/reason pairs. The journal is not torn and `d7_consumed` is true. The evidence
files, result review and postmortem diagnostics are byte-identical to `8a9dc77`, `1f0b86a` and
`b20c2b0`, and no earlier review record changed.

**TRIAL2_HISTORICAL_INTEGRITY_SOUND**

## 10. Validation

| Check | Result |
|---|---|
| `git diff --check` (`db45336..f1e7eea` and working tree) | clean |
| `python tools/validate_docs.py` | PASS |
| `cargo fmt --check` | PASS |
| Windows, Python 3.14.3, `python -m unittest discover -s tools/tests` | 774 OK, 74 skipped (POSIX-only) |
| WSL Ubuntu 24.04, Python 3.12.3, full suite | 774 OK, 1 skipped (compiled-image E6 test, no `cc` on `PATH`) |
| C compile | not required: `f1e7eea` changes no C |

**Experimental LAUNCH-EXEC ELF executions: ZERO. Cases posed: ZERO.** No launcher, helper or other
HELM ELF was executed, no D-7 path was taken and no workflow was dispatched.

## 11. Build

`launcher_spike.c`, `helper_report.c` and `helper_fork.c` still differ from the Trial #2 freeze.

**BUILD_8_REQUIRED_FOR_FUTURE_FREEZE**

## 12. Publication boundary and test quality

**Publication.** No new record field is published. The only new durable text is the assertion
detail "the child reported an explicit pre-exec failure at stage %s, errno %d, where the frozen helper
was to run and exit %d". It is formatted only after the predicate holds, so it carries a frozen stage
name of at most 13 characters and an integer between 1 and 4095. It contains no path, pid,
descriptor, raw status payload or unbounded text, and it still passes the ordinary sanitiser at the
publication boundary.

**RR_I1_PUBLICATION_BOUNDARY_SOUND**

**Tests.** `S4StructuredPreExecFailure.test_the_rr_i1_regression_against_53ad8bf` extracts the
`53ad8bf` modules and asserts the literal `["INVALID", None]` there. It then asserts FAIL /
`helper_exit_contradicted` at the candidate, using errno 7, whose absence from `ERRNO_NAMES` is
itself asserted against a literal frozen number set. It executes no HELM code. The class also covers:
* known errno at EXEC, CHDIR and DUP2;
* unnamed errno 7, 11 and 4095;
* fourteen malformed shapes (string, `None`, float, 0, negative, 4096, missing errno; empty, unknown,
  free-text or missing stage; timeout) and a boolean;
* a malformed record beside a decisive report;
* refusal, timeout, signal and wrong exit;
* the accepted path with a missing, malformed or truncated report, and PASS;
* the claim-token distinction;
* S2, S5 and S7 untouched.

The AB7 tests enforce "manifest plus declared addition" (§7). The gaps are that the malformed rows
use errno 7, which hides RR-M3, and that the `CandidateRecord` class itself does not check the new
declaration, which the AB7 tests do (RR-B4).

**RR_I1_MICRO_REGRESSION_TESTS_SOUND**

## 13. Independent pass

A fresh reviewer context produced §§1–7 and §12 read-only, given only the owner rules and code
pointers. It ran `git diff` and `git show`, Python scoring scripts on fabricated observations in
separate processes per revision, an AST comparison against `db45336`, `parse_spike_stdout` on
fabricated JSON, and the AB7, final and `CandidateRecord` suites in four disposable copies. It
executed no C binary and no runner invocation, constructed no `Authorisation`, called no `pose`,
`observe` or `_launch_and_observe`, and changed no repository file. Its verdicts were all SOUND, with
one MINOR and two BACKLOG items, recorded below. This session's mechanical checks (§§8–10) and its
64-row before/after matrix agree with them.

## 14. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| RR-M3 | MINOR | S4 §10.4 wording | Not from `launcher_spike.c`; only through a parse-valid tampered or fabricated receipt | An `ExecFailed` receipt with stage `""` or a timeout sub-disposition *and a named errno* FAILs on the shared rule's `ExecFailed:<ERRNO>` token, not INVALID. The status was already FAIL at `53ad8bf` and `7f98d4d`; only the outcome token changed. The §10.4 sentence "stays INVALID unless another side independently contradicts" overstates this, while the candidate record's "never decides on the launcher side" is exact | Correct the §10.4 wording at freeze (or gate S4's claim token on the structural fact in a future cycle); no correction cycle now |
| RR-B3 | BACKLOG_NONBLOCKING | Shared errno renderer | No — `%d` output cannot be a Python `bool` | The shared renderer treats `True` as errno 1 (`EPERM`); the S4 structural authority rejects booleans explicitly | None before freeze |
| RR-B4 | BACKLOG_NONBLOCKING | Test placement | n/a | `CandidateRecord` does not itself check `decisive_without_token_added`; the two AB7 tests enforce it, and tampering fails them | Optional at freeze |
| RR-B5 | BACKLOG_NONBLOCKING | Shared rule robustness | No — `parse_spike_stdout` rejects such receipts | A receipt with a missing `exec_failed_stage` and a named errno raises `TypeError` inside `evaluate`, in `53ad8bf` and `f1e7eea` alike | None before freeze |

Carried unchanged and not reopened: RR-M1 (P2 gate wording, MINOR), RR-M2 (P2/P4 rendezvous stall
over 3 s, MINOR, accepted), RR-B1 (S4 token naming), RR-B2 (`helper_fork` `pipe2` diagnostic).

No BLOCKER. No IMPORTANT. None of the micro review's BLOCKER/IMPORTANT conditions holds:
* an unnamed valid errno is FAIL;
* no malformed status is newly promoted to FAIL;
* the accepted ambiguity is not a contradiction;
* S5 and the unaffected cases did not move;
* no vocabulary was widened;
* the candidate record does not weaken the freeze;
* there is no undeclared drift, no leak and no Trial #2 mutation.

## 15. Verdicts

| Area | Verdict |
|---|---|
| Scope | **RR_I1_MICRO_SCOPE_SOUND** |
| Structural predicate | **RR_I1_STRUCTURAL_PREEXEC_FACT_SOUND** |
| Primary regression | **RR_I1_OLD_INVALID_NEW_FAIL_PROVEN** |
| S4 precedence | **RR_I1_S4_PRECEDENCE_SOUND** |
| Vocabulary | **RR_I1_ERRNO_VOCABULARY_BOUND_SOUND** |
| Unaffected cases | **RR_I1_UNAFFECTED_CASE_NONREGRESSION_SOUND** |
| P/O delta | **RR_I1_P_O_DELTA_NONE** |
| Candidate record | **RR_I1_CANDIDATE_RECORD_DELTA_SOUND** |
| NOT_FROZEN accounting | **RR_I1_NOT_FROZEN_DELTA_ACCOUNTING_SOUND** |
| Publication | **RR_I1_PUBLICATION_BOUNDARY_SOUND** |
| Trial #2 | **TRIAL2_HISTORICAL_INTEGRITY_SOUND** |
| Tests | **RR_I1_MICRO_REGRESSION_TESTS_SOUND** |
| Build | **BUILD_8_REQUIRED_FOR_FUTURE_FREEZE** |

**TRIAL #2 D-7 IS CONSUMED. LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE. TRIAL #2 MUST NOT BE
RERUN. NO TRIAL #3 IS AUTHORISED. NO TRIAL #3 FREEZE EXISTS.**

**CORRECTION REVIEW IS CLOSED; TRIAL #3 FREEZE PREPARATION MAY BEGIN.**

Freeze-step wording to carry: RR-M1 (§10.9 gate and P2 `setsid` ordering), RR-M3 (§10.4 malformed
`ExecFailed` sentence), and the earlier R-M2 (`--verify-freeze` reads source hashes only).

Classification: **TRIAL_3_FINAL_RRI1_MICRO_REVIEW_PASSED_READY_FOR_FREEZE_PREPARATION**
