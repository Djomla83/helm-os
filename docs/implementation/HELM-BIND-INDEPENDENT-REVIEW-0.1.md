# helm-bind 0.1 independent review

**Role:** independent adversarial review of the first Rust implementation, by a reviewer who
is not the implementation author.\
**Reviewed candidate:** `832a112decaa333ee3b618d52e966a20c443513e` on `product/helm-bind`.\
**Review branch:** `review/helm-bind-independent`, descending from the candidate, which is
preserved unchanged.\
**Authority:** Accepted [ADR-0023](../adr/ADR-0023-binding-authority.md), bounded 0.1
architecture scope, with the owner's 2026-09-09 corrections. Nothing here weakens it.

**Classification: READY_FOR_OWNER_MERGE.**

> **Owner annotation, 2026-09-09.** The owner accepted this recommendation and fast-forwarded
> main from `60a0e16962ac4fcd0af8e545a33bc7ded9bcc4b6` to the reviewed tip
> `4f51c1b2bc7e59b8142ccc4c328e2641c89327d3`. **No finding, verdict, limitation or wording of
> this review was edited by the acceptance**; section 26's closing statement that main was
> untouched describes the state at the time of review, which is preserved. Acceptance merges an
> experimental module: schema, API and numeric limits stay unstabilised, `publish = false`, and
> the residual limitations in section 25 stand unchanged. See the
> [acceptance record](../PROJECT_STATE.md#helm-bind-owner-acceptance).

The findings in section 3 were committed **before** any correction (`564cbc4`), so the
candidate as submitted is preserved in history rather than tidied away. `832a112` itself is
unchanged. No BLOCKER was found; the two IMPORTANT findings are corrected on this branch and
each is now enforced by a test that establishes the property it states.

## 1. Independently reconstructed state

| Claim | Method | Result |
|---|---|---|
| `origin/main` is the accepted base | `git rev-parse origin/main` | `60a0e16962ac4fcd0af8e545a33bc7ded9bcc4b6`, as instructed; no newer owner work |
| the exact four-commit lineage | `git log --parents 60a0e16..832a112` | `706e276` → `932631a` → `86ab17f` → `832a112`, each the single parent of the next |
| strict descendant, no divergence | `git merge-base`, `git rev-list --count 832a112..origin/main` | merge base equals main exactly; zero main-only commits |
| no merge commit | `git rev-list --merges 60a0e16..832a112` | zero |
| no force rewrite | all four cited commits resolve and are reachable | confirmed |
| main contains no helm-bind | `git ls-tree origin/main crates/` | only `helm-app-spec`, `helm-evidence`, `helm-observe` |
| sibling product sources byte-identical | `git diff origin/main..832a112 -- crates/helm-app-spec crates/helm-observe crates/helm-evidence` | **empty** |

Read in full: `AGENTS.md`, `docs/PROJECT_STATE.md`, `docs/DECISIONS.md`, ADR-0021, ADR-0022,
[ADR-0023](../adr/ADR-0023-binding-authority.md), the
[design report](../research/HELM-BIND-ARCHITECTURE.md), the
[author review](HELM-BIND-REVIEW.md), every `crates/helm-bind` source, test and fixture, the
current `helm-app-spec` model and validator, the current `helm-observe` plan, artifact and
result contract, the Cargo manifests and lock, and both workflows.

## 2. Independent dataflow trace

| Transition | Controlled by | Can fail | I/O | Alternate route | Caller strings acquiring semantics |
|---|---|---|---|---|---|
| bytes → `ValidatedBindingPlan` | untrusted mapping bytes | yes, `BindingPlanErrors` | none | none: `parse_binding_plan` is the only constructor | no; identifiers stay opaque |
| four inputs → `precheck` | all four | yes, `BindingRefusal` | none | none: `bind` calls it first and returns on error | no |
| spec → claim universe | **specification only** | no | none | none | roles and DLL names become claim *keys*, never comparison values |
| claim → typed comparator | claim kind, fixed by the mapping's closed vocabulary | no | none | none: five arms, each reading its own desired field | no |
| comparator → `ClaimState` | desired field plus one observation outcome | no | none | none | no |
| states → coverage and contradiction | claim states only | no | none | none | no |
| record → exact bytes | the record | no | none | none | role and DLL selectors are echoed verbatim; see IMPORTANT‑1 |
| bytes → SHA-256 | the bytes | no | none | none | no |

Whole-crate search over `crates/helm-bind/src`:

| Searched for | Result |
|---|---|
| `std::fs`, `std::process`, `std::net`, `std::env`, `std::time`, `Command` | **none** |
| `Path`, `PathBuf` | **none** |
| `RawFd`, `OwnedFd`, `rustix`, `libc` | **none** |
| `unsafe` | **none**; `#![forbid(unsafe_code)]` |
| randomness, clock, current directory, environment variables | **none** |
| `cfg` on product semantics | **none**; the only `cfg` is `#[cfg(test)]` |
| `HashMap`/`HashSet` iteration in serialization | **none**; fixed order, `BTreeSet` for key checks |
| build script | **none** |

**Verdict: no product authority and no alternate route.** Confirmed by source and by the
absence of any I/O API in the dependency surface the crate actually uses.

## 3. Findings

### BLOCKER: none.

### IMPORTANT-1 — the report-vocabulary claim is stronger than the truth. **Reproduced.**

**Location:** `crates/helm-bind/README.md`, "Report identity" section, last sentence;
`crates/helm-bind/src/model.rs`, test `the_report_carries_no_verdict_vocabulary_and_no_path`,
the `for banned in ["snapshot", "verified", "ready_to_launch", "installed"]` substring loop;
`docs/implementation/HELM-BIND-REVIEW.md` §3, the "No satisfaction, compatibility or
readiness verdict" row.

**What the documents say.** The README states that "the whole report is lowercase by
construction, so a verdict token in any casing cannot appear", and the author's own test
asserts that the report bytes do not *contain* `snapshot`, `verified`, `ready_to_launch` or
`installed`. Both read as an invariant over all inputs.

**What is actually true.** `BindingReport::new` writes validated role and DLL selectors into
the bytes through `push_id`, which passes exactly the `helm-app-spec` identifier alphabet
through unchanged. `ready`, `pass`, `fail`, `verified`, `complete`, `snapshot`, `compatible`,
`satisfied` and `installed` are **legal** `helm-app-spec` identifiers, so a perfectly valid
specification can put any of them into a report.

*Reproduced*, not derived: `caller_selectors_reach_the_report_bytes_verbatim` builds such a
specification, binds it against a genuine observation, and the exact bytes contain
`"role":"ready"`, `"role":"verified"`, `"role":"satisfied"` and the rest. The same run also
showed the substring claim is unachievable for a second reason: the binder's own coverage key
`observation_failed` contains `fail`.

**The contract question, resolved.** ADR-0023 forbids a satisfaction, compatibility or
readiness **verdict**, and forbids success vocabulary in the report's own terms. It does not
speak about caller data echoed back under a `"role"` key. Reading A is therefore correct: a
role named `ready` is legitimate selector data that asserts nothing, exactly as a file named
`passed.txt` would assert nothing. Reading B — literal token absence for all inputs — is
unreachable without either rejecting valid `helm-app-spec` identifiers, which this review is
forbidden to do and which no architecture authorises, or obfuscating selectors, which would
damage the auditability the report exists for. **No architecture question arises**; the
implementation is right and the claim about it is wrong.

**Consequence.** No security hole and no semantic defect: a selector can never occupy a slot
the binder controls, because `push_id` cannot emit a quote, so no role can close its string
and impersonate a claim class or a state. The damage is auditability: a reader of the README
would believe a property that does not hold, and the author's test creates that impression
from a single fixture.

**Minimum correction.** State the true property in the README and the author review — *no
term the binder itself emits is a verdict word* — and replace the fixture substring loop with
an assertion of that property against a specification whose selectors deliberately **are**
verdict words.

**Regression test.** `caller_selectors_reach_the_report_bytes_verbatim`, which enumerates
every term the binder emits and requires that none of them is a verdict word, that every
occurrence of a verdict word is either selector data in a role slot or a substring of a
binder term, and that no selector reaches a `claim` or `state` slot.

### IMPORTANT-2 — the cross-platform evidence claim is stronger than the run. **Log-derived.**

**Location:** `crates/helm-bind/README.md`, "Authority" section: *"Given the same four
validated inputs the exact `BindingReport` bytes are identical on Linux, Windows and macOS,
and CI checks that on all three rather than asserting it."*

**What the run actually did.** Run `34326971427` executed, on `ubuntu-24.04`, `windows-2025`
and `macos-15`: `cargo fmt --check`, clippy, and `cargo test -p helm-bind`. On Windows and
macOS that suite is the crate's unit tests plus the binding-plan contract tests. The
end-to-end `bind` tests are gated to Linux x86_64, because `ObservationArtifact` has **no
public constructor** anywhere else. What the three runners agreed on was the serialization of
a **manually constructed** record — the same 1497 bytes and digest
`d63d04e2…` — not `bind()` over four genuine inputs.

**Consequence.** The sentence claims CI demonstrated something it did not. The underlying
determinism claim is still well founded, for reasons the sentence does not give.

**Decision on the evidence question.** Source-derived purity plus three-platform execution of
the serializer, the claim universe, the algebra and the parser is **sufficient** for the
accepted "given identical inputs" determinism claim, and the end-to-end limitation is
retained explicitly. Three things support that: `bind` contains no platform-dependent
construct at all, verified by the section 2 search; the only step that could differ between
platforms is serialization, and that is exactly what the three runners execute identically;
and closing the gap would require a new public constructor or parser in `helm-observe`, which
this review is explicitly forbidden to add. **This is a wording defect, not an architecture
question.**

**Minimum correction.** Say what was executed where, in the README and in the author review.

**Regression test.** None is possible for a wording claim; the correction is the fix, and the
existing three-platform job remains the evidence for what it does cover.

### MINOR findings

| # | Location | Finding | Disposition |
|---|---|---|---|
| M1 | `bind.rs`, `body_claim` | Returns `NotObserved` both when the mapping supplies no entry and when a mapped target has no entry in the artifact. The second is a different fact. It is **unreachable** today, because the prechecks bind the artifact to the exact plan and `helm-observe` emits exactly one observation per plan target, but the collapse is written into the code | Correct to the conservative `ObservationNotInterpretable`, which changes no reachable behaviour and removes the collapse |
| M2 | `HELM-BIND-REVIEW.md` §3 | Calls the coverage theorem a property test over "every accepted spec"; the test iterates two committed fixtures. The **implementation** does guarantee it structurally, see section 5 | Correct the claim; the reviewer's generator now establishes the universal property |
| M3 | `model.rs` test | `text.split('"').skip(1).step_by(2).chain(text.split(':').skip(1))` mixes two tokenisations and yields junk tokens; harmless but imprecise | Folded into the IMPORTANT‑1 correction |
| M4 | `plan.rs`, `Digest::parse_hex` | Indexes `bytes[2 * i]` after a `text.len() != 64` guard. Sound for any input, because the length check precedes it and a 64-byte string always has those indices, but the safety depends on that ordering | No change; recorded so a future edit does not reorder it |

### NO FINDING

The parser; exact mapping identity; the claim universe and its order; refusal atomicity and
precedence; typed-domain separation; body comparison; entry-point root and path rules; the
optional-digest precedence; observation-state distinctness; the contradiction algebra; the
serializer's injectivity and bound; the four-digest identity; privacy; dependencies and
purity; scope. Sections 4 onward record the evidence.

**All eighteen independent adversarial tests passed against the unmodified candidate.** The
two failures the first CI run showed were defects in the reviewer's own test expectations —
an over-strict substring predicate and a wrong `not_observed` count — corrected in the
reviewer's tests, with the implementation's behaviour left alone in both cases.


---

## 4. Public API and construction boundary

`ValidatedBindingPlan` and `BindingReport` both hold private fields. The plan's only
constructor is `parse_binding_plan`; the report's `new` is `pub(crate)`. Neither derives or
implements `Deserialize`, `Default` or `From`, and the only `DeserializeSeed` in the crate is
the zero-sized internal `StrictScan` visitor, which produces `()`. `bind` takes exactly the
four accepted input types by reference and every accessor returns an immutable borrow or a
`Copy` value, so a report cannot be mutated after it exists.

The attempt required by the instruction — a downstream program that manufactures a successful
report without calling `bind` — is **impossible through the safe public API**, and this is now
enforced rather than argued: two `compile_fail` doctests, one per type, try to build a value
with struct-update syntax and are required to fail compilation. They ran green on all three
platforms.

## 5. Universal coverage theorem — established, and the author's evidence was weaker than claimed

The implementation makes the property **structurally unavoidable**. `semantic_claim_subjects`
pushes `SourceArchitecture`, `RuntimeFamily`, `WindowsArchitecture` and `PrefixRole`
unconditionally, before any input-dependent branch, and `evaluate` maps all four to
`UnsupportedBinding` in a single arm that consults neither the mapping nor the observation.
There is no path through the function that omits any of them.

The author's in-crate test nevertheless iterates two committed fixtures, while the author
review calls it a property over "every accepted spec". That gap is recorded as MINOR‑2 and is
closed here by
`every_valid_specification_has_at_least_four_unsupported_claims`, which generates valid
`helm-app-spec` documents across the schema boundaries — 1, 2 and 16 runtime artifacts × 1, 3
and 8 verification definitions × 0, 1 and 8 disabled DLLs × entry-point digest present and
absent, 54 combinations — parses each with the real validator, binds each against a genuine
observation, and asserts for every one:

- the four mandatory classes are present and `UnsupportedBinding`;
- `unsupported_binding == 4 + disabled_dll_count`, so the bound is not merely met but exact;
- `claims == 1 + 4 + artifacts + dlls + 2 + definitions`, recomputed independently;
- `claims <= 39`, with the maximum specification reaching exactly 39;
- the ten counters sum to `claims`, so coverage partitions the claim set;
- no outcome exists for contextual metadata or a selector key.

**Verdict: the universal theorem holds, by structure and by generated evidence.** The
implementation was never at fault; only the description of its test was.

## 6. Claim universe and order

Fixed and derived from the specification alone, independently confirmed for both the committed
fixtures and every generated document: source body, source architecture, runtime family, each
runtime artifact body in declaration order, Windows architecture, prefix role, each disabled
DLL in declaration order, entry-point presence, entry-point body, each verification definition
in declaration order. The specification digest, application ID, application version, artifact
labels and the role selectors themselves receive **no** outcome and are outside the coverage
denominator. Each disabled DLL is exactly one `UnsupportedBinding` claim instance.

## 7. Refusal atomicity and precedence

`bind` calls `precheck` first and returns on error, before any claim, coverage counter or
record exists, so a refusal cannot leave a partial report behind — that is structural, not
incidental. All eight refusals were re-exercised independently and each yields `Err`, no report
value of any kind and therefore no bytes and no digest.

Precedence is total and was verified with several conditions wrong at once:

1. the mapping's declared subject;
2. the observation's declared subject;
3. the artifact-to-plan pairing;
4. the asserted prefix root's existence;
5. then, in mapping declaration order, per claim: unknown target, unknown role, incompatible
   observable, entry-point root mismatch.

Tested: a mapping that is both foreign-subject and dangling yields the subject refusal; a
claim that is both unknown-target and unknown-role yields the target refusal; two broken claims
yield the first in mapping order.

## 8. Typed domains

There is no generic comparator anywhere. The claim kind selects the desired field inside
`evaluate`, and the five arms are the only readers. Attacked with a specification in which the
source, one runtime artifact and one verification definition declare the **same** body
identity, and one role name is shared between the runtime and verification namespaces: an
agreeing runtime artifact left the source and definition claims `NotObserved`, and the shared
role produced `Match` under one kind and `Mismatch` under the other. **Cross-domain
comparison is inexpressible, not merely discouraged.**

## 9. Body comparison

The full truth table was exercised with independent expected values: equal size and digest is
`Match`; differing size alone is `Mismatch(size)`; differing digest alone is
`Mismatch(digest)`; both is `Mismatch(size_and_digest)`. `FileFacts::bytes_read` is the
observed length under `helm-observe`'s completed-stream contract, and no desired size is
invented anywhere — in particular none is invented for the entry point, which the schema does
not declare. No body match carries any installation, provisioning, loading, execution or
runtime-selection meaning in the report vocabulary.

## 10. Entry-point root and path

Verified independently across the matrix: an entry mapping with the assertion absent and the
assertion present without an entry mapping are both refused at parse time; an unknown asserted
root and an entry target under another root are typed refusals, not claim results; the correct
root with a differing relative path is `Mismatch(entry_point_path)` **before** the observation
outcome is consulted; identical bytes at another relative path never bind the entry point; a
**case-only** path difference is a mismatch, so nothing is folded; presence and body may share
one target; and a target named `entry` gets no special treatment while a target named anything
else works identically.

## 11. Optional entry-point digest precedence

The implementation evaluates "the specification states no digest" first, after the global
prechecks. Exercised with the digest unspecified against: no body mapping, a correct mapping, a
wrong-path mapping, and an absent target. All four yield `DesiredValueUnspecified` and none
contradicts. Separately, a structurally broken mapping with the digest still unspecified is
refused with `UnknownRoot`, so an unstated desired value never suppresses a cross-document
refusal.

**Verdict: coherent and correct.** Promoting a path difference here would raise a
contradiction for a claim the specification never made, which is the worse error; the presence
claim still reports the path difference when it is mapped.

## 12. Observation-state taxonomy

Every reachable outcome keeps its own state and its own counter, confirmed against genuine
observations: absence, observer rejection with its typed reason, observer failure with its
typed reason, budget omission with its typed reason, and the observed-file comparison. Absence
is not a mismatch and does not contradict; a failure is not a mismatch; a rejection is not a
failure; an unmapped claim is neither absence nor unsupported binding; unsupported binding is
never a match.

## 13. `#[non_exhaustive]` handling — source-derived

`TargetOutcome`'s wildcard arm yields `ObservationNotInterpretable`. Reading the arm shows it
cannot yield `Match`, `Mismatch`, `NotObserved` or `UnsupportedBinding`: those are produced
only by named arms above it. The reason enums map an unknown reason to `Unrecognised` **inside
the known outcome kind**, which is coherent — the outcome kind is known and only the reason is
not, so collapsing it to `ObservationNotInterpretable` would discard true information. No
unsafe and no fake constructor was added to reach the branch; it is marked source-derived, and
the correction in MINOR‑1 removed the one place where a missing artifact entry could have
reached a benign state instead.

## 14. Contradiction and coverage algebra

Re-derived independently. `Contradicted` if and only if at least one claim is `Mismatch`, with
`mismatches` equal to the number of such claims. The author's property test covers all 512
subsets of the nine non-mismatch states crossed with four mismatch counts; the independent
suite adds the end-to-end direction over genuine observations, confirming that absence,
rejection, failure, omission, unsupported binding, an unmapped claim and an unspecified desired
value all leave `NoClaimContradicted` standing. The two axes are independent, and no code
anywhere derives a "good", "success" or "satisfied" notion from them — there is no such
function to derive it with.

## 15. Selector and vocabulary injection

See IMPORTANT‑1. Resolved as reading **A**: ADR-0023 forbids verdict vocabulary in the
binder's own terms, and caller selector data echoed under a `"role"` key is not a verdict.
The structural protection is real and was verified: `push_id` cannot emit a quote, so a
selector cannot close its string and reach a `claim` or `state` slot. The documentation and the
test now state that property instead of an unachievable one. No otherwise-valid
`helm-app-spec` role name is rejected by `helm-bind`.

## 16. Serializer, injectivity and bound

Fixed field order, no map iteration, integers rendered decimally, enumerations from closed
sets, valid JSON, and **no truncation call anywhere**. `push_id` accepts exactly the
`helm-app-spec` identifier alphabet — lowercase ASCII, digits, `.`, `_`, `-` — and the DLL
alphabet is a strict subset of it, so **no currently valid input loses a character**, and two
distinct legal selectors cannot serialize identically. Runtime and verification roles stay
distinguished by claim kind rather than by role, which the shared-role test in section 8
confirms end to end. The widest expressible 39-claim report with maximum-length identifiers is
proved by unit test to fit `MAX_REPORT_BYTES`, and the real 20-claim selector-attack report
measured 1,438 bytes.

## 17. Four-digest report identity

Each recorded identity was checked against an independent SHA-256 oracle over the exact bytes
of its own document, and `report.sha256()` against an oracle over the report's own bytes.
Changing only the binding plan's bytes, with an identical semantic result, moves the report
identity; changing only the observation artifact's bytes, again with an identical semantic
result, moves it too. The graph is acyclic: nothing consumes the report's own digest. A wrong
subject or a wrong plan pairing refuses rather than producing an identity.

## 18. Privacy

No observation relative path, host absolute path, cwd, hostname, timestamp or platform value
appears in report bytes, verified with adversarial logical IDs; the report contains no `/` at
all. Refusal codes are fixed strings and a refusal carries at most one validated logical
identifier. The test harness prints fixture paths in failure messages, which is developer
diagnostic output and not artifact content; the distinction is deliberate.

## 19. Cross-platform evidence — exactly what ran

**Executed on Ubuntu, Windows and macOS** (run `34355861658`): `cargo fmt --check`, clippy with
warnings denied, and `cargo test -p helm-bind`, which on those platforms covers the crate unit
tests, the author's binding-plan contract suite, the independent parser suite and both
`compile_fail` doctests. All three runners produced the identical fixed-record report: 1,497
bytes, digest `d63d04e2a55a61fa149729f48bc355fb18f28e309f8aa2c973589391bf80e923`.

**Executed on Linux only** (run `34355861835`): `bind` over four genuine inputs — the author's
11 end-to-end tests and the independent suite's 9.

**Not executed anywhere**: `bind` over four genuine inputs on Windows or macOS. It cannot be,
because `ObservationArtifact` has no public constructor outside `helm-observe`'s Linux backend,
and adding one is outside this review's authority.

**Verdict: sufficient, with the limitation retained.** The comparison contains no
platform-dependent construct at all, the only platform-sensitive step is serialization, and
serialization is exactly what the three runners execute identically. This is an evidence-scope
limitation to keep stating, not an architecture question, and it is now stated accurately in
both the README and the author review.

## 20. Test quality

Reviewed for false confidence. No tautological assertion, no shared helper between
implementation and oracle — digests are computed with `sha2` directly in the tests — no
expected value derived from the code under test, no length-only comparison, no test that
cannot fail, no environment-dependent skip counted as a pass, and no retry. The pinned fixture
digest was measured once and is re-measured on three runners every run, which is the right
shape for a determinism anchor. The history in which the A0 claim count was wrong and was
corrected is preserved, not rewritten.

Two overstatements were found and are recorded as MINOR‑2 and IMPORTANT‑1: a two-fixture test
described as a universal property, and a fixture substring assertion described as an invariant.
Both are corrected, and both are now backed by tests that actually establish the stated
property.

## 21. Dependencies and purity

Exactly `helm-app-spec`, `helm-observe`, `serde`, `serde_json`, `sha2`. No `helm-evidence`, no
`rustix`, no `libc`, no utility crate, no build script, no platform-dependent serializer
dependency, `unsafe` forbidden, no ambient I/O. The `sha2` `force-soft` unification is
performance and build composition only — the feature selects an implementation, not a digest —
and it was not touched. The existing separate package builds and the standalone
`cargo test -p helm-evidence` are retained.

## 22. Scope

No product output claims installation, compatibility, launch, Wine discovery, prefix ownership,
verification execution, evidence completeness, runtime provenance or package state. The only
matches for that vocabulary in product source are the refusal code
`INCOMPATIBLE_OBSERVABLE` and prose that explains what is **not** concluded. No `helm-launch`
work exists.

## 23. Corrections made on this branch

| Commit | Content |
|---|---|
| `e9daf4e` | The independent adversarial suites, committed before any correction |
| `e42e1ac`, `fe0ce62` | Two defects in the reviewer's own test expectations, corrected in the reviewer's tests with the implementation left alone |
| `564cbc4` | First-pass findings, recorded **before** any correction |
| `78e26de` | IMPORTANT‑1, IMPORTANT‑2, MINOR‑1, the `compile_fail` doctests, and the annotation on the author review |
| this commit | This report and the `PROJECT_STATE` entry |

`832a112` and the whole candidate lineage are preserved unchanged. No amend, rebase, squash or
force-push; main was not touched.

## 24. Validation on the corrected tip

Both workflows green: `34355861835` (Linux workspace) and `34355861658` (three-platform
purity). `cargo fmt --check`; workspace clippy with warnings denied; `cargo test --workspace
--locked`; `cargo test -p helm-bind --locked` — 7 unit, 11 author end-to-end, 14 author
contract, 9 independent Linux, 9 independent parser and 2 `compile_fail` doctests; the
standalone `cargo test -p helm-evidence` and both release package builds; 37 repository Python
tests; the documentation validator; the app-spec and evidence frozen fixtures; `git diff
--check`; and the privacy and artifact checks inside the suites above.

## 25. Residual limitations

1. **`bind` is not executed end to end off Linux**, section 19. Structural, and closing it
   needs a `helm-observe` API change that is not authorised here.
2. **The `#[non_exhaustive]` future-variant branches are source-derived**, section 13. No safe
   way exists to construct a future variant of another crate's enum, and none was faked.
3. **`EXDEV`-class and other observer states that need a mount or an old kernel** are not
   reachable from these tests; they arrive as `ObservationRejected` or `ObservationFailed`
   through the same code path as the states that are exercised.
4. **The generated specification corpus is systematic, not random**: 54 combinations at the
   schema boundaries rather than a fuzzed space of documents. The property it establishes is
   structural, so this is adequate, but it is enumeration rather than search.
5. **`Coverage` derives `Default` and has public fields**, so a caller can build one; it cannot
   become a report, and nothing consumes a caller-built `Coverage`.
6. **Seeds are recorded**: `0x0117202609090001` for the independent binding-plan corpus, 2,048
   cases. No seed was retried to obtain a pass.

## 26. Recommendation

After the corrections on this branch, `helm-bind` 0.1 is a faithful realisation of Accepted
ADR-0023. The comparison is pure and authority-free; the claim universe, the algebra, the typed
domains, the entry-point rules, the refusal atomicity and the identity graph all behave exactly
as accepted, and were verified independently rather than taken from the author's suite. No
BLOCKER was found, and the two IMPORTANT findings were claims about the code rather than
defects in it — both corrected, both now enforced by tests that establish the property they
state.

**READY_FOR_OWNER_MERGE.**

This is a reviewer recommendation. The merge is the owner's action; nothing here performs or
authorises it, and main is untouched at `60a0e16962ac4fcd0af8e545a33bc7ded9bcc4b6`.
