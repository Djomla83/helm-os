# Mapa dokumentacije

## Kanonski pregled

[Master dokument](../HELM_MASTER_PLAN.md) sadrži 42 poglavlja. Za brzo uvođenje
koristite [prvi agent zadatak](../AGENT_STARTER.md), [stanje projekta](PROJECT_STATE.md)
i [otvorena pitanja](OPEN_QUESTIONS.md).

## Experimental product code

[helm-evidence 0.1 owner acceptance](PROJECT_STATE.md#helm-evidence-owner-acceptance)
records the first merged module, corrected commit identities and successful main
Linux CI. [Module documentation](../crates/helm-evidence/README.md) defines its
experimental schema/API and limits; A0-7ZIP experimental FAIL remains unchanged.

## Odluke i istraživanja

[ADR indeks](DECISIONS.md): nineteen Proposed records and ADR-0020 Accepted for documentation language.
[RFC-0001](rfc/RFC-0001-app-evidence-model.md) definiše nacrt testne evidencije;
[RFC-0002](rfc/RFC-0002-threat-model.md) početni threat model.
[Mapa eksperimenata](experiments/README.md) vodi do devet planova; nijedan aplikacioni
test nije izvršen.
[Prethodni rad](research/PRIOR_ART.md), [registar tvrdnji](research/CLAIMS_REGISTER.md)
i [izvori](SOURCES.md) održavaju razliku između mogućnosti i dokaza.

[Foundation audit](research/FOUNDATION_AUDIT.md) (2026-09-06, engleski) je rezultat
EXP-001: pregled postojećeg ekosistema po primarnim izvorima, reuse matrica, analiza
stvarne novine, deset najrizičnijih pretpostavki i predlog najmanjeg PoC-a
([EXP-009](experiments/EXP-009.md)). To je desk research, ne rezultat testa aplikacija.
Revizija 2 (2026-09-07) sadrzi [dnevnik ispravki](research/FOUNDATION_AUDIT.md#s13) i rezultat
[Gate 0](experiments/EXP-009-GATE0-REPORT.md).

## Operativni rad

[Backlog](../planning/BACKLOG.md) ima osamnaest pripremljenih zadataka.
[Objavljivanje](runbooks/PUBLISH.md) i [izveštaj eksperimenta](runbooks/EXPERIMENT_REPORT.md)
opisuju postupke. [CONTRIBUTING](../CONTRIBUTING.md), [SECURITY](../SECURITY.md) i
[licencna odluka](../LICENSE-DECISION.md) definišu granice javne saradnje.

## Strukturirani primeri

[JSON šema](../schemas/app-test-record.schema.json),
[sintetički zapis](../examples/app-test-record.example.json) i
[nacrt kohorte](../planning/app-cohort.json). Nijedan nije rezultat app testa.

## Izvršene provere paketa

[Izveštaj o validaciji](VALIDATION.md) razdvaja dokumentacione provere od
neizvršenih aplikacionih eksperimenata.
