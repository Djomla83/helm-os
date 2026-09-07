# Gate 0 lab — manifest, runbook and teardown

| Field | Value |
|---|---|
| Lab id | `helm-lab-g0` |
| Purpose | Unblock the Wine-dependent *mechanics* checks of [EXP-009](EXP-009.md) Gate 0 |
| Authorised by | Maintainer instruction, 2026-09-07, bounded lab phase |
| Kind | WSL2 distribution (**fallback** option — see [§1](#1-why-wsl-and-not-a-vm)) |
| Disposable | Yes. Teardown in [§6](#6-teardown) removes it completely. |

**Current boundary, 2026-09-07:** the owner-authorised G0-2 completion is finished. The original
lab and two authorised clones are retained and stopped. The original provisioning account below
is historical; [completion state](#completion-state) records the additions and measurements.
No full desktop VM or application PoC has been started. A subsequently authorised desktop
application baseline is prepared but blocked on Hyper-V access; see [§7](#desktop-baseline).

---

## 1. Why WSL and not a VM

The authorisation preferred a disposable Ubuntu 24.04 desktop VM created with an *already installed
and working* virtualization tool. Read-only inventory established that this is not available to
this account:

| Check | Result |
|---|---|
| VirtualBox, VMware, QEMU on PATH or in Program Files | **Absent** |
| Hyper-V PowerShell module | Present |
| Hyper-V service `vmms` | Running — so the role is enabled |
| `Get-VM` | **Access denied**: "You do not have the required permission to complete this task." |
| Current process is administrator | **No** |
| `Hyper-V Administrators` group membership | Query returned nothing for this account |

Creating a Hyper-V VM therefore requires a **host administrator change**, which this authorisation
explicitly excludes. Per the stop rule, that operation was not attempted.

> **Single approval that would unlock the preferred VM path:** add the maintainer's account to the
> local **`Hyper-V Administrators`** group (an administrator action, and it requires signing out and
> back in to take effect). Nothing else about the plan changes. No hypervisor install, no Windows
> feature enablement and no reboot beyond that re-login is needed, because the Hyper-V role is
> already enabled and running.

The authorised fallback was used instead, and **only one environment was created**.

## 2. What was provisioned, exactly

| Property | Value |
|---|---|
| Distribution | `Ubuntu-24.04` from the existing WSL catalogue |
| Distro name | `helm-lab-g0` (new; `Ubuntu` untouched) |
| Location | `C:\Users\<user>\helm-lab\g0` (new directory; did not previously exist) |
| Virtual disk cap | 16 GB, sparse — conservative; actual usage is far lower |
| Expected disk usage | ~4–6 GB after packages |
| CPU / memory | **Inherited from the existing WSL2 utility VM. Not changed.** Global `.wslconfig` was deliberately **not** created or modified. |
| Network | Default WSL NAT, outbound only. No port forwarding, no inbound exposure, no bridged adapter. |
| Lab user | `helmlab`, unprivileged, `sudo` used only for package installation |

### 2.1. Exact host-side changes

Complete list. Nothing else on the host was modified.

1. Created directory `C:\Users\<user>\helm-lab\g0`.
2. Registered one new WSL distribution named `helm-lab-g0`, whose `ext4.vhdx` lives in that
   directory.
3. Nothing else. No global WSL configuration, no change to the existing `Ubuntu` distribution, no
   Windows feature, no driver, no firewall rule, no service, no registry change, no reboot.

### 2.2. Isolation applied inside the lab

Configured in `/etc/wsl.conf` **of this distribution only**, before any test ran:

```ini
[automount]
enabled = false          # intended to disable Windows-drive automount
[interop]
enabled = false          # intended to disable Windows-process interoperability
appendWindowsPath = false
[user]
default = helmlab
```

Applied with `wsl --terminate helm-lab-g0` — **never** a global `wsl --shutdown`.

The original report claimed that no personal folder was mounted, no browser session or credential
was reachable, and no Docker or SSH-agent socket was exposed. Those broad reachability statements
are historical claims, not a verified containment boundary.

<a id="interop-observation"></a>

### 2.3. Read-only publication observation, 2026-09-07

[Captured commands and output](evidence/G0-provenance-observation-2026-09-07.json) confirm that
`/etc/wsl.conf` contains the configuration above, but
`/proc/sys/fs/binfmt_misc/WSLInterop` reports `enabled` with interpreter `/init`. Effective blocking
of Windows-process execution is **UNVERIFIED** until a direct negative execution test is performed.
No such test was performed, and no configuration was changed during this publication task.

`/mnt/c` exists as a directory and is not a mountpoint (`mountpoint -q /mnt/c` returned 32).
The earlier assertion that the directory was absent was too strong. Configuration, registration
state and mount observations are reported separately; none establishes general containment.

## 3. Reproduction

### G0-2 completion prerequisite, registered 2026-09-07 at 14:37:38 UTC

The owner authorised completion of G0-2 from public baseline
`ca0aa5ae26a1d3b630f63eda8f87626d125248d1`, conditional on a direct negative interoperability
test. Main, origin/main and HEAD matched that SHA with a clean tree after fetching; documentation
validation and all 20 validator tests passed before changes.

Before provisioning or executing a runtime arm, copy the existing Windows system `cmd.exe` bytes
from `C:\Windows\System32\cmd.exe` through standard input to
`/home/helmlab/g0/interop-negative-20260907/cmd.exe`. Verify the SHA-256 against the source and
record its PE signature, file version and size. The executable remains outside Git. This avoids
mistaking a missing `/mnt/c` executable for an execution block. No Windows directory is mounted.

Run once as `helmlab`, directly through Linux process execution, without Wine or any environment
override, from the diagnostic directory:

```text
/home/helmlab/g0/interop-negative-20260907/cmd.exe /d /c echo HELM_G0_INTEROP_NEGATIVE_TEST
```

Capture exact arguments, exit code, stdout/stderr, timing, `/etc/wsl.conf`, binfmt registration,
mount observations and `WSL_INTEROP`. A returned marker establishes that Windows execution is
available and triggers the owner's STOP rule. An explicit interoperability refusal supports only
the tested execution-blocking property. Missing files, invalid binaries, an unrelated startup
failure or timeout cannot establish blocking; an ambiguous result also prevents provisioning.
The test has a 20-second timeout and no automatic retry. `/d` suppresses cmd AutoRun commands.
Neither configuration nor the interop environment will be enabled to make this test run.

The [capture script](../../tools/gate0/probe_wsl_interop.py) preserves the observation without
automatically treating a nonzero process exit as proof of containment. This is not G0-5 or a
sandbox certification. Runtime control definitions and runtime provisioning follow only if this
prerequisite is satisfied.

### G0-2 additional distribution plan, before creation

The direct [negative execution record](evidence/G0-2-wsl-interop-negative-2026-09-07.json) returned
exit 1, empty stdout and `UtilAcceptVsock:271: accept4 failed 110` after 10.012 seconds. This is
an interop connection failure, not an executable-not-found error; the transferred AMD64 PE matched
the SHA-256 of the Windows system file. The fixed marker was absent. The tested Windows-process
launch was unavailable with configuration unchanged; broader containment remains unreviewed.

At 14:39 UTC, available host disk space was 132,701,827,072 bytes (about 123.6 GiB).
The existing lab VHD occupied 11,848,908,800 bytes; its filesystem had about 4.64 GB free.
No earlier base image exists in the project lab area, so use a distribution-scoped termination
and a VHD export of this retained Ubuntu 24.04.4/Wine 11.17 lab as the common baseline. Preserve
the original distribution, and hash the exported image privately outside Git.

| Planned state | Host location (public path token) | Conservative storage allowance |
|---|---|---|
| Common retained baseline export | `C:\Users\<redacted>\helm-lab\g0-completion\baseline.vhdx` | 12 GiB |
| Staging distribution `helm-lab-g0-staging` | `C:\Users\<redacted>\helm-lab\g0-staging` | 16 GiB |
| Proton distribution `helm-lab-g0-proton` | `C:\Users\<redacted>\helm-lab\g0-proton` | 32 GiB |

Total additional allowance: 60 GiB; projected remaining host space about 63.6 GiB, above the
40 GiB safety reserve. Stop provisioning if measured free space or projected growth breaches that
reserve. These are storage estimates, not measurements. Images are sparse; runtime downloads and
prefixes stay within the respective lab disks. No global `.wslconfig` is changed. Do not run
`wsl --shutdown`. Before package installation in each clone, repeat only the same interoperability
prerequisite, preserving each separate record. Imported isolation configuration must match.

The owner permits these two extra distributions solely to avoid modifying the vanilla runtime.
The common baseline retains historical evidence. Fresh completion prefixes keep the old prefix
separate. Runtime setup differences and disk growth must be recorded per arm. The original
single-lab reproduction instructions below remain historical context.

Run from PowerShell on a machine with WSL 2 already installed. Adjust the location if desired.

```powershell
# 1. Create the lab directory (must not already exist)
New-Item -ItemType Directory -Path C:\Users\<you>\helm-lab\g0

# 2. Install a separately named distribution; do not launch it yet
wsl.exe --install Ubuntu-24.04 --name helm-lab-g0 --location C:\Users\<you>\helm-lab\g0 `
        --vhd-size 16GB --no-launch

# 3. Create the unprivileged lab user and write the isolation config, then restart THIS distro only
wsl.exe -d helm-lab-g0 -u root -- bash -lc 'useradd -m -s /bin/bash helmlab && printf "%s\n" \
  "[automount]" "enabled = false" "[interop]" "enabled = false" "appendWindowsPath = false" \
  "[user]" "default = helmlab" > /etc/wsl.conf'
wsl.exe --terminate helm-lab-g0
```

Package installation and the checks themselves are recorded in
[EXP-009-GATE0-REPORT.md](EXP-009-GATE0-REPORT.md) with exact commands and captured versions.

## 4. Scope of validity of any result from this lab

Binding on how results may be reported:

- Results are valid **only for this WSL2 configuration**, on kernel `6.6.87.2-microsoft-standard-WSL2`.
- They **must not** be generalised to bare-metal graphics, device support, battery life, or full
  physical-machine sleep and resume behaviour.
- The kernel is 6.6, so kernel-synchronisation behaviour requiring 6.14 or later cannot be exercised.
- There is no desktop environment, portal backend or tray host, so **no desktop-integration claim can
  be tested here at all.**
- **Hostile sandbox-escape testing is out of scope and stays BLOCKED**, because this lab's outer
  containment has not been reviewed. G0-5 is not attempted here.

## 5. Safety rules observed

- Trusted distribution packages and synthetic data only.
- Probes run as the unprivileged `helmlab` user.
- No installer, disk image, secret, credential or personal data is uploaded or committed.
- This lab is never attached as a runner for untrusted pull requests.
- The existing `Ubuntu` distribution is not modified in any way.

The previous publication-only boundary was superseded by explicit owner authorisation for G0-2
completion and up to two disposable runtime clones. That work now stops at Gate 0 review. No
further experiment, teardown, Hyper-V permission change or larger PoC is authorised by this
report. Original G0-3 remains BLOCKED; G0-3b's recorded PASS is limited to its separate mechanics
subtest. See the [current report](EXP-009-GATE0-REPORT.md#current-assessment).

<a id="completion-state"></a>

### 5.1. Retained G0-2 completion state

| Distribution | Runtime used | Final VHD bytes | State |
|---|---|---|---|
| `helm-lab-g0` | WineHQ devel `11.17~noble-1` | 13,805,551,616 | Stopped, preserved |
| `helm-lab-g0-staging` | WineHQ staging `11.16~noble-1` | 16,456,351,744 | Stopped, preserved |
| `helm-lab-g0-proton` | UMU 1.4.4 / UMU-Proton-10.0-4 / sniper `3.0.20260805.254768` | 16,088,301,568 | Stopped, preserved |

The [common export/import record](evidence/g0-2-completion-2026-09-07/clone-provisioning.json)
retains the initial sharing-violation failure and the justified provisioning retry. The successful
sequence used distribution-scoped termination, `wsl --export ... --format vhd`, two
`wsl --import ... --vhd` operations and `wsl --manage helm-lab-g0-proton --resize 32GB`.
The original distribution was preserved; no global WSL shutdown or configuration change occurred.
The export occupies 11,825,840,128 bytes outside Git, at the planned private lab location.

Each clone inherited the original config and was checked before package installation using the
same Windows-executable negative test. The capture script's distro metadata was changed from the
original constant to `WSL_DISTRO_NAME` before clone tests; its execution semantics were unchanged.
All three attempts returned interop connection failure/exit 1 with no marker. This supersedes the
earlier UNVERIFIED execution observation for that specific invocation only. It does not certify
the lab as a sandbox.

All work files are under `/home/helmlab/g0/completion-20260907` in each lab. Historical artifacts
and prefixes remain under their original paths. The frozen target and control executables have
identical hashes across runtimes. Control compilation and baseline dependencies are in the
[build record](evidence/G0-2-build-baseline-2026-09-07.json). The
[artifact index](evidence/g0-2-completion-2026-09-07/INDEX.md) links exact package commands, prefix
setup, invocation specs and stdout/stderr. These scripts and specs describe this bounded experiment,
not a general runtime-management interface.

Staging installed explicitly pinned `winehq-staging`, `wine-staging`, `wine-staging-amd64` and
`wine-staging-i386:i386`, all `11.16~noble-1`, from the existing WineHQ noble repository, with
`--no-install-recommends`. The absolute loader path selected staging even though the baseline's
devel runtime files remained in the clone. Package versions and hashes are in
[runtime identities](evidence/g0-2-completion-2026-09-07/runtime-identities.json).

Proton's launcher package and both runtime archives were pinned and SHA-256-checked before use.
The Steam runtime was extracted manually from the versioned archive, verified with upstream
`pv-verify`, then marked installed using UMU's helper. `UMU_RUNTIME_UPDATE=0` prevents replacement;
`GAMEID=umu-default` and `PROTONFIXES_DISABLE=1` exclude application fixes. Logs confirm fix execution
was skipped. Temporary locale generation, bundled Mono registration and other Proton defaults
are recorded runtime differences. Do not interpret those defaults as a clean upstream-Wine build.

The first Proton sequence lost console JSON and remains INCONCLUSIVE. The documented
`proton-dos-path` repetition used a new `prefix-dos-path` and the same binaries through Wine's
existing `Z:` mapping of the lab's Linux filesystem. This is not a Windows-drive mount or enabled
Windows interoperability. It supplied correct control output and `E_NOTIMPL`; no further runtime
repeat is planned. The complete comparison is in the [report](EXP-009-GATE0-REPORT.md#g0-2-completion).

Final additional VHD allocation was 46,327,136,256 bytes. Host free space was 86,622,150,656 bytes,
above the 40 GiB safety reserve. Staging's filesystem has only 732,360,704 bytes free; do not add
software there without another storage review. The
[final state record](evidence/g0-2-completion-2026-09-07/final-lab-state.json) confirms all three
project labs and the normal Ubuntu distro stopped. Normal Ubuntu was never entered or modified.

Original raw evidence with private host/account identifiers is archived outside the repository.
The [publication manifest](evidence/g0-2-completion-2026-09-07/publication-redactions.json) maps raw
and redacted identities. Images, runtime archives and executables are never committed. A narrow
Git attribute preserves capture bytes, including CRLF stdout, without changing other file rules.

## 6. Teardown

No teardown was executed or authorised in this completion task. The commands below are the
original lab's teardown reference; do not apply them automatically to the retained comparison
state. Preserve the private raw evidence and reviewed provenance before any later owner-approved
removal. The two clone names and locations are listed above.

The original runbook said the report's evidence was committed, "so teardown loses nothing".
That statement covers only the publication files. It does not cover retained prefixes, runtime
artifacts or private raw evidence. Copy and verify required state **before** any authorised teardown.

```powershell
wsl.exe --terminate helm-lab-g0
wsl.exe --unregister helm-lab-g0          # deletes the distribution and its ext4.vhdx
Remove-Item -Recurse -Force C:\Users\<you>\helm-lab\g0
```

Verify with `wsl.exe --list --verbose` that only the original distributions remain.

<a id="desktop-baseline"></a>

## 7. Separately authorised A0-7ZIP desktop VM — preparation only

The owner accepted G0-2 only as the controlled HRESULT comparison and authorised one Ubuntu
24.04 desktop VM / Windows x64 7-Zip baseline. The [definition](EXP-009.md#application-baseline)
and [application report](EXP-009-APP-BASELINE-REPORT.md) govern this separate subtask. None of the
other Gate 0 checks, architecture acceptance or the larger PoC is authorised.

### 7.1. Actual permission blocker and owner-only action

The [preflight](evidence/app-baseline-2026-09-07/hyperv-preflight.json) observed the Hyper-V
module `2.0.0.0` and running `vmms`, but an unelevated token without group SID `S-1-5-32-578`.
The local Hyper-V Administrators group was empty. Both `Get-VM` and `Get-VMSwitch` returned
permission errors. The known blocker remains; no VM creation was attempted.

A private `owner-hyperv-action.ps1` outside Git contains the exact current local account SID,
resolved with `Get-LocalUser` while preparing the action. The engineering agent has **not** run it.
The owner should inspect its text, then perform its command in a separate elevated 64-bit
PowerShell. Public form, with the private account component deliberately omitted:

```powershell
$helmOwner = Get-LocalUser -SID '<owner-account-SID>' -ErrorAction Stop
Add-LocalGroupMember -SID 'S-1-5-32-578' -Member $helmOwner -ErrorAction Stop
```

This adds only the selected account to **Hyper-V Administrators**, not local Administrators.
However, that group grants general access to Hyper-V features, **not only the HELM VM**.
[Microsoft's group documentation](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-groups#hyper-v-administrators)
and [the cmdlet reference](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.localaccounts/add-localgroupmember?view=powershell-5.1)
describe the scope and SID-based operation. Targeting the recorded SID avoids accidentally adding
the administrator account used for elevation.

After the owner performs this action, save work and sign out/in to obtain a fresh user token.
Reopen the coding-agent session normally; do not run it elevated. No host restart, bypass,
security-policy change or feature installation is part of the action. Recheck effective token
membership, `Get-VM` and `Get-VMSwitch` before provisioning. Permission is not inferred from an
owner message, membership alone, or elapsed time. If unavailable, stop at preparation.

### 7.2. Proposed configuration and capacity — not provisioned

| Property | Fixed plan |
|---|---|
| VM name | `helm-lab-desktop-7zip`, one new Generation 2 VM; first verify no name collision |
| Guest | Official Ubuntu `24.04.4` LTS desktop amd64; a lab choice, not HELM's final base OS |
| CPU / RAM | 4 vCPUs; fixed 8 GiB, dynamic memory off |
| Disk | One new dynamic VHDX capped at 32 GiB; no automatic growth of that cap |
| Project directory | `D:\helm-lab\exp009-a0-7zip` (did not exist at inspection) |
| Project subdirectories | `vm` for VM configuration/runtime-state files; `vm\os.vhdx`; `downloads` for ISO; `transfer` for synthetic artifacts |
| Firmware | Secure Boot stays on; use the guest-specific `MicrosoftUEFICertificateAuthority` template. Stop on unresolved boot failure; never disable host or guest Secure Boot to get a pass. |
| Checkpoints / automatic actions | Disable automatic checkpoints and checkpoint creation; automatic start Nothing; automatic stop ShutDown. No saved-state chain, clone or rollback. |
| Network | Select an existing suitable switch after permissions permit inspection. Prefer an existing suitable Default Switch; do not presume its presence. No new switch, forwarding or host firewall change. |
| Display / access | Basic VMConnect console initially; actual guest desktop/session/rendering and automation interface must be measured. No enhanced-session drive, clipboard or device redirection. |
| Account | Dedicated guest `helmlab`; ordinary desktop/application processes unprivileged |

[Storage observations](evidence/app-baseline-2026-09-07/storage-observation.json) found
573,119,229,952 bytes free on D: (about 533.76 GiB), and 86,528,421,888 on C: (80.58 GiB).
The host has 16 physical / 32 logical CPU cores and about 42.80 GiB free physical RAM at preflight.
The proposed CPU/RAM allocation is feasible by these observations, subject to recheck before use.

Reserve **50 GiB on D:** for 32 GiB disk growth, the 6,655,619,072-byte ISO (6.20 GiB), an 8 GiB
runtime-state allowance, and remaining space for metadata, temporary downloads and transfers.
Guest package storage is inside the 32 GiB disk cap; a separate download allowance remains in the
50 GiB host budget. No full ISO duplicate, saved-state copy or checkpoint is budgeted or authorised.
That leaves about 483.76 GiB on D:; maintain at least **40 GiB on both C: and D:** throughout.
Keep VM configuration/runtime-state locations on D:, not their default location on C:.

The initial C:-only estimate omitted runtime-state allowance and was corrected before provisioning.
[Microsoft documents RAM-sized runtime-state files for Hyper-V 2016](https://support.microsoft.com/en-us/servicing/os/windows-server/2017/12/hyper-v-vss-backs-up-a-large-vmrs-file-when-you-back-up-a-virtual-machine).
An 8 GiB allowance here is conservative planning, not a measurement of this uncreated Windows 11 VM.
Current allocation is **zero**. Do not shrink allocations, enlarge disks, delete prior labs/evidence,
or alter host paging to make a plan fit. Stop and report if measured growth invalidates the reserve.

### 7.3. Guest setup, transfer and evidence collection

Download the [pinned official ISO and runtime](evidence/app-baseline-2026-09-07/artifact-pins.json)
only after permission and capacity checks. Verify complete artifact bytes before use. The small
7-Zip installer already downloaded privately can be transferred or downloaded again at the exact
pin; neither route permits executing it on the Windows host. Guest package dependencies and all
setup commands must be captured, including failures. All required packages are installed inside
the VM only. Do not start any of the three WSL project labs or enter normal Ubuntu.

Obtain project source/fixtures over HTTPS at the committed definition revision, or use a
project-only artifact transfer. If SSH/SCP is needed, use a new guest-specific credential, isolated
known-hosts file and explicit project paths; disable agent forwarding and inherited SSH config.
Never expose an existing host key, SSH agent, browser, personal folder or Docker socket. No host
drive mount or clipboard bridge is authorised. Return only this experiment's outputs and logs.

Record `loginctl` session type, `XDG_SESSION_TYPE`, desktop versions, active display server,
`DISPLAY`/`WAYLAND_DISPLAY`, Wine graphics backend, renderer, resolution and scaling before each
workflow. If the session is X11, XWayland or remote desktop, label it accurately; do not infer
Wayland console coverage from the installed GNOME package. Record basic VMConnect versus any
guest automation tool. If GUI automation is unavailable, use an explicitly human-assisted run
with measured actions/attended time or report BLOCKED; a CLI run cannot substitute.

The [small helper](../../tools/app_baseline.py) reuses the existing command-capture function and
preserves stdout/stderr bytes as base64. Example syntax, to be executed **inside the guest** after
setup, using a new record filename each time:

```bash
python3 tools/app_baseline.py capture --cwd /home/helmlab/exp009-a0-7zip --timeout 600 install.json -- env WINEPREFIX=/home/helmlab/exp009-a0-7zip/prefix WINEARCH=win64 WINEDLLOVERRIDES=mscoree,mshtml= /opt/wine-devel/bin/wine downloads/7z2603-x64.exe
```

Prefix initialisation, package inventory, installed-file identities, GUI actions and boot IDs
need their own records. This command is a capture aid, not a claim that installation ran. Preserve
both workflow destinations and all failed attempts. Archive verification decompresses members in
memory against the frozen manifest; screenshots and zero process exit codes are insufficient.
Use the existing scoped byte-preserving Git attributes and distinguish redacted public artifacts
from private raw identities. Finish at the application-baseline review; no teardown or larger PoC.
