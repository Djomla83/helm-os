# Indeks arhitektonskih odluka

ADR-0020 is Accepted for documentation language. ADR-0022 is Accepted on 2026-09-08
with a bounded 0.1 scope on evidence from OBS-FS-01; it accepts an architecture boundary,
not an implementation, and excludes the descendant bind-mount case as unverified. On
2026-09-09 the owner clarified it, without changing its status or widening its scope:
filesystem cohort membership is an external support precondition, not an observation
attestation, and root admission applies only the necessary mechanism guards available
inside the explicit capability boundary. The supported cohort stays Linux x86_64 ext4.
On the same day the owner accepted the independently reviewed experimental helm-observe 0.1
implementation and fast-forwarded main to it; that records product-module acceptance under
ADR-0022, not another architecture decision, and expands none of its limits. ADR-0021 is Accepted on 2026-09-08
by repository owner Djomla83 only for building a pure, non-executable application
specification/validation library before observation or lifecycle execution.
Its schema/API remain experimental; a later separate owner instruction approved the
[independently corrected implementation](implementation/HELM-APP-SPEC-INDEPENDENT-REVIEW.md),
which is now merged as helm-app-spec 0.1. This records product-module acceptance,
not another architecture decision.
ADR-0023 is Proposed on 2026-09-09 for desired-versus-observed binding; it accepts nothing
and authorises no implementation. ADR-0001 through ADR-0019 remain Proposed. No other
architecture acceptance follows from this decision.

| ID | Odluka | Status |
|---|---|---|
| ADR-0001 | [Početi od postojećeg Linux user-space okruženja](adr/ADR-0001-linux-userspace-first.md) | Proposed |
| ADR-0002 | [Odvojiti binarnu kompatibilnost, native dostupnost i VM](adr/ADR-0002-compatibility-metrics.md) | Proposed |
| ADR-0003 | [Ponovno koristiti upstream komponente i ograničiti fork](adr/ADR-0003-upstream-first.md) | Proposed |
| ADR-0004 | [App Forge prvo kao priprema i validacija okruženja](adr/ADR-0004-appforge-orchestration.md) | Proposed |
| ADR-0005 | [Prefix nije sigurnosna granica](adr/ADR-0005-sandbox-boundary.md) | Proposed |
| ADR-0006 | [Verzionisani runtime uz održavanje i revokaciju](adr/ADR-0006-versioned-runtime-lifecycle.md) | Proposed |
| ADR-0007 | [Koristiti postojeći desktop u prvoj fazi](adr/ADR-0007-reuse-desktop-first.md) | Proposed |
| ADR-0008 | [SDK graditi na standardnom toolchain-u i Linux ABI-ju](adr/ADR-0008-sdk-standard-target.md) | Proposed |
| ADR-0009 | [Novi jezik i generalni semantički IR odložiti](adr/ADR-0009-language-deferred.md) | Proposed |
| ADR-0010 | [Korisnički AI je opcion i ne određuje autorizaciju](adr/ADR-0010-ai-optional-policy.md) | Proposed |
| ADR-0011 | [Windows VM ostaje posebna opciona putanja](adr/ADR-0011-vm-separate.md) | Proposed |
| ADR-0012 | [Javna dokumentacija mora odvojiti plan od rezultata](adr/ADR-0012-public-docs-evidence.md) | Proposed |

Sledeći zapisi su predloženi na osnovu [foundation audit-a](research/FOUNDATION_AUDIT.md)
i pisani su na engleskom po uputstvu vlasnika. Nijedan ne menja prethodne ADR-ove;
tenzije sa postojećim odlukama navedene su u
[poglavlju 10 audit-a](research/FOUNDATION_AUDIT.md#s10).

| ID | Odluka | Status |
|---|---|---|
| ADR-0013 | [Build, pin and ship HELM's own Wine runtime](adr/ADR-0013-pinned-wine-runtime.md) | Proposed |
| ADR-0014 | [Profiles are keyed on a version-scoped triple](adr/ADR-0014-version-scoped-profiles.md) | Proposed |
| ADR-0015 | [Evidence must be reproducible, gating, and must expire](adr/ADR-0015-evidence-expiry.md) | Proposed |
| ADR-0016 | [The application boundary runs on the host](adr/ADR-0016-host-side-sandbox.md) | Proposed |
| ADR-0017 | [Recovery is tiered, quiesced and reflink-based](adr/ADR-0017-data-safety-rules.md) | Proposed |
| ADR-0018 | [Integration via a Win32-to-portal bridge](adr/ADR-0018-win32-portal-bridge.md) | Proposed |
| ADR-0019 | [Publish the unsupportable class; decide the first product with evidence](adr/ADR-0019-scope-boundary.md) | Proposed |
| ADR-0020 | [Documentation language policy](adr/ADR-0020-documentation-language.md) | **Accepted 2026-09-07** |

Historical annotation, 2026-09-07: ADR-0017 was amended but remains Proposed; ADR-0015 still needs
review of its recorded corrections. ADR-0020 alone is Accepted, for documentation language.
The original G0-3 criterion remains BLOCKED; the separate mechanics subtest does not complete it.
See the [current Gate 0 assessment](experiments/EXP-009-GATE0-REPORT.md#current-assessment).

Proces je u [master planu](../HELM_MASTER_PLAN.md#s31). Licencna odluka ostaje zaseban uslov javnog open-source izdanja.

| ID | Decision | Status |
|---|---|---|
| ADR-0021 | [Make the second product module an inert application contract](adr/ADR-0021-second-product-module.md) — [selection analysis and owner refinements](research/SECOND-PRODUCT-MODULE-SELECTION.md); bounded architectural authority, with experimental helm-app-spec 0.1 now accepted on main | **Accepted 2026-09-08** |
| ADR-0022 | [Observe explicit targets without interpreting desired state](adr/ADR-0022-observation-authority.md) — [architecture analysis](research/HELM-OBSERVE-ARCHITECTURE.md), [independent review with Amendment 1](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md), [execution definition](experiments/obs-fs-01/) and [OBS-FS-01 PASS](experiments/OBS-FS-01-EXECUTION-REPORT.md). **Bounded acceptance**: Linux x86_64 and ext4 cohort only; descendant bind mounts excluded and unverified; three implementation-binding obligations, all met and independently verified on the review branch. [Owner clarification 2026-09-09](adr/ADR-0022-observation-authority.md#cohort-attestation-clarification): cohort membership is a caller precondition, not an observer attestation; `0xEF53` is a necessary ext-family guard only; no new authority added; scope unchanged. **Experimental helm-observe 0.1 is implemented, independently reviewed and owner-merged to main on 2026-09-09** at reviewed tip `626de914000263cc3206d28479db692ad0b40724`; schema and API unstabilised, `publish = false`, not a release | **Accepted 2026-09-08, clarified 2026-09-09** |
| ADR-0023 | [Compare desired claims with actual observations without producing a satisfaction verdict](adr/ADR-0023-binding-authority.md) — [design report](research/HELM-BIND-ARCHITECTURE.md). Proposes a pure, authority-free `helm-bind` 0.1 with no satisfaction, compatibility or readiness verdict, a two-axis result whose coverage can never be complete in 0.1, an explicit identity-bearing binding plan instead of target-ID heuristics, and no change to any existing crate. **Not accepted; authorises no implementation** | **Proposed** |
