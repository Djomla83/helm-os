# helm-gui — EXPERIMENTAL G2-D9 UI FIDELITY SPIKE

> **THIS IS NOT A HELM PRODUCT SURFACE AND NOT PRODUCTION GUI CODE.**
>
> It exists to answer one question, and it is finished when the owner has
> answered it:
>
> > Can GTK 4 + gtk-rs with selective libadwaita faithfully reproduce the
> > accepted HELM G2-D8 application?
>
> **No backend. No HELM crate dependency. No persistence. No file access. No
> process execution.** Every path, byte count, digest, timestamp, exit code and
> receipt value on screen is a constant in [`src/state.rs`](src/state.rs).
>
> Authorised by the owner's G2-D9 acceptance on 2026-09-21
> ([decision](../../docs/implementation/HELM-G2-UI-TOOLKIT-DECISION.md)). That
> acceptance selects a toolkit and authorises this spike. It does **not** accept
> a production GUI, connect any backend, or start G-1, G-2 or G-3.

## Authority

| Source | Governs |
|---|---|
| [`docs/prototypes/g2-html/index.html`](../../docs/prototypes/g2-html/index.html) | Visual and interaction truth. Where this spike differs, **the prototype is right and this is wrong** |
| [`docs/implementation/HELM-G2-VISUAL-KICKOFF.md`](../../docs/implementation/HELM-G2-VISUAL-KICKOFF.md) | Product semantics: what HELM may and may not claim |

## Running it

Linux with GTK 4 and libadwaita. On Debian or Ubuntu:

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential pkg-config
cargo run --manifest-path crates/helm-gui/Cargo.toml
```

**It is not part of the product workspace.** `crates/helm-gui` is in the root
`Cargo.toml`'s `exclude` list and carries its own `Cargo.lock`, because it needs
GTK development packages that the accepted crates do not and that the runners
executing `cargo clippy --workspace` do not have. Excluding it leaves the
accepted workspace, its lockfile and its gates exactly as they were. Always
build it with `--manifest-path`.

It does not build on Windows or macOS without a GTK 4 development environment.

## What it is built from

| File | What it is |
|---|---|
| [`src/state.rs`](src/state.rs) | The mock state and every constant. **No GTK dependency**, unit-tested |
| [`src/theme.rs`](src/theme.rs) | The HELM stylesheet, translated from `styles.css` |
| [`src/widgets.rs`](src/widgets.rs) | Shared builders that carry the prototype's measurements |
| [`src/screens.rs`](src/screens.rs) | The seven accepted surfaces |
| [`src/main.rs`](src/main.rs) | Window shell, rail, status bar, and one render pass |

`state.rs` is deliberately separate and deliberately not an interface. There is
no trait, no client and no abstraction a backend could be slotted into, so mock
values cannot be mistaken for integration. Connecting real HELM state would
**replace** that file, not implement against it.

Its tests are not UI tests. They pin the product semantics — no running claim,
no verdict, no install claim, a single-use authorisation, an indeterminate exec
status, a group sweep that is only ever *issued* — in the one place where drift
would otherwise be silent.

## Selective libadwaita

`libadwaita` is used for infrastructure only. Its GNOME idiom is refused, per
section 6.3 of the decision.

| Used | Why |
|---|---|
| `AdwApplication` | Application and style infrastructure |
| `AdwStyleManager`, forced light | **Theme locking.** HELM's palette is an accepted product decision, so a host theme cannot repaint it |
| `AdwClamp` | The measured content column |
| `.numeric` style class | Tabular figures — `tnum=1`, which bare GTK CSS cannot express |

**Not used, and deliberately:** `AdwPreferencesPage`, `AdwPreferencesGroup`,
`AdwActionRow`, `AdwStatusPage`, boxed-list style classes, or any other GNOME
Settings composition. **No core HELM screen depends on preference-row
composition.** The ledger rows, the graphite rail, the state marks and the
evidence blocks are HELM's own composition over `GtkBox` and `GtkGrid`.

## What the spike answered

Verified by building and running it on GTK 4.14.5 with libadwaita 1.5.0, and
comparing rendered screens against the prototype.

| | Question | Answer |
|---|---|---|
| A | Graphite frame and warm off-white sheet | **Yes.** Background, radius and the asymmetric inset are direct CSS |
| B | HELM Burgundy controlled and distinct | **Yes.** `@define-color` tokens, application-priority provider, theme locked |
| C | Ledger and fact layout without custom rendering | **Yes.** `GtkBox` and `GtkGrid` with size requests; no drawing code anywhere |
| D | Tabular figures | **Yes**, through libadwaita's `.numeric`. Bare GTK CSS has no `font-variant-numeric` |
| E | Typography hierarchy | **Approximated.** The serif/sans pairing and the scale hold; the faces are fallbacks |
| F | Normal/Advanced disclosure without layout instability | **Yes.** Per-section blocks toggle `visible`; nothing reflows around them |
| G | Keyboard, focus, accessibility | **Yes.** Tab traversal works, the focus ring is clearly visible, every action is a real `GtkButton` |
| H | Did any core screen need `AdwPreferencesGroup`/`AdwActionRow`? | **No.** None is used anywhere |

### Things GTK made harder, recorded rather than smoothed over

- **CSS cannot do layout.** Every measurement moved from the stylesheet into
  Rust. The design survives; a structural change is now a code change.
- **A wrapping `GtkLabel` has no `max-width`.** Its natural width is the whole
  string, so the prototype's pixel measures become `max-width-chars` and have to
  be approximated from the font size. `CHARS_PER_PX` in `widgets.rs` is that
  approximation, calibrated against rendered output.
- **A size request is a minimum, not a width.** CSS `width: 210px; flex: none`
  lets text overflow a fixed box; GTK grows the box instead and pushes the next
  column. The Evidence key column needs explicit wrapping to stay aligned.
- **Margins sit outside a border; CSS padding sits inside.** Section spacing had
  to become real CSS padding to put the rules where the prototype puts them.
- **GTK 4.14 has no CSS custom properties** — those need 4.16 — so the tokens
  use `@define-color`.

None of these blocked the design. All of them are friction, and they are the
honest answer to "how much does this toolkit fight the accepted direction".

### Known differences from the prototype

1. **Fonts.** Neither Instrument Sans nor Newsreader is bundled; no font binary
   enters the repository and **no HELM brand font is selected here**. Local
   fallback stacks keep the serif/sans pairing, so text metrics differ.
2. **Window controls.** The prototype draws three inert dots as a stand-in for
   window controls. This uses real `GtkWindowControls`, placed at the start
   edge — the same silhouette, but genuine, accessible controls.
3. **File chooser.** The prototype draws a modal stand-in for the desktop file
   dialog. This spike does not build a fake file chooser: the Choose screen
   offers three labelled stand-in candidates inline, opens no dialog and reads
   no filesystem. A later, separately authorised backend-connected spike would
   use `GtkFileDialog`, which routes through the desktop portal.

## Boundaries

No backend connection · no HELM crate dependency · no persistence, database or
daemon · no G-1, G-2 or G-3 · no Wine or Proton · no installer, update or
recovery work · no shell or compositor work · no network · no filesystem access
· no process execution.

The next gate is the **owner's visual review of this spike**. It is not a
production GUI and accepting it would be a separate decision.
