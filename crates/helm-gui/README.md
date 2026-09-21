# helm-gui — EXPERIMENTAL G2 first real backend-connected vertical

> **NOT AN ACCEPTED HELM PRODUCT SURFACE AND NOT PRODUCTION GUI CODE.**
>
> The accepted G2 interface driving the accepted `helm-launch` 0.1 end to end:
> a real local file chosen through the desktop file dialog, real executable and
> working-directory admission, a real parsed plan, a real one-shot
> authorisation, a real launch on a worker thread, and a real receipt.
>
> **There is no mock launch result anywhere.** Everything the Result and
> Evidence surfaces show comes from a real `LaunchOutcome` or a real
> `LaunchError`.
>
> Authorised by the owner on 2026-09-21, after the GTK fidelity spike was
> accepted. The vertical is **authorised and not yet accepted**; the next gate
> is the owner's review of it.

## What it does not do

No persistence of any kind — no database, no file, no config, no library store.
No installer, updater, repair or rollback. No `helm-app-spec`, `helm-bind`,
`helm-observe` or `helm-evidence`. No runtime selection, no Wine, no Proton, no
MicroVM. No containment, no sandboxing, no custom shell. No graphical subject
support and no desktop-session environment. No cancel, no stop, no session
handle.

**G-1, G-2 and G-3 are unauthorised and untouched.** `helm-launch` is consumed
through its accepted public API and is not modified, extended or wrapped.

## Authority

| Source | Governs |
|---|---|
| [`docs/prototypes/g2-html/index.html`](../../docs/prototypes/g2-html/index.html) | Visual and interaction truth |
| [`docs/implementation/HELM-G2-VISUAL-KICKOFF.md`](../../docs/implementation/HELM-G2-VISUAL-KICKOFF.md) | Product semantics: what HELM may and may not claim |
| [`crates/helm-launch`](../helm-launch/README.md) | Every fact on the Result and Evidence surfaces |

## Running it

Linux x86_64, with GTK 4 and libadwaita. On Debian or Ubuntu:

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential pkg-config
cargo run --manifest-path crates/helm-gui/Cargo.toml
```

Choose any bounded, non-graphical local ELF program — `/usr/bin/uname` and
`/usr/bin/true` are good first subjects — and any ordinary local folder.

**It is not part of the product workspace.** `crates/helm-gui` is in the root
`Cargo.toml`'s `exclude` list and carries its own `Cargo.lock`, because it needs
GTK development packages that the accepted crates do not and that the runners
executing `cargo clippy --workspace` do not have. This is **recorded temporary
architecture debt**, not a hidden exception; workspace integration is a later G2
decision, and meanwhile
[`.github/workflows/helm-gui.yml`](../../.github/workflows/helm-gui.yml)
validates this crate on its own.

### `--g2-preselect`, a smoke-test affordance

```sh
cargo run --manifest-path crates/helm-gui/Cargo.toml -- \
  --g2-preselect /usr/bin/uname ~/some-folder
```

This supplies **exactly the two values the file dialog would have returned**,
and nothing else. Everything after it is the ordinary real path: the same
caller-side open, the same accepted admission, the same parsed plan, the same
authorisation, the same launch. **It is not a demo mode and injects no mock
value** — there is no result to fake, because results only ever come from a real
`LaunchOutcome`. It exists because a file dialog cannot be driven from a script.
Without the flag the interface behaves exactly as it does for any person.

## How the vertical is put together

| File | What it is |
|---|---|
| [`src/vertical.rs`](src/vertical.rs) | The orchestration adapter: open, admit, plan, authorise, launch, worker threads |
| [`src/state.rs`](src/state.rs) | Real `helm-launch` facts mapped to the words HELM may say about them |
| [`src/theme.rs`](src/theme.rs) | The HELM stylesheet |
| [`src/widgets.rs`](src/widgets.rs) | Shared builders carrying the prototype's measurements |
| [`src/screens.rs`](src/screens.rs) | The seven accepted surfaces |
| [`src/main.rs`](src/main.rs) | Session, dialogs, the main-loop tick, one render pass |
| [`tests/real_launch.rs`](tests/real_launch.rs) | A real launch to a real receipt, with no window |

`vertical.rs` is deliberately **not** a HELM orchestrator. There is no trait, no
service, no provider and no manager — nothing a future subsystem could be
tempted to implement. It does one flow and stops.

### Where authority lives

The interface knows the selected paths because a person chose them, and it
displays them. That is **not** path-based authority: `helm-launch` never
receives a path. The caller opens the object the person selected and moves the
owned descriptor in; the accepted admission functions then inspect and pin that
already-open object, and the descriptor is the only execution authority that
exists.

### The fixed policy

There is no plan editor. One visible policy, stated in full on the Authority
screen and read back off the **validated plan** rather than off the constants
that produced it:

| | |
|---|---|
| Execution kind | `linux_exact_executable` |
| `argv` | one element: the selected file's own name |
| Environment | `empty` |
| Working directory | the admitted capability, identified as `g2-workdir` |
| Input | `closed_pipe_eof` |
| Capture per stream | 16 384 bytes, well under the accepted maximum |
| Run deadline | 30 000 ms |
| Termination | `SIGTERM`, then 5 000 ms grace |

The document is built with `serde_json`, never by string concatenation, and then
goes through the accepted parser like any other untrusted input.

### Threading

`helm_launch::launch` is synchronous and stays that way. It runs on a worker
thread so the window stays responsive. That is a GUI adaptation and nothing
more: **it creates no session handle, no stop, no cancel and no live child
state**, and the Attempt screen carries no controls at all and says why. Only
inert values cross back to the main thread; no GTK object leaves it. No new
`unsafe` — the crate forbids it.

### Single use, and retry

An `AuthorizedLaunch` is composed only when a person presses *Authorise this one
launch*, and is consumed exactly once. **Nothing authority-bearing is cached or
replayed.** *Attempt launch again* returns through fresh preparation: the paths
are re-opened and re-admitted from scratch, and a new authorisation is required.

## Product semantics held

- **Session only.** Nothing is written anywhere. Closing the program drops the
  session and deletes nothing on disk; restarting starts from nothing.
- **Launch attempt, never running-confirmed.** *Launch attempt in progress*
  means only that HELM's own call has not returned.
- **No exec-success claim.** A clean status-channel end-of-file stays
  `indeterminate · status_eof_without_record`, and the Result screen says HELM
  did not establish that the program began running — even on an ordinary run
  that ends by exit reporting 0.
- **No verdicts.** An exit code is reported and never interpreted.
- **No containment.** `group_sweep = issued` is stated as best-effort cleanup
  that does not establish that any descendant received it.
- **No receipt authenticity.** Unsigned, fabricatable, no provenance, digest is
  byte identity only. No shield, lock, badge, tick or seal anywhere.
- **Pre-child refusal is not a result.** When `launch` returns `Err`, no child
  was created and no receipt exists; Evidence says exactly that rather than
  inventing one.
- **Captured output is disclosed honestly.** Non-UTF-8 bytes are announced as
  such and rendered lossily *with a notice*, never silently replaced.
  Truncation is stated when true. Output is held in memory for the session only
  and is part of no receipt.

`src/state.rs` carries unit tests that hold these rules still. They are not UI
tests: they are the guard that stops backend wiring from quietly acquiring a
verdict vocabulary.

## Tests

```sh
cargo test --manifest-path crates/helm-gui/Cargo.toml
```

`tests/real_launch.rs` builds a purpose-built ELF fixture as a cargo binary,
gives it an explicit asserted mode, admits it and a scratch working directory
through the accepted API, parses the fixed plan, authorises, calls the real
`helm_launch::launch`, and asserts on the real receipt — including recomputing
the receipt digest over the receipt's own exact bytes. It requires no display,
so hosted Linux CI runs it. It also asserts the refusal paths: a non-ELF file, a
file offered as a working directory, and a directory offered as an executable.

**It does not assert `ExecSucceeded`,** because no such value exists.

## Known limitations

- **Brand fonts are not selected.** Local fallback stacks; no font binary is
  vendored.
- **`G2-UI-A11Y-01` is open.** The interface pins the colour scheme so a host
  theme cannot repaint an accepted product decision. System high-contrast
  preference is **not** reconciled with that, and must be before production UI
  acceptance.
- **Local files only.** A `GFile` with no local path is refused, with copy that
  says so. No remote URI acquisition, and no file manager of HELM's own.
- **Non-graphical subjects only.** The subject environment is empty by the
  accepted plan, so a graphical application is not a supported subject here.
