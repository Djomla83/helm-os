# BOUNDED INDEPENDENT REREVIEW OF P4PUB-05

> # 🔎 BOUNDED INDEPENDENT REREVIEW OF P4PUB-05
>
> **Session provenance. THIS SESSION AUTHORED OR MODIFIED NONE OF:**
> `2ff6b245bf1ea6f38d0b1be99395b439eb6e0ed3` (the third-publication failure disposition) and
> `8a00931fc22c506caf37b8e4b6db9e5d28c5af2f` (the release-library CI gate correction). It also
> authored no part of the published P4 head `6f9c73dfdde2e2321722519baa8fdf2764ca825f`, of the
> earlier publication chain, or of the P1–P4 implementation chain. It read the disposition and the
> correction under review and re-established every claim below from the preserved source, from the
> production release library as actually cross-compiled to the Linux target under the pinned
> toolchain, and from the workflow definitions at both heads.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **P1 ACCEPTED. P2 ACCEPTED. P3 ACCEPTED. P4 AUTHORISED — THIRD PUBLICATION FAILED, `P4PUB-05`
> WORKFLOW CORRECTION CANDIDATE, NOT ACCEPTED. P5 NOT AUTHORISED. THE COMPLETE helm-launch 0.1
> MODULE IS NOT PRODUCT-ACCEPTED. TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED` AND MUST NOT BE RERUN.
> TRIAL #4 IS NOT AUTHORISED.**

This is a **bounded** rereview of `P4PUB-05`: the root cause of the third P4 publication failure,
the supersession of the P3 release-library invariant, the correctness and probity of the
replacement positive release-library gate, the soundness of its assembly-artifact selection, the
preservation of the machine-code and fault-injection gates, the proof that no product source moved,
and the preservation of historical CI. It does **not** repeat the P4 lifecycle review and reopens
no already-reviewed P4 product semantics.

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **0** |
| `MINOR` | **3** (`P4DOC-01` carried; `P4PUB05-R1`, `P4PUB05-R2` new, nonblocking) |
| `GATE_PENDING` | **1** (hosted release-library gate) |
| `P4PUB-05` | **FIXED** |
| Old P3 negative oracle | **REMOVED** |
| New P4 positive oracle | **PROBATIVE** |
| Production release library | **ACTUALLY INSPECTED** |
| Assembly artifact selection | **SOUND** |
| Backend marker | **PROBATIVE** |
| Marker treated as public API | **NO** |
| Backend public | **NO** |
| Machine-code gates | **PRESERVED, byte-identical** |
| Fault-injection gate | **PRESERVED, byte-identical** |
| Step order | **PRESERVED** |
| Step 24 | **STILL CONFIGURED** |
| Product | **UNCHANGED** |
| Historical CI | **PRESERVED** |
| Classification | `HELM_LAUNCH_P4_P4PUB05_REREVIEW_PASSED_READY_FOR_NEW_CI` |

## 1. Starting state, independently verified

| Check | Observed |
|---|---|
| `git status --porcelain` | empty — clean worktree |
| `git branch --show-current` | `docs/helm-launch-architecture` |
| `git rev-parse HEAD` | `8a00931fc22c506caf37b8e4b6db9e5d28c5af2f` |
| `origin/docs/helm-launch-architecture` | `6f9c73dfdde2e2321722519baa8fdf2764ca825f` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Ahead / behind origin branch | **2 ahead, 0 behind** |
| Ancestry | `6f9c73d` → `2ff6b24` → `8a00931`, single parent each |
| Merge commits in range | **0** |

`origin` was fetched **read-only**. No ref was moved, no history was repaired and nothing was
pushed.

## 2. Step numbering, independently derived

The owner's disposition names *step 17* and *step 24*. Neither number appears in the workflow file,
so both were re-derived rather than assumed. Enumerating the job's steps at the published head
`6f9c73d` in file order, with GitHub's implicit `Set up job` as step 1:

| # | Step |
|---|---|
| 1 | `Set up job` (implicit) |
| 2 | `actions/checkout@fbc6f39…` |
| 3 | `Install Rust 1.95.0` |
| … | … |
| 16 | `Fault-injection positive control and release absence proof` |
| **17** | **`Confirm a release library instantiates no backend`** |
| 18–23 | capability-admission, P3 backend and traced-window, P3 fault-injection, off-cohort absence proofs, unsafe-confinement and boundary suites |
| **24** | **`Repository-level confinement checks, independent of cargo`** |
| 25 | `Print the deterministic receipt and plan identities` |

Both numbers land exactly on the steps the disposition names. The numbering is therefore confirmed
independently of the disposition's own claim.

## 3. Historical Phase 3 — preserved

| Run | Recorded |
|---|---|
| `helm-launch` `35511973984` | attempt 1 — **FAILURE** at step 17, `Confirm a release library instantiates no backend` |
| `HELM Rust workspace Linux` `35511973812` | attempt 1 — **SUCCESS** |
| Published failed Phase-3 head | `6f9c73dfdde2e2321722519baa8fdf2764ca825f` |

Both identifiers, both attempt numbers and both outcomes are preserved verbatim in
`docs/DECISIONS.md` and `docs/PROJECT_STATE.md`. Phase 1 (`94ee48da…`, runs `35499943908` and
`35499943903`, both attempt 1 **FAILURE**) and Phase 2 (`84b49ab9…`, runs `35507479482` and
`35507479458`, both attempt 1 **FAILURE**) remain recorded in the same disposition.

**NO RETRY. NO RERUN. NO REPLACEMENT.** No run was dispatched, cancelled, restarted or re-pointed
by this review, and no published history was amended.

## 4. Root cause — `P4PUB05_ROOT_CAUSE_CONFIRMED`

### 4.1 The failing step, as published

Step 17 at `6f9c73d` is a real production release build of the library, and a **negative** oracle
over its emitted assembly:

```yaml
      # A release library contains no backend at all, because P3 adds no public
      # consumer for it. …
      - name: Confirm a release library instantiates no backend
        if: runner.os == 'Linux'
        shell: bash
        run: |
          set -euo pipefail
          out="${RUNNER_TEMP}/p3-release-lib"
          rm -rf "$out"
          cargo rustc -p helm-launch --lib --locked --release --all-features \
            --target x86_64-unknown-linux-gnu --target-dir "$out" -- --emit=asm
          asm=$(find "$out" -name 'helm_launch-*.s' -print -quit)
          …
          if grep -q 'backend5child10child_main' "$asm"; then
            echo "FAILURE: a release library instantiated the P3 child entry point."
            exit 1
          fi
```

All four required properties hold: it **builds a production release Linux x86_64 `helm-launch`
library**; it **emits assembly**; it **fails when it finds the child backend marker**; and its
prose **explains this with the P3-era premise** that no public consumer exists.

### 4.2 The accepted P4 call path, re-established from source

| Edge | Location | Gating |
|---|---|---|
| public `launch` exported | `crates/helm-launch/src/lib.rs:295` — `pub use launch::{LaunchOutcome, launch};` | cohort `cfg` only |
| `launch` → `backend::spawn_for_lifecycle` | `crates/helm-launch/src/launch.rs:249` | **none** |
| `spawn_for_lifecycle` defined | `crates/helm-launch/src/backend/mod.rs:491` — `pub(crate) fn` | **none** |
| `spawn_for_lifecycle` → `spawn::spawn` | `crates/helm-launch/src/backend/mod.rs` — `spawn::spawn(&prepared, Fault::default())` | **none** |
| `spawn` defined | `crates/helm-launch/src/backend/spawn.rs:339` — `pub(super) fn` | **none** |
| `spawn` → `clone_and_dispatch` | `crates/helm-launch/src/backend/spawn.rs:405` | **none** |
| `clone_and_dispatch` → `child::child_main` | `crates/helm-launch/src/backend/spawn.rs:498` | **none** |
| `child_main` defined | `crates/helm-launch/src/backend/child.rs:76` — `pub(super) unsafe fn`, `#[inline(never)]` | **none** |

No edge is `cfg(test)`, debug-only, fault-injection-only or dead fixture material. The only
`#[cfg(test)]` in `spawn.rs` is the trailing test module at line 664. The fault-injection entry
points that *do* exist — `spawn_for_lifecycle_with_fault` and `stall_before_exec_fault` in
`backend/mod.rs`, and `launch_with_stall_coordination` in `launch.rs` — are each gated on
`#[cfg(all(feature = "test-fault-injection", debug_assertions))]` (the last additionally on
`cfg(test)`), so they are compiled out of a release build **even under `--all-features`**, exactly
as the correction's comment claims.

A P4 production release library is therefore **expected** to instantiate the private backend, and
the published negative oracle fails on precisely the evidence that proves P4 works.

**`P4PUB05_ROOT_CAUSE_CONFIRMED`.**

## 5. Supersession of the P3 invariant — `P4PUB05_P3_INVARIANT_SUPERSEDED`

"A release library instantiates no backend" was **valid for P3**, where the backend had no public
consumer, and is **false after accepted P4**, which exports `launch` on the cohort. The historical
P3 meaning is left intact — the disposition and the replacement comment both state what the old
gate asserted and why it was true then. The current P4 workflow no longer enforces it. Backend
presence in a P4 release library is **not** classified anywhere as a product regression; the
disposition records the defect as *evidence*, not *mechanism*, and this review concurs.

## 6. Correction scope — `P4PUB05_SCOPE_SOUND`

`git diff 6f9c73d..8a00931 --name-status`:

| File | Commit |
|---|---|
| `docs/DECISIONS.md` | `2ff6b24` |
| `docs/PROJECT_STATE.md` | `2ff6b24` |
| `.github/workflows/helm-launch.yml` | `8a00931` |

Commit 1 touches the two documentation files only (+202 lines, no deletions). Commit 2 touches the
workflow only (+36 / −11). A path-scoped diff over `crates/`, `Cargo.toml`, `Cargo.lock` and
`tools/` across the full range returns **zero** files. No crate source, no manifest or lockfile, no
tools, no product test.

Neither commit carries a `Co-Authored-By` trailer or any AI attribution.

## 7. The positive release-library gate — `P4PUB05_POSITIVE_RELEASE_GATE_SOUND`

The replacement step keeps the build **unchanged** and inverts only the assertion:

| Property | Published `6f9c73d` | Corrected `8a00931` |
|---|---|---|
| Build command | `cargo rustc -p helm-launch --lib --locked --release --all-features --target x86_64-unknown-linux-gnu --target-dir "$out" -- --emit=asm` | **identical** |
| Fresh target dir | `rm -rf "$out"` | **identical** (`$out` renamed `p3-release-lib` → `p4-release-lib`) |
| `--emit=asm` applied to | the real production `helm-launch` lib target | **identical** |
| Missing-assembly guard | `TEST ENVIRONMENT FAILURE`, `exit 1` | **identical** |
| Oracle | `if grep -q … then exit 1` | `if ! grep -q … then exit 1` |
| `set -euo pipefail` | present | **present** |
| `continue-on-error` | absent | **absent** |
| `if:` condition | `runner.os == 'Linux'` | **identical** |
| `shell` | `bash` | **identical** |

The gate now **fails when the expected backend evidence is absent** and **passes when it is
present**. The whole workflow contains no `continue-on-error` and no `--no-fail-fast`; the
`fail-fast: false` at line 84 is the pre-existing matrix strategy setting and is untouched.

The failure message added by the correction is notably honest: it names both possible causes —
"Either the public launch path no longer instantiates it, or this gate read the wrong artifact" —
rather than asserting a single diagnosis.

## 8. Assembly artifact selection — `P4PUB05_RELEASE_ASM_SELECTION_SOUND`

This was treated as load-bearing and **not** assumed. The gate selects with
`asm=$(find "$out" -name 'helm_launch-*.s' -print -quit)`, unchanged from the retired gate. For a
**negative** oracle a mis-selection yields a false pass; for a **positive** oracle it could yield a
positive marker read from an artifact that is not the production top-level library. The question is
therefore whether more than one matching artifact **can** exist.

### 8.1 Structural argument (route A)

| Condition | Finding |
|---|---|
| Stale artifacts | impossible — `rm -rf "$out"` immediately precedes the build |
| Which targets receive `--emit=asm` | `cargo rustc` applies trailing `--` flags to **exactly one** selected target; `--lib` selects the single `helm-launch` library target. No dependency and no other target is given `--emit=asm`, so only one `rustc` invocation emits assembly at all |
| A dependency named `helm_launch` | none — the crate's only dependencies are `serde`, `serde_json`, `sha2`, and cohort-gated `rustix` and `libc` |
| `build.rs` in `helm-launch` | none, and no `[build-dependencies]` |
| Multiple `crate-type`s | none — no `[lib]` table, so the default single `rlib` |
| Incremental fragments | none — no `[profile.release]` override in the workspace `Cargo.toml`, so `incremental` is off for release |
| `RUSTFLAGS` / config injection | none — the repository has no `.cargo/config.toml` |

### 8.2 Empirical confirmation

The exact CI build command was reproduced locally against the working tree under **rustc 1.95.0**,
cross-compiling to `x86_64-unknown-linux-gnu` with a fresh target directory:

```
Finished `release` profile [optimized] target(s) in 5.78s

find "$out" -name '*.s'              → 1 file
find "$out" -name 'helm_launch-*.s'  → 1 file
  …/x86_64-unknown-linux-gnu/release/deps/helm_launch-2effc41e17f34357.s
find "$out" -name '*cgu*'            → none
```

Exactly **one** assembly artifact exists in the entire target directory — not merely one matching
the glob — and no codegen-unit fragments are produced. That single file contains **both** the
public `launch` symbol and `child_main`, so it is the whole-crate assembly of the top-level
production library and not a partial codegen output.

With exactly one match, `find … -print -quit` is deterministic. **Route A is satisfied.** See
`P4PUB05-R1` for the forward-looking hardening observation.

## 9. Marker probity — `P4PUB05_MARKER_PROBATIVE`

The marker `backend5child10child_main` was reproduced in the actual emitted release assembly. It
occurs on five lines, and every one of them is executable-code material:

```
13039: .section .text._ZN11helm_launch7backend5child10child_main17h7e64a7170560004aE,"ax",@progbits
13041: .type    _ZN11helm_launch7backend5child10child_main17h7e64a7170560004aE,@function
13042: _ZN11helm_launch7backend5child10child_main17h7e64a7170560004aE:
13345: .size    _ZN…child_main…E, .Lfunc_end32-_ZN…child_main…E
13479: callq    _ZN11helm_launch7backend5child10child_main17h7e64a7170560004aE
```

It is therefore simultaneously an **allocatable-executable `.text` section**, a **`@function` type
declaration**, a **symbol definition**, a **non-zero-sized body**, and a **direct call edge**. It is
not a string constant, not debug metadata, not a comment, not a test symbol, not fault-injection
material and not unreachable fixture material.

The emitted body is substantive: **217 instruction lines** containing **16** `syscall` instructions
— the real closed child sequence, not a stub.

### 9.1 The call edge closes to the public entry point

The single `callq` to `child_main` lies inside
`_ZN11helm_launch7backend5spawn18clone_and_dispatch…E`, and the single `callq` to
`clone_and_dispatch` lies inside `_ZN11helm_launch6launch6launch…E` — the **public** `launch`, which
is present in the release assembly as an `@function`. `spawn_for_lifecycle` and `spawn` were
inlined into `launch`, which is exactly what the source structure predicts, and `clone_and_dispatch`
survives out of line because it is `#[inline(never)]`. The chain

```
helm_launch::launch::launch            (public)
  └─ callq backend::spawn::clone_and_dispatch
       └─ callq backend::child::child_main
```

is therefore closed **by direct call edges inside the production release artifact itself**, not
merely inferred from source.

### 9.2 No false-positive material

A search of the release assembly for `backend5tests`, `9injection`, `test_fault`,
`stall_before_exec` and `MODE_STALL` returns **zero** hits. `--all-features` does enable
`test-fault-injection`, but its `debug_assertions` co-gate removes it from the release build, as the
correction's comment states. There is no test or injection material in the inspected artifact that
could manufacture a positive.

### 9.3 Both gate directions reproduced

Running the exact CI shell logic of each version against the reproduced artifact:

| Oracle | Result |
|---|---|
| Retired negative gate (`if grep -q`) | `FAILURE: a release library instantiated the P3 child entry point.` → `exit 1` — **reproduces the hosted step-17 failure exactly** |
| New positive gate (`if ! grep -q`) | `P4 release library instantiates the private child backend, as expected` → **passes** |

This is **structural cross-build evidence only**. It does not substitute for hosted CI.

## 10. Marker is not product API — `P4PUB05_MARKER_SCOPE_SOUND`

The corrected workflow comment states it explicitly:

> This is a structural release-build evidence check. The symbol spelling is an implementation detail
> of the Rust version this workflow pins, not public API and not a product contract: `child_main`
> stays private and is reachable only from inside the crate. P5 may replace or harden this marker.

`docs/DECISIONS.md` matches: the marker is "a **CI evidence marker under the pinned Rust 1.95.0
toolchain**; it is neither public API nor a product contract, and P5 may harden or replace it."
Neither the workflow nor the documentation elevates the mangled spelling to a product contract, an
architectural symbol name or a stable ABI.

## 11. Pinned toolchain — `P4PUB05_TOOLCHAIN_ASSUMPTION_SOUND`

Step 3 still runs `rustup toolchain install 1.95.0 --profile minimal --component rustfmt,clippy`
followed by `rustup default 1.95.0`, and step 4 records `rustc -vV` and `cargo -V` into the log. The
correction does not touch either step; the only toolchain-related change is the added comment
sentence describing the pin.

The toolchain **is** genuinely pinned, so the fragility question does not arise at severity. The
reproduction above was run under a local `rustc 1.95.0 (59807616e 2026-04-14)`, the same version the
workflow pins, and produced the expected mangled spelling. The dependence on one mangled name is
acceptable under a pin and is explicitly flagged in-file as P5-replaceable.

## 12. Private backend boundary — `P4PUB05_PRIVATE_BACKEND_BOUNDARY_PRESERVED`

`crates/helm-launch/src/lib.rs` declares `mod backend;` **without** `pub`, and its public surface is
`authority::{…}`, `error::{…}`, `launch::{LaunchOutcome, launch}`, `model::{…}`, `plan::{…}` and
`receipt::{LaunchReceipt, MAX_RECEIPT_BYTES}`. There is no `pub use backend::…` line. A scan of
`crates/helm-launch/src/backend/` for `pub fn` / `pub struct` / `pub enum` excluding
`pub(crate)` / `pub(super)` / `pub(self)` returns **zero** items, so no raw child or backend item is
externally nameable.

The corrected workflow nowhere implies the backend is public; its comment says the opposite. The
accepted state — **public `launch` → private backend** — is preserved.

## 13. Machine-code gates — `P4PUB05_MACHINE_CODE_GATES_PRESERVED`

Both `Child machine-code closure — debug/test profile` (step 14) and `Child machine-code closure —
release codegen` (step 15) were compared command-for-command across `6f9c73d..8a00931`. Their
`run:` blocks, target directories, `--profile`, `--rustc-flag` and `--label` arguments are
**byte-identical**. Only the explanatory comment above step 15 changed.

Each still proves its own property independently of step 17: they invoke
`tools/helm_launch_child_closure.py` against the test-instantiated profile and walk the emitted call
closure from `child_main`, which is a different question from whether the production release library
reaches the backend at all.

## 14. Release-codegen comment — `P4PUB05_RELEASE_COMMENT_SOUND`

The stale claim removed:

> A plain release library build discards the whole backend, so release codegen settings are applied
> to the profile that does instantiate the child.

The replacement no longer claims the release library discards the backend, and distinguishes the two
properties correctly:

> This deliberately keeps the test-instantiated profile and applies the release optimisation and
> debug-assertion settings to it, because that keeps `child_main` an independently inspectable root
> of its own call closure rather than one symbol among many in a whole-library build. Whether the
> actual P4 production release library reaches the backend at all is a different property, proved
> separately by the release-library gate below.

**(A)** production release-library backend reachability → step 17. **(B)** closed-child
release-style machine-code analysis → the dedicated machine-code gate. The distinction is stated
accurately, and the stated reason for keeping the test profile — an independently inspectable
closure root — is borne out by the reproduction, where `spawn_for_lifecycle` and `spawn` were
inlined away in the whole-library build.

## 15. Injection gate — `P4PUB05_INJECTION_GATE_PRESERVED`

`Fault-injection positive control and release absence proof` (step 16) is **byte-identical** across
the correction: the same `tools/helm_launch_injection_proof.py`, the same
`--root "${RUNNER_TEMP}/p3-injection"`, the same `--target x86_64-unknown-linux-gnu`.

P4 production release backend presence does **not** imply fault injection is present, and the two
are proved separately: injection absence remains the injection proof's own job, and this review
independently confirmed zero injection material in the release assembly (§9.2). The two facts are
consistent and neither is derived from the other.

## 16. Step order — `P4PUB05_STEP_ORDER_PRESERVED`

The replacement gate remains **step 17**: after the fault-injection proof (16) and before the
capability-admission cases (18), the P3 backend and traced-window cases (19), the P3
fault-injection cases (20), the off-cohort absence proofs (21–22), the unsafe-confinement and
boundary suites (23) and step 24. The job still defines 24 steps. No step was reordered, no `if:`
condition changed — step 17 remains `if: runner.os == 'Linux'` — and nothing was moved to hide a
failure behind an earlier one.

## 17. Step 24 evidence accounting — `P4PUB05_PHASE3_STEP24_ACCOUNTING_SOUND`

The `helm-launch` explicit repository-level confinement step 24,
`Repository-level confinement checks, independent of cargo`, was **SKIPPED**, because step 17 failed
first. It is **not** separately defective and receives no correction. It remains configured in the
corrected workflow at step 24.

The workspace run `35511973812` nevertheless executed
`python3 -m unittest discover -s tools/tests -v`, and its logs show real **PASS** execution of
`test_helm_launch_confinement` and `test_helm_launch_machine_proofs`. Repository-level evidence
therefore had **substitute Phase-3 execution in the workspace run**.

This review records all three statements together and none of them alone:

* `helm-launch` step 24 is **NOT** called passed.
* Repository-level evidence is **NOT** called wholly absent.
* The next publication **must still reach and pass the explicit `helm-launch` step 24**.

Substitute evidence does not convert the failed `helm-launch` run into a success.

## 18. Product boundary — `P4PUB05_PRODUCT_UNCHANGED`

No change under `crates/**`, `Cargo.toml`, `Cargo.lock` or `tools/**` — confirmed by an explicit
path-scoped diff returning zero files. The public API, runtime, lifecycle, receipt, backend,
`unsafe` boundary and P3 child contract are untouched. No `FULL P4 REVIEW REQUIRED` condition is
triggered.

## 19. Validation

| Check | Result |
|---|---|
| `python tools/validate_docs.py` | **PASS** — 144 markdown files, 255 JSON files, 1644 link targets |
| `python -m unittest discover -s tools/tests` | **OK** — ran 831 tests, 0 failures, 74 skipped (Linux-only cases, off-cohort host) |
| `git diff --check` | clean |
| Workflow YAML parse | **valid** — 24 job steps; gate step present exactly once; `if: runner.os == 'Linux'`; `shell: bash`; no `continue-on-error` |
| `cargo test -p helm-launch --locked` | **PASS** — 42 + 20 + 17 + 15 + 19 + 2 + 23 tests, 0 failures, including 23 `compile_fail` doctests |
| `tests/p2_boundary.rs`, `tests/p4_boundary.rs`, `tests/plan_contract.rs` | **PASS** — 20 / 15 / 19, 0 failures |
| Release cross-build under pinned `rustc 1.95.0` | **reproduced** — assembly exists, exactly one artifact, expected marker present |
| Retired negative gate against that artifact | **would fail** — reproduces the hosted step-17 failure |
| New positive gate against that artifact | **passes** |

The host is `x86_64-pc-windows-msvc` with the `x86_64-unknown-linux-gnu` target installed, so the
release **library** cross-build is faithful (rlib output needs no cross linker) while the Linux
**runtime** suites cannot run here. That is the boundary of this evidence: it is structural
cross-build evidence only.

## 20. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P4PUB-05` | — | `.github/workflows/helm-launch.yml` step 17 | Yes — it failed the hosted Phase-3 run | Stale P3 negative release-library oracle, superseded by accepted P4 | **FIXED.** Replaced by a P4-correct positive gate; verified probative by local reproduction |
| `P4PUB05-R1` | `MINOR` / `BACKLOG_NONBLOCKING` | step 17 artifact selection | No — uniqueness holds today, structurally and empirically | `find … -print -quit` relies on exactly one `helm_launch-*.s` existing, which is true but is **not asserted** by the workflow. A future `[lib] crate-type` addition, a release `incremental` override, or a second crate named `helm_launch` would make the selection nondeterministic, and the step's own failure message already contemplates "this gate read the wrong artifact" | **CARRIED.** Not fixed here — outside the authorised correction scope. Suggested P5 hardening: assert the match count is exactly 1 |
| `P4PUB05-R2` | `MINOR` / `BACKLOG_NONBLOCKING` | step 17 oracle strength | No | The oracle greps a substring that matches the definition, the `.type`, the `.size` and the `callq` lines without distinguishing them, so it proves the backend child entry is **emitted**, not that it is **reachable from `launch`**. Emission of a crate-private, non-generic function in a release rlib already implies the collector reached it, and this review confirmed the explicit call edge directly (§9.1) | **CARRIED.** Not fixed here. Suggested P5 hardening: assert the `callq` edge, or use the closure walker |
| `P4DOC-01` | `MINOR` / `BACKLOG_NONBLOCKING` | README, `linux_admission.rs`, `backend/mod.rs`, `backend/spawn.rs` | No — prose and comments only | Stale P2/P3 "future launch / does not exist / not authorised" wording outside `authority.rs` | **CARRIED.** Not edited and not escalated here; no new evidence bears on it |

`BLOCKER`: **0**. `IMPORTANT`: **0**.

## 21. Pending gate

**`P4PUB-05` HOSTED RELEASE-LIBRARY GATE: PENDING NEW PUBLICATION CI.**

The corrected step 17 has been proved sound by local structural cross-build reproduction under the
pinned toolchain. It has **not** executed on hosted Linux. Only a new natural `helm-launch` run on a
new SHA can confirm it, and that run must also reach and pass step 24, which Phase 3 skipped.

## 22. Recommendation

`P4PUB-05` is **FIXED**. The correction is bounded to the workflow, the retired oracle is replaced
rather than deleted, the replacement is probative, its artifact selection is sound, its marker is a
real function definition **and** a real call edge reachable from the public `launch`, the
machine-code and injection gates are byte-identical, step order is unchanged, step 24 remains
configured, and no product source moved.

**THE `P4PUB-05` CORRECTION MAY BE PUBLISHED ON A NEW SHA FOR NEW NATURAL HOSTED VALIDATION.**

**P5 REMAINS NOT AUTHORISED. TRIAL #3 STAYS FROZEN `MECHANISM_REJECTED`. TRIAL #4 IS NOT
AUTHORISED.**

## 23. Next gate

**ONE FAST-FORWARD PUBLICATION** of the disposition, the workflow correction and this rereview,
**THEN NEW NATURAL HOSTED P4 CI.** No retry, no rerun and no replacement of any historical run.

## 24. Classification

`HELM_LAUNCH_P4_P4PUB05_REREVIEW_PASSED_READY_FOR_NEW_CI`
