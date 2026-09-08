# ADR-0021: Make the second product module an inert application contract

**Status:** Proposed\
**Draft date:** 2026-09-08\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

`helm-evidence` 0.1 is merged as the first experimental product module at authoritative main
`0b72e14f5d6101c281a8d9823168407da3e71b9a`. It verifies a declared evidence contract, not runtime
selection or execution. A0-7ZIP remains experimental FAIL. No architecture ADR was accepted by
that merge. The next dependency boundary must be falsifiable before introducing execution authority.

## Options

A: a bounded declarative app contract. B: a separate runtime/environment identity model.
C: a minimal lifecycle runner. The authoritative
[selection analysis](../research/SECOND-PRODUCT-MODULE-SELECTION.md) defines and scores all three,
traces the dependency DAG, and supplies the actual A0 example, proposed API/schema and falsifiers.

## Proposed decision

Recommend A, `helm-app-spec`: a pure library validating and identifying one application's desired
source, runtime artifact set, small environment declaration, entry-point reference and independent
verification references. Keep the necessary identity domain types inside this crate initially.
Do not add executable install/launch instructions or conflate desired state with observations.

This proposes only the next module boundary. It does not accept this ADR, authorise implementation,
choose a global catalogue/package format, change the broader product track, or accept another ADR.
The selection report contains the design sketch; no public API or command is implemented by it.

## Consequences and evidence

The proposal is reversible and requires no filesystem/process/network authority. It delays a
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

No existing ADR is superseded. Acceptance requires named owner review and a recorded decision.
