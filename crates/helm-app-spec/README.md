# helm-app-spec (experimental 0.1)

`helm-app-spec` parses untrusted JSON bytes into an immutable **desired-state**
application declaration or deterministic validation errors. Success establishes
only syntactic and semantic validity under schema 0.1. It establishes no installation,
runtime selection, loader execution, prefix or entry-point existence, test execution,
compatibility, trust, ownership or permission. Paths and references are inert data.

[ADR-0021](../../docs/adr/ADR-0021-second-product-module.md) is the architecture
authority. This README defines the implemented experimental schema; the earlier
selection report contains historical sketches. The separate owner instruction
authorised this library implementation. The [implementation review](../../docs/implementation/HELM-APP-SPEC-REVIEW.md)
records checks and remaining limits. `publish = false`; no licence is selected.

## API and immutable model

```rust
use helm_app_spec::{parse_spec, SpecErrors, ValidatedAppSpec};

fn inspect(bytes: &[u8]) -> Result<ValidatedAppSpec, SpecErrors> {
    let spec = parse_spec(bytes)?;
    let exact_document_identity = spec.spec_sha256().as_str();
    let desired_runtime_artifacts = spec.runtime().artifacts();
    let inert_entry_point = spec.entry_point().path().as_str();
    // Values only: no artifact, reference or entry point has been opened.
    let _ = (exact_document_identity, desired_runtime_artifacts, inert_entry_point);
    Ok(spec)
}
```

The public entry is
`parse_spec(&[u8]) -> Result<ValidatedAppSpec, SpecErrors>`.
[Model types and getters](src/model.rs) provide immutable borrowed views, with no
public constructors, mutable access, Deserialize or unchecked conversion. `Clone`,
`Debug`, `PartialEq` and `Eq` are implemented. `ValidatedAppSpec` exposes
`spec_sha256`, `application`, `runtime`, `environment`, `entry_point` and
`verification`. `Digest`, `Identifier` and `RelativeEntryPoint` expose `as_str`.
The four narrow enum vocabularies represent declared requirements only.

`SpecErrors::as_slice()` returns at most 64 ordered `SpecError` entries, each with
`code()`, `location()` and `explanation()`. `ErrorCode::as_str()` gives a stable
experimental machine code. Errors implement Display and `std::error::Error`.
Debug of a valid model includes its declared data; callers control where they log it.

## Exact document identity

`spec_sha256 = SHA256(exact input bytes)`. Whitespace, field order, JSON escapes and
metadata all contribute. Two byte-different documents with identical desired values
have distinct document identities under the usual SHA-256 collision assumption.
The parsed model is never serialized or canonicalized for identity. Model equality
includes the document digest; individual requirement getters permit comparisons
without asserting byte-identical documents. Array order is preserved as data and
does not specify an execution order. No serializer or semantic identity is provided.

## Closed JSON schema 0.1

All listed fields are mandatory except the two expressly optional fields. Explicit
`null` is rejected everywhere; omit an optional field to leave it unspecified.
Every object rejects unknown and duplicate decoded keys. There is no extension map,
implicit host default, include, URL, alternate input format or migration facility.

| Object | Fields and semantics |
|---|---|
| Root | `schema: "helm-app-spec"`, `version: "0.1"`, `application`, `runtime`, `environment`, `entry_point`, `verification` |
| application | `id` (local identifier), `version` (human metadata), `source` |
| application.source | `size` (desired bytes), `sha256` (desired digest), `architecture: "x86_64"` |
| runtime | `family: "wine"`, `artifacts` (1-16 requirements) |
| runtime.artifacts item | Unique `role` identifier, `size`, `sha256`, optional `label` metadata |
| environment | `windows_architecture: "win64"`, `prefix`, `disabled_dlls` (0-8 basenames) |
| environment.prefix | `role: "dedicated"` only; intent to associate a prefix with this application |
| entry_point | `path` (one inert prefix-relative entry point), optional `sha256` desired identity |
| verification | `definitions` (1-8 frozen pre-execution references) |
| verification.definitions item | Unique `role` identifier, `size`, `sha256` |

See the [A0 fixture](tests/fixtures/a0-7zip.json) and
[synthetic notes fixture](tests/fixtures/synthetic-notes.json) for complete documents.
The latter's digest/size pairs identify explicit fictional text recipes, not real
software. No core branch checks an application ID, Wine release or workflow name.

### Immutable artifact requirements

Source and runtime labels are metadata, not sufficient identity. Every runtime
requirement supplies its role, positive bounded size and SHA-256. A changed digest
with the same label remains a different requirement. Roles are unique within each
collection; runtime roles and verification roles have separate namespaces. Reusing
the same bytes for distinct roles is allowed. No global application catalogue exists.

A0's four WineHQ archives are represented as opaque artifact requirements. One
archive contains i386 runtime components; this does not change the x86_64 application
architecture or introduce cross-architecture execution semantics. Archive identities
do not prove an installed runtime tree or which loader later executed. There is no
package model, dependency resolution, acquisition, repository URL or loader observation.
The declared set is not a complete hermetic dependency closure.

### Environment and DLL declarations

Dedicated prefix intent grants no creation, adoption, inspection, deletion or
restoration authority. It supplies no host path, instance ID, data-ownership claim
or sandbox guarantee. Ownership remains unclassified, including user documents
inside a prefix. This is a contract limit, so a redundant `data_ownership` field is
omitted. Windows compatibility version and graphics/audio backends are not specified.

Each `disabled_dlls` basename requests that neither the native nor builtin DLL be
loaded under that name in a future bound environment. This models A0's explicit
`mscoree,mshtml=` disable intent. No effective configuration is observed or applied.
An empty list requests no disables. Names match `[a-z][a-z0-9_]{0,63}`, without an
extension. Duplicates reject; uppercase rejects instead of being normalized, so
case aliases cannot produce a valid duplicate. No arbitrary DLL override language.

### One portable entry-point spelling

The path is relative to a future Wine prefix, with the literal `drive_c/` followed
by one or more components. A0 uses `drive_c/Program Files/7-Zip/7zFM.exe`. That string
selects no prefix instance and is never converted to a host Path, normalized,
resolved, case-folded for lookup or opened. The required spelling is preserved.

Only ASCII letters/digits, `.`, `_`, `-`, space within components and `/` separators
are allowed. Components cannot be empty, `.` or `..`, start with space, or end with
space/dot. DOS device stems `con`, `prn`, `aux`, `nul`, `com1`-`com9`, `lpt1`-`lpt9`
(case-insensitive, including extensions and space before an extension) reject.
Absolute, UNC, drive-letter, backslash, alternate-stream, tilde, URL, Unicode,
control, expansion and shell-metacharacter spellings reject. Case itself is retained;
different casing is not asserted to identify different filesystem objects.

The bound is 1,024 ASCII bytes, 32 components including `drive_c`, and 255 bytes per
component. This deliberately supports less than the full Windows filename space.
No executable extension or actual PE format is established by path validation.
Omitted entry-point `sha256` means desired entry-point byte identity is unspecified.

### Verification references and identity direction

```text
app-spec -> frozen pre-execution definition / oracle / fixture bytes
later execution or evidence -> exact app-spec SHA-256
later execution or evidence -> observed outputs and results
```

References identify frozen definitions by role, SHA-256 and byte size. There is no
result-bundle field and no spec-to-result identity edge. No test workflow, shell
command, callback, provider or execution order is described. Locations and Git
provenance belong in fixture/review metadata outside this specification.

Parsing validates the identity declaration, not referenced content or chronology:
an arbitrary digest cannot reveal whether its unseen referent really is a pre-run
definition. A later authorised consumer must obtain the intended bytes and establish
their provenance and meaning. It must not silently substitute same-named files or
create expectations from resulting outputs. Validation is no attestation of that DAG.

## Untrusted input and deterministic errors

[The parser](src/parse.rs) builds a bounded temporary JSON tree with a custom Serde
visitor and private tree using deterministic BTreeMap objects (no randomized map
hashing, including under downstream map-feature unification). It checks each
decoded object key before insertion. Ordinary
`serde_json::Value` deserialization overwrites duplicate keys, demonstrated by the
regression test; direct Value parsing would violate this contract. No schema structs
or auxiliary maps bypass the visitor. Escaped duplicate names and identical duplicate
values reject, including in objects that would subsequently be unknown fields.

The pinned deserializer uses `from_reader(bytes)` with the concrete supplied
`&[u8]` as its in-memory reader. No file or caller-defined reader is accepted.
This avoids `from_slice`'s error-position scans through memchr's CPU discovery and
global dispatch cache. Line/column tracking stays local; public diagnostics still
collapse parser internals to the same bounded codes. The independent review's
isolated dependency-instrumentation probe reproduces that original purity defect
and checks the corrected path without changing the dependency cache.

| Bound | Limit |
|---|---:|
| Exact input bytes, checked first | 65,536 |
| Value depth (root depth 0) / total value nodes | 8 / 256 |
| Fields per JSON object / entries per JSON array | 16 / 16 |
| Decoded JSON key / string bytes | 80 / 2,048 |
| Local identifier | `[a-z0-9][a-z0-9._-]{0,79}` |
| Human metadata | 1-256 printable ASCII bytes, including inert shell-like text |
| Digest | Exactly 64 lowercase hex digits |
| Source, runtime artifact and definition size | JSON integer, 1 through 1,099,511,627,776 bytes (1 TiB) |
| Runtime artifacts / definition references / disabled DLLs | 1-16 / 1-8 / 0-8 |
| Entry-point bytes / components / component bytes | 1,024 / 32 / 255 |
| Error entries | First 63, then `ERROR_LIMIT` if any further error exists |

The 1 TiB declaration ceiling permits all A0 inputs and bounds numeric semantics;
it is not a download allowance or allocation request. Declared sizes never allocate
artifact buffers. No collection reserves from an untrusted size hint. An excess
array element is rejected before its subtree is parsed. Serde's default recursion
guard remains enabled alongside the stricter tree depth/node budgets. Serde may
allocate scratch for an escaped token up to the input bound before our decoded
string/key check; limits are not an exact peak-memory or wall-clock guarantee.

Input/syntax/duplicate-key/parser-budget errors return one document-level location
`$`. Semantic validation visits root and nested fields in the table's order; within
each object, one unknown-field diagnostic precedes missing fields in schema order,
then present fields are validated in schema order. Arrays retain declaration order.
An invalid parent/type or collection count suppresses child validation. No further
errors are accumulated after the 64-entry cap. Identical bytes produce equal models
or equal errors in equal order, with no timestamps, hostnames or random values.

Machine codes are defined by [ErrorCode](src/error.rs): `INPUT_TOO_LARGE`,
`JSON_INVALID`, `JSON_LIMIT`, `FIELD_DUPLICATE`, `FIELD_UNKNOWN`, `FIELD_MISSING`,
`FIELD_TYPE`, `SCHEMA_UNKNOWN`, `SCHEMA_VERSION`, `VALUE_UNSUPPORTED`,
`IDENTIFIER_INVALID`, `DIGEST_INVALID`, `METADATA_INVALID`, `SIZE_INVALID`,
`COLLECTION_LIMIT`, `DUPLICATE_RUNTIME_ARTIFACT`, `DUPLICATE_VERIFICATION_DEFINITION`,
`DUPLICATE_DLL`, `DLL_INVALID`, `PATH_UNSAFE`, `ERROR_LIMIT`.
Locations contain fixed schema names and bounded indices only. Explanations are
fixed bounded strings; raw parser errors and arbitrary input values/names are hidden.

## A0 provenance and excluded observations

The A0 fixture is assembled retrospectively from the original
[artifact pins](../../docs/experiments/evidence/app-baseline-2026-09-07/artifact-pins.json)
and definition commit `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`.
[Fixture provenance](tests/fixtures/provenance.json) maps the three definition
references to the exact protocol/oracle/content-fixture Git blob bytes. The
[development checker](../../tools/helm_app_spec_fixture.py) independently reads
those blobs and checks the fixture projection without fetching or opening installers.
It requires that original commit in the local Git clone.

Included: 7-Zip 26.03 x64 source (1,661,239 bytes and the original SHA-256), four
Wine `11.17~noble-1` archive requirements, dedicated win64 intent, the two explicit
DLL disables, GUI entry point and three frozen definition references.

Excluded: installed `7zFM.exe`/`7z.exe`/`7z.dll` hashes and version observations,
actual loader/server/mapping observations, installation outcome, host configuration,
prefix locator/instance, boot IDs, W1/W2 outcomes, output ZIP identity and resulting
evidence bundles. Those remain historical observations. The A0 fixture has no desired
entry-point digest because the original pre-run definition did not know it.
**A0's historical experimental FAIL remains unchanged.** Validation of this fixture
does not re-execute or regrade that experiment.

## Dependencies, authority and non-goals

Normal parsing performs no filesystem, network, process, environment-variable,
registry, package-manager, prefix, time, randomness or host-platform discovery.
There is no public I/O authority. SHA-256 uses pinned `sha2` with `force-soft`, so
the hashing call path does not use runtime CPU-feature discovery. The crate forbids
authored unsafe Rust. Upstream dependencies include unsafe code; this is not a
formal proof of their correctness or a comprehensive advisory/security audit.
Ordinary allocator/toolchain/OS correctness is assumed; process-wide out-of-memory
failure is outside the no-malformed-input-panic claim.

Three direct exact pins: `serde =1.0.228`, `serde_json =1.0.149`, `sha2 =0.10.9`.
All were already locked and reviewed for helm-evidence. No new third-party package
or version is introduced. `force-soft` is the only new dependency feature; Cargo
feature unification also chooses that SHA-256 backend for a combined workspace
build. This changes helm-evidence's SHA backend and can materially slow hashing;
it preserves digest and verifier-result semantics, not timing. A consumer linking
both crates also gets the software backend. Dependency renaming or resolver 3 does
not isolate features of the same package version. Its standalone dependency
declaration remains intact. There are no dev dependencies.

Build the existing evidence CLI with a separate
`cargo build -p helm-evidence --release --locked` invocation to retain its original
backend. The workspace CI tests the combined graph and the standalone evidence
crate, then builds each release crate separately. A later combined consumer must
evaluate this performance tradeoff explicitly; the current library does not
promise per-consumer SHA backend isolation. No local crypto implementation or shared
crypto crate is introduced. The independent review records before/after measurements.

Neither crate depends on the other. Tiny digest grammar duplication is intentional;
immutable identities are the conceptual link and no shared-utils crate is justified.
No CLI, server, daemon, database, package acquisition, installation recipe, execution,
Wine discovery, lifecycle, recovery, App Forge or additional architecture is added.

A possible future `helm-observe` would separately collect and compare observed state
under explicit read authority. It does not exist in this module and is not authorised
by this implementation. Validation can supply its desired input; it cannot create
observations by copying requirements.

## Development

Stable Rust 1.95.0 is the current reviewed toolchain. With cached dependencies:

```text
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo build -p helm-app-spec --release --locked
cargo build -p helm-evidence --release --locked
python -m unittest discover -s tools/tests -v
python tools/validate_docs.py
python tools/helm_app_spec_fixture.py
cargo bench -p helm-app-spec --bench parse --locked
python tools/check_helm_app_spec.py --output target/helm-app-spec-validation/validation.json
```

The benchmark is development tooling with its own timer/output, never part of the
library API. It measures repeated in-memory parsing, with no performance acceptance
target. The existing [Ubuntu workflow](../../.github/workflows/helm-evidence.yml)
now checks both workspace crates coherently, retaining the reviewed host runner,
toolchain, checkout SHA, read-only permissions and no release/deployment operations.
