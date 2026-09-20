# FRESH INDEPENDENT WHOLE-CRATE REVIEW OF HELM-LAUNCH 0.1

> **Review only. No product code, test, schema, vector, workflow or authority document was edited.
> Nothing was pushed. P5 is not accepted by this document, and no trial is authorised by it.**

| Item | Value |
|---|---|
| Repository | `D:\HELM\helm-os` |
| Branch | `docs/helm-launch-architecture` |
| Published accepted P4 base | `06223035828bfc6fec249bad7a5f2b5d520a7d59` |
| P5 authority commit | `9466c9fa7aba6bc2b6a3e1a4d381a00229056f3e` |
| P5 implementation candidate | `67a45022c54a14341a0c817a0df2d04404a9c343` |
| Local `HEAD` at review | `67a45022c54a14341a0c817a0df2d04404a9c343` |
| `origin/docs/helm-launch-architecture` | `06223035828bfc6fec249bad7a5f2b5d520a7d59` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Relation to `origin/docs/helm-launch-architecture` | **2 ahead, 0 behind**, linear, no merge |
| Worktree | **clean** before and after the review |
| Review host | Windows 11, `rustc`/`cargo` 1.95.0, Python 3.14.3 |
| Review date | 2026-09-20 |

**Result: 0 BLOCKER, 1 IMPORTANT, 6 MINOR.**
**Classification: `HELM_LAUNCH_P5_WHOLE_CRATE_REVIEW_NEEDS_FIX`.**

---

## 0. Session provenance and independence

This review session **authored or modified neither P5 commit**. It did not write
`9466c9fa7aba6bc2b6a3e1a4d381a00229056f3e` and did not write
`67a45022c54a14341a0c817a0df2d04404a9c343`. Independence is asserted from this session's own
history, not inferred from Git author metadata, as the instruction requires. The session began with
the repository already at `67a4502` and a clean worktree, and its only write is this file and the
one docs-only commit that carries it.

`origin` was fetched **read-only**. No branch was created, reset, rebased, amended or pushed. No
history was repaired.

The ancestry was confirmed by parent walk, not by log order:

```
06223035828bfc6fec249bad7a5f2b5d520a7d59
  -> 9466c9fa7aba6bc2b6a3e1a4d381a00229056f3e   (one parent)
    -> 67a45022c54a14341a0c817a0df2d04404a9c343 (one parent)
```

No commit in the range has two parents.

---

## 1. Authority read before the implementation

Read in full before any implementation file was opened:

* `AGENTS.md`
* `docs/adr/ADR-0024-launch-authority.md` — Accepted 2026-09-17, sections A to N, the falsifier list
  and the owner-decision table
* `docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md` — the current authority header and the
  [P5 authority note of 2026-09-20](HELM-LAUNCH-PRODUCTIZATION-PLAN.md#p5-authority-2026-09-20)
* `docs/DECISIONS.md` — the P1 to P4 acceptances and the
  [P5 authorisation](../DECISIONS.md#helm-launch-p5-authorised)
* `docs/PROJECT_STATE.md`
* `crates/helm-launch/README.md`
* `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` — the P5 receipt schema document
* `docs/implementation/helm-launch-receipt-0.1-test-vectors.json` — the P5 receipt vector artifact

Historical review artifacts (`HELM-LAUNCH-P1-INDEPENDENT-REVIEW.md`,
`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md`, `HELM-LAUNCH-P4-INDEPENDENT-LIFECYCLE-REVIEW.md`,
`HELM-LAUNCH-P4-P4PUB04-REREVIEW.md`, `HELM-LAUNCH-P4-P4PUB05-REREVIEW.md`) were consulted **only**
for finding provenance. Every claim below was re-established against the current tree.

Current accepted contract, as read: P1, P2, P3 and P4 **ACCEPTED**; P5 **AUTHORISED, NOT YET
ACCEPTED**; complete helm-launch 0.1 **NOT YET PRODUCT-ACCEPTED**; Trial #3 frozen
**`MECHANISM_REJECTED`**; **no Trial #4 authorised**.

---

## 2. Complete product inventory

### 2.1 Files reviewed

Every file below was opened and read in this session.

| Path | Lines | Role |
|---|---|---|
| `crates/helm-launch/Cargo.toml` | 85 | manifest, lint table, cohort-gated dependencies |
| `crates/helm-launch/README.md` | 539 | crate-facing documentation |
| `crates/helm-launch/src/lib.rs` | 306 | crate root, public re-exports, cohort gates, `compile_fail` doctests |
| `crates/helm-launch/src/plan.rs` | 821 | hostile-input plan parser and `ValidatedLaunchPlan` |
| `crates/helm-launch/src/model.rs` | 698 | closed fact vocabulary, `ReceiptRecord` |
| `crates/helm-launch/src/receipt.rs` | 1283 | deterministic serializer, `LaunchReceipt`, P5 vector tests |
| `crates/helm-launch/src/error.rs` | 858 | plan, admission, refusal and launch error vocabularies |
| `crates/helm-launch/src/layout.rs` | 594 | pure descriptor-layout planner |
| `crates/helm-launch/src/lifecycle.rs` | 1318 | pure lifecycle state machine |
| `crates/helm-launch/src/authority.rs` | 1350 | capability admission and single-use authorisation |
| `crates/helm-launch/src/launch.rs` | 2850 | public `launch`, observation loop, receipt emission |
| `crates/helm-launch/src/backend/mod.rs` | 673 | backend boundary, scoped `allow(unsafe_code)` |
| `crates/helm-launch/src/backend/spawn.rs` | 719 | preparation, signal mask, `clone3`, `ChildHandle` |
| `crates/helm-launch/src/backend/child.rs` | 503 | the closed post-clone child sequence |
| `crates/helm-launch/src/backend/syscall.rs` | 400 | the one raw x86_64 syscall shim |
| `crates/helm-launch/src/backend/injection.rs` | 111 | test-only fault injection, doubly gated |
| `crates/helm-launch/src/backend/tests.rs` | 2821 | backend test support and Linux cases |
| `crates/helm-launch/tests/plan_contract.rs` | 863 | Level 1 plan contract |
| `crates/helm-launch/tests/p2_boundary.rs` | 1303 | Level 1 boundary and lint-drift scans |
| `crates/helm-launch/tests/p3_boundary.rs` | 1094 | tree-walking unsafe-confinement and child-vocabulary scans |
| `crates/helm-launch/tests/p4_boundary.rs` | 700 | lifecycle, sweep, `Err` boundary and public-surface scans |
| `crates/helm-launch/tests/p5_regressions.rs` | 714 | **new in P5** — Level 4 rows A, B, C, H, J, the current-truth scan and the executable inventory |
| `crates/helm-launch/tests/linux_admission.rs` | 819 | Level 2 admission cases |
| `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` | 289 | **new in P5** — published receipt evidence contract |
| `docs/implementation/helm-launch-receipt-0.1-test-vectors.json` | 11 vectors | **new in P5** — published exact-byte vectors |
| `tools/helm_launch_release_reachability.py` | 133 | **new in P5** — release reachability checker |
| `tools/tests/test_helm_launch_release_reachability.py` | 153 | **new in P5** — its paired controls |
| `tools/helm_launch_child_closure.py` | — | accepted P3 machine-code gate and assembly parser |
| `tools/tests/test_helm_launch_machine_proofs.py` | — | the parser's controls |
| `.github/workflows/helm-launch.yml` | 382 | the hosted gate set |

`tests/p3_boundary.rs` walks `src/` and `tests/` at run time and fails on any file its inventory
(15 sources, 6 test files) does not name, so this file list cannot silently gain a member.

### 2.2 Public API by cfg cohort

**Portable (all platforms).** `parse_launch_plan`, `ValidatedLaunchPlan`, `ExecutionKind`,
`StdinMode`, `TerminationSignal`, the nine plan bound constants; `Digest`, `Stream`,
`EnvironmentMode`, `Backend`, `ElfType`, `ChildStage`, `IndeterminateReason`, `ExecStatus`,
`ChildEnd`, `GroupSweep`, `Completeness`, `AssertedContext`, `ExecutableMeasurement`, `Termination`,
`StreamFacts`, `ReceiptRecord`, `MAX_ID_BYTES`; `LaunchReceipt`, `MAX_RECEIPT_BYTES`;
`LaunchPlanError(s)`, `LaunchPlanErrorCode`, `MAX_PLAN_ERRORS`, `AdmissionError(Code)`,
`AuthorizationRefusal(Code)`, `LaunchError(Code)`, `PreparationStep`.

**Linux x86_64 only.** `admit_executable`, `admit_working_directory`, `authorize`,
`ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`, `MAX_EXECUTABLE_BYTES`,
`launch`, `LaunchOutcome`.

**Absent off the cohort.** All nine names above; verified by the `compile_fail` doctests in
`src/lib.rs` (lines 160–190) and by the fact that `mod authority`, `mod backend` and `mod launch`
are each `#[cfg(all(target_os = "linux", target_arch = "x86_64"))]`.

**Never public on any platform.** `backend`, `SpawnedChild`, `PreparedLaunch`, `ChildHandle`,
`MinimalLaunch`, `SpawnedForLifecycle`, `Fault`, `ChildPlan`, `NotPosed`, `layout`, `lifecycle`,
any pid, pidfd, raw descriptor, process-group id or fault selector. Eight `compile_fail` doctests
in `src/lib.rs` and the `p4_boundary` public-surface scans hold that.

---

## 3. P5 delta — independent boundary review

`git diff 0622303..67a4502` touches 19 files, +2144/−49.

I classified every changed hunk and then **mechanically verified** the classification: for each
changed `src/` file I stripped line comments and everything from the first `#[cfg(test)]`, and
compared the two revisions.

| File | Class | Mechanical result |
|---|---|---|
| `src/backend/mod.rs` | comment only (2 doc comments) | product region **byte-identical** |
| `src/backend/spawn.rs` | comment only (1 doc comment) | product region **byte-identical** |
| `src/receipt.rs` | test only (+349, all inside `#[cfg(test)] mod tests`) | product region **byte-identical** |
| `src/backend/tests.rs` | test only — the whole file is behind `#[cfg(test)] pub(crate) mod tests;` in `backend/mod.rs` | `NotPosed`, `set_and_assert_mode`, canonical fixture root, 7 call-site conversions |
| `tests/linux_admission.rs` | test comment only | 2 comments |
| `tests/p2_boundary.rs` | test only | the `include_str!` rule refinement of §8 |
| `tests/p3_boundary.rs` | test only | `EXPECTED_TESTS` 5 → 6 |
| `tests/p5_regressions.rs` | test only, new file | Level 4 |
| `crates/helm-launch/README.md` | documentation | current truth + the Level 1–4 inventory |
| `docs/**` (5 files, 2 new) | documentation | authority sync, schema, vectors |
| `tools/helm_launch_release_reachability.py` + its tests | CI / tooling, new | `P4PUB05-R2` |
| `.github/workflows/helm-launch.yml` | CI / tooling | `P4PUB05-R1`, `--no-fail-fast`, the P5 named step, two path entries |

**Untouched by P5:** `src/lib.rs`, `src/plan.rs`, `src/model.rs`, `src/error.rs`, `src/layout.rs`,
`src/lifecycle.rs`, `src/authority.rs`, `src/launch.rs`, `src/backend/child.rs`,
`src/backend/syscall.rs`, `src/backend/injection.rs`, `tests/p4_boundary.rs`,
`tests/plan_contract.rs`, `Cargo.toml`, `Cargo.lock`.

The author's claim that P5 introduces no new product semantics is **independently confirmed**. The
receipt serializer in particular is byte-identical to the accepted P4 one, which is the strongest
available statement that the published evidence contract documents the accepted code rather than a
P5 variant of it.

**`P5_PRODUCT_SEMANTIC_DELTA_ZERO`.**

---

## 4. Public API

The complete public surface was enumerated from source, not from documentation, and compared with
the authorised one. No `pid`, `pidfd`, raw descriptor, backend type, `SpawnedChild`,
`PreparedLaunch`, `ChildHandle`, process-group id, fault-injection selector or unsafe syscall
surface appears anywhere in it.

Construction and conversion surfaces:

* `ExecutableCapability`, `WorkingDirectoryCapability` and `AuthorizedLaunch` carry **no `derive`
  at all** — no `Clone`, `Copy`, `Default`, `From`, `Serialize` or `Deserialize`, and no public
  constructor or setter. `into_parts` and `descriptor` are `pub(crate)`.
* No `Serialize`/`Deserialize` derive exists anywhere in `crates/helm-launch/src`.
* `LaunchOutcome` has no constructor, no `Clone`, no `Default` and no `Deserialize`; its `Debug`
  prints digests and lengths only.
* `LaunchReceipt` derives `Clone, PartialEq, Eq, Hash` — correct for inert data — and its fields
  are private, so it cannot be built from parts or from bytes (three `compile_fail` doctests).
* `Digest::from_raw` is `pub(crate)`, so a caller cannot mint digests.

The only edge from an authorisation to a process is `launch`, it consumes its argument, and the
`compile_fail` doctests prove a second call does not compile.

**`WHOLE_PUBLIC_API_SOUND`.**

---

## 5. Plan parser and input contract

Reviewed as hostile-input code.

* **Input bound** `MAX_PLAN_BYTES = 32_768`, checked before anything is examined.
* **Duplicate keys** are refused on **decoded** keys: `Tree::visit_map` calls
  `input.next_key::<String>()`, which has already decoded JSON escapes, and rejects on
  `object.contains_key`. `serde_json::Value` is deliberately never used for untrusted bytes,
  because its object visitor keeps the last duplicate.
* **Nesting bound** `MAX_JSON_DEPTH = 8`, enforced in both `visit_map` and `visit_seq` before any
  element is consumed.
* **No untrusted size hint is ever reserved from** (`BTreeMap::new`, `Vec::new`, and
  `Vec::with_capacity(items.len().min(MAX_ARGS))` for argv).
* **Unknown fields, missing fields and type mismatches** have their own closed codes; no input text
  is echoed.
* **NUL rejection** on decoded argv bytes (`s.as_bytes().contains(&0)` → `ArgContainsNul`).
* **argv bounds**: 1..=64 elements, ≤ 4 096 bytes each, ≤ 131 072 total.
* **Environment** admits only `empty`; **stdin** only `closed_pipe_eof`; **execution kind** only
  `linux_exact_executable`; **termination signal** only `SIGTERM`.
* **Capture bound** `MAX_CAPTURE_BYTES = 65_536`; **timeout** 1..=600 000 ms; **grace** 0..=60 000 ms.
* **Logical-id grammar** `[a-z0-9][a-z0-9._-]{0,79}`, `MAX_ID_BYTES = 80`.
* **Digest grammar**: 64 lowercase hexadecimal characters.
* **Errors** are ordered, stable and capped at `MAX_PLAN_ERRORS = 64` with an explicit `ErrorLimit`
  marker so truncation is visible rather than silent.

A `ValidatedLaunchPlan` holds no descriptor, names no executable and has no method that reaches
`execveat`. `authorize` cannot be called without two capabilities that only `admit_*` can produce
from an owned descriptor. **No parser path grants execution authority.**

`P1-TEST-01` (the escaped duplicate-key vector) remains open and is untouched by P5; the reviewed
behaviour is correct, only the committed vector is a byte-identical literal duplicate.

**`WHOLE_PLAN_CONTRACT_SOUND`.**

---

## 6. Capability and authority model

`admit_executable` runs the accepted order exactly (ADR-0024 section C, `src/authority.rs:680-760`):

1. access mode — `O_RDONLY` only; `O_PATH`, `O_WRONLY` and `O_RDWR` refused;
2. `fstat` — regular file only; first metadata sample;
3. set-ID refusal — `S_ISUID` or `S_ISGID` refused;
4. size bound — above `MAX_EXECUTABLE_BYTES = 536_870_912`, refused before any body byte;
5. ELF cohort — 64-byte header read positionally at offset 0;
6. measurement — positional reads, fixed 64 KiB buffer, SHA-256;
7. detected instability — second metadata sample compared on size, mtime and ctime with
   nanoseconds, plus the byte count; **refusal** on any difference;
8. the capability, holding only inert facts.

Working-directory admission accepts an `O_RDONLY` directory with a caller identifier, refuses
`O_PATH` and non-directories, and checks no search permission.

The caller supplies already-open descriptors. There is **no path-based acquisition anywhere in
product code**: `p2_boundary.rs::product_code_names_no_host_state_and_no_pathname_api` forbids
`File`, `OpenOptions`, `open`, `openat`, `canonicalize`, `current_dir`, `read_dir`, `metadata` and
the rest in the product region of every portable source.

`authorize` performs zero I/O and refuses before any process exists when the plan's
working-directory identifier differs from the capability's. A plan error, an admission refusal or an
authorisation refusal produces **no receipt**, and the refused values' descriptors close.

`AuthorizedLaunch` is single-use (consumed by `launch`), not `Clone`, not `Default`, not
`Deserialize`, and not constructible from a receipt or from any data. **No receipt becomes
authority**: nothing in the crate reads receipt bytes at all.

**`WHOLE_AUTHORITY_MODEL_SOUND`.**

---

## 7. Measurement semantics — the load-bearing nonclaim

The nonclaim is that a pre-execution measurement is **not** a claim that the measured bytes are the
executed bytes. I checked every place a reader could pick up the opposite impression:

* **Field names** are `pre_exec_body_size`, `pre_exec_body_sha256`, `pre_exec_mode_bits` in both the
  model and the serialised receipt — the vectors emit exactly those spellings.
* **`src/lib.rs`** states it twice, including "never the identity of bytes that executed and never a
  statement about an interpreter, a shared library or any other part of a loaded-code closure".
* **README** "Non-claims" restates it and adds that not detecting instability **proves nothing**.
* **Schema document §4.4** — "**Pre-exec means pre-exec.** … It is not a claim about what ran, and
  it is not re-measured afterwards."
* **Tests** prove the behaviour, not only the prose: mutation after admission
  (`the_admitted_descriptor_is_executed_after_its_pathname_is_replaced`), a mode change after
  admission (`linux_admission.rs:479`, `:646`) and the instability-protocol tests in
  `authority.rs:1055`.

No text or value in the crate, the README, the schema or the vectors implies measured == executed.

**`WHOLE_MEASUREMENT_NONCLAIM_SOUND`.**

---

## 8. `unsafe` confinement

Enumerated every occurrence of the token as code:

| File | `unsafe {}` blocks | `unsafe fn` |
|---|---|---|
| `src/backend/child.rs` | 23 | 4 |
| `src/backend/spawn.rs` | 6 | 1 |
| `src/backend/injection.rs` | 2 | 1 |
| `src/backend/syscall.rs` | 1 | 1 |
| `src/backend/mod.rs` | 0 | 0 |
| `src/backend/tests.rs` | 0 | 0 |
| everywhere else | 0 | 0 |

The only occurrences outside `src/backend/` are prose (`src/launch.rs:24,26,1184,2567` — "This file
contains **no `unsafe`**") and two test comments. `p3_boundary.rs` re-establishes this by walking
the tree rather than from a fixed list, and fails if the token appears as code outside the four
backend files, if a second `#![allow(unsafe_code)]` appears, or if any test source relaxes the lint.

**P5 adds zero new `unsafe`.** The four `unsafe` occurrences in the P5 diff are all prose in
authority documents.

One `asm!` exists, in `src/backend/syscall.rs`, with pinned clobbers and no `options(`. **No
`extern` block exists anywhere in the crate** — the only `extern` hits are comments explaining its
absence. `libc` is a constants-only dependency, pinned by `const` assertions; `global_asm`,
`naked_asm`, `no_mangle` and `link_section` are forbidden tree-wide.

**`WHOLE_UNSAFE_CONFINEMENT_SOUND`.**

---

## 9. The closed child contract

`child_main` walks the accepted stage list in the accepted order, verified line by line in
`src/backend/child.rs`:

`DUP2` (3 calls) → `CLEAR_CLOEXEC` (3 `fcntl`) → `CHDIR` (`fchdir`, before the range close because
the directory descriptor is not preserved by it) → `CLOSE_RANGE` (three spans, borrowed rather than
copied so no `memcpy` is lowered) → `SETPGID(0,0)` → `SIGACTION` (1..=64 except `SIGKILL`/`SIGSTOP`,
while delivery is still fully blocked) → `SIGMASK` (the empty mask, only after the dispositions are
ready) → `NO_NEW_PRIVS` → `EXEC` (`execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`).

The file carries `#![no_implicit_prelude]`; `p3_boundary.rs` forbids a closed list of `std`,
`alloc`, `rustix`, allocation, formatting, printing and panicking vocabulary inside it, requires
each stage's syscall constant to appear, requires `#[inline(never)]` on the entry, requires the
`ChildPlan` to be taken by reference rather than by value, and forbids `NR_READ` in the child's own
sequence. The two exits are a successful `execveat` and `exit_group(127)` after one fixed 8-byte
record. The test-injection check sits at the stage boundary and is compiled only under the
non-default feature **and** `debug_assertions`.

Source proof and machine-code evidence are separate and neither substitutes for the other, as the
owner's `P3R-02` disposition requires.

**`WHOLE_CHILD_WINDOW_SOUND`.**

---

## 10. Machine-code closed-world proof

I reviewed `tools/helm_launch_child_closure.py`'s `Assembly` parser and `closure()` independently of
the prior reviews, and read the 30-odd committed controls in
`tools/tests/test_helm_launch_machine_proofs.py`.

| Requirement | Evidence |
|---|---|
| root resolution sound | `find_unique` anchors on the Itanium `17h` marker; ambiguity raises |
| direct call traversal | `CALL_MNEMONICS` in `ESCAPING_MNEMONICS`; followed transitively |
| direct tail-jump traversal | `UNCONDITIONAL_JUMP_MNEMONICS`; `test_a_direct_tail_jmp_to_an_internal_helper_is_traversed` |
| conditional branch traversal | `CONDITIONAL_JUMP_MNEMONICS`; six dedicated controls incl. GOT and prefixed forms |
| indirect transfer fail-closed | `operand.startswith("*")` → `indirect`, never a target |
| unsupported branch fail-closed | `is_branch_like` backstop → `unsupported`; `test_an_unsupported_branch_mnemonic_fails_closed` |
| missing root fail-closed | `test_a_missing_root_fails_loudly_rather_than_passing` |
| empty body fail-closed | `helm_launch_child_closure.py:493` |
| forbidden external edges fail | `categorise` + `test_a_memcpy_edge_in_the_child_fails`, `..._panic_edge_...`, `..._reached_only_transitively_fails` |
| positive controls live | `test_the_positive_control_fails_when_nothing_forbidden_exists` proves the control itself can fail |
| both profiles | workflow steps *Child machine-code closure — debug/test profile* and *— release codegen* |
| `P3R-15` vocabulary | `test_the_p3r15_reproduction_fails`, `test_the_branch_vocabulary_is_the_canonical_one` |

A local label the body does not define is `unresolved`, not ignored — the distinction that makes the
"intra-function branch" concession safe.

**`WHOLE_MACHINE_CODE_PROOF_SOUND`.**

---

## 11. Process creation, signal mask, pidfd

`spawn::spawn` (`src/backend/spawn.rs:339-459`):

1. the whole child plan is built in parent memory first;
2. `rt_sigprocmask(SIG_SETMASK, FULL_KERNEL_SIGSET, &saved)` — a **raw** full kernel signal set, so
   glibc's two internal real-time signals are included; a failure here returns before anything is
   created and the mask is unchanged;
3. `clone3` with `CLONE_PIDFD` and `exit_signal = SIGCHLD`; `const` assertions prove `CLONE_VM`,
   `CLONE_FILES`, `CLONE_VFORK` and `CLONE_THREAD` are all absent from the flag word;
4. **`setpgid(child, child)` is the first system call after `clone3`** — the two conversions before
   it (`i32::try_from`, `Pid::from_raw`) are pure arithmetic;
5. the pidfd is taken into an `OwnedFd` (bookkeeping, not a syscall) and moved into `ChildHandle`;
6. the mask is restored **after** the attempt, and on `spawn_for_lifecycle`'s path a restore failure
   is carried as a dropped fact rather than as an error, because a child exists by then.

There is **no `pidfd_open` fallback** and no numeric-PID direct-child fallback:
`p3_boundary.rs` forbids `pidfd_open` and `fork` in the backend, and
`p4_boundary.rs::the_direct_child_is_never_signalled_or_waited_for_by_numeric_pid` forbids
`waitpid`, `wait4` and `kill_process` tree-wide. Every direct-child signal is
`pidfd_send_signal(self.child.pidfd(), …)` and every wait is `waitid(WaitId::PidFd(...))`. The only
pid-valued call in the whole crate is the guarded group sweep.

`ENOSYS`/`EPERM` from `clone3` becomes `ProcessCreationUnavailable` with no child and no receipt.

**`WHOLE_PROCESS_CREATION_SOUND`.**

---

## 12. Group authority

`group_authority_established` is set on exactly one expression:
`setpgid(Some(pid), Some(pid)).is_ok()` in the parent, immediately after `clone3`. There is no
retry, no promote-on-`EACCES`, and no other writer.

Nothing infers it. `p4_boundary.rs::no_source_infers_group_authority_from_an_observed_process_group`
forbids `getpgid`, `getpgrp` and `tcgetpgrp` in every `src/` file and, separately, in the product
half of `src/launch.rs`. The child's own `setpgid(0, 0)` is documented in `child.rs` as establishing
nothing. No fixture report and no pid equality feeds the decision.

The `P3R-21` correction remains conceptually sound: because the parent's `setpgid` can legitimately
lose the race to an already-executed child and be answered `EACCES`, authority is genuinely
scheduler-dependent in an ordinary launch. P4 therefore added a **test-only** stall coordination
(`launch_with_stall_coordination`, compiled only under `cfg(test)` **and** the non-default feature
**and** `debug_assertions`) so the sweep tests do not silently complete having issued no group
signal — and the observation loop it drives is the unmodified product `Observer`.
`p4_boundary.rs::the_test_only_coordination_seam_is_not_reachable_from_the_public_launch` holds
that seam shut.

**`WHOLE_GROUP_AUTHORITY_SOUND`.**

---

## 13. The `Err`-versus-receipt boundary

The public entry point is four statements:

```rust
let started = Instant::now();
let spawned = backend::spawn_for_lifecycle(authorized).map_err(to_launch_error)?;
Ok(Observer::new(spawned).run(started))
```

`p4_boundary.rs::launch_returns_err_only_before_a_child_exists` asserts the body contains **exactly
one `?`**, that it is the pre-child spawn, and that the function ends in an unconditional `Ok`. I
re-derived the same by reading the compacted body.

`spawn_for_lifecycle` has two `?`: `spawn::prepare` (before `clone3`) and `spawn::spawn`. Inside
`spawn::spawn`, every `Err` return except one is raised before `clone3` succeeded. The exception is
`BackendError::PidfdNotProvided` at `spawn.rs:436`, reached only if `clone3` returned success with
`CLONE_PIDFD` and yet left `pidfd_slot < 0` — which the kernel contract makes impossible, and in
which case no `ChildHandle` could exist and there would be no authorised way to reach or observe the
child. It is already recorded as `P3R-03` (MINOR, open) and `F-P4-M2` (MINOR, record); P5 changes
nothing bearing on it, and no numeric-pid fallback is authorised. I **carry** it unchanged rather
than escalating.

Once the child exists, every later failure becomes a fact: a `poll` that cannot report readiness
drives a deadline instead of returning; `signal`, `sweep_group` and the reap all discard their
errors into the model's vocabulary; a mask-restore failure is dropped as a fact about the launching
thread.

**`WHOLE_ERR_RECEIPT_BOUNDARY_SOUND`** (with `P3R-03` / `F-P4-M2` carried).

---

## 14. Exec status — no success claim

There is no `ExecSucceeded`, `Launched`, `Started`, `Ran`, `exec_confirmed` or any success verdict
in the model, the serializer, the schema, the vectors, the README or the public API.

* `ExecStatus` has exactly two variants: `PreExecFailure { stage, errno }` and
  `Indeterminate(reason)`. A clean status EOF classifies as
  `Indeterminate(StatusEofWithoutRecord)` (`lifecycle.rs:330-342`), and that same event — and only
  that event — starts the run deadline.
* `p4_boundary.rs::no_exec_success_containment_or_authenticity_vocabulary_exists` walks the **whole**
  `src/` tree (not only the portable subset) forbidding `ExecSucceeded`, `exec_succeeded`,
  `is_success`, `contained`, `containment`, `sandbox`, `sandboxed`, `signature`, `signed`,
  `verify_receipt` and `authenticate`.
* `p2_boundary.rs::no_function_offers_a_success_reading` allows exactly four public `bool`-returning
  names, each a named receipt fact or `contains`.
* Four `compile_fail` doctests in `src/lib.rs` prove there is no `Into<bool>`, no truthiness, no
  `is_success()` and no `ExecStatus::ExecSucceeded`.
* Fixture reports in `src/backend/tests.rs` prove a fixture executed **as test evidence only**;
  `report_fixture_reports_without_the_backend` and
  `the_producer_report_schema_is_closed_and_a_malformed_report_is_refused` keep the producer and the
  consumer apart, and no product field reads a report.

I additionally decoded all 11 published vectors and extracted the 66 distinct quoted tokens they
emit; **none** matches the forbidden verdict vocabulary under whole-token or `_`/`-`/`.`-separated
word comparison.

**`WHOLE_NO_EXEC_SUCCESS_CLAIM`.**

---

## 15. Lifecycle model and real-loop agreement

`src/lifecycle.rs` is the single policy. `src/launch.rs` is its adapter and embeds no second policy:
every deadline, classification, latch and sweep decision is the model's. I mapped the adapter
completely:

| Real observation | Event | Model action | Real operation |
|---|---|---|---|
| `POLLIN` on status | `StatusReady` | `ReadStatus` | one bounded `read` |
| 8-byte record | `StatusRecord` | — | — |
| 9th byte / short record + EOF / bad stage / non-zero pad | `StatusMalformed` | — | — |
| zero-byte read | `StatusEof` | — | starts the run deadline only on a clean EOF |
| read error | `StatusReadFailed` | — | — |
| `POLLIN` on a stream | `StreamReady` | `ReadStream` | one bounded `read` |
| zero-byte read | `StreamEof` | — | — |
| read error | `StreamReadFailed` | — | — |
| pidfd `POLLIN` | `ChildEndReadable` | — | pidfd leaves the poll set |
| monotonic deadline | `DeadlineReached` | `SendSigterm` / `SendSigkill` / latch / drain close | `pidfd_send_signal` |
| Phase C | `Probed` | `SweepGroup` / none | `waitid(WNOWAIT)`, `kill_process_group` |
| Phase C | `Reaped` | — | `waitid(WEXITED\|WNOHANG)` on the pidfd |

Phases: **A** — observing (status, streams, pidfd); **B** — deadlines (`on_deadline` applies the
**earliest due one only** and returns, so an end observation can always interleave); **C** — cleanup
(`enter_cleanup_if_done` → `AwaitingProbe` or `AwaitingReap` → `Finished`).

`Observer::observe` applies a due deadline before waiting, then caps each `poll` at
`min(next_deadline, MAX_POLL_WAIT_MS = 250)`, and carries an absolute structural guard of
`total_bound_ms + 5 000`, which cannot be reached while the model still has deadlines of its own.

**`WHOLE_REAL_MODEL_AGREEMENT_SOUND`.**

---

## 16. Monotonic deadlines and bounds

`SPAWN_CONFIRM_TIMEOUT_MS = 5 000`, `POST_KILL_REAP_MS = 5 000`, `POST_EXIT_DRAIN_MS = 2 000`, plus
the plan's `timeout_ms` and `grace_ms`.

`now_ms()` is `Instant::elapsed()`, which is `CLOCK_MONOTONIC` on Linux; the `time` rustix feature is
deliberately not enabled. `spawned_at` is taken once, in `Observer::new`, and the model's origin is
`0`.

No deadline is ever reset. Each is an absolute `now + bound` stored once:

* `status_deadline` in `Lifecycle::new`;
* `run_deadline` on the clean status EOF, once, and only if the end is not already observed;
* `grace_deadline` when `SIGTERM` is sent;
* `kill_deadline` inside `send_sigkill`, which is guarded by `if !self.sigkill_sent`;
* `drain_deadline` on the first `ChildEndReadable`, guarded by `!end_observed && !end_not_observed`.

`EINTR` creates no event: `poll` returning `EINTR` simply returns from `wait_once`, and `read_once`
retries at most `MAX_READ_EINTR_RETRIES = 16` times and then ends the turn as `WouldBlock`. Traffic
and spurious wakeups likewise create no deadline movement.

Because `send_sigkill` is idempotent, `POST_KILL_REAP_MS` contributes **exactly once** on every
path. `Observer::run` then calls `disarm_drop_cleanup_after_lifecycle()` before building the
outcome, so the P3 `ChildHandle` drop guard cannot pay a second `SIGKILL` and a second
`POST_KILL_REAP_MS`. `p4_boundary.rs::the_total_bound_counts_one_post_kill_wait_and_the_drop_guard_is_handed_back`
asserts both halves.

Accepted total bound: `SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_KILL_REAP_MS +
POST_EXIT_DRAIN_MS`, exactly as `total_bound_ms` computes it with saturating arithmetic.

**`WHOLE_LIFECYCLE_BOUND_SOUND`.**

---

## 17. Stream draining and fairness

`read_once` performs **at most one** `read` of `READ_BUFFER_BYTES = 65 536` per ready descriptor per
turn and never loops to `EAGAIN`. The single buffer is allocated once and shared by both streams;
the per-stream prefix storage is allocated once at the plan's bound and cannot grow.

Consequences, each checked:

* a continuous producer cannot starve a deadline, the other stream, the pidfd or the lifecycle —
  `p4_boundary.rs::a_ready_stream_is_read_at_most_once_per_observation_turn` holds the structure,
  and `eight_mebibytes_on_both_streams_at_once_are_drained_in_full` holds the behaviour with **two
  threads in the image writing 8 MiB each simultaneously**, each blocking on its own pipe, so a
  serial drainer would deadlock rather than merely fail. The test also asserts the full 16 MiB is
  counted and hashed, that both prefixes are truncated, that no payload reaches the receipt, and
  that the whole thing finishes inside ten seconds;
* **read before hangup**: readiness only ever produces `Action::ReadStream`; `POLLHUP`/`POLLERR`/
  `POLLNVAL` is passed as informational `hangup` and never removes a descriptor.
  `readiness_with_a_hangup_reads_every_byte_before_end_of_file` exercises the combined event and a
  `cfg(test)` counter proves the combined readiness was genuinely encountered rather than assumed;
* **only a zero-byte read is EOF**;
* **`EINTR` creates no fact** — it is retried, then yields `WouldBlock`, and the descriptor stays in
  the poll set;
* **a read error is its own fact**, `Completeness::ReadFailed { errno }`, never EOF, and a later EOF
  cannot overwrite it (`t41_read_errors_are_their_own_facts_and_never_eof`);
* **no permanently-ready pidfd spin**: `Observer.end_observed` removes the pidfd from the poll set
  after its single `POLLIN`, and a `cfg(test)` loop-turn counter bounds the post-exit drain;
* **post-exit retained writers are bounded** by `POST_EXIT_DRAIN_MS`, recorded as
  `WriterRetainedAfterChildExit`.

**`WHOLE_STREAM_OBSERVATION_SOUND`.**

---

## 18. Termination

* run deadline expiry → `run_deadline_expired = true` **and** `sigterm_sent = true` on the same
  branch, then `pidfd_send_signal(SIGTERM)`;
* grace expiry → one `pidfd_send_signal(SIGKILL)`;
* pre-exec bound expiry → immediate `SIGKILL` by pidfd and `Indeterminate(PreExecStatusTimeout)`;
* a classified non-EOF status waits for the end only until the pre-exec bound, then one `SIGKILL`.

No numeric-pid direct-child signal exists. Facts stay causal-neutral: the receipt records
`sigterm_sent`, `sigkill_sent` and the observed `child_end` as three separate facts, and there is no
`KilledByLauncher` value anywhere. A child that ends by `SIGKILL` after the launcher sent `SIGKILL`
is recorded as those two facts and nothing more.

**`WHOLE_TERMINATION_SOUND`.**

---

## 19. `EndNotObserved`

Once `kill_deadline` passes without an observed end, `lifecycle.rs:425-440` sets
`end_not_observed = true` and `child_end = Some(ChildEnd::EndNotObserved)`, and fills any open
stream slot with `ReadStoppedChildEndNotObserved`.

The latch holds everywhere afterwards:

* `Event::ChildEndReadable` is guarded by `!self.end_not_observed`, so a late readiness is inert;
* `after_probe`'s `AlreadyReaped` arm uses `get_or_insert`, so a latched end stands;
* `after_reap` matches `(Some(latched), _) => latched` **first**, so nothing the final reap collects
  can revise it.

`t40_end_not_observed_is_latched_and_a_later_reap_never_revises_it` drives this across both
authority states, both kill paths and every collectable reap class.

There is **no blocking reap and no hidden second wait**: the only `waitid` calls are the `WNOWAIT`
probe and the `NOHANG` final reap, and `disarm_drop_cleanup_after_lifecycle` removes the
`ChildHandle` guard before `launch` returns.

`P4A-03` — no real integration case poses a genuine `SIGKILL` survivor — remains **MINOR / OPEN** as
previously dispositioned. I did **not** manufacture a blocker from the accepted absence of a
dangerous real test; the model-level proof is exhaustive and the real path is structurally identical.

**`WHOLE_END_NOT_OBSERVED_SOUND`** (with `P4A-03` carried).

---

## 20. Group sweep (Phase C)

`enter_cleanup_if_done` branches on authority before anything else:

* **`NotEstablished`** → `group_sweep = NotIssuedGroupNotEstablished`, `Action::Reap`, **no probe and
  no sweep**;
* **`Established`** → `Action::ProbeReaped`, then
  * `Probe::Unreaped` → `group_sweep = Issued`, `Action::SweepGroup`, **then** `Action::Reap`;
  * `Probe::AlreadyReaped` → `group_sweep = NotIssuedChildAlreadyReaped`, no sweep,
    `child_end.get_or_insert(EndUnobservable)`, then `Action::Reap`.

`group_sweep` is written once and the phase advances, so no retry and no second sweep is reachable;
`t30_with_group_authority_every_path_issues_exactly_one_sweep_before_the_reap` asserts exactly one
`SweepGroup` across every completion path and that nothing afterwards produces a second.

The probe is `waitid(P_PIDFD, EXITED|NOHANG|NOWAIT)` — non-consuming, so it cannot take the end the
reap must classify. `ECHILD`, and any refusal other than `EINTR`, is `AlreadyReaped` and suppresses
the sweep. Treating `EINTR` as `Unreaped` matches the accepted rule (only an observation that the
child was reaped withholds the sweep) and is in any case unreachable with `NOHANG`.

**Target safety.** `sweep_group` signals `Pid::from_raw(self.child.pid())` — the pid `clone3`
returned, never a group discovered by asking the system. It is reached only when the parent's own
`setpgid(child, child)` created that group **and** the probe just found the child unreaped, so the
pid and therefore the group id cannot have been recycled. The sweep is strictly before the reap,
which is the whole point: an unreaped child keeps its group id reserved.
`p4_boundary.rs::exactly_one_process_group_signal_site_exists_and_it_is_in_the_launch_slice` and
`the_sweep_is_guarded_by_a_non_consuming_probe` hold the structure; `p3_boundary.rs` proves the
backend issues no group signal at all.

**`WHOLE_GROUP_SWEEP_SOUND`.**

---

## 21. No containment claim

`GroupSweep::Issued` means the one call was made, and nothing else. Checked across code vocabulary,
README, schema document and vectors:

* `src/launch.rs::sweep_group` doc — "**best-effort cleanup, not containment** … no descendant is
  claimed to have received it, died, or been contained";
* `src/lib.rs` "Non-claims" — names `setsid`, `setpgid` and service handoff as survivors;
* README banner and "Non-claims";
* schema document §4.8 — "**`issued` is best-effort cleanup, not containment.**" and §5;
* `p4_boundary.rs` forbids the tokens `contained`, `containment`, `sandbox` and `sandboxed`
  tree-wide in `src/`.

No text claims all descendants were killed, the tree was contained, anything was sandboxed, or the
application was fully stopped.

**`WHOLE_NO_CONTAINMENT_CLAIM`.**

---

## 22. Reap and foreign reaper

* No consuming wait happens before the sweep decision — the probe is `WNOWAIT`.
* The final reap is `ChildHandle::reap_once()`, non-blocking, on the pidfd.
* `CLD_EXITED` → `Reap::Exited { code }`; `CLD_KILLED` → `Reap::Killed { signal }`; `CLD_DUMPED` →
  `Reap::Dumped { signal }` **preserving the signal number**, mapped to
  `ChildEnd::Signaled { signal, core_dumped: true }`. `reap_classification_keeps_exit_and_signal_distinct`
  covers all six classes; `a_signalled_child_keeps_its_signal_number_and_its_core_flag` covers the
  real path and — after the `P4PUB-01` correction — takes a producer self-test as its oracle rather
  than predicting the host's `core_pattern` behaviour from `RLIMIT_CORE`.
* `ECHILD` at the probe → `NotIssuedChildAlreadyReaped` plus `EndUnobservable`;
  `a_foreign_reaper_suppresses_the_sweep_and_leaves_the_end_unobservable` exercises it for real.
* `EndUnobservable` and latched `EndNotObserved` stay distinct: the latch wins in `after_reap` and in
  `after_probe`.

**`WHOLE_REAP_CLASSIFICATION_SOUND`.**

---

## 23. Receipt product model

`model.rs` and `receipt.rs` were read completely. There is **one** receipt model. The serializer is a
hand-written, fixed-order emitter with no map iteration, no clock, no host value and no
platform-dependent formatting.

Fifteen top-level fields in a fixed order; closed enum vocabularies throughout; no dynamic map
iteration; no raw host string (`working_directory_id` is a caller identifier constrained to
JSON-safe ASCII); no pid, fd or path; no timestamp; no elapsed duration; no raw stdout/stderr; no
hidden private payload. `p4_boundary.rs::the_receipt_record_carries_no_time_pid_or_descriptor_field`
pins the closed key set.

**`WHOLE_RECEIPT_MODEL_SOUND`.**

---

## 24. Receipt digest

`LaunchReceipt::from_record` is the only constructor: it serialises, then takes
`Digest::of(&bytes)` over exactly those bytes. Nothing is withheld, redacted or sanitised after
serialisation, and the receipt never contains its own digest.

I verified this end to end without using the crate: I decoded all 11 published vectors from
`exact_bytes_base16` and recomputed SHA-256 in Python. **All 11 digests matched**, and each also
matched the `exact_byte_length`.

**`WHOLE_RECEIPT_DIGEST_SOUND`.**

---

## 25. Receipt privacy

Dataflow audited from every sensitive source:

| Source | Reaches `LaunchOutcome` | Reaches `ReceiptRecord` / `LaunchReceipt` | Reaches `Debug` / `Display` / errors |
|---|---|---|---|
| stdout/stderr bytes | prefix, **memory only** | count + SHA-256 only | length only |
| argv | no | `argument_count` only | no |
| paths | no | no | no |
| descriptor ids | no | no | no |
| pid / pidfd | no | no | no |
| OS error text | no | `errno` numbers and fixed names only | fixed codes |

`Capture::absorb` counts and hashes **before** applying the prefix bound, so the digest covers every
drained byte while the retained prefix stays bounded and its storage is allocated once.
`LaunchOutcome`'s `Debug` prints `receipt_sha256` and prefix lengths; `LaunchReceipt`'s prints the
digest, the byte length and the record. The 8 MiB test additionally asserts no payload marker
appears in the receipt bytes at volume.

**`WHOLE_RECEIPT_PRIVACY_SOUND`.**

---

## 26. Receipt authenticity nonclaim

The receipt carries zero authority. Bytes may be copied and may be fabricated — the schema document
says so explicitly and explains that documenting the serializer makes fabrication easy **by design**.
There is no signature, MAC, certificate, attestation, provenance field or trusted `Deserialize`
constructor; `serde` derives do not exist in the crate at all, and two `compile_fail` doctests prove
`LaunchReceipt` and `ReceiptRecord` cannot be deserialised while a `serde_json::Value` can.

The digest means byte identity only. The schema document repeats this in §1 and §5 without turning
it into a trust mechanism, and the vector artifact's file-level `"authenticity": "none"` is checked
by two tests so it cannot quietly disappear.

I searched the whole crate and both P5 artifacts for signing/provenance vocabulary: every hit is
either a nonclaim or Rust's pointer-provenance API (`expose_provenance`) inside the backend, which
is a memory-model term and unrelated.

**`WHOLE_RECEIPT_AUTHENTICITY_NONCLAIM_SOUND`.**

---

## 27. P5 receipt schema fidelity

I compared `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` line by line against `serialize()` in
`src/receipt.rs`, and then against the real emitted bytes of all 11 vectors.

| Property | Document | Serializer / bytes |
|---|---|---|
| `schema` | `helm-launch-receipt` | identical literal |
| `version` | `0.1` | identical literal |
| field order | the 15 of §3 | identical, and my decoder confirmed the key order on every vector |
| whitespace | "no whitespace anywhere" | none emitted; my regex scan found no whitespace byte |
| duplicate keys | none | none; every vector's top level has 15 unique keys |
| nesting order | `asserted_context`, `executable`, `exec_status`, `child_end`, `termination`, `stdout`, `stderr` | identical |
| nullability | the two `asserted_context` digests only | both `null` and both set are encoded |
| numeric spelling | decimal integers, no exponent/fraction | `4096`, `18446744073709551615`, `-2147483648` |
| `pre_exec_mode_bits` | decimal, `0o755` → `493` | `493` in every base vector |
| hex encoding | 64 lowercase hex | confirmed lowercase on every digest |
| enum spellings | all closed vocabularies | every emitted token matched |
| backend spelling | `linux_x86_64_clone3_pidfd_execveat` | identical |
| stream completeness | 4 values, `errno` only on `read_failed` | identical |
| exec-status shapes | 2 kinds, `errno` only on `status_read_failed` | identical |
| child-end shapes | 4 kinds with their extra fields | identical |
| termination | 2 bools + `group_sweep` | identical |
| group-sweep dispositions | 3 values | identical |
| asserted-context meaning | caller assertion, never verified | matches `ValidatedLaunchPlan` accessors |
| pre-exec measurement meaning | pre-exec, not what ran | matches |
| size bound | 8192, a ceiling not a truncation point | matches `MAX_RECEIPT_BYTES`, and a test pins the published value to the constant |

The document invents **no field** and makes **no stronger claim** than the code. Its "Covered shapes"
table names 11 shapes and all 11 are encoded.

**`P5_RECEIPT_SCHEMA_EXACT`.**

---

## 28. Serialization byte contract

The schema is an exact-byte evidence contract, so I checked whether the document defines every
byte-affecting behaviour the vectors depend on.

| Byte-affecting behaviour | Defined where | Verified |
|---|---|---|
| whitespace | §2 "one JSON object, no whitespace anywhere" | no whitespace byte in any vector |
| object order | §2 "fixed" + §3 table | key order matched on all 11 |
| newline presence | covered by "no whitespace anywhere" | no trailing newline on any vector |
| lowercase hex | §2 "Digest encoding" | all digests lowercase |
| integer spelling | §2 "Numeric encoding" | decimal, signed where `i32` |
| `null` spelling | §4.2 "digest or `null`" | literal `null` |
| UTF-8 assumption | §2 "UTF-8, in practice pure ASCII" | every vector is pure ASCII |
| string escaping | §4.3 states the identifier grammar "admits only JSON-safe ASCII, so no escaping ever occurs in this field" | **no backslash appears in any vector**; every other string is a closed enum token or lowercase hex |

The escaping case is genuinely unreachable rather than merely untested: the identifier grammar is
`[a-z0-9][a-z0-9._-]{0,79}`, and the remaining strings are compile-time literals. The document
documents why, which is the right treatment and the one the instruction asks for.

**`P5_RECEIPT_BYTE_CONTRACT_SOUND`.**

---

## 29. Published test vectors

Eleven vectors, each carrying `name`, `exact_byte_length`, `sha256`, `exact_bytes_base16` and
`exact_bytes_utf8`, under a file-level envelope of `schema`, `version`, `receipt_schema`,
`receipt_version`, `max_receipt_bytes` and `authenticity`.

| # | Name | Bytes |
|---|---|---|
| 1 | `normal_direct_child_exit` | 1059 |
| 2 | `pre_exec_failure_exec_eacces` | 1052 |
| 3 | `status_eof_without_record_indeterminate` | 1059 |
| 4 | `run_deadline_sigterm_then_sigkill` | 1080 |
| 5 | `group_sweep_issued` | 1032 |
| 6 | `group_sweep_not_issued_group_not_established` | 1058 |
| 7 | `foreign_reaped_end_unobservable` | 1059 |
| 8 | `end_not_observed_latched` | 1051 |
| 9 | `stream_completeness_variants` | 1083 |
| 10 | `signaled_child_keeps_core_flag` | 1083 |
| 11 | `widest_near_max_receipt` | 1372 |

**Vector metadata is clearly outside the receipt bytes.** The file-level `schema` is
`helm-launch-receipt-test-vectors` while the receipt's own is `helm-launch-receipt`; the file-level
`"authenticity": "none"` appears **only** in the envelope and in no receipt's exact bytes, and the
schema document's §3 field table says "Nothing else appears", so it cannot be mistaken for a
`LaunchReceipt` field. All 11 digests are distinct, and a test asserts that.

No vector claims success (§14 above).

**`P5_VECTOR_SET_SOUND`.**

---

## 30. Production-serializer proof

`receipt::tests::published_receipt_vectors_match_the_production_serializer` rebuilds each record
in-crate and serialises it with `LaunchReceipt::from_record` — the **production** constructor and the
**production** `serialize()`. There is no shadow serializer and no second hand-written emitter
anywhere in the crate; I checked by reading `receipt.rs` in full and by confirming the product region
is byte-identical to the accepted P4 one.

For every vector the test requires:

1. committed `exact_bytes_base16` == production bytes;
2. `exact_bytes_utf8` as bytes == production bytes;
3. `exact_byte_length` == the real length;
4. SHA-256 **recomputed from the committed bytes** == the committed digest **and** == the product
   digest;
5. length ≤ `MAX_RECEIPT_BYTES`;
6. no verdict term in the bytes.

It also asserts the published name list and **order** match the in-crate set exactly, so no vector
can be withheld, and that all digests are distinct.

`tests/p5_regressions.rs::level4_every_published_receipt_digest_is_recomputable_from_the_artifact`
is the deliberately independent half: it never touches the serializer and recomputes every digest
from the committed file alone, which is exactly what an outside reader can do. My own Python
recomputation is a third, fully independent confirmation.

**`P5_VECTORS_USE_PRODUCTION_SERIALIZER`.**

---

## 31. Base16 / UTF-8 duplicate representation

Both in-crate tests assert `exact_bytes_utf8.as_bytes() == exact_bytes_base16` decoded, and I
confirmed it independently for all 11 vectors in Python.

* No Unicode normalisation is possible: every vector is **pure ASCII** (verified byte by byte).
* No newline ambiguity: no vector contains any whitespace byte.
* No lossy conversion: the base16 round-trip is exact and the UTF-8 copy contains no escape
  sequence at all.
* `exact_bytes_base16` is documented as "**the authoritative encoding**" in schema §6, and the
  in-crate test compares the production bytes against it first.

**`P5_VECTOR_BYTE_REPRESENTATION_SOUND`.**

---

## 32. Vector coverage

| Required shape | Encoded by |
|---|---|
| normal exit fact shape | `normal_direct_child_exit` |
| pre-exec failure | `pre_exec_failure_exec_eacces` (`stage: exec`, `errno: 13`) |
| status EOF indeterminate | `status_eof_without_record_indeterminate` (non-zero exit) |
| deadline / termination | `run_deadline_sigterm_then_sigkill` |
| sweep issued | `group_sweep_issued` |
| sweep not issued — authority absent | `group_sweep_not_issued_group_not_established` |
| foreign-reaped / `EndUnobservable` | `foreign_reaped_end_unobservable` |
| `EndNotObserved` serializable shape | `end_not_observed_latched` |
| stream completeness variants | `stream_completeness_variants` (`writer_retained_after_child_exit` + `read_failed`), `end_not_observed_latched` (`read_stopped_child_end_not_observed`), the rest (`complete_at_eof`) |
| signal / core-dumped | `signaled_child_keeps_core_flag` (`signal: 11`, `core_dumped: true`) |
| widest valid receipt | `widest_near_max_receipt` — see §33 |

Every shape the document claims is genuinely encoded; I confirmed each by decoding the bytes rather
than by reading the table. All four `Completeness` values, all four `ChildEnd` kinds, all three
`GroupSweep` values and both `ElfType` values appear.

**Recorded scope, not a defect.** The published set encodes 2 of the 4 `indeterminate` reasons
(`status_eof_without_record`, `status_read_failed`) and 1 of the 9 `stage` spellings (`exec`). The
document claims 11 named shapes, not exhaustive vocabulary coverage, and exhaustiveness is proved
in-crate by `every_variant_combination_is_bounded_injective_and_recomputable`, which drives
13 × 5 × 3 × 4 × 4 = 3 120 combinations through the production serializer and asserts byte
injectivity across all of them.

**`P5_VECTOR_COVERAGE_SOUND`**, with the scope recorded and `P5R-04`, `P5R-05` below.

---

## 33. Widest-receipt proof

The author reports the widest vector at 1372 bytes. I did **not** treat 8192 as a target, and I
established the true maximum independently.

Running the exhaustive in-crate combination test with output captured reports:

```
HELM-LAUNCH-WIDEST-RECEIPT: bytes=1401 ceiling=8192
```

So the widest **representable** receipt under the current fixed schema is **1401 bytes**, and the
published `widest_near_max_receipt` is **1372**. I reconstructed the 29-byte difference exactly, from
the emitted token lengths:

| Field | Published choice | Widest choice | Δ |
|---|---|---|---|
| `stderr.completeness` | `complete_at_eof` | `read_stopped_child_end_not_observed` | 20 |
| `stdout.completeness` | `read_failed` + `errno` | `read_stopped_child_end_not_observed` | 4 |
| `child_end.core_dumped` | `true` | `false` | 1 |
| `run_deadline_expired` | `true` | `false` | 1 |
| `termination.sigterm_sent` / `sigkill_sent` | `true` / `true` | `false` / `false` | 2 |
| `termination.group_sweep` | `not_issued_child_already_reaped` | `not_issued_group_not_established` | 1 |
| | | **total** | **29** |

1372 + 29 = 1401, which matches the empirical maximum exactly.

Assessment:

* **The bound obligation is discharged.** `MAX_RECEIPT_BYTES = 8192` is proved to be a ceiling by a
  production-serializer test that actually reaches 1401, not by the published vector.
* **No document claims maximality.** The schema's table says the vector carries "widest numeric and
  identifier values", which is literally true — it uses `u64::MAX`, `i32::MIN`, `u32::MAX` and an
  80-byte identifier. The shortfall is entirely in enum and boolean *spellings*, and the vector's
  own name says "near max".
* **No silent truncation** exists anywhere: `serialize` never cuts, and the bound is asserted rather
  than enforced.

Wider valid inputs therefore exist **and are tested**, just not published as a vector. I record this
as `P5R-04` (MINOR) rather than a coverage failure.

**`P5_MAX_RECEIPT_PROOF_SOUND`**, with `P5R-04` recorded.

---

## 34. Cross-platform vector design

The vector tests are **not** `cfg`-gated to Linux. `receipt.rs`'s `mod tests` carries only
`#[cfg(test)]`, and `tests/p5_regressions.rs` has no target gate at all. They construct inert
`ReceiptRecord` values and launch nothing, open no descriptor and touch no host state.

I executed them on this **Windows** host and they pass:

```
test receipt::tests::published_receipt_vectors_match_the_production_serializer ... ok
test receipt::tests::the_published_vectors_respect_the_accepted_receipt_bound ... ok
test level4_every_published_receipt_digest_is_recomputable_from_the_artifact ... ok
(all 7 p5_regressions tests ok)
```

The `include_str!` path uses `concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/…")`, which resolves
correctly with Windows separators — demonstrated, not assumed. No expected byte string is
platform-specific. The workflow's new *Name and run the P5 Level-4 regressions and published receipt
vectors* step carries **no `if: runner.os`**, so it runs on all three matrix legs.

**`P5_VECTOR_CROSS_PLATFORM_GATE_SOUND`.**

---

## 35. The `include_str!` boundary refinement

This was the hotspot I spent the most time on.

**Before P5**, `p2_boundary.rs::the_crate_declares_exactly_the_p2_modules` forbade `#[path`,
`include!(`, `include_bytes!(`, `include_str!(` and `cfg_if!` across the **whole** text of the eight
portable `src/` files.

**After P5**, `include_str!` alone is split:

```rust
for forbidden in ["#[path", "include!(", "include_bytes!(", "cfg_if!"] { … }   // whole source
let product: String = strip(product_code(source)).split_whitespace().collect();
assert!(!product.contains("include_str!("), …);                                 // product region
for (at, _) in test_code(source).match_indices("include_str!(") {
    assert!(argument.contains("helm-launch-receipt-0.1-test-vectors.json"), …);  // test region
}
```

Checked:

* **Product code remains prohibited.** The product-region check runs on `strip(product_code(source))`;
  `strip` removes comments and string literals, so `include_str!("x")` still leaves the matched
  `include_str!(` token and cannot be hidden in a literal or a comment.
* **The exception is strictly inside a test-only region.** `product_code`/`test_code` split at the
  first `#[cfg(test)]`, and a companion test
  (`the_test_only_region_of_every_source_is_last`) verifies every source keeps its test-only items
  last, so the split is meaningful rather than assumed.
* **Every allowed use is pinned to the vector file.** The argument window is read from the **raw**
  test region, so the literal is intact; the scan requires the filename and rejects anything else.
* **No other boundary was weakened.** `include!`, `include_bytes!`, `#[path` and `cfg_if!` remain
  forbidden across the whole source, unchanged.
* **File scope was not shrunk.** `SOURCES` was 8 entries at `0622303` and is 8 entries at `67a4502`.

There are exactly two `include_str!` sites that load the vector file:

1. `src/receipt.rs:949`, inside `#[cfg(test)] mod tests` — never compiled in a non-test build;
2. `tests/p5_regressions.rs:331`, at file scope in an **integration test crate**, which by
   construction is compiled only by `cargo test`.

**No macro or `cfg` arrangement can pull the vector include into product runtime.** Neither site is
reachable from a library build; there is no build script; `p3_boundary.rs` walks the tree at run time
and fails on any file its inventory does not name, so no new file can appear outside these scans.

The narrowing does not materially weaken the product boundary.

**`P5_INCLUDE_STR_BOUNDARY_REFINEMENT_SOUND`**, with `P5R-06` (a pre-existing scope gap, not a P5
narrowing) recorded below.

---

## 36. Level 4, row by row

### Row A — write-only helper regression

`level4_e5b_no_test_helper_reads_through_a_write_only_descriptor`. `write_only_bindings` finds
`let` bindings created by `File::create(` or `OpenOptions::new()…write(true)…open(` without
`.read(true)`; `reads_through` then checks whether that name is used as a read source anywhere in
the file. It scans **every** `.rs` file under `src/` and `tests/` — not one historical helper name.

Both controls are real and both are asserted: the synthetic violation must be reported as exactly
`["sink"]`, and the clean shape (re-open by path for reading) must not be reported. Current mutation
helpers do verify through separately opened read-capable handles — `set_and_assert_mode` reads back
via `fs::metadata`, and the admission fixtures via a fresh `O_RDONLY` open.

**PROBATIVE.** (The scanner is textual and covers the `std::fs` shapes; a `rustix::fs::open` with
`OFlags::WRONLY` would not be matched. No such helper exists today.)

### Row B — not-posed separation

`level4_a_setup_failure_is_typed_and_never_a_mechanism_result`. `NotPosed` is a test-only,
crate-private type in `src/backend/tests.rs` (itself behind `#[cfg(test)]`). `fail(self) -> !`
diverges, so no caller can continue with a substituted value; the message states what it is and,
explicitly, that it is **not** a launcher result, **not** a receipt and **not** a verdict about the
mechanism. The test asserts the type, the divergence, all five message phrases, that `require_tool`
routes through it, and that **no other `src/` file names `NotPosed`**. No public API changed.

**PROBATIVE.**

### Row C — fixture modes

`level4_every_generated_fixture_mode_is_asserted_after_writing`. `set_and_assert_mode` applies the
mode and reads it back through `fs::metadata`, failing with both the wanted and the observed mode.
The test asserts the helper exists, that its message survives, that **exactly one** raw
`fs::set_permissions(` remains in `src/backend/tests.rs` (the one inside the helper), and that the
compiled fixture asserts its own executable bits.

I verified the claimed seven conversions individually: `0o600` (EACCES), `0o755` (ENOEXEC), `0o755`
(ETXTBSY), `0o000` and `0o755` (chdir EACCES and restore), `0o755` (exact-descriptor original), and
`0o755` (replacement). Seven, all present.

I then inventoried **every** mode-setting site in the crate to check the instruction's "did the
inventory miss another generated fixture" question. One site outside the scanner's scope does not
follow the discipline — `src/launch.rs:1465` — recorded as `P5R-03`. All other sites
(`linux_admission.rs:80` inside `fixture()`, `authority.rs:925` inside its `fixture()`) assert the
applied mode, and the remaining `chmod` calls are post-admission mutations whose *effect* is what the
test asserts.

**PROBATIVE for `src/backend/tests.rs`; see `P5R-03` for the one unscoped site.**

### Row D — descriptor collision and layout

`generated_distinct_layouts_satisfy_every_property` (20 000 generated descriptor sets across four
distributions including numbers adjacent to `HIGHEST`), `simulated_hosts_end_with_exactly_stdio_in_the_image`
(5 000 simulated descriptor tables with 0/1/2 open or closed, lowest-free allocation, caller
descriptors with arbitrary close-on-exec), and the real Linux case
`an_authorised_object_executes_with_exactly_the_intended_descriptors`.

**PROBATIVE.**

### Row E — `CLD_DUMPED` keeps its signal

`a_signalled_child_keeps_its_signal_number_and_its_core_flag` (real, with a producer self-test as
oracle and a fixed non-core `SIGTERM` branch that is a real invariant rather than a host fact) and
`reap_classification_keeps_exit_and_signal_distinct` (all six reap classes in the pure model).

**PROBATIVE.**

### Row F — clean EOF is never exec success

`a_clean_status_eof_is_indeterminate_and_never_exec_success` (real),
`s5_status_eof_then_death_is_indeterminate_and_identical_to_a_normal_run` (model), and
`no_function_offers_a_success_reading` (structural). The structural scan covers the eight portable
sources only, but the tree-wide guarantee is carried by
`p4_boundary.rs::no_exec_success_containment_or_authenticity_vocabulary_exists`, which walks all of
`src/` including `launch.rs` and `backend/`.

**PROBATIVE.**

### Row G — `clone3` process-vs-thread trace disambiguation

`the_child_window_is_closed_from_a_multithreaded_allocating_parent`. The tracer requires exactly one
`CLONE_PIDFD` clone and filters `CLONE_THREAD`, so a thread creation cannot be mistaken for the
child.

**PROBATIVE** (Linux-gated; pending hosted execution).

### Row H — canonical fixture paths

`level4_fixture_paths_handed_to_children_are_canonical_and_absolute`. `fixture_root()` now
canonicalises and asserts absoluteness, and the test additionally asserts there is **exactly one**
`std::env::temp_dir()` in the module, so every child-facing path descends from that one canonical
root. `scratch_dir` builds from `fixture_root()`, and `src/launch.rs`'s real cases use that same
helper. `linux_admission.rs` has its own fixture root but creates no child, so no post-`fchdir`
resolution applies to it.

**PROBATIVE.**

### Row I — report producer vs evidence consumer

`report_fixture_reports_without_the_backend` and
`the_producer_report_schema_is_closed_and_a_malformed_report_is_refused`. The X2c lesson — a fixture
that cannot emit the report its success rule demands — is closed by proving the fixture reports
independently of the backend and that a malformed report is refused rather than guessed.

**PROBATIVE** (Linux-gated).

### Row J — published digest recomputable

`level4_every_published_receipt_digest_is_recomputable_from_the_artifact` (artifact only, no
serializer) and `published_receipt_vectors_match_the_production_serializer` (production serializer).
The two together close `R3-M1`: a published digest that cannot be recomputed from the published
artifact proves nothing. Independently confirmed by my own Python recomputation of all 11.

**PROBATIVE.**

### Row K — a read error is never EOF

`t41_read_errors_are_their_own_facts_and_never_eof`, which also proves a later EOF cannot overwrite a
recorded read failure and that a status read failure starts no run deadline.

**PROBATIVE.**

### Row L — no blocking reap after `SIGKILL`

`t40_no_end_within_the_kill_bound_is_end_not_observed_on_both_kill_paths`,
`t40_end_not_observed_is_latched_and_a_later_reap_never_revises_it` (across both authority states,
both kill paths and every collectable reap class), and
`the_total_bound_counts_one_post_kill_wait_and_the_drop_guard_is_handed_back`.

**PROBATIVE.**

### Row M — no sweep after a foreign reap

`a_foreign_reaper_suppresses_the_sweep_and_leaves_the_end_unobservable` (real) and
`t31_a_child_already_reaped_elsewhere_gets_no_sweep` (model, across every path, with the latched
`EndNotObserved` case called out).

**PROBATIVE.**

### Row N — no sweep without established authority

`t31_without_group_authority_no_path_issues_a_sweep` (every path) and
`no_source_infers_group_authority_from_an_observed_process_group` (structural, forbidding `getpgid`,
`getpgrp`, `tcgetpgrp`).

**PROBATIVE.**

### Row O — full signal mask across `clone3`

`the_blocked_mask_covers_every_signal_including_the_two_glibc_keeps_to_itself`, backed by the raw
`FULL_KERNEL_SIGSET` in `spawn.rs` and by the traced full-set mask immediately before `clone3`.

**PROBATIVE** (Linux-gated).

---

## 37. The executable Level 4 inventory

`the_level_4_inventory_names_a_real_test_for_every_accepted_obligation` walks 26 rows. For each it
requires the named file to exist in the walked tree, the file to define `fn <name>(`, and `#[test]`
to appear within the twelve lines preceding that definition. It rejects duplicated `(row, test)`
pairs, and finally asserts the set of row-letter initials equals exactly `{A..O}`.

It cannot pass merely because arbitrary function names exist: the file must be the claimed one, the
name must be a function definition, and it must be attributed as a test. The mapping is closed in
both directions — every row is represented and every letter A–O is covered.

Its one softness is that `is_marked_as_a_test` scans a twelve-line window rather than parsing
attributes, so a `#[test]` belonging to an immediately preceding function could in principle satisfy
it. That is traceability support, and I did **not** rely on it: every row above was judged by reading
the named test's own semantics.

**`P5_LEVEL4_INVENTORY_SOUND`.**

---

## 38. README current truth

The README is at current product truth. The slice table records P1–P4 **ACCEPTED** with their
decision links and P5 **AUTHORISED / IMPLEMENTED CANDIDATE, NOT YET ACCEPTED** with the whole-crate
review named as the next gate. "Still not implemented" now says P5 is authorised and implemented as a
candidate and lists what remains unauthorised.

The banner and "Non-claims" carry, accurately: public Linux `launch` only; the backend private and
unreachable; no process handle in the public API; no sandbox; no containment; no cgroup; no Wine or
Proton; no `PATH`, shell or pathname launch; no exec-success claim; no receipt authenticity; the
output prefix in memory only; no authority from parsing. Zero HELM crate dependencies is stated in
the dependency section and is true (§41).

The P5 additions — the Level 1–4 "What proves what" table and the row-by-row Level 4 table — are
accurate against the tree, and the executable inventory makes the Level 4 half self-checking rather
than circular.

No stale P2/P3/P4 current-status prose survives in the README. The title and banner enumerate
product surfaces ("P1 to P4"), which stays accurate because P5 adds none; the slice table
immediately below states P5's status explicitly, so a reader is not misled.

**`WHOLE_README_SOUND`.**

---

## 39. `P4DOC-01` closure and the manifest line

`P4DOC-01`'s recorded locations were **README, `linux_admission.rs`, `backend/mod.rs`,
`backend/spawn.rs`**. All four are corrected in P5, and in every case the change is prose only —
the mechanical comparison in §3 proves the product regions of `backend/mod.rs` and `backend/spawn.rs`
are byte-identical.

I then re-scanned the whole crate for the wider stale-truth vocabulary rather than trusting the
seven-phrase list. Remaining hits and my judgement:

| Location | Text | Judgement |
|---|---|---|
| `src/backend/injection.rs:3` | "This file does not exist in a default or release build" | **true** — it is doubly `cfg`-gated |
| `src/backend/mod.rs:117, :294` | "the observation loop / public boundary that would need it is P4's" | **true** — correct present-tense attribution |
| `src/launch.rs:1392` | "a future widening of `total_bound_ms`" | **true** — a hypothetical future change |
| `src/model.rs:134, :136` | "Descriptor 1 of a future executed image" | **true** — the image that will be executed |
| `README.md:428` | "Still **not authorised**…" | **true** — about backlog items |
| `Cargo.toml:69` | "the scoped Linux backend that **a later, separately authorised slice may add**" | **stale** — that slice is P3, accepted and in-tree |
| `tests/p2_boundary.rs:796` | "Surfaces of later slices **that are not authorised**…" above a list containing `launch` and `LaunchOutcome` | **stale** — P4 is accepted |

The manifest line is genuinely (B), stale current product text, not (A): the sentence's *reason* for
not inheriting the workspace lint table remains true, but "may add" says the backend does not exist
yet, and the manifest's own `description` twenty lines above already says the crate contains it. I
did not force this from the word "later" — the stale part is "may add". Both items are comment-level
accuracy debt with no code, doctest, oracle or contract effect, and neither is inside the P5 scan's
declared scope (`Cargo.toml` is not scanned at all).

**`P4DOC01_CLOSURE_SOUND`** for the four recorded locations; the two residual items are recorded as
`P5R-02` (MINOR).

---

## 40. The current-truth vocabulary scan

`current_crate_facing_text_does_not_deny_the_accepted_p4_surface` checks seven narrow phrases.

* **Positive control**: the exact comment P5 removed from `backend/mod.rs` must be caught, and the
  assertion pins the result to `["future P4"]`.
* **Negative control**: an ordinary historical sentence naming P4 must not be flagged.
* **Not a global ban**: `future`, `P4` and `P5` are ordinary words; only seven compound phrases are
  forbidden, so historical review documents remain writable.
* **Scope**: every `.rs` file under `src/` and `tests/`, plus `README.md`. `docs/` is deliberately
  excluded, because superseded statements are supposed to survive verbatim there — which is correct
  and is stated in the test.
* **Exclusions**: exactly one, the scanner's own file, which necessarily contains every phrase it
  searches for. An assertion requires the scanner to still see its own file, so the exclusion cannot
  become stale, and the test states that any other file needing excusing is a real finding.

It does not exclude files merely because they are inconvenient. Its one blind spot is that
`Cargo.toml` is not in scope — the source of `P5R-02`.

**`P5_CURRENT_TRUTH_SCAN_SOUND`**, with `P5R-02` recorded.

---

## 41. `P4PUB05-R1` — release-assembly artifact selection

The `find … -print -quit` first-match read is **gone**. The step now collects all matches into an
array, sorts them, and:

* `0` candidates → `BUILD EVIDENCE FAILURE`, exit 1;
* `≠ 1` candidates → `BUILD EVIDENCE FAILURE` listing every candidate, exit 1 — it explicitly
  refuses to read an arbitrary first match;
* the single candidate must fall under `"$out"/x86_64-unknown-linux-gnu/release/deps`, otherwise
  `BUILD EVIDENCE FAILURE`, exit 1.

`$out` is `rm -rf`'d first, so the premise (a fresh target directory, one selected target, no
dependency named `helm_launch`) is asserted rather than assumed. No `-print -quit` remains anywhere
in the workflow. This also supersedes the carried `P3R-14`.

**`P4PUB05_R1_CLOSED`.**

---

## 42. `P4PUB05-R2` — the release reachability tool

`tools/helm_launch_release_reachability.py` was read completely, together with its 10 controls.

It **imports** `Assembly` from the accepted P3 `tools/helm_launch_child_closure.py` and uses only
`.bodies`, `.edges()` and `.defined()`. It mutates nothing, monkeypatches nothing and redefines no
P3 semantics — I confirmed by reading both files.

It proves: public `launch` → direct control-transfer graph → `child_main`, by breadth-first search
over `Edges.targets`, which the P3 parser populates **only** from directly named or GOT-resolved
symbols. Indirect (`*`), unresolved and unsupported-branch operands go to separate lists and are
never traversable, so an unresolvable edge yields "no path" and the tool fails.

| Requirement | Mechanism | Control |
|---|---|---|
| missing root fails | `resolve` raises on zero matches | `test_a_needle_matching_nothing_fails` |
| missing `child_main` fails | same, for the target role | same |
| emitted but unreachable fails | BFS finds no path | `test_an_emitted_but_unreachable_target_fails` |
| ambiguous symbol fails | `resolve` raises on >1 and lists them | `test_an_ambiguous_needle_fails_rather_than_choosing` |
| indirect unresolved fails closed | not in `targets` | `test_an_indirect_only_edge_is_never_followed` |
| cycles terminate | `seen` set | `test_a_cycle_does_not_hang_the_search` |
| positive controls | direct chain, tail jump, longer chain | three tests |

`resolve()` only accepts symbols already in `asm.bodies`, so the target can never be an undefined
external. The `root == target` short-circuit is unreachable here because the two needles differ.

It is wired into CI twice: the checker itself runs against the real release assembly inside the
Linux release-library step, and its unit tests run in the repository-level confinement step on all
three platforms.

**`P4PUB05_R2_CLOSED`.**

---

## 43. Reachability versus optimisation

The tool is explicitly designed for an optimised build: it follows whatever out-of-line functions
survive and documents the expected shape —`spawn_for_lifecycle` and `spawn` normally inline into
`launch`, `clone_and_dispatch` survives because it is `#[inline(never)]`, so the observed chain is
`launch -> clone_and_dispatch -> child_main`. `child_main` itself carries `#[inline(never)]`, which
`p3_boundary.rs` asserts.

Mangled suffixes are handled by substring matching with an ambiguity failure; GOT and PLT spellings
are normalised by the P3 parser (`GOT_TARGET` and `target.split("@")[0]`); tail jumps are first-class
edges.

The proof is valid for the pinned compiler (1.95.0, installed explicitly in the workflow) and does
not require a stable ABI across future Rust versions. It **fails closed** if the expected roots or
edges disappear after a toolchain update: a root or target that resolves to zero symbols fails, one
that resolves to several fails, and a graph in which no direct chain survives fails. A future
inlining change cannot silently turn the gate green.

**`P5_RELEASE_REACHABILITY_PROOF_SOUND`.**

---

## 44. CI path filters — the one IMPORTANT finding

**Sound half.** `tools/helm_launch_release_reachability.py` and
`tools/tests/test_helm_launch_release_reachability.py` were both added to the `pull_request` and
`push` path filters of `.github/workflows/helm-launch.yml`, so a future change to the new tool or its
tests triggers the workflow that validates it. (`tools/tests/**` is also covered by
`helm-evidence.yml`.)

**Unsound half — `P5R-01`, IMPORTANT.** Neither

* `docs/implementation/helm-launch-receipt-0.1-test-vectors.json`, nor
* `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md`

appears in **any** workflow's path filter. I enumerated all seven workflows:

| Workflow | Covers the vectors? |
|---|---|
| `helm-launch.yml` | no — `Cargo.toml`, `Cargo.lock`, `crates/helm-launch/**`, five `tools/` paths, itself |
| `helm-evidence.yml` | no — five `crates/**`, five `tools/` paths, `tools/tests/**`, itself |
| `helm-bind.yml` | no |
| `helm-observe-independent-review.yml` | no — `push` restricted to `review/helm-observe-**` branches |
| `launch-exec-01-compile-only.yml` | no — `docs/experiments/launch-exec-01/**` only |
| `launch-exec-01-trial-002.yml`, `-003.yml` | `workflow_dispatch` only |

No workflow runs on all pushes, and `python3 tools/validate_docs.py` appears only inside
`helm-evidence.yml` and the helm-observe review workflow, neither of which is triggered by a
`docs/` change.

The committed exact-byte vectors are **executable evidence**, not prose: they are `include_str!`'d by
`crates/helm-launch/src/receipt.rs:949` and `crates/helm-launch/tests/p5_regressions.rs:331`, and two
tests assert byte-for-byte agreement with the production serializer and digest recomputability. A
commit on this milestone branch that changes only that file therefore **triggers no workflow at
all**, and the executable contract can drift unvalidated until some unrelated commit happens to
touch a filtered path.

This is precisely the instruction's IMPORTANT example: *"load-bearing vector/schema changes can
bypass CI"*. It is a CI-reachability defect only — the product, the serializer, the vectors and the
tests are all correct today, and the P5 publication commit itself does touch
`crates/helm-launch/**` and so will trigger the workflow. The remedy is two path entries in each of
the `pull_request` and `push` filters; I have deliberately **not** applied it.

**`P5_EVIDENCE_PATH_FILTERS_SOUND`: NO — see `P5R-01`.**

---

## 45. `--no-fail-fast`

Exactly two steps changed, both multi-target suite invocations:

```
cargo test -p helm-launch --locked --no-fail-fast
cargo test -p helm-launch --locked --features test-fault-injection --no-fail-fast
```

* Only intended multi-target suite invocations use it. No named gate step does — I diffed the
  complete step list against `0622303` and those two lines are the only `cargo` changes.
* Cargo still exits non-zero if **any** target failed, so the workflow step fails and, with default
  step semantics, the job stops before later steps.
* No test is rerun and no retry is introduced: `--no-fail-fast` only stops Cargo abandoning the
  remaining targets **of the same run**.
* No gate is converted to a warning.

The motivation is recorded honestly in the workflow: the P4 publication chain twice paid for a stale
oracle in a later target staying invisible behind an earlier failure.

The new P5 step re-executes `p5_regressions` and the two receipt-vector tests under explicit names.
That is the same log-visibility convention the existing "Name and run the P4 …" steps use — it makes
a silently empty suite impossible — and is not a retry.

**`P5_NO_FAIL_FAST_HARDENING_SOUND`.**

---

## 46. Prior gate preservation

I diffed the complete list of workflow step names and `cargo` invocations between `0622303` and
`67a4502`. **No step was removed or renamed.** The only changes are the two `--no-fail-fast` flags,
the rewritten artifact-selection block inside an existing step, the added reachability invocation
inside that same step, the added unit-test line in the confinement step, and one appended step.

Retained and verified present: capability admission; P3 machine-code closure in the debug/test
profile; P3 machine-code closure in the release codegen profile; the conditional-branch proof and
the rest of the machine-proof suite; the fault-injection positive control with the release-absence
proof; `strace` child-window and P3 backend cases; the P3 fault-injection cases (S5/S6); the named P4
lifecycle and receipt cases; the coordinated sweep cases; the foreign-reaper cases; receipt privacy
(inside the named P4 cases and the boundary suites); the total-bound case; the unsafe-confinement and
boundary suites; the repository-level confinement checks; the determinism anchors; and the
off-cohort absence checks.

The new P5 step was deliberately appended **last** so that the step numbers the P4 publication
evidence refers to — 17 for the release-library gate, 24 for the repository-level confinement checks
— keep their meaning. That is careful evidence hygiene.

**`P5_PRIOR_GATES_PRESERVED`.**

---

## 47. CI levels 1 to 4

One `purity` job over `[ubuntu-24.04, windows-2025, macos-15]`.

* **Ubuntu** reaches Levels 1–4: the full suites, both machine-code profiles, injection, `strace`,
  S5/S6, admission, the P4 lifecycle and sweep gates, the release-library and reachability gates, the
  repository-level confinement checks, the determinism anchors and the new P5 step.
* **Windows and macOS** reach the Level 1 portable surface: `fmt`, `clippy --all-targets
  --all-features`, the default and fault-injection suites, the off-cohort absence checks, the
  boundary suites, the determinism anchors and the P5 step. The Linux-only steps are gated with
  `if: runner.os == 'Linux'` / `!= 'Linux'`.
* **Receipt vectors run on all three**, both inside the default suite and under the explicitly named
  P5 step, which carries no OS condition.
* No privileged runner; no `sudo`; no root requirement.
* Nothing in the workflow is a trial, dispatches LAUNCH-EXEC-01, or carries D-7 semantics — the
  header states this and I confirmed no LAUNCH-EXEC-01 asset is built or run.

**`P5_CI_COVERAGE_SOUND`.**

---

## 48. Dependency graph

`Cargo.lock` records `helm-launch`'s dependencies as exactly:

```
libc, rustix, serde, serde_json, sha2
```

**Zero HELM crate dependencies.** No `helm-evidence`, `helm-app-spec`, `helm-observe` or `helm-bind`
appears in the manifest or anywhere in `src/`; the only occurrence of any of those names in the crate
is a manifest comment noting that the `rustix` pin matches `helm-observe`'s.

`rustix` is `=1.1.4`, `default-features = false`, features `std, fs, process, pipe, event` — cohort
gated; `libc` is `=0.2.189`, constants only, with no `extern` block and no `libc` call anywhere
(`p3_boundary.rs` fails on either). `serde =1.0.228`, `serde_json =1.0.149`, `sha2 =0.10.9`, all
exact pins already vetted for this workspace. `Cargo.toml` and `Cargo.lock` are **untouched by P5**,
so no dependency or feature was added.

`p4_boundary.rs::the_manifest_adds_only_the_event_feature_and_no_dependency` holds this.

**`WHOLE_DEPENDENCY_GRAPH_SOUND`.**

---

## 49. `helm-evidence` boundary

P5 does not implement backlog `B-02`. The diff touches no file under `crates/helm-evidence/`, and
nothing anywhere in the workspace was taught to parse a receipt, trust one, validate one
semantically, infer execution from one, or derive authority from one. The schema document states this
in §1 explicitly.

The published vectors are inert future input: a portable fixture, not a manifest, not signed, not
trusted, and consumed only by `helm-launch`'s own tests.

**`P5_HELM_EVIDENCE_BOUNDARY_PRESERVED`.**

---

## 50. No authenticity or provenance work

Searched the full P5 delta and the current crate for `sign`, `signature`, `MAC`, `certificate`,
`attest`, `provenance`, `hmac`, `ed25519`, `keypair` and trusted-verifier vocabulary.

Every hit is one of: an explicit nonclaim in prose (`src/lib.rs`, `src/launch.rs`, README, schema
document), a forbidden-token list in a test, the word `signal`, a type-signature mention, or Rust's
pointer-provenance API (`expose_provenance`) inside the backend, which is a memory-model operation
and unrelated to evidence provenance. `p4_boundary.rs` forbids `signature`, `signed`,
`verify_receipt` and `authenticate` as tokens tree-wide in `src/`.

No signing key, signature, MAC, certificate, attestation, trusted receipt verifier or provenance
constructor exists. `B-10` remains backlog.

**`P5_NO_PROVENANCE_WORK_SOUND`.**

---

## 51. Final evidence inventory

The README's "What proves what" tables map each accepted obligation to a source, a test, a CI gate
and, where relevant, a nonclaim. The Level 4 half is **executable**, so it cannot decay silently, and
I judged each row's semantics independently in §36 rather than accepting the table.

The inventory is not circular: it names test functions, and the executable inventory requires those
functions to exist, to be in the claimed file and to be attributed as tests. No row rests on "the
README says a test exists". Levels 1–3 are mapped to entry points that I confirmed exist and, for
Level 1 and the portable Level 4 rows, executed on this host.

**`P5_FINAL_EVIDENCE_INVENTORY_SOUND`.**

---

## 52. Non-verdict boundary

Searched the full crate, the README, the schema document and the vector artifact for prohibited
result language used as an application verdict.

`helm-launch` answers none of *works*, *compatible*, *success*, *passed*, *safe* or *verified*.
The crate's own forbidden list (18 terms) is enforced by `receipt.rs`'s vocabulary tests against
whole emitted terms and against each `_`/`-`/`.`-separated word, with the deliberate and correct
exception that `failure`/`failed` name an observed failed operation (`pre_exec_failure`,
`read_failed`) and are distinct words, not verdicts.

Exit code 0 is a fact: `ChildEnd::Exited { code }` is raw status, and both `src/lib.rs` and schema
§4.7 say `code` 0 means only that. The receipt is facts only; no function maps an outcome to a
`bool`, a `Result<(), _>` or an ordering.

My independent token extraction over all 11 vectors (§14) found **zero** verdict-vocabulary hits
among 66 distinct emitted tokens.

**`WHOLE_NONVERDICT_BOUNDARY_SOUND`.**

---

## 53. Local validation

Run on this Windows host at `67a4502`, worktree clean.

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo test --workspace --locked` | **PASS** |
| `cargo test -p helm-launch --locked` | **PASS** — 147 tests across 9 targets (44 lib, 20, 17, 15, 7, 19, 2, 23 doctests) |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** — same counts |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — `Ran 841 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | **PASS** — 146 markdown, 256 JSON, 1682 link targets |
| `git diff --check` | **PASS** — clean |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | **PASS** |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --locked -- -D warnings` (default features) | **PASS**, no dead-code warning |
| `cargo check -p helm-launch --target x86_64-unknown-linux-gnu --locked` | **PASS** |

P5-specific targets, each run and named:

| Target | Result |
|---|---|
| `tests/p5_regressions.rs` (7 tests: rows A, B, C, H, J, the current-truth scan, the executable inventory) | **PASS on Windows** |
| `receipt::tests::published_receipt_vectors_match_the_production_serializer` | **PASS on Windows** |
| `receipt::tests::the_published_vectors_respect_the_accepted_receipt_bound` | **PASS on Windows** |
| `receipt::tests::every_variant_combination_is_bounded_injective_and_recomputable` | **PASS**, reports widest 1401 / ceiling 8192 |
| `tools.tests.test_helm_launch_release_reachability` | **PASS** (inside the 841) |
| `tools.tests.test_helm_launch_machine_proofs` | **PASS** (inside the 841) |
| `tools.tests.test_helm_launch_confinement` | **PASS** (inside the 841) |
| `tests/p2_boundary.rs`, `tests/p3_boundary.rs`, `tests/p4_boundary.rs` (schema/static scans) | **PASS on Windows** |

**Independent scratch probes** (not committed, run from the session scratchpad):

1. a standalone Python decoder and SHA-256 recomputation over the committed vector file — all 11
   vectors: `base16 == utf8`, digest recomputed, top-level key order exact, no whitespace, pure
   ASCII, no escape sequence, all digests distinct;
2. a token extractor over the vector bytes — 66 distinct quoted tokens, zero verdict-vocabulary hits;
3. a mechanical product-region differ over every changed `src/` file between `0622303` and
   `67a4502`, establishing `P5_PRODUCT_SEMANTIC_DELTA_ZERO`.

No shadow product implementation was created. No WSL Rust toolchain was installed.

**Linux runtime cannot be executed from this host.** The Linux x86_64 cases — the backend suite, the
traced child window, S5/S6, the real lifecycle, sweep and foreign-reaper cases, both machine-code
profiles against a real emitted assembly, the release-library gate and the new reachability check
against a real release assembly — did not run here. That is expected and is not a review failure.

**`P5 LINUX LEVEL-4 VALIDATION: PENDING REVIEWED PUBLICATION CI`.**

---

## 54. Findings

### `P5R-01` — IMPORTANT — the published receipt vectors are outside every CI path filter

**Area:** CI evidence reachability. **Reachable:** yes.

`docs/implementation/helm-launch-receipt-0.1-test-vectors.json` is executable evidence, consumed by
`include_str!` in `crates/helm-launch/src/receipt.rs:949` and
`crates/helm-launch/tests/p5_regressions.rs:331`, and held to exact-byte agreement with the
production serializer by two tests. It appears in **no** workflow's `pull_request` or `push` path
filter, and no workflow in this repository runs unconditionally on push. A commit on the milestone
branch that changes only that file therefore triggers no validating job, so the exact-byte and digest
contract can drift unnoticed. `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` is equally uncovered,
and because `tools/validate_docs.py` runs only inside workflows whose own filters exclude `docs/`,
not even documentation validation runs on such a commit.

**Disposition:** not fixed here — this is a review-only task. The remedy is to add both paths to the
`pull_request` and `push` filters of `.github/workflows/helm-launch.yml` (the vectors at minimum;
the schema document alongside them, since the two must stay in step).

### `P5R-02` — MINOR — residual stale current-truth text outside the P5 scan scope

**Area:** documentation. **Reachable:** no — comments only; no code, doctest, oracle or contract
effect.

* `crates/helm-launch/Cargo.toml:69` — "the scoped Linux backend that **a later, separately
  authorised slice may add**". That slice is P3, accepted and in-tree; the manifest's own
  `description` on line 7 already says the crate contains the backend. The *reason* for not
  inheriting the workspace lint table is still true; the tense is not.
* `crates/helm-launch/tests/p2_boundary.rs:796-799` — "Surfaces of later slices **that are not
  authorised**…" above a list containing `launch` and `LaunchOutcome`, both now accepted. The list's
  purpose (those tokens must not appear in the eight portable sources) is unchanged and correct.

Neither location was among `P4DOC-01`'s four recorded ones, and `Cargo.toml` is outside the P5
current-truth scan's scope entirely.

**Disposition:** carry, or fold into a later documentation pass. Widening the scan to `Cargo.toml`
would catch the first.

### `P5R-03` — MINOR — one generated-fixture mode site outside the row-C discipline

**Area:** test hygiene, Level 4 row C. **Reachable:** no — a failure surfaces loudly.

`crates/helm-launch/src/launch.rs:1465`
(`an_object_without_execute_permission_reports_the_exec_stage_failure`) calls
`fs::set_permissions(&copy, fs::Permissions::from_mode(0o600)).expect("drop execute bits")` with **no
read-back assertion**, while the equivalent backend case
(`an_object_without_execute_permission_reports_exec_eacces`) was converted in P5 to
`set_and_assert_mode` and additionally re-reads the mode. The row-C scanner asserts "every fixture
mode must go through `set_and_assert_mode`" but scopes that to `src/backend/tests.rs` only.

Mitigating, and the reason this is MINOR rather than IMPORTANT: the specific `X2b`/`X2c`/`X4` defect
mechanism is `0666 & ~umask` at **file creation**, and `fs::set_permissions` is `chmod`, which is not
umask-filtered; the `.expect` already fails loudly if the call errors; and a mode that failed to
apply would break a named assertion rather than pass silently. It is a scanner-scope gap, not an
uncovered obligation.

**Disposition:** carry; route the site through `set_and_assert_mode` and widen the scanner to
`src/launch.rs` in a later test-only pass if the owner wishes.

### `P5R-04` — MINOR — the published "widest" vector is 29 bytes below the representable maximum

**Area:** published vectors. **Reachable:** no.

`widest_near_max_receipt` is 1372 bytes; the widest receipt the current fixed schema can express is
**1401 bytes**. The 29-byte gap is itemised in §33 and is entirely in enum and boolean *spellings*,
principally `stderr.completeness` (`complete_at_eof` where `read_stopped_child_end_not_observed` is
20 bytes longer).

No document claims maximality — the schema's table says "widest numeric and identifier values",
which is literally true, and the vector's name says "near max". The bound obligation is discharged by
`every_variant_combination_is_bounded_injective_and_recomputable`, which drives 3 120 combinations
through the production serializer and does reach 1401. There is no silent truncation anywhere.

**Disposition:** record. If the owner wants the published set to carry the true maximum, the change
is to set both streams' completeness to `read_stopped_child_end_not_observed`, `core_dumped` and
`run_deadline_expired` and both termination booleans to `false`, and `group_sweep` to
`not_issued_group_not_established`.

### `P5R-05` — MINOR — three vectors encode a termination combination the lifecycle cannot produce

**Area:** published vectors. **Reachable:** no.

In the accepted model, `sigterm_sent = true` is set on exactly one line
(`src/lifecycle.rs:419`), immediately after `run_deadline_expired = true` (line 418), and there is no
other writer of either field. So the product guarantees `sigterm_sent ⟹ run_deadline_expired`.

Three published vectors — `group_sweep_issued`, `group_sweep_not_issued_group_not_established` and
`end_not_observed_latched` — carry `"sigterm_sent":true` with `"run_deadline_expired":false`.

This is sound as a **serializer** fixture: the vectors' stated purpose (schema §6) is "a portable
fixture so that an independent implementation can check its reading of this document", and the schema
document states no cross-field invariant, so nothing documented is contradicted. It could nonetheless
lead an independent implementer to treat an unreachable combination as an observed shape.

**Disposition:** record. Either adjust those three records, or state in schema §6 that the vectors
exercise the serializer's field space and are not claimed to be reachable traces.

### `P5R-06` — MINOR — the include-macro boundary scan does not cover `src/launch.rs` or `src/backend/**`

**Area:** static boundary scope. **Reachable:** no. **Pre-existing, not introduced or widened by P5.**

`p2_boundary.rs::the_crate_declares_exactly_the_p2_modules` — the only test that forbids `#[path`,
`include!(`, `include_bytes!(`, `cfg_if!` and (in product code) `include_str!(` — iterates the
`SOURCES` array of eight portable files. `src/launch.rs` and the five `src/backend/*.rs` files are
not in it. An `include!` in `src/launch.rs` would not be caught by any test today.

`SOURCES` held the same eight entries at `0622303`, so P5 refined the rule **within** its historical
scope and did not shrink it. Residual risk is low: `p3_boundary.rs` walks the tree at run time,
fails on any unlisted file, and applies a large forbidden-token list to the backend; the child window
additionally carries `#![no_implicit_prelude]`; and there is no build script.

**Disposition:** carry as backlog. Extending the scan to the full `src/` tree would close it.

### Carried historical findings, re-checked against P5

| Id | Class | State after this review |
|---|---|---|
| `P1-TEST-01` | MINOR | **OPEN.** P5 does not touch the plan parser; behaviour correct, the committed escaped-duplicate vector is still a byte-identical literal |
| `P3R-03` / `F-P4-M2` | MINOR | **OPEN.** `PidfdNotProvided` remains the one post-`clone3` `Err`; unreachable under the `CLONE_PIDFD` contract; no numeric-pid fallback authorised |
| `P3R-04`, `P3R-05`, `P3R-06`, `P3R-07`, `P3R-08`, `P3R-12`, `P3R-16`, `P3R-17`, `P3R-18`, `P3R-19` | MINOR | **OPEN**, unchanged by P5; none has become load-bearing for the published evidence contract |
| `P3R-09` | MINOR | **Effectively closed by P5.** The README's new Level 1–4 inventory now names the machine-code gate and `tools/helm_launch_child_closure.py`; the remaining "greps a marker" sentence accurately describes the injection release-absence proof |
| `P3R-13` | BACKLOG | **OPEN**, unchanged |
| `P3R-14` | BACKLOG | **Superseded by `P4PUB05-R1`.** The first-match read is gone; the gate now fails closed on 0 or >1 candidates and on an unexpected directory |
| `P3R21-M1` | MINOR | **OPEN**, unchanged |
| `P3R21-M2` | MINOR | **No longer manifests.** I ran a default-feature Linux cross-check and clippy with `-D warnings`: both clean, no never-read warning. The P4 consumer reads `SpawnedForLifecycle.group_authority_established`, and P5's comment correction documents why the P3-only `MinimalLaunch` field is dead on that path |
| `P4DOC-01` | MINOR / BACKLOG | **CLOSED** for its four recorded locations; see `P5R-02` for two residual items elsewhere |
| `P4PUB05-R1` | MINOR / BACKLOG | **CLOSED** (§41) |
| `P4PUB05-R2` | MINOR / BACKLOG | **CLOSED** (§42, §43) |
| `P4A-03` | MINOR / OPEN | **CARRIED** as previously dispositioned; no blocker manufactured from the accepted absence of a dangerous real `SIGKILL`-survivor case |

No carried finding has become load-bearing because of P5's schema, vector or CI changes.

---

## 55. Verdict

| Gate | Result |
|---|---|
| BLOCKER | **0** |
| IMPORTANT | **1** — `P5R-01` |
| MINOR | 6 new (`P5R-01` excluded), plus the carried set above |
| P1 contract | **SOUND** |
| P2 contract | **SOUND** |
| P3 contract | **SOUND** |
| P4 contract | **SOUND** |
| P5 evidence contract | **SOUND in substance; its CI reachability is not** |
| public API | **SOUND** |
| authority | **SOUND** |
| `unsafe` | **CONFINED**, zero added by P5 |
| child closed world | **SOUND** |
| lifecycle | **SOUND** |
| stream handling | **SOUND** |
| group sweep | **SOUND** |
| receipt model, digest, privacy, authenticity nonclaim | **SOUND** |
| schema | **EXACT** |
| vectors | **EXACT** |
| Level 1 / 2 / 3 / 4 | **COVERED** |
| `P4DOC-01` | **CLOSED** for its recorded locations; `P5R-02` residual |
| `P4PUB05-R1` | **CLOSED** |
| `P4PUB05-R2` | **CLOSED** |
| new P5 tools | **PROBATIVE** |
| CI path filters | **NOT SOUND** — `P5R-01` |
| prior gates | **PRESERVED** |
| zero new `unsafe` | **YES** |
| zero HELM dependencies | **YES** |
| Linux runtime | **GATE_PENDING** |

PASS requires 0 BLOCKER **and** 0 IMPORTANT. One IMPORTANT finding stands.

**`HELM_LAUNCH_P5_WHOLE_CRATE_REVIEW_NEEDS_FIX`.**

`P5R-01` is a two-line change to one workflow's path filters. It touches no product code, no test, no
schema, no vector and no gate semantics. It is nevertheless load-bearing for the published evidence
contract, which is the whole point of the P5 slice: a published exact-byte artifact whose validating
test cannot be triggered by changing it is not an enforced contract.

**P5 IS NOT ACCEPTED BY THIS DOCUMENT.**
**THE COMPLETE HELM-LAUNCH 0.1 MODULE IS NOT PRODUCT-ACCEPTED.**
**TRIAL #3 REMAINS FROZEN `MECHANISM_REJECTED`. NO TRIAL #4 IS AUTHORISED.**

**Next gate: OWNER DECISION ON `P5R-01` — authorise the bounded CI path-filter correction, or
dispose of the finding — before the single publication for final hosted P1–P5 validation.**
