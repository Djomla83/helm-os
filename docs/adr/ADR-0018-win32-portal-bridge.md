# ADR-0018: Desktop integration is delivered by a Win32-to-portal bridge, and integration state is verified

**Status:** Proposed\
**Draft date:** 2026-09-06\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

The audit set out to find whether "feels unpolished" is caused by Wayland, by compositors, by
toolkits, or by Wine. For Windows applications the answer is overwhelmingly Wine, and the specific
missing pieces are enumerable rather than vague.

- `RegisterHotKey` is implemented only in the macOS driver. On both X11 and Wayland it falls through
  to a null driver that **returns success without registering anything**, so global hotkeys silently
  work only while a Wine window has focus.
- Tray balloons are drawn as internal Win32 tooltips and never reach the desktop notification
  service. Wine has essentially no D-Bus desktop integration at all.
- Desktop duplication and Media Foundation device capture are `E_NOTIMPL` stubs, and Wine links no
  PipeWire and speaks no portal, so screen sharing and camera access are unreachable for a Windows
  application no matter how its environment is configured.
- Suspend and resume notifications are stubs, so a Windows network application never learns that it
  was suspended and never triggers a reconnect.
- Meanwhile the host substrate is healthy and is not the bottleneck: xdg-desktop-portal exposes
  roughly thirty-two interfaces including a USB portal, and global-shortcut backends now exist on
  both major desktops.

Two further findings shape the decision. Portals are **pull-based** — the application must call
them — and a Windows application never will, which is why confined Wine applications are given broad
filesystem and device grants instead. And Wine still defaults to X11: its Wayland driver is
experimental, does not bind window activation, and binds only a clipboard protocol that neither
current major desktop implements.

Finally, a counter-example worth preserving: file associations, URL handlers and dock identity
already work through `winemenubuilder`. The common "generic icon" complaint is usually a missing
desktop file or a launch by path, not a protocol gap.

## Options considered

1. **Work around the desktop.** Rejected: it would build elaborate compensation for problems that are
   one upstream patch each, and would create a second, divergent integration path that itself rots.
2. **Wait for upstream protocols.** Rejected for tray specifically: the Wayland tray proposal has
   been unmerged since October 2024, and the StatusNotifierItem specification has been a draft at
   "Version 0.1, Publication Date: TBD" for well over a decade.
3. **Build a Wine-side bridge from Win32 APIs to portals and D-Bus, and verify the resulting
   integration state.**

## Proposed decision

Adopt option 3, in priority order by leverage per unit of work.

1. **Global hotkeys** — implement the driver hook against the global-shortcuts portal. This is the
   cleanest example of a visible symptom that is entirely Wine's fault and fixable without touching
   any compositor.
2. **Notifications** — route the tray notification path to the desktop notification service.
3. **Tray icons** — bridge the Win32 tray protocol to StatusNotifierItem. This is display-server
   independent, works today on desktops that host it, and must include the **re-registration
   recovery loop** that the audit identified as the entire difference between a well-behaved client
   and a broken one when a shell restarts.
4. **File chooser** — route the common dialogs to the file-chooser portal, which is what makes
   confinement usable at all: without it a confined application sees only what HELM pre-mounts.
5. **Suspend and resume** — a user-session agent on the login manager's sleep signal with a delay
   inhibitor, synthesising the Windows power-broadcast message into managed prefixes. Never a
   system-sleep hook, because the user slice is frozen there.

Two further commitments:

- **HELM v1 is an Xwayland product and says so.** The Wayland driver is an opt-in per-application
  profile, not the default. The X11 socket is recorded as the largest remaining hole in HELM's
  boundary rather than being quietly omitted.
- **Integration state is verified, not fire-and-forget.** For each managed application HELM checks
  that its desktop entry, icon, MIME associations and URL handlers actually exist and match, and it
  runs a startup capability probe recording which Wayland globals and portal interfaces the session
  actually advertises, so that silent upstream regressions become a dated signal.

Screen capture and camera are **explicitly deferred**. Implementing desktop duplication over
PipeWire and Media Foundation capture is a multi-month project, and it must be scoped separately.
Until then, video conferencing through a Windows application is a published non-goal.

## Consequences

This is upstream-shaped work: each item is a bounded patch that benefits every Wine user, which is
the kind of contribution that earns standing with upstream and answers the criticism that a wrapper
which files no fixes contributes nothing. It also means HELM's roadmap contains real Wine
development, and the owner must decide whether the project funds or performs it — an answer that
sets the maximum reliability HELM can ever deliver.

The tray and conformance work generalises beyond Wine and connects directly to
[ADR-0019](ADR-0019-scope-boundary.md) and to the Track B option in the audit: the same harness that
verifies a Wine application's integration verifies a native or Electron application's integration.

## Evidence

Foundation audit [§3.5](../research/FOUNDATION_AUDIT.md#s03),
[§4.7](../research/FOUNDATION_AUDIT.md#s04), [§5.2](../research/FOUNDATION_AUDIT.md#s05) items N4 and
N5, [§6](../research/FOUNDATION_AUDIT.md#s06), and constraint C6 in
[§7](../research/FOUNDATION_AUDIT.md#s07). The Wine 11.0 statement that the Wayland driver is
experimental was verified first-hand, per [§12](../research/FOUNDATION_AUDIT.md#s12).

Not covered by the audit and material to this decision: per-monitor and fractional scaling, complex
script input, and printing. Each belongs in the integration checklist before any claim of
first-class behaviour is published.

## Revisiting

Upstream Wine gains portal integration, making part of this work unnecessary — the correct response
is to contribute rather than to duplicate; or the Wayland driver reaches parity with the X11 driver
for application-shaped software, which would reverse the Xwayland default set above and should
trigger a re-measurement rather than an assumption.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
