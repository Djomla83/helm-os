# OBS-FS-01 execution definition

Disposable pre-implementation experiment code. **This is not `helm-observe` product
code**, is not a Cargo workspace member, defines no product API, and must not be
promoted into one. It exists to falsify the acquisition mechanism described in
[ADR-0022](../../adr/ADR-0022-observation-authority.md), which remains **Proposed**.

Operative expectations come from the amended
[independent review](../../implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md) at
`63ac6796296894945dff520423cb1a93351e8524` (Amendment 1). Nothing here may redefine
them; `frozen_cases.py` only transcribes them into machine-readable form.

| File | Role |
|---|---|
| `spike.c` | The mechanism under test. Static C binary, unprivileged, no product API |
| `frozen_cases.py` | Frozen case manifest: membership, byte recipes, schedules, seeds, safe outcome sets |
| `oracles.py` | Independent expected digests via Python `hashlib` over the recipes; never reads spike output |
| `harness.py` | Fixture builder, complete-trace ptrace tracer, deterministic syscall-boundary injector |
| `checker.py` | Syscall-policy checker and verdict evaluator; imports and links nothing from the spike |
| `run_obs_fs_01.py` | Runner that executes the frozen set and emits one sanitized report |

Build and run, unprivileged, on the approved Linux lab:

```text
gcc -O2 -Wall -Wextra -static -o spike spike.c
python3 run_obs_fs_01.py --spike ./spike --work <scratch> --out report.json
```

The static link is deliberate: it removes dynamic-loader syscalls from the trace so
that negative claims ("no data-open occurred for this special file") rest on a
complete trace rather than on filtering.
