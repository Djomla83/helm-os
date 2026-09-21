# NON-PRODUCT INTERACTION PROTOTYPE

HELM G2 — the first visible desktop application, as an **executable design reference**.

This directory is a throwaway HTML/CSS/vanilla-JavaScript prototype. It exists so that the owner can
open it, click through it, and judge layout, density, hierarchy, navigation, progressive disclosure,
copy and state differentiation. It is design evidence, nothing else.

## What this is not

- **No backend connection.** Nothing here runs a process, opens a file, reads the filesystem, or
  talks to `helm-launch` or any other crate.
- **No toolkit decision.** No G2 toolkit is selected. G2-D9 remains **pending**, and nothing in this
  directory selects, proposes, confirms or pre-empts a toolkit. Nothing here is built with, or
  implies, any candidate toolkit.
- **No implementation authority.** This prototype authorises no GUI code, no dependency, no
  `Cargo` change, no backend work and no branding decision.
- **Mock and session data only.** Every path, byte count, digest, exit code and receipt value is a
  fixed constant written into the source. Nothing is read from this computer.
- **Not production frontend code.** It is not structured, reviewed or tested as product code.
- **Not a decision to use web technology.** HTML was the fastest way to make an interaction
  reviewable. HELM is not a web application and this is not an argument that it should be.
- **Replaceable.** This prototype may be replaced or deleted once implementation work begins. It
  carries no guarantee of being kept in step with the product definition.

**The repository product definition is authoritative over everything here.** Where this prototype
and [HELM-G2-VISUAL-KICKOFF.md](../../implementation/HELM-G2-VISUAL-KICKOFF.md) disagree, the
kickoff document wins, and the prototype is wrong.

## Visual direction

**CANONICAL VISUAL DIRECTION: Record / graphite frame.** A graphite structural and navigation frame,
a warm off-white working sheet, a restrained HELM Burgundy accent, hairline rules and a strict
asymmetric grid. Typography and rules rather than floating cards; medium density; a structured
fact/ledger treatment for Authority, Result and Evidence; an append-only session record. This is
what `index.html` opens.

**SECONDARY REFERENCE: Instrument / restrained light sheet — retained, non-canonical.**
`alternates/instrument.html` is a sidebar-and-panel direction, kept only so the owner can compare
restraint, spacing and simplicity. It is labelled as non-canonical in the page itself. It is not a
live option and adopting it would need an owner decision.

Both use the working token set accepted at G2-D8 (kickoff 8.16.2):

| Token | Value | Role |
|---|---|---|
| HELM Burgundy | `#7A1F3D` | The single distinctive accent: identity, primary action, emphasis |
| Graphite | `#1F2937` | The structural and navigation frame |
| Warm Off-White | `#F8F6F4` | The working sheet |
| Muted Rose | `#D7A8B4` | Secondary accent; the accent where burgundy is unreadable on graphite |
| Cool Gray | `#CBD0D6` | Hairlines, separators, inactive structure |

This is a **working** token set, not a final brand identity. HELM Burgundy is identity and action
only: it does not mean *success*, *safe*, *verified*, *compatible*, *running* or *trusted*. **No
state is conveyed by colour alone** — every state carries its word from the closed vocabulary of
kickoff section 9 and a distinct glyph, with colour as reinforcement only.

## How to open it

Open `index.html` in any current browser. Double-click it, or:

```bash
xdg-open docs/prototypes/g2-html/index.html
```

```powershell
start docs\prototypes\g2-html\index.html
```

No build step, no npm, no `node_modules`, no server, no CDN, no web font, no network. It works
offline from `file://`. The only capability it asks of the browser is a digest function for the
receipt digest and a clipboard for the copy button; where either is unavailable it says so on the
surface rather than showing an invented value.

`screens.html` is a contact sheet showing all eight screens at once as live tiles, for reviewing
layout and density side by side. Hover a tile and choose **Open** to click through that screen at
full size.

## Screens

Each screen is reachable directly, as `index.html#<name>`. These hashes exist so the contact sheet
can show every screen; **they are not product navigation** — HELM has no URLs.

| # | Screen | Hash | Kickoff reference |
|---|---|---|---|
| 1 | Session home — nothing open | `#home` | 8.1, 8.2, 5.2.1 |
| 2 | Session home — one program, this session | `#session` | 8.1, 8.4, 5.2.1 |
| 3 | Choose local program | `#choose` | 8.3, 8.5, 10.3 |
| 4 | Authority review | `#authority` | 8.6, 11 |
| 5 | Launch available | `#authorised` | 8.5, 11.5 |
| 6 | Launch attempt in progress | `#attempt` | 8.7, 12.8 |
| 7 | Result — ended, reported 0 | `#result` | 8.8, 12 |
| 8 | Evidence — the receipt | `#evidence` | 8.9, 13 |

## Interactions that work

Session home → **Choose local program** → inspect the program → inspect the working folder →
**Open for this session** → **Review authority** → **Authorise this attempt** → **Attempt launch** →
**Launch attempt in progress** → **Result** → **Evidence** → **Close program** → back to an empty
session home.

Also clickable: the simulated desktop file chooser; three admission refusals
(`SET_ID_BITS_PRESENT`, `NOT_ELF`, `NOT_A_DIRECTORY`) and recovery from each; cancelling the choose
step; discarding an authorisation; attempting again, which re-authorises because an authorisation is
single-use; per-section technical disclosure; copying the exact receipt bytes; and closing the
program behind a confirmation that names the target and states that nothing on disk is deleted.

## Product constraints this prototype holds to

- **Session only.** Choosing a program installs nothing and adds nothing. The session home states on
  its surface that the entry lives for the session and that nothing was written to disk. There is no
  durable library, no *Install* and no *Add to HELM*.
- **No live *Running* claim.** *Launch attempt in progress* only, with the explicit note that HELM's
  call not returning is a fact about HELM's own call and not confirmation that the program started.
- **No functional cancel.** The attempt screen states that HELM cannot stop an attempt it started,
  and why.
- **Facts, not verdicts.** No PASS, FAIL, SUCCESS, COMPATIBLE, WORKS or SAFE. Exit code `0` is
  reported as the number the program reported, with the explicit line that HELM does not interpret
  it and is not the authority on what it means.
- **Indeterminacy is never collapsed into failure.** *Exec status indeterminate* sits beside the
  ordinary-looking end, with the statement that neither overrides the other.
- **No security language and no security theatre.** *Safe*, *sandboxed*, *contained*, *isolated*,
  *protected*, *secure* and *trusted* appear only inside explicit statements of what HELM does
  **not** do. The non-containment statement is prominent on Authority review.
  `group_sweep = issued` is disclosed with the sentence that it is not containment.
- **The receipt claims no authenticity.** No shield, lock, tick, badge or seal anywhere. The receipt
  is introduced as data that is not signed, carries no proof of origin, and that anyone can forge;
  a digest identifies bytes and says nothing about where they came from.
- **Honest, contextual navigation.** Before a program is chosen, only *Choose* is reachable; the
  remaining steps are disabled because nothing is chosen, not because they are future features.
  Updates, recovery, installer and runtime management are **not** shown as disabled controls.
- **Progressive disclosure is local, not global.** There is no Normal/Advanced mode switch. Each
  section offers its own *Technical detail*, exactly one layer deep.
- **No process identifiers, process descriptors or descriptor numbers** anywhere.

## Accessibility

Real `<button>` elements throughout, with no `div`/`span` click targets. Keyboard operable end to
end, with a visible focus ring — recoloured to Muted Rose on the graphite frame, where burgundy is
unreadable. Opening a disclosure moves focus into the panel and closing it returns focus to the
control that opened it. Semantic headings, a polite live region announcing state changes and
results, and `aria-expanded` / `aria-controls` / `aria-pressed` where they are needed. Every state
carries a word as well as a glyph and a colour. This is prototype-grade, not an accessibility
conformance claim.

## Known prototype-only fictions

- The file chooser is a stand-in, drawn in-page. The real flow uses the desktop file chooser. It is
  not a decision to build an in-application file manager.
- All paths, sizes, digests and receipt bytes are mock values. The stdout and stderr digests happen
  to be real SHA-256 values of sample byte strings; the plan and executable digests are invented.
  The receipt digest shown on the Evidence screen is genuinely computed in the page from exactly the
  bytes displayed above it.
- The attempt takes a fixed ~2.2 s and then produces one fixed result. There is no process.
- `pre_exec_mode_bits` is rendered as `493` in the receipt bytes and `0o755` in labelled tables. The
  exact wire encoding is a receipt-schema question, not a design question.
- Timings, the 30 s run bound and the 5 s grace period are illustrative plan values.

## Known gap, for owner attention

The prototype shows **one** result: *ended, reported 0*, with exec status indeterminate. It does not
carry a prototype-only state picker for the other accepted result states — *ended, reported
non-zero*; *run deadline expired* with termination facts; *signalled*; *end unobservable*; pre-exec
failure; and pre-child `LaunchError`. The three admission refusals are demonstrated, and the two
hardest semantic cases — indeterminate execution status and non-containment after a group sweep —
are shown by default. Adding the remaining result states would be new prototype work and was not
part of the integration; it is an owner decision, not an omission to fix silently.

## Curation note

This directory was curated from a designer delivery, not copied from it. Designer tool and export
artefacts, a print export, pasted screenshots, a stale copy of the kickoff document and three
superseded exploratory colour directions were all excluded. The kickoff document in this repository
is authoritative and was not overwritten.
