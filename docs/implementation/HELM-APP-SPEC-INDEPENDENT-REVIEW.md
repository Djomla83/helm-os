# helm-app-spec independent review

**Date:** 2026-09-08. **Review branch:** `review/helm-app-spec-independent`.
**Disposition:** **READY_FOR_OWNER_MERGE** for the corrected independent branch.
No unresolved BLOCKER or IMPORTANT finding remains for the current separate release
artifacts. Combined-consumer SHA feature coupling remains explicit below.

The original candidate has one BLOCKER and one IMPORTANT finding. The corrections
retain the bounded decision in [ADR-0021](../adr/ADR-0021-second-product-module.md).
This report is independent of the [authoring report](HELM-APP-SPEC-REVIEW.md); its
claims were checked against source, Git blobs and new executions. No merge,
helm-observe implementation, application rerun, licence selection or release occurs.

## Exact state and correction history

| Role | Exact commit |
|---|---|
| Authoritative main/base | `e32ab269fbe7c4151186d9257e0aff67c0c70197` |
| Candidate implementation | `7d1de4cd02268cf29ea1e4f13136336ce0c2c36f` |
| Candidate review tip | `18c7aa5e80617483b6f5afd0fea4affeb62f926c` |
| Independent tests and preserved reproductions | `2edbb070068f6e6dc01978f43874d9992a7fb86a` |
| Product/build corrections | `c9d6c42b331c65805a256514dcd3e0d83a7189f3` |

`git fetch --all --tags` and ancestry inspection established the linear chain
base -> implementation -> review tip. The tip changes only PROJECT_STATE, the
authoring report and its CI receipt. `git diff 7d1de4cd 18c7aa5 -- crates Cargo.toml
Cargo.lock tools .github` is empty. The review branch starts at the supplied tip;
the authoring branch was never reset, amended or force-pushed. Local main and
origin/main remain the supplied base. The correction changes one product expression
and its explanatory comment; helm-evidence product/test/fixture bytes and both
manifests and Cargo.lock remain identical to the candidate.

Read the repository instructions, starter, project state, master plan, decision
index, ADR-0021, selection analysis, authoring report, all app-spec Rust source,
all original tests/fixtures/benchmark, manifests/lockfile, workspace workflow,
evidence source and the original A0 definition/pins. Source review preceded edits.
The selection document's proposed field names do not override the implemented
experimental schema or the bounded accepted decision.

## Findings before corrections

### B1 — BLOCKER: parser error handling performs CPU discovery

**Location:** candidate `crates/helm-app-spec/src/parse.rs:69`,
`Deserializer::from_slice(bytes)`. Pinned serde_json 1.0.149
`src/read.rs:421-425` calculates SliceRead error positions with
`memchr::memrchr` and `memchr_iter().count()`. The resolved memchr 2.8.3 `std`
feature enables its x86_64 CPU dispatch. Its
`src/arch/x86_64/memchr.rs` `unsafe_ifunc!` macro initially calls `detect`, chooses
the backend and writes an atomic global function pointer.

**Scenario/reproduction:** in a fresh process, validate a valid A0 declaration,
then `b"{"`. No application execution is involved. The
[purity probe](../../tools/helm_app_spec_purity_probe.py) copies the cached dependency
into an isolated generated project and inserts a panic sentinel at that exact
dispatcher. The original candidate reaches it, exit 101. Before correction, current
source also reached it; the expected failing regression is preserved in
[purity-before.json](evidence/helm-app-spec-independent-2026-09-08/purity-before.json).
The panic is instrumentation, not an uninstrumented parser crash.

**Impact:** invalid-input validation violates the explicit no-host-discovery claim
and depends on mutable global dispatch state. It does not expose a filesystem,
network or process capability, and no hash/error semantic discrepancy was found.
Nevertheless, hashing's `force-soft` alone does not establish the promised purity.

**Minimum correction, applied:** use `Deserializer::from_reader(bytes)` with the
concrete `&[u8]` already supplied to the API. The reader is an in-memory slice,
never a file or caller-implemented reader. `IoRead<&[u8]>` counts positions locally
through `LineColIterator<Bytes<&[u8]>>`, avoiding SliceRead's memchr scans.
No dependency feature, version, public API or schema changes. After correction the
instrumented process completes valid, malformed, duplicate, budget and trailing
input cases without reaching the sentinel. The complete independent result
fingerprint is unchanged. Windows A0 parsing increased from 17.755 to 21.604 us
in the same existing benchmark; this measured cost is retained, not hidden.

### I1 — IMPORTANT: combined release build changes evidence hashing performance

**Locations:** candidate `crates/helm-app-spec/Cargo.toml:13`,
`sha2/force-soft`; `.github/workflows/helm-evidence.yml:47`,
`cargo build --workspace --release --locked`; README's former claim of no
helm-evidence observed behavior change.

**Scenario/reproduction:** build both workspace members. Actual
`cargo tree --workspace --locked -e features -i sha2` shows one sha2 0.10.9 package
with `force-soft` from app-spec, used by both crates. In contrast,
`cargo tree -p helm-evidence --locked -e features -i sha2` has only default/std.
Resolver 3 and dependency aliases do not isolate this shared package's features.
This agrees with the [Cargo resolver documentation](https://doc.rust-lang.org/cargo/reference/resolver.html#features).

The [preserved pre-correction comparison](evidence/helm-app-spec-independent-2026-09-08/candidate-windows-comparison.json)
uses the identical measurement program against the actual base source, candidate
combined dependencies, and candidate evidence alone. All registry lock records
remain pinned. Five samples follow ten warmups: 50 verifier calls/sample and 16
hash calls/sample. Synthetic payload bytes are `i % 251`; the 4 MiB bundle extends
the existing synthetic contract with one extra declared artifact. No historical
evidence is edited. Measurements are sample-mean medians, not latency percentiles.

| Windows operation | Base | Candidate combined | Candidate standalone |
|---|---:|---:|---:|
| A0 verification | 2.274 ms | 2.327 ms | 2.292 ms |
| Synthetic verification | 1.339 ms | 1.276 ms | 1.305 ms |
| Synthetic plus 4 MiB verification | 7.009 ms | 15.497 ms | 7.154 ms |
| SHA-256, 64 KiB | 27.475 us | 164.313 us | 28.663 us |
| SHA-256, 4 MiB | 1.764 ms | 10.586 ms | 1.809 ms |

**Impact:** about 6x hashing time and 2.21x time for the larger verifier fixture on
this SHA-capable host. The complete three verifier reports, not just verdicts,
compare equal across all modes. Digests also match Python hashlib. Small-fixture
I/O noise is not attributed to a speedup/regression. No CPU affinity, frequency,
thermal or cold-cache control was used; these are bounded observations.

**Minimum correction, applied:** keep `force-soft`; retain workspace tests for the
combined graph; additionally test evidence alone and build release crates in
separate Cargo invocations. The final evidence CLI build therefore retains its
baseline dependency features. Document the difference between semantic equality
and timing, and the exact standalone build command. No shared crypto crate, copied
crypto implementation or dependency upgrade is justified by this finding.

**Remaining explicit boundary:** `cargo build --workspace` and a consumer linking
both crates still unify to software SHA. This change isolates the current separate
release artifacts, not two libraries inside one executable. Combined consumers
must assess that documented cost. The review does not claim the coupling vanished.
There is currently no combined product executable or performance acceptance target.

**MINOR:** none separately identified. **NO FINDING** in the remaining reviewed
schema/model areas below; this is bounded evidence, not a proof of absence of bugs.

## Corrected purity trace

`parse_spec` first checks byte length, then calls the private bounded parser,
semantic validators and immutable constructors. The pinned reader consumes only
the supplied slice. Serde's seed/visitor traits perform in-memory dispatch; decoded
strings, vectors and BTreeMaps allocate ordinary process memory. No randomized map
is used. Unknown keys are checked but never interpreted as capabilities.

Semantic validation uses fixed names, bounded iteration, ASCII predicates and
integer comparisons. `safe_path` uses string splitting and ASCII case conversion,
not `std::path` or host containment checks. Errors use fixed code/explanation text
and fixed field names/bounded indices. No caller callback or reader is accepted.

`validate.rs:197` passes the original byte slice to `Sha256::digest`. In digest
0.10.7 this creates, updates and finalizes a local hasher; sha2 0.10.9's
`src/sha256.rs` selects `soft::compress` before architecture dispatch when
`force-soft` is present. Fixed SHA state/constants, block-buffer 0.10.4 padding,
generic-array 0.14.7 and integer compression operations do not inspect the host.
The soft implementation's fixed-size chunk conversion cannot fail for its 4-byte
chunks; the 64 KiB bound makes its length counters far smaller than overflow.

No reachable normal product operation performs filesystem/network access, process
execution, environment lookup, registry access, package/runtime lookup, clock or
randomness access, or CPU/platform discovery after B1. Dependency source plus the
targeted instrumentation support this conclusion. The memchr package remains in
the graph, including its dispatcher, but the corrected parse call no longer reaches
that path. Build scripts/toolchains and test/benchmark I/O are outside this API.
Ordinary allocator, standard-library, compiler and OS correctness remain assumptions;
custom global allocators, OOM, compromised dependencies and arbitrary target/toolchain
configurations are not a purity or formal panic guarantee.

## Identity, parser, duplicates and errors

Independent Python hashlib matches the five exact synthetic document identities:
original, whitespace-only change, field-order change, escaped-equivalent `n`, and
removed final newline. Every desired getter compares equal across these variants;
document equality differs because it includes the raw digest. Repeated identical
bytes and clones compare equal. Invalid input returns only `Err(SpecErrors)`;
even if an intermediate hash/model is constructed before accumulated errors are
checked, no validated identity escapes through the public API.

| Independent parser attack | Result/interpretation |
|---|---|
| Literal/escaped duplicate keys, unknown subtrees, nested maps, slash escapes and surrogate-pair equivalent keys | FIELD_DUPLICATE before insertion; every object uses the same visitor |
| Runtime roles, definition roles and DLLs with escaped-equivalent values | Corresponding duplicate-semantic code at the later array index |
| Uppercase IDs/roles/DLLs | Invalid grammar; cannot hide a valid case alias. Separate role namespaces and same bytes under distinct roles are documented |
| Arrays 16/17; objects 16/17 | Boundary parses; extra element/key yields JSON_LIMIT before its value subtree |
| Total value nodes 255/256/257 | First two parse; node 257 rejects, including an empty container; keys are not value nodes |
| Root depth 0, deepest value depth 8/9 | Depth 8 parses; 9 rejects. Empty deepest container and nonempty cases tested |
| Decoded strings 2048/2049; keys 80/81, literal and escaped | Inclusive decoded-byte bounds; excess yields JSON_LIMIT |
| Negative, negative zero, finite float/exponent, null/bool/string in a size field; u64 overflow representable as f64 | SIZE_INVALID; never coerced into an allowed integer |
| 1 and 1 TiB; 1 TiB+1 | Inclusive positive size bounds; excess is SIZE_INVALID |
| `1e309`, huge exponents/50,000-digit integer | Parser representation rejection, collapsed to JSON_INVALID, even though some tokens satisfy JSON's lexical number grammar |
| Bad UTF-8, lone surrogates, malformed numeric syntax, trailing document | JSON_INVALID; valid surrogate pairs parse as strings before schema checks |
| Trailing JSON whitespace | Accepted and included in the exact-byte digest |
| Malformed duplicate value; duplicate seventeenth key | First encountered duplicate/budget failure wins. Parser failure is not exhaustive syntax analysis |

The original 30 contract tests and three doctests remain. Eight additional tests
in [independent_review.rs](../../crates/helm-app-spec/tests/independent_review.rs)
cover the gaps and a separate seeded property pass. The author's 10,310-case corpus
is retained and rerun, not treated as exhaustive fuzzing.

Diagnostics are nonempty, deterministic and capped at 64 entries: first 63 errors
plus ERROR_LIMIT if another exists. All twelve independently specified sibling
errors survive simultaneous failures in application/source, runtime, environment,
entry point and verification. Construction-time `?` follows those sibling checks;
it does not accidentally short-circuit unrelated fields. Invalid parent shape or
collection count suppresses its children as documented. Unknown names yield one
parent-location error. Display/Debug of errors never echo long/private attacker
values or serde messages; the added tests bound combined rendering below 20,000
bytes. Model Debug intentionally contains valid declared data and is not redacting.

## Resource limits, paths and public model

The parser's 64 KiB ceiling applies before decoding/hashing. Depth 8, 256 values,
16 object keys/array entries, decoded key 80 and string 2048 keep the transient tree
small. Semantic maxima together require only 143 value nodes at depth 4; an
independently assembled document with maximum roles, labels, artifact/definition/DLL
counts, declared sizes and a 1024-byte path is 11,187 compact JSON bytes. There is
no impossible valid collection maximum imposed by the parser. Its 12,627-byte
indented spelling was also successfully parsed on Windows; the exact synthetic
input and check are preserved in the
[resource receipt](evidence/helm-app-spec-independent-2026-09-08/resource-calculation.json).
Verification/DLL
counts 9-16 get semantic COLLECTION_LIMIT; 17 gets parser JSON_LIMIT, as documented.

The input cap also bounds token scratch before the decoded-length check. The
corrected slice reader copies unescaped tokens into scratch as well as escaped
ones. Tree/model copying and allocator capacity cause amplification, but no claimed
artifact size or untrusted collection hint requests proportional allocation.
Error production/rendering is independently bounded. Peak allocation and a hard
wall-clock deadline were not measured/proved; no unexplained limit change is made.

Path checks reject Windows absolute/drive/UNC/backslash/ADS spellings, slash
traversal, repeated slash, `.`/`..`, leading component spaces, trailing dot/space,
controls, Unicode and DOS device names with mixed case/extensions/space before an
extension. `COM1`-`COM9`/`LPT1`-`LPT9` reject; `com0`, `com10`, `com01` remain allowed
under the stated grammar. `drive_cx`, `drive_c.`, `drive_c ` and `Drive_c` do not
satisfy the exact prefix. Original 32/33 component, 255/256 component-byte and
1024/1025 total-byte tests pass. ASCII-only is an explicit 0.1 limitation. Inert
grammar acceptance establishes neither filename existence nor containment.

All invariant-bearing fields are crate-private. Raw tree modules are private;
there is no public unchecked constructor, Deserialize, Default, mutable getter,
conversion or serializer for the validated model. Five independent external
compilation attempts reject private construction/empty collections, unchecked
newtypes, mutation through getters, serde deserialization bypass and raw-tree access
with the expected Rust errors. Public enum variants cannot introduce another runtime
family. Successful values necessarily have Wine, nonempty bounded artifacts and
definitions, unique roles, valid sizes/digests/path and no observed-state fields.

## Desired/observed boundary, A0 and generality

The schema and public types describe requirements. Source identity is not PE
inspection; prefix dedication is intent without location, ownership or authority;
entry path/optional digest is a future expectation without existence/verification.
Closed objects reject installation, actual loader, prefix-instance, registry,
execution, status/result and evidence-bundle fields. No success/PASS/boot/output
model is exposed by the validator.

Independent comparison with original commit
`5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae` confirms the source size/hash/version and
all four Wine archive identities. The original pins explicitly precede execution;
runtime package bodies were not yet downloaded at that recording. The frozen
protocol specifies fresh win64, `mscoree,mshtml=` and the 7-Zip GUI/default path.
The retrospective prefix-relative spelling honestly projects that intent.

The three exact Git blobs rehash to the fixture's protocol/oracle/content-fixture
references (34,585 / 9,093 / 1,290 bytes). These are pre-A0 definitions, not the
current subsequently annotated files. All installed binary identities, actual Wine
loader/server, install success, workflow results, ZIP output identities, boot IDs
and evidence verdicts remain excluded. The four archive requirements do not prove
the actual loader used, a hermetic closure or package-manager behavior. No Wine,
installer or artifact bytes were downloaded or executed in this review.

Verification refs contain only role/size/hash. There is no typed result edge and
no future evidence hash in spec identity. An opaque digest can always name unseen
mislabelled bytes; pure validation cannot attest content or chronology. That
documented limitation is consistent with declaration validity, not a schema-proven
global DAG or an execution authorization. Future evidence binding remains undesigned.

Synthetic notes uses different names, paths, artifact count, missing labels, empty
DLL list, optional entry digest and one fictional definition. Its identities hash
the declared fictional text recipes. Source search found no 7-Zip/experiment
special case. The independent generator also validates 4,000 arbitrary app IDs;
no additional fixture or expanded runtime scope was necessary.

## Validation, environment and evidence

Windows: Windows 11 build 26200, Ryzen 9 5950X, 16 physical/32 logical cores,
Rust/Cargo 1.95.0, LLVM 22.1.2, `x86_64-pc-windows-msvc`, Python 3.14.3. Hardware was
queried read-only. Ordinary WSL Ubuntu was briefly started for tool availability:
Ubuntu 24.04.4 / WSL2 6.6.87.2 had neither Rust nor a C compiler. It was returned to
its initially stopped state without setup changes; project labs/VMs were not started.
Linux execution used the existing GitHub-hosted Ubuntu 24.04 job, image
`20260831.293.1`, kernel `6.17.0-1022-azure`, AMD EPYC 9V74 with four exposed CPUs,
Python 3.12.3, Rust 1.95.0 / LLVM 22.1.2 and `x86_64-unknown-linux-gnu`.

The standalone [review tool](../../tools/helm_app_spec_independent_review.py) records
argv, expected/actual exits, elapsed times, bounded result summaries and source
identities. Five expected compile failures and the original dependency sentinel
failure are successful negative controls, not silently retried product passes.

The [independent hosted run](https://github.com/Djomla83/helm-os/actions/runs/34224397648)
completed **successfully** at `c9d6c42b331c65805a256514dcd3e0d83a7189f3`.
The final report commit changes documentation/receipts only; product, tests,
fixtures, manifests, tools and workflow bytes match this passing commit.

| Check | Windows | Hosted Linux |
|---|---|---|
| Corrected workspace tests | 86 passed, none failed/ignored | 86 passed, none failed/ignored |
| Unchanged original candidate workspace tests | 78 passed | 78 passed |
| Base and standalone evidence regression | 45 passed each | 45 passed each, including Linux native filesystem probe |
| New targeted/property tests on original and corrected parser | 8 passed each | 8 passed each |
| Public API misuse checks | Five expected compiler rejections | Same five expected rejections |
| Instrumented CPU-dispatch regression | Original sentinel reached; corrected avoided | Same result |
| fmt, clippy all targets/features, locked builds | Passed | Passed |
| Python regressions, docs, fixture checks | 37 tests and checks passed | 37 tests and checks passed |

The 86 workspace count is 38 app-spec integration tests, three app-spec compile-fail
doctests and 45 evidence tests; platform-specific evidence test selection differs
but the total happens to match. Actual logs identify those platform-specific tests.
The Windows workspace receipt retains its pre-commit HEAD metadata: its recorded
product/test/fixture/manifest/workflow hashes match the published correction.
Independent Windows and Linux receipts both record the actual published correction.
Separate final Windows release invocations also passed after the workspace build.

The bounded independent property pass uses seed `00212026090851a7`: 4,000 cases
each of multiple mutations of both fixtures, arbitrary bytes, generated valid IDs,
nested decoded-duplicate keys, and random-width/depth budget trees. The 20,000
cases produce 4,087 successes and 15,913 rejections, with no panic or nondeterminism
on repeated calls. Complete input/result Debug transcripts, including A0 and notes
models and key negative diagnostics, have identical SHA-256 on original/corrected
Windows/Linux:
`0464894558755ebfe5cc3c5e263e46a85b7fb7c7ab5e7d4953de922027616744`.
All runs completed within the tool's 180-second property-command bound. This is
bounded seeded testing, not exhaustive fuzzing or a per-input deadline guarantee.
No crash/semantic-defect input was found in the uninstrumented property pass.

The five exact-byte identities, all inventoried source bytes and complete evidence
reports also compare equal across Windows/Linux. The evidence reports' historical
A0 experimental FAIL remains unchanged. Linux performance independently confirms I1:

| Linux operation | Base | Corrected combined | Corrected standalone |
|---|---:|---:|---:|
| A0 verification | 0.436 ms | 0.528 ms | 0.431 ms |
| Synthetic verification | 0.194 ms | 0.207 ms | 0.193 ms |
| Synthetic plus 4 MiB verification | 3.530 ms | 15.501 ms | 3.550 ms |
| SHA-256, 64 KiB | 46.660 us | 230.075 us | 46.986 us |
| SHA-256, 4 MiB | 2.983 ms | 14.736 ms | 2.987 ms |

Thus combined 4 MiB hashing costs about 4.94x and larger-bundle verification 4.39x
on this runner; standalone results remain near base. The final Windows rerun
likewise gives 4 MiB hash medians 1.761 / 10.537 / 1.767 ms and larger verifier
6.763 / 15.462 / 6.890 ms. Different hardware and I/O prohibit interpreting the
Windows/Linux timing difference as a platform semantic defect.

Reproduction commands include:

```text
cargo tree --workspace --locked -e features -i sha2
cargo tree -p helm-evidence --locked -e features -i sha2
cargo test --workspace --locked
cargo test -p helm-evidence --locked
cargo build -p helm-app-spec --release --locked
cargo build -p helm-evidence --release --locked
python tools/check_helm_app_spec.py --output target/helm-app-spec-independent/workspace.json
python tools/helm_app_spec_independent_review.py --output target/helm-app-spec-independent/review.json
```

The independent command archives the exact base/candidate under ignored `target/`,
tests original bytes, adds the independent tests only to that temporary copy,
compares all results and uses isolated dependency instrumentation. It neither
checks out nor modifies main. The broad workspace-check helper intentionally also
exercises a combined release build; finish with the separate evidence build when
using its CLI artifact.

Published evidence:

- [Corrected Windows workspace commands](evidence/helm-app-spec-independent-2026-09-08/corrected-windows-workspace.json).
- [Corrected Windows independent comparison](evidence/helm-app-spec-independent-2026-09-08/corrected-windows-comparison.json).
- [Corrected Linux independent comparison](evidence/helm-app-spec-independent-2026-09-08/corrected-linux-comparison.json).
- [Cross-platform equality and source inventory](evidence/helm-app-spec-independent-2026-09-08/platform-comparison.json).
- [CI identity, steps and downloaded-log identity](evidence/helm-app-spec-independent-2026-09-08/ci-result.json).
- [Final ancestry, unchanged main/evidence and publication checks](evidence/helm-app-spec-independent-2026-09-08/final-publication.json).
- [Independent A0/lock/licence checks](evidence/helm-app-spec-independent-2026-09-08/provenance-and-dependencies.json)
  and [bounded advisory inventory](evidence/helm-app-spec-independent-2026-09-08/rustsec-summary.json).

Compact comparison receipts retain exact commands/exits, feature trees, test logs,
samples and report identities/summaries. Repeated large benchmark stdout is replaced
by its byte count/hash after full report equality was checked. Full local receipts
remain ignored; the full Linux receipt is reconstructible from CI's ordered JSON
string chunks. No build binaries, private provenance or downloaded artifacts are
committed. The receipts state their compaction and redaction scope.

## Dependencies, CI and limits

Direct pins remain serde 1.0.228, serde_json 1.0.149 and sha2 0.10.9. Full lockfile
package-record comparison, keyed by both name/version rather than name alone,
shows only app-spec's new workspace entry and edges relative to base. No third-party
version/checksum changed. Each direct package declares `MIT OR Apache-2.0`;
transitive licence metadata is inventoried, without accepting a HELM licence.

The bounded RustSec database tree check at
`bf25f6575a93a35f30796c65c0ed91bee7fa19fd` found no entries for serde, serde_json or
memchr; sha2's [RUSTSEC-2021-0100](https://rustsec.org/advisories/RUSTSEC-2021-0100.html)
affects 0.9.7 and is patched from 0.9.8, so does not apply to 0.10.9. The web package
pages for serde/serde_json did not load; the complete non-truncated upstream Git
tree supplied that limited inventory instead. No broad advisory audit, security
certification, dependency upgrade or licence decision is claimed.

CI remains GitHub-hosted `ubuntu-24.04`, contents:read, immutable checkout action,
non-persisted credentials, pinned Rust 1.95.0, 15-minute timeout, no secrets,
deployment, signing, self-hosted runner or OS matrix. Existing module/manifest/lock
triggers are correct; new review-tool paths are included. Both crates are actually
tested, with the added standalone evidence regression and separate release builds.
The historical workflow filename/default-branch displayed name is cosmetic.

Not proved: exhaustive fuzzing, arbitrary targets, tight peak memory/runtime bounds,
uncompromised toolchains/allocators, truthful provenance of arbitrary referents,
authentication/freshness, filesystem containment, Wine behavior, second real-app
compatibility, or any observation/execution/recovery property. Combined-build SHA
performance remains the explicit boundary described in I1. No data permissions,
protected branch settings, accepted ADR or historical A0 evidence changed.

Development history: original author failures remain in the authoring report.
The new B1 regression failed as intended before the fix. The publication audit also
caught a local path duplicated in an unpublished review receipt's feature-tree
field; it was redacted, the unpublished review commit was replaced before the first
normal push, and the audit passed. No authoring or published history was rewritten.
The first post-fix workspace command set passed all 14 commands but its wrapper
failed that publication audit before writing its receipt; the final audited receipt
comes from a successful rerun. These are tooling/publication failures, not erased
application outcomes. A0 experimental FAIL remains unchanged.

**Blockers:** none remaining within the bounded reviewed scope.

**One recommended next decision:** owner review and merge decision for the corrected
`review/helm-app-spec-independent` branch, with I1's combined-consumer cost explicit. No further
module or background work is authorized by this report.
