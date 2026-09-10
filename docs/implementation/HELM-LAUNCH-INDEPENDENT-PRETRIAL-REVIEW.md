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
> of that freeze was
> [`DRIVER_IMPLEMENTATION_NEEDS_OWNER_DECISION`](#11-classification-of-the-new-freeze).

> **That BLOCKER is now corrected. Its identifier is `PRE-D7-B1`, not `D-1`** — `D-1` collides
> with the owner decision `arm_i_scoped_unsafe_backend`, which is unchanged and unrelated. The
> owner chose the launcher-side observation channel; see
> [section 12](#12-pre-d7-b1-corrected-launcher-side-observation-channel). **71 of 72 cases are
> now posable.** The one that is not is **M3**, which section 12.4 returns as an owner question
> rather than answering with a self-assertion. That freeze's classification was
> [`DRIVER_CORRECTION_NEEDS_OWNER_DECISION`](#13-status-after-the-pre-d7-b1-correction).

> **The owner answered that question: amendment `M3-T`, made before the first valid trial.** M3
> becomes a **traced** case, so its acquisition is established from the external syscall record
> instead of a launcher self-assertion, and the traced set is amended prospectively from seven
> cases to eight. **All 72 cases are now posable.** See
> [section 14](#14-m3-t-m3-amended-to-a-traced-case-still-not_run) — including the one residual
> uncertainty (14.3) and a defect it surfaced that would have made *every* traced case INVALID
> (14.5). Current classification:
> [`M3_FREEZE_READY_FOR_BOUNDED_REVIEW`](#15-status-after-m3-t) — which is a request for review,
> not an authorisation.

> **That bounded review has now been done:
> [section 16](#16-bounded-independent-review-of-the-m3-t-descendant-freeze).** It resolves 14.3's
> `strace` uncertainty in the affirmative from upstream source, and confirms the freeze, the
> partition, the traced set and the traced-record fix. It also finds **one BLOCKER** — `strace -f`
> splits `clone3` across two lines, which currently makes M3 report `clone3_failed`, a **FAIL**
> that would render the whole run `MECHANISM_REJECTED` from a rendering artefact — and three
> IMPORTANT findings. Result:
> [`M3_T_NEEDS_PRETRIAL_FIXES`](#165-classification). **Nothing was corrected in that review**, and
> D-7 remains not authorised.

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

> **Identifier correction, appended.** The finding below was labelled **D-1** when this section was
> written. That collides with **owner decision D-1** (`arm_i_scoped_unsafe_backend`), which it has
> nothing to do with. The finding's identifier is now **PRE-D7-B1**, in the separate
> `PRE-D7-B<n>` review-finding namespace. The paragraph is left as written so the prior report is
> not falsified; **owner D-1 is unchanged and is not redefined.** The correction is recorded in
> [section 12](#12-pre-d7-b1-corrected-launcher-side-observation-channel).

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

---

## 12. PRE-D7-B1 corrected: launcher-side observation channel

**Appended after sections 1–11, which are preserved unchanged.** The owner chose the
**launcher-side** solution and ruled out the alternative explicitly: no new inherited helper
descriptor, and no change to `helper_report.c`'s reporting channel. **LAUNCH-EXEC-01 is still
NOT_RUN, D-7 is still not granted, and the valid trial count is still ZERO.**

### 12.1 What changed in the mechanism

`launcher_spike.c` already drained each stream, hashed every drained byte, and retained a bounded
prefix under `max_capture_bytes`. It then `free()`d that prefix without emitting it. The prefix is
now written out, base64-encoded, inside the stream block that already described it:

```
"stdout": { "bytes_drained": N, "drained_sha256": "...", "completeness": "...",
            "capture_prefix_length": K, "capture_prefix_truncated": bool,
            "capture_prefix_base64": "..." }
```

**What did not change.** `bytes_drained` and `drained_sha256` are still computed over *every*
drained byte, not over the prefix. `completeness` is untouched. The drain loop, the poll loop, the
bound and the truncation flag are untouched. **No descriptor was added to the child**: the report
still arrives on descriptor 1, and the executed image still sees exactly `{0, 1, 2}`. A test
asserts that no plan's invocation carries a report-descriptor flag of any kind.

Base64 was chosen because the retained bytes are arbitrary — the O-series recipe is
`byte[i] = (i*251 + tag) mod 256`, which is binary by construction — and a JSON string escape
would be neither length-preserving nor byte-safe. A round-trip test encodes `bytes(range(256))*4`.

### 12.2 The raw capture is internal, never public

The retained bytes are whatever the executed image wrote. On a hosted runner that can include
environment values, absolute paths and credentials, and the inherited environment carries
`ACTIONS_RUNTIME_TOKEN`. **A secret encoded in base64 is still a secret.**

So the boundary is drawn by **key, not by content**: `evidence.INTERNAL_ONLY_KEYS` is withheld
wholesale, at any depth, before any content is examined. Scanning an encoded blob for secrets is a
game the scanner loses, and a redaction rule that depends on recognising the encoding stops being
correct the moment the encoding changes. `receipt_view()` drops the field entirely; the sanitiser
replaces it with `<WITHHELD:internal-observation-input>`.

The driver may still decode it, parse the report, derive tokens and sanitise the result — that is
the whole point. What it may not do is republish it. Hostile synthetic tests push a GitHub PAT, an
AWS key id, a JWT, another account's SSH key path and a Windows user path through the encoded
capture and assert that none of them, nor the encoded blob itself, survives into the receipt view,
the sanitised record, or the full evidence document.

### 12.3 Report states, so incomplete evidence stays non-success

A decoded prefix is not automatically a report. Six states are now distinguished, and only the
first is evidence:

| State | Meaning |
|---|---|
| `complete` | the sentinel arrived and the JSON after it parsed, with a marker |
| `truncated` | the prefix stopped at the bound; the report was cut off |
| `malformed` | the sentinel arrived and what followed did not parse |
| `absent` | no sentinel, **and** the stream that would have carried it was complete |
| `stream_incomplete` | a writer was retained, or bytes were drained past the prefix |

`absent` is decisive; `stream_incomplete` is not, and the difference is load-bearing. **S2, S5 and
S7 rest on the ABSENCE of a report**, and an absence only means something when the stream that
would have carried it is known to be complete. The posed-check for those three accepts `absent`
only. Every report-derived rule — `argv_exact`, `environ_empty`, `fds_exactly_012`,
`signals_reset`, `no_new_privs_*`, `interpreter_ran_with_devfd`,
`privilege_transition_suppressed` — goes through one guard that admits `complete` alone.

### 12.4 M2 and M5 implemented; M3 returned as an owner question

**M2 — implemented.** The frozen contract fixes the arm exactly: ">=3 extra live threads, one with
an allocation in flight and one with a registered `pthread_atfork` handler". `--extra-threads N`
is TEST/CONTROL ONLY and starts them immediately before the clone; one thread allocates
continuously, one registers the handler. The control arm is the same plan with no threads, and the
two child windows must match. `--extra-threads` is deliberately **not** a `CHILD_INJECTION_MODE`:
it changes the parent's shape, not the child's syscall sequence, which is exactly why M2 can carry
it and still trace a production child window.

**M5 — implemented.** `--rejected-acquisition-arm` is TEST/CONTROL ONLY and runs before the
mechanism with its own fork, its own child and its own reap; the mechanism below it is unchanged
whether or not it ran. It reports **raw observations only** — the fd and the two errnos — and
derives no token. `observations.py` maps `ESRCH` to `pidfd_open_esrch`, `ECHILD` after a successful
open to `waitid_echild`, and otherwise `pidfd_open_succeeded`. The gate is enforced in the C
itself: an exit status is recorded only when one was actually observed.

**M3 — NOT implemented. OWNER DECISION REQUIRED.**

Section 4 of the definition fixes the authoritative evidence sources, and item 2 — a syscall record
of the launcher — is permitted **"only for the seven cases declared `traced: true` — E1, E7, F4,
F7, M1, M2, M4"**. M3 is not among them. The helper cannot observe its parent's acquisition.
`/proc/<pid>/fd` can show that a pidfd exists, and `fdinfo` can show which process it refers to,
but neither shows that it was obtained *in the same syscall that created the child* — which is the
whole of the claim. The oracles compute recipe digests only.

A receipt field reading `"pidfd_acquisition": "clone3"` would be the launcher asserting exactly
what M3 exists to evidence rather than assert, and adding a tracer for M3 would both contradict the
frozen restriction above and invent a preregistered dependency that does not exist. **No such field
was added.** M3 is left unposable and returned as an owner question.

### 12.5 Posability after the correction

| | Before | After |
|---|---|---|
| Driver plans | 72 | 72 (0 missing, 0 duplicate, 0 unknown) |
| Posable | 29 | **71** |
| Unposable | 43 (34 mandatory) | **1** — `M3`, conditional |

The preflight gate is now the stricter one the owner instruction requires: the trial halts unless
`unposable_cases() == []`, not merely unless no *mandatory* case is unposable. A conditional case
that cannot be posed is still a hole in the preregistered set, and the first valid trial is an
immutability boundary worth more than a partial answer. A test proves the gate reacts to a channel
being removed rather than to a hard-coded list — which surfaced a latent defect while it was being
written, where the supplied-channel set had been captured in a default argument and could not be
overridden at all.

**As things stand the runner halts at `HALT_PREFLIGHT` (exit 6) because of M3 alone.**

## 13. Status after the PRE-D7-B1 correction

**`DRIVER_CORRECTION_NEEDS_OWNER_DECISION`.**

PRE-D7-B1 is corrected, P-12 and P-14 are re-verified and extended, M2 and M5 have the arms their
frozen contracts name, and 71 of 72 cases are posable. One question remains and it is not the
driver's to answer: **how M3's atomic acquisition may be evidenced within the frozen contract, or
whether that contract should be amended.**

This is **not** a readiness statement. The corrected descendant freeze still requires the bounded
independent pre-trial re-review already mandated, and that review must examine the mechanism delta
— a change to `launcher_spike.c`, which is the artefact under test. **D-7 remains not authorised
and LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

---

## 14. M3-T: M3 amended to a traced case (still NOT_RUN)

**Appended after sections 1–13, which are preserved unchanged.** Section 12.4 returned M3 as an
owner question. The owner answered it by **amending the preregistration before the first valid
trial**, which is permitted precisely because no trial has begun. Earlier freezes are preserved and
**none of them contained this rule**; nothing here is backdated.

### 14.1 The amendment

**M3 becomes a traced case.** The traced set is amended prospectively:

| | Traced set |
|---|---|
| Before | `E1, E7, F4, F7, M1, M2, M4` — seven |
| After | `E1, E7, F4, F7, M1, M2, `**`M3`**`, M4` — eight |

M3 stays **conditional** on `clone3_unavailable`. The 72 / 54 / 11 / 7 partition is untouched. The
historical traced-set reading in [section 1](#1-independently-reconstructed-state) is left exactly as
recorded — it states what was true at `a8a7f19` and remains accurate for that commit.

### 14.2 The bounded claim, and what it is not

Frozen in the manifest as `M3_BOUNDED_CLAIM`:

> For the direct child in this candidate execution, the pidfd used by the launcher was returned
> through the `CLONE_PIDFD` facility of the **same** `clone3` syscall that created that direct
> child, rather than being acquired later by `pidfd_open()` from a numeric PID.

`M3_CLAIM_EXCLUSIONS` records, in the manifest rather than only in prose, that M3 does **not**
establish that Linux pidfds are universally race-free, that the kernel implementation has been
proved atomic, any general pidfd claim beyond this execution, or anything about timing-sensitive
normal-launch behaviour — M3 runs under a tracer, and section 8 of the amendment forbids
generalising a traced run into evidence about untraced timing.

### 14.3 External evidence only

**No `pidfd_acquisition` receipt field was added, and none is read.** A test asserts that a receipt
carrying `"pidfd_acquisition": "clone3"` yields **no token**, and a second test greps the rule's own
source to confirm it never consults such a field. A launcher describing its own behaviour is exactly
the self-assertion M3 exists to avoid.

The rule consumes the normalised syscall record and requires **all** of `M3_EVIDENCE_FACTS`:

| Fact | Established by |
|---|---|
| **A** | a `clone3(` line in the record; more than one is ambiguous and refused |
| **B** | `CLONE_PIDFD` among the decoded `flags=` |
| **C** | `pidfd=0x…` present in `clone_args`, i.e. an output location was supplied |
| **D** | the call's return value is a pid > 0 |
| **E** | see below — two admissible forms |
| **F** | no `pidfd_open()` whose target pid is the direct child |
| **G** | `waitid(P_PIDFD, N, …)` reaps **that** child through `N`, so `N` is its handle by construction |

**Fact E has two admissible forms**, and which one was used is recorded per run as `evidence_form`
rather than left to inference:

- **`direct`** — the tracer printed the kernel's write-back, `=> {pidfd=[N]}`, and `N` must equal the
  descriptor the lifecycle used. A mismatch is INVALID.
- **`closure`** — the tracer did not print it. The record can still close the question, but only if
  it contains **no `pidfd_open` at all**: with `CLONE_PIDFD` requested, an output location supplied,
  a descriptor used to reap this child, and no `pidfd_open` anywhere, there is no other route by
  which that descriptor could exist. If any `pidfd_open` is present in this form, the origin is not
  closed and the result is INVALID.

**Why two forms.** `strace`'s rendering of `clone_args` output fields could not be observed
first-hand: the local Ubuntu distribution has **no `strace`**, and §13 forbids tracing a HELM binary
to find out. Rather than fabricate a format from memory and then test the parser against the same
fabrication — which would be self-confirming — the parser accepts either rendering and refuses
anything ambiguous. Preflight now also records `strace_version`, so a trial's evidence names the
tracer that produced it. **This is the one residual uncertainty in M3-T and the bounded reviewer
should weigh it**; the failure mode is INVALID, never a false PASS.

**`pidfd_open` on the direct child is rejected outright** — `pidfd_acquired_by_pidfd_open` — which is
the distinction the amendment demands between `clone3(CLONE_PIDFD)` and `clone3`/`fork` → numeric
PID → `pidfd_open(PID)`. A `pidfd_open` aimed at an unrelated process does **not** reject, which is
why the target pid is compared rather than the mere presence of the call.

### 14.4 Host condition, and one consequence stated plainly

M3's only frozen block cause remains `clone3_unavailable`. The escape hatch was **not** broadened: if
`clone3` is supported and the trace fails to establish A–G, that is INVALID, not a legitimate
conditional block. A test asserts that a `no_tracer` environment does **not** produce a block cause
for M3.

**The consequence, which the amendment implies and which is not hidden here:** a host with no
permitted tracer now makes M3 **INVALID** rather than BLOCKED, because `no_tracer` is not one of M3's
frozen causes — and an INVALID conditional case makes the aggregate `MECHANISM_INCONCLUSIVE`. The
recommended runner has `strace` at `/usr/bin/strace` with `ptrace_scope 1`, directly observed and
recorded in the definition, so M3 is posable there. Adding `no_tracer` as a second cause would have
broadened the escape hatch, which §6 of the amendment forbids.

### 14.5 A defect found while wiring this

**Every traced case would have scored INVALID at trial time.** `checker.score_case` fails a traced
case whose record shows no syscall record, and `driver.evaluate()` never put one there — so E1, E7,
F4, F7, M1, M2 and M4 were all affected, not just M3, and the aggregate would have been
`MECHANISM_INCONCLUSIVE`. This predates M3-T; it surfaced only because M3 becoming traced made the
traced path load-bearing enough to test end to end. Fixed, and a test now scores all eight traced
cases through the checker with proper evidence.

### 14.6 Privacy

Raw pids and descriptor numbers are experiment-local identifiers and must not become part of a
receipt's identity. `normalise_acquisition()` publishes `DIRECT_CHILD` and `DIRECT_CHILD_PIDFD` plus
booleans, decoded flag names and counts — never a raw number. The raw facts and the raw tracer text
are withheld **by key** (`acquisition`, `lifecycle_uses`, `pidfd_open_calls`, `raw_trace`,
`trace_text`, `strace_output`), the same wholesale rule PRE-D7-B1 established for the capture prefix;
only the trace's SHA-256 is publishable. Tests assert that a record containing a host path inside
tracer text cannot reach serialised evidence.

### 14.7 Posability

| | Before M3-T | After |
|---|---|---|
| Driver plans | 72 | 72 (0 missing, 0 duplicate, 0 unknown) |
| Posable | 71 | **72** |
| Unposable | 1 (`M3`) | **0** |

The D-7 preflight gate is unchanged and still halts unless `unposable_cases() == []`; it simply no
longer fires. The retired `CH_ACQUISITION` channel was **removed** rather than left defined, so a
self-asserted acquisition channel does not exist to be used.

## 15. Status after M3-T

**`M3_FREEZE_READY_FOR_BOUNDED_REVIEW`.**

All 72 cases are posable, the driver is complete, P-12 and P-14 hold, and no C source changed — the
clean Build 5 compile evidence still covers the mechanism exactly.

This is **not** an authorisation and **not** a readiness statement for D-7. The bounded independent
pre-trial re-review is still required, and it now has three specific things to weigh: the M3-T
amendment itself, the `strace` clone3-rendering uncertainty in 14.3, and the traced-record defect in
14.5. **D-7 remains not authorised and LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of
ZERO.**

---

## 16. Bounded independent review of the M3-T descendant freeze

**Scope:** the delta `4da94b5..5d0b801` only. Sections 1–15 are preserved unchanged and nothing
earlier is reopened. **This section is the first-pass record and is committed before any
correction.** No experiment source or manifest was modified by this review. No case was posed, no
helper or spike was executed, no generated ELF was traced, and the D-7 flag was never passed.
**LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

### 16.1 Independently reconstructed state

Every value recomputed, not read from the author's summary.

| Claim | Method | Result |
|---|---|---|
| working tree | `git status --short` | clean |
| HEAD / origin | `git rev-parse` | `5d0b801` local; `origin` at `4da94b5`, so the candidate is local-only |
| delta scope | `git diff --stat` | 9 files, Python and documentation only |
| C or workflow change | `git diff --name-only -- '*.c' '*.yml' '*.h'` | **none** |
| historical commits | tree hashes of ten prior commits | all intact, none amended |
| attribution | `%(trailers:key=Co-Authored-By)` on the delta commit | **empty** — AGENTS.md policy followed |
| source hashes | independent SHA-256 | **16/16 exact** |
| definition hashes | independent SHA-256 | **3/3 exact** |
| manifest self-reference | key inspection | does not hash itself |
| partition | manifest + `summary()` | **72 / 54 / 11 / 7** |
| traced set | manifest + derived list | **E1, E7, F4, F7, M1, M2, M3, M4** — eight |
| driver | `completeness()` | **72 handlers**, 0 missing, 0 duplicate, 0 unknown |
| posability | `unposable_cases()` | **0 unposable** |
| status / D-7 | manifest | `NOT_RUN` / `false` |

### 16.2 What was verified sound

- **The bounded claim holds its boundary.** `M3_BOUNDED_CLAIM` and `M3_CLAIM_EXCLUSIONS` are in the
  manifest, not only in prose. No document reviewed claims universal pidfd race-freedom, kernel
  correctness, general atomicity, or timing semantics of the untraced mechanism.
- **No self-assertion.** `launcher_spike.c` contains no `pidfd_acquisition` field, and the rule's
  source does not reference one. A fabricated receipt carrying `"pidfd_acquisition": "clone3"`
  yields no token, confirmed directly.
- **The traced-record fix is real and does not over-reach.** All **eight** traced cases now carry a
  syscall record and score PASS through `checker.score_case` with proper fabricated evidence; with
  the trace absent, the record carries `trace: None` and scores **INVALID**. The fix does not
  manufacture a trace and does not bypass the checker.
- **`pidfd_open` discrimination is correct.** `pidfd_open(DIRECT_CHILD_PID)` yields
  `pidfd_acquired_by_pidfd_open`; `pidfd_open(UNRELATED_PID)` does not create a false failure.
- **Privacy holds.** Raw pids, descriptor numbers, pointer values, usernames, host paths and raw
  tracer text are all withheld; a fabricated record containing `31337`, `4242`, `0x7ffd1234`,
  `runner` and a `.netrc` path leaked none of them into serialised evidence. Normalised output
  carries only booleans, decoded flag names, counts and the symbolic `DIRECT_CHILD` /
  `DIRECT_CHILD_PIDFD`.
- **The BLOCKED escape hatch was not widened.** M3's only frozen cause remains
  `clone3_unavailable`; a `no_tracer` environment produces no block cause for M3, and
  `harness._derive_blocks` never maps a missing tracer to `clone3_unavailable`.
- **Posability wording is honest.** The manifest states the freeze "is NOT a readiness statement and
  grants nothing", and 72/72 is nowhere presented as acceptance or as a likely PASS.
- **The strace format is grounded upstream, not fabricated.** `strace/src/clone.c` prints `pidfd`
  on entry via `PRINT_FIELD_ADDR64` (so fact C's `pidfd=0x…` is real) and on exit via
  `printnum_fd()` behind `tprint_value_changed_struct_begin()` (so fact E's `=> {pidfd=[N]}` is
  real). The author's residual uncertainty in 14.3 is **resolved in the affirmative**: the direct
  form is available, and the closure form is therefore **not required** for M3 to be sound.

### 16.3 Findings

#### BLOCKER

**R-1 — `strace -f` splits `clone3` across two lines, and M3 then reports `clone3_failed`, which is
a FAIL.**

Under `-f`, a clone-family syscall is routinely rendered split, because the child begins running
before the parent's syscall returns:

```
111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, exit_signal=SIGCHLD} <unfinished ...>
[pid   222] execveat(3, "", NULL, NULL, AT_EMPTY_PATH) = 0
111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222
```

`observations.parse_pidfd_acquisition` matches only `_CLONE3_LINE` (`clone3\(\{…\}`), which appears
on the **unfinished** line. That line carries neither the return value nor the `=> {pidfd=[N]}`
block, both of which are on the **resumed** line — and `<... clone3 resumed>` does not match the
pattern at all. Verified against the frozen parser: `clone3_return=None`,
`clone3_succeeded=False`, `pidfd_from_clone3=None`.

**Consequence:** the rule returns `clone3_failed`. M3 is conditional and was posed, so
`checker.verdict` rule 1 fires and the aggregate is **`MECHANISM_REJECTED`** — a tracer *rendering
artefact* reported as a falsified mechanism claim. This is precisely the misclassification class of
P-1 and P-2, and it is the strongest possible wrong answer.

*Affected:* `observations.py`, `_CLONE3_LINE` / `_CLONE3_RESULT` and their single-line assumption.
*Blast radius:* M3 only. `parse_strace_child_window` already handles the resumed form and returns
the correct child window under the split rendering, so the other seven traced cases are unaffected.
*Disposition:* **must be fixed before D-7.** The parser must join the unfinished and resumed
fragments of the same `clone3` before extracting facts D and E. Until then the split rendering must
never yield `clone3_failed`; an unjoined record is INVALID at worst, never FAIL.

#### IMPORTANT

**R-2 — the closure form is over-permissive: six alternate acquisition routes each yield the M3
success token.**

Fabricated records exercising each route were classified `pidfd_acquired_atomically`:

| Route | Injected syscall |
|---|---|
| duplication | `dup2(4, 7)` then the lifecycle uses `7` |
| descriptor theft | `pidfd_getfd(9, 3, 0) = 4` |
| procfs handle | `openat(AT_FDCWD, "/proc/222", O_RDONLY\|O_DIRECTORY) = 4` |
| legacy clone | `clone(…, flags=CLONE_PIDFD\|SIGCHLD, parent_tid=[4]) = 333` |
| second child | `fork() = 333` |
| descriptor passing | `recvmsg(…, SCM_RIGHTS, cmsg_data=[4])` |

The closure argument concludes "no other route by which that descriptor could exist". That
conclusion is **true for the frozen source** — `launcher_spike.c` never duplicates or relocates the
pidfd (relocation happens before the clone), never calls `pidfd_getfd`, `recvmsg` or `clone`, opens
no `/proc/<pid>`, and its only `fork()` is inside the M5 arm, which M3's plan does not enable. But
**the trace does not exclude any of them and the parser tests for none of them.**

*Consequence:* the closure form's soundness rests on frozen-source invariants that are not
externally observed — which re-imports exactly the source-trust M3 was created to eliminate. The
definition's wording ("leaving no other route by which that descriptor could exist") therefore
**overclaims** what the evidence establishes.
*Disposition:* since 16.2 establishes that the **direct** form is available on any strace that
decodes `clone_args`, the cleanest repair is to **remove the closure form** and require fact E
directly. If it is retained, the parser must additionally require, *from the trace*, the absence of
every route above.

**R-3 — the `waitid` correlation ignores the rendered `si_pid`.**

Fact G is asserted as "`waitid(P_PIDFD, N, …)` reaps **the direct child** through `N`". The parser
extracts only `N` and never compares the rendered `si_pid` against `clone3`'s return value. A
fabricated record in which `waitid(P_PIDFD, 4, {si_pid=999})` follows `clone3(…) = 222` is
classified `pidfd_acquired_atomically`.

*Consequence:* a descriptor referring to a **different** process satisfies the correlation. strace
renders `si_pid`, so the check is available and costs nothing.
*Disposition:* require `si_pid == clone3_return` before treating the `waitid` as correlating; a
mismatch is INVALID.

**R-4 — the definition's fallback tracer does not exist, and M3-T widened the exposure.**

Section 4 of the definition promises: `strace` if preflight finds it, "otherwise a purpose-built
parent-side `ptrace` tracer if `ptrace_scope <= 1`, otherwise those cases are BLOCKED". No such
ptrace tracer is implemented — `driver.tracer_argv` returns `None` whenever `strace` is absent — and
`harness._derive_blocks` sets `no_tracer` **only** when `ptrace_scope` is *not* in `("0", "1")`.

*Consequence:* on a host with no `strace` but a permissive `ptrace_scope`, no traced case is
BLOCKED, every traced case is `not_posed` → INVALID, and the aggregate is
`MECHANISM_INCONCLUSIVE` with no legitimate BLOCKED path. Pre-existing, but the amendment moved M3
into that set and made it load-bearing for the acquisition claim.
*Disposition:* implement the promised fallback, or make the preflight gate halt when a traced case
has no available tracer, or amend the definition to drop the fallback. An owner-visible choice, but
all three are bounded.

#### MINOR

**R-5 — stale docstring.** `driver.tracer_argv` still reads "Only the seven cases declared
`traced: true` ever reach here". The amendment made it eight. Documentation only; no behavioural
effect.

### 16.4 Two propositions kept separate

Section 6 of the review instruction asks that these not be collapsed, and they are not:

1. **The kernel produced a pidfd.** Established by facts A–E: a single decoded `clone3`, with
   `CLONE_PIDFD` among its flags, an output location supplied, a successful child, and the kernel's
   write-back rendered.
2. **The descriptor the lifecycle used is that pidfd.** Established by fact G plus the equality
   check `written == correlated` in the **direct** form. In the **closure** form this second
   proposition is *not* independently established — it is inferred from the absence of `pidfd_open`
   alone, which R-2 shows is insufficient as trace evidence.

This is the precise reason the direct form is sound and the closure form is not.

### 16.5 Classification

**`M3_T_NEEDS_PRETRIAL_FIXES`.**

One BLOCKER (**R-1**) and three IMPORTANT findings (**R-2**, **R-3**, **R-4**), all bounded and all
repairable within the semantics the owner has already chosen. No new owner decision is required to
fix them, with the partial exception of R-4, where three acceptable repairs exist and the choice
between them is small.

Freeze integrity holds, the partition and traced set are exactly as declared, the traced-record fix
is sound, P-12's refusals hold everywhere except the closure path, P-14 holds, and no C source
changed. **D-7 remains not authorised and the valid trial count remains ZERO.**

---

## 17. Disposition of bounded findings R-1 through R-5

**Appended after section 16, which is preserved unchanged — the findings stand as recorded before
any correction.** This section records what was done about them. **LAUNCH-EXEC-01 remains NOT_RUN,
D-7 is not granted, and the valid trial count is ZERO.**

| ID | Severity | Disposition |
|---|---|---|
| **R-1** | BLOCKER | **FIXED** — tracer fragments rejoined per traced task |
| **R-2** | IMPORTANT | **FIXED by removal of the closure form** |
| **R-3** | IMPORTANT | **FIXED** — `waitid` requires `si_pid == clone3` return |
| **R-4** | IMPORTANT | **FIXED by mandatory `strace` preflight; no fallback** |
| **R-5** | MINOR | **FIXED** — docstring corrected to eight traced cases |

### 17.1 R-1 — fragments are rejoined before anything is interpreted

`join_trace_fragments()` pairs `<unfinished ...>` with `<... call resumed>` **keyed on the task the
tracer attributed the line to**, so fragments from different tasks are never spliced and no return
value is ever inferred from an unfinished half. The joined text is then parsed exactly as a
single-line call would be.

| Condition | Result |
|---|---|
| complete pair | one logical `clone3` observation |
| unfinished with no resume | **INVALID** |
| resume with no unfinished | **INVALID** |
| resume naming a *different* call | **INVALID** (both halves recorded, neither spliced) |
| two unfinished from one task | **INVALID** — ambiguous |
| unreadable task prefix | **INVALID** |
| genuine error return *after* rejoining | `clone3_failed` — a real mechanism FAIL |

Trace integrity is checked **first**, before any `clone3` semantics, so a formatting fragment can
never reach the failure tokens. The split record that previously produced `clone3_failed` now
produces the success token, verified directly.

A latent defect was found and fixed while writing this: an early draft tested the "malformed
prefix" pattern *before* the well-formed one, and `\s+` backtracking let `[pid   222]` satisfy it by
matching one space as the non-digit character — flagging every correctly prefixed line unreadable.
The well-formed prefix is now tried first, and a test pins four prefix spellings.

### 17.2 R-2 — the closure form is gone

`EVIDENCE_CLOSURE` no longer exists. Fact **E** requires the tracer-rendered write-back, which
[section 16.2](#162-what-was-verified-sound) established is available. A test asserts each of the
six routes the closure form accepted — `dup2`, `pidfd_getfd`, `/proc/<pid>`, legacy
`clone(CLONE_PIDFD)`, a second `fork()` child, `SCM_RIGHTS` — now yields **no token**, and another
asserts the attribute is absent rather than merely unused.

### 17.3 R-3 — correlation names the child

`_correlated_pidfd()` now selects only `waitid(P_PIDFD, N)` entries whose rendered `si_pid` equals
the `clone3` return, and requires exactly one such descriptor. Entries with no rendered `si_pid` —
an `ECHILD` retry — are ignored rather than fatal, so a later failed reap cannot erase an earlier
correct one. **An unrelated child's reap is now simply filtered out instead of making the record
ambiguous**, which is strictly better than the behaviour section 16 tested.

### 17.4 R-4 — strace is mandatory, and there is no fallback

Frozen as `STRACE_MIN_VERSION = (5, 4)` and `TRACER_REQUIREMENT`. The preflight gate HALTS when
`strace` is absent, below the floor, unreadable in version, or unable to attach because
`ptrace_scope` is restrictive. The parent-side `ptrace` fallback the definition once promised is
**withdrawn** from the active text; the historical statement that it had been proposed is left
intact.

This is **not** `clone3_unavailable` and never becomes a conditional cause — tests assert both,
including that no halt record mentions `clone3_unavailable` and that a `no_tracer` environment
yields no block cause for M3. A further test asserts the halt precedes any posing in `run_trial`,
and another greps the three modules for `PTRACE_*`, `bpf(`, `perf_event_open`, `libbpf`, `insmod`
and `sudo` **in code rather than comments** — the first draft of that guard failed on a comment
saying sudo is never used.

`no_tracer` remains M1/M2/M4's frozen cause in the manifest but is now unreachable in practice,
because the halt fires first. That is recorded rather than tidied away.

### 17.5 What did not change

The partition is **72 / 54 / 11 / 7**, the traced set is the same **eight**, `status` is `NOT_RUN`,
`d7_execution_authorised` is `false`, and **no C source byte changed** — all six digests are
identical to the source Build 5 compiled clean, so that evidence still applies and no new compile
was requested.

## 18. Status after R-1…R-5

**`M3_FIXES_READY_FOR_FINAL_BOUNDED_REVIEW`.**

All five findings are dispositioned, 312 tests pass, and static posability is 72/72/0. This is
**not** a readiness statement and it is deliberately not self-certified: the corrections were made
by the same hand that wrote the code under review, and **a final bounded independent verification
of exactly these fixes is required before D-7.**

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**
