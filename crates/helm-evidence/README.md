# helm-evidence (experimental 0.1)

`helm-evidence` is HELM's first bounded product module: a deterministic, offline,
read-only Rust library with a thin CLI. It determines whether an evidence bundle
satisfies its **declared evidence contract**, including required artifacts and their
SHA-256 identities. **It does not determine general application compatibility.**

This document and the [Rust contract types](src/model.rs) specify the implemented
contract. The schema and Rust API are experimental, not stable public interfaces.
The package is `publish = false`. No project licence or architecture ADR is accepted
by this implementation. [Starting state and reuse decisions](../../docs/implementation/HELM-EVIDENCE-START.md)
and the [implementation review](../../docs/implementation/HELM-EVIDENCE-REVIEW.md) record authority,
validation, measurements and limitations.
The [independent review](../../docs/implementation/HELM-EVIDENCE-INDEPENDENT-REVIEW.md)
records subsequent corrections and Linux validation; owner merge review remains required.

## Invocation

Requires stable Rust 1.95 or later. Dependencies are pinned in
[Cargo.lock](../../Cargo.lock); initial build dependency acquisition can use the
network, normal verification does not. No compiler or Python installation is needed
to run an already built binary.

```text
cargo build --release --locked --offline -p helm-evidence
helm-evidence verify <bundle-directory>
helm-evidence verify <bundle-directory> --json
```

From this repository on Windows, the built binary can be invoked as:

```text
target\release\helm-evidence.exe verify crates/helm-evidence/tests/fixtures/a0-7zip
target\release\helm-evidence.exe verify crates/helm-evidence/tests/fixtures/synthetic --json
```

The root is a caller-selected directory, containing `bundle.json`. Every opened
artifact path is relative to that directory. A missing root or manifest is
INCOMPLETE; unreadable/malformed input is INVALID. The library entry point is
`helm_evidence::verify(&Path) -> Report`; it returns structured checks instead of
throwing input errors or panicking. Human formatting is a separate function.

## Verdicts and exits

| Evidence verdict | Meaning | Exit |
|---|---|---:|
| COMPLETE | Every declared evidence requirement is satisfied. | 0 |
| INCOMPLETE | Required evidence, execution, boot identity or destination is absent/unestablished. | 1 |
| INVALID | Malformed/unsupported contract, bad identity, inconsistent oracle/control, unsafe path, read error or resource limit. | 2 |
| CLI usage error | Unsupported syntax; verification did not run. | 64 |
| Output error | Could not write the report, including a broken pipe. | 74 |

INVALID takes precedence over INCOMPLETE. Successes remain visible even when other
requirements fail. `sections` reports contract, artifacts, restart and each workflow
independently. A workflow section includes its own referenced evidence and restart
dependency; it does not inherit unrelated failures from another workflow. The overall
verdict includes **all** artifacts declared in the contract.

`experimental_verdict` and `recorded_steps` preserve the separate vocabulary
PASS / FAIL / INCONCLUSIVE / BLOCKED / NOT_RUN. A required step marked BLOCKED or
NOT_RUN is present as a record but has not executed, so its workflow is INCOMPLETE.
An executed FAIL or INCONCLUSIVE can be fully evidenced. A content FAIL with its
required output, valid controls and complete records can therefore yield COMPLETE
evidence while the experiment remains FAIL. No experimental outcome is recomputed
or promoted to an application rating.

No INCONCLUSIVE bundle state is necessary: an incorrect control yields INVALID
evidence for interpreting the subject, while the experiment's own INCONCLUSIVE
status remains representable. This does not regrade an experiment as an application
failure.

## Contract 0.1

The [small synthetic contract](tests/fixtures/synthetic/bundle.json) is an executable
shape example, explicitly fictional. The [A0 contract](tests/fixtures/a0-7zip/bundle.json)
is a post-experiment adapter of published records. Neither replaces the separate
[draft application-rating schema](../../schemas/app-test-record.schema.json).

All objects below deny unknown fields. JSON field order is irrelevant; array order
is retained in reports. Duplicate typed fields, IDs, case-folded artifact paths and
step IDs are rejected. References name declared artifact IDs, never URLs. Missing
or null `Option` fields mean no requirement/observation as described below; other
fields are mandatory. There is no executable expression or extension language.

| Object | Fields |
|---|---|
| Contract | `schema: "helm-evidence"`, `version: "0.1"`, `experiment`, `application`, `experimental_verdict`, `artifacts[]`, `workflows[]`, optional `restart` |
| Application | `id`, `version`, `source_sha256` (declared identity, not a claim the installer is included or rehashed) |
| Artifact | `id`, `path`, `sha256`; every listed artifact is required |
| Workflow | `id`, `phase` (`standalone`, `before_restart`, `after_restart`), `record` artifact ID, `steps[]`, optional `output` |
| Required step | `id`, nonempty `evidence[]` artifact IDs |
| Output requirement | `id`, `expected_path`, `sha256`, `content_result`, `content_manifest`, `known_good`, `deliberately_broken`, optional `local_artifact` |
| Each control | `result` artifact ID, `sha256` of the control output; known-good expects PASS, deliberately broken expects FAIL with a content identity defect |
| Restart requirement | `record` artifact ID, nonempty `evidence[]` artifact IDs |
| Workflow record | `version: "0.1"`, `workflow` ID, optional `boot_id`, `steps[]`, optional `output` observation |
| Recorded step | `id`, `status` in the experiment vocabulary |
| Output observation | `path`, `sha256` of the observed output |
| Restart record | `version: "0.1"`, `before_boot_id`, `after_boot_id` (nonempty, distinct) |

A restart requirement needs both before- and after-restart workflows in the
contract. A workflow on either side needs a boot identity matching that side of
the restart record and all declared restart evidence. A standalone workflow needs
no restart. There is exactly one optional output per workflow; this is enough for A0.

Workflow IDs cannot use the built-in report section IDs `contract`, `artifacts` or
`restart`. Content-manifest file keys must be distinct after JSON escape decoding;
conflicting or identical duplicate keys are rejected, without Unicode normalization.

Identifiers, including version labels and boot IDs, are 1–80 ASCII letters, digits,
dot, underscore or hyphen. Digests are exactly 64 lowercase hex digits. Workflow
steps, artifacts and required evidence references have explicit count bounds.

### Locations, byte identities and content results are separate

1. **Artifact exists:** a regular file was opened inside the bundle capability.
2. **Artifact identity matches:** its actual bytes match the manifest SHA-256.
3. **Output observed:** a typed workflow record declares its observed path/hash.
4. **Output location:** the recorded path equals `expected_path` exactly, with no
   normalization, case folding, search, substitution or repair.
5. **Content independently checked:** an identity-bound result from the existing
   output oracle is present. Its PASS/FAIL is recorded separately, not rerun.
6. **Required steps:** each has a record, explicit status and all evidence references.
7. **Workflow completeness:** all applicable location, identity, content evidence,
   controls, steps and restart requirements are satisfied.

`expected_path` uses the experiment's declared working-directory namespace. In
**recorded-evidence mode** (`local_artifact` absent), path/hash are historical
observations; the verifier does not claim those output bytes exist locally. Its
report explicitly includes `OUTPUT_BYTES_NOT_REQUIRED`.

If `local_artifact` is declared, that artifact's path must equal `expected_path`
and its expected hash must equal the output hash. The file must actually exist at
that path inside the bundle and be rehashed. A copy elsewhere cannot substitute.
This mode still requires the independent oracle record; hashing compressed/archive
bytes is not itself content verification. A stale location observation cannot
override a missing required local file.

### Reused oracle records

`content_result` and the control results consume the existing
[app_baseline.py](../../tools/app_baseline.py) JSON representation. Required fields:
`scope: "ZIP output-content verification only"`, `verdict` (PASS/FAIL), `errors[]`,
`files[]` (`path`, `bytes`, `sha256`), `archive_sha256`, `archive_bytes`, and
`fixture_manifest_sha256`. Original auxiliary metadata, including timestamps, is
allowed and ignored. Unknown auxiliary fields cannot add requirements or execute
behavior. A PASS must have no errors and the exact frozen file identities; FAIL
must have errors. The corrupted control must report FAIL and a differing file set
or content identity, not merely an unrelated error with all expected identities.

The frozen content manifest uses the existing `files` map of path to byte length/
SHA-256, `required_directories[]` and `allowed_directories[]`. Its artifact digest
must match every oracle record's `fixture_manifest_sha256`. Each oracle's archive
digest must match the expected output or control identity.

This module verifies recorded evidence, not ZIP contents again. It deliberately
reuses the original oracle's directory/encryption/CRC checks; those facts cannot
be independently reconstructed from the file-identity list alone. The oracle is
never executed by this module. Unicode archive-member names are supported as data;
they are never interpreted as host paths.

### Diagnostics

Every check has a stable `code`, `status` (SATISFIED / INCOMPLETE / INVALID /
NOT_REQUIRED), `workflow`, `artifact`, `requirement`, and fixed human `explanation`.
IDs identify exact failures without printing private absolute paths or raw evidence
strings. `recorded_steps` retains individual experiment statuses. Output JSON has
fixed struct-field order, declaration-order arrays, no timestamps, host paths or
timing fields; identical bytes and filesystem observations yield identical JSON.

| Codes | Meaning |
|---|---|
| `CONTRACT_FORMAT`, `SCHEMA_VERSION`, `CONTRACT_RULES` | Typed JSON, supported version, semantic contract invariants |
| `ARTIFACT_DECLARATION`, `WORKFLOW_DECLARATION`, `STEP_DECLARATION`, `OUTPUT_DECLARATION`, `LOCAL_OUTPUT_DECLARATION`, `RESTART_DECLARATION` | Identify the invalid declaration and its workflow/artifact/step requirement |
| `ARTIFACT_EXISTS`, `ARTIFACT_HASH`, `ARTIFACT_MISSING`, `ARTIFACT_UNREADABLE` | Presence, byte identity, absence, access failure |
| `PATH_UNSAFE`, `RESOURCE_LIMIT` | Containment/path/type policy or bounded resource failure |
| `EVIDENCE_REFERENCE`, `EVIDENCE_UNAVAILABLE`, `RECORD_FORMAT` | Required reference available, unavailable, or malformed |
| `RESTART_NOT_REQUIRED`, `RESTART_IDENTITIES`, `RESTART_DEPENDENCY`, `WORKFLOW_BOOT` | Restart declaration and matching boot evidence |
| `WORKFLOW_RECORD`, `STEP_PRESENT`, `STEP_EXECUTED` | Workflow/step records and executed status |
| `OUTPUT_OBSERVED`, `OUTPUT_IDENTITY`, `OUTPUT_DESTINATION`, `OUTPUT_BYTES_NOT_REQUIRED` | Separate output facts and declared byte scope |
| `CONTENT_MANIFEST`, `ORACLE_IDENTITY`, `CONTENT_VERIFIER` | Frozen expectation, oracle binding and result consistency |
| `KNOWN_GOOD_CONTROL`, `BROKEN_CONTROL` | Required controls behave as registered |
| `CONTENT_RESULT_PASS`, `CONTENT_RESULT_FAIL` | Observed content result, independent of evidence completeness |

Extra unlisted files are optional and ignored, even if malformed; the verifier does
not enumerate or discover artifacts. Unknown contract fields/versions are rejected,
so new mandatory requirements cannot be silently ignored.

## Security model

Bundles and their JSON, filenames and command text are untrusted. The only ambient
filesystem access opens the directory explicitly supplied by the caller. All
artifact reads then go through `cap_std::fs::Dir` capabilities; there is no
canonicalize-then-open fallback. cap-std uses handle-relative containment and
rejects escapes, including symlink resolution outside the root. This relies on
the upstream implementation and OS, not new unsafe Rust in HELM.

The module additionally rejects symlinks/reparse points in referenced paths,
absolute paths, traversal, backslashes, URLs, alternate streams, device names,
trailing spaces/dots and non-regular files. Stored artifact names and output
destinations use ASCII letters/digits/dot/underscore/hyphen/space, with `/`
separators; each component is nonempty and cannot be `.` or `..`. A caller may
select a root through its own symlink: that explicitly selected directory defines
the capability. Unlisted files and links are never read.

Each parent component is opened with no-follow semantics and retained as a directory
capability. Leaf opens also prohibit following links; Unix leaf opens are nonblocking
so a swapped-in FIFO cannot wait for a writer. Opened handles are checked for type
and Windows reparse attributes before reading. Metadata prechecks provide stable
diagnostics for quiescent unsafe objects; the open operations enforce the policy.

Artifact lookup uses the filesystem's case behavior (case-insensitive on the tested
Windows NTFS volume, case-sensitive on the tested Linux ext4 filesystem). A single
mis-cased stored reference may therefore exist only on Windows. Use exact stored
spelling for portable bundles. Duplicate case-folded declarations and recorded output
destination mismatches are rejected/classified identically on both platforms.

| Bound | Limit |
|---|---:|
| Contract bytes | 256 KiB |
| One artifact | 8 MiB |
| Total input bytes, including contract | 64 MiB |
| Artifacts / workflows / steps per workflow | 128 / 16 / 32 |
| Path bytes / components | 1,024 / 32 |
| Evidence references per step/restart | 128 |
| Oracle files / directories | 32 each |
| Oracle archive length / decompressed file length | 1 MiB each (existing synthetic oracle scope) |

Reads enforce limits on both metadata and actual bytes, including partial failed
reads and one extra byte to detect overflow. Serde's ordinary recursion
limit stays enabled for typed deserialization; ignored auxiliary oracle metadata
is skipped iteratively and can be deeper, within the input byte bounds.
The verifier does not execute bundle contents, fetch supplied
URLs, write into the bundle, decompress archives, interpret commands, use a shell,
or request host privileges. Library and CLI forbid authored unsafe Rust.

**Threat-model limits:** no signing, authentication, trusted timestamp, author
attestation, freshness/expiry, original-contract registry, or independent proof of
truthful capture. Someone who changes both data and declared hashes can create a
self-consistent lie. The caller/reviewer must select the intended contract; a
smaller contract does not establish a larger workflow. A0's provenance is separately
checked against its publication and preregistration by development tests.

Use a quiescent local bundle. This is not an atomic snapshot of concurrent file
changes, a hardlink-origin detector, a mount-point/hostile-filesystem sandbox or
a guaranteed wall-clock deadline. Concurrent mutation can cause rejection or an
inconsistent snapshot; confinement relies on cap-std even if a path changes between
checks. An opened directory remains the same capability after a rename, including
when it is detached from its original pathname; this is not live pathname ancestry
verification. Hashes identify bytes actually read, not the precheck's inode or a
whole-tree snapshot. No claim covers a compromised kernel, hostile filesystem implementation,
resource exhaustion outside the documented bounds, or a comprehensive security audit.
The independent review records the executed Windows and Ubuntu platforms.

## A0-7ZIP regression and history

[Fixture provenance](tests/fixtures/a0-7zip/provenance.json) identifies byte-for-byte
copies and explicit field projections from publication
`ae3f012fb1bfd7b20018c3faf6c71a6740a041fe`. The fixture is about 40 KiB and copies no
ZIP bodies, screenshots, installers, runtime packages or private provenance.
[The adapter](../../tools/helm_evidence_fixture.py) checks source publication hashes
and exact copies/projections. It is development tooling, not part of the verifier.

W1's observed path/hash come from the structured deviation; its required path comes
from the frozen protocol. W2's observed path comes from the original verifier
command's typed `argv[3]`, its hash from the result. Step/boot identities are projected
from the structured summary. No location is inferred by interpreting prose or
FileNotFoundError text. The original missing-destination record remains included.

Expected result: evidence INCOMPLETE; W1 INCOMPLETE despite CONTENT_RESULT_PASS;
W2 COMPLETE **within recorded-evidence scope**; original experimental FAIL preserved.
W2 success does not complete the two-workflow contract. Missing private ZIP bodies
are not silently invented or described as rehashed. If a future contract requires
them as local artifacts, they must be supplied or classified INCOMPLETE.

## Development and dependencies

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python tools/helm_evidence_fixture.py
python tools/validate_docs.py
python -m unittest discover -s tools/tests -v
git diff --check
```

| Direct dependency | Why it is needed |
|---|---|
| `cap-std 4.0.2` | Existing portable, capability-confined filesystem operations; avoids inventing security-sensitive OS wrappers. |
| `cap-fs-ext 4.0.2` | Matching upstream extension API for no-follow directory/file opens and Unix nonblocking opens; added to correct reproduced read/open races without replacing cap-std. |
| `serde 1.0.228` with derive | Typed, non-executable JSON contracts and deterministic reports. |
| `serde_json 1.0.149` | JSON reader/writer with bounded input and normal recursion protection. |
| `sha2 0.10.9` | RustCrypto SHA-256 instead of a new cryptographic implementation. |

No dev-only dependencies, CLI framework, async runtime, database, server, daemon,
RPC, plugins, GUI or workflow engine. Transitive/platform dependencies and their
licence metadata are inventoried in the review artifacts; a dependency's own licence
does not select HELM's licence. No dependency source is vendored or modified.

Linux x86_64 development tests also require Python 3 and permission to trace their
own child process with ptrace. They run without root and do not skip failures if
tracing is unavailable. Each verifier subprocess has a five-second deadline.
The [Linux workflow](../../.github/workflows/helm-evidence.yml) runs this module on
GitHub-hosted Ubuntu 24.04 with stable Rust 1.95.0 pinned explicitly. Updating that
pin requires review and rerunning the module checks. It uses `pull_request`/`push`,
read-only repository permission and no persisted checkout credentials, secrets,
self-hosted runners, deployment, release or artifact publication.

Primary references checked for the containment decision:
[cap-std design](https://github.com/bytecodealliance/cap-std/blob/main/README.md),
[Dir API](https://docs.rs/cap-std/4.0.2/cap_std/fs/struct.Dir.html), and
[patched Windows device-name advisory](https://github.com/bytecodealliance/cap-std/security/advisories/GHSA-hxf5-99xg-86hw).
The pinned version is newer than that advisory's 3.4.1 fix; this is not an exhaustive
dependency-security audit. Cargo.lock records exact resolutions and checksums.
