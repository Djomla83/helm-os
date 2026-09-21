# HELM G2 — UI TOOLKIT DECISION

> **Status: G2-D9 ACCEPTED by the owner on 2026-09-21. Docs only — acceptance selects a
> toolkit and authorises no implementation.**
> **Revision 2, 2026-09-21 — corrected under HELM-G2-D9-EVIDENCE-C** before the decision was
> taken. Five factual and decision-scope corrections were applied to revision 1; each is listed
> with its before and after in section 13, which is permanent evidence and is not rewritten.
> G2-D8 is untouched, D1 to D7 are untouched, and no product semantics changed.
> This document evaluates candidate toolkits for the first HELM graphical application, ends in a
> recommendation, and now carries the owner's decision on it. It authorises **no** GUI code, adds
> **no** dependency, changes **no** crate and touches **no** backend. The decision is recorded in
> [DECISIONS.md](../DECISIONS.md#g2-d9-toolkit-accepted). Written in English under
> [ADR-0020](../adr/ADR-0020-documentation-language.md).

---

## 1. Authority and position

| Item | State |
|---|---|
| G2-D1 to G2-D7 | **ACCEPTED** 2026-09-21 — [visual kickoff](HELM-G2-VISUAL-KICKOFF.md), section 20.1 |
| G2-D8 visual direction | **ACCEPTED** 2026-09-21 — visual kickoff, sections 8.16 and 20.4 |
| **G2-D9 UI toolkit** | **ACCEPTED** 2026-09-21 — this document; [decision record](../DECISIONS.md#g2-d9-toolkit-accepted) |
| Prototype toolkit | **GTK 4 + gtk-rs**, selected for the G2 prototype |
| Production standing | **PROVISIONAL GTK DEFAULT** — not a production lock |
| Named fallback | **Qt 6 / QML + CXX-Qt** |
| GTK/libadwaita version floor | **OPEN** — no version pair selected |
| GUI implementation | **NOT AUTHORISED** — D9 selects a toolkit, it does not open implementation |
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
| GTK | **4.24.0** stable | **2026-09-11** | Upstream `NEWS` on `main`; 4.22.5 is the head of the previous stable branch. See 2.3 |
| `gtk4` crate (gtk4-rs) | 0.11.5 | 2026-09-20 | crates.io registry API |
| libadwaita | **1.10.0**, requires **GTK >= 4.23.1** | with GNOME 51 | Upstream `meson.build` of the `libadwaita-1-10` branch; [GNOME 51 developer notes](https://release.gnome.org/51/developers/index.html). See 2.3 |
| libadwaita, previous | **1.9.4**, requires **GTK >= 4.21.1** | with GNOME 50 | Upstream `meson.build` of the `libadwaita-1-9` branch. See 2.3 |
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

### 2.3 GTK and libadwaita version floor — not selected here

Build-time compatibility between GTK and libadwaita is a hard constraint, not a preference:
**libadwaita declares a GTK minimum in its own build definition.** Read from the upstream
`meson.build` of each stable branch on 2026-09-21:

| libadwaita branch | Version | Declared `gtk_min_version` |
|---|---|---|
| `libadwaita-1-8` | 1.8.8 | `>= 4.19.4` |
| `libadwaita-1-9` | 1.9.4 | `>= 4.21.1` |
| `libadwaita-1-10` | 1.10.0 | **`>= 4.23.1`** |

GTK numbers odd minor versions as development series, so `>= 4.21.1` is satisfied in practice
by the **4.22.x** stable series and `>= 4.23.1` by the **4.24.x** stable series.

**Consequence, stated plainly: GTK 4.22.x and libadwaita 1.10 are not a compatible pair.**
libadwaita 1.10 does not build against GTK 4.22.x. Revision 1 of this document presented
GTK 4.22.4 and libadwaita 1.10 together as an unquestioned production pair without checking
that constraint. That is corrected here.

#### 2.3.1 What is actually shipped

Read on 2026-09-21 from the Debian sources API and the Launchpad publishing API. These are
**distribution archives**, which is what determines whether a user can run the application,
rather than upstream release pages, which only say what exists.

| Distribution | GTK 4 | libadwaita |
|---|---|---|
| Debian 12 bookworm, oldstable | 4.8.3 | 1.2.2 |
| Debian 13 trixie, **current stable** | 4.18.6 | 1.7.6 |
| Debian forky, testing | 4.22.4 | 1.9.2 |
| Debian sid, unstable | 4.24.0 | 1.9.2 |
| Ubuntu 24.04 LTS noble | 4.14.5 | 1.5.0 |
| Ubuntu 25.04 plucky | 4.18.5 | 1.7.0 |
| Ubuntu 25.10 questing | 4.20.1 | 1.8.0 |
| Ubuntu 26.04 LTS resolute | 4.22.4 | 1.9.1 |
| Ubuntu 26.10 stonking | 4.24.0 | 1.10.0 |

**libadwaita 1.10 is shipped by exactly one archive in that table** — Ubuntu 26.10, a
development release. Debian sid already carries GTK 4.24.0 but is still on libadwaita 1.9.2.
Targeting libadwaita 1.10 today means targeting a stack that is not yet generally installed
anywhere.

#### 2.3.2 The three candidate floors

| Option | Pair | Reach today | What it costs |
|---|---|---|---|
| **Broad reach** | GTK **4.18** + libadwaita **1.7** | Debian 13 stable, Ubuntu 25.04 and later | No `AdwSidebar`, which arrived in 1.9; no `AdwCssClassBinding`, which arrived in 1.10. HELM writes its own sidebar |
| **Conservative current** | GTK **4.22** + libadwaita **1.9** | Ubuntu 26.04 LTS, Debian forky and sid | `AdwSidebar` available; no `AdwCssClassBinding` |
| **Leading edge** | GTK **4.24** + libadwaita **1.10** | Ubuntu 26.10 only | Newest API, and effectively unavailable to most users until the next distribution cycle |

The Rust bindings do not constrain the choice. `gtk4` 0.11.5 carries feature flags from `v4_2`
to `v4_24`, and the `libadwaita` crate 0.9.2 carries `v1_1` to `v1_10` alongside `gtk_v4_2` to
`gtk_v4_24`; the floor is selected by which feature flags the application enables at build
time. `gtk4` 0.11.5 declares a **minimum Rust version of 1.92** — revision 1 said 1.83, taken
from a secondary summary, and that is corrected here from the registry metadata.

#### 2.3.3 This document does not select the floor

> **Superseded in part, 2026-09-21.** The owner later set a **spike** API floor of **GTK 4.18
> with libadwaita 1.7** when authorising the bounded implementation spike; see the
> [decision of 2026-09-21](../DECISIONS.md#g2-gtk-spike-authorised). That is an API floor for the
> spike only. **It is not a distribution support policy, not a packaging policy, not a minimum
> released image and not a production lock**, and the distribution-reach question this section
> records stays open. The reasoning below is left as written, because it is why the floor had to
> be decided explicitly rather than inherited.

**No target-machine package evidence exists in the repository.** The only Linux platform
recorded anywhere is the `ubuntu-24.04` GitHub Actions runner image used for headless CI. That
is a build and test environment, not a desktop target, and it would imply a far lower floor —
GTK 4.14 with libadwaita 1.5 — than any option above. It does not resolve the question and is
not treated as if it did.

The floor is therefore an **open decision**, carried in section 12. It must be made
explicitly: either the owner names the target distribution, or it becomes the first recorded
decision of the implementation spike, with the evidence recorded beside it. **It must not be
adopted silently by whatever happens to be installed on the first machine that builds it.**

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

GTK 4.24.0 stable, published 2026-09-11; `gtk4` crate 0.11.5, minimum Rust **1.92**.
LGPL-2.1-or-later for GTK; the gtk-rs bindings are MIT. The version floor actually adopted is
an open decision — see **2.3**, which is not settled by this assessment.

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

Assessed separately, as the owner required. libadwaita 1.10.0 ships with GNOME 51 and
**requires GTK >= 4.23.1**; libadwaita 1.9.4 requires GTK >= 4.21.1. The `libadwaita` crate is
0.9.2 and exposes `v1_1` to `v1_10`. **Which generation HELM targets is unresolved and is not
decided here — see 2.3.** Everything below holds for any of the three candidate floors unless
a specific version is named.

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

The accent behaviour is stated precisely here, because revision 1 stated it loosely:

- libadwaita **follows the system accent by default**;
- an application **can override the accent through documented, public CSS variables**.
  `--accent-bg-color`, `--accent-fg-color` and `--accent-color` are documented, and the
  [CSS variables reference](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html)
  states that *"applications can override these colors by re-declaring them"*, giving
  `:root { --accent-bg-color: #e01b24; }` as its own example;
- **`AdwStyleManager` still reports the system accent.** Its `accent-color` and
  `accent-color-rgba` properties are documented as *"the current system accent color"*, and a
  CSS override does not change what that API returns;
- overriding a background accent also requires overriding the matching standalone colour, per
  the same reference, so that contrast stays correct.

**HELM Burgundy is therefore ownable through public, documented API.** Revision 1 implied that a
HELM identity was something libadwaita *"permits but does not help with"*; that understated the
documented support and is corrected. What HELM must not rely on is anything **undocumented**.

**Material risks.** Restated to match the corrected finding above. The concern is **not** that the
HELM identity cannot be owned — it can, through public API. The concern is **dependence on
internal styling structure.** libadwaita's public surface — widgets, properties, style classes and
the documented CSS variables — is versioned and supported. Its internal widget composition and CSS
node structure are not, and an identity built by targeting those nodes breaks at GNOME cycles,
twice a year, as silent visual regressions that no test catches.

The second risk is drift: the more Adwaita widgets are used unmodified, the more HELM looks like a
GNOME application, which section 8.16.4 explicitly rejects.

The third risk is the version floor of 2.3, which is a packaging and reach question rather than a
visual one, and which is still open.

#### 6.2.1 Bounded selective-libadwaita rule

Binding if libadwaita is adopted. The boundary is **documented surface**, not "how much Adwaita".

**Permitted — public and documented:**

- libadwaita widgets with their documented properties, signals and methods;
- documented style classes;
- documented CSS variables, including the accent variables, redeclared at `:root`, with the
  matching standalone colours overridden alongside them;
- `AdwStyleManager` for reading system preferences, understanding that it reports the **system**
  accent and not a HELM override;
- automatic light, dark and high-contrast handling, which is kept rather than defeated.

**Not permitted — undocumented:**

- targeting libadwaita's internal CSS node names or a widget's internal composition;
- depending on the internal structure of any Adwaita widget template;
- copying or re-implementing the Adwaita stylesheet to reach an effect.

**Rule of resolution.** Where the HELM identity cannot be expressed through the permitted
surface, the correct response is to write a HELM-owned widget with HELM-owned CSS — never to
reach into libadwaita's internals. Every libadwaita widget is wrapped behind a thin HELM layer so
that dropping one later is a local change.

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

#### 6.4.1 Licensing, corrected

Slint 1.18.1 declares the SPDX expression
`GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` — a
**tri-licence**. Revision 1 described it as *"GPLv3, or a paid royalty-free or enterprise
subscription"*. That was wrong: the royalty-free branch is **not** a paid tier, and the error is
corrected here.

For a HELM desktop application the relevant branch is the **Slint Royalty-free Desktop, Mobile,
and Web Applications License, version 2.0**, which grants a *"world-wide, royalty-free,
non-exclusive license to use, reproduce, make available, modify, display, perform, distribute the
Software as part of a Desktop, Mobile, or Web Application"* — that is, **proprietary desktop use
at no cost**. Its conditions are specific and checkable:

| Condition in the royalty-free licence | Effect on HELM |
|---|---|
| Attribution: either an `AboutSlint` widget reachable from the top-level menu, or the Slint badge on a public download page | A **product** requirement, not an abstraction — it lands on a HELM screen and has to be designed for |
| No use within an **Embedded System** | Irrelevant while HELM is a desktop application; relevant if HELM ever ships as a device image |
| The Software may not be distributed alone, outside an Application | Not a constraint for HELM |
| An Application may not expose Slint's APIs | Not a constraint for HELM |
| Licence notices may not be removed or altered | Ordinary |

`LicenseRef-Slint-Software-3.0` is the paid commercial branch, needed for embedded use or to
drop the attribution condition. `GPL-3.0-only` remains available for a copyleft HELM.

**Net effect: materially better than revision 1 recorded.** Slint's criterion N rating is
corrected from WEAK to **GOOD**. What remains is a vendor-specific licence reference rather than
a standard OSI licence, a standing attribution obligation with a visible product consequence, and
an embedded carve-out that would matter if HELM ever became a device image. That is not as clean
as MIT or LGPL-2.1-or-later, but it does **not** force a subscription and it does **not** pre-empt
the owner's licence policy the way revision 1 claimed.

#### 6.4.2 Material risks, after the licence correction

**The governing risk is technical, not legal, and the disposition does not change.** The licence
correction removes an argument that should never have been made; it does not remove the
accessibility and performance evidence, which is what DEFER rests on:

- AccessKit's own repository states that its adapters do not yet support all UI element types or
  all schema properties;
- Slint's tracker carries open accessibility work on text-input widgets;
- Slint reports that it cannot yet use newer AccessKit capabilities such as list views;
- the accessibility feature is reported to have a significant performance cost on large list
  views — which is the exact shape of HELM's Advanced mode.

Against H1, H2 and H6 that is sufficient on its own. The secondary risk is ecosystem
concentration: Slint is open-core from one company.

**What HELM gains.** Rust-native ergonomics, total visual freedom, a small fast binary, and a
no-cost proprietary desktop licence.

**What HELM gives up.** Mature desktop accessibility today, native portal integration, distro
presence, and a standard OSI licence with no attribution obligation.

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
| N | Licensing implications | STRONG | STRONG | MIXED | GOOD | STRONG | GOOD |
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
2. **A byte-fidelity burden that the architecture adds rather than removes — corrected.**
   Revision 1 claimed that receipt bytes would *necessarily* cross a JSON boundary and that this
   made exact-byte copying impossible. **That claim was wrong and is withdrawn.** Tauri
   supports raw byte payloads in both directions: `InvokeResponseBody::Raw(Vec<u8>)` for command
   responses and `InvokeBody::Raw(Vec<u8>)` for request bodies, so exact receipt bytes can cross
   the IPC boundary unchanged. What is true, and weaker, is this: the **default** path is the
   `serde` path to `InvokeResponseBody::Json`, so byte fidelity becomes a property the application
   must deliberately preserve and test at every hop — Rust, IPC, JavaScript, DOM, clipboard —
   rather than a property it inherits. That is a cost, not a barrier, and **it is not on its own a
   reason to reject Tauri.**
3. **Footprint against the actual workload.** HELM's GUI is idle almost all of the time and busy
   rarely. WebKitGTK is among the heaviest dependency trees on a Linux desktop and runs a
   multi-process engine per window. The cost is paid continuously for a benefit HELM does not use.
4. **Permanent acquisition of a second platform.** Choosing a webview means adopting a JavaScript
   toolchain, an npm supply chain and a front-end framework lifecycle as standing dependencies of an
   OS-facing product, and maintaining them alongside Rust for the life of the product.
5. **No native integration advantage.** File dialogs still route through a portal plugin, so H4 is
   no easier than it is for Slint or iced.

### 8.1 The rejection basis, restated

With objection 2 corrected, the rejection rests on the remaining, HELM-specific constraints — and
not on any impossible-exact-bytes claim:

1. **Webview rendering and version variability.** The engine is WebKitGTK at whatever version
   the distribution ships, it differs from the engine used on other platforms, and HELM neither
   pins nor validates it. For a product whose culture is to record a claim together with the
   thing that produced it, an unpinned and unvalidated rendering surface is the wrong default.
2. **An extra browser and webview surface acquired permanently.** A web engine, a JavaScript
   toolchain, an npm supply chain and a front-end framework lifecycle become standing
   dependencies of an OS-facing product, maintained alongside Rust for its whole life.
3. **Mismatch with the desired native, deterministic UI path.** The accepted direction is a
   calm native Linux desktop application with portal integration, native accessibility and
   predictable behaviour. A webview reaches the same destination through more layers, each of
   which HELM would have to own, for no capability HELM needs.
4. **Footprint against the actual workload**, as in objection 3 above.

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

**Recommended disposition — conditional, not unconditional:**

| Scope | Disposition |
|---|---|
| **G2 prototype** | **SELECTED FOR G2 PROTOTYPE** |
| **G2 production application** | **PROVISIONAL PRODUCTION DEFAULT** — not locked |

**The production lock is conditional on the spike.** GTK becomes the production toolkit only
when the implementation spike of section 11 passes the exit criteria already defined there:
design cost, accessibility, virtualisation, keyboard and focus, and maintenance. Until those
criteria are met on something that actually runs, production standing is **provisional and
reversible**.

This wording is deliberate. Revision 1 recommended GTK for prototype and production
unconditionally, which asserted a production outcome that nothing in this document supports,
because nothing has been built. A dossier can rank candidates on evidence; it cannot lock a
production toolkit before a line of the interface exists.

**Qt 6 / QML + CXX-Qt is the named fallback**, not a candidate to be re-weighed from scratch.
If the spike fails any exit criterion, the fallback is taken and G2-D9 is re-opened. The spike
is not repaired by lowering a criterion.

**The major reason, stated plainly:** GTK is the only candidate that satisfies all five gating
requirements today, from a stack that every mainstream Linux distribution already packages — see
2.3, noting that the **version floor** within that stack is still open — under a licence that
leaves the owner's unadopted licence policy untouched. Accessibility, portal file choosing,
virtualised evidence tables and faithful copy are not features HELM would be adding to GTK — they
are what GTK already is. The criteria GTK loses on — design ceiling, UI automation, portability off
Linux — are real costs, but each is a cost HELM can absorb or mitigate, whereas an accessibility
gap or a licence pre-commitment is not.

**The bounded libadwaita rule** is defined normatively in **6.2.1** and is part of this
recommendation. In short: adopt libadwaita for **structure and behaviour**, and build the HELM
identity only on its **public, documented surface** — widgets, properties, style classes and the
documented CSS variables, including the accent variables, which HELM may legitimately redeclare.
Do **not** target undocumented CSS nodes or internal widget composition. Where the identity cannot
be expressed through the permitted surface, write a HELM-owned widget rather than fight Adwaita's
cascade, and keep every libadwaita widget behind a thin HELM layer so that dropping one later is a
local change.

**Honest statement of what this costs the owner.** GTK's design ceiling is lower than QML's, and the
owner is a graphic designer. HELM will be able to look like HELM in GTK — warm off-white, graphite
navigation, one burgundy accent, calm density — but achieving a strongly bespoke control vocabulary
will cost more effort in GTK than in QML, and some of that effort lands in GObject subclassing
rather than in design. That is the trade being recommended, and it should be accepted with open
eyes rather than discovered later.

### 10.2 Second choice — the named fallback

**Qt 6 / QML with a Rust domain layer behind CXX-Qt.**

**Recommended disposition: EXPLICIT FALLBACK.** Qt is the named destination if the GTK spike
fails, for the prototype and the production application alike, subject to two conditions being
accepted rather than deferred: the pre-1.0 status of CXX-Qt as a load-bearing boundary, and the
LGPLv3 obligations against an unadopted HELM licence policy.

The fallback triggers are exactly the spike exit criteria, and they are enumerated so that
taking the fallback is a recorded decision rather than an argument:

| Failed exit criterion | Fallback trigger |
|---|---|
| Design cost | The HELM identity is unreachable through GTK's public styling surface, or reaching it requires an amount of stylesheet code the owner judges unmaintainable |
| Accessibility | A screen reader does not read every control and status region, or state changes are not announced |
| Virtualisation | The Advanced table does not stay responsive over several thousand rows, or values cannot be selected and copied exactly |
| Keyboard and focus | Any screen is not fully operable from the keyboard, or focus does not enter and return correctly across disclosures |
| Maintenance | Holding the identity requires depending on undocumented libadwaita or GTK internals, contrary to 6.2.1 |

Qt wins outright on design freedom, animation, developer tooling and testability, and its Rust
boundary is architecturally clean — the accepted crates stay Rust, QML stays presentation, and no
descriptor crosses into QML. If the owner's priority ordering puts visual distinctiveness and
designer tooling above footprint and licence neutrality, Qt is the correct answer rather than a
consolation.

### 10.3 The conditions that would flip the ranking

Recorded now so that a later change is a decision rather than a drift.

| If this becomes true | Then |
|---|---|
| The spike fails any exit criterion of 11.3 | Take the named fallback in 10.2, re-open G2-D9, and record it — the provisional production default does not become permanent by default |
| The HELM identity proves to be materially damaged by GTK's styling model in the spike | Re-open G2-D9 and move to Qt / QML, rather than fighting the cascade or shipping a diluted identity |
| The owner adopts a licence policy compatible with LGPLv3 and accepts the footprint | Qt becomes a genuine co-primary rather than a second choice |
| CXX-Qt reaches 1.0 with a stability commitment | The largest technical objection to Qt is removed |
| Scripted UI automation becomes a hard requirement | Qt's advantage on criterion J becomes decisive |
| HELM tooling must run on Windows or macOS as a first-class target | Criterion R stops being a tolerable cost and the recommendation must be re-derived |

### 10.4 Reject and defer

| Candidate | Disposition | Reason |
|---|---|---|
| **Tauri / webview** | **REJECT** for prototype and production | Section 8.1: webview rendering and version variability that HELM neither pins nor validates, an extra browser and webview surface acquired permanently, mismatch with the desired native deterministic UI path, and footprint against an idle workload. **Not** rejected on byte fidelity — that objection was withdrawn in section 8 |
| **iced** | **REJECT for production; DEFER as re-evaluable** | No accessibility tree and no screen-reader support today, so H1 and H2 cannot be met and section 8.15 cannot be satisfied. Excellent on licence and testability; worth re-evaluating if AccessKit integration ships and matures |
| **Slint** | **DEFER**, unchanged | **Technical grounds only**, per 6.4.2: AT-SPI coverage via AccessKit is incomplete, text-input accessibility is open, newer AccessKit capabilities are not yet used, accessibility has a reported cost on large lists — the exact shape of HELM's Advanced mode — and there is no in-toolkit portal chooser. The licence objection of revision 1 is **withdrawn**: the royalty-free branch permits proprietary desktop use at no cost (6.4.1). Re-evaluate when the accessibility and large-list performance evidence changes |
| **libadwaita as a visual strategy** | **REJECT as a strategy; ACCEPT as a bounded library** | Adopting GNOME's visual language wholesale contradicts section 8.16.4. Adopting its structural widgets behind a HELM layer does not |
| **Qt Widgets instead of QML** | **DEFER** | Viable and lighter than QML, but weaker precisely on criterion F, which is the criterion that would make anyone choose Qt over GTK in the first place |

### 10.5 What this recommendation is not

The dispositions in 10.1 and 10.2 were this document's *recommendation*. **The owner accepted
them at G2-D9 on 2026-09-21**, and section 14 records the decision as taken. Two things did **not**
become true by that acceptance:

- **Production is not locked.** GTK's production standing is still *provisional*, conditional on
  the spike exit criteria of 11.3, and the fallback in 10.2 is still live.
- **Implementation is not authorised.** Accepting a toolkit authorises no dependency, no crate,
  no toolkit installation, no spike and no line of GUI code. That is a separate owner gate.

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
| **The distribution support policy** — how far back HELM must run, from the reach evidence of 2.3.1 and 2.3.2 | The owner. **Still open.** The **spike** API floor was set separately at **GTK 4.18 + libadwaita 1.7** on 2026-09-21; it binds the spike and decides no policy |
| **Whether the production default is locked** — 10.1 gives GTK a *provisional* production standing only | The owner, after the spike exit criteria of 11.3 are evaluated |
| HELM's licence policy, still **Proposed** | The owner, in an ADR — [LICENSE-DECISION.md](../../LICENSE-DECISION.md) |
| Final brand identity; the palette of 8.16.2 is explicitly a working set | The owner |
| Packaging and distribution format | A later gate; not decided by the toolkit |
| The two backend blockers G-2 and G-3 | Kickoff section 18; **not** gating this choice, per 16.6 |
| Whether a HELM shell will exist at all | Unproposed, unscoped, unauthorised — section 9 |

---

## 13. Correction record — HELM-G2-D9-EVIDENCE-C

Revision 1 of this document was committed on 2026-09-21 and corrected the same day, before the
owner decided G2-D9. Nothing was rewritten silently: each correction is stated with what revision
1 said and what the evidence actually shows.

| # | Revision 1 said | Corrected to | Evidence |
|---|---|---|---|
| **C1** | GTK 4.22.4 and libadwaita 1.10 presented together as a production pair; GTK minimum Rust 1.83 | The pair is **incompatible** — libadwaita 1.10.0 declares `gtk_min_version >= 4.23.1`. Three candidate floors are recorded and **none is selected**. `gtk4` 0.11.5 requires Rust **1.92** | Upstream `meson.build` of the `libadwaita-1-8/-1-9/-1-10` branches; GTK `NEWS` (4.24.0, 2026-09-11); Debian sources API; Launchpad API; crates.io registry — **2.3** |
| **C2** | libadwaita *"follows the system accent"*, and a HELM identity is something it *"permits but does not help with"* | Follows the system accent **by default**; the accent **can be overridden through documented CSS variables**; `AdwStyleManager` **still reports the system accent**. HELM Burgundy is ownable through public API. The maintenance concern is **dependence on internal styling structure**, not inability to own the accent | libadwaita CSS variables reference and `AdwStyleManager` documentation — **6.2**, with the bounded rule in **6.2.1** |
| **C3** | Slint is *"GPLv3, or a paid royalty-free or enterprise subscription"*; criterion N rated **WEAK** | Slint is **tri-licensed**: `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0`. The royalty-free branch permits **proprietary desktop use at no cost**, with an attribution condition and an embedded carve-out. Criterion N corrected to **GOOD**. **DEFER is unchanged**, now resting on technical evidence alone | crates.io SPDX for slint 1.18.1; upstream `LicenseRef-Slint-Royalty-free-2.0.md` — **6.4.1**, **6.4.2** |
| **C4** | Receipt bytes *necessarily* cross a JSON boundary in a webview, making exact-byte copying impossible | **Withdrawn.** Tauri supports raw payloads both ways — `InvokeResponseBody::Raw(Vec<u8>)` and `InvokeBody::Raw(Vec<u8>)` — so exact bytes can cross unchanged. What remains is that the **default** path serialises, so byte fidelity must be deliberately preserved and tested. **REJECT is unchanged**, now resting on rendering and version variability, the extra webview surface, the mismatch with a native deterministic UI path, and footprint | Tauri `ipc::InvokeResponseBody` and `ipc::InvokeBody` documentation — **8**, **8.1** |
| **C5** | GTK recommended for *"the G2 prototype and the G2 production application — both"*, unconditionally | **SELECTED FOR G2 PROTOTYPE**; **PROVISIONAL PRODUCTION DEFAULT**, with the production lock conditional on the spike exit criteria of 11.3. **Qt 6 / QML + CXX-Qt is the named fallback**, with enumerated triggers | **10.1**, **10.2**, **10.3**, **12** |

**What this round did not touch.** G2-D8 and the visual language are unchanged; D1 to D7 are
unchanged; no product semantics changed; the hard requirements H1 to H10 are unchanged; the
candidate list is unchanged; and the ranking order — GTK primary, Qt fallback, Tauri rejected,
iced rejected for production, Slint deferred — is unchanged. Two dispositions were **re-grounded
without being reversed**: Slint's DEFER and Tauri's REJECT now rest only on arguments that survive
the evidence.

---

## 14. Decision record

**G2-D9 was ACCEPTED by the owner on 2026-09-21**, on the evidence of this document at revision
2. The full record, including the owner's wording, is the
[decision of 2026-09-21](../DECISIONS.md#g2-d9-toolkit-accepted); the owner's terms are also
restated in section 20.5 of the [visual kickoff](HELM-G2-VISUAL-KICKOFF.md), whose gate table in
20.1 now reads **G2-D9 ACCEPTED**.

| What was decided | State |
|---|---|
| **GTK 4 + gtk-rs** | **SELECTED FOR G2 PROTOTYPE** |
| **Selective libadwaita** | **PERMITTED**, through the documented bounded public surface of 6.2.1 only |
| **GTK for production** | **PROVISIONAL PRODUCTION DEFAULT** — **not** a production lock; conditional on the exit criteria of 11.3 |
| **Qt 6 / QML + CXX-Qt** | **NAMED FALLBACK**, on the triggers of 10.2; an exit criterion must not be lowered to retain GTK |
| **GTK/libadwaita version floor** | At G2-D9: **OPEN**. Set for the **spike only** on 2026-09-21 at **GTK 4.18 + libadwaita 1.7**; the distribution support policy stays open, per 2.3.3 |
| **GUI implementation** | **NOT AUTHORISED** by this decision |

After this decision, and until the owner separately authorises the bounded implementation spike:

- the toolkit is selected, but **no dependency exists** — no `Cargo.toml` entry, no `Cargo.lock`
  entry, no crate, and no toolkit installation is authorised;
- **no GUI source exists** anywhere in the repository;
- the GTK and libadwaita version floor is **still not selected**, and must not be inherited from
  whatever a development machine has installed;
- the production default is **provisional**, not locked;
- no backend work, no `helm-launch` integration and no persistence is authorised;
- the HELM licence policy remains **Proposed** — [LICENSE-DECISION.md](../../LICENSE-DECISION.md)
  is unchanged and no toolkit choice accepted a licence for HELM;
- **GUI implementation is NOT AUTHORISED.**

**Update, 2026-09-21.** The owner subsequently
[authorised the bounded spike](../DECISIONS.md#g2-gtk-spike-authorised) of section 11 and set its
API floor at **GTK 4.18 + libadwaita 1.7**. That authorisation is bounded to sections 11.1 to 11.3
with their hard exclusions and exit criteria: it adds no backend connection, no `helm-launch`
call and no persistence, and it does **not** lock GTK for production.
