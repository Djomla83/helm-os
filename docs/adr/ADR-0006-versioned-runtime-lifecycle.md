# ADR-0006: Verzionisani runtime uz održavanje i revokaciju

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Pinning smanjuje promene ali može zadržati poznatu ranjivost ili zastareli protokol.

## Predložena odluka

Verzionisati runtime i profile, uz podržani rok, obnovu testova i plan migracije.

## Razmotrene alternative

Globalno menjati sve aplikacije; zamrznuti svaki runtime zauvek.

## Posledice

Potrebni su katalog, revokacija i ograničenje broja održavanih verzija.

## Potrebni dokazi i veze

EXP-005 i HELM-013. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Operativni trošak previsok ili aplikacioni update onemogućava predloženi model.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
