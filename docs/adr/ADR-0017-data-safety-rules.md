# ADR-0017: Recovery is tiered, quiesced and reflink-based, and never reverts user documents

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

HELM's recovery promise is that a failed update must never lose user work. The audit found that no
Linux system-level mechanism provides this today, and that the obvious implementations are actively
destructive for Wine prefixes.

- Flatpak, OSTree and bootc all revert the image and **deliberately** leave user data alone. Snapper
  excludes `/home` by default and documents that there is no mechanism to ensure data consistency
  when creating a snapshot. The ZFS desktop automation layer has been unmaintained since 2024.
- Only two systems do per-application user-data rollback in production, and both do it from inside
  the application vendor with full source knowledge — and both ship incomplete manifests. One of them
  explicitly does **not** revert its shared data tier. There is therefore **no shipped precedent** for
  atomically rolling back an application together with all of its user data.
- **Wine's registry save takes a non-atomic in-place truncate path whenever the target file is a
  symlink or has a link count above one.** Any hardlink-based snapshot scheme therefore causes the
  next registry flush to corrupt the live prefix *and* the snapshot together. Reflinks do not raise
  the link count and are safe.
- Wine defers registry saves for up to thirty seconds and never flushes them with `fsync`, so a
  snapshot of a running prefix can be missing the last half-minute of writes.
- Windows share modes are enforced only inside `wineserver`'s per-inode table and are invisible to
  every native Linux process. Byte-range locks degrade to advisory locks that Wine silently abandons
  on filesystems that reject them. HELM's own integration features — indexers, backup, sync — would
  therefore be unconstrained by the application's locking.
- The verified real-world total-loss incidents in this domain are **not** schema-compatibility
  failures. They are storage exhaustion corrupting an embedded database, and a launcher script with
  an empty path variable in a recursive delete removing a user's home directory along with a mounted
  backup drive. The second is exactly the class of software this project proposes to build.

## Options considered

1. **Whole-prefix snapshot and restore.** What every incumbent does, because nobody built the
   provenance to do better. Rejected as the *promise*: for a business application the prefix contains
   the user's data, so a whole-prefix restore can silently discard weeks of work — a worse outcome
   than the broken application it was meant to fix.
2. **Image-only rollback, leaving data to carry forward.** Rejected: this is precisely the state that
   breaks applications. Several well-known applications refuse to start on downgraded binaries with
   newer data, and an unknown fraction of Windows applications will write into a schema they half
   understand.
3. **Tiered rollback with quiescence, an ownership manifest and reflink copies.**

## Proposed decision

Adopt option 3, as six rules that are individually testable. A recovery implementation that does not
satisfy all six must not be described to users as protecting their work.

1. **Quiesce before copying, and verify the quiesce.** Stop the application and its background
   writers so that deferred state is flushed and file descriptors close. **Issuing a stop command is
   not sufficient evidence that it worked** — the documented Wine command escalates to an
   uncatchable kill after roughly ten seconds, which bypasses the shutdown flush. The procedure must
   confirm that the expected state was actually written before the copy is taken. No copy of a live
   prefix is honest.
2. **Classify every path, and keep the four recovery layers separate.** Each prefix path belongs to
   exactly one tier: *runtime state* (revertible), *application state* (revertible with the
   application), or *user data* (**never** reverted, backed up separately). The tier split is
   declared, not guessed. Deriving it requires observing the installer, which is why rule 6 exists.

   Separately, **runtime rollback, prefix rollback, user-document recovery and external or
   server-side state are four different things** and are designed, tested and reported separately.
   A virtual-machine checkpoint restores a whole machine; it is **not** validated application
   recovery and must never be presented as evidence of it.
3. **Produce an independent recoverable copy. Never a mutable source-to-snapshot hardlink.**
   *(Amended 2026-09-07 — the original wording mandated reflink support, which execution showed is
   not available on ordinary filesystems.)* The requirement is a property, not a mechanism: after
   the copy exists, **subsequent writes to the source must not change it**, and that independence is
   **verified**, not assumed. Acceptable mechanisms, in order of preference:
   - a **quiesced full copy** — stop all relevant writers, copy into a fresh destination, then
     verify contents, the metadata the application depends on, and independence from subsequent
     writes. This is the baseline and it works everywhere;
   - a filesystem snapshot or reflink copy **where capability-testing proves it is available**.
     Reflink is **optional and capability-tested**, never assumed — and a command that silently
     falls back to a full copy on failure is **not** evidence that reflink was used. Probe the
     capability explicitly and record the answer.

   In every case: group each database with its journal and write-ahead sidecar files as one atomic
   unit, retain N generations, and write the completion marker last and flush it, so that a torn
   copy self-invalidates.
4. **Stamp a forward-only version beside the data** and refuse to launch an older binary against
   newer data, rather than letting it write. Refusal is a visible, recoverable event; silent
   corruption is not.
5. **Check and reserve free space before writing any byte.** Storage exhaustion mid-update is the
   likeliest destroyer of user data, and it is unguarded almost everywhere.
6. **Delete only from an installer-generated manifest, never from a computed recursive path.** Every
   recursive delete asserts a non-empty, expected path prefix before running. This rule exists
   because the most destructive incident found in the research was caused by exactly this bug in
   exactly this kind of software.

Additionally: prefixes are refused on network filesystems, where locking is documented as a
corruption cause; regenerable payload directories are tagged so that users' existing backup tools
stay useful, while user-data directories are not.

## Consequences

Rule 2 is a permanent curation obligation, not a one-time engineering task. Even a vendor
snapshotting its own data with full source knowledge ships an incomplete manifest, so HELM will not
achieve completeness from the outside by heuristic. The honest response is to measure the miss rate
by tracing real applications and to publish it as the error bar on the recovery claim.

Rule 1 makes recovery invasive: the application must be stopped. That is accepted, because the
alternative is a promise HELM cannot keep.

The novelty claim narrows accordingly. Snapshot and restore is not novel. What is unbuilt is
attributing a regression to the axis that caused it and reverting only that axis while leaving the
user's documents alone.

## Evidence

Foundation audit [§3.8](../research/FOUNDATION_AUDIT.md#s03),
[§4.5](../research/FOUNDATION_AUDIT.md#s04), constraints C2, C3 and C9 in
[§7](../research/FOUNDATION_AUDIT.md#s07), and risk R-A7 in
[§8](../research/FOUNDATION_AUDIT.md#s08).

The ten-minute falsification test for rule 3 is gate check G0-3 in
[EXP-009](../experiments/EXP-009.md); criterion P5 of that experiment is the acceptance test for the
whole decision. One material question is untested and must be answered before recovery is promised:
whether restoring a snapshot consumes or invalidates a software licence activation.

## Corrections applied 2026-09-07 (ADR remains Proposed)

Gate 0 produced execution evidence that contradicts two of the six rules. This ADR must be amended
before it is taken to review; it is recorded here rather than silently edited.

1. **Rule 3 mandated reflink copies, and reflink was unsupported on both filesystems tested** —
   including the ext filesystem of a default Ubuntu install. See
   [EXP-009 Gate 0 report, G0-3a](../experiments/EXP-009-GATE0-REPORT.md). **Amended 2026-09-07 on
   owner instruction:** the rule now requires *independent recoverable state* as a verified
   property, with a quiesced full copy as the portable baseline and reflink treated as optional and
   capability-tested.
2. **Rule 1's quiesce step was not sufficient as written.** The documented command escalates to an
   uncatchable kill after roughly ten seconds, which bypasses the shutdown flush entirely.
   **Amended:** the rule now requires *verification* that the expected state was written, not the
   issuing of a command.

Two clarifications that strengthen rather than weaken the ADR:

3. **The hardlink prohibition is confirmed by execution and its justification is now stronger.**
   Contamination — the copy silently tracking live data — is deterministic, permanent and requires
   no crash. That alone disqualifies hardlink farms. Corruption of the live prefix is a separate
   failure requiring an additional fault, and must not be claimed without inducing one.
4. **One backup tool was wrongly listed as dangerous.** A hardlinking backup tool that links
   between successive backups leaves the live file's link count at one and is not affected. The
   general rule stands; that specific attribution is withdrawn.

Also note that the upstream deferred-save timing cited in the Context does not hold for the Proton
fork, which has no periodic save timer and performs the write client-side. The hardlink hazard is
the same there; the timing is not.

## Revisiting

An upstream mechanism appears that provides application-consistent per-application snapshots; or
measurement shows the ownership manifest cannot be derived accurately enough to be safe, in which
case HELM must reduce the promise to "we snapshot the whole prefix and tell you exactly what that
does" rather than claiming user-data safety.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
