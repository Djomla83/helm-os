# ADR-0013: Build, pin and ship HELM's own Wine runtime

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

The audit established three facts that make "use the Wine the user has" untenable, and one that
makes "pin the annual stable" untenable.

- Upstream Wine has **no maintained stable branch**. Wine 8.0 received two point releases and 9.0
  received one, but 10.0 and 11.0 have received none. Pinning the annual stable therefore buys a
  year with no bug or security fixes.
- Distribution packaging is a full major version behind on the most likely bases: Debian 13, Debian
  unstable and Ubuntu 26.04 LTS all carry Wine 10.0, which predates the release in which the new
  WoW64 mode and NTSync became real. Every recipe validated on 11.x is untested there.
- Every distribution ships exactly one Wine system-wide, so even a current distribution package
  cannot give per-application pinning, and upgrading it changes behaviour for every application at
  once — the exact global-regression failure mode HELM claims to eliminate.
- Three capabilities that decide whole application classes exist only in wine-staging, most
  importantly DirectComposition, which gates the entire WebView2, CEF and Electron class.

## Options considered

1. **Consume the distribution's Wine.** Cheapest, and immediately wrong: stale, system-wide, and
   unpinnable.
2. **Pin the annual upstream stable.** Simple to explain, but ships known-unfixed defects for up to
   a year with no backport channel.
3. **Consume Proton exclusively through umu.** Attractive — Valve maintains it — but Proton is
   game-shaped, its runtime is pinned per Proton major rather than per application, and Valve
   documents no support for use outside Steam.
4. **Build, pin and ship HELM's own Wine builds, with Proton available as an alternative runtime.**
   Highest recurring cost, and the only option that satisfies per-application pinning.

## Proposed decision

Adopt option 4.

- HELM builds and ships versioned Wine runtimes and treats a runtime build as a first-class,
  immutable, content-addressed artifact identified by its source revision, patch set, compiler and
  configuration.
- Proton, obtained and launched through umu, is supported as an **alternative** runtime for
  applications where it measurably wins. HELM executes `umu-launcher` as a subprocess and never
  links it, because it is GPL-3.0.
- wine-staging patch sets are a **per-application opt-in** recorded in the profile, never a global
  default, and every enabled patch set is recorded in the runtime identity so that upstream can be
  told what was actually run.
- Mandatory build and lifecycle rules, each traceable to a verified finding:
  - every prefix is created 64-bit with `WINEARCH=wow64`; pure `win32` prefixes are never created;
  - the removed `wine64` loader is never invoked;
  - `wineserver -k -w` is executed before any runtime switch and before any snapshot;
  - HELM's own per-application state is written under `HKCU\Software\Wine` or outside the prefix,
    never into keys that `wine.inf` rewrites;
  - ntsync availability is probed and reported rather than assumed, because it fails silently.

## Consequences

HELM inherits a continuous re-qualification obligation against a substrate that releases roughly
every two weeks. That cost is only affordable if automated evidence works, which makes this decision
and [ADR-0015](ADR-0015-evidence-expiry.md) a single bet rather than two. HELM also becomes, by
upstream's own classification, an unsupported third-party prefix manager, so it must be able to
reproduce any failure on plain Wine automatically before reporting it.

This decision creates a standing delta from upstream and therefore stands in tension with
[ADR-0003](ADR-0003-upstream-first.md). It does not replace it: upstream-first remains the policy
for *patches* — every local patch keeps a reason, an owner, a test, the upstream commit it applies
to, and a removal condition — while the *build* becomes HELM's own. If the owner rejects that
reading, this ADR must be rejected rather than quietly reinterpreted.

## Evidence

Foundation audit [§3.1](../research/FOUNDATION_AUDIT.md#s03), [§7](../research/FOUNDATION_AUDIT.md#s07)
(constraints C1, C2, C4, C7), and the first-hand verified Wine 11.0 release announcement recorded in
[§12](../research/FOUNDATION_AUDIT.md#s12). Remaining uncertainty: the `wine.inf` upgrade analysis is
graded LIKELY, not VERIFIED, and the empirical upgrade and downgrade diff has not been run.

## Revisiting

Upstream restores a maintained stable branch with backports; or the DirectComposition patch set
merges upstream, removing the staging dependency; or measurement shows the re-qualification cost of
own builds exceeds what the project can staff, in which case HELM must reduce its supported runtime
count rather than its evidence standard.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
