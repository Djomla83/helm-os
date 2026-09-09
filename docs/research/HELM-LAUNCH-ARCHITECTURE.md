# helm-launch 0.1 architecture and falsification plan

**Status: design under owner review. No implementation, no crate, no product API change.**\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`.\
**Task:** design HELM's fifth product module — the first whose purpose involves **actual
process execution** — and preregister what must be falsified before it may be implemented.\
**Not authorised here:** `crates/helm-launch`, Wine execution, 7-Zip execution, an A0 rerun,
lab modification, product API changes to existing crates, privilege escalation, sandbox
claims, or running the proposed experiment.

Everything below is derived from the **current** public Rust types and manifests on main,
read directly, and from primary Linux documentation. Where a claim rests on documentation
rather than on evidence produced in this repository, it is marked **documentation-derived**
and becomes an experiment obligation rather than an accepted fact.

## 1. Reconstructed current state

Four owner-merged experimental modules exist, all `publish = false`.

| Crate | Answers | Authority | HELM deps |
|---|---|---|---|
| `helm-app-spec` | what is **desired** | none; pure validation | none |
| `helm-observe` | what **actually exists** at explicit targets | caller-opened root descriptors | **none** |
| `helm-bind` | what follows from **comparing** the two | none; pure function | `helm-app-spec`, `helm-observe` |
| `helm-evidence` | is a declared **evidence bundle** complete and valid | read-only bundle directory | none |

Read in full for this design: `AGENTS.md` including the new **CI i disciplina push-a**
section, [project state](../PROJECT_STATE.md), [decisions index](../DECISIONS.md),
[ADR-0021](../adr/ADR-0021-second-product-module.md),
[ADR-0022](../adr/ADR-0022-observation-authority.md) with its cohort clarification,
[ADR-0023](../adr/ADR-0023-binding-authority.md),
[ADR-0005](../adr/ADR-0005-sandbox-boundary.md), every crate README and public module, the
[helm-observe architecture](HELM-OBSERVE-ARCHITECTURE.md), the
[helm-bind architecture](HELM-BIND-ARCHITECTURE.md), the
[OBS-FS-01 definition](../experiments/obs-fs-01/README.md) and
[execution report](../experiments/OBS-FS-01-EXECUTION-REPORT.md), and the workspace and crate
manifests.

### 1.1 Three facts from the current tree that constrain this design

**(a) The workspace forbids unsafe code, and `forbid` cannot be locally relaxed.**
`Cargo.toml` declares `[workspace.lints.rust] unsafe_code = "forbid"`, and every crate opts
in with `[lints] workspace = true`. An `#[allow(unsafe_code)]` **cannot** override `forbid`.
Any module needing unsafe must decline the workspace lint table and define its own. That is a
visible, reviewable act, not an accident — see [section 8](#8-the-unsafe-and-helper-question).

**(b) `rustix` is already vetted here, but does not cover this job.**
`helm-observe` pins `rustix = "=1.1.4"` with `default-features = false, features = ["std", "fs"]`.
Reading the vendored source directly:

| Needed primitive | rustix 1.1.4 status |
|---|---|
| `pidfd_open`, `pidfd_send_signal` | safe, `process` feature |
| `waitid` with `WaitId::PidFd(BorrowedFd)` | safe, `process` feature, Linux only |
| `setsid`, `kill_process`, `kill_process_group` | safe, `process` feature |
| `pipe_with(PipeFlags::CLOEXEC)` | safe, `pipe` feature |
| `dup2`, `fcntl_dupfd_cloexec` | safe, `io` |
| `execveat` | **`pub unsafe fn` in `src/runtime.rs`**, a `doc(hidden)` module whose header warns these "are not normal functions", that some "cannot be used in a process which also has a libc present", that "the safety requirements … are not fully documented", and that "the API for these functions is not considered stable" |
| `close_range` | **absent** — no wrapper at any feature level |

The two primitives this design depends on most are exactly the two rustix does not safely
provide. That is a design input, not a detail.

**(c) The desired entry point is a Windows path, not a Linux executable.**
`helm_app_spec::RelativeEntryPoint` is documented as an "inert prefix-relative Windows C-drive
entry-point spelling", beginning `drive_c/`. A generic Linux launcher cannot execute it and
must not pretend to. This single fact decides
[section 11](#11-relation-to-validatedappspec).

## 2. The fundamental safety boundary: data, intent, execution authority

Three things must never be confused, and 0.1 must make the confusion *inexpressible* rather
than merely discouraged.

```text
DATA          untrusted bytes; a LaunchPlan document; a BindingReport
INTENT        a validated LaunchPlan: "if someone were authorised, this is what would run"
AUTHORITY     an already-open executable descriptor a trusted caller deliberately handed over
```

**Parsing or validating a LaunchPlan grants zero execution authority.** `parse_launch_plan`
performs no I/O, opens nothing, resolves no name, and returns an inert value that cannot reach
`execveat` by any path. This mirrors `parse_plan` in `helm-observe` and `parse_binding_plan` in
`helm-bind`, both inert by construction.

**A `BindingReport` grants zero execution authority.** In particular:

> `Contradiction::NoClaimContradicted` **does not mean** ready, safe, permitted, compatible or
> launchable. It means no compared claim was a `Mismatch` of two known values.

That is not a stylistic warning; it is forced by `helm-bind`'s own accepted arithmetic.
Coverage in 0.1 is *necessarily* incomplete, because four mandatory semantic requirements —
source architecture, runtime family, Windows architecture, prefix role — have no comparator at
all, so `unsupported_binding >= 4` always. A caller can map two easy digests, leave everything
unobservable unmapped, and obtain `NoClaimContradicted`. Treating that as permission would
convert an honest "nothing contradicted among the little we compared" into "safe to execute".
**No current binding state may automatically authorize execution.**

Execution authority therefore comes from exactly one place: **a trusted caller passing an
already-open executable descriptor by value.** Authority is the descriptor — not a pathname,
not a name, not a verdict, not a policy field in a document.

```text
untrusted bytes ──parse_launch_plan──▶ ValidatedLaunchPlan     (intent, zero authority)
trusted caller  ──executable_from_fd──▶ ExecutableCapability   (authority, no intent)
plan ⊕ capability ⊕ context ──authorize──▶ AuthorizedLaunch    (the only executable thing)
AuthorizedLaunch ──launch──▶ LaunchReceipt                     (what actually happened)
```

Neither half alone can execute anything. `ValidatedLaunchPlan` holds no descriptor;
`ExecutableCapability` holds no argv, environment or timeout.

## 3. What "launch" is allowed to mean

`helm-launch` 0.1 may establish only these facts:

- an execution request was authorized by a trusted caller against an exact plan;
- an exact executable object was selected, with its measured size and SHA-256;
- process creation was attempted, and a direct child either was or was not created;
- `execveat` on that exact object succeeded, or failed with a specific error class;
- the **direct child** exited with a status, or was terminated by a signal;
- a timeout expired and the launcher issued a termination request;
- bounded stdout and stderr byte counts and digests were observed;
- a termination request was issued, and the direct child did or did not stop.

It must never infer, and its vocabulary must be unable to express: application worked ·
Windows software is compatible · installation is correct · runtime provenance is correct · a
UI appeared · a workflow succeeded · files were saved correctly · sandbox containment exists ·
the process tree was fully killed · rollback occurred · ready · pass · fail · verified ·
compatible.

The vocabulary rule is the one `helm-bind` reached after independent review: **no term the
module itself emits may be a verdict word.** Caller data echoed under a clearly non-semantic
key is not a verdict; a state named `Ok` or `Pass` in the launcher's own closed enumeration
is. `Exited { code: 0 }` is a process fact and reads as one.

## 4. The 0.1 product scope

Five designs, compared against the current tree.

| | Design | Verdict |
|---|---|---|
| **A** | Wine-specific launcher immediately | **Rejected** |
| **B** | Generic Linux exact-executable launch substrate; Wine adapter later | **Recommended** |
| **C** | Generic path-based `std::process::Command` launcher | **Rejected** |
| **D** | Split policy/orchestration from low-level execution into two crates now | **Rejected for 0.1** |
| **E** | Execution-plan validator only, with no execution at all | **Rejected** |

**A is rejected**, and the working hypothesis survives scrutiny for reasons traceable to this
repository rather than to taste:

1. **Wine runtime provenance is not established.** `helm-bind` binds a *runtime artifact body*
   — an archive's size and digest — and its README states a body match is "not that it was
   installed, not that it produced the current prefix, not that a current loader came from
   it". There is no evidenced path from a bound archive to the loader a launch would invoke.
2. **Dedicated prefix ownership is not established.** `asserted_prefix_root_id` is, per
   accepted ADR-0023, "a caller assertion, never an attestation". Launching *into* a prefix on
   the strength of an assertion would be the first place HELM turned an assertion into an act.
3. **Wine is multi-process by construction.** A `wineserver` outlives the process that started
   it, so one Wine process is not one application lifecycle. Direct-child lifecycle semantics —
   the only semantics 0.1 can honestly claim, see
   [section 24](#24-termination-and-the-process-tree) — do not describe a Wine session.
4. **Wine needs an environment and a prefix pathname contract** that does not yet exist and
   cannot be designed honestly without evidence; see
   [section 26](#26-prefix-capability-versus-wineprefix-pathname).
5. **GUI Wine needs a display and session environment**, an entire additional hidden-input
   surface.

**C is rejected** despite being least work. A pathname handed to `std::process::Command` is
re-resolved by the kernel at `execve` time, so the object that runs is whatever the name
denotes *then*, not the object that was inspected. That reopens the TOCTOU class
`helm-observe` was built to close, and makes an executable identity in a receipt unfounded.

**D is rejected for 0.1** on the repository's own precedent: one crate unless concrete evidence
demands more. There is no orchestration policy to separate — no accepted policy layer exists —
and a second crate invented now would be a home for exactly the "binding report means
permission" logic [section 12](#12-relation-to-bindingreport) rejects. The split becomes
justified when a real orchestrator exists.

**E is rejected** because a validator that never executes falsifies none of the process
semantics that make this module dangerous. The risk is in the execution, so the evidence must
come from execution.

> **Recommended 0.1 scope.** One crate, `helm-launch`. It executes **exactly one** explicitly
> authorized, already-open, regular ELF executable object on Linux x86_64, with a
> caller-declared argv, a closed environment contract, an explicit working-directory
> capability, an exact descriptor-inheritance invariant, bounded stdout/stderr observation, a
> monotonic timeout, and **direct-child lifecycle only**. It produces an identity-bearing
> `LaunchReceipt`. It is not a sandbox, does not manage a process tree, and contains no Wine,
> prefix, discovery, PATH, shell or policy logic of any kind.

## 5. Supported host cohort

The candidate mechanism needs these kernel primitives:

| Primitive | Purpose | Since |
|---|---|---|
| `execveat(fd, "", …, AT_EMPTY_PATH)` | execute the pinned object | Linux 3.19 |
| `pidfd_send_signal` | signal the direct child without PID-reuse risk | Linux 5.1 |
| `pidfd_open` | stable handle to the direct child | Linux 5.3 |
| `waitid(P_PIDFD, …)` | wait on that handle | Linux 5.4 |
| `close_range(first, last, 0)` | the descriptor-inheritance invariant | Linux 5.9 |

**Minimum kernel 5.9; architecture x86_64; operating system Linux.**

The honest cohort statement, in the shape ADR-0022's clarification established:

> The supported and to-be-evidenced 0.1 cohort is **Linux x86_64, kernel 5.9 or newer**.
> `helm-launch` does **not** attest the kernel version, the distribution, the mount
> configuration or the credentials it runs under; those are caller preconditions established
> outside the crate. The backend does not compile outside Linux x86_64, and no other Unix is
> claimed merely because `std::process` would compile there.

**API compile portability is separated from validated execution backend support**, exactly as
`helm-observe` does it: the plan parser, model, receipt serializer and error types compile and
are tested everywhere; the execution backend sits behind
`#[cfg(all(target_os = "linux", target_arch = "x86_64"))]` and exists nowhere else. A caller on
Windows or macOS can parse and inspect a plan and cannot obtain a launcher at all.

## 6. Executable authority

The trusted caller supplies an **already-open descriptor**, never a pathname to rediscover.

```rust
pub struct ExecutableCapability {
    fd: OwnedFd,       // O_RDONLY | O_CLOEXEC, owned, never re-derived from a name
    size: u64,         // fstat at admission
    sha256: Digest,    // measured through this same descriptor
    mode_bits: u32,    // recorded as a fact, never used as a gate
}

pub fn executable_from_fd(fd: OwnedFd) -> Result<ExecutableCapability, AdmissionError>;
```

There is deliberately **no path-based constructor**, matching `RootCapability` in
`helm-observe`, whose documentation states it exists so "the observer never resolves a name to
obtain authority".

**Must it be a regular file?** Yes. `fstat` must report `S_IFREG`. A directory, symlink, FIFO,
socket, device or `O_PATH`-only handle is refused. `O_PATH` is refused specifically because the
bytes cannot be read through it, and an unmeasurable executable defeats the identity the
receipt exists to carry.

**ELF only in 0.1?** Yes, checked by reading `7F 45 4C 46` through the same descriptor. This is
a deliberate narrowing, not a security check — valid ELF magic proves nothing about the
program. It exists to exclude interpreter scripts, and that exclusion buys something concrete:
**`execveat` on a `#!` script requires the kernel to hand the interpreter a `/dev/fd/N`
pathname, silently making script execution depend on procfs being mounted and on that
descriptor surviving into the interpreter.** Excluding scripts removes an entire hidden
dependency from the 0.1 contract. Documentation-derived; case **X2** in the experiment.

**Should scripts be rejected?** Yes in 0.1, for that reason. A future adapter wanting scripts
must first evidence the procfs and descriptor-survival behaviour.

**Should execute permission be checked before launch?** **No — recorded, not enforced.** Mode
bits are stored as a fact in the capability and the receipt. The launcher does not implement a
second, necessarily divergent permission model on top of the kernel's; `execveat` is the
authority and a permission failure arrives as typed `ExecFailed { errno: EACCES }`.
Pre-checking would add a race and would produce two different answers on the same object under
setuid, ACLs, `noexec` mounts or a MAC policy.

**Should the body SHA-256 be measured?** Yes, through the **same descriptor**, using positional
reads (`pread`) so no shared file offset is disturbed and no seek state is observable.

**How is the measurement bound to the exact object later executed?** By never letting go of the
descriptor. The capability owns the `OwnedFd`; `authorize` moves it into `AuthorizedLaunch`;
`launch` consumes that and hands **the same descriptor** to `execveat`. No pathname exists at
any point after admission. The receipt therefore asserts exactly: *these bytes were read
through this descriptor, and this descriptor was the exec target.*

**Does hashing affect atime or cache?** Yes, and it is stated rather than hidden. Reading
updates atime under default `relatime` and populates the page cache. `O_NOATIME` is not used
because it needs file ownership or `CAP_FOWNER`, and 0.1 must not require privileges. Measuring
is an act with observable side effects, and the README must say so.

**What descriptor mode is required?** `O_RDONLY | O_CLOEXEC`. Not `O_PATH` (unreadable), and
emphatically **not** writable: Linux `deny_write_access` makes `execveat` fail `ETXTBSY` while
any writable descriptor is open on the inode, so a writable capability would be a launcher that
cannot launch.

**What mount or filesystem claims are needed?** **None, and none are made.** Unlike
`helm-observe`, which admits an ext-family superblock magic as a necessary mechanism guard,
`helm-launch` asserts nothing about the filesystem holding the executable. A `noexec` mount
surfaces honestly as `ExecFailed { errno: EACCES }`. A filesystem guard here would invent an
attestation the module cannot back.

## 7. Executable TOCTOU: process-creation mechanisms compared

### A. `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`

| Property | Assessment |
|---|---|
| Same pinned object executed | **Yes.** The descriptor identifies the inode; no name is resolved at exec time |
| Pathname replacement after pin | **Irrelevant by construction** — there is no pathname |
| Descriptor inheritance requirement | The exec descriptor must survive until the syscall; `CLOEXEC` is correct, the kernel resolves it before replacing the image |
| CLOEXEC interaction | Desired: the descriptor does not appear in the new image |
| ELF vs script | ELF direct. `#!` makes the kernel synthesise `/dev/fd/N` for the interpreter → procfs dependency → **excluded in 0.1** |
| Dynamic loader | Normal `ld.so` behaviour for a dynamic ELF; the loader then resolves libraries by name, which is why loader-controlling environment variables are refused ([section 15](#15-environment-policy)) |
| procfs dependency | **None** for ELF |
| Mount namespace | None beyond the executable's own mount |
| Rust API safety | **Poor.** rustix exposes it only as `unsafe fn` in a `doc(hidden)` self-declared-unstable module; the realistic call is `libc::syscall(SYS_execveat, …)` |
| fork/multithread safety | The syscall is async-signal-safe; the surrounding child setup must be too |
| Control of inherited FDs | Complete, combined with `close_range` |
| Portability | Linux only |
| Error reporting | Exact errno over the exec-status pipe ([section 21](#21-spawn-versus-exec-confirmation)) |

### B. `fexecve(fd, argv, envp)`

Same intent, weaker guarantee. glibc implements `fexecve` **via `execveat` where available and
via `execve("/proc/self/fd/N", …)` otherwise**, so the mechanism silently degrades into option
C on a system without procfs or on an older kernel — and the caller cannot tell which happened.
A mechanism whose TOCTOU property depends on an invisible fallback is unacceptable for a module
whose entire point is pinning. **Rejected.**

### C. `execve("/proc/self/fd/N", …)`

Pins the correct inode — the magic link resolves to the open file, not a re-walked path — and
is reachable from safe Rust via `std::process::Command`. But it **hard-requires procfs**, breaks
in a mount namespace without `/proc`, exposes a synthetic pathname in `argv[0]` defaults and in
`/proc/<pid>/cmdline`, and makes correctness depend on a filesystem being mounted rather than on
a syscall contract. Acceptable as a documented **fallback** only if the experiment falsifies the
primary mechanism.

### D. `std::process::Command` with an ordinary pathname

**Rejected.** The kernel re-resolves the name at exec, so the executable identity in the receipt
would describe an object that did not necessarily run. This is precisely the falsifier "path
replacement changes the executed body". Convenience is not a justification.

### E. `posix_spawn` variants

Cannot exec a descriptor; it takes a pathname and so inherits D's defect or C's procfs
dependency. Its file actions (`addclose`, `adddup2`, glibc 2.34+ `addclosefrom_np`) are
attractive for descriptor hygiene, but Rust exposes **no** safe wrapper for file actions, so
reaching them needs the same unsafe FFI as the direct mechanism — without the exec-pinning
benefit. **Rejected as primary.**

### F. A small launch trampoline or helper binary

Moves fork/exec into a language where async-signal-safety after `fork` is idiomatic. But the
helper must itself **be found and executed**, reintroducing the pathname-discovery problem this
design exists to eliminate — unless embedded and executed from a `memfd`, which is strictly more
machinery than the syscall it replaces. It also adds a build-time C toolchain dependency, a
second product language, and a wire protocol whose parser is new attack surface. **Not
recommended**; see [section 8](#8-the-unsafe-and-helper-question).

### Recommendation

> **Primary: `fork()` + async-signal-safe child setup + `execveat(exec_fd, "", …,
> AT_EMPTY_PATH)`.** Documented fallback, only if the experiment falsifies the primary:
> `execve("/proc/self/fd/N", …)`, with the procfs dependency stated in the README and in the
> receipt's cohort statement.

**Stated plainly, as instructed:** the robust mechanism **cannot** be implemented in safe Rust
with current wrappers. rustix offers `execveat` only as an `unsafe fn` inside a `doc(hidden)`
module warning that its API is unstable and its safety requirements are not fully documented,
and offers no `close_range` at all. No safe crate in this workspace's dependency set closes that
gap. This is a fact about the ecosystem, not a preference, and it makes
[section 8](#8-the-unsafe-and-helper-question) a real decision.

## 8. The unsafe and helper question

`helm-observe` remained `#![forbid(unsafe_code)]`. **`helm-launch` cannot**, and the reason is
specific rather than general.

The child, between `fork` and `execveat`, must perform: `dup2` of three stdio descriptors,
`close_range` over everything else, `setpgid`, `fchdir` to the working-directory capability,
and `execveat`. Every one is a raw syscall — all async-signal-safe — but **no safe Rust wrapper
exists that can run arbitrary code in the child between fork and exec.**
`std::os::unix::process::CommandExt::pre_exec` is itself `unsafe fn`, and rustix has no
`close_range`.

| | Option | Trusted computing base | Verdict |
|---|---|---|---|
| **A** | Safe Rust wrappers only | Smallest — zero unsafe | **Cannot deliver the contract** |
| **B** | A tiny, isolated, heavily reviewed unsafe Linux backend | ~100 lines of unsafe, all syscalls, in one module, no allocation, no locks | **Recommended** |
| **C** | A C or Rust launch trampoline with a narrow protocol | A whole extra binary, its discovery, its build, its protocol parser | Rejected for 0.1 |

**Option A cannot deliver the contract.** The precise thing it cannot do is the exact
descriptor-inheritance invariant. `std::process::Command` sets `CLOEXEC` on the descriptors *it*
creates and `dup2`s the three stdio handles, but it does **not** close descriptors the host
process already holds without `CLOEXEC`. As the instruction warns, HELM opening its own
descriptors with `CLOEXEC` proves nothing about a host that opened a socket or a log file
without it. Marking every descriptor `CLOEXEC` from the parent instead is both racy in a
multithreaded host and an unacceptable side effect on descriptors HELM does not own.

So the choice is genuinely between **B** and **weakening the claim**. That trade must be the
owner's, and it is stated as such:

> **Owner decision required (D-1).** Either (i) permit a narrowly scoped `unsafe` backend in
> `helm-launch` only, so the exact descriptor-inheritance invariant can be claimed and tested;
> or (ii) keep `forbid(unsafe_code)` and **downgrade the published claim** to "descriptors
> `helm-launch` itself opens are `CLOEXEC`; descriptors the host process already holds without
> `CLOEXEC` may be inherited by the child", with experiment case **F2** expected to *fail* and
> that failure documented as a standing limitation.

**Option C is rejected for 0.1** because it does not solve a concrete process-creation safety
issue that B leaves open — it relocates the same syscalls into a binary that must then be found
and executed, which is the discovery problem this design exists to remove. The instruction is
explicit that a helper needs a concrete justification; there is none here.

If B is chosen, the mechanical constraints are part of the architecture, not implementation
detail:

- `helm-launch` **does not** use `[lints] workspace = true`; it declares its own `[lints]` table
  with `unsafe_code = "deny"` and a single `#[allow(unsafe_code)]` on one module, because
  `forbid` at workspace level cannot be overridden locally;
- the unsafe region is one file, contains no allocation, no `Vec` growth, no locking, no
  formatting and no panic path — every C string, the `argv` pointer array and the `envp` pointer
  array are fully materialised **before** `fork`;
- the region's only exits are `execveat` and `_exit`;
- it is covered by the strace-level evidence of LAUNCH-EXEC-01 rather than by assertion.

## 9. No shell, ever

0.1 must never construct `sh -c`, `bash -c`, `cmd /C`, a PowerShell invocation or any shell
command string. There is no command-string parser and no quoting logic anywhere, so no quoting
logic can be relied upon as a security control.

The design is `ExecutableCapability` + `argv[]`, where each argument is an independent bounded
value carried as its own JSON string in the plan and its own NUL-terminated C string in the
`argv` array. A literal argument containing `;` `|` `&` `$( )` spaces, quotes or backticks stays
**one literal argument**, because nothing ever concatenates arguments into a string. Cases
**A1** and **A2** exist to prove that rather than assert it.

## 10. The `LaunchPlan`

**An inert, exact-byte `LaunchPlan` is required.** The hypothesis holds for the same reason it
held for `helm-bind`'s mapping: the document materially decides what runs — argv, environment,
timeout, capture policy, termination policy — so a receipt that did not bind it would not be
reproducible from its own recorded inputs. **Exact-byte SHA-256 identity is required**, over the
supplied bytes including whitespace and field order, with no canonicalisation, exactly as
`helm-app-spec` and `helm-bind` do it.

Sketch, deliberately not copied from the previous schemas:

```json
{
  "schema": "helm-launch-plan",
  "version": "0.1",
  "subject_spec_sha256": "…",          // optional context, never authority
  "binding_report_sha256": "…",        // optional context, never authority
  "execution_kind": "linux_exact_executable",
  "argv": ["app", "--flag", "a b; c"],
  "environment": { "mode": "empty" },
  "working_directory": { "capability_id": "workdir" },
  "stdin": { "mode": "closed_pipe_eof" },
  "stdout": { "mode": "measure", "max_capture_bytes": 0 },
  "stderr": { "mode": "measure", "max_capture_bytes": 0 },
  "timeout_ms": 30000,
  "termination": { "signal": "SIGTERM", "grace_ms": 5000, "then": "SIGKILL" }
}
```

It must contain **no** numeric host descriptor, **no** ambient executable pathname, **no** shell
command, **no** arbitrary host root path, and **no** automatic success criterion. There is no
`expected_exit_code` field and no `success_if` field, deliberately: the moment a plan can
declare what counts as success, the launcher acquires a verdict it has no authority to issue.
`execution_kind` is a closed vocabulary whose only 0.1 member is `linux_exact_executable`, so a
future Wine adapter is a *new* member added under a *new* accepted decision, not a
reinterpretation of this one.

## 11. Relation to `ValidatedAppSpec`

**Recommendation: `helm-launch` 0.1 does not take `ValidatedAppSpec` as a functional input.**

The decisive fact is [section 1.1(c)](#11-three-facts-from-the-current-tree-that-constrain-this-design):
`EntryPointRequirement::path()` returns a `RelativeEntryPoint` documented as a
"prefix-relative Windows C-drive entry-point spelling" beginning `drive_c/`. A generic Linux
launch substrate cannot execute that path, and 0.1 explicitly does not do Wine. So the two
proposed benefits evaluate as:

- *derive the entry-point argument from the desired document rather than duplicating it* — there
  is nothing to derive in 0.1, because the desired entry point is not a Linux executable;
- *prevent a caller's plan from silently naming another entry point* — already impossible, and
  by a stronger mechanism: the plan names **no** executable at all. The executable is the
  descriptor, and the descriptor comes from the trusted caller.

The costs are real: taking `ValidatedAppSpec` would make a generic execution substrate
app-specific, and would pull `helm-app-spec` into the dependency graph for no functional gain.

The compromise keeps the identity benefit without the coupling: the plan may carry
`subject_spec_sha256` as an **opaque digest**, and the receipt records it. Consistency between
that digest and a real specification is the orchestration layer's job. Nothing in `helm-launch`
parses, interprets or depends on a specification. **Nothing is duplicated**, because a digest is
not a copy of the entry-point data.

## 12. Relation to `BindingReport`

Three designs:

| | Design | Verdict |
|---|---|---|
| **A** | `BindingReport` required as input, digest recorded, semantics never granting authority | Rejected for 0.1 — see below |
| **B** | Not an input to low-level launch; the orchestration layer associates it | **Recommended** |
| **C** | Launcher interprets `BindingReport` as launch policy | **Rejected outright** |

**C is rejected**, as instructed and independently: no accepted policy layer exists, and
`NoClaimContradicted` is reachable with `unsupported_binding >= 4` and almost nothing mapped.

**B is recommended over A** for a reason that emerged while working through A. Under A the
crate would depend on `helm-bind`, which transitively pulls in `helm-app-spec` **and
`helm-observe`** — and `helm-observe` is where `RootCapability` lives. Placing a read-authority
capability type in the launcher's dependency graph is precisely the accident
[section 35](#35-dependency-direction) warns against, and it buys nothing: everything
`helm-launch` needs from a report is two 32-byte digests.

So the rule, stated explicitly:

> **Comparison context is not execution authorization.** `helm-launch` 0.1 accepts
> `binding_report_sha256` as an **opaque digest** in the plan and records it in the receipt. It
> never receives a `BindingReport` value, never links `helm-bind`, and therefore contains no
> code path that can read `Contradiction` or `Coverage`. Reading a verdict is not "discouraged";
> it is **unavailable**.

That is impossibility by construction rather than by review, and it is strictly stronger than
option A's discipline. The orchestration layer, when one is accepted, is responsible for
checking that the report's `inputs().spec_sha256` matches the plan's `subject_spec_sha256`; the
launcher records both digests so that check is auditable after the fact.

## 13. The authorization object

```rust
pub fn parse_launch_plan(bytes: &[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors>;

pub fn executable_from_fd(fd: OwnedFd) -> Result<ExecutableCapability, AdmissionError>;
pub fn directory_from_fd(id: &str, fd: OwnedFd) -> Result<DirectoryCapability, AdmissionError>;

pub fn authorize(
    plan: ValidatedLaunchPlan,          // consumed
    executable: ExecutableCapability,   // consumed
    working_directory: DirectoryCapability, // consumed
) -> Result<AuthorizedLaunch, LaunchRefusal>;

pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchReceipt, LaunchError>; // consumed
```

The invariant is enforced by ownership, following `helm-observe`'s obligation-C pattern, whose
documentation notes `authorize` "consumes the `ValidatedPlan` … There is no public way to
express *authorise plan A, observe plan B*".

- **Authorize executable A, then execute B** — inexpressible. `authorize` consumes the
  capability by value and `launch` takes no executable argument.
- **Authorize plan A, then execute plan B** — inexpressible. `launch` takes no plan argument.
- **Add or substitute capabilities after authorization** — inexpressible. `AuthorizedLaunch` has
  only private fields, no public constructor, no setters, no `Default`, no `Deserialize`, and no
  `From`. A `compile_fail` doctest per type, as `helm-bind` does, keeps that true.

One deliberate strengthening of the instruction's sketch: **`launch` consumes
`AuthorizedLaunch` by value rather than borrowing it.** With `&AuthorizedLaunch`, one
authorization could be replayed into many executions, so the count of executions a caller
authorized would not equal the count that occurred. By value, one authorization means at most
one execution attempt, which is also what makes the receipt's relationship to the authorization
one-to-one.

## 14. Argument identity

**Is `argv[0]` caller-controlled or launcher-fixed?** **The plan must declare it explicitly**,
as `argv[0]`, with no default. The launcher has no honest value to invent: there is no pathname
after admission, so any synthesised `argv[0]` would be fabricated data in a receipt-bearing
system. Requiring it makes fabrication impossible and keeps the value where it belongs — caller
data with no semantic authority, recorded only via the plan digest.

- **NUL bytes** are rejected in every argument at parse time. `execve` cannot represent them, so
  accepting one would mean silently truncating an argument — a vulnerability, not a convenience.
- **Encoding:** UTF-8 without NUL, in 0.1. Unix `argv` is bytes, and this is a real narrowing:
  non-UTF-8 arguments are **unrepresentable in a 0.1 plan**. That is preferred over inventing a
  base64 escape layer that would complicate exact-byte identity for a case no current HELM
  workflow needs. It is recorded as a limitation, not hidden.
- **Bounds** are in [section 33](#33-resource-bounds).
- **The receipt does not reproduce argument values.** It records the plan SHA-256, which fully
  determines argv, plus the argument count. That satisfies identity and avoids copying possibly
  sensitive caller data into an artifact.

## 15. Environment policy

| | Policy | Verdict |
|---|---|---|
| **A** | Inherit the whole host environment | **Rejected** |
| **B** | Clear, then allow explicit key/value pairs | **Recommended, with a deny rule** |
| **C** | Closed named pass-through list from the host | Rejected for 0.1 |
| **D** | Adapter-defined environment profiles | Deferred to the Wine adapter |

**A is rejected.** Ambient inheritance is the largest hidden input available: it would make the
same plan behave differently on two machines, and it would silently carry `LD_PRELOAD`,
`LD_LIBRARY_PATH`, proxies, locale and credentials into the child.

**C is rejected for 0.1** because a pass-through list still reads host state, so the executed
semantics still depend on the machine; the plan would no longer determine the run.

**Recommendation.** Two modes: `{"mode": "empty"}` — the recommended default and the only mode
the first synthetic cohort needs — and `{"mode": "explicit", "entries": [{"name", "value"}]}`,
where the entries are the **complete** environment. Nothing is inherited in either mode.

Because environment variables change *which code the pinned executable actually loads*, the
following are **refused at parse time** in 0.1: any name beginning `LD_` (covering `LD_PRELOAD`,
`LD_LIBRARY_PATH`, `LD_AUDIT`), plus any name that is empty, contains `=`, or contains NUL.
Refusing them protects the one thing this module claims: that the object it measured is the
object that ran. `LD_PRELOAD` would let a caller with a valid plan run arbitrary other code
inside the measured process image, so accepting it would make the executable identity in the
receipt misleading. `PATH` is *allowed* as an explicit value, and it is recorded that it has no
effect on which executable `helm-launch` runs — the launcher uses a descriptor — and affects
only what the child itself may later exec. Case **V3** checks the policy behaves as written.

Wine's eventual needs — `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`, `WINEPREFIX` and others
— are **explicitly not** smuggled into this contract. No name is special-cased for Wine, and no
0.1 field anticipates one.

## 16. Working directory

An inherited ambient cwd is a hidden input of the same class as the environment, and the
instruction is explicit that it must not silently become part of execution semantics.

| Option | Verdict |
|---|---|
| No cwd contract (inherit ambient) | Rejected — hidden input |
| Fixed known cwd chosen by the launcher | Rejected — the launcher would be inventing a host path |
| Pathname cwd in the plan | Rejected — reintroduces ambient pathname resolution and TOCTOU |
| **Caller-supplied opened directory capability** | **Recommended** |

**Recommendation: `working_directory` is mandatory and is a `DirectoryCapability`** — an
`OwnedFd` on a directory plus a logical id the plan names. The child performs `fchdir(dir_fd)`
before exec. There is no pathname and no ambient cwd anywhere in the contract: a caller who does
not care must still supply a directory descriptor, and that act is visible.

`fchdir` in the child is part of the unsafe child-setup region, so it is covered by
[section 8](#8-the-unsafe-and-helper-question) rather than adding a new mechanism.

## 17. File-descriptor inheritance

The invariant, stated as a claim that must be falsifiable:

> **Exactly descriptors 0, 1 and 2 survive into the executed program**, bound to the
> launcher-supplied stdin, stdout and stderr endpoints. No other descriptor of the host process
> — `CLOEXEC` or not, owned by HELM or not — is present in the new image.

The mechanism, in child order:

1. `dup2` the three prepared endpoints onto 0, 1, 2 (`dup2` clears `CLOEXEC` on the new
   descriptor, which is exactly what is wanted for stdio);
2. `close_range` over everything above 2 **except** the two descriptors still needed — the exec
   descriptor and the exec-status pipe write end — which requires up to three `close_range` calls
   over the gaps between them;
3. `setpgid(0, 0)`; `fchdir(dir_fd)`;
4. `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`.

Both surviving descriptors are `CLOEXEC`, so a successful exec removes them and the invariant
holds in the new image. Step 2 is the load-bearing one, and it is the step
[section 8](#8-the-unsafe-and-helper-question) shows is unreachable from safe Rust.

Considered and rejected: `closefrom` variants (not a Linux syscall; the BSD/glibc wrapper reads
`/proc/self/fd`, adding a procfs dependency and a directory walk in the child);
`CLOSE_RANGE_CLOEXEC` (Linux 5.11, would raise the kernel floor for no benefit here);
parent-side marking of every descriptor `CLOEXEC` (racy in a multithreaded host, and an
unacceptable side effect on descriptors HELM does not own). Descriptor-number reuse is not a
hazard for the invariant because the child closes by *range*, not by remembered number.

**The claim is not made on the strength of HELM's own `CLOEXEC` hygiene.** Case **F2** exists
precisely because a host process may already hold a non-`CLOEXEC` descriptor: the harness
deliberately creates one before launch and the child must not see it. If option (ii) of decision
**D-1** is taken, F2 is expected to fail and the claim above must be replaced by the weaker one
stated there.

## 18. Standard input

Interactive stdin must never be inherited implicitly: it would let a child consume the host's
terminal input and would make behaviour depend on whether a terminal exists.

| Option | Verdict |
|---|---|
| **Pipe whose write end the parent closes immediately → EOF** | **Recommended, only mode in 0.1** |
| Caller-supplied stdin capability | Deferred; no 0.1 need |
| Bounded supplied bytes | Deferred; adds a writer and a second deadlock surface |
| `/dev/null` opened by path | Rejected — an ambient pathname lookup for no benefit |

The child sees an immediate EOF on descriptor 0. Interactive console and GUI input are out of
scope and no 0.1 field anticipates them.

## 19. Standard output and standard error

Two **separate** pipes, never merged: merging would destroy the distinction between a program's
results and its diagnostics, and that distinction is a fact worth recording.

**Draining is concurrent and deadlock-free by construction.** The parent runs a single-threaded
`poll()` loop over the stdout read end, the stderr read end and the exec-status pipe, with the
poll timeout computed from a `CLOCK_MONOTONIC` deadline. Reading one stream to completion before
the other — the classic bug — is impossible because no read is issued except on a descriptor
`poll` reported ready. A thread pool is deliberately avoided: threads in the parent complicate
the fork-safety argument in [section 8](#8-the-unsafe-and-helper-question) for no gain.

Per-stream modes, chosen in the plan:

| Mode | Receipt records | Bytes retained |
|---|---|---|
| `discard` | exact drained byte count | none |
| `measure` | exact drained byte count + SHA-256 of drained bytes | none |
| `capture_prefix` | count + SHA-256 + `truncated` flag | first `max_capture_bytes`, **in memory only** |

**Draining continues after the capture bound is reached**, discarding the excess, so a child that
writes more than the bound is never blocked by a full pipe. Capture is binary-safe: counts and
digests are over raw bytes with no encoding assumption. A read failure is a typed
`OutputCaptureFailed { stream, errno_class }` and is never silently reported as "no output".
If the deadline expires mid-stream the counts are marked partial rather than presented as final.

## 20. Output privacy

Child output may contain host paths, usernames, environment values, secrets or document
contents.

| | Design | Verdict |
|---|---|---|
| **A** | Receipt stores full bounded bytes | Rejected — turns arbitrary child output into an artifact |
| **B** | Receipt stores length + SHA-256 only | **Recommended for the receipt** |
| **C** | Receipt stores nothing unless requested | Weaker: loses the ability to detect that output changed |
| **D** | Separate private execution log versus public receipt | **Recommended for the bytes** |

**Recommendation: B for the receipt, D for the bytes.** The `LaunchReceipt` — the artifact
another layer may reference by digest — contains per stream only `{ bytes_drained, sha256,
truncated }`. Captured prefix bytes, when `capture_prefix` was requested, are returned in the
in-memory `LaunchOutcome` value and are **never** serialised into the receipt. A caller that
wants to keep them makes that decision, and its own privacy call, explicitly.

Stated bluntly, because it is easy to get wrong: **hashing the output does not prevent the child
from having observed secrets.** The child ran with the caller's credentials and could read
anything the caller can read. Output handling is about not *republishing* secrets in a HELM
artifact; it is not a confidentiality control.

## 21. Spawn versus exec confirmation

"A process object was created" and "the target executable was successfully executed" are
different facts, and conflating them would let `ENOENT` be reported as "launched, then exited
127".

The mechanism is the standard `CLOEXEC` exec-status pipe, created with
`pipe_with(PipeFlags::CLOEXEC)`:

- the child, on any pre-exec setup failure or on `execveat` failure, writes one fixed-size
  record `{ stage: u8, errno: i32 }` to the write end and calls `_exit(127)`;
- on a successful exec the write end is closed by the kernel because it is `CLOEXEC`, so the
  parent observes EOF with no record.

What the parent may then conclude, and nothing more:

| Parent observes | Conclusion |
|---|---|
| a complete record | `ExecFailed { stage, errno_class }` — a child existed, the target never ran |
| clean EOF, no record | exec succeeded; the executed image is the pinned object |
| EOF plus a short or partial record | `ExecStatusIndeterminate` — recorded as such, never guessed |

`ENOENT`, `EACCES`, `ENOEXEC`, `ENOMEM`, `E2BIG`, `ETXTBSY` and `EPERM` therefore arrive as
distinct `ExecFailed` error classes and can never be presented as an exit status.

## 22. Process identity

**Use a `pidfd`.** A numeric PID is reusable and is therefore unsuitable as long-lived
authority; a `pidfd` refers to one process for its whole lifetime.

`pidfd_open(pid)` immediately after `fork` is sound here, and the reasoning is worth recording:
the launcher never reaps the child before opening the pidfd, so the child is at worst a zombie
and its PID cannot have been recycled in the interval. Atomic `CLONE_PIDFD` via `clone3` would
remove even that reasoning step, at the cost of a second, more complex unsafe call site; it is
recorded as a possible later refinement, not a 0.1 requirement.

Signalling uses `pidfd_send_signal`, and waiting uses `waitid` with `WaitId::PidFd`, both of
which rustix already exposes safely.

**The receipt does not expose a numeric PID.** It is neither stable identity nor a fact any
consumer needs, and it is mild host information. The receipt records that a direct child was
created and what became of it.

## 23. Timeout

- **Clock:** `CLOCK_MONOTONIC` only. Never the wall clock, which can step.
- **When it starts:** after **confirmed exec**, that is, when the exec-status pipe reports EOF.
  Charging process setup to the program's budget would misreport what timed out. The pre-exec
  phase has its own separate fixed bound (`SPAWN_CONFIRM_TIMEOUT_MS`, 5000), so neither phase is
  unbounded.
- **Bounds:** `timeout_ms` in `[1, 600_000]`; `grace_ms` in `[0, 60_000]`.
- **On expiry:** issue the plan's termination signal to the direct child via `pidfd_send_signal`,
  wait up to `grace_ms`, then `SIGKILL`, then wait.
- **Classification**, and this is the part that must not blur:

| Situation | Receipt outcome |
|---|---|
| Child exited before the deadline | `Exited { code }` |
| Child killed by a signal the launcher did **not** send | `Signaled { signal, by_launcher: false }` |
| Deadline expired; child exited during grace | `TimedOut { disposition: ExitedDuringGrace { code } }` |
| Deadline expired; child killed by the launcher | `TimedOut { disposition: KilledByLauncher { signal } }` |
| Deadline expired; child still alive after `SIGKILL` and wait | `TerminationFailed` |

The launcher always knows which case applies because it knows whether it issued the signal.

**No determinism is claimed.** Timing depends on scheduling and load. Consistent with
`helm-observe` and `helm-bind`, the receipt carries **no wall-clock timestamp and no elapsed
duration** — the outcome is a fact, the duration is host timing that would add variance and a
mild side channel to an artifact meant to be compared by digest. Elapsed time remains available
in the in-memory `LaunchOutcome` for a caller that wants it.

## 24. Termination and the process tree

This is the section that decides Wine, so it is stated conservatively.

| Mechanism | What it does | What it does **not** guarantee |
|---|---|---|
| Kill the direct child (`pidfd_send_signal`) | Ends exactly one process | Nothing about any descendant |
| Kill the process group | Signals current group members | A child may `setsid`/`setpgid` out of the group before or after |
| New session (`setsid`) | Detaches from the launcher's terminal | Does not prevent a descendant creating its own session |
| cgroup v2 delegated subtree | **Real containment**: `cgroup.kill` ends every member | Requires a delegated writable subtree, which 0.1 does not have and must not create |
| `PR_SET_PDEATHSIG` | Signals the child when the *launcher* dies | Applies to the direct child only; does not help kill descendants |
| Subreaper (`PR_SET_CHILD_SUBREAPER`) | Lets the launcher reap orphaned descendants | Gives **visibility**, not control |

A process can fork, `setsid`, change process group, daemonise, or hand its work to an already
running service. None of the mechanisms available to an unprivileged 0.1 prevents that.

> **Recommended 0.1 claim: direct-child lifecycle only.** The child is placed in its own process
> group with `setpgid(0, 0)`, which gives a best-effort sweep target and stops the child
> receiving the launcher's terminal signals. Termination targets the **direct child** by
> `pidfd`. A process-group sweep may additionally be issued and is recorded as
> `group_sweep_issued: true` with **no** claim about descendants. `helm-launch` 0.1 provides
> **no process-tree containment**, and the receipt vocabulary contains no word that could be
> read as containment.

The hypothesis in the instruction therefore survives testing, and case **P1** is designed to
*demonstrate the limitation rather than to pass*: a helper forks a descendant that outlives its
parent, and the experiment records whether the descendant survives direct-child termination. If
it survives — which is expected — that result is preserved as the standing evidence for the
non-claim. cgroup v2 delegation is named as the future path to real containment and is out of
0.1 scope.

## 25. Wineserver and Wine-specific blockers — paper analysis only

**No Wine was executed and no A0 evidence was opened for this analysis.** What follows is
derived from Wine's documented architecture and from what this repository has and has not
established.

A Wine adapter would need, beyond generic launch 0.1:

1. **A multi-process lifecycle model.** `wineserver` is a per-prefix daemon started on demand
   that **outlives** the process that started it, so "the launched process exited" says nothing
   about whether the application session ended. Direct-child semantics do not describe this.
2. **A defined server shutdown authority.** Ending a session means something like
   `wineserver -k` or `wineboot -s` for a specific prefix — which is *another* execution,
   requiring its own executable capability and its own authorization, not a side effect of
   terminating the first child.
3. **An environment contract.** At minimum `WINEPREFIX`, plus `DISPLAY` or `WAYLAND_DISPLAY` and
   `XDG_RUNTIME_DIR` for anything with a UI. Each is a hidden input that
   [section 15](#15-environment-policy) currently refuses to guess at.
4. **A prefix pathname contract** — see [section 26](#26-prefix-capability-versus-wineprefix-pathname).
5. **Loader provenance.** `helm-bind` can bind a runtime *archive* body. Nothing establishes that
   the `wine` loader actually executed came from that archive. Calling an archive digest "loader
   provenance" is an explicit falsifier.
6. **Its own experiment.** None of the above is settled by LAUNCH-EXEC-01, which deliberately
   answers a narrower question.

**One Wine process is not one application lifecycle**, and 0.1 must not be extended to pretend
otherwise.

## 26. Prefix capability versus `WINEPREFIX` pathname

HELM prefers opened capabilities over ambient paths. Wine expects a prefix as a **pathname** in
`WINEPREFIX`. That is a genuine architectural collision, and it is not resolved here.

| Option | Analysis |
|---|---|
| Caller-supplied host prefix pathname | Simple, and abandons the capability model exactly where the stakes are highest: the path may be replaced between authorization and use |
| `/proc/self/fd/N` directory magic link | Discussed below; **unresolved** |
| Another capability-to-path bridge | No mechanism identified that Wine would accept |
| Staging or mount namespace | Real, but needs privileges or user namespaces — outside 0.1 and probably outside a future 0.2 |
| A future helper that holds the descriptor and re-exports a stable path | Speculative |

The `/proc/self/fd/N` option raises four questions this design cannot answer without evidence:

1. **Does the descriptor survive exec?** Only if it is deliberately *not* `CLOEXEC` — which
   directly contradicts the exact descriptor-inheritance invariant of
   [section 17](#17-file-descriptor-inheritance). A Wine adapter would need a declared, audited
   exception, not a quiet one.
2. **What happens for grandchildren?** `/proc/self/fd/N` resolves **per process**. `wineserver`
   and other Wine processes that do not inherit descriptor N would resolve the same string to
   something else or to nothing. A prefix path that means different things in different
   processes of one application is worse than a plain path.
3. **Does Wine canonicalise or reject it?** If Wine calls `realpath` on `WINEPREFIX`, the magic
   link resolves to the underlying pathname and the capability property evaporates; if Wine
   validates the prefix layout through the string, behaviour is unknown.
4. **What happens if the original path is renamed or replaced?** The descriptor still refers to
   the same directory inode, but a canonicalised path would report the new name — so the two
   models disagree exactly when it matters.

> **This needs a separate future experiment (provisionally LAUNCH-WINE-PREFIX-01), not an
> assumption.** It is explicitly out of scope for 0.1 and for LAUNCH-EXEC-01.

## 27. The sandbox boundary

A launch mechanism is **not** a sandbox, and 0.1 must say so bluntly in the README, in the ADR
and in the receipt's own documentation.

`helm-launch` 0.1 provides **no** filesystem isolation, **no** network isolation, **no** process
isolation, **no** registry isolation, **no** device isolation and **no** user-data protection.

The direct child runs with the **caller's own OS credentials** and can generally do anything the
caller can do: read and write the user's home directory, open sockets, connect to the display
server and session bus, and execute other programs. Clearing the environment, controlling argv
and closing descriptors reduces *accidental* inputs; none of it constrains what the child may
reach on its own initiative. Proposed [ADR-0005](../adr/ADR-0005-sandbox-boundary.md) already
records the related principle that a prefix is not a security boundary; the same holds, more
sharply, for a launcher.

Anything resembling containment — namespaces, seccomp, landlock, cgroup limits, a MAC profile —
is a separate module with its own architecture, its own evidence and its own accepted decision.

## 28. User credentials and privilege

0.1 **does not elevate privileges**. No `sudo`, no setuid helper, no file capabilities, no
`CAP_*` acquisition, no namespace creation, no seccomp or AppArmor manipulation. The child
executes under the same uid, gid, supplementary groups, rlimits and MAC context as the calling
process.

`O_NOATIME` is not used for the same reason: it would require ownership or `CAP_FOWNER`.

**"No elevation" is not sandboxing.** It means HELM adds no privilege; it does not mean the
child is confined, because the caller's own privileges may already be broad.

## 29. The `LaunchReceipt`

**It should be an exact identity-bearing artifact**, and the hypothesis holds for the same
reason it held for `BindingReport`: a later layer may need to refer to exactly one execution.

Identity inputs, all recorded inside the receipt bytes:

- `launch_plan_sha256` — determines argv, environment, timeout, capture and termination policy;
- the executable's observed identity: `size` and `sha256`, plus recorded `mode_bits`;
- `subject_spec_sha256`, when the plan carried one — opaque context;
- `binding_report_sha256`, when the plan carried one — opaque context;
- the execution outcome ([section 30](#30-receipt-vocabulary-and-when-a-receipt-exists));
- per-stream `{ bytes_drained, sha256, truncated }` for stdout and stderr;
- the cohort statement: `linux_exact_executable`, and the mechanism actually used.

`receipt_sha256 = SHA256(exact receipt bytes)`, over deterministic bytes with fixed field order
and no map iteration, exactly as `BindingReport` does it. No timestamp, hostname, PID, host path,
random identifier or pointer value appears anywhere.

**Execution is nondeterministic, so the same plan will not produce the same receipt**, and that
is normal rather than a defect. The receipt is reproducible *as a document* — its bytes are a
function of the recorded facts — but it is emphatically **not** a claim that the behaviour is
reproducible. Nothing in HELM may treat two matching receipt digests as evidence of
deterministic execution.

## 30. Receipt vocabulary, and when a receipt exists

The instruction's question — whether an authorization refusal should produce a receipt — is
decided by following `helm-bind`'s refusal rule, which independent review confirmed:

> **Authorization refusal produces no receipt.** `authorize` returns `Err(LaunchRefusal)`: no
> receipt bytes, no receipt digest, no execution facts. Nothing was executed, so manufacturing
> an execution artifact would be a lie of exactly the kind ADR-0023 forbids.
>
> **Once a direct child exists, a receipt always exists**, including when `execveat` failed. A
> real attempt occurred and its outcome is a fact worth recording.

Between those two lies one narrow case: `launch` was called but no child was ever created — for
example `fork` or `pipe` failed. That is `Err(LaunchError::NoAttempt { … })` with **no** receipt,
because no execution was attempted.

The outcome vocabulary, all process facts and no application semantics:

```text
ExecFailed { stage, errno_class }          the child existed; the target never ran
ExecStatusIndeterminate                    exec status could not be established; not guessed
Exited { code }                            the direct child exited with this status
Signaled { signal, by_launcher: false }    ended by a signal the launcher did not send
TimedOut { disposition }                   the deadline expired and the launcher acted
TerminationFailed                          the direct child did not stop after SIGKILL and wait
OutputCaptureFailed { stream, errno_class} a stream could not be drained
```

There is no `PASS`, `FAIL`, `OK`, `SUCCESS`, `WORKED`, `COMPATIBLE` or `READY` anywhere, and no
field from which such a value could be derived — there is no function that maps an outcome onto
a boolean.

## 31. Exit status

Exit code 0 means exactly one thing: **the direct process exited with status 0.**

It does not mean the application workflow passed, that the app launched correctly, that
compatibility succeeded, or that any user-visible UI appeared. Many programs exit 0 after doing
nothing, after printing a usage message, or after a GUI failed to open. Signal termination is
likewise only a process fact: `Signaled { SIGSEGV }` says a signal ended the process, not that
the application is broken in any particular way, and certainly not that the platform is
incompatible.

## 32. Executable identity and mutation

The chain is: **pin the descriptor → `fstat` metadata → read and hash through that same
descriptor → exec that same descriptor.** No pathname is ever consulted after admission, so
path replacement, rename and unlink are all irrelevant to *which object runs* — cases **E2**,
**E3** and **E4** exist to establish that rather than assume it.

Mutation of the object itself is the honest residual risk, and it is stated rather than papered
over. Two documentation-derived properties bear on it:

- Linux `deny_write_access` causes `execveat` to fail with `ETXTBSY` if **any** writable
  descriptor is open on the inode at exec time, and blocks new writable opens while the image is
  executing;
- neither property freezes the file's contents during the window **between** hashing and exec.

> **`helm-launch` does not promise that the executed bytes equal the hashed bytes.** It promises
> that the hashed object and the executed object are **the same inode, reached through the same
> descriptor**, and it records the measurement. Anyone able to write to that inode in the window
> between measurement and exec — which requires write access to the file — can invalidate the
> content claim, and `ETXTBSY` behaviour means such a writer usually breaks the exec instead.

This must be verified, not asserted: the mutation cases are part of the E series and their
outcome may force this paragraph to be rewritten before implementation.

## 33. Resource bounds

| Bound | Value | Rationale |
|---|---|---|
| `MAX_PLAN_BYTES` | 32 KiB | Larger than any honest plan; checked before parsing or hashing |
| `MAX_JSON_DEPTH` | 6 | The schema's own nesting plus slack |
| `MAX_ARGS` | 64 | Includes `argv[0]` |
| `MAX_ARG_BYTES` | 4 KiB | Well under the kernel's 128 KiB `MAX_ARG_STRLEN` |
| `MAX_ARGV_TOTAL_BYTES` | 128 KiB | |
| `MAX_ENV_ENTRIES` | 32 | |
| `MAX_ENV_NAME_BYTES` / `MAX_ENV_VALUE_BYTES` | 256 / 4 KiB | |
| `MAX_ENV_TOTAL_BYTES` | 32 KiB | |
| `MAX_ARGV_ENV_COMBINED_BYTES` | 192 KiB | Kept far below a typical 2 MiB `ARG_MAX` so `E2BIG` is a plan error, not a runtime surprise |
| `timeout_ms` | 1 … 600 000 | |
| `grace_ms` | 0 … 60 000 | |
| `SPAWN_CONFIRM_TIMEOUT_MS` | 5 000 | Bounds the pre-exec phase separately |
| `MAX_CAPTURE_BYTES` per stream | 64 KiB | Retained only in memory, never in the receipt |
| `MAX_RECEIPT_BYTES` | 8 KiB | The receipt holds digests and counts, not payloads |

No buffer is ever sized from an untrusted declared value: capture buffers are allocated at the
fixed bound, and draining past the bound discards rather than grows. There is no unbounded
`read_to_end` anywhere.

## 34. Error and refusal taxonomy

Four disjoint families, so that "launch failed" is never a single undifferentiated outcome.

**1. `LaunchPlanErrors`** — the document is invalid. `SchemaUnknown`, `VersionUnsupported`,
`JsonInvalid`, `DuplicateKey`, `DepthExceeded`, `PlanTooLarge`, `ExecutionKindUnsupported`,
`ArgvEmpty`, `ArgvTooMany`, `ArgTooLong`, `ArgContainsNul`, `ArgNotUtf8`, `EnvNameInvalid`,
`EnvNameRefused` (the `LD_` rule), `EnvTooMany`, `EnvTooLarge`, `TimeoutOutOfRange`,
`GraceOutOfRange`, `CaptureBoundOutOfRange`, `StdinModeUnsupported`, `TerminationSignalUnsupported`,
`WorkingDirectoryMissing`, `DigestMalformed`.

**2. `AdmissionError`** — a capability is unusable. `NotRegularFile`, `NotDirectory`,
`NotElf`, `Unreadable`, `MetadataUnavailable`, `DescriptorModeUnsuitable`.

**3. `LaunchRefusal`** — authorization refused; **no receipt exists**.
`WorkingDirectoryIdMismatch`, `UnsupportedPlatform`, `MechanismUnavailable`,
`ExecutionKindCapabilityMismatch`.

**4. Execution results.** `LaunchError::NoAttempt { errno_class }` when no child was created and
therefore no receipt exists; otherwise a `LaunchReceipt` carrying one of the outcomes in
[section 30](#30-receipt-vocabulary-and-when-a-receipt-exists). `ExecFailed` keeps its stage and errno class, so
`ENOENT`, `EACCES`, `ENOEXEC`, `ENOMEM`, `E2BIG` and `ETXTBSY` stay distinguishable and none of
them is ever reported as an exit status.

All codes are bounded, machine-stable strings from closed sets, as in the existing crates.

## 35. Dependency direction

> **Recommendation: `helm-launch` 0.1 depends on no HELM crate.**

This follows the strongest precedent in the tree. `helm-observe` states it "depends on neither
HELM crate; artifact identities, not Cargo types, connect the modules". `helm-bind` depended on
two crates because it genuinely needed their *typed models* to compare. `helm-launch` needs no
typed model from anyone — only opaque digests — so the dependency would buy nothing and would
cost the specific hazard [section 12](#12-relation-to-bindingreport) identified: depending on
`helm-bind` transitively pulls in `helm-observe`, placing `RootCapability` — a **read**
authority — inside the launcher's graph, where it must never be mistaken for launch authority.

| Crate | Depend? | Reason |
|---|---|---|
| `helm-evidence` | **No** | Evidence completeness is a different question; the dependency would invert the layering |
| `helm-observe` | **No** | Its capabilities are read authority. A `RootCapability` is permission to observe, never permission to execute |
| `helm-app-spec` | **No** | The desired entry point is a Windows path; nothing is derivable in 0.1 |
| `helm-bind` | **No** | Only two digests are needed, and linking it would pull in `helm-observe` |

External dependencies, each justified: `rustix` (already vetted here; `process`, `pipe`, `event`,
`fs`, `std`) for the safe syscalls; `libc` for `close_range` and the `execveat` syscall, which
rustix does not safely provide; `sha2` for digests; `serde` and `serde_json` for strict JSON
scanning, following the existing crates' pattern of a strict scanner over `serde_json` rather
than derived deserialisation. **No shared utility crate is created**: there is no concrete need,
and the identifier and digest grammars are small enough to restate.

## 36. The `sha2` feature composition

Because the recommended design links **no** HELM crate, `helm-launch` uses plain
`sha2 = "=0.10.9"` and **introduces no new feature unification**. The `force-soft` selection that
`helm-app-spec` requests does not reach it.

If a future orchestration crate links `helm-app-spec` and `helm-launch` together, that combined
graph inherits `force-soft` exactly as the `helm-bind` graph already does. The consequence is
recorded and is unchanged from what ADR-0023 accepted: the feature selects an **implementation,
not a digest**, so outputs are identical either way, and this is a performance and
build-composition effect only. **Hashing is not redesigned as part of launch architecture.**

## 37. Is a system experiment warranted?

**Yes, explicitly, and unlike `helm-bind` this is not a close call.**

ADR-0023 recorded that helm-bind needed no system experiment because the module "performs no
syscall and depends on no kernel or filesystem behaviour". Every load-bearing claim here is the
opposite: it is a claim about kernel behaviour that no pure test can settle.

| Claim | Why pure tests cannot settle it |
|---|---|
| The pinned object runs after its path is replaced | Requires a real `execveat` against a real replaced path |
| Shell metacharacters stay one literal argument | Requires observing the child's real `argv` |
| The environment is exactly what the plan said | Requires observing the child's real `environ` |
| Only descriptors 0, 1, 2 survive | Requires enumerating the child's real `/proc/self/fd` |
| Exec failure is distinguishable from exit 127 | Requires provoking real `ENOENT`, `EACCES`, `ENOEXEC` |
| Both streams drain without deadlock past pipe capacity | Requires real pipe back-pressure |
| Timeout and direct-child termination behave as described | Requires real signals and scheduling |
| A descendant survives direct-child termination | Requires a real fork in a real child |
| `ETXTBSY` and mutation-window behaviour | Requires real concurrent writers |

**LAUNCH-EXEC-01 is required before `crates/helm-launch` may be created.** It is designed and
preregistered in [its definition](../experiments/LAUNCH-EXEC-01-DEFINITION.md) and is **not run**
by this task.

## 38. Falsifiers

Reject the architecture, or any implementation of it, if any of the following is possible.

1. A validated `LaunchPlan` executes anything without a trusted capability.
2. A `BindingReport`, or any state within one, automatically grants execution permission.
3. Executable A is authorized but executable B runs.
4. A `PATH`, name or pathname lookup can substitute the executable.
5. Shell metacharacters in an argument alter argv semantics or reach a shell.
6. Arbitrary ambient environment leaks into the child when the contract says it does not.
7. An unrelated non-`CLOEXEC` descriptor is inherited while exact FD isolation is claimed.
8. Spawn success is reported as exec success.
9. Direct-child termination is described as process-tree containment.
10. A timeout, an empty environment or a closed descriptor set is described as sandboxing.
11. Exit code 0 is described as application success.
12. Any launch result is described as compatibility.
13. A Wine archive identity is described as loader provenance.
14. A receipt omits which exact plan and which exact executable it refers to.
15. An authorization refusal produces a receipt, or any artifact that reads like one.
16. Unbounded stdout or stderr can deadlock the launcher or exhaust memory.
17. Path replacement changes the executed body under a claimed pinned-exec mechanism.
18. `authorize` can be replayed to produce more executions than authorizations.

## 39. Crate and API sketch

Conceptual types only. Nothing here is implemented, and nothing fixes a signature.

```rust
// ---- inert document ------------------------------------------------------
pub fn parse_launch_plan(bytes: &[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors>;
impl ValidatedLaunchPlan {
    pub fn plan_sha256(&self) -> Digest;
    pub fn execution_kind(&self) -> ExecutionKind;
    pub fn subject_spec_sha256(&self) -> Option<Digest>;
    pub fn binding_report_sha256(&self) -> Option<Digest>;
    // argv/env are readable but carry no authority
}

// ---- capabilities, Linux x86_64 only -------------------------------------
pub fn executable_from_fd(fd: OwnedFd) -> Result<ExecutableCapability, AdmissionError>;
pub fn directory_from_fd(id: &str, fd: OwnedFd) -> Result<DirectoryCapability, AdmissionError>;

// ---- authorization: consumes everything ----------------------------------
pub fn authorize(
    plan: ValidatedLaunchPlan,
    executable: ExecutableCapability,
    working_directory: DirectoryCapability,
) -> Result<AuthorizedLaunch, LaunchRefusal>;

// ---- execution: consumes the authorization -------------------------------
pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError>;

pub struct LaunchOutcome {
    receipt: LaunchReceipt,               // the artifact
    stdout_prefix: Option<Vec<u8>>,       // in memory only, never serialised
    stderr_prefix: Option<Vec<u8>>,
    elapsed: Option<Duration>,            // in memory only
}
impl LaunchReceipt {
    pub fn exact_bytes(&self) -> &[u8];
    pub fn sha256(&self) -> Digest;
    pub fn outcome(&self) -> &ExecutionOutcome;
    pub fn inputs(&self) -> &ReceiptInputs;
}
```

`AuthorizedLaunch` and `LaunchReceipt` have private fields, no public constructor, no `Default`,
no `Deserialize` and no `From`, each guarded by a `compile_fail` doctest as in `helm-bind`.

## 40. Receipt identity graph

```text
   LaunchPlan exact bytes                ExecutableCapability
            │ sha256                       │ fstat + pread
            ▼                              ▼
   launch_plan_sha256              size + executable_sha256 + mode_bits
            │                              │
            │   subject_spec_sha256 ┐      │
            │   binding_report_sha256┘     │      (opaque context digests)
            └───────────┬──────────────────┘
                        ▼
                 AuthorizedLaunch
                        │  actual execution (nondeterministic)
                        ▼
        execution outcome + per-stream {count, sha256}
                        │
                        ▼
             LaunchReceipt exact bytes
                        │ sha256
                        ▼
                 receipt_sha256
```

The graph is acyclic: nothing consumes `receipt_sha256`, and no future evidence result is
carried inside the plan. There is no backward edge — in particular the plan cannot reference the
receipt that a run of that plan will produce.

## 41. Test and falsification strategy beyond the experiment

The experiment settles kernel semantics. In-crate tests must independently cover: strict plan
parsing including duplicate keys at every depth and every bound; the `LD_` refusal rule; NUL and
non-UTF-8 argument rejection; refusal atomicity — no receipt on any refusal path; receipt
serializer determinism, injectivity and the size bound; the absence of verdict vocabulary among
the terms the crate itself emits; `compile_fail` doctests for both unforgeable types; and a
source-level check that no verdict-deriving function exists. The Linux backend tests run only
in the cohort, exactly as `helm-observe`'s do, and the portable tests run on all three platforms
in CI.

## 42. Owner decisions required before implementation

| # | Decision | Recommendation |
|---|---|---|
| **D-1** | Permit a narrowly scoped `unsafe` backend in `helm-launch` only, or keep `forbid(unsafe_code)` and publish the weaker FD claim | Permit it, scoped as in [section 8](#8-the-unsafe-and-helper-question) |
| **D-2** | Accept `helm-launch` 0.1 scope B — one generic Linux exact-executable crate, no Wine | Accept |
| **D-3** | Accept zero HELM crate dependencies, with context carried as opaque digests | Accept |
| **D-4** | Accept direct-child lifecycle only, with process-tree containment explicitly not provided | Accept |
| **D-5** | Accept the mandatory `DirectoryCapability` working directory, with no ambient cwd mode | Accept |
| **D-6** | Accept UTF-8-only argv in 0.1, leaving non-UTF-8 arguments unrepresentable | Accept, recorded as a limitation |
| **D-7** | Authorise LAUNCH-EXEC-01 to run, and on which environment | Authorise on a GitHub-hosted `ubuntu-24.04` runner first |
| **D-8** | Accept that the receipt carries no elapsed duration and no timestamp | Accept |

None of these is decided here. ADR-0024 records them as **Proposed**.
