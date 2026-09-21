# LAUNCH-EXEC-01 experiment sources

**Trial #2 is complete** — freeze `ba41a3f`, GitHub run `34640280964`: 59 PASS /
6 FAIL / 6 INVALID / 1 BLOCKED, `MECHANISM_REJECTED`. Its D-7 is consumed, its
valid trial count is one, and it must never be rerun.

**Status: Trial #3 (`trial-003`) FROZEN — NOT_RUN.** No Trial #3 case has been
posed, the valid Trial #3 count is zero, and no Trial #3 result exists.
`crates/helm-launch` does not exist and must not be created before a valid trial
has run and been reviewed.

**D-7 for Trial #3 is NOT granted, and no Trial #3 dispatcher exists.** These
sources are frozen for independent freeze review, not for execution.
`run_launch_exec_01.py` refuses to pose a case without both a verified source
freeze and an explicit owner-authorisation flag, so no path in this repository
can pose a case by accident. Build 7 does not bind the Trial #3 C sources, and
formal Build 8 evidence is still required.

Authority: proposed [ADR-0024](../../adr/ADR-0024-launch-authority.md), the
[architecture](../../research/HELM-LAUNCH-ARCHITECTURE.md), the
[preregistered definition](../LAUNCH-EXEC-01-DEFINITION.md) and the
[pre-execution review](../../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md).

## What this code is not

Disposable pre-implementation spike code. It is **not** `crates/helm-launch`, is
not a Cargo workspace member, defines no product API, and must not be promoted
into one. Its only purpose is to falsify the mechanism the architecture
describes. A PASS here is evidence about kernel semantics and confers no
permission to implement.

## Files

| File | Role |
|---|---|
| `frozen_cases.py` | **Machine source of truth** for membership, classes, schedules and expectations. The prose table in the definition is derived from it |
| `oracles.py` | Independent expected values: `hashlib` over the frozen byte recipe, never reading spike output |
| `checker.py` | Verdict evaluator. Links nothing from the spike and never repairs a record |
| `harness.py` | Preflight inventory, static-link gate, fixture builder, descriptor pre-loader |
| `run_launch_exec_01.py` | Runner. Refuses to run: see above |
| `driver.py` | Case-posing driver: one concrete plan per frozen case, and the static proof that the plan table is exactly the membership. `--driver-completeness` reports it without posing anything |
| `observations.py` | P-12: observation → frozen outcome token, from closed vocabularies. Absent evidence yields no token |
| `evidence.py` | P-14: publication sanitiser and deterministic serialization. An environment value is never reproduced |
| `make_fixtures.py` | Deterministic generator for the non-compiled fixtures |
| `launcher_spike.c` | The mechanism under test: pin → measure → admission → `clone3(CLONE_PIDFD)` → child setup → `execveat`. Emits the bounded capture prefix base64-encoded (`PRE-D7-B1`) so the helper's report reaches the harness without a second descriptor. Carries TEST/CONTROL-ONLY arms that are never part of the candidate mechanism: `--extra-threads` (M2), `--rejected-acquisition-arm` (M5), the `--post-pin-control-fd` barrier, and the caller-state flags `--parent-fd-set-cloexec` (F3) and `--parent-close-low-fds` (F6, F7) |
| `helper_report.c` | Primary helper; reports its own observed process boundary from inside the executed image |
| `helper_alt.c` | Substitution detector: different body, different digest |
| `helper_dynamic.c` | The one deliberately dynamic helper (E7) |
| `helper_fork.c` | Process-tree negative control, in two descendant shapes |
| `helper_setid.c` | Set-user-ID admission fixture (D-9); exists to be refused |
| `SOURCE-HASHES.json` | The Trial #3 source freeze (`trial-003`): 17 source and 3 definition hashes. Does not hash itself, following OBS-FS-01; the Trial #2 manifest stays addressable at `ba41a3f` |
| `TRIAL-3-CORRECTION-CANDIDATE.json` | Superseded NOT_FROZEN correction provenance; not hashed and never freeze authority |
| `BUILD-EVIDENCE.md` | Append-only compile-only build record; not hashed |

## Owner decisions reflected here

| # | Decision | Where it shows up |
|---|---|---|
| **D-1** arm (i) | scoped unsafe backend; exact FD isolation is claimed and gated | F2 is mandatory with no documented-failure path |
| **D-9** | refuse `S_ISUID`/`S_ISGID` at admission | `SetIdBitsPresent` in `launcher_spike.c`; case X7 |
| **D-10** | empty environment only | `envp_empty` is the only environment the spike can pass; case V1 |
| **D-11** | `PR_SET_NO_NEW_PRIVS` before exec | stage `NO_NEW_PRIVS`; cases N1, N2, N3 |

## Two things this experiment deliberately cannot show

**Privileged transition suppression (N3).** D-11's guarantee is that exec may
not grant new privilege through set-user-ID, set-group-ID or file capabilities.
The runner is unprivileged and no privileged fixture is created, so the actual
suppression of a real transition is **untested**. The report must keep three
things apart and never collapse them: the kernel semantics this rests on, taken
from primary sources; the **directly observed** `NoNewPrivs: 1` state, which N1
establishes and N2 controls for; and the **untested** privileged transition,
which is recorded as BLOCKED and never presented as demonstrated.

**Plan-parse properties.** NUL-argument rejection and environment-name refusal
belong to a crate that does not exist and must not be created before this runs.
They are recorded in `frozen_cases.OUT_OF_SCOPE` and as in-crate obligations, and
no result here supports or refutes them.

## Build state

The C sources are authored and reviewed on a Windows host, where a Linux build
cannot honestly be attempted: faking one, or reaching for WSL merely to obtain a
green result, would manufacture evidence. Linux compile-only evidence comes from
the `launch-exec-01 compile-only pretrial` workflow and is recorded, append-only,
in `BUILD-EVIDENCE.md`; a source change invalidates the earlier builds for that
file. Nothing built there is executed. Any build failure found at a trial's own
preflight is a preflight finding to be recorded, not a defect to be quietly
patched after the freeze.
