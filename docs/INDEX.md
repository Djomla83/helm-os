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
vertical slice and the G2-D1 to G2-D9 acceptance gates. **All nine gates, G2-D1 to G2-D9, are
owner-accepted since 2026-09-21** — D1 to D7 with four recorded clarifications, and D8 recording the
*Record / graphite frame* visual direction in section 8.16. **G2-D9 (toolkit selection) was accepted
on 2026-09-21**: the selected technology is **GTK 4 + gtk-rs with selective libadwaita**. That
acceptance authorised **one bounded native UI fidelity spike with mock state only**; the spike and
then the first real backend-connected vertical have since been accepted, and neither accepts a
production GUI nor starts G-1, G-2 or G-3.

The [G2 non-product interaction prototype](prototypes/g2-html/README.md) is a clickable HTML
reference for the accepted *Record / graphite frame* direction; its entry point is
[index.html](prototypes/g2-html/index.html). It is **a NON-PRODUCT INTERACTION PROTOTYPE**: design
evidence only, with no backend connection, mock and session data only, **no toolkit decision** and
no implementation authority. It is not production frontend code and not a decision to use web
technology. The kickoff document is authoritative over it.

The [G2 UI toolkit decision](implementation/HELM-G2-UI-TOOLKIT-DECISION.md) evaluates GTK 4 with
gtk-rs, GTK 4 with selective libadwaita, Qt 6/QML with CXX-Qt, Slint, Iced and a Tauri/webview
control candidate against the accepted D8 prototype, and records the owner's **G2-D9 acceptance of
GTK 4 + gtk-rs with selective libadwaita**, with Qt 6/QML + CXX-Qt retained as the second choice
as an architecture fallback. *Selective* is binding: HELM takes libadwaita's infrastructure — style
management, a measured-content clamp and the `.numeric` tabular-figures style class — and refuses
its GNOME idiom, so the core screens are not built from stock preference-row composition. The
canonical D8 prototype remains the visual source of truth.

The **native GTK fidelity spike was accepted on 2026-09-21**, so no Qt spike is required, and the
owner then authorised the **first real backend-connected G2 vertical** — the GUI driving the
accepted `helm-launch` 0.1 end to end through its public API. That vertical was **accepted on
2026-09-22 and integrated into `main`** by fast-forward to `a40ae8e`, with no merge commit. The
accepted native GUI selects a real local executable, drives real `helm-launch` admission of it and
of a local working directory, builds and parses the bounded fixed launch plan, shows real authority
facts, creates single-use authority, performs the real synchronous launch on a worker while staying
responsive, and shows the real child-end facts, output prefixes, receipt and digest. The acceptance
evidence is the owner's real `GtkFileDialog` smoke run **without** the `--g2-preselect` developer
helper, plus first-natural hosted CI on the branch and on `main`.

**That acceptance closes the first vertical and nothing else.** There is no persistence and no
durable Library, no long-lived session or process handle, no desktop-session environment authority,
no graphical subject support, no Wine or Proton, no App Forge, no installer, update or repair, no
containment or sandbox claim, no receipt authenticity or provenance, and **no production GUI
acceptance**. `helm-launch` is unmodified. `Indeterminate(StatusEofWithoutRecord)` is still **not**
execution success, and the owner smoke's child exit code 0 does not change that.
`crates/helm-gui` is deliberately **outside the root Cargo workspace** with its own lockfile, which
is recorded temporary architecture debt, not a hidden exception. **G-1, G-2 and G-3 are not started
and not authorised**, and the next gate is the **owner's choice of the next product capability**.

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
