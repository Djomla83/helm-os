# ADR-0005: Prefix nije sigurnosna granica

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Odvojena konfiguracija ne dokazuje da aplikacija nema pristup host resursima.

## Predložena odluka

Koristiti odvojeni proverljivi sandbox/policy sloj i negativne testove.

## Razmotrene alternative

Prefix nazvati sandbox-om; rešavati svaki bug širenjem prava.

## Posledice

Mogući gubici legacy integracije; moraju biti vidljivi i testirani.

## Potrebni dokazi i veze

Threat model, EXP-004 i master poglavlje 17. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Backend ne može održati obavezni tok; novi pregled pre promene prava.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
