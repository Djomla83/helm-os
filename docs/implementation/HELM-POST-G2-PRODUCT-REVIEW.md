# HELM — POST-G2 PRODUCT REVIEW AND NEXT-CAPABILITY PLAN

> **Status: PROPOSED / REVIEW — docs only.**
>
> Date: 2026-09-22
>
> Repository baseline: `a4c6d719b13da84c854d7a203bc896f95bc0a4c7`
>
> This document records a post-acceptance engineering review of the first real G2 vertical and a
> proposed product sequence. It **authorises no implementation**, changes no accepted backend
> contract, and does not accept G-1, G-2, G-3 or a production GUI. Owner acceptance remains required
> before any new capability starts.

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

## 3. Post-G2 review findings requiring bounded verification

These findings come from code review of the accepted GUI adapter. They are **not** classified as
reproduced product defects yet. Both are small enough to verify before opening the next capability.

### 3.1 `PGR-01` — asynchronous result/context binding

**Status:** REVIEW FINDING — VERIFICATION REQUIRED

The GUI worker channel distinguishes executable admission, working-directory admission and launch
completion by message variant, but a message does not carry a request ID, generation ID, attempt ID
or immutable subject context. The main-thread handler applies a completed result to the session that
exists when the result is received. Executable display facts are then associated with current session
path/name state.

**Risk to verify:** if selection/session context changes while an earlier worker is still running, a
delayed response could be accepted into newer context. Reproduce delayed/reordered admission and a
long-running launch for subject A followed by selection of subject B before A returns.

**Required disposition before G-1 authorisation:**

- write a deterministic state-level regression that injects delayed/reordered worker results;
- verify whether current controls make the problematic sequence unreachable;
- if reachable, fix it in `helm-gui` by binding each asynchronous result to immutable operation
  identity/context and rejecting stale results;
- do not modify `helm-launch` for GUI convenience.

A temporary UI prohibition on replacing an active subject may reduce reachability, but long-term
integrity should not depend only on which button happens to be disabled.

### 3.2 `PGR-02` — caller-side open can precede admission by an unbounded wait

**Status:** REVIEW FINDING — VERIFICATION REQUIRED

The GUI adapter opens a selected local object with `File::open` before handing the owned descriptor
to `helm-launch` admission. Admission can reject a descriptor that is not a regular executable or
directory, but it cannot classify an object until caller-side open returns.

**Risk to verify:** a selected special filesystem object or a slow/problematic mount can leave the
worker waiting before admission is reached. The GTK main loop is separate, so this is not currently
a claim of a frozen window; the concern is an operation that never reaches a bounded HELM refusal.

**Required disposition before G-1 authorisation:**

- reproduce with a harmless local special-file fixture and a bounded harness;
- test object identity at the opened descriptor, not only a path checked before opening;
- if reproduced, harden caller-side opening/admission preparation in `helm-gui`;
- preserve the accepted descriptor-authority model;
- do not claim that a fix provides a universal timeout for arbitrary filesystem I/O.

### 3.3 What does not follow from these findings

Neither finding reopens the accepted first vertical. Neither justifies changing the file-dialog
architecture, changing the accepted `helm-launch` contract, adding a general orchestrator, or
starting G-2/G-3 inside a hardening patch.

## 4. Proposed immediate gate sequence

1. **G2 hardening verification:** reproduce or dismiss `PGR-01` and `PGR-02`.
2. **If reproduced, GUI-only correction:** smallest patch plus regression tests; accepted
   `helm-launch` remains read-only.
3. **Owner disposition:** close each finding with evidence.
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

### 5.3 Persistence and privacy rules

G-1 must not persist file descriptors, capabilities or `AuthorizedLaunch`; must not call an old
digest current before re-admission; must not move private host paths into exportable evidence merely
because a local store uses a locator; and must not store captured stdout/stderr as Library metadata
by default.

### 5.4 Storage properties before choosing a format

Define atomicity, interruption recovery, full-disk/permission failure, corrupt-record handling,
schema/version handling, concurrent-instance behaviour, migration/backup expectations, entry-count
bounds and startup-work bounds before selecting storage technology.

SQLite is a plausible candidate because it supplies mature transactional machinery; a small file
format can also be valid if crash/recovery semantics are explicit. HELM should not invent a custom
transaction journal without demonstrated need.

### 5.5 G-1 is not `helm-app-spec` integration by force

`helm-app-spec` 0.1 is a bounded desired-state model with Wine/win64-specific requirements. G-1
should not fabricate Wine fields merely to reuse that schema for a native local ELF Library entry.

### 5.6 Proposed G-1 acceptance tests

At minimum: add/restart persistence; missing locator; changed executable; wrong object kind;
re-admission refusal without history loss; interrupted persistence update; unknown-newer schema;
remove-from-Library without deleting program/documents; no persisted capability/authority; and no
stale worker result attaching to a newer Library entry/attempt. A serialization round trip alone is
not G-1 acceptance.

## 6. Cross-module support and cohort boundaries

| Component | Current bounded meaning relevant to product planning |
|---|---|
| `helm-gui` | Experimental native GTK G2 vertical; first-real path is Linux x86_64 and non-graphical subject execution |
| `helm-launch` 0.1 | Linux x86_64 execution authority/launch; no containment or desktop environment |
| `helm-observe` 0.1 | Linux x86_64 observation backend with empirically validated ext4 cohort; ext-family guard is not ext4 provenance |
| `helm-bind` 0.1 | Pure comparison; no compatibility/readiness verdict |
| `helm-evidence` 0.1 | Evidence-contract completeness, not application success/compatibility |
| `helm-app-spec` 0.1 | Wine/win64-specific desired-state semantics, not a universal Library record |

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

## 13. Proposed next owner decision

No new capability is authorised by this review.

The next proposed owner gate is:

> **Authorise a bounded post-G2 hardening verification for `PGR-01` and `PGR-02`.**

After both findings are dismissed with evidence or corrected and closed with regressions, the next
owner decision should be whether to authorise **G-1 DURABLE LIBRARY** under section 5. G-2 and G-3
remain later, separate decisions.
