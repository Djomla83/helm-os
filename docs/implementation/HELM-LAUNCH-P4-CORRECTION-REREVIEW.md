# HELM-LAUNCH P4 — BOUNDED INDEPENDENT P4 CORRECTION RE-REVIEW

> # 🔎 BOUNDED INDEPENDENT P4 CORRECTION RE-REVIEW
>
> **Session provenance. THIS RE-REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `b4c53418577b75fca75a71a04a4a00d63af90040` (the owner disposition) and
> `50a340ede53a964523c5cd4660e99d222c400333` (the P4 correction). It also authored no part of the
> P1, P2, P3 or P4 implementation chain, and no part of
> [`HELM-LAUNCH-P4-INDEPENDENT-LIFECYCLE-REVIEW.md`](HELM-LAUNCH-P4-INDEPENDENT-LIFECYCLE-REVIEW.md)
> at `351f985bb239496c1bd336e39279e8ff742de5cc`. It read the disposition and the correction under
> review, re-derived the accepted obligations from the
> [productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md) and
> [ADR-0024](../adr/ADR-0024-launch-authority.md), and re-established every claim below from the
> product source, the test source, the manifests and the workflow definition.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **P1 ACCEPTED. P2 ACCEPTED. P3 ACCEPTED. P4 AUTHORISED — CORRECTED CANDIDATE, NOT ACCEPTED.
> P5 NOT AUTHORISED. THE COMPLETE helm-launch 0.1 MODULE IS NOT YET PRODUCT-ACCEPTED.
> TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED` AND MUST NOT BE RERUN. TRIAL #4 IS NOT AUTHORISED.**

This is a **bounded** re-review. It does not repeat the first P4 review. It re-reviews the
correction against that review and the accepted owner disposition, and it independently checks that
the correction introduced no new `BLOCKER` or `IMPORTANT` defect.

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **0** |
| `MINOR` | 2 new, plus the 9 carried |
| `BACKLOG_NONBLOCKING` | 1 new, plus the 1 carried |
| `GATE_PENDING` | 1 |
| `F-P4-01` stream fairness | **FIXED** |
| `F-P4-02` `ChildHandle` finalization / total bound | **FIXED** |
| `F-P4-03` privacy test | **FIXED** |
| `F-P4-04` determinism test | **FIXED** |
| `F-P4-05` positive real group sweep | **FIXED** |
| `F-P4-06` real foreign reap / `ECHILD` | **FIXED** |
| `F-P4-07` accepted Level 3 rows | **FIXED** |
| `P4A-01` | **REMAINS NOT A FINDING** |
| `P4A-03` `EndNotObserved` real integration | **MINOR / OPEN**, carried as dispositioned |
| `P4A-05` non-spin | **FIXED + REGRESSION GUARDED** |
| `P4_CORRECTION_EINTR_SEMANTICS_SOUND` | **SOUND** |
| `F-P4-01_REAL_REGRESSION_PROBATIVE` | **PROBATIVE** (structural half deterministic; runtime half is the behavioural half — see `R-P4C-M1`) |
| `P4A-05_REGRESSION_GUARD_SOUND` | **SOUND** |
| `P4_CHILDHANDLE_FINALIZATION_STRUCTURE_SOUND` | **SOUND** |
| `P4_CHILDHANDLE_DISARM_TIMING_SOUND` | **SOUND** |
| `P4_ACCEPTED_TOTAL_BOUND_PRESERVED` | **PRESERVED** |
| `P4_TOTAL_BOUND_TEST_SOUND` | **SOUND** |
| `P4_NO_HIDDEN_SECOND_CLEANUP_SOUND` | **SOUND** |
| `P4_RECEIPT_DETERMINISM_TEST_PROBATIVE` | **PROBATIVE** |
| `P4_DETERMINISTIC_AUTHORITY_COORDINATION_SOUND` | **SOUND** |
| `P4_COORDINATION_SEAM_CONFINED` | **CONFINED** |
| `P4_ESCAPED_DESCENDANT_REAL_TEST_SOUND` | **SOUND** |
| `P4_CORRECTED_SWEEP_TARGET_SAFE` | **SAFE** |
| `P4_FOREIGN_REAP_TEST_HOST_ISOLATED` | **ISOLATED** |
| `P4_8MIB_DUAL_STREAM_TEST_PROBATIVE` | **PROBATIVE** |
| `P4_POLLIN_HUP_REGRESSION_PROBATIVE` | **PROBATIVE** |
| `P4_CLD_DUMPED_TEST_PROBATIVE` | **PROBATIVE** |
| `P4_CLD_DUMPED_HOST_PRECONDITION_HANDLING_SOUND` | **SOUND** |
| `P4_S3_REAL_TEST_PROBATIVE` | **PROBATIVE** |
| `P4_PURE_MODEL_UNCHANGED` | **UNCHANGED** (byte-identical) |
| `P4_RECEIPT_PRODUCT_SEMANTICS_UNCHANGED` | **UNCHANGED** (byte-identical) |
| `P4_P3_CHILD_CONTRACT_PRESERVED` | **PRESERVED** (byte-identical) |
| `P4_CORRECTION_UNSAFE_DELTA_ZERO` | **ZERO** |
| `P4_P3_REGRESSION_GATES_PRESERVED` | **PRESERVED** |
| `P4_CORRECTED_CI_REACHABILITY_SOUND` | **SOUND** |
| P4 Linux runtime validation | **`GATE_PENDING`** — no supported Linux execution occurred in this re-review |

**Classification: `HELM_LAUNCH_P4_CORRECTION_REREVIEW_PASSED_READY_FOR_PUBLICATION_CI`.**

All seven prior `IMPORTANT` findings are closed. No new `BLOCKER` or `IMPORTANT` was introduced.
The corrected P4 candidate may be published **once**, for load-bearing hosted Linux validation.

## 1. Target and starting state

| Item | Value |
|---|---|
| Accepted P3 base (published milestone) | `ef50e8865a4f14965a115c5dc26c72e45d4af2c9` |
| P4 authority | `41ac4f90e85da0688c9e76cdeec92ba904fcfe1d` |
| Original P4 implementation | `3d152ad0b7a422eb04160ba70697a457efd390c5` |
| First independent P4 review | `351f985bb239496c1bd336e39279e8ff742de5cc` |
| Owner disposition (full SHA) | `b4c53418577b75fca75a71a04a4a00d63af90040` |
| P4 correction | `50a340ede53a964523c5cd4660e99d222c400333` |
| This correction re-review | this commit, documentation only |
| Branch | `docs/helm-launch-architecture` |
| Local `HEAD` before this re-review | `50a340ede53a964523c5cd4660e99d222c400333` — **matched** |
| `origin/docs/helm-launch-architecture` | `ef50e8865a4f14965a115c5dc26c72e45d4af2c9` — **matched** |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **matched** |
| Relation to the remote branch | **5 ahead, 0 behind** — matched |
| Worktree before and after | **clean** |
| Fetch | **read-only**; nothing pushed, nothing rewritten, no history repaired |
| Re-review host | Windows 11, `cargo` 1.95.0; Linux **execution** not available |

Ancestry was verified as a linear chain of single-parent commits, not merely by
`merge-base --is-ancestor`:

```
ef50e88 -> 41ac4f9 -> 3d152ad -> 351f985 -> b4c5341 -> 50a340e
```

Every expected value in the instruction was matched exactly before any file under review was read.

## 2. Correction scope

`50a340e` touches **eight** files and **no documentation**:

| File | Change |
|---|---|
| `.github/workflows/helm-launch.yml` | +14, one new Linux step |
| `crates/helm-launch/README.md` | +16, two new paragraphs |
| `crates/helm-launch/src/backend/mod.rs` | +64, two test-only additions |
| `crates/helm-launch/src/backend/spawn.rs` | ±54, `ChildHandle` cleanup flag only |
| `crates/helm-launch/src/backend/tests.rs` | +149, additions only |
| `crates/helm-launch/src/launch.rs` | +1 256/−94 |
| `crates/helm-launch/tests/p3_boundary.rs` | +8, additions only |
| `crates/helm-launch/tests/p4_boundary.rs` | +207 |

### 2.1 The byte-identity claim, independently verified

The author claims twelve files are byte-identical between `3d152ad` and `50a340e`. This was checked
by comparing **blob object identities**, not by reading a diff:

| File | Blob identity at `3d152ad` vs `50a340e` |
|---|---|
| `crates/helm-launch/src/lifecycle.rs` | **IDENTICAL** |
| `crates/helm-launch/src/backend/child.rs` | **IDENTICAL** |
| `crates/helm-launch/src/backend/syscall.rs` | **IDENTICAL** |
| `crates/helm-launch/src/backend/injection.rs` | **IDENTICAL** |
| `crates/helm-launch/src/model.rs` | **IDENTICAL** |
| `crates/helm-launch/src/receipt.rs` | **IDENTICAL** |
| `crates/helm-launch/src/error.rs` | **IDENTICAL** |
| `crates/helm-launch/src/plan.rs` | **IDENTICAL** |
| `crates/helm-launch/src/lib.rs` | **IDENTICAL** |
| `crates/helm-launch/src/authority.rs` | **IDENTICAL** |
| `Cargo.toml` | **IDENTICAL** |
| `Cargo.lock` | **IDENTICAL** |

The claim holds exactly. Consequently:

| Required invariant | Result |
|---|---|
| Public API change | **NONE** — `lib.rs`, `model.rs`, `error.rs`, `plan.rs`, `authority.rs` unchanged; the one new function is `pub(crate)` under `cfg(test)` |
| Receipt schema change | **NONE** — `receipt.rs` and `model.rs` unchanged |
| Lifecycle policy change | **NONE** — `lifecycle.rs` unchanged |
| Child-window change | **NONE** — `child.rs`, `syscall.rs`, `injection.rs` unchanged |
| New `unsafe` outside the accepted boundary | **NONE** — see §18 |

No `FULL FRESH P4 REVIEW REQUIRED AGAIN` condition is met.

## 3. Owner disposition

[`b4c5341`](../DECISIONS.md#helm-launch-p4-independent-findings-disposition) was read in full. It is
documentation only ([`DECISIONS.md`](../DECISIONS.md), [`PROJECT_STATE.md`](../PROJECT_STATE.md)) and
changes no code, test, workflow, manifest, ADR contract, experiment or evidence. Every disposition
the re-review instruction requires is present and exact:

| Finding | Required disposition | Recorded disposition |
|---|---|---|
| `P4A-01` | NOT A FINDING | **NOT A FINDING**, "not to be rewritten for activity" |
| `F-P4-01` | IMPORTANT / MUST FIX | **IMPORTANT / ACCEPTED / MUST FIX** |
| `F-P4-02` / `P4A-02` | IMPORTANT / MUST FIX | **IMPORTANT / ACCEPTED / MUST FIX** |
| `F-P4-03` | IMPORTANT / TEST DEFECT / MUST FIX | **IMPORTANT / ACCEPTED / MUST FIX**, explicitly a test defect |
| `F-P4-04` | IMPORTANT / TEST CONTRACT DEFECT / MUST FIX | **IMPORTANT / ACCEPTED / MUST FIX**, explicitly a test defect |
| `F-P4-05` / `P4A-04` | IMPORTANT / MUST FIX WITH DETERMINISTIC REAL SWEEP EVIDENCE | **IMPORTANT / ACCEPTED / MUST FIX** over the "scheduler-dependent, disjunctive real sweep test" |
| `F-P4-06` | IMPORTANT / DETERMINISTIC REAL FOREIGN-REAP EVIDENCE | **IMPORTANT / ACCEPTED / MUST FIX**, split out of `P4A-03` |
| `F-P4-07` | IMPORTANT / MISSING LOAD-BEARING LEVEL-3 EVIDENCE | **IMPORTANT / ACCEPTED / MUST FIX** |
| `P4A-03` `EndNotObserved` real integration | MINOR / OPEN | **MINOR / OPEN**, carried |
| `P4A-05` | FIXED / REGRESSION GUARD REQUIRED | **CONFIRMED FIXED**; guard **required** as part of `F-P4-07` |
| Accepted total bound | UNCHANGED | **UNCHANGED**, "exactly one `POST_KILL_REAP_MS` contribution" |
| P5 | NOT AUTHORISED | **NOT AUTHORISED** |

## 4. `F-P4-01` — stream fairness

### 4.1 The defect, re-established from the original source

`3d152ad`'s `drain` was an unbounded loop:

```rust
fn drain(fd: BorrowedFd<'_>, mut absorb: impl FnMut(&[u8])) -> Drained {
    let mut buffer = [0_u8; READ_CHUNK_BYTES];
    loop {
        match rustix::io::read(fd, &mut buffer) {
            Ok(0) => return Drained::Eof,
            Ok(read) => { /* absorb, loop again */ }
            Err(Errno::INTR) => {}          // unbounded retry
            Err(Errno::AGAIN) => return Drained::WouldBlock,
            Err(errno) => return Drained::Failed(errno.raw_os_error()),
        }
    }
}
```

It returns only on end-of-file, a real error, or a momentarily empty pipe. A child that keeps its
pipe non-empty keeps the turn open, and the loop never reaches its monotonic time sample, its
deadline processing or its poll setup.

### 4.2 The corrected path

`read_once` ([`launch.rs:422`](../../crates/helm-launch/src/launch.rs)) performs **at most one**
`READ_BUFFER_BYTES` (65 536) read and returns. `Read::Progressed` is an explicit `return`, so a
successful read ends the turn. The only loop it contains is the bounded `EINTR` retry of §5.

Control then returns to `Observer::observe`, which on every turn, in this order:

1. asks the model for `facts()` and returns if the lifecycle finished;
2. samples monotonic time with `now_ms()`;
3. applies a due model deadline (`next_deadline()` and `now >= deadline`) and `continue`s;
4. checks the absolute guard (`now >= guard_ms`) and `continue`s;
5. computes `wait = min(next_deadline - now, MAX_POLL_WAIT_MS)` and sets up the poll.

Both stream descriptors, the status descriptor and — until the end is observed — the process
descriptor are in one `poll` set, and `wait_once` iterates **every** ready slot of that one
`revents` snapshot. If stdout and stderr are both ready, both are serviced in the same turn, each
for one bounded read. A continuously producing stdout therefore cannot starve stderr, the pidfd, the
run deadline, the grace deadline or the kill deadline: each of those is re-evaluated after a fixed
64 KiB of work.

The exec-status channel keeps its own fixed accumulator — `STATUS_RECORD_BYTES + 1` — and is read
through the same `read_once`, so it cannot drain without bound either.

### 4.3 Deterministic regression

[`p4_boundary.rs`](../../crates/helm-launch/tests/p4_boundary.rs) —
`a_ready_stream_is_read_at_most_once_per_observation_turn` — asserts against the compacted source
that `read_once` returns on `Read::Progressed`, contains **no** `loop{`, uses exactly the bounded
`for _ in 0..MAX_READ_EINTR_RETRIES` form, that `rustix::io::read(` appears exactly once in the whole
slice, that `read_once(` appears exactly three times (one definition, two callers), and that the
per-turn budget is the accepted plan constant. Reintroducing the unbounded drain fails this
deterministically, on every platform, in the `p4_boundary` suite that already runs in CI.

**`F-P4-01`: FIXED.**

## 5. `EINTR` retry — the new correction hotspot

`MAX_READ_EINTR_RETRIES = 16`. The semantics were audited against every accepted rule:

| Rule | Result |
|---|---|
| `EINTR` must not reset or extend a deadline | **HELD.** `read_once` touches no deadline. Deadlines are absolute values derived from `spawned_at`; `now_ms()` re-reads the monotonic clock on each turn |
| The retry cap must not invent a durable fact | **HELD.** Exhausting the 16 retries returns `Read::WouldBlock`, which is the *same* value as a genuine `EAGAIN`. It is **not** `Read::Failed`, so no `StreamReadFailed`, no `StatusReadFailed` and no `ReadFailed` completeness can arise from it |
| It must not become a false EOF | **HELD.** `Read::Eof` is produced only by `Ok(0)`. The model likewise treats only `Event::StreamEof` as end-of-file, and `Event::StreamReady { hangup: _ }` deliberately discards the hangup flag |
| Control must return to the outer loop and recompute time | **HELD.** `read_stream` puts the descriptor back (`capture.fd = Some(fd)`), emits no event, and the turn ends. The descriptor stays in the poll set and the next `poll` reports it ready again |
| A storm must not create an unbounded wait by restarting a full relative timeout | **HELD.** The `poll` timeout is recomputed every turn as `next_deadline.saturating_sub(now).min(250)` from the **absolute** deadline and a fresh monotonic sample. It is never restarted at its original relative value. `poll`'s own `EINTR` returns immediately and is handled the same way. The absolute guard `total_bound_ms + GUARD_SLACK_MS` is re-checked each turn and, once reached, drives the model with `u64::MAX` |

A pathological, sustained `EINTR` storm can make the loop turn without consuming bytes until a
deadline or the absolute guard fires. That is bounded work with a bounded end and no false product
fact; it is recorded as `R-P4C-B1`, non-blocking.

**`P4_CORRECTION_EINTR_SEMANTICS_SOUND`.**

## 6. The continuous-output regression

`continuous_output_cannot_starve_the_run_deadline`
([`launch.rs:1833`](../../crates/helm-launch/src/launch.rs)) runs the `spinner` fixture:

```rust
let chunk = vec![b'S'; 32 * 1024];
loop { if sink.write_all(&chunk).is_err() { return; } let _ = sink.flush(); }
```

| Requirement | Result |
|---|---|
| Capable of writing continuously past the run deadline | **YES.** An unconditional `loop`; it stops only when the write fails, which happens after the read end closes |
| Not naturally finite before the timeout | **YES.** There is no byte budget and no iteration count. The run deadline is 400 ms |
| Proves the real adapter reaches the run deadline | **YES.** `record.run_deadline_expired()` is asserted |
| Records deadline expiration | **YES**, same assertion |
| Advances through the required signal lifecycle | **YES.** `sigterm_sent()`, and `child_end == Signaled { signal: 15, core_dumped: false }` — the default disposition means the deadline's own `SIGTERM` ends it |
| Returns inside the accepted bound plus finite slack | **YES.** `accepted_total_bound_ms(400, 2_000) + 2_000 ms` |
| Not a large finite buffer that eventually drains | **CORRECT.** It also asserts `bytes_drained > 4 * READ_BUFFER_BYTES`, so a case that produced only a pipeful proves nothing and fails |

Against the original implementation the case hangs for as long as the child keeps the pipe
non-empty, and `launch` does not return until the child stops — which the fixture never does. The
one honest qualification is that "the child keeps the pipe non-empty" is a throughput race between a
`write`-looping child and a `read`-plus-SHA-256 parent, so the *runtime* half of this regression is
probabilistic rather than deterministic on an arbitrary runner. The **deterministic** half is the
structural `p4_boundary` case of §4.3, which cannot be raced. Recorded as `R-P4C-M1`, non-blocking:
the two halves together are probative, and no plausible reintroduction escapes both.

**`F-P4-01_REAL_REGRESSION_PROBATIVE`.**

## 7. `P4A-05` — the non-spin guard

Counter placement was checked against the **real** adapter loop, not a helper:

* `Metrics` is `#[cfg(test)]`-only, `pub(crate)`, and carries no decision the loop makes.
* `loop_turns` increments at the top of `Observer::observe`'s own `loop`.
* `poll_returns` increments in `Observer::wait_once`, only when `poll` reported at least one ready
  descriptor.
* `pollin_with_hangup` increments in `wait_once`'s `revents` walk.

`a_retained_writer_does_not_turn_the_post_exit_drain_into_a_spin`
([`launch.rs:1882`](../../crates/helm-launch/src/launch.rs)) drives the shape the finding names: the
`retainer` fixture starts a descendant that holds descriptors 1 and 2 for 20 s and then exits, so
the child ends while another process retains the stream writers. The case reads the counters from
the outcome of `run()`, which is the **public `launch`**, not a helper.

| Requirement | Result |
|---|---|
| Counter measures the real adapter loop | **YES** — `Observer::observe` / `Observer::wait_once`, reached only through `launch` |
| Child exits while another process retains writers | **YES** — `retainer` + `sleeper`, and the case asserts `Completeness::WriterRetainedAfterChildExit`, so a run where the drain did not really expire fails rather than passes |
| pidfd removed after end observation | **YES** — `end_observed` is set on the first `POLLIN`, unconditionally, and the pidfd is then omitted from the poll set while the handle stays owned for the reap |
| Post-exit drain remains bounded | **YES** — `POST_EXIT_DRAIN_MS`, and an empty poll set sleeps rather than spins |
| Finite meaningful ceiling | **YES** — 200 turns and 200 poll-returns, against a drain that produces about twenty bytes |
| Fails if an ended pidfd stays permanently ready | **YES** — every subsequent `poll` would return at once; the defect produced tens of thousands of turns against a ceiling of 200 |
| Not a counter around a bypassing helper | **CORRECT** |

**`P4A-05_REGRESSION_GUARD_SOUND`.**

## 8. `F-P4-02` — `ChildHandle` finalization

### 8.1 Structure

```rust
pub(crate) struct ChildHandle { pidfd: OwnedFd, pid: i64,
    end: Cell<Option<ChildEnd>>, reaped: Cell<bool>, cleanup_armed: Cell<bool> }
```

| Requirement | Result |
|---|---|
| `cleanup_armed: Cell<bool>` | **PRESENT** |
| Armed by default | **YES** — `cleanup_armed: Cell::new(true)` in `ChildHandle::new`, the only constructor |
| Crate-private disarm method | **YES** — `pub(crate) fn disarm_drop_cleanup_after_lifecycle(&self)`; no `pub` method, no re-export |
| `Drop` performs no cleanup if already reaped **or** disarmed | **YES** — `if self.reaped.get() || !self.cleanup_armed.get() { return; }` |
| P3 paths remain armed | **YES** — the sole caller of the disarm is `Observer::run`; `backend/mod.rs`'s P3 minimal launch never calls it |
| No `mem::forget` | **CONFIRMED** — absent from the whole crate |
| No `ManuallyDrop` | **CONFIRMED** — absent |
| No fd leak | **CONFIRMED** — no `into_raw_fd`; the `OwnedFd` still drops with the handle on every path, disarmed or not |
| No background reaper | **CONFIRMED** — no thread is spawned in product code |
| No public method | **CONFIRMED** |

`p4_boundary`'s `the_total_bound_counts_one_post_kill_wait_and_the_drop_guard_is_handed_back`
independently pins the armed-on-construction initializer, the exact early-return condition, the
single definition of the disarm, and the absence of `mem::forget`, `ManuallyDrop` and `into_raw_fd`
from `spawn.rs`.

**`P4_CHILDHANDLE_FINALIZATION_STRUCTURE_SOUND`.**

### 8.2 Disarm timing — traced, not assumed

Method existence was not accepted as proof. Every control-flow path through the public entry point
was traced.

```rust
pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError> {
    let started = Instant::now();
    let spawned = backend::spawn_for_lifecycle(authorized).map_err(to_launch_error)?;
    Ok(Observer::new(spawned).run(started))
}
```

* `launch` contains exactly **one** `?`, and it is the pre-child spawn. `p4_boundary`'s
  `launch_returns_err_only_before_a_child_exists` asserts that count against the parsed function
  body — and the correction *strengthened* that test, replacing a newline-delimited body scan with a
  brace-matching one that cannot silently widen the claim to the rest of the file.
* After that line a child exists and **no** path returns `Err`. There is no `map_err`, no second `?`,
  no early return.
* `Observer::run` calls `self.observe()` and then `self.child.disarm_drop_cleanup_after_lifecycle()`.
* `observe()` returns `Facts`, **not** `Result`. It has no `?`, no fallible early return and no
  panic-like propagation. Its only exit is `if let Some(facts) = self.life.facts() { return facts; }`.
* `Lifecycle::facts()` returns `Some` **only** when `phase == Phase::Finished`.
* `Phase::Finished` is assigned in exactly one place: `after_reap`, reached only by
  `Event::Reaped(_)`, which the adapter emits only in response to `Action::Reap`.
* `Action::Reap` is pushed from exactly two places, both in Phase C: `enter_cleanup_if_done` (the
  no-authority branch) and `after_probe` (both probe outcomes).

Therefore **every** path that reaches the disarm has completed Phase C and its one allowed
non-blocking reap attempt, and **no** path leaves `observe()` early.

Observation errors were checked specifically. A stream or status read failure produces
`Event::StreamReadFailed` / `Event::StatusReadFailed`, which are ordinary model inputs that close
that descriptor's slot and let the lifecycle progress to Phase C; they are not returns. A `poll`
that cannot report readiness drives `Event::DeadlineReached` rather than returning an error. A
signal failure is discarded (`let _ = ...`) by design, because the model records that the action was
taken, not that it succeeded. So:

* no path disarms while the lifecycle still expects `Drop` cleanup — the disarm is strictly after
  `observe()` returns, which is strictly after Phase C;
* no path completes the full P4 lifecycle and leaves `Drop` armed — `run` is the only consumer of
  `Observer`, and it disarms unconditionally.

`p4_boundary` additionally asserts that `disarm_drop_cleanup_after_lifecycle` is called from exactly
one place in the slice, and that within `run`'s body the offset of `let facts = self.observe();`
precedes the offset of the disarm.

**`P4_CHILDHANDLE_DISARM_TIMING_SOUND`.**

## 9. The accepted total bound

```rust
fn total_bound_ms(timeout_ms: u32, grace_ms: u32) -> u64 {
    SPAWN_CONFIRM_TIMEOUT_MS
        .saturating_add(u64::from(timeout_ms))
        .saturating_add(u64::from(grace_ms))
        .saturating_add(POST_KILL_REAP_MS)
        .saturating_add(POST_EXIT_DRAIN_MS)
}
```

This is the accepted formula of plan §8.6 and ADR-0024 §J exactly, with **one**
`POST_KILL_REAP_MS`. The first review's `F-P4-02` — the widened implementation constant — is gone.

### 9.1 The `EndNotObserved` path, traced

1. The lifecycle's `kill_deadline` (`now + POST_KILL_REAP_MS`, armed by the single `SIGKILL` it is
   allowed to send) expires without an observed end.
2. `end_not_observed` is latched, `child_end = Some(ChildEnd::EndNotObserved)`, and any open stream
   slot becomes `ReadStoppedChildEndNotObserved`.
3. `enter_cleanup_if_done` sees `observation_done` true through `end_not_observed` and enters
   Phase C — `ProbeReaped` under established authority, otherwise `Reap` directly.
4. The non-blocking `reap()` may return `Reap::NothingAvailable`.
5. `after_reap` matches `(Some(latched), _) => latched`, so `EndNotObserved` **stands**, and sets
   `Phase::Finished`.
6. `observe()` returns; `run` calls the disarm.
7. `ChildHandle::drop` sees `!cleanup_armed` and returns immediately. The `OwnedFd` pidfd and every
   pipe end still close. There is **no** second `SIGKILL`, **no** second `wait`, and **no** second
   `POST_KILL_REAP_MS`.

A child whose end was never observed is left unreaped to the host, which is precisely the accepted
`T40` / `end_not_observed` contract.

### 9.2 No hidden second cleanup

Every direct-child `SIGKILL` site in the crate:

| Site | Path | Status |
|---|---|---|
| `launch.rs:672` `pidfd_send_signal(self.child.pidfd(), signal)` | the lifecycle's own `SendSigterm` / `SendSigkill` | the **only** signals on the public P4 path |
| `spawn.rs:686` `self.send_sigkill()` in `Drop` | abandoned-child cleanup | now gated on `reaped \|\| !cleanup_armed`; unreachable after a completed P4 lifecycle |
| `backend/mod.rs:436` `child.send_sigkill()` | the **P3** minimal launch's own cleanup | unchanged, still armed, separately intact |

Every bounded reap:

| Site | Kind | Status |
|---|---|---|
| `launch.rs:716` `self.child.reap_once()` | non-blocking, via `Action::Reap` | exactly one per lifecycle |
| `spawn.rs:687` `reap_within(POST_KILL_REAP_MS)` in `Drop` | blocking, bounded | same gate; unreachable after P4 |
| `spawn.rs:625` `reap_within` | P3 helper | unchanged |

The public P4 path therefore contains exactly the lifecycle-authorised post-kill wait, and P3's
cleanup guard behaviour is separately intact.

**`F-P4-02`: FIXED. `P4_ACCEPTED_TOTAL_BOUND_PRESERVED`. `P4_NO_HIDDEN_SECOND_CLEANUP_SOUND`.**

### 9.3 Total-bound test probity

`accepted_total_bound_ms` in the test module spells the accepted formula out from the named
constants rather than calling `total_bound_ms`, so a future widening of the implementation **fails**
the assertions instead of travelling with them. `assert_implementation_bound_is_the_accepted_bound`
compares the two over three `(timeout, grace)` shapes including the plan maxima, and pins
`GUARD_SLACK_MS == 5_000`.

| Check | Result |
|---|---|
| Independently reconstructs the accepted formula | **YES** |
| Hidden extra `POST_KILL_REAP_MS` | **NONE** — one term, and `p4_boundary` asserts the token appears exactly once in `total_bound_ms`'s body |
| Inflated arbitrary slack | **NO** — `TEST_SLACK_MS = 2_000`, documented as scheduling slack and never added to the product guard |
| Run timeout + grace + kill wait | **COVERED** — `the_accepted_total_bound_holds_on_the_run_timeout_path`, `stubborn` ignores `SIGTERM` so the whole sequence is really paid |
| Pre-exec timeout | **COVERED** — `the_accepted_total_bound_holds_on_the_pre_exec_timeout_path`, which also asserts `elapsed >= SPAWN_CONFIRM_TIMEOUT_MS`, so the bound is really waited out |
| Retained writer / post-exit drain | **COVERED** — `the_accepted_total_bound_holds_when_a_writer_is_retained_after_the_exit`, gated on `WriterRetainedAfterChildExit` |
| Structural tests do not silently widen the bound | **CORRECT** |

The disarm claim additionally has **runtime** evidence, not only structural:
`a_disarmed_drop_guard_neither_signals_nor_waits`
([`backend/tests.rs`](../../crates/helm-launch/src/backend/tests.rs)) disarms a handle holding a live
60 s child, drops it, and asserts both that the child **survived** (so no unrecorded `SIGKILL`
exists) and that the drop took under 250 ms (so no second bounded wait exists), then cleans the
child up itself. `the_drop_guard_is_armed_by_default_and_cleans_up_an_abandoned_child` and
`a_reaped_child_is_never_signalled_again_by_a_drop` hold the other two corners.

**`P4_TOTAL_BOUND_TEST_SOUND`.**

## 10. `F-P4-03` — the privacy test

The banned raw substring `pid` is gone, as it had to be: the accepted mechanism identifier
`linux_x86_64_clone3_pidfd_execveat` contains it, so the original canary could never pass on the
cohort.

`no_captured_byte_and_no_host_detail_reaches_the_receipt` now asserts:

* each prohibited name as a **structured JSON key** (`"pid":`, `"pidfd":`, `"fd":`, `"host_path":`,
  `"path":`, `"cwd":`, `"argv":`, `"environment":`, `"timestamp":`, `"time":`, `"elapsed":`,
  `"duration":`, `"signature":`, `"signed":`, `"verified":`, `"authentic":`) is **absent**;
* the output canary is absent from the receipt's **exact bytes**, from the receipt's `Debug` and from
  `LaunchOutcome`'s `Debug`, and no fragment of it leaks;
* no `/` reaches the receipt, so no host path can;
* the accepted mechanism identifier is **present and unchanged** —
  `"backend":"linux_x86_64_clone3_pidfd_execveat"` — which is what makes the key-based claim the
  right one rather than a weakened one;
* the canary **does** appear in the in-memory prefix, so the case cannot pass by capturing nothing.

Product receipt code is untouched: `receipt.rs` and `model.rs` are byte-identical. The accepted
mechanism identifier and the receipt vocabulary are unchanged.

**`F-P4-03`: FIXED.**

## 11. `F-P4-04` — the determinism test

`the_receipt_is_deterministic_bounded_and_recomputable` now distinguishes serializer determinism
from runtime fact equality:

| Accepted property | Assertion |
|---|---|
| The same fixed `ReceiptRecord` serialises to the same exact bytes | `LaunchReceipt::from_record(receipt.record().clone())` produces byte-equal output and an equal digest, for each of two real launches |
| Every real receipt's digest commits to its own published bytes | `receipt.sha256() == Digest::of(receipt.exact_bytes())` |
| Size bound | `exact_bytes().len() <= MAX_RECEIPT_BYTES` |
| Equal runtime records imply equal bytes | asserted in the `==` branch |
| Unequal runtime records must **not** be normalised to equal bytes | asserted in the `!=` branch, which also pins that `group_sweep` is the only accepted scheduler-dependent difference |
| Two ordinary launches are **not** required to be byte-identical | **CORRECT** — that requirement, which ADR-0024 §L explicitly disclaims, is gone |
| No truthful scheduler fact is normalised away | **CORRECT** |

### 11.1 Fixed-record source

The fixed record is `receipt.record().clone()` — a record the **real** production path built from a
real launch, cloned through the ordinary model type. It is then fed to the **actual production
serializer**, `crate::receipt::LaunchReceipt::from_record`. There is no shadow serializer, no
hand-written JSON oracle, and no fixture record constructed outside model semantics.

**`F-P4-04`: FIXED. `P4_RECEIPT_DETERMINISM_TEST_PROBATIVE`.**

## 12. `F-P4-05` — deterministic positive group sweep

### 12.1 The coordination seam

The mechanism is exactly the one the instruction anticipates, and nothing more:

* `backend::stall_before_exec_fault()` requests the **existing** `injection::MODE_STALL_BEFORE_EXEC`
  with `stage::NONE` and `errno: 0`. No new injection mode, no new child syscall.
* `backend::spawn_for_lifecycle_with_fault` is `spawn_for_lifecycle` with that fault, returning the
  retained stdin write end via the **pre-existing** `release_child_side(retain_stdin_writer: bool)`
  parameter.
* The child blocks immediately before `execveat` in `read(0)`, so the parent's `setpgid(child, child)`
  — its first call after `clone3` — cannot lose the `P3R-21` race and be answered `EACCES`.
* `launch_with_stall_coordination` returns `(outcome, group_authority_established)`, so the caller
  **asserts** the precondition instead of assuming it. `coordinated_run` does exactly that.
* On release, the retained writer is dropped, the child sees end-of-file and walks into the ordinary
  `execveat`. The observation that follows is `Observer::new(spawned).run(started)` — the **same**
  `Observer`, the same model, the same Phase C as the product path.
* Authority is a fact produced by the **real parent's own** `setpgid` succeeding. No product
  authority bit is forced by test code, and nothing is inferred from child state.

**`P4_DETERMINISTIC_AUTHORITY_COORDINATION_SOUND`.**

### 12.2 Seam confinement

| Requirement | Result |
|---|---|
| Test-only | **YES** — `launch_with_stall_coordination` is `#[cfg(all(test, all(feature = "test-fault-injection", debug_assertions)))]`; `mod coordinated` is `#[cfg(all(feature = "test-fault-injection", debug_assertions))]` **inside** the `#[cfg(test)] mod tests` |
| Non-default feature and debug-assertions gated | **YES**, all three `test-fault-injection` occurrences in `launch.rs` carry the full two-condition gate |
| Absent from release | **YES.** `cfg(test)` excludes it from any consumer build. The backend helpers (`stall_before_exec_fault`, `spawn_for_lifecycle_with_fault`) are gated on `debug_assertions`, so even a release `--all-features` build excludes them |
| Not publicly selectable | **YES** — `pub(crate)`, and `p4_boundary` asserts `pubfnlaunch_with_stall_coordination` does not occur |
| Not compiled into normal release behaviour | **YES** |
| Public `launch` cannot reach it | **YES** — `p4_boundary`'s `the_test_only_coordination_seam_is_not_reachable_from_the_public_launch` asserts the gate immediately precedes the definition, that `launch`'s body names none of `stall`, `Fault`, `fault`, `coordination`, and that the product `spawn_for_lifecycle` still calls `spawn::spawn(&prepared, Fault::default())` |

The injection-confinement inventory in [`p3_boundary.rs`](../../crates/helm-launch/tests/p3_boundary.rs)
gained `src/launch.rs`. That is the legitimate consequence of reusing the existing gate, not a
weakening: the same test still requires **every** occurrence to be the full two-condition gate, and
all three occurrences in `launch.rs` additionally sit under `cfg(test)`. The P3 release-marker
absence proof (`tools/helm_launch_injection_proof.py`) and the "release library instantiates no
backend" step are unchanged and unaffected, because nothing new can reach a release object.

**`P4_COORDINATION_SEAM_CONFINED`.**

### 12.3 The positive real sweep

`established_authority_issues_exactly_one_real_sweep_that_reaches_its_own_group`:

| Requirement | Result |
|---|---|
| `group_authority_established == true`, deterministically | **YES** — `coordinated_run` asserts it before anything else |
| Fixture executes after release | **YES** — `exec_status == Indeterminate(StatusEofWithoutRecord)`, the clean-EOF shape of an ordinarily executed child (and still not exec success, per `T36`) |
| Same-group descendant exists | **YES** — `grouper` spawns `sleeper` with the inherited group and reports its pid |
| Direct child exits normally | **YES** — `child_end == Exited { code: 0 }` |
| Phase C real `WNOWAIT` probe runs | **YES** — established authority routes `enter_cleanup_if_done` to `Action::ProbeReaped` |
| Probe does not `ECHILD` | **YES** — otherwise the disposition would be `NotIssuedChildAlreadyReaped` |
| Exactly one real process-group `SIGKILL` | **YES** — `group_sweep == GroupSweep::Issued`, which the model assigns once, on the single `Probe::Unreaped` transition |
| Sweep before the direct-child reap | **YES** — `after_probe` pushes `SweepGroup` then `Reap`, and the adapter performs the model's actions in order; the pure model pins this for every path |
| Target is the child's dedicated group | **YES** — see §12.5 |
| No scheduler-dependent disjunction | **CORRECT** — there is no "either outcome" branch anywhere in this case |
| Real effect observed | **YES** — the same-group descendant is polled for up to 2 s and must be gone |
| Not a containment claim | **CORRECT** — stated in the case, and the escaped case is the other half of the same non-claim |

Also asserted: `!sigterm_sent()` and `!sigkill_sent()`, so the sweep is shown on a **normal exit**
path, which is the sharpest `T30` shape.

**`F-P4-05`: FIXED.**

### 12.4 The escaped descendant

`a_descendant_that_left_the_group_survives_the_sweep` uses the same deterministic authority
foundation through `coordinated_run`, and the same real sweep (`GroupSweep::Issued` asserted).

* The escaped descendant leaves the target group through `Command::process_group(0)` — a
  `setpgid(0, 0)` in the forked child, making it its own group leader. That is the accepted
  equivalent of `setsid` for this purpose.
* It must **survive** the launcher's group sweep; the assertion message states the claim explicitly:
  *"the sweep is best-effort cleanup, never containment."*
* The harness then kills and confirms the death of both descendants (`cleanup(same)`,
  `cleanup(escaped)`, then `assert!(!alive(escaped), "the harness must leave no descendant of its own
  running")`). `cleanup` sends `SIGKILL` and polls `/proc/<pid>` for up to 2 s.

**`P4_ESCAPED_DESCENDANT_REAL_TEST_SOUND`.**

### 12.5 Sweep target safety

```rust
fn sweep_group(&self) {
    let Ok(pid) = i32::try_from(self.child.pid()) else { return };
    let Some(group) = Pid::from_raw(pid) else { return };
    let _ = kill_process_group(group, Signal::KILL);
}
```

| Hazard | Why it cannot occur |
|---|---|
| Target `0` (the caller's own group) | `Pid::from_raw` is built on `NonZeroI32` and returns `None` for `0`, which returns early |
| Target `-1` (every permitted process) | `self.child.pid()` is the `clone3` return value in the parent, always positive; `i32::try_from` rejects anything out of range |
| Parent group, cargo-test group, runner-shell group, any unrelated group | The identifier is the **direct child's own pid**, and the sweep is reached only under `GroupAuthority::Established`, i.e. only when the launcher's own `setpgid(child, child)` succeeded. That call creates a **new** group whose id is the child's pid, which is unique among live processes. No group id is ever discovered by asking the system what group anything is in |
| pid reuse between probe and signal | The `WNOWAIT` probe immediately before the sweep establishes the child is still collectable, so the pid is held by an unreaped child and cannot have been reused |

No corrected real-sweep test can signal an unrelated host process.

**`P4_CORRECTED_SWEEP_TARGET_SAFE`.**

## 13. `F-P4-06` — the foreign reaper

### 13.1 Host isolation

The `SIGCHLD = SIG_IGN` disposition is carried by a **dedicated subprocess**, never by the cargo test
harness:

```c
int main(int argc, char **argv) {
    if (argc < 2) { return 2; }
    signal(SIGCHLD, SIG_IGN);
    execv(argv[1], &argv[1]);
    return 127;
}
```

The outer case builds every fixture **first**, while it can still wait for a compiler, then runs the
wrapper on this test binary with `--exact launch::tests::coordinated::foreign_reaper_inner_case
--ignored --test-threads 1`, passing the prebuilt paths in the environment. The inner case is
`#[ignore]`d so it never runs in the ordinary harness. A `SIG_IGN` disposition survives `execve`, so
the inner process — and only it — has the condition. No other Rust test is affected and there is no
global race.

The outer case asserts the subprocess succeeded, that the inner marker
`HELM-P4-FOREIGN-REAP-INNER: ok` was printed, **and** that the output contains `1 passed`, so a run
in which the inner case never executed fails rather than passing vacuously.

### 13.2 Authority first, then the real `ECHILD`

The inner case uses `coordinated_run`, so group authority is **established before the child is
released to execution**. This is what makes the case load-bearing: without it, "no sweep" would be
explained by missing authority rather than by the `ECHILD` guard. Because authority is established,
`enter_cleanup_if_done` routes to `Action::ProbeReaped`, and `Observer::probe` really issues
`waitid(P_PIDFD, …, WEXITED | WNOHANG | WNOWAIT)`. With the kernel auto-reaping this process's
children, that call is answered `ECHILD`, which maps to `Probe::AlreadyReaped`.

### 13.3 Expected facts

| Requirement | Assertion |
|---|---|
| `group_sweep == NotIssuedChildAlreadyReaped` | asserted |
| `child_end == EndUnobservable` | asserted |
| `EndNotObserved` was not already latched | the case uses bounds `(10_000, 1_000)` with the `quiet` fixture, which exits immediately; no `SIGKILL` is ever sent, so no kill deadline exists to latch. `!sigterm_sent()` and `!sigkill_sent()` are both asserted |
| Zero real group signal calls | `NotIssuedChildAlreadyReaped` is assigned on the `Probe::AlreadyReaped` branch, which pushes `Action::Reap` and **never** `Action::SweepGroup`; `sweep_group` is reachable only from `Action::SweepGroup` |
| `launch` returns boundedly | the subprocess runs to completion and the outer case asserts its exit status |

**`F-P4-06`: FIXED. `P4_FOREIGN_REAP_TEST_HOST_ISOLATED`.**

## 14. `F-P4-07` — the Level 3 evidence inventory

The first review named six missing rows; the owner disposition and the re-review instruction add the
fairness, positive-sweep and foreign-reap rows. Each accepted row was mapped independently to a
committed **real Linux** case. Pure lifecycle model tests were **not** counted where the plan
requires real integration.

| Accepted row | Committed real case | State |
|---|---|---|
| O5 — CPU / poll-return non-spin bound (the `P4A-05` guard) | `launch::tests::a_retained_writer_does_not_turn_the_post_exit_drain_into_a_spin` | **DELIVERED** (§7) |
| O8 — `POLLIN`+`POLLHUP`, 200 repetitions, real path | `launch::tests::readiness_with_a_hangup_reads_every_byte_before_end_of_file` | **DELIVERED** (§14.1) |
| O — 8 MiB simultaneous dual stream under 10 s | `launch::tests::eight_mebibytes_on_both_streams_at_once_are_drained_in_full` | **DELIVERED** (§14.2) |
| R3 — `CLD_DUMPED` with and without a core; `SIGABRT` | `launch::tests::a_signalled_child_keeps_its_signal_number_and_its_core_flag` | **DELIVERED** (§14.3) |
| S3 — exit 127 after a successful start stays `Exited` | `launch::tests::exit_127_from_a_started_image_is_an_exit_and_not_a_pre_exec_failure` | **DELIVERED** (§14.4) |
| Continuous-output deadline fairness | `launch::tests::continuous_output_cannot_starve_the_run_deadline` | **DELIVERED** (§6) |
| T30 — positive real group sweep with established authority | `launch::tests::coordinated::established_authority_issues_exactly_one_real_sweep_that_reaches_its_own_group` | **DELIVERED** (§12.3) |
| T30/T31 — escaped descendant survives | `launch::tests::coordinated::a_descendant_that_left_the_group_survives_the_sweep` | **DELIVERED** (§12.4) |
| R4 / T31 — real foreign reap, `ECHILD` | `launch::tests::coordinated::a_foreign_reaper_suppresses_the_sweep_and_leaves_the_end_unobservable` | **DELIVERED** (§13) |
| §8.6 bound asserted by the integration cases | three shapes, each from the independently spelled-out formula | **DELIVERED** (§9.3) |

No accepted row from the finding's inventory remains missing.

### 14.1 `POLLIN` + `POLLHUP` ×200

200 real product-path repetitions, matching the accepted plan count. Each launches the `writer`
fixture, which writes 3 000 bytes to stdout and 1 500 to stderr and exits at once, so readiness and
hangup commonly arrive in the same `revents`. Each case asserts exact byte counts on both streams,
`CompleteAtEof`, the exact stdout digest and `Exited { code: 0 }` — so bytes are proven consumed
**before** end-of-file processing.

Crucially, the accumulated test-only counter `pollin_with_hangup` must be **greater than zero**
across the 200 cases, with the message *"no repetition ever saw readiness together with a hangup, so
this case proved nothing"*. A run that merely performed write-and-close without ever producing the
combined readiness therefore fails rather than passing. The product-side guarantee is independent:
`Event::StreamReady { stream, hangup: _ }` discards the hangup flag and always asks for a read, and
only `Ok(0)` becomes EOF.

**`P4_POLLIN_HUP_REGRESSION_PROBATIVE`.**

### 14.2 8 MiB dual stream

The `dual` fixture spawns a thread that writes 8 MiB to stderr in 8 KiB chunks **while** the main
thread writes 8 MiB to stdout, each blocking on its own pipe, and joins at the end. Production is
genuinely concurrent: there is no "write all stdout, then all stderr" sequence, and a launcher that
drained one stream to end-of-file before touching the other would **deadlock** rather than fail an
assertion.

Exactness is asserted on both streams: `bytes_drained == 8 388 608`, `CompleteAtEof`,
`drained_sha256 == Digest::of(&vec![marker; VOLUME])` over the full 8 MiB, prefix lengths of exactly
4 096 and 2 048, both prefixes flagged truncated, and every prefix byte equal to its stream's marker.
The receipt is additionally checked to carry no raw payload at this volume. The whole launch must
complete in under 10 s, which is the accepted bound for the row.

It is an ordinary `launch::tests::` case with no feature gate, so the standing Linux CI step runs it.

**`P4_8MIB_DUAL_STREAM_TEST_PROBATIVE`.**

### 14.3 R3 — `CLD_DUMPED`

The consumer assertions are preceded by a **producer self-test**, which is the accepted `T50`
obligation. The same C crasher image is run **directly**, outside `helm-launch`, and the host's
behaviour is read through `std::os::unix::process::ExitStatusExt::core_dumped()` — real Linux
`waitid` semantics, not a shell exit-code inference.

| Environment assumption | Handling |
|---|---|
| `RLIMIT_CORE` | The image raises its **own** limit rather than depending on the runner's. Under `helm-launch` the case additionally asserts the fixture reported `core_allowed=1`, so an environment where the raise failed is named rather than silently passing |
| `core_pattern` | Read from `/proc/sys/kernel/core_pattern` and printed in **every** failure message, so a piping or non-dumping host is diagnosable from the log alone |
| Helper availability | The C crasher is built through the existing `c_fixture_binary` path, which is the same `require_tool` mechanism P3 already uses |
| Working directory / writable location | Each case runs in its own `scratch_dir` |
| Silent skip | **NONE.** There is no `return`, no `#[ignore]` and no `cfg`-skip. A host that cannot dump makes the case **fail**, with the message `TEST ENVIRONMENT PRECONDITION NOT MET: … that is a host fact, not a helm-launch defect.` The failure is therefore unambiguously classifiable as a **named test environment failure** rather than a product defect |
| Unbounded / large core artifacts | The crasher is a tiny C image, and `remove_core_files` deletes `core` and `core.*` from the scratch directory after both the self-test and the product case |

Receipt preservation is asserted for all three shapes: `Signaled { signal: 11, core_dumped: false }`,
`Signaled { signal: 6, core_dumped: false }` and `Signaled { signal: 11, core_dumped: true }`. The
actual terminating signal survives whichever `waitid` class the kernel reports, and `CLD_DUMPED` is
not flattened into `CLD_KILLED`.

**`P4_CLD_DUMPED_TEST_PROBATIVE`. `P4_CLD_DUMPED_HOST_PRECONDITION_HANDLING_SOUND`.**

### 14.4 S3

The accepted S3 obligation is recorded in plan row `T35`: *"a structured pre-exec failure carries
stage and errno and is never an exit status … `S3` `Exited:127` distinct … helper exiting 127 is
`Exited`"*, and in `T38`. So "exit 127 after a successful start stays `Exited`" **is** the accepted
S3 shape, not a substitute for a different missing signal case — the signal cases are R3, which §14.3
delivers separately and in full.

The committed case implements exactly that shape: `exec_status ==
Indeterminate(StatusEofWithoutRecord)` (never a `pre_exec_failure` synthesised from the exit status),
`child_end == Exited { code: 127 }`, and neither signal sent. The `EACCES` case elsewhere in the
suite is the contrast where a structured record really did arrive.

**`P4_S3_REAL_TEST_PROBATIVE`.**

## 15. `EndNotObserved` carry

Not reopened. `P4A-03-EndNotObserved` stays **MINOR / OPEN** as the owner dispositioned: no safe
natural Linux child ordinarily survives `SIGKILL` long enough to produce the real path, and no
dangerous kernel or privilege mechanism was manufactured to close it — correctly.

The exhaustive **pure model** coverage of the path is intact: `lifecycle.rs` is byte-identical, so
the latch, the `ReadStoppedChildEndNotObserved` stream disposition, the "a latched `EndNotObserved`
stands" rule in `after_probe` and `after_reap`, and the completion-path enumeration all survive
unchanged.

What the correction was required to add, it added: the hidden second `Drop` wait on that path is
proven gone **structurally** (§8.2, §9.1, plus `p4_boundary`) and, for the drop guard itself, at
**runtime** on Linux (§9.3, `a_disarmed_drop_guard_neither_signals_nor_waits`).

## 16. `P4A-01` regression

The prior independent verdict was **NOT A FINDING**, and the correction did not regress it.
`spawn_for_lifecycle` is unchanged: `mask_restore_errno` is still deliberately dropped with its
original reasoning, so a post-clone mask-restore failure still cannot become an `Err` from the public
path, and the P3 minimal launch still keeps its historical error instead. `spawn.rs`'s only change is
the `ChildHandle` cleanup flag; the mask-restore boundary, the P3 compatibility cleanup and fd
ownership are untouched. No unrelated rewrite was performed, as instructed.

**`P4A-01`: REMAINS NOT A FINDING.**

## 17. Unchanged product surfaces

| Surface | Result |
|---|---|
| Pure lifecycle model (`lifecycle.rs`) | **BYTE-IDENTICAL.** The accepted policy was not widened to accommodate the correction; total-bound semantics, the `POST_KILL_REAP_MS` constant, the latch and Phase C are all unchanged → **`P4_PURE_MODEL_UNCHANGED`** |
| Receipt product model (`model.rs`, `receipt.rs`) | **BYTE-IDENTICAL.** No durable schema field was removed, added or renamed to satisfy a test; the privacy and determinism corrections are entirely test-side → **`P4_RECEIPT_PRODUCT_SEMANTICS_UNCHANGED`** |
| Child contract (`child.rs`, `syscall.rs`) | **BYTE-IDENTICAL.** The existing stall mode is reused; no new post-clone syscall and no new injection mode (`injection.rs` is byte-identical too) → **`P4_P3_CHILD_CONTRACT_PRESERVED`** |

## 18. `unsafe` delta

**ZERO.** The diff `3d152ad..50a340e` over the crate's Rust sources adds and removes no line
containing `unsafe`; the only two `+` lines matching the token are documentation sentences stating
that no `unsafe` may be added outside the backend boundary.

`src/launch.rs` contains **no** `unsafe`: its four occurrences of the token are all in comments. The
crate root still carries `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`, and the backend's scoped
allowance is unchanged. The two new test helpers that need a disposition or a process group use a
three-line C wrapper and `std::os::unix::process::CommandExt::process_group` respectively, precisely
so that no `unsafe` appears outside the accepted boundary.

**`P4_CORRECTION_UNSAFE_DELTA_ZERO`.**

## 19. P3 regression gates

| Gate | Result |
|---|---|
| `P3R-20` fixture concurrency | **PRESERVED** — `backend/tests.rs` changed by additions only |
| `P3R-21` ordinary group semantics | **PRESERVED** — the product `setpgid` race is untouched; the seam does not change it, it only removes the race from a test by holding the child |
| `S6` positive authority | **PRESERVED** |
| Machine-code closed-world proof | **PRESERVED** — `child.rs`, `syscall.rs`, `injection.rs` byte-identical; both CI steps unchanged |
| Injection confinement | **PRESERVED** — see §12.2; the inventory grew by `src/launch.rs`, every occurrence there is correctly double-gated and additionally `cfg(test)`-gated, and the release-absence proof is unaffected |
| `strace` child window | **PRESERVED** — no new child-window syscall |
| `S5` / `S6` | **PRESERVED** |
| `p3_boundary` | **PRESERVED** — additions only; 17 cases pass locally |
| `p2_boundary` | **PRESERVED** — untouched |

**`P4_P3_REGRESSION_GATES_PRESERVED`.**

## 20. CI reachability

Every new load-bearing Linux integration case executes naturally after publication, on the hosted
runner, with no manual dispatch and no Trial workflow.

| Case | Executed by |
|---|---|
| Continuous output | `Name and run the P4 lifecycle and receipt cases` (`launch::tests::`), and `cargo test -p helm-launch --locked` |
| Non-spin guard | same |
| 8 MiB dual stream | same |
| `POLLIN`+`POLLHUP` ×200 | same |
| `CLD_DUMPED` | same |
| S3 | same |
| Positive group sweep | **new step** `Name and run the P4 coordinated sweep and foreign-reaper cases`, and the existing `cargo test -p helm-launch --locked --features test-fault-injection` |
| Escaped descendant | same |
| Foreign reap | same |
| Drop-guard arm / disarm / reaped cases | `Name and run the P3 backend and traced-window cases` (`backend::tests::`) |
| `p4_boundary`, `p3_boundary`, `p2_boundary` | `Run the unsafe-confinement and boundary suites`, on all three platforms |
| P3 machine-code, injection-confinement, `strace`, S5/S6 | unchanged steps |

The cases compiled only behind `test-fault-injection` have a CI step that **enables** the feature —
two of them, in fact: the pre-existing whole-suite run and the new named step. The new step is
`if: runner.os == 'Linux'`, inside the existing `push`/`pull_request` job; there is no
`workflow_dispatch` requirement.

**`P4_CORRECTED_CI_REACHABILITY_SOUND`.**

## 21. Local validation

Run on Windows 11, `cargo` 1.95.0, at `50a340e`, worktree clean.

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS**, no warning |
| `cargo test --workspace --locked` | **PASS** — 0 failed across every suite |
| `cargo test -p helm-launch --locked` | **PASS** — 42 lib + 20 + 17 `p3_boundary` + 15 `p4_boundary` + 19 + 2 + 23 doctests, 0 failed |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS**, 0 failed |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **OK** — 831 run, 74 skipped, 0 failed |
| `python tools/validate_docs.py` | **PASS** — 141 markdown files, 255 JSON files, 1 617 link targets |
| `git diff --check` | **clean** |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | **PASS**, no warning, after touching the changed sources to force a real re-check |

No WSL Rust toolchain was installed; the cross-target check uses the already-installed
`x86_64-unknown-linux-gnu` target.

The cross-target clippy run is worth naming separately. The Linux-only modules — `launch::tests`,
`backend::tests` and `coordinated` — are `cfg`-excluded off the cohort and therefore do **not**
compile in the Windows-host runs above. The `--target x86_64-unknown-linux-gnu --all-targets
--all-features` check **does** compile them. Every new Linux integration case in this correction
therefore type-checks and lints clean, including the feature-gated coordination seam. What remains
unverified locally is only their **execution**.

**P4 LINUX RUNTIME VALIDATION: `PENDING PUBLICATION CI`.** This is expected and is the separate
hosted gate, not a downgrade of any finding above.

## 22. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `R-P4C-M1` | `MINOR` | evidence | n/a (test) | The **runtime** half of the `F-P4-01` fairness regression depends on the `spinner` child outproducing the parent's read-and-hash loop, so on an arbitrary runner it is probabilistic rather than deterministic. The **structural** half (`p4_boundary::a_ready_stream_is_read_at_most_once_per_observation_turn`) is deterministic and platform-independent, and catches any reintroduction of the unbounded drain | **OPEN, NON-BLOCKING.** The fixture meets the accepted criterion — genuinely unbounded, not a large finite buffer — and the two halves together are probative |
| `R-P4C-M2` | `MINOR` | evidence | n/a (test) | `the_receipt_is_deterministic_bounded_and_recomputable`'s unequal-records branch asserts `group_sweep` is the **only** field that may differ between two launches of the same plan. If some other legitimately nondeterministic fact ever appears, the case fails loudly rather than silently weakening | **OPEN, NON-BLOCKING.** Failing loudly is the correct direction; noted so a future widening is a deliberate decision |
| `R-P4C-B1` | `BACKLOG_NONBLOCKING` | robustness | product | A sustained `EINTR` storm against the launching thread can make the observation loop turn repeatedly without consuming bytes, until a model deadline or the absolute guard fires. No deadline is extended, no false durable fact is produced and the loop still terminates within the accepted bound | **BACKLOG.** Bounded, correct, and a host condition outside the accepted contract |
| — | `GATE_PENDING` | hosted runtime | n/a | No supported Linux execution occurred in this re-review; the new integration cases are structurally reviewed and cross-target type-checked, not run | **P4 LINUX RUNTIME VALIDATION: `PENDING PUBLICATION CI`** |

Carried from the first review, unchanged and not reopened: `P4A-03-EndNotObserved` (`MINOR / OPEN`),
`F-P4-M1` … `F-P4-M8` (`MINOR`) and `F-P4-B1` (`BACKLOG_NONBLOCKING`).

**0 `BLOCKER`. 0 `IMPORTANT`.** No new `IMPORTANT` was introduced by the correction.

## 23. Boundary of this re-review

This re-review is recorded in documentation only. It changed no product code, test, workflow, Cargo
file, ADR contract, experiment or evidence; it did not rebase, amend, squash, push or touch `main`;
it promotes no traceability row to a stronger evidence class and **accepts no P4 gate**. Accepting
P4 remains the owner's decision.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, IMPLEMENTED, INDEPENDENTLY REVIEWED,
CORRECTED AND INDEPENDENTLY RE-REVIEWED — AND IS NOT YET ACCEPTED. P5 IS NOT AUTHORISED. THE
COMPLETE helm-launch 0.1 MODULE IS NOT PRODUCT-ACCEPTED.**

**Next gate: ONE FAST-FORWARD PUBLICATION OF THE COMPLETE REVIEWED P4 CHAIN, THEN THE FIRST NATURAL
HOSTED P4 CI.**
