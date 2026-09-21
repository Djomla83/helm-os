# LAUNCH-EXEC-01 — Trial #3 one-shot dispatcher independent review

**THIS REVIEWS THE DISPATCHER CANDIDATE.**
**IT DOES NOT AUTHORISE D-7 AND DOES NOT EXECUTE TRIAL #3.**

This is the independent review of the Trial #3 one-shot dispatcher candidate
[`.github/workflows/launch-exec-01-trial-003.yml`](../../.github/workflows/launch-exec-01-trial-003.yml).
The candidate was added by `f197386` on top of the published milestone tip `5a6be59`, and the
reviewer did not write it. The frozen mechanism, cases and runner were closed by the
[Trial #3 freeze review](HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md) and are not reopened. The
Build 8 evidence was accepted by the
[Build 8 evidence-correction re-review](HELM-LAUNCH-EXEC-01-BUILD-008-EVIDENCE-CORRECTION-REREVIEW.md)
and is examined here only as far as the dispatcher binds it.

Nothing was fixed, pushed or dispatched, and `main` was not touched. No GitHub Actions run was
created. The frozen runner ran only as `--verify-freeze` and `--driver-completeness`, which return
before its D-7 branch; it was never given `--i-have-owner-authorisation-d7`. No build directory
was created, no `launcher_spike` or helper ELF existed or ran, and no case was posed. The trial,
staging and report steps were exercised only with a stand-in `run_launch_exec_01.py` in an empty
directory.

**Classification: `TRIAL_3_DISPATCHER_REVIEW_PASSED_READY_FOR_MILESTONE_PUBLICATION`.**

Every load-bearing verdict is SOUND, and there is no BLOCKER or IMPORTANT finding. The one MINOR
finding, DSP-M1, is carried as operator preconditions in section 29. Five backlog notes are
recorded in section 31.

## Verdicts

| Area | Verdict |
|---|---|
| Scope | **TRIAL3_DISPATCHER_SCOPE_SOUND** |
| Byte identity | **TRIAL3_DISPATCHER_BYTE_IDENTITY_SOUND** |
| YAML / actions | **TRIAL3_DISPATCHER_YAML_STRUCTURE_SOUND** |
| One-shot guards | **TRIAL3_ONE_SHOT_GUARDS_SOUND** |
| Branch guard | **TRIAL3_MILESTONE_PUBLICATION_CANNOT_CROSS_EXECUTION_GUARD** |
| Freeze checkout | **TRIAL3_LITERAL_FREEZE_BINDING_SOUND** |
| Freeze review | **TRIAL3_FREEZE_REVIEW_BINDING_SOUND** |
| Build 8 | **TRIAL3_BUILD8_AUTHORITY_BINDING_SOUND** |
| Frozen bytes | **TRIAL3_FROZEN_BYTES_STABLE_THROUGH_AUTHORITY_TIP** |
| Source freeze | **TRIAL3_DISPATCHER_SOURCE_FREEZE_GATE_SOUND** |
| Definitions | **TRIAL3_DISPATCHER_EXPLICITLY_VERIFIES_3_OF_3_DEFINITION_HASHES** |
| Static gates | **TRIAL3_DISPATCHER_STATIC_GATES_SOUND** |
| Pre-boundary | **TRIAL3_PREBOUNDARY_DISCIPLINE_SOUND** |
| D-7 flag | **TRIAL3_DISPATCHER_D7_FLAG_PRESENT_BUT_UNAUTHORISED_SOUND** |
| Invocations | **TRIAL3_EXACTLY_ONE_TRIAL_RUNNER_INVOCATION** |
| Boundary | **TRIAL3_D7_BOUNDARY_DESCRIPTION_SOUND** |
| Preservation | **TRIAL3_EVIDENCE_PRESERVATION_SOUND** |
| Streams | **TRIAL3_STREAM_PUBLICATION_BOUNDARY_SOUND** |
| Exit code | **TRIAL3_RUNNER_EXIT_PROPAGATION_SOUND** |
| Timeout bound | **TRIAL3_DISPATCHER_TIMEOUT_BOUND_SOUND** |
| Hard timeout | **TRIAL3_HARD_TIMEOUT_PRESERVATION_MODEL_SOUND** |
| Artefact privacy | **TRIAL3_DISPATCHER_ARTIFACT_PRIVACY_SOUND** |
| Trial #2 isolation | **TRIAL2_TRIAL3_DISPATCHER_ISOLATION_SOUND** |
| Zero runs | **TRIAL3_DISPATCHER_ZERO_RUN_STATE_SOUND** |
| Negative controls | **TRIAL3_DISPATCHER_NEGATIVE_CONTROLS_SOUND** |
| Current D-7 | **TRIAL3_NO_CURRENT_D7_SOUND** |
| D-7 packet | **TRIAL3_PROPOSED_D7_PACKET_EXACT** |
| Publication plan | **TRIAL3_DISPATCHER_PUBLICATION_PLAN_SOUND** |

## 1. Target and starting state

| Item | Value |
|---|---|
| Dispatcher candidate | `f1973867a709735b7c7e967f7f4b320774de2aa2` |
| Parent / published milestone tip | `5a6be5959c5a9131afa1154c015369df46a5deb8` |
| Workflow path / name | `.github/workflows/launch-exec-01-trial-003.yml` / `launch-exec-01 trial-003 one-shot trial` |
| Workflow Git blob | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` |
| Workflow SHA-256 | `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8` |
| Trial #3 freeze | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| Manifest Git blob / SHA-256 | `8cd290b573408510f8c16cd8dafe676354140a38` / `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` |
| Independent freeze review | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| Formal Build 8 | run `34754901079`, job `103717520698`, attempt 1, `BUILD_8_FORMAL_COMPILE_ONLY_SUCCESS` |
| Build 8 durable job-log SHA-256 | `6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15` |
| Accepted Build 8 evidence | `5a6be5959c5a9131afa1154c015369df46a5deb8` |
| This review | the local commit that adds only this record, a child of `f197386` |
| Pushed | NO |

The starting state was checked before anything else:

* `git status --short` was empty, the branch was `docs/helm-launch-architecture` and `HEAD` was
  `f1973867…`.
* After a read-only `git fetch origin`, `origin/docs/helm-launch-architecture` was `5a6be595…`.
  `origin/main` and local `main` were both `d8a6887d508be3b4bb5707c40e6433cee21411b5`.
* `git rev-list --count origin/docs/helm-launch-architecture..HEAD` was 1, and the reverse count
  was 0. The candidate is exactly one local commit on the published tip.

## 2. Scope

`git show --stat --summary`, `git show --name-status` and `git diff 5a6be595..f197386` agree. The
candidate makes one change: `A .github/workflows/launch-exec-01-trial-003.yml`, 481 insertions.

It changes no source, test, manifest, definition, `BUILD-EVIDENCE.md`, `PROJECT_STATE.md`,
`DECISIONS.md` or Trial #2 workflow, and `main` is untouched. The Trial #2 dispatcher blob is
`1d030130f255ebb74ace047d04590d6aee2838df` at `5a6be59`, at `f197386` and on `main`.

**`TRIAL3_DISPATCHER_SCOPE_SOUND`.**

## 3. Workflow byte identity

Every identity was computed from the committed object, not from the working tree:

* `git rev-parse f197386:.github/workflows/launch-exec-01-trial-003.yml` is `64ce3d43…`.
* The blob's bytes, written out by `git show`, have SHA-256 `9158e293…`. `git hash-object` of
  those bytes returns `64ce3d43…` again, so the round trip is exact. The working-tree file has the
  same SHA-256.
* The file is 25,486 bytes and 481 lines. It has no CR byte and no tab, and it ends with a
  newline. Its only non-ASCII bytes are three U+2014 em dashes, in comment line 1 and in the names
  at lines 162 and 408.

This blob, at this path, is the object a later owner D-7 decision may bind.

**`TRIAL3_DISPATCHER_BYTE_IDENTITY_SOUND`.**

## 4. YAML and action structure

The exact committed bytes were parsed with PyYAML 6.0.3. Under YAML 1.1 the `on` key reads as
`True`. `actionlint` is not installed on this host, and it was not installed for this review.

| Requirement | Parsed value | Lines |
|---|---|---|
| Name | `launch-exec-01 trial-003 one-shot trial` | 50 |
| Triggers | `workflow_dispatch` only, one input `confirmation` (`type: string`, `required: true`) | 54–60 |
| Permissions | `contents: read` | 62–63 |
| Concurrency | group `launch-exec-01-trial-003`, `cancel-in-progress: false` | 66–68 |
| Job | `trial-003` on `ubuntu-24.04`, `timeout-minutes: 350` | 91–111 |
| Trial step | `id: trial`, `timeout-minutes: 330` | 408–411 |
| Shell | `defaults.run.shell: bash` | 86–88 |

* The job has only the keys `runs-on`, `timeout-minutes` and `steps`. There is no `permissions`
  override, `environment`, `container`, `services`, `strategy`, `needs`, `if` or
  `continue-on-error`. No step references `secrets.*`, `github.token` or `GITHUB_TOKEN`.
* **Actions.** There are exactly two `uses:`, and both are 40-hex commit pins, identical to the
  Trial #2 dispatcher's:
  * `actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09` is tagged `v5.1.0` and `v5`
    (GitHub API);
  * `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02` is tagged `v4.6.2` and
    `v4`.

  No mutable tag is used. The `# v5` and `# v4` trailers are whitespace-preceded comments, as
  intended.
* **The `#` truncation defect is closed.** Every step name that contains `#` is single-quoted:
  lines 155, 408, 429 and 453. The parsed names are complete: `Check out the Trial #3 freeze by
  literal SHA`, `TRIAL #3 — the frozen runner, exactly once`, `Stage the sanitised Trial #3
  evidence` and `Preserve the sanitised Trial #3 evidence`. The unquoted names contain no ` #`
  sequence.
* **Run bodies.** The nine `run:` bodies were extracted from the parsed document.
  * `bash -n` returned 0 for each under GNU bash 5.2.37.
  * Bodies 0, 2–6, 8 and 10 were also executed under GNU bash 5.2.21 on Linux (section 26).
    Body 7 ran only with the stand-in runner.
  * No body contains `${{`. Expressions appear only in `env:` (lines 118–122 and 471), `with:`
    (line 458) and `if:` (lines 430, 454 and 469).

**`TRIAL3_DISPATCHER_YAML_STRUCTURE_SOUND`.**

## 5. Trigger and one-shot guards

The guard is the first step (lines 116–149), and nothing is checked out before it.

* **Checks.** It requires all five: event `workflow_dispatch`, ref `refs/heads/main`,
  `run_number` 1, `run_attempt` 1 and confirmation `RUN-TRIAL-003-ONE-VALID-TRIAL`. Each failure
  is recorded, and one `exit 1` follows before checkout. It prints that nothing was checked out,
  built or posed and that no D-7 was consumed.
* **Input handling.** The confirmation and every context enter the shell through `env:` only.
  Each comparison quotes both operands, and the confirmation is never echoed.
* **What a failure leaves.** The job fails at step 0. The trial, staging, upload and report steps
  are skipped, so there is no checkout, build, case, trial result or D-7 consumption.
* **One shot.** A rejected first dispatch still takes `run_number` 1, so it spends this
  dispatcher. There is no second-dispatch path, because `run_number` must be 1. There is no
  rerun path, because `run_attempt` must be 1. There is no retry, resume or fallback: no loop
  around the runner, no `continue-on-error` and no `workflow_run` or `workflow_call`.

The extracted body was simulated in an empty directory on Linux, under
`bash --noprofile --norc -eo pipefail`, with a canary file path:

| Inputs | Result |
|---|---|
| all five correct | rc 0, `one-shot guards passed` |
| confirmation empty, lower-case, with a trailing space, or the Trial #2 token | rc 1, one HALT |
| confirmation `$(touch …)`, backtick `touch`, `x"; touch …; echo "`, the token followed by `$(touch …)`, `' \|\| touch … #` | rc 1, one HALT, no canary |
| confirmation `-n`, `=`, `!`, `::error::x` | rc 1, one HALT |
| ref `refs/heads/docs/helm-launch-architecture`, `refs/tags/main`, `refs/heads/main2` | rc 1, one HALT |
| `run_number` 2, `01` or `0` | rc 1, one HALT |
| `run_attempt` 2 | rc 1, one HALT |
| event `push` or `repository_dispatch` | rc 1, one HALT |
| all wrong | rc 1, five HALT lines |

No canary file was ever created.

**`TRIAL3_ONE_SHOT_GUARDS_SOUND`.**

## 6. Main-only guard and GitHub run numbering

**The milestone branch cannot pass.** The guard compares `github.ref` with the literal
`refs/heads/main`. A dispatch that selects `docs/helm-launch-architecture` has ref
`refs/heads/docs/helm-launch-architecture` and halts at step 0 (section 5).

**Milestone publication alone is not dispatchable.** Today
`gh run list --workflow launch-exec-01-trial-003.yml` answers
`HTTP 404: workflow launch-exec-01-trial-003.yml not found on the default branch`. The file carries
no event that fires on push. After the file reaches `main`, a dispatch may still name any ref that
carries the file, including the milestone branch. The guard rejects that dispatch, and it spends
run 1.

**Run-numbering assumptions, and what settles each:**

1. **Numbering is per workflow file, not global.**
   * Trial #1's `.github/workflows/launch-exec-01-trial.yml` (workflow `355035475`, now state
     `deleted`) had its own run 1, run `34500901306`.
   * Trial #2's new path (workflow `356056797`) started again at run 1, run `34640280964`,
     attempt 1.
   * Over the same five milestone pushes (2026-09-10 to 2026-09-13), `launch-exec-01-compile-only`
     numbered its runs 7–11 while `helm-evidence` numbered its runs 27–31.
2. **Numbering counts runs on every ref.** `helm-evidence` keeps one counter across `main`,
   `product/*`, `review/*` and the milestone branch: run 23 was a push to `main` and run 24 a push
   to the milestone. `helm-bind` numbered runs 1–7 on other branches and run 8 on `main`. Any run
   of this path on any ref therefore takes run 1. That includes a mistaken milestone dispatch, a pending run
   later cancelled by concurrency, and a failed run for a workflow file GitHub cannot parse. The
   dispatch on `main` is then run ≥ 2 and halts. This fails closed: it can spend the dispatcher,
   never grant a second trial. The file parses (section 4), and no push-triggered run is expected.
   The checklist still verifies zero runs after every push (section 29).
3. **A re-run keeps its `run_number` and increments `run_attempt`.** The attempt guard rejects
   it.
4. **A copy or rename to another path gets a fresh counter at 1.** No repository evidence
   contradicts this, and the guards do not bind `github.workflow_ref`. This is DSP-M1.
5. **Deleting and re-adding the same path.** Whether that resets the counter is not settled by
   repository evidence. Trial #1's workflow id survives in state `deleted`, but no re-add was
   observed. The plan never deletes the file before dispatch, so nothing relies on either answer.

**`TRIAL3_MILESTONE_PUBLICATION_CANNOT_CROSS_EXECUTION_GUARD`.**

## 7. Literal freeze checkout

* **Checkout** (lines 155–160). `ref:` is the literal `bebd8a5f83d4d0daebe9b068050cb5436289c75e`,
  never `github.sha`, an input, a branch or a tag. `fetch-depth: 0` and
  `persist-credentials: false` are set. `env.FREEZE_SHA` repeats the same literal. If the two ever
  diverged, the `HEAD` gate would halt.
* **Gates** (lines 171–184), in order and before any runner call:
  * `HEAD` equals `FREEZE_SHA`;
  * `git status --porcelain` is empty;
  * the manifest's SHA-256 equals `ea482c6f…`;
  * its file blob (`git hash-object`) and its tree blob (`git rev-parse HEAD:<manifest>`) both
    equal `8cd290b5…`.
* **Local check.** A fresh LF-preserving clone at the freeze has an empty porcelain, manifest
  SHA-256 `ea482c6f…` and both blobs `8cd290b5…`. The first scratch clone on the Windows host
  inherited `core.autocrlf=true`, so it was discarded before any hash was taken. A Linux clone
  in WSL (no `autocrlf`) reproduced the same values through the real gate body.
  `.gitattributes` pins `*.py`, `*.json` and `*.md` to `eol=lf`.
* **Negative controls** N1, N2 and N29–N32 each halt at this step (section 26).

**`TRIAL3_LITERAL_FREEZE_BINDING_SOUND`.**

## 8. Freeze-review binding

Lines 186–195 bind the freeze review from Git objects only:

* `git cat-file -e 69f13a7…^{commit}`;
* `git merge-base --is-ancestor <freeze> <freeze review>`;
* `git show 69f13a7…:docs/implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md`, written
  to a file;
* `grep -qF` of the full line form
  `Classification: **TRIAL_3_FREEZE_REVIEW_PASSED_READY_FOR_PUBLICATION_AND_BUILD_8**`.

No current-branch prose is read.

Independently, `69f13a7` is a commit, `bebd8a5` is its ancestor, and the record exists there with
728 lines. The bare token `TRIAL_3_FREEZE_REVIEW_PASSED_READY_FOR_PUBLICATION_AND_BUILD_8` occurs
**once** in the whole record, on line 728, the final classification line after the section-24
readiness statement. So no negative or contextual mention exists for the gate to match.

`grep -F` is a substring match, so a line that merely contained the full form would also match. The
bound commit is a literal SHA, and its record is content-addressed and was read here, so that case
is unreachable. The gate is stricter than Trial #2's bare-token grep. Control N6 is a record whose
classification is `NEEDS_FIX` and which mentions the accepted token only in a negative sentence;
it halts.

**`TRIAL3_FREEZE_REVIEW_BINDING_SOUND`.**

## 9. Build 8 authority binding

Lines 197–229 require:

* `5a6be59` to be a commit descending from the freeze review;
* the re-review record and `BUILD-EVIDENCE.md` to exist at that commit;
* nine exact strings.

No GitHub API call is made, and no raw log is relied on. Each string was counted in the bound
objects:

| Record at `5a6be59` | Required string | Occurrences | Context |
|---|---|---|---|
| re-review | `Classification: **BUILD_8_EVIDENCE_CORRECTION_REREVIEW_PASSED_READY_FOR_TRIAL3_AUTHORITY_DECISIONS**` | 1 (line 314) | final classification; the bare token also occurs only there |
| re-review | `**BUILD 8 EVIDENCE IS ACCEPTED FOR TRIAL #3 AUTHORITY DECISIONS.**` | 1 (line 286) | section 15, acceptance decision |
| re-review | ``run `34754901079`, job `103717520698`, run number 11, attempt 1`` | 1 (line 14) | header row "Formal Build 8" |
| re-review | `` `6c16e23e…0a15` `` | 2 (lines 88, 281) | BR-I1 fresh job log; acceptance decision |
| evidence | ``\| Formal run \| `34754901079`: run number 11, attempt 1,`` | 1 (line 760) | correction section, which governs |
| evidence | ``\| Formal job \| `103717520698` \|`` | 1 (line 761) | correction section |
| evidence | `\| Result \| **BUILD_8_FORMAL_COMPILE_ONLY_SUCCESS**` | 1 (line 764) | "unchanged … the one and only formal Build 8 attempt" |
| evidence | `` `6c16e23e…0a15` `` | 2 (lines 728, 771) | log identity table; durable log identity |
| evidence | ``head `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` `` | 1 (line 618) | Build 8 workflow-run row |

Neither record contains `NOT ACCEPTED`, `not accepted`, `NEEDS_FIX` or
`BUILD_8_FORMAL_COMPILE_ONLY_FAIL`. Together the gates establish the Build 8 facts from immutable
objects: the run, job, attempt 1, the formal result, the durable job-log SHA-256 and workflow head
`69f13a7`. The same substring caveat as section 8 applies, and it is unreachable for the same
reason.

Controls N7–N21 each change one binding and halt at this step: a wrong commit, a non-descendant
copy of the records, a non-accepting sentence, a wrong classification, attempt, job, result,
log SHA-256 or head.

**`TRIAL3_BUILD8_AUTHORITY_BINDING_SOUND`.**

## 10. Frozen-byte identity through the authority tip

**What the step does** (lines 234–283). It derives the closed sets from the checked-out manifest,
which step 2 already bound:

* the `sha256` keys, prefixed with the experiment directory, give 17 sources;
* the `definition_sha256` keys give 3 definitions.

It halts unless both counts are exact. It requires the manifest blob at both authority commits to
be `8cd290b5…`. For every path, it requires a blob at the freeze and the same blob at `69f13a7`
and at `5a6be59`. A path missing at an authority reads as `None` and is drift. There is no
directory diff.

**Independent recomputation from Git objects.** The manifest blob is `8cd290b5…` at the freeze,
at `69f13a7`, at `5a6be59` and at the candidate. For each of the 20 paths below, the blob is the
same at all four, and the SHA-256 of the freeze blob equals the manifest value:

| Kind | Path | Blob |
|---|---|---|
| source | `docs/experiments/launch-exec-01/README.md` | `a9f80c1d4661…` |
| source | `docs/experiments/launch-exec-01/checker.py` | `b73f7df7a672…` |
| source | `docs/experiments/launch-exec-01/driver.py` | `ebeb948b7f8d…` |
| source | `docs/experiments/launch-exec-01/evidence.py` | `51e8ad667176…` |
| source | `docs/experiments/launch-exec-01/frozen_cases.py` | `0ffda921a30f…` |
| source | `docs/experiments/launch-exec-01/harness.py` | `df12cad176ac…` |
| source | `docs/experiments/launch-exec-01/helper_alt.c` | `76515904eb7e…` |
| source | `docs/experiments/launch-exec-01/helper_dynamic.c` | `db25b86d30f1…` |
| source | `docs/experiments/launch-exec-01/helper_fork.c` | `a8cf5d9777dc…` |
| source | `docs/experiments/launch-exec-01/helper_report.c` | `0f60efae4364…` |
| source | `docs/experiments/launch-exec-01/helper_setid.c` | `c12261e4444a…` |
| source | `docs/experiments/launch-exec-01/journal.py` | `d58e223767d1…` |
| source | `docs/experiments/launch-exec-01/launcher_spike.c` | `3b9c573ca06c…` |
| source | `docs/experiments/launch-exec-01/make_fixtures.py` | `f4c3fa5030ee…` |
| source | `docs/experiments/launch-exec-01/observations.py` | `ee846555fdbf…` |
| source | `docs/experiments/launch-exec-01/oracles.py` | `898f62181239…` |
| source | `docs/experiments/launch-exec-01/run_launch_exec_01.py` | `03cbabc1dea2…` |
| definition | `docs/adr/ADR-0024-launch-authority.md` | `5dbb8c8ff69f…` |
| definition | `docs/experiments/LAUNCH-EXEC-01-DEFINITION.md` | `439b316f3b0c…` |
| definition | `docs/research/HELM-LAUNCH-ARCHITECTURE.md` | `3bc5b9e12ce1…` |

**Unhashed files.** At `5a6be59` the experiment directory holds exactly three unhashed files:
`BUILD-EVIDENCE.md`, `SOURCE-HASHES.json` and `TRIAL-3-CORRECTION-CANDIDATE.json`. That is the
manifest's `not_hashed_here` list. `git diff --name-status bebd8a5 5a6be595` lists only a
modification of `BUILD-EVIDENCE.md` and three added review records.

**No extra hashed path.** The set comes from the frozen manifest, and that manifest's blob must be
identical at both authorities. An authority commit therefore cannot add a hashed path. An
unhashed file cannot affect execution, because the runner runs the literal freeze.

**Controls.**

* N22–N27 each create drift at the acceptance commit and halt at this step: a Python source, a C
  source, a definition, the manifest, a missing source and a missing definition.
* N28 creates drift at the freeze-review commit and halts.
* P1 appends to `BUILD-EVIDENCE.md` at an authority commit and passes. The step does not
  over-reject the legitimate append-only record.

**`TRIAL3_FROZEN_BYTES_STABLE_THROUGH_AUTHORITY_TIP`.**

## 11. 17/17 source-freeze verification

Lines 285–299 run `python3 run_launch_exec_01.py --verify-freeze` in the experiment directory on
the literal checkout. The output goes to a file, so under `-e -o pipefail` the runner's own
return code 1 already fails the step. A second check requires `freeze_verified` to be exactly
`True` before anything later runs.

The frozen `verify_freeze()` (`run_launch_exec_01.py:70–87`) hashes only the manifest's `sha256`
table. That matches the manifest's `verify_freeze.checks`: "the 17 source hashes in sha256, and
nothing else". The step comment says precisely that and adds that it does not check
`definition_sha256`.

* **Positive runs.** On the LF clone the gate printed `"freeze_verified": true` with rc 0, under
  Python 3.14.3 on Windows and Python 3.12.3 on Linux.
* **Controls.** N31b (a drifted source) and N39 (a wrong source hash in the manifest) each fail
  the step.

**`TRIAL3_DISPATCHER_SOURCE_FREEZE_GATE_SOUND`.**

## 12. 3/3 definition-hash verification

Lines 301–349 close the Trial #2 dispatcher's gap:

| Requirement | Implementation | Control |
|---|---|---|
| Authority comes from `SOURCE-HASHES.json` | reads `definition_sha256` from the manifest bound in step 2 (lines 319–322) | — |
| The table is a mapping | lines 323–324 | N38 (a list) halts |
| Exactly 3 entries | lines 325–326 | N34b (4 entries) halts |
| Closed path set, nothing missing, nothing extra | literal `EXPECTED_PATHS` (lines 310–314) compared both ways (lines 327–331) | N35 (a swapped path, still 3 entries) halts |
| Each expected digest is 64 lower-case hex | `re.fullmatch(r"[0-9a-f]{64}", …)` (lines 335–336) | N37 (upper-case digest) halts |
| Each file exists and is readable | lines 337–341 | N33 (missing file) halts |
| Committed SHA-256 equals the manifest value | lines 339–347 | N32b (changed byte) and N36 (wrong manifest hash) halt |

The step does not hash three hard-coded files: the digests come from the manifest, and the literal
set only proves the manifest's table is the closed one. The positive run printed:

* `ok docs/adr/ADR-0024-launch-authority.md c1f3cce8…`;
* `ok docs/experiments/LAUNCH-EXEC-01-DEFINITION.md 0c7406b4…`;
* `ok docs/research/HELM-LAUNCH-ARCHITECTURE.md 97fbf0d8…`;
* `definition hashes verified: 3/3`.

Step 3 also proves the same three blobs unchanged at both authority commits. No case was posed by
any control.

**`TRIAL3_DISPATCHER_EXPLICITLY_VERIFIES_3_OF_3_DEFINITION_HASHES`.**

## 13. Static completeness gates

Lines 351–389 run `--driver-completeness` and evaluate its report together with
`frozen_cases.summary()`.

**Nothing executes.**

* `driver.completeness()` and `driver.unposable_cases()` walk the plan table as data. They compare
  names against `SETUPS`, `PARENT_STATES`, `observations.RULES`, `POSED_CHECKS` and
  `ALL_CHANNELS`, and they call none of those functions.
* `summary()` validates and counts frozen tables.
* No `Authorisation` exists on this path; `prepare()` and `pose()` both require one.

| Check | Frozen value |
|---|---|
| total / mandatory / conditional / recorded | 72 / 54 / 11 / 7 |
| traced, exact | `E1 E7 F4 F7 M1 M2 M3 M4` |
| `complete`, `frozen_total`, `driver_total` | `true`, 72, 72 |
| `missing`, `unknown`, `duplicates` | all empty |
| unresolved setups, parents, rules, checks; unknown channels | all empty |
| `unposable` | `{}`, so 72 statically posable and 0 unposable |

The positive run printed all eleven `ok` lines, under Python 3.14.3 and Python 3.12.3.

The evaluator was then run on doctored reports, each of which halted with rc 1: `complete: false`,
a non-empty `unposable`, `missing: [E1]`, `driver_total: 71`, a duplicate, an unresolved rule and
an unknown channel. A report with a key removed fails with `KeyError`, rc 1, which also fails
closed.

**`TRIAL3_DISPATCHER_STATIC_GATES_SOUND`.**

## 14. No pre-boundary execution

| Step before the trial step | What runs |
|---|---|
| 0 guards | shell comparisons only |
| 1 checkout | `actions/checkout` at the literal freeze |
| 2 identity | `git`, `sha256sum`, `grep -F` |
| 3 frozen bytes | Python calling `git rev-parse` |
| 4 source freeze | frozen runner `--verify-freeze`: import and hashing |
| 5 definitions | Python hashing |
| 6 static | frozen runner `--driver-completeness` plus `import frozen_cases`: a static walk |

* **No ELF exists before the trial step.** Every one of the freeze's 535 tree entries is mode
  `100644`, with no `target/` and no ELF, and no step before the trial step compiles anything.
* **Imports execute nothing.** The frozen module bodies hold definitions, constants and pure table
  construction (`_PLAN_LIST = _build_plans()`). A scan found no other top-level call.
* **Only the D-7 invocation can pose.** `main()` returns from `--verify-freeze` and
  `--driver-completeness` before its D-7 branch, and constructs `Authorisation` only after the
  flag (`run_launch_exec_01.py:350–363`, `:373–395`).

Inside the one D-7 invocation, and still before the first `case_pose_started`, the frozen runner:

* re-verifies the freeze;
* runs host queries (`uname`, `strace --version`, `gcc --version`, `ldd --version`, `getconf`);
* compiles a static probe and inspects it with `ldd`, a probe that is not a LAUNCH-EXEC helper or
  launcher;
* builds, hashes and re-verifies the build identity;
* calls `prepare()` for the first case.

None of these invokes `launcher_spike`.

**`TRIAL3_PREBOUNDARY_DISCIPLINE_SOUND`.**

## 15. D-7 flag and current authority

Lines 1–13 open the file with a capitalised statement: the candidate "HAS NO EXECUTION AUTHORITY
UNTIL THE OWNER LATER AUTHORIZES D-7 FOR THIS EXACT REVIEWED WORKFLOW BLOB AND IT IS PUBLISHED
BYTE-IDENTICALLY TO MAIN". The same header says Trial #3 D-7 is not authorised, Trial #3 is NOT_RUN
with a valid count of zero, and the flag "is not authority". It also gives the required order.

`--i-have-owner-authorisation-d7` occurs twice: in that comment (line 9) and in the one runner
invocation (line 414). It is absent from the guard, the gates, the verifier and completeness
calls, staging and reporting.

The flag's presence is acceptable for three reasons:

* The candidate is local and unpublished, and GitHub has no such workflow (section 25).
* It cannot run until it is on `main`.
* The plan publishes it to `main` only after an owner D-7 naming this blob (section 29).

**`TRIAL3_DISPATCHER_D7_FLAG_PRESENT_BUT_UNAUTHORISED_SOUND`.**

## 16. Exactly one trial runner invocation

`run_launch_exec_01.py` appears in three commands:

* line 291, `--verify-freeze`;
* line 357, `--driver-completeness`;
* line 414, the D-7 invocation: `python3 run_launch_exec_01.py --i-have-owner-authorisation-d7
  --build-dir "$BUILD_DIR" --out-dir "$OUT_DIR"`.

Only line 414 can cross D-7.

* **No repetition.** No shell loop surrounds any runner call. The only shell loop is staging's
  copy-by-name over four file names. There is no retry, resume, second trial, per-case invocation
  or fallback, no second job and no matrix.
* **Stand-in argv.** The stand-in received exactly `--i-have-owner-authorisation-d7 --build-dir
  target/launch-exec-01 --out-dir trial-output`. Those are the frozen runner's own defaults, and
  they resolve against the working directory staging uses.

**`TRIAL3_EXACTLY_ONE_TRIAL_RUNNER_INVOCATION`.**

## 17. Boundary semantics

**In the frozen runner** (`run_launch_exec_01.py:264–281`), each case is journaled
`case_entered` and prepared. Only a `ready` case gets `jrnl.case_pose_started(…)` before
`driver.pose(…)`. That record is written, flushed and `fsync`-ed (`journal.py:89–103`, `:134–155`).
The manifest's `durable_evidence.journal` freezes the consumption point, "the FIRST
case_pose_started record is successfully fsynced", and freezes the partial-journal readings.

**In the workflow.** The workflow's text matches:

* lines 40–45: dispatch, checkout, gates, preflight, build and runner startup consume nothing;
  before that record no case status or aggregate exists and the valid count stays zero; after it
  there is no retry, resume or rerun-to-green;
* lines 391–402: "The first such record would consume a Trial #3 D-7."

No line equates dispatch with consumption.

**Before the boundary.**

* `HALT_PREFLIGHT` and `HALT_BUILD_IDENTITY` return 6 with a `trial_end` that carries no
  aggregate.
* A guard or gate failure never starts the runner.

**`TRIAL3_D7_BOUNDARY_DESCRIPTION_SOUND`.**

## 18. Evidence preservation

* **Staging** (lines 429–449) and **upload** (lines 453–460) both run under
  `always() && steps.trial.outcome != 'skipped'`.
* **What is staged.** Staging copies each of `preflight.json`, `build-identity.json`,
  `journal.jsonl` and `evidence.json` by name from `trial-output`, if present. It adds
  `runner-stdout.json` only when non-empty. It ends with `exit 0`.
* **Upload.** The artefact name is `launch-exec-01-trial-003-evidence`, with `retention-days: 90`,
  which is the repository's `maximum_allowed_days` (90). The comment at lines 451–452 says a later
  step must commit the evidence before expiry.
* **After a pre-boundary gate failure** the trial step is skipped. Nothing is staged, and nothing
  exists to stage.

The trial, staging and report bodies were run on Linux with a stand-in runner:

| Stand-in outcome | Trial step | `code` output | Staged | Report rc |
|---|---|---|---|---|
| completed RUN, `MECHANISM_REJECTED`, rc 0 | 0 | 0 | all four files and `runner-stdout.json` | 0 |
| `HALT_PREFLIGHT`, rc 6 | 0 | 6 | `preflight.json`, `runner-stdout.json` | 6 |
| freeze `HALT`, rc 4, no output directory | 0 | 4 | `runner-stdout.json` | 4 |
| harness exception after the boundary, rc 1, empty stdout | 0 | 1 | `preflight.json`, `build-identity.json`, `journal.jsonl` | 1 |
| runner SIGKILL, rc 137 | 0 | 137 | the same three | 137 |
| step stopped by SIGTERM after 3 s | stopped | none | the same three, with both fsynced journal lines | 1 |
| step stopped by SIGINT after 3 s | stopped | none | the same three | 1 |

`MECHANISM_INCONCLUSIVE` takes the completed-RUN path, because the runner returns 0 for every RUN.

**`TRIAL3_EVIDENCE_PRESERVATION_SOUND`.**

## 19. Runner stdout and stderr

**Stdout** is redirected to `$RUNNER_TEMP/runner-stdout.json`. In the frozen experiment the only
writer to stdout is `evidence.publish` (`evidence.py:638–658`), which applies
`Sanitiser.record` and `serialise` first. No `print(` exists in the experiment's Python.

Every subprocess captures its output:

* `harness.py:49`, `:239`, `:245`, `:268` and `:275`;
* `driver.py:3651` and `:3792`, with `PIPE`;
* the tracer, which writes to its own `-o` file.

The staged stdout is therefore exactly one sanitised document, or nothing.

**Stderr** is not redirected, so it reaches only the job log. Staging never reads it. The stand-in
wrote a private marker to stderr, and it appeared in no staged file. The workflow never calls
stderr evidence.

The Trial #2 limitation stays in the backlog as DSP-B4: stderr in a public job log can be
unsanitised. It does not cross the artefact boundary.

**`TRIAL3_STREAM_PUBLICATION_BOUNDARY_SOUND`.**

## 20. Exit code and workflow conclusion

The frozen runner's codes were read from code, not prose (`run_launch_exec_01.py:229`, `:256`,
`:327` and `:350–398`):

| Path | Code |
|---|---|
| completed RUN, any aggregate, including `MECHANISM_REJECTED` and `MECHANISM_INCONCLUSIVE` | 0 |
| `HALT_PREFLIGHT` or `HALT_BUILD_IDENTITY` | 6 |
| freeze `HALT` (drift) | 4 |
| `NOT_RUN` without the flag (not reachable here) | 3 |
| uncaught harness exception | 1, Python's default |
| killed by signal *n* | 128 + *n*, as the shell reports it |

**Trial step** (lines 412–421). It runs `set +e`, then the runner with stdout redirected, then
`code=$?` with no pipe or subshell. It restores `set -e`, echoes the code, appends `code=` to
`GITHUB_OUTPUT` and exits 0, so preservation cannot be skipped.

**Report step** (lines 468–481). It runs after staging and upload, and exits with that code. If
no code was recorded, it prints "stopped, timed out or cancelled" and exits 1. Neither step reads
the aggregate. A completed `MECHANISM_REJECTED` trial is a green job, and the comment at lines
462–467 states the same code table.

The stand-in reproduced each code exactly (section 18).

**`TRIAL3_RUNNER_EXIT_PROPAGATION_SOUND`.**

## 21. Timeout arithmetic

The bound was re-derived from the frozen code by importing the plan table and reading the wait
sites. The author's table was not used as input.

**Per-launch watchdog** (`driver.py:3631`). It is `total_bound_ms + 5 s`, where `total_bound_ms`
is spawn-confirm + timeout + grace + post-exit drain (`oracles.py:101–103`). The defaults are
5,000 + 5,000 + 2,000 + 2,000 ms, so the watchdog is 19 s. The non-default plans are:

* T1, T2 and T6 at 2,000/2,000, giving 16 s;
* T3 at 2,000/5,000, giving 19 s;
* O3 at 60,000/2,000, giving 74 s.

**Launches.** There are 70 single-run plans, plus O8 and S7 at 200 repetitions each, plus M2's
control arm (`driver.py:3458–3480`). That is **471**: 3 at 16 s, 467 at 19 s and 1 at 74 s.

**READY wait.** `POST_PIN_READY_TIMEOUT_S` is 30 (`driver.py:373`, `:3804`). It is spent before
the watchdog, and only when `needs_barrier`. That means a setup declaring `post_pin` or
`hold_writer`, or a parent state holding a key of `BARRIER_PROVEN_STATE_KEYS` (`driver.py:897–908`,
`:1890–1891`, `:2056–2057`). The `none` setup and the `none` parent state declare neither. So the
**242** launches with any setup or parent state are a proven superset. The manifest's 20
`barrier_cases` account for 219 launches (S7's 200 included).

**Post-launch reads.** Each returns immediately when not armed:

* the liveness rendezvous, 3 s (`driver.py:3855`), only for setup `fork_helper`: P1–P4 and T5, 5
  launches;
* the fixture-health read, a 1 s first-byte wait and then buffered bytes only
  (`driver.py:1782–1816`): P1, P2 and P4, 3 launches;
* the fixture signal, 2 s in total (`driver.py:1647–1680`): O6 and O7, 2 launches.

**Preflight.** Seven host commands at 30 s each (`harness.py:47–73`), and the static-probe
compile at 120 s (`harness.py:239`).

| Term | Author | Re-derived, author's over-counts | Re-derived, exact sets |
|---|---|---|---|
| watchdogs: 3 × 16 + 467 × 19 + 74 | 8,995 s | 8,995 s | 8,995 s |
| post-launch reads | 30 s (7 × 3, 5 × 1, 2 × 2) | 30 s | 22 s (5 × 3, 3 × 1, 2 × 2) |
| READY 30 s | 7,260 s (242) | 7,260 s (242) | 6,570 s (219) |
| preflight 7 × 30 + 120 | 330 s | 330 s | 330 s |
| **total** | **16,615 s = 276.9 min** | **16,615 s = 276.9 min** | **15,917 s = 265.3 min** |

The 471 launches and 242 READY waits are correct as the author defines them. The READY count and
both read counts safely over-count. With the step at 330 minutes, the margin is 53.1 minutes on
the author's figure and 64.7 minutes on the exact sets.

**Not bounded by any frozen constant:**

* **The compile** (`harness.py:268`, `:275`). Build 8's whole compile-only job took 77 s.
* **`ldd` on the static probe** (`harness.py:245`). It is not in the author's table and is
  immaterial.
* **The drain after a watchdog kill** (`proc.communicate()` at `driver.py:3663–3664` and
  `:3845–3846`).
* **CPU for parsing, scoring, sanitising and `fsync`.**

For scale, Trial #2's trial step took 90 s on the same watchdog design, 72 cases with 400
repetitions (job `103397999925`, 19:42:35–19:44:05 UTC).

The 330-minute step timeout backstops every unbounded piece without corrupting the one-shot
protocol. A stop before the first `case_pose_started` is `TRIAL_NOT_STARTED`: D-7 is not
consumed, and run 1 is spent. A stop after it is `TRIAL_ABORTED_AFTER_BOUNDARY`: D-7 is consumed
and the aggregate is not derivable. Both readings are preregistered in the manifest's
`partial_reading`, and neither permits a retry. The job keeps 20 more minutes: the steps before
the trial took about 7 s in Trial #2, and staging, upload and report about 1 s.

**`TRIAL3_DISPATCHER_TIMEOUT_BOUND_SOUND`.**

## 22. Hard timeout and preservation

GitHub documents `timeout-minutes` on a step as the limit after which its process is killed. The
step then counts as failed, and later `always()` steps still run. A real step timeout could not be
observed, because no run may be created. The design does not depend on the killed process doing
anything after the kill:

| After a hard stop of the trial step | State |
|---|---|
| `journal.jsonl` | every record written before the kill is flushed and `fsync`-ed; at most a torn final line, which the reader drops (`journal.py:210–234`) |
| `preflight.json`, `build-identity.json` | present if the runner had reached them; each is written and closed before the first case |
| `evidence.json` | absent; it is written only after `trial_end` |
| `runner-stdout.json` | created empty by the redirection and not staged |
| `code` output | absent; the report says so and fails |

The stand-in stop simulations (section 18) produced exactly that shape.

* **After the boundary** the preserved journal reads `TRIAL_ABORTED_AFTER_BOUNDARY`, under the
  frozen partial-reading rules.
* **Job timeout.** The 350-minute job timeout is not relied on: the trial step starts seconds
  into the job, so its 330-minute stop comes first.
* **Wording.** The workflow claims only that staging and upload *run* after "a timeout or a
  cancellation". The report tells the reader that "Its preserved evidence decides what happened".
  It never claims `evidence.json` survives a stop. DSP-B2 records that the comment could spell out
  the table above.

**`TRIAL3_HARD_TIMEOUT_PRESERVATION_MODEL_SOUND`.**

## 23. Artefact privacy and action pins

* **What is uploaded.** The upload `path` is the staging directory
  `${{ runner.temp }}/launch-exec-01-trial-003-evidence/`. It has no glob or recursion into the
  workspace, and it receives only the files staged by name.
* **What stays out.** Nothing from the build directory is staged: binaries, fixtures,
  case-private FIFOs and raw `<case>.strace` records (`driver.py:3610`). Neither are raw captured
  bytes, the environment or stderr.
* **Decoy test.** Every stand-in run planted decoys: `trial-output/.env`, `trial-output/raw.strace`,
  `trial-output/sub/capture.bin`, and a decoy ELF and trace under `target/launch-exec-01`. None was
  staged, and a recursive search of every staging directory for their private markers found
  nothing.
* **Hidden files.** None is staged, so the action's hidden-file default is not relied on.
* **Pins.** Both external actions are immutably pinned (section 4). GitHub forces
  `upload-artifact@ea165f8d` (v4.6.2, Node 20) onto Node 24; Trial #2's job carries that
  annotation. It uploaded Trial #2's artefact in that mode on 2026-09-11: "Artifact
  launch-exec-01-trial-002-evidence has been successfully uploaded". DSP-B3 records that the
  runtime is outside the workflow's control.

**`TRIAL3_DISPATCHER_ARTIFACT_PRIVACY_SOUND`.**

## 24. Trial #2 isolation

| Property | Trial #2 | Trial #3 candidate |
|---|---|---|
| Path | `.github/workflows/launch-exec-01-trial-002.yml` | `.github/workflows/launch-exec-01-trial-003.yml` |
| Name | `launch-exec-01 trial-002 authorised trial` | `launch-exec-01 trial-003 one-shot trial` |
| Confirmation | `RUN-TRIAL-002-ONE-VALID-TRIAL` | `RUN-TRIAL-003-ONE-VALID-TRIAL` |
| Concurrency group | `launch-exec-01-trial-002` | `launch-exec-01-trial-003` |
| Artefact | `launch-exec-01-trial-002-evidence` | `launch-exec-01-trial-003-evidence` |
| Job id | `trial-002` | `trial-003` |
| Freeze | `ba41a3f…` | `bebd8a5…` |

* **Groups.** The other concurrency groups in the repository are three
  `${{ github.workflow }}-${{ github.ref }}` groups and `launch-exec-01-compile-only`, so none
  collides.
* **Cross-references.** The Trial #3 file names Trial #2 only in two comments (lines 30 and 301).
  The Trial #2 file contains no Trial #3 value.
* **Trial #2 unchanged.** The Trial #2 blob is unchanged (section 2). Its workflow `356056797`
  still has exactly one run, `34640280964`, run 1, attempt 1, `success`. Nothing here touched a
  Trial #2 run, artefact or record.

**`TRIAL2_TRIAL3_DISPATCHER_ISOLATION_SOUND`.**

## 25. Current zero-run state

Read-only GitHub, at review time:

* `GET /repos/Djomla83/helm-os/actions/workflows/launch-exec-01-trial-003.yml` returns 404, and
  `gh run list` for that file reports it not found on the default branch.
* The contents API returns 404 for the path on `main` and on `docs/helm-launch-architecture`. The
  candidate is published nowhere.
* The workflow registry lists five workflows, none of them Trial #3.
* `GET /actions/runs?per_page=100` has `total_count` 62, and no run's path contains `trial-003`.
* `main` is `d8a6887…` remotely and locally, and the milestone is `5a6be59…`.
* This review created no run and made no dispatch.

**`TRIAL3_DISPATCHER_ZERO_RUN_STATE_SOUND`.**

## 26. Negative controls

Every control used only the extracted step bodies, or the stand-in runner, with shell and Python.
The environment was WSL2 Ubuntu with kernel 6.6.87.2, GNU bash 5.2.21, Python 3.12.3 and
git 2.43.0.

The identity, freeze, definition and static controls ran steps 2–6 in a disposable clone of this
repository. Nineteen fabricated commits existed only in that clone. Step 7 was never run there, and
afterwards the clone had no `target/` and no `trial-output/`.

**Guards (section 5).** All 23 wrong inputs fail with rc 1 before checkout. No injection created
its canary.

| Control | Halts at |
|---|---|
| P0 exact bindings | passes steps 2–6 |
| P1 legitimate `BUILD-EVIDENCE.md` append at the acceptance commit | passes steps 2–6 |
| N1 wrong manifest blob; N2 wrong manifest SHA-256 | step 2 |
| N3 freeze review = Trial #2 freeze (not a descendant) | step 2, "does not descend" |
| N4 freeze review = the freeze (record absent); N5 nonexistent commit | step 2 |
| N6 freeze-review record with `NEEDS_FIX` and the accepted token in a negative sentence | step 2, "does not carry the accepted classification" |
| N7–N10 Build 8 authority = `69f13a7`, `b8acabf`, `d1dc86e`, `1cb2e05` | step 2, re-review record absent |
| N11 authority = the freeze; N12 records copied onto the freeze | step 2, "does not descend from the freeze review" |
| N13 re-review "NOT ACCEPTED"; N14 re-review `NEEDS_FIX` | step 2 |
| N15 re-review attempt 2; N16 re-review wrong job-log SHA-256 | step 2 |
| N17 wrong job; N18 attempt 2; N19 `…_FAILURE`; N20 wrong log SHA-256; N21 wrong head, all in the evidence record | step 2 |
| N22 `driver.py`, N23 `launcher_spike.c`, N24 ADR drift at the acceptance commit | step 3, `DRIFT` |
| N25 manifest drift; N26 missing source; N27 missing definition at the acceptance commit | step 3 |
| N28 `oracles.py` drift at the freeze-review commit | step 3 |
| N29 checked-out `HEAD` is `69f13a7` | step 2, not the freeze |
| N30 untracked file; N31 source drift; N32 definition drift; N34 manifest edit | step 2, tree not clean |
| N31b source drift; N39 wrong source hash in the manifest | step 4 |
| N32b definition byte drift; N33 definition missing; N34b extra entry | step 5 |
| N35 swapped path; N36 wrong hash; N37 upper-case digest; N38 table is a list | step 5 |
| doctored completeness: incomplete, unposable, missing, 71 handlers, duplicate, unresolved rule, unknown channel, absent key | step 6, rc 1 |

**Trial step.** The exit-code and preservation controls are in section 18. The frozen runner was
never invoked with D-7, and no case was posed.

**`TRIAL3_DISPATCHER_NEGATIVE_CONTROLS_SOUND`.**

## 27. No current D-7

* **`docs/DECISIONS.md`.** Every Trial #3 mention is negative: lines 250–251 say the decisions "do
  not authorise … a Trial #3 D-7"; line 275 says "It is not Trial #3, uses no D-7"; line 289 says "no
  D-7 exists for Trial #3". No Trial #3 D-7 record or anchor exists.
* **`docs/PROJECT_STATE.md`.** It states "no D-7 and no dispatcher" for Trial #3 (line 28) and
  claims no Trial #3 D-7 anywhere.
* **The candidate commit** changes neither file and creates no authority record. The workflow
  prose says authority is absent (section 15).
* **This review** records no D-7.

**`TRIAL3_NO_CURRENT_D7_SOUND`.**

## 28. Proposed D-7 packet

**PROPOSED ONLY — D-7 NOT AUTHORISED.**

Every binding below was verified independently in sections 3 and 7–10:

```text
Trial:                          trial-003
Freeze SHA:                     bebd8a5f83d4d0daebe9b068050cb5436289c75e
Manifest Git blob:              8cd290b573408510f8c16cd8dafe676354140a38
Manifest SHA-256:               ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70
Freeze review:                  69f13a78810e1d270e4540b1fa169c8a8e9eb5fd
Formal Build 8:                 run 34754901079, job 103717520698, attempt 1
Accepted Build 8 evidence:      5a6be5959c5a9131afa1154c015369df46a5deb8
Build 8 durable job-log SHA-256: 6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15
Dispatcher path:                .github/workflows/launch-exec-01-trial-003.yml
Dispatcher Git blob:            64ce3d433a47eaae3eb28b8bb28f7300a330d762
Dispatcher file SHA-256:        9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8
Required confirmation:          RUN-TRIAL-003-ONE-VALID-TRIAL
Proposed execution scope:       exactly ONE valid Trial #3 execution
D-7 consumption boundary:       first fsynced case_pose_started
```

**PROPOSED ONLY — D-7 NOT AUTHORISED.**

**`TRIAL3_PROPOSED_D7_PACKET_EXACT`.**

## 29. Publication plan

The intended order is:

1. D-7 stays absent.
2. Push the dispatcher commit and this review to the milestone branch.
3. Verify the exact workflow blob remotely.
4. Verify zero Trial #3 runs.
5. The owner decides D-7 for that exact blob.
6. Record D-7 prospectively.
7. Publish the exact workflow byte-identically to `main`.
8. Verify the blob on `main`.
9. Verify zero runs again.
10. One human dispatch.

**Why this order is sufficient.**

* **No execution before `main`.** The workflow executes only once it is on the default branch
  (section 6).
* **`main` only after D-7.** It reaches `main` only after a D-7 that names blob `64ce3d43…` at
  this path.
* **Only the first run passes.** The guards admit only the first run of this path, on `main`.
* **Blob drift is caught procedurally.** A file on `main` whose blob differs from the bound one
  could skip those gates. The workflow cannot hash itself, so that drift is caught by the blob
  checks in steps 7 and 9 of the checklist below and by the post-run check in step 11. DSP-M1
  makes those checks preconditions. With them, an unreviewed workflow cannot gain execution
  authority.

**Push-triggered CI preclassification.**

* The dispatcher path and this review's path match no push filter of the four push-triggered
  workflows:
  * `launch-exec-01-compile-only`: `docs/experiments/launch-exec-01/**`,
    `tools/tests/test_launch_exec_01.py` and its own file;
  * `helm-evidence`: Cargo files, `crates/**`, named tools, `tools/tests/**` and its own file;
  * `helm-bind`: Cargo files, crates and its own file;
  * `helm-observe-independent-review`: branches `review/helm-observe-**`.
* The milestone push and a `main` commit that adds only the workflow are therefore expected to
  start no run.
* Any run that appears is classified before anything else happens. Any run of the Trial #3 path
  is a STOP for the owner.

**Operator checklist.**

1. Keep D-7 absent. Push the milestone once, fast-forward, from `5a6be59` through this review
   commit.
2. Verify that `origin/docs/helm-launch-architecture` carries blob `64ce3d43…` at the path, and
   that the contents API `sha` on the milestone is `64ce3d43…`.
   `GET …/actions/workflows/launch-exec-01-trial-003.yml` must still return 404, with no run of
   the path.
3. The owner decides D-7 on the section-28 packet, binding the path, blob and SHA-256 together.
4. Record D-7 prospectively, in documentation only, without touching the workflow.
5. On `main`, add only this path from the reviewed commit, for example with
   `git checkout <this review commit> -- .github/workflows/launch-exec-01-trial-003.yml`.
6. Assert `git rev-parse HEAD:.github/workflows/launch-exec-01-trial-003.yml` equals `64ce3d43…`,
   and that the commit changes only that path. Push.
7. Verify that the contents API `sha` on `main` is `64ce3d43…`, that the workflow is listed
   `active` at that path, and that `…/workflows/launch-exec-01-trial-003.yml/runs` has
   `total_count` 0. Any run means STOP.
8. **DSP-M1 preconditions, until the run ends:**
   * do not edit, rename, copy or delete the file on `main`;
   * do not delete or force-push the milestone branch. The checkout must fetch `bebd8a5`,
     `69f13a7` and `5a6be59`.
9. Immediately before dispatch, re-verify the blob on `main` and zero runs. One human then
   dispatches once, selecting `main`, with `RUN-TRIAL-003-ONE-VALID-TRIAL`.
10. If a pre-boundary rejection spends run 1, do not bypass or edit the guard. STOP for the owner.
11. **After the run**, before any result review, verify:
    * the run's `path`, `event` `workflow_dispatch`, `head_branch` `main`, `run_number` 1 and
      `run_attempt` 1;
    * that `git rev-parse <run head_sha>:.github/workflows/launch-exec-01-trial-003.yml` equals
      `64ce3d43…`.

    Then preserve the artefact in the repository well within its 90 days.

**`TRIAL3_DISPATCHER_PUBLICATION_PLAN_SOUND`.**

## 30. Validation

**Windows host** (Python 3.14.3, GNU bash 5.2.37):

* `git diff --check`: clean for the candidate (`git diff --check 5a6be595 f197386`) and for this
  record (`git diff --cached --check` before the commit).
* `python tools/validate_docs.py`: PASS (127 Markdown files, 251 JSON files, 1,296 link targets).
* `python -m unittest discover -s tools/tests`: 783 tests, OK, 74 skipped.
* `cargo fmt --check`: rc 0.
* Source freeze verification: `freeze_verified: true`.
* Definition hashes: 3/3 through the extracted step body (Linux).
* YAML: PyYAML 6.0.3, parsed as described in section 4.
* `bash -n` on all nine `run:` bodies: rc 0.
* `actionlint`: not available, and not installed.

**Linux** (WSL2 Ubuntu, bash 5.2.21, Python 3.12.3, git 2.43.0): the section 5, 18 and 26
simulations.

**Read-only GitHub:** the workflow registry, runs, contents, commits, tags of both action
repositories, check-run annotations and job steps of Trial #2's job `103397999925` and Build 8's
job `103717520698`, and the artefact retention policy.

| Counter | Value |
|---|---|
| Workflow runs created | ZERO |
| Manual dispatches | ZERO |
| Experimental LAUNCH-EXEC ELF executions | ZERO |
| Cases posed | ZERO |
| Valid Trial #3 count | ZERO |

## 31. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| DSP-M1 | MINOR | exact path and blob authority | Only through a new commit to `main`, which is unprotected and has no rulesets | The guards bind the per-path `run_number`, not the executing path or blob. A copy or rename of this file on `main` would start a fresh counter at 1 and pass every guard, and an edit after D-7 would keep run 1 available to unreviewed bytes. A workflow cannot hash its own blob. | Not fixed; this review has no fix authority. Carried as checklist steps 7–9 and 11 in section 29. Same exposure as the accepted Trial #2 dispatcher. Non-blocking. |
| DSP-B1 | BACKLOG_NONBLOCKING | timeout table | No | The author's table omits the untimed `ldd` on the static probe (`harness.py:245`). It over-counts READY waits (242 against 219), liveness reads (7 against 5) and health reads (5 against 3). | Record only. The over-counts are safe, and the `ldd` call is immaterial and backstopped by the step timeout. |
| DSP-B2 | BACKLOG_NONBLOCKING | hard-timeout wording | No | The comments do not spell out what a hard stop leaves: no `evidence.json`, no `runner-stdout.json`, and possibly a torn final journal line. | Record. The frozen partial-reading rules already govern (section 22). |
| DSP-B3 | BACKLOG_NONBLOCKING | action runtime | Not at review time | `upload-artifact` v4.6.2 targets Node 20 and is forced onto Node 24. It worked that way for Trial #2 on 2026-09-11, and a later runner change is outside the workflow's control. | Record. The pin stays reviewed, and any change would need a new review. |
| DSP-B4 | BACKLOG_NONBLOCKING | runner stderr | Job log only | Carried from Trial #2 (DR-M2): stderr in a public job log can be unsanitised. | Carried. It does not reach the artefact. |
| DSP-B5 | BACKLOG_NONBLOCKING | project state | No | `PROJECT_STATE.md` still describes the freeze as unpublished, with Build 8 required and no dispatcher. It claims no D-7. | Update with the next owner state record, not in this review. |

No BLOCKER and no IMPORTANT finding exists.

## 32. Authority, next gate and classification

**TRIAL #2 D-7 IS CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE.**

**TRIAL #2 MUST NOT BE RERUN.**

**TRIAL #3 FREEZE IS PUBLISHED.**

**BUILD 8 FORMAL COMPILE-ONLY EVIDENCE IS PUBLISHED AND ACCEPTED.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ZERO.**

**TRIAL #3 D-7 IS NOT AUTHORISED.**

**TRIAL #3 IS NOT_RUN.**

**TRIAL #3 MUST NOT BE EXECUTED.**

There is no BLOCKER or IMPORTANT finding, and every load-bearing dispatcher verdict is SOUND:

**TRIAL #3 DISPATCHER IS READY FOR MILESTONE PUBLICATION WITH D-7 STILL ABSENT.**

That readiness grants nothing. This review does not authorise D-7, publish anything or execute
Trial #3. D-7, publication to `main` and the one dispatch remain separate owner-controlled gates,
in the order of section 29.

Classification: **TRIAL_3_DISPATCHER_REVIEW_PASSED_READY_FOR_MILESTONE_PUBLICATION**
