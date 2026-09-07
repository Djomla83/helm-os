# EXP-009 Gate 0 — results report

| Field | Value |
|---|---|
| Experiment | [EXP-009](EXP-009.md) revision 2, Gate 0 stage only |
| Date | 2026-09-07 |
| Executor | Development agent, unattended, on the maintainer's workstation |
| Reviewer | **not assigned** — this report is not accepted |
| Repository revision at execution | `e09a52f` |
| Approved budget | none set; no spend incurred, no software installed |
| Outcome summary | 1 PASS (partial), 4 BLOCKED, 0 FAIL |

> **This report contains no Windows compatibility result.** No Windows application was installed or
> run, because no Wine of any version is present in the available environment. Nothing here should
> be read as evidence about application compatibility.

---

## 1. Environment as found

Recorded before any check was attempted, per [EXP-009 §3](EXP-009.md#3-recording-requirements).
Machine-readable record: [`evidence/G0-environment-2026-09-07.json`](evidence/G0-environment-2026-09-07.json).

| Property | Value |
|---|---|
| Host OS | Microsoft Windows 11 Pro 10.0.26200 |
| Linux environment | WSL2, Ubuntu 24.04.4 LTS, kernel `6.6.87.2-microsoft-standard-WSL2` |
| GPU | NVIDIA GeForce RTX 3080, Windows driver 32.0.16.1664 |
| Free disk | 138 GB |
| Present | `python3` 3.12.3 (WSL), `python` 3.14.3 (Windows), `git` 2.53.0 |
| **Absent** | `wine`, `wine64`, `wineserver`, `winetricks`, `bwrap`, `flatpak`, `gcc`, `make`, `xdg-desktop-portal`, `gnome-shell`, `plasmashell`, `podman`, `docker`, `qemu-system-x86_64` |
| Foreign architectures | none enabled (no i386) |
| Display | WSLg present (`DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`), **no desktop environment, no portal backend, no tray host** |

### 1.1. What this environment can and cannot decide

These constraints were determined by probing, not assumed:

- **No Wine is installed**, so every Wine-dependent check is BLOCKED. Installing it would be a
  change to the maintainer's machine, which `AGENTS.md` requires an approved lab and human approval
  for. That approval was not sought or given, so it was not done.
- **No compiler** is present, so checks requiring a probe to be built are BLOCKED even before Wine.
- **Kernel 6.6** — the kernel synchronisation primitive the audit discusses requires 6.14 or later,
  so that behaviour could not be exercised here even with Wine installed.
- **No desktop environment, portal backend or tray host**, so **no desktop-integration claim can be
  tested in this environment at all**, with or without Wine.
- **The GPU is reachable only through the WSL2 driver path**, which is not a representative Linux
  graphics stack, so no graphics or driver conclusion is available here.
- **Neither available filesystem supports reflink copies** — which turned out to be a result in its
  own right, see G0-3a.

WSL2 would remain the wrong environment for the integration and graphics checks even if Wine were
installed. Those need a real Linux session on real hardware.

---

## 2. Results by check

### G0-1 — Is the motivating problem in the Windows-compatibility layer? → **BLOCKED**

| Field | Value |
|---|---|
| Outcome | **BLOCKED** |
| Blocked on | No Wine; no authorised test accounts; the reported symptom has never been reproduced on a machine under test. |
| Unblock condition | An approved lab with Wine installed, plus a reproduction of the user's actual symptom on an instrumented system. |

Desk evidence gathered for this check is recorded in the corrected
[audit §6](../research/FOUNDATION_AUDIT.md#s06). Two corrections were made to previously reported
findings, both recorded in [audit §13](../research/FOUNDATION_AUDIT.md#s13):

- The earlier claim that the tracker held "exactly one" bug for the motivating application was
  **wrong** — a quicksearch returns five — and, more importantly, the inference drawn from it was
  invalid. **Bug-report counts do not measure usage.** That inference is withdrawn.
- The earlier suggestion that a 2026 tray regression explained the user's instability is
  **withdrawn**. It was never reproduced, and the affected code path is not present in the shipping
  client under discussion.

**The cause of the user's reported experience remains unestablished.** No layer has been ruled in or
out by execution. That is the honest state, and it is why this check is BLOCKED rather than
INCONCLUSIVE: the decisive work has not been attempted, not merely failed to decide.

### G0-2 — Does the target application class need staging-only Wine features? → **BLOCKED**

| Field | Value |
|---|---|
| Outcome | **BLOCKED** |
| Blocked on | No Wine, no wine-staging, no Proton, and no C compiler in the environment. |
| Unblock condition | An approved lab with all three runtimes and a toolchain; the probe is small and the check is deterministic once they exist. |

No inference is offered in place of the measurement.

### G0-3a — Filesystem snapshot mechanics (Wine-free part) → **PASS**

| Field | Value |
|---|---|
| Outcome | **PASS** — the pre-registered expectation held on both filesystems tested |
| Executed | 2026-09-07, unattended |
| Probe | [`tools/gate0/probe_snapshot_semantics.py`](../../tools/gate0/probe_snapshot_semantics.py) (repository revision `e09a52f`) |
| Artifacts | [ext4](evidence/G0-3-snapshot-semantics-ext4.json), [drvfs](evidence/G0-3-snapshot-semantics-drvfs.json) |

Commands:

```bash
# WSL2 Ubuntu, native ext4
python3 probe_snapshot_semantics.py --workdir ~/helm-g0
# WSL2 Ubuntu, Windows drive via the 9p filesystem
python3 probe_snapshot_semantics.py --workdir /mnt/c/Users/<user>/AppData/Local/Temp/helm-g0
```

Observed, identically on both filesystems:

| Question | Result |
|---|---|
| Does an in-place truncate-and-rewrite of the live file change a hardlink "snapshot"? | **Yes.** The copy's digest changed and afterwards matched the live file exactly — the copy tracks live data. |
| Does write-temp-then-rename leave the hardlink copy intact? | **Yes.** The copy retained its original digest and the live file's link count dropped to one. |
| Is a zero-length window observable during the in-place rewrite? | **Yes.** The copy was observed at zero bytes mid-write. |
| Does the filesystem support reflink copies? | **No — on both.** `cp --reflink=always` failed with "Operation not supported". |

Filesystems recorded: the ext family on `/dev/sdd`, and the 9p filesystem for the mounted Windows
drive.

**Independent output check.** Verdicts were not taken from the probe's own summary text. Each was
recomputed from the recorded SHA-256 digests in the artifacts: the contamination verdict follows
from `after.snapshot == after.live` together with `before.snapshot != after.snapshot`; the
preservation verdict from `before.snapshot == after.snapshot`; and the reflink verdict from the
non-zero exit status of the copy command. All four recomputations agree with the reported verdicts.

**What this shows.** Hardlink farms are not snapshots: contamination is deterministic, silent, and
needs no crash. **This is disqualifying on its own**, which is the corrected form of the audit's
claim.

**What this does not show — stated plainly.** The probe uses a **synthetic writer**. Wine was not
installed and was not executed. It therefore says nothing about which save path Wine's own code
takes at runtime, and it does **not** demonstrate corruption of a live prefix — only that the window
in which a fault could cause it exists. Results are specific to the two filesystems tested.

**Unexpected result worth flagging to the maintainer:** neither available filesystem supports
reflink copies. **ADR-0017 proposed reflink snapshots as the safe mechanism**, and that mechanism is
unavailable on a default Ubuntu install. See [§4](#4-recommendations-on-the-proposed-adrs).

### G0-3b — Wine registry save behaviour → **BLOCKED**

| Field | Value |
|---|---|
| Outcome | **BLOCKED** |
| Blocked on | No Wine installed. |
| Unblock condition | An approved lab with a pinned Wine build; procedure specified in [EXP-009 §8](EXP-009.md#g0-3--are-hardlink-farms-safe-as-prefix-snapshots). |

Source reading performed for this check (VERIFIED_SOURCE, not execution) refined the audit and is
recorded as corrections C-7 and C-8: the in-place path has a precondition the audit omitted; one
backup tool was wrongly listed as dangerous and has been withdrawn; and the documented quiesce
command escalates to an uncatchable kill, so it cannot be assumed to flush cleanly.

### G0-4 — True state of the art in environment derivation → **BLOCKED**

| Field | Value |
|---|---|
| Outcome | **BLOCKED** |
| Blocked on | The incumbent tool is not installed; installing it requires an approved lab, and running it meaningfully requires Wine and a graphical session. |
| Unblock condition | Approved lab; then run it over the cohort and record whether the derived configuration actually runs each application. |

Source reading at a pinned commit sharpened what this check must measure, and corrected the audit
(C-3): the analyzer is static and never executes the target to test a suggestion, community-derived
suggestions default to off, dependency suggestions are named but not installed, and there is no
feedback channel. The decisive question — **does the derived configuration actually work** — is
exactly what nobody has measured, and it stays open.

### G0-5 — What does per-application confinement contain? → **BLOCKED**

| Field | Value |
|---|---|
| Outcome | **BLOCKED** |
| Blocked on | No `bwrap`, no Flatpak, no Wine, and no compiler for the probe binary. |
| Unblock condition | Approved lab with the sandbox tooling and a Windows toolchain for the probe. |

Nothing was inferred about confinement strength.

---

## 3. Costs and interventions

| Measure | Value |
|---|---|
| Software installed | **None.** No package was installed on the host or in WSL. |
| Money spent | None. |
| Host changes | None persistent. The probe wrote only to disposable temporary directories, which it removed. |
| Manual human interventions | **Zero** during execution. |
| Machine time | Under one minute for the executed probe; environment discovery a few minutes. |
| Automated research | 8 parallel agents, 611 tool calls, ≈1.19M tokens, ≈32 minutes wall-clock, for the corrections research supporting this report. |
| Agent necessity | The executed check (G0-3a) required **no** model at all — it is deterministic scripting. The model was used for source reading and for correcting the audit, which is the genuinely agent-shaped part. |

No baseline arm was measurable: with no Wine present, arms B0, B1 and B2 are all unavailable, so no
comparative cost claim is made.

---

## 4. Recommendations on the proposed ADRs

**All seven remain `Proposed`. None is accepted, and Gate 0 does not accept any of them.** These are
recommendations to the maintainer, based on what execution and corrected source reading now show.

| ADR | Recommendation | Basis |
|---|---|---|
| [ADR-0013](../adr/ADR-0013-pinned-wine-runtime.md) — ship a pinned Wine | **Hold.** Unaffected by Gate 0, but untested: no Wine was run. | Its premises are source-based and were not contradicted. |
| [ADR-0014](../adr/ADR-0014-version-scoped-profiles.md) — version-scoped profiles | **Hold, with one strengthening note.** The comparison against the mature incumbent shows a history-isolation mechanism already exists there as a natural hook for an application-build axis; the ADR should reference that prior art rather than presenting the idea as unprecedented. | Corrections C-4, C-5. |
| [ADR-0015](../adr/ADR-0015-evidence-expiry.md) — reproducible, expiring, gating evidence | **Amend before any acceptance.** Its novelty claim is materially overstated: traceability, artifact collection, a closed result vocabulary, reproducible rerun, non-pixel oracles and last-good attribution already ship in openQA. The defensible remainder is **artifact content-hash identity** and **mechanical verdict expiry**, plus two schema-shape fixes. The ADR should also evaluate adopting openQA rather than building. | Corrections C-4, C-5. |
| [ADR-0016](../adr/ADR-0016-host-side-sandbox.md) — host-side boundary | **Hold.** Untested; G0-5 is BLOCKED. | No new evidence either way. |
| [ADR-0017](../adr/ADR-0017-data-safety-rules.md) — tiered, quiesced, reflink-based recovery | **Amend before any acceptance — two rules are contradicted.** (i) Rule 3 mandates reflink copies; **reflink was unsupported on both filesystems tested**, including a default Ubuntu install, so the ADR must add a filesystem precondition and a supported fallback. (ii) Rule 1's quiesce step relies on a command that escalates to an uncatchable kill; it must **verify** the hive was written rather than assume it. The hardlink prohibition itself is **strengthened**, not weakened: contamination alone is disqualifying, confirmed by execution. | G0-3a; corrections C-7, C-8, C-10. |
| [ADR-0018](../adr/ADR-0018-win32-portal-bridge.md) — Win32-to-portal bridge | **Hold.** Untested; the environment has no portal backend. | No new evidence. |
| [ADR-0019](../adr/ADR-0019-scope-boundary.md) — publish the unsupportable class; decide the product with evidence | **Hold, and note that its deciding input is still missing.** G0-1 is BLOCKED, so the Track A / Track B decision has **not** acquired the evidence this ADR says should settle it. Do not settle it by argument in the meantime. | G0-1 BLOCKED. |

A new [ADR-0020](../adr/ADR-0020-documentation-language.md) proposes the documentation-language
amendment rather than continuing to override `CONTRIBUTING.md` silently.

---

## 5. Recommendation on proceeding to the PoC

**Do not start the Evidence Loop PoC yet.** Four of five Gate 0 checks are BLOCKED on the same
missing precondition, so the PoC's central assumptions are still untested.

The blocking item is small and specific: **an approved lab**. Concretely, a Linux environment with a
pinned Wine, a compiler, sandbox tooling and — for the integration and graphics questions — a real
desktop session on real hardware. WSL2 can unblock G0-2, G0-3b and part of G0-4. It cannot unblock
the integration or graphics questions at all, and a result obtained there must not be presented as
if it were.

Recommended order, none of which needs the PoC:

1. Maintainer decides whether to authorise a lab, and where it runs. This is the only real blocker.
2. Re-run Gate 0's blocked checks in that lab and re-issue this report.
3. Amend ADR-0015 and ADR-0017 as above, then take all seven ADRs to review as a set.
4. Only then decide on the PoC, with the Track A / Track B question answered by G0-1 rather than by
   preference.

**What Gate 0 did settle**, without a lab and at negligible cost: one proposed recovery mechanism is
unavailable on ordinary filesystems, one proposed test procedure was invalid and has been replaced,
one factual claim in the audit was wrong and is corrected, one causal claim was unsupported and is
withdrawn, and the evidence pillar's novelty is roughly half what was claimed. Those are all
cheaper to learn now than after implementation.

---

## 6. Preserved failures and limitations

- **No check was retried.** G0-3a ran twice by design, on two different filesystems, and both runs
  are reported.
- **One earlier session artifact was mishandled and is recorded here for completeness:** a saved
  anti-bot challenge page containing a public IP address was staged into a commit during repository
  setup, detected on inspection, removed before publication, and added to `.gitignore`. No
  credential was involved. It is reported because the experiment's own rules forbid dropping
  failures.
- **The audit's corpus counts remain unpinned** to specific revisions and must not be quoted as
  stable figures.
- **Several upstream sources block automated access.** This bounds every negative search result in
  the supporting research, and it also means "documents consulted" is not comparable between a human
  arm and an automated arm.
- **No application, benchmark, sandbox, graphics, integration or recovery result exists.** The
  compatibility rate remains unknown.
