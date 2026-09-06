# ADR-0003: Ponovno koristiti upstream komponente i ograničiti fork

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Trajan veliki fork povećava trošak svake sigurnosne i funkcionalne nadogradnje.

## Predložena odluka

Koristiti postojeće Wine/Proton/grafičke komponente i slati opšte popravke upstream-u.

## Razmotrene alternative

Prepisati Win32 od početka; trajno sakriti poreklo pod novim runtime nazivom.

## Posledice

Manji sopstveni obim, ali zavisnost od tuđeg lifecycle-a i procesa review-a.

## Potrebni dokazi i veze

EXP-001 i dokumentovan patch lifecycle. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Upstream ne prihvata neophodnu promenu; lokalni fork ostaje ograničen i obrazložen.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
