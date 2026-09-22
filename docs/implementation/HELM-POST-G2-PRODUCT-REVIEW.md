# HELM — POST-G2 PRODUCT REVIEW AND NEXT-CAPABILITY PLAN

> **Status: PROPOSED / REVIEW.** The product sequence below is a proposal.
>
> Date: 2026-09-22. Section 3 updated the same day with the findings' dispositions, and section 3.4
> with the owner's acceptance of the hardening.
>
> Repository baseline: `a4c6d719b13da84c854d7a203bc896f95bc0a4c7`
>
> This document records a post-acceptance engineering review of the first real G2 vertical and a
> proposed product sequence. It **authorises no implementation**, changes no accepted backend
> contract, and does not accept G-1, G-2, G-3 or a production GUI. Owner acceptance remains required
> before any new capability starts.
>
> **Section 3 is no longer a proposal.** Both findings were reproduced and corrected in
> `helm-gui` under the bounded hardening gate; section 3 now records what was reproduced, what was
> changed and what was not. **Both corrections are owner-accepted since 2026-09-22** — `PGR-01` and
> `PGR-02` are **OWNER-ACCEPTED RESOLVED**, classification `HELM_POST_G2_HARDENING_ACCEPTED`
> (section 3.4). Everything from section 5 onwards remains **PROPOSED and unauthorised**.

---

## 1. Purpose and authority

The first real native GTK backend-connected G2 vertical is accepted and integrated. The next task is
not to reopen that acceptance or to rebuild its evidence chain. It is to decide what should be made
true next, while preserving the project's existing distinctions between facts, evidence, authority
and product claims.

This review has four goals:

1. identify reachable integrity or bounded-interaction risks in the current GUI adapter;
2. define a narrow proposed contract for a durable Library without leaking execution authority into
   persistence;
3. keep support/cohort boundaries explicit across the existing modules; and
4. convert the long-term HELM vision into product properties that can be tested through real user
   workflows.

The accepted G2 visual direction and toolkit remain unchanged. The accepted `helm-launch` 0.1
public API remains unchanged.

## 2. Verified repository state

At the baseline named above:

- `main` contains the accepted first real G2 vertical and the docs-only acceptance record;
- `implementation/g2-ui-spike` and `main` point to the same accepted head;
- `helm-launch` 0.1 is product-accepted;
- `helm-gui` consumes only the accepted `helm-launch` public API among HELM crates;
- `helm-gui` is intentionally outside the root Cargo workspace with its own lockfile;
- the GUI has no durable store, installer, updater, repair, rollback, Wine/Proton runtime,
  containment, session handle or desktop-session environment authority;
- G-1, G-2 and G-3 are not started and not authorised.

The first vertical acceptance remains bounded by the existing semantic rules: a clean status EOF is
`Indeterminate(StatusEofWithoutRecord)`, a child exit code is a child-end fact rather than an
exec-success fact, the process-group sweep is best-effort cleanup rather than containment, and a
receipt digest is byte identity rather than provenance or authenticity.

## 3. Post-G2 findings: reproduced and corrected

These findings began as code review of the accepted GUI adapter, classified **REVIEW FINDING —
VERIFICATION REQUIRED**. Both were then reproduced with deterministic tests that failed against the
accepted code, and both were corrected in `helm-gui` alone. The regressions that exposed each one
are committed before its correction, so the provenance is in the history rather than in this
paragraph.

### 3.1 `PGR-01` — asynchronous result/context binding

**Status: CONFIRMED / FIXED / REGRESSION ADDED — OWNER-ACCEPTED RESOLVED 2026-09-22**
(section 3.4).

The GUI worker channel distinguished executable admission, working-directory admission and launch
completion by message variant, but a message carried no request, generation or attempt identity and
no immutable subject context. The main thread applied a completed result to whatever session existed
when the result arrived, and rebuilt the executable's display facts from the session's *current*
path.

**What reproduced.** Eight state-level regressions in
[`worker_binding.rs`](../../crates/helm-gui/tests/worker_binding.rs) drive the real session state
machine with real `helm_launch` capabilities and one real launch outcome. All eight failed against
the accepted code. Three sequences are reachable through the controls as drawn:

- **Closing the entry was undone by a late result.** "Close program" is live on the Library row and
  the Chosen program screen while an admission or an attempt is running. Closing left the pending
  flags set and cancelled nothing, so a late admission or launch result restored an entry the person
  had closed — including the working-directory capability, whose descriptor then outlived the entry
  it belonged to.
- **A launch result was reported under a later subject.** The nav rail and "Choose local program"
  stayed live during an attempt, so a different program could be opened and admitted before the
  first attempt returned. The first attempt's result was then reported as an ended attempt of the
  second program: `phase Ended, subject "beta-subject"`.
- **A second "Attempt launch again" started a second generation** while the first was still running,
  and the earlier generation satisfied the later one.

Two further sequences — replacing a selection while its own admission is still running — are **not**
reachable through the Choose surface, which hides the re-choose control while an admission is in
flight. They are tested anyway, at state level, because reachability is a property of whichever
controls are drawn today and the integrity of a HELM fact must not be. In those orders the stale
result kept the current subject's name and path but replaced its measurement digest.

**What changed, in `helm-gui` only.** An opaque `OperationId` is minted before each worker starts,
moves with the request and returns with the result; the session records which operation may change
each piece of state, and a result from any other operation is dropped — a capability closing its
descriptor as it goes, a `LaunchOutcome` discarded as data and never relabelled. Executable display
facts and `argv[0]` are built from the path that operation measured. An attempt owns immutable
copies of the subject and plan it was authorised from, and the Attempt, Result and Evidence surfaces
draw from those. As defence in depth the open program cannot be replaced while its attempt is
outstanding, refused in the state machine rather than by a disabled control.

The token is inert: it names nothing, resolves nothing, authorises nothing and ends with the
process. It is **not** a session handle, a process handle or a durable identity.

One further mislabel was found while fixing and corrected with the rest: after "Attempt launch
again" the session reported the previous attempt's `Ended` while the re-admission was still running.
A re-admission in flight now precedes a previous result.

### 3.2 `PGR-02` — caller-side open can precede admission by an unbounded wait

**Status: CONFIRMED / FIXED / REGRESSION ADDED — OWNER-ACCEPTED RESOLVED 2026-09-22**
(section 3.4).

The GUI adapter opened a selected local object with `File::open` before handing the owned descriptor
to `helm-launch` admission. Admission refuses anything that is not a regular file or a directory,
but it cannot classify an object until the caller-side open returns.

**What reproduced.** Two regressions in
[`bounded_admission.rs`](../../crates/helm-gui/tests/bounded_admission.rs) drive the adapter on a
worker thread, exactly as the interface does, against a harmless local FIFO with no writer, and wait
five seconds. Both timed out against the accepted code. The harness then opens the FIFO for writing,
which releases the worker immediately — evidence for the mechanism rather than a guess at it: the
operation was parked in `open(2)` waiting for a writer, which a person selecting a filename has no
reason to provide.

**What changed, in `helm-gui` only.** The caller-side open is `O_RDONLY | O_NONBLOCK` instead of
plain `O_RDONLY`. The same FIFO now reaches the accepted refusal in **20.6 µs** as a program
(`NotRegularFile`) and **16.1 µs** as a folder (`NotDirectory`).

- **No path check was introduced.** The object is opened once and that same descriptor is moved into
  admission. Nothing stats a path and then opens it, so no path-check/path-open race exists and the
  descriptor-authority model is exactly as accepted. Only the flags of the single open changed.
- **The accepted mode gate still passes.** `O_NONBLOCK` is neither part of the access mode nor
  `O_PATH`, so `helm-launch`'s `F_GETFL` check still sees `O_RDONLY`. Two further tests pin that a
  real ELF and a real directory are still admitted through the caller-side open.
- **No duplicated policy.** The adapter still inspects no extension, no name and no magic number, and
  adds no ELF validation of its own. `helm-launch`'s refusal vocabulary is unchanged and unweakened.

**What this does not make bounded.** Only the open, and only where the open was the thing waiting.
`O_NONBLOCK` has no effect on reads of a regular file, and a network filesystem, a failing device or
a pathological mount can still hold an open or a read for as long as the kernel does. There is no
timeout in this adapter and none is claimed. This was never a claim about the GTK main loop either:
admission already ran on a worker, so the window was not frozen — the operation simply never
finished.

### 3.3 What does not follow from these findings

Neither finding reopened the accepted first vertical, and neither correction touched `helm-launch`:
no source, API, schema or semantic change. Neither justified changing the file-dialog architecture,
adding a general orchestrator, or starting G-1, G-2 or G-3 inside a hardening patch. The session
state machine moved from the binary into the library so it could be driven without a window; that is
a relocation of the existing state machine, not a new abstraction.

**No production GUI acceptance follows.** The vertical remains the accepted experimental vertical it
was, now with two defects corrected.

### 3.4 Owner acceptance of the hardening — 2026-09-22

**Classification: `HELM_POST_G2_HARDENING_ACCEPTED`.** The owner accepted both corrections on
2026-09-22, after independent engineering verification and a real manual GUI smoke on the exact
corrected head.

#### 3.4.1 Evidence chain of record

| # | Link | State |
|---|---|---|
| 1 | Deterministic reproduction against the accepted code, each regression committed before its fix | `PGR-01`: eight state-level regressions, all failing (`45c4a67`). `PGR-02`: two FIFO regressions, both timing out (`0786f64`) |
| 2 | Correction confined to `crates/helm-gui` | `PGR-01`: `7fc2a9f`, and `0e0bc36` from self-review. `PGR-02`: `141423c`. `helm-launch` unmodified |
| 3 | Full crate suite after correction | 33 tests pass: the ten regressions, the two admission pins of section 3.2, and the 21 previously accepted tests |
| 4 | First natural hosted CI at the corrected head `f4179cd90ce40e7b8a7d92842c1bd516c86129ff` | **PASSED** — `helm-gui G2 real vertical` run `35756196671`, run number 5, `push`, attempt 1 |
| 5 | Owner real-GUI smoke on that exact head, without `--g2-preselect` | **PASSED** for the properties exercised (section 3.4.2) |
| 6 | Owner acceptance | **ACCEPTED** 2026-09-22 |
| 7 | Integration into `main` and its first natural hosted CI | **PENDING** when this record was written |

Hosted job `106842459308` on the corrected head:

| Gate | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --all-targets --locked -- -D warnings` | **PASS** |
| `cargo test --locked` — 33 tests: 16 unit, 4 bounded-admission, 5 real-launch, 8 worker-binding | **PASS** |
| `cargo build --locked` | **PASS** |
| Real launch to a real receipt, without a display | **PASS** |
| The product workspace is untouched by the GUI | **PASS** |

The refactor `b503149`, which moved the session state machine from the binary into the library so it
could be driven without a window, is accepted with the corrections as part of this bounded hardening.

#### 3.4.2 Owner manual smoke

The owner ran the ordinary GUI on the exact corrected head
`f4179cd90ce40e7b8a7d92842c1bd516c86129ff`:

```bash
cargo run --manifest-path crates/helm-gui/Cargo.toml --locked --bin helm-gui
```

`--g2-preselect` was **not** used. Every program and folder was chosen through the real GTK
chooser.

**Smoke A — the normal real flow: PASS.**

The owner chose `/usr/bin/uname`, and as working directory a local folder in the owner's home
directory. As in section 18.1.1 of the [toolkit decision](HELM-G2-UI-TOOLKIT-DECISION.md), the host
path of the working folder is not recorded here; the receipt does not retain it. HELM displayed:

| Fact | Displayed |
|---|---|
| program | `uname` |
| path | `/usr/bin/uname` |
| kind | regular file |
| size | 35 336 bytes |
| mode | `0o755` |
| ELF | `ET_DYN` |
| measurement digest, short form as displayed | `sha256:c6e0…8e89` |

The owner went through Choose → Chosen program → Authority → *Authorise this one launch* → Attempt →
Result → Evidence.

Result: the direct child ended by exit with code 0, and HELM said explicitly that it does not
interpret that as proof of execution; the run deadline did not expire; no `SIGTERM` and no `SIGKILL`
were sent; the best-effort group sweep was issued; stdout 6 bytes, stderr 0 bytes; the displayed
output was `Linux`.

| Evidence field | Displayed |
|---|---|
| exec status | `indeterminate` / `status_eof_without_record` |
| child end | `exited` / code 0 |
| termination | `false` / `false` / `issued` |
| stdout | 6 bytes / `complete_at_eof` |
| stderr | 0 bytes / `complete_at_eof` |
| pre-exec measurement | 35 336 / `sha256:c6e0…8e89` / `0o755` / `et_dyn` |

The receipt was shown as inert unsigned data, with no proof of origin and no authority.

**Smoke B — a long-running attempt, and `PGR-01` as a person meets it: PASS for the properties
exercised.**

The owner chose `/usr/bin/yes` with the same working folder, authorised, and started a real
attempt. While that attempt was counting toward its 30-second deadline, the owner navigated to
Library and tried *Choose local program*. **The program chooser did not open while the attempt was
outstanding.** After the attempt completed, choosing a program was available again.

That is the owner-visible face of the running-attempt replacement guard of section 3.1, at the layer
a person can reach: the interface declines to open a chooser whose answer would have to be refused.
The state-machine refusal behind it, which holds even if a chooser answer did arrive, is what
`t4_a_launch_result_must_not_be_reported_under_a_later_subject` asserts.

The attempt ended through the accepted deadline lifecycle. From the receipt the owner recorded for
one `yes` attempt:

| Receipt field | Value |
|---|---|
| run deadline expired | `true` |
| child end | signal 15, `core_dumped: false` |
| `SIGTERM` sent | `true` |
| `SIGKILL` sent | `false` |
| group sweep | `issued` |
| exec status | `indeterminate` / `status_eof_without_record` |
| executable body size | 35208 |
| executable SHA-256 | `f7bee5c97e510592cc16f088ecfb6a2af84d61ad5821abfb2628b43850e9551e` |
| ELF type | `et_dyn` |
| mode bits | 493 |
| stderr | 0 bytes |
| stdout drained in that run | `1463033856` bytes |

Stdout was very large, as expected of `yes`, while the GUI's capture stayed bounded. A later Result
screen showed the same lifecycle class: the direct child observed to end after signal 15, the
deadline expired, HELM asked the program to stop, and no success verdict.

**What the smoke did not exercise.** The owner did **not** manually run the separate sequence of
closing the program while an attempt is outstanding and then waiting for its late result. That case
is covered by the deterministic regression
`t3_closing_the_entry_must_not_be_undone_by_a_late_launch_result` in
[`worker_binding.rs`](../../crates/helm-gui/tests/worker_binding.rs) and by the hosted CI above —
**not** by the manual smoke.

What the smoke adds, and nothing more: the refactored GUI still performs the accepted normal flow;
the real long-running attempt path works; the real interface does not let the open program be
replaced while its attempt is outstanding; the deadline and `SIGTERM` path still works; and no crash
occurred.

A separate read-only host check, not part of the owner smoke, on the development machine's default
WSL distribution (kernel `6.6.87.2-microsoft-standard-WSL2`, coreutils `9.4-3ubuntu6.3`) found
`/usr/bin/uname` at 35336 bytes, mode `0755`, SHA-256
`c6e023b172383fae5318c600e6f2f3a153513df62bf06ad6c9269296d2a78e89` — the full value recorded in
section 18.1.1 of the toolkit decision, consistent with the short form displayed in smoke A — and
`/usr/bin/yes` at 35208 bytes, mode `0755`, SHA-256 equal to the value in the owner-recorded
receipt. It describes those files at check time, not the objects measured during the smoke.

#### 3.4.3 Development-environment output during the smoke

The owner's terminal showed:

- `libEGL warning: failed to get driver name`;
- Mesa loader device-information warnings;
- `MESA: error: ZINK: failed to choose pdev`;
- `egl: failed to create dri2 screen`;
- one GTK critical: `GtkBox … reports a minimum width … Expect overlapping widgets`.

The first four are the libEGL/Mesa/Zink output already carried as a WSLg development-environment
observation (toolkit decision, section 17.1 note B). The GTK critical is carried with them; this is
its first written record in the repository. Its cause was not investigated here and it is not
attributed to WSLg or to anything else.

During this smoke there was **no** `free(): invalid pointer`, **no** `Aborted`, **no** Rust panic and
no crash the owner observed. That is one more run without a recurrence of `G2V-FD-01`, **not** a
closure of it. Nothing here establishes a root cause, and nothing here claims any of these warnings
is fixed.

#### 3.4.4 Owner decision

| Finding | Status | Basis |
|---|---|---|
| `PGR-01` | **OWNER-ACCEPTED RESOLVED** | Deterministic reproduction before the fix; correction confined to `helm-gui`; regression suite; natural hosted CI success; owner real-GUI smoke on the exact corrected head |
| `PGR-02` | **OWNER-ACCEPTED RESOLVED** | Deterministic FIFO reproduction before the fix; caller-side correction with no TOCTOU path pre-check; regression suite; natural hosted CI success; the normal owner GUI flow still works on the exact corrected head |

Overall: **`HELM_POST_G2_HARDENING_ACCEPTED`**.

This acceptance means that `PGR-01` and `PGR-02` are corrected and accepted, that the refactored
session state machine is accepted as part of this bounded hardening, and that once this branch is
integrated the current first real G2 vertical includes these corrections.

It does **not** mean a production GUI; G-1 implemented or authorised; G-2 or G-3 started; sandboxing
or containment; Wine or Proton; an installer, updater or recovery; general compatibility; receipt
authenticity; or exec success. `Indeterminate(StatusEofWithoutRecord)` is still not execution
success, and smoke A's exit code 0 does not change that.

#### 3.4.5 Findings carried, not closed

Only `PGR-01` and `PGR-02` are closed. Everything else stands as it was:

| Finding | Standing |
|---|---|
| `G2V-FD-01` — WSLg / GTK file-chooser abort observed once | **NONBLOCKING**, carried; did not recur in this smoke; not closed |
| `G2V-M1` — `--g2-preselect` developer helper | **NONBLOCKING**; due before production UI acceptance |
| `G2-UI-A11Y-01` — forced-light versus system accessibility preference | **NONBLOCKING**; due before production UI acceptance |
| `crates/helm-gui` outside the root workspace | **TEMPORARY ARCHITECTURE DEBT** |
| Brand fonts not selected | **OPEN DESIGN ITEM** |
| WSLg libEGL/Mesa/Zink warnings | **DEVELOPMENT-ENVIRONMENT OBSERVATION**; seen again; not fixed |
| GTK `GtkBox` minimum-width critical | **OBSERVATION**, carried; cause not established |
| Pre-existing WSL2 `helm-launch` local-test observation | Carried unchanged |
| Root `Cargo.toml` comment still calls `helm-gui` "the bounded G2-D9 UI fidelity spike" | **STALE COMMENT**, cosmetic; carried, not corrected here |
| *Attempt launch again* title behaviour | **COSMETIC**, carried; not re-examined by this documentation-only record |
| `Phase::Inspecting` note reads "the file you chose" while a folder is being inspected | **COSMETIC**, carried; still present at the corrected head |
| `helm-launch` 0.1 nonblocking MINOR and BACKLOG findings | Carried unchanged ([acceptance decision](../DECISIONS.md#helm-launch-0-1-product-accepted)) |

## 4. Immediate gate sequence

1. ~~**G2 hardening verification:** reproduce or dismiss `PGR-01` and `PGR-02`.~~ **Done** — both
   reproduced.
2. ~~**If reproduced, GUI-only correction:** smallest patch plus regression tests; accepted
   `helm-launch` remains read-only.~~ **Done** — both corrected in `helm-gui`; `helm-launch`
   unmodified.
3. ~~**Owner disposition:** close each finding with evidence.~~ **Done** — both
   **OWNER-ACCEPTED RESOLVED** on 2026-09-22 (section 3.4).
4. **Integration:** fast-forward `planning/post-g2-product-review` into `main` and observe the
   first natural hosted CI there. Pending when section 3.4 was written.
5. **Owner decision on G-1:** only after that CI result is known, authorise or reject the
   durable-Library milestone.
6. **G-1 implementation:** persistence only; do not smuggle G-2 or G-3 into it.
7. **Later owner decisions:** G-2 long-lived session lifecycle and G-3 desktop-session authority
   remain separate architecture gates.
8. **First complete desktop workflow:** prove useful application work, not merely that a window
   appeared.

## 5. Proposed G-1 contract — durable Library

**Status:** PROPOSED — NOT AUTHORISED

G-1 should answer one user-level question:

> Can HELM remember a program across its own restart, then safely re-open and re-admit the current
> object before a new launch attempt without turning historical metadata into present authority?

### 5.1 Required behaviour

A first G-1 slice should support:

- add a local program to the Library;
- assign a stable HELM Library entry identity;
- persist appropriate metadata only;
- show the entry after HELM restarts;
- re-open and re-admit before every new launch attempt;
- detect and clearly display a missing, inaccessible or changed executable;
- remove an entry from the Library without implying uninstall or deleting user documents;
- retain no descriptor, capability or authorisation across process restart.

### 5.2 Identity model

| Concept | Meaning |
|---|---|
| Library entry identity | Stable identity of HELM's record for the user's chosen program |
| Local locator | Where HELM may try to open the object again; data, never execution authority |
| Last measurement | Historical observation of bytes/mode/type from a prior admission |
| Attempt identity | Identity binding one plan/context/authorisation/result sequence |
| Runtime/profile revision | Future environment identity; not to be fabricated in G-1 |

A changed executable may remain the same Library entry while its current measurement differs from the
last observed measurement. The UI must say exactly that rather than silently deleting history or
calling the changed object trusted/untrusted.

**These five must not collapse into one value**, and the hardening gate produced direct evidence for
the last of them. `PGR-01` was exactly a case of two of these being conflated: the measurement of
one object was reported under the locator and name of another, because the result of an operation
was not bound to the operation that produced it. The `OperationId` introduced by the correction is
attempt-scoped by construction — minted per operation, compared once, discarded with the process —
and G-1 must keep it that way. An attempt identity that survived a restart would be a claim that a
past attempt still applies, which is the same mistake in durable form.

### 5.3 Persistence and privacy rules

G-1 must not persist file descriptors, capabilities or `AuthorizedLaunch`; must not call an old
digest current before re-admission; must not move private host paths into exportable evidence merely
because a local store uses a locator; and must not store captured stdout/stderr as Library metadata
by default.

The accepted types already make most of this structurally hard rather than merely forbidden, which
is worth stating so G-1 does not try to work around it: `ExecutableCapability`,
`WorkingDirectoryCapability` and `AuthorizedLaunch` own their descriptors, are not `Clone`, are not
`Sync`, have no public constructor and no serialisation, and `AuthorizedLaunch` is consumed exactly
once by `launch`. There is no supported way to write one to a store and no supported way to rebuild
one from data. G-1 should not add one, and should not add a parallel record that stands in for one.

### 5.4 Storage properties before choosing a format

Define atomicity, interruption recovery, full-disk/permission failure, corrupt-record handling,
schema/version handling, concurrent-instance behaviour, migration/backup expectations, entry-count
bounds and startup-work bounds before selecting storage technology.

Four further requirements are foundational rather than optional, and belong on that list before a
format is chosen:

- **Where the store lives, and its own access control.** A Library of local paths is a record of
  what the person runs. Its directory and file modes are part of the contract, not an
  implementation detail, and must be stated and asserted rather than inherited from the umask.
- **Per-field bounds, not only entry-count bounds.** A locator, a display name and a schema
  version are untrusted lengths on the way back in. Every field needs a bound, in the same spirit as
  the accepted crates bounding everything they parse.
- **Deterministic read order.** The Library list must not depend on filesystem or index iteration
  order, or two HELM instances will show the same store differently.
- **Clock handling.** If an entry records when it was added or last attempted, the time source and
  the behaviour when the clock moves backwards must be defined. A timestamp is in the same class as
  a digest: a historical observation, never a statement about now.

SQLite is a plausible candidate because it supplies mature transactional machinery; a small file
format can also be valid if crash/recovery semantics are explicit. HELM should not invent a custom
transaction journal without demonstrated need.

### 5.5 G-1 is not `helm-app-spec` integration by force

`helm-app-spec` 0.1 is a bounded desired-state model with Wine/win64-specific requirements. G-1
should not fabricate Wine fields merely to reuse that schema for a native local ELF Library entry.

Checked against the current schema and model, this is stronger than "should not": schema 0.1 is
closed, every listed field is mandatory except two, and four of its vocabularies have exactly one
variant each — `SourceArchitecture::X86_64`, `RuntimeFamily::Wine`, `WindowsArchitecture::Win64`
and `PrefixRole::Dedicated`. `runtime.artifacts` requires between one and sixteen requirements, each
with a real size and SHA-256, `verification.definitions` requires between one and eight, and
`entry_point.path` is a prefix-relative Windows C-drive spelling beginning `drive_c/`. There is no
extension map and no migration facility. A native local ELF entry therefore cannot be expressed in
it at all without inventing a Wine runtime, its artifact digests and a Windows entry point — which
would be fabricated evidence, not reuse. G-1 needs its own record.

### 5.6 Proposed G-1 acceptance tests

At minimum: add/restart persistence; missing locator; changed executable; wrong object kind;
re-admission refusal without history loss; interrupted persistence update; unknown-newer schema;
remove-from-Library without deleting program/documents; no persisted capability/authority; and no
stale worker result attaching to a newer Library entry/attempt. A serialization round trip alone is
not G-1 acceptance.

Add one the hardening gate made concrete: **a locator that now names a special file.** A stored path
is a name, and what it names can change to a FIFO, a device or a directory between one session and
the next. `PGR-02` showed what that costs when re-opening is not bounded, so G-1's re-open before
re-admission must reach the same bounded refusal, and the entry must survive it as an entry whose
current object HELM declined — not as a deleted record and not as a stuck one.

## 6. Cross-module support and cohort boundaries

| Component | Current bounded meaning relevant to product planning |
|---|---|
| `helm-gui` | Experimental native GTK G2 vertical; first-real path is Linux x86_64 and non-graphical subject execution |
| `helm-launch` 0.1 | Linux x86_64 execution authority/launch; no containment or desktop environment |
| `helm-observe` 0.1 | Linux x86_64 observation backend with empirically validated ext4 cohort; ext-family guard is not ext4 provenance |
| `helm-bind` 0.1 | Pure comparison; no compatibility/readiness verdict |
| `helm-evidence` 0.1 | Evidence-contract completeness, not application success/compatibility |
| `helm-app-spec` 0.1 | Wine/win64-specific desired-state semantics, not a universal Library record |

Each row above was checked against the crate's own current documentation and code during the
hardening gate, and none of them overstates. Two are worth spelling out because the short form could
be read too widely:

- **`helm-launch` 0.1** is Linux x86_64 for *execution authority and launch*. Its portable model —
  the plan parser and the value types — exists elsewhere; `launch()` and `LaunchOutcome` do not
  exist off the cohort at all.
- **`helm-observe` 0.1**'s superblock guard reads magic `0xEF53`, which the Linux UAPI gives ext2,
  ext3 and ext4 alike. Passing it means only that the descriptor is on a filesystem reporting
  ext-family magic. The validated cohort is ext4 by the OBS-FS-01 evidence, not by that guard.

A future base-filesystem or snapshot decision can affect validation scope. Support should eventually
be published as an explicit cohort matrix instead of being inferred from one module.

## 7. Keep four product questions separate

A future HELM UI should not compress these into one green badge:

1. **Observation:** what was actually observed?
2. **Evidence completeness:** is the declared evidence contract complete?
3. **Workflow outcome:** did the specifically defined user workflow meet its oracle?
4. **Applicability/freshness:** do subject/runtime/hardware/environment identities still match the
   situation in which the user wants to rely on the old result?

## 8. Recovery should be a first-class product contract

Keep executable/application version, runtime revision, profile/configuration, mutable application
state/database, user documents and system image separate. Rollback is not backup. Before a change,
HELM should eventually state exactly which layers can be restored and which data is outside that
rollback.

A high-value future feature is a **recovery rehearsal** against disposable test state before a real
change. This section authorises no update or repair implementation.

## 9. Proposed long-term differentiators

These are product hypotheses, not novelty or market claims.

### 9.1 Workflow passport

Replace a generic "compatible" badge with a scoped statement bound to a real workflow and environment.
Historical evidence remains historical truth when an environment changes; only its current
applicability changes.

### 9.2 Recovery plan with declared boundary

Before a change, show which layers are reversible and which user data is outside rollback. After a
failure, offer the smallest recovery action whose scope is already known and whose result can be
checked.

### 9.3 Evidence-bound AI diagnostics

The first useful AI role should be explanatory and read-only: distinguish observation from
hypothesis, cite local facts, identify relevant changes since the last known-good workflow and
propose the next bounded test. Logs/application text/model output never become execution authority.
Later executable AI actions must still pass through typed operations, policy, user review and an
observable result.

## 10. Product UX implications

The accepted Record / graphite-frame visual direction remains the source of truth. Normal disclosure
should eventually be the daily-use default; Burgundy remains identity/action rather than
success/safety/trust; working-directory choice must not imply sandboxing; result copy keeps
exec-status separate from child-end; accessibility/high-contrast/keyboard/scaling are due before
production GUI acceptance.

## 11. What a useful next product proof looks like

After G-1 and separate owner-approved G-2/G-3 work, prove a real desktop workflow: open a controlled
test document, perform a predefined edit, save, close, reopen, verify data integrity, repeat after one
controlled environment/runtime change, and demonstrate the declared recovery path if that change
fails. The exact application and oracle must be selected before execution; this document does not
authorise that experiment.

## 12. Documentation discipline

`docs/INDEX.md` remains the short current map. `docs/PROJECT_STATE.md` may add current snapshots
while preserving historical milestone records. Accepted dossiers remain historical evidence.
Crate READMEs describe current status and link to accepted decisions/current reviews. Proposals such
as this document remain visibly separate from accepted architecture.

## 13. Next owner decision

No new capability is authorised by this review.

The previous proposed gate — a bounded post-G2 hardening verification for `PGR-01` and `PGR-02` —
has been carried out. Both findings were reproduced and corrected in `helm-gui`, each with
regressions committed before its correction, and section 3 records the evidence. The owner accepted
both corrections on 2026-09-22 (section 3.4): **`HELM_POST_G2_HARDENING_ACCEPTED`**.

The next step is integration, not a decision: fast-forward this branch into `main` and observe the
first natural hosted CI there. Only once that result is known is the next owner decision:

> **Whether to authorise G-1 DURABLE LIBRARY under section 5, without G-2 and without G-3.**

**G-1 is not authorised by this document.** Section 5 is a reviewed proposal and nothing more. G-2
long-lived session lifecycle and G-3 desktop-session environment authority remain later, separate
decisions, and no production GUI is accepted.
