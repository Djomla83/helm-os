# HELM — POST-G2 PRODUCT REVIEW AND NEXT-CAPABILITY PLAN

> **Status: PROPOSED / REVIEW.** The product sequence below is a proposal.
>
> Date: 2026-09-22. Section 3 updated the same day with the findings' dispositions.
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
> changed and what was not. Everything from section 5 onwards remains **PROPOSED and
> unauthorised**.

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

**Status: CONFIRMED / FIXED / REGRESSION ADDED.**

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

**Status: CONFIRMED / FIXED / REGRESSION ADDED.**

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

## 4. Immediate gate sequence

1. ~~**G2 hardening verification:** reproduce or dismiss `PGR-01` and `PGR-02`.~~ **Done** — both
   reproduced.
2. ~~**If reproduced, GUI-only correction:** smallest patch plus regression tests; accepted
   `helm-launch` remains read-only.~~ **Done** — both corrected in `helm-gui`; `helm-launch`
   unmodified.
3. **Owner disposition:** close each finding with evidence. The evidence is section 3 and the
   regressions it names; the closure itself is the owner's.
4. **Owner decision on G-1:** only then authorise or reject the durable-Library milestone.
5. **G-1 implementation:** persistence only; do not smuggle G-2 or G-3 into it.
6. **Later owner decisions:** G-2 long-lived session lifecycle and G-3 desktop-session authority
   remain separate architecture gates.
7. **First complete desktop workflow:** prove useful application work, not merely that a window
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
regressions committed before its correction, and section 3 records the evidence.

The next owner decision is therefore:

> **Whether to authorise G-1 DURABLE LIBRARY under section 5, without G-2 and without G-3.**

**G-1 is not authorised by this document.** Section 5 is a reviewed proposal and nothing more. G-2
long-lived session lifecycle and G-3 desktop-session environment authority remain later, separate
decisions, and no production GUI is accepted.
