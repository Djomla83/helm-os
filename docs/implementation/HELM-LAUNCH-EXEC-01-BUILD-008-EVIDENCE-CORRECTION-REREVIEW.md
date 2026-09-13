# LAUNCH-EXEC-01 — Build 8 evidence-correction re-review

**THIS IS A BOUNDED RE-REVIEW OF THE BUILD 8 EVIDENCE-RECORD CORRECTION. IT DOES NOT RE-RUN OR
RE-CREATE BUILD 8.**

**It fixes nothing, pushes nothing, dispatches nothing and authorises nothing.**

| | |
|---|---|
| Correction reviewed | `1cb2e05da2df098f765106cfefdd0fb534ac2e4a` — section "Build 8 evidence correction — post-review" in [BUILD-EVIDENCE.md](../experiments/launch-exec-01/BUILD-EVIDENCE.md) |
| Parent: independent evidence review | `d1dc86e14dfc68c45dc16e82301cf8fd0aac2856` — [Build 8 evidence review](HELM-LAUNCH-EXEC-01-BUILD-008-EVIDENCE-REVIEW.md) |
| Original Build 8 evidence | `b8acabfb5c3cf4193e56f2beffc3af63930a6178` |
| Trial #3 freeze / workflow head | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` / `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| Formal Build 8 | run `34754901079`, job `103717520698`, run number 11, attempt 1, event `push`, branch `docs/helm-launch-architecture`, conclusion `success` |
| Remote milestone | `origin/docs/helm-launch-architecture` = `69f13a7…` after a read-only fetch |
| Pushed | NO |

**Scope.** Only BR-I1 and BR-M1 to BR-M6 are re-reviewed, together with:
* the unchanged Build 8 result and artefact hashes;
* the raw-log policy;
* the future publication-CI preclassification;
* the authority statements;
* the correction's commit scope.

The Build 8 technical facts accepted in `d1dc86e` are not reopened.

**Reviewer independence.** This session wrote the original evidence `b8acabf`, the evidence review
`d1dc86e` and the correction `1cb2e05`. The load-bearing judgements below therefore come from a fresh
reviewer context. That reviewer had not seen the author's scratch files, worked read-only and
downloaded the job log itself. It was not allowed to run tests or tools. This session ran only those
mechanical validations (§12).

Line references: R is a line of BUILD-EVIDENCE.md at `1cb2e05`. L is a line of the freshly
downloaded job log after its UTF-8 BOM and each line's timestamp are removed.

## 1. Starting state

| Check | Result |
|---|---|
| `git status --short` | clean |
| branch / HEAD | `docs/helm-launch-architecture` / `1cb2e05da2df098f765106cfefdd0fb534ac2e4a` |
| remote branch after a read-only fetch, and `git ls-remote` | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| local chain beyond the remote | `1cb2e05` → `d1dc86e` → `b8acabf`, each with a single parent; `b8acabf`'s parent is `69f13a7` |

## 2. Correction commit scope

* **One file, appended only.** `git show --stat --summary 1cb2e05` reports one file and 176
  insertions, with no create, delete or mode lines. `--name-status` gives only
  `M docs/experiments/launch-exec-01/BUILD-EVIDENCE.md`, and the numstat is `176 0`.
* **Byte-level prefix.**
  * At `d1dc86e` the file is blob `e7d0c8072695dafd30ba1d14525663b772d88021`: 47,812 bytes,
    SHA-256 `30b67c90e0f2d86dc953871cbeaec465abcd5418efa16f7993a8b4b99554c57e`, the same as at
    `b8acabf`.
  * Those bytes are an exact prefix of the file at `1cb2e05`: blob
    `5425901add29a38cbb1a8493efa1af48a111272e`, 57,422 bytes, SHA-256
    `713d7e6f31539273e0594fa92a8a4a79f9f02c8ae86f85f98d703ec517146e16`.
  * The appended 9,610 bytes are 176 LF lines with no CR, no tabs and no trailing whitespace. No
    historical evidence line was deleted or rewritten.
* **Nothing else changed.** Across the whole tree of 537 entries only that one path differs between
  `d1dc86e` and `1cb2e05`. The unchanged ids include `SOURCE-HASHES.json` (`8cd290b5…`), the Build 8
  evidence review record, `.github`, `tools`, `PROJECT_STATE.md`, `DECISIONS.md`,
  `docs/experiments/evidence` and the three definition files. `b8acabf` and `d1dc86e` are untouched.
* **Attribution.** Author and committer are the owner's Git identity. The message has no trailers
  and no AI attribution.

**BUILD8_CORRECTION_SCOPE_SOUND**

## 3. Correction structure and provenance

* **Identifiers.** The correction names each required identifier in full: original evidence
  `b8acabf…` (R758), independent review `d1dc86e…` (R759), run `34754901079` (R760), job
  `103717520698` (R761), freeze `bebd8a5…` (R762), workflow head `69f13a7…` (R763).
* **Governing rule.** The earlier section "stays exactly as committed; where the two differ, this
  correction governs" (R752–753).
* **Explicit supersession.** Each inaccurate claim is named and superseded or withdrawn in its own
  subsection, rather than silently dropped: BR-I1 at R789–795, BR-M1 at R799, BR-M2 at R807, BR-M3 at
  R818, BR-M4 at R827, BR-M5 at R834–835 and BR-M6 at R844.
* **Quotes match the original.** Every quoted claim appears verbatim, whitespace-normalised, in the
  original section (R598–748). The one exception is cosmetic (RR-N3).

**BUILD8_CORRECTION_PROVENANCE_SOUND**

## 4. BR-I1 — durable log identity

**Fresh job log.** Two binary-safe downloads of `…/actions/jobs/103717520698/logs` were byte-identical,
182,255 bytes each, with SHA-256:

`6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15`

This equals the required value as a full string. GitHub still serves the log.

**Current run archive.** `…/actions/runs/34754901079/logs` returns 36,023 bytes with SHA-256
`0eb52f24c236bcee4a77ed95163faa6d095f4a613a5dc8f8f1641b04b9389b03` and two entries. Its
`0_compile-only.txt` is byte-identical to the job log. This is the same archive the evidence review
observed, so it has not changed again since then.

**What the correction says.**
* The job-log SHA-256, in full, is the only durable identity (R768–771, R787).
* The archive is "**not byte-stable**" (R775–776).
* Both archive observations appear only in a historical table: `f23db84b…` with 77,376 bytes and
  19 entries, and `0eb52f24…` with 36,023 bytes and 2 entries (R778–781). Both rows equal the evidence
  review's §5 and today's download.
* "Neither archive digest is a durable identity. Both are transient download observations." (R785)
* The earlier reproducibility claim is withdrawn: "No archive digest is reproducible authority."
  (R792–795)

**BR_I1_DURABLE_LOG_IDENTITY_CORRECTION_SOUND**

## 5. BR-M1 to BR-M6

| Item | What the correction now says | Checked against | Verdict |
|---|---|---|---|
| BR-M1 | supersedes the "Stability" row; the job-log SHA-256 was independently reproduced; the archive is regenerated and not byte-stable; archive-byte equality is not Build 8 authority (R799–803) | two identical fresh job-log downloads; the changed archive | **BR_M1_CORRECTION_SOUND** |
| BR-M2 | withdraws the "error" substring count; states "no compiler warning and no compiler error occurred"; attributes the two `printf: write error: Broken pipe` lines to `ldd --version \| head -1`; calls the checkout Node.js 20 notice a platform notice (R807–814) | L152–156: `+ ldd --version`, `+ head -1` and the two pipe lines; both compile steps print only `set -x` lines (L286–295, L301) and succeed; L1357 is the only `##[warning]`, and there is no `##[error]` | **BR_M2_CORRECTION_SOUND** |
| BR-M3 | `/tmp/probe` is workflow infrastructure passed to `ldd`; no experiment binary was executed; experiment binaries were inspected only as data; no produced experimental ELF crossed the execution boundary (R818–823) | L270–274: `+ file /tmp/probe`, `+ ldd /tmp/probe`, "not a dynamic executable". None of the log's 65 `set -x` command words contains `build`, `./`, `/home/` or `/tmp/`, so `build/` appears only as an operand | **BR_M3_CORRECTION_SOUND** |
| BR-M4 | withdraws "1,358 lines" from the identity, keeps the SHA-256 authoritative, and describes 1,357 LF-terminated lines with that definition (R827–830) | 1,357 LF bytes, last byte LF, no CR; a split on LF yields 1,358 elements, as the correction explains | **BR_M4_CORRECTION_SOUND** |
| BR-M5 | no exhaustive list; build outputs appear in compile commands, as operands to inspection and hash tools, and possibly in script, test or log text; no produced experiment `build/*` ELF is invoked as a command (R834–840) | the log's operand uses of `build/`; the fixture JSON and tool output; no `build/*` command word | **BR_M5_CORRECTION_SOUND** |
| BR-M6 | four distinct source rows: GitHub Actions API metadata, the exact job log, Git history, and independent review observations; nothing reduces the evidence to "the log" or "the log and Git" (R844–852) | API run, job, attempts and head-SHA queries; job-log lines for the checkout, freeze verification, environment, commands, link shape, marker, hashes, tests and refusal; Git workflow and manifest blobs; review §5 for both archive observations | **BR_M6_CORRECTION_SOUND** (one label note, RR-N1) |

## 6. Build result non-regression

* **The correction keeps the result.** It still states **BUILD_8_FORMAL_COMPILE_ONLY_SUCCESS**,
  "unchanged", and calls run `34754901079` "the one and only formal Build 8 attempt" (R764). It also
  says "no second Build 8 exists" (R754) and "Build 8 remains run `34754901079`" (R901).
* **Its only failure wording is about future CI.** "If it fails" refers to that ordinary run, never
  to Build 8.
* **GitHub confirms the identity.** Run `34754901079` still has `run_attempt` 1, conclusion
  `success` and a null `previous_attempt_url`. `…/attempts/2` returns HTTP 404, the run has one job,
  `103717520698`, and there are no runs for `b8acabf`, `d1dc86e` or `1cb2e05`.

**BUILD8_FORMAL_RESULT_UNCHANGED**

## 7. Artefact hash non-regression

Three full-string comparisons each gave 10 of 10: the correction's hash block (R872–881), the
original artefact table (R671–680), and the sha256sum lines of the fresh job log (L407–416).

```
build/helper_alt               d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d
build/helper_dynamic           c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc
build/helper_foreign.elf       415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52
build/helper_fork              b0c9c01f7cf5d514c15ee0e94027f38de41ef63e9eb675227773b2eb94c4fe44
build/helper_report            f04323e1061c753192ffc0959a2f9094ad7d89a8ceecb8c815785a6f78b2c5be
build/helper_setid             500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10
build/launcher_spike           2f5cf10a0b1375e04657e5da7cf7ad2bba7f1da814404a10ab11ffaa15bf55eb
build/magic_only.bin           3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359
build/script_fixture.sh        37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a
build/unloadable_in_cohort.elf 51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9
```

The correction contains no other 64-hex value besides the job-log digest and the two historical
archive digests.

**BUILD8_ARTIFACT_HASH_SET_UNCHANGED**

## 8. Raw-log policy

The correction concludes **RAW_BUILD8_LOGS_REPO_PRESERVATION_NOT_REQUIRED** and gives all seven
reasons (R856–865):
* every load-bearing fact is transcribed;
* those facts were independently checked while GitHub served the logs;
* the job-log SHA-256 is retained;
* the source, the workflow and the Git history support the negative claims;
* archives are not a stable identity;
* raw logs expose host identity;
* Builds 1 to 7 follow the same convention.

The reasons hold against the evidence:
* the evidence review checked the transcribed facts;
* the log does contain a worker ID, the Azure region and the VM host name, none of which the record
  copies;
* no file at `1cb2e05` contains a raw GitHub log.

Tests are covered by "the source" rather than named separately (RR-N4).

**RAW_BUILD8_LOG_POLICY_SOUND**

## 9. Future publication-CI preclassification

The correction (R884–904) records the owner's decision. The compile-only run that publication of the
Build 8 evidence and its reviews triggers is **POST_BUILD8_EVIDENCE_PUBLICATION_CI**:

* **What it is.** Ordinary CI. It is not Build 8, not attempt 2, not a retry and not replacement
  evidence. It is not Trial #3 execution and not D-7 evidence (RR-N5).
* **How to identify it.** By `event=push`, the publication head and the compile-only workflow. It is
  never identified by a run number, never dispatched by hand and never re-run.
* **If it passes,** no new Build 8 review is required because of it.
* **If it fails,** the failure is preserved and not retried. Build 8 remains `34754901079`, work stops
  before any dispatcher or D-7 publication, and the matter returns to the owner.
* **It cannot replace Build 8.** No later ordinary run can retroactively replace or invalidate it.

The cited workflow identity is right. The API lists `354258342`, "launch-exec-01 compile-only
pretrial", `.github/workflows/launch-exec-01-compile-only.yml`, active. At `1cb2e05` its push filter
covers `docs/experiments/launch-exec-01/**`, and `BUILD-EVIDENCE.md` differs from `69f13a7`, so
publication will trigger it as predicted.

**POST_BUILD8_PUBLICATION_CI_PRECLASSIFICATION_SOUND**

## 10. Authority

* **The authority statements are intact.** All nine appear whitespace-normalised as one contiguous
  bold block (R921–924), followed by "No Trial #3 dispatcher exists."
* **Nothing grants authority.** The correction says Build 8's evidence is "**not yet accepted**"
  (R908) and makes acceptance conditional on this re-review (R917).
* **No Trial #3 dispatcher exists.** GitHub lists five workflows, none for Trial #3.
  `git grep -i -E "trial-003|RUN-TRIAL-003"` over `.github` finds no match in any branch or
  remote-tracking ref, nor in the ten local tool checkpoint refs.

**BUILD8_CORRECTION_AUTHORITY_SOUND**

## 11. Backlog boundary

* **BR-B1:** the 2026-09-12 closing sentence (R595–596) is byte-unchanged, and the correction does
  not rework it.
* **BR-B3:** `PROJECT_STATE.md` and the manifest are unchanged, and the correction touches no state
  field.
* **BR-B4:** `.github` and the workflow blob are unchanged, and nothing proposes printing the manifest
  digest.
* **BR-B2:** it appears only as "covered by the preclassification above" (R918–919).

**BUILD8_CORRECTION_SCOPE_REMAINED_BOUNDED**

## 12. Validation

This session ran these on `1cb2e05` with a clean worktree:

| Check | Result |
|---|---|
| `git diff --check`, on the worktree and over `d1dc86e..1cb2e05` | clean |
| `python tools/validate_docs.py` | PASS (125 Markdown files, 251 JSON files, 1,291 link targets) |
| Windows 11, Python 3.14.3: `python -m unittest discover -s tools/tests` | **783 tests OK, 74 skipped** (POSIX-only) |
| WSL Ubuntu 24.04.4, Python 3.12.3, same suite | **783 tests OK, 1 skipped** — the compiled-image E6 test; no C compiler on `PATH` |
| `cargo fmt --check` | PASS |
| `run_launch_exec_01.py --verify-freeze` | `freeze_verified: true` |

The fresh reviewer used only read-only Git and GitHub operations: the job log, run, attempt and
workflow APIs, blob comparisons and ref searches.

| Boundary | Count |
|---|---|
| new Build 8 attempts | **ZERO** |
| manual workflow dispatches | **ZERO** |
| experimental LAUNCH-EXEC ELF executions | **ZERO** |
| cases posed | **ZERO** |

## 13. Findings

No BLOCKER, no IMPORTANT and no MINOR finding.

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| RR-N1 | BACKLOG_NONBLOCKING | Provenance label (R849) | Yes, by reading | "the push record" is filed under "GitHub Actions API metadata" but comes from the repository activity API. The `main` head fact has no source row. The kind of source (GitHub API) is right | None needed; optionally relabel "GitHub REST API (Actions and activity)" in a later append |
| RR-N2 | BACKLOG_NONBLOCKING | Archive timing wording (R780) | Yes | "immediately after the run", where the evidence review says "shortly after the run" and that the exact time was not recorded. The observation is non-authoritative | None needed |
| RR-N3 | BACKLOG_NONBLOCKING | Quote fidelity (R834) | Yes, by raw-text search | The BR-M5 quote omits R717's bold markers; the rendered text is identical | None |
| RR-N4 | BACKLOG_NONBLOCKING | Raw-log rationale (R861–862) | Yes | Tests are not named separately as support for the negative claims; they are covered by "the source" | None |
| RR-N5 | BACKLOG_NONBLOCKING | CI wording (R893) | Yes | "not D-7 evidence" where the owner's wording is "not D-7"; the meaning is materially the same | Optional alignment later |

## 14. Verdicts

| Area | Verdict |
|---|---|
| Correction scope | **BUILD8_CORRECTION_SCOPE_SOUND** |
| Structure and provenance | **BUILD8_CORRECTION_PROVENANCE_SOUND** |
| BR-I1 | **BR_I1_DURABLE_LOG_IDENTITY_CORRECTION_SOUND** |
| BR-M1 | **BR_M1_CORRECTION_SOUND** |
| BR-M2 | **BR_M2_CORRECTION_SOUND** |
| BR-M3 | **BR_M3_CORRECTION_SOUND** |
| BR-M4 | **BR_M4_CORRECTION_SOUND** |
| BR-M5 | **BR_M5_CORRECTION_SOUND** |
| BR-M6 | **BR_M6_CORRECTION_SOUND** |
| Build result | **BUILD8_FORMAL_RESULT_UNCHANGED** |
| Artefact hashes | **BUILD8_ARTIFACT_HASH_SET_UNCHANGED** |
| Raw-log policy | **RAW_BUILD8_LOG_POLICY_SOUND** |
| Future publication CI | **POST_BUILD8_PUBLICATION_CI_PRECLASSIFICATION_SOUND** |
| Authority | **BUILD8_CORRECTION_AUTHORITY_SOUND** |
| Backlog boundary | **BUILD8_CORRECTION_SCOPE_REMAINED_BOUNDED** |

## 15. Acceptance decision

The sole durable Build 8 log identity is the job-log SHA-256
`6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15`. The run log archive is a
transient, non-authoritative download observation.

There is no BLOCKER or IMPORTANT finding, and every correction verdict is SOUND:

**BUILD 8 EVIDENCE IS ACCEPTED FOR TRIAL #3 AUTHORITY DECISIONS.**

**What acceptance does not do.** It is not D-7 and grants no execution authority, and no Trial #3
dispatcher exists. The Build 8 evidence commit, its review, the correction and this re-review are
local and unpublished.

**What publication will trigger.** Publishing them will start the preclassified
POST_BUILD8_EVIDENCE_PUBLICATION_CI. If that run fails, work stops and the matter returns to the
owner before any Trial #3 dispatcher or D-7 publication.

**TRIAL #2 D-7 IS CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE.**

**TRIAL #2 MUST NOT BE RERUN.**

**TRIAL #3 FREEZE IS PUBLISHED.**

**BUILD 8 FORMAL COMPILE-ONLY ATTEMPT SUCCEEDED.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ZERO.**

**TRIAL #3 D-7 IS NOT AUTHORISED.**

**TRIAL #3 IS NOT_RUN.**

**TRIAL #3 MUST NOT BE EXECUTED.**

Classification: **BUILD_8_EVIDENCE_CORRECTION_REREVIEW_PASSED_READY_FOR_TRIAL3_AUTHORITY_DECISIONS**
