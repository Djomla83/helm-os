# ADR-0024: Execute one explicitly authorized object without granting authority from comparison

**Status:** **Proposed — not Accepted, and authorising no implementation**\
**Draft date:** 2026-09-09\
**Approver:** not entered\
**Acceptance date:** not entered\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`\
**Design basis:** [helm-launch architecture and falsification plan](../research/HELM-LAUNCH-ARCHITECTURE.md)\
**Experiment dependency:** [LAUNCH-EXEC-01](../experiments/LAUNCH-EXEC-01-DEFINITION.md), **NOT_RUN**

## Context

Four experimental product modules are owner-merged on main.
[helm-app-spec](../../crates/helm-app-spec/README.md) validates **desired** claims,
[helm-observe](../../crates/helm-observe/README.md) reports **actual** filesystem facts at
explicitly authorised targets under Accepted [ADR-0022](ADR-0022-observation-authority.md),
[helm-bind](../../crates/helm-bind/README.md) **compares** the two under Accepted
[ADR-0023](ADR-0023-binding-authority.md), and
[helm-evidence](../../crates/helm-evidence/README.md) checks evidence-bundle completeness.
None of them executes anything, and all three accepted ADRs say so deliberately: ADR-0022
records that "execution belongs to a future `helm-launch`", and ADR-0023 records that the
report "contains no readiness vocabulary, so a successful comparison confers no launch
permission".

`helm-launch` would be the first HELM module whose purpose involves **actual process
execution**, which changes the threat model rather than extending it. Every previous module
could, at worst, produce a wrong document. This one can run a program.

The specific hazard is already visible in the accepted arithmetic. `helm-bind` 0.1 coverage is
*necessarily* incomplete — four mandatory semantic requirements have no comparator, so
`unsupported_binding >= 4` always — which means `NoClaimContradicted` is reachable with almost
nothing compared. A launcher that read that state as permission would convert an honest
"nothing contradicted among the little we compared" into "safe to execute", reintroducing in
one module every claim the previous four were built to avoid.

## Options

**A. A Wine-specific launcher immediately.** **B. A generic Linux exact-executable launch
substrate first, with a Wine adapter later.** **C. A generic path-based
`std::process::Command` launcher.** **D. Split policy/orchestration and low-level execution
into two crates now.** **E. An execution-plan validator that never executes.**

The [design report](../research/HELM-LAUNCH-ARCHITECTURE.md#4-the-01-product-scope) scores all
five. A fails because Wine runtime provenance and prefix ownership are both unestablished — the
prefix association is, by ADR-0023, "a caller assertion, never an attestation" — and because
Wine is multi-process, so direct-child semantics do not describe a Wine session. C fails because
a pathname is re-resolved at exec, so the object that runs need not be the object inspected. D
fails because there is no accepted policy layer to separate, and a second crate would be a home
for exactly the authority confusion this ADR exists to prevent. E fails because it falsifies
none of the kernel semantics that make the module dangerous.

## Proposed decision

> Build `helm-launch` 0.1 as a **single-crate, Linux x86_64, capability-driven** launcher that
> executes **exactly one** explicitly authorized, already-open, regular ELF executable object,
> and that **emits no satisfaction, compatibility, readiness or success verdict of any kind**.

### Proposed authority boundary

**Data, intent and execution authority are three different things.**

- Parsing or validating a `LaunchPlan` grants **zero** execution authority. The validated plan
  holds no descriptor and cannot reach `execveat` by any path.
- A `BindingReport` grants **zero** execution authority.
  `Contradiction::NoClaimContradicted` **does not mean** ready, safe, permitted, compatible or
  launchable, and **no binding state may automatically authorize execution.**
- Execution authority is **an already-open executable descriptor a trusted caller passes by
  value**. Not a pathname, not a name, not a verdict, not a document field.

No `PATH` search, no executable-name lookup, no discovery, no shell, and no `wine` string
resolved from the ambient environment.

### Proposed 0.1 scope and cohort

Supported cohort: **Linux, x86_64, kernel 5.9 or newer**, that floor being set by
`close_range`. Cohort membership is a **caller precondition, not an attestation this crate
makes**, in the shape ADR-0022's clarification established. API compile portability is separated
from validated execution backend support: the plan parser, model and receipt serializer compile
everywhere; the execution backend exists only inside the cohort.

### Proposed mechanism

`fork()`, async-signal-safe child setup, then
`execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)` — chosen because the descriptor identifies
the inode and no name is resolved at exec time. `fexecve` is rejected because glibc silently
falls back to `/proc/self/fd`; ordinary path-based `Command` is rejected outright.
`execve("/proc/self/fd/N", …)` is retained only as a documented fallback with its procfs
dependency stated, and only if LAUNCH-EXEC-01 falsifies the primary mechanism.

### Proposed process-boundary contract

- **argv:** a vector, never a string. `argv[0]` is declared explicitly by the plan, because the
  launcher has no honest value to invent. NUL bytes rejected; UTF-8 only in 0.1, with non-UTF-8
  arguments a recorded limitation. **No shell, no quoting logic, no command-string parser.**
- **environment:** never inherited. Modes are `empty` (default) and `explicit`. Any name
  beginning `LD_` is refused at parse time, because such a variable changes which code the
  measured image actually loads and would make the receipt's executable identity misleading.
- **working directory:** a **mandatory caller-supplied directory capability**, `fchdir`ed in the
  child. There is no ambient-cwd mode and no pathname mode.
- **descriptors:** exactly 0, 1 and 2 survive into the executed image, achieved with `dup2` plus
  `close_range` over everything else. This claim rests on `close_range`, **not** on HELM's own
  `CLOEXEC` hygiene, because a host process may already hold non-`CLOEXEC` descriptors.
- **stdin:** a pipe whose write end the parent closes, giving immediate EOF.
- **stdout/stderr:** separate pipes, drained concurrently by a single-threaded `poll` loop so
  neither stream can deadlock; bounded capture with draining continuing past the bound.
- **exec confirmation:** a `CLOEXEC` exec-status pipe. Clean EOF proves exec; a record proves
  failure with its stage and errno. `ENOENT`, `EACCES`, `ENOEXEC` and `ETXTBSY` can **never** be
  reported as an exit status.
- **process identity:** a `pidfd`, not a reusable numeric PID; the receipt exposes no PID.
- **timeout:** `CLOCK_MONOTONIC`, started **after confirmed exec**, with a separately bounded
  pre-exec phase, a termination signal, a grace window, then `SIGKILL`.

### Proposed lifecycle limit, stated as a non-claim

> `helm-launch` 0.1 claims **direct-child lifecycle only** and provides **no process-tree
> containment.**

The child gets its own process group, which gives a best-effort sweep target. A process may
still fork, `setsid`, daemonise or hand work to an existing service, and nothing available to an
unprivileged 0.1 prevents that. A process-group sweep is recorded as issued, with no claim about
descendants. cgroup v2 delegation is named as the future path to real containment and is out of
scope. LAUNCH-EXEC-01 case **P1** exists to demonstrate this limitation rather than to pass.

### Proposed non-sandbox boundary

**A launch mechanism is not a sandbox.** 0.1 provides no filesystem, network, process, registry,
device or user-data isolation. The direct child runs with the **caller's own OS credentials** and
can generally do anything the caller can do. Clearing the environment, controlling argv and
closing descriptors reduces *accidental* inputs and constrains nothing the child does on its own
initiative. Proposed [ADR-0005](ADR-0005-sandbox-boundary.md) already records the related
principle that a prefix is not a security boundary. 0.1 also performs **no privilege change**:
no `sudo`, no setuid helper, no capability gain, no namespace, seccomp or MAC manipulation.
**"No elevation" is not sandboxing.**

### Proposed receipt semantics

A `LaunchReceipt` is an exact identity-bearing artifact: deterministic bytes plus SHA-256 over
exactly those bytes, binding the launch-plan digest, the executable's measured size and SHA-256,
optional opaque spec and binding-report context digests, the outcome, and per-stream byte counts
and digests. No timestamp, hostname, PID, host path or elapsed duration. The graph is acyclic.

**Execution is nondeterministic, so the same plan will not produce the same receipt.** That is
normal, and receipt reproducibility is **not** a claim of reproducible behaviour.

**Authorization refusal produces no receipt** — no bytes, no digest, no execution facts —
exactly as a `helm-bind` refusal produces no report. Once a direct child exists a receipt always
exists, including on exec failure. If `launch` was called but no child was ever created, the
result is an error with no receipt.

Outcome vocabulary is process facts only: `ExecFailed`, `ExecStatusIndeterminate`, `Exited`,
`Signaled`, `TimedOut`, `TerminationFailed`, `OutputCaptureFailed`. There is no `PASS`, `FAIL`,
`OK`, `SUCCESS`, `COMPATIBLE` or `READY`, and no function that maps an outcome onto a boolean.
**Exit code 0 means only that the direct process exited with status 0** — not that the workflow
passed, the app launched correctly, compatibility succeeded or a UI appeared.

### Proposed relation to the other modules

`helm-launch` 0.1 **depends on no HELM crate**, following `helm-observe`'s precedent that
"artifact identities, not Cargo types, connect the modules". Subject-specification and
binding-report context are carried as **opaque digests** in the plan and recorded in the receipt.
Consequently the crate contains **no code path that can read `Contradiction` or `Coverage`** —
reading a verdict is unavailable, not merely discouraged. It does not depend on `helm-observe`,
whose `RootCapability` is **read** authority and must never be mistaken for launch authority.
Because no HELM crate is linked, no new `sha2` `force-soft` unification is introduced; a future
orchestrator linking both inherits the effect already recorded under ADR-0023, which is build
composition only.

### Proposed unsafe-code consequence

The workspace declares `unsafe_code = "forbid"`, which **cannot** be relaxed locally by
`#[allow]`. The exact descriptor-inheritance invariant requires `close_range` in the child, and
no safe Rust wrapper can run code between `fork` and `exec`: `pre_exec` is itself `unsafe`, and
`rustix` 1.1.4 has no `close_range` and exposes `execveat` only as an `unsafe fn` in a
`doc(hidden)` module whose own documentation states its API is unstable and its safety
requirements are not fully documented. `helm-launch` would therefore decline the workspace lint
table and define its own, with a single narrowly scoped unsafe module. **This is an owner
decision (D-1)**, and its alternative is to keep `forbid` and publish the weaker claim that
descriptors the host already holds without `CLOEXEC` may be inherited.

## Consequences

HELM would gain a reviewable place for execution, with the authority confusion made structurally
unreachable rather than merely tested. The cost is honest and visible: 0.1 executes one Linux
ELF object and nothing else, it is not a sandbox, it does not contain a process tree, and it
cannot run the application the desired specification actually describes — because that entry
point is a Windows path inside a Wine prefix, and Wine is deliberately out of scope.

This ADR, if accepted, would settle the execution-authority boundary, the minimum 0.1 scope, the
mechanism candidate, the non-sandbox boundary, the receipt semantics, the relation to
`helm-bind`, the process-tree limitation and the experiment dependency. It stabilises no schema
or API.

## Falsification and approval boundary

Apply the design report's
[falsifiers](../research/HELM-LAUNCH-ARCHITECTURE.md#38-falsifiers). Reject any design in which
a validated plan executes without a trusted capability, a binding report grants permission,
executable A is authorized but B runs, a name lookup substitutes the executable, shell
metacharacters alter argv, ambient environment leaks against the contract, an unrelated
non-`CLOEXEC` descriptor is inherited while exact isolation is claimed, spawn success is
confused with exec success, direct-child termination is called containment, a timeout is called
sandboxing, exit 0 is called application success, a launch result is called compatibility, a
Wine archive identity is called loader provenance, a refusal produces a receipt, unbounded output
can deadlock or exhaust memory, or path replacement changes the executed body.

**A system experiment IS required**, unlike ADR-0023. Every load-bearing claim here is a claim
about kernel behaviour that pure tests cannot settle.
[LAUNCH-EXEC-01](../experiments/LAUNCH-EXEC-01-DEFINITION.md) is designed and preregistered, is
**NOT_RUN**, and uses only synthetic helpers — no Wine, no 7-Zip, no proprietary software and no
A0 access. **`crates/helm-launch` must not be created before it has run and been reviewed.**

This ADR accepts no licence, production release, privileges, receipt infrastructure, launcher
implementation, sandbox, recovery or larger HELM subsystem. Execution receipt is **not**
rollback: prefix snapshot and rollback remain a separate lifecycle capability, and 0.1 restores
nothing and claims no user-document protection. A future orchestration may combine snapshot,
launch, observe and rollback, but those are distinct authorities. No GUI is designed or
approved. ADR-0021, ADR-0022 and ADR-0023 are deliberately **not** amended, and no accepted ADR
is superseded. **A0-7ZIP remains experimental FAIL.**

## Owner decisions required

Eight decisions are tabulated in the design report's
[owner decisions](../research/HELM-LAUNCH-ARCHITECTURE.md#42-owner-decisions-required-before-implementation):
the scoped unsafe backend (D-1), the 0.1 scope (D-2), zero HELM dependencies (D-3),
direct-child-only lifecycle (D-4), the mandatory working-directory capability (D-5), UTF-8-only
argv (D-6), authorisation and environment for LAUNCH-EXEC-01 (D-7), and the receipt carrying no
duration or timestamp (D-8). **None is decided here.**
