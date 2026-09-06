# Početni backlog

Pripremljena tela, ne otvoreni udaljeni issue-i. Sve stavke su Draft.

| ID | Zadatak | Prioritet | Zavisnosti |
|---|---|---|---|
| HELM-001 | [Potvrditi obim G0 i odgovorne osobe](issues/HELM-001.md) | P0 | Nema |
| HELM-002 | [Potvrditi ime i licencnu politiku](issues/HELM-002.md) | P0 | HELM-001 |
| HELM-003 | [Pregledati postojeća rešenja i reuse mapu](issues/HELM-003.md) | P0 | HELM-001 |
| HELM-004 | [Definisati pilot kohortu i obavezne tokove](issues/HELM-004.md) | P0 | HELM-001 |
| HELM-005 | [Izabrati bazno okruženje i hardver](issues/HELM-005.md) | P0 | HELM-003, HELM-004 |
| HELM-006 | [Definisati threat model i lab izolaciju](issues/HELM-006.md) | P0 | HELM-005 |
| HELM-007 | [Reprodukovati komunikacioni problem](issues/HELM-007.md) | P0 | HELM-004, HELM-005, HELM-006 |
| HELM-008 | [Definisati format testnih dokaza](issues/HELM-008.md) | P0 | HELM-004 |
| HELM-009 | [Napraviti kontrolisanu instalacionu putanju](issues/HELM-009.md) | P1 | HELM-003, HELM-006, HELM-008 |
| HELM-010 | [Proveriti ažuriranje i integritet podataka](issues/HELM-010.md) | P1 | HELM-009 |
| HELM-011 | [Proveriti granice dozvola](issues/HELM-011.md) | P1 | HELM-006, HELM-009 |
| HELM-012 | [Izmeriti dodatni source build target](issues/HELM-012.md) | P1 | HELM-003, HELM-005 |
| HELM-013 | [Definisati održavanje runtime-a i profila](issues/HELM-013.md) | P1 | HELM-009, HELM-010 |
| HELM-014 | [Izmeriti doprinos malog agent pool-a](issues/HELM-014.md) | P1 | HELM-008 |
| HELM-015 | [Pripremiti odluku G1 ka G2](issues/HELM-015.md) | P1 | HELM-003, HELM-007, HELM-008 |
| HELM-016 | [Revidirati manifest posle prototipa](issues/HELM-016.md) | P2 | HELM-009, HELM-011 |
| HELM-017 | [Ispitati potrebu za OS slikom i novim UX-om](issues/HELM-017.md) | P2 | HELM-015 |
| HELM-018 | [Opravdati jezičko ili specijalizaciono istraživanje](issues/HELM-018.md) | Later | HELM-015 |

Prvi sadržinski posao je HELM-003 / EXP-001 nakon potvrde obima. Ne pokretati sve stavke paralelno.
