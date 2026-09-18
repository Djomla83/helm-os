# helm-launch (experimental 0.1) — P1 portable model + P2 capability admission + P3 internal backend

> **P1, P2 AND P3 IMPLEMENTATION.**
>
> **THERE IS NO PUBLIC `launch()` API.** No public function, type or constant of this crate can
> create a process, on any platform.
>
> **P3 DOES CREATE AND EXECUTE ONE PROCESS, INTERNALLY.** On Linux x86_64 a **crate-private**
> backend consumes an `AuthorizedLaunch`, creates one direct child with `clone3(CLONE_PIDFD)` and
> attempts `execveat` on the exact admitted descriptor. It is not reachable from outside the crate.
>
> **`unsafe` EXISTS, AND ONLY UNDER `src/backend/`.** The crate root still denies it
> (`#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`); exactly one scoped `#![allow(unsafe_code)]`
> sits at the backend module boundary.
>
> **NO PROCESS-GROUP SWEEP. NO RUN TIMEOUT, `SIGTERM` OR GRACE PERIOD. NO STREAM DRAIN POLICY.
> NO RECEIPT FROM A REAL LAUNCH. NO SANDBOX, NO CONTAINMENT, NO WINE, NO EXEC-SUCCESS CLAIM.**

| Slice | State |
|---|---|
| **P1 — portable model** | **ACCEPTED** 2026-09-17 ([decision](../../docs/DECISIONS.md#helm-launch-p1-accepted)), independently reviewed (0 BLOCKER, 0 IMPORTANT), green three-platform CI |
| **P2 — capability admission** | **ACCEPTED** 2026-09-18 ([decision](../../docs/DECISIONS.md#helm-launch-p2-accepted)), independently reviewed (0 BLOCKER, 0 IMPORTANT), green Linux runtime gate |
| **P3 — unsafe backend and child contract** | **AUTHORISED** 2026-09-18 ([decision](../../docs/DECISIONS.md#helm-launch-p3-authorised)); this tree is an **IMPLEMENTED CANDIDATE, NOT YET PRODUCT-ACCEPTED** and **not yet independently reviewed**. One fresh independent **unsafe** review is the next gate |
| **P4, P5** | **NOT AUTHORISED** |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |

`crates/helm-launch` implements the accepted [ADR-0024](../../docs/adr/ADR-0024-launch-authority.md)
and the owner-reviewed [productization plan](../../docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md),
one owner-authorised slice at a time. Schema, API and limits are experimental and unstabilised;
`publish = false`; this is not a release.

**P1-TEST-01 remains open**: it concerns the plan parser's escaped duplicate-key vector, which
neither P2 nor P3 touches.

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
| the process-creation backend (**crate-private**) | **Linux x86_64 only** | consumes an `AuthorizedLaunch`, creates one direct child and attempts one execution. **Not public, not re-exported, not reachable from outside the crate** |

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
AuthorizedLaunch           --╳-----------------------▶ process     public API
AuthorizedLaunch           --crate-private backend---▶ process     P3, internal only
```

**The public edge does not exist.** No public function consumes an `AuthorizedLaunch`: `launch`
does not exist on any platform, and the only consumer that can turn one into a process is
`pub(crate)`. An `AuthorizedLaunch` an external caller holds is inert, and `compile_fail` doctests
in the crate root prove exactly that — that `helm_launch::backend::launch_minimal` is unnameable,
that `AuthorizedLaunch::into_parts` is unreachable, and that no capability hands out a descriptor.

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

## P3 — the internal Linux x86_64 backend

`src/backend/` is compiled only under `cfg(all(target_os = "linux", target_arch = "x86_64"))`, is a
**private** module, and is re-exported nowhere.

**Parent preparation, before any child exists.** argv `CString`s and the null-terminated pointer
array; `envp = [NULL]`; four close-on-exec pipes (stdin, stdout, stderr, exec status); unconditional
`F_DUPFD_CLOEXEC` relocation of all six child-side descriptors to at least 3; the pure close-range
plan; non-blocking parent read ends. A failure here creates **no child**, and every descriptor
closes by `OwnedFd` drop.

**The clone window.** One raw `rt_sigprocmask(SIG_SETMASK, full kernel set)` on the calling thread —
the full set, including glibc's two internal real-time signals, which `sigfillset` omits and
`pthread_sigmask` strips; then `clone3` with **exactly** `CLONE_PIDFD` and `exit_signal = SIGCHLD`;
then, as the **first system call after the clone**, `setpgid(child, child)`; then the mask restore.
`SIGKILL` and `SIGSTOP` are never claimed blockable — the kernel removes them from any mask.

**The closed child sequence**, in this order and no other: `DUP2` ×3, `CLEAR_CLOEXEC` ×3, `CHDIR`,
`CLOSE_RANGE` ≤3, `SETPGID`, `SIGACTION` ×62, `SIGMASK`, `NO_NEW_PRIVS`, `EXEC`. The child computes
nothing: every number, address and byte was prepared by the parent and travels in one `repr(C)`,
`Copy`, `needs_drop == false` `ChildPlan`. It allocates nothing, locks nothing, formats nothing,
prints nothing, panics never and runs no destructor, and it exits only by a successful `execveat` or
by one 8-byte record followed by `exit_group(127)`.

**Exec status.** Exactly 8 bytes: `[stage, 0, 0, 0, errno little-endian]`. An explicit record is a
`pre_exec_failure`; **a clean end-of-file with no record is `indeterminate`** — that is what a child
killed after its last setup stage shows, so it is never read as exec success. No `ExecSucceeded`
value exists anywhere in the crate.

**Group authority without a sweep.** A successful parent `setpgid(child, child)` is recorded as a
boolean for a future slice. P3 issues **no** `kill(-pid, …)` and no other negative-pid signal; a
boundary test and a traced test both fail if one appears.

**The only two fixed bounds** are `SPAWN_CONFIRM_TIMEOUT_MS = 5000` and `POST_KILL_REAP_MS = 5000`.
They are the pre-exec bound and the bound on the non-blocking reap after the one direct-child
`SIGKILL` that a pre-exec timeout or bounded cleanup may send. They are **not** run-timeout
semantics: `plan.timeout_ms`, `SIGTERM`, `grace_ms` and a drain deadline are not executed anywhere.

**The unsafe surface is six operations**, each in its own block with its own `// SAFETY:` comment:
the two raw `rt_sigprocmask` calls, raw `clone3`, `OwnedFd::from_raw_fd(pidfd)` immediately after it,
the crossing into the child entry with the prepared plan pointer, the raw child syscalls, and the one
`core::arch::asm!` shim. Everything else the parent does goes through a safe `rustix` wrapper.
Forbidden even inside the backend: any other foreign interface, `libc::syscall`, `fork`, `vfork`,
`CLONE_VM`, `CLONE_VFORK`, `CLONE_FILES`, `CLONE_THREAD`, `pidfd_open`, `mmap`, `transmute`,
`static mut`, `std::process::Command`, `/proc/self/fd` execution, `fexecve` and pathname `execve`.

## Non-claims

* **No public process creation and no public process execution.** The P3 backend creates one direct
  child internally and attempts one execution of the admitted descriptor. Nothing public reaches it:
  `launch`, `LaunchOutcome`, a process handle, a pidfd, a child pid and a raw descriptor are all
  absent from the public API, and boundary tests plus `compile_fail` doctests fail on any of them
  appearing.
* **No exec-success claim.** A clean exec-status end-of-file is `indeterminate`. Nothing in the
  crate derives, infers or reports that a program ran.
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

`serde`, `serde_json` and `sha2` on every platform. On the Linux x86_64 cohort only:

* `rustix = "=1.1.4"` — the pin `helm-observe` already uses — with `default-features = false` and
  `std`, `fs`, `process` and `pipe` enabled. **`event` is deliberately not enabled**: the `poll`
  observation loop that would need it belongs to P4, which is not authorised. Neither are `thread`,
  `mm`, `net` or `runtime`.
* `libc = "=0.2.189"` — the version already locked and vetted here — **for constants only**. Every
  syscall number, signal number, clone flag and ABI constant is written as the literal the Linux
  x86_64 UAPI defines and then pinned against the `libc` constant of the same name in a `const`
  assertion, so a wrong number is a compile error rather than a wrong system call. No `libc`
  function is called, `libc::syscall` is not used, and **no `extern` block exists anywhere in the
  crate**.

No build script, and exactly one new line in `Cargo.lock`: `libc` joining `helm-launch`'s dependency
list. No package and no version was added.

One non-default feature, `test-fault-injection`, exists for the S5 and S6 tests. It is gated on the
feature **and** on `debug_assertions`, so no release build contains injection even if the feature is
enabled by accident, and a CI step greps the release artifact for a marker to prove it.

Outside the six authorised unsafe operations, every operating-system call goes through a safe
`rustix` wrapper: `F_GETFL`, `F_SETFL`, `F_DUPFD_CLOEXEC`, `fstat`, `pread`, `read`, `pipe2`,
`setpgid`, `waitid(P_PIDFD, WEXITED | WNOHANG)` and `pidfd_send_signal`.

## Lint policy

The crate does **not** inherit the workspace lint table (plan section 7.4, ADR-0024 section E).
Its manifest restates every workspace lint — `clippy::unwrap_used`, `clippy::expect_used` and
`clippy::panic` as `deny` — and adds `unsafe_op_in_unsafe_fn`,
`clippy::undocumented_unsafe_blocks` and `clippy::multiple_unsafe_ops_per_block`. The one accepted
difference is `unsafe_code = "deny"` instead of `forbid`, in the manifest and restated at the crate
root as `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`, because only `deny` leaves room for the one
scoped `allow`.

**That `allow` is now active, and only there.** `src/backend/mod.rs` carries
`#![allow(unsafe_code)]` as an inner attribute, so it covers that module and its descendants and
nothing else. The same module re-denies the backend discipline:
`clippy::indexing_slicing`, `clippy::arithmetic_side_effects`, `clippy::as_conversions`,
`clippy::missing_safety_doc`, `clippy::undocumented_unsafe_blocks`,
`clippy::multiple_unsafe_ops_per_block` and `unsafe_op_in_unsafe_fn`. So inside the one module that
talks to the kernel directly there is no indexing, no unchecked arithmetic, no `as` cast, no
undocumented unsafe block and no block holding two unsafe operations.

`tests/p2_boundary.rs` fails on any drift of either manifest table, on a crate root that does not
deny both lints or that forbids, on either lint named as code in the portable or authority sources,
on any other workspace member that stops inheriting the workspace lints, on the `unsafe` token
appearing anywhere in those sources, on any module beyond the eight, on product code that relaxes
the panic-free lints, on process, foreign-interface or raw-descriptor vocabulary used as code
outside the backend, on product code naming a pathname API or host state, on any portable module
naming a descriptor, on a P2 or P3 surface not gated by exactly
`cfg(all(target_os = "linux", target_arch = "x86_64"))`, on a changed admission constant, on a
capability type gaining a derive, and on any dependency or lockfile drift.

`tests/p3_boundary.rs` walks the tree rather than a fixed list, so a **new** file cannot escape it.
It fails on a source or test file the inventory does not name, on the `unsafe` token as code outside
`src/backend/`, on a second scoped `allow` anywhere, on any test source relaxing the lint, on a
second `asm!`, on the shim claiming `nomem`, `preserves_flags` or `readonly` or failing to declare
`rcx` and `r11` clobbered, on the child file losing `#![no_implicit_prelude]` or naming anything
that allocates, locks, formats, prints or panics, on the child stage order changing, on a `libc`
mention outside a `const` assertion, on a forbidden clone flag, on a procfs path, on any `kill` or
group signal, on `std::process::Command` in backend product code, on a fault-injection `cfg` without
`debug_assertions`, on a bare `pub` item in the backend, and on a P4 name appearing.

`tools/tests/test_helm_launch_confinement.py` asserts the same confinement facts a second time, in a
second language, without `cargo`.

## Accepted future behaviour — NOT IMPLEMENTED

ADR-0024 also accepts, as architecture only, the lifecycle that P4 would add: a public `launch`, a
`LaunchOutcome`, an observation loop over the status channel, the two streams and the pidfd, the
plan-driven run deadline with `SIGTERM` and its grace period, a general drain policy, the guarded
every-path process-group `SIGKILL` sweep, and a receipt emitted from a real launch. **None of that
exists in this crate.** The receipt vocabulary and the pure layout and lifecycle models describe the
contract so it can be tested before the loop exists; they observe nothing and execute nothing.
Implementing any of it needs a new explicit owner decision. P4 and P5 are **not authorised**.

## Testing

```text
cargo test -p helm-launch --locked
cargo test -p helm-launch --locked --features test-fault-injection
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

The P3 backend tests are **unit** tests in `src/backend/tests.rs`, because the P3 surface is
crate-private on purpose and must stay so. On Linux x86_64 they cover: the `clone_args` and kernel
`sigaction` ABI records, field by field; the full blocked mask, including the two signal numbers
glibc keeps to itself; the stage vocabulary and the 8-byte record, including every malformed shape;
descriptor isolation (an unrelated close-on-exec descriptor, an unrelated one **without**
close-on-exec, the caller's own executable descriptor, and an executed image that holds exactly
`{0, 1, 2}`); argv with spaces, shell metacharacters, a newline, an empty element and an arbitrary
`argv[0]`; an executed image that observes an **empty** environment while the launcher has `PATH`
set; the admitted working directory by `(st_dev, st_ino)`; `NoNewPrivs: 1` with a control that the
launching process has 0; exec failures `EACCES`, `ENOEXEC` and `ETXTBSY`, and `CHDIR: EACCES`;
execution of the admitted **descriptor** after its pathname is unlinked and replaced; the direct
child's identity, the closed stdin pipe and the carried-through P2 measurement; and, under the
non-default feature, S5 (death before exec with no record → indeterminate, never success), S6
(pre-exec stall → fixed bound, one pidfd `SIGKILL`, bounded reap, no zombie) and every child stage's
structured failure path.

Two tracer tests run a purpose-built launcher test process under `strace -f` — once from a
single-threaded parent, once from a parent with three threads actively allocating, and once with a
registered `pthread_atfork` handler preloaded — and assert: exactly one `clone3` carrying
`CLONE_PIDFD` and not `CLONE_THREAD` (the Rust test harness clones threads with `clone3` too, which
is the Trial #2 M2 confusion this filter exists to avoid); no `pidfd_open`; the full-set
`rt_sigprocmask` immediately before the clone; `setpgid(child, child)` as the **first** call after
it; the restore after that; `waitid` by `P_PIDFD` only; **no process-group signal anywhere**; the
child window containing only the permitted syscall set; the exact stage order with `fchdir` before
the first `close_range`; and an `execveat` with `AT_EMPTY_PATH`, an empty pathname and no procfs.
A missing `strace`, `rustc` or `cc` is a **test environment failure** with an explicit message,
never a silent skip.

**These tests do create and execute processes**, which is the newly authorised P3 capability and
ordinary validation of it. Every fixture is newly written here and compiled by `rustc` at test time,
and every report fixture passes a **producer self-test** — run directly through
`std::process::Command`, parsed, and its marker checked to name the intended fixture — before any
launcher test consumes its report. That is the Trial #3 X2c rule. No `launcher_spike`, no frozen
Python runner, no frozen helper ELF and no LAUNCH-EXEC-01 asset is used or built. This is **not**
Trial #3, **not** a Trial #4 and **not** D-7 activity.

The P2 admission tests still execute nothing.
