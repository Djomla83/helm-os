# ADR-0024: Execute one explicitly authorized object without granting authority from comparison

**Status:** **Accepted — 2026-09-17**\
**Draft date:** 2026-09-09\
**Revised:** 2026-09-16, in place, after the owner review of the productization plan\
**Approver:** Djomla83 (repository owner), by explicit written instruction of 2026-09-17\
**Acceptance date:** 2026-09-17\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`; revision base
`930ec14b940da9b136c7d2ad024b441d47ceba6c`; acceptance base
`03285d9d13f53c2d97d78bd4f50552c201941c8f`\
**Implementation authority:** **HELM-LAUNCH P1, P2, P3 and P4 accepted; P5 authorised and not yet
accepted.** P1
— the crate skeleton and the portable, pure model — was authorised by the
[owner decision of 2026-09-17](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised) and
[accepted the same day](../DECISIONS.md#helm-launch-p1-accepted). P2 — safe Linux x86_64 capability
admission and the single-use authorisation composition — was
[authorised on 2026-09-18](../DECISIONS.md#helm-launch-p2-authorised) and
[accepted on 2026-09-18](../DECISIONS.md#helm-launch-p2-accepted). P3 — the unsafe Linux x86_64
process-creation backend and the closed post-clone child contract — was
[authorised on 2026-09-18](../DECISIONS.md#helm-launch-p3-authorised) and
[accepted on 2026-09-19](../DECISIONS.md#helm-launch-p3-accepted) after a full independent unsafe
review, four bounded independent correction reviews and hosted Linux x86_64 validation. P4 — the
lifecycle, termination, public `launch` and real receipt slice — was
[authorised on 2026-09-19](../DECISIONS.md#helm-launch-p4-authorised) and
[accepted on 2026-09-20](../DECISIONS.md#helm-launch-p4-accepted) after one bounded independent
lifecycle review, four bounded independent correction re-reviews, three failed publications whose
evidence is preserved unchanged, and a fourth publication whose first natural hosted run passed
every load-bearing Linux gate on attempt 1. P5 — the final regression, evidence-contract, documentation and CI-hardening slice — was
[authorised on 2026-09-20](../DECISIONS.md#helm-launch-p5-authorised) and is **not yet accepted**.
P5 adds **no** architecture: it closes the plan's Level 4 row, publishes the receipt evidence
contract, corrects current crate-facing documentation and hardens CI, under the frozen P1–P4
semantic baseline. The P3
authorisation activates the scoped `unsafe` exception of section E **only under
`crates/helm-launch/src/backend/`**, and authorises process creation and an execution attempt
**only internally**: it adds **no public execution API**, no process-group sweep and no host
privilege acquisition. **P4 keeps that same `unsafe` boundary**: it adds the public `launch`,
`LaunchOutcome` and the guarded process-group cleanup sweep, all in safe code outside
`src/backend/`. **Neither the acceptance of P3 nor the acceptance of P4 changes any of the
architectural contract below**: both are status synchronisation, not new architecture decisions.
Accepting P4 grants **no** containment, sandboxing, exec-success claim or receipt authenticity, and
it does **not** product-accept the complete helm-launch 0.1 module.\
**Design basis:** the [helm-launch 0.1 productization plan](../implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md),
as amended by the [owner review of 2026-09-16](../DECISIONS.md#helm-launch-productization-plan-owner-review).
The [architecture and falsification plan](../research/HELM-LAUNCH-ARCHITECTURE.md) is historical
input wherever that plan's section 4 records it as superseded or refined.\
**Experiment:** [LAUNCH-EXEC-01](../experiments/LAUNCH-EXEC-01-DEFINITION.md). The formal trial
line is **CLOSED**, the Trial #3 frozen result is **`MECHANISM_REJECTED`**, and **no Trial #4 is
authorised**.

> **This revision replaces the Proposed text of 2026-09-09.** That text had been narrowed by the
> [pre-execution review](../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md) and by owner
> decisions D-1 to D-11. It is **stale and must not be accepted as-is**: it named `fork` plus
> `pidfd_open` as the mechanism, concluded exec from clean EOF together with a normal exit, and kept
> a `/proc/self/fd` fallback. The Trial #3 freeze bound its exact bytes as a definition input
> (SHA-256 `c1f3cce88438439aab6632a9c56450ae98d1c64adb31b13c742e2febb1476351`). Those bytes remain
> addressable at freeze commit `bebd8a5f83d4d0daebe9b068050cb5436289c75e` and are unchanged through
> `930ec14`. Every historical record that cites this ADR by line or section refers to them. This
> revision changes no freeze manifest, experiment source, LAUNCH-EXEC-01 definition or evidence
> file.

<a id="acceptance-2026-09-17"></a>

> **Accepted 2026-09-17 by the repository owner, Djomla83.** What is accepted is **this revised
> text**, as carried by the chain `174caad` → `930ec14` → `a810f50` → `03285d9`, read together with
> the owner-reviewed [productization plan](../implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md).
> It is not a retroactive approval of the stale Proposed text of 2026-09-09, and every historical
> section below — the earlier Proposed state, the pre-execution review, the trial history, the
> formal `MECHANISM_REJECTED` result, the X2c postmortem, unvalidated N3 and the absence of a
> Trial #4 — is kept as written.
>
> - **Scope.** The helm-launch 0.1 architecture boundary of sections A to N. Nothing more.
> - **Trial history is unchanged.** The Trial #3 frozen result remains **`MECHANISM_REJECTED`**, its
>   D-7 is **consumed**, its valid trial count is **one**, it **must not be rerun**, and **no Trial #4
>   is authorised**. The engineering disposition remains `PRODUCT_MECHANISM:
>   MECHANISM_NOT_IMPLICATED` by X2c. Acceptance does not rewrite Trial #3 as `MECHANISM_ACCEPTED`.
> - **Evidence classes are kept.** Acceptance does not claim that every post-experiment product
>   refinement was experimentally validated. Each rule keeps the class the plan's
>   [section 3](../implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md#3-experiment-to-product-traceability)
>   gives it: `EXPERIMENTALLY_SUPPORTED` (for the C spike on one kernel), `OWNER_POLICY`,
>   `DOCUMENTATION_DERIVED`, or `UNVALIDATED` — a product obligation to be validated by ordinary
>   product tests (section N). Acceptance promotes no row to a stronger class.
> - **Implementation.** Acceptance by itself authorises no implementation. The owner separately
>   authorised **P1 only**: the crate skeleton and a portable, pure model, with no process execution,
>   no `unsafe`, no host privilege and no experiment execution. P2 and later need a new owner
>   decision.

## Context

Four experimental product modules are owner-merged on main.
[helm-app-spec](../../crates/helm-app-spec/README.md) validates **desired** claims,
[helm-observe](../../crates/helm-observe/README.md) reports **actual** filesystem facts at
explicitly authorised targets under Accepted [ADR-0022](ADR-0022-observation-authority.md),
[helm-bind](../../crates/helm-bind/README.md) **compares** the two under Accepted
[ADR-0023](ADR-0023-binding-authority.md), and
[helm-evidence](../../crates/helm-evidence/README.md) checks evidence-bundle completeness.
None of them executes anything, and the accepted ADRs say so deliberately. ADR-0022 records that
"a future `helm-launch` owns execution". ADR-0023 records that the report "contains no readiness
vocabulary, so a successful comparison confers no launch permission".

`helm-launch` would be the first HELM module whose purpose involves **actual process
execution**, which changes the threat model rather than extending it. Every previous module
could, at worst, produce a wrong document. This one can run a program.

The specific hazard is already visible in the accepted arithmetic. `helm-bind` 0.1 coverage is
*necessarily* incomplete: four mandatory semantic requirements have no comparator, so
`unsupported_binding >= 4` always. `NoClaimContradicted` is therefore reachable with almost
nothing compared. A launcher that read that state as permission would turn an honest "nothing
contradicted among the little we compared" into "safe to execute", reintroducing in one module
every claim the previous four were built to avoid.

Since the first draft, the LAUNCH-EXEC-01 formal trial line ran and closed (section N), and the
productization plan translated its evidence into a product contract. On 2026-09-16 the owner
reviewed that plan, passed it with bounded amendments, and required this ADR to be revised to the
current intended contract before it could be considered for acceptance. The owner accepted the
revised text on 2026-09-17.

## Options

**A. A Wine-specific launcher immediately.** **B. A generic Linux exact-executable launch
substrate first, with a Wine adapter later.** **C. A generic path-based
`std::process::Command` launcher.** **D. Split policy/orchestration and low-level execution
into two crates now.** **E. An execution-plan validator that never executes.**

The [design report](../research/HELM-LAUNCH-ARCHITECTURE.md#4-the-01-product-scope) scores all
five. A fails because Wine runtime provenance and prefix ownership are both unestablished — under
ADR-0023 the prefix association is a caller assertion and never an attestation — and because
Wine is multi-process, so direct-child semantics do not describe a Wine session. C fails because
a pathname is re-resolved at exec, so the object that runs need not be the object inspected. D
fails because there is no accepted policy layer to separate, and a second crate would be a home
for exactly the authority confusion this ADR exists to prevent. E fails because it falsifies
none of the kernel semantics that make the module dangerous. The process-creation alternatives
are compared in section D.

## Accepted decision, 2026-09-17

> Build `helm-launch` 0.1 as a **single-crate, Linux x86_64, capability-driven** launcher that
> **authorises and attempts execution of exactly one** admitted, already-open, regular ELF64
> x86_64 object **through the exact authorised descriptor**, and that **emits no satisfaction,
> compatibility, readiness or success verdict of any kind**. Nothing it returns states that the
> attempt succeeded or that the measured image ran.

Every rule in sections A to N is part of this decision, proposed on 2026-09-09, revised on
2026-09-16 and **accepted on 2026-09-17**. The productization plan carries the algorithms, constants
and product tests it relies on, cited as "plan §". A difference between this ADR and the plan is a
defect, to be corrected by an owner-reviewed amendment and never resolved by an implementation
slice on its own.

### A. Authority

**Data, intent and execution authority are three different things.**

- **Execution authority is an already-open executable descriptor** that a trusted caller moves in
  by value and that admission accepts. It is never a pathname, a name, a verdict or a document
  field. With an admitted working-directory descriptor and a validated plan it is composed into
  one single-use authorisation, which the launch consumes. There is no `PATH` search, executable
  name lookup, discovery, shell, or `wine` string resolved from the ambient environment.
- **Parsing or validating a `LaunchPlan` grants zero execution authority.** The validated plan
  holds no descriptor and cannot reach `execveat` by any path.
- **A `ValidatedAppSpec`, an `ObservationArtifact`, `helm-observe`'s `RootCapability` and a
  `BindingReport` grant zero execution authority.** No function turns one of them, or bytes, a
  string, a path or any `serde` input, into an authority-bearing value. Only an owned descriptor
  that a trusted caller moves in carries authority.
- **`Contradiction::NoClaimContradicted` is never permission.** It does not mean ready, safe,
  permitted, compatible or launchable, and no binding state may authorise execution. The crate
  links no HELM crate (section M), so it has no code path that can read a binding result at all.
- **Plan context digests are caller assertions.** They are recorded, never compared, fetched or
  interpreted, and never evidence that a binding or an observation was consulted.
- **What the type system gives, and what it does not.** The safe API offers no public constructor,
  `Default`, `Clone`, `From`, setter or `Deserialize` for either capability or for the
  authorisation. A caller therefore cannot build an authority-bearing value from data, substitute
  the executable or the plan between authorisation and launch, or launch twice. That is a property
  of values inside one process that uses the safe API. It is **not** an authenticity property of any
  serialised data (section L).
- `helm-launch` trusts the caller who opened the descriptor. Deciding *whether* to launch belongs to
  a future orchestration layer, which does not exist, under its own accepted decision.

### B. 0.1 scope

- **Linux x86_64 only.** The capability, authorisation and launch APIs exist only under
  `cfg(all(target_os = "linux", target_arch = "x86_64"))`. The plan parser, receipt model and
  serializer compile everywhere. Off the cohort a caller can parse a plan and read a receipt, but
  cannot obtain a capability or a launcher. No backend portability is advertised, and Linux on
  another architecture is not claimed.
- **Kernel 5.9 or newer**, the floor set by `close_range`, is documentation-derived. The evidence
  comes from one kernel, `6.17.0-1022-azure`. Cohort membership is a **caller precondition, not an
  attestation this crate makes**, in the shape ADR-0022's clarification established.
- **One admitted, regular ELF64 x86_64 object**, as the admission contract defines it (section C):
  `ELFCLASS64`, `ELFDATA2LSB`, `EM_X86_64`, and `e_type` either `ET_EXEC` or `ET_DYN`, checked in
  the header. A valid header proves nothing about the program. A dynamically linked object still
  resolves its interpreter and libraries by name from host state (E7).
- **Not supported, not even partially:** `#!` scripts, or any other non-ELF object the kernel would
  hand to `binfmt_script` or `binfmt_misc`; `PATH`; any shell or command string; any pathname
  launch; Wine, Proton, PWA, MicroVM, prefix or `WINEPREFIX` orchestration; GUI or portals;
  installation, update, snapshot or rollback; interactive stdin; positive exec evidence.
- **Not a sandbox.** 0.1 provides no filesystem, network, process, registry, device or user-data
  isolation, and manipulates no namespace, seccomp filter, cgroup or MAC policy. The direct child
  runs with the caller's own credentials (section H) and can generally do anything the caller can
  do. Clearing the environment, fixing argv and closing descriptors reduce *accidental* inputs and
  constrain nothing the child does on its own initiative. Proposed
  [ADR-0005](ADR-0005-sandbox-boundary.md) records the related principle that a prefix is not a
  security boundary.
- **No process-tree containment.** Direct-child lifecycle only (section J).

### C. Admission

Executable admission runs on the descriptor the caller moved in, in this order, and resolves no
name at any step (plan §6.2).

1. **Readable, measurable, not writable.** An `O_PATH` descriptor is refused, because nothing can be
   measured through it (X6). An `O_WRONLY` or `O_RDWR` descriptor is refused: a writable capability
   would also be a mutation authority, and on the evidenced kernel a writer held at the exec instant
   made `execveat` fail `ETXTBSY` (E5). Only `O_RDONLY` is admitted, so an execute-only object is
   inadmissible. Measurability, not executability, is the binding constraint.
2. **Regular file.** Anything but `S_IFREG` is refused (X5). This `fstat` is the first metadata
   sample.
3. **Set-ID refusal.** `S_ISUID` or `S_ISGID` is refused (D-9, X7).
4. **Size bound.** A size above **512 MiB** (536 870 912 bytes), the initial 0.1 product bound, is
   refused before any byte is read.
5. **ELF cohort.** The 64-byte header, read positionally through the same descriptor, must carry the
   ELF magic and the cohort of section B. Anything else is refused, and this is where scripts stop
   (X2, E8). An in-cohort header does not promise that the kernel will load the object (X4 was
   admitted and then failed `ENOEXEC`).
6. **Measurement.** Positional reads from offset 0 through the same descriptor, until EOF or one
   byte beyond the step 2 size, with SHA-256 over the bytes read. The capability records the
   pre-execution body size, SHA-256, mode bits (`st_mode & 0o7777`) and ELF type.
7. **Detected instability.** A second metadata sample of the same descriptor is compared with the
   step 2 sample: size, modification time and status-change time, with nanoseconds. Admission is
   **refused when this protocol detects instability**: a sampled field differs, or the byte count
   read differs from the step 2 size. The protocol observes only those fields and that count. A
   change that moves none of them between the two samples is not detected, for example one the
   filesystem's timestamp resolution does not distinguish, or a write through a shared writable
   mapping whose timestamp update is deferred. **Not detecting instability does not prove** that no
   concurrent mutation occurred, that the object is immutable, that a snapshot exists, or that the
   measured bytes are the bytes later executed.

**What the measurement is: a pre-execution measurement of the pinned object, and nothing more.**
`execveat` opens the inode afresh through the descriptor at exec time, and the loader maps the
inode's contents as of that moment. The descriptor pins the inode; it does not freeze the contents.
Where the kernel applies its exec-time write denial, as the evidenced kernel does (E5), `ETXTBSY`
refuses only a writer still holding a writable description at the exec instant, never the ordinary
open-write-close sequence. Nothing here relies on that denial, which upstream Linux once removed and
later restored (commits `2a010c412853` and `3b832035387f`). E6 showed a length-preserving mutation
after admission executing. The fields are therefore named `pre_exec_body_size`,
`pre_exec_body_sha256` and `pre_exec_mode_bits`. They cover the main executable file body only, never the ELF interpreter,
shared libraries or any other part of the loaded-code closure. No attestation of any kind is made.
Sealed-memfd execution, the known route to a measured-equals-executed claim, is not adopted.

**Not checked, deliberately.** Execute permission is recorded, not enforced: the kernel decides at
`execveat`, and a denial is an explicit pre-exec failure record (X1, X3, X8). No filesystem, mount or
`noexec` claim is made. No file-capability attribute is read (section H). No `binfmt_misc`
registration is read; a registration that matches in-cohort x86_64 ELF bytes is a host precondition
(section J).

**Side effects, stated.** Measuring updates atime under `relatime` and fills the page cache.
`O_NOATIME` is not used, because it needs file ownership or `CAP_FOWNER`.

**Working-directory admission** accepts an `O_RDONLY` directory descriptor with a caller identifier
matching `[a-z0-9][a-z0-9._-]{0,79}`. It refuses `O_PATH` and anything that is not a directory, and
checks no search permission: the kernel decides at `fchdir` (S2, S7).

**Composition.** `authorize` consumes the plan and both capabilities and performs no I/O. It refuses,
before any process exists, when the plan's working-directory identifier differs from the
capability's. A plan error, an admission refusal or an authorisation refusal produces **no
receipt**, and the refused values' descriptors close.

### D. Process creation

**Mechanism: `clone3` with `CLONE_PIDFD`, then the closed child sequence of section E, then
`execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`.**

- **Preparation before any child exists.** The parent builds every argv C string, the
  NULL-terminated argv pointer array, `envp = [NULL]`, the exec-status record buffer, the pipes, the
  relocated descriptors (section G) and the `close_range` gaps. A failure here is an error, with no
  child and no receipt.
- **`clone3(CLONE_PIDFD)`** with `exit_signal = SIGCHLD`, and without `CLONE_VM`, `CLONE_VFORK`,
  `CLONE_FILES` or `CLONE_THREAD`. The child gets a copy-on-write address space, its own descriptor
  table and a copy of the signal dispositions. The pidfd comes from **the same system call that
  creates the child** (M3, traced direct form), including from a multi-threaded parent (M2).
- **Why not `fork` plus `pidfd_open`.** `pidfd_open` after `fork` names the right process only if
  the host has not set `SIGCHLD` to `SIG_IGN`, has not set `SA_NOCLDWAIT`, and runs no other reaper.
  A library inside someone else's process can establish none of the three. When one fails, the child
  can be reaped and its pid reused before `pidfd_open` runs, so the pidfd could name an unrelated
  process; that hazard is documentation-derived (`pidfd_open(2)`), and M5 recorded the rejected arm
  under `SIGCHLD = SIG_IGN` as `waitid_echild`. `CLONE_PIDFD` closes the window: the pidfd names the
  created child unconditionally, and only exit-*status* observability stays a caller precondition
  (R4, section J). `clone3` also avoids the `pthread_atfork` surface: glibc's `fork()` runs
  registered `pthread_atfork` child handlers in the child before any launcher code, and a raw
  `clone3` system call runs none. M2 passed with such a handler registered.
- **The pidfd's lifecycle.** An owned, close-on-exec descriptor in the parent. It is polled for exit
  readiness from creation and used for `pidfd_send_signal` and `waitid(P_PIDFD)`; the direct child
  is never individually signalled or waited by numeric pid. The only pid-valued calls are the
  parent's `setpgid` immediately after `clone3` and the guarded group sweep (section J). The pidfd
  closes before `launch` returns, never appears in the receipt, and never reaches the executed image.
- **Exact descriptor execution.** The child's only exec is
  `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)` on the relocated duplicate of the admitted
  descriptor. Path replacement, rename and unlink after admission do not change which inode is
  executed (E2–E4).
- **No fallback.** `ENOSYS` or `EPERM` from `clone3`, for example under a seccomp profile that hides
  it, is an error with no child and no receipt. An environment that blocks `clone3` is outside the
  cohort.

**Rejected alternatives, none of them a fallback:** `fork` plus `pidfd_open` (above);
`execve("/proc/self/fd/N")`, and `fexecve`, which glibc may silently implement through
`/proc/self/fd` — 0.1 has no procfs path; `std::process::Command`, which executes by pathname and
runs its own child code; `posix_spawn`, which takes a pathname; and a helper or trampoline binary,
which reintroduces discovery and a second program.

### E. Unsafe boundary

- **The exception, explicit (D-1 arm (i)).** The workspace sets `unsafe_code = "forbid"`, which a
  local `#[allow]` cannot relax. The exact descriptor-inheritance invariant needs system calls
  between `clone3` and `execveat` that no safe wrapper provides: `rustix` 1.1.4 has no
  `close_range`, and exposes `execveat` only as an `unsafe fn` in a `doc(hidden)` module.
  `helm-launch` therefore does not inherit the workspace lint table. It sets `unsafe_code = "deny"`
  crate-wide and allows `unsafe` in exactly **one small backend module**, `cfg`-gated to Linux
  x86_64, which holds both the post-clone child path and the parent calls around `clone3`. Every
  other crate keeps inheriting the workspace table.
- **All other workspace lint policy is restated, not dropped.** `clippy::unwrap_used`,
  `clippy::expect_used` and `clippy::panic` stay `deny`. The crate adds `unsafe_op_in_unsafe_fn`,
  `clippy::undocumented_unsafe_blocks` and `clippy::multiple_unsafe_ops_per_block`. The backend
  module also denies indexing and slicing, arithmetic side effects, `as` conversions and missing
  safety documentation. A lint-drift test fails if any workspace lint is missing or weaker in the
  crate, and an unsafe-confinement test fails if `unsafe` appears outside the backend module.
- **Closed post-clone system-call contract.** Between the `clone3` return and `execveat`, the child
  walks a fixed, ordered stage list (plan §8.4): `DUP2`, `CLEAR_CLOEXEC`, `CHDIR`, `CLOSE_RANGE`,
  `SETPGID`, `SIGACTION`, `SIGMASK`, `NO_NEW_PRIVS`, `EXEC`. It uses only `dup2`, `fcntl`, `fchdir`,
  `close_range`, `setpgid`, `rt_sigaction`, `rt_sigprocmask`, `prctl` and `execveat`, plus `write`
  and `exit_group` on failure. Every call goes through one raw system-call shim that runs no libc
  code. Any other system call in that window is a test failure.
- **No allocation, locking, formatting, panic or destructor-dependent child state.** Everything the
  child touches is prepared before `clone3` in parent memory. The child receives one plain-old-data
  plan with a compile-time assertion that it needs no drop. The child entry is called directly on the
  `clone3 == 0` branch, never returns and cannot unwind. Its only exits are a successful `execveat` or
  `exit_group(127)`.
- **Evidence and its limit.** M1, M2 and M4 showed a closed child window for a statically linked C
  spike. They do not show that a Rust implementation in a dynamically linked, multi-threaded host
  reproduces it. Product tests must re-establish the Rust child window by tracing it, and no formal
  trial is needed for that. Test-only fault injection compiles only under a non-default feature and
  is proven absent from release builds.

### F. Working directory, argv and environment

- **Working directory (D-5).** A mandatory capability, `fchdir`ed in the child **before** the range
  close, because its descriptor is not one of those the range close preserves. There is no ambient
  working directory and no pathname mode. The receipt records only the caller's identifier.
- **argv (D-6).** A vector, never a string, declared in the plan, with `argv[0]` supplied by the
  caller because the launcher has no honest value to invent. Each element is UTF-8 and NUL-free,
  including after JSON escape decoding. There are 1 to 64 elements of at most 4 096 bytes each and
  131 072 bytes in total; a violation is a plan error. No shell, quoting, joining, splitting or
  expansion exists anywhere (A1–A4, A6). Non-UTF-8 arguments are unrepresentable in 0.1.
- **Environment (D-10): exactly empty.** `envp` is a single NULL in every launch. Nothing is
  inherited or constructed: no `PATH`, `HOME`, `LD_*`, `GCONV_PATH`, locale, display or `WINE*`
  variable (V1). This is **hardening, not provenance**. It removes one way of steering the
  loaded-code closure and pins none of it.
- **stdin.** A pipe whose write end the parent closes, so the child reads immediate EOF.

### G. Descriptor contract

- **Exactly 0, 1 and 2 in the executed image.** stdin reads the launcher's pipe to EOF, and stdout
  and stderr write to the launcher's two pipes (F1–F4, F6, F7).
- **Host descriptors are isolated by `close_range` in the child**, whether or not they are
  close-on-exec and whether or not HELM opened them. The claim rests on that range close, **not** on
  HELM's own close-on-exec hygiene, because a host may hold, or another host thread may create,
  descriptors without close-on-exec (F2).
- **Collision-safe relocation.** Before `clone3`, the parent duplicates every child-side preserved
  descriptor — executable, working directory, stdin read end, stdout and stderr write ends, and the
  exec-status write end — with `F_DUPFD_CLOEXEC` to a number of at least 3, and closes the original,
  unconditionally. No final `dup2` source can then equal its target, even when the host has 0, 1 or 2
  closed, and the child still clears close-on-exec on 0, 1 and 2 explicitly. The range close skips
  inverted gaps, so adjacent preserved numbers are safe (F6, F7).
- **Launcher descriptors do not leak.** The executable and exec-status descriptors survive the range
  close only until a successful exec closes them, because they are close-on-exec (F4). That holds
  even when the caller supplied an executable descriptor without close-on-exec, since only its
  relocated duplicate reaches the child. The working-directory descriptor, every pipe end the child
  does not use and the pidfd never reach the executed image. The parent closes its copies of the
  child-side descriptors right after `clone3`, then the stdin write end, and every remaining
  descriptor before `launch` returns.

### H. Signals and `no_new_privs`

**Signal-mask protocol**, an owner-approved refinement:

1. **Before `clone3`, on the calling thread:** save the current signal mask and block **every
   blockable signal** for the clone window. This uses the raw `rt_sigprocmask` system call with a
   full kernel signal set, because glibc's `sigfillset` and `pthread_sigmask` exclude glibc's two
   internal real-time signals. `SIGKILL` and `SIGSTOP` are not blockable, and nothing here describes
   them otherwise.
2. **The child** starts with that blocked mask. It establishes the child setup of section E; resets
   every changeable signal disposition to `SIG_DFL` **while delivery is still blocked**, so no
   inherited host handler can run in the child; only once the dispositions are ready installs the
   intended final mask for the executed image, which is empty; then sets `no_new_privs` and reaches
   `execveat`, in the approved sequence.
3. **The parent** restores its saved mask after `clone3`, on the success and failure paths alike.

This controls **the calling thread and the child's inheritance boundary only**. Other threads of a
multithreaded caller keep their own masks and can still receive process-directed signals, so no
process-wide signal control is claimed. A glibc `set*id` call in another host thread, or a
cancellation of the calling thread, waits until the mask is restored. The protocol goes beyond the
frozen stage order, which cleared the mask before resetting dispositions (M4), and it is to be
validated by product tests, not by LAUNCH-EXEC-01. The executed image's end state — empty mask,
default dispositions — is the evidenced one (F5, T6).

**`no_new_privs` and set-ID (D-9, D-11).** Set-ID objects are refused at admission, and the child
sets `PR_SET_NO_NEW_PRIVS = 1` before exec, because file capabilities are a separate mechanism that
admission metadata does not carry. N1 observed `NoNewPrivs: 1` in the executed child, and N2
controlled for it. **N3 is BLOCKED**: no real set-ID or file-capability transition has been
exercised. That exec grants no new privilege therefore rests on kernel documentation for those
cases, and is never presented as demonstrated.

**Credentials.** The child runs with the caller's uid, gids, supplementary groups and rlimits. An
LSM policy may still change its security label at `execveat`, within what `no_new_privs` permits;
`helm-launch` neither prevents nor attests that. `helm-launch` makes no privilege change of its own:
no `sudo`, no setuid helper, no capability acquisition, no namespace, seccomp or MAC manipulation.
**None of that is sandboxing.** `no_new_privs` does not drop the caller's privileges, isolate
anything, or make the child's behaviour safe. "No elevation" is not confinement.

### I. Exec confirmation

- **The channel.** A close-on-exec exec-status pipe. On any setup-stage failure or `execveat`
  failure, the child writes one fixed 8-byte record, stage and errno, and exits with status 127.
- **An explicit pre-exec failure record** is the only positive fact this channel yields:
  `pre_exec_failure { stage, errno }`. `ENOENT`, `EACCES`, `ENOEXEC`, `ETXTBSY`, `E2BIG` and `ENOMEM`
  arrive as such records and **never** as an exit status.
- **Clean exec-status EOF alone is not positive exec proof.** A child killed after its last setup
  stage and before `execveat` closes the status write end by dying, and the parent observes exactly
  what a successful exec shows (S5). A later exit status does not change that: S4 recorded
  `ExecStatusIndeterminate` beside an observed exit status 7.
- **No unconditional `ExecSucceeded`.** The durable model has no `ExecSucceeded`, `Launched`,
  `Started` or `Ran` value, and no function derives one.
- **`ExecStatusIndeterminate` is first-class**, not an error. It is the exec status of every launch
  without a complete failure record, with a closed reason: status EOF without a record, a malformed
  record, the pre-exec deadline passing (S6), or a failed status read.
- A caller that needs to know that the program ran must establish it from the program's own
  observable effects, under its own policy, outside `helm-launch`. Any future positive exec-evidence
  mechanism needs its own owner decision and evidence.
- **Exit code 0 means only that the direct child exited with status 0.** It does not mean that a
  workflow passed, the application launched correctly, compatibility succeeded or a UI appeared.

### J. Lifecycle

> `helm-launch` 0.1 claims **direct-child lifecycle only** and provides **no process-tree
> containment.**

- **pidfd poll, signal and wait.** One `poll` loop watches the exec-status channel, stdout, stderr
  and the pidfd. Every deadline is `CLOCK_MONOTONIC`. A pidfd that becomes readable before a
  deadline is never classified as a timeout (T4, T5).
- **Bounded pre-exec phase.** If neither a record nor EOF arrives within `SPAWN_CONFIRM_TIMEOUT_MS`
  (5 000 ms), the launcher sends `SIGKILL` by pidfd at once, records `ExecStatusIndeterminate` with
  the pre-exec-timeout reason, and continues with the post-kill wait (S6). That outcome never means
  "nothing ran": exec may have completed just after the launcher stopped observing.
- **After a status record.** Following a pre-exec failure record or a malformed record, the launcher
  waits for the pidfd to report the child's end until the pre-exec deadline. If no end is observed by
  then, it sends `SIGKILL` by pidfd and continues with the post-kill wait.
- **Run timeout.** `timeout_ms` (1 to 600 000) starts at clean exec-status EOF, which is **not** exec
  confirmation. If it expires while the pidfd is still not readable, the launcher records that the
  run deadline expired, sends `SIGTERM` by pidfd and waits `grace_ms` (0 to 60 000). If the pidfd is
  still not readable after the grace period, it sends `SIGKILL` by pidfd (T1–T3, T6).
- **Bounded post-kill wait.** After every `SIGKILL` it sends to the direct child by pidfd, the
  launcher waits at most `POST_KILL_REAP_MS` (5 000 ms, the initial 0.1 bound) for the pidfd to
  report the child's end. If no end is observed within that bound, the receipt says exactly that —
  the end was not observed — the child is left unreaped to the host, and `launch` still returns. No
  blocking wait exists anywhere in the lifecycle.
- **Timeout and termination are recorded as facts, without causal claims:** whether the run deadline
  expired first, whether `SIGTERM` and `SIGKILL` were sent, and the observed end. `waitid` reports a
  signal number, not a sender, so a child that ended by `SIGKILL` after the launcher sent `SIGKILL`
  is recorded as those two facts, never as "killed by the launcher".
- **Dedicated process group.** The direct child is placed in a dedicated process group. The parent
  calls `setpgid(child, child)` as its first system call after `clone3` returns, and the child calls
  `setpgid(0, 0)` in its closed sequence, so the executed image starts in its own group whichever
  call runs first.
- **Group-sweep authority.** **Successful group establishment is the prerequisite for group-sweep
  authority**, and only the launcher's own successful `setpgid(child, child)` establishes it. The
  launcher never infers or guesses a group id: not from the child's own stage, not from the absence
  of a failure record, not from an `EACCES` meaning that the child had already exec'd, and not from
  the pid value alone. Without established authority **no group sweep is issued**, and the receipt
  records why.
- **Exactly one sweep, before the reap.** Once authority is established, **exactly one `SIGKILL`
  process-group sweep is issued on every completion and return path** — a normal direct-child exit,
  a pre-exec failure, an indeterminate exec status, either timeout, and an end that was not observed —
  **strictly before the direct child is reaped.** The ordering is load-bearing: an unreaped child
  keeps its process-group id reserved, and after the reap the id may be reused by processes
  `helm-launch` never created. If the launcher observes, just before the sweep, that the direct child
  was already reaped elsewhere (a caller-precondition violation, R4), the sweep can no longer precede
  the reap. It is then not issued, and that is recorded.
- **Applied specification, for owner review.** The owner fixed the rule: positive establishment,
  no inferred group id, one sweep before the reap. Two details are this revision's application of it,
  not separate owner decisions. First, the launcher's own successful `setpgid` is the only
  establishing event, so a parent call that loses the race to the child's exec leaves the image in
  its dedicated group without sweep authority. Second, a child observed already reaped elsewhere gets
  no sweep.
- **Issuance is recorded as a fact.** It means the one call was made. No claim follows that any
  descendant was killed, that any process received the signal, or that anything was contained.
- **Same-group descendant side effect, intentional.** A same-group background process may be
  terminated by `helm-launch` **even when the direct child exited normally**. This is the fixed 0.1
  cleanup policy; P1 recorded `descendant_died` under the spike's every-path sweep. A descendant that
  called `setsid` or `setpgid`, daemonised, or handed work to an existing service may survive (P2,
  P4). The sweep is **best-effort cleanup, never process-tree containment**. cgroup v2 delegation is
  the named future path to real containment and is out of scope.
- **Bounded post-child drain.** Once the direct child's end is observed, stdout and stderr drain for
  at most `POST_EXIT_DRAIN_MS` (2 000 ms). A stream still open then is recorded as retained by a
  writer after the child's exit (O6, O7, P4). Closing its read end may deliver `SIGPIPE` or `EPIPE`
  to the retaining process, an effect outside the claimed lifecycle, stated so that it is not
  mistaken for containment. A read failure is recorded as its own completeness value and is never
  treated as EOF.
- **Total bound.** `launch` returns within `SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms +
  POST_KILL_REAP_MS + POST_EXIT_DRAIN_MS`, plus scheduling slack.
- **Caller preconditions, documented, not attested and not enforced.** The host does not set
  `SIGCHLD` to `SIG_IGN` or use `SA_NOCLDWAIT`, and runs no thread that reaps children it did not
  create. A violation can make the exit status unobservable or suppress the sweep, and a foreign
  reaper acting between the launcher's last check and the sweep can defeat the sweep guard. It never
  makes the pidfd name a different process. No seccomp or LSM policy denies the system calls of
  sections D, E and H. No `binfmt_misc` registration matches in-cohort x86_64 ELF. The host is Linux
  x86_64 with kernel 5.9 or newer.
- **Wine remains blocked** on a future multi-process lifecycle model: `wineserver` outlives its
  starter, so one direct child is not an application session.

### K. Output and privacy

- **The durable receipt holds measurements, not output bytes.** Every stdout and stderr byte the
  launcher drains is counted and hashed with SHA-256. The receipt holds, per stream, the byte count,
  a digest over exactly the drained bytes, and completeness. The digest covers drained bytes, not
  "the child's output", and completeness alone says whether those bytes are the whole stream.
- **Raw bounded capture, if offered, is in memory only.** The plan proposes an optional per-stream
  prefix of at most 64 KiB, kept in the in-memory outcome. `helm-launch` never serialises, logs or
  publishes it, and `Debug` prints only its length. Persisting it is solely the caller's decision.
- **Not a confidentiality control.** Hashing output does not stop the child from reading secrets,
  since it runs with the caller's credentials. The boundary is about not republishing output in a
  HELM artifact.
- The receipt carries **zero execution authority and no authenticity claim** (section L).

### L. Receipt

- **Form.** Deterministic JSON with a fixed field order and no map iteration, plus SHA-256 over
  exactly those bytes. The receipt never contains its own digest. No field is withheld, redacted or
  sanitised after serialisation, so the digest can be recomputed from the published bytes (R3-M1
  applied prospectively).
- **Contents** (plan §11.2): the plan digest; the caller-**asserted** context digests; the
  working-directory identifier; the pre-execution measurement and ELF type; the argument count;
  environment mode `empty`; the exec status; the child end (exited with a code, signalled with a
  signal and a core flag, end unobservable, or end not observed); whether the run deadline expired;
  which termination signals were sent; whether the group sweep was issued, or why not; the per-stream
  count, digest and completeness; and one closed backend identifier.
- **No verdict vocabulary.** No emitted token is `pass`, `fail`, `ok`, `success`, `succeeded`,
  `ready`, `compatible`, `verified`, `worked`, `launched`, `sandboxed`, `contained`, `safe`,
  `authentic` or `signed`, and no function maps an outcome to a boolean, a `Result<(), _>` or an
  ordering.
- **No timestamp and no duration** (D-8); elapsed time exists only in the in-memory outcome. **No raw
  pid, pidfd or descriptor number**, and no host path, inode, device, hostname or kernel release.
- **A pre-execution measurement is not executed-body identity** (section C). No HELM layer may read
  `pre_exec_body_sha256` as "the executable body that ran".
- **When a receipt exists.** A plan error, an admission refusal, an authorisation refusal or a
  failure before a child exists returns an error and **no receipt**. Once `clone3` has created a
  child, `launch` returns a receipt on **every** path.
- **Not reproducible behaviour.** Execution is nondeterministic, so the same plan will not produce
  the same receipt. A receipt is reproducible as a document; that is not a claim of reproducible
  behaviour.
- **Authenticity: none claimed.** A durable `LaunchReceipt` is deterministic data. It **carries zero
  execution authority, may be copied, may be fabricated outside the crate, is not cryptographically
  signed, and is not proof of provenance by itself.** The safe API's refusal to construct
  authority-bearing values (section A) does not make serialised receipt bytes authentic, and
  `helm-launch` makes **no receipt-authenticity claim**. Provenance and bundle validation belong
  above `helm-launch`.
- This ADR stabilises no schema and no API.

### M. Relation to existing crates

- **`helm-launch` 0.1 has zero HELM crate dependencies** (D-3), following `helm-observe`'s precedent
  that artifact identities, not Cargo types, connect the modules. No new `sha2` `force-soft`
  unification is introduced; a future orchestrator linking both inherits the effect already recorded
  under ADR-0023, which is build composition only.
- **`helm-app-spec`, no dependency.** Its entry point is an inert, prefix-relative Windows path
  spelling and never Linux execution authority. The launch plan names no executable at all, so there
  is nothing to derive from a specification. Its digest may appear only as a caller assertion.
- **`helm-observe`, independent.** `RootCapability` is **read** authority and must never be mistaken
  for execution authority. An observation exposes no pinned descriptor, so it cannot be the object a
  launch executes. An observed digest equal to a receipt's pre-execution digest means that two reads
  saw the same bytes at two moments, not that the observed file ran.
- **`helm-bind`, no dependency.** Everything a launch could use from a report is one digest,
  recorded as a caller assertion. `NoClaimContradicted` is not permission.
- **`helm-evidence`, no dependency in either direction.** `LaunchReceipt` is not a `helm-evidence`
  type. A bundle can already list a receipt file as an artifact and check its presence and byte
  identity. Byte identity is not authenticity, and **`helm-evidence` does not become launch
  authority.** Semantic receipt checks would need a new evidence-contract version.
- **Integration composes above the modules**, in a future orchestration layer that does not exist,
  through exact-byte identities. Any policy that consults a specification, an observation or a
  binding before a trusted caller opens a descriptor is that layer's accepted decision, never a
  `helm-launch` property. **`helm-app-spec`, `helm-observe`, `helm-bind` and `helm-evidence` grant no
  execution authority.**

### N. Trial history and evidence disposition

- **The experiment.** LAUNCH-EXEC-01 preregistered its cases against a disposable, statically linked
  C spike. Trial #1 aborted after its immutability boundary with no derivable aggregate. Trial #2 was
  `MECHANISM_REJECTED` (59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED). Trial #3 ran 72 cases once on a
  GitHub-hosted `ubuntu-24.04` runner with kernel `6.17.0-1022-azure`, glibc 2.39 and an
  unprivileged user.
- **Trial #3 frozen result: `MECHANISM_REJECTED`, 70 PASS / 1 FAIL / 1 BLOCKED.** The sole FAIL is
  X2c and the sole BLOCKED is N3.
- **The sole FAIL, X2c, was subsequently accepted by owner postmortem as
  `PRODUCT_MECHANISM: MECHANISM_NOT_IMPLICATED`.** Its frozen success rule required a structured
  report that its executed script fixture could never emit, in a counterfactual control arm the
  product forbids. The frozen expectation is `EXPECTATION_EVIDENCE_MISMATCH`, observability is
  `OBSERVABILITY_INCOMPLETE`, and the harness is `STATIC_POSABILITY_DEFECT` with `FIXTURE_DEFECT`.
- **The formal result is not rewritten as accepted.** Trial #3 was not `MECHANISM_ACCEPTED`, X2c stays
  FAIL, the aggregate is not recomputed, and nothing in this ADR says otherwise. The disposition
  establishes only that X2c does not block productization of the supported ELF mechanism.
- **N3 remains unvalidated** for a real privilege transition (section H).
- **R3-M1**, a published aggregate digest that cannot be recomputed from the preserved evidence, **is
  future evidence-contract work**. This ADR applies its rule prospectively to the receipt (section L)
  and repairs no trial history.
- **What the evidence supports.** The Trial #3 PASS cases support the mechanism sequence for the C
  spike on that one kernel: descriptor pinning, cohort admission, literal argv, the empty
  environment, exactly descriptors 0 to 2, signal reset, pre-exec failure records, bounded draining,
  exit and signal classification, the timeout sequence, `no_new_privs`, conservative exec status, a
  group sweep issued before the reap (P3) with a same-group descendant recorded as
  `descendant_died` (P1), and `clone3(CLONE_PIDFD)` from a multi-threaded parent. That support does
  not transfer to a Rust implementation automatically (section E).
- **What the evidence does not support.** The refinements the owner approved on 2026-09-16 are
  writable-descriptor refusal, unconditional relocation, the signal-mask protocol, the
  group-authority guard, the bounded post-kill wait with its unobserved-end record, read failure
  distinct from EOF, causal-neutral termination facts, and the two initial bounds. Detected-instability
  refusal was set by owner amendment. **All of them are product design obligations to be validated
  by ordinary product tests, not by LAUNCH-EXEC-01 and not by another formal D-7 trial.** None is a
  claim that the experiment validated it.
- **The LAUNCH-EXEC-01 formal trial line is CLOSED. Trial #3 must not be rerun. No Trial #4 is
  authorised.**

## Consequences

HELM would gain a reviewable place for execution, with the authority confusion made structurally
unreachable rather than merely tested. The cost is honest and visible. 0.1 attempts to execute one
Linux ELF object and nothing else, and it cannot say that the object ran. It is not a sandbox. It
does not contain a process tree, and its cleanup may terminate same-group background processes even
after a normal exit. Its receipt is data without authenticity. And it cannot run the application the
desired specification actually describes, because that entry point is a Windows path inside a Wine
prefix, and Wine is deliberately out of scope.

This ADR settles the execution-authority boundary, the 0.1 scope and admission
contract, the `clone3(CLONE_PIDFD)` and `execveat` mechanism, the scoped unsafe exception, the
descriptor, signal and `no_new_privs` contract, the conservative exec-confirmation policy, the
direct-child lifecycle with its guarded cleanup sweep, the output, privacy and receipt semantics,
and the relation to the other modules. It stabilises no schema or API. **Accepting it does not by
itself authorise creating `crates/helm-launch` or any implementation.** That needs a separate owner
authorisation of the implementation slices. On 2026-09-17 the owner authorised and then accepted
**P1**; on 2026-09-18 authorised and then accepted **P2**; and on 2026-09-18 authorised **P3**, the
unsafe backend and closed child contract, which is not yet accepted. **P4 and P5 still need their
own owner decision.** The P1 and P2 acceptances authorise no process creation; the P3 authorisation
authorises process creation and an execution attempt **internally only**, with no public execution
API and no process-group sweep. The complete 0.1 module is not yet product-accepted.

## Falsification and approval boundary

Apply the design report's
[falsifiers](../research/HELM-LAUNCH-ARCHITECTURE.md#38-falsifiers). Reject any design in which a
validated plan executes without a trusted capability; a binding report, an observation or a
specification grants permission; executable A is authorised but B runs; a name lookup substitutes
the executable; shell metacharacters alter argv; ambient environment reaches the child; an unrelated
descriptor without close-on-exec is inherited while exact isolation is claimed; spawn success is
confused with exec success; direct-child termination is called containment; a timeout is called
sandboxing; exit 0 is called application success; a launch result is called compatibility; a
refusal produces a receipt; unbounded output can deadlock or exhaust memory; or path replacement
changes the executed inode. The pre-execution review added three: a pre-execution measurement is
presented as the identity of the body that executed; the receipt asserts a temporal or causal fact
the launcher did not observe; and `launch` can fail to return. The owner review of 2026-09-16 adds
five more. Reject any design in which:

- clean exec-status EOF, alone or with any exit status, is presented as exec success;
- a detection protocol that found nothing is presented as proof of stability, immutability or a
  snapshot;
- a receipt is presented as authentic, signed or proof of provenance;
- a group sweep is issued without positively established group authority, from an inferred group
  id, or after the reap;
- blocking the calling thread's signals is described as blocking `SIGKILL` or `SIGSTOP`, or as
  process-wide signal control.

**No new formal trial was required for acceptance, and none was run.** The mechanism's kernel
semantics rest on the Trial #3 evidence as bounded in section N. What acceptance needed was **owner
review of this revised text**, so that no superseded statement is accepted and no unvalidated
refinement is presented as established; that review was completed and the text accepted on
2026-09-17. Before any product merge, the plan's product tests must pass, including the traced Rust
child window, followed by an independent crate review and an owner merge decision.

This ADR accepts no licence, production release, privileges, receipt infrastructure, launcher
implementation, sandbox, recovery or larger HELM subsystem. An execution receipt is **not**
rollback: prefix snapshot and rollback remain a separate lifecycle capability, and 0.1 restores
nothing and claims no user-document protection. A future orchestration may combine snapshot,
launch, observe and rollback, but those are distinct authorities. No GUI is designed or approved.
ADR-0021, ADR-0022 and ADR-0023 are deliberately **not** amended, and no accepted ADR is superseded.
**A0-7ZIP remains experimental FAIL.**

## Owner decisions

**Recorded 2026-09-09**, on the design report's
[owner decisions](../research/HELM-LAUNCH-ARCHITECTURE.md#42-owner-decisions-required-before-implementation).
Deciding them is not accepting this ADR.

| # | Ruling |
|---|---|
| **D-1** | **Accepted, arm (i)** — the scoped unsafe backend; descriptor isolation is not weakened to preserve crate-wide `forbid` |
| **D-2** | Accepted — one exact-executable Linux substrate before Wine |
| **D-3** | Accepted — zero HELM crate dependencies |
| **D-4** | Accepted **as corrected** — what `helm-launch` owns and terminates, not permission to wait forever for descendants |
| **D-5** | Accepted — mandatory working-directory capability |
| **D-6** | Accepted — UTF-8-only argv, a plan-representation limit and not a kernel limit |
| **D-8** | Accepted — no timestamp and no elapsed duration in the receipt |
| **D-9** | **Refuse** set-user-ID and set-group-ID objects at admission |
| **D-10** | **Empty environment only**; `explicit` is removed from the 0.1 contract |
| **D-11** | **Require** `PR_SET_NO_NEW_PRIVS` in the child before exec |

**D-7**, execution authorisation for LAUNCH-EXEC-01, was granted separately for exactly one valid
Trial #1, Trial #2 and Trial #3 execution, and each grant is consumed. No D-7 is live, and none
exists for a Trial #4.

**Recorded 2026-09-16**, the
[productization plan owner review](../DECISIONS.md#helm-launch-productization-plan-owner-review),
`HELM_LAUNCH_PRODUCTIZATION_PLAN_OWNER_REVIEW_PASSED_WITH_BOUNDED_AMENDMENTS`:

| # | Ruling |
|---|---|
| **Q1** | **Approved** — this ADR must be revised before it can be considered for acceptance; this text is that revision |
| **Q2** | **Approved with exact narrowing** — the refinements listed in section N, as 0.1 design obligations to be validated by product tests; `POST_KILL_REAP_MS = 5000` and a 512 MiB admitted-size bound as initial 0.1 bounds |
| **Q3** | **Approved with a group-authority guard** — section J, whose establishing event and already-reaped case are this revision's applied specification, for owner review |
| **Amendments** | detected-instability wording (section C); no receipt-authenticity claim (section L); "authorises and attempts execution of exactly one" (the decision statement) |

**None of that accepted this ADR.** At the review of 2026-09-16 it stayed **Proposed**,
`crates/helm-launch` was not created, no implementation was authorised, and the next gate was
**owner review of this revised ADR**.

**Recorded 2026-09-17**, the
[owner acceptance decision](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised):

| # | Ruling |
|---|---|
| **ADR-0024** | **Accepted 2026-09-17** — this revised text, read with the owner-reviewed productization plan; approver Djomla83 |
| **Product contract** | **Accepted** — the helm-launch 0.1 architecture boundary only; evidence classes unchanged |
| **LAUNCH-EXEC-01** | Formal trial line **closed**; Trial #3 frozen result remains **`MECHANISM_REJECTED`**; D-7 consumed; no rerun; **no Trial #4** |
| **P1** | **Authorised** — crate skeleton and portable, pure model: no `unsafe`, no process creation or execution, no syscall shim, no descriptor admission, no filesystem I/O, no `launch` |
| **P2+** | **Not authorised** — each needs a new owner decision |

**Recorded 2026-09-18**, the P2 [authorisation](../DECISIONS.md#helm-launch-p2-authorised) and
[acceptance](../DECISIONS.md#helm-launch-p2-accepted) decisions. The 2026-09-17 table above is left
as written and records the state at that date; these rows state the current implementation
authority:

| # | Ruling |
|---|---|
| **P1** | **Accepted 2026-09-17** as the first product implementation slice |
| **P2** | **Authorised and accepted 2026-09-18** — safe Linux x86_64 capability admission and single-use authorisation composition: `ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`, `admit_executable`, `admit_working_directory`, `authorize`. Still **no** process creation, process execution, `unsafe`, host privilege or experiment execution, and no `launch` |
| **P3, P4, P5** | **Not authorised** — each needs a new owner decision. P2 acceptance does not activate the section E `unsafe` exception, and authorises no `clone3`, `execveat`, `backend/`, syscall shim, `asm`, pidfd, signal manipulation or child creation |
| **helm-launch 0.1 complete module** | **Not yet product-accepted** |
| **LAUNCH-EXEC-01** | unchanged: formal trial line **closed**; Trial #3 frozen result remains **`MECHANISM_REJECTED`**; D-7 consumed; no rerun; **no Trial #4** |

**Recorded 2026-09-18**, the P3 [authorisation](../DECISIONS.md#helm-launch-p3-authorised)
decision. The two tables above are left as written and record the state at their dates; these rows
state the **current** implementation authority. The accepted architectural contract of sections A to
N is unchanged by this sync:

| # | Ruling |
|---|---|
| **P1** | **Accepted 2026-09-17** |
| **P2** | **Accepted 2026-09-18** |
| **P3** | **Authorised 2026-09-18, not yet accepted** — the unsafe Linux x86_64 process-creation backend and the closed post-clone child contract: `src/backend/{mod,spawn,child}.rs` and one private syscall module, one raw x86_64 `asm!` syscall shim, parent preparation from a consumed `AuthorizedLaunch`, raw `rt_sigprocmask` around `clone3`, `clone3(CLONE_PIDFD)` with `exit_signal = SIGCHLD`, parent-side `setpgid(child, child)`, pidfd ownership, the section 8.4 child sequence, the exec-status pipe, crate-private spawn and result structures, a bounded non-blocking direct-child reap, direct-child pidfd `SIGKILL` only for the fixed pre-exec timeout and bounded test cleanup, an internal non-public `launch_minimal` for tests, and a non-default `test-fault-injection` feature |
| **P3 safety transition** | **Process creation: authorised internally. Process execution attempt: authorised internally. Public process-execution API: none. Unsafe: authorised only under `crates/helm-launch/src/backend/`. Host privilege acquisition: none. Process-group sweep: not authorised in P3** |
| **P3 exclusions** | public `launch()`, `LaunchOutcome`, any public execution entry point or process handle, receipt emission from a real launch, the P4 observation loop, plan-driven run timeouts, `SIGTERM`/grace lifecycle, general stream drain policy, the process-group sweep, process-tree containment, Wine, orchestration, sandboxing |
| **P4, P5** | **Not authorised** — each needs a new owner decision |
| **helm-launch 0.1 complete module** | **Not yet product-accepted** |
| **LAUNCH-EXEC-01** | unchanged: formal trial line **closed**; Trial #3 frozen result remains **`MECHANISM_REJECTED`**; D-7 consumed; no rerun; **no Trial #4** |
