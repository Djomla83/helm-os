# LAUNCH-EXEC-01 — Trial #2 bounded independent review of freeze `f417984`

Bounded independent review of the delta-corrected Trial #2 candidate
`f41798455662579c3895f7a4af50bac17d47bb43`, which corrects N-1 to N-5 of the failed delta review
[`43371c6`](HELM-LAUNCH-EXEC-01-TRIAL-002-DELTA-REVIEW.md) in commit `5e04642`. Review horizon:
local `2591f35ce9832a94e7c868cfcf6fe2369dc16168`, used only as post-freeze Build 7 evidence. The
reviewer did not implement the correction. Nothing was fixed, no case was posed, no `launcher_spike`
or helper ELF ran, no dispatcher was created, no D-7 was granted and nothing was pushed.

**Classification: `TRIAL_2_NEEDS_PRETRIAL_FIXES`.**

The correction does what it says for creating and proving parent state, for executed-body
assertions, for E5's writer relation, for O5's measurements and for durable posing evidence. The
freeze, Build 7 and every preservation check pass. Four reachable BLOCKERs remain in how the frozen
driver classifies what a posed case showed: a repeated-trial aggregation introduced by this
correction turns a first-trial mismatch into a PASS, and three posed checks absorb their own case's
preregistered FAIL condition as INVALID (O8, S2/S7, F7).

## A. Review target and boundary

| Item | Verified state |
|---|---|
| Branch / HEAD | `docs/helm-launch-architecture`, clean; HEAD `2591f35` |
| Remote milestone tip | `f41798455662579c3895f7a4af50bac17d47bb43` (`git ls-remote`); local is one commit ahead |
| Sequence | `43371c6` (failed review) → `5e04642` (correction) → `f417984` (freeze) → `2591f35` (Build 7 record) |
| `43371c6` | the only commit that ever touched `HELM-LAUNCH-EXEC-01-TRIAL-002-DELTA-REVIEW.md`; unchanged since |
| `5e04642` | 11 files: definition, BUILD-EVIDENCE, README, SOURCE-HASHES, driver, evidence, frozen_cases, launcher_spike.c, observations, two test files |
| `f417984` | `PROJECT_STATE.md` (one section added) and `SOURCE-HASHES.json` only |
| `2591f35` | `BUILD-EVIDENCE.md` only, 61 insertions, 0 deletions — append-only |
| `f417984..HEAD` | exactly `BUILD-EVIDENCE.md`; no hashed source or definition changed after the freeze |
| Trial #1 | freeze `89c923a` is an ancestor of HEAD, tree `01d090952348af25845a7880166f22557fa393d6`; result review untouched since `f7e5ca0`; still `TRIAL_ABORTED_AFTER_BOUNDARY`, `AGGREGATE_NOT_DERIVABLE_FROM_FROZEN_EVIDENCE`, `D7_AUTHORIZATION_CONSUMED`; no line added by `5e04642` or `f417984` reconstructs an E1–E5 status |
| Dispatch | the only `launch-exec-01` `workflow_dispatch` run is Trial #1's `34500901306`; no Trial #2 dispatcher exists in `.github/workflows` |

## B. Freeze integrity

Recomputed from the git blobs at `f417984`, independently of the repository's own verifier:

| Check | Result |
|---|---|
| Source hashes | **17 / 17** exact (blobs at `f417984`, at HEAD and in the working tree) |
| Definition hashes | **3 / 3** exact |
| `SOURCE-HASHES.json` | blob `4a7dfbae013afd5876ffe309bd8aed2dcd761efd`, SHA-256 `dc004c470e96611cec31eca7462659cbcd16f0afffc5f7139eecd6af7cdec2c8`; byte-identical at HEAD |
| Rehashed vs `e4f49f2` | README, driver, evidence, frozen_cases, launcher_spike.c, observations, and the definition — matching the freeze commit's own list |
| Partition | 72 total, 54 mandatory, 11 conditional, 7 recorded |
| Traced | 8: E1 E7 F4 F7 M1 M2 M3 M4 |
| Driver | 72 handlers, complete, 72 posable, 0 unposable (`--driver-completeness`); `--verify-freeze` true |
| Status | `NOT_RUN`, `d7_execution_authorised: false`, valid Trial #2 count ZERO |

Loading `frozen_cases.py` from `e4f49f2` and `f417984` side by side: every field of every case —
`blocked_if`, `cls`, `gates`, `instant_reject`, `predict`, `safe`, `series`, `traced` — is
identical, as are membership, the class lists, block reasons, documentation gates, `REPEAT_TRIALS`,
`STAGES` and the child injection modes. The only change is three added `PARENT_CONTROL_MODES`
descriptions for `--post-pin-control-fd`, `--parent-fd-set-cloexec` and `--parent-close-low-fds`.
**Nothing preregistered moved.**

## C. N-1 — parent state: `PARENT_STATE_BINDING_DEFECT`

Every state is created for one launcher and proven in that launcher's own `/proc` entries by the
harness at the post-pin barrier, before `clone3`, except M2's, which the receipt proves. Traced
through `AppliedParentState` (driver.py 1537–1641), `_run_with_post_pin` (2940–3017) and
launcher_spike.c `main` (591–830). The creation path was also run live on Linux (WSL, Python 3.12.3)
with the frozen `AppliedParentState` and the exact Popen keyword set, using `grep`/`cat` as the
child, through **both** the vfork and the fork branches of `_posixsubprocess`.

| Case | Created by / in / when | Survives Python spawn? | Proven by | Result |
|---|---|---|---|---|
| V1 | `env=` of Popen; launcher's initial environ | explicit `env` | names in `/proc/<launcher>/environ`, values never kept | **SOUND** |
| F2 | harness `open` + `set_inheritable`, `pass_fds`; same number in launcher | yes: live, the fd arrived with flags `0o100000`, no `O_CLOEXEC`, beside the capture pipes and the control socket | `/proc/<launcher>/fdinfo/<n>`, matched by inode | **SOUND** |
| F3 | as F2, then `--parent-fd-set-cloexec` at the top of `main`, before the M5 arm and the pin | as F2 | `fdinfo` `O_CLOEXEC` bit | **SOUND** |
| F5 / T6 | harness `pthread_sigmask` + `SIG_IGN` inside `spawning()` around Popen only, `restore_signals` off | yes: vfork restores the caller's blocked mask in the child and resets only handled dispositions; fork inherits; restoration runs after Popen returns, i.e. after the launcher has exec'd | `SigBlk` / `SigIgn` of `/proc/<launcher>/status` | **SOUND** |
| R4 / M5 | `SIGCHLD` `SIG_IGN`, same path | yes (live) | `SigIgn` | **SOUND** |
| F6 | `--parent-close-low-fds 3` at the top of `main` | n/a | fd 0 is the pinned inode, fd 1 the capability, fd 2 free | **SOUND** |
| F7 | `--parent-close-low-fds 1` | n/a | barrier: fd 0 is the pinned inode; posed check on the child's `close_range` spans | **DEFECT on the failure path — F417-B4** |
| M2 | plan's `--extra-threads 3`, started immediately before `clone3`, alive until after the receipt | n/a | receipt `parent_shape` 3 + atfork, control arm 0 | **SOUND** (F417-N1) |

The launcher changes no signal state and opens nothing before the barrier except its exec object and
capability, so the barrier observation is the state at `clone3`. Harness state is restored
unconditionally in `finally`; live, mask and dispositions were identical after every spawn. The
inherited descriptor is closed by `release()` after the launcher returns.

**Control flags.** Only `AppliedParentState` emits them: F3 from `inherit_fd.cloexec`, F6 and F7
from `close_low_fds`. No plan's own `spike_flags` carries one, and the M2 control arm's `flags=()`
does not affect them. They are placed before the first `--arg`, and every `--arg` consumes its value,
so no helper argument can become a flag nor a flag a helper argument. Both run once, before the M5
arm, the pin and the barrier observation that proves them; `--parent-fd-set-cloexec` refuses fd < 3
and `--parent-close-low-fds` refuses K > 3.

**F6 equivalence, traced descriptor by descriptor.** Before the close: 0, 1 (stdout pipe), 2
(stderr pipe) and the control socket `c ≥ 3`. Fd 1 is saved as `r ≥ 3` with `FD_CLOEXEC`; 0–2 are
closed; the exec object opens as 0 and the capability as 1 — proven at the barrier. The control
socket closes; the first `pipe2` end lands on 2 and the rest fill ≥ 3; `ST_RELOCATE` moves exec 0,
capability 1 and pipe end 2 above 2 and closes the originals; the pidfd then takes 0. The child
`dup2`s three descriptors above 2 onto 0–2 (old ≠ new, so the no-op cannot arise), clears
`FD_CLOEXEC` explicitly, and `close_range` removes `r`. This is the frozen construction — the
launcher's own exec descriptor, capability and a pipe end on 0–2 — plus one extra `CLOEXEC`
descriptor above 2 that the child closes. A harness-side close is impossible: the receipt channel
is the launcher's stdout.

**F7 adjacency.** The control socket closes before `pipe2`, so the pipes fill every hole and the
exec descriptor relocates to `status_w + 1` unless an inherited descriptor sits there, in which case
adjacency is not observed and the case is honestly not posed. On the success path the proof is
sound: the uncovered set above 2 must be exactly `{exec_fd, exec_fd ± 1}`, with a terminal span to
`UINT_MAX` and every return observed. The frozen child never closes either preserved descriptor, so
that set is the pair. On the failure path it is not sound — see F417-B4.

## D. N-2 — executed body: `EXECUTED_BODY_ASSERTIONS_SOUND`

Posing and showing are separated as preregistered. Fabricated observations through the frozen
`evaluate` and `checker.score_case`:

| Observation | E1–E4 | E6 | E6d / X1 |
|---|---|---|---|
| declared marker / pre-change mode | PASS | PASS (`MUTATED`) | PASS |
| `helper_alt` marker, or unrecognised text | **FAIL** `executed_body_mismatch` | FAIL | — |
| report truncated, or present without a marker | INVALID | INVALID | — |
| starting digest or pre-exec measurement missing | INVALID | INVALID | — |
| starting digest differs from the measurement | **FAIL** `measurement_mismatch` | FAIL | — |
| recorded mode = mode after the change, or unrelated | — | — | **FAIL** `mode_measurement_mismatch` |
| mode missing from the receipt, or no landed fact | — | — | INVALID |
| wrong marker and a wrong exit together | FAIL | — | — |

An unobservable assertion suppresses only a token that would otherwise PASS; a non-matching token
still FAILs. No violation token is any case's expectation. `helper_alt` emits the sentinel and a
`{"marker":"helper_alt"}` report, so E2/E4 substitution reaches the assertion. E3's identity rests on
the report and on a digest taken at binding time, before the unlink, so losing the path loses
nothing. E4 binds the symlink's `realpath` and the retarget never touches the base. On an inert
synthetic file under Linux, `pwrite_marker` changed exactly the 16-byte guarded region to
`MUTATED\0…` and no adjacent byte. The receipt's `pre_exec_mode_bits` come from the `fstat` before
the barrier. The harness's `mode_before` is read at the barrier, before `chmod`.

## E. N-3 — E5: `E5_WRITER_RELATION_SOUND`

The writer is `os.open(path, O_WRONLY)` opened at the barrier — after the pin, the measurement and
admission. Its `(st_dev, st_ino)` comes from `fstat` on the writer itself. It is compared with the
inode behind each of the launcher's live `/proc/<launcher>/fd/*` entries, and the case is posed only
if the launcher holds a read-only descriptor on it. At E5 that can only be the pinned exec
descriptor: the capability is a directory and E5 inherits nothing. A reopened pathname never enters
the comparison. The durable fact is the relation and access modes, never the pair. The writer is
held in `_open_writers` until `release_setup_resources` after the launcher returns. A failed open,
`fstat` or relation closes it and leaves the case not posed. The live stand-in tests for both
branches pass on Linux.

**E5b regression: none.** `_setup_writer_open_closed` is byte-identical to `e4f49f2`: `O_RDONLY`,
`pread` exactly one byte, close; `O_WRONLY`, `pwrite` that byte, close; launch. There is no read
through the writer and no `O_RDWR`, and a short transfer returns `not_posed`.

## F. N-4a — O5: `O5_BOUNDS_SOUND`

CPU is `getrusage(RUSAGE_CHILDREN)` user+system, in seconds × 1000, taken before Popen and after
`communicate` has reaped the launcher. O5 has no barrier, and the harness reaps nothing else in that
window. Live on Linux, a child burning 0.30 s CPU measured 315 ms. A child sleeping 0.60 s measured
11 ms, its interpreter start-up: this is CPU, not wall time. A child that waited for a grandchild
burning 0.30 s measured 333 ms, so the reaped helper is included. Its only effect is to make the gate
stricter. The poll counter increments after every drain `poll()` return, including the `EINTR` and
timeout `continue` paths. The drain loop has no other busy path, the counter is a `long`, and it
reaches the observation straight from the parsed stdout. Boundaries through the frozen evaluator:
polls 9999 and 10000 PASS, 10001 FAIL; CPU 199 PASS, 200 and 201 FAIL; either measurement absent
INVALID. The inequalities are the frozen ones: CPU `< 200`, polls `≤ 10000`.

## G. N-4b — S2/S7: `S2_S7_FORCED_STATE_DEFECT`

The construction is sound. Each case gets a case-private `<CASE>_workdir` passed as `--work-dir`,
and the launcher opens it before the barrier. The harness proves a launcher descriptor on that inode
from `/proc` and only then sets mode `0000`. Without that descriptor there is no `chmod`: the case is
not posed and the mode stays `0755`. `restore_after_trial` returns the mode to `0755` after every
repetition, and `release_setup_resources` removes the directory. Every repetition carries its own
landing fact, is posed per trial and has its token derived per trial. Root remains BLOCKED on
`euid_zero`.

Two reachable classification defects remain. **F417-B1:** a mismatch in trial 0 disappears when a
later trial carries the prediction. **F417-B3:** a landed trial whose image nonetheless ran scores
INVALID. The frozen S7 row says "any trial reporting exec success … is a FAIL".

## H. N-5 — durable posing evidence: `POSING_EVIDENCE_SOUND`

`posing_evidence` carries the binding, every landing fact (public keys only, indexed per repetition),
the posed check and its result, the measurements the case's checks and assertions read, and the
cleanup problems. It is present on PASS, FAIL and post-setup INVALID records. An E4 record passed
through the **public** sanitiser, an fsynced journal and replay came back identical. Digests and
basenames survived; no `/home`, temporary path, `st_ino`, `st_dev` or `pid` appeared in the raw
journal. `repr(OSError)` carries no filename, and `_problem` keeps only the errno name. The record
faithfully carries what the driver decided — including the false "did not materialise" of F417-B2
to B4. The key is omitted on BLOCKED and pre-setup not-posed records (F417-M1); the blocked cause or
`not_posed` reason is still recorded.

## I. Related corrections

* **E5b** — no regression (section E).
* **E6c** — `MAP_SHARED`, `PROT_READ|PROT_WRITE` through libc on a case-private `O_RDWR`
  descriptor, which is closed at once. The harness then proves from `/proc/self/fd` that no
  descriptor of its own refers to the inode, and from `/proc/self/maps` that a shared writable
  mapping remains. The write lands in the guarded marker region, not the ELF header. The mapping is
  released deterministically after the launcher returns. `body_length_changed` still maps a
  successful exec to `Exited:mutated`. Recorded semantics, safe set and gate are unchanged.
  (F417-N3.)
* **R4** — `never_reports_exited_zero` is computed in `evaluate` from the derived token.
  `Exited:42` and `ExitStatusUnobservable` PASS, `Exited:0` FAILs, and an absent token does not hold
  the gate. Recorded class, safe set and gates are unchanged.

## J. O8: `O8_SHORT_TRIAL_CLASSIFICATION_DEFECT`

The frozen row reads: "POLLIN and POLLHUP in the same poll() return, 200 trials; any trial short of
512 bytes is a FAIL". The plan's posed check `trial_floor_512` (driver.py 1855–1869, byte-identical
since Trial #1's `89c923a`) runs before the rule and returns false for any trial whose
`bytes_drained < 512`. The case is then recorded `not_posed` — "the forced state did not
materialise" — and scored INVALID. Fabricated, with every harness fact present: all 200 trials at
512 bytes PASS, while one posed trial at 400 bytes or 0 bytes scores **INVALID**. The short drain is
exactly the mechanism failure O8 exists to catch. For a mandatory case this turns
`MECHANISM_REJECTED` into `MECHANISM_INCONCLUSIVE`. **BLOCKER (F417-B2).**

## K. Build 7

| Item | Verified |
|---|---|
| Run | `34575558065`, attempt 1, event `push`, conclusion **success**, every step success |
| SHA binding | `headSha` `f41798455662579c3895f7a4af50bac17d47bb43`; the checkout step printed it twice; `--verify-freeze` then reported `freeze_verified: true` against that commit's manifest, which hashes `launcher_spike.c` as `0a45447b…`, so the changed source is the one compiled |
| Commands | `cc -O2 -Wall -Wextra -static` for the four static helpers; `… -static -pthread` for `launcher_spike`; `cc -O2 -Wall -Wextra` for `helper_dynamic` — the frozen commands |
| Diagnostics | zero compiler warnings or errors; the only "error" strings are two `ldd --version \| head` broken-pipe lines in the inventory; plus a git hint and a Node 20 deprecation annotation |
| Identities | ten SHA-256 values printed by `sha256sum build/*`, transcribed exactly in `2591f35`; `launcher_spike` `4d42212f3b4d45ca46e415961e36e8ee4d45090a15fb3b77a6f8971e661a6546` (Build 6 `9e9e5803…`); the other nine each already appear in the pre-Build-7 record |
| Static / dynamic | `file` and `readelf`: the five static targets show no PT_INTERP, `helper_dynamic` has one, and the assertion step passed; the E6 marker guard is present |
| Non-trial tests | `Ran 562 tests`, `OK`, none skipped |
| Executions | no `set -x` line runs a `build/` path or any produced ELF; `--post-pin-control-fd`, `--parent-fd-set-cloexec`, `--parent-close-low-fds` and the D-7 flag appear **zero** times; the runner, invoked without D-7, exited 3 `NOT_RUN` |
| Rust workspace | `34575558173`, attempt 1, **success**, same SHA |
| Size gap | **NONBLOCKING (F417-N5)**: no build record 1–7 carries binary sizes and no contract requires them for compile-only evidence; Trial #2's own build identity records size + SHA-256 of the exact binaries before case 1. `2591f35` marks every binary size "not emitted" and invents none, and calls the artefacts compile-only, not Trial #2 build identity |

Experimental ELF executions in Build 7: **ZERO**.

## L. Preregistration: `TRIAL2_F417_PREREGISTRATION_INCOMPLETE`

Definition §9.5 and the manifest preregister the parent-state contract and its table, posing versus
showing, the four assertions and their violation tokens, E5's relation, O5's sources and bounds,
S2/S7's construction and repetition rule, durable `posing_evidence`, both TEST/CONTROL flags and the
derived barrier set (20) apart from the historical T2-R1 set (10). What is not fixed is which
frozen statement governs when they disagree about the same observation:

* O8 — §3 says a short trial is a FAIL; the plan's posed check, under §9.5's definition of "posed",
  makes it INVALID.
* S2/S7 — §3 says a trial reporting exec success is a FAIL and "no helper report may be received";
  the `no_helper_report` posed check makes it INVALID. §9.5 says "a trial whose token differs is a
  FAIL", and the implementation does not do that when the odd trial is the first (F417-B1).
* F7 — §3 names an inverted-gap `EINVAL` as the failure; the §9.5 posed check makes it INVALID.
* §9.5 says every `case_completed` record carries `posing_evidence`, but BLOCKED records do not
  (F417-M1).

An owner would have to choose between these after the observation, which is what preregistration
exists to prevent.

## M. Findings

| ID | Severity | Area | Current reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| F417-B1 | **BLOCKER** | repeated trials (O8, S7) | yes — fabricated through the frozen evaluator; **introduced by `5e04642`**, absent from `e4f49f2` and `89c923a` | driver.py 2460–2464 replaces the case token with the first LATER trial that differs from trial 0. When trial 0 is the odd one out, that later token is the prediction and the case PASSes: S7 with trial 0 `ExecStatusIndeterminate` and 199 correct trials → PASS; O8 with trial 0 at 512 bytes but a wrong digest, or at 600 bytes, and 199 correct → PASS. This is a false PASS on a mandatory case, contrary to §9.5. The suite only tests a later odd trial | pre-trial fix: any trial whose token is not the frozen expectation must decide the case; add first-, middle- and last-trial tests |
| F417-B2 | **BLOCKER** | O8 | yes; present since `89c923a` | `trial_floor_512` scores a posed short trial INVALID; the frozen row says FAIL (section J) | pre-trial fix: a short drain on a posed trial is a showing, not a posing failure |
| F417-B3 | **BLOCKER** | S2, S7 | yes; the check predates the correction, but N-4b made the forced state provable at the barrier | with the barrier fact landed, a trial whose image ran (report present, `Exited:0`) is "not posed" → INVALID; frozen S2/S7 require FAIL. `test_a_reported_image_in_any_trial_is_not_posed` enshrines it | pre-trial fix, or an explicit owner amendment of the S2/S7 rows before D-7 |
| F417-B4 | **BLOCKER** | F7, parent-state truth | yes; the posed check is **new in `5e04642`** | `layout_is_adjacent` (observations.py 780–803) returns None whenever any `close_range` return is not an observed success. F7's named failure — an inverted-gap `close_range` returning `EINVAL` — is therefore recorded as "the forced state did not materialise" → INVALID. Yet the barrier had proven the parent state, and the inverted span `(a+1, b-1)` itself proves the pair adjacent. The durable record then states a false parent-state fact | pre-trial fix: derive adjacency from an inverted span too, and score the observed `EINVAL` as FAIL (see F417-I1) |
| F417-I1 | IMPORTANT | report-based rules (A, V, F, N, X2c) | yes; present since `89c923a` | `_usable_report` returns None without a report, so a posed case whose child demonstrably failed before exec scores INVALID, not FAIL — an explicit `ExecFailed` status record is positive evidence, not missing evidence. F7 stays INVALID after F417-B4 for this reason. F6's fd-1 `FD_CLOEXEC` failure also scores INVALID: the kernel closes fd 1 at exec, so no report can arrive | owner decision on the rule, then fix with F417-B4 |
| F417-I2 | IMPORTANT | M2 control arm | yes; the outcome predates the correction, the path is new | a control arm that does not return, or is not posed, fails `threaded_parent_observed` → INVALID, although a launcher non-return is FAIL in any class | fix with the classification batch |
| F417-I3 | IMPORTANT | preregistration | yes | §3 rows and §9.5/driver disagree for O8, S2/S7 and F7 (section L) | resolve in the definition with the fixes, before a new freeze |
| F417-M1 | MINOR | N-5 | yes | `posing_evidence` is omitted on BLOCKED and pre-setup not-posed records, contrary to "every case_completed record … even when the case is not posed"; truth and privacy unaffected | align the text or always write the key |
| F417-N1 | BACKLOG_NONBLOCKING | M2 | — | the thread proof is the creator's own `parent_shape`: independent of the declaration, not of the creator. M2's own trace could corroborate it. Preregistered as is | backlog |
| F417-N2 | BACKLOG_NONBLOCKING | F5, T6, R4, M5 | observed live | with `restore_signals` off, Python's start-up `SIGXFSZ` (and `SIGPIPE` for R4/M5) also reach the launcher ignored; this is undeclared, but it only makes the child's reset stricter, and the proofs test inclusion | backlog |
| F417-N3 | BACKLOG_NONBLOCKING | E6c | no practical path | `_shared_mapping_of` matches the maps inode column without the device | backlog |
| F417-N4 | BACKLOG_NONBLOCKING | O5 | no practical path | cumulative CPU is rounded to integer ms before the difference, ±1 ms at the edge, finer than kernel accounting | backlog |
| F417-N5 | BACKLOG_NONBLOCKING | Build 7 | — | compile-only binary sizes were not emitted; no contract requires them | none |
| F417-N6 | BACKLOG_NONBLOCKING | provenance | — | `43371c6`, `5e04642`, `f417984` and `2591f35` carry an agent `Co-Authored-By` trailer, contrary to the AGENTS.md commit-authorship rule. History is not rewritten; this record's commit carries none | keep out of future commits |

No previously accepted backlog item is reopened.

## N. Executed during the review

Read-only `git` and `gh` inspection. A scratchpad script recomputed every hash from git blobs. The
same script loaded `frozen_cases.py` from both freezes side by side. The runner's
`--verify-freeze` and `--driver-completeness` modes were run; both return before any
`Authorisation` exists. `validate_docs.py` and `git diff --check` were run. The Python suite ran on
Windows: 562 tests, OK, 26 Linux-only tests skipped. It ran on Linux (WSL, Python 3.12.3): 562
tests, OK, none skipped. `cargo fmt --check` passed.

`cargo test --workspace --locked` on Windows failed two `helm-evidence` `review_regressions` tests
with an OS path-not-found error. No Rust or Cargo file changed since `e4f49f2`, and the Linux run
`34575558173` on `f417984` is green, so this is environmental. Two scratchpad probes ran as well.
The first fed fabricated observations to the frozen `evaluate` and checker. The second, Linux-only,
spawned `grep`, `cat` and the Python interpreter through the frozen `AppliedParentState` and Popen
arguments, and measured `RUSAGE_CHILDREN`.

**No `launcher_spike`, no helper, no generated ELF and no preregistered case was executed; no
`Authorisation` was constructed; the D-7 flag was never passed.** Trial #2 ELF executions: ZERO.
Cases posed: ZERO. Valid trial count: ZERO.

## O. Authority and classification

TRIAL #2 D-7 REMAINS NOT AUTHORISED

LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT REMAINS ZERO

TRIAL #2 F417984 IS NOT READY FOR D-7

**`TRIAL_2_NEEDS_PRETRIAL_FIXES`.** The fixes are bounded to classification. First, repeated-trial
aggregation. Second, the posing-versus-showing boundary of three posed checks and of report-based
rules when the child demonstrably failed pre-exec. Third, M2's control-arm non-return. Then align
definition §3 and §9.5, freeze again, record fresh Linux compile-only evidence only if C changes,
and run one more bounded independent review. Trial #1, its freeze and its result review, and the
delta review `43371c6`, are preserved unchanged. This record is not amended by the correction that
follows it.
