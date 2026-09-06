# Indeks arhitektonskih odluka

Svi zapisi su početni predlozi, ne odobrene tehničke obaveze.

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

Proces je u [master planu](../HELM_MASTER_PLAN.md#s31). Licencna odluka ostaje zaseban uslov javnog open-source izdanja.
