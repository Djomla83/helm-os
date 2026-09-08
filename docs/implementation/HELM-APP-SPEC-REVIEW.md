# helm-app-spec implementation candidate review

**Date:** 2026-09-08. **Disposition:** **READY_FOR_INDEPENDENT_REVIEW**.
Local author checks passed; hosted CI is recorded separately below. Independent
review has not occurred. The author must not merge this module.

## Authority and reconstructed state

`git fetch origin`, then `git rev-parse main origin/main HEAD` returned
`e32ab269fbe7c4151186d9257e0aff67c0c70197` three times; `git status --porcelain=v1`
was empty. `git switch -c product/helm-app-spec` created the dedicated product branch.
No legitimate newer work was reset. [ADR-0021](../adr/ADR-0021-second-product-module.md)
is exactly **Accepted — bounded module-selection decision only**, approved by owner
Djomla83 on 2026-09-08. Its original separate-implementation-approval requirement
was satisfied by the current explicit owner instruction. The accepted ADR is unchanged.

Read completely: AGENTS, AGENT_STARTER, master plan, PROJECT_STATE, DECISIONS,
ADR-0021, the second-module selection analysis, helm-evidence README and public
API/model, A0-7ZIP report, and the independent helm-evidence review. Traced the
original application/Wine pins and frozen definition blobs, plus the relevant
installed/runtime observations to distinguish them from desired declarations.
Historical sketches are not evidence of implemented functionality.

## Change and minimal design refinement

One library, `crates/helm-app-spec`, implements exactly:

```rust
pub fn parse_spec(bytes: &[u8]) -> Result<ValidatedAppSpec, SpecErrors>;
```

The [module README](../../crates/helm-app-spec/README.md) is the implemented schema
0.1 authority under ADR-0021. Its API section and [model](../../crates/helm-app-spec/src/model.rs)
define all public getters and typed wrappers. Every domain field is private to the
crate; no unchecked constructors, Deserialize, mutable getters or serialization are
exposed. Error codes, locations and fixed explanations have a separate bounded API.

Before coding, the A0/ADR walkthrough selected these refinements:

- JSON only, `schema: helm-app-spec`, `version: 0.1`; every object is closed.
- Application ID/version metadata plus source size/SHA-256/x86_64. No redundant
  display name, source URL, catalogue ID or installer recipe.
- Wine-only runtime with 1-16 unique role/size/SHA-256 artifacts and optional labels.
  No package architecture taxonomy: A0's i386 archive is opaque artifact data.
- Dedicated win64 intent and 0-8 explicit DLL disables. A0 justifies `mscoree` and
  `mshtml`; lowercase basename grammar prevents case aliases. No general overrides.
- One inert prefix-relative `drive_c/...` path, optional desired SHA-256, omitted
  when unknown. No source-derived installed digest is invented. Explicit null rejects.
- 1-8 frozen definition references with role/size/SHA-256. Git paths and commit
  provenance remain external fixture metadata; no evidence-format/result fields.
- Prefix ownership remains unclassified as a semantic limit. No redundant constant
  ownership/identity-scope field and no instance locator are necessary.

No additional architecture value or provider was added. Limits are documented
including the 64 KiB input ceiling, 1 TiB declared-size ceiling, structural budgets,
80-byte identifiers, 1,024-byte/32-component paths, and 64-diagnostic cap. The size
ceiling bounds declaration semantics, never memory allocation or acquisition.

## Identity, purity and parser investigation

`spec_sha256` hashes **the original byte slice**. Whitespace and field-order
regressions prove equal desired values with distinct exact document hashes. The
whitespace test also compares against independent Python hashlib digests of the
actual fixture bytes. No canonical JSON, normalization or model hash exists.

The product call graph is `parse_spec -> bounded JSON -> semantic validation ->
immutable model/errors`. There are no filesystem, network, process, environment,
registry, clock, random, package or prefix operations. Relative paths are validated
as ASCII strings, with no host Path conversion or resolution. References are never
opened. Declaration success cannot establish actual runtime installation or use.

Pinned `serde_json 1.0.149` source `src/value/de.rs:121-145` inserts decoded object
keys into a map and overwrites duplicates. The test demonstrates that ordinary
Value deserialization accepts the last decoded `x` key. The new custom visitor
checks duplicates **before insertion at every object**, including escaped names,
identical values, optional/null values and objects nested inside unknown fields.
No auxiliary map bypasses that visitor. The private raw tree uses BTreeMap rather
than serde_json::Map so optional downstream preserve_order feature unification
cannot introduce randomized map hashing into the pure product path. It retains Serde's recursion protection,
adds depth/node/string/key/object/array budgets, ignores untrusted size hints and
rejects extra array elements before deserializing their subtrees.

Pinned `sha2 0.10.9` normally chooses an architecture backend that can detect CPU
features. Its `src/sha256.rs` selects software compression first when `force-soft`
is enabled. The new crate requires that feature even outside this workspace.
Source: [RustCrypto sha2 feature inventory](https://docs.rs/crate/sha2/0.10.9/features)
and the exact locally cached source. A docs.rs serde_json API request failed to
return a usable page; the pinned source and executed duplicate-key test supply
the parser evidence instead. No fallback installer or dependency upgrade was used.

The parser returns one `$` error on syntax, duplicate or resource failure. Semantic
errors use schema/declaration order; unknown names are not echoed. The first 63
errors are retained and an explicit terminal ERROR_LIMIT reports omission. Error
messages never expose parser internals, raw private values or unknown field names.
Successful model Debug includes declared values and is not a redacting log API.

## A0 and second application

[A0 fixture](../../crates/helm-app-spec/tests/fixtures/a0-7zip.json): retrospective
desired declaration, not a historical pre-run app-spec artifact. Installer size
1,661,239 and SHA-256
`0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd`, four WineHQ
`11.17~noble-1` archive requirements, dedicated win64 intent, two DLL disables and
`drive_c/Program Files/7-Zip/7zFM.exe`. Archive roles follow the pinned package names
only as local opaque role identifiers, with package versions/architectures in labels.
The archive set does not prove an installed runtime closure or which loader executed.

Three pre-execution references use original Git blob bytes at
`5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`: protocol 34,585 bytes, oracle 9,093,
content fixture 1,290. [Provenance](../../crates/helm-app-spec/tests/fixtures/provenance.json)
records exact hashes and paths. [The fixture checker](../../tools/helm_app_spec_fixture.py)
compares those independent historical inputs with the new JSON projection. It never
fetches, inspects or opens the installer, Wine archives or installed entry point.

Intentionally excluded observations: installed executable/DLL hashes and versions,
loader/server/mapped-file facts, installation result, prefix path/instance, host
configuration, boot IDs, workflow PASS/FAIL, output ZIP identity and resulting
helm-evidence bundle. The A0 entry-point expected digest is omitted. Historical
experiment FAIL and W1 wrong-destination/W2 scoped success are unchanged.

[Synthetic notes](../../crates/helm-app-spec/tests/fixtures/synthetic-notes.json)
has a different application ID/version/path, one fictional runtime requirement,
no disables, one frozen definition reference and an optional desired entry digest.
Identities are real SHA-256/length calculations over explicitly fictional text
recipes in the fixture tool, not fabricated hashes of claimed software. Tests also
vary the application ID independently, with no application-specific core logic.

The reference direction remains `app-spec -> frozen definitions`, and later
`execution/evidence -> exact app-spec SHA-256 + observed results`. Unseen digest
content cannot establish that a referent actually precedes execution or contains a
definition. That provenance remains a later caller/reviewer duty; no result format,
execution mechanism or content attestation is claimed by this validator.

## Positive and negative test matrix

The [contract tests](../../crates/helm-app-spec/tests/contract.rs) derive expected
outcomes from the owner's matrix, accepted ADR and original pins. They are author
tests, not an independent review. A separate reviewer must assess their adequacy.

| Required case | Test coverage |
|---|---|
| Positive A0 and synthetic app | Exact source/runtime/definition identities and narrow typed views; alternate valid IDs and artifact counts |
| 1 empty input; 3 malformed JSON | Empty/whitespace/truncation, broken objects/arrays, trailing commas, malformed UTF-8, BOM, lone surrogate, trailing document, huge exponent |
| 2 oversized input | Exactly 65,536 bytes accepts a valid padded document; 65,537 rejects before UTF-8 parsing |
| 4 schema; 5 version | Unknown schema/version reject with distinct codes |
| 6 unknown field; 7 missing field | Every object level; removal of every required field; wrong JSON object types |
| 8 malformed digest; 9 uppercase digest | All identity locations, wrong length/nonhex/non-ASCII, explicit null; optional entry identity |
| 10 invalid app ID | Empty, leading punctuation, uppercase, spaces, separators, control/shell/Unicode, 80/81-byte boundary; runtime/definition roles too |
| 11 duplicate runtime role | Same and conflicting artifact identities for one role; later declaration identified |
| 12 zero/absurd size | All size locations: 0, negative, float/exponent, negative zero, noninteger/null, u64 overflow; inclusive 1/1 TiB boundaries |
| 13 runtime; 14 architecture | Proton, wrong casing, x86/ARM, win32/wow64 and shared prefix reject |
| 15 absolute path; 16 traversal; 17 alternate separators | Host/drive/UNC/device, parent/current/empty components, slash/backslash, streams, URLs, DOS names, trailing dot/space |
| 18 path depth/length | 32/33 components, 255/256 component bytes, 1,024/1,025 total bytes |
| 19 duplicate DLL | Duplicate declaration; mixed case fails narrow grammar; extension/override/list/control spellings reject |
| 20 verification identity | Malformed digest/size/role and duplicate definition role; resulting evidence fields reject |
| 21 byte-different whitespace | Equal desired values; different exact-byte SHA-256, independently calculated expected digests |
| 22 field order | Both declarations valid with equal desired values and different raw identity |
| 23 malicious long IDs/arrays | Huge identifiers/arrays, excessive keys, deep unknown subtrees, aggregate node budget; individual and combined valid maxima |
| 24 shell-like strings | Path/ID/DLL metacharacters reject; printable label text preserves shell-like content as inert data |
| Duplicate JSON keys | Ordinary/escaped duplicates at every schema object; nested unknown/array objects and repeated optional/null values |
| Desired/observed separation | Observed runtime/installation/entry point/results, environment/registry maps, execution/recovery hooks reject |
| Determinism and diagnostics | 100 repetitions per valid fixture and multi-error document, exact code/location order, no private marker, bounded explanations/error cap |
| Malformed-input panic corpus | 256 one-byte values, every A0 truncation, four mutations at every A0 byte, and 24 repeated-byte inputs through 64 KiB; each result repeated |
| Public invariants | Compile-fail docs reject unchecked Digest construction and mutation through artifact views |
| Existing module regressions | Full workspace tests and A0 evidence fixture/provenance checker; no historical source change |

## Dependency and CI changes

Three exact direct dependencies reuse the existing pins: serde 1.0.228,
serde_json 1.0.149 and sha2 0.10.9. No dev dependencies or new third-party package
versions. Cargo.lock adds only helm-app-spec and its dependency edges. The only
new feature is `sha2/force-soft`; workspace feature unification selects the same
software backend in the combined helm-evidence build without altering its byte
semantics. Existing standalone helm-evidence manifest/source/API are unchanged.
Dependency licence metadata is an inventory, not acceptance of a HELM licence.

The existing `.github/workflows/helm-evidence.yml` is kept at its stable path and
declares the name HELM Rust workspace Linux. Same GitHub-hosted Ubuntu 24.04,
Rust 1.95.0, immutable checkout action, read-only permission, no stored credentials,
15-minute deadline. Commands now cover the workspace and repository Python/docs
checks. Full Git checkout supplies frozen definition blobs for provenance tests.
No deployment, release, signing, self-hosted runner or secret input.

## Actual validation and measurements

Windows 11 build 26200; AMD Ryzen 9 5950X, 16 physical/32 logical cores confirmed
read-only. Rust/Cargo 1.95.0, LLVM 22.1.2, x86_64-pc-windows-msvc; Python 3.14.3.
Linux validation passed on the hosted Ubuntu 24.04 job with Rust 1.95.0 and runner
image ubuntu24/20260831.293. No VM/WSL lab was started for this task.

The [Windows validation receipt](evidence/helm-app-spec-2026-09-08/validation.json)
records 14 successful commands and the complete output/source identity inventory.
Its HEAD is the starting main because checks ran before the candidate commit;
the included per-file SHA-256 values bind the actual new source/test/fixture bytes.

| Required check | Executed result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS: 30 new contract tests, 3 new doctests, 45 unchanged helm-evidence tests on Windows; none ignored |
| `cargo build --workspace --release --locked` | PASS |
| `cargo check -p helm-app-spec --all-targets --all-features --locked` | PASS |
| `python -m unittest discover -s tools/tests -v` | PASS: 37 tests, including the new historical fixture projection test |
| `python tools/validate_docs.py` | PASS: 88 Markdown / 229 JSON files at this checkpoint; local structure only |
| Both fixture checkers | PASS: new three-file fixture projection and unchanged 22-file A0 evidence fixture |
| `git diff --check`, staged whitespace | PASS; checked again before publication |
| Privacy / forbidden-artifact / historical audit | PASS: changed/new unignored UTF-8 files and decoded streams scanned, all 153 historical publication hashes match, no tracked target output; bounded scan, not comprehensive secret detection |
| Malformed-input safety | All 10,310 deterministic corpus cases returned without parser panic; each result repeated identically; not exhaustive fuzzing or a formal proof |
| Determinism | 100 repeated equal models per valid fixture; 100 equal ordered multi-error results, stable raw hashes and independent whitespace-hash expectations |
| Hosted Ubuntu CI | PASS at implementation commit `7d1de4cd02268cf29ea1e4f13136336ce0c2c36f`: workspace fmt, Clippy, tests, release build, 37 Python tests, docs and frozen-fixture check; [run 34219988167](https://github.com/Djomla83/helm-os/actions/runs/34219988167) |

Physical Rust LOC includes comments and blank lines: **1,129 product**, **874 test**,
**37 benchmark** (nonblank: 1,075 / 841 / 36). Before this module, workspace product
LOC for helm-app-spec was zero; existing helm-evidence is unchanged. This exceeds
the selection sketch's 400-700 product estimate because the implementation includes
the duplicate-safe bounded raw tree, explicit immutable getters/docs and structured
field diagnostics. No excess comes from execution, discovery, generic providers or
canonicalization; independent review should still assess simplification opportunities.

Direct dependencies: **3**, all reused; third-party package/version additions **0**;
existing lock entries changed **0**. The receipt inventories 21 resolved third-party
packages in this crate's workspace closure (including direct, build and platform
packages) and their licence metadata/features. No standalone shared identity crate.

Windows release `libhelm_app_spec.rlib`: **440,720 bytes**, including Rust metadata
and excluding transitive archives; not an installed product footprint. Peak memory
was not measured because no cheap reliable per-parse peak measurement was used.

Benchmark methodology: release build, in-memory bytes, 1,000 warmups per case, then
five samples of 10,000 parse/drop calls using `black_box`; one recorded run on the
host above, no CPU affinity/frequency isolation. Median of sample mean times:

| Input | Exact bytes | Median time per call | Sample mean range |
|---|---:|---:|---:|
| A0 | 2,006 | 17.808 microseconds | 17.760-17.865 microseconds |
| Synthetic notes | 1,001 | 8.749 microseconds | 8.727-8.765 microseconds |
| Malformed JSON | 10 | 0.225 microseconds | 0.223-0.226 microseconds |
| Oversized | 65,537 | 0.076 microseconds | 0.0757-0.0764 microseconds |

These are baseline observations, not latency percentiles, acceptance targets,
micro-optimization claims or application execution measurements. The benchmark
timer/output reside only in development code.

Reproduce using `python tools/check_helm_app_spec.py --output target/helm-app-spec-validation/validation.json`.
The script runs the exact required Cargo commands, release/check builds, benchmark,
repository Python/docs checks, both fixture checks and whitespace checks; it reuses
the existing privacy/publication audit at this task's base and verifies no existing
lockfile package or evidence-crate changes. Output binds product/test/fixture bytes
by hash. Measurements are observations, not performance acceptance targets.

Development failures retained here: the initial cargo check ran before the declared
benchmark file existed and reported the missing bench; one pinned-source rg call
used a PowerShell wildcard literally and failed until expanded; initial contract
test compilation found a wrongly escaped lone-surrogate test literal and a
temporary-reference lifetime error. Those test/tooling errors were corrected before
the first passing test run. No expected rejection was removed or weakened. No
application experiment failed or ran in this task.

## Publication and independent-review boundary

Implementation commit `7d1de4cd02268cf29ea1e4f13136336ce0c2c36f` was pushed normally
to `product/helm-app-spec`. Local main and origin/main remained the authorized
starting commit `e32ab269fbe7c4151186d9257e0aff67c0c70197`. The complete staged
implementation diff was inspected; all 23 recorded source/fixture/manifest
identities matched the validation receipt. The publication audit scanned 22
candidate files, verified 29 indexed fixture artifacts and all 153 historical
publication hashes. The working tree was clean after the implementation push.

The [CI receipt](evidence/helm-app-spec-2026-09-08/ci-result.json) records successful
Ubuntu validation of that implementation commit, completed at 11:19:20 UTC on
2026-09-08. All 30 new contract tests, three new doctests and 45 existing
helm-evidence tests passed, including its Linux filesystem regressions. GitHub's
run metadata still shows the workflow's default-branch name, helm-evidence Linux;
the receipt's actual job steps demonstrate the expanded workspace checks.

The follow-up publication report changes only this review, PROJECT_STATE and the
CI receipt. Product code, tests, fixtures, manifests, tools and workflow remain
identical to the passing CI commit. Documentation validation passed (88 Markdown,
230 JSON files, 882 local link targets), as did diff checks and the bounded
publication audit (23 candidate files, 153 historical hashes) for this update; the
Rust CI result applies to the implementation commit, not a claimed rerun at the
report-only tip. The complete CI log is retained in ignored local target output;
its digest and selected result lines are recorded in the receipt.

This is **READY_FOR_INDEPENDENT_REVIEW**. No independent reviewer has approved
the module, no merge was performed, and no production release is authorized.

## Limits, data/permission impact and next decision

No product I/O or data/permission changes. New files are source, synthetic/declared
JSON, documentation and development checks. Generated target output remains ignored.
No new licence, expense, host privilege, branch protection or protected-main change.

Unverified: exhaustive fuzzing/formal panic proof, precise peak memory, hard runtime
deadline, comprehensive dependency/advisory audit, authentication/freshness, referenced
content/chronology, full Windows path space, real second-application behavior,
installation/runtime/loader/prefix/entry-point existence, compatibility and recovery.
Allocator exhaustion or compromised dependencies/OS are outside the bounded parser
claim. Product usefulness beyond declaration validation remains a hypothesis.

Independent review has not occurred. No merge or further module is authorised to
the author. **One recommended next decision:** assign independent review of this
candidate's schema, pure call graph, parser/error behavior and acceptance tests.
