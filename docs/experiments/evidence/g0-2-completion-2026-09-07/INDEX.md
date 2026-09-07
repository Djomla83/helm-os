# G0-2 controlled completion evidence â€” 2026-09-07

These artifacts support the [current Gate 0 report](../../EXP-009-GATE0-REPORT.md#g0-2-completion).
The [comparison](comparison.json) is a derived assessment. Captured observations and raw failures
remain separate. [Publication redactions](publication-redactions.json) identify private raw hashes
and distinct public hashes; redacted files are not byte-identical to their private originals.
The final folder preserves process-output bytes, including CRLF stdout, through scoped Git attributes.
Checkpoint `273509d4bc551666e66902e47e535d3c2444de96` normalized six vanilla/staging stdout files
to LF. The final commit restores their retained raw bytes. The digest manifest explicitly records
both checkpoint and captured/final identities; the original capture JSON digests keep their raw meaning.

The [pre-existing historical vanilla observation](../G0-2-dcomposition-vanilla-wine-11.17.json)
is unchanged and remains INCONCLUSIVE. Its controlled completion is a separate record.

| Sequence | Capture and parsed records | Target stdout | Target stderr |
|---|---|---|---|
| vanilla | [capture](vanilla-controlled.json) | [stdout](vanilla-target-stdout.txt) | [stderr](vanilla-target-stderr.txt) |
| staging | [capture](staging-controlled.json) | [stdout](staging-target-stdout.txt) | [stderr](staging-target-stderr.txt) |
| proton | [capture](proton-controlled.json) | [stdout](proton-target-stdout.txt) | [stderr](proton-target-stderr.txt) |
| proton-dos-path | [capture](proton-dos-path-controlled.json) | [stdout](proton-dos-path-target-stdout.txt) | [stderr](proton-dos-path-target-stderr.txt) |

The initial Proton sequence has no probe JSON and remains INCONCLUSIVE. The DOS-path repetition
was registered before execution and preserved separately. No HRESULT was available from the first
attempt. [Resource accounting](resource-accounting.json) includes its cost and other tooling failures.

## Artifact inventory

Every artifact below is intended for Git tracking. [The digest manifest](artifact-manifest.json)
records SHA-256 and size for these publication files; private runtime archives and VHDs are excluded.

| Artifact | Bytes |
|---|---|
| [clone-provisioning.json](clone-provisioning.json) | 4009 |
| [comparison.json](comparison.json) | 1646 |
| [final-lab-state.json](final-lab-state.json) | 1555 |
| [host-display.json](host-display.json) | 258 |
| [host-observation.json](host-observation.json) | 987 |
| [proton-assembly.json](proton-assembly.json) | 4541 |
| [proton-broken-stderr.txt](proton-broken-stderr.txt) | 5475 |
| [proton-broken-stdout.txt](proton-broken-stdout.txt) | 0 |
| [proton-controlled.json](proton-controlled.json) | 21735 |
| [proton-dos-path-broken-stderr.txt](proton-dos-path-broken-stderr.txt) | 5681 |
| [proton-dos-path-broken-stdout.txt](proton-dos-path-broken-stdout.txt) | 86 |
| [proton-dos-path-controlled.json](proton-dos-path-controlled.json) | 23626 |
| [proton-dos-path-good-stderr.txt](proton-dos-path-good-stderr.txt) | 5674 |
| [proton-dos-path-good-stdout.txt](proton-dos-path-good-stdout.txt) | 85 |
| [proton-dos-path-setup.json](proton-dos-path-setup.json) | 9404 |
| [proton-dos-path-spec.json](proton-dos-path-spec.json) | 1217 |
| [proton-dos-path-target-stderr.txt](proton-dos-path-target-stderr.txt) | 5657 |
| [proton-dos-path-target-stdout.txt](proton-dos-path-target-stdout.txt) | 94 |
| [proton-good-stderr.txt](proton-good-stderr.txt) | 5473 |
| [proton-good-stdout.txt](proton-good-stdout.txt) | 0 |
| [proton-provision.json](proton-provision.json) | 77382 |
| [proton-runtime-inventory.json](proton-runtime-inventory.json) | 6174 |
| [proton-setup.json](proton-setup.json) | 8838 |
| [proton-spec.json](proton-spec.json) | 1036 |
| [proton-target-stderr.txt](proton-target-stderr.txt) | 5462 |
| [proton-target-stdout.txt](proton-target-stdout.txt) | 0 |
| [publication-redactions.json](publication-redactions.json) | 5356 |
| [resource-accounting.json](resource-accounting.json) | 9641 |
| [runtime-identities.json](runtime-identities.json) | 6994 |
| [staging-broken-stderr.txt](staging-broken-stderr.txt) | 866 |
| [staging-broken-stdout.txt](staging-broken-stdout.txt) | 86 |
| [staging-controlled.json](staging-controlled.json) | 7700 |
| [staging-good-stderr.txt](staging-good-stderr.txt) | 866 |
| [staging-good-stdout.txt](staging-good-stdout.txt) | 85 |
| [staging-provision.json](staging-provision.json) | 79691 |
| [staging-runtime-inventory.json](staging-runtime-inventory.json) | 137955 |
| [staging-setup.json](staging-setup.json) | 8013 |
| [staging-spec.json](staging-spec.json) | 548 |
| [staging-target-stderr.txt](staging-target-stderr.txt) | 1000 |
| [staging-target-stdout.txt](staging-target-stdout.txt) | 95 |
| [vanilla-broken-stderr.txt](vanilla-broken-stderr.txt) | 236 |
| [vanilla-broken-stdout.txt](vanilla-broken-stdout.txt) | 86 |
| [vanilla-controlled.json](vanilla-controlled.json) | 5759 |
| [vanilla-good-stderr.txt](vanilla-good-stderr.txt) | 236 |
| [vanilla-good-stdout.txt](vanilla-good-stdout.txt) | 85 |
| [vanilla-runtime-inventory.json](vanilla-runtime-inventory.json) | 132852 |
| [vanilla-setup.json](vanilla-setup.json) | 7058 |
| [vanilla-spec.json](vanilla-spec.json) | 546 |
| [vanilla-target-stderr.txt](vanilla-target-stderr.txt) | 354 |
| [vanilla-target-stdout.txt](vanilla-target-stdout.txt) | 94 |

Source references for pinned runtime configuration: [UMU 1.4.4](https://github.com/Open-Wine-Components/umu-launcher/releases/tag/1.4.4),
[UMU launcher source](https://github.com/Open-Wine-Components/umu-launcher/blob/cf3d1b107147480c447ffbfb3f789dc74335074c/umu/umu_run.py),
[UMU-Proton-10.0-4](https://github.com/Open-Wine-Components/umu-proton/releases/tag/UMU-Proton-10.0-4),
[Steam runtime version directory](https://repo.steampowered.com/steamrt3/images/3.0.20260805.254768/).
