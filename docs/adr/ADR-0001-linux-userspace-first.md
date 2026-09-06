# ADR-0001: Početi od postojećeg Linux user-space okruženja

**Status:** Proposed\
**Datum nacrta:** 2026-09-06\
**Odobravalac:** nije unet\
**Datum prihvatanja:** nije unet

## Kontekst

Ne znamo da li aplikacioni sloj donosi dovoljno vrednosti da opravda sopstvenu OS sliku.

## Predložena odluka

Prvo napraviti mali vertikalni prototip na postojećoj distribuciji; kernel i ISO ne razvijati kao prvi korak.

## Razmotrene alternative

Novi kernel; trenutni fork pune distribucije; samo paketni manager bez test infrastrukture.

## Posledice

Dobijamo raniji dokaz vrednosti. Početni proizvod neće izgledati kao potpuno novi OS.

## Potrebni dokazi i veze

EXP-001 i uslovi G1/G2; tačna baza se odlučuje zasebno. Videti [master plan](../../HELM_MASTER_PLAN.md) i
[mapu eksperimenata](../experiments/README.md).

## Preispitivanje

Dokazano ograničenje postojeće baze ili uspešan G2 koji opravdava OS integraciju.

Ovaj zapis nije ljudsko odobrenje. Promena u `Accepted` zahteva ime/ulogu odobravaoca,
datum i review referencu; postojeći datum nacrta nije datum prihvatanja.
