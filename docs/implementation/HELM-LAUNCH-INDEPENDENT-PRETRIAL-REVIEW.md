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

---

## 19. Final bounded independent verification of R-1 … R-5

**Scope:** the correction delta `378fce2..dc1149f` only. Sections 1–18 are preserved unchanged.
**This section records the verification result before any further correction, and corrects
nothing.** No experiment source, definition or manifest was modified — all remain byte-identical to
`dc1149f`. No case posed, no helper or spike executed, no generated HELM ELF traced, D-7 flag never
passed. **LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

### 19.1 Independently reconstructed state

| Claim | Method | Result |
|---|---|---|
| working tree / branch | `git status --short` | clean, `docs/helm-launch-architecture` |
| candidate | `git rev-parse HEAD` | `dc1149f` ✓ |
| delta scope | `git diff --stat` | 8 files: Python, tests, docs, manifest |
| C / workflow change | `git diff --name-only -- '*.c' '*.h' '*.yml'` | **none** |
| source hashes | independent SHA-256 | **16/16 exact** |
| definition hashes | independent SHA-256 | **3/3 exact** |
| partition | **AST parse, no import** | **72 / 54 / 11 / 7** |
| traced set | **AST parse, no import** | **E1, E7, F4, F7, M1, M2, M3, M4** — eight |
| driver | `completeness()` | **72 handlers**, 0 missing / duplicate / unknown |
| posability | `unposable_cases()` | **72 posable, 0 unposable** |
| status / D-7 | manifest | `NOT_RUN` / `false` |
| C vs clean Build 5 | digest comparison against `002e1e4` | **all six identical** |
| suite | `unittest discover` | 312 tests, all pass |

### 19.2 The strace contract is now grounded — and the 5.4 floor holds

The previous review resolved the *existence* of the direct rendering. This one checked whether the
**declared floor** actually provides every rendering the frozen parser needs, by inspecting the
**`v5.4` tag itself** rather than release notes:

| Required rendering | Evidence at `v5.4` |
|---|---|
| `clone3` decoded, flags symbolic | `PRINT_FIELD_FLAGS(..., clone_flags, "CLONE_???")` — `CLONE_PIDFD` renders by name |
| `clone_args.pidfd` on **entry** (fact C) | `if (arg.flags & CLONE_PIDFD) PRINT_FIELD_ADDR64(", ", arg, pidfd)` |
| pidfd write-back on **exit** (fact E) | `if (arg.flags & CLONE_PIDFD) { tprintf("%spidfd=", pfx); printnum_fd(tcp, arg.pidfd); }` under `" => {"` |
| `waitid` idtype as `P_PIDFD` (fact G) | `printxval(waitid_types, tcp->u_arg[0], "P_???")`, and `xlat/waitid_types.in` contains `P_PIDFD 3` |
| `si_pid` in siginfo (fact G) | `printsiginfo_at(tcp, tcp->u_arg[2])` |
| unfinished / resumed | long-standing strace behaviour, unchanged |

**`STRACE_MIN_VERSION = (5, 4)` is defensible.** An initial reading of the mailing-list archive
suggested `P_PIDFD` decoding post-dated 5.7; inspecting the tag disproved that — the xlat entry and
the exit-time write-back are both present at 5.4, which is consistent with `P_PIDFD` having landed
in Linux 5.4 (2019-11) and strace 5.4 shipping the same month. One caveat recorded for
completeness: the automated read of `v5.4/clone.c` gave a narrative sentence contradicting its own
quoted code about the entry branch; the **quoted code** shows the entry printing, and the
master-branch read agrees, so the conclusion rests on the code rather than the narration.

### 19.3 Findings

#### BLOCKER

**V-1 — a `clone3` record with no parseable return value still becomes `clone3_failed`, a FAIL, and
therefore `MECHANISM_REJECTED`.**

R-1 closed the *split* rendering. It did not close the *truncated* one. A record whose final line is
cut — the tracer killed, the file read while still being written, a full disk — pairs its fragments
**successfully**, so every R-1 integrity counter reads zero:

```
111 clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, exit_signal=SIGCHLD} <unfinished ...>
[pid 222] execveat(3, "", NULL, NULL, AT_EMPTY_PATH) = 0
111 <... clone3 resumed> => {pidfd=[4]}, 88          <-- record ends here
```

Observed against the frozen parser: `fragments_unmatched=0, fragments_orphaned=0,
fragments_ambiguous=0, lines_malformed=0`, `clone3_return=None`, `clone3_succeeded=False` →
token **`clone3_failed`** → **FAIL** → aggregate **`MECHANISM_REJECTED`**. A single-line
(unsplit) `clone3` truncated the same way reaches the identical outcome.

*Root cause:* `clone3_succeeded` is derived as `clone3_return > 0`, which conflates **"the syscall
returned an error"** with **"no return value was observed"**. The first is a mechanism result; the
second is missing evidence.

*Why the author's test did not catch it:* `test_truncated_final_line_is_invalid` removes the
trailing newline, which **glues the following `poll(...)` line onto the resumed fragment**. The
joined text then ends `…88111   poll([{fd=4…}], 1, 5000) = 1`, and the return-value regex picks up
`= 1` — so the test observes `clone3_return=1, clone3_succeeded=True` and passes for a reason
unrelated to truncation. That is a second, quieter defect in the same place: **a return value was
silently adopted from an adjacent line**, yielding a wrong direct-child pid.

*Consequence:* the exact failure class R-1 was raised to eliminate — a tracer/IO artefact producing
the strongest possible wrong verdict — remains reachable.
*Disposition:* **must be fixed before D-7.** Treat an absent return value as INVALID, distinct from
an observed error return; and bound the joined text so a fragment cannot absorb the next line.

#### IMPORTANT

**V-2 — a self-contradictory record is accepted: the same pidfd reaping two different children
still PASSes.**

With `waitid(P_PIDFD, 4, {si_pid=222})` **and** `waitid(P_PIDFD, 4, {si_pid=333})` after
`clone3(…) = 222`, the si_pid filter keeps only the first, leaves a single descriptor, and the case
scores **PASS**. A pidfd refers to exactly one process, so such a record cannot be a faithful
rendering of anything — it indicates a tracer or parser problem, and accepting it discards that
signal. §7 of the verification instruction names this case explicitly.
*Disposition:* a descriptor whose `si_pid` values conflict should be INVALID.

**V-3 — the checker accepts an empty trace object as a syscall record.**

`checker.score_case` tests `record.get("trace") is None`, so `{}`, `[]`, `""` and `0` all satisfy the
traced-case requirement and score **PASS**. The verification instruction states directly that no
fabricated or default empty trace object may satisfy checker requirements.

Not currently reachable through the driver — `parse_strace_child_window` returns `None` or a
populated dict, and `evaluate()` copies that through — so this is **latent**, not live. But the
guarantee the definition relies on is weaker than it reads.
*Disposition:* require a mapping carrying a non-empty `child_syscalls`.

#### MINOR

**V-4 — E1, E7, F4 and F7 declare `traced: true` and nothing ever reads their trace.**

A deliberately malformed trace (`child_syscalls` not a list, stage sequence outside the frozen
vocabulary) still yields **PASS** for these four, because their outcome rules consume the receipt
and the helper report, not the syscall window; only M1, M2, M3 and M4 examine trace content. The
checker requires merely that a trace *exists*. The definition offers the trace as establishing that
"`execveat(...)` was the syscall used on the pinned descriptor and that the child window contains
nothing else" — for these four that is not checked.

Pre-existing, **not created by this delta** — recorded because this verification exercised it, and
because V-3 would otherwise let an empty object satisfy even the existence requirement.

### 19.4 What was verified sound

- **R-1, for every case except truncation.** Fragments are keyed on the traced task **and** the
  syscall name; a resume naming a different call is refused rather than spliced. Verified INVALID
  (never FAIL): lost resumed half, orphan resume, mismatched resume, two unfinished from one task,
  a resume attributed to another task, malformed prefix, empty record, line noise. Verified PASS:
  the split rendering itself, interleaving from a third task, `strace: Process N attached` and
  `+++ exited +++` noise, a `--- SIGCHLD … si_pid=222 ---` delivery line, three task-prefix
  spellings, and whitespace variation. Simultaneous unfinished `clone3` from two tasks rejoin
  correctly and are then refused as ambiguous — the honest answer.
- **The regex backtracking class was re-checked.** The malformed-prefix trap the author hit is gone;
  the well-formed prefix is matched first and the patterns are anchored with no nested quantifiers
  over shared alphabets.
- **R-2 is complete.** `EVIDENCE_CLOSURE` does not exist. All six alternate routes — `dup2`,
  `pidfd_getfd`, `/proc/<pid>`, legacy `clone(CLONE_PIDFD)`, a second `fork()` child, `SCM_RIGHTS` —
  and the bare no-write-back baseline are all **INVALID**. Absence of `pidfd_open` is one required
  fact, not proof of origin.
- **R-3 correlates properly.** Verified: pid match PASSes; mismatch, absent `si_pid`, two
  descriptors claiming the same child, and a clone3/lifecycle descriptor mismatch are all INVALID;
  an unrelated child's reap and a repeated reap on the same descriptor do not disturb it.
  **Ignoring a later `ECHILD` is sound**: the frozen lifecycle reaps once, `ECHILD` renders no
  `si_pid`, and treating a post-reap failure as erasing an earlier correct observation would let a
  benign second call destroy valid evidence.
- **R-4 matches the owner's choice.** `strace` is the sole tracer; no `ptrace`, eBPF, helper binary
  or root path was added (verified by a code-only grep that excludes comments). Missing, old,
  unreadable-version and unusable tracers each HALT. No halt record mentions `clone3_unavailable`,
  and `no_tracer` yields no block cause for M3. The halt provably precedes any posing: in
  `run_trial` the `HALT_PREFLIGHT` return sits before the case loop.
- **R-5 is done.** No stale "seven" remains in active text; the single remaining occurrence is
  inside the blockquote explicitly marked **SUPERSEDED**, which is correct to leave.
- **P-14 holds for reconstructed traces.** A trace containing pid `31337`, pointer `0x7ffd1234`,
  `/home/runner/.netrc` and the account name leaked none of them; the raw text is withheld by key,
  the digest survives, and the normalised facts carry `DIRECT_CHILD` / `DIRECT_CHILD_PIDFD`.
- **The seven non-M3 traced cases did not regress**: all score PASS with evidence and INVALID
  without it.

### 19.5 On static posability

`72 handlers / 72 posable / 0 unposable` is confirmed. It proves **case-path completeness only** —
that every preregistered case has a construction, an evidence channel and a scoring path. It is
**not** mechanism acceptance, not a prediction of runtime success, not compatibility, not readiness,
and not D-7 authorisation. The manifest's own wording is consistent with this.

## 20. Final verification classification

**`FINAL_BOUNDED_NEEDS_PRETRIAL_FIXES`.**

R-2, R-3, R-4 and R-5 are correctly resolved. **R-1 is not**: the truncated-record path (**V-1**)
still lets an incomplete trace produce `MECHANISM_REJECTED`, which is the exact defect class R-1
existed to close, and the test intended to cover it passes for an unrelated reason. Two IMPORTANT
findings (**V-2**, **V-3**) and one MINOR (**V-4**) accompany it.

All four are bounded and repairable within the semantics already chosen; none requires a new owner
decision. The strace contract is grounded, the 5.4 floor is defensible, the `ptrace_scope` gate is
correct, freeze integrity holds, and **D-7 remains not authorised with a valid trial count of
ZERO.**

---

## 21. Disposition of final bounded findings V-1 through V-4

**Appended after sections 19–20, which are preserved unchanged — the findings stand as recorded
before any correction.** This section records what was done about them, and one new finding the
work surfaced. **LAUNCH-EXEC-01 remains NOT_RUN, D-7 is not granted, and the valid trial count is
ZERO.**

| ID | Severity | Disposition |
|---|---|---|
| **V-1** | BLOCKER | **FIXED** — three-state return, and a bounded logical record |
| **V-2** | IMPORTANT | **FIXED** — contradictory `waitid` identity is INVALID |
| **V-3** | IMPORTANT | **FIXED** — one shared structural traced-evidence gate |
| **V-4** | MINOR | **FIXED** — the gate covers all eight traced cases |
| **V-5** | IMPORTANT | **NEW, OPEN** — not fixed here; see 21.5 |

### 21.1 V-1 — the return is three-state, and the record is bounded

`clone3_succeeded` is gone. `parse_return_state()` returns `RETURN_OBSERVED_SUCCESS`,
`RETURN_OBSERVED_ERROR` or `RETURN_NOT_OBSERVED`, and the rule treats them as three different
facts: an observed error is `clone3_failed`, a **missing** return is **INVALID**, and no absent
return can produce any mechanism-failure token.

`split_syscall_record()` delimits the argument list by counting the syscall's **own** parentheses,
skipping quoted strings so a path containing `(` cannot unbalance it, and requires everything after
the matching close to be the result and nothing else. A record that never closes — truncated, or run
together with the following line — yields no return at all rather than borrowing one.

All five scenarios the instruction named are pinned by test:

| | Input | Result |
|---|---|---|
| **A** | resumed with no return, then `poll(...) = 1` | INVALID; `clone3_return` is **not** 1 |
| **B** | resumed truncated before `= …` | INVALID |
| **C** | proper resume `= 222`, then `poll(...) = 1` | return **222** |
| **D** | resume `= -1 EPERM`, then an unrelated `= 7` | `clone3_failed`; the 7 ignored |
| **E** | quoted argument containing `\n`-like noise | boundary unmoved |

**The exact record that previously produced `MECHANISM_REJECTED` now produces INVALID →
`MECHANISM_INCONCLUSIVE`**, asserted end-to-end through join → parser → P-12 → checker → aggregate.

### 21.2 V-2 — consistency before filtering

`_correlated_pidfd()` now groups `waitid(P_PIDFD, …)` observations by descriptor **first**. A
descriptor carrying more than one distinct `si_pid` makes the record self-contradictory and
**INVALID** — a pidfd refers to exactly one process, so filtering to the wanted child and discarding
the conflict would hide a tracer or parser fault behind a PASS. Only after that check does it
require a unique descriptor whose `si_pid` equals the `clone3` return. Entries with no rendered
`si_pid` are still ignored, so an `ECHILD` retry cannot erase an earlier correct observation.

### 21.3 V-3 / V-4 — one gate, all eight cases

`checker.valid_trace_record()` is the single place the invariant lives, and a test asserts it is
defined once and called once. For **every** `traced: true` case it rejects `None`, `{}`, `[]`, `""`,
`0`, a mapping without `child_syscalls`, a `child_syscalls` of the wrong type or empty or holding
non-names, and a record marked `integrity_ok: False` or `truncated: True` — **INVALID, before any
PASS expectation, and never FAIL**. `parse_strace_child_window()` now carries `integrity_ok` so the
gate can see reconstruction failures it did not itself parse.

This closes V-4 directly: E1, E7, F4 and F7 read the receipt and the helper report rather than the
window, and could previously PASS while carrying a malformed record. The gate is deliberately
**structural** — what a window must *contain* remains each case's own frozen business.

Two fixtures that had been relying on the hole (`record["trace"] = []`) were corrected rather than
the gate loosened.

### 21.4 What did not change

Partition **72 / 54 / 11 / 7**, traced set the same **eight**, `status` `NOT_RUN`,
`d7_execution_authorised` `false`, **72 handlers / 72 posable / 0 unposable**, and **no C source
byte changed** — all six digests identical to the source Build 5 compiled clean. R-1 through R-5
were re-checked and none regressed.

### 21.5 V-5 — a new finding, recorded and NOT fixed

**IMPORTANT. `evidence.Sanitiser` redacts the account name by unbounded substring replacement.**

Re-running P-14 surfaced it. `Sanitiser.text()` does `out.replace(self._user, "<USER>")`, so any
username that appears as a substring of evidence text corrupts it:

| username | text | becomes |
|---|---|---|
| `ci` | `specific` | `spe<USER>fic` |
| `run` | `truncated` | `t<USER>cated` |
| `test` | `latest_status` | `la<USER>_status` |
| `u` | `stage_sequence` | `stage_seq<USER>ence` |

These are realistic CI account names. It **over-redacts rather than leaking**, so it damages
evidence instead of exposing it — field names, syscall names and stage names are all reachable, and
a hex-like username could corrupt a digest.

*Disposition:* **not fixed in this task.** It is a new finding outside the V-1…V-4 disposition the
owner authorised, and this repository's discipline is review → owner decision → fix. It is recorded
here and in the freeze's `open_findings` so it cannot be lost, and the test that exposed it carries
a comment naming it.

*Also closed while re-verifying P-14:* the parsed child window carried the direct child's **raw
pid** into published evidence via `record["trace"]["child_pid"]`. `child_pid` is now withheld by
key, alongside the other experiment-local identifiers; `child_syscalls` and `stage_sequence` remain
published, and the trace digest still survives.

## 22. Status after V-1 … V-4

**`FINAL_FIXES_READY_FOR_SHORT_REVIEW`.**

346 tests pass, static posability is 72/72/0, and the adverse-verdict path is closed for every
incomplete, malformed and ambiguous record tested. This is **not** a readiness statement and is
deliberately not self-certified: the corrections were made by the same hand that wrote the code the
final review examined.

**One final short independent delta verification is required before D-7**, and it must also dispose
of **V-5**. **D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of
ZERO.**

---

## 23. Disposition of V-5

**Appended after sections 19–22, which are preserved unchanged.** V-5 was recorded OPEN at freeze
`2621381`; the owner authorised a bounded fix. **LAUNCH-EXEC-01 remains NOT_RUN, D-7 is not
granted, and the valid trial count is ZERO.**

### 23.1 The old algorithm, and what it actually did

```python
out = out.replace(self._user, "<USER>")          # unbounded substring
out[self.text(k)] = self.record(v, _key=k)       # and it rewrote KEYS
```

Both are gone. Re-verifying also found a **third** corruption the finding had not named: the
generic high-entropy rule `[A-Za-z0-9_\-+/=]{28,}` destroyed frozen vocabulary independently of any
username — `WriterRetainedAfterChildExit` is **exactly 28 characters** and became
`<OPAQUE:2a78fa59>`, as did `never_reports_unobserved_exit_status`. A completeness value and an M5
gate name, both published evidence.

### 23.2 The new algorithm

Redaction now happens to **values**, according to what a value *is*:

| Class | Rule |
|---|---|
| **Structural keys** | **never rewritten.** The only key-level operation is `INTERNAL_ONLY_KEYS` withholding, which replaces the value and leaves the key byte-exact |
| **Frozen vocabulary** | taken from the manifest — case ids, predictions, safe sets, gates, stages, block reasons, syscall lists, injection and control modes, decisions — and never rewritten. Derived from `frozen_cases`, so a token added there is covered without editing the sanitiser |
| **Path roots** | matched only at a **path boundary** (`(?![A-Za-z0-9._+-])`), so a short root like `/w` can no longer rewrite `/work` |
| **Username** | redacted **only** as a whole path **component**, or as the value of an explicit host-identity field |
| **Environment values** | unchanged — never reproduced; name plus length and digest prefix |
| **Host-identity fields** | `user`, `username`, `login`, `account`, `owner`, `hostname`, … — the **whole value** is redacted |

`/opt/runtime`, `/tmp/truncated-output` and `/var/lib/runner-tools` are untouched by username
`run`, because `runtime` is not `run`.

### 23.3 Verification

- **204 checks** — 12 adversarial usernames (`ci`, `run`, `test`, `id`, `pid`, `fd`, `exec`,
  `clone`, `wait`, `user`, `root`, `u`) × 17 vocabulary tokens — **zero corruptions**. A second
  test drives every token straight out of `frozen_cases` (60+ tokens) against 13 usernames.
- **Key immutability** is asserted by recursively capturing every mapping key before and after
  sanitisation, for eight adversarial usernames.
- **P-14 was not weakened.** Still redacted or withheld: Linux home paths, another account's home,
  Windows profile paths, temp directories below a profile, the build/work roots, GitHub PATs, AWS
  key ids, JWTs, every environment value, every `INTERNAL_ONLY` field, and `child_pid`. A test
  asserts the fix did **not** simply disable username redaction — `/usr/lib/alice/plugin.so` still
  loses the component.
- **Determinism** holds: the same document serialises byte-identically across repeated runs, for
  every adversarial username.
- **V-1 … V-4 and R-1 … R-5 all re-verified**, including the full adverse-verdict path, which
  reports **no deviations**.

### 23.4 What did not change

Partition **72 / 54 / 11 / 7**, traced set the same **eight**, **72 handlers / 72 posable / 0
unposable**, `status` `NOT_RUN`, `d7_execution_authorised` `false`, and **no C source byte
changed** — all six digests identical to the source Build 5 compiled clean. No case membership,
classification, outcome vocabulary, checker algebra, M3 evidence semantic, strace preflight or D-7
gate was touched: V-5 is an evidence-sanitisation fix only.

## 24. Status after V-1 … V-5

**`FINAL_SANITISER_FIX_READY_FOR_SHORT_REVIEW`.**

360 tests pass. This is **not** a readiness statement and is deliberately not self-certified: the
corrections were made by the same hand that wrote the code the final review examined.

**One final short independent delta verification of V-1 through V-5 is required before D-7.**
**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

---

## 25. Final short independent verification of `b107ff9..3f735d5`

**Scope: the V-1…V-5 correction delta only.** Sections 1–24 are preserved unchanged and nothing was
corrected here. Frozen source, definition and manifest are byte-identical to `3f735d5`, and the
freeze still verifies. No case posed, no helper or spike executed, no HELM ELF traced.
**LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

### 25.1 Verified

| | Result |
|---|---|
| Freeze | **16/16** source + **3/3** definition hashes exact; no C changed |
| Partition / traced set | **72 / 54 / 11 / 7**; **E1, E7, F4, F7, M1, M2, M3, M4** |
| Handlers / posable / unposable | **72 / 72 / 0**; `NOT_RUN`, D-7 `false` |
| **V-1** | truncated record → **INVALID → `MECHANISM_INCONCLUSIVE`**, never FAIL; the next line's `= 1` is **not** adopted; a genuine `-1 EPERM` still reaches `clone3_failed` → `MECHANISM_REJECTED` |
| **V-2** | `222,222` valid · `222,333` on one descriptor **INVALID** · foreign-only INVALID · two descriptors claiming one child INVALID · later `ECHILD` harmless · unrelated child does not contaminate |
| **V-3 / V-4** | **8 cases × 11 invalid shapes = 88 checks, all INVALID**; valid traces still score; the gate is defined once and called once |
| **V-5** | no `replace(self._user, …)`; keys never pass through `text()`; keys immutable |
| **P-14** | home/other-account/Windows/temp paths, PATs, JWTs, env values, all `INTERNAL_ONLY` fields and `child_pid` still redacted or withheld; a username path **component** still goes; serialisation deterministic |
| **R-1…R-5, M3** | all intact; no `pidfd_acquisition` field exists in `launcher_spike.c`; M3's only cause remains `clone3_unavailable` |
| Suite | **360 tests, all pass** |

The opaque rule still redacts genuine secrets, and SHA-1/SHA-256 digests are still preserved — the
vocabulary exemption did not disable P-14.

### 25.2 Vocabulary audit

**473 fixed symbolic values** were enumerated from the actual published shape — every
`plan.as_dict()` for all 72 cases, `frozen_cases.summary()`, every predict/safe/gate/stage/cause/
syscall/mode/decision, the checker and observation vocabularies, the normalised acquisition object
and the child window — and each was driven through the sanitiser under **13 adversarial usernames**.
**30** are ≥28 characters and therefore at risk from the generic opaque rule.

**Result: `CURRENT_VOCABULARY_INCOMPLETE`** — by exactly one token.

#### F-1 — IMPORTANT — `sigterm_blocked_sigpipe_ignored` is corrupted in published evidence

`plan.as_dict()["parent"]` for case **T6** is `sigterm_blocked_sigpipe_ignored`, 31 characters. It
is not in the manifest-derived vocabulary and not in the literal allowlist, so the generic
`[A-Za-z0-9_\-+/=]{28,}` rule rewrites it to `<OPAQUE:dad063b7>` in the published document.

Confirmed by sanitising the **real** document rather than the token in isolation, and it happens
under **every** username tested — this is the length rule alone, **not** the username defect V-5
fixed. It is the same class as `WriterRetainedAfterChildExit`, which was caught and allowlisted;
this one was missed.

*Consequence:* T6's parent-state name is unreadable in evidence. It leaks nothing, changes no
verdict, and touches neither P-12 nor the checker — but §6's required invariant is that *every*
legitimate fixed public-evidence token reachable in this candidate is preserved byte-exact, and this
one is not.
*Disposition:* add the driver's `PARENT_STATES`, `SETUPS` and `POSED_CHECKS` names to the
vocabulary, so the allowlist is derived from the driver as well as the manifest.

#### F-2 — MINOR — A4's 4096-byte argument is published as an opaque token

A4's frozen argument is `"x" * MAX_ARG_BYTES`, and `plan.as_dict()["helper_args"]` carries it into
evidence, where the opaque rule replaces it with `<OPAQUE:a2e659da>`. This is **data, not a fixed
symbolic token**, the substitution is deterministic, and the case's verdict rests on `argv_exact`
rather than on the string being readable. Recorded as observed behaviour, not a defect.

#### F-3 — MINOR — a dead posed-check registration

`returned_before_descendant_lifetime` (35 characters) is registered in `POSED_CHECKS` and
**referenced by no plan**, so it never reaches evidence today. It would be corrupted like F-1 if a
future plan used it. Recorded so the two findings are fixed together.

## 26. Final short verification classification

**`FINAL_SHORT_REVIEW_NEEDS_FIXES`.**

V-1 through V-5 are genuinely resolved, R-1 through R-5 and M3 remain resolved, P-12 and P-14 hold,
freeze integrity is exact, and the trial count is ZERO. The single obstacle to
`READY_FOR_OWNER_D7` is **F-1**: `CURRENT_VOCABULARY_INCOMPLETE`, which §6 makes a precondition.
F-1 and F-3 share one bounded fix.

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

---

## 27. Disposition of F-1 and F-3

**Appended after sections 25–26, which are preserved unchanged — the final short verification's
findings stand as recorded.** **LAUNCH-EXEC-01 remains NOT_RUN, D-7 is not granted, and the valid
trial count is ZERO.**

| ID | Severity | Disposition |
|---|---|---|
| **F-1** | IMPORTANT | **FIXED** — vocabulary derived from every symbolic registry |
| **F-2** | — | **NOT A DEFECT**, unchanged: A4's long argument stays `<OPAQUE:…>` |
| **F-3** | MINOR | **RESOLVED** by the same derivation, without being made reachable |

### 27.1 The class, not the instance

`sigterm_blocked_sigpipe_ignored` was missing because the allowlist was **hand-written**. Adding
that one string would have fixed the instance and left the class open, so the fixed vocabulary is
now **derived from the registries**:

| Source | Contributes |
|---|---|
| `frozen_cases` | case ids, predictions, safe sets, gates, stages, block reasons, syscall lists, injection and control modes, decisions, M3 evidence facts |
| `observations` | `RULES`, spike dispositions and refusals, signal and errno tables, report states, return states, lifecycle and exec constants |
| `checker` | per-case statuses and aggregate verdicts |
| published schema | `normalise_acquisition()` field names, trace-integrity fields, role names |
| **`driver`** | **`SETUPS`, `PARENT_STATES`, `POSED_CHECKS`, `ALL_CHANNELS`** |

`driver` imports `evidence`, so a module-level import back would be circular; it is imported
**lazily** and cached only on success. Nothing in `driver`'s module body sanitises anything, so the
registries are always populated by the time a value is redacted. Verified under five import orders
and with `driver` deliberately not pre-imported — **379 tokens protected** in every case.

### 27.2 Not over-broadened

Argv contents, environment values, host paths, captured bytes and tracer text are **not**
vocabulary. The negative control is explicit and tested: **A4's 4096-byte argument is still
published as `<OPAQUE:a2e659da>`**, because it is data. Genuinely opaque probes, GitHub PATs, AWS
key ids and JWTs are still redacted, and SHA-1/SHA-256 digests are still preserved.

### 27.3 The closure test is not circular

`independent_vocabulary()` in the suite walks the registries **itself** rather than calling
`evidence.vocabulary()`, and a separate test asserts the sanitiser's derivation covers that
independent enumeration. If a source is ever dropped from the sanitiser, the two disagree and the
test fails — which is the point; deriving both sides from one helper would make the test agree with
any mistake. A further test registers a fresh 38-character `PARENT_STATES` key at runtime and
asserts it is protected, proving the class is closed rather than the two named strings.

The whole published plan table is also exercised: every symbolic field of all **72** plans, under
**13 adversarial usernames**, byte-exact.

### 27.4 What did not change

Partition **72 / 54 / 11 / 7**, traced set the same **eight**, **72 handlers / 72 posable / 0
unposable**, `status` `NOT_RUN`, `d7_execution_authorised` `false`, and **no C source byte
changed**. V-1…V-5, R-1…R-5, PRE-D7-B1 and M3 were all re-verified and remain closed, including the
full adverse-verdict path, which reports no deviations.

## 28. Status after F-1 / F-3

**`VOCABULARY_FIX_READY_FOR_MICRO_REVIEW`.**

374 tests pass and the current vocabulary is closed. Not self-certified: this fix was made by the
same hand, and **one final micro-review of the F-1/F-3 delta is required before D-7.**

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

---

## 29. Final micro-review of `abe747f..f1f49ca`

**Bounded to the F-1/F-3 vocabulary delta.** Sections 1–28 preserved; nothing corrected here.
Frozen candidate `f1f49ca` is byte-identical and still verifies. No case posed, no helper or spike
executed, no HELM ELF traced. **LAUNCH-EXEC-01 remains NOT_RUN, valid trial count ZERO.**

**Delta scope confirmed:** `evidence.py`, its tests, the freeze and this document — **no** change to
`frozen_cases.py`, `driver.py`, `observations.py`, `checker.py`, `run_launch_exec_01.py`, the
definition, or any C source.

### 29.1 Verified

| | Result |
|---|---|
| Freeze | 16/16 source + 3/3 definition hashes exact |
| Partition / traced set | **72 / 54 / 11 / 7** and the eight traced cases, by **AST parse** |
| Handlers / posable / unposable | **72 / 72 / 0**; `NOT_RUN`, D-7 `false` |
| **F-1** | T6's real `plan["parent"]` is `sigterm_blocked_sigpipe_ignored` before *and* byte-exact after sanitisation, through the real `record()` path, under all 13 usernames |
| **F-3** | registered in `POSED_CHECKS`, referenced by no plan, still protected — and **not** a literal exception: the token appears in `evidence.py` only inside a comment, and removing the driver registries from the derivation loses protection (`<OPAQUE:dad063b7>`), proving the cover is registry-class |
| **Vocabulary** | **366** tokens enumerated independently of `evidence.vocabulary()`; **zero missing**; **zero corrupted across 4 758 checks** (366 × 13 usernames) |
| **F-2** | A4's 4096-byte argument still `<OPAQUE:a2e659da>` and **not** in the vocabulary; opaque probes, PATs, AWS ids and JWTs still redacted; digests preserved |
| **P-14** | no username substring rule; keys byte-exact; host paths and username **path components** redacted while `/opt/runtime` is untouched; env values never raw; `INTERNAL_ONLY` and `child_pid` withheld; deterministic. Vocabulary preservation and privacy hold **simultaneously** |
| **Prior fixes** | V-1…V-5, R-1…R-5 and M3 all re-verified; the adverse-verdict path reports **no deviations** |
| Suite | **374 tests, all pass** |

### 29.2 Finding

#### M-1 — IMPORTANT — the lazy driver import caches a partial registry permanently and fails open

Seven import scenarios were run in **fresh interpreters**. Import order is irrelevant — evidence
first, driver first, checker/observations first, and evidence-alone all yield an identical **379**
tokens with the F-1 token protected. Repeated queries are stable across 50 calls. A failed import
(`sys.modules['driver'] = None`) is **not** cached and recovers fully on the next call. Importing
creates no build directory and poses nothing.

**One scenario fails.** With a driver module whose registries exist but are **empty** — the state
`driver.py` is genuinely in between creating `SETUPS`/`PARENT_STATES`/`POSED_CHECKS` and the
decorators filling them — `_driver_vocabulary()` treats the read as a success and caches it:

```
F  PARTIAL driver module (registries present but EMPTY)  334 False PARTIAL_CACHED_PERMANENTLY
```

45 tokens are lost, including `sigterm_blocked_sigpipe_ignored`, and the guard
`if _DRIVER_VOCABULARY is not None` means it **never recovers**. Separately, a failed import returns
an empty set and sanitisation **continues** with a knowingly-incomplete vocabulary rather than
refusing — the fail-open behaviour the instruction says to prefer against.

*Reachability:* **not reachable in this frozen candidate.** `vocabulary()` is called only from
`_opaque_token`, i.e. during sanitisation, and nothing in `driver.py`'s module body sanitises
anything — verified by inspection of its import-time work. So no evidence a real trial produces
could be affected today. It is recorded because the instruction makes "no silently cached incomplete
vocabulary" an explicit criterion, and because the guard is one edit away from becoming reachable.

*Disposition:* refuse to cache an empty registry read, and fail closed — raise, or fall back to a
conservative "protect everything symbolic" posture — rather than silently publishing corrupted
symbolic evidence.

## 30. Micro-review classification

**`MICRO_REVIEW_NEEDS_FIXES`.**

`CURRENT_VOCABULARY_CLOSED` and every other criterion holds: F-1 and F-3 are genuinely fixed at the
class level, F-2 remains a valid negative control, P-14 is sound, all prior bounded fixes remain
closed, freeze integrity is exact and the trial count is ZERO. The single obstacle is **M-1**, which
makes the lazy-import mechanism `LAZY_IMPORT_NEEDS_FIX`.

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

## 31. M-1 fix: fail-closed driver-vocabulary acquisition

Owner decision after section 30: fix **M-1** before D-7. This section records what changed and what
was verified. It is a fix record, not a readiness statement.

### 31.1 What was wrong

`evidence._driver_vocabulary()` imported `driver` lazily — necessarily, since `driver` imports
`evidence` — and then read `driver.SETUPS`, `PARENT_STATES`, `POSED_CHECKS` and `ALL_CHANNELS`
directly. Python publishes a module object in `sys.modules` **before** executing its body, so
`import driver` can succeed against a module whose decorators have not run yet. Reading those
globals cannot distinguish

* *empty because this freeze registers nothing* from
* *empty because the body is still executing*,

and the empty answer was cached permanently by `if _DRIVER_VOCABULARY is not None`. Measured: 334
tokens instead of 379, `sigterm_blocked_sigpipe_ignored` lost, no recovery for the life of the
process. A separate path returned `frozenset()` on import failure and **continued sanitising**, so
published symbolic evidence would have been silently rewritten into `<OPAQUE:...>` digests.

### 31.2 The interface, not a guess

`driver.registry_vocabulary()` is defined as the **last statement of `driver.py`'s module body**,
after every registry is populated. Its mere existence is therefore a positive readiness fact rather
than an inference from globals that may or may not be filled in: a partially initialised driver does
not have the attribute. A test asserts by AST that it is the last top-level definition, so appending
anything after it fails rather than silently weakening the signal, and a second test asserts the
function calls nothing but `tuple` and `sorted` — it is consulted during sanitisation and must never
act.

It is **not a second source of truth**: it returns the registries themselves, as tuples so a
consumer cannot mutate the driver through what it is handed.

### 31.3 Failure and cache semantics

`evidence.VocabularyUnavailable` is raised — never swallowed, never replaced by an empty set — on
every one of: driver import failure; a partially initialised driver with the registries absent;
registries present but empty; a required registry missing from the snapshot; a registry of the wrong
type; a registry holding a value that is not a name; an accessor that is not callable or that
raises; a snapshot that is not a mapping.

| state | cached? |
|---|---|
| failed load | **no** |
| partial load | **no** |
| complete validated load | yes — as an immutable `frozenset` |
| later successful calls | identical object |

`Sanitiser.text`, `Sanitiser.record` and `serialise` resolve the vocabulary **before** emitting
anything. That matters: only `_opaque_token` consults the vocabulary, and only for runs of at least
28 characters, so a record that happened to contain no long token would otherwise have been
published having never established that the vocabulary was available. The halt now depends on the
driver's state, not on which data the case produced.

### 31.4 Import matrix — sixteen fresh interpreters

Import order is a property of a whole interpreter, so every scenario runs in a new one rather than
by poking `sys.modules` in a process where everything is already imported.

```
A  evidence first, then driver     COMPLETE 379 True True True
B  driver first                    COMPLETE 379 True True True
C  checker+observations first      COMPLETE 379 True True True
C2 evidence alone                  COMPLETE 379 True True True
D  driver import FAILS             HALT 4 published=0 not_cached
E  failure then recovery           HALT 1 published=0 not_cached / COMPLETE 379
F  partial: registries ABSENT      HALT 4 published=0 not_cached / COMPLETE 379
G  partial: registries EMPTY       HALT 4 published=0 not_cached / COMPLETE 379
G2 partial: a registry MISSING     HALT 4 published=0 not_cached / COMPLETE 379
G3 partial: registry MISTYPED      HALT 4 published=0 not_cached / COMPLETE 379
G4 accessor RAISES                 HALT 4 published=0 not_cached / COMPLETE 379
G5 accessor NOT CALLABLE           HALT 4 published=0 not_cached / COMPLETE 379
G6 snapshot NOT a mapping          HALT 4 published=0 not_cached / COMPLETE 379
G7 registry holds a non-name       HALT 4 published=0 not_cached / COMPLETE 379
H  50 repeated queries             STABLE 1 size, 1 object id / COMPLETE 379
I  import poses nothing            SIDE_EFFECTS False False / COMPLETE 379
```

`HALT 4` means all four public entry points — `vocabulary()`, `Sanitiser.text`, `Sanitiser.record`
and `serialise` — refused; `published=0` that nothing was emitted; `not_cached` that both
`_DRIVER_VOCABULARY` and `_VOCABULARY` were still `None` afterwards. Every refused state then
recovers to the same complete 379-token set. Distinct complete sizes across the whole matrix: **one**.

### 31.5 Current vocabulary remains closed

The independent oracle — deliberately not `evidence.vocabulary()` — enumerates **366** fixed
symbolic tokens by walking the registries itself.

| | |
|---|---|
| independently enumerated | 366 |
| missing from the sanitiser | **0** |
| token × username checks | 4,758 |
| corrupted | **0** |

`sigterm_blocked_sigpipe_ignored` is still T6's real `plan["parent"]` before sanitisation and
byte-exact after it through the real `record()` path under all 13 adversarial usernames.
`returned_before_descendant_lifetime` is still registered, still referenced by no plan, and still
protected. **`CURRENT_VOCABULARY_CLOSED`.**

### 31.6 Negative controls unchanged

A4's 4096-byte argument is still `<OPAQUE:a2e659da>` and still absent from the vocabulary; the
high-entropy probe, PAT, AWS key id and JWT are still redacted; SHA-1 and SHA-256 digests are still
preserved. Protection was not broadened into a length exemption.

P-14 re-confirmed: keys byte-exact, host identity redacted whole, environment values never
reproduced, `child_pid`, `acquisition`, `raw_trace` and `capture_prefix_base64` withheld by key,
`/opt/runtime` untouched by the username rule, serialization deterministic.

### 31.7 Scope

No case membership, case class, checker rule, M3 semantics, trace semantics, strace preflight, P-12
rule or D-7 gate changed. **No C source byte changed** — the six `.c` digests still match the
manifest and Build 5 remains applicable. Partition **72 / 54 / 11 / 7**, traced set **E1 E7 F4 F7 M1
M2 M3 M4**, **72 handlers / 72 posable / 0 unposable**, `status = NOT_RUN`,
`d7_execution_authorised = false`.

390 tests pass (374 + 16 new). `validate_docs` PASS. `cargo check --workspace --all-targets` exit 0.
Freeze verification `true` against the new descendant manifest. Runner default invocation exits 3.
Zero experimental ELF executions, zero preregistered cases posed.

### 31.8 Classification

**`M1_FIX_READY_FOR_FINAL_CHECK`.**

This fix was written by the same author who wrote the code it corrects, and the tests that exercise
it are the author's own. One final independent M-1 check is required before D-7.

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

## 32. Final independent M-1 check of `c418284`

Frozen candidate `c41828477a776e8ce55c00a02d6ee452f3680dc1`, clean tree, branch
`docs/helm-launch-architecture`, 10 commits unpushed. Review only; the candidate is byte-identical
after it. No case posed, no ELF executed, no tracer attached to a HELM binary.

### 32.1 Freeze

16/16 source and 3/3 definition hashes exact. Recomputed by AST parse of `frozen_cases.py` rather
than by importing it: **72 total, 54 mandatory, 11 conditional, 7 recorded**, zero duplicate ids,
traced set exactly **E1 E7 F4 F7 M1 M2 M3 M4**. Driver: **72 handlers, 72 posable, 0 unposable**.
`status = NOT_RUN`, `d7_execution_authorised = false`, **valid trial count ZERO**. `git diff 002e1e4
HEAD -- *.c` is empty and all six C digests match the manifest, so **Build 5 still applies**.

### 32.2 The accessor design, attacked directly

The interesting claim is 3C — that whenever `registry_vocabulary` exists, the registries it returns
are already complete. Stubbing a module cannot test that, so this check executed **driver's real
module body one top-level statement at a time**, installing each prefix in `sys.modules` as the
genuine partially-initialised module, and asked `evidence` for the vocabulary after every prefix.
The cache was **never reset** across the whole sweep, so any poisonable state would have shown up at
the end.

```
top-level statements in driver.py    : 95
prefixes that executed and REFUSED   : 94
prefixes that executed and ANSWERED  :  1   (prefix 95 -- the complete body)
incorrect answers / poisoned caches  : NONE
final real import, same interpreter  : 379 tokens, F-1 token present, frozenset
```

Every proper prefix refused. Exactly one prefix answered, and it answered completely. By AST:
`registry_vocabulary` is the last of the 95 top-level statements, is bound exactly once, carries no
decorator and no default expression, calls only `tuple` and `sorted`, and no module-level statement
rebinds or deletes `SETUPS`, `PARENT_STATES`, `POSED_CHECKS` or `ALL_CHANNELS` after their single
assignment (indices 30, 31, 32, 22 of 95).

3F: no Python `threading`, `_thread`, `multiprocessing`, `concurrent.futures` or `importlib.reload`
appears anywhere in the experiment or its tests — the only threading in the model is
`launcher_spike.c`'s own, in the process under test. The frozen use model is a single-threaded,
single-shot CLI, so no second importer can observe an incomplete snapshot.

One residual, recorded as an observation rather than a finding: under `importlib.reload(driver)` the
attribute stays bound from the previous execution while the body re-runs, which is the one state
where the last-statement convention alone would not hold. The **content validation** catches it
independently — a re-run body resets the registries to empty and an empty registry is refused — and
nothing in the candidate reloads anything. The design does not rest on the convention alone.

**`ACCESSOR_READINESS_SOUND`.**

### 32.3 Failure and recovery matrix

Thirty rows, each a fresh interpreter, asserting four things directly rather than parsing a report:
refusal, nothing emitted, nothing cached, full recovery. Seven public routes were attempted per row
— `vocabulary()`, two `Sanitiser.text` calls, two `Sanitiser.record` calls, and two `serialise`
calls.

| state | result |
|---|---|
| evidence first / driver first / checker first / evidence alone | 379 tokens, frozenset |
| import failure; registries absent; registries empty | refused 7/7, emitted 0, cached nothing |
| one registry missing; wrong registry type; registry holds `None`, `''` or a number | refused 7/7 |
| accessor raises; accessor not callable; snapshot is a list; snapshot is `None` | refused 7/7 |
| recovery after every one of the above | 379 tokens, F-1 token present |
| 50 repeated queries | one size, one object id |

Rows with any failure: **0**.

### 32.4 No content-dependent bypass

`_opaque_token` is the only consumer of the vocabulary and fires only on runs of at least 28
characters, so the pre-fix code would have published a short record without ever checking. With a
partially initialised driver installed and a record whose longest string is four characters:

```
Sanitiser.text           HALTED
Sanitiser.record         HALTED
serialise                HALTED
record then serialise    HALTED
nothing cached           True
```

Bypass hunt: the only routes in the trial path that produce sanitised bytes are `Sanitiser.text`,
`Sanitiser.record` and `serialise`, and all three are guarded. `receipt_view` and
`evidence_document` only rearrange and their output must still pass `serialise`; `host_identity`,
`env_name` and `env_entry` are reducers that can only emit a fixed label or a digest. The runner's
other `json.dumps` sites (`--verify-freeze`, `--driver-completeness`) print file names, digests and
driver registry names, and `harness.py` / `oracles.py` print only from their `__main__` blocks.

### 32.5 Current vocabulary

Enumerated independently of `evidence.vocabulary()` and of the author's test helper, by walking the
published structures themselves: **434 fixed symbolic tokens**, 39 of them at least 28 characters,
checked against **17** adversarial usernames — **7,378 checks, 0 corrupted**.

66 of the 434 are absent from the sanitiser's derived allowlist. Every one was classified: 19 are
multi-word prose (help strings and notes), 11 are numeric strings, 29 are short CLI flags — none has
a 28-character contiguous run, so the opaque rule can never fire on them — 6 are SHA-256 digests
preserved by the digest rule, and the last is A4's 4096-byte argument, which is the intended
negative control. **Fixed symbolic tokens missing or corrupted: zero.**

`sigterm_blocked_sigpipe_ignored` is T6's real `plan["parent"]` and byte-exact after `record()` under
all 17 usernames. `returned_before_descendant_lifetime` is registered, referenced by no plan, and
protected.

**`CURRENT_VOCABULARY_CLOSED`.**

### 32.6 P-14 and negative controls

A4's argument is still `<OPAQUE:a2e659da>` and absent from the vocabulary. All seven credential
shapes redacted; SHA-1 and SHA-256 digests preserved. Keys byte-exact; `user`, `hostname` and
`logname` redacted whole; environment values reduced to length and digest while a declared name
survives; every `INTERNAL_ONLY` key withheld. Against the shapes the driver actually builds — a
parsed strace window and a parsed acquisition — the serialised document contains **no** direct-child
pid, **no** pointer, **no** raw `si_pid` and **no** tracer text; `acquisition_normalised` publishes
only the `DIRECT_CHILD` / `DIRECT_CHILD_PIDFD` roles. Serialization deterministic over repeated runs.

### 32.7 Prior-fix regression

R-3 correlates on a well-formed record and V-2 returns `None` — INVALID, never FAIL — on a
contradictory `si_pid`. The V-3/V-4 gate rejects a missing, non-mapping, structurally-invalid,
truncated or syscall-less record for all eight traced cases. R-2 is intact: `closure` survives only
in comments recording its removal, `launcher_spike.c` has no `pidfd_acquisition` field and the rule
reads none. R-4: `STRACE_MIN_VERSION` is `(5, 4)`; absent, unreadable and 5.3 tracers all fail
preflight and 5.4 passes; no ptrace or eBPF fallback exists. 109 targeted tests and the full 390-test
suite pass.

### 32.8 Finding

**P-16 — IMPORTANT — the published preflight reproduces the host name.** `harness.preflight()`
records `uname -a`, whose value embeds the machine's hostname. `uname` is not in
`HOST_IDENTITY_KEYS`, and the string contains no 28-character run, so the sanitiser passes it
through unchanged; `kernel_release` already records `uname -r` separately, so the `-a` variant
contributes the hostname and little else. It reaches published evidence by two routes: the sanitised
success document, and the `HALT_PREFLIGHT` document, which `run_trial` returns **unsanitised** and
`main` writes through `serialise`. Demonstrated with a fabricated preflight block; nothing was
executed.

This is **pre-existing and unrelated to M-1** — `run_launch_exec_01.py` and `harness.py` are
untouched by `c418284` and by `f1f49ca` — and it is not a secret disclosure. It is recorded because
the check criteria name "host identity remains redacted" explicitly, and because the first artefact a
D-7 trial publishes is a preflight block.

### 32.9 Classification

**`FINAL_M1_CHECK_NEEDS_OWNER_DECISION`.**

M-1 itself passes every criterion: `ACCESSOR_READINESS_SOUND`, `CURRENT_VOCABULARY_CLOSED`,
fail-closed publication verified against seven routes in thirty states, recovery verified from every
partial state, P-14 otherwise intact, freeze exact, trial count ZERO. The M-1 fix is finished. What
remains is the owner's call on P-16 — correct it before D-7, or accept it and proceed — which is a
decision, not a defect in this fix.

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**

## 33. P-16 fix: host minimisation and one publication boundary

Owner decision after section 32: fix **P-16** before D-7. This section records what changed and what
was verified. It is a fix record, not a readiness statement.

### 33.1 The two halves

P-16 was one finding with two independent causes, and fixing either alone would have left the leak
reachable.

**Collection.** `harness.preflight()` ran `uname -a`, whose value is
`Linux <nodename> <release> #<build> <arch> GNU/Linux`. The architecture (`uname -m`) and the kernel
release (`uname -r`) were **already collected separately**, so the broad form contributed the
machine's nodename and nothing else this experiment needs. It is now `uname -s` — the kernel name,
the one narrow fact that was missing. `uname -s` cannot carry a nodename. A test reads harness's AST
and asserts uname is only ever invoked with `-s`, `-r`, `-m`, never `-a` or `-n`, and that no
host-identity command (`hostname`, `hostnamectl`, `dnsdomainname`, `whoami`, `id`, `getent`) is
invoked at all.

**Publication.** `run_trial` sanitised its success document and returned the `HALT_PREFLIGHT`
document raw, which then went straight to the serializer carrying the whole environment inventory. A
failed mandatory preflight is not more trustworthy than a completed trial. The defect was structural:
sanitising at each return site means every new return site has to remember, and this one did not.

### 33.2 One boundary

`evidence.publish(document, sanitiser=None, stream=None)` is now the only way anything leaves this
process. It sanitises, serialises, writes, and returns the text so a test can assert on what the
boundary produced rather than on how a caller reached it.

`run_trial` no longer sanitises at all. It returns `(code, document, sanitiser)` with the document
**raw**, from both its return paths, and the boundary applies P-14 exactly once. Double sanitisation
is therefore impossible by construction rather than by inspection — there is only one place where it
happens.

Everything goes through it: the trial document, `HALT_PREFLIGHT`, the freeze-drift `HALT`, the
`NOT_RUN` document, `--verify-freeze`, `--driver-completeness`, `--preflight-only`, and the
`__main__` blocks of `harness.py`, `oracles.py`, `make_fixtures.py` and `frozen_cases.py` —
`make_fixtures.write()` returns the absolute path of every fixture it writes, and was printing them
raw.

The durable invariant is whole-tree, not a source line: **`json.dumps` occurs in exactly one place in
the experiment, `evidence.serialise`, and no module other than `evidence` writes to stdout.** Both
are asserted by walking the AST of every module in the directory, alongside behavioural tests over
the real entry points.

A second line of defence covers a field of that shape reappearing: `HOST_DESCRIPTOR_KEYS` — `uname`,
`uname_all`, `uname_a`, `nodename`, `node`, `domainname`, `fqdn`, `hostname_fqdn` — has its **value
withheld whole**, as `<WITHHELD:host-descriptor>`, with the key left byte-exact. Whole, because a
hostname is a short arbitrary token: no length, entropy or shape rule can separate it from the kernel
build string sitting beside it, so scanning is a game the scanner loses.

### 33.3 The original leak, through every public form

`helm-secret-host-9371` injected into a `uname -a`-shaped value:

| form | result |
|---|---|
| `uname`, `uname_all`, `nodename`, `node`, `fqdn` values | `<WITHHELD:host-descriptor>` |
| `hostname`, `host`, `runner_name` values | `<USER>` |
| `--preflight-only` document | probe absent |
| `NOT_RUN`, `--verify-freeze`, `--driver-completeness` | probe absent |
| `HALT_PREFLIGHT` document via the real halt path | probe absent |

The halt test reaches the real `HALT_PREFLIGHT` return with `preflight_gates` replaced so no
static-link probe is compiled and `auth=None`, which the halt path never touches — that it returns
before any case is posed is itself the property being relied on. It asserts the document is **raw on
the way out of `run_trial`** and clean after the boundary, so the boundary is demonstrably what
removes the taint rather than luck upstream.

### 33.4 Negative controls

The fabricated preflight failure carries a hostname, a username, a home path, a PAT-shaped token, a
runtime-token environment entry, a kernel release and an architecture. After the boundary:

```
hostname                gone
username                gone
home path               <HOME>-rooted / <ABSPATH>
token                   <CREDENTIAL>
environment value       <VALUE:len=44,...>
kernel_name             Linux
kernel_release          6.5.0-1015-azure
arch                    x86_64
strace                  /usr/bin/strace
status                  HALT_PREFLIGHT
reason                  "...no case was posed and LAUNCH-EXEC-01 remains NOT_RUN"
halts[0].gate           clone3
```

Data minimisation removed the nodename and kept every fact the experiment needs. The halt remains
interpretable.

### 33.5 P-14, M-1 and prior findings

`CURRENT_VOCABULARY_CLOSED`: 366 tokens enumerated independently, **0 missing, 0 corrupted** across
4,758 checks. F-1's token still byte-exact through the real `record()` path; F-3 still protected.
A4's argument still `<OPAQUE:a2e659da>`; credentials still redacted; digests preserved; keys
byte-exact; environment values never raw; `INTERNAL_ONLY` fields and `child_pid` withheld;
serialization deterministic.

M-1 intact and now covering more: `record` and `serialise` each establish the vocabulary first, so
the new boundary **inherits** the fail-closed rule — a test installs a partially initialised driver
and asserts `publish` raises rather than emitting. The thirty-row fresh-interpreter matrix still
passes with zero failures.

R-1…R-5, V-1…V-5, M3-T, PRE-D7-B1, F-1, F-3 and M-1 all re-verified. One prior test was rewritten
from source-slicing to AST — R-4's "the halt precedes any posing" check searched for the first
literal `HALT_PREFLIGHT` and broke when a docstring mentioned it, which said nothing about the
invariant. The invariant it asserts is unchanged and still holds.

### 33.6 Scope

No case membership, case class, checker algebra, M3 evidence, trace parser semantics, strace
preflight requirement, P-12 token semantics or D-7 gate changed. `driver.py`, `checker.py` and
`observations.py` are **byte-identical**. **No C source byte changed** — all six digests match and
Build 5 still applies.

The definition's preflight inventory is amended from `uname -a` to `uname -s`, so
`LAUNCH-EXEC-01-DEFINITION.md` is re-hashed in `definition_sha256`. That is a change to a
definition-hashed document and is called out here deliberately: leaving it would have left the
preregistration describing a collection the harness no longer performs.

407 tests pass (390 + 17). `validate_docs` PASS. `cargo check --workspace --all-targets` exit 0.
Freeze verification `true`. Partition **72 / 54 / 11 / 7**, traced set **E1 E7 F4 F7 M1 M2 M3 M4**,
**72 handlers / 72 posable / 0 unposable**, `status = NOT_RUN`, `d7_execution_authorised = false`.
Zero experimental ELF executions, zero preregistered cases posed.

### 33.7 Classification

**`P16_FIX_READY_FOR_FINAL_CHECK`.**

Written by the same author who wrote the code it corrects, and the tests that exercise it are the
author's own. One final independent P-16 micro-check is required before D-7.

**D-7 remains not authorised. LAUNCH-EXEC-01 remains NOT_RUN with a valid trial count of ZERO.**
