# HELM G2 — UI TOOLKIT DECISION

> ## G2-D9 — ACCEPTED 2026-09-21
>
> **Selected: GTK 4 + gtk-rs, with *selective* libadwaita.**
>
> **Second choice, if the fidelity spike fails materially: Qt 6 + QML + CXX-Qt.**
>
> The owner accepted the primary recommendation of section 13 and authorised **one bounded native
> UI fidelity spike** (section 15). This document is the record of that decision and of the
> analysis behind it.
>
> **What acceptance does and does not do.** It selects HELM's UI technology and authorises the
> spike. It does **not** make the GUI production-ready, connect any backend, add any HELM crate
> dependency, introduce persistence, or start G-1, G-2 or G-3. The next gate is the **owner's
> visual review of the native fidelity spike**, not production acceptance.
>
> **Selective, not wholesale.** Section 6.3 is binding: HELM takes libadwaita's infrastructure and
> refuses its idiom. This decision is **not** a statement that HELM's screens should be built from
> stock libadwaita widgets, and the canonical D8 prototype remains the visual source of truth.

Prepared 2026-09-21, immediately after G2-D8 acceptance and the publication of the canonical
prototype at `498e1b7798ca2754e69f3900aebde084c8ffc0e8`. Accepted the same day; ecosystem versions
were re-verified against primary sources at acceptance and the corrections are carried in
section 4.

Sections 5 to 11 record the comparative analysis as it stood at the decision. **They are not
reopened by acceptance** and are preserved as the reasoning of record.

---

## 1. Current G2 authority

| Gate | State |
|---|---|
| G2-D1 to G2-D7 | **ACCEPTED** 2026-09-21, with four recorded clarifications |
| G2-D8 — visual direction | **ACCEPTED** 2026-09-21, *Record / graphite frame* ([kickoff 8.16](HELM-G2-VISUAL-KICKOFF.md)) |
| G2-D9 — UI toolkit selection | **ACCEPTED** 2026-09-21 — GTK 4 + gtk-rs with selective libadwaita |
| Bounded native UI fidelity spike | **AUTHORISED** (section 15) |
| Production GUI | **NOT ACCEPTED** — the spike answers a fidelity question and nothing more |
| Backend connection | **NOT AUTHORISED** |

What is settled: what HELM G2 does, what it refuses to claim, how it is laid out, how it reads, and
what it looks like. What is not settled: what draws it.

`crates/helm-launch` 0.1 is product-accepted and unchanged by anything here. This decision concerns
a **presentation layer only**. HELM's domain logic stays in Rust regardless of which candidate wins;
no candidate requires moving product logic into another language, and a candidate that did would be
rejected on that ground alone.

### 1.1 The prototype is design evidence, not a technology argument

The canonical D8 artefact is HTML, CSS and vanilla JavaScript. **This is not an argument for a web
technology.** HTML was the fastest medium in which to make an interaction reviewable and it is
treated here as a drawing, not as a codebase. Section 10 evaluates the webview option on its
technical merits and rejects it; the prototype's medium is not counted in its favour.

### 1.2 Mock state is expected

The owner accepts that the prototype carries mock state throughout. That is the intended condition
at D8. The prototype becomes materially more useful once real HELM state is connected behind it, and
**that connection is not a reason to revisit the visual design**. D9 selects the production
technology capable of implementing the accepted interface faithfully; it does not reopen the
interface.

---

## 2. Canonical D8 visual target

Every candidate below is assessed against one concrete artefact —
[docs/prototypes/g2-html/index.html](../prototypes/g2-html/index.html) — not against toolkits in the
abstract. These are the demands that artefact actually makes.

### 2.1 Structural demands

| Demand | As built in the prototype |
|---|---|
| Custom window shell | A 36 px graphite bar (`#1A222D`), three inert dots, a centred window label. Not a stock header bar with stock controls |
| Asymmetric desktop layout | A fixed 250 px graphite rail against a flexible content sheet; the rail is part of the window frame, not a panel floating on a background |
| Inset working sheet | The warm sheet (`#F8F6F4`) is inset with an asymmetric margin (`0 16px 16px 0`) and a 6 px radius, so graphite frames it on three sides only |
| Measured content column | Content clamps to 1000 px and centres inside a wider sheet, with 38/48/44 px padding |
| Status bar | A hairline-separated bar with subject state on the left and three tabular facts on the right |
| Modal overlay | A 50 %-opacity scrim over the whole window carrying a drawn stand-in for the desktop file dialog |

### 2.2 Typographic and fine-detail demands

| Demand | As built in the prototype |
|---|---|
| Two-family pairing | A serif display face for headings and the pull-quote; a sans face for everything else |
| Negative tracking | `-0.006em` globally, `-0.015em` on display headings |
| Tabular figures | Every path, byte count, digest, timestamp, exit code and receipt value aligns in columns |
| Hairlines in three weights | `#1F2937` for structural rules, `#DED9D3` for section rules, `#E6E1DB` for row rules — all 1 px |
| Ledger rows | A fixed 196 px label column against a flexible 620 px body; numbered lists with a 28 px ordinal column and 150/170/210 px key columns |
| Monospace evidence blocks | Digest and raw receipt bytes, wrapped on arbitrary characters, selectable and copyable |
| Geometric state marks | 9 px outlined square, outlined diamond (rotated 45°), outlined circle, filled square — carrying state alongside its word, never colour alone |
| A 2 px meter | The elapsed-against-bound bar on the attempt screen |

### 2.3 Behavioural demands

Sidebar navigation across seven screens; a modal chooser with Escape-to-close; progressive technical
disclosure that shows and hides per-section detail; an attempt state that deliberately offers **no
controls at all**; copy-to-clipboard of exact bytes; real buttons throughout, keyboard reachable,
with a visible focus ring.

### 2.4 What the design does *not* demand

This matters as much as what it does, and it is where several candidates' headline strengths turn
out to be irrelevant.

- **No animation of consequence.** One meter fill. No transitions carry meaning.
- **No custom-rendered graphics.** No canvas, no charts, no shaders, no 3D, no particle work.
- **No card elevation.** The direction explicitly rejects floating cards, drop shadows and
  manufactured hierarchy (kickoff 8.16.4). The single shadow in the whole design is on the file
  chooser stand-in, which production replaces with a real system dialog.
- **No exotic layout.** Every screen is boxes in rows and columns with fixed and flexible tracks.

**The accepted design is structurally conventional and stylistically distinct.** It needs precise
control over *paint* — colour, border, radius, spacing, font, tracking, transform — over a layout
model every candidate already has. It does not need a drawing surface. Any candidate that can be
styled precisely can build this; the differentiator is therefore **platform integration**, not
rendering power.

### 2.5 Production replaces one prototype element

The drawn "desktop file chooser (stand-in)" is the one screen element that is explicitly a
placeholder for a system service. In production it becomes a real file dialog, routed through
`xdg-desktop-portal` where appropriate. Each candidate is judged on how cleanly it gets there.

---

## 3. Decision criteria

Grouped by weight for this product, which is a **Linux OS project** whose first visible surface is a
launcher that must be trustworthy on the machine it manages.

### 3.1 Hard requirements — failure here disqualifies

- Native Wayland support, with X11 fallback
- An accessibility tree reachable by AT-SPI, so Orca and related tooling can drive the application
- Keyboard navigation and focus management sufficient for full keyboard operation
- IME and complex-input handling
- A real system file chooser, portal-routed
- Clipboard, high-DPI and fractional-scaling correctness
- Rust as the product language, with product logic staying in Rust
- A licence compatible with shipping HELM

### 3.2 Heavily weighted — the owner's stated priority

- **Design freedom**: can the accepted visual language be built without fighting the framework, and
  without a custom rendering layer nobody can maintain?
- Precise typographic control, including tracking, line height and figure style
- Freedom from a stock host identity — HELM must not read as stock GNOME, stock Qt or generic web
- Custom widget composition, and control styling that survives without losing accessibility
- **Theme independence**: a user's system theme must not be able to repaint HELM and break the
  accepted design

### 3.3 Weighted

Linux desktop maturity · Rust interoperability · resource footprint · startup time · packaging and
distribution · build complexity · testability and UI automation · developer tooling · ecosystem
health · long-term maintenance risk · suitability for evidence-heavy, text-dense surfaces

### 3.4 Explicitly out of scope

- **Desktop notifications** are not needed by G2 and are treated as a later, portal-level concern
  that no candidate blocks.
- **Future App Library complexity** is considered only as *list and detail density at scale* —
  virtualised lists, sorting, incremental search. No candidate is selected on speculation about
  features HELM has not specified.

### 3.5 The future HELM shell is a separate question

**No candidate is chosen on the assumption that it must later become HELM's compositor or desktop
shell.** G2 is the first HELM desktop *application*. A future HELM shell is a different problem with
different constraints — compositor protocols, session management, privileged surfaces — and may well
use a different substrate. Choosing an application toolkit for shell reasons today would weight
speculation above the product actually in hand.

The one legitimate forward-looking consideration is narrower: **does the choice strand HELM in an
ecosystem with no path forward?** That is assessed under maintenance risk, not as a shell
requirement.

---

## 4. Candidate inventory

Versions re-verified against primary sources at acceptance on 2026-09-21 — the GNOME GitLab release
tags for GTK and libadwaita, and the crates.io registry for the Rust bindings. Full source list in
section 16.4.

| # | Candidate | Current state |
|---|---|---|
| A | **GTK 4 + gtk-rs** | **GTK 4.24.0 (2026-09-11)**, the series shipped with GNOME 51; 4.22.5 (2026-09-10) is the preceding stable series. `gtk4` crate **0.11.5 (2026-09-20)**, released on a 1–2 month cadence; `glib` 0.22.10 |
| B | **GTK 4 + selective libadwaita** | **libadwaita 1.10.0 (2026-09-14)**, shipped with **GNOME 51 (2026-09-16)**; `libadwaita` crate **0.9.2 (2026-07-07)**, which pairs with `gtk4 ^0.11` and `glib`/`gio`/`pango ^0.22`. CSS variables and media queries have been supported since 1.8 |
| C | **Qt 6 / QML + CXX-Qt** | Qt 6.11.2 current; Qt 6.8 LTS supported to 2029-10-08; CXX-Qt 0.7, bridge macro API stabilised, heading to 1.0 |
| D | **Slint** | 1.18.1 (2026-09-21); triple-licensed GPLv3 / royalty-free / commercial |
| E | **Iced** | 0.14 (December 2025); self-described experimental; one more release planned before 1.0 |
| F | **Tauri / webview** *(control)* | Tauri 2.10.1 (2026-03-04); WebKitGTK 4.1 on Linux |

No further candidate is included. egui was considered and set aside: it is an immediate-mode
toolkit whose text layout, accessibility and native integration are weaker than every candidate
above for a text-dense, evidence-heavy desktop application, and including it would not change the
decision. Relm4 and libcosmic are not separate candidates — Relm4 is an architectural layer over
gtk4-rs and is noted under A/B; libcosmic is a fork of Iced and is noted under E.

---

## 5. GTK 4 + gtk-rs

### 5.1 Maturity and Rust bindings

GTK 4 is the mainline toolkit of the GNOME platform and the best-tested GTK generation on Wayland.
The `gtk4` crate is a first-party binding maintained by the gtk-rs organisation, released on a
1–2 month cadence through 2026, generated from the same GObject introspection tooling that gives
Rust bindings to the wider GObject ecosystem. Rust is a fully supported way to write GTK 4
applications, not a community afterthought. Relm4 0.11 sits above it for applications that want an
Elm-shaped state model; that is an optional architectural choice, not a dependency of this decision.

### 5.2 Linux integration — the strongest of any candidate

Native Wayland. Native AT-SPI. Native IME. `GtkFileDialog` (GTK 4.10+) is the modern file-chooser
API and routes through `xdg-desktop-portal` where appropriate, so HELM gets the user's real system
dialog rather than a drawn approximation. Clipboard, high-DPI and fractional scaling are handled by
the platform layer. For portal surfaces beyond the file chooser, the `ashpd` crate provides
idiomatic Rust access with explicit GTK 4 integration.

This is the criterion HELM weighted hardest, and GTK 4 wins it outright.

### 5.3 Accessibility — the strongest of any candidate

GTK 4 dropped ATK and speaks AT-SPI directly. Widgets implement `GtkAccessible`, setting roles,
states, properties and relations taken more or less directly from WAI-ARIA, with an AT-SPI backend
exposing them on the accessibility bus. Standard controls are accessible by default. Most widgets
carry the labels and relations Orca needs.

Gaps remain — screen-reader journey coverage, high-contrast compliance and automated auditing in CI
are not solved problems anywhere — but GTK 4 starts from a working accessibility tree rather than
from nothing. **No other candidate can say that on Linux today.**

### 5.4 Styling: what GTK CSS can and cannot do

This is the decisive technical question for HELM, and it needs to be stated precisely rather than
waved at.

**GTK 4 CSS supports**, per current GTK documentation: `font-family`, `font-size`, `font-weight`,
`font-style`, `font-variant`, `letter-spacing`, `text-transform`, `line-height`, `text-decoration`,
`text-shadow`; `color`, `opacity`, `filter`, `caret-color`; `border-width`, `border-style`,
`border-color`, `border-radius`, `border-image`; `padding` and `margin` on all sides; `box-shadow`,
`transform`, `transform-origin`; the full transition and animation properties; and the background
properties. Selectors are CSS Level 3, including `:hover`, `:focus`, `:disabled`, `:checked`,
`:nth-child()` and the standard combinators. Custom properties — CSS variables — are supported from
GTK 4.16.

**GTK 4 CSS does not do layout.** There is no `display`, no `flex`, no `grid`, no `position`, no
`width`/`height`. Layout is expressed in code through layout managers (`GtkBoxLayout`,
`GtkGridLayout`, `GtkCenterLayout`). Percentages are not accepted for margin, min-width or padding.
Physical units resolve through GTK's `-gtk-dpi` rather than a fixed 96 dpi, and `rem` resolves
differently from the web.

**What this means for the accepted design.** Map section 2 onto that capability list:

| HELM demand | GTK 4 route |
|---|---|
| Graphite frame, warm sheet, 6 px radius, asymmetric inset | CSS background, border-radius, margin — direct |
| Three hairline weights | CSS border — direct |
| Burgundy tokens | CSS custom properties (4.16+) — direct |
| Tracking, line height, two families | CSS `letter-spacing`, `line-height`, `font-family` — direct |
| Rotated diamond state mark | CSS `transform: rotate(45deg)` — direct |
| 2 px meter | CSS-styled `GtkProgressBar` or a styled box — direct |
| 250 px rail / flexible sheet | `GtkBox` with a size request — code, not CSS |
| 196 px label / flexible body ledger rows | `GtkGrid` or `GtkBox` with size requests — code, not CSS |
| 1000 px clamped, centred column | Custom layout, or `AdwClamp` — see section 6 |
| Modal scrim overlay | `GtkOverlay` — code |
| Custom thin title bar | `gtk_window_set_titlebar()` with an arbitrary widget — supported |

Every *visual* demand is directly expressible. Every *layout* demand moves from CSS into Rust code.
For a designer this is the real cost: **paint and structure separate**. In the prototype a single
CSS rule says both "196 px wide" and "15 px semibold graphite"; in GTK those become a size request in
Rust and a style rule in CSS. The design survives intact; the authoring experience is less direct,
and a design change that alters structure requires a code change rather than a stylesheet change.

**One item this dossier left open, now resolved.** Tabular figures run through the entire ledger
design, and `font-feature-settings` / `font-variant-numeric` are **not** listed among GTK 4's
supported CSS properties. The open question was therefore how HELM gets tabular figures at all.

**It is answered by libadwaita.** libadwaita provides a **`.numeric` style class** that makes a
widget use tabular figures, documented as equivalent to a `PangoAttrFontFeatures` attribute with
`tnum=1`, and intended for exactly HELM's case — multiple labels vertically aligned, or numbers
that change while staying aligned. HELM therefore applies `.numeric` to ledger values, byte counts,
digests, timestamps and the elapsed readout, with a direct Pango `font_features` attribute available
as the fallback for any widget that style class does not reach.

This is worth stating plainly: **the tabular-figures problem is solved by the support layer, not by
bare GTK.** It is an independent, concrete argument for selective libadwaita that was not available
when section 6 was first written. The spike still verifies it in practice (section 15.4) rather than
taking the documentation's word for it.

### 5.5 Avoiding a stock GNOME identity

GTK 4's default theme is Adwaita, so an unstyled HELM would read as a GNOME application. The
standard remedy is an application stylesheet loaded at `GTK_STYLE_PROVIDER_PRIORITY_APPLICATION`,
which sits above the theme. Given the property list in 5.4, HELM's palette, hairlines, typography
and state marks can all be imposed that way.

**The residual risk is theme interference.** On bare GTK 4 a user's system theme can still repaint
widget internals HELM did not explicitly override, and HELM has no way to enumerate what a
third-party theme will do. For an application whose visual precision is an accepted product decision
this is a genuine hazard — and it is the single strongest argument for section 6.

### 5.6 Packaging, testing, tooling

Flatpak and distribution packaging for GTK 4 applications are thoroughly trodden ground. GTK
Inspector provides live widget and CSS inspection, which is the closest analogue to a browser's
element inspector and directly useful when translating the prototype. UI automation runs through
AT-SPI, which means the same tree that serves accessibility also serves testing — a genuine
double benefit that no candidate without an accessibility tree can offer.

### 5.7 Assessment

| Dimension | Rating |
|---|---|
| Linux desktop maturity | **STRONG** |
| Wayland | **STRONG** |
| Accessibility | **STRONG** |
| Portals / file chooser | **STRONG** |
| Keyboard, focus, IME | **STRONG** |
| Rust interoperability | **STRONG** |
| Design freedom | **GOOD** — full control of paint; layout leaves CSS for code |
| Theme independence | **MIXED** — application CSS is overridable at the edges |
| Footprint and startup | **GOOD** |
| Packaging | **STRONG** |
| Testability / automation | **STRONG** |
| Ecosystem health | **STRONG** |
| Maintenance risk | **STRONG** |
| Licensing | **STRONG** — LGPL, no obligations that affect HELM |

---

## 6. GTK 4 + selective libadwaita

Assessed separately from GTK itself, as instructed, and explicitly **not** as all-or-nothing.

### 6.1 What libadwaita offers HELM

**Theme locking.** libadwaita applications use their own stylesheet and do not follow arbitrary
system GTK themes. For most projects this is contentious; **for HELM it is the point**. G2-D8 is an
accepted product decision about how HELM looks. A toolkit configuration in which a user's theme can
silently overwrite that decision is a worse fit than one that cannot. This directly answers the
residual risk in section 5.5.

**CSS variables and media queries.** libadwaita 1.8 fully supports CSS media queries, letting light,
dark and high-contrast styles be defined in one file, and combines them with variables. HELM's token
set — burgundy, graphite, warm off-white, muted rose — maps onto exactly that mechanism, and
high-contrast support is an accessibility obligation HELM would otherwise have to hand-roll.

**A content clamp.** `AdwClamp` implements "clamp to a maximum width and centre in a wider parent",
which is precisely the 1000 px measured column of section 2.1. Without it, HELM writes a custom
layout manager for a problem libadwaita has already solved.

**Dialogs and adaptive primitives.** `AdwDialog` and `AdwAlertDialog` give modal presentation with
correct focus handling and accessibility, replacing the hand-built scrim of the prototype. Adaptive
navigation primitives exist if HELM ever needs a narrow layout.

**Tabular figures.** The `.numeric` style class delivers `tnum=1` on any widget, which is the one
typographic demand of section 2.2 that bare GTK CSS cannot express. See section 5.4.

### 6.2 What libadwaita costs HELM

**An opinionated base stylesheet.** libadwaita's stylesheet expresses GNOME's visual conventions —
rounded boxed lists, generous padding, card-shaped preference groups, a specific control idiom.
HELM's accepted direction explicitly rejects card surfaces and manufactured elevation
(kickoff 8.16.4). Adopting libadwaita's *widgets* while rejecting its *look* means overriding more
defaults than bare GTK would require in those places.

**Idiom pull.** The strongest pull toward stock GNOME is not the stylesheet but the widget
vocabulary. `AdwPreferencesPage`, `AdwPreferencesGroup`, `AdwStatusPage` and the boxed-list style
classes encode GNOME's information architecture. Building HELM's ledger rows out of
`AdwPreferencesGroup` would make HELM look like a settings panel — which is exactly the failure mode
the owner is concerned about.

### 6.3 The selective position

The distinction that resolves the tension: **take libadwaita's infrastructure, refuse its idiom.**

| Take | Refuse |
|---|---|
| `AdwApplication` / `AdwStyleManager` — theme locking, light/dark/high-contrast | `AdwPreferencesPage`, `AdwPreferencesGroup` |
| `AdwClamp` — the measured content column | `AdwStatusPage` |
| `AdwDialog` / `AdwAlertDialog` — modal presentation and focus | boxed-list and card style classes |
| CSS variables and media queries | `AdwBanner`, `AdwToast` and other GNOME-idiom furniture unless a real need appears |
| Adaptive navigation primitives, if narrow layouts ever matter | Adwaita's default spacing and control look wherever it contradicts D8 |

Ledger rows, the graphite rail, the state marks and the evidence blocks stay as HELM's own composed
widgets over `GtkBox` and `GtkGrid`, styled by HELM's stylesheet. libadwaita supplies the window
shell, the theme lock, the clamp and the dialogs.

This is a policy the spike can test directly: if HELM's screens can be built this way without
`AdwPreferencesGroup` appearing anywhere, the policy holds.

### 6.4 Assessment

As GTK 4 in section 5.7, with these deltas:

| Dimension | Rating |
|---|---|
| Theme independence | **STRONG** *(up from MIXED — the decisive improvement)* |
| Design freedom | **GOOD** — unchanged in ceiling; higher floor via clamp and dialogs, with a live risk of idiom drift |
| Accessibility | **STRONG** — dialogs and high-contrast support improve on bare GTK |
| Maintenance risk | **STRONG** — but tied to GNOME's release cadence and to libadwaita deprecations |

---

## 7. Qt 6 / QML + CXX-Qt

### 7.1 Design freedom — the best of any candidate

QML composes interfaces from primitives — `Rectangle`, `Text`, `Item` — with arbitrary anchoring and
layout, a full property-binding system, states and transitions as first-class constructs, and shader
effects when wanted. There is no stock look to override because there is no stock look imposed:
`Rectangle` is a rectangle. Every demand in section 2 is direct, including layout, which QML
expresses declaratively rather than pushing into imperative code.

Typographic control is precise: `Text` and `FontLoader` expose family, weight, letter spacing, line
height and OpenType font features directly, so the tabular-figures question that section 5.4 leaves
open for GTK has a clean answer here.

For a designer, QML is also the most *legible* of the candidates. Its mental model — a declarative
tree with bindings — is closer to the HTML prototype than anything else on this list, and `qmlscene`
hot-reloading gives an iteration loop comparable to editing CSS in a browser. **If design velocity
were the only criterion, this would be the recommendation.**

### 7.2 Rust interoperability

CXX-Qt 0.7 stabilised the `cxx-qt` bridge macro API and is working toward 1.0 with `cxx-qt-build`
and `cxx-qt-lib` next. It supports implementing `QObject` subclasses in Rust, usable from C++, QML
and JavaScript, and can either integrate Rust into a C++/CMake application or build a Rust
application with Cargo.

**HELM's domain logic would stay in Rust.** CXX-Qt is explicitly designed for that shape: Rust
objects exposed as QML models and properties, with QML doing presentation only. The concern that Qt
forces product logic into C++ does not apply.

What it does add is a **second build system and a C++ toolchain**. A Qt SDK must be present, CMake
enters the build, and the Rust↔Qt boundary is generated code that must be maintained across Qt and
CXX-Qt versions. Compared with `cargo build` against a GObject-introspected binding, this is a
material increase in build complexity — for a project whose existing crate is a careful,
tightly-bounded Rust artefact, that is a real cost.

### 7.3 Linux integration and accessibility

Qt 6 has mature Wayland support and a well-developed Linux desktop presence through KDE. File
dialogs route to the system dialog via the platform theme, including portal integration.

**Accessibility is the weak point relative to GTK.** Qt Quick exposes accessibility through the
`Accessible` attached property, bridged to AT-SPI on Linux. It works, but it is opt-in per item
rather than default-on: a `Rectangle` and a `Text` composed into a custom control are not accessible
until HELM declares them so. Because section 7.1's design freedom comes precisely from composing
primitives rather than using stock controls, **HELM would be building its accessibility tree largely
by hand**. Qt Quick's accessibility is also less mature than Qt Widgets', and both are less battle-
tested on Linux than GTK's.

This is a direct and uncomfortable trade: the same property that makes QML the best canvas for
HELM's visual language makes it the most work to make accessible.

### 7.4 Licensing, footprint, packaging

Qt 6 is dual-licensed: LGPLv3/GPLv3 open source, or commercial. LGPLv3 is workable for HELM provided
Qt is dynamically linked and relinking rights are preserved — a constraint to note now rather than
discover at packaging time, and one that becomes sharper if HELM ever wants static linking or a
tightly controlled image. Qt 6.8 LTS is supported to 2029-10-08; Qt 6.11 is current with support to
2027-03-17.

Footprint is the largest of the native candidates: Qt6 Core/Gui/Quick plus the QML engine and its
JavaScript runtime. For a launcher this is heavier than GTK and much heavier than Slint.

### 7.5 Assessment

| Dimension | Rating |
|---|---|
| Design freedom | **STRONG** — the best on this list |
| Typography control | **STRONG** |
| Animation / transitions | **STRONG** *(and largely unused by this design)* |
| Linux desktop maturity | **STRONG** |
| Wayland | **STRONG** |
| Portals / file chooser | **GOOD** |
| Keyboard, focus, IME | **GOOD** |
| Accessibility | **MIXED** — bridged and real, but opt-in per item and weaker on Linux than GTK |
| Rust interoperability | **GOOD** — CXX-Qt 0.7 is real and pre-1.0 |
| Build complexity | **MIXED** — C++ toolchain, CMake, generated boundary |
| Footprint / startup | **MIXED** |
| Packaging | **GOOD** |
| Testability | **GOOD** |
| Ecosystem health | **STRONG** |
| Maintenance risk | **GOOD** — Qt is durable; CXX-Qt is young |
| Licensing | **MIXED** — LGPLv3 obligations, or commercial |

---

## 8. Slint

### 8.1 Strengths

Rust-first: Slint's primary target language is Rust, with no foreign build system and no C++
toolchain. The `.slint` declarative language gives design freedom close to QML's — custom
everything, no imposed look — with a compiler that generates Rust. Footprint and startup are the
best of any candidate here by a wide margin, because Slint was built for resource-constrained
targets. Current release 1.18.1, dated the day this dossier was prepared; the project ships
continuously.

For reproducing section 2's visual language, Slint is comfortably capable.

### 8.2 Material risks

**Accessibility is not yet sufficient for an OS-facing application.** Slint's accessibility goes
through AccessKit to AT-SPI. Two documented gaps matter directly to HELM:

- **Text input widgets are not exposed** to the accessibility tree
  ([slint-ui/slint#2895](https://github.com/slint-ui/slint/issues/2895)). HELM G2's current screens
  are read-only, so this is survivable *today* — but HELM will not stay read-only.
- **Enabling accessibility has a significant performance cost on large lists**
  ([slint-ui/slint#3867](https://github.com/slint-ui/slint/issues/3867)), with the AccessKit path
  dominating runtime. A future App Library is exactly a large list.

The AccessKit layer beneath is itself still settling: a fix for the Unix adapter reporting disabled
controls as enabled was merged upstream on 2026-09-01 and had not yet reached a published release as
of 2026-09-21.

**Native desktop integration requires assembly.** Slint has no built-in native file dialog; HELM
would reach for `rfd` or `ashpd` and wire portal integration itself. GTK gets this from
`GtkFileDialog` for free.

**Single-vendor and market-focus risk.** Slint is controlled by one company whose commercial
emphasis is embedded. Desktop Linux is a supported target, not the strategic one. That is a
different risk profile from GTK (GNOME Foundation, distribution-wide) or Qt (Qt Company plus KDE).

**Licensing needs a decision.** Slint is triple-licensed: GPLv3; a royalty-free desktop licence
that requires disclosing Slint use — via an `AboutSlint` widget or badge — and excludes embedded;
or commercial. Any of these is usable, but each carries an obligation HELM would have to accept
deliberately, including a visible attribution surface in a product whose UI copy is tightly
controlled.

### 8.3 Assessment

| Dimension | Rating |
|---|---|
| Design freedom | **STRONG** |
| Rust interoperability | **STRONG** |
| Footprint / startup | **STRONG** |
| Build complexity | **STRONG** |
| Wayland | **GOOD** |
| Keyboard / focus | **GOOD** |
| Accessibility | **WEAK** — documented gaps in text input and list performance |
| IME | **MIXED** |
| Portals / file chooser | **MIXED** — external crates, hand-wired |
| Packaging | **GOOD** |
| Testability | **MIXED** |
| Ecosystem health | **MIXED** — single vendor, embedded-first |
| Maintenance risk | **MIXED** |
| Licensing | **MIXED** — triple licence, attribution or commercial |

---

## 9. Iced

### 9.1 Strengths

The most Rust-native architecture on the list and the cleanest state model: an Elm loop where every
state change is explicit, which suits HELM's discipline about states and facts unusually well.
Custom widgets and custom rendering are available, giving real visual freedom. 0.14 brought
substantial input work — IME preedit with variable text size, cursor-size awareness, `unfocus` and
`is_focused` focus operations, `physical_key` on keyboard events — plus X11 and Wayland feature
flags, headless-mode testing and end-to-end testing support. Iced is proven at desktop scale:
COSMIC is built on it.

### 9.2 The disqualifying gap

**Iced has no accessibility.** Neither iced 0.14 nor the winit version beneath it depends on
AccessKit. An Orca user sees nothing — the application is simply absent from the accessibility tree.
[iced-rs/iced#552](https://github.com/iced-rs/iced/issues/552) tracks this and it remains open.

For a Linux OS project this is not a rough edge to work around. A launcher that mediates access to
the user's programs and cannot be operated by assistive technology is not shippable as HELM's first
visible surface. Nothing HELM can do at application level fixes it, because the gap is below HELM in
the stack.

That COSMIC ships a **fork** (`libcosmic`) carrying its own accessibility work is the clearest
available evidence: the people who most needed iced to be accessible on the desktop concluded they
had to maintain their own tree. HELM adopting iced today means either accepting no accessibility, or
taking on that same fork burden.

### 9.3 Further risks

The project describes itself as experimental; 1.0 has not landed. The built-in widget set is thin
relative to HELM's needs, so more is hand-built — which, without an accessibility layer, compounds
9.2. Native integration (file dialogs, portals) is assembled from external crates as with Slint.

### 9.4 Assessment

| Dimension | Rating |
|---|---|
| Rust-native architecture | **STRONG** |
| State model fit for HELM | **STRONG** |
| Design freedom | **GOOD** |
| Testability | **GOOD** — headless and end-to-end testing in 0.14 |
| Wayland | **GOOD** |
| IME / input | **GOOD** — materially improved in 0.14 |
| Keyboard / focus | **GOOD** |
| **Accessibility** | **WEAK** — absent; screen readers see nothing |
| Portals / file chooser | **MIXED** |
| Widget maturity | **MIXED** |
| API stability | **MIXED** — self-described experimental, pre-1.0 |
| Maintenance risk | **MIXED** |
| Licensing | **STRONG** — MIT |

---

## 10. Tauri / webview — control candidate

Included as a reference point, not as a live option, and judged on technical merit only. **The fact
that the D8 prototype is HTML is not evidence for Tauri and is given no weight.**

### 10.1 What it would get right

Visual fidelity would be near-perfect and nearly free, because the accepted design already exists as
CSS. Iteration speed for the designer would be the best available. This is a genuine strength and it
should be stated plainly before it is rejected.

### 10.2 Why it is rejected

**The Linux webview is the weakest platform.** Tauri uses WebKitGTK 4.1 on Linux, against WKWebView
on macOS and WebView2 on Windows. Documented problems include high input latency and low frame rates
in GPU-heavy views that run fast in a normal browser, and — with some drivers, NVIDIA in
particular — disagreements between WebKitGTK and the graphics driver producing anything from subtle
rendering artefacts to blank windows. A blank window is a total failure of a launcher. HELM's
primary and only committed platform is exactly the one where the webview is least dependable.

**Rendering is not consistent across engines.** WebKitGTK, WKWebView and WebView2 render some CSS
differently, so the "write once" fidelity advantage is conditional even on its own terms.

**Accessibility depends on the embedded engine**, not on HELM, and WebKitGTK's AT-SPI bridging is
less predictable than GTK's native tree. HELM would inherit an accessibility story it cannot
inspect, fix or reason about.

**Architecturally wrong for this product.** HELM is an operating-system project; `helm-launch` 0.1
is a deliberately minimal, auditable Rust crate whose whole design premise is a small, inspectable
authority boundary. Embedding a browser engine — a large, fast-moving, network-capable attack
surface with its own JIT and its own update cadence — in order to draw a program launcher inverts
that premise. The resource cost and the security-review surface are both far out of proportion to
the interface in section 2, which is boxes, rules and text.

**It also contradicts the product's own character.** HELM's accepted direction is "serious, local,
capable". Shipping a web runtime to draw a launcher undercuts that claim in precisely the dimension
HELM is trying to establish.

### 10.3 Assessment

| Dimension | Rating |
|---|---|
| Visual fidelity to D8 | **STRONG** |
| Designer iteration speed | **STRONG** |
| Linux rendering reliability | **WEAK** — documented driver and performance failures |
| Accessibility | **WEAK** — inherited from the engine, not inspectable |
| Footprint | **WEAK** |
| Security / review surface | **WEAK** |
| Desktop integration | **MIXED** |
| Suitability for an OS-facing application | **WEAK** |

**Verdict: REJECTED** for the production HELM application, on Linux rendering reliability,
accessibility opacity, and architectural mismatch with an OS project. It remains legitimate as what
it already is — the medium of the design prototype.

---

## 11. Comparison table

Descriptive ratings only. No numeric scoring.

| Criterion | GTK 4 | GTK + sel. libadwaita | Qt 6 / QML | Slint | Iced | Tauri |
|---|---|---|---|---|---|---|
| Linux desktop maturity | STRONG | STRONG | STRONG | GOOD | GOOD | MIXED |
| Wayland | STRONG | STRONG | STRONG | GOOD | GOOD | MIXED |
| **Accessibility** | **STRONG** | **STRONG** | MIXED | WEAK | **WEAK** | WEAK |
| Keyboard / focus | STRONG | STRONG | GOOD | GOOD | GOOD | GOOD |
| IME / input | STRONG | STRONG | GOOD | MIXED | GOOD | GOOD |
| Native file chooser | STRONG | STRONG | GOOD | MIXED | MIXED | MIXED |
| Portal integration | STRONG | STRONG | GOOD | MIXED | MIXED | MIXED |
| Clipboard | STRONG | STRONG | STRONG | GOOD | GOOD | GOOD |
| High-DPI / fractional | STRONG | STRONG | STRONG | GOOD | GOOD | MIXED |
| **Design freedom** | GOOD | GOOD | **STRONG** | **STRONG** | GOOD | STRONG |
| Typography control | GOOD | GOOD | STRONG | STRONG | GOOD | STRONG |
| Custom widget composition | GOOD | GOOD | STRONG | STRONG | GOOD | STRONG |
| Animation / transitions | GOOD | GOOD | STRONG | STRONG | GOOD | STRONG |
| **Theme independence** | MIXED | **STRONG** | STRONG | STRONG | STRONG | STRONG |
| Avoids stock host identity | GOOD | GOOD | STRONG | STRONG | STRONG | MIXED |
| Rust interoperability | STRONG | STRONG | GOOD | STRONG | STRONG | GOOD |
| Build complexity | STRONG | STRONG | MIXED | STRONG | STRONG | MIXED |
| Footprint / startup | GOOD | GOOD | MIXED | STRONG | GOOD | WEAK |
| Packaging / distribution | STRONG | STRONG | GOOD | GOOD | GOOD | MIXED |
| Testability / UI automation | STRONG | STRONG | GOOD | MIXED | GOOD | MIXED |
| Developer tooling | STRONG | STRONG | STRONG | GOOD | MIXED | GOOD |
| Ecosystem health | STRONG | STRONG | STRONG | MIXED | GOOD | GOOD |
| Maintenance risk | STRONG | STRONG | GOOD | MIXED | MIXED | MIXED |
| Licensing | STRONG | STRONG | MIXED | MIXED | STRONG | STRONG |
| Evidence-heavy UI suitability | STRONG | STRONG | STRONG | GOOD | GOOD | STRONG |
| Future App Library density | STRONG | STRONG | STRONG | MIXED | GOOD | GOOD |
| **Prototype suitability** | GOOD | GOOD | STRONG | STRONG | GOOD | STRONG |
| **Production suitability** | STRONG | **STRONG** | GOOD | MIXED | WEAK | WEAK |

### 11.1 What HELM gains and gives up, per candidate

| Candidate | HELM gains | HELM gives up |
|---|---|---|
| **GTK 4** | The best Linux integration and the only default-on accessibility tree; one accessibility tree serving both Orca and UI automation; trivial Rust build | CSS-driven layout; some theme-interference exposure |
| **GTK + selective libadwaita** | All of the above, plus theme locking that protects the D8 decision, a ready content clamp, accessible dialogs, and high-contrast via media queries | Constant discipline against GNOME idiom drift; more overriding where Adwaita's look contradicts D8 |
| **Qt 6 / QML** | The best design canvas and the most legible authoring model for a designer; declarative layout; precise font-feature control | Default accessibility — the tree is hand-built; a C++ toolchain and CMake; the heaviest native footprint; LGPL obligations |
| **Slint** | Rust-first with no foreign build system; the smallest, fastest binary; strong visual freedom | Accessibility sufficient for an OS product; hand-wired portals; single-vendor risk; a licence obligation to accept deliberately |
| **Iced** | The cleanest state model on the list, well matched to HELM's discipline; genuinely good testing support | Accessibility entirely — or the cost of maintaining a fork; API stability before 1.0 |
| **Tauri** | Near-free visual fidelity and the fastest design iteration | Linux rendering reliability; an inspectable accessibility story; a small review surface; architectural coherence with an OS project |

---

## 12. Risks

### 12.1 Risks in the recommended direction

| Risk | Severity | Mitigation |
|---|---|---|
| GTK CSS cannot express layout, so structural design changes become code changes | Medium | The spike builds the two hardest layouts — the ledger rows and the clamped column — before anything is committed |
| ~~Tabular figures may not be reachable from GTK CSS~~ | **Closed** | Resolved by libadwaita's `.numeric` style class (`tnum=1`), with Pango `font_features` as fallback. See 5.4; the spike still verifies it in practice |
| libadwaita idiom pull toward a stock GNOME look | Medium | The explicit take/refuse policy in 6.3; the spike must reach all seven screens without `AdwPreferencesGroup` |
| The two-family serif/sans pairing may not survive font availability across distributions | Medium | Font selection is deliberately **not** part of D9; the spike uses fallback stacks and records the result, as the prototype already does |
| GNOME release cadence forces churn; libadwaita deprecations | Low–Medium | Selective use keeps the libadwaita surface small and replaceable |
| gtk-rs binding lag behind a new GTK release | Low | Cadence is 1–2 months; HELM does not need bleeding-edge GTK |

### 12.2 Risks in the alternatives

| Risk | Where | Severity |
|---|---|---|
| Hand-built accessibility tree across every custom control | Qt / QML | High |
| C++ toolchain and generated Rust↔Qt boundary maintained across two upstreams | Qt / QML | Medium |
| Text inputs invisible to assistive technology once HELM gains input | Slint | High |
| Accessibility performance collapse on a large App Library list | Slint | Medium–High |
| Single-vendor, embedded-first strategic focus | Slint | Medium |
| No accessibility at all; or maintaining a fork to get it | Iced | High |
| Blank windows or severe latency on some Linux graphics drivers | Tauri | High |

### 12.3 Cross-cutting risk

**Accessibility is the axis that separates this field**, and it is the one HELM cannot retrofit at
application level. Design freedom differences between the candidates are real but recoverable with
effort; an absent accessibility tree is not something HELM can build from above. That asymmetry
drives the recommendation more than any other single factor.

---

## 13. Primary recommendation

> Advice to the owner. **D9 is not accepted by this document.**

### **PRIMARY: GTK 4 + gtk-rs, with selective libadwaita (candidate B)**

**Why this fits HELM specifically.**

The reasoning turns on section 2.4. HELM's accepted design is *structurally conventional and
stylistically distinct*: a sidebar, a scrolling sheet, rows of labelled facts. It needs precise
control of colour, border, radius, spacing, type and one rotation — and it needs no canvas, no
shaders, no animation of consequence and no exotic layout. Every one of those visual demands is
directly expressible in GTK 4 CSS, as section 5.4 maps item by item.

Because the design does not need a rendering engine, **the toolkits that win on rendering power win
on a criterion this product does not exercise**. What remains decisive is everything HELM declared a
hard requirement: Wayland, AT-SPI, IME, portal-routed file dialogs, clipboard, high-DPI. GTK 4 is
strongest on every one, and it is the only candidate where accessibility is *default-on* rather than
assembled — which matters doubly, because the same AT-SPI tree that serves Orca also serves UI
automation, so HELM gets its testing story from the same investment.

Selective libadwaita closes bare GTK's one real weakness. Theme locking means a user's system theme
cannot silently repaint HELM and overwrite an accepted product decision. `AdwClamp` supplies the
measured column; `AdwDialog` supplies accessible modal presentation; CSS variables and media queries
carry HELM's tokens and give high-contrast support that HELM would otherwise hand-roll. The
take/refuse policy in 6.3 keeps the GNOME idiom out.

**What risk remains.** Two things, both named in 12.1 and both answerable by the spike rather than
by argument. First, layout leaves CSS and enters Rust, so the designer's authoring loop is less
direct than it is in the prototype or in QML — the design survives, the ergonomics do not. Second,
the tabular-figures question is genuinely open and it runs through every ledger row in the product.
Neither is a reason to decide differently in advance; both are reasons the spike exists.

### **SECOND CHOICE: Qt 6 / QML with CXX-Qt (candidate C)**

**What it does better.** Three things, clearly. QML's declarative tree keeps *layout and paint
together*, which is how the prototype is written and how a designer thinks — that is a real and
ongoing advantage, not a one-off. Typography is more precisely controllable, including the font
features GTK leaves in question. And `qmlscene` hot-reloading gives an iteration loop close to
editing CSS in a browser, where GTK offers inspection but not the same immediacy.

**Why the owner might legitimately choose it instead.** If designer velocity and long-run design
freedom are judged to outweigh accessibility maturity and build simplicity, Qt is the better
instrument. That is a defensible reading of HELM's priorities — the owner is a designer, the visual
identity is an accepted product decision, and a toolkit that fights the design every day imposes a
real compounding cost. Choosing Qt would mean accepting, deliberately, that HELM builds its
accessibility tree by hand across custom controls, carries a C++ toolchain, and takes on LGPLv3
obligations. Those are acceptable costs if the owner judges them worth paying; they are not hidden
ones.

### **DEFER**

- **Slint** — genuinely attractive on Rust-nativeness, footprint and visual freedom. Revisit when
  text-input accessibility ([#2895](https://github.com/slint-ui/slint/issues/2895)) lands, when the
  list-performance cost of the accessibility feature ([#3867](https://github.com/slint-ui/slint/issues/3867))
  is resolved, and when the AccessKit Unix adapter has settled. The licence choice and its
  attribution obligation would also need a deliberate owner decision.
- **Iced** — revisit at or after 1.0, and only once upstream accessibility exists
  ([#552](https://github.com/iced-rs/iced/issues/552)). Its state model is the best fit for HELM's
  discipline of any candidate here, which makes the accessibility gap genuinely regrettable rather
  than merely disqualifying.

### **REJECT**

- **Tauri / webview** — rejected for the production application on the technical grounds in 10.2:
  Linux rendering reliability, an accessibility story HELM cannot inspect or fix, a browser-engine
  review surface disproportionate to the interface, and architectural inversion of `helm-launch`'s
  minimal-authority premise. It remains what it is today: the medium of the design prototype.

---

## 14. Second choice — summary for the gate

| | Primary | Second |
|---|---|---|
| Candidate | GTK 4 + gtk-rs + selective libadwaita | Qt 6 / QML + CXX-Qt |
| Decisive strength | Best Linux integration; only default-on accessibility tree | Best design canvas; layout and paint stay together |
| Decisive weakness | Layout moves from CSS into Rust code | Accessibility tree is hand-built per custom control |
| Build | `cargo build` | Cargo + CMake + Qt SDK + C++ toolchain |
| Licence | LGPL, no obligations affecting HELM | LGPLv3 obligations, or commercial |
| Footprint | Moderate | Largest of the native candidates |
| If the owner values… | dependability, accessibility, simplicity | design velocity, typographic precision, declarative layout |

---

## 15. First implementation spike

**Not authorised by this document.** This scope exists so that, if the owner accepts D9, the first
step is already bounded. If the owner selects the second choice instead, the same scope applies with
the toolkit substituted — the question it answers does not change.

### 15.1 The question the spike answers

> **Can this toolkit faithfully implement the accepted HELM UI?**

Not: can it run the HELM backend. Not: is the architecture right. One question, answered by looking
at the result beside the canonical prototype.

### 15.2 In scope

1. Application and window shell, including the custom thin graphite title bar
2. The graphite rail: brand wordmark, four navigation items with the rose active indicator, and the
   bottom status rail
3. Library / session screen, including the ledger row and its state mark
4. Chosen program screen, with all five sections
5. Authority review, including the numbered grant and non-grant lists
6. Launch attempt state, including the elapsed meter — **and carrying no controls**, as accepted
7. Result, including the show/hide program-output disclosure
8. Launch details / Evidence, including the digest and the raw receipt bytes, selectable and
   copyable
9. HELM visual tokens as a stylesheet: burgundy, graphite, warm off-white, muted rose, and the three
   hairline weights
10. Normal / Advanced disclosure behaviour exactly as the canonical prototype accepts it

### 15.3 Explicitly out of scope

No backend connection. No link to `helm-launch` or any HELM crate — every value stays a constant, as
in the prototype. No persistence. No G-1, no G-2, no G-3. No Wine, no Proton. No custom shell or
compositor work. No packaging or distribution work. No real file chooser — the chooser stand-in may
remain drawn, since the spike tests visual fidelity, not portal integration.

### 15.4 Questions the spike must answer explicitly

These are the open items from sections 5 and 12, written as acceptance questions:

- Do tabular figures work in practice, and through which mechanism — libadwaita's `.numeric` style
  class, a direct Pango `font_features` attribute, or font choice? Record which was used.
- Can the 196 px / flexible ledger row be built so it reads identically to the prototype?
- Can the 1000 px clamped, centred column be achieved with `AdwClamp` without importing GNOME idiom?
- Can the custom thin title bar replace the stock header bar cleanly?
- Do the four state marks — outlined square, rotated diamond, outlined circle, filled square —
  render crisply at 9 px on fractional scaling?
- Do all three hairline weights hold at 1 px across scale factors?
- Did any screen require `AdwPreferencesGroup`, `AdwStatusPage` or a boxed-list class? *(If yes, the
  take/refuse policy of 6.3 needs revisiting.)*
- Is the accessibility tree populated and navigable by Orca, without per-widget hand-annotation?

### 15.5 How the spike is judged

Open the spike and the canonical prototype side by side, as the owner did at D8. The spike passes if
silhouette, rail proportion, sheet inset, content measure, hairline weights, typographic hierarchy,
ledger alignment and state marks read as the same design. Font substitution differences are expected
and acceptable, as they already are in the canonical prototype.

A failed spike is a useful result. It would mean the primary recommendation is wrong for this design
and the second choice should be tried against the same scope — which is why the scope is defined
independently of the toolkit.

---

## 16. Owner D9 decision gate

### 16.1 The decision

On 2026-09-21 the owner selected the primary recommendation of section 13:

> **GTK 4 + gtk-rs, with selective libadwaita**, with **Qt 6 / QML + CXX-Qt** retained as the second
> choice should the fidelity spike fail materially.

The owner additionally authorised **one bounded native UI fidelity spike** to the scope of
section 15.

### 16.2 What acceptance authorises, and what it does not

| Authorised | Not authorised |
|---|---|
| GTK 4 + gtk-rs as HELM's UI technology | Production GUI acceptance |
| Selective libadwaita, under the policy of 6.3 | Wholesale libadwaita idiom adoption |
| The UI dependencies needed for the spike | Any HELM crate dependency in the UI |
| One bounded fidelity spike, mock state only | Backend connection of any kind |
| — | Persistence, a database or a daemon |
| — | G-1, G-2 or G-3 |
| — | Wine, Proton, installer, update or recovery work |
| — | Custom shell or compositor work |
| — | Merging to `main` |

A completed spike connects no backend and accepts no GUI. It answers one question — *can this
toolkit faithfully reproduce the accepted D8 application?* — and hands the answer to the owner.

### 16.3 Current state of this gate

| | |
|---|---|
| G2-D9 | **ACCEPTED** 2026-09-21 |
| Toolkit selected | **GTK 4 + gtk-rs with selective libadwaita** |
| Second choice on material spike failure | Qt 6 / QML + CXX-Qt |
| Bounded fidelity spike | **AUTHORISED** |
| Production GUI | **NOT ACCEPTED** |
| Backend connection | **NOT AUTHORISED** |
| D8 prototype | unchanged and owner-approved; remains the visual source of truth |
| Next gate | **Owner visual review of the native GTK fidelity spike** |

### 16.4 Sources

Consulted 2026-09-21. Versions and dates are as reported by these sources on that date.

- [GTK 4 CSS overview](https://docs.gtk.org/gtk4/css-overview.html) and
  [GTK 4 CSS properties](https://docs.gtk.org/gtk4/css-properties.html) — supported properties,
  selectors, and the documented differences from web CSS
- [GTK 4 accessibility](https://docs.gtk.org/gtk4/section-accessibility.html) and
  [Evolving accessibility, GTK development blog](https://blog.gtk.org/2023/06/21/evolving-accessibility/)
  — the ATK-to-AT-SPI transition and `GtkAccessible`
- [gtk4-rs releases](https://github.com/gtk-rs/gtk4-rs/releases) and
  [gtk4 on crates.io](https://crates.io/crates/gtk4) — binding versions, cadence and MSRV
- [Relm4](https://relm4.org/) and [relm4 on crates.io](https://crates.io/crates/relm4) — the
  optional architectural layer over gtk4-rs
- [GTK release tags, GNOME GitLab](https://gitlab.gnome.org/GNOME/gtk/-/tags) and
  [libadwaita release tags](https://gitlab.gnome.org/GNOME/libadwaita/-/tags) — primary-source
  version and date verification at acceptance: GTK 4.24.0 (2026-09-11), libadwaita 1.10.0
  (2026-09-14)
- [GNOME 51 release notes](https://release.gnome.org/51/) — GNOME 51, 2026-09-16
- [libadwaita style classes — `.numeric`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/style-classes.html)
  — the tabular-figures style class, documented as equivalent to `PangoAttrFontFeatures` with
  `tnum=1`
- [libadwaita crate on crates.io](https://crates.io/crates/libadwaita) — binding version and the
  `gtk4 ^0.11` / `glib ^0.22` pairing
- [libadwaita 1.8 released ahead of GNOME 49, Phoronix](https://www.phoronix.com/news/libadwaita-1.8-Released)
  and [Libadwaita 1.8 arrives alongside GNOME 49, Linuxiac](https://linuxiac.com/libadwaita-1-8-arrives-alongside-gnome-49/)
  — CSS variables, media queries, typography classes, as introduced
- [Qt releases](https://doc.qt.io/qt-6/qt-releases.html) and
  [Qt licensing](https://doc.qt.io/qt-6/licensing.html) — current versions, LTS windows, licence
  model
- [CXX-Qt 0.7 release, KDAB](https://www.kdab.com/cxx-qt-0-7/) and
  [KDAB/cxx-qt](https://github.com/KDAB/cxx-qt) — bridge API stabilisation and 1.0 direction
- [Slint releases](https://github.com/slint-ui/slint/releases),
  [Slint licensing FAQ](https://github.com/slint-ui/slint/blob/master/FAQ.md) and
  [Slint Linux platform notes](https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/linux/general/)
  — current version, triple licence, Linux support statement
- [slint-ui/slint#2895](https://github.com/slint-ui/slint/issues/2895) — text input widgets not
  exposed to accessibility
- [slint-ui/slint#3867](https://github.com/slint-ui/slint/issues/3867) — accessibility performance
  on large lists
- [AccessKit issues](https://github.com/AccessKit/accesskit/issues) — AT-SPI adapter state as of
  2026-09
- [iced-rs/iced releases](https://github.com/iced-rs/iced/releases) and
  [iced-rs/iced](https://github.com/iced-rs/iced) — 0.14 contents and project self-description
- [iced-rs/iced#552](https://github.com/iced-rs/iced/issues/552) — accessibility support, open
- [Tauri releases](https://v2.tauri.app/release/tauri/) and
  [Tauri Linux graphics issues](https://v2.tauri.app/develop/debug/linux-graphics/) — WebKitGTK
  version, documented driver and performance problems
- [ashpd](https://github.com/bilelmoussaoui/ashpd) and
  [ashpd file chooser docs](https://docs.rs/ashpd/latest/ashpd/) — Rust XDG portal access
- [XDG Desktop Portal convenience libraries](https://flatpak.github.io/xdg-desktop-portal/docs/convenience-libraries.html)

**Where a source is a community report rather than primary documentation, it is treated as
indicative and not as a specification.** Every claim in this dossier that would change the
recommendation rests on primary documentation or on an issue in the project's own tracker.
