# HELM-LAUNCH P4 — FRESH INDEPENDENT LIFECYCLE / RECEIPT REVIEW

> # 🔎 FRESH INDEPENDENT P4 LIFECYCLE / RECEIPT REVIEW
>
> **Session provenance. THIS REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `41ac4f90e85da0688c9e76cdeec92ba904fcfe1d` (the P4 authority commit) and
> `3d152ad0b7a422eb04160ba70697a457efd390c5` (the P4 implementation candidate).
> It authored no part of the P1, P2 or P3 implementation chain and no part of any earlier
> helm-launch review. It read both commits under review, re-derived the accepted contract from
> [ADR-0024](../adr/ADR-0024-launch-authority.md) and the
> [productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md), and re-established every behavioural
> claim below from the product source, the test source, the manifests and the workflow definition.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **P1 ACCEPTED. P2 ACCEPTED. P3 ACCEPTED. P4 AUTHORISED — IMPLEMENTED CANDIDATE, NOT ACCEPTED.
> P5 NOT AUTHORISED. THE COMPLETE helm-launch 0.1 MODULE IS NOT YET PRODUCT-ACCEPTED.
> TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED` AND MUST NOT BE RERUN. TRIAL #4 IS NOT AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **7** |
| `MINOR` | 8 |
| `BACKLOG_NONBLOCKING` | 1 |
| `GATE_PENDING` | 1 |
| `P4_AUTHORITY_AND_COMMIT_SCOPE` | **SOUND** |
| `P4_PUBLIC_LAUNCH_BOUNDARY` | **SOUND** |
| `P4_ERR_VS_RECEIPT_BOUNDARY` | **SOUND** |
| `P4_SPAWN_RESTORE_TRANSITION` | **SOUND** (`P4A-01` **NOT A FINDING**) |
| `P4_REAL_LOOP_MATCHES_PURE_MODEL` | **NOT SOUND** — the real loop's total bound and the model's disagree (`F-P4-02`) |
| `P4_POLL_PROGRESS` | **SOUND** (`P4A-05` defect independently proven absent; **no regression guard exists**) |
| `P4_DEADLINE_ACCOUNTING` | **NOT SOUND** — an unbounded drain can starve every deadline (`F-P4-01`) |
| `P4_EINTR_BOUND` | **SOUND** for `poll`, `waitid` and the signal path; **not bounded** inside `drain` (`F-P4-01`) |
| `P4_READ_BEFORE_HANGUP` | **SOUND** |
| `P4_STREAM_FAIRNESS` | **NOT SOUND** (`F-P4-01`) |
| `P4_STREAM_ACCOUNTING` | **SOUND** |
| `P4_PHASE_A` | **SOUND** |
| `P4_NO_EXEC_SUCCESS_CLAIM` | **SOUND** |
| `P4_PHASE_B` | **SOUND** |
| `P4_TIMEOUT_GRACE_KILL` | **SOUND** in the lifecycle; one **unrecorded** extra direct `SIGKILL` in `Drop` (`F-P4-02`) |
| `P4_END_NOT_OBSERVED_LATCH` | **SOUND** |
| `P4_TOTAL_BOUND` | **NOT SOUND** — `P4A-02` **CONFIRMED, RECLASSIFIED `IMPORTANT`** (`F-P4-02`) |
| `P4_SWEEP_BEFORE_REAP` | **SOUND** |
| `P4_GROUP_AUTHORITY_GUARD` | **SOUND** |
| `P4_GROUP_SWEEP_TARGET` | **SOUND** (one defence-in-depth `MINOR`) |
| `P4_FOREIGN_REAPER_GUARD` | **SOUND** by construction; **untested in the real loop** (`F-P4-06`) |
| `P4_EXACTLY_ONE_GROUP_SWEEP` | **SOUND** |
| `P4_REAL_GROUP_SWEEP_TEST_PROBATIVE` | **NOT PROBATIVE** — `P4A-04` **CONFIRMED, RECLASSIFIED `IMPORTANT`** (`F-P4-05`) |
| `P4_GROUP_SWEEP_TEST_ISOLATION` | **SOUND** |
| `P4_NO_CONTAINMENT_CLAIM` | **SOUND** |
| `P4_REAP_CLASSIFICATION` | **SOUND** |
| `P4_CHILDHANDLE_DROP_INTEGRATION` | **NOT SOUND** on the `EndNotObserved` path (`F-P4-02`) |
| `P4_POST_EXIT_DRAIN_BOUND` | **SOUND** |
| `P4_STREAM_TESTS_PROBATIVE` | **PARTIAL** (`F-P4-07`) |
| `P4_END_NOT_OBSERVED_REAL_EVIDENCE_STATUS` | **MODEL ONLY** — `MINOR`, no safe deterministic seam exists |
| `P4_FOREIGN_REAP_REAL_EVIDENCE_STATUS` | **MODEL ONLY** — `IMPORTANT`, an accepted deterministic seam exists and was not used |
| `P4_TOTAL_BOUND_TEST_PROBATIVE` | **NOT PROBATIVE** — the test asserts the widened implementation constant (`F-P4-02`) |
| `P4_SINGLE_RECEIPT_MODEL` | **SOUND** |
| `P4_RECEIPT_DETERMINISM` | **SOUND** for the serializer; the *test* of it is unsound (`F-P4-04`) |
| `P4_RECEIPT_DIGEST` | **SOUND** |
| `P4_RECEIPT_SIZE_BOUND` | **SOUND** |
| `P4_RECEIPT_PRIVACY` | **SOUND** in the product; the canary *test* cannot pass (`F-P4-03`) |
| `P4_RECEIPT_HOST_PRIVACY` | **SOUND** |
| `P4_NO_AUTHENTICITY_CLAIM` | **SOUND** |
| `P4_OUTCOME_OWNS_NO_DESCRIPTORS` | **SOUND** |
| `P4_OUTCOME_DEBUG_PRIVACY` | **SOUND** |
| `P4_PUBLIC_API_NEGATIVE_PROOFS` | **SOUND** |
| `P4_PUBLIC_ERROR_SURFACE` | **SOUND** |
| `P4_DEPENDENCY_GRAPH` | **SOUND** |
| `P4_UNSAFE_BOUNDARY` | **SOUND** — zero new `unsafe`, zero `unsafe` in `src/launch.rs` |
| `P4_P3_CHILD_CONTRACT_PRESERVED` | **SOUND** — `child.rs` and `syscall.rs` byte-unchanged |
| `P4_P3_REGRESSION_GATES_PRESERVED` | **PRESERVED** |
| `P4_SYNCHRONOUS_RETURN_BOUNDARY` | **SOUND** |
| `P4_TEST_EVIDENCE_SEPARATION` | **SOUND** |
| `P4_TEST_HELPER_CLEANUP` | **SOUND** |
| `P4_DOCUMENTATION` | **SOUND** |
| `P4_CI_DESIGN` | **SOUND** in shape; two committed P4 cases would red it (`F-P4-03`, `F-P4-04`) |
| P4 Linux runtime validation | **`GATE_PENDING`** — no supported Linux execution occurred in this review |

**Classification: `HELM_LAUNCH_P4_INDEPENDENT_REVIEW_NEEDS_FIX`.**

**The P4 candidate must not be published for hosted Linux validation as it stands.** Two of its own
committed Linux-only cases cannot pass — one deterministically — so the load-bearing hosted gate
would be red before any product question was reached. Separately, one reachable product path can
wait without bound, and one reachable product path exceeds the accepted total bound by a fixed
extra `POST_KILL_REAP_MS`.

## 1. Target and starting state

| Item | Value |
|---|---|
| Accepted P3 base | `ef50e8865a4f14965a115c5dc26c72e45d4af2c9` |
| P4 authority commit | `41ac4f90e85da0688c9e76cdeec92ba904fcfe1d` |
| P4 implementation candidate | `3d152ad0b7a422eb04160ba70697a457efd390c5` |
| Branch | `docs/helm-launch-architecture` |
| Local `HEAD` before this review | `3d152ad0b7a422eb04160ba70697a457efd390c5` |
| `origin/docs/helm-launch-architecture` | `ef50e8865a4f14965a115c5dc26c72e45d4af2c9` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Relation to the remote branch | **2 ahead, 0 behind** |
| Worktree before this review | **clean** |
| Fetch | **read-only**; nothing pushed, nothing rewritten |
| Review host | Windows 11, `cargo` 1.95.0; Linux execution **not** available |

Every expected value in the instruction was matched exactly before any file was read.

## 2. Authority and commit scope

### 2.1 `41ac4f9` — the authority commit

Documentation only: [`DECISIONS.md`](../DECISIONS.md), [`PROJECT_STATE.md`](../PROJECT_STATE.md),
[ADR-0024](../adr/ADR-0024-launch-authority.md) and the
[productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md). No crate, manifest, workflow, test or
tool file is touched.

| Requirement | Result |
|---|---|
| Precedes the P4 code | **YES** — `41ac4f9` is the parent of `3d152ad` |
| Authorises P4 only | **YES** — [`HELM_LAUNCH_P4_AUTHORISED`](../DECISIONS.md#helm-launch-p4-authorised) |
| Leaves P5 unauthorised | **YES**, stated in all four documents |
| Leaves Trial #4 unauthorised | **YES**; Trial #3 stays frozen `MECHANISM_REJECTED` |
| Alters accepted P1/P2/P3 semantics | **NO** — the ADR delta is confined to the *Implementation authority* header paragraph; sections A–N are untouched |
| Alters the plan's accepted contract | **NO** — one new status note (§16, `p4-authority-2026-09-19`) and one status line in the header block |
| Weakens any P4 rule | **NO** — §3 of the decision *restates* the `Err`/receipt boundary, the no-exec-success rule, the `EndNotObserved` latch, the guarded sweep, the non-containment non-claim and the pidfd-only identity rule |

### 2.2 `3d152ad` — the implementation candidate

Fourteen crate files, one workflow and one Python confinement suite. No unrelated backlog cleanup,
no drive-by refactor, no change to another crate, no change to `Cargo.lock` or the workspace
manifest.

`src/backend/child.rs`, `src/backend/syscall.rs`, `src/backend/injection.rs`, `src/authority.rs`,
`src/model.rs` and `src/layout.rs` are **byte-unchanged**.

**`P4_AUTHORITY_AND_COMMIT_SCOPE_SOUND`.**

## 3. Public API diff

Every new public item, inventoried independently from `src/lib.rs` and the module sources:

| New public item | Where | Cohort |
|---|---|---|
| `launch(AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError>` | `src/launch.rs` | Linux x86_64 only |
| `LaunchOutcome` | `src/launch.rs` | Linux x86_64 only |
| `LaunchError`, `LaunchErrorCode`, `PreparationStep` | `src/error.rs` | portable data, producers cohort-gated |

`LaunchReceipt`, `ReceiptRecord`, `MAX_RECEIPT_BYTES` and the model vocabulary are pre-existing P1
surface, unchanged.

| Claim | Result |
|---|---|
| `launch` and `LaunchOutcome` absent off cohort | **PROVEN** — `mod launch;` and the re-export are each immediately preceded by `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]`, asserted by `tests/p4_boundary.rs`, `tests/p3_boundary.rs` and `tools/tests/test_helm_launch_confinement.py`; the `compile_fail` doctests `use helm_launch::launch;` and `use helm_launch::LaunchOutcome;` were moved into the off-cohort `cfg_attr` block, so they are a real claim on Windows and macOS rather than a false one on Linux |
| No public `pid`, `pidfd`, raw fd, `Backend` handle, `PreparedLaunch`, `SpawnedChild`, group id, fault injection or lifecycle handle | **PROVEN** — `LaunchOutcome`'s fields are `LaunchReceipt`, two `Vec<u8>`, two `bool` and one `Duration`; `backend` stays a private `cfg`-gated module with no public item; `p2_boundary` pins the crate root to exactly three `launch` tokens and one `LaunchOutcome` token and forbids `launch_minimal`, `PreparedLaunch`, `SpawnedChild`, `ChildHandle`, `MinimalLaunch` and `ChildPlan` |
| `AuthorizedLaunch` consumed once | **PROVEN** — `launch` takes it by value; the `compile_fail` doctest `twice(a)` is committed and executed by `cargo test` |

Ran locally: 23 `compile_fail` doctests pass in the crate root, `plan.rs` and `receipt.rs`.

**`P4_PUBLIC_LAUNCH_BOUNDARY_SOUND`.**

## 4. `Err` versus receipt — the boundary

`launch` has exactly one fallible step
([`launch.rs:214-221`](../../crates/helm-launch/src/launch.rs#L214-L221)):

```
let spawned = backend::spawn_for_lifecycle(authorized).map_err(to_launch_error)?;
Ok(Observer::new(spawned).run(started))
```

`tests/p4_boundary.rs` pins that the compacted body of `launch` contains exactly one `?` and ends
in the unconditional `Ok`. I re-derived the same fact from the source rather than from the test.

Complete audit of `spawn_for_lifecycle`'s error set
([`backend/mod.rs:491`](../../crates/helm-launch/src/backend/mod.rs#L491)):

| Error | Raised where | Child exists? |
|---|---|---|
| `Preparation { Pipe / Relocate / NonBlocking }` | `spawn::prepare` | **no** |
| `InternalInvariant` from `plan_number` / address conversion | before `clone3` | **no** |
| `SignalMaskFailed` | `rt_sigprocmask` before `clone3` | **no** |
| `ProcessCreationUnavailable` / `ProcessCreationFailed` | `clone3` itself failed | **no** |
| `PidfdNotProvided` | after a successful `clone3` | **yes** — see `F-P4-M2`, structurally unreachable |
| `SignalMaskRestoreFailed` | **not returned** on this path — carried as `mask_restore_errno` and dropped | n/a |
| `SignalFailed` | not reachable from this entry point | n/a |

Every post-child observation failure is a receipt fact, not an error, and I verified each one
individually:

| Post-child condition | Disposition |
|---|---|
| signal-mask restore failure after `clone3` | **fact, discarded** on the P4 path (it describes the launching thread, not the child) |
| `poll` failure | **fact** — the loop stops waiting on readiness and lets the model's deadlines finish; never an error |
| stream read failure | `Completeness::ReadFailed { errno }` |
| status read failure | `ExecStatus::Indeterminate(StatusReadFailed { errno })` |
| `waitid` probe failure | `Probe::AlreadyReaped` → `GroupSweep::NotIssuedChildAlreadyReaped` |
| `waitid` reap failure | `ChildEnd::EndUnobservable` |
| `pidfd_send_signal` failure | ignored; only the *action taken* is recorded, never delivery |
| group-sweep failure | ignored; `GroupSweep::Issued` records the call, not its effect |
| serialization / invariant | the serializer is infallible; `Facts` is complete by construction before `Phase::Finished` |

No path hides a live or attempted child behind `Err` on the public P4 entry point.

**`P4_ERR_VS_RECEIPT_BOUNDARY_SOUND`.**

## 5. `P4A-01` — the spawn mask-restore transition

`spawn()` no longer returns `SignalMaskRestoreFailed` after the child exists; it returns
`SpawnedChild { child, group_authority_established, mask_restore_errno }`
([`backend/spawn.rs:445-458`](../../crates/helm-launch/src/backend/spawn.rs#L445-L458)).

**A. Can a mask-restore failure escape the public `launch()` as `Err`?**
**NO.** `spawn_for_lifecycle` destructures `mask_restore_errno: _`, so the value cannot reach a
return. `to_launch_error` does map the variant, but only as a total match over a crate-private enum
— the comment says so and the source confirms it is unreachable from this entry point. A library
that maps rather than panics on an internal invariant is the right choice here.

**B. Does the P3 compatibility path leak, abandon, double-kill, double-reap or hide a child?**
**NO.** `launch_with` re-raises the historical error at
[`backend/mod.rs:418-424`](../../crates/helm-launch/src/backend/mod.rs#L418-L424), *before*
`release_child_side`. On that early return Rust drops locals in reverse declaration order:
`child` first — `ChildHandle::Drop` sends one `SIGKILL` through the pidfd and reaps within
`POST_KILL_REAP_MS`, setting `reaped` so no second signal can follow — then `prepared`, whose
`OwnedFd` fields close every descriptor including the parent's copies of the six child-side ends.
That is exactly the pre-P4 behaviour, relocated by one stack frame. `launch_minimal` is internal
and has no public consumer.

**C. When are the child-side pipe ends released relative to this path?**
On the P4 path, `release_child_side(false)` runs **after** `spawn` returns `Ok`
([`backend/mod.rs:511`](../../crates/helm-launch/src/backend/mod.rs#L511)), so the parent's copies
of the six child-side descriptors and the stdin writer close before the observation loop begins —
every pipe can reach EOF. On the P3 error path it never runs; `PreparedLaunch`'s drop closes the
same descriptors instead. No descriptor is released twice and none survives.

**D. Can `ChildHandle::Drop` produce unexpected blocking or duplicate lifecycle actions?**
On the P3 compatibility path, no: the child is `SIGKILL`ed, so the bounded reap returns at once.
On the **public P4 path** it can — see `F-P4-02`. That is not a `P4A-01` effect; it is the
`P4A-02` interaction.

**E. Is every fd and child handle owned exactly once?**
**YES.** `PreparedLaunch` → `ParentEnds` → `SpawnedForLifecycle` → `Observer` is a chain of moves;
`Capture::fd` is an `Option<OwnedFd>` taken and either returned or dropped; `ChildHandle` owns the
pidfd and is dropped once. The committed test
`the_outcome_owns_no_descriptor_and_consumes_its_authorisation` checks `/proc/self/fd` before and
after (but see `F-P4-M5`).

**`P4_SPAWN_RESTORE_TRANSITION_SOUND`. `P4A-01`: NOT A FINDING.**

## 6. Pure model versus the real loop

`src/launch.rs` holds **no lifecycle policy of its own**. Every classification, deadline, latch and
sweep decision comes from `Lifecycle::step`; the adapter only converts I/O into events and events
into syscalls.

| Real observation | Event | Model action | Performed operation | Resulting fact / event |
|---|---|---|---|---|
| `poll` reports the status fd (with or without `HUP`) | `StatusReady { hangup }` | `ReadStatus` | `read` to `EAGAIN`/`0`/error | `StatusRecord` / `StatusMalformed` / `StatusEof` / `StatusReadFailed` |
| `poll` reports a stream (with or without `HUP`) | `StreamReady { stream, hangup }` | `ReadStream` | `read` to `EAGAIN`/`0`/error, counting and hashing | `StreamEof` / `StreamReadFailed` / nothing |
| `poll` reports `POLLIN` on the pidfd | `ChildEndReadable` | — | pidfd leaves the poll set | `end_observed`, drain deadline armed |
| monotonic `now >= next_deadline()` | `DeadlineReached` | `SendSigterm` / `SendSigkill` / none | `pidfd_send_signal(TERM \| KILL)` | `sigterm_sent` / `sigkill_sent`, next deadline armed, or a completeness value latched |
| observation complete, authority established | — | `ProbeReaped` | `waitid(P_PIDFD, WEXITED\|WNOHANG\|WNOWAIT)` | `Probed(Unreaped \| AlreadyReaped)` |
| probe says unreaped | — | `SweepGroup` | `kill_process_group(child_pid, SIGKILL)` | `GroupSweep::Issued` |
| cleanup | — | `Reap` | `waitid(P_PIDFD, WEXITED\|WNOHANG)` | `Reaped(Exited \| Killed \| Dumped \| Unclassifiable \| NothingAvailable)` |

`Action::SweepGroup` is executed synchronously and `Action::Reap` evaluates `self.reap()` at the
moment the action is processed, so the sweep strictly precedes the reap on every path.

**Policy that does live outside `Lifecycle::step`, and its disposition:**

1. **The Phase A byte grammar** — “exactly 8 bytes, pad zero, known stage” versus “short then EOF”
   versus “more than 8”. This is `read_status`'s, not the model's. It matches plan §8.5 exactly,
   but the pure model cannot test it and no real-loop case covers the malformed spellings
   (`F-P4-M7`).
2. **The absolute `guard_ms` backstop** — an extra bound with no counterpart in the accepted plan.
   Benign as written, and `F-P4-01` shows it is not the structural guarantee its comment claims
   (`F-P4-B1`).
3. **`poll` failure → `DeadlineReached`** — an invented mapping; see `F-P4-M4`.
4. **The adapter's own `end_observed` flag** — an I/O detail (poll-set membership), not policy; the
   model ignores repeated `ChildEndReadable` anyway.

**The one substantive divergence is the total bound.** The model's own randomized test
([`lifecycle.rs:1229`](../../crates/helm-launch/src/lifecycle.rs#L1229)) asserts over 20 000
generated event orders that the lifecycle finishes within
`SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_KILL_REAP_MS + POST_EXIT_DRAIN_MS` — the
accepted §8.6 bound, with `POST_KILL_REAP_MS` counted **once**. The real loop's
`total_bound_ms` counts it **twice** and the real-loop bound test asserts that widened value plus
5 000 ms of slack. The model and the adapter therefore disagree on a load-bearing accepted
property.

**`P4_REAL_LOOP_MATCHES_PURE_MODEL`: NOT SOUND — see `F-P4-02`.** Everything else in the table above
agrees.

## 7. Poll set and loop progress

The poll set is built fresh each iteration and holds at most four descriptors: status, stdout,
stderr, pidfd ([`launch.rs:647-667`](../../crates/helm-launch/src/launch.rs#L647-L667)).

| Descriptor | Removed when | Verified |
|---|---|---|
| status | `read == 0` or read error → `self.status = None`, `OwnedFd` dropped | **yes** |
| stdout / stderr | `read == 0` or read error → `Capture::fd` left `None`, `OwnedFd` dropped | **yes** |
| pidfd | first `POLLIN` → `self.end_observed = true`, excluded from every later set; the handle stays owned for the reap | **yes** |

**`P4A-05` independently proven absent.** A pidfd stays readable for the rest of the process's
life once the child has ended, so leaving it in the set would make every subsequent `poll` return
immediately and turn the post-exit drain into a 100 % CPU spin. The `end_observed` guard removes
it after exactly one readiness, and it is set unconditionally on `POLLIN` — including when the
model ignores the event because `EndNotObserved` is already latched, which is the case that would
otherwise spin hardest. No other descriptor can stay permanently ready: a stream or status fd that
is ready is read, and readiness without data is a zero-byte read, which removes it.

An empty poll set does not block: `wait_once` sleeps for the computed wait instead of calling
`poll` with no descriptors ([`launch.rs:668-675`](../../crates/helm-launch/src/launch.rs#L668-L675)),
so the wait stays bounded by the model's next deadline.

**`P4_POLL_PROGRESS_SOUND`** — with the caveat that **no committed test guards this regression**.
The accepted plan §14.3 O-series names “early stdout close with CPU and poll-return bounds”; it is
not implemented (`F-P4-07`).

## 8. Deadlines, `EINTR` and the drain

### 8.1 Inventory

| Deadline | Armed at | Value | Source |
|---|---|---|---|
| Phase A pre-exec | `Lifecycle::new` | `spawned_at + 5 000` | monotonic |
| Phase B run | clean status EOF, only if the end is not already observed | `now + timeout_ms` | monotonic |
| grace | run deadline expiry, after `SIGTERM` | `now + grace_ms` | monotonic |
| post-kill | every `SIGKILL` the lifecycle sends, at most one | `now + 5 000` | monotonic |
| post-exit drain | `ChildEndReadable` | `now + 2 000` | monotonic |
| absolute guard | `Observer::new` | `total_bound_ms + 5 000` | monotonic |

All six derive from `Instant`, which is `CLOCK_MONOTONIC` on Linux; the rustix `time` feature is
deliberately not enabled and `p4_boundary` asserts that. `now_ms()` floors
`spawned_at.elapsed().as_millis()`, so a deadline fires at most ~1 ms late and never early.

### 8.2 Arithmetic

`saturating_add` throughout; `deadline.saturating_sub(now).min(250)` for the wait; `timeout_ms ≤
600 000` and `grace_ms ≤ 60 000` are plan-validated, so the sum cannot overflow `u64`. The
`Timespec` conversion is exact for waits of at most 250 ms. `grace_ms == 0` arms the grace deadline
at `now`, which the top-of-loop check consumes on the next iteration — one extra iteration, no
unbounded poll. A deadline is applied *before* any wait, so a due deadline is never deferred past a
`poll`; conversely a readiness observed before the deadline is delivered first and suppresses it,
which is T4/T5.

No deadline is extended by `EINTR`, spurious readiness, stream traffic, a loop iteration or any
other event: every wait is recomputed from an absolute monotonic deadline.

### 8.3 `EINTR`

| Call | Handling | Bounded? |
|---|---|---|
| `poll` | `Err(INTR)` returns to the outer loop, which recomputes `now` and the remaining wait from the absolute deadline | **yes** — an `EINTR` storm cannot recreate a fresh full timeout |
| `read` | retried inside `drain` | **no deadline recheck** — see `F-P4-01` |
| `waitid` probe | `Err(INTR)` → `Probe::Unreaped`, the conservative branch (a sweep may be issued; no signal is suppressed on an unknown result) | **yes** |
| `waitid` reap | `Err(INTR)` → `None`, `reaped` left unset; the caller does not loop | **yes** |
| `pidfd_send_signal` | result discarded; the action is recorded, not its delivery | **yes** |

### 8.4 The drain

`drain` ([`launch.rs:332-349`](../../crates/helm-launch/src/launch.rs#L332-L349)) loops until
`EAGAIN`, a zero-byte read or a read error. It has **no iteration cap, no byte cap and no deadline
recheck**. This is `F-P4-01`.

**`P4_DEADLINE_ACCOUNTING`: NOT SOUND. `P4_EINTR_BOUND`: SOUND except inside `drain`.
`P4_STREAM_FAIRNESS`: NOT SOUND.**

## 9. Read before hangup

`PollFd::new(fd, PollFlags::IN)` still receives `HUP`, `ERR` and `NVAL` in `revents`. The adapter
computes `hangup` but passes it as *informational only*: for a stream it always emits
`StreamReady { stream, hangup }`, and the model always answers `Action::ReadStream` while the
stream slot is unset. `Drained::Eof` is produced **only** by `Ok(0)`; `POLLHUP` never removes a
descriptor and never becomes EOF. A descriptor leaves the set only on `read == 0` or a read error,
and `drain` reads to `EAGAIN` before returning, so every buffered byte is consumed before the EOF
that follows it is recorded. No byte can be lost to a hangup.

O8 test probity: the **pure model** covers `POLLIN | POLLHUP` ordering. The accepted plan's
real-loop O8 case (“POLLIN+POLLHUP, 200 repetitions”) is **not implemented** (`F-P4-07`).

**`P4_READ_BEFORE_HANGUP_SOUND`.**

## 10. Stream accounting

`Capture::absorb` ([`launch.rs:290-311`](../../crates/helm-launch/src/launch.rs#L290-L311)) counts,
then hashes, then retains — in that order, once per chunk, before any prefix decision. Both parent
read ends are made non-blocking in `prepare` (plan §8.2 step 5, accepted P3 code) before any child
exists.

| Property | Result |
|---|---|
| Every drained byte counted exactly once | **YES** — `bytes = bytes.saturating_add(len)` on the one path |
| Every drained byte hashed exactly once | **YES** — `hasher.update(chunk)` on the one path |
| Prefix holds the first `N` bytes only | **YES** — `room = limit - prefix.len()`, append `min(len, room)` |
| Bytes past `N` counted and hashed, not retained | **YES** |
| `N = 0` | **works** — empty prefix, `truncated` set on the first non-empty chunk |
| stream length `== N` | **works** — `truncated` stays false (untested, `F-P4-M7`) |
| stream length `> N` | **works** — `truncated` true (tested) |
| stream length `< N` | **works** — `truncated` false (tested) |
| Unbounded buffer | **NO** — `Vec::with_capacity(limit)` allocated once, never grown past `limit` |

The plan bound is `MAX_CAPTURE_BYTES = 65 536` per stream, validated at parse time.

**`P4_STREAM_ACCOUNTING_SOUND`.**

## 11. Phase A — exec status

`read_status` ([`launch.rs:444-502`](../../crates/helm-launch/src/launch.rs#L444-L502)) accumulates
into a 9-byte buffer, so one byte beyond the fixed record is detectable.

| Observation | Emitted | Model classification | Matches plan §8.5 |
|---|---|---|---|
| exactly 8 bytes, pad zero, known stage, then EOF | `StatusRecord`, `StatusEof` | `PreExecFailure { stage, errno }` | **yes** |
| 8 bytes with an unknown stage or a non-zero pad | `StatusMalformed` | `Indeterminate(StatusRecordMalformed)` | **yes** |
| 1–7 bytes then EOF | `StatusMalformed`, `StatusEof` | `Indeterminate(StatusRecordMalformed)` | **yes** |
| more than 8 bytes (one read or two) | `StatusMalformed` | `Indeterminate(StatusRecordMalformed)` | **yes** |
| 0 bytes then EOF | `StatusEof` | `Indeterminate(StatusEofWithoutRecord)`, run deadline armed | **yes** |
| deadline with the status still `Awaiting` | `DeadlineReached` | `Indeterminate(PreExecStatusTimeout)` + immediate `SIGKILL` | **yes** |
| read error | `StatusReadFailed` | `Indeterminate(StatusReadFailed { errno })`, **never** EOF | **yes** |

Event ordering within one read is `[record-or-malformed, then EOF]`, so the model learns the record
shape before it classifies the end-of-file — the case a naive implementation gets wrong.

Interleavings I checked explicitly:

* **pidfd readable before status EOF** — `ChildEndReadable` sets `end_observed`; the status channel
  stays in the poll set and keeps being read to EOF within the Phase A deadline. A clean EOF that
  arrives after the end was observed does **not** arm the run deadline (`if !self.end_observed`),
  which is correct: the child has already ended.
* **status and pidfd ready in the same `poll`** — both revents are processed in the fixed slot
  order in one pass at one `now`; the status fd is first.
* **partial record then deadline** — `PreExecStatusTimeout` wins, because the status is still
  `Awaiting`; the partial bytes are discarded, not guessed.

No clean-EOF path becomes exec success anywhere.

**`P4_PHASE_A_SOUND`.**

## 12. No exec-success claim

Searched the whole crate, the README, the workflow and the docs delta for `ExecSucceeded`,
`exec_succeeded`, “exec confirmed”, “successfully ran”, “execution succeeded”, “launched
successfully”. Every occurrence is a **non-claim** in prose or a forbidden-token list in
`tests/p4_boundary.rs`. No public API, receipt field, enum variant, `Display` text or document
introduces a positive exec claim. The run deadline is armed at the `StatusEof` event and nowhere
else. The committed real-loop case
`a_clean_status_eof_is_indeterminate_and_never_exec_success` asserts it end to end.

The purpose-built fixtures report their own execution to the test only; nothing maps a fixture
report onto a product `exec_status`. The `retainer` and `grouper` fixtures print pids that the test
parses from the in-memory prefix, never from the receipt.

**`P4_NO_EXEC_SUCCESS_CLAIM`. `P4_TEST_EVIDENCE_SEPARATION_SOUND`.**

## 13. Phase B — the run deadline

`run_deadline = now + timeout_ms` is armed at `Event::StatusEof` with no record and no malformed
flag, and only when the end is not already observed
([`lifecycle.rs:331-343`](../../crates/helm-launch/src/lifecycle.rs#L331-L343)). It is armed at
spawn nowhere, at clone nowhere, and at a confirmed exec nowhere — there is no such event.

* **Child ends before the deadline** — `next_deadline()` filters `run_deadline` by `!ended`, so the
  deadline never becomes due, `run_deadline_expired` stays false and no `SIGTERM` is emitted, even
  though Phase C cleanup follows later. Tested in the real loop
  (`a_child_that_ends_before_the_deadline_expires_nothing`) and in the model (T4/T5).
* **Deadline expires first** — `run_deadline_expired = true`, one `SendSigterm`, grace armed. Then,
  on grace expiry, one `SendSigkill`. Tested in the real loop for both the ends-in-grace and the
  ignores-`SIGTERM` shapes.
* **Simultaneous readiness and deadline** — `observe` applies a due deadline at the top of the
  iteration, *before* the next `poll`; a `poll` that returns readiness before the deadline is due
  delivers `ChildEndReadable` first and the deadline is then filtered away. A readiness the kernel
  reports only after the deadline has passed but before it is applied also suppresses the deadline.
  That direction is conservative — it never fabricates a timeout — and is within the “classification
  is from what was observed” rule and the plan's scheduling slack.

**`P4_PHASE_B_SOUND`.**

## 14. `SIGTERM`, grace, `SIGKILL`

`Lifecycle::send_sigkill` is guarded by `!self.sigkill_sent`, and `SendSigterm` is emitted only from
the single run-deadline branch, which is guarded by `!self.run_deadline_expired`. `on_deadline`
applies **the earliest due deadline only** and returns, so repeated `DeadlineReached` events drain
them in order and cannot double-fire a branch. The randomized model test asserts
`count(SendSigterm) ≤ 1` and `count(SendSigkill) ≤ 1` over 20 000 sequences.

Every signal goes through `pidfd_send_signal(self.child.pidfd(), signal)` — one call site, pinned by
`p4_boundary` and by the Python confinement suite. No numeric-pid direct-child signal exists
anywhere in the crate: `waitpid`, `wait4`, `killpg`, `tkill` and `tgkill` are forbidden tokens in
every source.

`grace_ms == 0` transitions deterministically on the next loop iteration.

Facts are recorded without causation: `sigterm_sent`, `sigkill_sent` and the observed `child_end`
are three independent fields. There is no `KilledByLauncher` value anywhere.

**One deviation:** on the `EndNotObserved` path `ChildHandle::Drop` sends a **second, unrecorded**
direct `SIGKILL` before `launch` returns. See `F-P4-02`.

**`P4_TIMEOUT_GRACE_KILL_SOUND`** in the lifecycle; the `Drop` signal is carried under `F-P4-02`.

## 15. The `EndNotObserved` latch

Three, and only three, sites mutate `Lifecycle::child_end`:

| Site | Behaviour |
|---|---|
| [`lifecycle.rs:441`](../../crates/helm-launch/src/lifecycle.rs#L441) | the kill-deadline branch sets `EndNotObserved`, guarded by `!end_not_observed` |
| [`lifecycle.rs:493`](../../crates/helm-launch/src/lifecycle.rs#L493) | `after_probe(AlreadyReaped)` uses `get_or_insert(EndUnobservable)` — **cannot overwrite** |
| [`lifecycle.rs:513-522`](../../crates/helm-launch/src/lifecycle.rs#L513-L522) | `after_reap` matches `(Some(latched), _) => latched` — **cannot overwrite** |

Once latched, `Event::ChildEndReadable` is also inert (`if !self.end_observed && !self.end_not_observed`),
so a late pidfd readiness cannot arm a drain deadline or change the classification. The real
adapter cannot bypass this: it owns no `child_end` of its own and copies `Facts` verbatim into the
`ReceiptRecord`.

Model coverage is exhaustive: `t40_end_not_observed_is_latched_and_a_later_reap_never_revises_it`
runs both authority values × both kill paths × `Exited`/`Killed`/`Dumped`, and the randomized test
asserts `end_not_observed ⇔ child_end == EndNotObserved` on every generated order.

**`P4_END_NOT_OBSERVED_LATCH_SOUND`.**

## 16. `P4A-02` — the total bound, independently re-rated

**The author's `MINOR` is not inherited. Confirmed and reclassified `IMPORTANT` (`F-P4-02`).**

`total_bound_ms` ([`launch.rs:90-97`](../../crates/helm-launch/src/launch.rs#L90-L97)) counts
`POST_KILL_REAP_MS` twice. The four questions:

**1. Can public `launch()` wait one `POST_KILL_REAP_MS` in the lifecycle and then another in
`Drop`?** **YES.** On the `EndNotObserved` path the final `Action::Reap` calls
`ChildHandle::reap_once`, whose `waitid(WNOHANG)` returns `Ok(None)` for a child that has not
ended. That leaves `reaped == false` and `end == None`. `ChildHandle::Drop`
([`backend/spawn.rs:639-646`](../../crates/helm-launch/src/backend/spawn.rs#L639-L646)) then sends
one `SIGKILL` and calls `reap_within(POST_KILL_REAP_MS)`, which polls with 10 ms sleeps for up to
5 000 ms. Note the Phase C `probe()` uses a direct `waitid` and does **not** set `reaped`, so it
cannot disarm the guard either.

**2. Is the second bound reachable on a valid public path?** **YES.** `EndNotObserved` is a
first-class accepted receipt state, not an error state. It is reached whenever a child does not end
within 5 000 ms of a `SIGKILL` — uninterruptible sleep on a stuck device or network filesystem, or
a frozen cgroup. Rare, but a real host condition the contract exists to describe.

**3. Does it occur before `launch()` returns?** **YES.** `run(mut self, …)` consumes the
`Observer`; `self` — and therefore `child` — is dropped at the end of `run`'s body, after the
`LaunchOutcome` is constructed and before control returns to `launch`.

**4. Does it violate the accepted total-bound contract?** **YES.** Plan §8.6 and ADR §J state
`SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_KILL_REAP_MS + POST_EXIT_DRAIN_MS` plus
scheduling slack, and §8.6 adds that “every `SIGKILL` sent to the direct child by pidfd, on the
pre-exec and the run path alike, is followed **only** by the bounded kill wait of 8.5”. The `Drop`
`SIGKILL` and its wait are a second one. ADR §J also says that at the post-kill bound “the child is
left unreaped to the host, and `launch` still returns” — the implementation instead signals it
again and waits again.

**5. Is the test asserting a widened implementation bound?** **YES.**
`every_launch_returns_inside_the_accepted_total_bound`
([`launch.rs:1336-1352`](../../crates/helm-launch/src/launch.rs#L1336-L1352)) computes
`total_bound_ms(200, 200) + GUARD_SLACK_MS = 22 400 ms`. The accepted §8.6 bound for the same plan
is `12 400 ms`. The test name says “accepted”; the constant is 1.8× it. The worst case that suite
can actually produce finishes in roughly 500 ms, so the assertion discriminates nothing either way.

**Aggravating:** the pure model's own randomized test asserts the **accepted** single-`POST_KILL_REAP_MS`
bound over 20 000 sequences, so the model and the real adapter contradict each other on exactly this
property — which is what the P4 stop condition (“state-machine scripts agree with the real loop on
shared scenarios”) exists to prevent.

**Not unbounded:** the second wait is a fixed 5 000 ms, so this is not a `BLOCKER` by itself.

**`P4_TOTAL_BOUND`: NOT SOUND. `P4_TOTAL_BOUND_TEST_PROBATIVE`: NOT PROBATIVE.**

## 17. pidfd end observation and sweep-before-reap

The probe is `waitid(WaitId::PidFd(…), EXITED | NOHANG | NOWAIT)`
([`launch.rs:544-556`](../../crates/helm-launch/src/launch.rs#L544-L556)). `WNOWAIT` leaves the
child collectable, so the probe cannot consume the end the reap must classify, and it does not
touch `ChildHandle`'s `end`/`reaped` cells. Nothing else in the observation phase waits on the
child: readiness comes from `poll` on the pidfd, never from a consuming wait.

The only consuming wait is `Action::Reap`, and the model emits it strictly after `SweepGroup` in
`after_probe`. The adapter preserves the order because actions are performed in sequence and
`self.reap()` is evaluated at the point the `Reap` action is handled.

**`P4_SWEEP_BEFORE_REAP_SOUND`.**

## 18. Group authority, target safety and the foreign-reaper guard

**Authority source.** `group_authority_established` is set by exactly one expression —
`setpgid(Some(pid), Some(pid)).is_ok()` in `spawn`, the first system call after `clone3` — and is
carried verbatim into `GroupAuthority::Established | NotEstablished`. Nothing infers it. `getpgid`,
`getpgrp` and `tcgetpgrp` appear nowhere in product code, asserted by `p4_boundary` and the Python
suite. With `NotEstablished` the model emits **zero** `ProbeReaped` and **zero** `SweepGroup`
actions on every path, asserted for all six generated paths by `t31_without_group_authority_no_path_issues_a_sweep`.

**Target safety.** `sweep_group` ([`launch.rs:567-576`](../../crates/helm-launch/src/launch.rs#L567-L576))
derives the target solely from `self.child.pid()`, the value `clone3` returned, and calls
`kill_process_group`, which rustix implements as `kill(-pid, SIGKILL)`. There is no path to
`kill(0, …)` (current group) or `kill(-1, …)`. The group id equals the direct child's pid and was
created by the launcher's own `setpgid`; the `WNOWAIT` probe immediately before the sweep has just
established that the child is still unreaped, and an unreaped child — zombie or running — keeps its
pid, and therefore its process-group id, reserved. Pid/group-id reuse is excluded for as long as
that holds. The one residual window — a foreign reaper acting between the probe and the sweep — is
the documented caller-precondition violation R4, which ADR §J and plan §8.7 already state.

One defence-in-depth observation: `Pid::from_raw` is **not** the positivity guard it looks like
(`F-P4-M1`).

**Foreign reaper / `ECHILD`.** `Err(_)` from the probe — including `ECHILD` — maps to
`Probe::AlreadyReaped`, which sets `GroupSweep::NotIssuedChildAlreadyReaped`, emits **no**
`SweepGroup`, and inserts `EndUnobservable` only if nothing is latched. `Err(INTR)` maps to
`Unreaped`, which is the conservative branch (it issues the sweep rather than suppressing it on an
unknown result, and never signals on a *known* reap). The subsequent `reap_once` returns
`EndUnobservable` on `ECHILD` and sets `reaped`, so `Drop` cannot signal a pid this handle no
longer owns. The guard is sound by construction, but it is exercised in the pure model only
(`F-P4-06`).

**Exactly one sweep.** `after_probe` is the only producer of `Action::SweepGroup`, it is reached
once because `enter_cleanup_if_done` is guarded by `phase == Phase::Observing` and immediately moves
to `AwaitingProbe`, and `Phase::AwaitingProbe` accepts only `Event::Probed`. There is no retry loop
around the group kill; the `Result` is discarded. `t30_with_group_authority_every_path_issues_exactly_one_sweep_before_the_reap`
asserts one sweep on all six paths and that no later event produces a second. The product records
issuance, never delivery: `GroupSweep::Issued` is set before the call and is unaffected by its
result, and every document says so explicitly.

**`P4_GROUP_AUTHORITY_GUARD_SOUND`. `P4_GROUP_SWEEP_TARGET_SOUND`. `P4_FOREIGN_REAPER_GUARD_SOUND`
by construction. `P4_EXACTLY_ONE_GROUP_SWEEP_SOUND`.**

## 19. `P4A-04` — real group-sweep test probity, independently re-rated

**The author's `MINOR` is not inherited. Confirmed and reclassified `IMPORTANT` (`F-P4-05`).**

Both committed P-series cases accept a disjunction:

* `a_launch_records_exactly_one_guarded_group_sweep_disposition` asserts only
  `Issued | NotIssuedGroupNotEstablished`.
* `the_sweep_reaches_a_same_group_descendant_and_never_an_escaped_one` branches on the observed
  disposition and asserts the descendant is gone **only in the `Issued` arm**.

If both runs take the `NotIssuedGroupNotEstablished` arm, the suite is green having exercised the
real `kill_process_group` call **zero times**. Nothing else in the crate exercises it: P3 issues no
group signal at all, and `p4_boundary` only reads the source. So the P4 authority's four requested
positive facts — authority present ⇒ one sweep; sweep before reap; same-group descendant cleaned
up; escaped descendant survives — have **no guaranteed real-loop evidence**. Only the last is
asserted unconditionally.

**Is a deterministic mechanism available?** One exists and is already accepted: the
`test-fault-injection` pre-exec stall, which
`injected::a_child_that_stalls_before_exec_is_bounded_killed_and_reaped` uses to make
`group_authority_established == true` deterministic (the [`P3R-21` review](HELM-LAUNCH-P3-P3R21-REVIEW.md)
records exactly that reasoning). It is **not reachable** from `launch()`, because
`spawn_for_lifecycle` hard-codes `Fault::default()` — correctly, since the public API must not
select fault injection. So the existing seam cannot be used without a product change, and I do not
propose one here.

I note, without prescribing it, that one committed P4 case already runs a fixture whose `execveat`
fails (`an_object_without_execute_permission_reports_the_exec_stage_failure`); a child that never
completes an `execve` cannot answer the parent's `setpgid` with `EACCES`. Whether that makes
authority deterministic enough to carry a positive sweep assertion is an implementation question
for the author and the owner, not a reviewer's design.

The `P3R-21` constraint is respected either way: **requiring** `Established` on an ordinary
uncoordinated launch would repeat the defect that failed the second P3 publication. The finding is
not “assert the scheduler's outcome”; it is “the load-bearing sweep behaviour currently has no
deterministic real evidence at all”.

**`P4_REAL_GROUP_SWEEP_TEST_PROBATIVE`: NOT PROBATIVE.**

## 20. `P4A-03` — missing real-loop cases, independently re-rated

**The author's `MINOR` is split.**

| Case | Pure model | Real loop | Deterministic seam available? | Rating |
|---|---|---|---|---|
| `EndNotObserved` | **exhaustive** — both kill paths × both authorities × three late reaps, plus 20 000 randomized orders | **absent** | **no** — it needs a child that survives `SIGKILL`, which cannot be arranged safely or portably | **`MINOR`** (`F-P4-M6`) |
| foreign reap / `ECHILD` sweep suppression | **covered** — `t31_a_child_already_reaped_elsewhere_gets_no_sweep` on all six paths | **absent** | **YES** — plan §14.3 names it: “pidfd lifecycle: test process sets `SIGCHLD = SIG_IGN` → `end_unobservable`, sweep not issued, `launch` returns (R4, T31)” | **`IMPORTANT`** (`F-P4-06`) |

For `EndNotObserved` the author's judgement holds: the model coverage is unusually strong, the
adapter mapping (`Ok(None)` → `Reap::NothingAvailable`) is three lines, and no authorised
parent-side seam can provoke a real one. Accept as a model-only behaviour for this slice, and record
it.

For the foreign-reap case the judgement does not hold. The accepted plan already specifies a safe,
deterministic, parent-side real-loop mechanism for it, and the same row is also the T31 negative
case. The established technique for process-global test state is already in the suite (the F6
“dedicated test subprocess” pattern). P4 delivered neither this row nor the plan's companion row
“test-only parent delay past exec → no group authority, sweep not issued”, so the real adapter's
`AlreadyReaped` and `NotEstablished` branches have never executed against a kernel.

**`P4_END_NOT_OBSERVED_REAL_EVIDENCE_STATUS`: MODEL ONLY — `MINOR`.**
**`P4_FOREIGN_REAP_REAL_EVIDENCE_STATUS`: MODEL ONLY — `IMPORTANT`.**

## 21. Reap classification and `ChildHandle::Drop` integration

`reap_once` is `waitid(P_PIDFD, WEXITED | WNOHANG)` — non-blocking, descriptor-addressed, no
`waitpid` or `wait4` fallback anywhere in the crate. `classify` tests `exited()`, then `dumped()`,
then `killed()`, so `CLD_DUMPED` keeps its signal number and sets `core_dumped: true` (R3); anything
else is `EndUnobservable`. `Reap::Unclassifiable` and `Reap::NothingAvailable` both resolve to
`EndUnobservable` **only if nothing is latched**.

Drop behaviour at each public return path:

| Path | `reaped` at return | `Drop` action | Extra wait |
|---|---|---|---|
| normal exit, signalled exit, exec failure, indeterminate status, pre-exec timeout, run timeout | **true** (`waitid` collected the zombie) | none | none |
| probe saw `ECHILD` | **true** (`reap_once` latched `EndUnobservable` on the error) | none | none |
| final `reap_once` hit `EINTR` | false, but the child is a zombie | `SIGKILL` + immediate collect | ~0 |
| **`EndNotObserved`** | **false** | **`SIGKILL` + up to 5 000 ms** | **`F-P4-02`** |

No already-reaped child is signalled — `Drop` returns early on `reaped`, and `reaped` is set by both
the success and the error arms of `reap_once`. No descriptor survives: `ChildHandle` owns the pidfd,
`Capture` owns the stream ends, `Observer` owns the status end, and all are dropped with `Observer`
inside `run`.

**`P4_REAP_CLASSIFICATION_SOUND`. `P4_CHILDHANDLE_DROP_INTEGRATION`: NOT SOUND on the
`EndNotObserved` path only.**

## 22. Post-exit drain

`ChildEndReadable` arms `drain_deadline = now + POST_EXIT_DRAIN_MS` (2 000 ms). Both streams keep
draining; at expiry every still-open stream becomes `WriterRetainedAfterChildExit` and the loop
finishes, closing the remaining read descriptors when `Observer` drops. There is no wait on
descendant-held writers beyond that bound.

The committed case `a_writer_that_outlives_the_child_is_bounded_and_named_as_such` uses a **real**
descendant (`retainer` spawns `sleeper 20000`, which inherits stdout and stderr and outlives the
direct child), asserts `WriterRetainedAfterChildExit`, asserts return well inside the bound, and
cleans the descendant up by pid.

**`P4_POST_EXIT_DRAIN_BOUND_SOUND`.**

## 23. Receipt: source of truth, determinism, digest, size, privacy, authenticity

**Single model.** `Observer::record` builds one `ReceiptRecord` from `Facts` plus the plan and the
P2 measurement, and `LaunchReceipt::from_record` is the only constructor. `src/launch.rs` defines no
alternative receipt type, no second serializer and no parallel field set. The serializer in
`receipt.rs` is unchanged by P4 except for one `cfg_attr` predicate.

**Determinism.** The serializer is a hand-written, fixed-order string builder with no map iteration,
no clock, no host value and no platform-dependent formatting. Every receipt field is enumerated:
`backend` (closed token), `plan_sha256`, two optional asserted digests, the validated
`working_directory_id` (grammar `[a-z0-9][a-z0-9._-]{0,79}`, so no escaping is needed and no host
text can enter), the pre-exec measurement, `argument_count`, `environment_mode`, `exec_status`,
`child_end`, `run_deadline_expired`, `termination`, and the two stream fact groups. **No timestamp,
`Instant`, `Duration`, pid, pidfd, fd number, host path, random value or scheduler timing value
appears in `ReceiptRecord`**, pinned by a field-level scan in `p4_boundary`. Elapsed time lives only
in `LaunchOutcome`.

**Digest.** `from_record` serialises, then hashes exactly those bytes; `exact_bytes()` returns the
same buffer; nothing is added, removed, redacted or sanitised afterwards; the receipt never contains
its own digest. R3-M1 is satisfied. `the_receipt_is_deterministic_bounded_and_recomputable`
recomputes `Digest::of(receipt.exact_bytes())` per launch, and the P1 anchor recomputes against a
hex constant computed outside Rust. I re-ran the P1 anchor locally: 1 091 bytes,
`c2e58e88…525e949`; data-variant 1 229 bytes, `19771d9c…f7bfc52a`.

**Size.** `MAX_RECEIPT_BYTES = 8 192`. The P1 widest-receipt test enumerates every fact-variant
combination at the widest numeric and digest values and asserts the maximum stays strictly below the
ceiling; the widest observed is far under it. The serializer never truncates, so no syntactically
valid but false receipt can be produced; the bound is a proven property, not a cut point, which is
what the plan accepted.

**Privacy.** Raw output bytes exist in `LaunchOutcome`'s two prefixes and nowhere else. `Capture`
hands the serializer only `bytes_drained`, `drained_sha256` and `completeness`. `LaunchOutcome`'s
`Debug` prints the receipt digest, two prefix lengths and two truncation flags —
`finish_non_exhaustive`, never the bytes. `LaunchReceipt`'s `Debug` prints the digest, the byte
length and the record, which holds no bytes. `LaunchError` carries a code, an optional step and an
optional errno — no host text on any path.

**The canary test that proves this cannot pass. See `F-P4-03`.**

**Host-data privacy.** No argv value, working-directory path, descriptor number, pid, environment
value or OS error string reaches the durable receipt. `working_directory_id` is a caller-chosen
identifier the plan digest already covers, and it is an accepted receipt field (ADR §L). No
`Display`-derived host text exists: every token comes from a closed `as_str` table.

**Authenticity.** No `authentic`, `verified`, `trusted`, `signed`, `provenance` or `attestation`
claim exists as code; `p4_boundary` and the Python suite forbid `signature`, `signed`,
`verify_receipt` and `authenticate` as tokens. `LaunchReceipt` and `ReceiptRecord` have no
`Deserialize`, proven by two `compile_fail` doctests plus a positive control that the *same*
deserialisation into `serde_json::Value` compiles. There is no verification constructor. Every
document states that the digest identifies bytes and nothing about origin.

**`P4_SINGLE_RECEIPT_MODEL_SOUND`. `P4_RECEIPT_DIGEST_SOUND`. `P4_RECEIPT_SIZE_BOUND_SOUND`.
`P4_RECEIPT_HOST_PRIVACY_SOUND`. `P4_NO_AUTHENTICITY_CLAIM`.
`P4_RECEIPT_DETERMINISM`: serializer SOUND, test unsound (`F-P4-04`).
`P4_RECEIPT_PRIVACY`: product SOUND, test cannot pass (`F-P4-03`).**

## 24. `LaunchOutcome` ownership, `Debug` and public negative proofs

`LaunchOutcome { receipt, stdout_prefix, stderr_prefix, stdout_truncated, stderr_truncated, elapsed }`
holds no `OwnedFd`, `BorrowedFd`, `RawFd`, `Pid`, pidfd, `ChildHandle`, capability or
`AuthorizedLaunch`, and implements no `Drop`. Every descriptor is closed before `launch` returns
(all three pipe read ends with `Observer`; the pidfd with `ChildHandle`; the six child-side copies
in `release_child_side`).

Committed negative proofs, all executed by `cargo test` on all three platforms:

| Must be impossible | Proof |
|---|---|
| construct `LaunchOutcome` from fields | `compile_fail`: `helm_launch::LaunchOutcome { receipt: r }` |
| construct `LaunchReceipt` from bytes/fields | `compile_fail`: `LaunchReceipt { bytes, ..real }` |
| deserialise a receipt or record | two `compile_fail` cases plus a compiling positive control |
| obtain raw process identifiers | no public accessor; `p4_boundary` field scan |
| clone an `AuthorizedLaunch` | no `Clone`; `compile_fail` in `plan.rs`/`authority.rs` surface |
| launch twice | `compile_fail`: two `launch(a)` calls |
| invoke the backend | `compile_fail`: `use helm_launch::backend;`, `helm_launch::backend::launch_minimal` |
| decompose an authorisation / read a capability descriptor | `compile_fail`: `a.into_parts()`, `c.descriptor()` |
| select fault injection | no public API; the feature is gated on `feature AND debug_assertions` and is absent from release builds (P3 gate, preserved) |
| `launch` / `LaunchOutcome` off cohort | `compile_fail` inside the off-cohort `cfg_attr` block, and `p3_boundary` asserts they are inside it |

**`P4_OUTCOME_OWNS_NO_DESCRIPTORS`. `P4_OUTCOME_DEBUG_PRIVACY_SOUND`.
`P4_PUBLIC_API_NEGATIVE_PROOFS_SOUND`.**

## 25. Public error surface

One family: `LaunchError` with `LaunchErrorCode` (four closed codes) and `PreparationStep` (four
closed steps). No parallel hierarchy, no `anyhow`, no boxed `dyn Error`, no host path or string, no
pid, no fd. `Display` renders the code, an optional step and an errno from the crate's own closed
symbolic table. Every member is raised before a child exists; after-child conditions have no
spelling here. `std::error::Error` is implemented without a source chain.

**`P4_PUBLIC_ERROR_SURFACE_SOUND`.**

## 26. Dependencies, unsafe and the P3 child contract

| Check | Result |
|---|---|
| New package | **none** — `Cargo.lock` and the workspace manifest are byte-unchanged |
| rustix features | `std, fs, process, pipe` + **`event`** only; `time`, `thread`, `mm`, `net`, `runtime`, `use-libc` all off, asserted three times (Rust boundary suite, Python suite, `p4_boundary`) |
| Clock source | `std::time::Instant` = `CLOCK_MONOTONIC` on Linux; the `time` feature is deliberately not enabled |
| Direct `libc` call | **none** — `libc` stays constants-only, and `p4_boundary` forbids the token `libc` in `src/launch.rs` |
| `linux-raw-sys` direct dependency | **none** |
| New `unsafe` | **zero** anywhere; `src/launch.rs` contains the token only in two prose lines |
| Scoped `allow(unsafe_code)` | still exactly one file: `src/backend/mod.rs` |
| `child.rs` / `syscall.rs` | **byte-unchanged** — no `poll`, no signal lifecycle, no receipt, no allocation and no new syscall entered the child window |
| Cross-target check | `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` — **clean on a fresh target directory**, so `src/launch.rs` and its Linux-only test module do type-check |

**`P4_DEPENDENCY_GRAPH_SOUND`. `P4_UNSAFE_BOUNDARY_SOUND`. `P4_P3_CHILD_CONTRACT_PRESERVED`.**

## 27. P3 regression gates

Every load-bearing P3 gate still runs in `.github/workflows/helm-launch.yml`, unchanged in
substance and with none replaced by a P4 suite:

machine-code child closure (debug/test **and** release codegen, each with a positive control);
fault-injection positive control and release-absence proof; “a release library instantiates no
backend”; `linux_admission` named and run; `backend::tests::` named and run, which carries the
`strace` child-window cases, the stage order, the pidfd acquisition, S5, S6, the `P3R-20`
fixture-concurrency correction and the `P3R-21` group-semantics correction; the fault-injection
`backend::tests::` pass; `p3_boundary`; `p2_boundary`; and both Python suites.

`p4_boundary` is **added** to the boundary step rather than substituted for anything.

**`P4_P3_REGRESSION_GATES_PRESERVED`.**

## 28. Synchronous boundary and test-process hygiene

`launch` is synchronous. `src/launch.rs` spawns no thread (`std::thread` is used only for `sleep`),
starts no executor, keeps no service thread, detaches no observer and owns no thread pool. Nothing
survives the return: the whole lifecycle lives on the calling thread.

Every P4 test that creates a descendant cleans it up by pid with a bounded wait: the `retainer`
descendant, and both the same-group and escaped descendants of `grouper`. The escaped descendant is
expected to survive the sweep — that is the accepted non-claim — and the harness kills it itself.
No test can signal the cargo harness group: a sweep target is always the direct child's own
dedicated group, created by the launcher's `setpgid`, and without that success no sweep is issued at
all. Fixture builds use the `P3R-20` content-addressed, `hard_link`-published protocol, including
the new C fixture builder.

**`P4_SYNCHRONOUS_RETURN_BOUNDARY_SOUND`. `P4_GROUP_SWEEP_TEST_ISOLATION_SOUND`.
`P4_TEST_HELPER_CLEANUP_SOUND`.**

## 29. Documentation and CI design

The crate README now states accurately: `launch()` exists on Linux x86_64 and nowhere else; the real
lifecycle exists; a receipt is emitted; a guarded process-group sweep exists; it is **not**
containment and **not** a sandbox; no Wine; no exec-success claim; P4 is an implemented candidate,
not accepted, with one fresh independent review as the next gate; P5 is not authorised. The slice
table is correct, the dependency section is correct, and “Accepted future behaviour — NOT
IMPLEMENTED” was replaced rather than left stale. Historical P3 text elsewhere remains historically
correct. No current-status text still claims that process execution is internal-only.

The workflow executes the P4 lifecycle cases on `ubuntu-24.04` — the unfiltered `cargo test -p
helm-launch --locked` step already runs them, and the named step re-runs them for log visibility,
so there is no structural skip. Off the cohort the workflow *fails* if any `launch::` test exists.
Windows and macOS remain portable/off-cohort only. No trial semantics, no privileged runner, no
`workflow_dispatch`-only gate, no artifact publication.

**`P4_DOCUMENTATION_SOUND`. `P4_CI_DESIGN_SOUND`** in shape — but two committed P4 cases would make
that green gate red (`F-P4-03`, `F-P4-04`).

## 30. Local validation performed

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo test --workspace --locked` | **PASS** — helm-launch: 42 lib, 20 `p2_boundary`, 17 `p3_boundary`, **12 `p4_boundary`**, 19 `plan_contract`, 2 + 23 doctests (23 `compile_fail`) |
| `cargo test -p helm-launch --locked` | **PASS** |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — 831 tests, 74 skipped by platform |
| `python tools/validate_docs.py` | **PASS** — 140 markdown, 1 578 link targets, 255 JSON |
| `git diff --check` | **PASS** |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | **PASS** on a fresh target directory |

**No Linux execution of the crate occurred.** No WSL Rust toolchain was installed. The only Linux
command run was an unprivileged throughput measurement in an existing distribution, used as evidence
for `F-P4-01` and described there; it built nothing and executed no part of this repository.

**P4 LINUX RUNTIME VALIDATION: `PENDING PUBLICATION CI`.** That is expected at this gate, not a
review failure — but see `F-P4-03` and `F-P4-04` before publishing.

## 31. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `F-P4-01` | **IMPORTANT** | deadline / stream fairness | **YES**, ordinary child | `drain` loops to `EAGAIN` with no cap and no deadline recheck; a child that writes faster than SHA-256 absorbs starves every deadline and the absolute guard | **OPEN** |
| `F-P4-02` | **IMPORTANT** | total bound / `Drop` | **YES**, `EndNotObserved` | A second fixed `POST_KILL_REAP_MS` and a second unrecorded direct `SIGKILL` occur in `ChildHandle::Drop` before `launch` returns; the bound test asserts the widened constant; model and adapter disagree | **OPEN** (`P4A-02` confirmed, reclassified) |
| `F-P4-03` | **IMPORTANT** | test / hosted gate | **YES**, every run | The privacy canary asserts the receipt bytes contain no `"pid"`, but every receipt contains `"linux_x86_64_clone3_pidfd_execveat"`; the Linux-only case fails deterministically | **OPEN** |
| `F-P4-04` | **IMPORTANT** | test / hosted gate | **YES**, intermittently | `the_receipt_is_deterministic_bounded_and_recomputable` asserts byte equality across two real launches over the scheduler-dependent `group_sweep` field, which ADR §L explicitly disclaims | **OPEN** |
| `F-P4-05` | **IMPORTANT** | sweep evidence | n/a (evidence) | No committed real test deterministically exercises a group sweep; both P-series cases can complete having issued none | **OPEN** (`P4A-04` confirmed, reclassified) |
| `F-P4-06` | **IMPORTANT** | foreign-reap evidence | n/a (evidence) | No real-loop case for `ECHILD` sweep suppression, although plan §14.3 names a deterministic `SIGCHLD = SIG_IGN` seam; the T31 “parent delay past exec” negative case is also absent | **OPEN** (`P4A-03` half, reclassified) |
| `F-P4-07` | **IMPORTANT** | accepted Level 3 coverage | n/a (evidence) | Accepted O/R/S/T/P rows not delivered: the CPU/poll-return non-spin bound (the only guard against `P4A-05` returning), `POLLIN+POLLHUP` ×200 in the real loop, 8 MiB concurrent under a 10 s bound, R3 `CLD_DUMPED` classification, S3 exit-127-after-start | **OPEN** |
| `F-P4-M1` | MINOR | sweep target | no | `Pid::from_raw` rejects only zero (its positivity check is `debug_assert`), so sweep-target safety rests solely on `clone3`'s return contract, not on the visible guard | record |
| `F-P4-M2` | MINOR | `Err` boundary | no | `BackendError::PidfdNotProvided` is a post-`clone3` `Err` that abandons a live child with no `ChildHandle`; `CLONE_PIDFD` makes it unreachable; inherited accepted P3 code | record |
| `F-P4-M3` | MINOR | outcome accuracy | yes | `LaunchOutcome::elapsed()` is captured before the `Observer` drops, so on the `EndNotObserved` path it understates real wall time by up to 5 000 ms | record |
| `F-P4-M4` | MINOR | poll failure | no | A persistent `poll` failure maps to `DeadlineReached`; if no deadline is due the loop returns immediately and spins on CPU until the next one | record |
| `F-P4-M5` | MINOR | test robustness | yes | `open_descriptor_count()` before/after races the default parallel test harness, where `backend::tests` concurrently opens pipes, files and processes | record |
| `F-P4-M6` | MINOR | `EndNotObserved` evidence | n/a | Real-loop coverage absent; no safe deterministic seam exists; pure-model coverage is exhaustive | accept for this slice |
| `F-P4-M7` | MINOR | coverage detail | n/a | No case for a stream exactly equal to its capture bound; the Phase A “more than 8 bytes is malformed” rule lives in the adapter and is tested neither portably nor in the real loop | record |
| `F-P4-M8` | MINOR | deadline origin | yes | The Phase A 5 000 ms bound starts at `Observer::new`, marginally after `clone3` and `release_child_side`, rather than at child creation | record |
| `F-P4-B1` | BACKLOG_NONBLOCKING | extra mechanism | yes | The absolute `guard_ms` backstop has no counterpart in the accepted plan; benign, and `F-P4-01` shows it is not the structural guarantee its comment claims | record |
| `F-P4-G1` | GATE_PENDING | runtime | n/a | P4 Linux runtime validation pending publication CI | expected |

### `F-P4-01` — an unbounded drain can starve every deadline

**IMPORTANT. Reachable on an ordinary path.**

`drain` ([`launch.rs:332-349`](../../crates/helm-launch/src/launch.rs#L332-L349)) is called from
`read_stream`, which the model requests once per readiness. It loops until `EAGAIN`, EOF or a read
error, with no iteration cap, no byte cap and no deadline recheck. `observe`'s deadline test and
the absolute `guard_ms` test both live **outside** it
([`launch.rs:611-644`](../../crates/helm-launch/src/launch.rs#L611-L644)), so neither is evaluated
while a drain is in progress.

For the drain to end, the reader must empty the pipe. It cannot if the writer refills it at least as
fast: the reader pays a 16 KiB `read` plus a SHA-256 update per chunk, the writer pays only a
`write`. Measured on this host (WSL2 Ubuntu, kernel 6.6, 32 CPUs), moving 1 000 MiB through a pipe
took 0.570 s with a non-hashing reader and 0.763 s with `sha256sum` as the reader — so the hashing
reader is ~35 % slower than a plain copy, and the write side alone is faster still. `sha2` here
selects a `cpufeatures`-detected x86 backend, so this is the *fast* hashing case. With reader and
writer on different CPUs the pipe stays non-empty and `read` does not return `EAGAIN`.

**Consequence.** A direct child that writes to one stream continuously — an ordinary verbose
program in a loop, no adversary required — prevents `launch` from ever evaluating the run deadline,
the grace deadline, the kill deadline, the drain deadline or the absolute guard. `launch` then does
not return until the child stops writing or closes the stream. Memory stays bounded (the prefix is
allocated once and excess bytes are discarded), so this is a liveness defect, not a leak.

This defeats plan §8.6, ADR §J's “no blocking wait exists anywhere in the lifecycle”, and the
module's own claim that the absolute guard makes “no unbounded wait after child creation” a
**structural** property. The `both_streams_are_drained_concurrently…` case does not exercise it: its
fixture allocates a `Vec` per 4 KiB chunk and flushes, so the reader wins comfortably; and a child
that blocks writing the *other* stream self-resolves, which is why the two-stream interleaving case
passes.

§58 lists “deadline can starve under continuous output” as an `IMPORTANT` example, and I classify it
there. The owner should note that its effect reaches the `BLOCKER` example “unbounded lifecycle
wait”, and may reasonably rate it higher.

### `F-P4-02` — `P4A-02`, confirmed and reclassified

**IMPORTANT.** Full analysis in §16. In short: reachable on the accepted `EndNotObserved` path;
adds a fixed second `POST_KILL_REAP_MS` before `launch` returns; sends a second, unrecorded direct
`SIGKILL`; contradicts ADR §J's “the child is left unreaped to the host”; the bound test asserts
`22 400 ms` where the accepted bound for the same plan is `12 400 ms`; and the pure model's own
randomized test asserts the accepted bound, so the model and the real loop disagree.

### `F-P4-03` — the privacy canary cannot pass on the cohort

**IMPORTANT. Deterministic, every run.**

`no_captured_byte_and_no_host_detail_reaches_the_receipt`
([`launch.rs:1491-1493`](../../crates/helm-launch/src/launch.rs#L1491-L1493)) asserts:

```
for forbidden in ["pid", "pidfd", "signature", "verified", "authentic"] {
    assert!(!text.contains(forbidden), "the receipt names `{forbidden}`");
}
```

where `text` is the receipt's exact bytes. The serializer unconditionally emits
`"backend":"linux_x86_64_clone3_pidfd_execveat"` as the third field
([`receipt.rs:152-154`](../../crates/helm-launch/src/receipt.rs#L152-L154);
`Backend::as_str` at [`model.rs:185`](../../crates/helm-launch/src/model.rs#L185)). That token
contains both `pidfd` and `pid`, so the loop fails on its first iteration for **every** receipt. The
P1 determinism anchor `FIXTURE_BYTES` shows the same substring in the hand-written expected bytes.

This is **not** a product defect: the receipt is correct and holds no pid. The substring assertion
is the defect. Because the case is Linux-only it has never executed, and it will make the hosted P4
lifecycle gate red before any product question is reached.

### `F-P4-04` — receipt determinism asserted over a scheduler-dependent field

**IMPORTANT. Intermittent.**

`the_receipt_is_deterministic_bounded_and_recomputable`
([`launch.rs:1451-1460`](../../crates/helm-launch/src/launch.rs#L1451-L1460)) launches the same
fixture twice and asserts the two receipts are byte-identical. Every field is fixed across the two
launches **except** `termination.group_sweep`, which is `issued` or
`not_issued_group_not_established` depending on whether the parent's `setpgid` won the race against
the child's `execveat` — the very outcome the [`P3R-21` review](HELM-LAUNCH-P3-P3R21-REVIEW.md)
established is scheduler-dependent, and which failed a hosted P3 publication in practice.

ADR §L is explicit: “**Not reproducible behaviour.** Execution is nondeterministic, so the same plan
will not produce the same receipt.” The test asserts a property the accepted contract disclaims. The
legitimate intent — proving no timestamp, duration, pid or path is in the receipt — is already
covered by the P1 determinism anchor and by the `p4_boundary` field scan, neither of which needs two
real launches.

### `F-P4-05` — `P4A-04`, confirmed and reclassified

**IMPORTANT.** Full analysis in §19.

### `F-P4-06` — `P4A-03` foreign-reap half, reclassified

**IMPORTANT.** Full analysis in §20.

### `F-P4-07` — accepted Level 3 O/R/S/T/P rows not delivered

**IMPORTANT.** Plan §14.3 is the specification the P4 stop condition (“Level 3 green”) refers to.
These O/R/S/T/P rows have no real-loop case in this candidate and none elsewhere in the crate:

| Accepted row | State |
|---|---|
| “early stdout close with CPU and poll-return bounds” (O5) | **absent** — this is the only committed guard that would catch `P4A-05` returning, and `P4A-05` was a real defect found during implementation |
| “POLLIN+POLLHUP, 200 repetitions” (O8) | **absent** in the real loop (pure model only) |
| “8 MiB concurrently under a 10 s bound” | **absent** — the committed case writes 512 KiB + 384 KiB |
| “`SIGSEGV` with `RLIMIT_CORE = 0` and with a core allowed; `SIGABRT`” (R3) | **absent** — `CLD_DUMPED` classification never runs against a kernel |
| “exit 127 after a successful start stays `exited`” (S3) | **absent** |
| “every integration test asserts `launch` returned within §8.6's bound” | **partial** — two cases assert an elapsed bound, and one of them uses the widened constant (`F-P4-02`) |

None of these shows the product defective. They are missing evidence for behaviour the P4 authority
called load-bearing, and the first is a missing regression guard for a defect that already occurred.

## 32. Author-reported findings, independently re-rated

| ID | Author | This review | Evidence |
|---|---|---|---|
| `P4A-01` spawn mask-restore semantic transition | reported | **NOT A FINDING** | Public `launch` cannot return `Err` after a child exists via this path: `spawn_for_lifecycle` destructures the errno to `_`. The P3 compatibility path preserves its historical behaviour exactly, with `ChildHandle::Drop` before `PreparedLaunch`'s drop, one `SIGKILL`, one bounded reap, `reaped` latched, no leak, no double observation, and every fd owned once (§5) |
| `P4A-02` double `POST_KILL_REAP_MS` | `MINOR` | **CONFIRMED, RECLASSIFIED `IMPORTANT`** (`F-P4-02`) | Reachable on the accepted `EndNotObserved` path; occurs before `launch` returns; exceeds the accepted §8.6 bound by a fixed 5 000 ms; adds an unrecorded second direct `SIGKILL`; the bound test asserts the widened constant; the pure model asserts the accepted one (§16) |
| `P4A-03` missing real-loop `EndNotObserved` / foreign-reap cases | `MINOR` | **SPLIT.** `EndNotObserved`: **CONFIRMED `MINOR`** (`F-P4-M6`). Foreign reap / `ECHILD`: **RECLASSIFIED `IMPORTANT`** (`F-P4-06`) | No safe deterministic seam exists for a child that survives `SIGKILL`, and the pure model is exhaustive there. But plan §14.3 already specifies a deterministic `SIGCHLD = SIG_IGN` real-loop mechanism for R4/T31, and it was not used (§20) |
| `P4A-04` disjunctive real sweep test | `MINOR` | **CONFIRMED, RECLASSIFIED `IMPORTANT`** (`F-P4-05`) | Both P-series cases can complete green having issued zero real group signals; nothing else in the crate exercises `kill_process_group`; the deterministic mechanism that exists is unreachable from `launch()` (§19) |
| `P4A-05` pre-commit busy-loop defect | reported fixed | **CONFIRMED FIXED**, independently proven from source; **but NO REGRESSION GUARD EXISTS** (`F-P4-07`) | `end_observed` removes the pidfd from the poll set after exactly one readiness, unconditionally on `POLLIN`, including when the model ignores the event; no other descriptor can remain permanently ready; an empty set sleeps rather than spins (§7). The accepted O5 CPU/poll-return case that would guard it is not implemented |

## 33. Pass conditions

| Condition | Required | Actual |
|---|---|---|
| `BLOCKER` | 0 | **0** |
| `IMPORTANT` | 0 | **7** |
| `P4_PUBLIC_LAUNCH_BOUNDARY` | SOUND | **SOUND** |
| `P4_ERR_VS_RECEIPT_BOUNDARY` | SOUND | **SOUND** |
| `P4_NO_EXEC_SUCCESS_CLAIM` | SOUND | **SOUND** |
| `P4_PHASE_A` | SOUND | **SOUND** |
| `P4_PHASE_B` | SOUND | **SOUND** |
| `P4_GROUP_SWEEP_GUARD` | SOUND | **SOUND** |
| `P4_SWEEP_BEFORE_REAP` | SOUND | **SOUND** |
| `P4_END_NOT_OBSERVED_LATCH` | SOUND | **SOUND** |
| `P4_DIRECT_CHILD_PIDFD_LIFECYCLE` | SOUND | **SOUND** |
| `P4_STREAM_DRAIN` | SOUND | **NOT SOUND** (`F-P4-01`) |
| `P4_POST_EXIT_DRAIN_BOUND` | SOUND | **SOUND** |
| `P4_TIMEOUT_GRACE_KILL` | SOUND | **SOUND** in the lifecycle; one unrecorded `Drop` signal (`F-P4-02`) |
| `P4_TOTAL_BOUND` | SOUND | **NOT SOUND** (`F-P4-02`) |
| `P4_RECEIPT_DETERMINISM` | SOUND | **SOUND** (serializer); test unsound (`F-P4-04`) |
| `P4_RECEIPT_PRIVACY` | SOUND | **SOUND** (product); test cannot pass (`F-P4-03`) |
| `P4_NO_AUTHENTICITY_CLAIM` | SOUND | **SOUND** |
| `P4_OUTCOME_OWNS_NO_DESCRIPTORS` | SOUND | **SOUND** |
| `P4_REAL_LOOP_MATCHES_PURE_MODEL` | SOUND | **NOT SOUND** (`F-P4-02`) |
| `P4_PUBLIC_API_NEGATIVE_PROOFS` | SOUND | **SOUND** |
| `P4_P3_REGRESSION_GATES` | PRESERVED | **PRESERVED** |
| P4 Linux runtime | may be GATE_PENDING | **`GATE_PENDING`** |

**Pass conditions are NOT met.**

## 34. Recommendation and next gate

**P4 CANDIDATE REQUIRES CORRECTION / OWNER REVIEW BEFORE PUBLICATION.**

Publishing as it stands would burn a hosted run on `F-P4-03`, which is red on the first P4 case
that produces a receipt, and would tell the owner nothing about the product. The four product and
evidence findings (`F-P4-01`, `F-P4-02`, `F-P4-05`, `F-P4-06`) and the coverage gap (`F-P4-07`) are
independent of that and need owner disposition on their own.

**Next gate: OWNER REVIEW OF THE INDEPENDENT P4 FINDINGS**, then a corrected P4 candidate, then one
bounded independent re-review, then the load-bearing hosted Linux P4 CI.

**P5 REMAINS NOT AUTHORISED. NO TRIAL #4 IS AUTHORISED. TRIAL #3 STAYS FROZEN
`MECHANISM_REJECTED` AND MUST NOT BE RERUN. THE COMPLETE helm-launch 0.1 MODULE IS NOT YET
PRODUCT-ACCEPTED.**

## 35. Boundary of this review

This document changes no product code, no test, no workflow, no manifest, no ADR, no plan and no
decision record. It pushes nothing, rebases nothing and rewrites no history. It authorises no
slice, no trial and no publication, and it weakens no accepted P4 rule. It is one independent
reading of `ef50e88..3d152ad` against the accepted contract, with the local validation of §30 and
without any Linux execution of this repository.
