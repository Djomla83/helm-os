# HELM G2 — GTK IMPLEMENTATION SPIKE

> **Status: AUTHORISED, NOT IMPLEMENTED. BLOCKED on the build and runtime environment.**
> The owner authorised the bounded GTK spike on 2026-09-21 with an API floor of **GTK 4.18 +
> libadwaita 1.7**. **No environment available to this work can build or run at that floor**, and
> the batch was explicitly forbidden from installing host packages, adding repositories or lowering
> the floor. Exit criteria 1 to 6 are therefore **NOT_RUN**, no prototype source was written, and
> **GTK is not accepted, not rejected and not production-locked**. Written in English under
> [ADR-0020](../adr/ADR-0020-documentation-language.md).

---

## 1. Label vocabulary

Every statement in this document carries one of these, and no statement carries more than one.

| Label | Meaning |
|---|---|
| **FACT** | Observed directly, with the command and its output |
| **RESULT** | A verdict that follows from recorded FACTs |
| **NOT_RUN** | Could not be attempted; no verdict exists |
| **BLOCKED** | Attempted or attemptable, but prevented by a stated cause |
| **OWNER MANUAL GATE** | Reserved for the owner's own judgement; an agent may not self-award it |

**Nothing in this document is labelled PASS.** No exit criterion was exercised.

---

## 2. Owner authorisation

**FACT.** The owner authorised the spike on 2026-09-21. The decision is recorded in full, with the
owner's own wording, at [DECISIONS.md](../DECISIONS.md#g2-gtk-spike-authorised).

| Item | Value |
|---|---|
| Authorised work | **Only** the bounded spike of [toolkit decision](HELM-G2-UI-TOOLKIT-DECISION.md) sections 11.1 to 11.3 |
| Minimum API floor | **GTK 4.18** and **libadwaita 1.7** |
| Floor status | Implementation floor **for this spike only** — not a distribution support policy, not a packaging policy, not a minimum released image, not a production lock |
| Backend, `helm-launch`, persistence | **NOT AUTHORISED** |
| Production toolkit lock | **NOT GRANTED** — GTK remains a *provisional* production default |
| Host package installation | **NOT AUTHORISED** |
| Failure rule | An exit criterion must not be weakened; the response is to return to G2-D9 and its named Qt fallback |

---

## 3. The blocker

**RESULT: the spike cannot be built or run on any environment reachable from this work.**

The cause is not a defect in the spike, in GTK, or in the accepted floor. It is that no environment
here carries GTK 4.18 or libadwaita 1.7, and the one Linux environment present **cannot reach that
floor from its own archives at all**.

### 3.1 Host

**FACT.** The host is Windows.

| Probe | Output |
|---|---|
| `uname -a` | `MINGW64_NT-10.0-26200 ... x86_64 Msys` — Windows 11, Git Bash |
| `rustc --version` | `rustc 1.95.0 (59807616e 2026-04-14)` |
| `cargo --version` | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| `which pkg-config` | not found |
| `pkg-config --modversion gtk4` | `command not found` |
| `pkg-config --modversion libadwaita-1` | `command not found` |
| MSYS2 GTK tree (`/c/msys64`) | does not exist |
| vcpkg | not found |

**FACT.** A resolved dependency graph at the accepted floor fails to compile on this host at the
first system-library dependency:

```
cargo:warning=Could not run `PKG_CONFIG_PATH= PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1
  pkg-config --libs --cflags glib-2.0 'glib-2.0 >= 2.66'`
The pkg-config command could not be found.
```

The failure is in the `glib-sys` build script for target `x86_64-pc-windows-msvc`. It is reached
before any HELM code, so nothing about the spike's own source is being tested by it.

### 3.2 The only Linux environment present

**FACT.** Four WSL 2 distributions are registered: `Ubuntu` (default), `helm-lab-g0`,
`helm-lab-g0-staging`, `helm-lab-g0-proton`. `Ubuntu` and `helm-lab-g0` were probed read-only. No
distribution was started for any purpose other than inspection, and none was modified.

| Probe, in `Ubuntu` and in `helm-lab-g0` | Output |
|---|---|
| `pkg-config` | `MISSING` |
| `pkg-config --modversion gtk4` | `NOT_INSTALLED` |
| `pkg-config --modversion libadwaita-1` | `NOT_INSTALLED` |
| `cargo` | `MISSING` |

**FACT — and this is the decisive one.** `Ubuntu` is **Ubuntu 24.04.4 LTS**, and its archives do
not carry the accepted floor:

```
libgtk-4-dev:       Installed: (none)   Candidate: 4.14.5+ds-0ubuntu0.10
libadwaita-1-dev:   Installed: (none)   Candidate: 1.5.0-1ubuntu2
```

**RESULT.** **Authorising `apt install` would not unblock this spike.** The best version Ubuntu
24.04 can offer is **GTK 4.14.5 with libadwaita 1.5.0**, which is below the accepted floor of GTK
4.18 + libadwaita 1.7 on both libraries. Reaching the floor requires a *different* environment, not
a package installation in this one.

This is exactly the reach evidence already recorded in section 2.3.1 of the
[toolkit decision](HELM-G2-UI-TOOLKIT-DECISION.md), now confirmed on the actual machine.

### 3.3 What was deliberately not done

**FACT.** Per the authorisation and the batch instruction, this work did **not**: use `sudo`;
install any host or WSL package; add any external repository or PPA; change any desktop or session
configuration; or lower the accepted API floor to whatever happened to be installed. No unknown
file was created, deleted, cleaned or reset.

---

## 4. What was verified anyway

The dependency layer does not need the system libraries to be resolved, so the parts of the
authorisation that can be proven without them were proven.

### 4.1 Dependency resolution at the exact floor

**FACT.** Resolution was exercised in a scratch directory outside the repository, from this
manifest:

```toml
[dependencies]
gtk = { package = "gtk4", version = "=0.11.5", features = ["v4_18"] }
adw = { package = "libadwaita", version = "=0.9.2", features = ["v1_7"] }
```

| Command | Result |
|---|---|
| `cargo generate-lockfile` | **exit 0** — `Locking 67 packages to latest compatible versions` |
| Resolved `gtk4` / `gtk4-sys` | **0.11.5** |
| Resolved `libadwaita` / `libadwaita-sys` | **0.9.2** |
| `gtk4` minimum Rust version | **1.92**, satisfied by the local 1.95.0 |

### 4.2 API feature floor holds

**FACT.** `cargo tree -e features` on the resolved graph enables exactly the cumulative feature
chain up to the floor, and nothing above it:

| Crate | Features enabled | Features **absent** |
|---|---|---|
| `gtk4` | `v4_2`, `v4_4`, `v4_6`, `v4_8`, `v4_10`, `v4_12`, `v4_14`, `v4_16`, **`v4_18`** | `v4_20`, `v4_22`, `v4_24` |
| `libadwaita` | `v1_1`, `v1_2`, `v1_3`, `v1_4`, `v1_5`, `v1_6`, **`v1_7`** | `v1_8`, `v1_9`, `v1_10` |

**RESULT.** The floor is expressible exactly as the owner set it, and no higher API is enabled
silently.

### 4.3 Dependency boundary

**FACT.** The resolved lockfile contains **68** package entries and **zero** matches for
`name = "helm`. No accepted HELM product crate — `helm-evidence`, `helm-app-spec`, `helm-observe`,
`helm-bind`, `helm-launch` — appears anywhere in the graph, and no path dependency into `crates/`
exists.

**RESULT.** The dependency boundary required by the spike is achievable and was demonstrated. It is
not yet enforced by a committed manifest, because no manifest was committed — see section 5.

### 4.4 The accepted backend workspace is unaffected

**FACT.** `cargo test --workspace --locked` on the repository root: **exit 0**, **287 passed, 0
failed** across 33 test binaries and doctest sets.

**RESULT.** Nothing in this batch touched the accepted backend, and the workspace remains green.
**This says nothing about GTK.** It is a pure Rust and backend result and must not be read as
runtime validation of anything in the spike.

---

## 5. Why no prototype source was written

**RESULT — a deliberate decision, recorded rather than silently made.**

The batch instruction is explicit that when the required libraries are missing or below the floor,
the correct action is to *"report BLOCKED for local build/runtime and continue only with work that
can be proven without pretending execution occurred."*

Source for the spike cannot be proven here. It could not be compiled, formatted, linted, tested or
run, so committing it would place several hundred lines of unverified GTK code into a repository
whose own rules require a primary source or a test artifact behind every technical claim. At the
accepted floor specifically, the details that most need compiling to verify are exactly the ones the
exit criteria depend on:

* which accessibility announcement API exists at GTK 4.18, which criterion 3 turns on;
* `GtkColumnView` factory and list-model specifics at 4.18, which criterion 4 turns on;
* which libadwaita widgets exist at **1.7** — for example `AdwSidebar` arrived in **1.9** and is
  therefore *not* available at the floor, so the sidebar of kickoff 8.16.6 must be HELM-owned;
* which documented CSS variables and style classes are present at 1.7, which criteria 1 and 6 turn
  on, and which the bounded rule of toolkit decision 6.2.1 restricts to documented surface only.

Writing that blind would produce code needing a full rewrite once an environment exists, and would
bury the finding that actually needs an owner decision. The manifest and the resolution facts of
section 4 are recorded here as evidence instead, so the next batch starts from proven ground.

**No `prototypes/g2-gtk-spike/` directory was created.** The root workspace membership and the root
`Cargo.toml` and `Cargo.lock` are untouched.

---

## 6. Exit criteria

All six are **NOT_RUN**. None was exercised, so none has a verdict, and none may be reported as
passing or failing.

| # | Criterion | Status | Why |
|---|---|---|---|
| **1** | HELM palette and density recognisable; does not read as a stock GNOME application | **NOT_RUN** | No build, no run, no screenshot. Final verdict is in any case an **OWNER MANUAL GATE** |
| **2** | Every screen fully keyboard-operable; focus enters a disclosure and returns to its opener | **NOT_RUN** | No running application to exercise |
| **3** | Screen reader reads every control and status region; state change announced | **NOT_RUN** | No application, and no AT-SPI or screen-reader session available. A method call would not be evidence even if there were one |
| **4** | Advanced table of >= 5,000 synthetic rows scrolls without perceptible lag; values selectable and copyable exactly | **NOT_RUN** | No build; no factory instrumentation exists to record |
| **5** | Normal/Advanced switching does not lose the reader's place | **NOT_RUN** | No running application to exercise |
| **6** | Stylesheet cost recorded | **NOT_RUN** | No stylesheet was written, so byte size, line count, rule count and HELM-owned selector count are all **NOT_RUN**, not zero |

**Criterion 6 note.** A measurement of zero would be false. No stylesheet exists to measure, which
is a different statement, and the distinction matters because criterion 6 is evidence for the
owner's maintenance judgement.

---

## 7. Screenshots and visual evidence

**NOT_RUN.** No screenshot was captured, and none was fabricated.

---

## 8. Accessibility evidence limitations

**NOT_RUN**, and worth stating precisely for whoever runs this next.

Even in a working environment, criterion 3 cannot be satisfied by code inspection or by a unit
test. It requires a real AT-SPI or screen-reader observation of a running application. Calling a
documented announcement API is necessary and is not sufficient: a method that was called is not a
state change that was announced. When this spike is executed, criterion 3 must record the screen
reader used, the session type, and what was actually heard or read from the accessibility tree —
otherwise it stays NOT_RUN.

---

## 9. Virtualisation evidence limitations

**NOT_RUN.** When executed, criterion 4 must record the total synthetic row count, factory
setup and bind counts sufficient to show that widget creation is bounded relative to the row count,
the environment, and the responsiveness judgement kept **separate** from the instrumentation.
No performance threshold may be invented after seeing the numbers.

---

## 10. Known defects

**None recorded**, because no code exists to carry a defect. This is not a statement that the
design is sound.

---

## 11. What the owner needs to decide

**OWNER MANUAL GATE.** The spike is blocked on a decision only the owner can make: where it runs.

| Option | What it means | Cost and risk |
|---|---|---|
| **A. Provide a Linux environment already at or above the floor** | A machine, VM or WSL distribution running Ubuntu 25.04+, Debian forky or sid, Fedora, or another distribution shipping GTK >= 4.18 and libadwaita >= 1.7, with a graphical session and a screen reader available | Cleanest. The floor is met natively; criteria 1 to 6 all become executable. Requires provisioning |
| **B. Authorise a new WSL distribution at a suitable release** | Register a distribution that can reach the floor, then authorise installing its GTK and libadwaita development packages | Needs explicit host-mutation authority. Note that a WSL graphical session constrains criterion 3, since the screen-reader story under WSLg is not established |
| **C. Authorise a container or Flatpak SDK at the floor** | Build inside a toolbox, podman or Flatpak SDK image carrying GTK >= 4.18 | Unblocks build, lint and pure tests. Criteria 1 to 5 still need a real graphical session, so this is a partial unblock |
| **D. Lower the floor to GTK 4.14 + libadwaita 1.5** | Would let the existing Ubuntu 24.04 build | **Explicitly forbidden by the authorisation**, and recorded here only so the option is visibly rejected rather than quietly absent |

**Option D is not available to this work.** The owner set the floor and instructed that it must not
be silently lowered; changing it would be a new owner decision, not an implementation choice.

**FACT worth carrying into that decision.** Ubuntu 24.04 LTS is also the image used by this
repository's hosted CI. Any future decision to run the spike, or a later GUI, in CI inherits the
same gap: that runner cannot reach the accepted floor either.

---

## 12. Status after this batch

| Item | State |
|---|---|
| G2 GTK spike | **AUTHORISED, NOT IMPLEMENTED, BLOCKED on environment** |
| Exit criteria 1 to 6 | **NOT_RUN** |
| Prototype source | **NOT WRITTEN** |
| Toolkit dependency in the repository | **NONE** — root `Cargo.toml` and `Cargo.lock` untouched, workspace membership unchanged |
| GTK as production toolkit | **PROVISIONAL**, unchanged and **not locked** |
| Qt 6 / QML + CXX-Qt | **NAMED FALLBACK**, unchanged and **not implemented** |
| G2-D9 | **ACCEPTED**, unchanged — this batch reopens nothing |
| Distribution support policy | **OPEN**, unchanged |
| HELM licence policy | **Proposed**, unchanged |
| Backend, `helm-launch`, persistence | **untouched and unauthorised** |
| G-1, G-2, G-3 | **unopened** |
| Trial #4 | **NOT AUTHORISED** |

**This is not an exit-criterion failure.** No criterion was exercised, so the failure rule of the
authorisation — return to G2-D9 and the named Qt fallback — is **not** triggered, and Qt must not
be started. The spike is paused on an environment decision, not on a toolkit finding.

**Next gate: the owner's environment decision in section 11.** Once an environment at the floor
exists, the spike resumes at section 11.1 of the toolkit decision with nothing else changed.
