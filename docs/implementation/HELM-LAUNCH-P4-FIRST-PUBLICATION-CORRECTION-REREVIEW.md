# BOUNDED INDEPENDENT RE-REVIEW OF FIRST P4 PUBLICATION TEST CORRECTIONS

> # 🔎 BOUNDED INDEPENDENT RE-REVIEW OF FIRST P4 PUBLICATION TEST CORRECTIONS
>
> **Session provenance. THIS SESSION AUTHORED OR MODIFIED NONE OF:**
> `cfe2537ba638c536d2edd2a9660998713a443b89` (the first-publication failure disposition) and
> `aed3fd2564f47d2b0a999b553133d057b68783e1` (the test / evidence correction). It also authored no
> part of the published P4 head `94ee48da0cc412f0d8043e34b914c198615b923e` or of the P1–P4
> implementation chain. It read the disposition and the correction under review and re-established
> every claim below from the preserved source, the manifests and the workflow definitions.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata. See `R-P4PUB-I1`, which records
> that the two commits under review depart from that policy.

> **P1 ACCEPTED. P2 ACCEPTED. P3 ACCEPTED. P4 AUTHORISED — FIRST PUBLICATION FAILED, TEST / EVIDENCE
> CORRECTION CANDIDATE, NOT ACCEPTED. P5 NOT AUTHORISED. THE COMPLETE helm-launch 0.1 MODULE IS NOT
> PRODUCT-ACCEPTED. TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED` AND MUST NOT BE RERUN. TRIAL #4 IS
> NOT AUTHORISED.**

This is a **bounded** re-review of `P4PUB-01`, `P4PUB-02` and `P4PUB-03`, plus an independent proof
that P4 product semantics are unchanged by the correction. It does not repeat the P4 lifecycle
review and reopens no already-reviewed P4 behaviour.

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **1** (`R-P4PUB-I1`, commit-attribution policy, outside this session's remit to repair) |
| `MINOR` | 0 new |
| `BACKLOG_NONBLOCKING` | 0 new |
| `GATE_PENDING` | 1 |
| `P4PUB-01` core-flag oracle | **FIXED** |
| `P4PUB-02` harness cleanup / reap | **FIXED** |
| `P4PUB-03` descriptor-ownership isolation | **FIXED** |
| `RLIMIT_CORE` fixed-`false` assumption | **REMOVED** |
| Non-core signal positive control | **PROBATIVE** |
| Positive `CLD_DUMPED` producer gate | **PROBATIVE** |
| Disarmed `Drop` load-bearing assertions | **PRESERVED AND SHARPENED** |
| Harness terminates **and reaps** | **YES** |
| `/proc` zombie oracle | **NOT SOLE PROOF** — secondary, after a proven reap |
| Exact descriptor equality | **PRESERVED** |
| Product semantics | **UNCHANGED** |
| `unsafe` delta | **ZERO** |
| Historical runs | **PRESERVED** — NO RETRY, NO RERUN, NO REPLACEMENT |
| P4 Linux runtime | **`GATE_PENDING`** |

The three publication failures are fixed, no oracle was weakened, and the product is
byte-identical. The single `IMPORTANT` is not a product, test or evidence defect: it is a
repository commit-attribution policy violation on the two commits under review, recorded because
this session is forbidden to amend and publication would make it permanent.

## 1. Starting state, independently verified

| Requirement | Observed | Result |
|---|---|---|
| Branch | `docs/helm-launch-architecture` | **OK** |
| Worktree | clean (`git status --porcelain` empty) | **OK** |
| Local `HEAD` | `aed3fd2564f47d2b0a999b553133d057b68783e1` | **OK** |
| `origin/docs/helm-launch-architecture` | `94ee48da0cc412f0d8043e34b914c198615b923e` | **OK** |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` | **OK** |
| Relation to published milestone | **2 ahead, 0 behind** | **OK** |
| Ancestry | `94ee48d` → `cfe2537` → `aed3fd2`, each a single parent | **LINEAR** |

`origin` was fetched read-only. No history was repaired, amended, rebased, squashed or pushed.

## 2. Preserved first publication

Both natural runs remain exactly as published, in `docs/DECISIONS.md` §"Owner disposition
2026-09-20 — HELM-LAUNCH P4 FIRST PUBLICATION" and `docs/PROJECT_STATE.md`:

| Run | Recorded |
|---|---|
| `helm-launch` `35499943908` | attempt 1, event `push` — **FAILURE**; Linux job `106049803287` failed; Windows and macOS **SUCCESS** |
| `HELM Rust workspace Linux` `35499943903` | attempt 1, event `push` — **FAILURE**; Linux job `106049803383` failed |
| Head under test | `94ee48da0cc412f0d8043e34b914c198615b923e` |

Every mention of either run identifier in `docs/` was read. All five carry **attempt 1 / FAILURE**.
**No text implies either run passed, was fixed, rerun, retried, cancelled, restarted or replaced.**
The disposition states **NO RETRY. NO RERUN. NO REPLACEMENT.** and that a corrected head receives a
new SHA and new natural run identifiers.

## 3. Correction scope

`git diff 94ee48d..aed3fd2` touches **four files and no others**:

| File | Commit | Kind |
|---|---|---|
| `docs/DECISIONS.md` | `cfe2537` | disposition |
| `docs/PROJECT_STATE.md` | `cfe2537` | disposition |
| `crates/helm-launch/src/launch.rs` | `aed3fd2` | test-only region (proved in §8) |
| `crates/helm-launch/src/backend/tests.rs` | `aed3fd2` | test module, wholly `#[cfg(test)]` |

The disposition commit touches no code; the correction commit touches no documentation.

## 4. `P4PUB-01` — root cause, re-established independently

**`P4PUB01_ROOT_CAUSE_CONFIRMED`.**

The first publication asserted `core_dumped: false` for `segv-nocore` and `abort-nocore` on the
sole ground that the fixture had set `RLIMIT_CORE.rlim_cur = 0`. That is **not** a portable Linux
invariant across every `core_pattern` configuration.

A soft `RLIMIT_CORE` of zero suppresses a core only on the path where the kernel itself writes the
dump and compares the limit against the dump size. When `/proc/sys/kernel/core_pattern` begins with
`|`, the kernel hands the dump to a userspace handler instead of writing a file, and the soft limit
does not govern that path — the dump proceeds and the wait status is reported with the core flag
set. The runner observed exactly that: `Signaled { signal: 11, core_dumped: true }`.

The product **preserved the kernel-reported classification**, which is its obligation under the P4
contract: `CLD_DUMPED` must reach the receipt as `core_dumped: true` rather than being flattened
into `CLD_KILLED`. The defect was in the test's oracle, not in the launcher. **This is not treated
as a product defect, and no preserved evidence implicates the P4 product mechanism.**

## 5. `P4PUB-01` — the non-core positive control

**`P4PUB01_NONCORE_CONTROL_PROBATIVE`.**

The crasher fixture gains a `term` shape ([`launch.rs:1143`](../../crates/helm-launch/src/launch.rs#L1143)).
It is the correct shape for a fixed branch:

* it sets `sets_limit = 0` and therefore **touches `RLIMIT_CORE` not at all**;
* it prints its identity, mode and `core_allowed=-1`, then calls `raise(SIGTERM)` **on itself,
  after executing**. The signal is the fixture's own, not the launcher's;
* `SIGTERM`'s default disposition terminates **without** a core on any host and under any
  `core_pattern`, so `core_dumped: false` here is a real invariant rather than a host fact.

The producer self-test runs that same image directly first and establishes both facts before any
launcher claim ([`launch.rs:2133`](../../crates/helm-launch/src/launch.rs#L2133)): `signal ==
Some(SIGTERM)` and `!core_dumped`, each with a named `TEST ENVIRONMENT PRECONDITION NOT MET`
message. Only then is the launcher required to produce
`ChildEnd::Signaled { signal: SIGTERM, core_dumped: false }`.

**The timeout logic cannot explain this `SIGTERM`.** The case asserts all three launcher facts:

| Fact | Assertion |
|---|---|
| `sigterm_sent` | `!record.termination().sigterm_sent()` |
| `sigkill_sent` | `!record.termination().sigkill_sent()` |
| `run_deadline_expired` | `!record.run_deadline_expired()` |

A launcher-generated `SIGTERM` would set `sigterm_sent` and, on the deadline path,
`run_deadline_expired`. Both are required false, so the classification is the fixture's own. The
control is **probative**.

## 6. `P4PUB-01` — core-generating shapes

**`P4PUB01_HOST_OBSERVED_CORE_CLASSIFICATION_SOUND`.**

`segv-nocore`, `abort-nocore` and `segv-core` are all **retained**; none was deleted to obtain
green. For every one of them the **same fixture mode is executed directly first**, through
`producer_end` ([`launch.rs:2086`](../../crates/helm-launch/src/launch.rs#L2086)), whose oracle is
the standard library's own reading of the wait status:

* `ExitStatusExt::signal()`
* `ExitStatusExt::core_dumped()`

`producer_end` also asserts the X2c marker — `fixture=p4-crasher` and `mode=<mode>` from the
fixture's own stdout — so the case cannot silently measure a different shape than it names.

The launcher is then required to report **exactly what the producer reported**:

```rust
ChildEnd::Signaled { signal, core_dumped: producer.core_dumped }
```

`core_dumped` is carried from the host observation, never predetermined from `RLIMIT_CORE`. The
`RLIMIT_CORE` manipulation is retained as *shape* coverage rather than as an oracle: each case
additionally requires `core_allowed == 1` in the fixture's stdout **under helm-launch**, proving
the shape configured its own limit as the case names it. **The fixed-`false` assumption is
`REMOVED`.**

## 7. `P4PUB-01` — positive `CLD_DUMPED` gate, diagnostics and coverage

**`P4PUB01_CLD_DUMPED_PRODUCER_GATE_SOUND`. `P4PUB01_HOST_DIAGNOSTICS_SOUND`. `P4PUB-01: FIXED`.**

The positive case is load-bearing and correctly ordered
([`launch.rs:2222`](../../crates/helm-launch/src/launch.rs#L2222)). The producer must establish
`core_dumped == true` **before** any launcher result is trusted. Without it, a launcher that
flattened every `CLD_DUMPED` into `CLD_KILLED` would pass everything above.

If the host cannot establish that condition the case **fails** with a named
`TEST ENVIRONMENT PRECONDITION NOT MET`, carrying the fixture's own report and the actual
`/proc/sys/kernel/core_pattern` value. It is an `assert!`, not a skip: **no silent skip, and no
launcher verdict before producer proof.**

Claim discipline holds. Diagnostics report the **actual** `/proc/sys/kernel/core_pattern`, read at
failure time by `core_pattern()` ([`launch.rs:1297`](../../crates/helm-launch/src/launch.rs#L1297)),
with an explicit `<unreadable: …>` fallback. A repository-wide search finds **no** hard-coded claim
that any host uses Apport, `systemd-coredump` or any other named handler; the only matches are
unrelated pre-existing package manifests under `docs/experiments/evidence/`. The comments describe
the piped-handler mechanism generically and assert nothing about which handler a runner has.

The correction is valid within its declared preconditions for all three conditions:

| Host condition | Behaviour |
|---|---|
| File `core_pattern`, soft limit 0 | producer reports `false`; receipt must equal `false` |
| Piped `core_pattern` | producer reports `true`; receipt must equal `true` |
| Cores disabled / unavailable | `segv-core` producer proof fails with the named precondition, by design |

Coverage is preserved or improved, not weakened:

| Old coverage | State |
|---|---|
| Signal number retained | **YES** — `SIGSEGV`, `SIGABRT`, `SIGTERM` all asserted exactly |
| False-core branch proven | **YES** — by `SIGTERM`, a real invariant, plus a producer gate |
| True-core branch proven where host supports it | **YES** — `segv-core`, gated on producer proof |
| Host-dependent SEGV/ABRT mirrored faithfully | **YES** — receipt must equal producer |
| Anything asserted trivially, `core_dumped` ignored, or conditionally skipped | **NO** |

Core files the crash shapes leave in their own scratch directory are removed by
`remove_core_files` after the producer run and after each launcher run.

## 8. `P4PUB-02` — root cause, re-established independently

**`P4PUB02_ROOT_CAUSE_CONFIRMED`.**

The original sequence disarmed the `Drop` guard, dropped the launch, proved the child survived,
proved `Drop` returned quickly, then killed the child from the harness and polled `/proc/<pid>` —
**without ever reaping it**.

A killed **direct child** is not removed from the process table when it dies. It becomes a zombie
and remains visible at `/proc/<pid>` until its parent collects it with a `wait`-family call. The
harness was that parent and never waited, so the poll could never observe absence and the cleanup
could not finish. **`/proc` presence was not a valid reap oracle.** Both load-bearing assertions had
already passed before that point; the defect was in harness disposal only.

## 9. `P4PUB-02` — load-bearing assertions preserved and sharpened

**`P4PUB02_DISARM_ASSERTIONS_PRESERVED`.**

Both assertions survive, and the survival oracle is now **stronger** than the one that failed
([`backend/tests.rs:2659`](../../crates/helm-launch/src/backend/tests.rs#L2659)):

**A — the disarmed `Drop` did not signal the child.** The strongest expected pre-cleanup
observation is exactly what is implemented: a **non-consuming child-end probe**.

```rust
fn child_has_ended(probe: BorrowedFd<'_>) -> Result<bool, rustix::io::Errno> {
    rustix::process::waitid(
        rustix::process::WaitId::PidFd(probe),
        WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
    ).map(|status| status.is_some())
}
```

`NOWAIT` leaves the end available, so asking cannot itself reap and cannot change what the cleanup
then observes. The case requires `Ok(false)` — the child has **not** ended — and distinguishes the
two failure shapes in its message: `Ok(true)` is a child signalled and not collected, `Err(ECHILD)`
one signalled **and** collected. The first publication's `/proc` presence check would have passed on
the zombie a killing drop leaves behind; this one fails on it.

**B — the disarmed `Drop` did not wait another `POST_KILL_REAP_MS`.** `elapsed` is measured across
`drop(launch)` alone and required `< 250 ms`.

**The cleanup cannot make either assertion vacuous.** Ordering was checked in the source: `probed`
is captured *before* `harness_terminate_and_reap` runs, and the probe is non-consuming. The
assertions are then evaluated, with the `probed` assertion ahead of the cleanup-failure `panic!`,
so a drop that killed **and** reaped is reported as the product fact it is rather than as a cleanup
failure. Cleanup runs on both the passing and failing paths, so no case leaves a background child.

## 10. `P4PUB-02` — test pidfd ownership

**`P4PUB02_TEST_PIDFD_OWNERSHIP_SOUND`.**

```rust
let probe = launch.child.pidfd().try_clone_to_owned().expect(…);
```

| Requirement | Finding |
|---|---|
| Duplicate belongs solely to the test harness | **YES** — an `OwnedFd` local to the case |
| Product `ChildHandle` ownership unchanged | **YES** — `spawn.rs` is byte-identical; `pidfd()` is a pre-existing `pub(crate)` accessor returning `BorrowedFd` |
| Dropping the product handle invalidates the duplicate | **NO** — an ordinary descriptor duplicate survives independently |
| Fd leak | **NONE** — `probe` is closed once when the case ends |
| Double-close | **NONE** — the handle closes its own copy; the harness closes only the duplicate |
| Public pidfd exposure | **NONE** — `pub(crate)`, and the four boundary suites are unchanged |
| Production API change | **NONE** — the only import delta in the whole file is `BorrowedFd` |

## 11. `P4PUB-02` — harness cleanup, zombie oracle

**`P4PUB02_HARNESS_REAP_SOUND`. `P4PUB-02: FIXED`.**

After the load-bearing observations, `harness_terminate_and_reap`
([`backend/tests.rs:2600`](../../crates/helm-launch/src/backend/tests.rs#L2600)) terminates the
deliberately surviving child **and actually reaps it**. The reap is addressed by
`WaitId::PidFd` on the harness's own duplicate — a test-only pidfd wait, with **no product numeric-pid
lifecycle fallback**. The numeric pid is used solely to issue the `kill -9`, after the product facts
have already been observed; that is test-only disposal and touches no product contract.

Cleanup is bounded by a test-owned `HARNESS_CLEANUP_BOUND` of 5 s, documented as having nothing to
do with `POST_KILL_REAP_MS`, which the product owns. Exceeding it, or a refusal indicating something
else collected the child, returns an error that the case turns into a `panic!`. **No background
child survives the case.**

The zombie oracle is correctly demoted. `pid_present` is documented as answering *presence*, not
liveness and not reaping, and `!pid_present(pid)` appears **only as a secondary assertion, after the
reap has been proven**. It is never the sole proof.

The corrected case fails if `Drop` kills unexpectedly (probe ≠ `Ok(false)`), if `Drop` blocks
unexpectedly (`elapsed ≥ 250 ms`), or if the harness cannot reap (cleanup `Err`).

## 12. `P4PUB-03` — root cause, re-established independently

**`P4PUB03_ROOT_CAUSE_CONFIRMED`.**

The original oracle compared `/proc/self/fd` counts before and after a launch. `/proc/self/fd` is
the descriptor table of the **whole test binary**, and the Rust harness runs cases on parallel
threads by default. Any unrelated case opening or closing a descriptor between the two reads moves
the count without the launch having leaked or reclaimed anything.

The preserved evidence is decisive: on the **same SHA**, the `helm-launch` workflow **passed** this
case while the workspace workflow **failed** it with `before = 11, after = 10`. A count that
**falls** is not a descriptor leak. A process-global count is vulnerable to unrelated parallel test
activity, and the two runs differ only in what else was executing alongside it.

## 13. `P4PUB-03` — subprocess isolation

**`P4PUB03_SUBPROCESS_ISOLATION_SOUND`.**

The outer case ([`launch.rs:2786`](../../crates/helm-launch/src/launch.rs#L2786)) builds the fixture
and the working directory, then launches a **dedicated subprocess** — this same test binary, via
`std::env::current_exe()` — passing the prebuilt paths through `HELM_P4_FIXTURE` and
`HELM_P4_WORKDIR`, so the inner process compiles nothing and measures only the launch.

| Requirement | Mechanism |
|---|---|
| Dedicated subprocess | `Command::new(current_exe())` |
| Only the inner case runs | `--exact launch::tests::descriptor_ownership_inner_case` |
| `--exact` | **present** |
| `--test-threads=1` | **present** |
| Ignored/marker mechanism | inner case is `#[ignore = "driven by …, in a dedicated process"]`, invoked with `--ignored` |

**No recursion.** The outer case is not `#[ignore]`d, so `--ignored` excludes it from the inner
process even before the name filter applies. The inner invocation is therefore doubly constrained —
by `--ignored` and by an `--exact` fully-qualified name — and cannot re-enter the outer case.

**No unrelated library test executes** in the inner process, for the same two reasons.

**The outer case fails if the inner case never ran.** `status.success()` alone would be
insufficient, because libtest exits successfully when a filter matches nothing. The case therefore
requires **both** additional oracles:

* `HELM-P4-FD-OWNERSHIP-INNER: ok`, a marker the inner case prints only after its final assertion,
  surfaced by `--nocapture`;
* the literal `1 passed`, with the message *"the inner case did not actually run, so this proved
  nothing"*.

A zero-match run reports `0 passed; … 1 filtered out` and fails both. The execution oracle is
strong.

## 14. `P4PUB-03` — inner oracle and freedom from interference

**`P4PUB03_FD_ORACLE_STRICT_AND_SOUND`. `P4PUB-03: FIXED`.**

Inside the isolated process ([`launch.rs:2759`](../../crates/helm-launch/src/launch.rs#L2759)):

1. `let before = open_descriptor_count();`
2. exactly **one** completed launch, `run(&fixture, &workdir, &Spec::new(&["quiet", "0"]))`
3. `let after = open_descriptor_count();`
4. `assert_eq!(before, after, …)` — **exact equality**
5. `let _ = outcome.into_receipt();` — the outcome is consumed and dropped as intended
6. `assert_eq!(open_descriptor_count(), before, …)` — the final count still equals the **baseline**

**No `<=`, no tolerance, no ignored difference.** The source comment states the reasoning the owner
required: `after <= before` would hide a real leak and a tolerance would hide a small one.

The enumeration is symmetric. `open_descriptor_count` opens a directory descriptor to read
`/proc/self/fd`, that descriptor is one of the entries it counts, and it is opened and closed inside
the function on **every** call. It therefore contributes exactly one to both sides of every
comparison and biases neither.

**`P4PUB-03` interference is removed.** The inner subprocess has its own descriptor table, runs a
single test on a single thread, and shares descriptor-table activity with no unrelated Rust test.
Threads and processes started by the tested launch itself are part of the case and complete under
the normal lifecycle before `after` is taken; step 6 additionally catches any descriptor released
late, during outcome consumption.

## 15. Product byte / semantic identity

**`P4_PRODUCT_SEMANTICS_UNCHANGED_BY_PUBLICATION_CORRECTION`.**

This was proved mechanically by blob comparison, not accepted from the author's statement. Every
file below has an **identical git blob hash** at `94ee48d` and `aed3fd2`:

| File | Blob (identical at both commits) |
|---|---|
| `crates/helm-launch/src/lifecycle.rs` | `39b457b4966eef5b587fb5b2b8eed123b6b14112` |
| `crates/helm-launch/src/model.rs` | `301867b2a252ae6794838291204c7bd95d34eb32` |
| `crates/helm-launch/src/receipt.rs` | `280dbf9e3702bf283c8799482186d4e6ce2f0100` |
| `crates/helm-launch/src/error.rs` | `4d56daefaa7bc6e58b2e1cb3dc2be1681b2189a4` |
| `crates/helm-launch/src/plan.rs` | `f7691ff59575a396411d0027261581a0646d4297` |
| `crates/helm-launch/src/authority.rs` | `3a877d61faf44c81460031b2bf92d07d9f6f4927` |
| `crates/helm-launch/src/lib.rs` | `2deb8650088c0c4afc1b471743ab51b0832bc6e6` |
| `crates/helm-launch/src/layout.rs` | `b878beb9f8a8c5189e95f7fbfe6ef6cfbcccfe83` |
| `crates/helm-launch/src/backend/mod.rs` | `312ab85e90870e080e7a0fc50c5b7b4771b9dfae` |
| `crates/helm-launch/src/backend/spawn.rs` | `90f645c2e060121e5304f745b57b6adfd235ab78` |
| `crates/helm-launch/src/backend/child.rs` | `d8b75c677dcc99618a5dbcff2a6e5f026a64c0db` |
| `crates/helm-launch/src/backend/syscall.rs` | `38d95fb4f8b1a0b36d83c82712a427b05a7deb55` |
| `crates/helm-launch/src/backend/injection.rs` | `fa3704d0e10f55da5231111ba96efc74e1121d86` |
| `Cargo.toml` | `953ca8a35cb74ce9fe08077c1d4ac7dce02f2be3` |
| `Cargo.lock` | `b2df4153d01ce87f1beee8715cb7eb0cdaa63a11` |
| `.github/workflows/helm-launch.yml` | `11f7c0cdbb70b6b0f5039d4765a49ff10a0d43bd` |
| `.github/workflows/helm-evidence.yml` | `cc09fc4af5083b69f947be56cffdd206027650c4` |

`crates/helm-launch/src/launch.rs` contains both product and test code, so its product portion was
compared **mechanically**. The file's `#[cfg(test)] mod tests` opens at line 937/946 and runs to
end-of-file at both commits. Lines 1–936 hash identically:

```
sha256(launch.rs lines 1..936) @ 94ee48d = 8934971849a1be65acc95b6da75bdb421b28dccc003cac475b13a2caf74d8fd5
sha256(launch.rs lines 1..936) @ aed3fd2 = 8934971849a1be65acc95b6da75bdb421b28dccc003cac475b13a2caf74d8fd5
```

Every diff hunk in the file begins at line **1119 or later** — that is, entirely inside the test
module. `crates/helm-launch/src/backend/tests.rs` is declared `#[cfg(test)] pub(crate) mod tests;`
in `backend/mod.rs`, so it is wholly test code and compiles into no product artifact.

Consequently, and by construction rather than by assertion:

| Surface | State |
|---|---|
| Launch runtime policy | **UNCHANGED** |
| `Lifecycle` | **UNCHANGED** |
| `LaunchOutcome` public API | **UNCHANGED** |
| Receipt schema | **UNCHANGED** |
| Receipt serializer | **UNCHANGED** |
| Group sweep behaviour | **UNCHANGED** |
| `ChildHandle` product behaviour | **UNCHANGED** |
| Stream fairness | **UNCHANGED** |
| Deadlines | **UNCHANGED** |
| Signal handling | **UNCHANGED** |
| Backend child contract | **UNCHANGED** |
| `unsafe` boundary | **UNCHANGED** |
| Cargo dependency graph | **UNCHANGED** (both manifests identical) |
| CI structure | **UNCHANGED** (both workflows identical) |

**A full P4 review is not required.**

## 16. `unsafe` delta

**`P4PUB_CORRECTION_UNSAFE_DELTA_ZERO`.**

The diff over the correction range contains **no added or removed line matching `unsafe`**. Per-file
occurrence counts are identical at both commits:

| File | `94ee48d` | `aed3fd2` |
|---|---|---|
| `backend/child.rs` | 28 | 28 |
| `backend/mod.rs` | 11 | 11 |
| `backend/spawn.rs` | 8 | 8 |
| `backend/injection.rs` | 3 | 3 |
| `backend/tests.rs` | 3 | 3 |
| `backend/syscall.rs` | 2 | 2 |
| `launch.rs` | 4 | 4 |
| `lib.rs` | 4 | 4 |

**Zero new `unsafe`, inside or outside the accepted backend boundary.** The fixtures that need
`getrlimit`, `setrlimit` and `raise` remain separate C programs compiled at test time, so no
foreign interface is named in Rust and the crate-root denials still hold.

## 17. Workflow reachability

**`P4PUB_CORRECTED_TESTS_CI_REACHABLE`.**

**No workflow modification was necessary**, and none was made — both workflow files are
byte-identical.

| Corrected case | Reached by |
|---|---|
| `launch::tests::a_signalled_child_keeps_its_signal_number_and_its_core_flag` | `cargo test -p helm-launch --locked`; `… --lib -- --nocapture launch::tests::` |
| `launch::tests::the_outcome_owns_no_descriptor_and_consumes_its_authorisation` | same |
| `launch::tests::descriptor_ownership_inner_case` | driven by its outer case, in a dedicated process |
| `backend::tests::a_disarmed_drop_guard_neither_signals_nor_waits` | `cargo test -p helm-launch --locked`; `… --lib -- --nocapture backend::tests::` |

All are plain `#[test]` cases needing no feature, so they execute under both
`cargo test -p helm-launch --locked` and `cargo test --workspace --locked` on hosted Linux. The
inner case is `#[ignore]`d **deliberately and correctly**: it is not hidden behind a feature, it is
driven by its own outer case, which the default suite does run. That is the intended shape, not a
case the default suite never enables.

The off-cohort invariant is preserved. `mod launch` and `mod backend` are gated
`#[cfg(all(target_os = "linux", target_arch = "x86_64"))]` in the unchanged `lib.rs`, so the new
inner case does not leak into the non-Linux listing. This was verified on this Windows host:
`cargo test -p helm-launch --locked --lib -- --list` yields **zero** `launch::` or `backend::`
entries, so the workflow's "Confirm no backend or launch test exists off the cohort" step still
holds.

All three first-publication failures occur in the default suite and therefore **can close before the
later named CI steps**, which is what the first publication prevented.

## 18. Historical evidence

**`P4_FIRST_PUBLICATION_HISTORY_PRESERVED`.** See §2. Runs `35499943908` and `35499943903` are both
recorded as **attempt 1, FAILURE**, with **NO RETRY, NO RERUN, NO REPLACEMENT**. No correction text
states or implies that either run is fixed or passed. A corrected SHA will create **new** evidence
under new natural run identifiers rather than revising these.

## 19. Local validation

Executed in this session at `aed3fd2`, on Windows 11, `rustc 1.95.0`:

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo test --workspace --locked` | **PASS** — 0 failures |
| `cargo test -p helm-launch --locked` | **PASS** |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — 831 run, 74 skipped, 0 failures |
| `python tools/validate_docs.py` | **PASS** — 142 markdown, 255 JSON, 1630 link targets |
| `git diff --check` | **PASS** |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | **PASS** |

The Linux cross-target run was **forced to recompile** rather than accepted from cache, and its
verbose invocation was inspected to confirm it is the one that matters: the lib `--test` unit is
compiled with `--target x86_64-unknown-linux-gnu`, so the Linux-only `mod launch` and `mod backend`
— including all corrected test code — were genuinely type-checked and clippy-clean under the
crate-root denials. This closes compilation, **not** runtime.

**P4 FIRST-PUBLICATION CORRECTION LINUX RUNTIME: PENDING NEW PUBLICATION CI.**

Every corrected case is Linux-only and this host cannot execute them. Cross-compilation does not
close runtime, and this re-review does not claim it does.

## 20. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `R-P4PUB-I1` | `IMPORTANT` | repository policy | n/a — commit metadata | `cfe2537` and `aed3fd2` each carry a `Co-Authored-By: Claude Opus 5 (1M context)` trailer. `AGENTS.md` §"Autorstvo commit-a" forbids a `Co-Authored-By` trailer for Claude or any other agent and states the rule is stronger than a tool's default instruction. These are the **only two** commits on this branch carrying one; every other commit, including the published `94ee48d`, complies | **OWNER DECISION.** Not a product, test or evidence defect and it changes nothing about `P4PUB-01`…`03`. Both commits are still **unpublished** (`origin` is at `94ee48d`), so amending them rewrites **no** published history. After publication the trailers become permanent under the repository's own no-history-repair discipline. This session is forbidden to amend, rebase or squash, so it reports rather than repairs |
| — | `GATE_PENDING` | hosted runtime | Linux CI | No supported Linux execution occurred in this re-review; the corrected cases are structurally reviewed and cross-target type-checked, not run | **P4 FIRST-PUBLICATION CORRECTION LINUX RUNTIME: PENDING NEW PUBLICATION CI** |

**0 `BLOCKER`. 1 `IMPORTANT`. 0 new `MINOR`. 0 new `BACKLOG_NONBLOCKING`.**

No finding was raised against `P4PUB-01`, `P4PUB-02` or `P4PUB-03`. All three are **FIXED**, no
oracle was weakened, and no speculative unrelated concern is attached to this gate.

## 21. Boundary of this re-review

This re-review is recorded in documentation only. It changed no product code, test, workflow, Cargo
file, ADR contract, experiment or evidence; it did not fix code, amend, rebase, squash, push or
touch `main`; it did not rerun any historical CI; it promotes no traceability row and **accepts no
P4 gate**. Accepting P4 remains the owner's decision.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, PUBLISHED ONCE WITH A FAILED HOSTED
VALIDATION, CORRECTED IN TEST AND EVIDENCE ONLY, AND INDEPENDENTLY RE-REVIEWED — AND IS NOT
ACCEPTED. P5 IS NOT AUTHORISED. THE COMPLETE helm-launch 0.1 MODULE IS NOT PRODUCT-ACCEPTED.**

**Next gate: OWNER DECISION ON `R-P4PUB-I1`, THEN ONE FAST-FORWARD PUBLICATION OF THE REVIEWED
CORRECTION CHAIN, THEN NEW NATURAL HOSTED P4 CI.**
