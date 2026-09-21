# NON-PRODUCT INTERACTION PROTOTYPE

This directory holds the **owner-approved G2-D8 visual and interaction prototype** for HELM G2, the
first visible desktop application. It is an executable design reference and nothing else.

It was normalised from the owner-approved designer final delivery. **An earlier prototype in this
directory was superseded and removed**; it was built from an older designer variant and is not a
selection candidate. There is one approved design here and no alternates.

Open it by double-clicking [index.html](index.html), or from `file://`. No server, no command, no
build step.

## What this is not

- **No backend connection.** Nothing here runs a process, opens a file, reads the filesystem, or
  talks to `helm-launch` or any other crate.
- **No filesystem or process authority.** The file chooser is a drawn stand-in. Choosing a file
  changes a variable in `prototype.js` and nothing else.
- **No persistence.** Nothing is written to disk, to storage or to a cookie. Reloading the page
  returns it to its starting state.
- **No toolkit decision.** No G2 toolkit is selected. **G2-D9 remains pending**, and nothing in this
  directory selects, proposes, confirms or pre-empts a toolkit. Nothing here is built with, or
  implies, any candidate toolkit.
- **No GUI implementation authority.** This prototype authorises no GUI code, no dependency, no
  `Cargo` change, no backend work and no branding decision.
- **Not a decision to use web technology.** HTML, CSS and vanilla JavaScript are a design-review
  medium — the fastest way to make an interaction reviewable. HELM is not a web application and this
  is not an argument that it should be.
- **Mock data only.** Every path, byte count, digest, exit code, timestamp and receipt value is a
  fixed constant written into `prototype.js`. Nothing is read from this computer.
- **Not production frontend code.** It is not structured, reviewed or tested as product code, and it
  may be replaced or deleted once implementation work begins.

**The repository product definition is authoritative over everything here.** Where this prototype
and [HELM-G2-VISUAL-KICKOFF.md](../../implementation/HELM-G2-VISUAL-KICKOFF.md) disagree, the kickoff
document wins and the prototype is wrong.

## Visual direction

The approved direction is **Record / graphite frame** (kickoff 8.16): a graphite structural and
navigation frame, a warm off-white working sheet, a single restrained HELM Burgundy accent,
typography and hairline rules rather than floating cards, medium information density, and a
structured fact/ledger treatment for Authority, Result and Evidence.

| Token | Value | Role |
|---|---|---|
| HELM Burgundy | `#7A1F3D` | Identity, primary action, emphasis |
| Graphite | `#1F2937` | Structural and navigation frame |
| Warm Off-White | `#F8F6F4` | The working sheet |
| Muted Rose | `#D7A8B4` | Secondary accent on graphite |

The designer final refines hairline usage on the warm sheet to `#DED9D3` and `#E6E1DB` rather than
the working-token Cool Gray `#CBD0D6`; the token set is a working set and may be refined without
re-opening G2-D8.

**Colour carries no claim.** Burgundy means identity and action only. Every state carries its word
and a distinct non-colour shape — an outlined square, an outlined diamond, an outlined circle, a
filled square — so no state is conveyed by colour alone. There is no shield, lock, badge, tick or
seal anywhere, and in particular none on the receipt digest.

## Screens and states

Seven screens, plus one overlay. This list is generated from the approved final; it is descriptive
only.

| Screen | Purpose | Main action | Prototype interaction |
|---|---|---|---|
| **Library** | What HELM has open for this session, and the way to choose another program | Choose local program | Ledger row for the session entry with its state word and shape; Open program, Review authority, Close program |
| **Choose local program** | Admit a program file and a working folder | Choose a program file… | Opens the chooser overlay; renders admitted measurement, or a refusal; Continue appears once both are admitted |
| *Desktop file chooser (overlay)* | Stand-in for the desktop file dialog | Pick a file | Three fixed entries producing one admission and two refusals (`not_elf`, `set_id_bits_present`); Escape or Cancel closes it |
| **Chosen program** | The session entry: Overview, Runtime, Result, Updates, Recovery | Review what this will be permitted to use → Attempt launch | Recovery offers only what HELM can actually do; no repair, rollback, reset or re-prepare |
| **Authority review** | The single-use authorisation: what HELM will permit, and what it does not do | Authorise this one launch | Eight numbered ledger rows, a plain-language non-containment statement, and a technical-detail grid under Advanced disclosure |
| **Launch attempt in progress** | HELM's launch call has not returned | *(none)* | Elapsed against the bound, a meter, the deadline sequence, and an explicit *No cancel* statement. **The screen carries no controls at all.** |
| **Result** | Ended, reported 0 | Attempt launch again | Five numbered qualifying facts; Show/Hide what was printed |
| **Launch details** | What exactly was recorded | Copy the exact receipt bytes | Eight receipt-fact rows, the digest, and the exact bytes. This is the whole disclosure; there is no further level |

The **Advanced disclosure / Normal disclosure** item in the status bar is a button. It switches the
per-section technical detail on and off.

## Product semantics this prototype holds to

- **Session only, never installation.** Choosing is not installing and not adding. The copy is
  *Choose local program* / *Open local program*, and the surface says the selection lasts for this
  session and writes nothing to disk. There is no durable application library.
- **Launch attempt, never running-confirmed.** *Launch attempt in progress* is a fact about HELM's
  own call. The prototype never claims the program is running, started successfully or launched
  successfully.
- **No cancel and no stop.** `launch` is synchronous and returns no handle, so there is nothing to
  cancel and nothing to observe live. The attempt screen offers no control, functional or otherwise,
  and says why.
- **No verdicts.** HELM does not issue PASS, FAIL, SUCCESS, WORKS, COMPATIBLE, SAFE, TRUSTED or
  VERIFIED. Exit code 0 is a fact reported by the program; HELM does not interpret it.
- **No exec-success claim.** Clean exec-status end-of-file stays `indeterminate ·
  status_eof_without_record`, and the Result screen states plainly that HELM did not establish that
  the program began running.
- **No containment.** The process-group sweep is best-effort cleanup. The prototype says so, and
  says that a descendant that left the group survives it.
- **No receipt authenticity.** The receipt is unsigned data with no proof of origin and no execution
  authority; anyone can write bytes that look like it, and its digest identifies exactly those bytes
  and nothing more.
- **No graphical-subject claim.** The subject environment is empty — not your home folder, not your
  display, not your language — and there is no runtime selection: Wine, Proton, a PWA runtime and a
  MicroVM do not exist in HELM and are not offered.

## How it runs

Plain HTML, CSS and vanilla JavaScript, in three files plus this README:

| File | What it is |
|---|---|
| [index.html](index.html) | All seven screens and the chooser overlay, as static markup |
| [styles.css](styles.css) | The approved visual system, as tokens and classes |
| [prototype.js](prototype.js) | A small state machine and the mock constants |

No npm, no `package.json`, no `node_modules`, no bundler, no framework, no build system, no backend,
no server, no CDN, no web font, no network request of any kind. It works offline from `file://`.

The one browser capability it asks for is the clipboard, on the Evidence screen. Where the clipboard
is unavailable — which it may be under `file://` — the button says so and invites selecting the
bytes, rather than reporting a copy that did not happen.

Actions are real `<button>` elements, reachable by keyboard, with a visible focus ring; Escape closes
the chooser. Toolkit-level accessibility is a D9 and implementation concern and is not settled here.

## Known fidelity differences from the designer final

The designer delivery is authoritative for visual design, layout and interaction presentation. These
are the deliberate departures, recorded rather than hidden.

1. **Fonts.** The designer final loads *Newsreader* and *Instrument Sans* from Google Fonts. Remote
   fonts are not used here and no font binary is vendored into the repository; both are replaced by
   local fallback stacks that keep the serif/sans pairing. The serif metrics differ, so text-heavy
   screens sit a few pixels higher and the Authority pull-quote wraps to three lines rather than
   two. **No production font is selected by this task.**
2. **Disclosure switch.** The designer final exposed Normal/Advanced as a control in the designer
   tool's property panel, which does not exist outside that tool. It is normalised onto the existing
   status-bar disclosure item, which is now a button. **Note for the owner:** kickoff 8.16.6 expects
   disclosure to be *local rather than global*. The technical detail is attached per section, one
   layer deep, as 8.16.6 asks — but the switch itself is application-wide, as the approved final
   built it. This is recorded, not resolved.
3. **Run deadline.** The designer final exposed the deadline as an adjustable property. It is fixed
   here at its default of 30 seconds, matching the copy on the Authority and Attempt screens.
4. **Copy label.** The designer final showed a fixed *Copied 512 exact bytes*. Here the copy is
   genuinely attempted and the label reports the actual byte count of the displayed receipt bytes.
5. **Link-styled actions.** Secondary actions the designer drew as `<a href="#">` are `<button>`
   elements styled identically, so that every action is a real button.

Designer runtime machinery — the `support.js` runtime, the `x-dc` component wrapper, `sc-if`
elements, `style-hover` attributes and the generated property panel — is **not** carried into this
repository. Visual and interaction fidelity was the objective; the designer tool's architecture is
not HELM's.
