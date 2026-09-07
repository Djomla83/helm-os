# EXP-009 A0-7ZIP — desktop application baseline preparation

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
