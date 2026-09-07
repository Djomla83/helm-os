# helm-evidence — implementation starting record

Recorded 2026-09-07 before implementation, on owner authority for this module only.

- Local HEAD: `ae3f012fb1bfd7b20018c3faf6c71a6740a041fe`.
- Fetched `origin/main`: `ae3f012fb1bfd7b20018c3faf6c71a6740a041fe`.
- `git status --porcelain=v1`: empty; starting branch `main`.
- Dedicated implementation branch: `product/helm-evidence`; no reset or history rewrite.
- Installed stable toolchain: Rust 1.95.0 (`59807616e`, 2026-04-14), Cargo 1.95.0
  (`f2d3ce0bd`, 2026-03-21), `x86_64-pc-windows-msvc`, LLVM 22.1.2.

The owner accepts A0-7ZIP as a completed experimental baseline with its overall FAIL
preserved. This authorises an offline, read-only Rust evidence verifier only, not the
Evidence Loop PoC, another subsystem, an architecture ADR, a licence, or a release.

## Existing material reviewed and reuse decisions

| Existing source | Reuse / boundary |
|---|---|
| [A0 contract](../experiments/EXP-009.md#application-baseline) | Independent, preregistered W1/W2 destinations, required steps and controls supply regression expectations. |
| [A0 report](../experiments/EXP-009-APP-BASELINE-REPORT.md) and [index](../experiments/evidence/app-baseline-execution-2026-09-07/INDEX.md) | Historical evidence is immutable; no application rerun. |
| [Structured result](../experiments/evidence/app-baseline-execution-2026-09-07/result-summary.json) | Retain FAIL, step outcomes and different guest boot identities. |
| [ZIP oracle](../../tools/app_baseline.py) and [frozen expected contents](../../tools/fixtures/exp009-7zip.json) | Consume existing oracle records and frozen content identities; do not port ZIP decompression, invoke the helper, or generate missing application outputs. |
| [Draft app-test schema](../../schemas/app-test-record.schema.json), [RFC-0001](../rfc/RFC-0001-app-evidence-model.md) | Reuse separation of identity, mandatory workflows and evidence. This application-rating draft cannot express destination/control/restart checks; leave it unchanged and define a separate narrow, unstable contract. |
| [RFC-0002](../rfc/RFC-0002-threat-model.md), [ADR-0015](../adr/ADR-0015-evidence-expiry.md), [ADR-0012](../adr/ADR-0012-public-docs-evidence.md) | Untrusted input, false-pass prevention, and separation of proposal from execution; no ADR acceptance or expiry/signing implementation. |
| [Documentation validator](../../tools/validate_docs.py) and existing tests | Retain and run existing checks; Rust verification is a separate library/CLI. |

## Bounded implementation decisions

Use COMPLETE, INCOMPLETE and INVALID, with INVALID taking precedence. Experimental
PASS/FAIL/INCONCLUSIVE/BLOCKED/NOT_RUN remains separate. Incorrect controls mean INVALID
evidence for interpretation; a fourth bundle verdict adds no necessary distinction.

Public A0 does not contain ZIP bodies. Its adapter will explicitly require recorded output
identity/location and existing independent content-verifier evidence. It must never describe
those ZIP bytes as locally rehashed. A declared local output artifact, when used by a contract,
is additionally mandatory and checked. W1's correct content cannot override its wrong location.

Prefer the maintained Bytecode Alliance cap-std filesystem capability for confined reads over
a handwritten canonicalize/check/open sequence. Use Serde/serde_json for strict data parsing
and RustCrypto SHA-256 for artifact identity. No executor, ZIP parser, network client, async
runtime, database, or general workflow engine is needed.
