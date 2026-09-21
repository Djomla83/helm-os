# BOUNDED INDEPENDENT REREVIEW OF P4PUB-04

> # 🔎 BOUNDED INDEPENDENT REREVIEW OF P4PUB-04
>
> **Session provenance. THIS SESSION AUTHORED OR MODIFIED NONE OF:**
> `e52c371fb88c176996da783cf675febe16be15e2` (the second-publication failure disposition) and
> `fd1829785110983a77ab5395270f5e8bc20727ad` (the `authority.rs` doctest correction). It also
> authored no part of the published P4 head `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1`, of the
> earlier publication chain, or of the P1–P4 implementation chain. It read the disposition and the
> correction under review and re-established every claim below from the preserved source, from the
> crate as compiled against the Linux target, and from the workflow definitions.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **P1 ACCEPTED. P2 ACCEPTED. P3 ACCEPTED. P4 AUTHORISED — SECOND PUBLICATION FAILED, `P4PUB-04`
> CORRECTION CANDIDATE, NOT ACCEPTED. P5 NOT AUTHORISED. THE COMPLETE helm-launch 0.1 MODULE IS NOT
> PRODUCT-ACCEPTED. TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED` AND MUST NOT BE RERUN. TRIAL #4 IS
> NOT AUTHORISED.**

This is a **bounded** rereview of `P4PUB-04`: the root cause of the second P4 publication failure,
the `authority.rs` doctest and doc-comment correction, the preservation of the valid authority
negative proofs, the proof that the product runtime and public API did not change, and the
preservation of historical CI. It does not repeat the P4 lifecycle review and reopens no
already-reviewed P4 product semantics.

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **0** |
| `MINOR` | **1** (`P4DOC-01`, carried, nonblocking) |
| `GATE_PENDING` | **1** (hosted Linux doctest runtime) |
| `P4PUB-04` | **FIXED** |
| Stale `launch` `compile_fail` | **REMOVED** |
| Stale `LaunchOutcome` `compile_fail` | **REMOVED** |
| Positive `launch` API proof | **PROBATIVE** |
| Positive `LaunchOutcome` proof | **PROBATIVE** |
| Process executed by a doctest | **NONE** |
| Single-use authority proofs | **PRESERVED** |
| Product runtime semantics | **UNCHANGED** |
| Public API | **UNCHANGED** |
| Workflow | **UNCHANGED** |
| Classification | `HELM_LAUNCH_P4_P4PUB04_REREVIEW_PASSED_READY_FOR_NEW_CI` |

## 1. Starting state, independently verified

| Check | Observed |
|---|---|
| `git status --porcelain` | empty — clean worktree |
| `git branch --show-current` | `docs/helm-launch-architecture` |
| `git rev-parse HEAD` | `fd1829785110983a77ab5395270f5e8bc20727ad` |
| `origin/docs/helm-launch-architecture` | `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Ahead / behind origin branch | **2 ahead, 0 behind** |

`origin` was fetched **read-only**. No ref was moved, no history was repaired and nothing was
pushed.

Ancestry is linear, with one parent at every step and no merge:

```
84b49ab9d7bf4f7c9d10c7e55f327778f2855da1   (published, failed)
  -> e52c371fb88c176996da783cf675febe16be15e2   (disposition)
       -> fd1829785110983a77ab5395270f5e8bc20727ad   (correction, local HEAD)
```

## 2. Preserved second publication

The published head `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1` and both of its natural runs are
recorded and unmodified:

| Run | Recorded state |
|---|---|
| `helm-launch` `35507479482` | run #7, **attempt 1**, event `push` — **FAILURE** |
| `HELM Rust workspace Linux` `35507479458` | run #39, **attempt 1**, event `push` — **FAILURE** |

The first publication remains recorded alongside it:

| Run | Recorded state |
|---|---|
| `helm-launch` `35499943908` | **attempt 1**, event `push` — **FAILURE** |
| `HELM Rust workspace Linux` `35499943903` | **attempt 1**, event `push` — **FAILURE** |

**NO RETRY. NO RERUN. NO REPLACEMENT.** No historical result is rewritten into success anywhere in
the correction range; the corrected head will carry a new SHA and new run identifiers, which are new
evidence rather than a revision of either phase.

## 3. Correction scope

`git diff 84b49ab..fd18297 --name-only` reports exactly three files, split across the two commits
as authorised:

| Commit | Files |
|---|---|
| `e52c371` (disposition) | `docs/DECISIONS.md`, `docs/PROJECT_STATE.md` — **only** |
| `fd18297` (correction) | `crates/helm-launch/src/authority.rs` — **only** |

Within `authority.rs` the change is confined to doc comments, ordinary comments, the doctest code
embedded in the docs, and two `cfg_attr` lint-reason strings. No other source file, no test file and
no workflow is touched. §15 proves mechanically that no executable Rust behaviour changed.

## 4. `P4PUB-04` — root cause, re-established independently

Re-established from the preserved published head, not from the disposition text.

At `84b49ab` the `AuthorizedLaunch` documentation in `crates/helm-launch/src/authority.rs` carried
two `compile_fail` doctests, under a prose claim that **"Nothing in this crate can consume it to
create a process, because no `launch` function exists"**:

```rust
/// ```compile_fail
/// fn execute(a: helm_launch::AuthorizedLaunch) {
///     let _ = helm_launch::launch(a);
/// }
/// ```
///
/// ```compile_fail
/// fn outcome() -> helm_launch::LaunchOutcome {
///     todo!()
/// }
/// ```
```

At the same head, `crates/helm-launch/src/lib.rs:295` publicly exported both items on the cohort:

```rust
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use launch::{LaunchOutcome, launch};
```

`LaunchError` is exported unconditionally, and `authority.rs` is compiled only under
`cfg(all(target_os = "linux", target_arch = "x86_64"))`. Therefore on the P4 Linux cohort both
snippets **correctly compiled**, and their `compile_fail` expectations **correctly reported
FAILED** — the preserved log line being `Test compiled successfully, but it is marked
compile_fail`, with `test result: FAILED. 40 passed; 2 failed`. The count of exactly two failures
matches exactly the two stale blocks, and no third doctest failed.

The defect is **stale test/evidence documentation, not a product failure**. The product export is
the accepted P4 surface; the negative proofs were P3-era statements that the P4 implementation
commit did not retire. It stayed invisible because the module is cohort-only — off cohort and on the
Windows development host the snippets correctly fail to compile and the `compile_fail` blocks pass —
and because at the first publication the library target failed first, so `cargo` never reached the
doctest target.

**`P4PUB04_ROOT_CAUSE_CONFIRMED`.**

## 5. Positive `launch` API doctest

The stale launch-negative block is replaced by:

```rust
/// ```
/// fn execute(
///     a: helm_launch::AuthorizedLaunch,
/// ) -> Result<helm_launch::LaunchOutcome, helm_launch::LaunchError> {
///     helm_launch::launch(a)
/// }
///
/// let _: fn(
///     helm_launch::AuthorizedLaunch,
/// ) -> Result<helm_launch::LaunchOutcome, helm_launch::LaunchError> = execute;
/// ```
```

| Requirement | Result |
|---|---|
| Type-checks the accepted cohort shape `AuthorizedLaunch -> launch(..) -> Result<LaunchOutcome, LaunchError>` | **yes** — and it matches `pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError>` at `src/launch.rs:246` exactly |
| `AuthorizedLaunch` passed **by value** | **yes** — `a: helm_launch::AuthorizedLaunch`, which is what makes one authorisation unrepeatable |
| Authority construction required | **no** — no capability is admitted, nothing is authorised, no descriptor is opened |
| Process actually executed | **no** — see §7 |
| Runtime side effect | **none** |
| False exec-success claim | **none** |

Independently type-checked against the real Linux-target crate from a scratch crate **outside the
repository** (`cargo check --target x86_64-unknown-linux-gnu`, clean). The function-pointer
coercion is what makes the block probative rather than decorative: it pins the full signature,
including the by-value argument and the `Result<LaunchOutcome, LaunchError>` return, so a later
drift in any of the three types breaks the doctest.

**`P4PUB04_POSITIVE_LAUNCH_DOCTEST_SOUND`.**

## 6. `LaunchOutcome` type-presence proof

The stale `LaunchOutcome`-negative block is replaced by:

```rust
/// ```
/// fn outcome(o: helm_launch::LaunchOutcome) -> helm_launch::LaunchOutcome {
///     o
/// }
///
/// let _: fn(helm_launch::LaunchOutcome) -> helm_launch::LaunchOutcome = outcome;
/// ```
```

| Requirement | Result |
|---|---|
| Proves the type is nameable on cohort | **yes** — named in both argument and return position, and the coercion forces the name to resolve |
| Manufactures a `LaunchOutcome` value | **no** — the body is the identity `o`, so no value is constructed and no private field is reached |
| Executes `launch` | **no** — `launch` is not mentioned |
| Implies `LaunchOutcome` exists off cohort | **no** — the whole module is `cfg`-gated to the cohort, so this is a cohort proof and can make no off-cohort statement; see §11 |

Independently type-checked against the Linux-target crate from outside the repository. The proof
complements, and does not replace, the untouched `compile_fail` in `lib.rs` proving that
`LaunchOutcome` has no public constructor.

**`P4PUB04_LAUNCHOUTCOME_DOCTEST_SOUND`.**

## 7. No process execution from a doctest

The two positive blocks are the only code blocks the correction changed; everything else in the
diff is prose. In both blocks the only item that names `launch` is a **function definition** that is
never invoked:

* rustdoc wraps a snippet with no `fn main` as `fn main() { <snippet> }`. `execute` and `outcome`
  therefore become nested items inside `main`.
* the only statement referring to either is a **function-pointer coercion**
  (`let _: fn(..) -> .. = execute;`), which names the function without calling it.
* no `main` body invokes `launch`; no block constructs an authority; no fixture is built; no
  descriptor is opened; no process is created.

A function definition, a type annotation and a function-pointer coercion are exactly the permitted
shapes. Running the corrected doctests creates no child process and performs no I/O.

**`P4PUB04_DOCTESTS_COMPILE_ONLY`.**

## 8. Single-use negative proofs preserved

Code-block accounting for `authority.rs` across the correction:

| Measure | `84b49ab` | `fd18297` |
|---|---|---|
| Total doc code blocks | 26 | **26** |
| `compile_fail` blocks | 22 | **20** |
| Plain (compiling) blocks | 4 | **6** |

Exactly two blocks flipped from `compile_fail` to positive **in place**. No negative proof was
deleted, and the block count is unchanged — so nothing was removed to make the doctest suite green.

Semantic coverage around `AuthorizedLaunch`, re-verified by compiling each snippet against the
**Linux target** from a scratch crate outside the repository. Every one still fails to compile, and
for the on-point reason rather than an unrelated missing import:

| Proof | Result | Compiler reason |
|---|---|---|
| not `Sync` | **fails** | `E0277` — `Cell<()>` cannot be shared between threads safely |
| no public field constructor | **fails** | cannot construct `AuthorizedLaunch` with struct literal syntax due to private fields |
| no `Default` | **fails** | `E0599` — no associated item named `default` |
| no `Clone` — authority not replayable | **fails** | `E0308` — `&AuthorizedLaunch` from the blanket `Clone`, mismatched with the `AuthorizedLaunch` return |
| no `From<LaunchReceipt>` | **fails** | `E0308` — only the reflexive `From<T> for T` applies |
| no `Deserialize` | **fails** | `E0277` — `AuthorizedLaunch: serde::Deserialize<'de>` is not satisfied |

The `serde` proofs are on-point rather than accidental: `serde_json = "=1.0.149"` is an ordinary
`[dependencies]` entry of the crate, so the path resolves and the failure is the missing trait.

The API-level single-use proofs live in `lib.rs`, which the correction does not touch, and were
verified the same way:

| Proof | Result | Compiler reason |
|---|---|---|
| `launch(a)` twice — **not replayable** | **fails** | `E0382` — use of moved value: `a` |
| `LaunchOutcome` has no public constructor | **fails** | private fields |
| `into_parts` crate-private | **fails** | `E0624` — method is private |
| `descriptor()` crate-private | **fails** | `E0624` — method is private |

`E0382` is the decisive proof that `launch` consumes the authority by value and that one
authorisation cannot be replayed. It is unchanged by this correction and still live.

**`P4PUB04_SINGLE_USE_PROOFS_PRESERVED`.**

## 9. `AuthorizedLaunch` current truth

The corrected prose states, and the source supports:

* `AuthorizedLaunch` is a **single-use** authority value for the launch API.
* On this cohort the accepted P4 `launch` consumes it **by value and exactly once** — matching
  `pub fn launch(authorized: AuthorizedLaunch)` and proved by `E0382`.
* It is **not replayable** — no `Clone`, no `Default`, no public constructor, no `From`, no
  `Deserialize`.
* Capabilities still do not execute themselves: "The value executes nothing by itself and exposes no
  descriptor", and no method of any type in the module creates a process.
* Raw descriptor and process authority remain private: `descriptor()` and `into_parts()` are
  `pub(crate)` and proved unreachable from outside, and the module text places every raw operation
  in `src/backend/`, which `lib.rs` keeps a private module with no `pub use backend::` line.
* Off cohort the whole authority and execution surface is absent through the `cfg` boundary, which
  the module header states without trying to prove it here.

No wording claims execution succeeded, exec was confirmed, the image definitely ran, containment,
sandboxing or receipt authenticity. The only occurrences of "sandbox" in the file are the
pre-existing **non-claims** — "the sandbox state are not checked and not attested" and "Admission
grants no privilege and is not a sandbox".

**`P4PUB04_AUTHORITY_PROSE_SOUND`.**

## 10. Other `authority.rs` stale current-status text

The authorised same-file cleanup removed the stale claims that had become false, and the
replacements are narrow and accurate:

| Location | Was | Now |
|---|---|---|
| module diagram | `AuthorizedLaunch --╳--▶ process` | `AuthorizedLaunch --launch (P4, launch.rs)--▶ attempted execution` |
| module prose | "**The last edge does not exist.** There is no `launch` … inert in-process value that nothing can execute" | "**The last edge is not in this module.** Nothing here creates a process … the accepted P4 slice provides the public `launch`" |
| `ExecutableCapability` | "no function in this crate, creates a process" | "no method of this type creates a process, and admission never becomes execution" |
| `WorkingDirectoryCapability` | "belongs to an execution slice that is not authorised" | "belongs to the P4 launch path, not to admission" |
| `admit_working_directory` | "belongs to an execution slice which does not exist" | "belongs to the P4 launch path, not to admission" |
| `executable_measurement` | "no future execution would either" | "the P4 launch path does not either — it carries this measurement through untouched" |

The descriptor accessor reason was specifically checked, because it is the claim most easily
overstated:

```
- reason = "P2 only pins the descriptor; the execution slice that would use it is not authorised"
+ reason = "admission pins the descriptor; launch moves it out via into_parts, so only tests borrow"
```

Verified against the product path rather than accepted from the text. `launch` calls
`backend::spawn_for_lifecycle(authorized)` (`src/launch.rs:249`), which calls
`authorized.into_parts()` (`src/backend/spawn.rs:161`); `into_parts` destructures the capability and
**moves the `OwnedFd` out** rather than borrowing it. Every remaining `.descriptor()` call site in
the crate is at `authority.rs:1161`, `1271`, `1274` and `1283`, all after the `#[cfg(test)]`
boundary at line 846 — so "only tests borrow" is exactly right, and the accessor is correctly still
`dead_code` outside tests for the stated reason.

**`P4PUB04_AUTHORITY_CURRENT_STATUS_SYNCED`.**

## 11. Measurement wording

The corrected sentence claims the P4 launch path carries the admission measurement through rather
than recomputing it. Independently verified in the product path, not accepted because it is
non-executable:

* `AuthorizedLaunch::into_parts` takes `let measurement = executable.measurement;` — the admitted
  value is moved out verbatim.
* `backend::SpawnedForLifecycle` carries it, documented at `src/backend/mod.rs:475` as "The P2
  measurement, carried through unchanged", and at `src/backend/spawn.rs:89` as "The pre-execution
  measurement `admit_executable` recorded, carried …".
* `Observer::new` destructures `measurement` from the spawned value and stores it on the observer.
* `Observer::record` writes `executable: self.measurement` straight into the `ReceiptRecord`.
* No re-measurement exists on the path: `measure_body` and `admit_executable` are never called from
  `launch.rs` or the backend product code, and the only `sha256` calls there are over the plan and
  the drained streams, never over the executable.

The statement is supported.

**`P4PUB04_MEASUREMENT_WORDING_SOUND`.**

## 12. No exec-success claim

A case-insensitive search of the corrected `authority.rs` for `ExecSucceeded`, "exec confirmed",
"execution succeeded", "successfully ran", "succeeded", "confirmed", "containment", "sandbox" and
"authentic" returns no positive product verdict. The only hits are the pre-existing non-claims
quoted in §9.

The new wording is deliberately attempt-shaped: "Consuming the authority is an **attempt**, never a
statement that the image ran: a clean status EOF stays `Indeterminate(StatusEofWithoutRecord)`", and
"no exec-success state exists". Clean exec-status EOF therefore remains an indeterminate fact, and
this file's wording does not resolve it.

**`P4PUB04_NO_EXEC_SUCCESS_CLAIM`.**

## 13. Product token / semantic identity

Mechanical proof, not inspection alone. Stripping every comment line (`//`, `///`, `//!`) from
`authority.rs` at both revisions and diffing the remainder:

| Measure | `84b49ab` | `fd18297` |
|---|---|---|
| Non-comment lines | 848 | **848** |
| Differing non-comment lines | — | **2** |

The two differing lines are **only** the `reason = "…"` strings inside
`#[cfg_attr(not(test), allow(dead_code, reason = …))]` on
`ExecutableCapability::descriptor` and `WorkingDirectoryCapability::descriptor`. A `reason` string
is diagnostic metadata: it does not alter the lint level, the lint name, the `cfg` predicate or any
behaviour.

Consequently **nothing** changed in: struct definitions, function signatures, function bodies,
visibility, `cfg` expressions, trait implementations, derives, constants, runtime strings used as
product output, or executable expressions. `authority.rs` contains **zero** occurrences of `unsafe`
at both revisions, and the whole correction range contains no added or removed `unsafe` line.
Lifecycle, receipt, backend, plan, model and error sources are untouched, since `authority.rs` is
the only source file in the range.

**`P4PUB04_PRODUCT_RUNTIME_UNCHANGED`.**

## 14. Public API

`crates/helm-launch/src/lib.rs` is not in the correction range, so every export is unchanged by
construction, and §13 shows nothing in `authority.rs` changed visibility.

| Surface | Linux x86_64 | Off cohort |
|---|---|---|
| `launch` | **PRESENT** (`pub use launch::{LaunchOutcome, launch};` under the cohort `cfg`) | **ABSENT** |
| `LaunchOutcome` | **PRESENT** | **ABSENT** |
| `AuthorizedLaunch`, `ExecutableCapability`, `WorkingDirectoryCapability`, `admit_executable`, `admit_working_directory`, `authorize`, `MAX_EXECUTABLE_BYTES` | **PRESENT** | **ABSENT** |

No public item was added and none was removed.

**`P4PUB04_PUBLIC_API_UNCHANGED`.**

## 15. Off-cohort boundary

The correction encodes **no** off-cohort absence claim in `authority.rs`, which would be unsound
there because the module exists only on the cohort. Off-cohort absence stays with its dedicated
owners, all untouched by this range:

* the `#![cfg_attr(not(all(target_os = "linux", target_arch = "x86_64")), …)]` block in `lib.rs`,
  which carries `compile_fail` proofs for `use helm_launch::launch;` and
  `use helm_launch::LaunchOutcome;` as well as for the admission, capability and authorisation
  items;
* `crates/helm-launch/tests/p4_boundary.rs`, including
  `the_public_launch_is_gated_on_the_linux_x86_64_cohort`, which asserts the cohort `cfg` attribute
  on both `mod launch;` and `pub use launch::{LaunchOutcome, launch};`;
* `tests/p2_boundary.rs` and `tests/p3_boundary.rs`.

`git diff --name-only 84b49ab..fd18297 -- crates/helm-launch/tests/` is empty. Windows and macOS
behaviour is therefore unchanged: `launch` absent, `LaunchOutcome` absent, Linux authority and
execution surface absent.

**`P4PUB04_OFF_COHORT_PROOF_SEPARATION_SOUND`.**

## 16. `P4DOC-01` — bounded crate scan

Stale `P2`/`P3`-era current-status wording outside `authority.rs`, confirmed present and **not
edited** in this rereview, per owner disposition:

| Location | Stale text |
|---|---|
| `crates/helm-launch/README.md:138` | "that kernel decision belongs to a **future execution slice**" |
| `crates/helm-launch/tests/linux_admission.rs:321-323` | "A **future launch** would duplicate the descriptor … and that future launch **does not exist here**" |
| `crates/helm-launch/tests/linux_admission.rs:582-583` | "belongs to an **execution slice which does not exist here**" |
| `crates/helm-launch/src/backend/mod.rs:362` | "for a **future P4 sweep**; nothing in P3 consumes it" |
| `crates/helm-launch/src/backend/mod.rs:375` | "carried through unchanged for a **future P4 receipt**" |
| `crates/helm-launch/src/backend/spawn.rs:527` | "group identity a **future P4 slice would need**" |

None is load-bearing:

* **Not a doctest.** `README.md` is not included as documentation anywhere — there is no
  `include_str!` or `doc =` reference to it in `src/` or `Cargo.toml`, so its eight fences are never
  compiled. The other five are `//` or `///` comments with no code block.
* **Not an executable assertion.** The `linux_admission.rs` occurrences are comments inside test
  bodies; the assertions around them are unaffected.
* **Not a CI oracle.** No test reads `crates/helm-launch/README.md` and no test asserts on these
  strings. The `README.md` references in `tools/tests/` belong to the LAUNCH-EXEC-01 trial-3 asset
  set and to `validate_docs` fixtures, not to this crate's README.
* **Not a public API contract.** `mod backend;` is private and `lib.rs` carries no
  `pub use backend::…` line, so the `backend` comments describe crate-private items.

`P4DOC-01`: **CARRIED MINOR / NONBLOCKING.** Documentation and comment accuracy debt only. It does
not block the `P4PUB-04` correction or its publication. Disposition target: P4 acceptance
documentation sync, or P5 documentation hardening.

## 17. Workflow

`git diff --name-only 84b49ab..fd18297 -- .github/` is **empty**: no workflow file changed.
`--no-fail-fast` appears nowhere in `.github/workflows/`, and is not requested here. The workflow
behaved correctly — `cargo test` stopping at the first failing target is what exposed the stale
doctest, and it is also why every later step of the failed runs is **INCOMPLETE rather than passed**.
CI hardening remains P5 or backlog territory.

**`P4PUB04_WORKFLOW_UNCHANGED`.**

## 18. Historical evidence

| Run | State |
|---|---|
| `35507479482` | **attempt 1, FAILURE — preserved** |
| `35507479458` | **attempt 1, FAILURE — preserved** |
| `35499943908` | **attempt 1, FAILURE — preserved** |
| `35499943903` | **attempt 1, FAILURE — preserved** |

**NO RETRY. NO RERUN. NO REPLACEMENT.** Nothing was pushed, no ref was moved and no published
history was amended. A future corrected SHA produces new runs, which will be new evidence.

## 19. Local validation

Run at `fd1829785110983a77ab5395270f5e8bc20727ad` on Windows 11, with a clean worktree before and
after.

| Check | Result |
|---|---|
| `cargo fmt --check` | **pass** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **pass** |
| `cargo test --workspace --locked` | **pass** — every target `ok`, 0 failed |
| `cargo test -p helm-launch --locked` | **pass** — 42 unit, 0 failed |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **pass** — 0 failed |
| `cargo build -p helm-launch --release --locked` | **pass** |
| `python -m unittest discover -s tools/tests` | **pass** — `Ran 831 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | **pass** — 143 markdown files, 255 JSON files, 1643 link targets |
| `git diff --check` | **clean** |
| `cargo clippy --workspace --all-targets --all-features --locked --target x86_64-unknown-linux-gnu -- -D warnings` | **pass** |

Independent cross-target type-check, from a scratch crate **outside the repository** depending on
`helm-launch` by path and built with `cargo check --target x86_64-unknown-linux-gnu`:

* both new positive doctest bodies, copied verbatim, **compile**;
* all six `AuthorizedLaunch` negative snippets and all four `lib.rs` API-level negative snippets
  **fail to compile**, each with the on-point diagnostic recorded in §8.

**This is a type-check, not a hosted rustdoc doctest run.** It does not close the hosted Linux
gate. `authority.rs` is cohort-only, so the Windows host collects **23** doctests where the Linux
runner reported **42**; the corrected blocks are among those this host cannot execute.

**Required pending gate — `P4PUB-04 LINUX DOCTEST RUNTIME: PENDING NEW PUBLICATION CI`.**

## 20. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P4PUB-04` | — | `authority.rs` doctests | n/a | Two P3-era `compile_fail` proofs contradicted the accepted P4 cohort export and correctly failed on Linux | **FIXED** — replaced in place by probative positive proofs; root cause independently confirmed |
| `P4DOC-01` | `MINOR` / `BACKLOG_NONBLOCKING` | README, `linux_admission.rs`, `backend/mod.rs`, `backend/spawn.rs` | No — prose and comments only; no doctest, assertion, oracle or public contract | Stale P2/P3 "future launch / does not exist / not authorised" wording outside `authority.rs` | **CARRIED.** Not edited here. Target: P4 acceptance doc sync or P5 doc hardening |
| `P4PUB-04-G1` | `GATE_PENDING` | hosted Linux rustdoc | n/a | Corrected doctests are cohort-only and cannot execute on this host | **PENDING NEW PUBLICATION CI** |

`BLOCKER`: **0**. `IMPORTANT`: **0**.

## 21. Boundary of this rereview

This rereview covers only the second P4 publication failure root cause, the `authority.rs`
doctest and doc-comment correction, the preservation of the valid authority negative proofs, the
proof that the product runtime and public API did not change, and the preservation of historical
CI. It did not redo the P4 lifecycle review and reopened no already-reviewed P4 product semantics.

Nothing was fixed, no code, test, doc or workflow outside this artifact was edited, nothing was
amended, rebased or pushed, no historical CI was rerun, P5 was not started and Trial #4 was not
authorised.

## 22. Recommendation

`P4PUB-04` is **FIXED** with **0 BLOCKER** and **0 IMPORTANT**. The correction is documentation
only, the product runtime and public API are mechanically proved unchanged, every still-valid
authority negative proof is preserved and independently re-verified against the Linux target, the
new positive proofs are probative and execute no process, and all historical evidence is intact.

**THE `P4PUB-04` CORRECTION MAY BE PUBLISHED ON A NEW SHA FOR NEW NATURAL HOSTED VALIDATION.**
**P5 REMAINS NOT AUTHORISED.**

Next gate: one fast-forward publication of the disposition, the correction and this rereview,
then new natural hosted P4 CI. The hosted Linux doctest runtime gate closes only on that new run.

**`HELM_LAUNCH_P4_P4PUB04_REREVIEW_PASSED_READY_FOR_NEW_CI`.**
