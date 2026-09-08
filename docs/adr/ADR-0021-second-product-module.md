# ADR-0021: Make the second product module an inert application contract

**Status:** **Accepted — bounded module-selection decision only**\
**Draft date:** 2026-09-08\
**Approver:** Repository owner (Djomla83), by explicit written instruction of 2026-09-08\
**Acceptance date:** 2026-09-08

## Context

`helm-evidence` 0.1 is merged as the first experimental product module at authoritative main
`0b72e14f5d6101c281a8d9823168407da3e71b9a`. It verifies a declared evidence contract, not runtime
selection or execution. A0-7ZIP remains experimental FAIL. No architecture ADR was accepted by
that merge. The next dependency boundary must be falsifiable before introducing execution authority.

## Options

A: a bounded declarative app contract. B: a separate runtime/environment identity model.
C: a minimal lifecycle runner. The
[selection analysis](../research/SECOND-PRODUCT-MODULE-SELECTION.md) defines and scores all three,
traces the dependency DAG, and supplies the actual A0 example, proposed API/schema and falsifiers.

## Accepted decision and limits

The owner selected A, **`helm-app-spec`**, as HELM's second product module, with this exact decision:

> Build a pure, non-executable application specification/validation library before observation or lifecycle execution.

**Acceptance does not stabilise the eventual schema/API and does not authorise implementation.**
The requirements below constrain the bounded decision; the report's concrete field names, numeric
limits, API, internal layout and later-module sequence remain design sketches for separate review.
No other ADR is accepted or superseded, no product track or licence is selected, and no larger
Evidence Loop PoC, catalogue, package format or execution subsystem is authorised.

## Owner requirements for the later design

1. **Exact document identity.** A concrete app-spec document is identified by SHA-256 of its exact
   source bytes. Version 0.1 introduces neither canonical JSON nor semantic canonicalisation.
   Semantically equivalent but byte-different specifications may have distinct document identities.
2. **Desired state only — hard architectural boundary.** Declarations never constitute evidence
   that a runtime was installed, a particular loader executed, a prefix exists, installation
   succeeded, an entry point exists, or verification ran. Observed state belongs to a later
   module, currently expected to be `helm-observe`; that name and its implementation remain advisory.
3. **Bounded runtime artifact requirements.** Values such as `Wine 11.17` are metadata, not
   sufficient runtime identity. Requirements use a small bounded set of immutable artifact
   identities, each with a role/identifier, byte size and SHA-256, plus optional label/version
   metadata. An observed installed-loader hash is not proof of desired runtime state. Archive
   requirements and installed/loaded observations remain separate. No package acquisition or
   cross-distribution package model is included.
4. **Acyclic verification references.** The spec may reference frozen verification definitions,
   oracle/fixture definitions or equivalent pre-execution contracts. It must not reference a
   resulting evidence bundle that in turn identifies that spec. The identity-reference direction is:

   ```text
   app-spec -> frozen verification definition
   later execution/evidence -> exact app-spec byte identity
   later execution/evidence -> observed outputs/results
   ```

   These are references, not chronological dependency arrows. The spec's raw byte digest does
   not depend on future results; no `app-spec -> evidence bundle -> app-spec` identity cycle exists.
5. **Pure validation.** The library takes bytes and returns a validated model or validation errors.
   Normal validation performs no filesystem access, network access, process execution,
   Wine/runtime discovery, environment inspection, package lookup or prefix creation. Relative
   paths are inert validated data, never instructions or capabilities to resolve/open/execute them.
6. **Minimal typed environment vocabulary.** Include only concepts justified by A0 or the immediate
   next observation experiment: Windows architecture, dedicated-prefix intent/role, explicit
   DLL-disable requirements where needed, entry point and runtime artifact requirements. No
   arbitrary environment-variable/configuration language or speculative graphics/dependency knobs.
7. **Bounded runtime family.** Initial validation may support only Wine, with schema versioning
   for later extensions. Do not add Proton or generic runtime-provider machinery for future plans.
8. **Parser safety requirements, not implemented here.** Bound input size; reject unknown mandatory
   schema versions, unknown fields in the closed schema and duplicate/ambiguous declarations;
   use deterministic validation/error ordering; validate identifiers, digests and path spelling;
   avoid panics on malformed untrusted input. Exact limits and implementation need separate review.

## Consequences and evidence

The selected boundary is reversible and requires no filesystem/process/network authority. It delays a
standalone identity crate until real consumers justify extraction, and delays a runner until
runtime binding, prefix lifecycle, execution authority and capture contracts have evidence.
A validated declaration establishes no compatibility, execution or recovery guarantee.

[A0](../experiments/EXP-009-APP-BASELINE-REPORT.md#current-assessment) supplies exact source/package
identities and the wrong-destination counterexample. The
[independent evidence review](../implementation/HELM-EVIDENCE-INDEPENDENT-REVIEW.md) scopes what the
existing verifier actually checks. [G0-3b](../experiments/EXP-009-GATE0-REPORT.md#2a) supplies the
post-snapshot user-data-loss counterexample. All limits and proposed tests are in the selection
analysis; none of these results demonstrates the proposed module's usefulness by execution.

## Revisiting

Reject or redesign if A0 cannot be represented honestly, app-specific core branches are necessary,
requirements become observations, spec/evidence identities form a cycle, user-data ownership is
ambiguous for destructive use, or a separate pure identity component is independently shown to
be the prerequisite with useful consumers. Apply the report's full falsification criteria.

The original Proposed analysis is preserved in commit
`08f29c53095a51947e9662ad2d0d7931ccb606ae`. This dated owner decision refines it without rewriting
that history. Implementation still requires a separate owner instruction.
