# helm-bind 0.1 independent review

**Role:** independent adversarial review of the first Rust implementation, by a reviewer who
is not the implementation author.\
**Reviewed candidate:** `832a112decaa333ee3b618d52e966a20c443513e` on `product/helm-bind`.\
**Review branch:** `review/helm-bind-independent`, descending from the candidate, which is
preserved unchanged.\
**Authority:** Accepted [ADR-0023](../adr/ADR-0023-binding-authority.md), bounded 0.1
architecture scope, with the owner's 2026-09-09 corrections. Nothing here weakens it.

**Classification: pending — first pass recorded, corrections not yet applied.**

This section is committed **before** any correction, so the candidate as submitted is
preserved in history rather than tidied away. `832a112` itself is unchanged.

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
