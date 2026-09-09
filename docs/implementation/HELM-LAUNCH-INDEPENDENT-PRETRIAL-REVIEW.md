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

## 5. What this review did not do

No preregistered case was posed. No binary was executed — the SHA-256 audit was a
transliteration, and every checker attack used fabricated records. The case-posing driver was not
implemented and both runner safety barriers were left intact. `crates/helm-launch` was not
created. No Wine, no 7-Zip, no A0, no `sudo`, no package installation. The frozen commit
`66bf4b6` was not amended, rebased or squashed.
