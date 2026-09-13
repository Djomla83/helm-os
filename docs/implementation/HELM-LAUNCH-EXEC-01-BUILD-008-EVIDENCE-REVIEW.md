# LAUNCH-EXEC-01 — Build 8 evidence review

**THIS REVIEWS PRESERVED BUILD 8 EVIDENCE. IT DOES NOT CREATE BUILD EVIDENCE AND DOES NOT AUTHORISE
TRIAL #3.**

**IT FIXES NOTHING, PUSHES NOTHING, RE-RUNS NOTHING AND DISPATCHES NOTHING.**

| | |
|---|---|
| Reviewed evidence commit | `b8acabfb5c3cf4193e56f2beffc3af63930a6178` — local and unpublished; GitHub answers "No commit found" for it |
| Reviewed Trial #3 freeze | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| Workflow head | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` — the [independent freeze review](HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md) |
| Formal Build 8 run / job | `34754901079` / `103717520698` |
| Workflow | `launch-exec-01 compile-only pretrial`, id `354258342`, `.github/workflows/launch-exec-01-compile-only.yml`, blob `bdf9a1f232a9f66ba05f260048543469d7b160f1` |
| Manifest | `SOURCE-HASHES.json`, Git blob `8cd290b573408510f8c16cd8dafe676354140a38`, SHA-256 `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` |
| Remote milestone | `origin/docs/helm-launch-architecture` = `69f13a7…` after a read-only fetch |
| Pushed | NO |

**Result.**
* Every Build 8 fact about the build is confirmed against primary evidence: run and job identity, a
  single push-triggered attempt, the freeze binding, the workflow revision, the environment, the
  compile set, the link shape, all ten artefact hashes, the Build 7 delta, the Linux tests, the
  runner's refusal and the compile-only boundary.
* The preserved record is **not** accepted as written. Its run-log-archive identity cannot be
  reproduced (BR-I1), and six smaller statements are imprecise. The build itself needs no repeat.

**Reviewer independence.** This session wrote `b8acabf`, made the publication push and extracted
the Build 8 facts. The load-bearing judgements below therefore come from two fresh reviewer contexts.
Neither had seen the author's scratch files, and both worked read-only:

* one on GitHub primary evidence — the run and job API, freshly downloaded logs, and Build 7's log;
* one on Git — commit scope, freeze binding, workflow revision, source delta and authority.

This session ran the mechanical validations (§21) and re-downloaded both logs itself to confirm the
digest result (§5). It consulted its own copy of the archive, downloaded right after the run, only to
explain the mismatch and never for a verdict.

Log line numbers below (L) refer to the job log after removing its UTF-8 BOM and each line's
timestamp. Record line numbers (R) refer to `docs/experiments/launch-exec-01/BUILD-EVIDENCE.md` at
`b8acabf`.

## 1. Starting state

| Check | Result |
|---|---|
| `git status --short` | clean |
| branch | `docs/helm-launch-architecture` |
| HEAD | `b8acabfb5c3cf4193e56f2beffc3af63930a6178` |
| remote branch after a read-only fetch, and `git ls-remote` | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| commits in `69f13a7..b8acabf` | exactly one, `b8acabf`, whose only parent is `69f13a7` |

## 2. Build evidence commit scope

* **Scope.** `git show --stat --summary b8acabf` lists one file, 152 insertions, no creation, deletion
  or mode change. `--name-status` gives only `M docs/experiments/launch-exec-01/BUILD-EVIDENCE.md`,
  and the numstat is `152 0`.
* **Append-only at the byte level.**
  * At `69f13a7` the file is 36,424 bytes (blob `c6b8d7c9e3feb752a457a1122b17eb6b3782a910`, SHA-256
    `1621b9e76839d6652d31a160da3b9655cd51f34f1bdb16506f29c86493c98c08`).
  * Those bytes are an exact prefix of the 47,812 bytes at `b8acabf` (blob
    `e7d0c8072695dafd30ba1d14525663b772d88021`, SHA-256
    `30b67c90e0f2d86dc953871cbeaec465abcd5418efa16f7993a8b4b99554c57e`).
  * The appended 11,388 bytes are 152 LF lines of UTF-8 with no CR and no trailing whitespace. No
    historical build section was edited or removed.
* **Nothing else changed.** The manifest, the 17 sources, the 3 definitions, `.github`, `tools`,
  `docs/experiments/evidence`, `docs/DECISIONS.md` and `docs/PROJECT_STATE.md` have identical object
  ids at `69f13a7` and `b8acabf`. There is no D-7 change and no dispatcher.
* **Attribution.** Author and committer are the owner's Git identity. The commit has no trailers and
  no AI attribution.

**BUILD8_EVIDENCE_COMMIT_SCOPE_SOUND**

## 3. Independent GitHub run identity

From `gh api` on `actions/runs/34754901079`, `…/runs/34754901079/jobs`,
`…/runs/34754901079/attempts/1` and `actions/jobs/103717520698`:

| Field | Value |
|---|---|
| workflow | `launch-exec-01 compile-only pretrial`, id `354258342`, path `.github/workflows/launch-exec-01-compile-only.yml` |
| event / branch / head | `push` / `docs/helm-launch-architecture` / `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| run number / attempt | 11 / 1; `previous_attempt_url` null |
| status / conclusion | `completed` / `success` |
| created_at / run_started_at / updated_at | 2026-09-13T11:37:02Z / 11:37:02Z / 11:38:22Z (the `attempts/1` view reports 11:38:23Z) |
| job | `103717520698` (`compile-only`), run `34754901079`, attempt 1, the only job; `completed` / `success`; 11:37:05Z to 11:38:22Z |
| steps | 1–15, 30 and 31 all `success`; freeze verification (step 4) at 11:37:08–09Z; the first compile (step 6) at 11:37:10Z; the tests (step 14) at 11:37:13Z–11:38:19Z; the refusal (step 15) at 11:38:19–21Z |

**BUILD8_RUN_IDENTITY_SOUND**

## 4. Build 8 uniqueness and no retry

* **Compile-only history.** The workflow has 11 runs in total, all `push` and all attempt 1. There has
  never been a `workflow_dispatch` run, and the only run created since 2026-09-12 is `34754901079`.
* **Runs for head `69f13a7`.** There are exactly two:
  * `34754901079`, the compile-only run;
  * `34754901075`, "HELM Rust workspace Linux" (`helm-evidence.yml`, workflow `352797925`, run 31,
    push, attempt 1, success at 11:41:31Z). Its `tools/tests/**` path filter triggered it. It is
    ordinary CI and not a Build 8 attempt.
* **No other runs.** No other run was created in any workflow since 2026-09-12.
* **No retry.** `…/runs/34754901079/attempts/2` returns HTTP 404, and `jobs?filter=all` lists one job.
* **The push.** GitHub's activity record `43436477647` is a push to
  `refs/heads/docs/helm-launch-architecture`, before `1f0b86a13ece63badf28d7cef97d580ebc359a99`,
  after `69f13a7`, at 2026-09-13T11:37:00Z, and it is a fast-forward.

**BUILD8_SINGLE_FORMAL_ATTEMPT_PROVEN**

## 5. Fresh log download

Each form was downloaded binary-safely several times: by the primary-evidence reviewer at about
14:10Z and at 14:25:12Z, and by this session at 14:38:25Z. Repeated downloads of the same form were
byte-identical.

| Form | Record (R728, R729) | Served on 2026-09-13 from 14:10Z | Full-string equal |
|---|---|---|---|
| Job log, `…/actions/jobs/103717520698/logs` | 182,255 bytes; SHA-256 `6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15` | 182,255 bytes; 1,357 LF bytes, last byte LF, no CR; SHA-256 `6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15` | **yes** |
| Run log archive, `…/actions/runs/34754901079/logs` | 77,376 bytes; 19 entries; SHA-256 `f23db84bd3e4badcaa210a23f4239cf5f0dbb54b289db0accf37fc230a07f2af` | 36,023 bytes; 2 entries; SHA-256 `0eb52f24c236bcee4a77ed95163faa6d095f4a613a5dc8f8f1641b04b9389b03` | **no** |

The served archive holds two entries:
* `0_compile-only.txt` — 182,255 bytes, byte-identical to the job log;
* `compile-only/system.txt` — 621 bytes, SHA-256
  `ed81cd88ebcefbcad17a79052e7a242deaddce459c07f3731938b592bfc4c3d1`.

The author's retained copy of the archive, downloaded shortly after the run, is the recorded 77,376
bytes, 19 entries and `f23db84b…`. Its `0_compile-only.txt` is byte-identical to today's job log. The
exact time of that download was not recorded.

The record therefore hashed exactly what GitHub served then. GitHub has since regenerated the archive
without its per-step entries. The build log content is unchanged, but the recorded archive identity
can no longer be re-verified, and the archive is not a durable identity. GitHub still serves both
forms, so retention did not limit this review.

**BUILD8_LOG_DIGEST_MISMATCH** — the run-log-archive identity. The job-log digest matches exactly.

## 6. Raw log repository preservation

**RAW_BUILD8_LOGS_REPO_PRESERVATION_NOT_REQUIRED**

* **The record already carries every load-bearing fact, and all were confirmed against the log:**
  * immutable run, job and head identity;
  * the checkout SHA and `freeze_verified: true`;
  * the environment identity;
  * the exact compile commands and their success;
  * link and marker facts;
  * all ten full SHA-256 values;
  * the test result;
  * the refusal text and exit code;
  * the zero-D-7, zero-case and zero-execution statements;
  * the job-log digest.
* **The negative claims do not depend on the log bytes.** The workflow, the tests and the runner at
  `69f13a7` are in Git, and each step's conclusion stays in GitHub run metadata.
* **Committed raw bytes would add a third copy of facts already verified, not a missing load-bearing
  fact.** The raw log also names the runner VM host, worker and region, which the project's P-16
  host-identity rule avoids publishing.
* **Archive bytes would not help.** GitHub regenerates the archive, so its bytes are not stable
  (§5).

This conclusion assumes BR-I1 is corrected, so that the record names the job-log digest as its only
durable log identity.

## 7. Freeze binding

| # | Requirement | Evidence | Holds |
|---|---|---|---|
| 1 | `bebd8a5` is in the publication chain | `bebd8a5` is the only parent of `69f13a7`, and an ancestor of `b8acabf` | yes |
| 2 | only a docs-only review separates `bebd8a5` from `69f13a7` | `git diff --name-status bebd8a5 69f13a7`: `A docs/implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md` alone | yes |
| 3 | `SOURCE-HASHES.json` byte-identical | identical bytes at `bebd8a5`, `69f13a7` and `b8acabf` | yes |
| 4 | 17 frozen sources byte-identical | 17 / 17: identical blobs, and every SHA-256 at `69f13a7` equals the manifest | yes |
| 5 | 3 definitions byte-identical | 3 / 3: identical blobs, and every SHA-256 equals the manifest | yes |
| 6 | manifest blob at the workflow head | computed `8cd290b573408510f8c16cd8dafe676354140a38`, equal to `git rev-parse 69f13a7:…` | yes |
| 7 | manifest SHA-256 | `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` | yes |
| 8 | `freeze_verified: true` before compilation | L173–176 in step 4 (11:37:08–09Z); the first `cc` is at L269 in step 6 (11:37:10Z); the checkout printed `69f13a7` at L98 and L110 | yes |
| 9 | trial identity | manifest `trial` `trial-003`; `status` NOT_RUN; `valid_trial_count` 0; `d7_execution_authorised` false | yes |

The experiment directory tree (`80661518320ffdf8419552dcca66a9f5c91d9d20`), the `.github/workflows`
tree and the `tools/tests` tree are identical at `bebd8a5` and `69f13a7`.

**BUILD8_BINDS_BEBD8A5_TRIAL3_FREEZE**

## 8. Workflow revision

* **Unchanged revision.** `.github/workflows/launch-exec-01-compile-only.yml` is blob
  `bdf9a1f232a9f66ba05f260048543469d7b160f1` at `69f13a7`, `bebd8a5`, `1f0b86a`, `ba41a3f` and
  `b8acabf`, and at Build 7's commit `f417984`. It last changed in `a7ae6b9` (2026-09-09), so Build 7
  and Build 8 ran the same revision, and the publication changed nothing under `.github`.
* **Triggers.** `workflow_dispatch`, and `push` to `docs/helm-launch-architecture` on
  `docs/experiments/launch-exec-01/**`, `tools/tests/test_launch_exec_01.py` or the workflow file.
  Permissions are `contents: read`, and concurrency never cancels a running job.
* **Steps, in order.**
  1. checkout with full history;
  2. environment inventory;
  3. `run_launch_exec_01.py --verify-freeze`;
  4. preflight probes;
  5. a static-link probe on `/tmp/probe`;
  6. compile the static targets;
  7. compile the dynamic helper;
  8. `make_fixtures.py build`;
  9. `file` and `readelf` on the binaries;
  10. PT_INTERP assertions;
  11. the `HELM-MARK` grep;
  12. `sha256sum`;
  13. `unittest discover`;
  14. the runner with no arguments, requiring exit 3.
* **Nothing executes a built binary or a trial.** Freeze verification comes before every compile.
  Built binaries are only read as data, no step passes the D-7 flag, and no trial step exists. The
  inventory, preflight and probe steps run system tools, a `clone3` probe with rejected arguments,
  and `ldd` on `/tmp/probe` only.

**BUILD8_WORKFLOW_REVISION_SOUND**

## 9. Execution boundary

From the actual job log:

* **Commands.**
  * The `set -x` command words are exactly `cat`, `cc`, `command`, `echo`, `file`, `for`, `getconf`,
    `grep`, `head`, `id`, `ldd`, `mkdir`, `printf`, `readelf`, `sha256sum`, `sort`, `true` and
    `uname`. The only `[command]` word is `/usr/bin/git`.
  * No `build/*` path is ever a command word. `build/` paths appear only as operands or text: `cc -o`
    outputs, `for` lists, `echo` text, `file`, `readelf`, `grep -c` and `sha256sum` operands, the
    fixture JSON, and the echoed step scripts.
* **`ldd`** targets only `--version` and `/tmp/probe`.
* **Flags.** `--i-have-owner-authorisation-d7`, `--post-pin-control-fd`, `--parent-fd-set-cloexec`,
  `--parent-close-low-fds`, `--fixture-health-fifo`, `--liveness-fifo` and `--bypass-admission` occur
  0 times.
* **Test code at `69f13a7`.**
  * `Authorisation(` appears in the runner behind the D-7 flag check, in review prose, and in a
    negative test inside `assertRaises(PermissionError)` that creates no instance. Guards against
    `Authorisation(True)` pass in all five launch-exec suites.
  * No test references the workflow's `build` directory or executes a build output.
  * The compiled-image E6 test compiles `helper_report.c` into a temporary directory, reads the image
    with `read_bytes()`, checks its bytes in memory, deletes the directory, and never executes it.

| Boundary fact | Value |
|---|---|
| experimental LAUNCH-EXEC ELF executions | ZERO |
| cases posed | ZERO |
| D-7 | NOT USED |
| runner invoked without authority | exit 3 |

**BUILD8_COMPILE_ONLY_BOUNDARY_SOUND**

## 10. Environment

From the fresh job log. The runner's host name is deliberately not copied here.

| Fact | Log value |
|---|---|
| runner / image | runner `2.337.0`; image `ubuntu-24.04`, version `20260907.300.1` (L16–17) |
| OS | `Ubuntu` / `24.04.5` / `LTS` (L11–13); `PRETTY_NAME="Ubuntu 24.04.5 LTS"` |
| kernel / arch | `6.17.0-1022-azure`, `x86_64` (L127, L129) |
| uid | `1001` |
| compiler | `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` |
| glibc | `ldd (Ubuntu GLIBC 2.39-0ubuntu8.8) 2.39` (L155) |
| page size | 4096 |
| NoNewPrivs / Seccomp / Seccomp_filters | 0 / 0 / 0 |
| ptrace_scope | 1 |
| strace | `/usr/bin/strace`, `strace -- version 6.8` |
| binfmt_misc | `enabled`: `llvm-16`, `llvm-17` and `llvm-18` runtimes (magic `4243`) and `python3.12` (magic `cb0d0d0a`) |
| preflight | `clone3` available (`EINVAL`, rejected arguments); `noexec_writable` `/run/lock`, `/dev/mqueue`; `parent_no_new_privs` 0; block reason `unprivileged_runner` only |

L154 and L156 are `/usr/bin/ldd: … printf: write error: Broken pipe`. They come from
`ldd --version | head -1` in the inventory step: pipe noise, not a compile failure.

**BUILD8_ENVIRONMENT_RECORD_SOUND**

## 11. Compile commands

The exact `set -x` lines from the log:

```
cc -O2 -Wall -Wextra -static -o build/helper_report helper_report.c                  (L288)
cc -O2 -Wall -Wextra -static -o build/helper_alt helper_alt.c                        (L290)
cc -O2 -Wall -Wextra -static -o build/helper_fork helper_fork.c                      (L292)
cc -O2 -Wall -Wextra -static -o build/helper_setid helper_setid.c                    (L294)
cc -O2 -Wall -Wextra -static -pthread -o build/launcher_spike launcher_spike.c       (L295)
cc -O2 -Wall -Wextra -o build/helper_dynamic helper_dynamic.c                        (L301)
```

* **All six succeeded.** Both compile steps concluded `success`, and neither contains any output line
  other than its `set -x` lines, so there is no compiler diagnostic.
* **No diagnostics elsewhere either.** Of the 8 lines anywhere in the log that contain "warning" or
  "error":
  * one is a git `hint:` line (L64);
  * two are the `ldd` pipe lines;
  * five are test names and one a test docstring, all passing;
  * one is GitHub's `##[warning]` Node.js 20 deprecation notice for `actions/checkout@v4` (L1357).
* **No `##[error]` line exists.** Only two lines contain `error:`, the `ldd` lines.

**BUILD8_COMPILE_SET_SOUND**

## 12. Static and dynamic link shape

| Binary | `file` (log) | PT_INTERP |
|---|---|---|
| `helper_report` | ELF 64-bit LSB executable, x86-64, statically linked (L326) | none (L330) |
| `helper_alt` | ELF 64-bit LSB executable, x86-64, statically linked (L335) | none (L339) |
| `helper_fork` | ELF 64-bit LSB executable, x86-64, statically linked (L344) | none (L348) |
| `helper_setid` | ELF 64-bit LSB executable, x86-64, statically linked (L353) | none (L357) |
| `launcher_spike` | ELF 64-bit LSB executable, x86-64, statically linked (L362) | none (L366) |
| `helper_dynamic` | ELF 64-bit LSB pie executable, x86-64, dynamically linked, interpreter `/lib64/ld-linux-x86-64.so.2` (L371) | `INTERP` and `[Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]` (L374–375) |

The PT_INTERP assertion step printed nothing and succeeded. The marker step printed `marker guard
present` (L399).

**BUILD8_LINK_AND_MARKER_EVIDENCE_SOUND**

## 13. Build 8 hash set

The complete mapping printed at L407–416 is shown below.

| File | SHA-256 |
|---|---|
| `build/helper_alt` | `d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d` |
| `build/helper_dynamic` | `c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc` |
| `build/helper_foreign.elf` | `415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52` |
| `build/helper_fork` | `b0c9c01f7cf5d514c15ee0e94027f38de41ef63e9eb675227773b2eb94c4fe44` |
| `build/helper_report` | `f04323e1061c753192ffc0959a2f9094ad7d89a8ceecb8c815785a6f78b2c5be` |
| `build/helper_setid` | `500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10` |
| `build/launcher_spike` | `2f5cf10a0b1375e04657e5da7cf7ad2bba7f1da814404a10ab11ffaa15bf55eb` |
| `build/magic_only.bin` | `3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359` |
| `build/script_fixture.sh` | `37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a` |
| `build/unloadable_in_cohort.elf` | `51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9` |

The log names exactly these ten files. They equal the owner's expected mapping and the record's
artefact table (R671–680) as full strings, 10 of 10.

**BUILD8_ARTIFACT_HASH_SET_EXACT**

## 14. Build 7 comparison

**Build 7's own log is still served.** Run `34575558065`, job `103187099980`: 140,875 bytes, SHA-256
`81bad5dff1d2422d77c902b1230a29f76ce518be6740ff86cdc7771ec0335979`, identical on two downloads, same
runner and image. Its mapping equals Build 7's table in BUILD-EVIDENCE.md, 10 of 10 full strings.

| Output | Build 7 | Build 8 | Changed |
|---|---|---|---|
| `helper_fork` | `3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb` | `b0c9c01f7cf5d514c15ee0e94027f38de41ef63e9eb675227773b2eb94c4fe44` | yes |
| `helper_report` | `423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236` | `f04323e1061c753192ffc0959a2f9094ad7d89a8ceecb8c815785a6f78b2c5be` | yes |
| `launcher_spike` | `4d42212f3b4d45ca46e415961e36e8ee4d45090a15fb3b77a6f8971e661a6546` | `2f5cf10a0b1375e04657e5da7cf7ad2bba7f1da814404a10ab11ffaa15bf55eb` | yes |
| `helper_alt`, `helper_dynamic`, `helper_setid`, `helper_foreign.elf`, `magic_only.bin`, `script_fixture.sh`, `unloadable_in_cohort.elf` | as in §13 | identical full hashes | no |

* **The C sources behind that delta.** Build 7 compiled the C bytes of `f417984`, which equal
  `ba41a3f`'s and both "still binds" tables. Against the manifest at `69f13a7`, exactly
  `launcher_spike.c`, `helper_report.c` and `helper_fork.c` changed. `helper_alt.c`,
  `helper_dynamic.c` and `helper_setid.c` did not. The manifest's `build` block lists the same sets.
* **Fixtures.** `make_fixtures.py` changed only in its declared modes. Imported in isolation from both
  revisions, its four fixture byte strings are identical. The hashes, not source identity, show that
  the unchanged outputs did not move under the same runner image.

**BUILD8_BUILD7_DELTA_CORRESPONDS_TO_CHANGED_C_SOURCES**

## 15. Python test evidence

* `Ran 783 tests in 64.334s` (L1286) is followed by `OK` (L1288) with no skip count, and 783 result
  lines end in `... ok`.
* "skipped" appears only in a test name (L987) and a docstring (L992). No FAIL, ERROR or traceback
  appears.
* The Trial #3 compiled-image test, `test_a_compiled_scratch_helper_has_one_region_in_one_symbol`,
  executed and passed: L1158 ends in `... ok`.

**BUILD8_LINUX_TEST_EVIDENCE_SOUND**

## 16. Runner refusal

* **Step 15's script:** `set +e`, `python3 run_launch_exec_01.py`, `code=$?`, `set -e`,
  `echo "runner exit code: $code (3 = refused, D-7 not granted)"`, `test "$code" -eq 3`.
* **Output:**
  * `"status": "NOT_RUN"` (L1338);
  * `"reason": "D-7 (execution authorisation) is not granted. This definition is frozen for
    independent pre-trial review, not for execution. No case was posed."` (L1337);
  * `runner exit code: 3 (3 = refused, D-7 not granted)` (L1340).
* **Conclusion:** the step succeeded. No D-7 flag or authorisation appears anywhere before it (§9).

**BUILD8_RUNNER_REFUSAL_SOUND**

## 17. BUILD-EVIDENCE.md fidelity

The Build 8 section (R598–748) was compared claim by claim with the primary evidence above.

* **Confirmed.** Every field below matches the primary evidence:
  * the build number, freeze, commit built, manifest blob and SHA-256, and the 21-of-21 byte identity;
  * the publication time and `main` head;
  * run id, run number, attempt, event, branch, head and workflow;
  * the job and its times and conclusions, and "every step succeeded";
  * uniqueness, including the Rust workspace run and its success;
  * the source binding;
  * runner, image, OS, kernel, compiler and C library;
  * the three command rows and the fixtures row, and "Compiler diagnostics: none";
  * the single `##[warning]` annotation;
  * the sources-compiled table and the environment table;
  * all ten artefact hashes, their classifications and the Build 7 prior values;
  * the linkage, sizes and modes bullets;
  * the test paragraph and the refusal wording;
  * the `set -x` verb list, the `ldd` target and the zero-flag counts;
  * the job-log byte count and SHA-256, and the not-committed statement;
  * the classification line and the authority lines.
* **Defect (BR-I1).** The run-log-archive row (R729) and the claim that the digests let a reviewer
  verify a fresh download (R734–735) do not hold for the archive (§5).
* **Imprecise (BR-M1 to BR-M6):**
  * R730, the stability claim;
  * R632, "only two `error` strings";
  * R608, "No binary produced here was … `ldd`'d";
  * R728, "1,358 lines";
  * R717, the list of places `build/` paths appear;
  * R732, the statement of provenance.

Every build fact is right. A record whose own verification mechanism fails for one of its two
declared log identities is nonetheless materially overstated, so the record needs correcting.

**BUILD8_PRESERVED_RECORD_FIDELITY_DEFECT**

## 18. Post-record validation wording

* **The record claims no local runs.** Neither the Build 8 section nor the commit message of
  `b8acabf` says anything about local Windows or WSL runs, a rewrap, or a re-run test module; a search
  finds no "Windows" or "WSL". Every test statement in the record describes the GitHub job, and it
  does not claim a full Windows suite after the rewrap.
* **The validation gap is now closed anyway.** This review ran the full Windows suite on the committed
  bytes of `b8acabf`: 783 tests OK, 74 skipped, all POSIX-only (§21).

**BUILD8_POST_RECORD_VALIDATION_WORDING_SOUND**

## 19. Historical and authority non-regression

* **`b8acabf` changes none of the following:** Trial #2's evidence or history, the Trial #3 freeze
  bytes, `SOURCE-HASHES.json`, any workflow, any source, `DECISIONS.md` or `PROJECT_STATE.md`.
* **The new section's authority statements match the manifest and definition §10.12.** They say
  the freeze is published, Build 8 is "**not** D-7", Trial #3 is NOT_RUN, its D-7 is not authorised,
  its valid count is zero, and it must not be executed. It claims only the publication-and-Build-8
  step.
* **No Trial #3 dispatcher exists anywhere:**
  * not in 15 refs, including 10 local tool checkpoint refs that are not on the remote;
  * not in GitHub's five workflows;
  * no `trial-003` or `RUN-TRIAL-003` string under `.github` in any ref.
* **The Trial #2 dispatcher still checks out literal `ba41a3f` and gates on the Trial #2 manifest.**

**BUILD8_AUTHORITY_NONREGRESSION_SOUND**

## 20. Raw log retention risk

GitHub raw logs are retention-limited. The repository preserves their digests but not their bytes,
as for Builds 1 to 7.

* **The job log is verified.** The job-log digest was freshly verified as a full string while GitHub
  still serves it.
* **Every fact is transcribed and confirmed.** Every load-bearing Build 8 fact is transcribed in the
  record and confirmed in §§3–16.
* **The archive digest is the exception.** It is not reproducible even within retention (BR-I1), and
  committing archive bytes would not repair that.

Long-term auditability therefore does not depend on the raw bytes, provided the record is corrected
to rely on the job-log digest alone.

**RAW_BUILD8_LOGS_REPO_PRESERVATION_NOT_REQUIRED**

## 21. Validation

| Check | Result |
|---|---|
| `git diff --check`, worktree and `69f13a7..b8acabf` | clean |
| `python tools/validate_docs.py` at `b8acabf` | PASS (124 Markdown files, 251 JSON files, 1,289 link targets) |
| Windows 11, Python 3.14.3: `python -m unittest discover -s tools/tests` at `b8acabf` | **783 tests OK, 74 skipped** (POSIX-only) |
| WSL Ubuntu 24.04.4, Python 3.12.3, same suite at `b8acabf` | **783 tests OK, 1 skipped** — the compiled-image E6 test, no C compiler on `PATH` |
| `cargo fmt --check` | PASS |
| `run_launch_exec_01.py --verify-freeze` at `b8acabf` | `freeze_verified: true` |
| independent audits | GitHub run, jobs, attempts and activity API; two fresh log forms, each downloaded several times; Build 7's log; Git byte identity, scope and prefix proofs; workflow revision; source and fixture delta; authority search |

Only read-only Git and GitHub operations, hashing, static inspection and isolated reads of Git bytes
were used. Nothing was compiled, dispatched, re-run, cancelled or pushed.

| Boundary | Count |
|---|---|
| experimental LAUNCH-EXEC ELF executions | **ZERO** |
| cases posed | **ZERO** |
| new Build 8 attempts | **ZERO** |
| manual dispatches | **ZERO** |

## 22. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| BR-I1 | IMPORTANT | Log identity (R729, R734–735) | Yes, for any future verification of the archive | The recorded archive (77,376 bytes, 19 entries, `f23db84b…`) is no longer what GitHub serves (36,023 bytes, 2 entries, `0eb52f24…`), so the record's promise that its digests verify a fresh download fails for the archive. The job-log digest matches, and the archive's combined log is byte-identical to the job log, so no build fact is affected | Correct the record without a rebuild. Append a dated correction that names the job-log SHA-256 as the only durable log identity and records both archive observations as regenerated and non-durable. Then one bounded re-review of that correction |
| BR-M1 | MINOR | Stability claim (R730) | Yes | "A second download of both files was byte-identical" held for the author's downloads right after the run, but reads as a durable property the archive does not have | Fold into the BR-I1 correction |
| BR-M2 | MINOR | "error" strings (R632) | Yes, by search | "The log contains only two `error` strings": "error" occurs 13 times on 8 lines (two `ldd` lines, five test names, one docstring); only two lines contain `error:`. Build 7's section uses the same wording | Correct the wording; the substance (no compiler error) holds |
| BR-M3 | MINOR | Execution banner (R608) | Yes | "No binary produced here was executed, traced, `ldd`'d or loaded": the workflow's `/tmp/probe` was `ldd`'d (L272), as R665 and R716 say. The banner is true of every experiment binary, and Builds 4 and 5 carry the same banner | Correct to "No experiment binary …" |
| BR-M4 | MINOR | Line count (R728) | Yes | "1,358 lines": the job log has 1,357 LF-terminated lines; 1,358 is the element count of a split on LF | Correct, stating the definition |
| BR-M5 | MINOR | `build/` paths (R717) | Yes | The list of places `build/` paths appear is incomplete: they are also `cc -o` outputs, `echo` text, the `grep -c` operand, fixture JSON and step-script text. "Never as a command" holds | Correct the wording |
| BR-M6 | MINOR | Provenance (R732) | Yes | "Every fact in this section was read from that log or from Git": run, job and step metadata, uniqueness, the Rust run, the push time and the `main` head came from the GitHub REST API. The values are right | Correct the provenance sentence |
| BR-B1 | BACKLOG_NONBLOCKING | Earlier closing sentence (R605–606) | No | Only "no Trial #3 freeze exists" in the 2026-09-12 sentence is out of date; its Trial #2 D-7 and no-authorisation parts still hold | Optional wording in the correction |
| BR-B2 | BACKLOG_NONBLOCKING | Next publication | Yes, when `b8acabf` or a correction is pushed | The compile-only push filter covers `docs/experiments/launch-exec-01/**`. Publishing a BUILD-EVIDENCE.md change will start compile-only run 12 automatically, as such pushes started runs 3 and 6 | The owner classifies that run in advance as ordinary CI, neither Build 8 nor a retry |
| BR-B3 | BACKLOG_NONBLOCKING | State fields | No | PROJECT_STATE.md still says the freeze is not published, and the immutable manifest says `published: false` and `independent_freeze_review: "REQUIRED"` (freeze review FR-B3, FR-B4) | A later authorised state record |
| BR-B4 | BACKLOG_NONBLOCKING | Log-visible binding | No | The compile-only log shows the freeze binding through `freeze_verified: true` and the checkout SHA, not by printing the manifest digest | Optionally print `sha256sum SOURCE-HASHES.json` in a future workflow revision |

No BLOCKER. One IMPORTANT (BR-I1), limited to the preserved record's log identity. None of the
owner's other BLOCKER or IMPORTANT conditions holds:
* the run and job are right;
* there was one formal attempt and no retry;
* the freeze binding is established;
* no compiler failure or warning is hidden;
* the target set, link shape and artefact hashes are exact;
* no experimental ELF was executed, no D-7 was used and no case was posed;
* Build 8 grants no execution authority.

The Trial #3 correction and freeze designs were not reopened.

## 23. Verdicts

| Area | Verdict |
|---|---|
| Evidence commit scope | **BUILD8_EVIDENCE_COMMIT_SCOPE_SOUND** |
| Run identity | **BUILD8_RUN_IDENTITY_SOUND** |
| Uniqueness | **BUILD8_SINGLE_FORMAL_ATTEMPT_PROVEN** |
| Fresh log digests | **BUILD8_LOG_DIGEST_MISMATCH** (archive identity; job log matches) |
| Raw logs | **RAW_BUILD8_LOGS_REPO_PRESERVATION_NOT_REQUIRED** |
| Freeze binding | **BUILD8_BINDS_BEBD8A5_TRIAL3_FREEZE** |
| Workflow revision | **BUILD8_WORKFLOW_REVISION_SOUND** |
| Execution boundary | **BUILD8_COMPILE_ONLY_BOUNDARY_SOUND** |
| Environment | **BUILD8_ENVIRONMENT_RECORD_SOUND** |
| Compile set | **BUILD8_COMPILE_SET_SOUND** |
| Link and marker | **BUILD8_LINK_AND_MARKER_EVIDENCE_SOUND** |
| Hash set | **BUILD8_ARTIFACT_HASH_SET_EXACT** |
| Build 7 delta | **BUILD8_BUILD7_DELTA_CORRESPONDS_TO_CHANGED_C_SOURCES** |
| Linux tests | **BUILD8_LINUX_TEST_EVIDENCE_SOUND** |
| Refusal | **BUILD8_RUNNER_REFUSAL_SOUND** |
| Record fidelity | **BUILD8_PRESERVED_RECORD_FIDELITY_DEFECT** |
| Post-record validation wording | **BUILD8_POST_RECORD_VALIDATION_WORDING_SOUND** |
| Authority | **BUILD8_AUTHORITY_NONREGRESSION_SOUND** |

## 24. Authority and next gate

**TRIAL #2 D-7 IS CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE.**

**TRIAL #2 MUST NOT BE RERUN.**

**TRIAL #3 FREEZE IS PUBLISHED.**

**BUILD 8 FORMAL COMPILE-ONLY EVIDENCE EXISTS.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ZERO.**

**TRIAL #3 D-7 IS NOT AUTHORISED.**

**TRIAL #3 IS NOT_RUN.**

**TRIAL #3 MUST NOT BE EXECUTED.**

There is one IMPORTANT finding (BR-I1), so:

**BUILD 8 EVIDENCE IS NOT ACCEPTED FOR TRIAL #3 AUTHORITY.**

**Why it is not accepted.** The refusal concerns the preserved record, not the build. Every verdict
about the formal compile is SOUND.

**What unblocks it:**
1. A docs-only correction appended to BUILD-EVIDENCE.md, as its append-only rule requires. It
   addresses BR-I1, and preferably BR-M1 to BR-M6.
2. One bounded re-review of that correction.
3. An advance owner classification of the compile-only run that publishing it will trigger (BR-B2).

No new Build 8 attempt is needed or authorised by this review.

Classification: **BUILD_8_EVIDENCE_REVIEW_NEEDS_FIX**
