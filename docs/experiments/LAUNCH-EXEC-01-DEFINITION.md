# LAUNCH-EXEC-01 — preregistered execution definition

**Status: FROZEN FOR PRE-TRIAL REVIEW — NOT_RUN. No trial has been executed and no result
exists.**\
**Execution authorisation (D-7): NOT GRANTED.** Freezing the definition is not permission to run
it. See [section 8](#8-what-this-definition-does-not-authorise).\
**Authoritative base:** `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`.\
**Authority:** Proposed [ADR-0024](../adr/ADR-0024-launch-authority.md) and the
[helm-launch architecture](../research/HELM-LAUNCH-ARCHITECTURE.md). Neither is Accepted, and
this definition authorises nothing by itself.

This document **freezes the case set** for the first `helm-launch` mechanism experiment, before
any trial, following the discipline that worked for
[OBS-FS-01](obs-fs-01/README.md): the definition is committed first, and the mechanism is not
edited after the first valid trial.

> **Machine source of truth.** Membership, classes, schedules and expectations live in
> [`launch-exec-01/frozen_cases.py`](launch-exec-01/frozen_cases.py). **The tables in section 3
> are generated from it**, and the counts in section 6 are derived from it rather than typed.
> Where the prose and the manifest disagree, the manifest is authoritative and the prose is the
> defect. `tools/tests/test_launch_exec_01.py` enforces the manifest's own invariants —
> no duplicate ids, an exact three-way partition, a single expectation per case, and a total
> aggregate precedence — without invoking the spike or posing any case.

> **History.** The original preregistration `be0251b` froze 43 cases. The
> [three-workstream pre-execution review](../implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md)
> found sixteen BLOCKERs among 53 findings and re-froze it at 71. Owner decisions **D-9**
> (refuse set-id objects), **D-10** (empty environment only) and **D-11** (`no_new_privs`) then
> changed membership to **72**, frozen at `66bf4b6`. The
> [independent pre-trial review](../implementation/HELM-LAUNCH-INDEPENDENT-PRETRIAL-REVIEW.md)
> then found six further BLOCKERs in that freeze — three mandatory cases were **unposable**
> against the frozen spike, the N2 control and `clone3` availability were both host properties
> the freeze would have scored as mechanism defects, and the checker let any class absorb a
> BLOCKED status. Membership is still **72**; the class partition changed to **54 / 11 / 7** and
> the spike gained the two modes and the capture retention its own cases required. Every earlier
> commit is preserved unchanged, `66bf4b6` included.

## 0. What this experiment answers, and what it does not

> **It answers:** can HELM execute **one explicitly authorized synthetic Linux executable** with
> the process-boundary semantics the architecture claims?

> **It does not answer:** whether HELM can safely manage a Wine application lifecycle. That
> needs a separate later experiment and is out of scope here.

**No Wine. No 7-Zip. No proprietary software. No A0 lab. No privileged operation.** Only
synthetic helper executables built from source committed alongside this definition.

**Plan-parse properties are not answered here.** The original A5 (NUL argument) and V3
(`LD_PRELOAD`) are deleted, and under **D-10** the explicit-environment cases V2, V4 and V5 go
with them: all five are properties of `crates/helm-launch`, which does not exist and which
ADR-0024 forbids creating before this experiment runs, and a NUL byte cannot be delivered through
a C spike's `argv` at all. They are recorded in `frozen_cases.OUT_OF_SCOPE` and as in-crate
obligations in
[architecture section 41](../research/HELM-LAUNCH-ARCHITECTURE.md#41-test-and-falsification-strategy-beyond-the-experiment),
so the deletion cannot later read as coverage. **No result here supports or refutes them.**

**One further thing this experiment cannot show.** D-11 guarantees that exec may not grant new
privilege through set-user-ID, set-group-ID or file capabilities. The runner is unprivileged and
**no privileged fixture is created**, so the report must keep three things apart and never
collapse them: the **kernel semantics** this rests on, taken from primary sources; the **directly
observed** `NoNewPrivs: 1` state, which N1 establishes and N2 controls for; and the **untested**
privileged transition, which is case N3, is BLOCKED by construction, and is never presented as
demonstrated.

## 1. Scope and disposability

All experiment code is **disposable pre-implementation spike code**. It is not
`crates/helm-launch`, is not a Cargo workspace member, defines no product API, and must not be
promoted into one. Its only purpose is to falsify the mechanism described in the architecture.

The experiment is written in C and Python. A C spike makes `fork`/`exec` semantics and the
syscall trace clearer; product language preference must not distort syscall evidence.

**A C spike always has `close_range` available, so under D-1 arm (ii) an F-series PASS would not
transfer to a `forbid(unsafe_code)` Rust implementation.** The owner has taken **arm (i)**, so
that caveat does not apply to this run; it is recorded because the transfer question is a
property of the evidence, not of the decision.

## 2. Artefacts, committed and hashed before the first trial

Committed **before** any trial and hashed in
[`launch-exec-01/SOURCE-HASHES.json`](launch-exec-01/SOURCE-HASHES.json), as OBS-FS-01 did. The
manifest does not hash itself, following the existing convention.

| File | Role |
|---|---|
| `frozen_cases.py` | **Machine source of truth**: membership, class partition, frozen D-1 arm, schedules, per-case `traced` flag, expectations |
| `oracles.py` | Independent expected values computed with `hashlib`, never reading spike output |
| `checker.py` | Verdict evaluator; links nothing from the spike and never repairs a record |
| `harness.py` | Preflight inventory, static-link gate, fixture builder, descriptor pre-loader |
| `run_launch_exec_01.py` | Runner; refuses to pose a case without a verified freeze **and** owner authorisation |
| `driver.py` | **Case-posing driver**: a concrete plan per frozen case — pinned object, forced state, spike mode, argv, oracle expectation, evidence channel, outcome rule. `completeness()` proves the plan table is exactly the membership; `unposable_cases()` reports, statically, every case whose declared evidence channel the mechanism cannot supply |
| `observations.py` | **P-12**: the frozen function from observation to outcome token, built from closed vocabularies. Missing evidence yields no token, never a plausible one |
| `evidence.py` | **P-14**: the publication sanitiser and deterministic evidence serialization. An environment value is never reproduced |
| `make_fixtures.py` | Deterministic generator for `helper_foreign.elf`, `unloadable_in_cohort.elf`, `magic_only.bin`, `script_fixture.sh` |
| `launcher_spike.c` | The mechanism under test: pin → measure → admission → `clone3(CLONE_PIDFD)` → child setup → `execveat` |
| `helper_report.c` | Primary helper; reports its own observed process boundary from inside the executed image |
| `helper_alt.c` | Substitution detector: different body, different digest, marker at a fixed length so E6's mutation can be length-preserving |
| `helper_dynamic.c` | The one deliberately **dynamically linked** helper (E7) |
| `helper_fork.c` | Process-tree negative control, in two descendant shapes |
| `helper_setid.c` | Set-user-ID admission fixture (D-9); exists to be refused |
| `README.md` | What the sources are, and what they are not |

`helper_report.c` reports, from inside the executed image: its exact `argv` element by element
with lengths; its **complete** `environ`; its full descriptor set enumerated with
`fcntl(F_GETFD)` over `[0, RLIMIT_NOFILE)` — which **opens nothing** — plus each descriptor's
`FD_CLOEXEC` flag; its `SigBlk`, `SigIgn`, `SigCgt` and `SigPnd` masks, kept separate so
**blocked, ignored and caught/default are distinguishable**; its `NoNewPrivs` state; its real and
effective uid and gid; its working-directory identity as `st_dev`/`st_ino` of `.`, never as a
pathname; a self-identity marker; the requested stream volumes; and a requested exit code.

**Report channel.** The report is written to **descriptor 1 only**, behind the frozen sentinel
`HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n`; payload bytes precede it and report bytes follow. **No
descriptor above 2 is ever passed to a helper, in any case.** The original definition had the
report travel on "a descriptor the harness controls" — a descriptor above 2 that must survive
`close_range` into the executed image — so F1 and F4 would have scored PASS while "exactly 0, 1
and 2 survive" was false. `/proc/self/fd` is read only in F1 and F4 as a secondary cross-check,
and the descriptor that reading consumes is reported by number and is the only one excused.

> **CORRECTED — see the note that follows this one.** The finding below is preserved as recorded.
>
> **PRE-TRIAL FINDING `PRE-D7-B1` (driver implementation; LAUNCH-EXEC-01 still NOT_RUN): the
> report channel does not reach the harness.** Writing the case driver established that
> `launcher_spike.c`
> retains the capture prefix in a `malloc`'d buffer, fills it, and `free()`s it at the end of
> `main` without ever emitting it. The receipt carries `bytes_drained`, `drained_sha256`,
> `completeness` and the retained-prefix **counts** — never the bytes. A helper report is
> therefore produced inside the launcher and **no channel carries it out**, so every expectation
> written against the report is unobservable as frozen: `argv_exact`, `environ_empty`,
> `fds_exactly_012`, `signals_reset`, `no_new_privs_1`/`no_new_privs_0`,
> `interpreter_ran_with_devfd`, and the exec evidence that separates **S1 from S5**.
> `driver.unposable_cases()` computes the affected set statically, and the runner refuses to
> start a trial while any **mandatory** case is in it. Recorded here as a finding and **not
> corrected here**: the correction changes the mechanism under test, which is an owner decision
> rather than a driver detail.

> **`PRE-D7-B1` CORRECTION, by owner decision: the launcher emits the prefix it already retains.**
> Each stream block in the receipt now carries `capture_prefix_length`,
> `capture_prefix_truncated` and `capture_prefix_base64`. The encoding is deterministic and
> binary-safe because the frozen output recipe is binary by construction.
>
> **What did not change, and this is the point of choosing this shape:** `bytes_drained` and
> `drained_sha256` are still over *every* drained byte, `completeness` is untouched, and **no
> descriptor was added to the helper**. The report still travels on descriptor 1 behind the frozen
> sentinel, and the executed image still sees exactly `{0, 1, 2}` — so F1, F2, F3, F4, F6 and F7
> measure the same set they always did.
>
> **The retained bytes are INTERNAL observation input and are never published.** They are whatever
> the executed image wrote, which on a hosted runner can include environment values, absolute
> paths and credentials. The driver decodes them, derives tokens and sanitises the result;
> `evidence.INTERNAL_ONLY_KEYS` withholds the field wholesale by key, at any depth, because a
> secret encoded in base64 is still a secret and scanning an encoded blob is a game the scanner
> loses.
>
> **Absence of a report is only evidence when the stream was complete.** Six report states are
> distinguished — complete, truncated, malformed, absent, and stream-incomplete — and only
> `complete` produces a token. S2, S5 and S7 rest on a *decisive* absence; a truncated prefix or a
> retained writer is `stream_incomplete` and settles nothing.
>
> **Remaining gap: `M3`.** Section 4 permits a syscall record only for the seven cases declared
> `traced: true`, and M3 is not one of them, so no frozen evidence source can show that the pidfd
> was acquired in the *same syscall* that created the child. A receipt field naming the
> acquisition would be a self-assertion of exactly what M3 exists to evidence, and none was added.
> M3 is **the only unposable case** and is returned as an owner question.

> **`M3-T` AMENDMENT, by owner decision, made BEFORE the first valid trial.** The gap above is
> closed by amending the preregistration rather than by weakening the evidence rule. **M3 becomes a
> traced case**, so the traced set is amended prospectively from seven to **eight**:
> `E1, E7, F4, F7, M1, M2, `**`M3`**`, M4`. M3 stays **conditional** on `clone3_unavailable`, and
> the 72 / 54 / 11 / 7 partition is untouched. The earlier freezes are preserved exactly as they
> were; **none of them contained this rule**, and this note does not pretend otherwise.
>
> **The bounded claim, and only this claim** — frozen as `frozen_cases.M3_BOUNDED_CLAIM`:
>
> > For the direct child in this candidate execution, the pidfd used by the launcher was returned
> > through the `CLONE_PIDFD` facility of the **same** `clone3` syscall that created that direct
> > child, rather than being acquired later by `pidfd_open()` from a numeric PID.
>
> M3 does **not** establish that Linux pidfds are universally race-free, that the kernel
> implementation has been proved atomic, or any general claim beyond this one execution. It runs
> under a tracer, so it is also **not** evidence about timing-sensitive normal-launch behaviour;
> its evidence is restricted to the acquisition facts. `frozen_cases.M3_CLAIM_EXCLUSIONS` records
> those exclusions in the manifest rather than only in prose.
>
> **The evidence is external, never a self-assertion.** No `pidfd_acquisition` receipt field was
> added and none is read; the rule is written against the syscall record alone, and a test asserts
> that a receipt claiming `clone3` produces no token. The normalised trace must establish all of
> `frozen_cases.M3_EVIDENCE_FACTS` — **A** a parent-side `clone3` occurred; **B** its flags contain
> `CLONE_PIDFD`; **C** `clone_args.pidfd` is supplied as the kernel's output location; **D** the
> call created the direct child; **E** the same syscall yielded a pidfd; **F** no separate
> `pidfd_open()` acquired that child's handle; **G** the descriptor is correlated to the launcher's
> direct-child handle through the frozen lifecycle, by `waitid(P_PIDFD, …)`.
>
> **Fact E has two admissible forms**, recorded per run as `evidence_form`. *Direct*: the tracer
> printed the kernel's write-back (`=> {pidfd=[N]}`). *Closure*: it did not, and the record instead
> contains **no `pidfd_open` at all** alongside B, C, D and G — leaving no other route by which that
> descriptor could exist. Anything less is INVALID, never PASS.
>
> **`pidfd_open` on the direct child is rejected outright**, which is the distinction §5 of the
> amendment demands: `clone3(CLONE_PIDFD)` acquisition is not the same as
> `clone3`/`fork` → numeric PID → `pidfd_open(PID)`, and the latter is the design this mechanism
> rejected. A `pidfd_open` aimed at an unrelated process does not reject.
>
> **Host condition unchanged.** M3's only frozen block cause remains `clone3_unavailable`. If
> `clone3` is supported and the trace fails to establish A–G, that is **not** a legitimate
> conditional block — it is INVALID. The escape hatch was not broadened. One consequence follows
> and is stated rather than hidden: a host with no permitted tracer now makes M3 **INVALID** rather
> than BLOCKED, because `no_tracer` is not one of M3's frozen causes. The recommended runner has
> `strace`, directly observed, so M3 is posable there.
>
> **Raw identifiers stay local.** Pids and descriptor numbers are experiment-local; published
> evidence carries `DIRECT_CHILD` and `DIRECT_CHILD_PIDFD` plus booleans, decoded flag names and
> counts. The raw tracer text is never published — only its SHA-256.

**Frozen output recipe.** `byte[i] = (i * 251 + tag) mod 256`, `tag = 1` for stdout and `2` for
stderr. `oracles.py` computes every expected count and digest from this and the declared volume
alone. A byte *volume* is not a byte *recipe*, and without this the O-series oracles are
uncomputable.

**The O-series runs its helper with `--no-report`, and that is load-bearing.** The report and the
measured payload share descriptor 1, and the launcher retains only a bounded prefix, so for any
stream larger than that bound nobody ever holds the whole stream and the sentinel cannot be used
to split it after the fact. A stream carrying both would therefore have a `drained_sha256` over
payload **plus** sentinel **plus** report, which can never equal the frozen-recipe digest. With
`--no-report` the stream is exactly the recipe bytes, `bytes_drained` equals the declared volume
and `drained_sha256` equals the oracle digest. **Exec evidence for those cases is the payload
itself**: only the pinned helper can produce the recipe, so its presence establishes that the
image ran, without needing the report S1 relies on.

**Retained prefix.** `capture_prefix` retains the first `MAX_CAPTURE_BYTES` of each stream **in
memory only**, and draining continues past the bound so a child that writes more is never blocked
by a full pipe. The `truncated` flag describes **that buffer**, not the stream, which is why it
appears in the spike's `retained_prefix_not_in_receipt` object and **never in the receipt**.

**Test-only fault injection is separated from the mechanism sequence.** `frozen_cases` declares
`CHILD_TEST_INJECTION_SYSCALLS` and `CHILD_INJECTION_MODES` alongside
`CHILD_PERMITTED_SYSCALLS`. The child calls `nanosleep` under `--stall-pre-exec-ms` (S6) and
`kill`/`getpid` under `--die-before-exec` (S5); none belongs to the candidate mechanism sequence.
**M1, M2 and M4 trace the production configuration**, in which no injection mode is passed, and a
trace containing an injection syscall in a case that declared no injection is a **FAIL**. Without
that split M1's minimality claim would hold only because the injecting cases happen not to be
traced — an accident of classification rather than a rule.

**Frozen child-setup stage vocabulary**, in order: `RELOCATE`, `DUP2`, `CLEAR_CLOEXEC`, `CHDIR`,
`CLOSE_RANGE`, `SETPGID`, `SIGMASK`, `SIGACTION`, `NO_NEW_PRIVS`, `EXEC`. Every post-fork
operation maps to exactly one, and `NO_NEW_PRIVS` precedes `EXEC` (D-11). "The correct stage" is
unverifiable without a closed set.

**Static linking is a precondition, not a preference.** A complete trace is what makes the
negative claims "no other descriptor was present" and "no other file was opened" evidence rather
than filtering. Before any trial the harness compiles a probe with
`cc -O2 -Wall -Wextra -static` and verifies it with `ldd`. **If the static link fails for any
reason, every case is BLOCKED and the aggregate is `MECHANISM_INCONCLUSIVE`.** A dynamically
linked helper must **not** be substituted, in whole or for any individual case; `musl-gcc` must
not be substituted; no package is installed and no `sudo` is used. `build-essential` and
`libc6-dev` are **not** in the recommended runner's manifest, so this gate is live rather than
theoretical. The single exception is `helper_dynamic`, which exists precisely to expose the
loader activity every other helper excludes.

## 3. Preregistered cases — 72

Generated from `frozen_cases.py`. Every case has a **single frozen expected outcome**, except
where a safe outcome set is preregistered because every member is equally honest; any result
outside it is a FAIL, not a retry.

### E - executable identity and TOCTOU (12)

| # | Class | Expected | Case |
|---|---|---|---|
| **E1** | mandatory | `Exited:0` | normal pinned executable; marker matches; measured digest equals the independent hashlib digest taken before the run |
| **E2** | mandatory | `Exited:0` | rename(2) of helper_alt OVER the pathname after the pin; the pinned body runs and the marker is helper_report |
| **E3** | mandatory | `Exited:0` | original pathname renamed then unlinked after the pin |
| **E4** | mandatory | `Exited:0` | the pinned leaf was a symlink whose target is retargeted after the pin; the substituted body never executes |
| **E5** | mandatory | `ExecFailed:ETXTBSY` | the HARNESS process holds a second O_WRONLY descriptor on the same inode across launch; mandatory (st_dev, st_ino) oracle proves it is the same inode, never pathname equality |
| **E5b** | mandatory | `Exited:0` | writer opens O_WRONLY, writes and CLOSES before launch; no ETXTBSY, because the write-deny reference is taken at exec time |
| **E6** | mandatory | `Exited:0` | length-preserving ELF-valid mutation, writer closed before exec: the MUTATED marker runs and the recorded digest is pre-mutation. Documentation gate: PASSes only if the field is named pre_exec_body_sha256 and no text claims the measured bytes ran |
| **E6b** | mandatory | safe set: `Exited:mutated`, `ExecFailed:ENOEXEC`, `Signaled:SIGBUS`, `Signaled:SIGKILL` | ftruncate + rewrite to a different length. Every member is honest: a torn image fails the loader's header checks or faults beyond i_size. Any outcome PRESENTED as 'the measured body ran' is a FAIL. Documentation gate |
| **E6c** | recorded; gated: `no_claim_that_measured_bytes_ran` | safe set: `ExecFailed:ETXTBSY`, `Exited:mutated` | shared writable mapping survives the fd close. The one genuinely unresolved kernel question: whether a surviving i_mmap_writable mapping leaves i_writecount at zero could not be settled from primary sources, so this is recorded and carved out of the gate |
| **E6d** | conditional (BLOCKED if euid_zero) | `ExecFailed:EACCES` | fchmod to 0 between admission and exec; establishes that pre_exec_mode_bits is likewise a pre-execution measurement |
| **E7** | mandatory | `Exited:0` | the pinned object is DYNAMICALLY linked; the trace shows the kernel resolving PT_INTERP and ld.so resolving DT_NEEDED BY NAME. Without this no case executes a dynamic object at all. Also establishes that an empty environment does not pin the loaded-code closure |
| **E8** | mandatory | `refused:ElfNotInCohort` | foreign-architecture ELF (EM_AARCH64); execveat must never be reached. The runner's binfmt_misc registrations are recorded before the first trial |

### A - argv (5)

| # | Class | Expected | Case |
|---|---|---|---|
| **A1** | mandatory | `argv_exact` | argument containing spaces arrives as ONE byte-identical element |
| **A2** | mandatory | `argv_exact` | shell metacharacters and a newline arrive as one literal element; no shell, no expansion |
| **A3** | mandatory | `argv_exact` | empty argument arrives as one element of length 0 |
| **A4** | mandatory | `argv_exact` | argument of exactly MAX_ARG_BYTES arrives byte-identical |
| **A6** | mandatory | `argv_exact` | argv[0] differs from any pathname and is observed exactly |

### V - environment (D-10: empty only) (1)

| # | Class | Expected | Case |
|---|---|---|---|
| **V1** | mandatory | `environ_empty` | host sets HELM_LEAK_CANARY, LD_LIBRARY_PATH, PATH and HOME before launch; the child's environ is EXACTLY empty. Under D-10 this is the whole of the environment contract the mechanism can establish, and any inherited name is a FAIL |

### F - descriptor inheritance (7)

| # | Class | Expected | Case |
|---|---|---|---|
| **F1** | mandatory | `fds_exactly_012` | baseline; the set is verified IN THE EXECUTED IMAGE, and the pre-exec set {0,1,2,exec_fd,status_w} is not a violation |
| **F2** | mandatory | `fds_exactly_012` | an unrelated NON-CLOEXEC descriptor is open in the parent at exec time. Under frozen D-1 arm (i) the child not seeing it is the ONLY pass; there is no documented-failure acceptance path |
| **F3** | mandatory | `fds_exactly_012` | a CLOEXEC descriptor is open before launch |
| **F4** | mandatory | `fds_exactly_012` | neither the exec descriptor nor the exec-status pipe survives |
| **F5** | mandatory | `signals_reset` | harness blocks {SIGTERM, SIGUSR1} and sets SIGPIPE and SIGUSR2 to SIG_IGN. The child's SigBlk must be empty and SigIgn must contain nothing the launcher did not set. The helper reports SigBlk/SigIgn/SigCgt separately so blocked, ignored and caught/default are distinguishable |
| **F6** | mandatory | `fds_exactly_012` | harness closes 0, 1 and 2 before launch, so the launcher's own pipes and exec fd can land on 0-2. A child stdio descriptor with FD_CLOEXEC set (the dup2(fd,fd) no-op) is a FAIL |
| **F7** | mandatory | `fds_exactly_012` | exec fd and status write end at ADJACENT numbers; no close_range call may return EINVAL from an inverted gap |

### X - exec failure and admission (10)

| # | Class | Expected | Case |
|---|---|---|---|
| **X1** | conditional (BLOCKED if euid_zero) | `ExecFailed:EACCES` | execute bits cleared by fchmod AFTER admission recorded the mode; the recorded bits are the pre-fchmod value |
| **X2** | mandatory | `refused:ElfNotInCohort` | #! script refused at admission; execveat never reached |
| **X2b** | mandatory | `ExecFailed:ENOENT` | same script, admission bypassed in a frozen spike mode, exec fd O_CLOEXEC as mandated. ENOENT per execveat(2) BUGS, NOT ENOEXEC: the interpreter is never invoked and no procfs dependency is hit |
| **X2c** | mandatory | `interpreter_ran_with_devfd` | same script without O_CLOEXEC: the interpreter runs and receives /dev/fd/N. Recording X2b without X2c would attribute a CLOEXEC artefact to scripts as a class |
| **X3** | mandatory | `ExecFailed:EACCES` | a file that has never had an execute bit; distinct from X1 only in that no post-admission change occurred |
| **X4** | mandatory | `ExecFailed:ENOEXEC` | `unloadable_in_cohort.elf`: a 64-byte ELF64 header that PASSES the cohort rule -- ELFCLASS64, ELFDATA2LSB, EM_X86_64, ET_EXEC -- but has e_phnum = 0, so it is admitted and reaches execveat with nothing for the loader to map. The earlier 4-byte magic-only fixture could not pose this case at all: once admission became a 64-byte HEADER check rather than a four-byte magic check, a 4-byte file was refused as ElfNotInCohort and could never reach execveat to produce ENOEXEC |
| **X5** | mandatory | `refused:NotRegularFile` | capability on a directory descriptor |
| **X6** | mandatory | `refused:DescriptorModeUnsuitable` | capability on an O_PATH descriptor, obtained with the frozen spike mode --exec-fd-o-path. A HELM admission rule, NOT a kernel limitation: execveat(2) accepts O_PATH descriptors, and the refusal exists because the body cannot be measured through one. The mode and the DescriptorModeUnsuitable refusal both have to exist in the spike for this case to be posable at all |
| **X7** | mandatory | `refused:SetIdBitsPresent` | D-9: a set-user-ID object is refused at admission, deterministically. No privileged fixture and no cross-UID elevation is manufactured; the fixture is chmod u+s on a file the running user already owns |
| **X8** | conditional (BLOCKED if no_noexec_mount) | `ExecFailed:EACCES` | executable on a noexec mount; no mount is created and no sudo is used |

Every X case and case E5 must produce `ExecFailed` or an admission refusal, never `Exited { 127 }`.
Conflating them is a FAIL.

### O - output (8)

| # | Class | Expected | Case |
|---|---|---|---|
| **O1** | mandatory | `stream_exact` | stdout only, 4 KiB, mode measure; count and digest match the frozen-recipe oracle; completeness CompleteAtEof |
| **O2** | mandatory | `stream_exact` | stderr only, 4 KiB; correct stream; streams never merged |
| **O3** | mandatory | `stream_exact` | both streams emit 8 MiB concurrently, far past pipe capacity; both CompleteAtEof; completes under 10000 ms with timeout_ms=60000. A TimedOut outcome is a FAIL |
| **O4** | mandatory | `stream_exact` | output exceeds MAX_CAPTURE_BYTES; digest over ALL drained bytes, retained prefix exactly at the bound, and the in-memory truncated flag must NOT appear in the receipt |
| **O5** | mandatory | `stream_exact` | child closes stdout early and keeps writing stderr. The poll() return count must be at most 10000 and CPU under 200 ms, because a spinning launcher also satisfies 'no hang' |
| **O6** | mandatory | `Exited:0` | descendant inherits fds 1 and 2 and sleeps 30 s holding them while the direct child exits 0 immediately. launch() must return within the total bound and measurably before the sleep ends, with completeness WriterRetainedAfterChildExit. A non-return, a bare TimedOut, or CompleteAtEof is a FAIL |
| **O7** | mandatory | `Exited:42` | exit status AND a stderr capture failure are simultaneously true; the receipt must carry both. A receipt that can report only one of the two facts is a FAIL |
| **O8** | mandatory | `stream_exact` | POLLIN and POLLHUP in the same poll() return, 200 trials; any trial short of 512 bytes is a FAIL |

### R - exit and signal (4)

| # | Class | Expected | Case |
|---|---|---|---|
| **R1** | mandatory | `Exited:0` | helper exits 0 |
| **R2** | mandatory | `Exited:42` | helper exits 42 |
| **R3** | mandatory | `Signaled:SIGSEGV` | self-raised; the helper sets RLIMIT_CORE=0 first so no core dump is produced (systemd-coredump is installed on the runner) |
| **R4** | recorded; gated: `never_reports_exited_zero`, `never_hangs` | safe set: `ExitStatusUnobservable`, `Exited:42` | host sets SIGCHLD to SIG_IGN. Recorded because the outcome depends on a caller precondition the crate cannot enforce; gated because reporting a status it did not observe is never acceptable |

### T - timeout (6)

| # | Class | Expected | Case |
|---|---|---|---|
| **T1** | mandatory | `TimedOut` | helper sleeps well past timeout_ms |
| **T2** | mandatory | `TimedOut:KilledByLauncher:SIGKILL` | helper ignores SIGTERM and the grace window elapses |
| **T3** | mandatory | `TimedOut:ExitedDuringGrace:9` | frozen schedule: handler sleeps 500 ms then _exits 9, timeout_ms=2000, grace_ms=5000 |
| **T4** | mandatory | `Exited:9` | frozen schedule: sleeps timeout_ms-500 then _exits 9. With the pidfd polled the exit is OBSERVED against the deadline, so TimedOut{ExitedDuringGrace} is now a FAIL, not a safe set |
| **T5** | mandatory | `Exited:7` | child exits 7 just before the deadline while a descendant retains fds 1 and 2, so the exit is invisible without polling the pidfd. Any TimedOut disposition is a FAIL |
| **T6** | mandatory | `TimedOut:KilledByLauncher:SIGKILL` | harness parent has SIGTERM BLOCKED and SIGPIPE ignored; the wall-clock shape must match T2 from a default-signal parent, which it can only do if the child's signal state was reset |

### N - no_new_privs (D-11) (3)

| # | Class | Expected | Case |
|---|---|---|---|
| **N1** | mandatory | `no_new_privs_1` | D-11: the executed helper observes NoNewPrivs: 1 in /proc/self/status. This is the directly observed state |
| **N2** | conditional (BLOCKED if parent_no_new_privs_set) | `no_new_privs_0` | CONTROL: a frozen spike mode skips the prctl, and the helper must then observe NoNewPrivs: 0. Without this arm a positive N1 would be uninformative, because the runner could already have set no_new_privs process-wide. CONDITIONAL because that same possibility makes the control unposable: no_new_privs is inherited and CANNOT be cleared, so a parent that already has it set forces the child to observe 1. Preflight records the parent's own NoNewPrivs before any case; if it is 1, N2 is BLOCKED as a host property and N1 is then recorded as UNCONTROLLED in the report |
| **N3** | conditional (BLOCKED if unprivileged_runner) | `privilege_transition_suppressed` | the actual suppression of a set-user-ID or file-capability privilege TRANSITION. Expected BLOCKED on an unprivileged runner, and deliberately so: no privileged fixture is created to manufacture cross-UID elevation. The report must record this as an UNTESTED privileged transition resting on primary-source kernel semantics, never as a demonstrated one |

### P - process-tree negative controls (4)

| # | Class | Expected | Case |
|---|---|---|---|
| **P1** | recorded; gated: `launch_returns_within_total_bound`, `completeness_reported` | safe set: `descendant_survived`, `descendant_died` | descendant outlives its parent. Its first three actions after fork are to close 0/1/2 and reopen them on /dev/null, so a lifecycle control can never hang a mandatory case; O6 and P4 are the deliberate exceptions that retain the pipes |
| **P2** | recorded; gated: `launch_returns_within_total_bound`, `completeness_reported` | safe set: `descendant_survived`, `descendant_died` | same, with setsid; establishes that a group sweep is best-effort |
| **P3** | recorded; gated: `sweep_strictly_before_reap` | safe set: `sweep_issued`, `sweep_not_issued` | the sweep syscall, if issued, must appear strictly BEFORE waitid. After the reap the pgid may have been reused and the launcher would be signalling processes it never created |
| **P4** | recorded; gated: `launch_returns_within_total_bound`, `writer_retained_after_child_exit_reported` | safe set: `descendant_survived`, `descendant_died` | setsid descendant RETAINING fds 1 and 2; establishes that a group sweep does not release a pipe and that the bounded drain, not the sweep, is what makes the launcher terminate |

**Isolation and ordering of the negative controls.** P1 and P2 run **last**, after every mandatory
and conditional case has a recorded status. Their descendant's first three actions after `fork`
are to close 0, 1 and 2 and reopen them on `/dev/null`, so a lifecycle control can never make a
mandatory case hang; it signals liveness through a harness FIFO and `_exit`s after
`P_DESCENDANT_LIFETIME_MS = 20000`. **O6 and P4 are the deliberate exceptions** that retain the
pipes, because that is the state under test, and both are gated on bounded return. The harness
confirms no descendant remains before declaring the run complete. A P-case whose `fork` did not
occur, or whose descendant is still alive at run end, is **INVALID**.

Descendant survival is **recorded, not predicted** — the point is to establish a limitation
honestly. The **liveness sub-assertions are gates**, because a launcher that never returns is not
a limitation.

### S - spawn/exec confirmation (7)

| # | Class | Expected | Case |
|---|---|---|---|
| **S1** | mandatory | `Exited:0` | clean EOF with no record AND the helper report received on fd 1 AND the direct child exited normally. The report is the exec evidence; clean EOF alone is not a PASS |
| **S2** | conditional (BLOCKED if euid_zero) | `ExecFailed:CHDIR:EACCES` | the directory capability's own descriptor is fchmod'ed to 0000 after admission, so the child's fchdir fails. No helper report may be received: its absence is the independent proof that the image never ran |
| **S3** | mandatory | `Exited:127` | the helper itself exits 127 after a SUCCESSFUL exec and must not be confused with exec failure |
| **S4** | mandatory | `Exited:7` | rapid exec-and-exit: the parent sleeps 200 ms after clone3 returns and before it polls, so the child has certainly execed and exited first. A forced schedule, not a timing hope |
| **S5** | mandatory | `ExecStatusIndeterminate` | child killed between the last setup stage and execveat. The parent observes clean EOF with no record, BYTE-IDENTICAL to S1, and no helper report. Reporting exec success is a FAIL and a FAIL of falsifier 8. The discriminating case for the whole confirmation design |
| **S6** | mandatory | `ExecStatusIndeterminate:PreExecTimeout` | pre-exec stall past SPAWN_CONFIRM_TIMEOUT_MS. The launcher must terminate and reap: a leaked child or zombie is a FAIL |
| **S7** | conditional (BLOCKED if euid_zero) | `ExecFailed:CHDIR:EACCES` | S2's construction 200 times, so the record and the hangup arrive in the same poll() return. Any trial reporting exec success, Exited:127 or ExecStatusIndeterminate is a FAIL |

S3 and S5 together are the discriminating pair: S3 is the case a naive implementation reports as
exec failure, and S5 is the case it reports as exec success.

### M - mechanism minimality and parent shape (5)

| # | Class | Expected | Case |
|---|---|---|---|
| **M1** | conditional (BLOCKED if no_tracer) | `child_syscalls_within_frozen_set` | the traced child window from the clone3 return to execveat. This is the evidence that REPLACES the architecture's assertion of 'no allocation, no locking, no formatting, no panic path' |
| **M2** | conditional (BLOCKED if no_tracer) | `identical_to_single_threaded_arm` | parent has >=3 extra live threads, one with an allocation in flight and one with a registered pthread_atfork handler. Without this the mechanism is evidenced only for a single-threaded parent, which no real HELM caller is |
| **M3** | conditional (BLOCKED if clone3_unavailable); **traced** under amendment `M3-T` | `pidfd_acquired_atomically` | **Amended `M3-T`:** established from the EXTERNAL syscall record, never from a launcher field naming its own acquisition. The bounded claim, the seven facts A–G and the two admissible forms of fact E are in the `M3-T` note in section 2. clone3(CLONE_PIDFD) is the PRIMARY acquisition: it removes the three pidfd_open caller preconditions a library cannot establish, and avoids the pthread_atfork surface glibc's fork() opens. CONDITIONAL because a kernel version floor establishes that the syscall EXISTS, not that it is PERMITTED: a seccomp policy can reject clone3 on a supporting kernel. Preflight probes availability without creating a child; an unavailable clone3 disables the whole mechanism and is a preflight gate, so every case is then BLOCKED rather than this one case FAILing |
| **M4** | conditional (BLOCKED if no_tracer) | `sequence_matches_frozen_stages` | the implemented child sequence is matched syscall-for-syscall against the frozen STAGES order, including that CHDIR precedes CLOSE_RANGE and NO_NEW_PRIVS precedes EXEC. A spike whose sequence differs is INVALID, not PASS |
| **M5** | recorded; gated: `never_reports_unobserved_exit_status` | safe set: `pidfd_open_esrch`, `waitid_echild`, `pidfd_open_succeeded` | the REJECTED fork+pidfd_open acquisition under SIGCHLD=SIG_IGN, recorded to evidence why clone3(CLONE_PIDFD) is primary rather than asserting it. Not a candidate mechanism |

## 4. Instrumentation

Authoritative evidence, in preference order:

1. **The helper's own report**, on descriptor 1 behind the frozen sentinel — the primary evidence
   for argv, environ, descriptors, signal state, `NoNewPrivs`, credentials and cwd identity,
   because it is observed from inside the executed image.
2. **A syscall record of the launcher**, collected **only** for the eight cases declared
   `traced: true` — **E1, E7, F4, F7, M1, M2, M3, M4** — establishing that
   `execveat(exec_fd, "", …, AT_EMPTY_PATH)` was the syscall used on the pinned descriptor and
   that the child window contains nothing else. Collected with `strace` if preflight finds it,
   otherwise a purpose-built parent-side `ptrace` tracer if `ptrace_scope <= 1`, otherwise those
   cases are **BLOCKED**. **Directly observed on the recommended runner, 2026-09-09:** `strace`
   **is** present at `/usr/bin/strace` and `ptrace_scope` is `1`, so the traced cases are posable
   there. An earlier form of this definition asserted the opposite, inferred from the
   runner-image package manifest; the compile-only pre-trial job observed the runner itself and
   the claim is corrected rather than carried forward. Availability is still re-probed at
   preflight and never assumed; no `sudo` and no package installation is authorised.
3. **`/proc/<pid>/fd` inspection** from the harness, as a cross-check.
4. **Independent oracles** in Python: `hashlib` over the **frozen byte recipe**, never reading
   spike output.

**Tracing authority.** Every case declares `traced` in `frozen_cases.py`. **Every R, T, O, P and
S case declares `traced: false`**, because `ptrace(2)` delivers a tracee's exit and signal
notifications to the **tracer** before the real parent, and the launcher *is* the real parent
whose `waitid` classification those cases exist to measure. A case declared `traced: false` that
is nevertheless observed under a tracer is **INVALID**, not FAIL. The manifest's own self-tests
enforce this.

**No retries.** A case is run once per trial. A result outside its prediction or safe set is a
FAIL, and FAIL, INVALID and BLOCKED results are preserved rather than re-run.

**Freeze point.** Every artefact in section 2 is committed and hashed in `SOURCE-HASHES.json`
before any trial. **The first valid trial** is the first execution of any preregistered case
under the frozen definition on the recorded environment; every earlier run is preparation. After
it, **no artefact in section 2 may be edited**: a defect discovered afterwards ends the trial and
starts a new, separately recorded one.

**Binary evidence lives outside the source freeze, and this is deliberate.**
`SOURCE-HASHES.json` is **immutable once written**: writing built-binary digests into it would
change the file, and therefore its own digest, so the artefact certifying the freeze would no
longer be the artefact that was frozen. Build evidence is instead an **append-only** document,
`BUILD-EVIDENCE.md` in the experiment directory, one section per build, each recording the exact
source freeze SHA it was built from, the runner and kernel, the architecture, the compiler
identity and version, the exact link command, the link result, the SHA-256 of every produced
binary, the static/dynamic inspection output, and the SHA-256 of every generated fixture. **No
binary hash is ever inserted retroactively into an earlier commit.**

**Preflight halt.** If, before the first valid trial, a frozen expectation is found to be
factually wrong about documented Linux behaviour, execution **halts** and the correction is put to
the owner as a separate decision, exactly as
[OBS-FS-01 did](OBS-FS-01-PREFLIGHT-AND-HALT.md). Correcting an expectation after observing
behaviour is goalpost movement and is forbidden. **The C sources have not been compiled on
Linux**, so a build failure at preflight is a preflight finding to be recorded under this rule,
not a defect to be quietly patched after the freeze.

## 5. Privacy and sanitisation

The spike is started from a **scrubbed** parent environment containing only the names declared in
`frozen_cases.py` plus the V1 canaries, and the harness records that scrubbing. The published
report replaces the runner work directory with `<WORK>` and the home directory with `<HOME>`, and
contains **no** absolute host path. Any environment name observed in a child that is not declared
is reported as `<UNDECLARED:` first eight hex of its SHA-256 `>` and its value is **never**
reproduced, so a V1 leak is detectable without republishing what leaked — on a hosted runner the
inherited environment includes `ACTIONS_RUNTIME_TOKEN` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN`.
Captured stream bytes are never published: only counts, digests and completeness. R3 sets
`RLIMIT_CORE = 0` before raising, so no core dump is produced.

## 6. Aggregate verdict rules

**Per-case status**, one of exactly four, assigned once and never revised:

- **PASS** — posed as written, outcome is its single frozen prediction or a member of its frozen
  safe set, and every frozen invariant held.
- **FAIL** — posed as written and its outcome is not that.
- **INVALID** — could not be *posed*: a fixture could not be built, a required forced state did
  not land, a required oracle could not be computed, a traced case produced no trace, or the case
  was observed under an instrumentation its `traced` declaration forbids.
- **BLOCKED** — could be posed but the *environment* cannot host it, with a cause named in the
  frozen `BLOCK_REASONS` set.

A case in the frozen membership with **no recorded status is INVALID, never absent** — absence is
not a recorded environmental cause, and it is never silently ignored.

**Only a conditional case may absorb a BLOCKED status, and only with the cause its own manifest
entry names.** Letting any class absorb a block would retire a load-bearing gate by
reclassification: a *recorded* case scored BLOCKED would skip its gated sub-assertions — for
P1–P4 those are the liveness gates that close the descendant-held-pipe hole — while the run still
reached ACCEPTED. A mandatory or recorded case recorded as BLOCKED is **INVALID**.

**The harness must record whether `launch()` returned, always.** A record lacking
`launch_returned` is **INVALID**. A hang is exactly the case where a record could otherwise be
absent, and an absent record would understate a known result as an open question; the harness
therefore emits `launch_returned: false` from its own watchdog, and that is a **FAIL**.

**Frozen block reasons**, and nothing else may be used: `euid_zero`, `no_tracer`,
`no_noexec_mount`, `unprivileged_runner`, `parent_no_new_privs_set`, `clone3_unavailable`. The
last two exist because a host property must never be scored as a falsified mechanism claim:
`no_new_privs` is inherited and irreversible, so a parent that already has it set makes N2's
control arm unposable; and a kernel version floor establishes that `clone3` *exists*, not that
policy *permits* it. **If N2 is BLOCKED, N1's observation is recorded as UNCONTROLLED** — the
positive result stands, but without its control it cannot distinguish "the launcher set the bit"
from "the bit was already set".

**Class partition**, frozen in `frozen_cases.py` before the first trial and derived, not typed:

| Class | Count | Members |
|---|---|---|
| **Mandatory** — must PASS | **54** | E1–E5, E5b, E6, E6b, E7, E8; A1–A4, A6; V1; F1–F7; X2, X2b, X2c, X3–X7; O1–O8; R1–R3; T1–T6; N1; S1, S3, S4, S5, S6 |
| **Conditional** — may be BLOCKED with a frozen cause | **11** | E6d, X1, X8, N2, N3, S2, S7, M1, M2, M3, M4 |
| **Recorded** — outcome not predicted; gated on named sub-assertions | **7** | E6c, R4, M5, P1, P2, P3, P4 |
| **Total** | **72** | |

**D-1 is a frozen input, not a scoring choice.** The owner's arm — **(i)**, the scoped unsafe
backend with exact FD isolation claimed — is recorded in `frozen_cases.py` before the first trial
and **cannot be selected after any result is known**. F2 is therefore **mandatory**, its only PASS
is that the child does not see the descriptor, and **there is no documented-failure acceptance
path**.

**Aggregate precedence.** Applied in this order; evaluation stops at the first rule that fires.
The order is **total**: every assignment of statuses to the frozen membership reaches exactly one
verdict, and the three verdicts are disjoint.

| Order | Verdict | Condition |
|---|---|---|
| 1 | **MECHANISM_REJECTED** | Any mandatory **or conditional** case is FAIL; **or** any recorded case fails a gated sub-assertion; **or** `launch()` fails to return within its declared total bound in **any** case, including P1–P4; **or** the receipt asserts a temporal or causal fact the launcher did not observe |
| 2 | **MECHANISM_INCONCLUSIVE** | No such FAIL, and any **mandatory** case is INVALID or BLOCKED, or any conditional or recorded case is INVALID |
| 3 | **MECHANISM_ACCEPTED** | Otherwise: every mandatory case PASS, every conditional case PASS or **BLOCKED with a frozen cause**, and every recorded case carrying a non-INVALID outcome that passes its gates |

A **conditional case BLOCKED with a recorded cause does not make the run inconclusive** — that is
what the class means, and a rule that treated it otherwise would make the class meaningless and
would render the whole experiment inconclusive by construction, since **N3 is BLOCKED by design on
any unprivileged runner**. A **mandatory** case BLOCKED still does.

Precedence 1 over 2 follows OBS-FS-01's frozen precedence — FAIL before BLOCKED before
INCONCLUSIVE before PASS: a falsified claim is a result, and an unposable case never converts it
into an open question.

**Instant rejection.** A single FAIL of any of these fourteen rejects the mechanism outright,
because each falsifies a claim the architecture is built on — pinned execution, the receipt's
executable identity, cohort admission, exact descriptor isolation, the D-11 privilege contract,
deadlock-free draining, bounded return, and the exec-confirmation inference:

> **E2, E4, E6, E6b, E8, F2, N1, N2, O3, O6, S4, S5, S7, X7**

**A launcher-side non-return is never a recordable outcome.** It is a rejection, in every case
including the negative controls. This is the single rule that closes the acceptance path the
original 43-case definition left open.

**E6, E6b and E6d are documentation gates as well as mechanism gates.** They cannot be PASSed by
observing the expected kernel behaviour while the architecture, the ADR or the receipt schema
still presents the pre-execution measurement as the identity of the executed body.

No aggregate verdict may be reported as a percentage, and there is no partial credit.

The experiment establishes **mechanism viability only**. It confers no permission to implement,
no compatibility claim, no sandbox claim and no readiness claim, and **A0-7ZIP remains
experimental FAIL** irrespective of its outcome.

## 7. Environment

**Recommended first environment: a GitHub-hosted `ubuntu-24.04` runner.** It is unprivileged,
disposable, already used by this repository's CI, and its kernel is far above the 5.9 floor.

**Preflight inventory, recorded before the first case:** `uname -a`, `/etc/os-release`,
`uname -m`, the kernel release, the **glibc version**, the target triple, `geteuid`,
`kernel.yama.ptrace_scope`, `command -v strace`, the static-link probe, `getconf PAGESIZE`, the
pipe capacity via `fcntl(F_GETPIPE_SZ)`, **`/proc/sys/fs/binfmt_misc/status` and every
registration**, and the `nosuid`/`noexec` mount survey from `/proc/self/mountinfo`.

**Workflow discipline.** The experiment workflow sets `cancel-in-progress: false` — the existing
workflows set it true, which would truncate a trial mid-run — has no `pull_request` trigger, and
runs only on `workflow_dispatch`. GitHub's "re-run failed jobs" is forbidden; any re-run is a
**new trial** with its own report. Evidence is preserved under `docs/experiments/evidence/`.

**A0 must not be reused or modified for this experiment**, and no preserved A0 evidence is opened.

If a case needs a specific older kernel, a mount configuration a hosted runner cannot provide, or
a privileged operation, that case is marked **BLOCKED** and a fresh disposable lab is proposed in
a **separate** owner request. No `sudo`, no privilege change and no lab modification is authorised
by this definition.

## 8. What this definition does not authorise

Running the experiment; creating `crates/helm-launch`; executing Wine or 7-Zip; touching A0;
modifying any lab; changing any existing crate; or treating any result here as acceptance of
ADR-0024.

**The case set is now frozen and D-9, D-10 and D-11 are resolved, but D-7 — execution
authorisation — is NOT granted.** Freezing a definition is what makes it reviewable, not what
makes it runnable. `run_launch_exec_01.py` refuses to pose a case without both a verified source
freeze and an explicit owner-authorisation flag, and the case-posing driver is deliberately left
unimplemented until D-7 is granted, so that no path in this repository can pose a case by
accident.
