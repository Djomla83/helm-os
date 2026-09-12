# LAUNCH-EXEC-01 Trial #2 — evidence provenance

**This file is NOT Trial #2 evidence.**

It was written on 2026-09-12, after the trial had already executed and after its
artifact had already been published. It records where the five preserved files
came from and what was verified about them. It carries no observation, no case
status and no verdict of its own. If this document and the preserved evidence
ever disagree, the preserved evidence is authoritative and this document is
wrong.

The five files in this directory are the Trial #2 evidence. They are copied
byte-for-byte from the published GitHub Actions artifact. They were not
pretty-printed, re-serialised, sorted, re-encoded, re-generated or edited in any
way, and nothing was copied into them out of the Actions log.

## Execution that produced the evidence

| Field | Value |
| --- | --- |
| Run ID | `34640280964` |
| Job ID | `103397999925` |
| Workflow ID | `356056797` |
| Workflow name | `launch-exec-01 trial-002 authorised trial` |
| Workflow path | `.github/workflows/launch-exec-01-trial-002.yml` |
| Event | `workflow_dispatch` |
| Head branch | `main` |
| Head SHA (dispatcher publication) | `d8a6887d508be3b4bb5707c40e6433cee21411b5` |
| `run_number` | `1` |
| `run_attempt` | `1` |
| Started / completed (UTC) | 2026-09-11T19:42:28Z / 2026-09-11T19:44:07Z |
| GitHub conclusion | `success` (infrastructure status only, not the trial verdict) |
| Frozen source checked out | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` |
| Frozen manifest Git blob | `6f000fac9a48625d3c9def18e16ae7ce1b61efa8` |
| Frozen manifest SHA-256 | `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94` |
| Final pretrial review | `5da397c6a79aea7c9a626789480903d50df0b7b4` |
| Owner D-7 authorisation record | `900fe5a403eded9f148494f557fb357fea3a207e` |
| Frozen runner exit code | `0` |

## Artifact

| Field | Value |
| --- | --- |
| Artifact name | `launch-exec-01-trial-002-evidence` |
| Artifact ID | `10279663503` |
| Artifact ZIP size | 96826 bytes |
| Artifact ZIP SHA-256 | `38ee4628eb810ae021257aac78316a6ff8b23f6c6d67c646b4200037e805837c` |
| Downloaded / preserved | 2026-09-12 |

The ZIP is transport only and is deliberately not committed. The five extracted
files below are the sanctioned evidence.

## Preserved files

| File | SHA-256 |
| --- | --- |
| `preflight.json` | `4669c8490811ef5a3bd0fff812c877ee46e7d7c0a830670456e0bc0b5019a892` |
| `build-identity.json` | `cbf015a64b7b5bf0034fb6e638bbfdaaccbc9110a95236ea58f06efac45bb21b` |
| `journal.jsonl` | `451e471d68ab8bb6509eca3974ab54bcfa08b89ecf37bd99f7541dfd664a6558` |
| `evidence.json` | `c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f` |
| `runner-stdout.json` | `c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f` |

`evidence.json` and `runner-stdout.json` are byte-identical: the frozen runner
writes exactly one sanitised document to stdout, and the workflow preserved both
the file the runner wrote and the stdout it emitted, independently. Their
agreement is a cross-check, not a duplicate.

All five files are UTF-8 with LF line endings and a trailing newline, exactly as
uploaded. `.gitattributes` pins this directory as byte-exact so that no
line-ending normalisation can ever rewrite them.

## What was verified before preservation

The following were checked against the downloaded bytes, not taken on trust:

- the artifact ZIP SHA-256 recomputed locally matches the digest the
  `upload-artifact` step reported and the digest the Actions API reports;
- each of the five extracted files hashes to the value above, and those values
  match the `sha256sum` the workflow printed on the runner before upload;
- `evidence.json` and `runner-stdout.json` compare equal byte for byte;
- the artifact contains exactly these five files and nothing else — no ZIP, no
  raw `strace`, no runner stderr, no build intermediates, no PIDs or FDs;
- run `34640280964` is the only run of workflow `356056797`, and it has exactly
  one attempt.

## Sanitisation

The five files were produced through the frozen experiment's own P-14
publication boundary (`evidence.Sanitiser`). They were already sanitised when
they were uploaded. Nothing in this preservation step sanitised, filtered or
otherwise altered them.

## Related records

- Independent result review:
  [HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md](../../../implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md)
- Experiment definition:
  [LAUNCH-EXEC-01-DEFINITION.md](../../LAUNCH-EXEC-01-DEFINITION.md)
- Frozen experiment sources: [launch-exec-01/](../../launch-exec-01/)
