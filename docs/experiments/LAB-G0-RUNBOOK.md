# Gate 0 lab — manifest, runbook and teardown

| Field | Value |
|---|---|
| Lab id | `helm-lab-g0` |
| Purpose | Unblock the Wine-dependent *mechanics* checks of [EXP-009](EXP-009.md) Gate 0 |
| Authorised by | Maintainer instruction, 2026-09-07, bounded lab phase |
| Kind | WSL2 distribution (**fallback** option — see [§1](#1-why-wsl-and-not-a-vm)) |
| Disposable | Yes. Teardown in [§6](#6-teardown) removes it completely. |

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

Current publication boundary: no staging or Proton/UMU provisioning, G0-2 continuation, Hyper-V
permission change, teardown, or larger PoC. Original G0-3 remains BLOCKED; G0-3b's recorded PASS is
limited to its separate mechanics subtest. See the [current report](EXP-009-GATE0-REPORT.md#current-assessment).

## 6. Teardown

Evidence must be copied out **before** teardown; the report's evidence files are already committed
to the repository, so teardown loses nothing.

```powershell
wsl.exe --terminate helm-lab-g0
wsl.exe --unregister helm-lab-g0          # deletes the distribution and its ext4.vhdx
Remove-Item -Recurse -Force C:\Users\<you>\helm-lab\g0
```

Verify with `wsl.exe --list --verbose` that only the original distributions remain.
