# HELM-LAUNCH P3 — BOUNDED INDEPENDENT REVIEW OF P3R-21

> # 🔎 BOUNDED INDEPENDENT REVIEW OF `P3R-21` — NOT A FULL P3 UNSAFE REVIEW
>
> **Session provenance.** **THIS REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `a92c8a4d1e6e241464462fa28c2c310e6bc5e66d` (the owner disposition recording the second hosted
> publication failure and accepting `P3R-21`) and
> `e39fca6aa38dd037fae4be23ede1560d24b8ddff` (the `P3R-21` harness correction).
> It authored no part of the P3 implementation chain, no part of the `P3R-20` correction chain and
> no part of either commit under review. It read those commits, re-derived the root cause from the
> pre-correction source at `80ea89b8`, and re-established the corrected behaviour from the product
> source, the injection source, the harness source and the workflow definitions.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **This document does NOT supersede
> [`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md)
> (`4c834415`), [`HELM-LAUNCH-P3-CORRECTION-REREVIEW.md`](HELM-LAUNCH-P3-CORRECTION-REREVIEW.md)
> (`5c577d45`), [`HELM-LAUNCH-P3-P3R15-REREVIEW.md`](HELM-LAUNCH-P3-P3R15-REREVIEW.md)
> (`3a9368f8`) or [`HELM-LAUNCH-P3-P3R20-REVIEW.md`](HELM-LAUNCH-P3-P3R20-REVIEW.md)
> (`80ea89b8`).** Those remain the authoritative P3 independent review record for every area this
> correction does not touch. This review is bounded to `P3R-21`: its root cause, its correction, the
> determinism of the S6 positive-authority assertion, the separation of process-group state from
> parent-side authority, the preservation of `P3R-20`, proof that product backend semantics are
> unchanged, and the preservation of both historical publication results.

> **P1 ACCEPTED. P2 ACCEPTED. P3 PUBLISHED CANDIDATE — SECOND HOSTED VALIDATION FAILED —
> `P3R-21` CORRECTION CANDIDATE. P4 AND P5 NOT AUTHORISED. TRIAL #3 STAYS `MECHANISM_REJECTED`
> AND MUST NOT BE RERUN. TRIAL #4 NOT AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **0** |
| `MINOR` | 2 |
| `GATE_PENDING` | 1 |
| `P3R-21` | **INDEPENDENTLY VERIFIED CORRECTED** |
| Root cause | **FULLY EXPLAINED** — a scheduler-dependent parent-authority outcome was asserted as a guarantee |
| Ordinary launch requires parent authority | **NO** |
| Ordinary executed-image group leadership | **PROVEN BY `pgid_is_self`** |
| `pgid_is_self` promoted into authority | **NO** |
| S6 stall strictly before `execveat` | **PROVEN** |
| S6 parent `setpgid` before any possible exec | **PROVEN** |
| Other reachable parent `setpgid` failure under S6 | **NONE FOUND** |
| S6 `group_authority_established == true` | **DETERMINISTIC** |
| Trace contract | **UNCHANGED** |
| `P3R-20` | **REMAINS FIXED** |
| Product backend | **BYTE-UNCHANGED** |
| Process-group sweep | **ABSENT** |
| Group-authority policy | **UNCHANGED** |
| Historical evidence | **BOTH PUBLICATIONS PRESERVED** |
| `P3R-21` Linux runtime regression | **`GATE_PENDING`** — pending corrected publication CI |

**Classification: `HELM_LAUNCH_P3_P3R21_REVIEW_PASSED_READY_FOR_CORRECTED_PUBLICATION_CI`.**

## 1. Target and starting state

| Item | Value |
|---|---|
| Published P3 head (historical, `P3R-20` corrected) | `80ea89b8eef40dc1de68993eac3e140a4925d9b3` |
| Previous published head (historical, first) | `3a9368f845452110afc859ed899acae9384c7d9b` |
| Owner-disposition commit (`P3R-21` accepted) | `a92c8a4d1e6e241464462fa28c2c310e6bc5e66d` |
| `P3R-21` correction commit | `e39fca6aa38dd037fae4be23ede1560d24b8ddff` |
| Branch | `docs/helm-launch-architecture` |
| Local `HEAD` before this review | `e39fca6aa38dd037fae4be23ede1560d24b8ddff` |
| `origin/docs/helm-launch-architecture` | `80ea89b8eef40dc1de68993eac3e140a4925d9b3` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED** |
| Local relation to published branch | **2 ahead, 0 behind** |
| Worktree at start | **CLEAN** |
| Push | **NO** |
| Historical CI rerun | **NONE** |

`git fetch origin` was read-only. No repair, rebase or synchronisation of history was performed.

## 2. Commit scope

Both commits are within their authorised bounds.

| Commit | Files changed | Verdict |
|---|---|---|
| `a92c8a4d` | `docs/DECISIONS.md` (+168), `docs/PROJECT_STATE.md` (+71) | **IN SCOPE** — documentation only |
| `e39fca6a` | `crates/helm-launch/src/backend/tests.rs` (+51 / −5) | **IN SCOPE** — harness only |

`git diff --name-status 80ea89b e39fca6` returns exactly three paths:
`crates/helm-launch/src/backend/tests.rs`, `docs/DECISIONS.md`, `docs/PROJECT_STATE.md`.

`docs/DECISIONS.md` is a **pure append**: the single hunk is `@@ -1769,3 +1769,171 @@`, so every
earlier byte of the file — including the accepted `ADR-0024` row — is unchanged.

### 2.1 Product byte-identity

Blob identity compared directly between `80ea89b8` and `e39fca6a`:

| Path | Blob | Result |
|---|---|---|
| `crates/helm-launch/src/backend/child.rs` | `d8b75c67` | **IDENTICAL** |
| `crates/helm-launch/src/backend/spawn.rs` | `5575639e` | **IDENTICAL** |
| `crates/helm-launch/src/backend/syscall.rs` | `38d95fb4` | **IDENTICAL** |
| `crates/helm-launch/src/backend/mod.rs` | `12467e87` | **IDENTICAL** |
| `crates/helm-launch/src/backend/injection.rs` | `fa3704d0` | **IDENTICAL** |
| `crates/helm-launch/src/authority.rs` | `3a877d61` | **IDENTICAL** |
| `crates/helm-launch/src/lib.rs` | `c3b3b524` | **IDENTICAL** |
| `Cargo.toml` | `953ca8a3` | **IDENTICAL** |
| `Cargo.lock` | `b2df4153` | **IDENTICAL** |
| `.github/workflows/` (tree) | `3945bec5` | **IDENTICAL** |
| `tools/helm_launch_child_closure.py` | `61133090` | **IDENTICAL** |
| `tools/tests/test_helm_launch_machine_proofs.py` | `3ccf8622` | **IDENTICAL** |
| `docs/adr/ADR-0024-launch-authority.md` | `97a76f9c` | **IDENTICAL** |
| `docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md` | — | **UNCHANGED** (not in diff) |

`git diff --name-only 80ea89b e39fca6 -- crates/` lists **only** `backend/tests.rs`. No product,
manifest, workflow or machine-proof file changed. A full P3 unsafe review is **not** re-triggered.

### 2.2 The harness file outside the two hunks

The correction has exactly two hunks, `@@ -1417,18 +1417,45 @@` and `@@ -1673,6 +1700,25 @@`. The
remainder of the 2 472-line file (now 2 518 lines) was compared region by region against `80ea89b8`
at the correct offsets:

| Region | Old lines | New lines | Result |
|---|---|---|---|
| A | 1 – 1416 | 1 – 1416 | **BYTE-IDENTICAL** |
| B | 1435 – 1672 | 1462 – 1699 | **BYTE-IDENTICAL** |
| C | 1679 – 2472 | 1725 – 2518 | **BYTE-IDENTICAL** |

Nothing in the harness changed except the two intended sites.

## 3. Root cause of `P3R-21`, re-established independently

The author report was not taken on trust. The pre-correction test was read at `80ea89b8` and the
product implementation was read at `HEAD`.

### 3.1 What the pre-correction test asserted

At `80ea89b8`, `backend::tests::the_parent_establishes_group_authority_and_issues_no_group_signal`
ran an ordinary, uncoordinated `launch_minimal` and then asserted, **unconditionally**, at line
`1426`:

```rust
assert!(
    launch.group_authority_established,
    "the parent's own setpgid(child, child) did not succeed, so no later slice could sweep"
);
```

The panic location recorded in the owner disposition —
`crates/helm-launch/src/backend/tests.rs:1426:5` — matches that `assert!` exactly, as does the panic
message. The historical failure record is corroborated by the pre-correction source, not merely by
the report.

### 3.2 What the product actually guarantees

[`spawn.rs:409-418`](../../crates/helm-launch/src/backend/spawn.rs) — the parent branch, immediately
after `clone3`:

```rust
// 3. `setpgid(child, child)` is the first system call after `clone3`.
//    Only its success establishes group-sweep authority; an error
//    establishes nothing, is not retried and is not interpreted further.
//    The two conversions before it are pure arithmetic, not system calls.
let child_pid = i32::try_from(cloned).ok().and_then(Pid::from_raw);
let group_authority_established = match child_pid {
    Some(pid) => setpgid(Some(pid), Some(pid)).is_ok(),
    None => false,
};
```

Independently verified against the source, not the comment:

* the boolean is written **once**, from `setpgid(...).is_ok()` on the **parent** side;
* any error yields `false`; there is **no retry**, **no** `errno` interpretation, and **no**
  promotion of any child-side result;
* the two statements between `clone3` and `setpgid` are `i32::try_from` and `Pid::from_raw`, both
  pure arithmetic, so `setpgid` really is the parent's first post-`clone3` system call;
* `SpawnedChild.group_authority_established` is carried into `MinimalLaunch` unmodified
  (`mod.rs:395-440`), and nothing in P3 reads it.

Separately, [`child.rs:217-226`](../../crates/helm-launch/src/backend/child.rs) — stage 5 of the
closed child sequence, before `execveat`:

```rust
unsafe { issue(plan, stage::SETPGID, syscall::NR_SETPGID, 0, 0, 0, 0, 0, 0) };
```

### 3.3 The permitted scheduling outcome

Both calls target the same end state, so after either one the child leads a process group whose
identifier is its own pid. The two are unordered with respect to each other: `clone3` returns in the
parent and starts the child, and Linux does not order the parent's next instruction against the
child's.

The kernel's `setpgid` refuses with `EACCES` when the named child has already executed — the
`PF_FORKNOEXEC` condition, cleared by `execve`/`execveat`. Therefore this interleaving is fully
legal:

1. `clone3` returns in both processes;
2. the child runs stages 1 – 8, including its own `setpgid(0, 0)`;
3. the child reaches `execveat` and succeeds;
4. only now does the parent issue `setpgid(child, child)`, and the kernel answers `EACCES`;
5. `group_authority_established` is `false`;
6. the executed image nevertheless leads its own process group, and `pgid_is_self` is `true`.

Every P3 invariant holds in that trace. The pre-correction test failed it anyway.

**This model is confirmed by the historical evidence and is not merely asserted.** On the identical
head `80ea89b8` and the identical `ubuntu-24.04` runner image, the same test binary and the same
test **passed** in workspace run `35461333920` and **failed** in helm-launch run `35461333887`. A
defect deterministic in the product could not differ between two runs of the same code on the same
image; a scheduling race can, and did.

**Verdict: the `80ea89b8` helm-launch failure is FULLY EXPLAINED by the test asserting a
scheduler-dependent parent-authority outcome as though it were a guarantee. No product defect is
implicated. No alternative defect was found.**

## 4. The corrected ordinary-launch test

`backend::tests::the_executed_image_leads_its_group_and_p3_issues_no_group_signal`
([`tests.rs:1443-1460`](../../crates/helm-launch/src/backend/tests.rs)):

```rust
let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"group"])).expect("launch");
assert!(!launch.sigkill_sent, "a normal run needs no signal at all");
let completed = complete(launch);
let report = parse_report(&completed.stdout);
assert!(report.pgid_is_self, "the executed image does not lead its own process group");
assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
```

| Property | Result |
|---|---|
| Requires `group_authority_established == true` | **NO** |
| Requires `group_authority_established == false` | **NO** |
| Mentions `group_authority_established` outside a doc comment | **NO** — the identifier survives only at `tests.rs:1424`, inside prose |
| Asserts `report.pgid_is_self` | **YES** |
| Infers parent authority from `pgid_is_self` | **NO** |
| Asserts normal completion | **YES** — `ChildEnd::Exited { code: 0 }` |
| Asserts no signal was issued | **YES** — `!launch.sigkill_sent` |

Every retained assertion is a deterministic P3 fact:

* `!launch.sigkill_sent` — `sigkill_sent` is set only when the fixed pre-exec bound expires and
  `child.send_sigkill()` succeeds through the **pidfd** (`mod.rs:418-427`). An ordinary launch
  reaches exec, the `CLOEXEC` status pipe closes, the parent observes end-of-file rather than a
  timeout, and no signal is ever issued. Not scheduler-dependent.
* `report.pgid_is_self` — true after **either** `setpgid`, and both are unconditional. Not
  scheduler-dependent.
* `ChildEnd::Exited { code: 0 }` — the fixture's normal end. Not scheduler-dependent.

`pgid_is_self` is **not a constant**: the producer self-test at `tests.rs:1205-1212` runs the same
fixture outside the backend and requires the field to be `false`, because the directly-run fixture
inherits the harness's group. The launched `true` is therefore a result. The same field was already
asserted `true` by the accepted direct-child test at `tests.rs:1282-1289` before this correction, so
the corrected test introduces no new claim about the product — only a differently scoped one.

**`P3R21_ORDINARY_GROUP_TEST_SOUND`.**

## 5. Process-group state and parent-side authority stay distinct

The two facts are kept separate at every layer, and the correction strengthens rather than blurs the
distinction.

| Fact | Meaning | Where established |
|---|---|---|
| `report.pgid_is_self` | The **executed image** observes itself as leader of its own process group | Written by the fixture from its own process-group identity; true after either `setpgid` |
| `launch.group_authority_established` | The **parent's own** `setpgid(child, child)` returned success | `spawn.rs:414-418`, parent side only |

* The corrected test's doc comment states the separation explicitly and in the load-bearing
  direction: `pgid_is_self` "is evidence that the image leads its group, and it is **not** evidence
  that the parent holds sweep authority. Nothing in this test promotes one into the other."
* The corrected test contains **no** assertion on `group_authority_established` at all, so no
  inference from `pgid_is_self` to authority is structurally possible.
* The S6 comment repeats the rule from the other side: "Only that call's success sets the fact.
  Nothing is inferred from the child's own `setpgid`."
* The product comments that carry the rule — `spawn.rs:317-320`, `spawn.rs:409-412`,
  `child.rs:219-222`, `mod.rs:356-358` — are **byte-unchanged**.

No `pgid_is_self` ⇒ `group_authority_established` reasoning appears anywhere in the corrected
harness, the product, or the disposition.

**`P3R21_GROUP_FACT_SEPARATION_SOUND`.**

## 6. The S6 positive-authority assertion

`injected::a_child_that_stalls_before_exec_is_bounded_killed_and_reaped`
([`tests.rs:1680-1742`](../../crates/helm-launch/src/backend/tests.rs)) now asserts, at
`tests.rs:1717-1721`:

```rust
assert!(
    result.group_authority_established,
    "the child was held before exec, so the parent's setpgid(child, child) could not \
     have raced execveat; without its success no later slice could sweep"
);
```

The author's reasoning was not accepted as given. The ordering was re-derived from source.

### 6.1 Source ordering

**Parent** (`spawn.rs`, then `mod.rs:395-440`), in order:

1. `prepare` — every fallible operation, **before** any child exists;
2. block every blockable signal on this thread;
3. `clone3(CLONE_PIDFD)`;
4. error check (pure), `i32::try_from` (pure), `Pid::from_raw` (pure);
5. **`setpgid(child, child)`** — the parent's first system call after `clone3`;
6. pidfd ownership (bookkeeping), mask restore;
7. `release_child_side(retain_stdin_writer)`;
8. `observe_exec_status` — the fixed pre-exec bound, `SPAWN_CONFIRM_TIMEOUT_MS`;
9. on timeout only: one pidfd `SIGKILL`, then the bounded reap.

**Child** (`child.rs`), in order: `DUP2` → `CLEAR_CLOEXEC` → `CHDIR` → `CLOSE_RANGE` → `SETPGID`
→ `SIGACTION` → `SIGMASK` → `NO_NEW_PRIVS` → **S6 injection** → `EXECVEAT`.

The injection call site is at `child.rs:318-325`, textually after the `NO_NEW_PRIVS` `issue` at
`child.rs:293-310` and before the `execveat` `issue` at `child.rs:333-345`. There is no branch, loop
or early return between them. **S6 injection occurs strictly after `NO_NEW_PRIVS` and strictly
before `EXECVEAT`.** `PROVEN`.

### 6.2 The stall cannot return to exec while the parent-side bound is active

[`injection.rs:86-108`](../../crates/helm-launch/src/backend/injection.rs), `MODE_STALL_BEFORE_EXEC`:

```rust
let read = unsafe { syscall::syscall6(syscall::NR_READ, 0, byte.as_mut_ptr()..., 1, 0, 0, 0) };
if read == 0 {
    return;          // end-of-file only
}
// every other outcome loops
```

The **only** exit that reaches `execveat` is a literal zero return — end-of-file. A short read, a
one-byte read, or any error-encoded return re-enters the loop. This matters: the child's stage 7
`SIGMASK` installs an empty mask and stage 6 `SIGACTION` restores default dispositions, so signals
are deliverable during the stall; even so, an `EINTR`-shaped return cannot fall through, because it
is not zero.

End-of-file requires **every** write end of the stdin pipe to be closed. It cannot happen here:

* `mod.rs:404` — `retain_stdin_writer` is `true` exactly when
  `fault.mode == injection::MODE_STALL_BEFORE_EXEC`;
* the parent's `stdin_write` is owned by `PreparedLaunch` across `spawn` — so it is open throughout
  `clone3` and `setpgid` — and `release_child_side` then **retains** it rather than dropping it
  (`spawn.rs:131-136`);
* it is carried into `MinimalLaunch.stdin_write` and is still open while `observe_exec_status` runs
  the whole bound.

The child therefore blocks in `read` until the parent's `SIGKILL`, and never executes.
**`MODE_STALL_BEFORE_EXEC` cannot return to exec while the parent-side pre-exec status wait is
active. `PROVEN`.**

### 6.3 An independent, assertion-level proof of the same ordering

The S6 test does not rest on the injection argument alone. Two assertions **precede** the new one:

```rust
assert_eq!(result.exec_status, ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout));
assert!(result.sigkill_sent, "the pre-exec bound must send one SIGKILL");
```

Every pipe in this backend is created `pipe_with(PipeFlags::CLOEXEC)` (`spawn.rs:249`). A successful
`execveat` therefore closes the child's status write end, and `observe_exec_status` returns on
`Ok(0)` with `timed_out: false` (`mod.rs:470-477`). `PreExecStatusTimeout` is produced **only** when
the full bound elapsed with neither a record nor end-of-file — which is possible only if the child
did **not** exec. So by the time the authority assertion is evaluated, the test has already proven
that no exec occurred within the bound, and the parent's `setpgid` ran before the bound began.

The same gating covers the degenerate case of a real stage failure in child stages 1 – 8: such a
child writes a status record and exits, the parent observes a record rather than a timeout, and the
test fails at the **first** assertion — never reaching, and never falsely satisfying, the authority
assertion.

**Parent `setpgid` strictly precedes any possible exec in S6. `PROVEN`.**

## 7. Other reachable parent `setpgid` failure modes under S6

Preventing exec does not by itself make `setpgid` infallible, so every other refusal path was
enumerated against the actual P3 clone contract.

**Contract, verified in source.** `CloneArgs::for_direct_child` (`syscall.rs:313-322`) sets
`flags: CLONE_PIDFD` and `exit_signal: SIGCHLD`, everything else zero; `const` assertions at
`syscall.rs:334-338` prove the flag word is a single bit and excludes `CLONE_VM`, `CLONE_FILES`,
`CLONE_VFORK` and `CLONE_THREAD`. A repository-wide search finds **no** `setsid`, `unshare`,
`CLONE_NEWPID` or any other namespace mechanism in `crates/helm-launch/src`. The child is a direct
child, in the parent's session, single-threaded, its own thread-group leader, never a session
leader, and not reaped before the call.

| Refusal | Condition | Reachable under S6? |
|---|---|---|
| `EINVAL` | `pgid < 0` | **NO** — `pgid == pid`, the `clone3` return, and `Pid::from_raw` already rejects non-positive values (yielding `false` without a syscall) |
| `EINVAL` | target is not a thread-group leader | **NO** — a fresh single-threaded child is its own leader |
| `ESRCH` | target is neither the caller nor a child of the caller | **NO** — direct child; the parent issues no `waitid` before `setpgid`, so even a hypothetical early exit leaves an unreaped zombie whose pid is still resolvable |
| `EPERM` | target is in a different session | **NO** — no `setsid`, no namespace flag, anywhere in the crate |
| `EPERM` | target is a session leader | **NO** — the child never calls `setsid`; the `PERMITTED` traced set (`tests.rs:2110-2121`) contains no `setsid` |
| `EPERM` | move into a group in another session | **NO** — the `pgid != pid` branch is not taken |
| `EACCES` | target has already executed | **NO** — removed by the S6 stall (§6.2), and independently excluded by the preceding `PreExecStatusTimeout` assertion (§6.3) |

The child's own `setpgid(0, 0)` cannot create a new refusal for the parent either: the parent's call
then names a group the child already leads, which is a permitted no-op, and process-group leadership
is not the session-leader flag that `EPERM` tests.

**No reachable parent `setpgid` failure remains under S6.
`P3R21_S6_AUTHORITY_ASSERTION_DETERMINISTIC`.**

## 8. Child `setpgid` does not establish authority

| Check | Result |
|---|---|
| The product boolean is written only from the parent call | **YES** — the single write site is `spawn.rs:414-418` |
| Any child-side result is read, promoted or inferred from | **NO** — `spawn.rs` never reads a child-side status for this fact |
| The S6 assertion is evidence of the **parent** syscall's success | **YES** — it reads `result.group_authority_established`, whose only source is that call |
| Product code changed by the correction | **NO** — `spawn.rs`, `child.rs`, `mod.rs`, `injection.rs` all byte-identical (§2.1) |

The S6 comment states the rule in the correct direction and does not weaken it. No reasoning of the
form "the child's `setpgid(0, 0)` succeeded, therefore parent authority exists" appears anywhere.

## 9. Trace contract

The traced-window assertions are in region **A** and region **C** of the harness (§2.2) and are
therefore **byte-unchanged** by the correction.

| Trace property | Assertion | Changed? |
|---|---|---|
| `clone3` immediately preceded by a full-set `SIG_SETMASK` | `tests.rs:2040-2051` | **NO** |
| **`setpgid` is the first parent syscall after `clone3`** | `tests.rs:2052-2055` | **NO** |
| `setpgid` names the child twice | `tests.rs:2056-2060` | **NO** |
| Mask restored immediately after `setpgid` | `tests.rs:2061-2069` | **NO** |
| Child window permitted syscall set | `tests.rs:2110-2128` | **NO** |
| Child stage order, exactly one `setpgid` | `tests.rs:2148` | **NO** |
| No `kill` / `tgkill` / `tkill` anywhere | `tests.rs:2072-2078` | **NO** |

Critically, the trace asserts the parent `setpgid`'s **name, position and arguments** and **never
its return value**. The only traced result asserted to be `0` is the child's `execveat`
(`tests.rs:2183-2186`). The accepted property remains **attempt and ordering, not unconditional
success**, and the correction did not make the trace require `setpgid == 0` on an uncoordinated run.

**`P3R21_TRACE_CONTRACT_UNCHANGED`.**

## 10. No group sweep

| Check | Result |
|---|---|
| Negative-pid signal anywhere in `crates/helm-launch/src` | **NONE** |
| `killpg`, group signalling | **NONE** |
| Signals actually issued by the product | Exactly one: `pidfd_send_signal(self.pidfd, Signal::KILL)` at `spawn.rs:547` — the **direct child**, through its pidfd |
| `lifecycle.rs` `GroupSweep` / `Action::SweepGroup` | A **pure model**: its header states it "reads no clock, polls nothing, signals nothing, waits for nothing, reaps nothing and touches no descriptor"; it imports only `crate::model` and issues no system call |
| `sigkill_sent` semantics | `mod.rs:359-360`: "Whether P3 sent the **direct child** one `SIGKILL` through its pidfd" |
| Correction diff introduces any sweep | **NO** — the diff adds only assertions and comments |

The ordinary test's `!launch.sigkill_sent` is therefore a direct-child cleanup fact and implies
nothing about a process-group sweep, which does not exist.

**PROCESS-GROUP SWEEP: ABSENT. GROUP AUTHORITY POLICY: UNCHANGED. PUBLIC `launch()`: ABSENT**
(`launch_minimal` and `launch_minimal_with_fault` are `pub(crate)`; `mod.rs:377-381` states the
non-public boundary, and it is byte-unchanged). **P4 / P5: NOT AUTHORISED.**

## 11. `P3R-20` regression check

`P3R-20`'s correction lives entirely in harness regions **A** (the `fixture_binary` build protocol,
`tests.rs:355-455`) and **C** (the concurrency regression test, `tests.rs:2291-2518`), both proven
byte-identical to `80ea89b8` in §2.2.

| `P3R-20` mechanism | Location | Result |
|---|---|---|
| Fixture-build nonce protocol (`FIXTURE_BUILD_NONCE`, `fetch_add`) | `tests.rs:396` | **UNCHANGED** |
| Unique source pathname per build | `tests.rs:397-398` | **UNCHANGED** |
| Unique staging output pathname | `tests.rs:399` | **UNCHANGED** |
| `hard_link` no-replace publication | `tests.rs:440-441` | **UNCHANGED** |
| Concurrency regression test | `tests.rs:2385-2518` | **UNCHANGED** |

**`P3R-20`: REMAINS FIXED.** Its hosted-Linux verification on `80ea89b8` — run `35461333920`, and
`concurrent_builders_of_one_fixture_publish_exactly_one_object` executing and passing inside
`35461333887` before that run's later failure — is not disturbed by this correction.

## 12. Historical evidence

`a92c8a4d` was checked line by line against the required record.

| Required record | Present and accurate? |
|---|---|
| `80ea89b8` is the corrected publication head | **YES** |
| helm-launch run `35461333887`, attempt 1 — **FAILURE** | **YES** |
| Workspace run `35461333920`, attempt 1 — **SUCCESS** | **YES** |
| First publication `3a9368f8`: `35442641728` attempt 1 FAILURE, `35442641743` attempt 1 FAILURE | **YES** — preserved in both the new section and the retained earlier section |
| `P3R-20` hosted-Linux verified corrected | **YES** |
| `P3R-21` accepted as **IMPORTANT** | **YES** |
| Classified **TEST / EVIDENCE CONTRACT** defect | **YES** |
| Product mechanism **not implicated** | **YES** |
| Later load-bearing P3 gates **incomplete** | **YES** — `P3R-G1` / `P3R-G2` `GATE_PENDING`; the skipped steps are enumerated |
| No retry / rerun / replacement | **YES** — stated in both documents |
| P4 / P5 not authorised | **YES** |
| Trial #3 frozen `MECHANISM_REJECTED`, Trial #4 not authorised | **YES** |
| Any text implying `80ea89b8` validation **passed** | **NONE FOUND** |

The quoted panic — `crates/helm-launch/src/backend/tests.rs:1426:5` with the message "the parent's
own setpgid(child, child) did not succeed, so no later slice could sweep" — matches the
pre-correction source at `80ea89b8` exactly (§3.1). The narrower sub-test claim that this same test
passed in `35442641728` is consistent with that run being recorded as an overall **FAILURE** on
`P3R-20` fixture-build defects; both documents continue to record `35442641728` as attempt 1
FAILURE. **No historical result is reinterpreted, retried, rerun or replaced.**

## 13. Local validation

Run at `HEAD = e39fca6a` on Windows 11 x86_64, the reviewer's host.

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo test --workspace --locked` | **PASS** |
| `cargo test -p helm-launch --locked` | **PASS** — 42 + 19 + 17 + 19 + 2 |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** — same counts |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — `Ran 831 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | **PASS** — 139 markdown, 255 JSON, 1538 link targets |
| `git diff --check` | **PASS** — no whitespace defect |
| **`cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings`** | **PASS** |

The cross-target clippy run is the most informative of these. It compiles the
`cfg(all(target_os = "linux", target_arch = "x86_64"))` backend **and** the
`cfg(all(feature = "test-fault-injection", debug_assertions))` `injected` module for Linux, so both
corrected sites — the renamed ordinary test and the new S6 assertion — are proven to **type-check
and lint clean for the target platform**. It does not execute them.

**The backend test cases did not run on this host** and cannot: `lib.rs:228-243` gates the backend on
Linux x86_64, and the off-cohort CI step at `helm-launch.yml:216-226` requires that no `backend::`
test even exists elsewhere. The 42 lib tests that ran here are the platform-independent model.

**`P3R-21 LINUX RUNTIME REGRESSION: PENDING CORRECTED PUBLICATION CI`.** This is a gate, not a
review failure.

No attempt was made to manufacture scheduler timing to reproduce the ordinary race. The correction
removes dependence on that race; measuring its frequency would not be evidence either way.

## 14. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P3R21-M1` | `MINOR` | Test naming / assertion hygiene | n/a | The ordinary test's name claims "p3 issues no group signal", but its only signal-related assertion is `!launch.sigkill_sent`, a **direct-child pidfd** fact. The group-signal-absence claim is actually carried by the traced-window `kill`/`tgkill`/`tkill` scan (`tests.rs:2072-2078`) and by the structural absence of any negative-pid call in the crate. | **NO ACTION.** The same clause and the same assertion existed in the pre-correction name and survived the full P3 independent unsafe review. The correction neither introduces nor worsens it, and the claim is true of P3 as a whole. |
| `P3R21-M2` | `MINOR` | Evidence placement | n/a | Positive parent-authority evidence now exists **only** inside the `injected` module, so it runs only under `--features test-fault-injection` (`helm-launch.yml:120` and `:205`). Both of those steps were **skipped** in `35461333887` and have never executed on hosted Linux. | **NO ACTION.** This placement is exactly what the owner disposition authorised, and it is the only way to make the fact deterministic. Recorded so the gate below is read correctly. |
| `P3R21-G1` | `GATE_PENDING` | Linux runtime | — | Neither corrected case has executed on Linux. The reviewer's host cannot run the backend suite. | **PENDING CORRECTED PUBLICATION CI.** |

`BLOCKER`: **0**. `IMPORTANT`: **0**.

No speculative finding is raised. In particular: the ordinary test no longer depends on scheduler
timing; `pgid_is_self` is nowhere treated as parent authority; S6 does prevent exec before the
parent's `setpgid`; and no other reachable `setpgid` failure makes the S6 assertion
non-deterministic.

## 15. Recommendation

**`P3R-21 CORRECTION MAY BE PUBLISHED FOR NEW HOSTED VALIDATION ON A NEW SHA.`**

**Next gate: ONE FAST-FORWARD PUBLICATION OF THE REVIEWED `P3R-21` CORRECTION CHAIN, FOLLOWED BY NEW
NATURALLY TRIGGERED HOSTED P3 CI.** Nothing is rerun: `35442641728`, `35442641743` and
`35461333887` stay exactly as they are, and a corrected result must come from a **new** SHA.

**Classification: `HELM_LAUNCH_P3_P3R21_REVIEW_PASSED_READY_FOR_CORRECTED_PUBLICATION_CI`.**

**P1 AND P2 REMAIN ACCEPTED. P3 REMAINS A PUBLISHED CANDIDATE WHOSE HOSTED VALIDATION IS INCOMPLETE.
P4 AND P5 ARE NOT AUTHORISED. TRIAL #3 STAYS `MECHANISM_REJECTED` AND MUST NOT BE RERUN. TRIAL #4 IS
NOT AUTHORISED.**
