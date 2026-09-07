# helm-evidence — bounded implementation review

Review date: 2026-09-08. Scope: the first read-only Rust product module authorised by
the owner. This is a review candidate, not a stable API or production release.
A0-7ZIP remains accepted as a completed experimental baseline with overall **FAIL**,
without any application rerun. No architecture ADR or licence is accepted.

## Repository and architecture

[Starting record](HELM-EVIDENCE-START.md): fetched local HEAD and origin/main both
`ae3f012fb1bfd7b20018c3faf6c71a6740a041fe`, clean `main`; work branched to
`product/helm-evidence`. No newer work was reset.

One Cargo workspace/member, [helm-evidence](../../crates/helm-evidence/README.md).
The library contains typed contract/report models, confined bounded reads,
identity/completeness rules and consumption of the existing ZIP-oracle records.
The CLI parses `verify <directory> [--json]`, invokes the library, formats a report
and returns its documented exit code. Four direct dependencies, no dev-only
dependencies. No application launcher, runtime manager or workflow executor.

The version 0.1 contract records application/experiment identity, required artifact
identities, explicit steps/statuses, expected destination, independent content
evidence, good/broken controls and before/after boot requirements. It does not
replace the draft application-test schema. COMPLETE / INCOMPLETE / INVALID describe
evidence; the original experiment vocabulary remains separate.

## Regression and synthetic checks

The [A0 fixture](../../crates/helm-evidence/tests/fixtures/a0-7zip/bundle.json) uses
22 JSON files totalling 40,366 bytes, including contract/provenance. Copies are
byte-identical to published records; projections are checked against machine fields
by the [adapter](../../tools/helm_evidence_fixture.py). No ZIP bodies, screenshots,
private provenance, installers or runtime artifacts are duplicated.

Expected and executed: overall INCOMPLETE; W1 INCOMPLETE with CONTENT_RESULT_PASS
and OUTPUT_DESTINATION failure; W2 COMPLETE within recorded-evidence scope. The
experimental FAIL and failed V1 record remain intact. Unpublished ZIPs were neither
rehashed nor content-tested again.

| Test case | Expected evidence result |
|---|---|
| Complete synthetic fixture | COMPLETE |
| Missing required artifact | INCOMPLETE |
| Wrong artifact hash | INVALID |
| Content PASS at wrong/unestablished destination | INCOMPLETE |
| Equivalent file elsewhere, required local destination missing | INCOMPLETE |
| Missing known-good or broken control | INCOMPLETE |
| Good control rejected / broken control accepted | INVALID |
| Broken control lacks a content identity defect | INVALID |
| Restart evidence or workflow boot identity missing | INCOMPLETE |
| Equal restart IDs / workflow on wrong boot | INVALID |
| Unknown schema version / mandatory contract field | INVALID |
| Unexpected unlisted optional artifact | COMPLETE |
| Missing / BLOCKED / NOT_RUN step | INCOMPLETE |
| Fully evidenced experimental or content FAIL | COMPLETE, original FAIL preserved |
| Duplicate IDs, bad oracle binding, false content PASS | INVALID |
| Traversal, absolute/URL/stream/device path | INVALID |
| Escaping file/directory/manifest symlink; dangling/internal symlink; Windows junction | INVALID |
| Directory as file; manifest/file/total resource limits | INVALID |
| Malformed bytes, JSON or record | INVALID without a malformed-input panic |
| Normal verification | No file-content or tree mutation |
| Untrusted command text / private-path markers | No execution / no raw path echo |

Tests assert separately on content and destination. A0's preregistration, independent
Python oracle and published records supply the material expected result, not the
new Rust implementation. The authoring agent is not the final release approver.

## Validation and measurements

[Validation receipt](evidence/helm-evidence-2026-09-08/validation.json) records the
actual command arrays, exit codes and output. All checks passed: formatting, Clippy
with `-D warnings`, 43 Rust tests (3 unit, 30 integration behavior/CLI, 10 security),
documentation validator, all 36 Python tests (the existing 35 plus A0 projection
provenance), fixture audit and both working/index diff whitespace checks.
No tests were skipped. The audit verifies all 153 A0 publication artifact hashes,
unchanged historical experiment/ADR/oracle files, publication paths and decoded
command streams. These are implementation/publication checks, not new application
results. The privacy scan is bounded and not a comprehensive secret detector.

[Measurements and dependency inventory](evidence/helm-evidence-2026-09-08/measurements.json)
retain all samples and per-file LOC, including exact dependency versions/licence
metadata. Toolchain: stable Rust 1.95.0 (`59807616e`, 2026-04-14), Cargo 1.95.0,
LLVM 22.1.2, `x86_64-pc-windows-msvc`; Python 3.14.3. Host: Windows 11 Pro
`10.0.26200`, AMD Ryzen 9 5950X, 16 physical/32 logical cores, 67,020,520 KiB visible
RAM. No WSL, Hyper-V, Wine or application process was started. Linux/macOS execution
remains untested.

| Baseline measure | Observed |
|---|---:|
| Rust physical LOC, no generated files | 2,051 total: 1,233 under src; 818 under tests |
| Rust nonblank, non-line-comment LOC | 1,913 total: 1,154 under src; 759 under tests |
| Direct dependencies / dev-only | 4 / 0 |
| Active target dependencies, including transitive/build | 39 |
| Lockfile dependencies, all targets | 59 |
| Default release executable size | 561,664 bytes |
| Synthetic CLI wall time, median / min / max | 7.64685 / 7.3420 / 8.1800 ms |
| A0 CLI wall time, median / min / max | 9.19760 / 8.8210 / 10.2726 ms |
| Maximum observed synthetic / A0 peak working set | 4,747,264 / 4,800,512 bytes |

Thirty measured invocations per fixture, alternating after three warmups each;
timing includes process startup, verification, JSON formatting and piped stdout.
Peak memory is Windows `K32GetProcessMemoryInfo` on the child handle, not private
heap. Cache was warm; no affinity or background-load control. LOC counts include
brace-only lines and inline comments; they exclude blank and `//`-only lines in
the second definition. These are baseline observations, not performance targets.

Reproduce with [the check collector](../../tools/check_helm_evidence.py) and
[measurement tool](../../tools/measure_helm_evidence.py):

```text
python tools/check_helm_evidence.py --output target/review-validation.json
cargo build --release --locked --offline -p helm-evidence
python tools/measure_helm_evidence.py --binary target/release/helm-evidence.exe --samples 30 --output target/review-measurements.json
```

Development failures: initial compile reported an unused import; Clippy found a
collapsible conditional. Both were fixed in source. The first Windows-junction
test rejected the escape, then failed in PowerShell cleanup. Cleanup now removes
only the junction entry with non-recursive `remove_dir`, and asserts the outside
canary remains. No failing assertion or lint requirement was weakened. One atomic
documentation patch had a context mismatch and changed no files; it was reapplied
with the correct context.

Final source review tightened total-byte accounting for partial/failed reads and
added contextual diagnostics for invalid declarations. An additional regression
rejects a local output reference whose stored path differs from the required one.
The release binary was rebuilt and the final measurements above were then taken.

## Data, permissions and limitations

Normal verification is read-only, offline and unprivileged. `target/` is ignored;
the documentation scanner excludes Cargo output as it already excludes `build/`
and `dist/`. Historical sources/evidence are untouched. New documentation is English
under ADR-0020.

The [security model](../../crates/helm-evidence/README.md#security-model) documents
containment, resource limits and reliance on cap-std/OS behavior. Unsupported:
signatures/authentication/freshness, concurrent-write snapshots, hardlink origins,
mount policy, hostile filesystems, ZIP re-verification, GUI interpretation, boot
chronology validation and general compatibility ratings. Other historical formats
need explicitly reviewed adapters. Storage paths are ASCII; archive names may be
Unicode. Stable API/schema and other platforms are unverified.

No new host privilege, configuration change, package installation or cost was
required; Cargo downloaded declared build dependencies only. No product or lab
subsystem was added beyond the verifier.

## Publication and next decision

Publication uses a normal push of the dedicated review branch; main and protection
rules are unchanged. Generated binaries, target output and private raw provenance
are excluded. The final response supplies the resulting commit and verified remote
state; this document does not claim that a future commit already exists remotely.

One recommended next bounded coding task, after owner review: add a Linux CI job
for this verifier's existing tests, with symlink-swap and special-file containment
fixtures. This extends validation of the same module. It is not started here.
