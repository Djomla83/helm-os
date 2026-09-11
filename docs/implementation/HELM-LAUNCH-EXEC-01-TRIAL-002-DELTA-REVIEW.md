# LAUNCH-EXEC-01 — Trial #2 bounded independent delta review

Bounded independent delta review of the corrected Trial #2 candidate
`e4f49f2bdb77c78cb57354584bc78f08a50531ea`, conducted read-only against the local review horizon
`f0e77c5ba4e628f9824bde481f7d28f9160cd413`. Three independent reviewer workstreams and one
synthesizer; every BLOCKER and IMPORTANT finding below was re-verified by the synthesizer against
source. Nothing was fixed, no case was posed, no `launcher_spike` or helper ELF ran, no dispatcher
was created and no D-7 was granted.

**Classification: `TRIAL_2_NEEDS_BOUNDED_CORRECTION`.**

The four corrections the review was scoped to (T2-R1 to T2-R4), Build 6 and every preservation
check pass. The same frozen driver nevertheless contains reachable BLOCKERs: mandatory cases that
would PASS without their preregistered condition, and cases whose posed-checks read keys nothing
produces, which guarantee an INCONCLUSIVE aggregate after D-7 is consumed. Every one of them is
byte-identical in the Trial #1 freeze `89c923a` and the first Trial #2 freeze `542a18e`; no earlier
review recorded any of them. This record is preserved as written and is not amended by the
correction that follows it.

## A. Repository state reconstructed

| Item | State |
|---|---|
| HEAD / branch | `f0e77c5` on `docs/helm-launch-architecture`; clean tree |
| Remote milestone tip | `53c00393eb4ff5471042b9c9860f42fc29bab047`; local two commits ahead, unpushed |
| Corrected freeze | `e4f49f2`; lineage `542a18e` → `4454e02` → `07277e7` → `e4f49f2` → `53c0039` → `fbded73` → `f0e77c5` all reachable |
| Trial #1 | closed: freeze `89c923a`, `TRIAL_ABORTED_AFTER_BOUNDARY`, D-7 consumed, no aggregate, E1–E5 `UNKNOWN_FROM_PRESERVED_EVIDENCE`; result review unchanged since `f7e5ca0` |
| Trial #2 | `NOT_RUN`; D-7 NOT granted; valid count ZERO; no `workflow_dispatch` run after `34500901306` |
| Dispatchers | Trial #1's retired from the branch and `main` (blob `135a3edc`, identical at `eb6825f`, `f487ea3`, `e4f49f2`); no Trial #2 dispatcher anywhere |

**Known CI failure, preserved.** Run `34525876630` on `53c0039` failed only at
`validate_docs.py`, on the dead link to the retired dispatcher in `docs/PROJECT_STATE.md`.
Classified `KNOWN_DOCUMENTATION_REGRESSION_NON_MECHANISM`; not rerun. The local commit `f0e77c5`
already turns the link into plain text and `validate_docs.py` passes at HEAD. Optional MINOR
refinement: pin the sentence to its immutable source (`f487ea3`, run `34500901306`).

## B. Bounded gate matrix

| # | Gate | Verdict | Evidence |
|---|---|---|---|
| 1 | T2-R1 post-pin barrier, ten affected cases | **FINDING** | Barrier after pin/fstat/admission/measurement and before `pipe2`/`clone3`/`execveat`; control fd closed before `clone3`, `pass_fds` only; C only synchronises; all six control failures end INVALID; setup-key inventory is exactly the expected eight. But E2/E4/E6 assertions are never evaluated (N-2), E5 cannot be scored (N-3), and the landing proof is not persisted (N-5) |
| 2 | T2-R2 durable boundary, partial journal | **PASS** | `case_pose_started` fsynced before `pose()`; NOT_STARTED / ABORTED / COMPLETED readings match the frozen table; nothing fabricated |
| 3 | T2-R3 72/72 build-identity binding | **PASS** | One `bind_build_identity`, called once; 56 / 5 / 8 / 1 / 1 + N3 not posed = 72; corrected E5b is separate O_RDONLY and O_WRONLY one-byte operations |
| 4 | T2-R4 replay integrity | **PASS** (minor) | All eight impossible histories refused; replay does not re-verify `trial_end` prerequisites, which the frozen writer cannot violate |
| 5 | Build 6 compile-only, zero executions | **PASS** | Run `34525876757`, ubuntu-24.04, head `53c0039`, kernel 6.17.0-1022-azure, gcc 13.3.0, glibc 2.39; hashes match `BUILD-EVIDENCE.md`; no built ELF invoked as a command |
| 6 | Trial #1 and Trial #2 preservation | **PASS** | `e4f49f2..HEAD` touches only the retired workflow, one PROJECT_STATE line and an append-only BUILD-EVIDENCE hunk; 17 + 3 hashes recomputed exact |

## C. New findings

**BLOCKERs producing a false PASS**

* **N-1 — parent states are declared but never applied.** `plan.parent` is read once and
  `_run_once` consumes only `env`. F2's non-CLOEXEC descriptor, F3's CLOEXEC descriptor, F5 and T6's
  blocked and ignored signals, F6's closed stdio, F7's adjacent descriptors and R4/M5's
  `SIGCHLD=SIG_IGN` are never created; `harness.open_non_cloexec_descriptor` is never called and no
  plan has a `posed_when` guard. F2 (mandatory, instant-reject, "the ONLY pass"), F5, F6, F7 and T6
  would PASS without their preregistered condition. The T2-R1 closed schema covered `SETUPS` and not
  `PARENT_STATES`.
* **N-2 — executed-body identity is never asserted.** `rule_process_disposition` and
  `exec_confirmation` accept any report carrying a marker and never compare its value. `helper_alt`
  emits a valid report and exits 0, so a launcher that re-resolved the path and ran the substituted
  body would still PASS E2 and E4. E6's "the MUTATED marker runs", E1's measured digest and E6d/X1's
  recorded mode bits are never compared. E6's `pwrite_marker` writes "original + 1" rather than the
  declared marker.

**BLOCKERs forcing INCONCLUSIVE after D-7 is consumed**

* **N-3 — E5's posed check reads `writer_inode` and `pinned_inode`, which nothing produces.** The
  mandatory case is always INVALID.
* **N-4a — O5's posed check reads `spike_cpu_ms`, which nothing produces.** The mandatory case is
  always INVALID; `poll_returns` is also absent and treated as acceptable.
* **N-4b — S2/S7 `work_dir_kind="fchmod_zero"` is never read.** The helper reports, so
  `no_helper_report` fails and the posed conditional cases are INVALID.

**BLOCKER on the evidence criterion**

* **N-5 — critical evidence dies in the observation.** `post_pin_evidence`,
  `build_identity_binding` and `cleanup_problems` live only in `obs`; `evaluate()`, the journal and
  the evidence document persist only `record`, so preserved evidence cannot show that any forced state
  landed, and definition §9.1 and §9.4 are not met.

**IMPORTANT**

* **N-6 — E6c's `descriptor_closed=True` is false.** CPython's `mmap` duplicates the descriptor.
  The mapping's own file reference keeps the write reference alive either way, so the kernel
  condition is effectively unchanged; downgraded from a reviewer's BLOCKER. The mutation hits ELF
  byte 0.

**MINOR**

* R4's gate reads an `outcome_token` that is never set, so it always passes.
* `SOURCE-HASHES.json` `durable_evidence.build_identity.reuse_proof` keeps the stale T2-R3 wording.
* The manifest lists X8 under `post_pin_barrier.affected_cases`; X8 needed only `cleanup`. The ten
  are the T2-R1 producer-only set; the barrier set is nine.
* Mode-list and comment drift; `elapsed_ms` includes the handshake.

## D. Previously known nonblocking items

Replay does not re-verify the `trial_end` prerequisites; the aggregate input digest covers
unpersisted raw records; the `case_pose_started` wording says every prerequisite has landed while
post-pin state lands after it; the reading table lacks rows for a halt `trial_end` and a refused
journal; the runner does not refuse an existing journal; the build directory is not required to be
empty; setup exceptions abort rather than mark not-posed; no directory fsync. None is reachable on the
frozen writer path.

## E. Executed during the review

Read-only git and `gh` commands; `validate_docs.py`; `run_launch_exec_01.py --verify-freeze` and
`--driver-completeness` after confirming from source that both return before any `Authorisation`;
scratchpad scripts hashing git blobs, parsing `driver.py` by AST, and feeding synthetic JSONL to
`journal.py` alone. **No preregistered case, no `launcher_spike` and no experimental ELF was
executed.** Nothing was compiled, modified, committed or pushed.

## F. Classification

**`TRIAL_2_NEEDS_BOUNDED_CORRECTION`.** The correction is bounded: extend the closed-schema and
consumer-proof discipline from `SETUPS` to parent states, posed-check inputs and semantic plan fields;
add per-case executed-identity assertions; persist the normalised landing evidence in the durable
record; then a new freeze and a fresh bounded independent review.
