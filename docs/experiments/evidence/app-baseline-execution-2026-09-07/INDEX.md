# A0-7ZIP execution evidence — 2026-09-07

Public continuation baseline: `7b9ec6f91d33d44578179da467cb01f4da55ea2c`.
Immutable definition/helper: `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`.
[Current report](../../EXP-009-APP-BASELINE-REPORT.md#current-assessment) and
[structured result](result-summary.json): overall **FAIL**, because W1 missed its registered
destination after agent input error. The planned W2 passed. Neither a valid accidental ZIP nor
W2 replaces W1. Two planned attempts, two actual attempts, no workflow retry.

## Evidence map

| Subject | Records |
|---|---|
| Actual non-elevated Hyper-V access | [Permission](hyperv-access.json), [capacity](capacity-preflight.json), [creation](vm-creation-command.json), [VM properties](vm-created.json) |
| Ubuntu artifact authentication | [ISO bytes/hash/download](iso-download.json), [checksum signature](ubuntu-checksum-signature.json), [failed GPG lookup](ubuntu-key-inspect.json), [corrected lookup](ubuntu-key-inspect-relative.json) |
| Guest installation | [Selections and instrument identities](guest-install-definition.json), [OS/media inventory](guest-os-bootstrap-inventory.json), [private log inventory](guest-install-log-inventory.json) |
| Project-only SSH bootstrap | [Fingerprint comparison and preserved keyscan failure](ssh-bootstrap-verification.json), [successful keyscan](ssh-keyscan-git.json), [effective guest account/file modes](guest-ssh-initial.json) |
| Pinned vanilla Wine | [Full provisioning command/output](guest-wine-provision.json), [four package digests](guest/expected-wine-sha256.txt), [package differences](guest/package-differences.json), [before inventory](guest/packages-before.tsv), [after inventory](guest/packages-after.tsv), [downloaded package file sizes](guest/downloaded-packages.tsv), [APT index hashes](guest/apt-index-sha256.txt) |
| Unchanged test source and controls | [Source bundle identity](source-transfer-identity.json), [control execution definition](guest-controls-command-definition.json), [guest fixture identities](guest/fixture-record.json), [15 guest helper tests](guest/records/guest-helper-tests.json) |
| Before-restart controls | [Good accepted](guest/controls/before-good.json), [corruption rejected](guest/controls/before-bad.json), [full command captures](guest-controls-before.json) |
| Prefix and application installation | [Prefix initialization](guest/records/prefix-initialization.json), [installer](guest/records/7zip-installation.json), [installed PE identities](guest/records/installed-before.json) |
| Actual GUI/session provenance | [Before environment](guest/records/environment-before.json), [before scale](guest-display-scale-before.json), [Wine X11 process mappings](guest/records/wine-backend-before.json), [221 chronological console events](console-actions.json) |
| W1 failure | [GUI process stdout/stderr](guest/records/workflow-before.json), [required-path verification failure](guest/records/verify-before-required.json), [accidental ZIP content check](guest/records/verify-before-accidental.json), [deviation recorded before W2](workflow-w1-deviation.json) |
| Guest restart | [Normal restart command](guest-r1-restart.json), [new boot ID/session environment](guest-session-after-restart.json), [guest boot journal inventory](guest-admin-final-accounting.json) |
| After-restart controls/identities | [Good accepted](guest/controls/after-good.json), [corruption rejected](guest/controls/after-bad.json), [same installed hashes](guest/records/installed-after.json) |
| W2 coverage/output | [GUI process capture](guest/records/workflow-after.json), [independent content verification](guest/records/verify-after.json), [after environment](guest/records/environment-after.json), [Wine mappings](guest/records/wine-backend-after.json), [after scale](guest/records/display-scale-after.json), [after colour depth](guest-display-depth-after.json) |
| Preservation and cost | [Before archive identity](guest-before-evidence-preserve.json), [after archive identity](guest-after-evidence-preserve.json), [resource accounting](resource-accounting.json), [stopped final host/VM state](host-final-state.json) |
| Failures and limits | [All execution annotations](execution-annotations.json), [initial native transport/GPG methodology](pre-application-methodology.json), [framebuffer crash](boot-001-failure.json), [initial input failure](keyboard-input-failure.json), [batch-size observation](scancode-batch-observation.json), [control archive metadata differences](control-archive-platform-observation.json) |

## Selected GUI frames

These are derived RGB565-to-PNG diagnostic frames, not a rendering oracle. The publication
manifest records the separate raw-frame and PNG identities. No image retouching or private-host
desktop capture was used.

- Ubuntu [selected installation settings](screens/ubuntu-066-install-summary.png) and
  [completed installation](screens/ubuntu-095-install-status.png).
- 7-Zip [installer](screens/ubuntu-144-7zip-installer.png) and
  [installed confirmation](screens/ubuntu-146-7zip-install-result.png).
- W1 [GUI launch](screens/ubuntu-148-w1-launch.png), [fixture parent](screens/ubuntu-151-w1-inputs.png),
  [initial Add dialog](screens/ubuntu-154-w1-add-dialog.png), and
  [wrong-destination ZIP retained](screens/ubuntu-158-w1-dialog-state.png).
- R1 [guest login available](screens/ubuntu-162-r1-login-ready.png).
- W2 [relaunch](screens/ubuntu-167-w2-launched.png), [Add dialog](screens/ubuntu-170-w2-add-dialog.png),
  [inspected destination/settings](screens/ubuntu-174-w2-settings.png),
  [dialog completion](screens/ubuntu-176-w2-result.png), and
  [output directory readback](screens/ubuntu-180-w2-output-present.png).

The W2 readback frame's title/list identify the new output directory while the editable address
field still shows the previous text. That single frame establishes no persistent rendering defect
or rendering correctness. The independent verifier decides archive contents.

## Experimental instrumentation, not product implementation

The following are exact sources used for this bounded execution, retained for review. They assume
the explicit project VM/account/archive layout and are not permission to rerun provisioning or
the workflows. The original [registered helper](../../../../tools/app_baseline.py) and
[fixture manifest](../../../../tools/fixtures/exp009-7zip.json) were not edited.

| Source | Purpose |
|---|---|
| [provision-wine.sh](scripts/provision-wine.sh) | One guest-only pinned package setup, with signed metadata and body hashes before Wine installation |
| [prepare-baseline.sh](scripts/prepare-baseline.sh) | Verify transferred sources/installer and run the unchanged fixture/control helper |
| [observe-guest.py](scripts/observe-guest.py) | Capture guest/session metadata, PE identities and Wine process mappings |
| [guest-console.ps1](scripts/guest-console.ps1) | Input/capture through documented Hyper-V APIs, asserting this single VM's name and GUID |
| [guest-type.ps1](scripts/guest-type.ps1) | Fixed US-ASCII scancode conversion in short batches; credentials omitted from action records |
| [framebuffer-to-png.py](scripts/framebuffer-to-png.py) | Bounded conversion; original returned bytes and extra trailing bytes retained separately |
| [ssh-command.py](scripts/ssh-command.py) | Reuse existing command capture over an isolated, explicitly authenticated project connection; private password input not logged |

Primary API references: [Hyper-V keyboard scancodes](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-keyboard-typescancodes),
[synthetic mouse](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-syntheticmouse),
[framebuffer thumbnails](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/getvirtualsystemthumbnailimage-msvm-virtualsystemmanagementservice),
[Ubuntu image verification](https://ubuntu.com/tutorials/how-to-verify-ubuntu).

## Raw and publication provenance

[Publication manifest](publication-manifest.json) binds every public artifact here except itself
to its SHA-256. Git binds the manifest. The [private artifact inventory](private-artifact-manifest.json)
publishes hashes/lengths only for retained originals, including failed instrumentation and captures.
Bare `ubuntu-*` / `boot-*` filenames in setup records identify that private inventory; the selected
public PNGs are under `screens/`. They are not broken promises that all raw frames were committed.

Public command captures deterministically redact the owner home/account components, account SID,
host name, host LAN address and guest connection address. Tokens include `C:\Users\<redacted>`,
`<redacted-account>`, `<redacted-account-sid>`, `<redacted-host>`, `<redacted-host-lan-ip>` and
`<lab-guest-ip>`; nested escaped path components can use `<redacted-account>`. Synthetic project
accounts `helmsetup`/`helmlab`, VM identifiers, guest paths, runtime values, errors and results remain
actual. Redacted UTF-8 stdout/stderr base64 fields describe publication bytes, not original bytes.
Each changed capture lists fields and original raw hash; its publication hash is distinct.

Installers, ISO/VHD/runtime packages, ZIP/tar bodies, unselected raw frames, credentials and keys
remain outside Git. The Ubuntu installation archive can include guest setup-secret hashes and is
private. Selected PNGs are the only new binary evidence. Captures are byte-preserved by the narrow
Git attribute for this directory. No prior report/evidence was regenerated or overwritten.

## Publication validation

[Documentation validation](validation-docs-first.json) passed, as did
[all 35 repository tests](validation-tests.json): 20 validator tests and 15 verifier/capture tests.
[The publication audit](publication-audit-working.json) checked public/raw hashes, preserved W1
evidence, unchanged helper/fixture/ADR statuses, private identifiers in files and reachable Git
blobs, and decoded base64 streams. The initial audit's
[UTF-8 decoding correction](validation-methodology.json) is preserved separately. These are
repository/provenance checks; they do not change the experiment's FAIL verdict.
