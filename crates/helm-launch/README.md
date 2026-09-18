# helm-launch (experimental 0.1) — P1 portable model + P2 capability admission

> **P1 AND P2 IMPLEMENTATION ONLY.**
>
> **NO EXECUTION BACKEND EXISTS.**
>
> **NO PROCESS CAN BE CREATED OR EXECUTED BY THIS CRATE.**
>
> **`AuthorizedLaunch` NOW EXISTS AND CANNOT BE EXECUTED**, because `launch()` does not exist.
>
> **NO `unsafe` CODE.** The crate root denies it (`#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`),
> and no `allow` of either lint exists.

| Slice | State |
|---|---|
| **P1 — portable model** | **ACCEPTED** 2026-09-17 ([decision](../../docs/DECISIONS.md#helm-launch-p1-accepted)), independently reviewed (0 BLOCKER, 0 IMPORTANT), green three-platform CI |
| **P2 — capability admission** | **AUTHORISED** 2026-09-18 ([decision](../../docs/DECISIONS.md#helm-launch-p2-authorised)); this tree is an **implemented candidate, not yet product-accepted and not yet independently reviewed** |
| **P3, P4, P5** | **NOT AUTHORISED** |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |

`crates/helm-launch` implements the accepted [ADR-0024](../../docs/adr/ADR-0024-launch-authority.md)
and the owner-reviewed [productization plan](../../docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md),
one owner-authorised slice at a time. Schema, API and limits are experimental and unstabilised;
`publish = false`; this is not a release.

This edition of the README resolves independent-review finding **P1-DOC-02**: the stale
"not independently reviewed" sentence is replaced by the per-slice review state in the table above,
and the receipt section now states per-kind field presence exactly. **P1-TEST-01 remains open**: it
concerns the plan parser's escaped duplicate-key vector, which P2 does not touch.

## What exists

| Surface | Platform | What it is |
|---|---|---|
| `parse_launch_plan` | portable | untrusted bytes to an inert `ValidatedLaunchPlan`, or ordered, capped `LaunchPlanErrors` |
| `ValidatedLaunchPlan` | portable | caller **intent**: argv, working-directory identifier, bounds, asserted digests. No descriptor, no executable, no authority |
| `Digest` | portable | 32 bytes, spelled as 64 lowercase hex characters. It identifies bytes and means nothing more |
| `ReceiptRecord` and the fact enums | portable | the accepted 0.1 receipt contract as closed, non-verdict data |
| `LaunchReceipt` | portable | exact serialised bytes plus SHA-256 over exactly those bytes; **no public producer exists** |
| `LaunchPlanErrors`, `AdmissionError`, `AuthorizationRefusal` | portable | fixed codes and closed-schema locators; no input text, host path, pid or descriptor number |
| descriptor-layout planner (crate-private) | portable | arithmetic over descriptor **numbers as inert integers** |
| lifecycle model (crate-private) | portable | a deterministic state machine over synthetic events and a caller-supplied time |
| `admit_executable` → `ExecutableCapability` | **Linux x86_64 only** | admits one already-open object through the descriptor the caller moved in |
| `admit_working_directory` → `WorkingDirectoryCapability` | **Linux x86_64 only** | admits one already-open directory with a caller-supplied logical identifier |
| `authorize` → `AuthorizedLaunch` | **Linux x86_64 only** | composes a validated plan with both capabilities, with **zero I/O** |
| `MAX_EXECUTABLE_BYTES` | **Linux x86_64 only** | the accepted 512 MiB admission bound |

Off the Linux x86_64 cohort the six P2 names **do not exist in the public API** and are not stubbed;
`compile_fail` doctests in the crate root prove that on every non-cohort platform, and a sibling
block proves their presence on the cohort. No support for another platform is advertised, and Linux
on another architecture is not claimed.

## Authority

```text
untrusted bytes            --parse_launch_plan-------▶ ValidatedLaunchPlan          intent only
caller-owned executable fd --admit_executable--------▶ ExecutableCapability
caller-owned cwd fd + id   --admit_working_directory-▶ WorkingDirectoryCapability
plan + executable + cwd    --authorize---------------▶ AuthorizedLaunch
AuthorizedLaunch           --╳-----------------------▶ process
```

**The last edge does not exist.** No function consumes an `AuthorizedLaunch`, because `launch` does
not exist on any platform. An `AuthorizedLaunch` is an inert in-process value.

* Execution authority is **an already-open descriptor a trusted caller moves in by value**. It is
  never a pathname, a name, a verdict or a document field. The product API takes no path, resolves
  no name, opens nothing, searches no `PATH`, runs no shell and reads no ambient environment.
* No function turns bytes, a `String`, a `Path`, a `ValidatedAppSpec`, an `ObservationArtifact`, a
  `RootCapability`, a `BindingReport`, a `Contradiction`, a `Coverage`, any `serde` input, receipt
  bytes or a `LaunchReceipt` into a capability or an authorisation. The crate links **no HELM
  crate**, so no such type is even nameable in it.
* `Contradiction::NoClaimContradicted` is **not permission**. Plan `asserted_context` digests are
  **inert caller assertions**: `authorize` does not read, compare, fetch or interpret them, and
  their presence, absence and value change nothing.
* `ExecutableCapability`, `WorkingDirectoryCapability` and `AuthorizedLaunch` have private fields,
  no public constructor, no `Default`, no `Clone`, no `From`, no setter, no `Serialize`, no
  `Deserialize` and no public raw-descriptor accessor. Each is `Send` and **not** `Sync`.
  `compile_fail` doctests pin every one of those absences.

## Executable admission

Performed in the accepted order, entirely through the one descriptor the caller moved in:

| Step | Check | Refusal |
|---|---|---|
| 1 | access mode via `F_GETFL`: `O_PATH`, `O_WRONLY` and `O_RDWR` refused, only `O_RDONLY` admitted | `DESCRIPTOR_MODE_UNSUITABLE` |
| 2 | `fstat` — regular file. **First metadata sample** | `NOT_REGULAR_FILE` |
| 3 | `S_ISUID` or `S_ISGID` present; no privilege transition is attempted | `SET_ID_BITS_PRESENT` |
| 4 | first sample size above **512 MiB**, refused **before any body byte is read** | `EXECUTABLE_TOO_LARGE` |
| 5 | 64 bytes read positionally at offset 0: magic absent or fewer than 64 bytes | `NOT_ELF` |
| 5 | magic present but outside `ELFCLASS64` / `ELFDATA2LSB` / `EM_X86_64` / `ET_EXEC`\|`ET_DYN` | `ELF_NOT_IN_COHORT` |
| 6 | measurement: positional reads, fixed **64 KiB** buffer, from offset 0 until end of file or **one byte beyond** the first sample's size; SHA-256 over exactly the bytes read | `READ_FAILED` |
| 7 | **second metadata sample** compared with the first on `st_size`, `st_mtime`, `st_ctime`, with nanoseconds, plus the byte count against the first sample's size | `MEASUREMENT_INSTABILITY_DETECTED` |
| 8 | the capability, owning the same descriptor and the inert admitted facts | — |

Descriptor flags or metadata that cannot be obtained are `METADATA_UNAVAILABLE`: the mode and the
object kind are then unknown, so admission refuses rather than guessing.

The admitted facts are `pre_exec_body_size`, `pre_exec_body_sha256`, `pre_exec_mode_bits`
(`st_mode & 0o7777`, from the **first** accepted sample) and `elf_type`. No host path, inode or
device identity, or descriptor number is recorded or exposed, and `Debug` prints admitted facts
only. The shared file offset is never used or moved, which a test verifies through a duplicate of
the same open file description.

**Not checked, deliberately.** Execute permission (recorded, never enforced — the kernel decides at
execution time, which this crate cannot reach), filesystem type, mount options, `noexec`,
`security.capability` attributes, `binfmt_misc` registrations, the dynamic loader, shared libraries,
the kernel version, credentials and sandbox state. `/proc/sys/fs/binfmt_misc` is not read.
`O_NOATIME` is not used, because it requires file ownership or `CAP_FOWNER`. The descriptor's
close-on-exec flag is irrelevant and is not inspected; `F_GETFL` does not even report it.

**Side effects of admission, stated.** Reading the body **may update atime** under the host's mount
policy and **populates the page cache**.

## Working-directory admission

The identifier must match `[a-z0-9][a-z0-9._-]{0,79}`, validated by **the same crate-private
parser the plan uses** — there is no second, divergent validator. Only an `O_RDONLY` directory
descriptor is admitted; `O_PATH` is refused. The directory is **never enumerated**, no path is
resolved or recorded, **no search permission is checked** — that kernel decision belongs to a
future execution slice — and nothing changes any working directory. The capability exposes exactly
the caller's logical identifier.

## Authorisation

`authorize` consumes the plan and both capabilities and performs **zero I/O**. It checks exactly
one relation — the plan's `working_directory.capability_id` against the capability's logical
identifier — and refuses a mismatch with `WORKING_DIRECTORY_ID_MISMATCH` before any process could
exist. Because all three inputs are consumed, a refusal drops both capabilities and closes their
descriptors; no refused value hands authority back. On success the plan and both capabilities move
in **unchanged**: nothing is re-measured, reopened, re-resolved or stat-ed, and no binding context
is inspected.

## Non-claims

* **No process creation and no process execution.** Nothing in this crate can create, signal, wait
  for or reap a process. `LaunchOutcome`, `launch`, a `backend` directory, `clone3`, `execveat`,
  pidfd acquisition or signalling, `waitid`, `close_range`, `fchdir`, `PR_SET_NO_NEW_PRIVS`, child
  pipes, timeouts and a polling lifecycle all do not exist, and a boundary test fails on any of
  their vocabulary appearing as code.
* **Measurement is a pre-execution measurement of the pinned object.** It is **not** the identity
  of bytes that executed, and says nothing about an ELF interpreter, a shared library or any other
  part of a loaded-code closure. The descriptor pins the inode; it does not freeze the contents.
* **Detected instability means only that the protocol detected instability.** Not detecting it
  **proves nothing**: it is not evidence that no mutation occurred, that the inode is immutable,
  that a snapshot exists, or that the measured bytes are the bytes any later execution would run.
  The protocol observes exactly five metadata fields and one byte count; a change that moves none
  of them is not detected.
* **An in-cohort ELF header proves nothing about the program**, and admission is not a statement
  that the object could ever be executed successfully.
* **Not a sandbox**, and **no containment** of any kind. Admission grants no privilege.
* **No Wine**, Proton, prefix or runtime handling. **No `PATH`, no shell**, no command string, no
  name lookup, no pathname launch.
* **No authority from parsing.** A validated plan is intent. No plan, specification, observation,
  binding report or evidence bundle grants execution authority.
* **No receipt authenticity.** Receipt bytes may be copied or fabricated outside this crate, are
  not signed, and are not proof of provenance. A digest identifies bytes, not their origin.
* **Cross-platform determinism is asserted by tests**, and is only as strong as the platforms those
  tests have actually run on. The Linux admission tests run on Linux only.

## The 0.1 plan document

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

## The 0.1 receipt model

Fixed field order, no whitespace, no map iteration. Per-kind field presence is exact: `errno` is
mandatory with `pre_exec_failure` and absent otherwise; `reason` belongs to `indeterminate` only;
`code` belongs to `exited`, `signal` and `core_dumped` to `signaled`; a stream's `errno` exists
only with `read_failed`.

```text
schema, version, backend, plan_sha256, asserted_context{subject_spec_sha256, binding_report_sha256},
working_directory_id, executable{pre_exec_body_size, pre_exec_body_sha256, pre_exec_mode_bits, elf_type},
argument_count, environment_mode, exec_status{kind, stage|reason, errno?}, child_end{kind, code|signal, core_dumped},
run_deadline_expired, termination{sigterm_sent, sigkill_sent, group_sweep},
stdout{bytes_drained, drained_sha256, completeness, errno?}, stderr{…}
```

* **No exec-success value exists.** `exec_status` is `pre_exec_failure` or `indeterminate`.
  Clean exec-status EOF alone is not positive exec proof.
* **No public producer exists**, and no observation of any of these facts happens in this tree: no
  process runs, so nothing is observed. The model fixes the accepted contract so it can be
  serialised and tested before any backend exists.
* No timestamp, duration, pid, descriptor number, host path, output byte, prefix length or
  authenticity field exists, and a test pins the closed key set.
* Every emitted spelling and every schema key is checked against a verdict vocabulary (`pass`,
  `fail`, `ok`, `success`, `successful`, `succeeded`, `ready`, `compatible`, `verified`,
  `worked`, `launched`, `sandboxed`, `contained`, `safe`, `authentic`, `signed`, `trusted`).
* The widest expressible receipt stays below `MAX_RECEIPT_BYTES` (8 192).

**Deterministic test vectors.** Two fixed test records serialise to exact expected bytes whose
SHA-256 was computed outside Rust; a fixed plan has a fixed identity; two admission fixtures have
externally computed body digests. They are asserted in the tests. **They are deterministic test
data, not records of any launch, and not authentic.**

## Dependencies

`serde`, `serde_json` and `sha2` on every platform. On the Linux x86_64 cohort only,
`rustix = "=1.1.4"` — the pin `helm-observe` already uses — with `default-features = false` and
only `std` and `fs` enabled. `process`, `pipe`, `event` and `thread` are deliberately **not**
enabled: a feature is not enabled before the slice that needs it, and no such slice is authorised.
There is **no direct `libc` dependency**, no build script and no new package or version in
`Cargo.lock`.

Every operating-system call goes through a safe `rustix` wrapper — `F_GETFL`, `fstat` and `pread` —
and no code manipulates a raw descriptor number.

## Lint policy

The crate does **not** inherit the workspace lint table (plan section 7.4, ADR-0024 section E).
Its manifest restates every workspace lint — `clippy::unwrap_used`, `clippy::expect_used` and
`clippy::panic` as `deny` — and adds `unsafe_op_in_unsafe_fn`,
`clippy::undocumented_unsafe_blocks` and `clippy::multiple_unsafe_ops_per_block`. The one
accepted difference is `unsafe_code = "deny"` instead of `forbid`, in the manifest and restated at
the crate root as `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`, because only `deny` leaves room
for the one scoped `allow` that a later, separately authorised backend slice may need. No such
slice is authorised and no such `allow` exists.

`tests/p2_boundary.rs` fails on any drift of either table, on a crate root that does not deny both
lints or that forbids, on either lint named anywhere else in `src/`, on any other workspace member
that stops inheriting the workspace lints, on the `unsafe` token appearing anywhere in `src/` or
`tests/`, on any module beyond the seven, on product code that relaxes the panic-free lints, on
process, foreign-interface or raw-descriptor vocabulary used as code, on product code naming a
pathname API or host state, on any module other than `src/authority.rs` naming a descriptor, on a
P2 surface not gated by exactly `cfg(all(target_os = "linux", target_arch = "x86_64"))`, on a
changed admission constant, on a capability type gaining a derive, and on any dependency or
lockfile drift.

## Accepted future behaviour — NOT IMPLEMENTED

ADR-0024 accepts, as architecture only, a Linux x86_64 backend that would create one direct child
with `clone3(CLONE_PIDFD)`, execute the exact admitted descriptor with `execveat`, keep exactly
descriptors 0–2, observe the child through a pidfd, and emit a receipt. **None of that exists in
this crate.** The receipt vocabulary and the pure layout and lifecycle models describe that
contract so it can be tested before any backend exists; they observe nothing and execute nothing.
Implementing any of it needs a new explicit owner decision, and the accepted `unsafe` exception
stays reserved for that slice and is **not active**.

## Testing

```text
cargo test -p helm-launch --locked
cargo clippy -p helm-launch --all-targets --all-features --locked -- -D warnings
```

Portable tests are pure and self-contained: parser contract and bounded adversarial inputs,
exhaustive receipt variant combinations, the verdict-vocabulary guard, descriptor-layout properties
over generated and simulated descriptor tables, lifecycle scripts for the S5, S6, T1–T5, O6, T30,
T31, T40 and T41 scenarios plus generated event orders, the admission and refusal vocabularies,
`compile_fail` type boundaries, and the P2 boundary inspection.

The Linux x86_64 admission tests (`tests/linux_admission.rs`, plus unit tests in
`src/authority.rs`) build fixtures as the trusted caller and cover: admitted `ET_EXEC` and `ET_DYN`
objects; `#!` scripts, non-ELF and short objects; each wrong cohort field; a directory and a
character device; `O_PATH`, `O_WRONLY` and `O_RDWR`; a descriptor without close-on-exec; set-user-ID
and set-group-ID objects; a sparse object above the size bound; externally computed body digests;
the unchanged shared file offset; recorded mode bits; identifier grammar, `NotDirectory` and
`O_PATH` for the working directory; matching and mismatching authorisation; asserted-context
irrelevance; descriptor discipline across many refusals; and the detected-instability rule for each
observed field through a `cfg(test)`-only internal seam that no release build contains.

**No test executes an admitted object**, invokes `launcher_spike` or any helper ELF, or creates a
process: the crate has no function that could. These are ordinary product tests, not LAUNCH-EXEC-01
cases, not a formal trial and not D-7 activity.
