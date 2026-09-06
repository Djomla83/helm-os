# ADR-0019: Publish the structurally unsupportable class, and decide the first product with evidence

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

Two findings in the audit are uncomfortable and are easier to leave implicit. Both should be
decided explicitly instead.

**First, some application classes are unreachable regardless of engineering effort.** Anti-cheat is
the ecosystem's worked example: two major systems have had working support for years, and it is
still a per-publisher opt-in, with community tracking showing a majority of tracked titles broken.
Kernel-mode attestation is structurally out of reach because Wine loads drivers only in user space.
The audit found the same shape in business software: kernel-mode endpoint security agents, hardware
licence dongles, networked DCOM, MSIX and WinRT packages, installers that depend on PowerShell (a
stub that returns success without executing anything), and scheduled tasks that Wine accepts and
persists but never fires. Announcing coverage before naming these is how a project gets judged
against the cases it structurally cannot win.

**Second, the project's motivating example does not support its proposed architecture.** The audit
found no evidence that communication-application unreliability on Linux is a Wine problem: those
vendors ship native Linux builds, WineHQ's tracker holds one Viber bug from 2013, the one
application that would need Wine ships in a package format Wine cannot install, and the dominant
2026 failure class was a tray-protocol regression affecting native and Electron applications
alike. That points at a different first product than the master plan assumes.

## Options considered

1. **Stay silent on both, and let scope emerge.** Rejected: it produces exactly the overclaiming
   that [ADR-0012](ADR-0012-public-docs-evidence.md) exists to prevent, and it defers a product
   decision until after the architecture is frozen, which is the most expensive possible moment.
2. **Decide the first product now, by argument.** Rejected: the two tracks have different evidence
   requirements and the audit's own reading could be wrong. A decision made now would be a
   preference wearing the clothes of a conclusion.
3. **Publish the exclusions immediately, and decide the product with a cheap pre-registered test.**

## Proposed decision

Adopt option 3.

**Part 1 — publish the structurally unsupportable class now.** HELM's compatibility model carries an
explicit, public class of applications it does not attempt, each with the mechanism that blocks it:
kernel-mode anti-cheat and endpoint security agents; hardware licence dongles and driver-based
licence systems; networked DCOM; MSIX and WinRT packages; installers that require PowerShell or that
hard-verify their own signature until the relevant Wine defects are fixed; and applications whose
vendor forces a server-side auto-update that HELM has no contract to freeze. This class is a
first-class part of the compatibility model, not a footnote, and it is stated before any coverage
claim.

**Part 2 — decide the first product with the Gate 0 result, not with an opinion.** The two candidate
products are set out in the audit:

- **Track A** — Windows-application compatibility management through per-application Wine
  environments. Novelty in version-scoped profiles, evidence and attributable recovery. Incumbents
  exist. The target class appears *harder* than games, not easier.
- **Track B** — desktop-integration reliability for applications as they are, native and Electron
  included: conformance testing of tray, notification and portal behaviour, runtime supervision with
  re-registration recovery, and repair of packaging metadata. Directly addresses the observed user
  pain. No incumbent was found. Principal risk is that the target set is drifting toward the browser.

The decision is made after gate checks G0-1 through G0-5 in [EXP-009](../experiments/EXP-009.md),
recorded in a new ADR that supersedes this part of this one, and it is allowed to be "both, in this
order" — but not "unstated".

## Consequences

Publishing exclusions first costs marketing reach and buys credibility, and it prevents the project
from spending engineering effort against problems whose gating factor is a vendor's policy rather
than code.

Deferring the product decision by a week costs a week. Making it wrongly costs the project.

Part 1 constrains every future public statement and is compatible with
[ADR-0012](ADR-0012-public-docs-evidence.md). Part 2 does not overturn any existing ADR: the
existing records describe *how* HELM would build a Windows-compatibility layer, and remain correct
for Track A. If Track B is chosen first, several of them become deferred rather than wrong, and that
should be recorded explicitly rather than by silence.

## Evidence

Foundation audit [§3.7](../research/FOUNDATION_AUDIT.md#s03),
[§6](../research/FOUNDATION_AUDIT.md#s06), constraint C10 in
[§7](../research/FOUNDATION_AUDIT.md#s07), risks R-A1 and R-A3 in
[§8](../research/FOUNDATION_AUDIT.md#s08), and gate checks G0-1 and G0-2 in
[§9.1](../research/FOUNDATION_AUDIT.md#s09).

Known weakness in the evidence: the analogy from game anti-cheat to business-software security
agents is asserted rather than measured — no equivalent survey of endpoint agents, dongles or
attestation under Wine was performed. The unsupportable class therefore needs its own evidence, one
mechanism at a time, before it is published as final.

## Revisiting

A vendor opts in, or an upstream defect that blocks a whole class is fixed, in which case that class
leaves the exclusion list with a dated note; or the Gate 0 result contradicts the audit's reading of
the motivating example, in which case Track A is confirmed on evidence rather than by default.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
