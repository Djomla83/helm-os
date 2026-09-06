# ADR-0007: Koristiti postojeći desktop u prvoj fazi

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Novi compositor, accessibility stack i settings mogu potrošiti resurse pre dokaza kompatibilnosti.

## Predložena odluka

Svoj upravljački UX razvijati iznad postojeće desktop osnove; novi shell odložiti.

## Razmotrene alternative

Odmah prepisati desktop; samo promeniti temu i tvrditi da je problem rešen.

## Posledice

Rani prototip zavisi od izabrane desktop integracije, ali nasleđuje zrelije osnovne tokove.

## Potrebni dokazi i veze

Rezultati komunikacionog i portal eksperimenta. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Jasno ograničenje postojeće integracije i tim sposoban za dugoročno održavanje.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
