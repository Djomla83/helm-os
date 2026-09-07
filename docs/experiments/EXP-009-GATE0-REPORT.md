# EXP-009 Gate 0 — results report

| Field | Value |
|---|---|
| Experiment | [EXP-009](EXP-009.md) revision 2, Gate 0 stage only |
| Date | 2026-09-07 |
| Executor | Development agent, unattended, on the maintainer's workstation |
| Reviewer | **not assigned** — this report is not accepted |
| Repository revision at execution | `e09a52f` |
| Approved budget | No new spend; Round 2 installed packages inside the disposable lab. |
| Current outcome summary | **G0-1 through G0-5: BLOCKED.** Separate mechanics subtests G0-3a and G0-3b: PASS within their recorded scope. G0-2 vanilla arm: INCONCLUSIVE; staging and Proton/UMU: NOT_RUN. See [current assessment](#current-assessment). |

> **This report contains no Windows application compatibility result.** Round 2 executed real Wine
> 11.17 in a disposable lab, but only against synthetic probes — no Windows application was
> installed or run. Nothing here should be read as evidence about application compatibility.
>
> **Round 1** (2026-09-07, morning) ran with no lab available. **Round 2** (2026-09-07, afternoon)
> ran after the maintainer authorised a bounded lab phase. Round 1's results are preserved
> below as historical observations; Round 2 is in [§2A](#2a-round-2-results-after-the-lab-was-provisioned).
> Publication corrections are explicitly dated below. They change status interpretation and redact
> private paths; they do not change the observations or the registered acceptance criteria.

<a id="current-assessment"></a>

## Current assessment — publication review, 2026-09-07

This section supersedes the earlier aggregate counts and status interpretations. The earlier
Round 2 summary was "3 PASS, 1 PARTIAL, 3 BLOCKED, 0 FAIL"; it mixed questions, subtests and checks,
and used `PARTIAL`, which is not in [EXP-009's outcome vocabulary](EXP-009.md#2-outcome-vocabulary).

| Registered check or separate subtest | Current outcome | Basis and remaining limit |
|---|---|---|
| G0-1 | BLOCKED | Authorised accounts and a reproduction are absent. |
| G0-2 overall | BLOCKED | Required runtime comparison is incomplete; the vanilla arm also lacks required control evidence. |
| G0-3 as originally registered | BLOCKED | Neither mechanics subtest establishes completion of the original Wine-specific criterion. The invalid historical procedure was not executed as registered; the later probe also lacks the prescribed read-only-hive case and an explicit machine-hive write. |
| G0-4 | BLOCKED | Application cohort and desktop session are absent. |
| G0-5 | BLOCKED | Reviewed containment boundary is absent; no escape test was attempted. |
| G0-3a, separately added filesystem subtest | PASS | Synthetic writer and two filesystems only. |
| G0-3b, separately reported filesystem/recovery semantics with Wine | PASS | Recorded hardlink contamination, full-copy independence and post-copy synthetic-document absence after restoring the old copy. This is not satisfaction of original G0-3, a corruption test, or application recovery. |

| G0-2 runtime arm | Outcome | Runtime identity and preserved observation |
|---|---|---|
| Vanilla | INCONCLUSIVE | `wine-11.17`; WineHQ `11.17~noble-1`; `DCompositionCreateDevice` returned `0x80004001` (`E_NOTIMPL`). |
| Wine staging | NOT_RUN | No staging build was provisioned or pinned. |
| Proton/UMU | NOT_RUN | No launcher, Proton build or Steam Linux Runtime was provisioned or pinned. |

The [unchanged vanilla artifact](evidence/G0-2-dcomposition-vanilla-wine-11.17.json) retains its
historical `PARTIAL` label and interpretation. Those fields are not the current assessment.
[The probe source](../../tools/gate0/dcomp_probe.c) contains neither the prescribed known-good
control call nor a deliberately broken control. Their execution is not evidenced. EXP-009 §5
therefore does not permit a PASS. No numerical G0-2 repetition count is specified in the registered
procedure; the report's claim of two observations is not backed by separate per-run records.

Read-only inspection found one saved vanilla stdout record, and confirmed the source, executable,
vanilla JSON and G0-3b JSON hashes against the retained lab files. Exact commands and output are in
[the provenance observation](evidence/G0-provenance-observation-2026-09-07.json). This inspection
did not execute or repeat a probe. Original probe stderr and repetition timing remain unavailable.

**Interpretation correction:** historical language below and in the foundation audit says this
probe decides the Chromium-embedding application class. That inference is withdrawn. This operation
alone establishes neither compatibility nor incompatibility of Chromium, Electron, WebView2,
embedded-browser applications or games. `S_OK` would not establish correct rendering; `E_NOTIMPL`
records this runtime's response to the tested call only.

**WSL interoperability:** `/etc/wsl.conf` requests disabled interoperability, but the observed
`WSLInterop` registration reads `enabled`. Effective Windows-process execution blocking is
**UNVERIFIED** until a direct negative execution test is performed. `/mnt/c` exists as a directory
and is not a mountpoint. Neither observation is a containment audit; see the
[lab runbook](LAB-G0-RUNBOOK.md#interop-observation).

ADR-0020 remains Accepted for documentation language only. All architecture ADRs remain Proposed.
The owner's current authorisation ends at publication/provenance repair: no staging or Proton/UMU
provisioning, G0-2 continuation, Hyper-V change or larger PoC is authorised by this correction.

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

### G0-3a — Filesystem snapshot mechanics (Wine-free subtest) → **PASS**, with a provenance caveat

> **Provenance check, performed 2026-09-07 at the maintainer's instruction — and it failed.**
> The criterion registered *before* execution, in `EXP-009` revision 1 as committed in `f6fe363`,
> described G0-3 as a **Wine registry** test. What was actually executed is a **filesystem-mechanics
> test driven by a synthetic writer, with no Wine involved**. The `G0-3a` / `G0-3b` split that makes
> this subtest legitimate was written in revision 2 **after** the run.
>
> Consequences, applied rather than argued away:
> - **G0-3 as originally registered is BLOCKED, not passed.** It is recorded as `G0-3b` below and
>   remains blocked on Wine.
> - **This subtest is reported as a newly added subtest**, not as satisfaction of the original
>   criterion. The criterion has not been relaxed to fit the result.
> - What *was* pre-registered for this subtest is narrower but real: its three questions and its
>   verdict logic were written into
>   [`probe_snapshot_semantics.py`](../../tools/gate0/probe_snapshot_semantics.py) before it ran, and
>   that file is committed, so the expectations were fixed in code ahead of the observation.
> - **This result must never be described as execution of Wine registry operations, or as a
>   demonstration of successful application recovery.** It is neither.

| Field | Value |
|---|---|
| Outcome | **PASS** for the subtest as encoded in the probe; the expectations were fixed in probe source before execution, but **not** in the experiment document |
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

<a id="2a"></a>

## 2A. Round 2 results, after the lab was provisioned

Lab: `helm-lab-g0`, a disposable WSL2 Ubuntu 24.04 distribution. Full manifest, isolation
configuration, reproduction steps and teardown: [LAB-G0-RUNBOOK.md](LAB-G0-RUNBOOK.md).

**The preferred VM path was not taken, and was not attempted.** Hyper-V's role is enabled and its
service is running, but `Get-VM` returns *"You do not have the required permission"* and the account
is not in `Hyper-V Administrators`. Creating a VM therefore requires a host administrator change,
which the authorisation excludes, so the stop rule applied. **Single approval that would unlock it:**
add the account to the local `Hyper-V Administrators` group (admin action, requires re-login). No
hypervisor install, Windows feature or reboot beyond that is needed.

Runtime installed: **`wine-devel` pinned at `11.17~noble-1`** from `dl.winehq.org/wine-builds/ubuntu`
noble/main — the exact release whose source the audit's registry findings were read from.

### G0-3b — filesystem/recovery semantics with Wine → **PASS** (separate subtest)

| Field | Value |
|---|---|
| Outcome | **PASS for the recorded mechanics subtest only.** Both recorded controls behaved as expected. Original G0-3 remains BLOCKED. |
| Probe | [`tools/gate0/probe_wine_registry.py`](../../tools/gate0/probe_wine_registry.py), transferred into the lab and verified **byte-identical** (`4ec6d1f4…`) to the committed source |
| Evidence | [`G0-3b-wine-registry-wsl2.json`](evidence/G0-3b-wine-registry-wsl2.json) (digest `75ce39d5…`, verified equal in-lab and in-repo) |

Questions were fixed in the probe source before execution. Observed with **real Wine 11.17**:

Publication correction: the earlier heading said "the Round 1 blocker is cleared". Wine's presence
cleared a missing-tool precondition, not the original experiment criterion. The probe issues an
HKCU write; changes in both hives are not evidence of an explicit HKLM write or the prescribed
read-only-hive case. The latter is absent from the probe and evidence.

| # | Question | Result |
|---|---|---|
| Q1 | Does a Wine registry write reach a hardlink copy of the prefix? | **Yes — contaminated.** `system.reg` and `user.reg` both changed in the copy, and all three hives track the live prefix exactly. |
| Q2 | Does a quiesced full copy stay unchanged after further writes? | **Yes — independent.** No hive in the copy changed. |
| Q3 | Does a user document created *after* the copy survive restoring that copy? | **No.** It is destroyed by a whole-prefix restore. |
| Q4 | Which hive records an `HKCU` write? | **Both** `system.reg` and `user.reg` changed. |
| — | Control: known-good (copy with no intervening write) | Compared **equal**, as required |
| — | Control: deliberately broken (mutated copy) | Difference **detected**, as required |
| — | Reflink capability, explicitly probed | **Unsupported** on the lab filesystem — a third filesystem, consistent with Round 1 |

**What this establishes.** The audit's hardlink claim moves from VERIFIED_SOURCE plus a synthetic
analogue to **VERIFIED_EXECUTION against real Wine**. Contamination is confirmed: a hardlink farm is
not a snapshot. The amended ADR-0017 baseline — a quiesced full copy — is confirmed to produce
genuinely independent state, verified by comparison rather than assumed.

**Q3 is the most consequential result in this report.** A naive whole-prefix restore silently
destroyed synthetic user work created after the recovery point. That is precisely the failure
ADR-0017 rule 2 exists to prevent, and it is now demonstrated rather than argued.

**Q4 refines audit correction C-9 rather than confirming it.** The correction said revision 1
inspected the wrong hive. In fact *both* hives changed on an `HKCU` write, so revision 1's procedure
would have coincidentally detected a change — but for the wrong reason, and its fatal defect
(comparing two hardlinks to the same inode, which are identical by construction) is unaffected. The
correction stands; its stated rationale is narrowed.

**An incidental finding that supports the ADR-0017 quiesce amendment.** `wineserver -k` returned a
**non-zero exit status** on both invocations, while the verification step (`wineserver -w` plus a
process check) confirmed the server had actually stopped. A design that trusted the command's exit
status would have drawn the wrong conclusion in both directions. This is exactly why the amended
rule requires *verification*, not issuance.

### G0-2 — DirectComposition comparison → **BLOCKED** (vanilla arm INCONCLUSIVE)

| Field | Value |
|---|---|
| Outcome | **BLOCKED overall; vanilla arm INCONCLUSIVE.** The vanilla observation is preserved, but required controls are not evidenced and the other arms did not run. |
| Probe | [`tools/gate0/dcomp_probe.c`](../../tools/gate0/dcomp_probe.c), cross-compiled with `x86_64-w64-mingw32-gcc (GCC) 13-win32` |
| Evidence | [`G0-2-dcomposition-vanilla-wine-11.17.json`](evidence/G0-2-dcomposition-vanilla-wine-11.17.json) (digest `63252e36…`) |

Observed on **vanilla Wine 11.17**: `dcomp.dll` loads, the entry point resolves, and
`DCompositionCreateDevice` returns **`0x80004001` (E_NOTIMPL)**. The earlier report stated
"Reproduced twice"; separate repetition records were not found during publication review. This
claim is retained as historical context, not upgraded into verified repetition evidence.

**Arms not tested, and therefore not concluded:** wine-staging (its package conflicts with the
installed `winehq-devel`) and Proton (needs `umu-launcher`, not installed). **The audit's claim is
that staging succeeds where vanilla does not — that half remains unverified.** What is now
established is only that the vanilla arm returns E_NOTIMPL, which is consistent with the audit and
insufficient to confirm it.

### G0-1, G0-4, G0-5 → still **BLOCKED**

| Check | Why it remains blocked |
|---|---|
| G0-1 | Needs authorised test accounts and a reproduction of the user's actual symptom. A lab removes missing-tool blockers; it does not supply an authorised account or a reproduction. |
| G0-4 | Needs the incumbent manager plus a graphical desktop session and the application cohort. The lab has no desktop environment. |
| G0-5 | **Deliberately kept blocked.** Hostile sandbox-escape testing must not run in an environment whose outer containment has not been reviewed. Not attempted. |

### Scope of validity

These results are valid **only for this WSL2 configuration** on kernel
`6.6.87.2-microsoft-standard-WSL2`. They must not be generalised to bare-metal graphics, device
support, battery life or physical sleep and resume. The lab has no desktop environment, portal
backend or tray host, so **no desktop-integration claim was or could be tested**. Wine reported
`libEGL` warnings about absent DRI3 during prefix creation, which is expected here and is a further
reason no graphics conclusion is available.

---

## 3. Costs and interventions

| Measure | Round 1 (no lab) | Round 2 (lab) |
|---|---|---|
| Software installed on the **host** | None | **None.** All packages were installed inside the disposable lab only. |
| Money spent | None | None |
| Persistent host changes | None | One new directory and one new WSL distribution — enumerated in the [runbook §2.1](LAB-G0-RUNBOOK.md) |
| Disk consumed | 0 | **13.4 GB** on the host (free space 138.0 → 124.6 GB). The lab's `ext4.vhdx` is 11.0 GB against its 16 GB sparse cap; the remainder is package download cache. |
| Manual human interventions during execution | 0 | **0** |
| Machine time | ~1 min probe, a few min discovery | ~6 min provisioning and package installation, ~30 s of probe execution |
| Model usage | Source reading and audit corrections | Same; **no probe required a model to execute** |
| Agent necessity | G0-3a is deterministic scripting | G0-3b and G0-2 are also deterministic scripting. The model's contribution was writing the probes and diagnosing an inconsistent tool report, not running anything. |

**A baseline arm was still not measurable.** Arms B0, B1 and B2 compare the effort of installing and
running a real Windows *application*; no application was installed in either round, so no
comparative cost claim is made. Round 2 does not change this — installing Wine is a precondition for
the baseline, not the baseline itself.

**One tooling failure is preserved rather than dropped.** During G0-2 the first command reported the
compiler as absent and the built binary as missing, while simultaneously producing valid probe
output — a self-contradictory result. It was not accepted: a separate verification step confirmed the
compiler was installed (GCC 13-win32), the binary existed as a 250,960-byte PE32+ executable, and the
result reproduced. The first report was an artefact of output ordering within a compound shell
command, not a real failure — but it was investigated before anything was recorded, not explained
away afterwards.

---

## 4. Recommendations on the proposed ADRs

**All seven remain `Proposed`. None is accepted, and Gate 0 does not accept any of them.** These are
recommendations to the maintainer, based on what execution and corrected source reading now show.

| ADR | Recommendation | Basis |
|---|---|---|
| [ADR-0013](../adr/ADR-0013-pinned-wine-runtime.md) — ship a pinned Wine | **Hold, with two premises now confirmed by execution.** The WineHQ repository carries `winehq-stable` 11.0.0.0 and 10.0.0.0 and `winehq-devel` through 11.17 for this base, and `winehq-devel` **hard-depends on the i386 package** — both were previously graded LIKELY and are now VERIFIED_EXECUTION. Pinning an exact version string worked as the ADR assumes. | Round 2 package installation. |
| [ADR-0014](../adr/ADR-0014-version-scoped-profiles.md) — version-scoped profiles | **Hold, with one strengthening note.** The comparison against the mature incumbent shows a history-isolation mechanism already exists there as a natural hook for an application-build axis; the ADR should reference that prior art rather than presenting the idea as unprecedented. | Corrections C-4, C-5. |
| [ADR-0015](../adr/ADR-0015-evidence-expiry.md) — reproducible, expiring, gating evidence | **Amend before any acceptance.** Its novelty claim is materially overstated: traceability, artifact collection, a closed result vocabulary, reproducible rerun, non-pixel oracles and last-good attribution already ship in openQA. The defensible remainder is **artifact content-hash identity** and **mechanical verdict expiry**, plus two schema-shape fixes. The ADR should also evaluate adopting openQA rather than building. | Corrections C-4, C-5. |
| [ADR-0016](../adr/ADR-0016-host-side-sandbox.md) — host-side boundary | **Hold.** Untested; G0-5 is BLOCKED. | No new evidence either way. |
| [ADR-0017](../adr/ADR-0017-data-safety-rules.md) — tiered, quiesced recovery | **Amended 2026-09-07 on owner instruction; still `Proposed`.** Rule 3 now requires *independent recoverable state* as a verified property, with a quiesced full copy as the portable baseline and reflink optional and capability-tested. Rule 1 now requires *verifying* the quiesce. Rule 2 now separates the four recovery layers explicitly. **Round 2 supports all three amendments with execution evidence:** the full copy was independent, the hardlink copy was contaminated by real Wine, `wineserver -k` returned non-zero while having genuinely stopped, and reflink was unsupported on a third filesystem. **Recommend accepting after review** — it is now the best-evidenced ADR in the set. | G0-3a, G0-3b; corrections C-7, C-8, C-10. |
| [ADR-0018](../adr/ADR-0018-win32-portal-bridge.md) — Win32-to-portal bridge | **Hold.** Untested; the environment has no portal backend. | No new evidence. |
| [ADR-0019](../adr/ADR-0019-scope-boundary.md) — publish the unsupportable class; decide the product with evidence | **Hold, and note that its deciding input is still missing.** G0-1 is BLOCKED, so the Track A / Track B decision has **not** acquired the evidence this ADR says should settle it. Do not settle it by argument in the meantime. | G0-1 BLOCKED. |

[ADR-0020](../adr/ADR-0020-documentation-language.md) was accepted by the owner for documentation
language only. No architecture ADR was accepted.

---

## 5. Recommendation on proceeding to the PoC

**Historical recommendations:** the paragraphs below retain the earlier engineering assessment.
The [current assessment](#current-assessment) corrects the G0-3 scope and G0-2 application-class
inference. The current owner instruction permits provenance repair only; none of these proposed
next steps is authorisation to execute it.

**Do not start the Evidence Loop PoC yet — but the reason has changed, and narrowed.**

After Round 2, the data-safety foundation is no longer speculative. The mechanism ADR-0017 depends
on is now demonstrated with real Wine: hardlink farms are contaminated, a quiesced full copy is
genuinely independent, and a whole-prefix restore destroys work created after the recovery point.
That is the part of the design most likely to cause irreversible user harm, and it is now
evidence-backed rather than argued.

What still blocks the PoC is unchanged in kind:

1. **The PoC's central thesis remains untested.** It is a claim about the *false-pass rate of
   automated evidence* and the *human cost of re-qualification*. Neither can be measured without
   installing and driving real applications, which needs a graphical desktop and an authorised
   application cohort. The lab supplies neither.
2. **G0-1 is still blocked**, so the Track A / Track B product decision still has no evidence.
3. **G0-2 is only half-done.** The claim that gates the Chromium-embedding application class is a
   *comparison* between vanilla, staging and Proton. Only vanilla ran.

Recommended next steps, in order, none of which is the PoC:

1. **Complete G0-2 cheaply.** A second disposable lab with `winehq-staging` pinned, plus `umu` for a
   Proton arm, finishes the comparison in roughly an hour. It is the highest value per unit of
   effort remaining, because it decides whether a whole application class is in scope.
2. **Take ADR-0017 to review with the Round 2 evidence attached**, and consider accepting it.
3. **Decide the environment for the application-level work.** A desktop session on real hardware, or
   the Hyper-V approval noted in [§2A](#2a). This is the gate on everything that remains.
4. **Only then** reconsider the PoC, with G0-1 answered.

**What Gate 0 has now settled, at a cost of one disposable lab and no host software:** one proposed
recovery mechanism was unavailable on every filesystem tested and the ADR was amended; the
replacement mechanism was verified to work; a whole-prefix restore was shown to destroy user work;
the hardlink prohibition moved from inference to demonstration; two packaging premises of ADR-0013
were confirmed; one audit correction was itself refined by execution; and a self-contradictory tool
report was caught and investigated rather than recorded. Every one of those is cheaper to learn now
than after implementation.

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
- **No application, benchmark, sandbox, graphics or integration result exists.** The compatibility
  rate remains unknown. Round 2 produced *state-independence* results, which are not application
  recovery results: nothing was shown to still work after a restore, only that state was or was not
  independent.
- **The lab was deliberately not torn down.** It is retained so the two missing G0-2 runtime arms
  can be completed cheaply. Teardown instructions are in
  [LAB-G0-RUNBOOK.md §6](LAB-G0-RUNBOOK.md#6-teardown); all evidence is already committed to the
  repository, so teardown loses nothing. Current cost of retention: 11.0 GB.
- **Publication status:** the results in this report are committed locally but, at the time of
  writing, **are not present on the remote**. See the report's closing note in the project state.

<a id="publication-provenance"></a>

## 7. Publication redaction and private provenance — 2026-09-07

The owner authorised one rewrite of the three local-only commits, solely for publication privacy
and the status/reference corrections recorded here. Public base
`e09a52fa2ed95c759e3e9370d4b0f0e95ce1fa20` and all its ancestors are preserved. The original local
HEAD was `76c0337f6f6af45b6755cebcee4ea1b138db57cb`; the other original local commits were
`0395c40a905a2fa11c18f4168cce74f252d43ebf` and `3b9da61e4ce6292464dddb282228d195909545d9`.
These are private provenance identifiers, not public source links.

The older `e09a52f` execution-revision references identify the recorded working base, not a commit
containing the probe sources. Those sources first appear in original local commits `0395c40`
(filesystem probe) and `3b9da61` (Wine probes), alongside their results. The available Git history
does not independently establish an immutable pre-run source revision. Recorded source/artifact
hashes and the historical registration caveat are preserved; rewriting does not improve that
historical evidence retroactively.

Before editing, a complete Git bundle was created outside the repository, hashed, marked read-only,
verified with `git bundle verify`, restored to a separate bare repository, and checked with
`git fsck --full`. The restored HEAD matched exactly. The bundle, raw artifacts, private redaction
map and publication receipt are retained locally and must never be uploaded. The receipt records
the old/new commit mapping and remote SHAs after publication; this document cannot contain its own
final commit SHA without changing that SHA.

Rule **G0-PATH-REDACTION-1** was defined before application. It replaces only the operator account
component in `/home/<redacted>/...` and `/mnt/c/Users/<redacted>/...` with the literal token
`<redacted>`. No replacement username is invented. Synthetic lab user `helmlab`, technical versions,
filesystem/device identifiers, file suffixes and failure text are retained.

In each affected JSON, only `/checks/Q4_reflink_support/stderr` and `/filesystem/path` have
redacted values; `publication_redaction` records the transformation and original raw SHA-256.
All other experimental values are unchanged. The two files each contained three path occurrences,
repeated in all three unpublished commit trees. Their publication forms are **not byte-identical**
to the raw artifacts. [The manifest](evidence/G0-publication-redaction-2026-09-07.json) records full
raw and publication SHA-256 identities and byte lengths:

| Artifact | Raw SHA-256 (private bytes) | Publication SHA-256 (tracked bytes) |
|---|---|---|
| [G0-3a ext4](evidence/G0-3-snapshot-semantics-ext4.json) | `c9b8c1d4fb9eb6ae2d29594d9a8dc84e3fb988a2f1cb5dac1c478f0b64fc5f48` | `0283a7069535087671a559c96f234e5a8daf1d66d86ef2f314b8b23ffdbb53ea` |
| [G0-3a drvfs](evidence/G0-3-snapshot-semantics-drvfs.json) | `85b2f81f9f3c6fdc7beb2902e3a45c9d83b304ab00a9ec45f09312a1818dd7e5` | `63d123f3613bb47969ed262303c2837344f8862396ecdcd1e8731b3422a94492` |

The unchanged vanilla and G0-3b JSON digests above retain their original meaning. No raw failing
observation was deleted, no probe source changed, and no acceptance criterion was revised. This
work involves one agent, no subagents, no new packages and no new experiment execution. It makes
no automation-cost or application-compatibility claim.

**Preserved publication tooling failure:** the first local rewrite reached its final cleanliness
assertion and failed because the two newly generated metadata JSON files had Windows CRLF line
endings, while Git compared normalized LF text. Only those new metadata files were normalized;
the redacted experiment artifacts and their published SHA-256 values were unaffected. The failed
local attempt and correction are retained in the private receipt. This was a repository tooling
correction, not a retry of any experiment.
