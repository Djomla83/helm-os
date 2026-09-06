# ADR-0002: Odvojiti binarnu kompatibilnost, native dostupnost i VM

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Jedan zbirni procenat može prikriti da originalna Windows aplikacija ne radi.

## Predložena odluka

Kohortu definisati pre testiranja; nepoznate i blokirane stavke ne brojati kao uspeh; VM odvojiti.

## Razmotrene alternative

Jedan marketinški skor; broj pokrenutih prozora; izbacivanje neuspešnih aplikacija.

## Posledice

Metrika je stroža ali razumljiva. Potrebni su tačni scenariji i obavezni profili.

## Potrebni dokazi i veze

HELM-004, HELM-008 i master poglavlje 5. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Dokazana nejasnoća metodologije, uz očuvanje istorije i paralelni prikaz stare kohorte.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
