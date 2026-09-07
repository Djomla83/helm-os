# ADR-0016: The application boundary runs on the host, using bubblewrap plus Landlock

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

[ADR-0005](ADR-0005-sandbox-boundary.md) establishes that a Wine prefix is not a security boundary.
It does not say where the boundary runs. The audit produced four findings that decide that question.

- **Flatpak cannot host HELM's architecture.** Its seccomp policy unconditionally denies `unshare`,
  `setns`, `mount`, `pivot_root`, `chroot` and `clone(CLONE_NEWUSER)`. A per-Windows-application
  `bubblewrap` sandbox nested inside a HELM Flatpak is therefore impossible by construction, not
  merely awkward. This is why the leading incumbent carries a second, less expressive
  sub-sandbox backend — and on that path `/dev/ntsync` silently disappears and NTSync stops working.
- **The Flatpak boundary is already punctured for Wine workloads.** Every shipping Wine-hosting
  Flatpak grants development-mode syscalls, multi-architecture execution, all devices, and an X11
  socket that Flatpak's own documentation calls "completely insecure". What remains is roughly a
  home-directory boundary.
- **Unprivileged user namespaces are not guaranteed.** `bubblewrap` 0.12.0 removed setuid support
  entirely, and Ubuntu 24.04 and later restrict unprivileged user namespaces through AppArmor. A
  HELM sandbox helper without an AppArmor profile will fail to start on the most common target.
- **Setuid helpers are the known-bad architecture.** Firejail's eighteen CVEs, overwhelmingly local
  privilege escalation to root, trace to a setuid-root binary with a large privileged argument
  parser.

## Options considered

1. **Ship HELM as a Flatpak and rely on Flatpak as the boundary.** Rejected: nesting is impossible,
   the inherited boundary is already punctured, and describing it as HELM's security boundary would
   be a misrepresentation.
2. **Rely on the Steam runtime container.** Rejected: it is runtime abstraction, not confinement.
   Valve's own shipped documentation lists the user's home directory and several other trees as
   shared read/write by default.
3. **A setuid helper with rich policy flags.** Rejected on the Firejail evidence.
4. **A host-side sandbox composed from bubblewrap, Landlock and a HELM-authored seccomp filter,
   with a small privileged service only where genuinely required.**

## Proposed decision

Adopt option 4.

- HELM's sandbox helper runs **on the host**. HELM's own graphical tooling may still be distributed
  as a Flatpak, but that Flatpak is a delivery vehicle and is never described as the boundary.
- The boundary is composed of: `bubblewrap` for the mount and namespace topology; **Landlock** as a
  namespace-free second layer that survives Ubuntu's userns restriction and works where `unshare` is
  denied; and a HELM-authored seccomp filter, because Flatpak's binary development-mode switch is
  too coarse — HELM wants `ptrace` confined to the application's own process tree without also
  granting performance-counter access.
- **HELM ships an AppArmor profile for its sandbox helper.** This is a packaging obligation, not
  optional hardening.
- **HELM ships no setuid binary.** Where a privileged operation is unavoidable it is a small,
  audited, D-Bus-activated service with a narrow typed interface, never a command-line tool with
  many flags.
- The policy — not the mechanism — is the thing HELM owns and the thing that must be reviewed. HELM
  contributes no isolation mechanism and must not imply that it does.
- HELM implements a **permission-learning mode**: run an application under the intended policy in
  log-only mode and emit the minimal policy that lets it work. Snap has this concept and Flatpak does
  not, and its absence is a large part of why published manifests grant all devices.
- **An escape harness is a permanent regression test, not a one-off audit.** A small Windows binary
  that enumerates and attempts access outside the prefix runs against every supported configuration
  on every change.

## Consequences

HELM takes on cross-distribution packaging work it would not need inside Flatpak: AppArmor on
Ubuntu, no reliance on SELinux confinement on Fedora, and a userns-availability probe with a
Landlock-only degraded mode.

Confinement will cost functionality, and the audit is explicit that Windows applications resist it:
installers want system-wide paths, and launchers expect to find sibling installations. Under
[ADR-0005](ADR-0005-sandbox-boundary.md) those losses must be visible and tested rather than fixed by
silently widening permissions. HELM must also record honestly that v1 keeps an X11 socket open — see
[ADR-0018](ADR-0018-win32-portal-bridge.md) — and that this is the largest remaining hole in an
otherwise tight policy.

## Evidence

G0-2 prerequisite annotation, 2026-09-07: [direct negative attempts](../experiments/EXP-009-GATE0-REPORT.md#g0-2-completion)
in the three disposable WSL labs failed to launch a Windows system executable with interoperability
configured off. This checks only the recorded execution property. It is not a reviewed outer
containment boundary, an escape matrix, or completion of G0-5. This ADR remains **Proposed**.

Foundation audit [§3.4](../research/FOUNDATION_AUDIT.md#s03),
[§4.3](../research/FOUNDATION_AUDIT.md#s04), constraint C5 in
[§7](../research/FOUNDATION_AUDIT.md#s07), and risk R-A6 in
[§8](../research/FOUNDATION_AUDIT.md#s08). Gate check G0-5 in
[EXP-009](../experiments/EXP-009.md) produces the first escape matrix.

Remaining uncertainty: no measurement exists of the minimal permission set that real Windows
applications actually need. Producing one would be a useful contribution to the wider ecosystem
whether or not HELM ships.

## Revisiting

Flatpak gains a nested-sandbox capability with device granularity; or Landlock gains the coverage
that would let it replace rather than complement namespaces; or measurement shows the confinement
cost in lost functionality is unacceptable for the target application class, in which case the
honest response is to publish the limitation, not to widen permissions silently.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
