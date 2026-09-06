# ADR-0009: Novi jezik i generalni semantički IR odložiti

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Nema dokaza da novi jezik rešava kritičan problem bolje od postojećih jezika i API ugovora.

## Predložena odluka

Najpre stabilan sistemski API, manifest i provere; jezik/MLIR samo kao zasebno opravdano istraživanje.

## Razmotrene alternative

Pisati OS paralelno sa novim jezikom; pokušati sve jezike nasilno provući kroz naš IR.

## Posledice

Manji početni obim. Ne odbacujemo dugoročnu ideju, ali tražimo merljiv razlog.

## Potrebni dokazi i veze

Definisan problem, jednostavniji baseline i poseban budžet. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Eksperiment pokaže značajnu prednost u ispravnosti ili korisnom razvoju.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
