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

[helm-launch 0.1](../crates/helm-launch/README.md) is the launch module: P1 to P5 are
**accepted**, and the complete 0.1 module is **product-accepted** since 2026-09-21 at reviewed head
`dd92a85fcf0cc35306123ef2dc39adb148ddc616`. Its published evidence contract is the
[receipt schema 0.1](implementation/HELM-LAUNCH-RECEIPT-0.1.md) with
[portable test vectors](implementation/helm-launch-receipt-0.1-test-vectors.json); the vectors are
data, carry no authenticity claim, and their digests are recomputable from the published bytes.

## Vizuelni proizvod (G2)

[HELM G2 — visual product kickoff](implementation/HELM-G2-VISUAL-KICKOFF.md) defines the first
visible HELM desktop application: a repository capability audit, the information architecture, the
primary journey, the screen map and specifications, the state and error vocabulary, the first real
vertical slice and the G2-D1 to G2-D9 acceptance gates. **All nine gates are owner-accepted since
2026-09-21**: D1 to D7 with four recorded clarifications, **D8** as a visual system direction —
reference hierarchy, working palette, status colour rule, character and medium density, in section
8.16 on the terms of 20.4 — and **D9**, the UI toolkit, on the terms of section 20.5. The
palette is a working token set, not a final brand identity, and no generated concept is authoritative for
product semantics.

[HELM G2 — UI toolkit decision](implementation/HELM-G2-UI-TOOLKIT-DECISION.md) is the **G2-D9
dossier prepared for the owner**: derived hard requirements, the A to S criteria, assessments of
GTK 4 with gtk-rs, GTK 4 with selective libadwaita, Qt 6 / QML with CXX-Qt, Slint and iced, a technical
rejection of the webview option, and a comparative table with version evidence read on 2026-09-21.
It carries the [accepted G2-D9 decision](DECISIONS.md#g2-d9-toolkit-accepted) of 2026-09-21:
**GTK 4 + gtk-rs is selected for the G2 prototype**; selective libadwaita is permitted only
through its documented bounded public surface; **GTK is a provisional production default, not a
production lock**; **Qt 6 / QML + CXX-Qt is the named fallback** if the spike fails any exit
criterion; and the **GTK/libadwaita version floor stays OPEN**, to be decided explicitly before any
version feature flag is adopted. On 2026-09-21 the owner
[authorised the bounded GTK implementation spike](DECISIONS.md#g2-gtk-spike-authorised) with an API
floor of **GTK 4.18 + libadwaita 1.7** — a floor for that spike only, not a distribution support
policy and not a production lock. Backend integration, `helm-launch`, persistence and a production
toolkit lock stay unauthorised.

## Odluke i istraživanja

[ADR index](DECISIONS.md): nineteen Proposed records, ADR-0020 Accepted for documentation language,
and [ADR-0021](adr/ADR-0021-second-product-module.md) Accepted for the bounded `helm-app-spec`
selection. The [selection report](research/SECOND-PRODUCT-MODULE-SELECTION.md) records owner
refinements; no schema/API stability or implementation authorisation follows.
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
[Raspored lokalnog stanja](operations/LOCAL-STATE.md) je konvencija koja razdvaja autoritativno
Git radno stablo od mašinski lokalnog VM, privatnog, download i scratch stanja, i razlikuje
istorijske putanje iz dokaza od tekućeg operativnog rasporeda.

## Strukturirani primeri

[JSON šema](../schemas/app-test-record.schema.json),
[sintetički zapis](../examples/app-test-record.example.json) i
[nacrt kohorte](../planning/app-cohort.json). Nijedan nije rezultat app testa.

## Izvršene provere paketa

[Izveštaj o validaciji](VALIDATION.md) razdvaja dokumentacione provere od
neizvršenih aplikacionih eksperimenata.
