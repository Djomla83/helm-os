# LAUNCH-EXEC-01 independent pre-trial review

**Role:** independent pre-trial review of the frozen mechanism and its disposable C/Python
harness, before D-7 execution authorisation.\
**Reviewed candidate:** `a8a7f1978e78a45d2208cb57f2178081a1fa4570` on
`docs/helm-launch-architecture`.\
**Frozen experiment-source commit:** `66bf4b612733c878fe145b8c99df65a21198940a`, preserved
unchanged.\
**Authority:** Proposed [ADR-0024](../adr/ADR-0024-launch-authority.md). Not Accepted, and this
review does not accept it.

> **No preregistered case was posed.** No helper, fixture or spike binary was executed. The
> `--i-have-owner-authorisation-d7` flag was never passed, the case-posing driver was not
> implemented, and no Wine, 7-Zip or A0 access occurred. **LAUNCH-EXEC-01 remains NOT_RUN.**

**First-pass classification: NEEDS_PRETRIAL_FIXES.** Six BLOCKER and six IMPORTANT findings.
None changes an owner decision or the ADR's architecture; all are bounded pre-trial corrections
of the kind section 24 of the owner instruction permits, because zero trials have occurred.

This section-3 finding set is committed **before any correction**, so the reviewed candidate is
preserved rather than tidied away.

> **Final classification after corrections and Linux compile evidence:
> READY_TO_AUTHORIZE_FROZEN_TRIAL.** See [section 6](#6-corrections-applied) for what changed,
> [section 7](#7-linux-compile-evidence) for the build, and
> [section 8](#8-final-classification) for the basis. The superseded freeze `66bf4b6` is
> preserved unchanged; the corrected sources are frozen anew. **LAUNCH-EXEC-01 is still NOT_RUN,
> ADR-0024 is still Proposed, and D-7 is still not granted** — this recommends authorising the
> trial, it does not authorise it.

> **Read [section 10](#10-post-review-note-the-case-driver-was-implemented-still-not_run) before
> acting on the classification above.** That classification assessed the **mechanism and its
> source freeze**, at a time when the case-posing driver did not exist — as section 8 says in
> terms. Implementing the driver afterwards surfaced a new BLOCKER (**D-1**: the helper report
> never leaves the launcher), which makes **43 of the 72 cases unposable, 34 of them mandatory**.
> Sections 1–9 are preserved exactly as written; section 10 is appended, and the classification
> of the **new** freeze is
> [`DRIVER_IMPLEMENTATION_NEEDS_OWNER_DECISION`](#11-classification-of-the-new-freeze).

## 1. Independently reconstructed state

Nothing below is taken from the author's summary; every value was recomputed.

| Claim | Method | Result |
|---|---|---|
| `origin/main` | `git rev-parse` | `5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e`, as instructed |
| branch tip, local and remote | `git rev-parse` | both `a8a7f1978e78a45d2208cb57f2178081a1fa4570` |
| strict ancestry | `git merge-base --is-ancestor` | main, `55f0f78` and `66bf4b6` are all ancestors of the tip |
| divergence | `git rev-list --count HEAD..origin/main` | **0** |
| merge commits | `git rev-list --merges --count` | **0** |
| every hashed file | independent SHA-256 over the **git object store** at `a8a7f19` | **13/13 exact match, 0 mismatches** |
| manifest self-reference | key inspection | does **not** hash itself; no file present-but-unhashed, none hashed-but-absent |
| definition content digest | independent SHA-256 | `08cc3b10…108f20`, equals the owner-stated value |
| architecture / ADR digests | independent SHA-256 | both match the manifest |
| frozen inputs | manifest fields | `status = NOT_RUN`, `d7_execution_authorised = false`, ten decisions D-1…D-11 |
| fabricated binary digests | `built_binary_sha256` key scan | **none**; only an explanatory `_note` |
| membership | independent recount, not the author's `validate()` | **72 ids, 0 duplicates**, classes disjoint, union equals membership, **56 / 9 / 7** |
| mandatory scoring escape | scan for `blocked_if` on mandatory cases | **none** |
| traced set | independent scan | `E1, E7, F4, F7, M1, M2, M4`; **no R/T/O/P/S case is traced** |

**The spike's hand-rolled SHA-256 was verified by transliteration**, not by execution: the exact C
was transliterated to Python including its padding logic and compared against `hashlib` across
the boundary lengths that break naive implementations (0, 55, 56, 57, 63, 64, 65, 119, 120, 127,
128 bytes, 3.3 KiB, and 200 000 bytes fed in 64 KiB chunks as the spike feeds it). **All match.**
E1's and E6's identity comparison therefore rests on a correct digest. No finding.

**Checker algebra was attacked with fabricated records only.** Every class × every status, every
conditional case blocked with each frozen cause, all conditionals blocked simultaneously, and
every single-case perturbation across the whole membership. The precedence is total and every
perturbation lands in exactly one of the three frozen verdicts. Two defects surfaced anyway, as
P-3 and P-8 below.

## 2. What was verified sound

Recorded because a review that lists only defects misrepresents the artefact.

- The freeze is real and exact: 13/13 file digests, and the definition, architecture and ADR
  content digests all match independently.
- The membership arithmetic is correct and the class partition is a genuine partition.
- The tracing rule holds: no case that measures `waitid` classification is traced, which is what
  keeps `ptrace`'s notification reordering out of the R/T/O/P/S results.
- The child sequence order is correct where it matters most: `CHDIR` precedes `CLOSE_RANGE`, so
  the working-directory descriptor is used before the range close destroys it; `FD_CLOEXEC` is
  explicitly cleared on 0/1/2 rather than relying on `dup2`'s non-identity behaviour; and the
  `close_range` gap arithmetic guards both inverted-range cases.
- The parent closes its own copies of all four pipe endpoints immediately after the clone, which
  is what makes every stated EOF protocol reachable.
- The group sweep is issued strictly before `waitid`, so the process-group ID cannot have been
  recycled.
- `POLLHUP` is never acted on before `read()` returns 0, so a child that writes the exec-status
  record and exits cannot be reported as a successful exec.
- SHA-256 is correct, as above.

## 3. First-pass findings

### BLOCKER

**P-1 — N2 cannot be posed on a host whose parent already has `no_new_privs`, and the freeze
classifies that environment as a mechanism defect.**
`no_new_privs` is inherited across `fork` and `execve` and **cannot be cleared once set**. N2 is
the control arm that must observe `NoNewPrivs: 0` when the spike skips the `prctl`. If the
runner's parent process already has the bit set — a containerised job, a hardened runner image, a
job step run under a wrapper that sets it — the child inherits `1` and N2 can **never** observe
`0`. N2 is **mandatory and on the instant-rejection list**, so that environment produces
`MECHANISM_REJECTED`: an inherited host property reported as a falsified mechanism claim. This is
exactly the misclassification the owner instruction warns against.
`harness.preflight()` does **not** record the parent's `NoNewPrivs` (verified: no occurrence in
`harness.py`), and `BLOCK_REASONS` has no cause that could absorb it. **The current freeze cannot
represent this honestly.**

**P-2 — `clone3` availability is assumed, and its absence would also present as a mechanism
failure.**
The kernel-version floor establishes that the syscall *exists*, not that it is *permitted*.
A seccomp policy — Docker's default profile is the well-known case — can return `EPERM` or
`ENOSYS` for `clone3` on a kernel that supports it. `launcher_spike.c:486` handles this as
`die("clone3")`: `exit(2)`, no record, no receipt. M3 is **mandatory**, and there is no frozen
block reason and no preflight probe for it (verified: no `clone3`, `seccomp` or `ENOSYS`
handling anywhere in `harness.py`). Since `clone3` is now the primary acquisition, its
unavailability disables the whole mechanism, so this belongs in preflight as a gate, alongside
the static-link gate — not in a case result.

**P-3 — the checker lets any class absorb a `BLOCKED` status, so a recorded case's gated
sub-assertions can be skipped while the run is still ACCEPTED.**
`checker.score_case` validates only that the cause is in the frozen set; it never checks that the
case's **class** may be blocked. Only conditional cases have a `blocked_if` in the manifest, yet
a *recorded* case scored `BLOCKED` returns `BLOCKED`, and `verdict` excludes recorded-BLOCKED
from its unposed set. Demonstrated with fabricated records:
`recorded P1 BLOCKED -> case=BLOCKED aggregate=MECHANISM_ACCEPTED`.
P1–P4's gates are `launch_returns_within_total_bound` and
`writer_retained_after_child_exit_reported` — **the very assertions that closed the
descendant-held-pipe BLOCKER of the previous review**. A driver that derives block reasons from
the environment without checking class would silently retire them.

**P-4 — X6 is mandatory and unposable: the spike has no `O_PATH` mode and cannot emit
`DescriptorModeUnsuitable`.**
X6 expects `refused:DescriptorModeUnsuitable`. The spike opens the executable as
`O_RDONLY | O_CLOEXEC` with no flag to request `O_PATH`, and its admission emits only
`NotRegularFile`, `SetIdBitsPresent` and `ElfNotInCohort` (verified by exhaustive grep of the
refusal assignments). The expected token is unreachable, so a correct mechanism scores FAIL.

**P-5 — X4 is mandatory and unposable: its fixture is refused at admission and can never reach
`execveat`.**
X4 expects `ExecFailed:ENOEXEC` from "a 4-byte file that is exactly the ELF magic". Admission now
reads 64 bytes and requires `n >= 20` plus `EI_CLASS`, `EI_DATA`, `e_machine == EM_X86_64` and
`e_type ∈ {ET_EXEC, ET_DYN}`. `magic_only.bin` is **4 bytes**, so `n >= 20` fails and it is
refused as `ElfNotInCohort` — `execveat` is never reached and `ENOEXEC` is unreachable. The case
text is a leftover from when admission was a four-byte magic check, and the A-4 correction that
replaced magic with a header cohort silently invalidated it.

**P-6 — O4 is mandatory and unposable: the spike implements no capture-prefix retention.**
O4 expects "retained prefix exactly at the bound" and an in-memory `truncated` flag. The spike
has **no** capture mode, no retained buffer and no truncated flag; `MAX_CAPTURE_BYTES` does not
appear in `launcher_spike.c` at all. It hashes and counts every drained byte and discards them.
The expectation describes a mechanism the frozen spike does not contain.

### IMPORTANT

**P-7 — the O-series digest evidence is unobtainable as frozen, because the report shares
descriptor 1 with the measured payload and no bytes survive to be split.**
The sentinel protocol assumes someone can split the stream into payload and report. Nobody can:
the launcher hashes all drained bytes into one digest and retains none, so the checker receives
`bytes_drained` and `drained_sha256` over **payload + sentinel + report**, which cannot equal the
frozen-recipe digest over the payload alone. `oracles.split_report` and `oracles.payload_matches`
have no real input from a run. O1 and O2 would fail against a correct mechanism whenever the
helper emits a report; O3 (8 MiB) and O4 could not be split even if bytes were retained. Each
O-case must declare whether its helper emits a report, and the digest each expectation refers to
must be named.

**P-8 — a launcher non-return that leaves no record scores INCONCLUSIVE, not REJECTED.**
The definition's strongest rule is that a launcher-side non-return "is never a recordable
outcome. It is a rejection, in every case including the negative controls." The checker enforces
that only when a record exists carrying `launch_returned: false` or an over-bound `elapsed_ms`.
A true hang is precisely the case where the harness may record nothing, and a missing record
scores INVALID → `MECHANISM_INCONCLUSIVE` (demonstrated with a fabricated record). That
understates a known result as an open question. The harness must be required to emit
`launch_returned: false` from its own watchdog.

**P-9 — test-only child injections are in the post-clone path but outside the frozen permitted
syscall set, and nothing separates them from the mechanism sequence.**
`child_main` calls `nanosleep` under `--stall-pre-exec-ms` (S6) and `kill(getpid(), SIGKILL)`
under `--die-before-exec` (S5). None of `nanosleep`/`clock_nanosleep`, `kill` or `getpid` is in
`CHILD_PERMITTED_SYSCALLS`. M1 does not currently catch this only because S5 and S6 happen to be
`traced: false` — an accident of case classification, not a rule. The protocol must distinguish
the **candidate mechanism child sequence** from **declared test-only fault injection**, and M1
must be defined as tracing the production configuration, or its minimality claim is tautological.

**P-10 — `helper_report` has no fixed-length locatable marker, so E6's length-preserving mutation
is under-specified.**
`helper_alt.c` declares `volatile char g_marker[16]` precisely so a mutation can be
length-preserving. `helper_report.c` has only `const char *marker = "helper_report";` — a bare
string literal of a different length, with no guarantee of a unique or stable location in the
built image. E6 and E6b require flipping that marker in place without changing file length; as
frozen, the fixture construction depends on searching the binary for a 13-byte literal.

**P-11 — the source freeze is designed to be mutated after freezing.**
`SOURCE-HASHES.json` carries a `built_binary_sha256` object whose `_note` says digests are
"recorded by the harness at preflight of the first authorised trial". Writing them changes the
frozen file, and therefore its own digest, so the artefact that certifies the freeze would no
longer be the artefact that was frozen. Binary evidence belongs in a **separate, append-only
preflight document**, leaving `SOURCE-HASHES.json` immutable.

**P-12 — the observation-to-outcome-token mapping is not frozen.**
The manifest freezes expected outcomes as tokens (`argv_exact`, `fds_exactly_012`,
`stream_exact`, `no_new_privs_1`, `signals_reset`, …). Nothing maps the spike's JSON to those
tokens; that mapping lives in the case-posing driver, which is deliberately unimplemented. So the
function from observation to verdict is scheduled to be written **after** results could be known,
which is where goalpost movement would occur. It must be frozen as part of the D-7 work, and its
absence must be an explicit pre-trial obligation rather than an omission.

### MINOR

**P-13 — `move_above_2` re-adds `CLOEXEC` to a deliberately non-`CLOEXEC` exec descriptor.**
It relocates with `F_DUPFD_CLOEXEC`. Under `--exec-fd-no-cloexec` (X2c) this would silently
restore the flag the case exists to remove, whenever the descriptor landed at 0–2. X2c does not
close stdio today, so it is latent rather than active.

**P-14 — publication sanitisation is unwired.** `run_launch_exec_01.DECLARED_ENV_NAMES`,
`sanitise_env_name` and `sanitise_path` exist but nothing calls them, because the driver is
absent. The helper reports complete environment values by design, so the sanitiser must be wired
before any report is published.

**P-15 — `helper_setid` is chmod'ed `04755` unconditionally.** On a `nosuid` filesystem the bit
is inert at exec, but X7 is an admission test reading `st_mode` from `fstat`, so the case still
poses correctly. Recording only; preflight already surveys `nosuid` mounts.

## 4. Verdicts by area

| Area | Verdict |
|---|---|
| Freeze integrity and hashes | **SOUND** — 13/13 exact, self-reference correct, no fabricated digests |
| Manifest and counts | **SOUND** — 72 / 56 / 9 / 7 independently reproduced |
| Checker algebra | **SOUND WITH CORRECTIONS** — total and disjoint, but P-3 and P-8 |
| D-11 / N1 / N3 | **SOUND** — `prctl` is the last stage before exec, N1 is kernel-backed, N3 is honestly BLOCKED |
| N2 parent precondition | **BLOCKER** — P-1 |
| `clone3` / pidfd | **SOUND WITH CORRECTIONS** — ABI and ordering correct; availability unhandled, P-2 |
| Child setup / FD isolation | **SOUND** — ordering, relocation, CLOEXEC clearing and gap arithmetic all correct |
| Signal reset | **SOUND** — mask cleared and dispositions reset before exec; F5 observes the three states separately |
| Exec confirmation | **SOUND** — clean EOF alone never means exec; S5 remains discriminating |
| Output / drain | **BLOCKER** — P-6, P-7 |
| Descendants / process group | **SOUND WITH CORRECTIONS** — sweep ordering correct; P-3 could retire the gates |
| Executable identity | **SOUND WITH CORRECTIONS** — wording is clean and the digest is correct; P-10 |
| Admission / cohort | **BLOCKER** — cohort rule is correct and stronger than magic, but P-4 and P-5 |
| Environment / privacy | **SOUND WITH CORRECTIONS** — `envp` is exactly empty by construction; P-14 |
| Source / binary hash lifecycle | **IMPORTANT** — P-11 |

## 6. Corrections applied

All bounded, all pre-trial, none touching an owner decision or the ADR's architecture. Membership
stays **72**; the class partition moves to **54 mandatory / 11 conditional / 7 recorded**, because
two cases that were mandatory turned out to depend on host properties.

| Finding | Correction |
|---|---|
| **P-1** | N2 becomes **conditional** on a new frozen cause `parent_no_new_privs_set`, and preflight reads the launcher's own `NoNewPrivs` before any case. If N2 is BLOCKED, **N1 is recorded as UNCONTROLLED** rather than silently standing alone |
| **P-2** | M3 becomes **conditional** on `clone3_unavailable`, and preflight probes `clone3` by calling it with a NULL argument and size 0 — a supporting kernel rejects the arguments **without forking**, an unsupporting one returns `ENOSYS`, a seccomp policy returns `EPERM`/`EACCES`. Availability is established with no child created and nothing resembling a trial |
| **P-3** | The checker now permits **only a conditional case** to absorb a block, and only with the cause its own manifest entry names. A mandatory or recorded case recorded as BLOCKED is INVALID |
| **P-4** | The spike gains `--exec-fd-o-path` and a `DescriptorModeUnsuitable` refusal, so X6 is posable |
| **P-5** | X4 gets `unloadable_in_cohort.elf` — a header that **passes** the cohort rule and still cannot load (`e_phnum = 0`) — so it reaches `execveat` and can produce `ENOEXEC` |
| **P-6** | The spike implements `capture_prefix` retention and the `truncated` flag, both in memory and both reported **outside** the receipt |
| **P-7** | The O-series runs with `--no-report`; the payload itself is the exec evidence there, because only the pinned helper can produce the frozen recipe |
| **P-8** | A record lacking `launch_returned` is **INVALID**, forcing the harness to record a hang rather than omit it |
| **P-9** | `CHILD_TEST_INJECTION_SYSCALLS` and `CHILD_INJECTION_MODES` separate declared fault injection from the mechanism sequence; M1/M2/M4 trace the production configuration |
| **P-10** | `helper_report` gains a guarded fixed-length marker region, verified present in the built image |
| **P-11** | `built_binary_sha256` is removed from the source freeze; binary evidence lives in append-only [`BUILD-EVIDENCE.md`](../experiments/launch-exec-01/BUILD-EVIDENCE.md), which the manifest deliberately does **not** hash |
| **P-13** | `move_above_2` no longer re-adds `CLOEXEC` to a deliberately non-`CLOEXEC` exec descriptor |

**P-12** and **P-14** are **not** corrected here and are carried forward as explicit D-7
obligations: the observation-to-token mapping and the publication sanitiser both live in the
case-posing driver, which this review is forbidden to implement. They are recorded so the
omission cannot later read as coverage.

Two further findings came from the compile job itself and are corrected:

- **`strace` IS installed** on the recommended runner (`/usr/bin/strace`). The definition
  asserted the opposite, inferred from the runner-image package manifest. Direct observation beat
  the inference and the claim is corrected rather than carried forward.
- **The `no_noexec_mount` derivation tested existence, not usability.** `/dev`, `/proc` and `/sys`
  are noexec on the runner but none can host a fixture, so X8 would have been left unblocked and
  would have FAILed for an environment reason. It now tests writability.

## 7. Linux compile evidence

The frozen C sources had never been compiled on Linux. A bounded **compile-only** workflow was
added, restricted to this branch, which compiles and inspects and **never executes a produced
binary, never invokes the case runner in trial mode and never passes the D-7 flag**. Full record:
[`BUILD-EVIDENCE.md`](../experiments/launch-exec-01/BUILD-EVIDENCE.md).

Ubuntu 24.04.4, kernel `6.17.0-1022-azure` x86_64, `cc 13.3.0`, glibc 2.39, euid 1001. The source
freeze was verified on the runner **before** anything was built. **All six targets linked with
zero warnings and zero errors at `-Wall -Wextra`.** Inspection as data only: the five static
targets carry **no `PT_INTERP`**, `helper_dynamic` requests
`/lib64/ld-linux-x86-64.so.2` — which is what E7 requires — and the E6 marker guard is present in
the built `helper_report`. No dynamic build was substituted for a static one, no package was
installed and no `sudo` was used.

The job's **one** failure was a pre-existing repository fixture test that resolves a blob at a
named commit and errored under `actions/checkout`'s default shallow clone. Every one of the 85
tests belonging to this experiment passed. Per the correction boundary that is a **PRETRIAL
finding, not a mechanism verdict** — no preregistered case ran, so no experiment verdict of any
kind exists. The workflow now uses `fetch-depth: 0`.

## 8. Final classification

**READY_TO_AUTHORIZE_FROZEN_TRIAL.**

- Freeze hashes verified exactly, independently, from the git object store.
- No unresolved BLOCKER or IMPORTANT: the six BLOCKERs and the correctable IMPORTANTs are fixed,
  and the two that cannot be fixed without implementing the driver are recorded as D-7
  obligations rather than quietly dropped.
- Manifest and checker algebra independently verified, including a full class × status matrix and
  every single-case perturbation across the membership.
- C sources independently audited instruction by instruction, and the hand-rolled SHA-256
  verified by transliteration against `hashlib`.
- The exact frozen sources compile and link on the target Linux, and their static/dynamic
  properties were established **without executing them**.
- The `no_new_privs` control is posable honestly, and where it is not, N2 blocks and N1 is
  recorded as uncontrolled.
- `clone3` availability is probed rather than assumed, without creating a child.
- The output and lifecycle state machine closes, and a launcher non-return is a rejection that
  can no longer hide as a missing record.
- Binary-hash evidence is immutable and reviewable, and separate from the source freeze.
- The runner still cannot pose a case: both safety barriers are intact and the driver is absent.
- **Zero experiment cases were run.**

**What this does not do.** It does not grant D-7, accept ADR-0024, or authorise
`crates/helm-launch`. It recommends that the frozen trial may now be authorised, and the
remaining work before a first trial is the case-posing driver — including the frozen
observation-to-token mapping of **P-12** and the publication sanitiser of **P-14**, both of which
must be written and frozen *before* results can be known.

## 9. What this review did not do

No preregistered case was posed. No binary was executed — the SHA-256 audit was a
transliteration, and every checker attack used fabricated records. The case-posing driver was not
implemented and both runner safety barriers were left intact. `crates/helm-launch` was not
created. No Wine, no 7-Zip, no A0, no `sudo`, no package installation. The frozen commit
`66bf4b6` was not amended, rebased or squashed.

---

## 10. Post-review note: the case driver was implemented (still NOT_RUN)

**Appended after the review above, which is preserved unchanged.** Nothing in sections 1–9 is
edited, withdrawn or restated: the first-pass **NEEDS_PRETRIAL_FIXES**, the six original BLOCKERs,
their correction history, and the **READY_TO_AUTHORIZE_FROZEN_TRIAL** classification recorded at
`5729458` all stand exactly as written.

**What that classification was about.** It assessed the **mechanism and its source freeze** —
hashes, manifest arithmetic, checker algebra, the C sources read instruction by instruction, and a
clean Linux compile. It was explicit that "the runner still cannot pose a case: both safety
barriers are intact and the driver is absent", and it named the case-posing driver, the **P-12**
observation-to-token mapping and the **P-14** sanitiser as the remaining work. It was never a
readiness statement about a driver that did not exist.

**What this task added.** The driver (`driver.py`), the P-12 mapping (`observations.py`) and the
P-14 sanitiser (`evidence.py`), plus 116 new non-executing tests. **No case was posed, no helper
or spike was executed, no generated ELF was run, and the D-7 flag was never passed.** The default
invocation still refuses at exit 3. No C source byte changed, so the compile evidence in
[`BUILD-EVIDENCE.md`](../experiments/launch-exec-01/BUILD-EVIDENCE.md) remains exactly valid — the
six C digests are unchanged from Build 1 and Build 2.

### 10.1 A new BLOCKER, found by implementing rather than by reading

**D-1 — the helper report never leaves the launcher, and 40 cases depend on it.**

`launcher_spike.c` allocates `out_prefix`/`err_prefix` with `malloc`, fills them under the
`capture_prefix` bound, and `free()`s them at the end of `main`. **They are never written
anywhere.** The receipt carries `bytes_drained`, `drained_sha256`, `completeness` and the
retained-prefix *counts* — never the bytes.

So `helper_report.c` faithfully produces its report on descriptor 1, the launcher faithfully
drains it, and **no channel carries it to the harness**. Every frozen expectation written against
the report is therefore unobservable as frozen:

| Token | Cases |
|---|---|
| `argv_exact` | A1, A2, A3, A4, A6 |
| `environ_empty` | V1 |
| `fds_exactly_012` | F1, F2, F3, F4, F6, F7 |
| `signals_reset` | F5 |
| `no_new_privs_1` / `no_new_privs_0` | N1, N2 |
| `interpreter_ran_with_devfd` | X2c |
| `privilege_transition_suppressed` | N3 |
| exec evidence for a lifecycle token | E1–E4, E5b, E6, E6b, E6c, E7, R1–R4, S1, S2, S3, S5, S7, T1–T4, T6 |

**This is the S1/S5 discrimination, lost.** The previous review's own summary records that "clean
EOF alone never means exec" and that S5 "remains discriminating". It does — but only to an observer
who can see whether a report arrived, and no such observer exists. Without the channel, S1 and S5
are indistinguishable at every interface the harness can reach.

Three further cases are unposable for separate, narrower reasons, each a mechanism arm that was
never built:

- **M2** — needs a launcher with ≥3 extra live threads. `launcher_spike.c` has no threading mode;
  `pthread` occurs only in its comments.
- **M3** — needs the pidfd acquisition path to be *observable*. The receipt names none, so an
  atomic acquisition can only be asserted, which is exactly what M3's own note says it exists to
  avoid.
- **M5** — needs the **rejected** `fork` + `pidfd_open` arm to record an outcome. The spike
  implements no such arm; it appears only in comments.

### 10.2 Scale, computed statically

`driver.unposable_cases()` walks the plan table as data and runs nothing:

| | Count |
|---|---|
| Frozen membership | **72** |
| Driver plans | **72** — 0 missing, 0 duplicate, 0 unknown |
| **Posable against the frozen mechanism** | **29** (20 mandatory, 5 conditional, 4 recorded) |
| **Unposable** | **43** (**34 mandatory**, 6 conditional, 3 recorded) |
| Unposable for want of the report channel alone | **40** |

With 34 mandatory cases unposable, the aggregate is **`MECHANISM_INCONCLUSIVE` before the first
case is posed**. A test asserts exactly this against the checker, and asserts that supplying the
report channel alone would reduce the unposable set to `{M2, M3, M5}`.

**The runner therefore refuses to start.** `preflight_gates()` halts at `HALT_PREFLIGHT` (exit 6)
while any mandatory case lacks an evidence channel. That is deliberate and load-bearing: the first
valid trial is an **immutability boundary**, and consuming it on a run that is known in advance to
be inconclusive would close the trial and force a new preregistration for nothing.

### 10.3 Why this was not corrected here

Correcting D-1 means changing `launcher_spike.c` — the mechanism under test — to emit bytes it
currently discards, or changing `helper_report.c` to write its report somewhere else. Either is a
change to a frozen artefact of the experiment, not harness glue, and the owner instruction is
explicit that a concrete implementation mismatch proving the frozen contract cannot be implemented
is to be **reported, not resolved by changing semantics**. The two candidate corrections are also
not equivalent — one alters the launcher, the other alters what F1/F4 observe — so choosing
between them is an owner decision.

**P-12 and P-14 are discharged.** Both are implemented, frozen before any result can be known, and
tested: every frozen outcome token has a demonstrated derivation, no rule can produce a token from
absent evidence, and no environment value is ever reproduced.

## 11. Classification of the new freeze

**`DRIVER_IMPLEMENTATION_NEEDS_OWNER_DECISION`.** Not READY, and deliberately not.

- The driver, the P-12 mapping and the P-14 sanitiser are complete, tested and frozen.
- The frozen mechanism cannot supply the evidence 43 of its own 72 expectations are written
  against, 34 of them mandatory.
- The correction is an owner decision about the mechanism under test, not a driver detail.

**The new descendant freeze requires a bounded independent pre-trial re-review before D-7.** This
note recommends nothing about authorisation; D-7 is not granted, ADR-0024 is still Proposed, and
**LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**
