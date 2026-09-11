# LAUNCH-EXEC-01 — Trial #2 dispatcher bounded independent infrastructure review

This is a bounded independent infrastructure review of the owner's Trial #2 D-7 binding
([record](../DECISIONS.md#d-7-authorised-trial-002), `900fe5a`, anchor repair `8b6410b`) and of the
one-shot dispatcher `.github/workflows/launch-exec-01-trial-002.yml` (`f6425da`). **It is not an
experiment review.** The frozen mechanism and cases were closed by the
[final review of `ba41a3f`](HELM-LAUNCH-EXEC-01-TRIAL-002-BA41A3F-FINAL-REVIEW.md) (`5da397c`),
and no case semantics are reopened here. The reviewer did not write the dispatcher.

Nothing was fixed, pushed or dispatched. No workflow was dispatched and no GitHub Actions run was
used to test anything. The frozen runner was never invoked with the D-7 flag. No `launcher_spike`
or helper ELF ran, no `Authorisation` was constructed and no case was posed. D-7 was not consumed.
Every simulation ran on local scratch copies. The trial step was exercised only through a stub
`python3` in an empty directory where the real runner does not exist.

**Classification: `TRIAL_2_DISPATCHER_READY_FOR_PUBLICATION`.**

Every load-bearing check is sound, and no BLOCKER or IMPORTANT finding exists. Two MINOR findings
are recorded:

* **DR-M1** — operator preconditions for publication and dispatch, carried as the checklist in
  section V;
* **DR-M2** — the runner's stderr is an unsanitised public channel. It is bounded by audit to
  interpreter diagnostics, and it holds no credential.

Three backlog notes are also recorded.

## A. Target and history

| Item | Value |
|---|---|
| D-7 authorisation | `900fe5a403eded9f148494f557fb357fea3a207e` |
| Anchor repair | `8b6410becdfb792170e21cf16bc4d6db70b139c7` |
| Dispatcher | `f6425da7d4c6b318df458532d1a5eac7e27dbfa1` |
| Path / name | `.github/workflows/launch-exec-01-trial-002.yml` / `launch-exec-01 trial-002 authorised trial` |
| Reviewed workflow blob | `1d030130f255ebb74ace047d04590d6aee2838df` (identical at `f6425da` and `8b6410b`) |
| Reviewed workflow SHA-256 | `8c5199a2140e3ba61203feb751df825380f3c397e8cf0207cec4cc630ddf7432` (no CR byte) |
| Freeze / manifest blob / SHA-256 | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` / `6f000fac9a48625d3c9def18e16ae7ce1b61efa8` / `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94` |
| Final review | `5da397c6a79aea7c9a626789480903d50df0b7b4` |
| Remote milestone | `f417984`, an ancestor of the local tip (fast-forward only) |
| `main` | `b766917`, equal to `origin/main`; three workflows, none of them a trial dispatcher |

The history is linear: `ba41a3f` → `5da397c` → `900fe5a` → `f6425da` → `8b6410b`. The whole of
`ba41a3f..8b6410b` touches four paths:

* `5da397c` adds only its review record;
* `900fe5a` appends 69 lines to `DECISIONS.md` and inserts a 22-line section at the top of
  `PROJECT_STATE.md`;
* `f6425da` adds only the workflow;
* `8b6410b` re-inserts the one anchor line `900fe5a` displaced.

The net diff from `5da397c` to `8b6410b` removes **zero** lines. The frozen paths are unchanged:
the experiment directory, the definition, ADR-0024 and the architecture research document. So are
every Trial #1 record and dispatcher history, and every earlier review record. The intermediate
`900fe5a` did lack the AB7 anchor. That transient state is kept as history and was repaired
forward, not rewritten.

**`AUTHORIZATION_HISTORY_SOUND`.**

## B. D-7 binding

[`DECISIONS.md#d-7-authorised-trial-002`](../DECISIONS.md#d-7-authorised-trial-002) binds the
following exactly:

* freeze `ba41a3f12be4…7ae4`;
* manifest blob `6f000fac…efa8` and SHA-256 `616dc6b3…9a94`;
* final review `5da397c6…0b7b4`, recorded with its classification
  `TRIAL_2_READY_FOR_NEW_D7_DECISION`;
* scope: **exactly one** valid Trial #2 execution.

Consumption happens when the **first successfully fsynced `case_pose_started`**. That is the same
boundary as definition section 9.3 and the manifest's `durable_evidence.journal.boundary`. After
it, there is no rerun, resume or retry. A refused dispatch, a failed gate, `HALT_PREFLIGHT` or
`HALT_BUILD_IDENTITY` produces no case status, no aggregate and no change to the valid trial
count, and it does not consume D-7. The record states the dispatcher-bricking consequence and
leaves any further step to the owner. The frozen `status: NOT_RUN` and
`d7_execution_authorised: false` stay untouched inside the manifest, as was done for Trial #1.
`PROJECT_STATE.md` restates the same binding.

**`D7_BINDING_SOUND`.**

## C. Trigger surface

The workflow was parsed as YAML with PyYAML 6.0.3; the `on` key reads as `True` under YAML 1.1:

* The only trigger is `workflow_dispatch`. There is no `push`, `pull_request`, `schedule`,
  `workflow_run`, `repository_dispatch` or `workflow_call`.
* The one input is `confirmation`, of `type: string` and `required: true`.
* Top-level `permissions: {contents: read}`. The job declares no `permissions`, `environment`,
  `services`, `container`, `strategy`, `continue-on-error`, `if` or `needs`.
* No step references `secrets.*` or `github.token`. The repository has no environments, and its
  default workflow permission is `read`.

**`DISPATCH_TRIGGER_SURFACE_SOUND`.**

## D. Default-branch publication model

GitHub's documentation for the `workflow_dispatch` event says:

> "This event will only trigger a workflow run if the workflow file exists on the default branch."

The default branch is `main`, and the repository is **public**. `main` does not contain the file.
`GET /actions/workflows/launch-exec-01-trial-002.yml` returns 404: the workflow is not registered
and has no run. So publishing only the milestone branch does **not** make it dispatchable. The
reviewed file must be copied to `main` byte-identically, as blob `1d030130…`. For a dispatch,
GitHub runs the workflow file at `GITHUB_SHA`, the tip of the selected ref. The copy on `main` is
therefore the thing that executes, and it must stay identical until the dispatch.

After publication, a dispatch may name any ref that also carries the file, including the milestone
branch. The step-0 guard rejects that ref. The rejection spends run 1, which is DR-M1(a).

**`DEFAULT_BRANCH_PUBLICATION_MODEL_SOUND`.**

## E. First-step one-shot guards

Step 0 is the first executable step. No `uses:` step, checkout, build or runner comes before it.
Its inputs reach the shell only through `env:`, and the parsed step bodies contain no `${{ }}`.
The confirmation value is compared, never echoed. This was simulated under GitHub's documented
`bash --noprofile --norc -eo pipefail`, in an empty directory:

| Input | Result |
|---|---|
| all five correct | rc 0, `one-shot guards passed` |
| `run_number` 2, or `01` | rc 1, HALT |
| `run_attempt` 2 | rc 1, HALT |
| ref `refs/heads/docs/helm-launch-architecture`, or `refs/tags/main` | rc 1, HALT |
| event `push` | rc 1, HALT |
| all wrong | rc 1, five HALT lines, "Nothing was checked out, built or posed" |

Twenty wrong confirmations were each rejected with rc 1:

* the empty string;
* the value with a leading space, a trailing space or a trailing newline, and the lower-case value;
* the Trial #1 string;
* `$(touch …)`, a backtick `touch`, a quote-breaking `x"; touch …; echo "` and `…; touch …`, with
  the injection forms also appended to the correct value;
* `' || touch … #`;
* `=`, `!`, `(`, `)`, `-n` and `-o`;
* `::error::x`.

No canary file was created.

**`ONE_SHOT_GUARDS_SOUND`.**

## F. run_number fail-closed model

A pre-boundary rejection of run 1 spends this dispatcher, while D-7 stays authorised and
unconsumed and the valid trial count stays zero. That is intended fail-closed behaviour, and it is
not a defect. GitHub's contexts reference supports each assumption:

* `run_number` is *"A unique number for each run of a particular workflow in a repository. This
  number begins at 1 for the workflow's first run."* The workflow is keyed by file path. The
  repository shows independent counters: `launch-exec-01-compile-only` has runs 1–9 and
  `helm-evidence` has runs 25–29 over the same pushes. Trial #1's `launch-exec-01-trial.yml` had
  its own run 1. The new path has no registered workflow and no run, so its first run is 1.
  Unrelated workflows cannot increment it.
* **Publishing creates no run.** The file has no event that fires on push. There is one caveat,
  which this review did not verify: a workflow file GitHub cannot parse can surface as a failed
  push run. That is why section V checks for zero runs after each push (DR-M1(d)).
* `run_attempt` *"increments with each re-run."* A re-run is possible for 30 days. It re-executes
  the job from step 0 and is rejected before checkout. The run's displayed conclusion then shows
  the rejected attempt. Attempt 1's artefact is unaffected.

If a pre-boundary rejected run 1 ever happens: do not bypass the guard, and do not edit the
`run_number` logic in place. **STOP** for a new owner decision.

**`RUN_NUMBER_ONE_SHOT_MODEL_SOUND`.**

## G. Concurrency

The group is `launch-exec-01-trial-002` with `cancel-in-progress: false`. No other group in the
repository collides with it:

* the three `${{ github.workflow }}-${{ github.ref }}` groups;
* `launch-exec-01-compile-only`.

GitHub allows at most one running member per group, and a newer request replaces only a
**pending** member, never the running trial. Any second request gets `run_number` ≥ 2 when it is
created. Whether it is queued behind the trial, replaced while pending or dispatched after the
trial completes, it fails step 0 before checkout.

**`DISPATCH_CONCURRENCY_SOUND`.**

## H. Hard-coded identity

The workflow `env` hard-codes these values exactly:

* `FREEZE_SHA=ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`;
* `MANIFEST_SHA256=616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94`;
* `MANIFEST_GIT_BLOB=6f000fac9a48625d3c9def18e16ae7ce1b61efa8`;
* `FINAL_REVIEW_SHA=5da397c6a79aea7c9a626789480903d50df0b7b4`.

The checkout `ref:` repeats the freeze literal. If the two literals diverged, the HEAD gate would
halt. The operator supplies only `confirmation`. Trial #1's operator-supplied SHA inputs are gone.

**`HARDCODED_IDENTITY_SOUND`.**

## I. Checkout exactness

The pinned `actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09` resolves to tags `v5` and
`v5.1.0`; the upload action `ea165f8…` resolves to `v4` and `v4.6.2`. Its source at that commit
does the following:

* It maps a 40-hex `ref` to `commit`, with an empty `ref` (`input-helper.ts` lines 92–95).
* With `fetch-depth: 0`, it fetches `+refs/heads/*:refs/remotes/origin/*` and the tags. If the
  commit is still absent, it fetches the SHA directly.
* It checks the commit out detached, and removes the auth header because `persist-credentials` is
  `false`.

It never uses `main`, `github.sha` or a tag. Step 2 then asserts `git rev-parse HEAD ==
FREEZE_SHA` and an empty `git status --porcelain` before anything runs.

**`EXACT_FREEZE_CHECKOUT_SOUND`.**

## J. Final-review fetch and binding

The descendant `5da397c` is reachable on a fresh runner only through the all-heads fetch.
Simulation, on a scratch clone:

* With every head fetched and HEAD detached at the freeze, step 2 passes.
* A depth-1 fetch of the freeze alone fails at `git cat-file -e 5da397c…^{commit}` and halts. So
  `fetch-depth: 0` is load-bearing and fails closed.

Today the remote milestone is at `f417984`, which holds **neither** `ba41a3f` nor `5da397c`. A
dispatch now would halt. Section V pushes the milestone through this review **before** `main`
publication, so both commits are reachable from `refs/heads/docs/helm-launch-architecture`. They
must stay reachable until the dispatch (DR-M1(b)).

**`FINAL_REVIEW_FETCH_AND_BINDING_SOUND`.**

## K. The review cannot have mutated frozen bytes

Four checks apply:

* `merge-base --is-ancestor` requires the review to descend from the freeze.
* The classification is read from the **record file at the review commit**, never from a commit
  message.
* `git diff --quiet FREEZE FINAL_REVIEW -- <paths>` covers every frozen byte. The paths are the
  experiment directory, which holds every manifest `sha256` entry and the manifest itself, plus the
  three `definition_sha256` documents.
* All four pathspecs exist at both commits, so no typo can pass silently.

Synthetic descendants in a scratch clone gave these results:

| Synthetic descendant | Result |
|---|---|
| mutates the experiment README, the definition or ADR-0024 | halts: "frozen bytes differ" |
| commit message claims the classification, record lacks it | halts on the classification |
| an ancestor review, `898a31c` | halts: "does not descend" |
| doc-only descendant carrying the classification (control) | passes |

**`REVIEW_CONTENT_BINDING_SOUND`.**

## L. Manifest identity

Each of the following is asserted independently: the working-file SHA-256, the working file's
`git hash-object`, and the tree entry `HEAD:<manifest>`. Then the frozen `--verify-freeze` must
report `true`. At the freeze, all three equal the authorised values, and the blob's content hashes
to `616dc6b3…9a94` with no CR byte. `.gitattributes` gives `eol=lf`, so a Linux checkout is
byte-identical to the blob.

`git hash-object` applies that normalisation, so byte exactness comes from `sha256sum`. Both are
asserted. A harmless tamper test changed manifest bytes while the index reported clean, using
`assume-unchanged`. It was caught by the SHA-256 check, and the dirty-tree case is caught by the
clean check.

**`MANIFEST_GATES_SOUND`.**

## M. Static completeness gates

Step 4 runs `--driver-completeness`, which walks tables and calls no handler. It then asserts the
following before the trial step exists:

* 72 total, 54 mandatory, 11 conditional and 7 recorded;
* traced exactly `E1 E7 F4 F7 M1 M2 M3 M4`;
* complete, with 72 frozen, 72 handlers, and nothing missing, unknown, duplicated or unresolved;
* 72 posable, 0 unposable.

At the freeze, every line was `ok`. The same gate halted on three doctored inputs: one unposable
case, a driver total of 71, and one missing case. A failure there skips the trial step, and the
runner's own `preflight_gates()` repeats the completeness and channel gates before the first case.

**`STATIC_PREEXEC_GATES_SOUND`.**

## N. Single D-7 invocation

The runner is invoked three times: `--verify-freeze`, `--driver-completeness` and the one trial
invocation. The D-7 flag appears on exactly one line, in the step marked THE AUTHORISED TRIAL #2.
There is no `--preflight-only`, loop, matrix, retry, `continue-on-error`, fallback, or cleanup
re-execution. The staging, upload and report steps run no Python. No heredoc constructs
`Authorisation`.

**`SINGLE_D7_RUNNER_INVOCATION_SOUND`.**

## O. Exit-code capture

The trial step runs `set +e`, then invokes the runner with stdout redirected to a file, then
captures `code=$?`, restores `set -e`, echoes the code and writes it to `GITHUB_OUTPUT`, then
exits 0. There is no pipe, `tee`, `timeout` wrapper or subshell, so `$?` is the runner's own
status.

A stub `python3` was used in an empty directory, under GitHub's shell flags. The step exited 0
every time, and `GITHUB_OUTPUT` held the runner's exact status: `code=0/1/3/4/6/42/255`, 137 for
SIGKILL and 143 for SIGTERM. The stub received exactly `run_launch_exec_01.py
--i-have-owner-authorisation-d7 --build-dir target/launch-exec-01 --out-dir trial-output`. These
are the runner's own defaults, and they resolve against the same working directory as staging.

**`RUNNER_EXIT_CAPTURE_SOUND`.**

## P. Timeouts and postmortem

The frozen plan table was read statically: 72 plans, with O8 and S7 at 200 repetitions each.

* **Every launch watchdog firing in every repetition** gives **149.9 min**. That covers
  `total_bound_ms + 5 s`, the P-series 3 s liveness read and the O6/O7 2 s signal read. This is the
  dispatcher comment's "about 150 minutes".
* **Also letting the 30 s READY wait expire at every launch that can use the barrier** gives at
  most **284.9 min**. That is every launch except O8's 200, which have no setup and no parent
  state.

The step timeout is 330 min and the job timeout is 350 min. GitHub's hosted limit is *"up to 6
hours of execution time"*, so there is little room above 350. The pre-trial steps and the
staging, upload and report steps take minutes, well inside the 20-minute reserve. For scale,
Trial #1 built and ran five cases in 8.4 s.

Simulations of a stopped step:

* SIGINT to the process group gives shell status 130 and no `code` output.
* SIGTERM to the shell gives 143 and no `code` output.
* A killed runner under a surviving shell records 137 or 143.

The trial outcome is then `failure` or `cancelled`, not `skipped`, so staging, upload and report
still run. The report prints "reported no exit code … Its preserved evidence decides" and fails.
After the boundary, the journal reads `TRIAL_ABORTED_AFTER_BOUNDARY` under section 9.3. Nothing
in the workflow derives a verdict.

**`TIMEOUT_AND_POSTMORTEM_SOUND`.**

## Q. Artefact preservation, `runner-stdout.json` and the whitelist

Staging and upload run under `always() && steps.trial.outcome != 'skipped'`. They copy each
existing file of `preflight.json`, `build-identity.json`, `journal.jsonl` and `evidence.json` by
name, print `not produced` for each absent one, and never fail.

| Staging simulation | Staged |
|---|---|
| abort, with decoy `*.strace`, `sub/capture.bin`, `.env` and a build-dir trace present | `preflight.json`, `journal.jsonl`, `runner-stdout.json` only |
| crash with empty stdout | the two journal-side files only |
| completed | all four plus `runner-stdout.json` |
| runner died before the out dir existed | nothing; step rc 0, and the upload only warns |

**`FROZEN_ARTIFACT_PRESERVATION_SOUND`** and **`ARTIFACT_WHITELIST_SOUND`**: files are staged by
explicit name, with no recursion, no build directory, no raw trace, capture, FIFO or environment,
and no hidden file.

**`runner-stdout.json`.** The four-file list in definition section 9.4 and the manifest's
`artifact_contract.files` is the durable set the trial writes. Its stated purpose is that each file
is P-14 sanitised before the disk write, "so a later upload is never the first privacy boundary".
It does not say "only". The same frozen manifest's `publication_boundary` governs **runner
stdout** itself, in two statements:

* "no document reaches stdout without passing `Sanitiser.record` and `serialise`";
* its `covered_paths` names the trial evidence document, `HALT_PREFLIGHT` and `HALT` — documents
  the runner emits on stdout.

Its invariant is that no module other than `evidence` writes to stdout, and this review's grep
finds no `print(` anywhere in the experiment's Python. On the D-7 path, stdout is exactly one
`evidence.publish` output:

* the RUN document, byte-identical to `evidence.json` (the same deterministic serialisation over
  the same stateless sanitiser);
* a `HALT_PREFLIGHT` or `HALT_BUILD_IDENTITY` document, whose facts are also in the journal and
  `preflight.json`;
* or the freeze `HALT`, unreachable after steps 2–3.

A crash leaves it empty, and it is then not staged. Trial #1, under the same definition, uploaded
its runner stdout as `evidence.json`. The extra file adds no data category and replaces no frozen
file. It is not durable-evidence authority: the journal decides under section 9.3.

**`RUNNER_STDOUT_EXTRA_ARTIFACT_ALLOWED`.**

## R. The GitHub job log is also publication

The repository is public, so its Actions logs are public. The dispatcher's comment is true
mechanically: stderr is not staged, and it goes to the job log. The job log **is** a publication.

**What can reach the log.**

* The workflow's own lines print SHAs, hashes, `ls -l` of the staged files, and the verify-freeze
  JSON, which already passed P-14.
* There is no `set -x`.
* The confirmation value is never echoed. GitHub's own step header does list it, and it is a
  public dispatch input.
* Runner **stderr** reaches the log unredacted.

**What runner stderr can contain, by audit of the frozen Python.**

* No module writes to stderr or calls `print`.
* All eleven `subprocess` sites capture stderr, with `capture_output=True` or `stderr=PIPE`. There
  is no `os.fork`, `exec*`, `system` or `posix_spawn` call.
* The parsers of captured bytes and of tracer text return `None` on decode or JSON failure; they
  do not raise. The only explicit raises that embed data are three:
  * `harness.build`'s compiler stderr, which comes before the boundary and holds paths and lines of
    public source;
  * journal errors, which name cases;
  * vocabulary errors.
* stderr can therefore carry only interpreter diagnostics: tracebacks with the frozen files'
  absolute paths and public source lines, exception text, `KeyboardInterrupt` on a stop, and a
  shell `Terminated`.

Trial #1's public log shows exactly this: a traceback under `/home/runner/work/helm-os/helm-os`
ending `OSError: [Errno 9] Bad file descriptor`. That path is GitHub's fixed hosted layout, which
the checkout step prints in every run. It is not a private host path.

**Credentials.**

* No secret or `github.token` is mapped into any step.
* `contents: read` grants no `id-token`, so no OIDC request token exists.
* `persist-credentials: false` removes the checkout token from `.git/config`.
* D-10 scrubs the child's environment down to declared names.
* One point was not observed live: that GitHub exports its runtime token only to JavaScript and
  container actions, never to `run:` steps, and masks registered secrets in logs. This review
  relies on that platform behaviour without testing it.

The frozen P-14 rules govern published documents and the durable files. The preregistered
hosted-runner environment necessarily shows step stderr, as it did for Trial #1, whose result
review used that traceback as boundary evidence. No privacy-relevant category can be present. What
remains is DR-M2.

**`GITHUB_LOG_PUBLICATION_BOUNDARY_SOUND`.**

## S. Upload failure

If `upload-artifact` fails:

* The report step still runs under `always()` and propagates the runner's code, or 1.
* No step retries.
* A re-run is rejected by the `run_attempt` guard.
* The mechanism verdict is not changed. It becomes unreadable instead: the only remaining public
  trace is the job log, which holds the staged files' sizes and SHA-256s, the runner's exit code
  and any stderr.

The expected project reading is as follows:

* The trial's status cannot be read from preserved durable evidence.
* Under section 9.3's deliberate bias, D-7 is treated as **consumed** once the D-7-flagged runner
  step has executed, because an unfsynced boundary cannot be shown.
* No aggregate exists, and the valid trial count does not increase.
* Any further step is an owner decision.

The workflow invents no retry.

**`UPLOAD_FAILURE_SEMANTICS_SOUND`.**

## T. Final outcome propagation

The last step runs only after staging and upload. It exits with the runner's code, or 1 when no
code exists. It states that a CI colour is not a verdict. `run_trial` returns 0 for **any**
completed trial, including `MECHANISM_REJECTED`, so green means only "completed". Red means a
halt, an exception, a stop or an upload failure. The workflow never parses a status or aggregate
and never labels red as `MECHANISM_REJECTED`.

**`FINAL_OUTCOME_PROPAGATION_SOUND`.**

## U. Trial #1 isolation

* `launch-exec-01-trial.yml` is absent at the milestone tip and on `main`, and it is absent from
  the workflow list.
* Run `34500901306` is unchanged: `run_number` 1, `run_attempt` 1, `failure`, last updated
  2026-09-10T16:15:33Z. Its attempt 2 does not exist, and its `launch-exec-01-evidence` artefact
  is intact.
* The new path, name, concurrency group and artefact name all differ from Trial #1's. No Trial #1
  value appears in the new file.

**`TRIAL1_DISPATCH_ISOLATION_SOUND`.**

## V. Publication sequence and operator checklist

`main` has no branch protection and no rulesets; Trial #1's dispatcher was committed directly
there. Actions are enabled with `allowed_actions: all`, and the 90-day retention equals the
repository maximum. The proposed sequence is sufficient with the checks below. It does not require
merging the milestone into `main`.

1. Commit this review locally.
2. Push the complete milestone sequence through this review commit, **once**.
3. Verify that the remote milestone contains `ba41a3f`, `5da397c`, `900fe5a`, `f6425da` and this
   review. **DR-M1(d):** `gh api repos/Djomla83/helm-os/actions/workflows/launch-exec-01-trial-002.yml`
   still returns 404, with no run.
4. On `main`, run `git checkout <this review commit> -- .github/workflows/launch-exec-01-trial-002.yml`.
   Assert `git rev-parse HEAD:.github/workflows/launch-exec-01-trial-002.yml` =
   `1d030130f255ebb74ace047d04590d6aee2838df`, and that the commit changes only that path.
5. Commit and push that publication on `main`. The remote contents API `sha` for the file on
   `main` must equal `1d030130…`.
6. Verify that GitHub lists the workflow `launch-exec-01 trial-002 authorised trial` at that path,
   `active`, and that no Trial #1 workflow is listed.
7. Verify `…/actions/workflows/launch-exec-01-trial-002.yml/runs` has `total_count` 0. A failed
   run here means **STOP** for an owner decision.
8. A human dispatches **once**, selecting **`main`**, with `RUN-TRIAL-002-ONE-VALID-TRIAL`
   (DR-M1(a)). Until then, the milestone branch is not deleted or force-pushed (DR-M1(b)) and the
   file on `main` is not edited (DR-M1(c)).

**`PUBLICATION_SEQUENCE_SOUND`.**

## W. Validation performed

**Windows host:**

* `git diff --check ba41a3f 8b6410b` and `git diff --check f6425da^ f6425da` were clean.
* `python tools/validate_docs.py`: PASS.
* `python -m unittest discover -s tools/tests`: 667 tests, OK, 40 skipped.

**Linux (WSL Ubuntu, GNU bash 5.2.21, Python 3.12.3, git 2.43.0), scratch clones and empty
directories only:**

* `bash -n` on all seven `run:` bodies: 0.
* The simulations of sections E, J, K, L, M, O, P and Q.
* The static watchdog computation of section P.

**Read-only GitHub:**

* repository visibility, workflows and runs;
* the Trial #1 run and its log;
* the action tag pins and the pinned checkout source;
* protection, rulesets, Actions permissions and retention;
* documentation for contexts, events, concurrency, re-runs and limits.

**Not performed:**

* No actionlint or JSON-schema validator was available. The YAML was parsed and its structure
  asserted, so steps 3 and 7 of section V guard the invalid-file hypothesis.
* No GitHub Actions run of the dispatcher.
* No live observation of runner environment variables.

## X. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| DR-M1 | MINOR | operation | only by operator action; fails closed | (a) A dispatch on a ref other than `main` spends run 1. (b) `ba41a3f` and `5da397c` must stay reachable from a remote head until dispatch. (c) The file on `main` must stay blob `1d030130…`. (d) Zero runs must be confirmed after each push, in case GitHub turns an unparseable file into a run (hypothesis). | Carried as the section V checklist. No dispatcher change. |
| DR-M2 | MINOR | log publication | abnormal paths only | Runner stderr is an unsanitised public channel. The audit bounds it to interpreter diagnostics with no credential. An exception message could still carry an experiment-local value, such as a descriptor number or a build or temporary path, that P-14 withholds from the durable files. | Recorded. The result review treats the log as unsanitised, non-authoritative context and quotes it with the workspace prefix removed, as Trial #1's did. |
| DR-B1 | BACKLOG_NONBLOCKING | timeout comment | no | "About 150 minutes" is the launch-watchdog bound, 149.9 min. With every barrier READY wait expiring, the bound is ≤ 284.9 min, still under 330. | Record only. |
| DR-B2 | BACKLOG_NONBLOCKING | diagnostics | no | On a hash drift, `set -e` stops step 3 before `cat`, so the drift detail is not logged. Unreachable after step 2's HEAD and clean-tree assertions. | Record only. |
| DR-B3 | BACKLOG_NONBLOCKING | retention | post-trial | The artefact expires after 90 days, while definition section 7 preserves evidence under `docs/experiments/evidence/`. | Commit the downloaded evidence well before expiry. |

No BLOCKER or IMPORTANT finding exists.

## Y. State and classification

TRIAL #2 D-7 AUTHORISED BUT NOT YET EXECUTED

LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT REMAINS ZERO

TRIAL #2 DISPATCHER IS READY FOR PUBLICATION

**`TRIAL_2_DISPATCHER_READY_FOR_PUBLICATION`.** This review grants nothing beyond what the owner's
D-7 already grants. It does not publish or dispatch. The following are preserved unchanged: the
freeze, the D-7 record, the dispatcher, Trial #1 and every earlier review. This record is not
amended by anything that follows it.
