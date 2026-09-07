# Istorija promena

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
