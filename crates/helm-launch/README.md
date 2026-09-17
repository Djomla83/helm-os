# helm-launch (experimental 0.1) — P1 portable model

> **P1 IMPLEMENTATION ONLY.**
>
> **NO EXECUTION BACKEND EXISTS.**
>
> **NO PROCESS CAN BE CREATED BY THIS CRATE.**
>
> **NO EXECUTABLE OR WORKING-DIRECTORY CAPABILITY EXISTS YET.**
>
> **NO `unsafe` CODE.** The source forbids it outright (`#![forbid(unsafe_code)]`).

This crate is the first, owner-authorised implementation slice of `helm-launch`, under Accepted
[ADR-0024](../../docs/adr/ADR-0024-launch-authority.md) and the owner-reviewed
[productization plan](../../docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md). The owner
[authorised P1 only on 2026-09-17](../../docs/DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised):
the crate skeleton and a portable, pure model. **P2 and every later slice are not authorised.**
Schema, API and limits are experimental and unstabilised; `publish = false`; this is not a
release, and it has not been independently reviewed.

## What P1 implements

Everything below is pure, portable Rust. It compiles and is tested identically on Linux,
Windows and macOS, performs no filesystem, process, network, environment or clock access, and
depends on no HELM crate.

| Surface | What it is |
|---|---|
| `parse_launch_plan` | untrusted bytes to an inert `ValidatedLaunchPlan`, or ordered, capped `LaunchPlanErrors` |
| `ValidatedLaunchPlan` | caller **intent**: argv, working-directory identifier, bounds, asserted digests. No descriptor, no executable, no authority |
| `Digest` | 32 bytes, spelled as 64 lowercase hex characters. It identifies bytes and means nothing more |
| `ReceiptRecord` and the fact enums | the accepted 0.1 receipt contract as closed, non-verdict data |
| `LaunchReceipt` | exact serialised bytes plus SHA-256 over exactly those bytes; **no public producer exists** |
| `LaunchPlanErrors` | fixed codes and closed-schema locators; no input text, host path, pid or descriptor number |
| descriptor-layout planner (crate-private) | arithmetic over descriptor **numbers as inert integers**: unconditional relocation to `>= 3`, the stdio mapping, and the ranges to close |
| lifecycle model (crate-private) | a deterministic state machine over synthetic events and a caller-supplied time; it emits the actions a future backend would take and the facts a receipt would record |

### The 0.1 plan document

```json
{
  "schema": "helm-launch-plan",
  "version": "0.1",
  "execution_kind": "linux_exact_executable",
  "argv": ["tool", "--flag", "a b; c"],
  "environment": { "mode": "empty" },
  "working_directory": { "capability_id": "workdir" },
  "stdin": { "mode": "closed_pipe_eof" },
  "stdout": { "capture_prefix_bytes": 0 },
  "stderr": { "capture_prefix_bytes": 4096 },
  "timeout_ms": 30000,
  "termination": { "signal": "SIGTERM", "grace_ms": 5000 },
  "asserted_context": {
    "subject_spec_sha256": "…64 lowercase hex…",
    "binding_report_sha256": "…64 lowercase hex…"
  }
}
```

| Rule | Value |
|---|---|
| document size | at most 32 768 bytes, checked before parsing |
| identity | SHA-256 over the exact input bytes; no canonicalisation |
| structure | strict JSON; duplicate **decoded** keys refused at every depth, including escaped spellings; nesting at most 8; unknown keys refused |
| `argv` | 1–64 JSON strings; each at most 4 096 bytes after decoding and free of NUL, including the JSON NUL escape; at most 131 072 bytes in total; `argv[0]` is caller data and is never invented |
| `environment.mode` | `empty` only |
| `working_directory.capability_id` | `[a-z0-9][a-z0-9._-]{0,79}` — an identifier, never a path |
| `stdin.mode` | `closed_pipe_eof` only |
| `capture_prefix_bytes` | integer 0–65 536 per stream |
| `timeout_ms` | integer 1–600 000 |
| `termination` | `signal` is `SIGTERM` only; `grace_ms` integer 0–60 000 |
| `asserted_context` | optional; each member optional; 64 lowercase hex; `null` is refused; recorded only, never compared or interpreted |

Absent by design: any executable path or name, host path, shell or command string, `PATH`,
environment entry, descriptor number, expected exit code, success condition, and any Wine,
prefix or runtime field. Numbers must be JSON integers; `1.0` and `1e3` are refused.

Note that `MAX_ARGV_TOTAL_BYTES` cannot be exceeded through a document, because the document
bound is smaller and JSON escape decoding never lengthens a string. The rule is still enforced
and is unit-tested directly.

### The 0.1 receipt model

Fixed field order, no whitespace, no map iteration:

```text
schema, version, backend, plan_sha256, asserted_context{subject_spec_sha256, binding_report_sha256},
working_directory_id, executable{pre_exec_body_size, pre_exec_body_sha256, pre_exec_mode_bits, elf_type},
argument_count, environment_mode, exec_status{kind, stage|reason, errno?}, child_end{kind, code|signal, core_dumped},
run_deadline_expired, termination{sigterm_sent, sigkill_sent, group_sweep},
stdout{bytes_drained, drained_sha256, completeness, errno?}, stderr{…}
```

* **No exec-success value exists.** `exec_status` is `pre_exec_failure` or `indeterminate`.
  Clean exec-status EOF alone is not positive exec proof.
* No timestamp, duration, pid, descriptor number, host path, output byte, prefix length or
  authenticity field exists, and a test pins the closed key set.
* Every emitted spelling and every schema key is checked against a verdict vocabulary (`pass`,
  `fail`, `ok`, `success`, `successful`, `succeeded`, `ready`, `compatible`, `verified`,
  `worked`, `launched`, `sandboxed`, `contained`, `safe`, `authentic`, `signed`, `trusted`).
* The widest expressible receipt stays below `MAX_RECEIPT_BYTES` (8 192).

**Deterministic test vectors.** Two fixed test records serialise to exact expected bytes whose
SHA-256 was computed outside Rust; a fixed plan has a fixed identity. They are asserted in the
tests on every platform. **They are deterministic test data, not records of any launch, and not
authentic.**

## Public API boundary

```text
untrusted bytes --parse_launch_plan--> ValidatedLaunchPlan    intent only, no authority
```

* `ValidatedLaunchPlan`, `LaunchReceipt` and `ReceiptRecord` have private fields and no public
  constructor, `Default`, setter, `Serialize` or `Deserialize`. `compile_fail` doctests pin
  this, including deserialising receipt bytes into `LaunchReceipt`.
* No function named `launch`, `authorize`, `admit_executable` or `admit_working_directory`
  exists, and no type named `ExecutableCapability`, `WorkingDirectoryCapability`,
  `AuthorizedLaunch` or `LaunchOutcome` exists. They are **not stubbed**.
* No fact type has `is_success`, `ok`, a `bool` conversion or a verdict variant.

## Lint policy

The crate does **not** inherit the workspace lint table (plan section 7.4, ADR-0024 section E).
Its manifest restates every workspace lint — `clippy::unwrap_used`, `clippy::expect_used` and
`clippy::panic` as `deny` — and adds `unsafe_op_in_unsafe_fn`,
`clippy::undocumented_unsafe_blocks` and `clippy::multiple_unsafe_ops_per_block`. The one
accepted difference is `unsafe_code = "deny"` in the manifest instead of `forbid`, which a later,
separately authorised backend slice would need; in P1 the source still forbids it.
`tests/p1_boundary.rs` fails on any drift of either table, on any other workspace member that
stops inheriting the workspace lints, on any `unsafe` token in `src/`, on any module beyond the
six P1 modules, and on operating-system, execution or later-slice identifiers used as code.

## Accepted future behaviour — NOT IMPLEMENTED

ADR-0024 accepts, as architecture only, a Linux x86_64 backend that would admit an already-open
ELF descriptor, create one direct child with `clone3(CLONE_PIDFD)`, execute the exact descriptor
with `execveat`, keep exactly descriptors 0–2, observe the child through a pidfd, and emit a
receipt. **None of that exists in this crate.** The receipt vocabulary and the pure layout and
lifecycle models describe that contract so it can be tested before any backend exists; they
observe nothing and execute nothing. Implementing any of it needs a new explicit owner decision.

## Non-claims

* **No process execution.** Nothing in this crate can create, signal, wait for or reap a process.
* **Not a sandbox**, and **no containment** of any kind.
* **No Wine**, Proton, prefix or runtime handling.
* **No `PATH`, no shell**, no command string, no name lookup.
* **No authority from parsing.** A validated plan is intent. No plan, specification,
  observation, binding report or evidence bundle grants execution authority.
* **No receipt authenticity.** Receipt bytes may be copied or fabricated outside this crate, are
  not signed, and are not proof of provenance. A digest identifies bytes, not their origin.
* **No stability or measured-equals-executed claim**, since no measurement exists in P1.
* **Cross-platform determinism is asserted by tests**, and is only as strong as the platforms
  those tests have actually run on.

## Testing

```text
cargo test -p helm-launch --locked
cargo clippy -p helm-launch --all-targets --all-features --locked -- -D warnings
```

The tests are pure and self-contained: parser contract and bounded adversarial inputs,
exhaustive receipt variant combinations, the verdict-vocabulary guard, descriptor-layout
properties over generated and simulated descriptor tables, lifecycle scripts for the S5, S6,
T1–T5, O6, T30, T31, T40 and T41 scenarios plus generated event orders, `compile_fail` type
boundaries, and the P1 boundary inspection. No test runs a process or any LAUNCH-EXEC-01 asset.
