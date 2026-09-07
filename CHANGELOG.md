# Istorija promena

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
