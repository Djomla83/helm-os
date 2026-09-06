# ADR-0004: App Forge prvo kao priprema i validacija okruženja

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Binarna adaptacija ne može se obećati kao univerzalna zamena za Windows semantiku.

## Predložena odluka

Originalni binary čuvati kada je moguće; analizirati, pripremiti profil, izolovati i testirati.

## Razmotrene alternative

Generalno EXE→ELF prepisivanje; automatski generated patch pri svakom korisničkom install-u.

## Posledice

Dobijamo realističniji pipeline bez garancije nativnog porta.

## Potrebni dokazi i veze

EXP-003, EXP-004 i manifest RFC. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Profiling pokaže usku korisnu binarnu specijalizaciju; zahteva zaseban eksperiment.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
