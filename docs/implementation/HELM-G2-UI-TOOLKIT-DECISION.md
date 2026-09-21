# HELM G2 — UI TOOLKIT DECISION

> **Status: dossier prepared for the owner. G2-D9 is PENDING. Docs only.**
> This document evaluates candidate toolkits for the first HELM graphical application and ends in a
> recommendation. It **selects nothing**, authorises **no** GUI code, adds **no** dependency,
> changes **no** crate and touches **no** backend. G2-D9 is an owner decision and is recorded in
> [DECISIONS.md](../DECISIONS.md) when the owner makes it. Written in English under
> [ADR-0020](../adr/ADR-0020-documentation-language.md).

---

## 1. Authority and position

| Item | State |
|---|---|
| G2-D1 to G2-D7 | **ACCEPTED** 2026-09-21 — [visual kickoff](HELM-G2-VISUAL-KICKOFF.md), section 20.1 |
| G2-D8 visual direction | **ACCEPTED** 2026-09-21 — visual kickoff, sections 8.16 and 20.4 |
| **G2-D9 UI toolkit** | **PENDING** — this document |
| GUI implementation | **NOT AUTHORISED** |
| Backend work | **NOT AUTHORISED**; `helm-launch` 0.1 is product-accepted and unchanged |
| Toolkit dependency in the workspace | **NONE**, and none is added by this document |

This document expands the criteria of the visual kickoff, section 19, into a candidate evaluation.
It does not replace that section and does not alter any accepted gate.

### 1.1 What this document is not

It is not a benchmark, not a build, not a spike and not a procurement. No toolkit was installed, no
crate was added, no `Cargo.toml` was touched and no prototype was compiled for it. Every statement
about a toolkit is sourced from that project's own current documentation or registry metadata, or
is labelled as judgement.

It is also not a branding decision, not a licence decision for HELM itself, and not an answer to the
two recorded backend blockers G-2 and G-3 of the visual kickoff, section 18. Per section 16.6 of
that document, the HELM GUI is an ordinary native Linux application and is **not** a HELM subject,
so neither blocker gates this choice.

---

## 2. Method and evidence base

### 2.1 Method

1. The hard requirements were derived first, from the **accepted** product definition — not from
   toolkit marketing. They are listed in section 3 and each one cites the section of the visual
   kickoff that imposes it.
2. Candidates were then assessed against the criteria of section 4, which are the owner's A to S
   list.
3. Current version and date evidence was taken from each project's own release pages, its own
   documentation, and the crates.io registry API. Registry dates are the authoritative publication
   dates for the Rust-facing components.
4. A criterion that could not be settled from a primary source in this pass is marked as such rather
   than filled with an impression.

Search-engine summaries and blog posts were used only to locate primary documents, never as the
evidence itself.

### 2.2 Versions observed, 2026-09-21

All rows were read on **2026-09-21**. Dates are publication dates of the named version.

| Component | Version | Date | Source of the figure |
|---|---|---|---|
| GTK | 4.22.4 stable | 2026-04-30 | [gtk.org](https://www.gtk.org/) release series; 4.23 is the development series |
| `gtk4` crate (gtk4-rs) | 0.11.5 | 2026-09-20 | crates.io registry API |
| libadwaita | 1.10 | with GNOME 51 | [GNOME 51 developer notes](https://release.gnome.org/51/developers/index.html); 1.9 shipped with GNOME 50 |
| `libadwaita` crate | 0.9.2 | 2026-07-07 | crates.io registry API |
| Qt | 6.10 current, 6.8 LTS | 6.8 LTS supported to 2029 | [Qt releases](https://doc.qt.io/qt-6/qt-releases.html) |
| CXX-Qt | 0.10.0 | 2026-08-24 | crates.io registry API; [KDAB/cxx-qt](https://github.com/KDAB/cxx-qt) |
| Slint | 1.18.1 | 2026-09-21 | crates.io registry API; [slint-ui/slint releases](https://github.com/slint-ui/slint/releases) |
| iced | 0.14.0 | 2025-12-07 | crates.io registry API; [iced-rs/iced releases](https://github.com/iced-rs/iced/releases) |
| Tauri | 2.11.6 | 2026-09-19 | crates.io registry API |
| AccessKit | 0.25.0 | 2026-08-29 | crates.io registry API; [AccessKit](https://github.com/AccessKit/accesskit) |
| `ashpd` (XDG portals) | 0.13.13 | 2026-07-17 | crates.io registry API |
| `rfd` (file dialogs) | 0.17.2 | 2026-01-12 | crates.io registry API |

Two observations follow from the table itself and are worth stating before any opinion:

- **Every candidate is alive.** Nothing here is evaluated on 2023 impressions; five of the twelve
  rows were published within the last ninety days, and one within the last day.
- **`accesskit`, `ashpd` and `rfd` are listed deliberately.** They are the components a toolkit
  without native Linux desktop integration must bolt on, and their presence or absence in a
  candidate's dependency graph is evidence, not opinion.

---

## 3. What the first HELM GUI must actually do

The accepted definition already constrains the toolkit. These are **gating** requirements: a
candidate that cannot meet one does not become acceptable by scoring well elsewhere.

| # | Hard requirement | Imposed by |
|---|---|---|
| **H1** | Real assistive-technology support on Linux, not a stated intention. Every control **and every status region** carries a meaningful accessible name; a state word is part of that name. | Kickoff 8.15 |
| **H2** | Programmatic announcements. Entering and leaving *Launch attempt in progress*, and the arrival of a result, must be announced to assistive technology. This is a live-region or announcement API, not a label. | Kickoff 8.15 |
| **H3** | Full keyboard operability, with focus moving into a disclosure panel on open and returning to its opener on close. | Kickoff 8.15 |
| **H4** | A real native file chooser for *Choose local program* and the working folder, via the desktop portal. | Kickoff 8.3, T2, T3 |
| **H5** | Every technical value selectable and copyable, individually and in full; receipt bytes copied **unreformatted**. | Kickoff 8.15, 13.3 |
| **H6** | Dense, tabular, virtualised Advanced views that stay responsive over long captured output and long evidence lists. | Kickoff 8.9, 8.12, 13.4, 15 |
| **H7** | Progressive disclosure as a first-class structure, with mode switching that does not re-flow the page so far that the person loses their place. | Kickoff 8.12, 8.14, 15 |
| **H8** | Discrete phase-based progress. **No indeterminate spinner anywhere in the launch path.** | Kickoff 8.14 |
| **H9** | A long, synchronous, blocking `launch` call driven from a worker thread, with results marshalled back to the interface thread — creating no async API, no session handle and no cancel. | Kickoff 12.8, clarification 2 |
| **H10** | A recognisably HELM visual identity: warm off-white surfaces, graphite navigation, one burgundy accent, and no state conveyed by colour alone. | Kickoff 8.16 |

**H1, H2 and H5 are the discriminating requirements.** They are where the candidates genuinely
differ, and they are the requirements most often assumed rather than checked.

---

## 4. Criteria

The owner's A to S list, restated as what each means *for this product*.

| Key | Criterion | What is actually assessed |
|---|---|---|
| A | Linux desktop maturity | Years in production on Linux desktops, breadth of shipped applications, distro presence |
| B | Wayland maturity | Wayland as the primary session: scaling, fractional scaling, input, popups, clipboard |
| C | Rust interoperability | Cost and safety of the boundary between accepted Rust crates and the interface layer |
| D | Accessibility | Working AT-SPI exposure today, including H1 and H2, not a roadmap |
| E | Keyboard and focus | Focus chains, focus containment in disclosure panels, mnemonics, IME |
| F | Custom visual-design freedom | How far a custom visual language can go, and what it costs to keep it working |
| G | Reproducing the HELM direction | Whether section 8.16 specifically is achievable without fighting the toolkit |
| H | File chooser and portals | Native chooser through xdg-desktop-portal, in-toolkit or bolted on |
| I | Packaging and distro integration | Whether the toolkit is already on the target machine, and what shipping costs |
| J | Testing and UI automation | Headless runs, inspectable state, scripted interaction |
| K | Long-term maintenance | Governance, bus factor, release discipline, breaking-change history |
| L | Performance, startup, memory | Cost of an application that is mostly idle and occasionally busy |
| M | Dependency footprint | What enters the build and what enters the image |
| N | Licensing implications | Obligations placed on HELM, against an **unadopted** HELM licence policy |
| O | Developer tooling | Inspector, live preview, hot reload, designer-facing tooling |
| P | Progressive-disclosure technical UI | Native support for the pattern of section 15 |
| Q | Evidence-heavy screens | Virtualised lists and tables, text selection, copy fidelity |
| R | Portability outside the final OS image | Cost if HELM tooling must later run on another platform |
| S | Relationship to a future HELM shell | Deliberately **low weight** — see section 9 |

Ratings are **STRONG**, **GOOD**, **MIXED**, **WEAK**. No numeric score is produced, because the
evidence does not support arithmetic and a total would hide which criteria are gating.

---

## 5. Candidates

Five serious candidates and one that is assessed only far enough to decide whether it is serious.

| Candidate | Treated as |
|---|---|
| GTK 4 + gtk-rs | Serious |
| GTK 4 + gtk-rs with selective libadwaita | Serious, and assessed **separately** from the row above |
| Qt 6 / QML with a Rust bridge (CXX-Qt) | Serious |
| Slint | Serious |
| iced | Serious |
| Tauri / webview | Assessed to the point of decision only — section 8 |

**Not included, and why.** Qt Widgets rather than QML: it is a viable engineering choice but it is
the weaker half of Qt on criterion F, which is the criterion the owner specifically raised; where it
differs from QML it is noted inside the Qt assessment rather than given a column. egui and other
immediate-mode libraries: immediate-mode rendering has no retained accessibility tree, which fails
H1 and H2 structurally rather than temporarily. Flutter and Electron: neither materially changes the
decision — Flutter's Linux desktop embedder is GTK-hosted and adds a second runtime on top, and
Electron is the webview case of section 8 with a larger footprint. The list is not inflated.

---

## 6. Assessments

### 6.1 GTK 4 + gtk-rs

GTK 4.22.4 stable, `gtk4` crate 0.11.5, minimum Rust 1.83. LGPL-2.1-or-later for GTK; the gtk-rs
bindings are MIT.

**Strengths.** Present on essentially every Linux desktop already installed. Accessibility is
native and structural: GTK 4 exposes **AT-SPI on Linux and BSD** through the `GtkAccessible`
interface, with roles mapped to WAI-ARIA, and states, properties and relations carried by every
standard control, per the
[GTK accessibility documentation](https://docs.gtk.org/gtk4/section-accessibility.html). That covers
H1 directly, and GTK provides an announcement path for H2. `GtkFileDialog` goes through the desktop
portal, satisfying H4 with no extra dependency. `GtkColumnView` and `GtkListView` are virtualised
list and table widgets, which is exactly H6, and `GtkLabel` selectability plus `GtkTextView` cover
H5 including unreformatted copy. `GtkExpander`, `GtkRevealer` and `GtkStack` make H7 ordinary.
Discrete phase progress (H8) is trivial. The GTK Inspector is a genuinely strong live debugging and
accessibility-auditing tool.

**Weaknesses.** Styling is GTK's **CSS subset**, not web CSS: selectors are limited, layout is done
by widgets rather than by stylesheet, and the node names a stylesheet targets are toolkit
implementation detail. Custom widgets mean GObject subclassing, which in Rust is real ceremony
(`glib::wrapper!`, `ObjectSubclass`, properties, signals) even though it is well documented. UI
automation is the weakest of the mature options: there is no first-class scripted-interaction
framework, and testing falls back to AT-SPI tooling and GTK's own test backend.

**Material risks.** A HELM stylesheet loaded at application priority can be perturbed by a user's
own `gtk-4.0/gtk.css` and by distro themes; this is manageable but must be designed for, not
discovered. GTK's CSS node structure is not a compatibility promise, so a deeply-targeted stylesheet
carries per-release maintenance.

**What HELM gains.** Accessibility, portal integration, virtualised evidence tables and copy
fidelity — the four gating requirements — essentially for free, from a stack already on the target
machine, under a licence that constrains HELM the least.

**What HELM gives up.** Some design ceiling relative to QML, a weaker automation story than Qt, and
comfortable portability off Linux.

### 6.2 GTK 4 + selective libadwaita

Assessed separately, as the owner required. libadwaita 1.10 ships with GNOME 51; the `libadwaita`
crate is 0.9.2.

libadwaita is **not** a different toolkit. It is a widget and style layer on top of GTK 4, so every
GTK finding above still holds. What changes is the visual contract and the structural vocabulary.

**Strengths.** The structural widgets map almost one-to-one onto HELM's accepted screen map:
navigation split views for sidebar-plus-content, status pages for empty states, toast overlays,
alert dialogs, preference-style rows, and expander rows for progressive disclosure. libadwaita 1.9
added dedicated sidebar widgets, and 1.10 adds `AdwCssClassBinding` for binding CSS classes to
object properties declaratively. Adopting these removes a large amount of layout code that HELM
would otherwise write and maintain itself. Light and dark and **high-contrast** appearances are
handled automatically, which serves 8.15 rather than fighting it.

**Weaknesses.** libadwaita carries GNOME's visual conventions on purpose. Its guidance is to *"use
CSS variables instead of hardcoded colors"* and to *"use accent color variables and the `.accent`
style class"* rather than to impose a bespoke palette, per the
[libadwaita styles documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/styles-and-appearance.html).
Its behaviour is to follow the **system** accent; an application that overrides the accent must then
handle foreground colours itself, and `AdwStyleManager` continues to report the system colour rather
than the override. A HELM-burgundy identity is therefore something libadwaita permits but does not
help with.

**Material risks.** The decisive one: **the libadwaita stylesheet is not a stable public API.** A
HELM identity built by overriding its internal styling is exposed to breakage at every GNOME cycle —
two releases a year — and the breakage surfaces as visual regressions, which are exactly the
regressions no test catches. The second risk is drift: the more Adwaita widgets are used unmodified,
the more HELM looks like a GNOME application, which section 8.16.4 explicitly rejects.

**What HELM gains.** Substantially less structural UI code, correct light/dark/high-contrast
behaviour, and a proven sidebar-and-content architecture.

**What HELM gives up.** Visual distinctiveness by default, and a maintenance liability proportional
to how hard the identity pushes against the stylesheet.

**This is not a reason to reject libadwaita.** It is a reason to bound it — see section 7.

### 6.3 Qt 6 / QML with CXX-Qt

Qt 6.10 current with Qt 6.8 LTS supported into 2029; CXX-Qt 0.10.0, published 2026-08-24 and
actively released by KDAB across 2025 and 2026.

**The Rust boundary, stated precisely.** Using Qt does **not** require C++ product logic. The
accepted crates — `helm-evidence`, `helm-app-spec`, `helm-observe`, `helm-bind`, `helm-launch` —
remain the domain layer in Rust and are not ported. QML is presentation only. What Qt requires is a
bridge that exposes Rust state as `QObject`-shaped properties, signals and invokables; CXX-Qt is
that bridge, and it is MIT or Apache-2.0. The realistic division is: Rust owns every file
descriptor, every capability and every receipt; the bridge exposes plain values and enumerations;
QML never sees a descriptor. That division is clean and is one of Qt's better properties here.

**Strengths.** The strongest design freedom of any candidate, and the strongest for a
designer-led identity specifically. Qt Quick Controls supports one-off customisation, reusable
custom controls, and **complete custom styles** — a directory of QML files named after the controls,
built on `QtQuick.Templates`, declared by a `qmldir`, per the
[customisation documentation](https://doc.qt.io/qt-6/qtquickcontrols-customize.html). That is a
documented, supported path to a wholly HELM visual language rather than a themed host toolkit.
States, transitions and animation are first-class language constructs. Accessibility is real: Qt
implements `QAccessible` with an AT-SPI bridge on Linux, used with Orca in production for over a
decade. Testing is the best of the field — `QtTest`, QML `TestCase`, `qmltestrunner`, and the
`offscreen` platform plugin for genuinely headless runs. Qt Creator and Qt Design Studio are the
strongest designer-facing tooling of any candidate.

**Weaknesses.** Footprint: a QML application pulls Qt Quick, Qt Quick Controls and a scene-graph
graphics stack, and both startup and resident memory exceed GTK's for a mostly-idle utility.
Packaging is correspondingly heavier. And the accessibility strength comes with a catch that matters
here: **a fully custom QML style owns its own accessibility annotations.** Qt Quick Controls carry
`Accessible` defaults; bespoke items do not, and HELM's H1 and H2 would have to be written
deliberately for every custom control. That is achievable and it is not a defect, but it converts
accessibility from a property of the toolkit into a discipline of the team.

**Material risks.** Two, both real.

1. **CXX-Qt is pre-1.0.** The build-system API was declared stable at 0.8 with a stated intent to
   avoid backwards-incompatible changes, and the cadence is healthy, but 0.10.0 is still a `0.x`
   release from a single vendor, load-bearing for the entire application boundary. This is the
   single largest technical risk in the Qt option.
2. **LGPLv3.** Open-source Qt is LGPL-3.0 or GPL-3.0, with a commercial licence as the alternative,
   per [Qt licensing](https://doc.qt.io/qt-6/licensing.html). LGPLv3 carries relinking obligations
   and, for a "User Product", installation-information obligations. HELM's own licence policy is
   **Proposed and not adopted** — see [LICENSE-DECISION.md](../../LICENSE-DECISION.md). Choosing Qt
   therefore pre-commits part of a decision the owner has not yet made, and it does so specifically
   in the direction that matters most for an eventual OS image.

**What HELM gains.** Maximum visual freedom, the best testing story, and a clean Rust-domain /
QML-presentation split.

**What HELM gives up.** Footprint, a pre-1.0 boundary crate, self-owned accessibility on custom
controls, and licence freedom that is not yet the owner's to spend.

### 6.4 Slint

Slint 1.18.1, published 2026-09-21 — the same day as this survey. Development is fast and visible.

**Strengths.** The best Rust ergonomics of any candidate: a declarative `.slint` DSL compiled at
build time, no FFI, no GObject ceremony, no separate scripting runtime. Slint renders every element
itself, so visual freedom is total and the HELM palette is trivially expressible. Animations are
first-class. Startup and memory are excellent. Developer tooling — live preview, a VS Code
extension, SlintPad — is good, and there is a testing API.

**Weaknesses.** Desktop Linux is not Slint's centre of gravity; embedded is. Accessibility is
provided through **AccessKit** on the winit backend, enabled by default, and AccessKit's own
repository states that its adapters *"don't yet support all types of UI elements or all of the
properties in the schema"*. Slint's own tracker carries open accessibility work on text-input
widgets, a report that it cannot yet take advantage of newer AccessKit capabilities such as list
views, and a report that the accessibility feature has a significant performance impact on large
list views. For a product whose Advanced mode is *made of* long lists and tables, that last one is
not a footnote. There is no in-toolkit portal file chooser: H4 requires `rfd` or `ashpd` as a
separate dialog stack.

**Material risks.** **Licensing is the governing risk.** Slint is GPLv3, or a paid royalty-free or
enterprise subscription. For any HELM that is not itself GPLv3, a commercial subscription becomes a
permanent operating dependency of an OS-facing product. As with Qt, this pre-empts an owner decision
that has not been made — but more sharply, because the alternative to the copyleft branch is a
recurring commercial relationship with a single vendor rather than a one-time compliance posture.
The second risk is ecosystem concentration: Slint is open-core from one company.

**What HELM gains.** Rust-native ergonomics, total visual freedom, a small fast binary.

**What HELM gives up.** Mature desktop accessibility today, native portal integration, distro
presence, and licence neutrality.

### 6.5 iced

iced 0.14.0, published 2025-12-07. A substantial release: reactive rendering, hot reloading,
input-method support, explicit `x11` and `wayland` feature flags, headless-mode testing,
first-class end-to-end testing, and the `comet` debugger.

**Strengths.** MIT licensed, which is the cleanest licence position of any candidate and leaves the
owner's policy entirely open. Rust-native with no FFI. The 0.14 testing work is genuinely notable —
headless testing and time-travel debugging are better than what GTK offers. Total visual freedom
through a custom renderer. Small and fast. It has a serious production downstream in the COSMIC
desktop, which is real evidence of viability at scale.

**Weaknesses, and the decisive one.** **iced has no screen-reader support.** It does not integrate
AccessKit; neither iced 0.14 nor its winit dependency pulls `accesskit` or any AT-SPI binding, and
the accessibility issue on the iced tracker remains open. This is not a maturity gradient, it is an
absence: there is no accessibility tree for AT-SPI to read. **H1 and H2 cannot be met**, and section
8.15 of the accepted definition is not a preference that can be traded away.

**Material risks.** Even if AccessKit integration lands, it would then be new code in a `0.x`
toolkit, and HELM would be depending on the maturity curve rather than on a shipped capability.
H4 also requires an external dialog crate.

**What HELM gains.** The cleanest licence, excellent testability, Rust-native ergonomics.

**What HELM gives up.** Accessibility, which is gating. That ends the evaluation for the production
application regardless of the rest.

---

## 7. Comparative table

Ratings are **for HELM's accepted requirements**, not general quality. Gating criteria are marked.

| | Criterion | GTK 4 + gtk-rs | GTK 4 + sel. libadwaita | Qt 6 / QML + CXX-Qt | Slint | iced | Tauri / webview |
|---|---|---|---|---|---|---|---|
| A | Linux desktop maturity | STRONG | STRONG | STRONG | MIXED | MIXED | GOOD |
| B | Wayland maturity | STRONG | STRONG | STRONG | GOOD | GOOD | GOOD |
| C | Rust interoperability | STRONG | STRONG | MIXED | STRONG | STRONG | MIXED |
| **D** | **Accessibility (gating)** | **STRONG** | **STRONG** | **GOOD** | **MIXED** | **WEAK** | **MIXED** |
| **E** | **Keyboard / focus (gating)** | **STRONG** | **STRONG** | **GOOD** | **MIXED** | **MIXED** | **GOOD** |
| F | Design freedom | GOOD | MIXED | STRONG | STRONG | STRONG | STRONG |
| G | Reproduces HELM direction | GOOD | MIXED | STRONG | STRONG | STRONG | STRONG |
| **H** | **File chooser / portal (gating)** | **STRONG** | **STRONG** | **GOOD** | **MIXED** | **MIXED** | **GOOD** |
| I | Packaging / distro | STRONG | STRONG | MIXED | MIXED | MIXED | MIXED |
| J | Testing / UI automation | MIXED | MIXED | STRONG | GOOD | STRONG | MIXED |
| K | Long-term maintenance | STRONG | STRONG | MIXED | MIXED | MIXED | MIXED |
| L | Performance / startup / memory | STRONG | STRONG | MIXED | STRONG | STRONG | WEAK |
| M | Dependency footprint | MIXED | MIXED | WEAK | GOOD | GOOD | WEAK |
| N | Licensing implications | STRONG | STRONG | MIXED | WEAK | STRONG | GOOD |
| O | Developer tooling | GOOD | GOOD | STRONG | GOOD | GOOD | STRONG |
| P | Progressive-disclosure UI | STRONG | STRONG | STRONG | GOOD | GOOD | GOOD |
| **Q** | **Evidence-heavy screens (gating)** | **STRONG** | **STRONG** | **STRONG** | **MIXED** | **MIXED** | **GOOD** |
| R | Portability off the OS image | MIXED | MIXED | STRONG | STRONG | STRONG | STRONG |
| S | Future shell relationship (low weight) | GOOD | GOOD | STRONG | MIXED | GOOD | WEAK |

Reading the table honestly: **no candidate wins every criterion, and the ranking is decided by the
five gating rows, not by the fourteen others.** GTK — with or without libadwaita — is the only
candidate that is STRONG on all five. Qt is acceptable on all five. Slint and iced are not, today.

---

## 8. The webview option, and why it is rejected

Tauri 2.11.6. On Linux, Tauri renders with **WebKit2GTK** — the documented prerequisite is
`libwebkit2gtk-4.1-dev`, alongside `libxdo`, `librsvg2` and an app-indicator library, per the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

The rejection is technical, not aesthetic. Webview interfaces can be beautiful and accessible; ARIA
is a mature accessibility model and WebKitGTK bridges it to AT-SPI. The objections are these:

1. **Non-deterministic rendering surface.** Tauri uses the platform webview, so on Linux the engine
   is WebKitGTK at whatever version the distribution ships — not the engine used on Windows or
   macOS. HELM's entire product culture is that a claim is recorded with the thing that produced it.
   An interface whose rendering and layout engine varies per machine, and which HELM neither pins
   nor validates, is at odds with that culture for no capability HELM needs.
2. **An uncontrolled transformation stage over bytes that must not be transformed.** Section 13.3 of
   the accepted definition requires that copying a receipt copies the **exact bytes**,
   unreformatted, because reformatting changes the digest. A webview architecture places a JSON and JavaScript
   marshalling boundary between `helm-launch` receipts and the screen. Every value HELM shows would
   round-trip through a serialisation layer whose normalisation behaviour HELM does not own. This is
   the strongest single objection: it is not a performance complaint, it is a correctness hazard in
   exactly the part of the product that must be literal.
3. **Footprint against the actual workload.** HELM's GUI is idle almost all of the time and busy
   rarely. WebKitGTK is among the heaviest dependency trees on a Linux desktop and runs a
   multi-process engine per window. The cost is paid continuously for a benefit HELM does not use.
4. **Permanent acquisition of a second platform.** Choosing a webview means adopting a JavaScript
   toolchain, an npm supply chain and a front-end framework lifecycle as standing dependencies of an
   OS-facing product, and maintaining them alongside Rust for the life of the product.
5. **No native integration advantage.** File dialogs still route through a portal plugin, so H4 is
   no easier than it is for Slint or iced.

**Verdict: REJECT for both the G2 prototype and the G2 production application.** HELM should not
become a web application accidentally, and here it would not even be a favourable accident.

---

## 9. The shell future, kept separate

HELM may eventually have a custom shell. That possibility is recorded, and it is deliberately given
**low weight** here, for a reason that is structural rather than diplomatic:

**a compositor is a different program.** A future HELM shell would be a separate process built on a
compositor library — Smithay or wlroots in the Rust and C ecosystems, or Qt's Wayland Compositor
APIs — and would make its own toolkit decision at its own gate. Nothing about the toolkit used by a
desktop application running *inside* a session constrains the framework used to *implement* a
session. GTK even has a panel and layer-shell story for surfaces that need it.

The converse is the real danger: choosing a toolkit because it could one day also build a
compositor would mean accepting worse accessibility, worse portal integration or worse evidence
tables **today**, in the only HELM product that actually exists, to hedge a shell that has not been
proposed, scoped or authorised. That trade is rejected. Criterion S is recorded in the table and
does not move the recommendation.

---

## 10. Recommendation

### 10.1 Primary recommendation

**GTK 4 with gtk-rs, using libadwaita selectively and under a bounded rule, with the HELM identity
expressed as a HELM-owned style layer.**

**Suitable for: the G2 prototype and the G2 production application — both.** This matters. It means
the spike is not throwaway work and the prototype is the first increment of the product rather than
a rehearsal for it.

**The major reason, stated plainly:** GTK is the only candidate that satisfies all five gating
requirements today, from a stack already installed on the target machine, under a licence that
leaves the owner's unadopted licence policy untouched. Accessibility, portal file choosing,
virtualised evidence tables and faithful copy are not features HELM would be adding to GTK — they
are what GTK already is. The criteria GTK loses on — design ceiling, UI automation, portability off
Linux — are real costs, but each is a cost HELM can absorb or mitigate, whereas an accessibility
gap or a licence pre-commitment is not.

**The bounded libadwaita rule.** Adopt libadwaita for **structure and behaviour**: navigation split
views, status pages, toasts, dialogs, rows and expander rows. Do **not** build the HELM identity by
overriding libadwaita's internal CSS nodes, because that stylesheet is not a stable API and the
breakage it produces is visual, silent and untested. Express the identity through GTK CSS custom
properties, a HELM token sheet, and a small number of HELM-owned custom widgets where the identity
genuinely demands one. Wrap libadwaita widgets behind a thin HELM layer so that dropping any one of
them later is a local change. If the design and the stylesheet come into direct conflict, the
correct response is to write a HELM widget, not to fight Adwaita's cascade.

**Honest statement of what this costs the owner.** GTK's design ceiling is lower than QML's, and the
owner is a graphic designer. HELM will be able to look like HELM in GTK — warm off-white, graphite
navigation, one burgundy accent, calm density — but achieving a strongly bespoke control vocabulary
will cost more effort in GTK than in QML, and some of that effort lands in GObject subclassing
rather than in design. That is the trade being recommended, and it should be accepted with open
eyes rather than discovered later.

### 10.2 Second choice

**Qt 6 / QML with a Rust domain layer behind CXX-Qt.**

**Suitable for: the G2 prototype and the G2 production application — both**, subject to two
conditions being accepted rather than deferred: the pre-1.0 status of CXX-Qt as a load-bearing
boundary, and the LGPLv3 obligations against an unadopted HELM licence policy.

Qt wins outright on design freedom, animation, developer tooling and testability, and its Rust
boundary is architecturally clean — the accepted crates stay Rust, QML stays presentation, and no
descriptor crosses into QML. If the owner's priority ordering puts visual distinctiveness and
designer tooling above footprint and licence neutrality, Qt is the correct answer rather than a
consolation.

### 10.3 The conditions that would flip the ranking

Recorded now so that a later change is a decision rather than a drift.

| If this becomes true | Then |
|---|---|
| The HELM identity proves to be materially damaged by GTK's styling model in the spike | Re-open G2-D9 and move to Qt / QML, rather than fighting the cascade or shipping a diluted identity |
| The owner adopts a licence policy compatible with LGPLv3 and accepts the footprint | Qt becomes a genuine co-primary rather than a second choice |
| CXX-Qt reaches 1.0 with a stability commitment | The largest technical objection to Qt is removed |
| Scripted UI automation becomes a hard requirement | Qt's advantage on criterion J becomes decisive |
| HELM tooling must run on Windows or macOS as a first-class target | Criterion R stops being a tolerable cost and the recommendation must be re-derived |

### 10.4 Reject and defer

| Candidate | Disposition | Reason |
|---|---|---|
| **Tauri / webview** | **REJECT** for prototype and production | Section 8: non-deterministic rendering surface, a marshalling layer over bytes that must stay literal, footprint against an idle workload, and a second platform acquired permanently |
| **iced** | **REJECT for production; DEFER as re-evaluable** | No accessibility tree and no screen-reader support today, so H1 and H2 cannot be met and section 8.15 cannot be satisfied. Excellent on licence and testability; worth re-evaluating if AccessKit integration ships and matures |
| **Slint** | **DEFER** | Strong Rust ergonomics and visual freedom, but AT-SPI coverage is incomplete via AccessKit, accessibility has a reported cost on large lists — the exact shape of HELM's Advanced mode — there is no in-toolkit portal chooser, and GPLv3-or-subscription pre-empts an owner licence decision. Re-evaluate if the licence question is settled and accessibility matures |
| **libadwaita as a visual strategy** | **REJECT as a strategy; ACCEPT as a bounded library** | Adopting GNOME's visual language wholesale contradicts section 8.16.4. Adopting its structural widgets behind a HELM layer does not |
| **Qt Widgets instead of QML** | **DEFER** | Viable and lighter than QML, but weaker precisely on criterion F, which is the criterion that would make anyone choose Qt over GTK in the first place |

### 10.5 What this recommendation is not

It is not a selection. **G2-D9 is PENDING and remains an owner decision.** Nothing here authorises
a dependency, a crate, a spike or a line of GUI code.

---

## 11. Implementation spike, after G2-D9 acceptance only

Proposed scope, **not authorised and not implemented**. It exists so that the owner can see what
would be asked for next, and so that D9 acceptance is not mistaken for implementation authority.

### 11.1 Scope

A very small application shell, and nothing else:

1. Application shell and window.
2. Sidebar and navigation between the four surfaces of kickoff section 8.16.6.
3. **Library** — empty state and in-session state only, presented as in-memory for the session, per
   kickoff clarification 1 and section 5.2.1.
4. **Authority review** — static layout, including the containment statement of section 11.4 and
   the four grants and four non-grants, as text.
5. **Launch attempt** — static or state-driven layout with discrete phases, no spinner, driven by
   hardcoded state, per H8.
6. **Result** — static or state-driven layout, with the Advanced disclosure present and the digest
   rendered as data per section 13.3.
7. **HELM visual tokens** — the working palette of section 8.16.2 as a token sheet.

### 11.2 Hard exclusions

- **No backend connection.** The spike calls no accepted crate and adds no HELM crate dependency.
  A backend connection requires separate authorisation.
- No `helm-launch` call, no subject process, no execution of anything.
- No file chooser wired to real admission, no real measurement, no real receipt.
- No persistence and no Library that outlives the process.
- No update, repair or recovery surfaces.
- No `DESIGN_ONLY_FUTURE` control carrying primary emphasis.

### 11.3 What the spike must demonstrate to count as successful

Stated in advance so that the result is falsifiable rather than impressionistic.

| # | Exit criterion |
|---|---|
| 1 | The HELM palette and density of 8.16 are recognisable, and the result does not read as a stock GNOME application |
| 2 | Every screen is fully operable from the keyboard alone, including disclosure controls, with focus entering and returning correctly |
| 3 | A screen reader reads every control and every status region with a meaningful name, and a simulated state change is announced |
| 4 | An Advanced table of several thousand synthetic rows scrolls without perceptible lag, and any value can be selected and copied exactly |
| 5 | Mode switching between Normal and Advanced does not lose the reader's place |
| 6 | The amount of stylesheet code needed to reach criterion 1 is recorded, as the honest measure of the maintenance cost accepted in 10.1 |

Criterion 6 exists because section 11 of the owner's direction is right: *customisable* and *cheap
to maintain* are not the same property, and the spike is the cheapest place to find out which one
GTK is for HELM.

---

## 12. What this document leaves open

| Open item | Owner of the answer |
|---|---|
| G2-D9 itself | The owner |
| HELM's licence policy, still **Proposed** | The owner, in an ADR — [LICENSE-DECISION.md](../../LICENSE-DECISION.md) |
| Final brand identity; the palette of 8.16.2 is explicitly a working set | The owner |
| Packaging and distribution format | A later gate; not decided by the toolkit |
| The two backend blockers G-2 and G-3 | Kickoff section 18; **not** gating this choice, per 16.6 |
| Whether a HELM shell will exist at all | Unproposed, unscoped, unauthorised — section 9 |

---

## 13. Decision record

**G2-D9 is PENDING.** When the owner decides, the decision is recorded in
[DECISIONS.md](../DECISIONS.md) and the gate table in the
[visual kickoff](HELM-G2-VISUAL-KICKOFF.md), section 20.1, is updated to match. Until then:

- no toolkit is selected;
- no dependency exists;
- **GUI implementation is NOT AUTHORISED.**
