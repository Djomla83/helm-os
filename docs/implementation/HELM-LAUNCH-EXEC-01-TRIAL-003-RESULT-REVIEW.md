# LAUNCH-EXEC-01 — Trial #3 result review

> **THIS IS AN INDEPENDENT REVIEW OF THE PRESERVED TRIAL #3 RESULT.
> IT DOES NOT CHANGE THE FROZEN RESULT AND DOES NOT DIAGNOSE X2c.**

This review covers the single execution authorised under the Trial #3 D-7. The reviewer did not
author the frozen experiment, the dispatcher, the trial workflow or the evidence-preservation
commit. Nothing was fixed, re-run, re-dispatched or posed. No experimental ELF was executed or
compiled, and no frozen or preserved byte was touched.

**Trial #3 completed. Its frozen aggregate is `MECHANISM_REJECTED` (70 PASS / 1 FAIL / 1 BLOCKED),
and that result is sound: it reproduces exactly, offline, from the preserved evidence and the
frozen `bebd8a5` checker.**

The review keeps two questions apart:

* **A. What the frozen trial decided.** The preregistered checker decides this from the preserved
  records. It is covered here.
* **B. Why X2c produced that result.** This is **out of scope**. Section 17 lists open questions
  only; it gives no diagnosis.

Every value below was recomputed or re-read from a primary source during this review: fresh GitHub
API responses, a fresh job log, two fresh artifact downloads, the Git object store and the frozen
modules archived from `bebd8a5`. None was copied from `PROVENANCE.md`, the preservation commit
message or the owner instruction.

## 1. Starting state

| Check | Observed |
|---|---|
| Worktree | clean |
| Branch | `docs/helm-launch-architecture` |
| `HEAD` (preservation commit) | `11bddc5ca8a56f5b71f817f6aa28c01a7f26c93b` |
| Parent | `0c5652d8bd429aa93eafd15a3d3da44f92cfd3ee` |
| `origin/docs/helm-launch-architecture` (after a read-only fetch) | `0c5652d8bd429aa93eafd15a3d3da44f92cfd3ee` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Preservation commit on any remote branch | no, so it is **local only** |

## 2. Preservation commit scope

`git diff-tree -r 11bddc5` lists exactly seven paths:

| Status | Path | Blob |
|---|---|---|
| M | `.gitattributes` | `650491f0…` |
| A | `…/PROVENANCE.md` | `00badf18…` |
| A | `…/preflight.json` | `9759a116…` |
| A | `…/build-identity.json` | `7ec7b4e8…` |
| A | `…/journal.jsonl` | `8b85af5a…` |
| A | `…/evidence.json` | `01fc3705…` |
| A | `…/runner-stdout.json` | `01fc3705…` |

`git diff 0c5652d 11bddc5` is empty over `docs/DECISIONS.md`, `docs/PROJECT_STATE.md`,
`docs/experiments/launch-exec-01/`, `docs/adr/`, `docs/research/`, `.github/`, `crates/` and
`tools/`. So the commit changed no frozen source, definition, manifest, dispatcher, decision record,
project state, source, test or tool.

The `.gitattributes` change adds two lines. They pin the Trial #3 `*.json` and `*.jsonl` files as
`-text whitespace=cr-at-eol`, the same rule Trial #2's directory has. `PROVENANCE.md` stays ordinary
`text eol=lf`. Its third line reads **"This file is NOT Trial #3 evidence."**

**`TRIAL3_PRESERVATION_SCOPE_SOUND`**

## 3. Execution identity

Fresh Actions API data:

| Field | Observed |
|---|---|
| Run | `34883316368` |
| Job | `104107679380` (`trial-003`, the only job; `total_count: 1`) |
| Workflow | id `357912663`, name `launch-exec-01 trial-003 one-shot trial`, path `.github/workflows/launch-exec-01-trial-003.yml` |
| Event | `workflow_dispatch` |
| Head branch / SHA | `main` / `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| `run_number` / `run_attempt` | `1` / `1` |
| Run conclusion / job conclusion | `success` / `success` |
| Run started / job started / job completed (UTC) | 2026-09-14T18:51:04Z / 18:51:08Z / 18:52:43Z |
| Runner image | `ubuntu-24.04`, version `20260907.300.1` |

The workflow's run list has `total_count: 1`, and that one run is `34883316368`.
`attempts/1` exists and `attempts/2` returns **404**. **There is exactly one Trial #3 run, with no
second attempt.**

GitHub's `success` is infrastructure status only. It is not the experimental verdict.

**`TRIAL3_EXECUTION_IDENTITY_SOUND`**

## 4. Dispatcher authority

At the exact run head `501a7fa95c4884da4fec9a20a512c2d63f2b30cc`:

| | Computed | D-7 record |
|---|---|---|
| `git rev-parse 501a7fa:.github/workflows/launch-exec-01-trial-003.yml` | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` |
| `git hash-object` of the blob content | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` | — |
| SHA-256 of the blob content | `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8` | `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8` |

The same blob sits at that path in the dispatcher candidate `f197386`, the dispatcher review
`4f19897` and the D-7 record `0c5652d`. The run commit `501a7fa` has parent `d8a6887`, and its only
change against that parent is `A .github/workflows/launch-exec-01-trial-003.yml`. So D-7 operator
invariants 1, 3 and 9 hold for the run that executed.

**`TRIAL3_RUN_USED_AUTHORISED_DISPATCHER_BYTES`**

## 5. Job gates, from the primary job log

The job log was downloaded twice. Both copies are 55,101 bytes with SHA-256
`56e23b35d68e14d2c0e5d6bca2151c74b6904357bb09007e9fd019646cc55ea6` and are byte-identical. The API
step list shows steps 1–12 `success` in order.

| Gate | Log evidence |
|---|---|
| One-shot guard | `event=workflow_dispatch ref=refs/heads/main run_number=1 run_attempt=1`, then `one-shot guards passed` |
| Literal freeze checkout | `actions/checkout@fbc6f399…` with `ref: bebd8a5f83d4d0daebe9b068050cb5436289c75e`; `git checkout --progress --force bebd8a5f…`; `HEAD is now at bebd8a5` |
| Checked-out HEAD | `checked out: bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| Manifest SHA-256 | `manifest sha256: ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` |
| Manifest blob, file and tree | both `8cd290b573408510f8c16cd8dafe676354140a38` |
| Freeze review and Build 8 evidence bindings | `bound to freeze, manifest, freeze review and accepted Build 8 evidence` |
| Frozen bytes through the authority commits | `17/17 frozen sources and 3/3 definitions identical at the freeze, the freeze review and the Build 8 evidence acceptance` |
| 17-source freeze verification | `"freeze_verified": true`, then `freeze_verified: true (17 source hashes)` |
| Definition verification | three `ok` lines (ADR-0024 `c1f3cce8…`, definition `0c7406b4…`, architecture `97fbf0d8…`), then `definition hashes verified: 3/3` |
| Membership | `ok total 72`, `ok mandatory 54`, `ok conditional 11`, `ok recorded 7` |
| Traced set | `ok traced 8 exact` |
| Handlers | `ok driver complete`, `ok 72 frozen`, `ok 72 handlers`, `ok nothing missing, unknown or duplicated`, `ok every setup, parent, rule, check and channel resolves` |
| Posability | `ok 72 posable, 0 unposable` |
| Frozen runner | one step, one `python3 run_launch_exec_01.py --i-have-owner-authorisation-d7 --build-dir … --out-dir …` invocation, 18:51:15Z → 18:52:40Z, `frozen runner exit code: 0` |
| Staging after the runner | the stage step starts at 18:52:40.37Z and prints `ls -l` and `sha256sum` for the five files |
| Upload after staging | `SHA256 digest of uploaded artifact zip is 92ee4703…caeee`; `Final size is 108184 bytes. Artifact ID is 10364580293` |
| Final exit-reporting step | `frozen runner exit code: 0`, `A CI colour is not a verdict; the durable evidence is.` |

The log has no `HALT` output line, no `DRIFT` line and no `##[error]`. Its only warning is the
upload action's Node.js 20 deprecation notice. The job's green colour is the runner's exit code 0,
which the dispatcher gives any completed `RUN`, including `MECHANISM_REJECTED`. **It is not
`MECHANISM_ACCEPTED`.**

**`TRIAL3_EXECUTION_GATES_SOUND`**

## 6. Artifact identity

Artifact `10364580293` was downloaded **twice**, fresh, through the Actions API. The preservation
author's scratch ZIP was not used.

| | Observed |
|---|---|
| Name | `launch-exec-01-trial-003-evidence` |
| Producing run / head | `34883316368` / `501a7fa…` (artifact API `workflow_run`) |
| Size, both downloads | 108184 bytes |
| SHA-256, both downloads | `92ee47033513761ec5947ae10e3175bb78bd0a444799cc4fb3c3f5fbb3bcaeee` |
| Downloads byte-identical | yes (`cmp`) |
| API `digest` / upload step's printed digest | both `sha256:92ee4703…caeee` |
| Created / expires (UTC) | 2026-09-14T18:52:41Z / 2026-12-13T18:51:05Z, not expired |
| ZIP integrity (`testzip`) | no bad member |
| Members | `build-identity.json`, `evidence.json`, `journal.jsonl`, `preflight.json`, `runner-stdout.json`, all at the root, no directory entries, **nothing else** |

**`TRIAL3_ARTIFACT_IDENTITY_SOUND`**

## 7. Preservation fidelity

For each member, the file extracted from each fresh download was compared with `cmp` against the
working-tree file and against `git cat-file -p HEAD:<path>`. All three are identical.

| File | Bytes | SHA-256 (recomputed) | Committed blob |
|---|---:|---|---|
| `preflight.json` | 3713 | `4669c8490811ef5a3bd0fff812c877ee46e7d7c0a830670456e0bc0b5019a892` | `9759a1163ee3ebb22f7240cd2ad8e669a75fda31` |
| `build-identity.json` | 2525 | `49b51f210654f70746d30e7d937757aee0e452f524c28dde8e2ad40c6745cde5` | `7ec7b4e8ffb7babc7570b614e1849f7832b37aeb` |
| `journal.jsonl` | 221576 | `1355e4e8ea9d90d1140a01ed74fd0edb359366f84fb324b3fd27eb8ff654e5d6` | `8b85af5aba997f6fbeb54e5136dc9318e0b21097` |
| `evidence.json` | 372561 | `77db0165b0dc3ce1e3d6e70c695f41ef2d5359909146f8a7eece94265eae13d2` | `01fc37052003a224cc3279113d7fa9913f195505` |
| `runner-stdout.json` | 372561 | `77db0165b0dc3ce1e3d6e70c695f41ef2d5359909146f8a7eece94265eae13d2` | `01fc37052003a224cc3279113d7fa9913f195505` |

* The digests and byte counts also equal the `sha256sum` and `ls -l` output the stage step printed
  on the runner.
* **`evidence.json` and `runner-stdout.json` are byte-identical** and resolve to one blob.
* No file contains a CR or a BOM. Each ends in LF.
* `git ls-files --eol` reports `i/lf w/lf attr/-text` for all five, which is the same as the Trial #2
  evidence files, so no normalisation can rewrite them.
* **No re-serialisation.** Each file is exactly its own frozen re-serialisation:
  `evidence.serialise` for the three JSON documents and `evidence.serialise_line` for each of the
  219 journal lines. So nothing was pretty-printed, sorted or re-encoded after publication.

**`TRIAL3_PRESERVED_BYTES_EXACT`**

## 8. Sanitisation and the publication boundary

The frozen boundary is [`evidence.py`](../experiments/launch-exec-01/evidence.py) at `bebd8a5`, with
SHA-256 `a91e02a9…893a` (it matches the manifest). It defines the rules:

* `Sanitiser.record` keeps keys unchanged.
* It withholds the value of every `INTERNAL_ONLY_KEYS` key: capture prefixes, `acquisition`,
  `lifecycle_uses`, `pidfd_open_calls`, `child_pid`, `raw_trace`, `trace_text` and `strace_output`.
* It blanks host-identity and host-descriptor keys.
* It never reproduces an `environ` value.
* In free text it replaces roots, credential shapes and non-system absolute paths, and turns long
  non-vocabulary runs into `<OPAQUE:sha256[:8]>`.

The run's journal writes every record through that sanitiser (`journal.Journal._append`). The
runner writes the three JSON documents through `evidence.publish`.

These checks ran on the preserved files with the frozen module. A keyword scan was supplementary
only:

| Check | Result |
|---|---|
| **Fixed point.** Frozen `Sanitiser.record`, with runner-shaped roots, applied to `preflight.json`, `build-identity.json`, `evidence.json` and each of the 219 journal records | returns every document **unchanged**. No path, credential shape, long non-vocabulary run or internal-only value survives that the frozen rules would still redact. |
| Internal-only keys present | `child_pid` 16 times, all `<WITHHELD:internal-observation-input>`: `trace.child_pid` of `E1 E7 F4 F7 M1 M2 M3 M4`, once in `evidence.json` and once in the journal. `acquisition` twice, also withheld (the manifest's `vocabulary.acquisition` object). No `raw_trace`, `trace_text`, `strace_output` or capture key appears. |
| Host identity or descriptor keys | none present |
| `environ` entries | none present |
| Credential patterns (the frozen `_CREDENTIAL_PATTERNS`) | zero matches |
| Non-digest, non-vocabulary runs of 28+ characters | zero left |
| pid/tid/fd-named fields with numeric values | none. The only fd-named fields are the booleans `adds_helper_fd` and `exec_fd_at_0`. |
| Host strings (`/home/`, `/tmp/`, `ACTIONS_`, `GITHUB_`, `RUNNER_`, `ghs_`, `[pid `, strace framing) | none. The only `runner` hits are prose words such as `unprivileged_runner` and "a runner crash". |
| Placeholders inside the 72 case **records** | only the eight withheld `trace.child_pid` values |
| Placeholders elsewhere | `preflight.os_release` URLs and five non-system mount points become `<ABSPATH>`; one `<OPAQUE:a2e659da>` sits in A4's plan `helper_args`, which is A4's 4096-byte argument and is data by frozen design; the rest are in the published `freeze` object (below) |

### The published `freeze` object versus `SOURCE-HASHES.json`

The frozen `SOURCE-HASHES.json` at `bebd8a5` was compared semantically with `evidence.json`'s
`freeze` and with the journal's `trial_begin.freeze`.

* **Frozen `Sanitiser.record(SOURCE-HASHES.json)` equals the published `freeze` object exactly.**
  That holds with runner-shaped roots and with no roots at all, so the result does not depend on any
  guessed host path. The journal's `trial_begin.freeze` equals it as well.
* A path-by-path walk finds **20 differing paths**, and no key is added or removed:
  * **18 string paths** carry `<OPAQUE:…>` substitutions (20 tokens):
    * `classification`;
    * `correction_candidate_record.path` and `.state`;
    * `driver.note` (`unposable_against_this_freeze`);
    * `durable_evidence.journal.partial_reading.incomplete_case` and
      `.pose_started_and_no_valid_trial_end` (2 tokens);
    * `freeze_lineage[6]`, `[8]` and `[10]`;
    * `lineage.trial_001_aggregate`, `.trial_001_outcome`, `.trial_001_result_review` and
      `.trial_002_result_review` (2 tokens);
    * `posing_versus_showing.fixture_signal.read`;
    * `posing_versus_showing.posed_check_inputs.retention_observed[0]`;
    * `supersession_reason` (`docs/experiments/LAUNCH-EXEC-01-DEFINITION`);
    * `trial_2.evidence`;
    * `trial_2.result_review`.
  * **1 string path** carries an `<ABSPATH>` substitution: `e5b_correction.trial_001_defect`, where
    `/pread` becomes `<ABSPATH>`.
  * **1 object path** is withheld by key: `vocabulary.acquisition`, because `acquisition` is an
    `INTERNAL_ONLY_KEYS` key.
* For every string path, the published value **is exactly** frozen `Sanitiser.text(original)`.
  Every `<OPAQUE:h>` token equals the first 8 hex digits of the SHA-256 of a qualifying run in the
  original, and no substituted run remains in the output.
* No non-sanitiser change exists. The manifest's `sha256` table, `definition_sha256`, commit SHAs
  and every non-string value survive unchanged. So do the two literal `<OPAQUE:…>` mentions already
  present in the manifest's own prose (`open_findings.F-2`, `vocabulary.not_vocabulary`).
* The placeholders expose an 8-hex digest prefix, never the suppressed text. Every suppressed value
  here is public, committed manifest prose, so nothing private was at stake. The substitutions are
  the frozen sanitiser over-redacting fixed text (R3-B1). They are **not evidence drift**.

**`TRIAL3_PUBLICATION_SANITISATION_SOUND`**

## 9. Preflight and build identity

### `preflight.json`

* Top-level keys are `manifest` and `preflight`.
* `preflight` holds exactly the 19 fields
  [`harness.preflight()`](../experiments/launch-exec-01/harness.py) collects: `arch`,
  `binfmt_misc_entries`, `binfmt_misc_status`, `block_reasons`, `clone3`, `gcc`, `geteuid`, `glibc`,
  `kernel_name`, `kernel_release`, `mountinfo_noexec`, `mountinfo_nosuid`, `noexec_writable`,
  `os_release`, `pagesize`, `parent_no_new_privs`, `ptrace_scope`, `strace` and `strace_version`.
* Every field is a runner-environment fact. **None is a trial, run, time or host identifier.** The
  frozen code collects no timestamp, and it removed `uname -a` precisely so that no nodename enters
  this file.
* `manifest` is `frozen_cases.summary()`, the membership shape.

Byte identity with Trial #2's `preflight.json` is therefore not evidence of a stale copy. Both runs
used runner image `ubuntu-24.04` `20260907.300.1`, kernel `6.17.0-1022-azure`, gcc 13.3.0,
glibc 2.39, strace 6.8, euid 1001 and the same membership shape, so identical facts serialise
deterministically to identical bytes. The Trial #3-specific evidence agrees with the file:

* the journal's durable `preflight` record (`n = 1`, `trial: "trial-003"`) equals
  `preflight.json.preflight`;
* `evidence.json.preflight` equals it too;
* `preflight.json.manifest` equals the frozen `summary()`, `evidence.json.membership` and the
  journal's `trial_begin.membership`;
* no `trial-output/` path is tracked at `bebd8a5`, so the file cannot be a checked-in copy.

### `build-identity.json`

* It parses. Its keys are `trial: "trial-003"`, `build` (six compiled targets) and `artefacts` (12
  objects, each with `size`, `sha256` and an ELF-derived `kind`).
* `artefacts` equals the journal's durable `build_identity` record (`n = 2`) and
  `evidence.json.build.artefacts`. `build` equals `evidence.json.build.targets`, and each of its six
  digests equals the matching artefact digest.
* **Point-of-use bindings.** Of the 72 cases, 71 carry a `build_identity_binding`, and every one is
  `bound: true`:
  * 56 `DIRECT_BASE`;
  * 5 `BYTE_IDENTICAL_CASE_COPY`;
  * 8 `INTENTIONAL_MUTATION_TARGET`;
  * 1 `SYMLINK_TO_BASE` (E4, whose object digest equals `helper_report`'s);
  * 1 `NON_BUILD_OBJECT` (X5, `X5_dir`, which has no base artefact).
* Every `base_sha256` equals this trial's build identity for the named artefact. `N3` has no binding
  because it was blocked before preparation.
* `build_identity` is durable at `n = 2`, before the boundary at `n = 4`.

Build 8's compile-only binaries were not compared with anything. The trial built and hashed its own.

**`TRIAL3_PREFLIGHT_BUILD_IDENTITY_EVIDENCE_SOUND`**

## 10. Journal structure

The preserved journal was read with the frozen [`journal.read`](../experiments/launch-exec-01/journal.py)
and replayed with the frozen `journal.replay`. Both come from the `bebd8a5` archive, and the module's
SHA-256 `63931640…64d6` matches the manifest.

| Check | Result |
|---|---|
| Durable records / lines | **219** / 219 |
| `n` | `0 … 218`, strictly monotonic, no gap and no duplicate |
| `trial` | `"trial-003"` on every record |
| Torn tail | none (`tail_truncated: false`); every line ends in LF and parses |
| Frozen replay | accepts the journal and raises no ordering contradiction; nothing was repaired |

| Kind | Count |
|---|---:|
| `trial_begin` | 1 |
| `preflight` | 1 |
| `build_identity` | 1 |
| `case_entered` | 72 |
| `case_pose_started` | **71** |
| `case_completed` | 72 |
| `trial_end` | 1 |

* 72 distinct cases were entered and 72 distinct cases completed. The entered set equals the
  completed set, and the entry order equals the frozen `MEMBERSHIP` order.
* `entered_not_completed` is empty. No case completed twice.
* Every case orders `case_entered` < `case_pose_started` (where present) < `case_completed`, and
  each case completes before the next is entered.
* **The only case without `case_pose_started` is `N3`**: `case_entered` at `n = 168`, then
  `case_completed` at `n = 169`.
* Every record's `posing_evidence.invocation.pose_started` agrees with the journal for all 72 cases.

**`TRIAL3_JOURNAL_STRUCTURE_SOUND`**

## 11. D-7 consumption

From the frozen source:

* `journal.Journal._append` serialises the sanitised record, calls `write`, `flush()` and
  `os.fsync()`, and only then increments `n` and returns
  ([`journal.py`](../experiments/launch-exec-01/journal.py), lines 89–103).
* `case_pose_started`'s docstring defines it as the immutability boundary and **the moment D-7 is
  consumed**, written and fsynced **before** the launcher is started.
* [`run_launch_exec_01.py`](../experiments/launch-exec-01/run_launch_exec_01.py) calls
  `jrnl.case_pose_started(name, index)` only when `prepared["ready"]`, and calls it immediately
  before `driver.pose(...)`.
* D-7 operator invariant 12 names the same boundary.

From the preserved journal:

* the first durable `case_pose_started` is **case `E1` at `n = 4`**;
* `build_identity` is durable at `n = 2`, before the boundary.

**`TRIAL3_D7_BOUNDARY_CROSSED`**

**TRIAL #3 D-7 IS CONSUMED**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ONE**

**TRIAL #3 MUST NOT BE RERUN**

## 12. Valid completion

* There is exactly one `trial_end`, at **`n = 218`**, the final record: `status: "RUN"`,
  `halts: null`, `aggregate: "MECHANISM_REJECTED"`, `counts: {BLOCKED: 1, FAIL: 1, PASS: 70}`,
  `detail: {failing_cases: ["X2c"]}`.
* The frozen `journal.project_status(replay(...))` returns **`TRIAL_COMPLETED`**,
  `d7_consumed: true`, aggregate `MECHANISM_REJECTED`. It does not return
  `TRIAL_ABORTED_AFTER_BOUNDARY`.
* The `trial_end` aggregate, counts and detail equal `evidence.json`'s. `evidence.json` has
  `status: "RUN"`, `uncontrolled: []` and `notes: []`.

**`TRIAL3_VALID_TRIAL_COMPLETION_SOUND`**

## 13. Frozen case status set

All 72 preserved case records were inspected:

| Status | Count |
|---|---:|
| PASS | **70** |
| FAIL | **1** |
| BLOCKED | **1** |
| INVALID | **0** |

No record carries `not_posed`, and every status is one of the four frozen values.

### X2c — the sole FAIL

| Field | Preserved value |
|---|---|
| Frozen class | mandatory (`frozen_cases`), `traced: false` |
| Frozen prediction / plan rule | `interpreter_ran_with_devfd` (a single prediction, no safe set) |
| Status | **FAIL** |
| Observed outcome | **`ExecStatusIndeterminate`** |
| Checker reason | `outcome 'ExecStatusIndeterminate' is not the single frozen prediction 'interpreter_ran_with_devfd'` |
| Record reason | `the stream that carries the report is complete and holds none: clean exec-status EOF with no positive evidence that the pinned image ever ran; EOF alone never means exec` |
| Invocation | `mechanism_invoked: true`, `pose_started: true`; binding `DIRECT_BASE`, `bound: true` for `script_fixture.sh` |
| Journal | `case_entered` `n = 87`, `case_pose_started` `n = 88`, `case_completed` `n = 89` |

The record reason is the artifact's own statement: the report stream is complete and empty, and a
clean exec-status EOF alone is not positive evidence that the pinned image ran.

### N3 — the sole BLOCKED

| Field | Preserved value |
|---|---|
| Frozen class | conditional, `blocked_if: unprivileged_runner` |
| Status | **BLOCKED** |
| Blocked cause | `unprivileged_runner` (the only entry in `preflight.block_reasons`; `geteuid: 1001`) |
| Invocation | `mechanism_invoked: false`, `pose_started: false`, `blocked_cause: unprivileged_runner` |
| Journal | no `case_pose_started` |

**`TRIAL3_FROZEN_CASE_STATUS_SET_SOUND`**

## 14. Frozen checker replay

**Method.**

1. `git archive bebd8a5f83d4d0daebe9b068050cb5436289c75e docs/experiments/launch-exec-01` was
   extracted into a disposable scratch directory outside the repository.
2. The replay ran under WSL, Ubuntu Python 3.12.3, as `python3 -I -B` from `/tmp`, with the
   archive's directory as the only experiment path on `sys.path`.
3. Every loaded experiment module was confirmed to come from the archive, and each SHA-256 matches
   the manifest:

   | Module | SHA-256 |
   |---|---|
   | `checker.py` | `6201a2dc…a7f9b0` |
   | `journal.py` | `63931640…64d6` |
   | `evidence.py` | `a91e02a9…893a` |
   | `frozen_cases.py` | `d29a58ad…60212` |
   | `oracles.py` | `b056c374…2eeb` |
   | `driver.py` | `84602534…723e1` |

   The sanitiser's fail-closed vocabulary accessor imports `driver.py` (and through it `harness`,
   `make_fixtures` and `observations`). Only `driver.registry_vocabulary()` was called.
4. The preserved journal was loaded as data with `journal.read`, replayed with `journal.replay`, and
   `journal.recovered_records` were scored with `checker.report`.

No handler, setup, build or pose function ran. No ELF was built or executed and no case was posed.

**Result.**

| Compared with `evidence.json` and the journal | Reproduced |
|---|---|
| Aggregate | `MECHANISM_REJECTED`: match |
| Counts | `{PASS: 70, FAIL: 1, BLOCKED: 1}`: match |
| Detail | `{failing_cases: ["X2c"]}`: match |
| All 72 case statuses | match `evidence.json` and every `case_completed.status` |
| All 72 case reasons | match `evidence.json` and every `case_completed.reason`, character for character |
| Case records | every journal `case_completed.record` equals the `evidence.json` record |
| Scoring `evidence.json`'s own records instead of the journal's | the identical report |

**Mechanical verdict analysis** with the frozen `checker.verdict`:

* Rule 1 (any FAIL → `MECHANISM_REJECTED`) fires on exactly `["X2c"]`.
* **X2c alone is sufficient.** With every case PASS except X2c FAIL, the result is
  `MECHANISM_REJECTED`.
* **N3 does not cause rejection.** With every case PASS except N3 at its recorded BLOCKED, the
  result is `MECHANISM_ACCEPTED`, `conditional_blocked_with_cause: ["N3"]`. A conditional case
  BLOCKED on its own frozen `blocked_if` fires neither rule 1 nor rule 2.
* *For calibration only, and not a result:* the preserved statuses with X2c alone changed to PASS
  give `MECHANISM_ACCEPTED`. That is a mechanical property of the frozen algebra. It is not a claim
  about what X2c should have been.
* `score_all` iterates the full 72-case `MEMBERSHIP`, so no status is omitted.

**`TRIAL3_FROZEN_RESULT_REPRODUCES_EXACTLY`**

## 15. Aggregate input digest

**What it commits to.** In
[`run_launch_exec_01.py`](../experiments/launch-exec-01/run_launch_exec_01.py), lines 320–324, the
runner computes:

```
aggregate_input_digest = oracles.digest_of(evidence.serialise_line(records).encode("utf-8"))
```

That is SHA-256 over the compact, sorted-key JSON of `records`: the map from case name to the
**raw** `driver.evaluate` record. That map is the checker's input, and the digest is taken
**before** the publication boundary. The journal sanitises inside `_append`, and `evidence.json` is
sanitised by `publish`. No other construction exists in the frozen tree.

**Recomputation.** The same frozen construction over the preserved records gives:

| Input | Digest |
|---|---|
| journal `recovered_records` | `cd11b4dde23f679850762e2b9367cb8c9e0f1d26b0c358bfabef476b59a9fa18` |
| `evidence.json` case records | `cd11b4dde23f679850762e2b9367cb8c9e0f1d26b0c358bfabef476b59a9fa18` |
| recorded in `trial_end` | `9110ca21be211c574f284acf51f5f256e11a21b206718386f20924b1c2683b2f` |

**They are not equal, and by construction they cannot be.**

* The raw records carry each traced child's integer pid as `trace.child_pid`, set in
  `observations.py` line 878.
* The frozen boundary withholds that key (`INTERNAL_ONLY_KEYS`). All eight traced cases' published
  records carry `<WITHHELD:internal-observation-input>` there.
* Every other sanitiser rule leaves a visible placeholder, and no other placeholder appears in any
  case record. So those eight withheld pids are the only difference between the published records
  and the digest input.
* The digest input therefore cannot be rebuilt from preserved evidence, and the frozen API offers
  no way to reproduce the digest from published records.

No substitute digest construction was invented or adopted. The recorded value is a well-formed
SHA-256 hex string.

**Consequence.** The digest is a commitment no third party can check. That does not weaken the
result:

* the aggregate, counts, detail and all 72 statuses and reasons were reproduced directly from the
  preserved records (Section 14);
* those records match between two independently written documents, the fsynced journal and the
  published `evidence.json`.

The exact equality this review was asked to require is still **not achieved**, so the verdict is
not SOUND.

**`TRIAL3_AGGREGATE_INPUT_DIGEST_DEFECT`**, scoped to *not independently recomputable by
construction*. It is recorded as **R3-M1 (MINOR)** and does not bear on the result: Section 14 is
the load-bearing reproduction, and no preserved value contradicts it.

## 16. Preservation provenance

[`PROVENANCE.md`](../experiments/evidence/LAUNCH-EXEC-01-TRIAL-003-2026-09-14/PROVENANCE.md) was
checked line by line against the primary sources above.

| Claim | Verified against | Result |
|---|---|---|
| "This file is NOT Trial #3 evidence", written after the run | the file | accurate and prominent (line 3) |
| Run, job, workflow id/name/path, event, head branch/SHA, `run_number`, `run_attempt`, conclusion (infrastructure only) | Actions API | accurate |
| Run start 18:51:04Z; job 18:51:08Z / 18:52:43Z; trial step 18:51:15Z / 18:52:40Z; runner image `20260907.300.1` | API steps, job log | accurate |
| Freeze, manifest blob and SHA-256, freeze review, Build 8 evidence, dispatcher review, D-7 record, dispatcher blob and SHA-256 at the head | Git objects, D-7 record | accurate |
| Runner exit code 0 | job log | accurate |
| Artifact name, id, size, ZIP SHA-256, created/expires | API, two downloads | accurate |
| ZIP is transport only; the five extracted files are the sanctioned evidence | commit contents | accurate: no ZIP committed |
| File byte counts and SHA-256; evidence equals stdout | recomputed | accurate |
| `preflight.json` equals Trial #2's, blob `9759a116…`; no trial/run id or timestamp; not a stale copy | Section 9 | accurate |
| UTF-8, LF, trailing newline; `.gitattributes` byte-exact pin | Section 7 | accurate |
| Verification list: only run, one attempt, one job, one artifact; `501a7fa` blob and SHA-256 and single-path change; every pre-trial gate; "all eleven static checks" | API, Git, job log | accurate: the membership step prints exactly eleven `ok` lines |
| Authority commits reachable from `origin/docs/helm-launch-architecture` | `git merge-base --is-ancestor` | accurate for `bebd8a5`, `69f13a7`, `5a6be59`, `f197386`, `4f19897`, `0c5652d` |
| Job log 55101 bytes, SHA-256 `56e23b35…`, not committed | two fresh downloads | accurate |
| Sanitisation performed by the frozen P-14 boundary, nothing altered at preservation | Sections 7 and 8 | accurate |
| No X2c diagnosis; no historical evidence modified | the file, Section 2 | accurate |

`PROVENANCE.md` does not transcribe D-7 consumption or the frozen result. It deliberately "carries
no observation, no case status and no verdict of its own", like the Trial #2 `PROVENANCE.md`. That
is an omission, not a misstatement (R3-M2). This review and `PROJECT_STATE.md` carry both facts.

**`TRIAL3_PRESERVATION_PROVENANCE_SOUND`**

## 17. Result versus postmortem

**What this review states:**

* X2c is the sole frozen FAIL.
* Its observed outcome is `ExecStatusIndeterminate`.
* Its frozen expected rule is `interpreter_ran_with_devfd`.
* Its frozen checker reason and record reason are exactly as quoted in Section 13.
* X2c alone mechanically causes `MECHANISM_REJECTED`.

**What it does not state:** whether X2c reflects a launcher mechanism defect, an expectation
defect, an observability defect, a kernel behaviour, a script-fixture defect or a harness defect.
Those are postmortem questions. Trial #3's frozen statuses are immutable whatever a postmortem later
finds. A later finding that an expectation or harness was wrong does not turn X2c into a PASS, and
any future experiment needs a new freeze and a new trial number.

### POSTMORTEM QUESTIONS — NOT RESULT FINDINGS

These are questions only. None is answered here, and none is implied to have a particular answer.

1. Which disposition and exec-confirmation state did the launcher itself record for X2c? The
   published record carries only the rendered token and the observation reason.
2. Did the X2c interpreter process start at all, and if it did, what did it write to the stream the
   report travels on?
3. Is `interpreter_ran_with_devfd` reachable under X2c's frozen fixture, spike flags
   (`--bypass-admission`, `--exec-fd-no-cloexec`), declared channels (`receipt`, `report`) and the
   frozen exec-evidence rule?
4. How do the Trial #3 PASS observations of the sibling cases X2, X2b, X3 and X4 relate to X2c's
   path, if at all?
5. What does X2c's preserved `elapsed_ms: 1` constrain, and what does it not?
6. Which of these can a bounded, non-posing, offline diagnostic answer without executing the
   launcher or posing a case?

## Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| — | — | — | — | **No BLOCKER and no IMPORTANT.** Execution identity, dispatcher bytes, gates, artifact identity, preservation, sanitisation, journal, D-7 consumption, completion, case statuses and the frozen replay are all sound. | — |
| R3-M1 | MINOR | aggregate input digest | yes, in every trial under this freeze | `aggregate_input_digest` hashes the pre-sanitisation records. The eight withheld `trace.child_pid` values make it impossible to recompute from preserved evidence, so it cannot be verified independently. | Recorded. The result is unaffected (Section 14). No Trial #3 change is possible. A future freeze could commit to the published records instead; that is an owner/backlog question. |
| R3-M2 | MINOR | provenance | n/a | `PROVENANCE.md` does not transcribe D-7 consumption or the frozen result. It is silent on them, not wrong. | No change to the preserved commit. Carried by this review and `PROJECT_STATE.md`. |
| R3-B1 | BACKLOG_NONBLOCKING | publication sanitiser | yes | The frozen sanitiser over-redacts public text: 20 manifest paths (including the whole `vocabulary.acquisition` object) and the `os-release` URLs. The published `freeze` object is therefore not byte-identical to `SOURCE-HASHES.json`. It changes no semantics and exposes nothing. | None for Trial #3. Consider for future sanitiser design. |
| R3-B2 | BACKLOG_NONBLOCKING | frozen journal wording | no | `journal.py` still calls `case_pose_started` "the Trial #2 immutability boundary", calls `project_status` "the frozen Trial #2 reading" and defaults `trial="trial-002"`. The runner passes `trial-003`, and all 219 records carry it. | None for Trial #3. Wording only. |

The fact that X2c FAILed is the trial result. It is not a review defect. The 70 PASS cases were not
reopened, and no preserved value contradicts any of them.

## Next gate

**TRIAL #3 RESULT IS CONFIRMED; OWNER MAY PROCEED TO ONE BOUNDED X2c POSTMORTEM**

No BLOCKER or IMPORTANT finding exists, and every load-bearing verdict is sound. The aggregate input
digest (R3-M1) is recorded as a non-load-bearing MINOR defect. If the owner holds that the digest's
independent recomputation is required for confirmation, this review instead needs an owner decision.

> **TRIAL #3 D-7 IS CONSUMED.**
> **LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ONE.**
> **TRIAL #3 MUST NOT BE RERUN.**
> **X2c IS THE SOLE FROZEN FAIL. THIS RESULT REVIEW DOES NOT DIAGNOSE WHY X2c FAILED.**
> **NO TRIAL #4 IS AUTHORISED.**

## Classification

**`TRIAL_3_RESULT_REVIEW_PASSED_READY_FOR_BOUNDED_X2C_POSTMORTEM`**

## Validation

| Check | Result |
|---|---|
| `git diff --check` | clean |
| `python tools/validate_docs.py` | PASS |
| `python -m unittest discover -s tools/tests` (Windows) | 783 tests OK, 74 skipped |
| `python3 -m unittest discover -s tools/tests` (WSL Ubuntu, kernel `6.6.87.2-microsoft-standard-WSL2`, Python 3.12.3) | 783 tests OK, 1 skipped. No C compiler exists in that environment, so the scratch compile-inspection test cannot compile, and nothing was compiled. |
| `cargo fmt --check` | pass |
| `run_launch_exec_01.py --verify-freeze` | `freeze_verified: true` |
| Frozen offline replay (Section 14) | exact match |
| New Trial #3 runs / reruns / retries | **0 / 0 / 0** |
| Cases posed / experimental ELF executions / experiment compilations | **0 / 0 / 0** |

## Review boundary

This review verified the execution, the dispatcher, the artifact, the preservation, the publication
boundary, the journal, D-7 consumption and completion. It reproduced the result offline with the
frozen checker.

It did **not**:

* diagnose X2c;
* re-audit the 70 PASS cases;
* modify any preserved evidence, `.gitattributes`, `DECISIONS.md`, frozen input, dispatcher,
  source, test or workflow;
* dispatch anything;
* push anything.
