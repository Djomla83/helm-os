# ADR-0015: Evidence must be reproducible, machine-generated, gating, and must expire

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

The audit set out to find whether HELM's evidence pillar is occupied. It is not, but only in a
narrowed form, and the reasons each near-miss fails are specific and checkable.

- **WineHQ AppDB** has genuinely structured columns, but its entire hardware dimension is a
  five-value GPU-vendor enum plus a three-value driver enum; there is no application build hash, no
  attached artifacts, no reproducible procedure, and no expiry. Its freshness mechanism is social —
  it removes inactive maintainers — and nothing ever invalidates a result. It also has no machine
  interface and its `robots.txt` disallows all crawlers, so it cannot be mined; obtaining it is a
  relationship task, not an engineering one.
- **ProtonDB** is openly licensed and bulk-downloadable, with a genuinely useful seven-axis fault
  taxonomy, but it records no application build identifier at all, stores hardware as free text, and
  only about a fifth of recent reports name a specific runtime build. The median age of an
  application's newest report is 736 days.
- **Steam Deck Verified** is the real incumbent and is closer to HELM's pitch than the master plan
  assumes: a published rubric, per-build review, and automatic re-testing triggered by a new Proton
  release. It is human-judged, games-only, closed, and confined to hardware Valve controls.
- **Wine's own conformance suite** is rigorous, reproducible and daily — and tests the Windows API,
  not applications. It is also not a clean oracle: the real-Windows reference columns themselves
  show failures.
- The entire open per-application fix corpus — several hundred entries across Proton's launch script
  and `umu-protonfixes` — has **zero** automated verification that any fix still works.

The governance lesson is from `autopkgtest`: its data stays fresh because failures block migration.
Evidence that gates nothing decays, which is exactly what happened to AppDB and ProtonDB.

## Options considered

1. **Adopt AppDB or ProtonDB ratings as HELM's compatibility signal.** Rejected: unversioned against
   a specific runtime build, frequently years stale, and rating a whole application rather than the
   configuration that made it work. Building selection or public claims on them would encode noise
   as fact.
2. **Human attestation with a version stamp.** Honest, and the fallback if automation fails, but it
   does not scale and it is what the funded incumbents already do.
3. **Machine-generated, reproducible, expiring evidence that gates a decision.** The only option that
   is both novel and useful — and the one whose feasibility is unproven.

## Proposed decision

Adopt option 3, and treat its feasibility as the project's primary open question rather than an
assumption.

- An evidence record binds a **specific application build** to a **specific runtime build** on a
  **specific hardware profile**, and includes the **procedure** needed to re-run it. Any record
  missing one of those four is not evidence.
- Every record carries a **verified-on date and an expiry**. On expiry the associated profile is
  **suppressed rather than applied**, because a stale workaround can break an application that now
  works without it.
- Evidence **gates something**: whether a profile is offered, updated or withdrawn. Evidence that
  gates nothing will rot, as it has everywhere else.
- HELM adopts ProtonDB's fault taxonomy and open-data posture rather than inventing a vocabulary,
  and publishes its own data under terms compatible with whatever it derives from.
- HELM consumes `test.winehq.org` as a free daily substrate-health signal, and must not present it
  as application evidence.
- A **false pass is treated as a defect of the highest severity.** A signal that degrades into "it
  launched and did not crash for thirty seconds" is worse than publishing nothing, because it will
  be cited by users making decisions.

## Consequences

The oracle problem is the hard part and it is not solved by this decision. Wine implements no UI
Automation control patterns, so the entire off-the-shelf Windows GUI-automation ecosystem is
unusable; element discovery works, so the viable technique is find-then-synthesise-input, and the
strongest existing implementation of that is Xalia (MIT), which HELM should extend rather than
rebuild. Wine exposes nothing on AT-SPI, so Linux-native GUI test tools are blind to Wine windows
and can only test HELM's own interface.

HELM must therefore state publicly which workflows it can assert deterministically and which it
cannot, and it must scope its claims to what it can actually re-run. Applications whose mandatory
workflows cannot be asserted are reported as such rather than quietly marked as passing.

This extends [ADR-0002](ADR-0002-compatibility-metrics.md) by adding reproducibility, hardware
granularity and expiry to the metric definition. It does not replace its three-metric separation.
It also constrains [RFC-0001](../rfc/RFC-0001-app-evidence-model.md), whose schema will need a
procedure reference, a hardware profile with real granularity, and an expiry field.

## Evidence

Foundation audit [§3.6](../research/FOUNDATION_AUDIT.md#s03),
[§4.6](../research/FOUNDATION_AUDIT.md#s04), [§5.2](../research/FOUNDATION_AUDIT.md#s05), and risk
R-A2 in [§8](../research/FOUNDATION_AUDIT.md#s08). The state of Wine's UI Automation support was
verified first-hand, per [§12](../research/FOUNDATION_AUDIT.md#s12).

The deciding measurement is pre-registered in [EXP-009](../experiments/EXP-009.md): build the weakest
signal HELM would publish for exactly three applications and measure its false-pass rate by hand
against injected breakages.

## Corrections required before acceptance (recorded 2026-09-07)

Bounded application evidence annotation, 2026-09-07: [A0-7ZIP](../experiments/EXP-009-APP-BASELINE-REPORT.md#current-assessment)
produced a content-correct ZIP at the wrong W1 destination after agent actuation error. The
registered protocol remains FAIL despite valid controls and successful W2. This supports keeping
output validity separate from required workflow coverage. It does not evaluate expiry, profile
gating, the proposed three-application PoC, or the openQA option below. Status remains **Proposed**.

The novelty claim in this ADR is materially overstated and must be narrowed before review. A
comparison against openQA — which the original audit did not perform — found that it already ships
per-job input recording with pinned test-code commits, per-module artifact collection, a closed
result vocabulary defined in code, clone-and-reproduce rerun, non-pixel oracles including exit-code
and captured-output assertions, and automatic last-good attribution.

The defensible remainder is narrower than this ADR implies:

- **Two genuine absences:** a content hash identifying the exact artifact under test, and any
  mechanism that expires or re-validates a finished verdict.
- **Two schema-shape fixes:** hardware provenance exists but as mutable current state rather than an
  immutable per-run record; and application identity exists only as an untyped setting, though a
  history-isolation mechanism already provides the natural hook for it.

The ADR should also record an option it never considered: **adopting openQA and adding the missing
properties**, rather than building an evidence system. That option must be evaluated and either
taken or explicitly rejected with reasons.

See [audit §1.3](../research/FOUNDATION_AUDIT.md#s01) and corrections C-4 and C-5 in
[audit §13](../research/FOUNDATION_AUDIT.md#s13).

## Revisiting

The proof of concept measures a non-zero false-pass rate on the weakest signal, in which case this
decision is replaced by version-stamped human attestation at a scale the project can staff; or
upstream Wine gains UI Automation pattern support, which would materially reduce the cost of the
oracle and should trigger a re-plan.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
