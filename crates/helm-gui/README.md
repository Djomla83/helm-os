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
> Authorised by the owner on 2026-09-21 after the GTK fidelity spike was
> accepted, then **owner-accepted on 2026-09-22 and integrated into `main`**.
> That acceptance applies to this bounded first real vertical only. It does
> **not** accept a production GUI, persistence, G-1, G-2 or G-3.

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
| [`src/session.rs`](src/session.rs) | The session state machine, and the rule for which worker result may change what |
| [`src/state.rs`](src/state.rs) | Real `helm-launch` facts mapped to the words HELM may say about them |
| [`src/theme.rs`](src/theme.rs) | The HELM stylesheet |
| [`src/widgets.rs`](src/widgets.rs) | Shared builders carrying the prototype's measurements |
| [`src/screens.rs`](src/screens.rs) | The seven accepted surfaces |
| [`src/main.rs`](src/main.rs) | Dialogs, the main-loop tick, one render pass |
| [`tests/real_launch.rs`](tests/real_launch.rs) | A real launch to a real receipt, with no window |
| [`tests/worker_binding.rs`](tests/worker_binding.rs) | Worker results delivered out of order, against the real state machine |
| [`tests/bounded_admission.rs`](tests/bounded_admission.rs) | A special file must reach a HELM refusal, not wait forever |

`vertical.rs` is deliberately **not** a HELM orchestrator. There is no trait, no
service, no provider and no manager — nothing a future subsystem could be
tempted to implement. It does one flow and stops.

`session.rs` is the same state machine the window has always run. It lives in the
library rather than the binary so a test can drive it without opening a window,
and in particular so worker results can be delivered in orders a person cannot
reliably produce by hand. It is **not** a framework: one concrete session, the
transitions the seven surfaces actually offer, and the rule for applying a worker
result. Nothing in it draws anything.

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

**While an attempt is outstanding the open program cannot be replaced.** The
session refuses a new program or folder selection until the attempt returns, so
one attempt's facts cannot be drawn against another program's entry. Closing the
program is still available, and does what it says: the entry ends, and the result
of an attempt that returns afterwards is discarded rather than reported against
an entry that no longer exists. The launch itself is synchronous and has no
cancel — by design, since G-2 is unauthorised — so it runs to completion on its
worker; what closing changes is only what HELM will say about it.

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

## Post-G2 hardening: both findings reproduced and corrected

The [post-G2 product review](../../docs/implementation/HELM-POST-G2-PRODUCT-REVIEW.md)
raised two code-review findings about this crate. Both **reproduced**, and both were
corrected here — `helm-launch` was not touched. Section 3 of that document carries the
detail; this is what changed in the crate.

### `PGR-01` — asynchronous result/context binding

**CONFIRMED / FIXED / REGRESSION ADDED.** A worker message said what *kind* of result it
was and nothing about *which operation* produced it, so a result could change state that
belonged to a different operation. Three of the eight reproduced sequences are reachable
through the controls as drawn: closing the entry while work was outstanding was undone by
the late result; a launch result was reported as an ended attempt of a program selected
afterwards; and a second *Attempt launch again* was satisfied by the first generation.

[`tests/worker_binding.rs`](tests/worker_binding.rs) drives the real session state machine
with real capabilities and one real launch outcome, delivering results in orders a person
cannot reliably produce by hand. All eight failed before the correction.

The correction is in [`src/session.rs`](src/session.rs): a `vertical::OperationId` is
minted before each worker starts, travels with the request and returns with the result, and
only the operation currently allowed to change a piece of state may change it. Anything
else is dropped — a capability closing its descriptor as it goes, a `LaunchOutcome`
discarded as data and never relabelled. Display facts and `argv[0]` come from the path that
operation measured. An attempt owns immutable copies of the subject and plan it was
authorised from, and the Attempt, Result and Evidence surfaces draw from those, so a result
cannot be drawn under a subject that did not produce it. As defence in depth the open
program cannot be replaced while its attempt is outstanding — refused in the state machine,
not by a disabled button.

**The token is inert.** It names nothing, resolves nothing, authorises nothing and ends
with the process. It is **not** a session handle, a process handle or a durable identity;
G-1 and G-2 remain unauthorised.

### `PGR-02` — caller-side open before admission

**CONFIRMED / FIXED / REGRESSION ADDED.** The adapter opened the selected object before
`helm-launch` could classify it, and opening a FIFO with no writer read-only blocks in
`open(2)` until a writer appears — so the operation never reached a HELM refusal at all.

[`tests/bounded_admission.rs`](tests/bounded_admission.rs) drives the adapter on a worker
against a harmless local FIFO and waits five seconds. Both cases timed out before the
correction; the harness then opens the FIFO for writing, which releases the worker at once
and is the evidence for the mechanism.

The caller-side open is now `O_RDONLY | O_NONBLOCK`. The same FIFO reaches the accepted
refusal in **20.6 µs** as a program (`NotRegularFile`) and **16.1 µs** as a folder
(`NotDirectory`). Nothing stats a path and then opens it, so no path-check/path-open race
exists and the descriptor-authority model is exactly as accepted; only the flags of the
single open changed. `O_NONBLOCK` is neither part of the access mode nor `O_PATH`, so the
accepted `F_GETFL` gate still sees `O_RDONLY`, and two tests pin that a real ELF and a real
directory are still admitted.

**This does not make filesystem I/O bounded, and does not claim to.** Only the open, and
only where the open was the thing waiting. A network filesystem, a failing device or a
pathological mount can still hold an open or a read for as long as the kernel does. There
is no timeout here. It was never a claim about the window either: admission already ran on
a worker, so the window was not frozen — the operation simply never finished.

### What still does not follow

Both corrections are **owner-accepted since 2026-09-22** — `PGR-01` and `PGR-02` are
**OWNER-ACCEPTED RESOLVED** — after hosted CI and a real owner GUI smoke on the corrected
head; section 3.4 of the review records that smoke, what it did and did not exercise, and
every finding still carried. No production claim is promoted, no production GUI is
accepted, and **G-1 remains unauthorised**.

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
