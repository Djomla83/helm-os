# LAUNCH-EXEC-01 Trial #3 — evidence provenance

**This file is NOT Trial #3 evidence.**

It was written on 2026-09-14, after the trial had already executed and after its
artifact had already been published. It records where the five preserved files
came from and what was verified about them. It carries no observation, no case
status and no verdict of its own. If this document and the preserved evidence
ever disagree, the preserved evidence is authoritative and this document is
wrong.

The five files in this directory are the Trial #3 evidence. They are copied
byte-for-byte from the published GitHub Actions artifact. They were not
pretty-printed, re-serialised, sorted, re-encoded, re-generated or edited in any
way, and nothing was copied into them out of the Actions log.

## Execution that produced the evidence

| Field | Value |
| --- | --- |
| Run ID | `34883316368` |
| Job ID | `104107679380` |
| Workflow ID | `357912663` |
| Workflow name | `launch-exec-01 trial-003 one-shot trial` |
| Workflow path | `.github/workflows/launch-exec-01-trial-003.yml` |
| Dispatcher Git blob at the head SHA | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` |
| Dispatcher SHA-256 at the head SHA | `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8` |
| Event | `workflow_dispatch` |
| Head branch | `main` |
| Head SHA (dispatcher publication) | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| `run_number` | `1` |
| `run_attempt` | `1` |
| Run started (UTC) | 2026-09-14T18:51:04Z |
| Job started / completed (UTC) | 2026-09-14T18:51:08Z / 2026-09-14T18:52:43Z |
| Trial step started / completed (UTC) | 2026-09-14T18:51:15Z / 2026-09-14T18:52:40Z |
| Runner image | `ubuntu-24.04`, version `20260907.300.1` |
| GitHub conclusion | `success` (infrastructure status only, not the trial verdict) |
| Frozen source checked out | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| Frozen manifest Git blob | `8cd290b573408510f8c16cd8dafe676354140a38` |
| Frozen manifest SHA-256 | `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` |
| Independent freeze review | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` |
| Accepted Build 8 evidence | `5a6be5959c5a9131afa1154c015369df46a5deb8` |
| Independent dispatcher review | `4f1989737631b66f8cd4ebc8f7fdfc71ec1a5165` |
| Owner D-7 authorisation record | `0c5652d8bd429aa93eafd15a3d3da44f92cfd3ee` |
| Frozen runner exit code | `0` |

## Artifact

| Field | Value |
| --- | --- |
| Artifact name | `launch-exec-01-trial-003-evidence` |
| Artifact ID | `10364580293` |
| Artifact ZIP size | 108184 bytes |
| Artifact ZIP SHA-256 | `92ee47033513761ec5947ae10e3175bb78bd0a444799cc4fb3c3f5fbb3bcaeee` |
| Created / expires (UTC) | 2026-09-14T18:52:41Z / 2026-12-13T18:51:05Z |
| Downloaded / preserved | 2026-09-14 |

The ZIP is transport only and is deliberately not committed. The five extracted
files below are the sanctioned evidence.

## Preserved files

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `preflight.json` | 3713 | `4669c8490811ef5a3bd0fff812c877ee46e7d7c0a830670456e0bc0b5019a892` |
| `build-identity.json` | 2525 | `49b51f210654f70746d30e7d937757aee0e452f524c28dde8e2ad40c6745cde5` |
| `journal.jsonl` | 221576 | `1355e4e8ea9d90d1140a01ed74fd0edb359366f84fb324b3fd27eb8ff654e5d6` |
| `evidence.json` | 372561 | `77db0165b0dc3ce1e3d6e70c695f41ef2d5359909146f8a7eece94265eae13d2` |
| `runner-stdout.json` | 372561 | `77db0165b0dc3ce1e3d6e70c695f41ef2d5359909146f8a7eece94265eae13d2` |

`evidence.json` and `runner-stdout.json` are byte-identical: the frozen runner
writes exactly one sanitised document to stdout, and the workflow preserved both
the file the runner wrote and the stdout it emitted, independently. Their
agreement is a cross-check, not a duplicate.

`preflight.json` is also byte-identical to the Trial #2
[`preflight.json`](../LAUNCH-EXEC-01-TRIAL-002-2026-09-11/preflight.json), so the two
share one Git blob, `9759a1163ee3ebb22f7240cd2ad8e669a75fda31`. The file holds only
the frozen membership summary and runner-environment facts such as architecture,
kernel release, toolchain versions, `binfmt_misc` entries and mount observations.
It has no trial identifier, run identifier or timestamp. It is not a stale copy:
no `trial-output/` file is tracked at the freeze, and the Trial #3 journal's own
`preflight` record (`n = 1`, trial `trial-003`) carries the same preflight object.

All five files are UTF-8 with LF line endings and a trailing newline, exactly as
uploaded. `.gitattributes` pins this directory as byte-exact so that no
line-ending normalisation can ever rewrite them.

## What was verified before preservation

The following were checked against the downloaded bytes, the job log and the
GitHub API, not taken on trust:

- the artifact ZIP SHA-256 recomputed locally matches the digest the
  `upload-artifact` step printed and the digest the Actions API reports, and its
  size matches both; a second download was byte-identical;
- the ZIP passes an integrity check and contains exactly these five files at its
  root and nothing else — no ZIP, no raw `strace`, no runner stderr and no build
  intermediates;
- each of the five extracted files hashes to the value above, and those values
  and byte counts match the `sha256sum` and `ls -l` the workflow printed on the
  runner before upload;
- `evidence.json` and `runner-stdout.json` compare equal byte for byte;
- run `34883316368` is the only run of workflow `357912663`; it has exactly one
  attempt (attempt 2 does not exist), one job and one artifact;
- the run's path, event `workflow_dispatch`, head branch `main`, `run_number` 1
  and `run_attempt` 1 are those the dispatcher review's post-run check requires;
- `git rev-parse 501a7fa:.github/workflows/launch-exec-01-trial-003.yml` is
  `64ce3d433a47eaae3eb28b8bb28f7300a330d762` and the file hashes to
  `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8`, as D-7
  operator invariant 9 requires; that path is the only change `501a7fa` makes
  against its parent `d8a6887`;
- the job log shows every pre-trial step passing: the one-shot guards
  (`event=workflow_dispatch ref=refs/heads/main run_number=1 run_attempt=1`), the
  checkout of `bebd8a5`, the manifest, freeze-review and Build 8 bindings, 17/17
  sources and 3/3 definitions unchanged, `freeze_verified: true`, definition
  hashes 3/3 and all eleven static checks; the trial step and the report step
  both print `frozen runner exit code: 0`;
- at preservation, `bebd8a5`, `69f13a7`, `5a6be59`, `f197386`, `4f19897` and
  `0c5652d` are reachable from `origin/docs/helm-launch-architecture`.

The log of job `104107679380` is 55101 bytes with SHA-256
`56e23b35d68e14d2c0e5d6bca2151c74b6904357bb09007e9fd019646cc55ea6`, and two
downloads were byte-identical. It is recorded for cross-reference only. It is
not Trial #3 evidence and is not committed.

## Sanitisation

The five files were produced through the frozen experiment's own P-14
publication boundary (`evidence.Sanitiser`). They were already sanitised when
they were uploaded. Nothing in this preservation step sanitised, filtered or
otherwise altered them.

## Related records

- Owner D-7 authorisation:
  [DECISIONS.md](../../../DECISIONS.md#d-7-authorised-trial-003)
- Independent dispatcher review:
  [HELM-LAUNCH-EXEC-01-TRIAL-003-DISPATCHER-REVIEW.md](../../../implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-DISPATCHER-REVIEW.md)
- Experiment definition:
  [LAUNCH-EXEC-01-DEFINITION.md](../../LAUNCH-EXEC-01-DEFINITION.md)
- Frozen experiment sources: [launch-exec-01/](../../launch-exec-01/)
- Trial #2 evidence: [LAUNCH-EXEC-01-TRIAL-002-2026-09-11/](../LAUNCH-EXEC-01-TRIAL-002-2026-09-11/)

When this file was written, no independent Trial #3 result review existed. The
result these files carry needs one before ADR-0024 or `crates/helm-launch` may
advance.
