# EXP-009 A0-7ZIP — desktop application baseline

<a id="current-assessment"></a>

## Current execution assessment — 2026-09-07

**Overall A0-7ZIP: FAIL against the registered two-workflow protocol.** The first GUI attempt
created a correct ZIP at the wrong destination because the agent submitted the Add dialog too
early. The second, originally planned post-restart attempt satisfied its GUI and output checks.
The first failure was not retried, moved to the expected path or replaced by the second result.
This is an automation/protocol failure, not evidence that 7-Zip is incompatible with Wine.

The owner-authorised continuation began at clean, fetched
`main == origin/main == 7b9ec6f91d33d44578179da467cb01f4da55ea2c`; no newer work was present.
The definition/helper commit remains `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`, published before
application installation. Work stayed on `experiment/exp009-app-baseline`; no history was rewritten.
The [definition](EXP-009.md#application-baseline) and all expectations remain unchanged.
The [structured result](evidence/app-baseline-execution-2026-09-07/result-summary.json) and
[evidence index](evidence/app-baseline-execution-2026-09-07/INDEX.md) distinguish each dimension.

### Effective permission, host changes and VM

The [permission capture](evidence/app-baseline-execution-2026-09-07/hyperv-access.json) observed
an unelevated token, effective Hyper-V Administrators SID `S-1-5-32-578`, successful `Get-VM`
(zero VMs before provisioning), and successful `Get-VMSwitch`. The owner reports personally
performing the previously prepared group action; the agent changed no membership or elevation.
This group grants Hyper-V access generally, not only access to the HELM VM.

| Property | Executed configuration |
|---|---|
| Host | Windows 11 Pro `10.0.26200`; Ryzen 9 5950X, 16 physical/32 logical cores; about 64 GiB RAM |
| VM | `helm-lab-desktop-7zip`, ID `bd39f424-5b26-479d-846f-2f28f6639637`; Generation 2, configuration version 12.0 |
| CPU / memory | 4 vCPUs; fixed 8 GiB, dynamic memory disabled |
| Disk | One dynamic VHDX, `D:\helm-lab\exp009-a0-7zip\vm\os.vhdx`, cap 32 GiB; EFI FAT32 and root ext4 |
| Firmware / network | Secure Boot On, `MicrosoftUEFICertificateAuthority`; existing `Default Switch` |
| Lifecycle | Checkpoint type Disabled, automatic checkpoints false, zero checkpoints; automatic start Nothing, stop ShutDown |
| Access | VM-scoped Hyper-V synthetic keyboard/mouse and framebuffer APIs; project-only SSH/SCP with a new restricted key and verified host fingerprint |
| Accounts | `helmsetup` for guest package administration; `helmlab`, UID/GID 1001 and no supplementary groups, for the application |
| Final state | VM Off after normal guest poweroff; ISO automatically ejected by the installer and retained on disk; all three project WSL labs and normal Ubuntu stopped |

The [capacity preflight](evidence/app-baseline-execution-2026-09-07/capacity-preflight.json),
[creation record](evidence/app-baseline-execution-2026-09-07/vm-created.json), and
[final inventory](evidence/app-baseline-execution-2026-09-07/host-final-state.json) record the actual
allocation. Host changes were this one VM and its D: project files, private evidence/project key
files, and repository changes. No new switch, port forward, host folder/device/clipboard redirection,
GPU passthrough, driver, Windows feature, global WSL setting or host-security change was made.
No Windows installer ran on the host. Host boot time remained 2026-09-04; only the guest restarted.
Default integration services were retained, with Guest Service Interface disabled.

### Frozen artifacts and actual desktop

Ubuntu `24.04.4 LTS desktop amd64`, media build `20260210`, was downloaded from the official release
route: 6,655,619,072 bytes, SHA-256
`3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e`.
The full ISO hash and checksum signature were verified before boot; Ubuntu signing fingerprint
`843938DF228D22F7B3742BC0D94AA3F0EFE21092` matched the official verification instructions.
[Download](evidence/app-baseline-execution-2026-09-07/iso-download.json) and
[signature evidence](evidence/app-baseline-execution-2026-09-07/ubuntu-checksum-signature.json)
are preserved. This lab choice does not select HELM's base OS.

WineHQ vanilla used noble/main at `https://dl.winehq.org/wine-builds/ubuntu`:
`winehq-devel`, `wine-devel`, `wine-devel-amd64` and `wine-devel-i386:i386`, all **11.17~noble-1**.
All four bodies matched the [previously committed pins](evidence/app-baseline-2026-09-07/artifact-pins.json)
before installation. APT recorded a trusted signature from
`D43F640145369C51D786DDEA76F1A20FF987672F`. The loader reported `wine-11.17`, SHA-256
`d96472c8c5e6567bb9d800b1c2261c764a1588d7913d50ef3474357e5f549f33`; the four packages were held.
[Full provisioning output](evidence/app-baseline-execution-2026-09-07/guest-wine-provision.json),
[actual setup script](evidence/app-baseline-execution-2026-09-07/scripts/provision-wine.sh), and
[dependency changes](evidence/app-baseline-execution-2026-09-07/guest/package-differences.json)
record 240 added and seven changed packages across Wine provisioning, including prerequisite tools.
The before inventory already included Ubuntu installer updates and SSH setup.

The official Windows x64 7-Zip **26.03** installer was reused from preparation and hash-checked
on the host and in the guest before execution: 1,661,239 bytes, SHA-256
`0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd`.
Its standard GUI installed into `C:\Program Files\7-Zip` in the fresh application prefix.

| Installed file | Bytes | SHA-256, unchanged after R1 |
|---|---:|---|
| `7zFM.exe` | 1,003,520 | `7f7067b2264fbf8cbd1348cf7f41a0e35c928de750d53533c963ebe334dbd612` |
| `7z.exe` | 577,536 | `6ee3c0ed0b27663c1b948ae85a7c0bb073aed1498983182f3f0df1f6a8c30b2f` |
| `7z.dll` | 1,906,688 | `65e4c1f855f9ef6e8f0f5df8e3f27d9eb5f07311408639da0a1ca0b8f4871b0d` |

All three PE machine values were `0x8664`; file/product versions were `26.03`.
[Before](evidence/app-baseline-execution-2026-09-07/guest/records/installed-before.json) and
[after](evidence/app-baseline-execution-2026-09-07/guest/records/installed-after.json) records retain
the complete resource strings. The CLI executable was identified, not used to create either ZIP.

Actual guest kernel was `7.0.0-31-generic` after normal Ubuntu installation updates. Both GUI runs
used a local GNOME Shell 46.0 Wayland console session, 1024×768, one monitor, scale 1.0. Wine's
`7zFM.exe` mapped **winex11.so/winex11.drv**, so application coverage is **XWayland**, not Wine's
native Wayland driver. GLX reported software `llvmpipe (LLVM 20.1.2, 256 bits)`, Mesa
`25.2.8-0ubuntu0.24.04.2`, no acceleration. XWayland root depth 24 was queried only after R1;
before-R1 colour depth was not separately captured. RGB565 is the thumbnail format, not proof of
desktop depth. [Before/after environments and backend records](evidence/app-baseline-execution-2026-09-07/INDEX.md)
preserve these distinctions. No remote desktop session or forwarding was used.

The registered `WINEPREFIX=/home/helmlab/exp009-a0-7zip/prefix`, `WINEARCH=win64`,
`WINEDLLOVERRIDES=mscoree,mshtml=` and `/opt/wine-devel/bin/wine` remained fixed. The same prefix
and installed application survived R1. No winetricks, staging, Proton, switching, rollback or
recovery was applied. The prefix is application state; the VM is the trusted-software lab boundary.

### Controls, workflow outcomes and coverage

| Check | Verdict | Observed evidence |
|---|---|---|
| Guest oracle validity H0, before and after R1 | PASS | Good ZIP accepted / exit 0; valid-CRC seeded content corruption rejected / exit 1 in both sequences |
| Guest helper unit checks | PASS | All 15 tests passed under Python 3.12.3 before application installation |
| Install A1 / identities A2 | PASS | Ordinary installer GUI completed, exit 0; all three installed x64 26.03 identities verified |
| First GUI workflow W1 | FAIL | Agent's format-selection Enter prematurely submitted Add, producing `inputs/fixture.zip` instead of the required destination |
| Registered output V1 | FAIL | `outputs/before-restart/workflow.zip` absent; verifier exit 1, preserved FileNotFoundError |
| Supplementary check of W1's accidental ZIP | PASS | Correct expected paths and all decompressed bytes; does not satisfy W1/V1 |
| Guest restart R1 | PASS | Normal `systemctl reboot`; boot ID changed from `2d64c822-205d-4bfc-bdaa-f21e50cd38bc` to `77fc6b8f-d5c2-4fec-ba14-d3c680d2e0cb` |
| Second GUI workflow W2 | PASS | Existing GUI relaunched; fixture selected; ZIP / Deflate / Normal, empty passwords and correct fresh destination inspected before clicking OK |
| Second independent output V2 | PASS | `outputs/after-restart/workflow.zip` contains every expected entry and exact file bytes, with no extra/duplicate/encrypted entries |
| Overall registered A0-7ZIP | FAIL | Two planned attempts executed; only one fully satisfied its workflow expectation; no reliability-rate estimate |

The six files cover text, binary, empty content, nesting, spaces and non-ASCII names; the empty
directory is also required. Both content-verified ZIPs are 2,282 bytes with SHA-256
`8f7b7192808b03275239785bc561127e4a1e3cee5edb8471bfff1d33703356e9`. Their equal archive hashes
are not the correctness oracle: the verifier checked decompressed paths, sizes and content hashes.
See the [failed registered V1](evidence/app-baseline-execution-2026-09-07/guest/records/verify-before-required.json),
[supplementary V1](evidence/app-baseline-execution-2026-09-07/guest/records/verify-before-accidental.json),
and [V2](evidence/app-baseline-execution-2026-09-07/guest/records/verify-after.json).

The [W1 deviation](evidence/app-baseline-execution-2026-09-07/workflow-w1-deviation.json) was recorded
before R1/W2. No first-attempt repeat was added. The second attempt set the destination first,
inspected the [complete settings](evidence/app-baseline-execution-2026-09-07/screens/ubuntu-174-w2-settings.png),
and clicked OK explicitly. [Console actions](evidence/app-baseline-execution-2026-09-07/console-actions.json)
and process captures establish GUI provenance separately from output validity. Control ZIP hashes
differ from the earlier Windows-host controls because of creator-platform metadata; their expected
names/bytes and outcomes are unchanged, as recorded in the
[metadata comparison](evidence/app-baseline-execution-2026-09-07/control-archive-platform-observation.json).

### Failures, effort and limits

The [execution annotations](evidence/app-baseline-execution-2026-09-07/execution-annotations.json)
preserve the native computer-use pipe failure, initial framebuffer-copy CLR crash, dropped text/
scancodes, GPG path error, Windows ssh-keyscan KEX error, Ubuntu setup warnings, W1 actuation error,
unprivileged APT-cache measurement denial, and evidence-assembly/inspection mistakes. Wine prefix
initialisation retained OLE `0x80004002`, RpcSs/setupapi/Common-Controls diagnostics; GUI processes
retained EGL DRI3 warnings and Wine stub messages. None was hidden by a runtime change or retry.
The original private frames/logs/scripts remain available by digest; redacted publication copies
are explicitly distinguished in the [manifest](evidence/app-baseline-execution-2026-09-07/publication-manifest.json).

[Resource accounting](evidence/app-baseline-execution-2026-09-07/resource-accounting.json): ISO
download 229.15 seconds; Ubuntu installation observed over about 33 minutes 48 seconds; Wine
provisioning 685.83 seconds; prefix initialisation 60.26 seconds; installer capture 56.13 seconds;
GUI process captures 169.75 and 191.30 seconds, including agent interaction/inspection pauses.
The ISO added 6,655,619,072 downloaded bytes; four pinned Wine bodies total 303,879,358 bytes.
APT reported about 417 MB of package downloads plus metadata/prerequisites. Ubuntu install-update
traffic and network overhead were not completely metered. The installer was reused, not downloaded
again. File-length inventory on D: is 20.175 GiB, including the 13.973 GiB VHDX; C: and D: retained
79.992 and 513.920 GiB free. NTFS allocation overhead and unrelated volume activity are not assigned
to the task. No cap increase or lab/evidence deletion was needed.

One model/agent performed console actions and analysis; no delegated agents and no human GUI
actions. One owner permission action was reported and its effect verified. Human attended time,
active engineering minutes and model billing/tokens were not measured. Session wall time includes
a roughly 25-minute pause and automated package execution; it is not human or active engineering
effort. This was agent-operated GUI work with inspection and debugging, not a fixed unattended run.

**Correctness:** the output oracle discriminated controls and both observed ZIP contents matched;
the registered two-workflow protocol failed. **Automation feasibility:** this console interface
can operate the GUI, but the premature submission demonstrates a workflow error. **User experience:**
not independently measured; one readback frame retained a stale address-field display, with no
persistence or rendering conclusion. **Economics:** no comparison showing a HELM advantage ran.

This establishes installation, concrete GUI operation and one fully satisfied post-restart ZIP
workflow for these exact components. It does not establish the complete two-workflow compatibility
claim, rendering correctness, a reliability rate, other 7-Zip features/applications, bare-metal
graphics/devices/performance, sandbox strength, or lower maintenance cost. G0-1, original G0-3,
G0-4 and G0-5 remain BLOCKED; G0-2 remains accepted only in its controlled HRESULT scope. No
architecture ADR was accepted and no HELM product module or larger PoC was started.

Repository documentation validation and all 35 tests passed (20 validator tests and 15 helper
tests). The [publication checks](evidence/app-baseline-execution-2026-09-07/INDEX.md#publication-validation)
verify evidence hashes, private/public provenance, decoded command streams, reachable history and
unchanged registered helpers/ADR statuses. These checks validate the publication, not the failed
workflow. The preserved UTF-8 audit-tool correction is a separate tooling deviation.

**Recommended owner decision:** review this failed protocol result with the successful bounded
observations preserved. One proposed first product-code task is an offline, read-only evidence-
bundle verifier for this exact workflow: check pinned hashes and required scenario/destination/
restart evidence, and refuse completeness when W1 is absent despite a valid ZIP elsewhere. Reuse
the current content oracle; add no launching, runtime selection, recovery, database or desktop
integration. This is a proposal requiring separate authorisation. Stop at A0-7ZIP review.

## Historical preparation at 7b9ec6f — retained snapshot

The following preparation report is the state before effective Hyper-V access was available.
Its present-tense descriptions and NOT_RUN/BLOCKED rows are historical and are superseded by the
execution assessment above; its failures and artifacts are retained.

| Field | Observed state |
|---|---|
| Date | 2026-09-07 |
| Executor | One primary engineering agent; no delegated agents |
| Owner authorisation | One existing-Hyper-V Ubuntu desktop VM and one Windows x64 7-Zip baseline, separately from Gate 0 and the larger PoC |
| Reviewed public starting point | `3f947888067243ba3cedcc3f77916344d9d10a25`; fetch confirmed HEAD/main/origin/main equal, clean tree; no newer work was present |
| Definition | [EXP-009 §13](EXP-009.md#application-baseline), committed with the helper/pins as `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae` before any application installation |
| Overall A0-7ZIP | **BLOCKED — Hyper-V permission is unavailable** |
| Application hypothesis | **inconclusive**: no application or guest desktop was executed |
| Owner result review | Pending for this baseline; G0-2 alone was accepted in its controlled HRESULT-comparison scope |

## What was actually observed

The existing Hyper-V module is `2.0.0.0` and `vmms` is running. The current token is not elevated,
has no Hyper-V Administrators SID, and the local group has no members. `Get-VM` and `Get-VMSwitch`
both returned permission errors. Their captured exception HRESULT is `-2146233088`; this is a
management exception, not an application or DirectComposition HRESULT. Exact commands, timestamps,
exceptions and redaction provenance are in the [preflight](evidence/app-baseline-2026-09-07/hyperv-preflight.json).

The single observed execution blocker is Hyper-V access. Switch suitability, VM creation, boot,
guest dependencies and GUI automation remain untested behind it; they are not inferred failures.
No fallback hypervisor or WSL distribution was created. All three project WSL labs and normal
Ubuntu were observed stopped and were not entered, changed or restarted.

An exact owner-specific administrator action was prepared privately and not executed. The
[runbook](LAB-G0-RUNBOOK.md#desktop-baseline) describes its public template and why Hyper-V
Administrators membership grants access to Hyper-V generally, not only the proposed VM. The
owner must review and perform the action, then obtain a fresh normal user token. The engineering
agent did not change membership, elevate its session, restart the host or alter Windows features.

## Proposed VM and pinned application inputs

No VM currently exists for this subtask. The fixed proposal is `helm-lab-desktop-7zip`, Generation 2,
4 vCPUs, fixed 8 GiB RAM, one dynamic 32 GiB VHDX and a dedicated unprivileged guest desktop account.
Keep Secure Boot enabled with the guest-specific Linux-compatible certificate template. Use an
existing suitable switch after it can be inspected; no switch or port-forward creation is allowed.

The proposed location is `D:\helm-lab\exp009-a0-7zip`, which did not exist at inspection. D: had
533.76 GiB free; C: had 80.59 GiB. A 50 GiB total D: budget includes full VHD growth, ISO, an 8 GiB
runtime-state allowance and temporary artifacts, leaving about 483.76 GiB. Preserve at least
40 GiB on both host volumes. The initial C:-only estimate omitted runtime-state files and was
corrected before provisioning. See [storage observations](evidence/app-baseline-2026-09-07/storage-observation.json)
and the [capacity plan](LAB-G0-RUNBOOK.md#desktop-baseline). Nothing was written to D:.

| Input | Identity | Actual preparation result |
|---|---|---|
| Ubuntu desktop | `24.04.4 LTS amd64`; ISO SHA-256 `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e` | Official expected checksum and 6,655,619,072-byte size pinned. ISO body not downloaded or verified; checksum signature and complete-file verification remain mandatory before boot. |
| Windows 7-Zip | `26.03`, x64 EXE, SHA-256 `0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd` | 1,661,239 bytes downloaded through 7-zip.org's project release route. Local digest matched release asset digest; AMD64 PE header checked. Never executed on the Windows host or in a guest. |
| WineHQ vanilla | Four devel packages, `11.17~noble-1`, Ubuntu noble/main | Package identities/digests frozen and four URLs returned HTTP 200. Bodies not downloaded here; signed APT metadata and package-body verification are required inside the guest. No staging selection or runtime switch. |

[Artifact pins and request records](evidence/app-baseline-2026-09-07/artifact-pins.json) preserve
sources, timestamps, sizes, hashes and the failed metadata request. Availability and expected
digests do not mean a package was installed. Installed 7-Zip executable identities remain NOT_RUN.

## Results and coverage

| Dimension / scenario | Result | Evidence boundary |
|---|---|---|
| Host helper unit checks | PASS | Synthetic verifier/capture checks; no Wine, VM or application coverage |
| Host known-good / corrupted oracle execution | PASS for host harness validity | Good ZIP accepted (exit 0); seeded corruption rejected (exit 1). This cannot substitute for guest controls |
| Guest harness controls H0 | NOT_RUN | No guest exists |
| Install A1 / installed identities A2 | NOT_RUN | No installer execution |
| GUI launch and first workflow W1 | NOT_RUN | No desktop session; 0 of 2 planned GUI workflows |
| Independent application-output check V1 | NOT_RUN | No application-created ZIP |
| Guest restart R1 | NOT_RUN | No VM exists; host not restarted |
| Post-restart workflow W2 / V2 | NOT_RUN | No second GUI execution or output |
| Overall baseline | BLOCKED | Hyper-V permission prerequisite absent |

The helper creates six synthetic files plus directory fixtures. Its independent verifier checks
exact file paths, the empty directory and every decompressed file's size/content digest. It rejects
the deliberate payload corruption even when ZIP CRCs are internally valid. It does not write
archive-controlled paths. The tests also cover an independently encoded ZIP, Deflate, duplicate
members, missing/extra paths, CRC corruption, truncation, bounded size and preserved capture bytes.
These are tests of the helper, not hostile VM probes or a certification of its security.

The [labelled host control execution](evidence/app-baseline-2026-09-07/harness-controls.json)
ran after definition commit `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`, using Windows Python 3.14.3.
Fixture preparation and the two verifications took 0.280 seconds in total. The
[good record](evidence/app-baseline-2026-09-07/oracle-good.json) is PASS; the
[corrupted record](evidence/app-baseline-2026-09-07/oracle-corrupted.json) is FAIL specifically for
the binary payload's decompressed content. Their ZIP digests and sizes are in the
[fixture record](evidence/app-baseline-2026-09-07/fixture-record.json); synthetic ZIP bodies remain
outside Git. Public captures redact only declared private path/identity fields, with separate
raw/public hashes in the [publication manifest](evidence/app-baseline-2026-09-07/publication-manifest.json).
The earlier definition checkpoint's NOT_RUN control row was its state before this labelled run;
the current row supersedes it without implying that guest controls ran.

## Failures, effort and interpretation

- The permission errors are preserved. No elevation or fallback was attempted.
- An assumed WineHQ `Packages.xz` URL returned HTTP 404. The actual Release metadata advertised
  `Packages.gz`; preparation used it and verified its hash against Release. Earlier successful
  downloads were reused with digest checks. This was metadata discovery, not an application retry.
- Python warned about a malformed entry in the Windows certificate store. TLS verification was
  not disabled and host certificates were not modified.
- A runbook patch had a context mismatch and changed no files; the subsequent patch used the
  observed file context. This is editing effort, not a test failure.
- The C:-only storage estimate was corrected to include runtime-state allowance and use D:.
  No lab or prior evidence was deleted to make room.

Tracked preparation downloads total 2,249,348 response-body bytes; additional small source/research
reads were not fully metered. ISO/package bodies and VM allocation added **zero** bytes. The
installer, downloaded metadata, private preflight and prepared owner command remain outside Git.
Model engineering includes source reading, capacity planning, fixture/oracle implementation,
privacy handling and documentation. One agent was used; model tokens/billing and human attended
minutes are unavailable. No owner-side action has been observed during preparation.

[Resource accounting](evidence/app-baseline-2026-09-07/resource-accounting.json) records the
measured preparation interval and command times separately. At 17:43 UTC, private task files
totalled 2,328,377 bytes by file length, including the installer and synthetic archives; this is
not a filesystem-allocation measurement. The
[final preparation inventory](evidence/app-baseline-2026-09-07/preparation-final-state.json)
still found absent Hyper-V membership, stopped WSL distributions and no proposed D: VM directory.
Actual host changes are repository files/Git commits plus this private preparation/archive area.
No VM, VHD, ISO, installed runtime, group, service, firewall, driver or global WSL change was made.

[Validation output](evidence/app-baseline-2026-09-07/validation.json) records passing documentation
checks, all 20 validator tests and all 15 helper tests. Evidence-link, digest, privacy and Git-index
checks accompany publication. Installers, archives, private raw records and the owner-specific
administrator command are excluded from Git. These checks do not change the BLOCKED baseline.

**Correctness:** helper tests support the bounded output oracle only. **Automation feasibility:**
Hyper-V access is blocked; GUI actuation is untested. **User experience:** unmeasured. **Economics:**
small preparation downloads and measured command times are not a HELM cost advantage.

This preparation proves neither that 7-Zip works in Wine nor that a desktop workflow survives a
guest restart. It establishes no rendering, general compatibility, bare-metal, performance,
device-support or sandbox result. A future completed baseline still would not establish that
HELM improves existing components or reduces maintenance cost. G0-1, original G0-3, G0-4 and G0-5
remain BLOCKED; G0-2 remains accepted only in its previously documented scope. Architecture ADRs
remain Proposed and the larger Evidence Loop PoC is unauthorised.

**Next action:** the owner reviews and performs the prepared Hyper-V group action, if desired;
then this same bounded baseline can resume after actual access and capacity checks. No automatic
continuation is promised. Stop at preparation while the permission prerequisite is unavailable.

**One proposed first HELM module, for a later bounded authorisation after baseline review:** an
offline, read-only evidence-bundle verifier for this one workflow. It would check pinned artifact
hashes and required installation/GUI/restart/output evidence, and refuse a completeness claim
when a required step is absent. Reuse this oracle; add no launching, runtime selection, recovery,
database or desktop integration. This is a proposal, not module implementation or ADR acceptance.
