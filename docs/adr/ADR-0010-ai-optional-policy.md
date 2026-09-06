# ADR-0010: Korisnički AI je opcion i ne određuje autorizaciju

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Model može pogrešno protumačiti zahtev ili slediti instrukciju iz nepouzdanog sadržaja.

## Predložena odluka

AI prvo read-only; izvršne operacije kroz tipizirani API, policy i odobrenje.

## Razmotrene alternative

Obavezan cloud asistent; model sa trajnim root pristupom.

## Posledice

Potrebna odvojena ne-AI putanja svih osnovnih funkcija.

## Potrebni dokazi i veze

Threat model i testovi nedozvoljenih radnji. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Novi usko definisan capability, uz nezavisan sigurnosni review.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
