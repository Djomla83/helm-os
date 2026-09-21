# LAUNCH-EXEC-01 — Trial #2 pretrial independent review

Bounded independent review of the Trial #2 frozen candidate
`542a18ed12b320d15d156cd7071cb04694f7f35d`. The reviewer did not author the corrections. Nothing
was fixed, no case was posed, no `launcher_spike` or helper ran, and no dispatcher was created.

**The corrections themselves are sound. The review found a pre-existing defect that makes eight
E-series cases test something other than what they preregister, and it blocks a new D-7.**

## 1. Review target

Clean tree on `docs/helm-launch-architecture`, HEAD `542a18e`. The Trial #2 sequence is exactly three
commits on top of `eb6825f`:

| Commit | Role |
|---|---|
| `478c8a1` | Trial #1 owner acceptance |
| `73dff4d` | E5b and preservation implementation |
| `542a18e` | Trial #2 freeze |

Trial #1 is intact: its freeze `89c923a`, its result review `f7e5ca0` and its acceptance are
unamended, the review document is unchanged since `f7e5ca0`, and `89c923a` is still reachable from
HEAD. Its dispatcher is absent from `origin/main`; GitHub reports that workflow as `state: deleted`
and no longer lists it. No Trial #2 dispatcher exists.

## 2. Freeze

**17/17** source hashes and **3/3** definition hashes recomputed exact; every frozen `.py` is
covered and the manifest still does not hash itself. AST-recomputed: **72 / 54 / 11 / 7**, traced
**E1 E7 F4 F7 M1 M2 M3 M4**, **72 handlers / 72 posable / 0 unposable**, `status = NOT_RUN`,
`d7_execution_authorised = false`, **Trial #2 valid trial count ZERO**. Lineage records the Trial #1
freeze, run, outcome, consumed D-7, accepted result review, the Trial #2 reason, and that Trial #2 is
not a retry.

**No semantic drift.** `frozen_cases.py`, `checker.py`, `observations.py`, `oracles.py`,
`make_fixtures.py`, `README.md` and all six C sources are **byte-identical** to the Trial #1 freeze,
as are the definition, ADR-0024 and the architecture report. Comparing all 72 case entries field by
field between the two freezes: **no class, prediction, safe set, gate, traced flag or block cause
moved.** The `driver.py` delta is three hunks: two new helpers, the E5b setup, and `TrialContext`.

## 3. BLOCKER — eight cases do not perform the manipulation that defines them

Tracing the build-identity chain that §8 asks for — build → identity → setup → descriptor pin →
execution — leads to what a setup hands the execution path. Comparing every key the setups
**produce** with every key the driver **reads**:

```
keys setups produce   : cleanup, exec_path, extra_helper_args, hold_writer,
                        liveness_fifo, mutated_marker, not_posed, post_pin
keys the driver reads : exec_path, extra_helper_args, liveness_fifo, not_posed
PRODUCED BUT NEVER READ: cleanup, hold_writer, mutated_marker, post_pin
```

`post_pin` is written by seven setups and read by nothing — not in `driver.py`, not in any other
module, and not in `launcher_spike.c`. The spike's complete flag list contains no substitution,
mutation or retarget flag, and `_run_once` is a single blocking `subprocess.run` with no sync point
between the launcher's pin and its exec. `hold_writer` is the same: E5 declares that the harness
holds an `O_WRONLY` descriptor across launch, and nothing ever opens it.

| Case | Class | Predicts | Declares | What would actually happen |
|---|---|---|---|---|
| E2 | mandatory | `Exited:0` | rename `helper_alt` over the path after the pin | no rename — passes without the substitution it tests |
| E3 | mandatory | `Exited:0` | rename away and unlink after the pin | no mutation — passes vacuously |
| E4 | mandatory | `Exited:0` | retarget the symlink after the pin | no retarget — passes vacuously |
| **E5** | mandatory | `ExecFailed:ETXTBSY` | harness holds an `O_WRONLY` fd across launch | **no writer held, so no ETXTBSY — FAIL** |
| E6 | mandatory | `Exited:0` | `pwrite` the marker after the pin | no mutation — passes vacuously |
| E6b | mandatory | safe set | truncate and rewrite after the pin | no mutation |
| E6c | recorded | gated | shared writable mapping write after the pin | no mutation |
| **E6d** | conditional | `ExecFailed:EACCES` | `fchmod 0` after admission | **no fchmod, so no EACCES — FAIL** |

The consequence is not merely weak evidence. E5 is **mandatory**, and a mandatory FAIL fires
aggregate rule 1: a Trial #2 run against this freeze would most likely report
**`MECHANISM_REJECTED`** on the strength of a condition the harness never created — a false
falsification, which is precisely the outcome this whole preregistration exists to prevent. The five
vacuous passes are the mirror image: they would contribute to acceptance without testing anything.

`cleanup` (X8) and `mutated_marker` (E6) are dead in the same way; their effect is cosmetic by
comparison.

**This is pre-existing, not a Trial #2 regression.** The same dead keys are in the Trial #1 freeze,
and Trial #1 actually posed E2–E5 in this degraded form. No test in the suite references `post_pin`
or `hold_writer` at all, which is why 449 green tests did not catch it.

## 4. Preregistration

| | |
|---|---|
| A. `journal.py` in the frozen source set | **YES** — in `sha256`, 17/17 |
| B. Its role and contract discoverable from the authoritative preregistration | **YES** — `durable_evidence.journal` gives file, format, kinds, crash semantics, immutability |
| C. case-entered / case-completed semantics frozen | **YES** — `entered_vs_completed` |
| D. Build identity before the first case frozen | **YES** — `build_identity.when` |
| E. Public artifact contract frozen | **YES** — files, sanitisation, `preserved_after`, `never_persisted` |
| F. **Crash interpretation frozen** | **NO** |

F is the gap, and it is the exact hazard §3 names. The manifest freezes how a torn journal is
**read**. It does not freeze what a partial journal **means**. After a Trial #2 abort, a reader would
still have to choose:

1. the trial-level outcome term for a journal with no `trial_end` — Trial #1's review had to invent
   `TRIAL_ABORTED_AFTER_BOUNDARY`;
2. whether a partial journal may be aggregated at all — the frozen rule "a case with no recorded
   status is INVALID, never absent" still reads as licensing it, and the owner had to override that
   by decision for Trial #1; nothing frozen here prevents a future reader from doing what the owner
   rejected;
3. the status of an entered-not-completed case — `journal.py` defines `ENTERED_NOT_COMPLETED` in
   code, and the manifest never names it or `UNKNOWN_FROM_PRESERVED_EVIDENCE`;
4. whether an aborted Trial #2 consumes its D-7 — settled for Trial #1 by owner decision, not by the
   freeze.

**`TRIAL2_PREREGISTRATION_INCOMPLETE`.**

## 5. E5b

The frozen sequence is `open(O_RDONLY)` → `pread` one byte at 0 → `close` → `open(O_WRONLY)` →
`pwrite` that byte at 0 → `close` → launch. Verified by following each descriptor **variable** rather
than asking whether the function reads anywhere: exactly one `O_WRONLY` open with no read on it,
exactly one `O_RDONLY` open with no write on it, `O_RDWR` in no open's flags, the reader closed before
the writer opens, and both opens inside `try/finally`. A read or write of other than exactly one byte
returns `not_posed`, which the checker scores INVALID.

**On real Linux** — the owner's ordinary Ubuntu WSL distro, Python only, inert temporary files, no
HELM binary executed:

```
distro : Ubuntu 24.04.4 LTS      kernel : Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
pread on O_WRONLY                : OSError errno=9 Bad file descriptor   <- Trial #1's defect
corrected setup                  : exec_path (posed)
byte-identical                   : True   (digest 10fc3c51a152e90e before and after)
control: pread on that writer fd : errno 9 (Bad file descriptor)         <- the writer IS write-only
short read / short write         : not_posed -> INVALID
unit suite on Linux              : Ran 449, OK (0 skipped)
```

The control is the point: the writer the corrected code opens still refuses a read on this kernel, so
`O_WRONLY` still means what E5b claims. The executable body is unchanged, the write is real, and the
writer is closed before launch.

**`E5B_CLAIM_PRESERVED`.**

## 6. Descriptor defect class

Following each descriptor variable across every module: **no access-mode mismatch and no use after
close** anywhere in the Python. The four `os.open` sites are E5b's reader and writer,
`_descendant_alive` (`O_RDONLY|O_NONBLOCK`, read) and `open_non_cloexec_descriptor` (`O_RDONLY`, no
transfer). The C descriptor sites were inspected: `helper_fork` writes to an `O_WRONLY` FIFO,
`helper_report` reads an `O_RDONLY` procfs file, `launcher_spike` opens the executable
`O_RDONLY | …` before its `pread`s.

**T2-C-LATENT independently confirmed unreachable.** `--bypass-admission` with `--exec-fd-o-path`
would reach the digest loop with an `O_PATH` descriptor, because the O_PATH refusal sits inside the
admission branch. Across all 72 plans: three set `--bypass-admission`, one sets `--exec-fd-o-path`,
none sets both, and the only flag override in the driver is M2's `baseline_flags=()`. Not reachable
from any frozen case path. **BACKLOG_NONBLOCKING**, and a C change would cost Build 5 for nothing.

## 7. Build identity

The chain holds. `harness.build_identity` hashes every file on disk after the build, keyed by
basename, with size, SHA-256 and an ELF class read from the program headers. The runner writes it
durably and then calls `driver.verify_build_identity`, which **re-hashes every recorded artefact**
immediately before the first case; a deviation halts with zero cases posed. The driver builds nothing
— no `harness.build`, no `cc`, no `-static` anywhere in it.

**`BUILD_IDENTITY_BOUND_TO_USED_BYTES`** — universally at the boundary.

One correction to the freeze's own wording. It says *"the two setups that open a pristine build
product verify the digest at the point of use"*. There are **ten** such setups, and **17 of 72 cases**
— E4, E7, E8, N3, O6, P1–P4, T5, X2, X2b, X2c, X4, X5, X7, X8 — are not re-verified at their point of
use. Those cases rely on the pre-boundary verification plus the argument that no case mutates a
pristine product. The argument holds today, but it is an argument rather than a check, and the
manifest states the coverage as broader than it is. Recorded as **T2-R3**.

Mutation cases are correctly out of scope for a point-of-use identity check: a case that
intentionally changes its own copy must not be required to retain the base digest, and the setups
that mutate all work on `<CASE>_<binary>` copies rather than the build product.

## 8. Journal

Schema, framing and durability audited directly. `_append` runs **serialise → write → flush →
fsync**, the file is opened for append with `newline="\n"`, and `evidence.serialise_line` escapes CR
and LF inside values, so one record is exactly one physical line. Simulated crashes at each point:

| crash point | records | truncated | completed | entered |
|---|---|---|---|---|
| before write | 3 | False | E1 | E1, E2 |
| **during write (torn tail)** | 3 | **True** | E1 | E1, E2 |
| after write before flush | 3 | False | E1 | E1, E2 |
| after fsync | 3 | False | E1 | E1, E2 |

Replay does not repair, reorder or invent; it refuses a duplicate completion; and the writer refuses
to complete a case twice or complete one never entered. There is no update or delete API. The
single-writer model is enforced by the frozen runner, which is the only caller.

**Trial #1's exact shape, through the real implementation** — five completed, E5b entered, then
`OSError(9)`, closed and replayed:

```
completed              : E1 E2 E3 E4 E5
entered not completed  : E5b
last entered/completed : E5b / E5
E6 onward present      : False
status invented for E5b: False
preflight durable      : True      build identity durable : True
after a torn last line : E1-E5 intact
```

**`JOURNAL_CRASH_SEMANTICS_SOUND`**, with two minor notes below.

## 9. Reconstruction

`journal.recovered_records` returns only completed cases; E5b is absent and no status is invented for
it or for cases never entered. The runner has **no journal-driven recovery or resume path at all** —
`journal.read` and `replay` are never called by it — so there is no route by which a crashed trial
silently continues. Aggregation happens only after the full 72-case loop, so a partial journal cannot
produce an aggregate through the runner, and in-memory state cannot diverge from the journal: each
completed case is journalled with the status the document later carries. What is *not* frozen is
whether a **reader** may aggregate a partial journal — that is finding T2-R2.

## 10. P-14 on the durable files

Attacked through the fields the frozen driver really produces — a broad `uname`, host-identity keys,
a path value, an environment value, `child_pid`, `acquisition`, `raw_trace`,
`capture_prefix_base64`, `spike_stderr`:

| file | leaks |
|---|---|
| `preflight.json` | none |
| `build-identity.json` | none |
| `journal.jsonl` | a bare account name in `spike_stderr` free text |
| `evidence.json` | the same |

Every other probe was caught: the host descriptor became `<WITHHELD:host-descriptor>`, identity keys
`<USER>`, paths `<ABSPATH>`, environment values `<VALUE:len=…>`, and `child_pid`, `acquisition`,
`raw_trace` and the capture prefix were withheld by key. Sanitisation happens **before** the disk
write in every case, so the GitHub upload is not the first privacy boundary.

The residue is V-5 working as designed: a username is redacted as a whole path component or an
explicit host-identity value, never as an arbitrary substring, because substring replacement is what
corrupted `truncated` and `specific`. `spike_stderr` is the one free-text channel that could carry a
bare name, and `launcher_spike.c`'s `die()` prints `launcher_spike: <fixed message>: <strerror>` with
no path and no account name. Recorded as **T2-R7, BACKLOG_NONBLOCKING**.

## 11. Artifact contract

`preflight.json`, `build-identity.json`, `journal.jsonl` and `evidence.json`, frozen in the manifest
with their sanitisation rule, the states after which they are preserved, and what is never persisted.
`final_document` is explicitly *"a summary of durable facts, never their only copy"*, and
`preserved_after` includes a harness exception and a crash — so a missing `evidence.json` after an
abort is anticipated and the other three remain authoritative. Confirmed by running the runner to a
simulated abort: preflight and build identity were on disk, non-empty, before the crash.

## 12. Findings

| ID | Severity | Area | Summary |
|---|---|---|---|
| **T2-R1** | **BLOCKER** | case semantics | `post_pin`, `hold_writer`, `mutated_marker` and `cleanup` are produced by setups and read by nothing; E2, E3, E4, E5, E6, E6b, E6c, E6d never perform the manipulation that defines them. E5's mandatory FAIL would likely force `MECHANISM_REJECTED` from a condition never created. Pre-existing, also true at the Trial #1 freeze |
| **T2-R2** | **IMPORTANT** | preregistration | Crash *interpretation* is not frozen: no trial-level term for a journal without `trial_end`, no rule forbidding aggregation of a partial journal, no frozen status for an entered-not-completed case, no rule on D-7 consumption by an aborted trial |
| **T2-R3** | **IMPORTANT** | build identity | The freeze says two setups verify at point of use; there are ten, and 17 of 72 cases are not re-verified there. Binding at the boundary is universal, so the claim overstates coverage rather than the binding failing |
| T2-R4 | MINOR | journal | `replay()` accepts a `case_completed` with no preceding `case_entered`; the write side refuses it, the read side does not. Reachable only against a tampered file |
| T2-R5 | MINOR | journal | `read()` uses `splitlines()`, which breaks on U+2028, U+2029 and U+0085; one such character in a payload truncates the journal from that point. No frozen field can currently carry one — `errors="replace"` yields U+FFFD |
| T2-C-LATENT | BACKLOG | C | `--bypass-admission` + `--exec-fd-o-path` would `pread` an `O_PATH` descriptor. Independently confirmed unreachable from all 72 plans |
| T2-R6 | BACKLOG | privacy | The build-identity map is keyed by artefact name and P-14 never sanitises keys. Keys are basenames of frozen products taken before the first case, so unreachable |
| T2-R7 | BACKLOG | privacy | A bare account name in free text survives, by V-5 design. `spike_stderr` is the only channel and `die()` emits no name or path |
| T2-R8 | BACKLOG | dispatcher | The Trial #1 dispatcher file remains on the milestone branch. GitHub reports the workflow `state: deleted` and does not list it, so it is not dispatchable, but merging that branch to `main` would recreate a dispatcher naming a consumed authorisation |

## 13. Validation

| | Windows | Linux (Ubuntu 24.04 WSL) |
|---|---|---|
| `unittest discover -s tools/tests` | **Ran 449, OK** (1 skipped) | **Ran 449, OK** (0 skipped) |
| E5b access sequence | AST only | **exercised on inert files** |

`git diff --check` clean · `validate_docs` PASS · `cargo check --workspace --all-targets` exit 0 ·
freeze verification `true` · driver completeness 72/72/0 · runner default exit 3.

**Trial #2 experimental ELF executions ZERO · cases posed ZERO · valid trial count ZERO.**

## 14. Trial #1

Freeze unamended, result review unmodified, no status manufactured for E1–E5, no aggregate created,
D-7 not reused. It remains historical evidence.

## 15. Classification

**`TRIAL_2_NEEDS_PRETRIAL_FIXES`.**

`E5B_CLAIM_PRESERVED`, `BUILD_IDENTITY_BOUND_TO_USED_BYTES` and `JOURNAL_CRASH_SEMANTICS_SOUND` all
hold, and the preservation architecture does what Trial #1 lacked. What blocks a new D-7 is **T2-R1**,
which would make eight cases report on manipulations that never occur, plus
`TRIAL2_PREREGISTRATION_INCOMPLETE` (**T2-R2**) and the coverage overstatement in **T2-R3**.

T2-R1 is worth stating plainly: it is not a flaw in the Trial #2 work. It was there for Trial #1,
Trial #1 ran four of the affected cases, and only tracing the setup-to-execution binding that this
review was asked to trace surfaced it. Running Trial #2 as frozen would spend the one authorised
execution on a trial whose E-series results could not be believed either way.

**TRIAL_2 D-7 REMAINS NOT AUTHORISED. LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT REMAINS ZERO.**
