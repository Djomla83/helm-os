# ADR-0008: SDK graditi na standardnom toolchain-u i Linux ABI-ju

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Novo ime target-a ne rešava Windows biblioteke niti ponašanje sistemskih API-ja.

## Predložena odluka

Prvo koristiti Clang/CMake i odgovarajući standardni Linux target/sysroot; ograničiti podržane projekte.

## Razmotrene alternative

Novi LLVM triple bez potrebe; univerzalni compiler dodatak sa obećanjem jednog klika.

## Posledice

Manje fragmentacije. Windows-only zavisnosti i dalje mogu tražiti ručan port.

## Potrebni dokazi i veze

EXP-006, S08/S09/S18 i dependency audit. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Stvarni ABI zahtev koji standardna putanja ne može da podrži.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
