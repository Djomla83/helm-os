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

> **Corrected 2026-09-09 under the
> [three-workstream pre-execution review](../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md).**
> The review's sixteen BLOCKER findings, among 53 in total, are applied across this document and the preregistration. The largest are that the executable digest
> is a **pre-execution measurement** and was described as the identity of the body that ran
> ([section 32](#32-executable-identity-and-mutation)); that the child-setup sequence closed the
> working-directory descriptor before using it, and the parent never closed its own copies of the
> pipe ends, so **no launch could have succeeded as written**
> ([sections 17](#17-file-descriptor-inheritance) and
> [19](#19-standard-output-and-standard-error)); that the drain loop had no termination condition
> surviving a descendant holding a stdio write end (section 19); that inherited signal state was
> an unclosed ambient input (section 17); that the credentials claim was false for a set-user-ID
> object ([section 28](#28-user-credentials-and-privilege)); and that clean EOF does not prove
> exec ([section 21](#21-spawn-versus-exec-confirmation)). The design's original commit `ae4591e`
> is preserved unchanged in history. **ADR-0024 remains Proposed and LAUNCH-EXEC-01 remains
> NOT_RUN.** Two new owner decisions, **D-9** and **D-10**, are recorded in
> [section 42](#42-owner-decisions-required-before-implementation).

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

**Minimum kernel 5.9; architecture x86_64; operating system Linux; and no libc version floor**,
because the backend invokes `execveat` and `close_range` through `libc::syscall(SYS_…)` rather
than through the glibc wrappers, which are gnu-only and `glibc >= 2.34`
([section 35](#35-dependency-direction)).

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

**ELF only in 0.1?** Yes — but **magic alone is not enough**, and the check is therefore on the
header, not on four bytes. Admission reads the first 64 bytes through the same descriptor and
requires `e_ident[EI_MAG]` = `7F 45 4C 46`, `EI_CLASS = ELFCLASS64`, `EI_DATA = ELFDATA2LSB`,
`e_machine = EM_X86_64` (62), and `e_type` in `{ET_EXEC, ET_DYN}`. Anything else is refused as
`AdmissionError::ElfNotInCohort`. This is a deliberate narrowing, not a security check — a
well-formed header proves nothing about the program — and it does two concrete things.

First, it excludes interpreter scripts: **`execveat` on a `#!` script requires the kernel to hand
the interpreter a `/dev/fd/N` pathname, silently making script execution depend on procfs being
mounted and on that descriptor surviving into the interpreter** — and under the mandated
`O_CLOEXEC` capability mode the call does not even get that far, failing `ENOENT`
(see [section 7](#7-executable-toctou-process-creation-mechanisms-compared) and case **X2b**).

Second, and this is why magic alone would not do: `binfmt_misc` recognises a binary by matching
bytes at the start of the file **with a mask**, is documented with `\x7fELF` examples, is
consulted **ahead of** `binfmt_elf`, and locates its interpreter **by full pathname** at exec
time. `qemu-user-static` registers exactly such an entry — masked `e_machine` over ELF magic —
and foreign architectures are HELM's own problem domain. A four-byte check would therefore let a
HELM-authorized capability be routed to an interpreter resolved from a mutable path, with the
pinned object demoted to an argument, which is precisely what
[falsifier 4](#38-falsifiers) forbids. Pinning the cohort in the header closes that by
construction. Cases **X2**, **X2b**, **X2c** and **E8**; the runner's
`/proc/sys/fs/binfmt_misc/status` and every registration are recorded before the first trial.

**Should scripts be rejected?** Yes in 0.1, for both reasons. A future adapter wanting scripts
must first evidence the procfs and descriptor-survival behaviour — and must hand the kernel a
**non-`CLOEXEC`** descriptor to work at all, contradicting
[section 17](#17-file-descriptor-inheritance)'s inheritance invariant, which is a second
independent reason scripts are out of 0.1.

**Should execute permission be checked before launch?** **No — recorded, not enforced.** Mode
bits are stored as a **pre-execution measurement** in the capability and the receipt, and the
kernel evaluates permission again at exec: `fs/exec.c:do_open_execat` opens with
`.acc_mode = MAY_EXEC`, so a `fchmod` between admission and exec changes the answer and the
recorded bits describe the earlier moment (case **E6d**). The launcher does not implement a
second, necessarily divergent permission model on top of the kernel's; `execveat` is the
authority and a permission failure arrives as typed `ExecFailed { errno: EACCES }`.
Pre-checking would add a race and would produce two different answers on the same object under
setuid, ACLs, `noexec` mounts or a MAC policy.

**Should the body SHA-256 be measured?** Yes, through the **same descriptor**, using positional
reads (`pread`) so no shared file offset is disturbed and no seek state is observable.

**How is the measurement bound to the exact object later executed?** By never letting go of the
descriptor. The capability owns the `OwnedFd`; `authorize` moves it into `AuthorizedLaunch`;
`launch` consumes that and hands **the same descriptor** to `execveat`. No pathname exists at
any point after admission. The receipt therefore asserts exactly: *these bytes were read through
this descriptor **before the execution attempt**, and this descriptor was the exec target.*

It does **not** assert that these bytes are the bytes the kernel executed, and the distinction is
load-bearing rather than pedantic. `execveat` does not execute the caller's open file
description: `fs/exec.c:do_open_execat` performs a **fresh** open through the descriptor's own
path with `.acc_mode = MAY_EXEC`, takes the write-deny reference there, and `binfmt_elf` maps the
inode's pages as of that moment. There is no snapshot at admission. The measurement is therefore
a **pre-execution measurement of the pinned inode** and never a measurement of the executed
image — which is why the receipt field is named `pre_exec_body_sha256`
([section 32](#32-executable-identity-and-mutation)).

**Does hashing affect atime or cache?** Yes, and it is stated rather than hidden. Reading
updates atime under default `relatime` and populates the page cache. `O_NOATIME` is not used
because it needs file ownership or `CAP_FOWNER`, and 0.1 must not require privileges. Measuring
is an act with observable side effects, and the README must say so.

**What descriptor mode is required?** `O_RDONLY | O_CLOEXEC`. Not `O_PATH` — **the kernel would
accept it**, since `execveat(2)` states that `AT_EMPTY_PATH` works with descriptors obtained with
`O_PATH`, but the bytes cannot be read through it and an unmeasurable executable defeats the
identity the receipt exists to carry. The refusal is HELM's, not the kernel's, and case **X6**
must not be read as a kernel fact. Emphatically **not** writable either: Linux's exec-time write
deny makes `execveat` fail `ETXTBSY` while a writable descriptor is **held open** on the inode at
the exec instant, so a writable capability would be a launcher that cannot launch.

This narrows what 0.1 can launch, and the narrowing is stated rather than discovered later: the
kernel requires only `MAY_EXEC` to execute an object, so an execute-only object (mode `0111`, or
one readable only by another user) is executable by the kernel and **inadmissible here**.
Measurability, not executability, is the binding constraint, and that is the price of the
receipt.

**What mount or filesystem claims are needed?** **None, and none are made.** Unlike
`helm-observe`, which admits an ext-family superblock magic as a necessary mechanism guard,
`helm-launch` asserts nothing about the filesystem holding the executable. A `noexec` mount
surfaces honestly as `ExecFailed { errno: EACCES }`. A filesystem guard here would invent an
attestation the module cannot back.

## 7. Executable TOCTOU: process-creation mechanisms compared

### A. `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`

| Property | Assessment |
|---|---|
| Same pinned object executed | **Yes, for an object handled by `binfmt_elf`.** The descriptor identifies the inode and no name is resolved for *it*. The kernel still resolves a dynamic ELF's `PT_INTERP` by pathname, and a `binfmt_misc` entry matching the object would resolve its interpreter by pathname — excluded at admission ([section 6](#6-executable-authority)) |
| Pathname replacement after pin | **Irrelevant by construction** for the pinned object — there is no pathname. Not a statement about the interpreter or the libraries |
| Content mutation after measurement | **Not prevented.** The measurement is a pre-execution measurement; the fd pins the inode, not its contents ([section 32](#32-executable-identity-and-mutation)) |
| Descriptor inheritance requirement | The exec descriptor must survive until the syscall; `CLOEXEC` is correct, the kernel resolves it before replacing the image |
| CLOEXEC interaction | Desired: the descriptor does not appear in the new image — and correct **only** because the object is a regular ELF. For anything needing an interpreter, an `O_CLOEXEC` exec descriptor makes `execveat` fail **`ENOENT`**: `execveat(2)` ERRORS and BUGS, and `fs/exec.c` sets `BINPRM_FLAGS_PATH_INACCESSIBLE` when `get_close_on_exec(fd)` |
| ELF vs script | ELF direct. `#!` under an `O_CLOEXEC` descriptor is `ENOENT`; without `CLOEXEC` the kernel synthesises `/dev/fd/N` for the interpreter → procfs dependency → **excluded in 0.1**, cases **X2b** and **X2c** |
| Dynamic loader | Normal `ld.so` behaviour for a dynamic ELF. The kernel resolves `PT_INTERP` **by pathname at exec**, and `ld.so` then resolves `DT_NEEDED` by name, so the absolute reading of row 1 holds for the *pinned object only*. Loader-controlling environment variables are refused as hardening ([section 15](#15-environment-policy)), which reduces one route to that surface and measures none of it |
| procfs dependency | **None** for an ELF handled by `binfmt_elf`. `binfmt_script` and `binfmt_misc` both take the `/dev/fd/N` route and are excluded at admission |
| Mount namespace | None beyond the executable's own mount |
| Rust API safety | **Poor.** rustix exposes it only as `unsafe fn` in a `doc(hidden)` self-declared-unstable module; the realistic call is `libc::syscall(SYS_execveat, …)` |
| fork/multithread safety | The raw `execveat`, `close_range`, `dup3`, `fchdir` and `setpgid` syscalls mutate no userspace state and are safe to issue post-`fork`. Note that `execveat` and `close_range` are Linux-specific and therefore absent from POSIX's `signal-safety(7)` list, so this is an argument from the syscall contract, not a citation. **glibc's `fork()` additionally runs every registered `pthread_atfork` handler in the child before any helm-launch code**, outside the reviewed unsafe region; `clone3` with `CLONE_PIDFD` avoids both that and the `pidfd_open` precondition of [section 22](#22-process-identity), and is measured as arm (b) of case **M3**. The surrounding child setup must be async-signal-safe, and case **M1** is what establishes that rather than assertion |
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
detail. **Declining the workspace lint table drops all four of its lints, not just one**, so the
three clippy denies must be restated verbatim or the policy is weakened by accident — that is
the semantic-preservation requirement, and it is the part most easily lost:

```toml
# crates/helm-launch/Cargo.toml — the workspace table is deliberately NOT
# inherited: it sets unsafe_code = "forbid", which cannot be relaxed by a local
# #[allow], and exactly one module in this crate must allow unsafe (D-1 arm (i)).
# Every workspace lint is restated below; nothing is dropped.
[lints.rust]
unsafe_code            = "deny"   # "forbid" at workspace level
unsafe_op_in_unsafe_fn = "deny"

[lints.clippy]
unwrap_used = "deny"              # restated verbatim from the workspace table
expect_used = "deny"
panic       = "deny"
undocumented_unsafe_blocks    = "deny"   # added: this crate has an unsafe surface
multiple_unsafe_ops_per_block = "deny"
```

- the crate root restates the policy in source — `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`
  — so a reviewer sees it without opening the manifest;
- exactly **one** narrowly named module carries `#[allow(unsafe_code)]`, and it is `#[cfg]`-gated
  to the cohort exactly as `helm-observe` gates its `authority`/`linux`/`observe` modules, so the
  unsafe surface does not exist off Linux x86_64;
- `undocumented_unsafe_blocks` is what turns "every unsafe block carries a documented invariant"
  from a review convention into a compile error;
- the unsafe region is one file, contains no allocation, no `Vec` growth, no locking, no
  formatting and no panic path — every C string, the `argv` pointer array and the `envp` pointer
  array are fully materialised **before** `fork`;
- the child entry point is a single `extern "C" fn(*const ChildArgs) -> !` over one `#[repr(C)]`
  POD, with `const _: () = assert!(!core::mem::needs_drop::<ChildArgs>());` in the same module,
  so no destructor-dependent value can reach the child path by construction;
- the module additionally denies `clippy::indexing_slicing` and
  `clippy::arithmetic_side_effects`, which are panic sources `clippy::panic` does not catch;
- the region's only exits are `execveat` and `_exit`.

**That list is a set of claims, so it needs a falsifier rather than an assurance.** Case **M1**
supplies it: the child's traced syscalls between the `fork`/`clone` return and `execveat` must be
exactly the enumerated set, and any `brk`, `mmap`, `munmap`, `mprotect`, `futex`, `openat`,
`set_robust_list` or `getrandom` in that window is a FAIL. **M2** poses the multithreaded parent
that every real HELM caller is, and **M4** matches the implemented sequence against the frozen
one, so the accepted mechanism is the documented mechanism. The proof obligation is also larger
than "~100 lines of syscalls" suggests: the hard parts are the pointer lifetimes spanning `fork`,
the `-> !` contract on the child path, and the `pthread_atfork` surface glibc's `fork()` opens
before any helm-launch code runs.

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
Refusing them is **hardening, not provenance**, and the distinction is load-bearing. That the
pinned object is the object handed to `execveat` is established by the descriptor and holds under
**every** environment mode; the environment policy neither establishes it nor can weaken it. What
refusing `LD_*` reduces is one well-known surface for injecting *additional* code into the
process after the kernel has mapped the measured body.

That surface is never empty and is never measured. **`pre_exec_body_sha256` covers only the main
executable file body.** It does not cover the ELF interpreter named in `PT_INTERP`, any
`DT_NEEDED` shared object, anything later `dlopen`ed, or any other part of the loaded-code
closure; the kernel resolves `PT_INTERP` **by pathname at exec** and `ld.so` resolves the rest
from `DT_RPATH`, `DT_RUNPATH`, `/etc/ld.so.cache` and the default library directories — host
state this crate neither reads nor attests. **This is equally true in `empty` mode**: an empty
environment does not pin the loaded-code closure, it removes one way of steering it.

The `LD_` rule is also a **prefix denylist, not a closure**. glibc's own secure-execution list
strips `GCONV_PATH`, `GETCONF_DIR`, `HOSTALIASES`, `LOCALDOMAIN`, `LOCPATH`, `MALLOC_TRACE`,
`NIS_PATH`, `NLSPATH`, `RESOLV_HOST_CONF`, `RES_OPTIONS`, `TMPDIR` and `TZDIR` alongside the
`LD_*` names, and 0.1 refuses none of those — `GCONV_PATH` in particular directs glibc to load
gconv objects from a caller-chosen directory. No completeness may be inferred from the rule, and
case **V5** records that such a name is accepted so the incompleteness is evidenced rather than
assumed. `PATH` is *allowed* as an explicit value, and it is recorded that it has no effect on
which executable `helm-launch` runs — the launcher uses a descriptor — and affects only what the
child itself may later exec.

Whether 0.1 should offer `explicit` at all is **owner decision D-10**: `empty` is already the
only mode the first synthetic cohort needs, and dropping `explicit` would remove a
caller-controlled surface and the weakest reasoning in this section. If `explicit` is retained,
it is acceptable only with the narrowings above plus an `environment_mode` field in the receipt,
so a consumer need not re-read the plan to know a caller-supplied environment was in force.

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

The mechanism, in order. **Step 0 is in the parent and steps 1–7 are in the child, and the
ordering is load-bearing rather than stylistic** — an earlier draft of this section closed the
working-directory descriptor before using it, which no launch could have survived:

0. **In the parent, before `fork`:** renumber the exec descriptor, the exec-status pipe write
   end, the working-directory descriptor and the three prepared stdio endpoints above 2 with
   `fcntl(F_DUPFD_CLOEXEC, 3)` if any of them is 0, 1 or 2, so the `dup2` step and the
   `close_range` gap arithmetic have no special cases. The kernel allocates the lowest free
   descriptor, so a host with stdio closed — a daemon, a `systemd` unit, a test harness — makes
   this reachable, not hypothetical;
1. `dup2` the three prepared endpoints onto 0, 1, 2, then unconditionally `fcntl(0, F_SETFD, 0)`,
   `fcntl(1, F_SETFD, 0)`, `fcntl(2, F_SETFD, 0)`. `dup2` clears `FD_CLOEXEC` on *newfd* **only
   when `oldfd != newfd`** — `dup(2)`: "if *newfd* has the same value as *oldfd*, then dup2()
   does nothing" — so the explicit clear is required, not defensive. Step 0 makes the no-op case
   unreachable and the clear makes the invariant hold even if it is not;
2. `fchdir(dir_fd)` — **before** any range close, because `dir_fd` is not one of the descriptors
   the range preserves;
3. `close_range` over everything above 2 **except** the two descriptors still needed — the exec
   descriptor and the exec-status pipe write end — which requires up to three
   `close_range(first, last, 0)` calls over the gaps between them, **skipping any range whose
   first exceeds its last**, since `close_range(2)` returns `EINVAL` in that case and the child's
   only exits are `execveat` and `_exit`;
4. `setpgid(0, 0)`; the parent also calls `setpgid(child, child)` and ignores `EACCES`, so the
   group exists before any sweep can be issued;
5. `rt_sigprocmask(SIG_SETMASK, <empty set>, NULL)` to clear the inherited blocked set;
6. `rt_sigaction(sig, SIG_DFL)` for every signal the host may have set to `SIG_IGN`;
7. `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`.

Both surviving descriptors are `CLOEXEC`, so a successful exec removes them and the invariant
holds in the new image. Step 3 is the load-bearing one, and it is the step
[section 8](#8-the-unsafe-and-helper-question) shows is unreachable from safe Rust. The frozen
sequence is matched syscall-for-syscall against the trace in case **M4**, because the accepted
mechanism must be the documented one.

**Signal state is the fifth ambient input, and 0.1 closes it explicitly** — steps 5 and 6 exist
for that and nothing else. `signal(7)`: a child inherits a copy of the parent's signal mask,
"the signal mask is preserved across `execve(2)`", and "during an `execve(2)`, the dispositions
of handled signals are reset to the default; **the dispositions of ignored signals are left
unchanged**". So without steps 5 and 6 the executed program inherits the host's blocked set and
every disposition the host set to `SIG_IGN`, through both `fork` and `execveat` — exactly the
class of hidden input [section 15](#15-environment-policy) refuses for the environment, and worse
because the plan cannot express it. This is not hypothetical for a Rust host: Rust's startup code
sets `SIGPIPE` to `SIG_IGN` before `main`, so a backend omitting the reset guarantees that every
child runs with `SIGPIPE` ignored, silently changing the behaviour of a child that writes to a
closed pipe — the scenario case O5 tests. A host that blocks `SIGTERM` would likewise guarantee
that [section 23](#23-timeout)'s grace window is burned in full. Both syscalls are on
`signal-safety(7)`'s POSIX list. Cases **F5** and **T6**; the receipt makes no claim about signal
state, the point being that the plan and not the host determines it.

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
`poll()` loop over **the pidfd**, the stdout read end, the stderr read end and the exec-status
pipe, with the poll timeout computed from a `CLOCK_MONOTONIC` deadline. Reading one stream to
completion before the other — the classic bug — is impossible because no read is issued except on
a descriptor `poll` reported ready. A thread pool is deliberately avoided: threads in the parent
complicate the fork-safety argument in [section 8](#8-the-unsafe-and-helper-question) for no
gain.

Three rules make that loop actually terminate, and each closes a way it otherwise would not.

**Parent-side closes, immediately after `fork` returns and before the loop.** The parent closes
its own copies of the exec-status pipe **write** end, the stdout pipe **write** end, the stderr
pipe **write** end and the stdin pipe **read** end, and closes the stdin pipe write end to
deliver the immediate EOF of [section 18](#18-standard-input). The child's `CLOEXEC` closures are
necessary and **not sufficient**: `pipe(7)` reaches end-of-file only when **every** descriptor
referring to the write end is closed, and `fork(2)` gave the parent its own copy of each. Without
this the exec-status pipe never reaches EOF, every launch burns `SPAWN_CONFIRM_TIMEOUT_MS`, and
the drain loop never terminates even with no descendant in the picture.

**A descriptor leaves the poll set on its terminal event, and only after `read()` returns 0.**
`poll(2)` returns POLLHUP in *revents* whether or not it was requested in *events*, so a
descriptor left in the set after end-of-file makes every subsequent `poll()` return immediately
and converts the loop into a busy spin that consumes the timeout without progress — a state a
naive "no deadlock, completed inside the timeout" oracle would score as success. POLLHUP is never
acted on before the descriptor has been read to `read() == 0`.

**A bounded post-exit drain, because direct-child lifecycle does not imply direct-child
liveness.** A descendant forked after exec inherits descriptors 1 and 2 — the launcher's own pipe
write ends — and `pipe(7)` withholds EOF while any of them lives. The direct child can therefore
exit and be reaped while both streams stay open forever, and neither POLLIN nor POLLHUP is ever
raised. On child end the launcher continues the loop over the two read ends for at most
`POST_EXIT_DRAIN_MS`, then stops reading, closes the read end, and records
`WriterRetainedAfterChildExit` for that stream. Closing that read end may deliver `SIGPIPE` or
`EPIPE` to the retaining descendant: that is an effect on a process outside the claimed
lifecycle, it is stated here so it cannot be mistaken for containment, and it is the price of
bounding the launcher's own tail. Total bound:
`SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_EXIT_DRAIN_MS`, and that bound is itself
a tested property. Cases **O6**, **P4** and **T5**.

Per-stream modes, chosen in the plan:

| Mode | Receipt records | Bytes retained |
|---|---|---|
| `discard` | exact drained byte count | none |
| `measure` | exact drained byte count + SHA-256 of drained bytes | none |
| `capture_prefix` | count + SHA-256 + `truncated` flag | first `max_capture_bytes`, **in memory only** |

**Draining continues after the capture bound is reached**, discarding the excess, so a child that
writes more than the bound is never blocked by a full pipe. Capture is binary-safe: counts and
digests are over raw bytes with no encoding assumption. A read failure is a typed
per-stream `completeness` value of [section 30](#30-receipt-vocabulary-and-when-a-receipt-exists)
and is never silently reported as "no output", and never collapses the process disposition.
If the deadline expires mid-stream the stream's `completeness` says so; **the digest is never
presented as a digest of "the child's output"**, only of exactly the bytes this launcher read.

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
| clean EOF, no record, **and the direct child exited normally** | exec succeeded; the executed image is the pinned object |
| clean EOF, no record, **and the direct child was terminated by a signal** | `ExecStatusIndeterminate` — exec cannot be distinguished from pre-exec death |
| EOF plus a short or partial record | `ExecStatusIndeterminate` — recorded as such, never guessed |

**Clean EOF alone does not prove exec, and the earlier form of this table said it did.** A child
killed between `fork` and `execveat` closes its inherited `CLOEXEC` write end by dying; the
parent's copy is already closed; so the parent observes a clean EOF with no record that is
byte-identical to the successful case. The disposition of the direct child is what separates
them, which is why it is part of the rule and why case **S5** poses exactly that state.

**"Clean EOF" means `read()` returned 0 after every available byte was read.** `poll(2)` reports
POLLIN and POLLHUP together when a child writes a record and immediately exits, so hangup is
never interpreted before the descriptor is drained to `read() == 0` — interpreting it first
reports exec success for a child that never execed, which is
[falsifier 8](#38-falsifiers) reached through this table. The record is well below `PIPE_BUF` and
is therefore atomic (`pipe(7)`), so the child's write loops on `EINTR` and a short record
indicates an interrupted transfer, staying `ExecStatusIndeterminate` rather than being
reconstructed. Cases **S5** and **S7**.

`ENOENT`, `EACCES`, `ENOEXEC`, `ENOMEM`, `E2BIG`, `ETXTBSY` and `EPERM` therefore arrive as
distinct `ExecFailed` error classes and can never be presented as an exit status.

## 22. Process identity

**Use a `pidfd`.** A numeric PID is reusable and is therefore unsuitable as long-lived
authority; a `pidfd` refers to one process for its whole lifetime.

`pidfd_open(pid)` immediately after `fork` is sound **only under three conditions the crate
cannot establish**, and `pidfd_open(2)` NOTES states them: the disposition of `SIGCHLD` has not
been set to `SIG_IGN`; `SA_NOCLDWAIT` was not specified; and the zombie was not reaped elsewhere
in the program, by a signal handler or by `wait` in another thread. `helm-launch` is a
**library** running inside a host process it does not control, and the direct child is that
host's child, so these are **caller preconditions, in the shape ADR-0022's cohort clarification
established: the crate does not attest them and cannot enforce them.** The failure mode is not
benign — under `SIGCHLD = SIG_IGN` the child is auto-reaped, its PID becomes immediately
reusable, and `pidfd_open` may return `ESRCH` or open an unrelated process the launcher then
signals at timeout.

Atomic `CLONE_PIDFD` via `clone3` **removes the precondition entirely**, at the cost of a second
unsafe call site, and it also avoids the `pthread_atfork` surface of
[section 8](#8-the-unsafe-and-helper-question). It is therefore measured as arm (b) of case
**M3** and adopted for 0.1 if the `fork` + `pidfd_open` arm shows any loss — not deferred as a
refinement.

**The pidfd is polled, not only signalled.** `pidfd_open(2)`: the descriptor becomes readable
when the task terminates and becomes a zombie, and reports a hangup once it is reaped. It is
therefore in the `poll` set from `fork` onward, which is what lets
[section 23](#23-timeout) classify from an **observed** ordering rather than from what the
launcher did.

If the reap is stolen anyway, `waitid` fails `ECHILD` and the launcher has learned *that* the
child ended and cannot learn *how*. That is `ExitStatusUnobservable`, a distinct outcome from
`ExecStatusIndeterminate` — which is about exec confirmation, not exit disposition — and it is
never guessed and never reported as `Exited { 0 }`. Case **R4**.

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
  unbounded. **On that bound expiring** with no record and no EOF, the launcher terminates the
  direct child by `pidfd` (termination signal, `grace_ms`, `SIGKILL`), **reaps it**, and emits
  `ExecStatusIndeterminate { phase: PreExecTimeout }`. Stating the bound without stating the
  action would leave a live process, a live pidfd and four open pipe ends inside a library
  caller's address space. That outcome **may not be read as "nothing ran"**: exec may have
  completed a moment after the launcher stopped observing. Case **S6**.
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

**The launcher classifies from what it observed, not from what it did.** `Exited` and `Signaled`
are emitted when the pidfd became readable **before** the deadline; `TimedOut` only when the
deadline expired while the pidfd was still not readable. The earlier form of this section said
"the launcher always knows which case applies because it knows whether it issued the signal",
and that is false twice over: with the pidfd absent from the poll set the launcher never observes
an early exit at all and reports `TimedOut { ExitedDuringGrace }` for a child that exited before
the deadline and was never affected by any signal; and `waitid` reports a signal *number*, not a
*sender*. `KilledByLauncher` therefore records only that the launcher issued that signal and that
the child then ended with it — it does not attest that the launcher's signal was the one that
ended it. A `SIGTERM` delivered to an already-dead zombie is simply discarded (`kill(2)`).
Cases **T4** and **T5**.

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
> `pidfd`. A process-group sweep is issued **exactly once, and always strictly before the direct
> child is reaped**, and is recorded as `group_sweep_issued: true` with **no** claim about
> descendants. `helm-launch` 0.1 provides **no process-tree containment**, and the receipt
> vocabulary contains no word that could be read as containment.

**The sweep ordering is load-bearing, not stylistic.** [Section 22](#22-process-identity) refuses
a numeric PID as authority because it is reusable, and a process-group ID is a number from the
same space with no `pgidfd` equivalent. POSIX reserves a process-group ID only while the group is
non-empty, and a zombie is still an existing process (`kill(2)`), so the sweep is safe **only
while the direct child remains unreaped**. Issued after `waitid`, it may signal a process group
the system has already reassigned — an unprivileged `SIGKILL` at an unrelated group owned by the
same user, which is the launcher acting on processes it never created and a stronger defect than
the descendant limitation this section openly accepts. Whether a sweep is issued is fixed by this
decision and is **not** an implementer's option; "may additionally be issued" is not a contract.
Case **P3**.

**Direct-child lifecycle is a claim about what `helm-launch` ends, not about what it waits for.**
A descendant that inherits stdout or stderr holds a write end of the launcher's own pipe, so
those streams may never reach end-of-file even after the direct child is reaped. The bounded
post-exit drain of [section 19](#19-standard-output-and-standard-error) is what makes the
launcher terminate in that state; the sweep is not, and must not be described as if it were.

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
process **unless the authorized object is set-user-ID, set-group-ID, or carries file
capabilities**, in which case the kernel applies them at `execveat` and the child's credentials
differ from the caller's.

That exception is not a wording nit: this section states the upper bound on what a launch can do,
and the unconditional form of it was wrong. `execve(2)` honours those bits and ignores them only
under `no_new_privs`, a `nosuid` mount, or ptrace — and 0.1 sets none of the three, since it
explicitly creates no namespace and no seccomp filter and treats mode bits as a recorded fact
rather than a gate. `execveat` with `AT_EMPTY_PATH` takes the same `do_open_execat` path and
applies the same credential transition; there is no `execveat`-specific suppression. So a trusted
caller handing over a descriptor on a set-user-ID-root object gets a child with **different
credentials from the caller**, and `helm-launch` neither prevents that nor attests it. The bits
are visible in the receipt's `pre_exec_mode_bits`, which is a pre-execution measurement
([section 32](#32-executable-identity-and-mutation)) and not a guarantee about the exec.

Whether 0.1 should instead **refuse `S_ISUID`/`S_ISGID` at admission** — a one-line gate that
makes the paragraph above true by construction, at the cost of a capability 0.1 has no use for —
is **owner decision D-9**. Case **X7**.

`O_NOATIME` is not used for the same reason: it would require ownership or `CAP_FOWNER`.

**"No elevation" is not sandboxing.** It means HELM adds no privilege; it does not mean the
child is confined, because the caller's own privileges may already be broad.

## 29. The `LaunchReceipt`

**It should be an exact identity-bearing artifact**, and the hypothesis holds for the same
reason it held for `BindingReport`: a later layer may need to refer to exactly one execution.

Identity inputs, all recorded inside the receipt bytes:

- `launch_plan_sha256` — determines argv, environment, timeout, capture and termination policy;
- the executable's **pre-execution measurement**: `pre_exec_body_size`, `pre_exec_body_sha256`
  and `pre_exec_mode_bits`. The names carry the two narrowings the fields actually have —
  `pre_exec` because the measurement was taken before the execution attempt and the kernel maps
  the inode again at exec, and `body` because it covers only the main executable file body and
  never the ELF interpreter, the shared libraries or any other part of the loaded-code closure.
  **No HELM layer may read these as "the executable body that ran"**;
- `subject_spec_sha256`, when the plan carried one — opaque context;
- `binding_report_sha256`, when the plan carried one — opaque context;
- the process disposition ([section 30](#30-receipt-vocabulary-and-when-a-receipt-exists));
- per-stream `{ bytes_drained, drained_sha256, completeness }` for stdout and stderr, where
  `drained_sha256` is the SHA-256 of **exactly the `bytes_drained` bytes this launcher read from
  that stream** and is never a digest of "the child's output"; `completeness` is the closed
  vocabulary of section 30 and is the only field that says whether those bytes are the whole
  stream. The `truncated` flag of [section 19](#19-standard-output-and-standard-error) describes
  the in-memory retained prefix, is not a statement about the stream, and does not appear here;
- `group_sweep_issued`, recording only that a sweep was issued to the child's process group and
  nothing about which processes received it or what became of them;
- `environment_mode`, from the closed set `{empty, explicit}`, so a consumer need not re-read the
  plan to know whether a caller-supplied environment was in force;
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

The outcome vocabulary, all process facts and no application semantics. A receipt records **one
process disposition** and, independently, **one completeness disposition per stream**. They are
orthogonal facts and neither may absorb the other — a child that exits 42 while a stream read
fails has two facts, and a sum type could carry only one, so whichever it kept the receipt would
assert a falsehood by omission. Both accepted modules already record partiality on its own axis:
`helm-observe`'s `TargetOutcome::Failed { failure, observed_kind, partial_bytes }` keeps it per
target, and ADR-0023's accepted result is per-claim states plus two independent axes for exactly
this reason.

```text
process_disposition:
  ExecFailed { stage, errno_class }     the child existed; the target never ran
  ExecStatusIndeterminate { phase }     exec status could not be established; not guessed
  Exited { code }                       the direct child exited with this status
  Signaled { signal, launcher_signal_issued: bool }
  TimedOut { disposition }              the deadline expired with the child still running
  TerminationFailed                     the direct child did not stop after SIGKILL and wait
  ExitStatusUnobservable { reason }     a child existed and exec was confirmed, but its exit
                                        disposition was consumed outside the launcher

stream_completeness (stdout, stderr, each):
  CompleteAtEof                         the pipe reached end-of-file and was fully drained
  DeadlineTruncated                     the launcher stopped reading when the deadline expired
  WriterRetainedAfterChildExit          the direct child ended, another process still held a
                                        write end, and the bounded post-exit drain expired
  CaptureFailed { errno_class }         a read failed
```

`TerminationFailed` and every `stream_completeness` value other than `CompleteAtEof` may coexist
with any process disposition. `ExitStatusUnobservable` is deliberately **not** folded into
`ExecStatusIndeterminate`: that one is about *exec confirmation*, and overloading it with an
*exit-status* fact would be the same collapse this structure exists to prevent.

**What the receipt may claim in the descendant state**, verbatim, because it is the easiest place
to over-claim: the direct child exited with the recorded status; for each stream, `bytes_drained`
bytes were read by this launcher before it stopped reading, and `drained_sha256` is over exactly
those bytes; `WriterRetainedAfterChildExit` states one observed fact, that at the moment the
launcher stopped reading at least one process other than the direct child still held a write end
open. The receipt makes **no** claim about the total volume the child and its descendants
produced, **no** claim about whether more was written afterwards, **no** claim about whether any
descendant is still running, and nothing that may be read as containment.

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
over. Three documentation-derived properties bear on it:

- Linux's exec-time write deny (`exe_file_deny_write_access()` in `fs/exec.c:do_open_execat`)
  makes `execveat` fail with `ETXTBSY` if a writable open file description exists on the inode
  **at the instant of the exec syscall**, and blocks new writable opens while the image is
  executing;
- **that reference is taken only at exec time**, so a writer that opens for writing, writes and
  **closes before the exec** leaves `i_writecount` at zero and never triggers `ETXTBSY`;
- neither property freezes the file's contents during the window **between** hashing and exec.

> **`helm-launch` does not promise that the executed bytes equal the hashed bytes, and `ETXTBSY`
> does not close that window.** It promises that the hashed object and the executed object are
> **the same inode, reached through the same descriptor**, and that the measurement was taken
> **before the execution attempt**. `ETXTBSY` refuses the exec only for a writer still *holding*
> a writable descriptor at the exec instant; the ordinary open-write-close sequence — every
> editor, `cp`, `install`, `dd conv=notrunc` and package manager — is never refused, the modified
> body executes, and the receipt carries the pre-execution digest. The field is therefore named
> `pre_exec_body_sha256` and documented as a pre-execution measurement, and **no HELM layer may
> read it as "the executable body that ran"**.

An earlier form of this section said such a writer "usually breaks the exec instead". That was
unfounded and is withdrawn: the write-deny reference is taken at exec time, so the
measure-to-exec window is not covered at all, and the common case is exactly the one that is not
refused.

**The known path to the strong claim, recorded and not adopted.** Copying the measured bytes into
a `memfd_create(MFD_ALLOW_SEALING)` object, applying
`F_SEAL_WRITE|F_SEAL_SHRINK|F_SEAL_GROW|F_SEAL_SEAL`, and executing the sealed memfd would make
measured bytes and executed bytes provably identical. The cost is a full copy, the loss of
set-user-ID and on-disk identity semantics, a changed `/proc/self/exe`, and an interaction with
`vm.memfd_noexec`/`MFD_NOEXEC_SEAL`. It is recorded here as the mechanism that would earn the
stronger claim, **not** as a claim 0.1 makes.

This must be verified, not asserted: cases **E5**, **E5b**, **E6**, **E6b**, **E6c** and **E6d**
are the E-series mutation cases, and **E6, E6b and E6d are documentation gates as well as
mechanism gates** — none may be scored PASS while any document still presents the pre-execution
measurement as the identity of the executed body.

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
| `POST_EXIT_DRAIN_MS` | 2 000 | Bounds draining after the direct child ends, for the case where a descendant still holds a stdio write end. **Fixed by the architecture, never a plan field**, so a plan cannot lengthen the launcher's own tail |
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
`fs`, `std`) for the safe syscalls; `libc` for the `execveat` and `close_range` **syscall numbers
and the `syscall` entry point**, which rustix does not safely provide — deliberately **not** the
crate's glibc wrappers for either, because both are declared for `linux-gnu` only and
`close_range`'s is marked "Added in glibc 2.34", so using them would add an invisible
**glibc >= 2.34** runtime floor and fail to link on any musl target; `sha2` for digests; `serde` and `serde_json` for strict JSON
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
19. A receipt field, its name, or its prose presents a **pre-execution measurement** as the
    identity of the body that executed; or the executable digest is described as covering the
    ELF interpreter, the shared libraries or the loaded-code closure; or any environment mode,
    including `empty`, is described as establishing which code the process loaded; or the `LD_`
    refusal is described as complete.
20. The receipt asserts a temporal or causal fact the launcher did not observe — in particular a
    `TimedOut` disposition for a child observed to have exited before the deadline, or an exit
    status the launcher did not obtain.
21. `launch()` can fail to return while the direct child's lifecycle has ended.

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
            │ sha256                       │ fstat + pread, BEFORE the exec attempt
            ▼                              ▼
   launch_plan_sha256       pre_exec_body_size + pre_exec_body_sha256
            │                            + pre_exec_mode_bits
            │                              │
            │   subject_spec_sha256 ┐      │  (the main file body only: never the
            │   binding_report_sha256┘     │   interpreter, the libraries or the
            │        (opaque context)      │   loaded-code closure)
            └───────────┬──────────────────┘
                        ▼
                 AuthorizedLaunch
                        │  actual execution (nondeterministic; the kernel re-opens
                        │  and re-maps the inode at exec, so these bytes are not
                        ▼  attested to be the executed bytes)
   process_disposition + per-stream {bytes_drained, drained_sha256, completeness}
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

The experiment settles kernel semantics; **plan-parse properties are not settled by it and are
not preregistered in it.** LAUNCH-EXEC-01 originally carried a NUL-argument case (A5) and an
`LD_PRELOAD` case (V3); both were deleted, because both are properties of a crate that does not
exist and that must not be created before the experiment runs, and a NUL byte cannot be delivered
through a C spike's `argv` at all. They are obligations here instead, and **no experiment result
supports or refutes them**.

In-crate tests must independently cover: strict plan
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
| **D-8** | Accept that the receipt carries no elapsed duration and no timestamp | Accept, **conditional on the pidfd being polled** — excluding the duration is only safe once the exit-versus-deadline ordering is an *observed* fact ([section 23](#23-timeout)) |
| **D-9** | Refuse `S_ISUID`/`S_ISGID` objects at admission, or record and permit them | **Refuse in 0.1**, with `AdmissionError::SetIdBitsPresent`. The non-elevation claim of [section 28](#28-user-credentials-and-privilege) is load-bearing for the whole non-sandbox boundary, and 0.1 has no setuid use case |
| **D-10** | Environment `empty`-only in 0.1, or retain `explicit` with the narrowings of [section 15](#15-environment-policy) | **`empty`-only.** It is already the only mode the first synthetic cohort needs, and it removes a caller-controlled surface rather than adding one |

None of these is decided here. ADR-0024 records them as **Proposed**. D-9 and D-10 were added by
the [pre-execution review](../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md), and both
change LAUNCH-EXEC-01's case membership, so the definition cannot be frozen until they are ruled
on.
