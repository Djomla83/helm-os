# ADR-0014: An application profile is keyed on a version-scoped triple, not on an application identity

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

Every existing per-application fix registry binds a workaround to an *application identity* and
nothing else. The audit confirmed this across four independent implementations: `umu-protonfixes`
has no build identifier anywhere, the umu database schema has no version column, the Lutris
installer DSL has no version predicate, and CrossTie carries install *detection* but no
applicability range. Valve's own private per-title configuration is likewise keyed on the
application identifier.

For games that omission is nearly harmless: a storefront pins the build and titles update rarely.
Business software inverts every one of those assumptions — it updates silently, out of band, often
monthly, with no store mediating. And upstream Wine states the failure mode directly: applying
tweaks that are no longer needed can prevent an application that now runs fine from working at all.

So a workaround with no applicability range does not become *inapplicable* when the application
updates. It becomes silently **wrong**, and keeps being applied. The harm grows with catalogue size.

The audit also found that the analysis-and-derivation half of the App Forge concept in master plan
chapter 13 already shipped: Bottles release 61.0 (2026-01-24) performs PE and YARA analysis of an
executable, its neighbours and extracted installer payloads to derive a suggested environment, with
a packaged compatibility database added on 2026-07-31.

## Options considered

1. **Contribute version scoping to an existing catalogue.** Rejected: no existing schema can express
   it, and retrofitting version binding onto thousands of existing records is more expensive than
   requiring it from the first entry.
2. **Key profiles on application identity, as the incumbents do, and rely on maintainers noticing
   breakage.** Rejected: this is the observed failure mode, and it scales in the wrong direction.
3. **Key every profile on a mandatory version-scoped triple.** Higher authoring cost per profile,
   and the only design that can express when a fix stops applying.

## Proposed decision

Adopt option 3.

A profile is addressed by a mandatory triple, and a profile that cannot state all three parts is
invalid rather than global:

- **stable application identity**, derived from the artifact rather than from a storefront — PE
  version resource, MSI product code, installer hash, signing certificate — with `umu` identifiers
  adopted where one exists so that HELM interoperates rather than forking the namespace;
- **an applicability range over the application version or build**;
- **an applicability range over the substrate**: runtime build, and where relevant graphics driver
  and kernel capability.

Further consequences of the decision:

- HELM copies the existing fix *vocabulary* nearly verbatim from `umu-protonfixes` — it is the
  accumulated empirical answer to what actually has to change — but not its data model.
- HELM copies CrossTie's install-detection primitives and Steam's precedence model, in which a
  per-application user override beats a vendor recommendation, which beats a global default.
- HELM copies Heroic's practice of an explicit config version with a migration path.
- **HELM does not claim automatic environment derivation as novel.** Derivation may be implemented
  or consumed, but the published differentiator is that a derived environment is *verified*, which
  nobody measures today.

## Consequences

Authoring a profile becomes more expensive and more honest. The catalogue becomes smaller and more
truthful. Profile count becomes a liability metric as well as an asset metric, because every profile
carries a re-validation obligation.

This stands in tension with [ADR-0004](ADR-0004-appforge-orchestration.md), which presents analysis
and planning as App Forge's contribution. This ADR does not repeal that decision; it narrows the
claim of novelty within it and moves the defensible part to version scoping and evidence. If the
owner disagrees, the correct outcome is to reject this ADR explicitly.

## Evidence

Foundation audit [§5.1 and §5.2](../research/FOUNDATION_AUDIT.md#s05),
[§3.3](../research/FOUNDATION_AUDIT.md#s03), and risks R-A4 and R-A5 in
[§8](../research/FOUNDATION_AUDIT.md#s08). The corpus counts behind these findings are
order-of-magnitude signals only and must be re-derived at a pinned revision before being quoted, per
[§11](../research/FOUNDATION_AUDIT.md#s11).

Two pre-registered tests decide whether the declarative story is wishful: express twenty existing
upstream fixes in this schema and count how many still need an escape hatch to arbitrary code — if
more than roughly a fifth do, say so publicly; and retest ten workarounds that were required on Wine
9.x against 11.17, with and without the tweak, to obtain the first real measurement of recipe
half-life.

## Revisiting

The measured recipe half-life turns out to be long enough that version scoping is unnecessary
overhead; or an upstream catalogue adopts version scoping first, in which case HELM should
contribute to it rather than compete.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
