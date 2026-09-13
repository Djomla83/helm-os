# LAUNCH-EXEC-01 — Trial #3 freeze review

**THIS IS A FREEZE-INTEGRITY REVIEW, NOT A CORRECTION RE-AUDIT.**

**IT FIXES NOTHING, AMENDS NOTHING, PUBLISHES NOTHING, BUILDS NOTHING AND AUTHORISES NOTHING.**

| | |
|---|---|
| Reviewed freeze | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` — `docs(launch-exec-01): freeze trial 3 candidate` |
| Parent (final RR-I1 micro review) | `030d88a6c96e1dd97b3ebf7839dc4b2dafed9d70` |
| Final RR-I1 fix | `f1e7eead6f604d28d436cfa474c7efc44d1dc886` |
| Historical Trial #2 freeze | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` |
| Remote milestone | `origin/docs/helm-launch-architecture` = `1f0b86a13ece63badf28d7cef97d580ebc359a99` after a read-only fetch |
| Publication | local only: 9 commits ahead of and 0 behind the remote; no remote ref contains `bebd8a5` |
| Pushed | NO |

**Scope and independence.** This review ran in a fresh session that took no part in writing
`bebd8a5` or the correction sequence. It started from a clean worktree at `bebd8a5`. Every
byte-level result below was recomputed from Git objects (`git cat-file`), from scratch extractions
of the committed trees, or from a disposable `--shared` clone. None relies on the freeze commit
message or on the freeze author's numbers.

It did not re-review S4 or the P-series design. It did not reopen RR-M2, RR-B1 to RR-B5, R-B1 or
R-B2: the freeze commit changes no executable semantics (§2), so their reachability is unchanged.

The [final RR-I1 micro review](HELM-LAUNCH-EXEC-01-TRIAL-003-RRI1-MICRO-REVIEW.md) closed the
correction review; this record reviews only the freeze cut on top of it.

**No experimental LAUNCH-EXEC ELF was executed and no case was posed** (§21).

## 1. Starting state

| Check | Result |
|---|---|
| `git status --short` | clean |
| `git branch --show-current` | `docs/helm-launch-architecture` |
| `git rev-parse HEAD` | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| `git fetch origin` (read-only) | `origin/docs/helm-launch-architecture` = `1f0b86a1…`; `origin/main` = `d8a6887d…` |
| `git rev-list --left-right --count origin/docs/helm-launch-architecture...HEAD` | `0 9` |
| `git branch -r --contains bebd8a5` | none |
| Commits `1f0b86a..bebd8a5` | nine, all by the owner's Git identity, with no `Co-Authored-By` or generated-by trailer |

## 2. Exact freeze delta

`git diff --numstat 030d88a..bebd8a5` touches exactly nine files, with 705 insertions and 208
deletions. `030d88a` itself adds only its review record, so the parent's frozen inputs are
`f1e7eea`'s.

| Path | +/− | Classification | What changed |
|---|---|---|---|
| `docs/experiments/launch-exec-01/run_launch_exec_01.py` | +7 / −6 | trial-003 identity transition | the module docstring's status, and `TRIAL_ID = "trial-002"` → `"trial-003"`; no other statement |
| `docs/experiments/launch-exec-01/README.md` | +17 / −10 | trial-003 identity transition; navigation | Trial #2 complete and Trial #3 FROZEN / NOT_RUN status; file-table rows for the manifest, the superseded record and `BUILD-EVIDENCE.md` |
| `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md` | +98 / −42 | approved wording corrections; trial-003 identity transition | header status; §10 title, preregistration paragraph and review-chain table; the manifest paragraph and **R-M2**; **RR-M3** in §10.4; **RR-M1** in §10.9; §10.12 freeze and build status; the old §10 anchor kept as an explicit `id` |
| `docs/experiments/launch-exec-01/SOURCE-HASHES.json` | +222 / −75 | manifest/freeze validation | rehash of the drifted inputs (§4); Trial #3 status blocks (§5); contract blocks describing the reviewed code (below) |
| `docs/experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json` | +18 / −3 | candidate-record supersession | `state`, `superseded_by`, `is_not`, `correction_micro_review`, `freeze_step_items_status`, `requires` (§11) |
| `docs/PROJECT_STATE.md` | +28 / −0 | navigation/project-state update | a new top section for the local freeze (§16) |
| `tools/tests/test_launch_exec_01_ab7.py` | +35 / −24 | manifest/freeze validation | decisive-set checks read the Trial #3 manifest; the Build 7 check reads the `ba41a3f` manifest with `git show` |
| `tools/tests/test_launch_exec_01_final.py` | +6 / −21 | manifest/freeze validation | the manifest-plus-delta union is removed; the posed-check input map must equal the checks in use |
| `tools/tests/test_launch_exec_01_trial3.py` | +274 / −27 | manifest/freeze validation; candidate-record supersession | classes `Trial3Freeze` and `Trial3FreezeCapturesReviewedSemantics`; Trial #2 checks read `ba41a3f` with `git show`; `CandidateRecord` rewritten as superseded provenance |

**Reviewed correction bytes now frozen.** No correction byte changed in this commit. The seven
correction inputs appear in the delta only as manifest hashes, and they are byte-identical at
`030d88a` and `bebd8a5` (§4).

**No hidden semantics.**

* The only changed executable file is `run_launch_exec_01.py`. Its two hunks are the module
  docstring and the `TRIAL_ID` literal. No other experiment `.py` or `.c` file changed.
* The manifest's new contract blocks are data. No frozen module reads manifest content to decide
  anything:
  * `run_launch_exec_01.py` reads only the `sha256` map, to verify, and embeds the whole document
    as `freeze`;
  * `driver.TrialContext` stores `freeze` and never reads it;
  * `evidence.evidence_document` copies it.
* Each new block describes code already reviewed through `030d88a`:
  * `assertions.helper_exit_corroborated`, S4 added to `executed_marker`, `expected_markers.S4`,
    the decisive-without-token set, the `s4` block and `liveness_fixture_health` match the
    candidate record's reviewed `contract_delta`, and the committed tests that tie manifest to
    code pass (§21);
  * the removed `posed_check_inputs.returned_before_descendant_lifetime` entry was already stale at
    Trial #2. The check is registered in `driver.py` at both freezes, no plan poses with it at
    `ba41a3f` or at `bebd8a5`, and the checks in use are the same six at both:
    `exec_status_pair_adjacent`, `fixture_descendant_signalled`, `no_helper_report`,
    `retention_observed`, `same_inode_as_writer` and `threaded_parent_observed`.

**TRIAL3_FREEZE_DELTA_MECHANICAL_AND_REVIEWED**

## 3. SOURCE-HASHES manifest — byte audit

Recomputed from the committed tree, not from the report:

| Identity | Reported | Recomputed |
|---|---|---|
| Git blob | `8cd290b573408510f8c16cd8dafe676354140a38` | `git ls-tree`; `git hash-object --stdin` over `git show`; Python SHA-1 over `blob <len>\0` and the bytes — all `8cd290b5…4140a38` |
| SHA-256 | `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` | `sha256sum` and Python `hashlib` — both `ea482c6f…cda79ba70` |
| Size and line endings | — | 58,610 bytes, LF only |
| JSON | — | parses; no duplicate key in the manifest or in the superseded record |

Every bound hash was then recomputed from `git cat-file blob bebd8a5:<path>`. Source paths resolve
against the experiment directory and definition paths against the repository root.

* **Sources: 17 / 17 exact** — `README.md`, `checker.py`, `driver.py`, `evidence.py`,
  `frozen_cases.py`, `harness.py`, `helper_alt.c`, `helper_dynamic.c`, `helper_fork.c`,
  `helper_report.c`, `helper_setid.c`, `journal.py`, `launcher_spike.c`, `make_fixtures.py`,
  `observations.py`, `oracles.py`, `run_launch_exec_01.py`.
* **Definitions: 3 / 3 exact** — `docs/adr/ADR-0024-launch-authority.md`,
  `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md`, `docs/research/HELM-LAUNCH-ARCHITECTURE.md`.
* **Closed sets against `ba41a3f`.** The `sha256` and `definition_sha256` key sets are identical to
  the Trial #2 manifest's: 17 and 3, nothing added and nothing removed. The same audit over
  `ba41a3f` gives 17 / 17 and 3 / 3 against its own manifest.
* **Unhashed files.** The experiment directory at `bebd8a5` holds 20 files: the 17 bound sources
  and exactly the three `not_hashed_here` entries (`BUILD-EVIDENCE.md`, `SOURCE-HASHES.json`,
  `TRIAL-3-CORRECTION-CANDIDATE.json`).

**TRIAL3_MANIFEST_BYTE_INTEGRITY_SOUND**

## 4. Frozen byte drift against Trial #2

Derived by comparing Git blobs at `ba41a3f` and `bebd8a5` for every bound path, then attributing
each change with `git log ba41a3f..bebd8a5 -- <path>`:

| Frozen input | SHA-256 `ba41a3f` → `bebd8a5` | Changed by | Same bytes at `030d88a` | Accounted as |
|---|---|---|---|---|
| `driver.py` | `844db6e3…` → `84602534…` | `7f98d4d`, `53ad8bf` | yes | reviewed correction |
| `frozen_cases.py` | `ee4b9599…` → `d29a58ad…` | `7f98d4d`, `53ad8bf` | yes | reviewed correction |
| `make_fixtures.py` | `14b3c276…` → `baa61170…` | `7f98d4d` | yes | reviewed correction |
| `observations.py` | `4626814e…` → `d8bda42f…` | `7f98d4d`, `53ad8bf`, `f1e7eea` | yes | reviewed correction |
| `helper_fork.c` | `a62a6a1e…` → `52089697…` | `7f98d4d`, `53ad8bf` | yes | reviewed correction |
| `helper_report.c` | `99770ed8…` → `ceaac0dc…` | `7f98d4d` | yes | reviewed correction |
| `launcher_spike.c` | `0a45447b…` → `4d6a0ac7…` | `7f98d4d` | yes | reviewed correction |
| `README.md` | `9f795a85…` → `c35c4afa…` | `bebd8a5` only | no | freeze mechanics (status) |
| `run_launch_exec_01.py` | `2b55aa6b…` → `1c84d194…` | `bebd8a5` only | no | freeze mechanics (`trial-003`) |
| `LAUNCH-EXEC-01-DEFINITION.md` | `b403bbea…` → `0c7406b4…` | `7f98d4d`, `53ad8bf`, `f1e7eea`, `bebd8a5` | no | reviewed correction text plus the three approved wording corrections |

The other eight sources (`checker.py`, `evidence.py`, `harness.py`, `helper_alt.c`,
`helper_dynamic.c`, `helper_setid.c`, `journal.py`, `oracles.py`) and the other two definitions are
byte-identical to `ba41a3f`. The Git-derived set equals the freeze author's reported set and the
manifest's own rehash list in `supersession_reason`.

Outside the bound set, the directory also differs from `ba41a3f` in three unhashed files:
`BUILD-EVIDENCE.md` (the `7f98d4d` "Build 7 no longer binds" section), `SOURCE-HASHES.json`, and
the added superseded record. No frozen byte is unexplained.

**TRIAL3_FROZEN_BYTE_DRIFT_ACCOUNTED**

## 5. Manifest semantics

| Requirement | Committed [manifest](../experiments/launch-exec-01/SOURCE-HASHES.json) |
|---|---|
| trial | `trial: "trial-003"`; `trial_3.trial: "trial-003"` |
| status | `status: "NOT_RUN"`; `freeze_state: "FROZEN"`; `trial_3.freeze_state: "FROZEN"` |
| valid Trial #3 count | `valid_trial_count: 0`; `trial_3.valid_trial_count: 0` |
| D-7 | `d7_execution_authorised: false`; `trial_3.d7: "NOT_AUTHORISED"` |
| execution | `trial_3.execution: "NOT_RUN"`; `trial_3.dispatcher: "NONE"`; `trial_3.published: false` |
| aggregate | `trial_3.aggregate: null`; no top-level `aggregate` key |
| membership | `case_membership`: total 72, mandatory 54, conditional 11, recorded 7 |
| traced | `traced_cases.cases`: `E1 E7 F4 F7 M1 M2 M3 M4` |
| handlers, posable, unposable | `driver.case_handlers: 72`; `posable_against_this_freeze: 72`; `unposable_against_this_freeze: []`; `unposable_mandatory: []` |
| build | `build.status: "BUILD_8_REQUIRED"`; `build_7_binds: false`; `build_8_evidence: null`; `trial_3.build: "BUILD_8_REQUIRED"` |
| gates | `requires`: the independent freeze review, then publication and Build 8, then a new owner D-7 and a Trial #3 dispatcher behind its own review; "this freeze grants nothing" |
| classification | `TRIAL_3_LOCAL_FREEZE_READY_FOR_INDEPENDENT_REVIEW` — a request for review, not an execution state |

A walk over every value found:

* one boolean `true`, `driver.complete`;
* `MECHANISM_*` strings only where Trial #2 is described (`trial_2`, `lineage`, `freeze_lineage`,
  `supersession_reason`) and in the unchanged artifact-preservation contract's list of possible
  outcomes;
* no field implying that Trial #3 executed, a valid count above zero, a D-7, an aggregate or
  existing Build 8 evidence.

The `trial_2` block matches the committed record: freeze `ba41a3f`, run `34640280964`,
59 / 6 / 6 / 1, `MECHANISM_REJECTED`, D-7 `CONSUMED`, valid count 1, rerun `FORBIDDEN`, manifest
blob `6f000fac…` and SHA-256 `616dc6b3…`.

**TRIAL3_MANIFEST_AUTHORITY_STATE_SOUND**

## 6. Trial-003 active identity

A static trace of every durable artefact a future Trial #3 run would write, from
`run_launch_exec_01.py` at `bebd8a5`:

| Artefact | Identity path | Result |
|---|---|---|
| `journal.jsonl`: `trial_begin`, `preflight`, `build_identity`, `case_entered`, `case_pose_started`, `case_completed`, `trial_end` | `journal.Journal(out / JOURNAL_FILE, sanitiser, trial=TRIAL_ID)` at `:218`; `Journal._append` stamps `record["trial"] = self._trial` (`journal.py:95`) after copying the payload, so no payload can override it | `trial-003` |
| `build-identity.json` | the literal `{"trial": TRIAL_ID, "build": …, "artefacts": …}` at `:243` | `trial-003` |
| `evidence.json` and runner stdout | `evidence.evidence_document` embeds the committed manifest as `freeze` (`freeze.trial`, `freeze.trial_3.trial`); `trial_begin` embeds the same manifest | `trial-003` |
| `preflight.json` | carries no trial field, as in Trial #2's design | n/a |
| output file names | fixed constants in an operator-chosen `--out-dir`; no identity in any name, as in Trial #2's design | n/a |

`TRIAL_ID` is assigned once, at `:178`, and never rebound. An AST walk of all 10 frozen Python
sources finds no second assignment, no `global TRIAL_ID`, no attribute store, and no `Journal`
subclass or wrapper. The only other `"trial"` keys in frozen code (`driver.py:2996` and `:3285`)
are repetition indices inside case records. The runner was not executed in trial mode (§21).

**TRIAL3_ACTIVE_IDENTITY_SOUND**

## 7. `journal.py` default — load-bearing check

`journal.Journal.__init__(self, path, sanitiser, trial="trial-002")` is unchanged since Trial #2;
`journal.py` has SHA-256 `63931640…` at both freezes. The owner's four conditions:

1. **Every active Trial #3 path passes `trial-003` explicitly.** The 10 frozen Python sources
   contain exactly one `Journal(...)` construction, `run_launch_exec_01.py:218`, with the keyword
   `trial=TRIAL_ID`. The build-identity document uses `TRIAL_ID` directly, and `evidence.json`
   carries the manifest's `trial-003`.
2. **No silent fallback is reachable.** Nothing else constructs a journal, and both files are
   hash-bound: a default-trial constructor added to the runner changes its hash, fails
   `--verify-freeze` and fails the exact-bytes test (§9). Fabricated calls against the extracted
   `bebd8a5` modules, in an isolated interpreter and without running the runner, confirm it:
   * the runner's exact constructor call, driven through all seven record kinds and the real
     sanitiser, reads back `trial-003` on every record;
   * the build-identity document, published through `evidence.publish`, reads back `trial-003`;
   * an evidence document embedding the committed manifest reads back `freeze.trial` and
     `freeze.trial_3.trial` as `trial-003`;
   * a constructor that omits `trial=` yields `trial-002`, so the default is real but unreachable.
3. **Tests pin the identity.**
   * `Trial3Freeze.test_the_trial_identity_is_trial_003` requires
     `runner.TRIAL_ID == "trial-003" == MANIFEST["trial"]`, the source strings `trial=TRIAL_ID`
     and `{"trial": TRIAL_ID,`, and no `"trial-002"` literal in the runner. It also keeps Trial
     #2's preserved journal at `trial-002`.
   * `test_the_manifest_is_the_trial_3_freeze_and_grants_nothing` pins the manifest's `trial` and
     `trial_3`, which `evidence.json` embeds.
   * The exact-bytes test and `--verify-freeze` pin both files' bytes.
   * The pin is textual plus hash-bound. No committed test reads a written Trial #3 record back
     (FR-B1).
4. **The default is not freeze authority.** Only generic journal tests exercise it. Some rely on
   it (`test_launch_exec_01_driver.py:4196–4255`, `test_launch_exec_01_delta.py:864`); the others
   pass `trial-002` explicitly as historical fixtures (`test_launch_exec_01_ab7.py:203`,
   `test_launch_exec_01_final.py:622`, `test_launch_exec_01_driver.py:3443`). The freeze's identity
   authority is the manifest's `trial-003` and the runner's `TRIAL_ID`.

**TRIAL3_JOURNAL_DEFAULT_HARMLESS_EXPLICIT_ID_PROVEN**

## 8. Historical Trial #2 dispatcher

The [Trial #2 dispatcher](../../.github/workflows/launch-exec-01-trial-002.yml) was read and not
modified. Its blob `1d030130…` is identical at `bebd8a5`, at `1f0b86a` and on `origin/main`
(`d8a6887`), and it last changed in `f6425da` and `d8a6887`.

* **One-shot guards before any checkout** (`:79–112`). The event must be `workflow_dispatch`, the
  ref `refs/heads/main`, `run_number` 1, `run_attempt` 1, and the confirmation
  `RUN-TRIAL-002-ONE-VALID-TRIAL`. Any failure exits before checkout.
* **Literal bytes** (`:117–122`). `actions/checkout` uses
  `ref: ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`, never an input or the dispatched tip.
* **Identity gates** (`:124–159`).
  * `HEAD` must equal `FREEZE_SHA` and the tree must be clean.
  * The manifest's SHA-256 and its file and tree blobs must equal Trial #2's `616dc6b3…` and
    `6f000fac…`.
  * The Trial #3 manifest (`ea482c6f…`, blob `8cd290b5…`) fails all three, so `bebd8a5` bytes
    cannot reach the trial step even if a guard were passed.
* **Boundary** (`:232–245`). The runner's D-7 invocation is the only step that can pose a case. It
  runs only after every guard and gate succeeded, and the preservation steps after it never invoke
  the runner.
* **Consumed.** [PROVENANCE.md](../experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/PROVENANCE.md)
  records run `34640280964` as `run_number` 1, `run_attempt` 1. A read-only `gh run list` on
  2026-09-13 returns exactly that one run of workflow `356056797`.
  * A second dispatch has `run_number` of at least 2 and halts before checkout.
  * "Re-run jobs" has `run_attempt` of at least 2 and halts.
  * This guard rests on GitHub's per-workflow run numbering; the literal checkout and the manifest
    gates do not.
* **Not Trial #3 authority.** The file names only Trial #2's D-7, freeze, manifest and final
  review. No workflow names Trial #3:
  * no file name or `.github` content in any local or remote ref matches `trial-003`,
    `RUN-TRIAL-003`, `trial_3` or `trial #3`;
  * `gh workflow list --all` shows five workflows: helm-bind, the Rust workspace, the helm-observe
    review, compile-only and this dispatcher;
  * the compile-only workflow never passes the D-7 flag and asserts that the runner refuses with
    exit 3.

**TRIAL2_DISPATCHER_HISTORICALLY_ISOLATED**

## 9. Active freeze validation

On the working tree, which equals `bebd8a5`:

| Invocation | Result |
|---|---|
| `python run_launch_exec_01.py --verify-freeze` | `{"detail": "frozen sources match the manifest", "freeze_verified": true}`, exit 0 |
| `python run_launch_exec_01.py`, no authority | `status: NOT_RUN`, "D-7 (execution authorisation) is not granted … No case was posed.", exit 3 — a refusal by design, not a freeze failure |

Independent tampers were applied to a disposable `git clone --shared` of this repository at
`bebd8a5`. Each was checked with the clone's own committed verifier and with
`Trial3Freeze.test_the_closed_input_set_is_bound_exactly` and
`Trial3Freeze.test_the_exact_frozen_bytes_verify`, then reverted:

| Tamper | `--verify-freeze` | Closed-set test | Exact-bytes test (sources and definitions) |
|---|---|---|---|
| none (baseline) | true, exit 0 | OK | OK |
| A1: one byte in `checker.py` | **false**, exit 1, names `checker.py` | OK | **FAIL** |
| A2: one byte in `launcher_spike.c` | **false**, exit 1, names `launcher_spike.c` | OK | **FAIL** |
| B: `helper_setid.c` missing | **false**, exit 1, `helper_setid.c: missing` | **FAIL** | **FAIL** |
| C1: undeclared `undeclared_input.py` | true — source-only by design | **FAIL** | OK |
| C2: undeclared `undeclared_input.c` | true — source-only by design | **FAIL** | OK |
| D1: one byte inside `LAUNCH-EXEC-01-DEFINITION.md` | true — source-only by design | OK | **FAIL** (definition drift) |
| D2: one byte in `ADR-0024-launch-authority.md` | true — source-only by design | OK | **FAIL** (definition drift) |
| restored | true, exit 0 | OK | OK |

The clone was clean after restoration and was then deleted.

The closed-set test covers every file kind the frozen code consumes from the experiment directory:

* `harness.build` compiles named targets only, and no frozen C source includes a local header;
* the only directory scans in frozen code read `/proc/sys/fs/binfmt_misc`, the build directory and
  `/proc/<pid>/fd`.

Definition §10, the manifest's `verify_freeze` block and R-M2 all say the verifier is source-only
(§15).

**TRIAL3_ACTIVE_FREEZE_VALIDATION_SOUND**

## 10. Retirement of the freeze-plus-delta model

* `test_launch_exec_01_ab7.py`:
  * `test_only_declared_cases_may_fail_without_a_rule_token` reads
    `posing_versus_showing.decisive_without_token.assertions` from the Trial #3 manifest;
  * `AB7Preregistration` compares that set with `ob.DECISIVE_WITHOUT_TOKEN_ASSERTIONS` directly;
  * the Build 7 test reads the Trial #2 manifest with `git show ba41a3f:…` and requires the working
    bytes to equal the Trial #3 manifest.
* `test_launch_exec_01_final.py`: `PreregistrationAgrees` reads only the manifest. The posed checks
  in use must equal the preregistered set, and the manifest's input map must equal them. The
  manifest's assertions must equal `ob.ASSERTIONS`, with exact cases, reads and violation tokens.
* `test_launch_exec_01_trial3.py`:
  * `Trial3Freeze` takes its expected paths as literals of this freeze and its expected hashes
    from the committed manifest, never from the file under test;
  * the Trial #2 checks fetch `ba41a3f` with `git show` (`Trial2StaysImmutable`,
    `CandidateRecord.test_its_historical_drift_is_the_difference_between_the_two_manifests`,
    `UnchangedByTheCorrection`).
* The superseded record is still read in two places, both as provenance:
  * Trial #2's historical T1 and S4 prediction literals, compared with `ba41a3f` bytes;
  * `CandidateRecord`, which checks the supersession, the committed-against-committed drift and
    that the declared delta is the code.
* An edit to that unhashed record can only add failures. No test takes a union with it, and
  `verify_freeze` never reads it.
* Arbitrary additions are refused (§9, C1 and C2). The current bytes pass only because the manifest
  hashes match (§9, A1 to D2).

**TRIAL3_OLD_DELTA_AUTHORITY_RETIRED**

## 11. Candidate record supersession

`TRIAL-3-CORRECTION-CANDIDATE.json` at `bebd8a5`:

* **State.** `record: TRIAL_3_CORRECTION_CANDIDATE`, `state: SUPERSEDED_BY_TRIAL_003_FREEZE`. No
  `NOT_FROZEN` claim remains, and `is_not` now says the Trial #3 manifest replaced the arrangement
  the record described.
* **Grants nothing.** `trial_3_authorised: false`, `trial_3_d7: null`, `trial_3_executed: false`,
  and `requires`: "nothing … This record grants nothing."
* **Bound to the manifest.** `superseded_by` names `trial-003`, `SOURCE-HASHES.json`, `FROZEN`,
  `authority: "none"` and the manifest SHA-256 `ea482c6f…`, which recomputes equal.
* **No cycle.** The manifest names the record by path and state only, with
  `authority: "none; this manifest alone is the Trial #3 freeze"`, and lists it in
  `not_hashed_here`. The record can therefore carry the manifest's hash without circularity.
* **Provenance preserved.** Scope, the declared drift lists, `contract_delta`, the three review
  entries (`ee8cd91`, `db45336`, `030d88a`) and the historical `build` block are kept.
* **Not read by the runner.** `run_launch_exec_01.py` never names it.

**TRIAL3_CANDIDATE_SUPERSESSION_SOUND**

## 12. Global case shape

Computed in an isolated interpreter from the experiment directory extracted at `bebd8a5`, and at
`ba41a3f` for comparison. The computation walks the frozen tables and plans and calls no handler.

| Fact | `bebd8a5` | `ba41a3f` |
|---|---|---|
| total / mandatory / conditional / recorded, from `fc.summary()` and counted from each entry's `cls` | 72 / 54 / 11 / 7 | 72 / 54 / 11 / 7 |
| unique membership names | 72 of 72 | 72 of 72 |
| traced, from `summary` and from each entry's `traced` | `E1 E7 F4 F7 M1 M2 M3 M4` | same |
| `driver.completeness()` | complete; frozen 72, driver 72; nothing missing, unknown, duplicated or unresolved | same |
| `CASE_PLANS`, plan list, unique plans | 72, 72, 72 — equal to the membership set | same |
| `driver.unposable_cases()` | `{}`: 72 statically posable, 0 unposable | same |
| protected vocabulary, `len(evidence.vocabulary())` | **409** | 405 |

Membership order is identical. A per-case diff over every frozen-case field shows that only T1 and
S4 differ, in `predict` and `note`, which is the reviewed correction. No class, traced flag, gate,
safe set or block cause moved.

The four added vocabulary tokens are exactly `TimedOut:KilledByLauncher:SIGTERM`,
`helper_exit_contradicted`, `helper_exit_corroborated` and `launcher_receipt_claim`; none was
removed. The manifest's `tokens_protected: 409` is the number the committed tooling checks, and it
equals this recomputation.

**TRIAL3_GLOBAL_FROZEN_SHAPE_SOUND**

## 13. Reviewed S4 facts, frozen

Fabricated receipts and reports were scored with `driver.evaluate` and `checker.score_case` from
the extracted `bebd8a5` modules. S4's plan: rule `launcher_receipt_claim`, assertions
`executed_marker` and `helper_exit_corroborated`, prediction `ExecStatusIndeterminate`, class
mandatory, no posed check.

| Observation | Status | Outcome | `helper_exit_corroborated` |
|---|---|---|---|
| `ExecFailed`, stage `EXEC`, errno 7 (absent from `ERRNO_NAMES`), no report | **FAIL** | `helper_exit_contradicted` | violated |
| `ExecFailed` at `SETPGID` with errno 11, and at `CLOSE_RANGE` with errno 4095, no report | **FAIL** | `helper_exit_contradicted` | violated |
| accepted, `Exited` 7, `CLD_EXITED`; the report absent, stream-incomplete, truncated, malformed, or parsed with no declared exit | **INVALID** | none | unobservable |
| `Exited` 7, `CLD_EXITED`, and a complete report naming `helper_report` and declaring exit 7 | **PASS** | `ExecStatusIndeterminate` | holds |

**Claim versus termination.** On the accepted path the claim rule renders
`ExecStatusIndeterminate`, which equals the prediction, and `_s4_receipt_exit` holds. The claim
alone never contradicts: without a report the case is INVALID, not FAIL.
`DECISIVE_WITHOUT_TOKEN_ASSERTIONS` is `{helper_exit_corroborated, no_executed_image}`.

The freeze changed no S4 byte, so the accepted quirks RR-B1, RR-B3 and RR-B5 keep their
reachability.

**TRIAL3_S4_REVIEWED_SEMANTICS_FROZEN_EXACTLY**

## 14. Reviewed P1, P2 and P4 facts, frozen

The same method, applied to each of P1, P2 and P4 (class recorded, rule `descendant_lifecycle`, no
prediction), with the liveness byte both absent and present:

| Fixture health | Status | Outcome |
|---|---|---|
| no health observation; unreadable; empty, with no `PROBE_REACHED`; an early `LIVENESS_OPEN_FAILED:2`; `LIVENESS_UNEXPECTED_OPEN`; malformed | **INVALID** | none |
| `PROBE_REACHED` with `LIVENESS_OPEN_FAILED:2` or `LIVENESS_WRITE_FAILED:32` | **INVALID** | none |
| `PROBE_REACHED`, no failure, no liveness byte | eligible | `descendant_died` |
| `PROBE_REACHED`, no failure, a liveness byte | eligible | `descendant_survived` |

All 54 rows were scored, and the three cases agree on every row. Both eligible rows score PASS,
because the recorded cases carry no prediction.

The ordering in the frozen bytes:

* **Descendant** (`helper_fork.c:194–213`): stdio release unless `--retain-stdio`, then `setsid`
  when `--setsid`, then `liveness_probe`, then the blocking liveness open.
  * `liveness_probe` (`:127–148`) opens the health FIFO without blocking, and writes
    `PROBE_REACHED` only after the non-blocking liveness write open fails with `ENXIO`.
  * It closes the gate's write end last, whether or not the probe succeeded.
* **Direct child** (`:241–250`): reads the gate to end-of-file, then returns.
* **Harness** (`driver.py:3716–3724` and `:3855–3877`): opens the liveness read end only after
  `launch()` returned, and reads the health channel after the rendezvous. The gate is therefore
  released before any liveness byte can exist.
* **Launcher** (`launcher_spike.c:969–993`): sweeps only after its drain loop, which ends on the
  direct child's exit, an exec failure or the deadline.
* **Plans:** P1 `--release-stdio`, P2 `--release-stdio --setsid`, P4 `--retain-stdio --setsid`.
* **Rule** (`observations.py:1850–1875`): reads only `liveness_fixture_health` and
  `descendant_alive_after_launch`. P2's `setsid` ordering is fixture construction and decides
  nothing.

**TRIAL3_P_REVIEWED_SEMANTICS_FROZEN_EXACTLY**

## 15. Approved wording fixes

**A. RR-M1, §10.9 "Ordering".** The text says the gate:

* "waits only until the descendant has done its case's own preparation — stdio release for P1 and
  P2, `setsid` for P2 and P4 — and reached the probe point";
* "is fixture synchronisation, never a result";
* "releases the direct child before any liveness byte can exist, because the harness opens the
  liveness reader only after `launch()` returned";

and that survival "is decided by the launcher's own lifecycle and group sweep alone". Each clause
matches the plans and the source order in §14. When the probe itself fails, the gate still closes
and the case is INVALID, which the §10.9 table already states.

**RR_M1_P2_GATE_WORDING_SOUND**

**B. RR-M3, §10.4.** The false universal "stays INVALID" is gone. The text now separates three
things:

* **RR-I1's structural authority.** A malformed `ExecFailed` record gains none, and S4's
  launcher-side check never decides on it.
* **The unchanged shared rule.** Where it still renders a token, S4 FAILs on that token.
* **Other decisive channels.** A report declaring another exit still FAILs S4; otherwise the case
  is INVALID.

Fabricated rows confirm each clause:

| Row | Status | Outcome |
|---|---|---|
| named errno 13 beside an empty stage | FAIL | `ExecFailed:EACCES` |
| named errno 13 beside a `KilledByLauncher` timeout | FAIL | `ExecFailed:EACCES` |
| errno 7 beside an empty stage | INVALID | — |
| errno `"13"`, no report | INVALID | — |
| errno `"13"` and a report declaring exit 3 | FAIL | `helper_exit_contradicted` |
| errno `"13"` and a report declaring exit 7 | INVALID | — |
| stage `UNKNOWN`, errno 7 | INVALID | — |

The text's parser and launcher claims hold as well:

* `parse_spike_stdout` rejects stage `UNKNOWN` and accepts an empty stage beside a named errno;
* `launcher_spike.c` assigns `timeout_disposition` only in the non-`ExecFailed` branch
  (`:1011–1023`) and prints `stage_name()` for `ExecFailed` alone (`:1047`), which is never `""`.
  It cannot emit either malformed shape.

**RR_M3_S4_WORDING_SOUND**

**C. R-M2, §10 "Two verifications, not one".** The text says "`--verify-freeze` checks the
manifest's 17 source hashes and nothing else; it does not read `definition_sha256`. The three
definition hashes are verified separately, by the manifest-coupled freeze tests in `tools/tests`."
This matches:

* `verify_freeze()`, which iterates `manifest["sha256"]` only (`run_launch_exec_01.py:70–87`);
* the manifest's `verify_freeze` block;
* `Trial3Freeze.test_the_exact_frozen_bytes_verify` and tampers D1 and D2.

README.md says "a verified source freeze", and PROJECT_STATE says the manifest "binds" 17 source
and 3 definition hashes. No text in the freeze delta overclaims the verifier. The historical Trial
#2 dispatcher carries an older sentence that does (FR-B5).

**R_M2_FREEZE_VERIFIER_WORDING_SOUND**

## 16. README and project state

* **README.md** separates the two trials.
  * Trial #2 is complete: `ba41a3f`, run `34640280964`, 59 / 6 / 6 / 1, `MECHANISM_REJECTED`,
    D-7 consumed, valid count one, never to be rerun.
  * Trial #3 (`trial-003`) is FROZEN and NOT_RUN: no case posed, valid count zero, no result.
    "D-7 for Trial #3 is NOT granted, and no Trial #3 dispatcher exists." The sources are frozen
    for review, not for execution, and formal Build 8 evidence is still required.
* **PROJECT_STATE.md**, in its new top
  [section](../PROJECT_STATE.md#launch-exec-01-trial-003-freeze):
  * "cut locally and is not pushed", with the manifest SHA-256 of §3 and the identifier
    `trial-003`;
  * the three wording corrections, the superseded record and the shape of §12;
  * "Build 8 is required and does not exist yet";
  * Trial #3 "NOT_RUN, valid count zero, no D-7 and no dispatcher. The freeze is not published.
    One independent Trial #3 freeze review is required before publication and Build 8."
* Neither claims remote publication, Build 8, a Trial #3 execution, a D-7 or an aggregate. The
  older PROJECT_STATE sections below keep their dated wording, and the new top section supersedes
  them (FR-B4).

**TRIAL3_REPOSITORY_STATE_DOCUMENTATION_SOUND**

## 17. DECISIONS and BUILD-EVIDENCE non-changes

* **DECISIONS.md records owner decisions**: D-7 grants, acceptances and the
  [postmortem decisions](../DECISIONS.md#trial-002-postmortem-decisions).
  * None of the three earlier freeze commits, `f417984`, `ab74356` and `ba41a3f`, touched it.
  * It holds no Trial #3 D-7, and a freeze is not an owner decision, so no entry is due.
* **[BUILD-EVIDENCE.md](../experiments/launch-exec-01/BUILD-EVIDENCE.md) records builds and their
  binding.** The Trial #2 precedent has the same shape as Trial #3:
  * the correction `5e04642` appended "Build 6 no longer covers `launcher_spike.c`";
  * the freeze `f417984` did not touch the file;
  * Build 7 was recorded after publication, in `2591f35`.

  `ab74356` and `ba41a3f` appended "Build 7 still binds" sections only because each asserted that
  an existing build still bound the new freeze. For Trial #3, `7f98d4d` already appended "Build 7
  no longer binds the Trial #3 correction candidate", and there is no build to record. A section
  added now would be a fake build record.
* **The freeze state lives in PROJECT_STATE.md**, together with the manifest's `build` block and
  definition §10.12.

**TRIAL3_DECISION_AND_BUILD_RECORD_BOUNDARY_SOUND**

## 18. Build 7 and Build 8 boundary

Build 7 (run `34575558065`) compiled the C bytes recorded in BUILD-EVIDENCE.md's "Build 7 still
binds the AB7 correction freeze" table:

| C source | Build 7 | Trial #3 freeze | Changed |
|---|---|---|---|
| `launcher_spike.c` | `0a45447b…` | `4d6a0ac7…` | **yes** |
| `helper_report.c` | `99770ed8…` | `ceaac0dc…` | **yes** |
| `helper_fork.c` | `a62a6a1e…` | `52089697…` | **yes** |
| `helper_alt.c` | `7993aa63…` | `7993aa63…` | no |
| `helper_dynamic.c` | `b3c398d8…` | `b3c398d8…` | no |
| `helper_setid.c` | `c609755d…` | `c609755d…` | no |

The changed set is exactly `helper_fork.c`, `helper_report.c` and `launcher_spike.c`, which is the
manifest's `c_sources_changed_since_build_7`.

* `harness.py`, which holds the build commands, is unchanged.
* `make_fixtures.py` changed through the reviewed fixture-mode correction; definition §10.2
  records the fixture bytes as unchanged.
* Every current-state document — the manifest, definition §10.12, README.md and PROJECT_STATE.md —
  says Build 7 does not bind Trial #3. The "Build 7 still binds" sentences that remain are dated
  Trial #2 records.
* No compile was performed in this review. WSL has no C compiler on `PATH`, and the compiled-image
  E6 test skipped there.

**BUILD_8_REQUIRED_FOR_TRIAL_3_FREEZE**

**BUILD 8 EVIDENCE DOES NOT YET EXIST.**

## 19. Trial #2 historical integrity

Replayed in an isolated interpreter with all 17 `ba41a3f` sources, each extracted with
`git cat-file` and verified against the `ba41a3f` manifest. The working Trial #3 manifest was never
read. The five evidence files were extracted at `bebd8a5`; their SHA-256 equal PROVENANCE.md's, and
their bytes equal `8a9dc77`'s.

| Check | Result |
|---|---|
| journal | 216 records, not torn, every record `trial-002` |
| replay | 72 entered, 72 completed, 68 pose-started; first `case_pose_started` is `E1` at `n = 4`; `TRIAL_COMPLETED`, `d7_consumed: true` |
| `trial_end` | `RUN`, `MECHANISM_REJECTED`, 59 / 6 / 6 / 1 |
| `checker.report` over the recovered records | **59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED**, **`MECHANISM_REJECTED`**, and a `detail` equal to the published one |
| all 72 status/reason pairs | published `evidence.json` = journal `case_completed` = `score_case` on the journal records = `score_all` = `report.statuses` = `score_case` on the published records: **zero mismatches** |
| embedded freeze identity | `trial-002` |

These diffs are all empty:

* `git diff 8a9dc77 bebd8a5` over the evidence directory;
* `git diff 1f0b86a bebd8a5` over the evidence directory and the result review;
* `git diff b20c2b0 bebd8a5` over the postmortem diagnostics and the result review.

The freeze commit touches no evidence and no earlier review record.

**TRIAL2_HISTORICAL_INTEGRITY_SOUND**

## 20. No execution authority

* **Counts and D-7.** The valid Trial #3 count is 0 in the manifest, the definition, README.md and
  PROJECT_STATE.md. The manifest has `d7_execution_authorised: false` and `d7: NOT_AUTHORISED`,
  and DECISIONS.md holds no Trial #3 D-7.
* **No dispatcher.** No Trial #3 workflow exists in any ref or in `gh workflow list --all` (§8).
* **No run artefacts.** There is no Trial #3 evidence directory, no execution record and no
  aggregate. The only commits naming Trial #3 are the correction, review and freeze commits.
* **No bypassing wording.** A search for `READY_FOR`, "ready to execute" or "ready to run",
  `trial_3_authorised: true`, "Trial #3 authorised", and D-7-granted-for-Trial-#3 phrasings finds
  only:
  * Trial #2 and other-module classifications;
  * negative statements such as "NO TRIAL #3 IS AUTHORISED";
  * the micro review's `…_READY_FOR_FREEZE_PREPARATION`;
  * the manifest's `…_READY_FOR_INDEPENDENT_REVIEW`.
* **Gates in order.** Every Trial #3 status text lists the remaining gates: the freeze review,
  publication and Build 8, a new owner D-7, and a reviewed dispatcher.
* **The runner still refuses.** Without the flag it exits 3. With the flag it can pose only after
  `--verify-freeze`, and no workflow passes the flag for Trial #3.

**TRIAL3_NO_EXECUTION_AUTHORITY_SOUND**

## 21. Validation

| Check | Result |
|---|---|
| `git diff --check 030d88a bebd8a5` and the working tree | clean |
| `python tools/validate_docs.py` | PASS before this record (123 Markdown files, 251 JSON files, 1,280 link targets) and PASS with it (124, 251, 1,288) |
| `cargo fmt --check` | PASS |
| Windows 11, Python 3.14.3: `python -m unittest discover -s tools/tests` | **783 tests OK, 74 skipped** (POSIX-only) |
| WSL Ubuntu 24.04.4, kernel `6.6.87.2-microsoft-standard-WSL2`, Python 3.12.3: the same command at `bebd8a5` | **783 tests OK, 1 skipped** — `test_a_compiled_scratch_helper_has_one_region_in_one_symbol`, no C compiler on `PATH` |
| source freeze verification | `--verify-freeze` → `freeze_verified: true`, exit 0 (§9) |
| definition hash verification | 3 / 3 recomputed (§3); `Trial3Freeze.test_the_exact_frozen_bytes_verify` OK, and FAIL under D1 and D2 |
| default invocation | NOT_RUN refusal, exit 3 |
| independent audits | manifest identity and 17 + 3 hashes, Git-derived drift, global shape and vocabulary, fabricated identity, S4 and P scoring, Trial #2 replay, clone tampers |

Only these operations were used: Git reads, a read-only `git fetch`, read-only `gh run list` and
`gh workflow list`, hashing, static inspection, fabricated Python observations in isolated
interpreters, scratch extractions, and one disposable clone, since deleted. The runner's trial
mode, `Authorisation`, `pose`, `observe` and `_launch_and_observe` were not used. No workflow was
dispatched, nothing was pushed, and nothing was compiled.

**Experimental LAUNCH-EXEC ELF executions: ZERO. Cases posed: ZERO.**

## 22. Findings

No BLOCKER. No IMPORTANT. No MINOR.

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| FR-B1 | BACKLOG_NONBLOCKING | Identity test strength | No — both files are hash-bound, and §7 proves one explicit constructor | `test_the_trial_identity_is_trial_003` pins `trial-003` through source substrings and the manifest. No committed test reads a written Trial #3 journal or build-identity record back, and the substrings alone would not catch an added default-trial constructor | Optional: a read-back assertion in the existing fabricated `run_trial` journal tests; `tools/tests` is not a frozen input |
| FR-B2 | BACKLOG_NONBLOCKING | `journal.py` historical wording | No — every active path passes `trial=TRIAL_ID` | Beside the owner-accepted `trial="trial-002"` default, the `case_pose_started` docstring says "the Trial #2 immutability boundary" and `project_status` says "the frozen Trial #2 reading". The manifest's `durable_evidence.journal.boundary` carries the general wording | None; tidy only if `journal.py` is ever rehashed |
| FR-B3 | BACKLOG_NONBLOCKING | Freeze-time state in future evidence | By design — every Trial #3 `trial_begin` and `evidence.json` embeds the manifest | The embedded `status: NOT_RUN`, `trial_3.execution: NOT_RUN`, `d7: NOT_AUTHORISED`, `dispatcher: NONE`, `published: false` and `independent_freeze_review: REQUIRED` describe the moment of the freeze, and will sit inside a RUN document, as Trial #2's `NOT_RUN` and `d7_execution_authorised: false` did | The Trial #3 dispatcher review and result review should state that the embedded block is freeze-time state; D-7 authority comes from DECISIONS.md |
| FR-B4 | BACKLOG_NONBLOCKING | Dated closing sentences | No — dated, append-only records | BUILD-EVIDENCE.md's 2026-09-12 section ends "no Trial #3 freeze exists"; DECISIONS.md's 2026-09-12 decision ends "No Trial #3 exists"; PROJECT_STATE's candidate section says no freeze exists, below the new section that supersedes it | The Build 8 section should name the freeze and the manifest SHA-256 it binds; earlier records stay unedited |
| FR-B5 | BACKLOG_NONBLOCKING | Historical dispatcher comment | No — the Trial #2 dispatcher cannot reach Trial #3 bytes (§8) | `launch-exec-01-trial-002.yml:161` says "The frozen verifier is authoritative for every file the manifest covers", but `--verify-freeze` checks source hashes only (R-M2); that dispatcher bound the definitions separately, with `git diff --quiet` | Leave the file unchanged. A Trial #3 dispatcher must not inherit the sentence and must bind the definition hashes explicitly |

**Carried unchanged and not reopened:** RR-M2 (MINOR, accepted); RR-B1 to RR-B5; R-B1 and R-B2;
AB7-N1 to AB7-N3; T2-R5 to T2-R8; T2-C-LATENT. The freeze commit changes no executable semantics,
so their reachability is unchanged.

None of the owner's BLOCKER or IMPORTANT conditions holds:

* the manifest hashes match, and the frozen-input set is the established closed set;
* no reachable Trial #3 path emits `trial-002`, and the identity is right;
* the delta holds no unreviewed semantics, and active validation no longer relies on the manifest
  plus a delta;
* no case, class or traced flag drifted;
* the candidate record does not conflict with the manifest;
* Trial #2 is not mutated;
* there is no accidental D-7 or execution authority, no privacy-relevant change, and no false
  Build 7 claim.

## 23. Verdicts

| Area | Verdict |
|---|---|
| Freeze delta | **TRIAL3_FREEZE_DELTA_MECHANICAL_AND_REVIEWED** |
| Manifest bytes | **TRIAL3_MANIFEST_BYTE_INTEGRITY_SOUND** |
| Byte drift | **TRIAL3_FROZEN_BYTE_DRIFT_ACCOUNTED** |
| Manifest authority state | **TRIAL3_MANIFEST_AUTHORITY_STATE_SOUND** |
| Active identity | **TRIAL3_ACTIVE_IDENTITY_SOUND** |
| `journal.py` default | **TRIAL3_JOURNAL_DEFAULT_HARMLESS_EXPLICIT_ID_PROVEN** |
| Trial #2 dispatcher | **TRIAL2_DISPATCHER_HISTORICALLY_ISOLATED** |
| Active freeze validation | **TRIAL3_ACTIVE_FREEZE_VALIDATION_SOUND** |
| Old delta authority | **TRIAL3_OLD_DELTA_AUTHORITY_RETIRED** |
| Candidate record | **TRIAL3_CANDIDATE_SUPERSESSION_SOUND** |
| Global shape | **TRIAL3_GLOBAL_FROZEN_SHAPE_SOUND** |
| S4 | **TRIAL3_S4_REVIEWED_SEMANTICS_FROZEN_EXACTLY** |
| P1, P2, P4 | **TRIAL3_P_REVIEWED_SEMANTICS_FROZEN_EXACTLY** |
| RR-M1 wording | **RR_M1_P2_GATE_WORDING_SOUND** |
| RR-M3 wording | **RR_M3_S4_WORDING_SOUND** |
| R-M2 wording | **R_M2_FREEZE_VERIFIER_WORDING_SOUND** |
| Repository state | **TRIAL3_REPOSITORY_STATE_DOCUMENTATION_SOUND** |
| Decision and build records | **TRIAL3_DECISION_AND_BUILD_RECORD_BOUNDARY_SOUND** |
| Build | **BUILD_8_REQUIRED_FOR_TRIAL_3_FREEZE** |
| Trial #2 | **TRIAL2_HISTORICAL_INTEGRITY_SOUND** |
| Execution authority | **TRIAL3_NO_EXECUTION_AUTHORITY_SOUND** |

## 24. Authority and next gate

**TRIAL #2 D-7 IS CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE.**

**TRIAL #2 MUST NOT BE RERUN.**

**TRIAL #3 FREEZE EXISTS LOCALLY.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ZERO.**

**TRIAL #3 D-7 IS NOT AUTHORISED.**

**TRIAL #3 MUST NOT BE EXECUTED.**

There is no BLOCKER or IMPORTANT finding, and every load-bearing freeze verdict is SOUND:

**TRIAL #3 FREEZE IS READY FOR PUBLICATION AND FORMAL BUILD 8 COMPILE-ONLY EVIDENCE.**

That readiness grants nothing. This review is only the first of the four steps that
[definition section 10](../experiments/LAUNCH-EXEC-01-DEFINITION.md#10-trial-3-freeze--frozen-not-authorised-not-run)
requires. Publication with formal Build 8 evidence, a new owner D-7 and a reviewed Trial #3
dispatcher remain separate gates, in that order.

Classification: **TRIAL_3_FREEZE_REVIEW_PASSED_READY_FOR_PUBLICATION_AND_BUILD_8**
