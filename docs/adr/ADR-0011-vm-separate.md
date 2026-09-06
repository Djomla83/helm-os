# ADR-0011: Windows VM ostaje posebna opciona putanja

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

VM nije slobodna reimplementacija Windowsa i ima posebne resursne/licencne zahteve.

## Predložena odluka

Posebno označiti VM, ne sabirati u Wine kompatibilnost i ne obećavati microVM osobine.

## Razmotrene alternative

VM sakriti od korisnika i računati sve kao native podršku.

## Posledice

Veća transparentnost, ali više izbora i složeniji UX.

## Potrebni dokazi i veze

Zaseban proof-of-concept i provera konkretnih licencnih uslova. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Postoji stvarna potreba pilota i dokazana prihvatljivost implementacije.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
