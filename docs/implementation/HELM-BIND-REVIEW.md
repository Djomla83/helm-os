# helm-bind 0.1 implementation review (author)

**Disposition: READY_FOR_INDEPENDENT_REVIEW.** Implemented on `product/helm-bind`.
Not owner-merged, not released, and not independently reviewed.\
**Date:** 2026-09-09. **Authoritative base:** `60a0e16962ac4fcd0af8e545a33bc7ded9bcc4b6`.\
**Authority:** Accepted [ADR-0023](../adr/ADR-0023-binding-authority.md), bounded 0.1
architecture scope, with the owner's 2026-09-09 corrections applied.

This is the author's own review. It maps the implementation back to every accepted invariant
and records what was **not** done.

## 1. Crate boundary

One new workspace member, `crates/helm-bind`, library only, `publish = false`. Direct
dependencies: `helm-app-spec`, `helm-observe`, `serde`, `serde_json`, `sha2`. **No**
dependency on `helm-evidence`, no `rustix`, no utility or shared-contract crate, and no
reparsing or re-implementation of either sibling contract. `#![forbid(unsafe_code)]` holds
and the workspace lints (`unwrap_used`, `expect_used`, `panic` denied) apply unchanged.

Neither `helm-app-spec` nor `helm-observe` was modified in any way. `git diff` over their
source trees is empty.

## 2. Public API

```rust
pub fn parse_binding_plan(bytes: &[u8]) -> Result<ValidatedBindingPlan, BindingPlanErrors>;

pub fn bind(
    spec: &helm_app_spec::ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &helm_observe::ValidatedPlan,
    observation: &helm_observe::ObservationArtifact,
) -> Result<BindingReport, BindingRefusal>;

impl BindingReport {
    pub fn exact_bytes(&self) -> &[u8];
    pub fn sha256(&self) -> Digest;
    pub fn contradiction(&self) -> Contradiction;
    pub fn coverage(&self) -> &Coverage;
    pub fn claims(&self) -> &[ClaimBinding];
    pub fn inputs(&self) -> &InputIdentities;
    pub fn record(&self) -> &BindingRecord;
}
```

There is no public constructor for `ValidatedBindingPlan` other than parsing, none at all for
`BindingReport`, no `Deserialize` on any validated or output type, and no API anywhere that
accepts a caller-supplied expected hash.

## 3. Accepted-invariant map

| Accepted ADR-0023 invariant | Where | Tests |
|---|---|---|
| Pure, authority-free, no OS backend | whole crate; section 9 search | cross-platform CI on three operating systems |
| No satisfaction, compatibility or readiness verdict | `model.rs`: the vocabulary contains no such token | `the_report_carries_no_verdict_vocabulary_and_no_path` |
| Ten distinct claim states | `model::ClaimState` | `coverage_counts_sum_to_the_claim_count`, and the Linux state matrix |
| `Contradicted` iff at least one `Mismatch` | `model::Contradiction::from_states` | `only_a_mismatch_contradicts`, an exhaustive property test over all 512 subsets of the nine non-mismatch states crossed with four mismatch counts |
| Nothing else promoted to contradiction | same | same, plus `absence_rejection_failure_and_omission_never_collapse` |
| Coverage counts semantic claims only | `bind::semantic_claim_subjects` | `claim_order_is_fixed_and_excludes_contextual_metadata` |
| `unsupported_count >= 4` for every valid spec | four unconditional subjects in `semantic_claim_subjects` | `every_specification_carries_at_least_four_unsupported_semantic_claims`, over both fixtures, no observation needed |
| Maximum 39 semantic claim instances | claim universe arithmetic | same test, plus `the_widest_expressible_report_stays_inside_the_ceiling` |
| Inert closed `BindingPlan`, five kinds, no expected values | `plan.rs` | `unknown_schema_version_and_fields_are_rejected` rejects `expected_sha256` and `path` as unknown fields |
| Duplicate decoded keys rejected at every depth | `plan::StrictScan` seeded visitor | `duplicate_decoded_keys_are_rejected_at_every_depth`, including an escaped spelling inside an array-nested object |
| Exact-byte mapping identity, no canonicalisation | `plan.rs` | `identity_is_over_exact_bytes_with_no_canonicalisation`, checked against an independent `sha2` oracle |
| One mapping per semantic claim key | `plan.rs` duplicate-claim check | `a_semantic_claim_key_may_be_mapped_only_once` |
| Two entry claims may share one target | same | `both_entry_point_claims_may_share_one_target` |
| Conditional `asserted_prefix_root_id` | `plan.rs` presence rule | `the_prefix_root_assertion_is_present_exactly_when_an_entry_claim_is_mapped` |
| All prechecks before any claim | `bind::precheck`, called first | `every_broken_document_combination_is_refused_without_a_report` |
| Typed refusals, no partial binding | `error::BindingRefusalCode`, eight codes | same |
| Both entry claims require `regular_file_sha256` | `precheck` observable check | refusal test maps a claim to a `directory_metadata` target |
| Both entry claims bind the path byte for byte | `bind::entry_path_agrees` | `the_same_content_at_another_relative_path_never_binds_the_entry_point` |
| No manufactured entry-point digest | `entry_point_body` step 1 | `an_unstated_entry_point_digest_is_never_synthesised` |
| No target-ID heuristic | nothing reads a target's spelling | `a_target_id_spelling_carries_no_semantics` |
| Body comparison uses size **and** digest | `bind::body_claim` | `body_mismatches_distinguish_size_from_digest` |
| Four-digest acyclic report identity | `model::InputIdentities` | `the_report_binds_exactly_the_four_input_identities` |
| Deterministic serializer, no map iteration | `BindingReport::new`, fixed field order | `the_fixture_report_has_one_platform_independent_identity` on three platforms |
| No path in a report | only IDs and codes are serialized | asserted in two tests |

## 4. Claim universe, exactly as accepted

`semantic_claim_subjects` derives the universe from the specification alone, which is what
makes the coverage theorem checkable without any observation, on every platform. Order:

1. application source body
2. application source architecture — `UnsupportedBinding`
3. runtime family — `UnsupportedBinding`
4. each runtime artifact body, in declaration order
5. Windows architecture — `UnsupportedBinding`
6. prefix role — `UnsupportedBinding`
7. each disabled-DLL requirement, in declaration order — `UnsupportedBinding`
8. entry-point presence
9. entry-point body
10. each verification-definition body, in declaration order

The specification digest, the application ID, the application version, artifact labels and
role identifiers themselves receive **no** claim outcome and are **not** in the coverage
denominator. Maximum instances `1 + 1 + 1 + 16 + 1 + 1 + 8 + 1 + 1 + 8 = 39`.

## 5. Precedence, written down so it cannot drift

Refusals happen first, all of them, before any claim exists. Then per claim:

- an unsupported class is `UnsupportedBinding`, and is never mapped or compared;
- `EntryPointBody` with no desired digest is `DesiredValueUnspecified` **first**, before
  mapping and before path binding. The specification made no claim, so there is nothing to
  contradict, and promoting a path difference here would raise a contradiction for a claim the
  specification never stated. A path difference is still reported by the presence claim;
- otherwise an unmapped bindable claim is `NotObserved`;
- otherwise, for entry-point claims, a differing target path is `Mismatch` before the
  observation outcome is looked at;
- otherwise the observation outcome decides.

## 6. Observation outcome translation

`ObservedFile` compares typed known facts. `ObservedDirectory` is `ObservationRejected`
(wrong kind) — unreachable through a validated mapping, since the observable is checked
first, and kept explicit anyway. `Absent` stays `Absent`. `Rejected`, `Failed` and
`NotObserved` keep their own states with a bounded typed reason. Any future
`#[non_exhaustive]` variant becomes `ObservationNotInterpretable`, never `Match`,
`NotObserved` or `UnsupportedBinding`.

Within a known outcome kind, an unrecognised *reason* becomes `Unrecognised` rather than
`ObservationNotInterpretable`, because the outcome kind is still known; only the reason is
not. That distinction is deliberate and is offered to independent review.

## 7. Self-review falsifiers

Answered from source, not from intent.

| Question | Answer |
|---|---|
| Can omitted unsupported claims produce a success token? | **No.** No success token exists, and four unsupported claims are unconditional |
| Can spec A use an observation for spec B? | **No.** `ObservationSubjectSpecMismatch`, refused before any claim |
| Can an artifact from plan A be paired with plan B? | **No.** `ObservationPlanMismatch` |
| Can a caller provide an expected digest? | **No.** The closed schema has no such field; it is an unknown field |
| Can runtime archive bytes be reported as installed-loader identity? | **No.** The claim kind fixes which desired field is read; a report says `runtime_artifact_body` and nothing else |
| Can an entry point with the same content at the wrong path become `Match`? | **No.** Path binding precedes outcome interpretation, for both entry claims |
| Can an entry-point target under another root become a claim result? | **No.** `EntryPointRootMismatch`, a refusal |
| Can `Absent` become `Mismatch`? | **No.** Separate states; only value disagreement is a mismatch |
| Can an observation failure become `Mismatch`? | **No.** Separate state and separate counter |
| Can unsupported binding become `Match`? | **No.** The unsupported classes never consult a mapping |
| Can a target-ID spelling carry semantics? | **No.** Nothing in the crate inspects a target ID except by exact equality with a mapping's declared target |
| Can `bind` perform any I/O? | **No.** Section 9 search finds no I/O API in the crate |
| Can report identity depend on host, platform or time? | **No.** Fixed serializer, no clock, no host value; checked on three platforms |
| Can a refusal produce report bytes? | **No.** `precheck` returns before any claim or record is built |

No question answered YES, so this pass found no blocker.

## 8. Tests

**Cross-platform, 21.** Seven crate unit tests: the claim universe and the four-claim
theorem over both fixtures, fixed claim order, the widest-report bound, the deterministic
fixture-report identity, the contradiction property test, and the coverage partition.
Fourteen binding-plan contract tests: acceptance and declaration order, shared entry targets,
exact-byte identity against an independent oracle, escaped-spelling identity and escaped
duplicates, duplicate decoded keys at every depth, closed schema including rejected expected
values and paths, role presence rules, one mapping per claim key, the conditional prefix-root
assertion, every ceiling on both sides, identifier and digest grammar, malformed input
without panic, a 1024-case seeded corpus with a reviewer-chosen seed, and bounded private
diagnostics.

**Linux x86_64, 11.** End-to-end against **genuine** observations produced by `helm-observe`
over a synthetic tree whose bodies reproduce the committed fixture identities: a complete
correct mapping, unmapped claims, typed size and digest mismatches, the no-heuristic test,
both entry-point path rules, the unstated-digest case on the A0 fixture, every observation
outcome kept distinct, all eight refusals, the four-digest identity, and claim ordering.

Fixtures are the committed `synthetic-notes.json` and `a0-7zip.json`. Nothing branches on
7-Zip, a Wine version or an A0 role name; A0 is used only as a regression input.

**Cross-platform evidence, measured not asserted.** Run `34326971427` ran the crate on
`ubuntu-24.04`, `windows-2025` and `macos-15`. All three produced the identical fixture
report: **1497 bytes, `d63d04e2a55a61fa149729f48bc355fb18f28e309f8aa2c973589391bf80e923`**.
Run `34326971463` ran the full Linux workspace, including the 11 end-to-end tests.

## 9. Purity search

| Searched for | Result in `crates/helm-bind/src` |
|---|---|
| `std::fs`, `std::process`, `std::net`, `std::env`, `std::time` | **none** |
| `Path`, `PathBuf` | **none** |
| `RawFd`, `OwnedFd`, `rustix`, `libc` | **none** |
| `unsafe` | **none**; forbidden crate-wide |
| `cfg` on product semantics | **none**; the only `cfg` is `#[cfg(test)]` |
| global mutable state, `static mut`, `lazy_static` | **none** |
| `HashMap` iteration in serialization | **none**; fixed field order and `BTreeSet` for key checks |

## 10. Known limitations

- **End-to-end `bind` tests are Linux x86_64 only.** `ObservationArtifact` has no
  constructor outside `helm-observe`'s observation backend, so a genuine artifact cannot be
  produced elsewhere, and adding one would be a change to `helm-observe`'s API that this task
  does not authorise. The binding-plan parser, the claim universe, the verdict algebra and the
  report serializer — including its exact fixture digest — are tested on all three platforms,
  and the comparison logic contains no platform-dependent construct. Independent review should
  attack this gap directly, and the owner may wish to decide whether `helm-observe` should
  gain a parsing or test constructor later.
- **The `#[non_exhaustive]` future-variant path is covered by a wildcard arm, not by a
  constructed unknown variant.** No unsafe or fake public constructor was added to reach it.
  Independent review should attack it.
- **`Unrecognised` reasons** keep a known outcome kind with an unknown reason, rather than
  collapsing to `ObservationNotInterpretable`. Deliberate; offered for review.
- **`sha2` feature unification** now necessarily applies to any binder-containing graph.
  Performance and build composition only; digests are identical either way. Existing separate
  package builds and the standalone `cargo test -p helm-evidence` are retained.
- Schema, API and limits are experimental and unstabilised.

## 11. Not done, deliberately

No `helm-launch`. No change to `helm-app-spec` or `helm-observe`, in semantics, API or
dependency features. No filesystem, network or process authority in product code. No lab or
VM execution, no A0 rerun, no Wine, no 7-Zip. No merge to main. No new system experiment: the
module is pure, so adversarial and property tests are the falsification mechanism, exactly as
ADR-0023 accepted. **A0-7ZIP remains experimental FAIL.**
