# Istorija promena

## 2026-09-09 — helm-observe 0.1 owner acceptance

Merged the independently reviewed observation library by strict fast-forward as HELM's
third experimental product-code module. The authoritative
[acceptance record](docs/PROJECT_STATE.md#helm-observe-owner-acceptance) identifies the
reviewed tip, the preserved first-pass findings, the corrections and the verified CI runs.
The [independent review](docs/implementation/HELM-OBSERVE-INDEPENDENT-REVIEW-0.1.md) reports
one BLOCKER and three IMPORTANT findings, all resolved: a foreign process's procfs
descriptor directory could be admitted, duplicate decoded JSON keys escaped detection, the
backend was not gated to the accepted architecture, and the `0xEF53` guard could not attest
ext4. The last was resolved by an owner
[clarification to Accepted ADR-0022](docs/adr/ADR-0022-observation-authority.md#cohort-attestation-clarification):
cohort membership is an external support precondition, not an observation attestation.
Supported cohort stays Linux x86_64 and ext4; ext2, ext3 and storage locality remain
unattested and unsupported; descendant bind mounts stay excluded. Schema and API remain
experimental and `publish = false`. No release, no architecture ADR acceptance, no
`helm-bind` or `helm-launch`, and A0-7ZIP remains experimental FAIL.

## 2026-09-08 — bounded helm-app-spec selection accepted

Repository owner Djomla83 accepted [ADR-0021](docs/adr/ADR-0021-second-product-module.md)
only for a pure, non-executable application specification/validation library before observation
or lifecycle execution. Refined the [selection report](docs/research/SECOND-PRODUCT-MODULE-SELECTION.md)
with raw-byte document identity, desired/observed separation, bounded runtime artifact requirements,
acyclic verification references, pure validation and parser/environment limits.
The [acceptance record](docs/PROJECT_STATE.md#helm-app-spec-selection-acceptance) preserves the
original analysis history. Schema/API remain unsettled; implementation is not authorised.
No product code changed. A0-7ZIP experimental FAIL and all other ADR statuses remain unchanged.

## 2026-09-08 — helm-evidence 0.1 owner acceptance

Merged the corrected review history by fast-forward as HELM's first experimental
product-code module. Main's Linux CI passed. The authoritative
[acceptance record](docs/PROJECT_STATE.md#helm-evidence-owner-acceptance) identifies
the owner-approved tip, corrective commit, fixed review findings and CI runs.
Schema/API remain experimental; A0-7ZIP FAIL and known limitations are preserved.
No production release, architecture ADR acceptance or next subsystem is included.

## 0.4.4 — 2026-09-07

Executed the separately authorised A0-7ZIP desktop baseline after verifying the owner's completed
Hyper-V group action was effective in a non-elevated session. Created and retained one Ubuntu
24.04.4 Generation 2 VM within the registered limits, with Secure Boot and the existing switch.
Verified pinned WineHQ vanilla 11.17 and Windows x64 7-Zip 26.03; guest controls and executable
identities passed. The first GUI attempt created its correct-content ZIP at the wrong destination
because of agent actuation error; W1/V1 remain FAIL without retry. The planned post-restart W2/V2
passed. Overall A0-7ZIP is FAIL against the unchanged two-workflow protocol, not an application
incompatibility conclusion.

Published scoped experimental instrumentation, commands, synthetic GUI evidence, package and
output identities, failures, resource measurements and private/public provenance. VM and prior labs
are stopped and preserved. No product module, runtime switching, recovery, architecture acceptance
or larger PoC was started. Work stops at application-baseline review.

## 0.4.3 — 2026-09-07

Recorded owner acceptance of G0-2 only for the controlled HRESULT comparison and the separately
authorised EXP-009 A0-7ZIP desktop application baseline. Existing Hyper-V access remains denied;
prepared the exact owner-only group action privately without executing it. No VM or application
workflow ran, and all prior labs remain stopped.

Pinned official Windows x64 7-Zip 26.03, Ubuntu 24.04.4 desktop and WineHQ vanilla 11.17 package
identities. Added small synthetic-fixture/output-verifier code using the existing capture helper,
unit checks, a two-workflow preregistration and a conservative D: storage plan. Preserved setup
errors and private/public evidence provenance. Host helper checks do not imply application success.
No architecture ADR or larger PoC was accepted or started.

## 0.4.2 — 2026-09-07

Completed the owner-authorised G0-2 HRESULT comparison from public baseline `ca0aa5ae26a1d3b630f63eda8f87626d125248d1`.
Direct negative Windows-execution tests supported blocking for the specific invocation in all
three disposable WSL labs. Frozen controls accompanied the unchanged DirectComposition probe:
WineHQ vanilla 11.17 and UMU-Proton-10.0-4 returned `E_NOTIMPL`; staging 11.16 returned `S_OK`.
G0-2 PASS denotes a valid controlled comparison, not rendering or application compatibility.

Preserved historical vanilla INCONCLUSIVE and an initial Proton INCONCLUSIVE capture. One
documented, preregistered DOS-path repetition recovered console observations without changing
probe semantics, controls, runtime pins or verdict criteria. Recorded exact artifacts, dependency
differences, stdout/stderr, warnings, provisioning failure, disk growth and engineering effort.
Redacted host-name diagnostics and local account paths with distinct raw/publication hashes;
repaired an unpublished local checkpoint before publication. Evidence byte preservation is scoped
in Git attributes. No public history was rewritten.

Updated the [report](docs/experiments/EXP-009-GATE0-REPORT.md#current-assessment), claims, project
state and lab runbook. Original G0-3 and G0-1/G0-4/G0-5 remain BLOCKED. All architecture ADRs remain
Proposed. Labs are retained and stopped. Work ends at Gate 0 review; no larger PoC was started.

## 0.4.1 — 2026-09-07

Owner-authorised publication/provenance repair of the three local-only Gate 0 commits; the public
base and logical commit sequence are preserved. A verified private bundle and original raw
artifacts were retained outside the repository before deterministic redaction of operator account
components in two evidence files. Redacted fields, raw hashes and distinct publication hashes are
recorded in the [Gate 0 report](docs/experiments/EXP-009-GATE0-REPORT.md#publication-provenance).

Current statuses corrected without changing probe output or acceptance criteria: original G0-3
remains BLOCKED; G0-3a/G0-3b PASS only as separate mechanics subtests. Vanilla G0-2 retains
`0x80004001` but is INCONCLUSIVE without required control evidence; overall G0-2 is BLOCKED and
staging/Proton arms are NOT_RUN. Historical application-class inference is withdrawn. Intended
disabled WSL interoperability and observed enabled registration are recorded separately; effective
execution blocking is UNVERIFIED. No new experiment or runtime provisioning was performed, and no
architecture ADR was accepted. Older entries below retain the original reporting history.

## 0.4.0 — 2026-09-07

Jezicka politika usvojena: ADR-0020 je **Accepted** uz imenovano odobrenje vlasnika i merodavan je
za tu temu; `CONTRIBUTING.md` sada upucuje na njega. Nijedan arhitektonski ADR nije prihvacen.

Izvrsena ogranicena Gate 0 lab faza. Preferirani VM put nije bio dostupan (Hyper-V uloga je
ukljucena, ali nalog nema dozvolu; to bi trazilo administratorsku izmenu koja nije odobrena), pa je
koriscen odobreni fallback: jedna jednokratna WSL2 distribucija `helm-lab-g0`, izolovana od Windows
diskova i interop-a, sa neprivilegovanim korisnikom. Dodat
`docs/experiments/LAB-G0-RUNBOOK.md` sa manifestom, reprodukcijom i uputstvom za uklanjanje.

Rezultati: 3 PASS, 1 PARTIAL, 3 BLOCKED, 0 FAIL. Sa stvarnim Wine 11.17 potvrdjeno je da hardlink
kopija prefiksa biva kontaminirana, da je quiesced puna kopija nezavisna i da dokument nastao posle
tacke oporavka **ne prezivljava** vracanje celog prefiksa. Vanilla Wine vraca E_NOTIMPL za
DirectComposition; staging i Proton grane nisu testirane, pa poredjenje ostaje neresenо.

ADR-0017 izmenjen po uputstvu vlasnika: trazi nezavisno stanje oporavka kao **proverenu osobinu**,
sa quiesced punom kopijom kao prenosivom osnovom; reflink je opcion i mora se eksplicitno testirati.
ADR-0017 ostaje `Proposed`.

Zabelezeno je i da round-1 PASS **ne** zadovoljava kriterijum registrovan pre izvrsenja; prijavljen
je kao novi podtest, bez naknadnog ublazavanja kriterijuma. Nijedan aplikacioni test nije izvrsen;
stopa kompatibilnosti ostaje nepoznata. PoC nije odobren.

## 0.3.0 — 2026-09-07

Foundation audit ispravljen na reviziju 2: deset zabelezenih ispravki u
`docs/research/FOUNDATION_AUDIT.md` (poglavlje 13), uz razdvajanje ocena VERIFIED_SOURCE i
VERIFIED_EXECUTION. Povucen zakljucak o uzroku problema sa komunikacionom aplikacijom; broj bug
prijava ne meri upotrebu. Tvrdnja o novosti evidence sloja znatno suzena posle poredjenja sa
openQA. UIA zakljucak ogranicen na fiksiran tag i na pattern actuation.

Gate 0 delimicno izvrsen (`docs/experiments/EXP-009-GATE0-REPORT.md`): jedan PASS, cetiri BLOCKED,
nijedan FAIL. Izvrsen probe `tools/gate0/probe_snapshot_semantics.py` na dva fajl sistema; hardlink
kopija potvrdjeno nije snapshot, a reflink nije podrzan ni na jednom testiranom fajl sistemu, sto
protivreci ADR-0017. EXP-009 prepravljen na reviziju 2 sa recnikom ishoda, baseline granom,
kontrolama, pravilima statisticke postenosti i pravilima bezbednog testiranja oporavka.

Dodat ADR-0020 koji **predlaze** izmenu jezicke politike u `CONTRIBUTING.md` umesto dosadasnjeg
precutnog odstupanja. ADR-0015 i ADR-0017 nose zabelezene ispravke. Nijedan ADR nije prihvacen.
PoC nije odobren. Nijedan aplikacioni test nije izvrsen; stopa kompatibilnosti ostaje nepoznata.

## 0.2.0 — 2026-09-06

Foundation audit (EXP-001) izvršen kao desk research nad primarnim izvorima:
`docs/research/FOUNDATION_AUDIT.md`, reuse matrica, analiza novine, deset najrizičnijih
pretpostavki i predlog najmanjeg PoC-a (`docs/experiments/EXP-009.md`). Dodato sedam
predloženih ADR-ova (0013–0019); nijedan postojeći ADR nije menjan. Ažurirani
`docs/PROJECT_STATE.md`, indeksi i registar tvrdnji.

Predlozeni naziv repozitorijuma promenjen iz `helm-os-design` u `helm-os`; master plan je
podignut na reviziju 0.1.1 sa zapisom te promene.

Novi dokumenti su na engleskom po uputstvu vlasnika, što odstupa od jezičnog pravila u
`CONTRIBUTING.md` i traži odluku vlasnika. Nijedan aplikacioni test, benchmark, sandbox
audit ni hardverski test nije izvršen. Stopa kompatibilnosti ostaje nepoznata.

## 0.1.0 — 2026-09-06

Početni master plan, pravila rada za agente, predlozi ADR-ova i RFC-ova, planovi
osam eksperimenata, backlog sa pripremljenim telima issue-a, nacrt kohorte i
strukturiranog evidence zapisa, lokalni validator i uputstvo za objavljivanje.

Nema implementacije OS-a, app runtime-a, SDK-a ili jezika. Nijedan aplikacioni test
nije izvršen kao deo ovog dokumentacionog paketa. Udaljena GitHub objava nije
potvrđena ovom revizijom. Licencna odluka je na čekanju.
