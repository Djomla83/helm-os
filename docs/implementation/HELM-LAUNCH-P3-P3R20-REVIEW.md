# HELM-LAUNCH P3 — BOUNDED INDEPENDENT REVIEW OF P3R-20

> # 🔎 BOUNDED INDEPENDENT REVIEW OF `P3R-20` — NOT A FULL P3 UNSAFE REVIEW
>
> **Session provenance.** **THIS REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `86d798ebb66291d55e222e056e0f73e6b4df5fb6` (the owner disposition recording the first hosted
> publication failure and accepting `P3R-20`) and
> `f15d19d43fa0c50e58a8a0236ef17a410ad770cc` (the `P3R-20` harness correction).
> It authored no part of the P3 implementation chain. It read those commits, re-derived the root
> cause from the pre-correction source, and re-established the corrected behaviour from `rustc`
> output and from standalone probes written in this session outside the repository.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact, not an inference from commit metadata.

> **This document does NOT supersede
> [`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md)
> (`4c834415`), [`HELM-LAUNCH-P3-CORRECTION-REREVIEW.md`](HELM-LAUNCH-P3-CORRECTION-REREVIEW.md)
> (`5c577d45`) or [`HELM-LAUNCH-P3-P3R15-REREVIEW.md`](HELM-LAUNCH-P3-P3R15-REREVIEW.md)
> (`3a9368f8`).** Those three remain the authoritative P3 independent review record for every area
> this correction does not touch. This review is bounded to `P3R-20`: its root cause, its
> correction, the probity of its regression, the preservation of the historical first-publication
> evidence, and proof that normal product backend semantics are unchanged.

> **P1 ACCEPTED. P2 ACCEPTED. P3 PUBLISHED CANDIDATE — FIRST HOSTED VALIDATION FAILED —
> `P3R-20` CORRECTION CANDIDATE. P4 AND P5 NOT AUTHORISED. TRIAL #3 STAYS `MECHANISM_REJECTED`
> AND MUST NOT BE RERUN. TRIAL #4 NOT AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| `BLOCKER` | **0** |
| `IMPORTANT` | **0** |
| `MINOR` | 3 |
| `GATE_PENDING` | 1 |
| `P3R-20` | **INDEPENDENTLY VERIFIED CORRECTED** |
| Unique source pathname | **SOUND** |
| Unique staged output pathname | **SOUND** |
| Atomic no-replace publication | **SOUND** |
| Winner / loser protocol | **SOUND** |
| Final object stability | **SOUND** |
| Concurrent regression test | **PROBATIVE** |
| Product backend | **BYTE-UNCHANGED** |
| Historical first failure | **PRESERVED** |
| Linux concurrency regression runtime | **`GATE_PENDING`** — pending corrected publication CI |

**Classification: `HELM_LAUNCH_P3_P3R20_REVIEW_PASSED_READY_FOR_CORRECTED_PUBLICATION_CI`.**

## 1. Target and starting state

| Item | Value |
|---|---|
| Published P3 head (historical) | `3a9368f845452110afc859ed899acae9384c7d9b` |
| Owner-disposition commit | `86d798ebb66291d55e222e056e0f73e6b4df5fb6` |
| `P3R-20` correction commit | `f15d19d43fa0c50e58a8a0236ef17a410ad770cc` |
| Branch | `docs/helm-launch-architecture` |
| Local `HEAD` before this review | `f15d19d43fa0c50e58a8a0236ef17a410ad770cc` |
| `origin/docs/helm-launch-architecture` | `3a9368f845452110afc859ed899acae9384c7d9b` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| Local relation to remote | **2 ahead, 0 behind** |
| Worktree | clean before and after |
| Pushed | **NO** |

`origin` was fetched read-only. No history was repaired, rebased or rewritten.

## 2. Commit scope

`86d798e` touches exactly `docs/DECISIONS.md` and `docs/PROJECT_STATE.md`.
`f15d19d` touches exactly `crates/helm-launch/src/backend/tests.rs`.

`git diff --stat 3a9368f f15d19d` reports **three files in total** — those two documents and the
test harness. That single fact discharges every byte-identity requirement at once: `child.rs`,
`spawn.rs`, `syscall.rs`, `mod.rs`, `injection.rs`, `lib.rs`, `Cargo.toml`, `Cargo.lock`,
`.github/workflows/**`, `tools/helm_launch_child_closure.py`,
`tools/tests/test_helm_launch_machine_proofs.py`, ADR-0024 and the productization plan are all
untouched. Blob identity was additionally confirmed per file:

| File | `3a9368f` blob | `f15d19d` blob |
|---|---|---|
| `backend/child.rs` | `d8b75c67` | **identical** |
| `backend/spawn.rs` | `5575639e` | **identical** |
| `backend/syscall.rs` | `38d95fb4` | **identical** |
| `backend/mod.rs` | `12467e87` | **identical** |
| `backend/injection.rs` | `fa3704d0` | **identical** |
| `src/lib.rs` | `c3b3b524` | **identical** |
| `Cargo.toml` / `Cargo.lock` | `953ca8a3` / `b2df4153` | **identical** |

`backend/authority.rs` is absent from the crate in both revisions; exec authority lives in
`crates/helm-launch/src/authority.rs`, which is likewise untouched. **No product semantics changed,
so a full P3 unsafe review is not required.**

## 3. Root cause — independently re-established

The author report was not taken on trust. The pre-correction `fixture_binary` was read from
`3a9368f:crates/helm-launch/src/backend/tests.rs`:

```rust
let source_path = fixture_root().join(format!("{stem}.rs"));
fs::write(&source_path, source).expect("write fixture source");
let staged = fixture_root().join(format!("{stem}.{}.staging", std::process::id()));
let status = Command::new("rustc")
    .args(["--edition", "2021", "-C", "debuginfo=0", "-o"])
    .arg(&staged).arg(&source_path).status().expect("run rustc");
// ...
fs::rename(&staged, &binary).expect("stage fixture");
```

Three independently verified facts:

1. **`fixture_root()` is one shared directory** — a `OnceLock` over
   `temp_dir()/helm-launch-p3-fixtures`. Every builder writes into it.
2. **Every concurrent builder derives the same names.** `stem` is content-addressed, so builders of
   the same fixture share it; `std::process::id()` is identical for every thread of one process.
   The source pathname `{stem}.rs` and the staged pathname `{stem}.{pid}.staging` are therefore
   **shared, not unique**. Seventeen `#[test]` functions call `report_fixture()`, and cargo runs
   tests as parallel threads of a single process — confirmed by counting the call sites.
3. **`rustc` derives its intermediate object basenames from the `-o` pathname.** This is the
   load-bearing mechanical claim, so it was measured rather than assumed. Compiling with
   `-o …/stemUNIQUE-1234-7.staging --crate-name p3r20_deadbeef -C save-temps=yes` emitted, into the
   **output directory**:

   ```
   stemUNIQUE-1234-7.p3r20_deadbeef.691603254ca7264e-cgu.0.rcgu.o
   stemUNIQUE-1234-7.7hx20cjhg28qrdivxgdetqjs7.rcgu.o
   ```

   The basename is prefixed by the **`-o` filestem**, not by the crate name and not by the source
   name.

Consequences, and the hosted symptoms they explain:

| Historical symptom | Explanation |
|---|---|
| `rust-lld: error: undefined hidden symbol: …` (run `35442641728`) | Two concurrent `rustc` runs wrote the same `…-cgu.N.rcgu.o` paths; one linker read an object another was still writing, so the fixture's own code-generation units were incomplete. |
| `cannot open …-cgu.0.rcgu.o: No such file or directory` (run `35442641743`) | One `rustc` deleted the shared intermediate object another was about to link. |
| `rename` `NotFound` | `fs::rename(&staged, &binary)` after a peer had already renamed the one shared staged pathname away. |
| `MeasurementInstabilityDetected`, `ETXTBSY`, `ENOEXEC` | `fs::rename` is **unconditional and replacing**: a late builder replaced an already-published inode while other tests were executing that pathname. |
| A **different set of tests failing each time** | The collision is a race, so which builder loses varies per run. |

**The historical two-run failure is coherently and completely explained by the shared fixture build
namespace.** No alternative root cause was found. The product launcher mechanism is not implicated:
nothing in the failing path is `helm-launch` product code — it is the test harness's fixture
compiler.

This was reproduced independently. A standalone replica of the **pre-correction** algorithm, written
in this session outside the repository, run with six `Barrier`-released threads and real `rustc`:

```
[OLD      ] succeeded=1/6 runs_correctly=true leftovers=1 errors=["rustc failed", ...]
[CORRECTED] succeeded=6/6 runs_correctly=true leftovers=0 errors=[]
```

Five of six builders lose under the old algorithm. On this Windows host the collision surfaces as
`LNK1104: cannot open file …probeold.20452.staging` — the same shared-output-namespace defect,
reported by a different linker. The corrected algorithm has all six succeed, produce a correctly
running object, and leave nothing behind.

## 4. Unique build namespace — `P3R20_SOURCE_NAMESPACE_SOUND`, `P3R20_STAGING_NAMESPACE_SOUND`

```rust
let nonce = FIXTURE_BUILD_NONCE.fetch_add(1, Ordering::Relaxed);
let unique = format!("{stem}-{}-{nonce}", std::process::id());
let source_path = fixture_root().join(format!("{unique}.rs"));
let staged      = fixture_root().join(format!("{unique}.staging"));
```

* **Unique source path: YES.** `{stem}-{pid}-{nonce}.rs`. Two builders never write one file.
* **Unique staged output: YES.** `{stem}-{pid}-{nonce}.staging`, and therefore — by the measured
  derivation in §3 — a unique `…rcgu.o` intermediate namespace as well.
* **Process id alone is not relied upon.** The nonce is the in-process discriminator; the pid
  separates processes. Both are present.
* **`Ordering::Relaxed` is correct here.** The only property required of `fetch_add` is that no two
  callers observe the same value, which the atomicity of the read-modify-write guarantees
  irrespective of ordering. No data is published through this counter. The regression's
  before/after `load` is ordered by thread `join`, which is itself a happens-before edge.
* **Overflow is not a reachable defect.** `AtomicU64` wraparound needs 2^64 compiler invocations in
  one process; no live name can be reused before then. Not raised as a finding.
* **Cleanup cannot cross builders.** Both removed names embed this builder's pid and nonce, so no
  builder can unlink another's source or staged file.

## 5. Crate name and `rustc` invocation

```rust
let crate_name: String = stem.chars()
    .map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
```

* Valid Rust crate-name syntax: the stem is `{name}-{16 hex}`, so the mapping yields
  `[A-Za-z0-9_]+` beginning with a letter.
* **The crate name contains no pid and no nonce** — it is derived from the content-addressed stem
  alone, so every concurrent builder compiles the same fixture under the same crate identity.
* Sharing a crate name across concurrent builders is **safe**, because §3 measured that the `-o`
  filestem prefixes every intermediate object basename; the unique `-o` path alone separates them.
* `--crate-name` is **necessary**, not decorative: without it `rustc` derives the crate name from
  the source-file stem, which now carries the nonce, and the hyphen/dot constraint the commit
  message describes is real.
* **The unique source pathname does not alter intended fixture behaviour.** Nothing hashes or
  compares the fixture *binary*; the only `Sha256` inputs in the harness are fixture *source* text.
  The report fixture's stderr marker comes from its `NAME` constant, and the assertions check
  `contains(REPORT_FIXTURE_NAME)`, not a path.

### MINOR-1 — the "same bytes" claim is stronger than the truth, and than the contract

The commit message states the emitted fixture is "the same bytes whichever builder compiled it".
That is not exactly true: `rustc` embeds the source pathname in panic-location strings, which was
confirmed by finding the source filename inside a probe binary built with `-C debuginfo=0`. Builders
therefore emit objects that differ in those bytes.

This is **not a defect**. Byte-for-byte reproducibility is not part of the product or test contract,
nothing consumes the fixture's bytes, and the content-addressed name still describes the *source*
content, which is what the cache is keyed on. The claim in the commit message is simply overstated.
**Non-load-bearing. No change required.**

## 6. Atomic no-replace publication — `P3R20_ATOMIC_PUBLICATION_SOUND`

```rust
match fs::hard_link(&staged, &binary) {
    Ok(()) => {}
    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
    Err(error) => panic!("FIXTURE BUILD FAILURE: could not publish fixture {name}: {error}"),
}
let _ = fs::remove_file(&staged);
```

* The staged executable is **complete before publication**: `rustc` has exited successfully and its
  status has been checked.
* **There is no unconditional `rename`.** `fs::rename` no longer appears in `fixture_binary`.
* **There is no `if !final.exists() { rename(…) }` TOCTOU protocol.** Publication is a single
  `link(2)`, which fails `EEXIST` atomically in the kernel — the decision and the act are the same
  operation.
* Success ⇒ this builder published. `AlreadyExists` ⇒ another builder published and this one defers.
  Every other error is **loud**, and deliberately not swallowed.
* Staged and final are both `fixture_root()` children, so they are on one filesystem — the
  precondition `hard_link` needs. The staged name can never equal the final name, since it carries
  the `.staging` suffix.

## 7. Winner / loser behaviour and cleanup — `P3R20_WINNER_LOSER_PROTOCOL_SOUND`

| | Winner | Loser |
|---|---|---|
| Publication | `hard_link` succeeds; `binary` names the completed staged inode | `AlreadyExists`; the winner's object is left untouched |
| Own staging | removed afterwards; the inode survives under `binary` | removed; it was a **different** inode, dropped unused |
| Own source | removed unconditionally after `rustc` exits | removed unconditionally after `rustc` exits |
| Returns | the content-addressed final path | the same content-addressed final path |

No cleanup path can unlink the shared final object: the only `remove_file` targets in
`fixture_binary` are `source_path` and `staged`, both of which embed the builder's own pid and
nonce. **No builder ever removes or replaces a name it does not own.**

### MINOR-2 — the staged file is not removed on the publication-error panic path

If `hard_link` fails with something other than `AlreadyExists`, the `panic!` fires before
`remove_file(&staged)`, leaving one staged object behind. This is a loud, already-failing path;
retaining the artifact is arguably useful evidence. **Non-load-bearing. No change required.**

## 8. Final object stability — `P3R20_FINAL_OBJECT_STABILITY_SOUND`

The regression captures `dev` and `ino` per builder at return, compares every observer against the
first, then re-stats after all joins:

```rust
assert_eq!((*observed_device, *observed_inode), (device, inode), /* … */);
assert_eq!((settled.dev(), settled.ino()), (device, inode), /* … */);
assert_eq!(settled.nlink(), 1, "a staging link outlived its builder");
```

* A losing builder that published over the winner — as the old unconditional `rename` would — would
  show a different inode to builders that had already returned. That is caught.
* The post-join `settled` check catches replacement after the observation window.
* **The `nlink == 1` assertion is correctly placed and is not the transient-nlink trap.**
  `hard_link` does give the staged inode two names, but the winner's `remove_file(&staged)` runs
  before it returns, so by the time all six threads have joined every staging name is gone. The
  assertion is evaluated only after the joins. It correctly proves both that the winner released its
  staging name and that no loser left a second link.

**Required final claim holds:** after all concurrent builders complete, every observer resolves the
same path and the same `(dev, ino)`, and the winner was not replaced.

## 9. Cold-cache regression probity — `P3R20_CONCURRENCY_TEST_PROBATIVE`

`concurrent_builders_of_one_fixture_publish_exactly_one_object` was read in full.

| Requirement | Result |
|---|---|
| Real `fixture_binary` | **YES** — the function under review, not a replica |
| Real `rustc` compilation | **YES** |
| Same fixture name and source for every thread | **YES** — one `source`, cloned |
| Multiple threads | **YES** — six |
| `Barrier`-synchronised start | **YES** — `Barrier::new(6)`, all six released together |
| No sleep-based race assumption | **YES** — nothing sleeps |
| Genuinely uncached identity | **YES** — see below |

**The cold cache is enforced, not assumed.** The marker embeds pid and nanosecond clock, so the
source text — and therefore the content-addressed digest — differs per run and per process. The test
then **independently recomputes** the path `p3r20-{sha256(source)[..16]}` and asserts
`!published.exists()` before spawning anything. It does not merely trust the timestamp string; it
checks the resulting cache key. Collision with an existing entry is excluded by the assertion
itself.

The fixture source is deliberately shaped to need several code-generation units and a real link, so
each builder is a genuine, long compiler invocation rather than one that finishes before it can
overlap.

### MINOR-3 — `compiled >= 2` is satisfiable by unrelated builders

The guard that at least two builders reached the compiler is computed from the **global**
`FIXTURE_BUILD_NONCE`:

```rust
let compiled = FIXTURE_BUILD_NONCE.load(Ordering::Relaxed) - before;
assert!(compiled >= 2, /* … */);
```

That counter is incremented by *every* `fixture_binary` call that reaches `rustc`, including cold
builds of `report_fixture` running concurrently in other test threads. On a cold CI run the
assertion can therefore be satisfied without two builders **of this fixture** having compiled. As
written it proves less than it reads.

It is nonetheless **not an `IMPORTANT`**, because the false pass it guards against is unreachable by
construction. `Barrier::new(6)` releases all six threads only once all six have arrived; each then
reaches `binary.exists()` within microseconds. For five threads to take the cache-hit path, the
first would have to complete `fs::write`, a full multi-CGU `rustc` compile and link, and the
`hard_link`, inside that microsecond window. Real concurrent construction is a **synchronisation
fact established by the barrier**, not by this counter. Tightening the guard to count only this
fixture's builders would make the assertion say what it means. **Non-blocking.**

## 10. Regression result assertions

Verified present and correct:

* every caller returns (`join().expect(…)`) — so no builder panicked on a link failure, a missing
  intermediate object or a failed staging rename;
* **all six results are checked**, winners and losers alike — the loops iterate `&observed`, so no
  loser result is ignored in favour of a single winning thread;
* every caller returns the same content-addressed path;
* the published object is executable (`mode & 0o111`), **actually runs**, and prints the exact
  marker it was built with — a partially linked object could not;
* all observers agree on `(dev, ino)`, and the identity is unchanged after all joins;
* exactly one name refers to the object (`nlink == 1`);
* no builder left a source or staged file behind — the leftover scan is prefixed with `{stem}-`,
  which matches exactly this test's six builders' temporaries and nothing else;
* the test removes the object it created, unlike the shared fixtures.

## 11. Cross-process correctness

The correctness basis is the **filesystem**, not a process-local lock — and indeed no mutex is
introduced, so no lock is held across launching anything.

* `pid` separates live processes' temporary names.
* The per-process nonce separates builders inside one process.
* `hard_link` arbitrates the cross-process publication race atomically in the kernel.

**PID reuse was considered and is not a live defect.** A stale temporary from a dead process could in
principle share a name with a live builder, but the stem is content-addressed, so the source text
written would be byte-identical, and `rustc` truncates the staged output. The outcome is correct
either way. A *live* collision is prevented by pid uniqueness among running processes. This is not
expanded into `P3R-12`, which concerns stale **cache content** verification and **remains separately
`MINOR` and open**.

## 12. Build-failure paths and diagnostics

* `rustc` missing or unusable ⇒ `require_tool` raises **`TEST ENVIRONMENT FAILURE`**, unchanged.
* `rustc` ran and refused the source ⇒ **`FIXTURE BUILD FAILURE: rustc ran and exited …`**, a new
  and distinct message that names the exit status and says the compiler is present. The two
  conditions are no longer confused, and neither hides the underlying error.
* On compiler failure the builder removes **its own** source and staged file, and nothing else.
* Publication failure does not remove the shared final object.
* No cleanup path introduces a second failure mode: every removal is `let _ = fs::remove_file(…)` on
  a name this builder owns.

## 13. Cache-hit path

`if binary.exists() { return binary; }` is unchanged, and remains the existing P3 content-addressed
cache model. The correction creates no new reachable safety or evidence defect on this path, so
**`P3R-12` is not promoted**. This review is not a cache-integrity redesign.

## 14. `atfork_helper` scope — author statement verified

Independently checked, not accepted on report:

* `atfork_helper` has **exactly one caller**, at `tests.rs:2163`, inside
  `a_registered_pthread_atfork_handler_does_not_run_in_the_child`.
* That is a single `#[test]`, which cargo runs once, on one thread.
* It spawns no builder thread of its own; the multithreading it exercises is inside the *traced
  child process*, after the helper is already built.
* Across processes, the pid in its staging name already separates builders.

**No reachable concurrent same-process construction path exists**, so leaving `atfork_helper`
untouched is correct and the correction's scope was not silently widened. The in-place comment
records the condition and what to do if a second caller is ever added.

## 15. Product boundary

| Item | Result |
|---|---|
| P3 product backend semantics | **BYTE-UNCHANGED** (§2) |
| Public `launch()` | **ABSENT** — no `pub fn launch` anywhere in `crates/helm-launch/src/` |
| Process-group sweep | **ABSENT** — no `killpg`, no `kill(-pid)`, no sweep; the only `SETPGID` is the child's own `setpgid(0, 0)` stage, in files byte-identical to `3a9368f` |
| Public API surface | unchanged — `src/lib.rs` byte-identical |
| Unsafe code | unchanged — no `unsafe` was added, removed or moved |
| P4 / P5 | **NOT AUTHORISED** |

## 16. Historical evidence — preserved

`86d798e` was inspected. It records, accurately and without softening:

| Item | Recorded |
|---|---|
| Published head | `3a9368f845452110afc859ed899acae9384c7d9b` |
| `35442641728` (helm-launch Linux) | **attempt 1, `push`, FAILURE** |
| `35442641743` (workspace Linux) | **attempt 1, `push`, FAILURE** |
| `35442641707` (helm-bind) | attempt 1, `push`, SUCCESS |
| Retry / rerun / replacement | **NONE** |
| `P3R-20` | **IMPORTANT / ACCEPTED / MUST FIX** |
| Classification | **TEST / EVIDENCE DEFECT** |
| Product launcher mechanism | **NOT IMPLICATED BY THIS FAILURE** |
| P3 hosted validation | **INCOMPLETE**; `P3R-G1` / `P3R-G2` `GATE_PENDING` |
| P4 / P5 | **NOT AUTHORISED** |
| Trial #3 | frozen `MECHANISM_REJECTED`, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

Both documents state explicitly that a corrected run must come naturally from a **new** SHA and
**does not replace** the historical result. The recorded linker errors — `undefined hidden symbol`
and `cannot open …-cgu.0.rcgu.o` — match the mechanism independently measured in §3.

**NO RETRY. NO RERUN. NO REPLACEMENT.** The first validation of `3a9368f` remains failed forever. No
historical evidence has been rewritten to suggest the first publication passed.

## 17. Local validation

| Command | Result |
|---|---|
| `cargo fmt --check` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **PASS** (host) |
| `cargo clippy … --target x86_64-unknown-linux-gnu -- -D warnings` | **PASS** — this is the run that actually type-checks the corrected, Linux-gated backend harness |
| `cargo test --workspace --locked` | **PASS** — 0 failed |
| `cargo test -p helm-launch --locked` | **PASS** — 0 failed |
| `cargo test -p helm-launch --locked --features test-fault-injection` | **PASS** — 0 failed |
| `cargo build -p helm-launch --release --locked` | **PASS** |
| `python -m unittest discover -s tools/tests` | **PASS** — `Ran 831 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | **PASS** |
| `git diff --check` | **PASS** |

The host is Windows x86_64. The backend suite — including
`concurrent_builders_of_one_fixture_publish_exactly_one_object` — is `cfg`-gated to Linux x86_64 and
**cannot execute here**. It is type-checked by the Linux cross-clippy run above, and its algorithm
was exercised on this host by the standalone replica in §3.

> **`P3R-20` LINUX CONCURRENCY REGRESSION: PENDING CORRECTED PUBLICATION CI.**
> This is a `GATE_PENDING`, not a review failure. The review passes structurally with the real Linux
> execution gate still outstanding.

## 18. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P3R20-R1` | `MINOR` | commit message | n/a | "same bytes whichever builder compiled it" is overstated; `rustc` embeds the unique source pathname in panic locations | **No change required.** Byte reproducibility is not a contract; nothing consumes the fixture's bytes (§5) |
| `P3R20-R2` | `MINOR` | cleanup hygiene | only on an already-panicking path | the staged object is not removed when `hard_link` fails with an error other than `AlreadyExists` | **No change required.** Loud failure path; the artifact is evidence (§7) |
| `P3R20-R3` | `MINOR` | regression probity | yes, but harmless | `compiled >= 2` reads the global nonce, which unrelated concurrent `fixture_binary` calls also increment | **Non-blocking.** The barrier already makes concurrent construction a synchronisation fact; the guard could be tightened to count only this fixture's builders (§9) |
| `P3R20-G1` | `GATE_PENDING` | Linux runtime | — | the concurrency regression has not executed on Linux | Pending corrected publication CI (§17) |
| `P3R-12` | `MINOR` | cache integrity | — | pre-existing, unrelated to this correction | **Remains separately open.** Not promoted (§13) |

No `BLOCKER`. No `IMPORTANT`.

## 19. Recommendation

**`P3R-20` CORRECTION MAY BE PUBLISHED FOR NEW HOSTED VALIDATION ON A NEW SHA.**

The new hosted run is **new validation evidence on a new SHA**. It is **not** a retry, rerun or
replacement of the historical `3a9368f` failure, which stands permanently as failed.

## 20. Next gate

**ONE FAST-FORWARD PUBLICATION OF THE REVIEWED `P3R-20` CORRECTION CHAIN, FOLLOWED BY NEW NATURALLY
TRIGGERED HOSTED P3 CI.**

Not authorised by this review: pushing, rerunning historical CI, starting P4 or P5, adding a public
`launch()`, adding a process-group sweep, or Trial #4.

**Classification: `HELM_LAUNCH_P3_P3R20_REVIEW_PASSED_READY_FOR_CORRECTED_PUBLICATION_CI`.**
