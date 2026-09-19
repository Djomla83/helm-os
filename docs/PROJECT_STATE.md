# Stanje projekta

<a id="helm-launch-p3-accepted"></a>

## HELM-LAUNCH P3 ACCEPTED, 2026-09-19 — hosted Linux validation passed on the third publication; P4 and P5 not authorised

The owner [accepted HELM-LAUNCH P3](DECISIONS.md#helm-launch-p3-accepted) as the third helm-launch
product implementation slice, at head `8a359ee4215b6c803dcc5b527010612dabbd110b`. This supersedes
the "next gate" of the sections below, which are left as written. **P1, P2 and P3 are accepted; P4
and P5 stay not authorised**, and the complete helm-launch 0.1 module is **not yet
product-accepted**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (contract unchanged by this acceptance) |
| **Accepted P3 head** | **`8a359ee4215b6c803dcc5b527010612dabbd110b`** |
| Previous published head | `80ea89b8eef40dc1de68993eac3e140a4925d9b3` |
| Publication | **ONE FAST-FORWARD PUSH**, `80ea89b..8a359ee`, no force, no tags |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED** |
| helm-launch hosted run | **`35467256138`**, run 5, attempt 1, `push` — **SUCCESS** |
| Workspace hosted run | **`35467255952`**, run 37, attempt 1, `push` — **SUCCESS** |
| helm-bind hosted run | not triggered — path filter unmatched |
| LAUNCH-EXEC-01 trial workflow | **NONE RAN** |
| Retry / rerun / replacement | **NONE** |
| Review findings, every P3 review | **0 BLOCKER**, **0 IMPORTANT** |
| **`P3R-20`** / **`P3R-21`** | **CLOSED / VERIFIED CORRECTED** |
| Public `launch()` | **ABSENT** |
| Process-group sweep | **ABSENT** |
| N3 real privilege transition | **UNVALIDATED / NONCLAIM** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |
| Complete helm-launch 0.1 | **NOT YET PRODUCT-ACCEPTED** |
| Next gate | **OWNER DECISION ON WHETHER TO AUTHORISE HELM-LAUNCH P4** |

* **Acceptance rests on executed gates, not on a green job colour.** On the first natural run of
  `8a359ee`: `P3R-20` Linux concurrency regression **PASSED**; `P3R-21` ordinary Linux regression
  **PASSED** (the default lib suite moved from 80 passed / 1 failed to **81 passed / 0 failed**);
  `P3R-21` S6 positive-authority regression **PASSED**; P3 Linux runtime validation **PASSED**
  (**29 backend tests passed, 0 failed**, 1 ignored by design, not `cfg`-skipped); machine-code
  closed-world validation **PASSED** in both debug and release codegen with **0 external, 0
  indirect, 0 unresolved and 0 unsupported** transfers and a live positive control in each;
  injection-confinement validation **PASSED**; `strace` child-window validation **PASSED** under
  real `strace` 6.8; S5 and S6 validations **PASSED**; and the public-API and unsafe-confinement
  boundary proofs **PASSED**. Windows and macOS ran the off-cohort proofs successfully, and no
  Linux backend result is inferred from them.

* **Both failed publications remain permanent evidence.** The first publication of `3a9368f8`
  failed (`35442641728`, `35442641743`, both attempt 1), and the `P3R-20` correction publication of
  `80ea89b8` failed on helm-launch (`35461333887`, attempt 1) while its workspace run succeeded
  (`35461333920`, attempt 1). **Nothing was retried**: no workflow rerun, no job rerun, no
  replacement dispatch, no fix pushed to either published head. The success at `8a359ee` is **new
  evidence after reviewed corrections** and does **not** convert either earlier publication into a
  pass.

* **The accepted P3 boundary is internal only.** P3 owns Linux x86_64 process creation,
  `clone3(CLONE_PIDFD)`, the pidfd-owned direct-child lifecycle, the `execveat` of the exact
  admitted descriptor, the closed post-clone child contract, scoped `unsafe` under
  `src/backend/` alone, the parent `setpgid` attempt and its conservative group-authority fact, the
  fixed pre-exec bound, bounded direct-child `SIGKILL` cleanup and the test-only fault injection.
  It owns **no** public `launch()`, no `LaunchOutcome`, no public process handle, no P4 run
  lifecycle, no run timeout, no `SIGTERM` or grace policy, no process-group sweep, no drain policy,
  no real receipt emission, no sandbox and no Wine integration. A clean exec-status end-of-file
  stays **`Indeterminate`** and no `ExecSucceeded` product state exists.

* **Group-authority semantics are unchanged.** Only a successful parent-side
  `setpgid(child, child)` — the parent's first system call after `clone3` — establishes
  group-sweep authority. Any error establishes nothing; there is no retry and no inference from the
  child's own `setpgid(0, 0)`. Executed-image group leadership and parent-side sweep authority stay
  two separate facts, and P3 issues no group signal at all.

* **Nonblocking findings are carried, not fixed.** `P3R-03` … `P3R-09`, `P3R-12`, `P3R-16` …
  `P3R-19`, `P3R21-M1` and `P3R21-M2` stay **MINOR / open**; `P3R-13` and `P3R-14` stay
  **`BACKLOG_NONBLOCKING`**. `P3R21-M2` now has a concrete hosted manifestation: default-feature
  Linux builds warn that `group_authority_established` is never read, because its only reader is
  the S6 assertion behind `test-fault-injection`. That warning is **nonblocking**, invalidates no
  hosted P3 semantics or evidence, and **no code was changed to remove it in this acceptance**.


<a id="helm-launch-p3-second-publication-failed"></a>

## HELM-LAUNCH P3 corrected chain published, second hosted validation failed, 2026-09-19 — P3R-20 hosted-verified corrected, P3R-21 accepted

The owner [dispositioned the second P3 publication failure](DECISIONS.md#helm-launch-p3-second-publication-failure-disposition).
The `P3R-20` correction chain **is published**, and **`P3R-20` is verified corrected on real hosted
Linux**. The hosted validation the publication existed to obtain **failed again**, on a different
and previously masked defect, **`P3R-21`**. This supersedes the "next gate" of the sections below,
which are left as written. P1 and P2 stay **accepted**; P3 stays **authorised**; P4 and P5 stay
**not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (unchanged by this disposition) |
| **Published head** | **`80ea89b8eef40dc1de68993eac3e140a4925d9b3`** |
| Previous published head | `3a9368f845452110afc859ed899acae9384c7d9b` |
| Publication | **ONE FAST-FORWARD PUSH**, `3a9368f..80ea89b`, no force, no tags |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED** |
| P3R-20 bounded independent review | `80ea89b8eef40dc1de68993eac3e140a4925d9b3` — independence **SATISFIED** |
| helm-launch hosted run | **`35461333887`**, attempt 1, `push` — **FAILURE** |
| Workspace hosted run | **`35461333920`**, attempt 1, `push` — **SUCCESS** |
| helm-bind hosted run | not triggered — path filter unmatched |
| Retry / rerun / replacement | **NONE** |
| **`P3R-20`** | **HOSTED LINUX VERIFIED CORRECTED** |
| **`P3R-21`** | **IMPORTANT, ACCEPTED, MUST FIX** — TEST / EVIDENCE CONTRACT defect |
| Product launcher mechanism | **NOT IMPLICATED** |
| `P3R-G1` / `P3R-G2` | **GATE_PENDING** — hosted validation incomplete |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **MECHANISM_REJECTED**, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **BOUNDED HARNESS CORRECTION OF `P3R-21`, THEN ONE BOUNDED INDEPENDENT REVIEW OF `P3R-21`** |

* **Both hosted publication results are preserved.** The first publication of `3a9368f` failed
  (`35442641728`, `35442641743`, both attempt 1), and the `P3R-20` correction publication of
  `80ea89b` failed on helm-launch (`35461333887`, attempt 1) while its workspace run succeeded
  (`35461333920`, attempt 1). **Nothing was retried**: no workflow rerun, no job rerun, no
  replacement dispatch, no fix pushed to either published head. No `launch-exec-01` workflow
  triggered and **no trial was run**. A corrected run must come naturally from a **new** SHA and
  does not replace either historical result.

* **`P3R-20` is verified corrected on hosted Linux.**
  `backend::tests::concurrent_builders_of_one_fixture_publish_exactly_one_object` executed on real
  Linux x86_64 and passed; every historical fixture-build collision signature is absent; all
  seventeen `report_fixture` consumers passed; and the workspace workflow, which previously failed
  from the same defect, completed end to end including the `tools/tests` machine-proof suite and
  `validate_docs.py`.

* **`P3R-21` is accepted as IMPORTANT and is a TEST / EVIDENCE CONTRACT defect.**
  `the_parent_establishes_group_authority_and_issues_no_group_signal` requires
  `group_authority_established == true` on an ordinary uncoordinated launch. That is not an accepted
  P3 guarantee: only a successful parent-side `setpgid(child, child)` sets the fact, any parent-side
  error leaves it `false` without retry or inference, and the child's own stage-5 `setpgid(0, 0)`
  means the executed image leads its group whichever call ran first. Linux permits the parent call
  to fail with `EACCES` once the child has executed. The race is demonstrated, not assumed: on the
  same head and runner image the same test passed in `35461333920` and failed in `35461333887`, and
  it passed in the first publication run `35442641728`.

* **Hosted validation is incomplete.** Because the default helm-launch test step failed, every later
  step was skipped — fault injection, release build, both machine-code closure proofs, the
  injection-confinement proof, capability admission, the backend and traced-window cases, S5/S6,
  the off-cohort emptiness checks, the boundary suites, the repository-level confinement checks and
  the receipt identities. None of those gates may be inferred from the successful workspace run.

* **The accepted group-authority rule is not amended.** Only a successful parent-side
  `setpgid(child, child)` establishes group-sweep authority. The authorised correction is bounded to
  the status documents and `crates/helm-launch/src/backend/tests.rs`: the ordinary test must assert
  that the executed image leads its own process group and that P3 issues no group signal, without
  requiring either value of `group_authority_established`, and the positive authority fact must be
  asserted deterministically under the existing test-only S6 pre-exec stall. Process-group state and
  group-sweep authority stay two separate facts.

<a id="helm-launch-p3-publication-failed"></a>

## HELM-LAUNCH P3 published, first hosted validation failed, 2026-09-19 — P3R-20 accepted, publication validation incomplete

The owner [dispositioned the first P3 publication failure](DECISIONS.md#helm-launch-p3-publication-failure-disposition).
The complete P3 chain **is published**; the hosted validation it was published to obtain **failed**.
This supersedes the "next gate" of the sections below, which are left as written. P1 and P2 stay
**accepted**; P3 stays **authorised**; P4 and P5 stay **not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (unchanged by this disposition) |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **PUBLISHED CANDIDATE / HOSTED VALIDATION FAILED** |
| Previous remote milestone | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| **Published head** | **`3a9368f845452110afc859ed899acae9384c7d9b`** |
| Publication | **ONE FAST-FORWARD PUSH**, no force, no tags, no publication commit |
| `main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED** |
| P3 bounded independent P3R-15 re-review | `3a9368f845452110afc859ed899acae9384c7d9b` — independence **SATISFIED** |
| Pre-publication classification | **`HELM_LAUNCH_P3_P3R15_REREVIEW_PASSED_READY_FOR_PUBLICATION_CI`** |
| helm-launch hosted run | **`35442641728`**, attempt 1, `push` — **FAILURE** |
| Workspace hosted run | **`35442641743`**, attempt 1, `push` — **FAILURE** |
| helm-bind hosted run | `35442641707`, attempt 1, `push` — SUCCESS |
| Retry / rerun / replacement | **NONE** |
| **P3R-20** | **IMPORTANT, ACCEPTED, MUST FIX** |
| Failure classification | **TEST / EVIDENCE DEFECT** |
| Product launcher mechanism | **NOT IMPLICATED BY THIS FAILURE** |
| P3R-G1 / P3R-G2 | **GATE_PENDING** — hosted validation incomplete |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **BOUNDED HARNESS CORRECTION OF P3R-20, THEN ONE BOUNDED INDEPENDENT REVIEW OF P3R-20** |

* **The publication is exactly one fast-forward push.** The twelve-commit P3 chain was published to
  `docs/helm-launch-architecture`; nothing was amended, rebased, squashed or force-pushed, no
  publication commit was created, no tag was pushed, and `main` is untouched. No merge to `main` is
  authorised.
* **The first hosted runs are preserved as failed.** Both Linux workflows failed on attempt 1 and
  **nothing was retried**: no workflow rerun, no job rerun, no replacement dispatch, no fix pushed
  to the published head. A corrected run must come naturally from a **new** SHA and does not replace
  the historical result. No LAUNCH-EXEC-01 workflow triggered and **no trial was run**.
* **P3R-20 is accepted as IMPORTANT and is a TEST / EVIDENCE DEFECT.** `fixture_binary` in the P3
  test harness stages every fixture build under a name made unique only by `std::process::id()`,
  which **parallel test threads share**. Seventeen call sites request the same content-addressed
  `report` fixture, so concurrent `rustc` invocations collide on the source pathname, the `-o`
  pathname and the intermediate `.rcgu.o` basenames derived from it. The two runs show the same race
  with different timing and **different failing test sets** — 76 passed / 4 failed with
  `undefined hidden symbol`, and 78 passed / 2 failed with `cannot open …-cgu.0.rcgu.o` — which is
  what proves a race rather than a deterministic defect.
* **The product launcher mechanism is not implicated.** The runner had `strace 6.8`, `cc` and
  `rustc 1.95.0`, all verified before any test ran. The backend suite was **not** `cfg`-skipped, and
  real hosted Linux evidence was obtained for the launcher before the abort: the traced child-window
  cases passed from both a single-threaded and a multithreaded allocating parent, along with the
  `pthread_atfork` host condition, the `clone3` UAPI record, the stage vocabulary, the full signal
  mask, the raw `rt_sigaction` layout, `no_new_privs`, the empty environment, the admitted-directory
  identity, execution of the admitted descriptor after its pathname is replaced, `ETXTBSY`,
  `ENOEXEC`, the eight-byte failure record, and the parent establishing group authority while
  issuing **no group signal**. `MeasurementInstabilityDetected` in one failing test is product code
  **correctly refusing** an object the harness was replacing underneath it.
* **Hosted validation is incomplete.** Because the first test step failed, every later step of both
  Linux jobs was skipped: the debug and release-codegen machine-code closure gates, the
  release-library DCE contrast, the injection positive control and release absence proof, the S5 and
  S6 bounded-cleanup cases, the Linux admission suite, the `--nocapture` traced-window record, the
  Linux boundary suites, and the P3R-15 conditional-branch machine proofs — skipped in **both**
  failing runs. This was the **first hosted execution of the P3 Linux backend ever**: the two
  earlier green runs of that workflow predate the backend, and the suite cannot run on the
  Windows developer host.
* **The correction is bounded to the test harness.** Unique per-invocation source and staging
  pathnames (process id **plus** a process-local monotonic nonce), atomic **no-replace** publication
  of the content-addressed fixture, a `Barrier`-synchronised concurrent regression over the real
  builder with real `rustc`, and fixture-build failures described as such rather than as environment
  failures. P3R-12 stays MINOR, open and out of scope, and `atfork_helper` is to be inspected but
  not broadened into. No product backend, manifest, lockfile, workflow, checker, ADR or contract
  change is authorised, and nothing is pushed.

<a id="helm-launch-p3-conditional-branch-dispositioned"></a>

## HELM-LAUNCH P3 conditional-branch finding dispositioned, 2026-09-19 — bounded rereview done, P3R-10 and P3R-11 fixed, P3R-15 accepted, publication still blocked

The owner [dispositioned the bounded rereview findings](DECISIONS.md#helm-launch-p3-conditional-branch-disposition)
for the corrected HELM-LAUNCH P3 candidate. This supersedes the "next gate" of the sections below,
which are left as written. P1 and P2 stay **accepted**; P3 stays **authorised**; P4 and P5 stay
**not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (unchanged by this disposition) |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **AUTHORISED / CORRECTED CANDIDATE / CORRECTION REQUIRED** |
| P3 full independent unsafe review | `4c834415e8e0224b1eb1ce6c6546245cdfda0962` |
| P3 bounded evidence correction | `f144d3004826276a5f2281ffea146c2e99645033` |
| **P3 bounded independent correction re-review** | **`5c577d45f62e2e3adc35a6c04a5ed0d8465a4366`** — independence **SATISFIED** |
| Re-review classification | **`HELM_LAUNCH_P3_CORRECTION_REREVIEW_NEEDS_FIX`** |
| P3R-10 / P3R-11 | **INDEPENDENTLY VERIFIED FIXED** |
| P3R-01 / P3R-02 | **REMAIN FIXED** |
| Backend product semantics | **BYTE-UNCHANGED BY THE EVIDENCE CORRECTION** |
| P3R-15 | **IMPORTANT, ACCEPTED, MUST FIX BEFORE PUBLICATION** |
| P3R-16 | **MINOR, OPEN, NONBLOCKING** — out of scope for this correction |
| Publication | **BLOCKED** pending the bounded correction of P3R-15 |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **BOUNDED CHECKER CORRECTION OF P3R-15, THEN ONE BOUNDED INDEPENDENT RE-REVIEW OF P3R-15** |

* **The bounded rereview is independent.** `5c577d45` was produced by a session that authored
  neither `7b749b8` nor `f144d30`. It is a **bounded** review of the evidence correction and does
  **not** supersede the full independent unsafe review `4c834415`. **Together the two form the P3
  pre-publication independent review record.**
* **Both previously required corrections are verified fixed.** **P3R-10**: sixteen emitted keys and
  sixteen accepted schema keys match exactly, `Report` derives no `Default` so no zero can be left
  behind, `tgid` and `pgid_is_self` are separate facts separately asserted, and the producer
  self-test proves `tgid` against a pid the harness observed itself. The parser was extracted and
  executed outside the repository. **P3R-11**: a differential against the pre-correction checker
  reproduces the original fail-open on `jmpq *%rax` and shows it closed, along with three further
  fail-open holes; 1809 and 450 indirect transfers are now seen where the old parser saw 1771 and 421.
* **P3R-01 and P3R-02 remain fixed**, re-established from fresh builds: the injection proof passes
  with exact `compiler-artifact` selection and every archive member inspected, and the regenerated
  debug, release-codegen and injection child closures each report 0 external, 0 indirect and 0
  unresolved edges. The release proof is **probative, not DCE-only**.
* **P3R-15 is accepted as IMPORTANT and blocks publication.** The machine-code checker can silently
  discard a **conditional branch whose target escapes the current function**. A real emitted
  `jno <function symbol>` exists in this crate's release-codegen assembly, and a synthetic
  `jno memcpy@PLT` in `child_main` passes the current checker reporting zero external, zero indirect
  and zero unresolved edges. Zero such branches occur inside the child closure today, so **no
  current closure result is wrong**, but the gate does not fail closed as required.
* **The required correction is checker-level and test-level only**: an explicit branch vocabulary
  covering `call`/`jmp`, the canonical `Jcc` family and `loop*`; direct targets classified as
  intra-function, traversed edge, or fail-closed; indirect transfers still failing closed with no
  speculative resolver; an unsupported control-flow mnemonic failing closed; and table-driven tests
  including the `jno memcpy@PLT` reproduction. **No prior machine-proof negative may be weakened,
  and P3R-16 is out of scope.**
* **The Linux runtime and `strace` child-window gates stay pending publication CI.** No product
  code, test, workflow, Cargo file, ADR or experiment is changed by this disposition, and nothing
  is pushed.

<a id="helm-launch-p3-independent-review-dispositioned"></a>

## HELM-LAUNCH P3 independent review dispositioned, 2026-09-19 — independence satisfied, two new IMPORTANT findings, publication still blocked

The owner [dispositioned the independent-review findings](DECISIONS.md#helm-launch-p3-independent-review-disposition)
for the corrected HELM-LAUNCH P3 candidate. This supersedes the "next gate" of the sections below,
which are left as written. P1 and P2 stay **accepted**; P3 stays **authorised**; P4 and P5 stay
**not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (unchanged by this disposition) |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **AUTHORISED / CORRECTED CANDIDATE / CORRECTION REQUIRED** |
| P3 bounded correction | `672228b8eeeef95cf72bb07051c0ccdec1ae261f` |
| **P3 independent unsafe review** | **`4c834415e8e0224b1eb1ce6c6546245cdfda0962`** — independence **SATISFIED** |
| Review classification | **`HELM_LAUNCH_P3_INDEPENDENT_UNSAFE_REVIEW_NEEDS_FIX`** |
| P3R-00 | **CLOSED** — the independent review gate artifact exists |
| P3R-01 / P3R-02 | **INDEPENDENTLY VERIFIED FIXED** |
| P3R-10 / P3R-11 | **IMPORTANT, ACCEPTED, MUST FIX BEFORE PUBLICATION** |
| Publication | **BLOCKED** pending the bounded correction |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **BOUNDED CORRECTION OF P3R-10 AND P3R-11, THEN ONE BOUNDED INDEPENDENT CORRECTION RE-REVIEW** |

* **The independent review gate is satisfied.** `4c834415` was produced by a session that authored
  none of the five P3 commits, so it is the independent P3 unsafe-review artifact that **P3R-00**
  recorded as still owed. The author self-review `168fe133` stays **diagnostic evidence only**.
  **P3R-00 is closed.**
* **Both previously required corrections are verified fixed, independently.** **P3R-01**: exact
  `compiler-artifact` selection, fresh build roots, every archive member inspected, and a positive
  control that hits real object code; a release build with `-Cdebug-assertions=on` showed the
  `#[used]` marker present, which isolates the release absence to the `cfg` gate and not to dead-code
  elimination. **P3R-02**: the child entry borrows the `ChildPlan`, and regenerated Linux x86_64
  assembly shows 0 external runtime edges and exactly the 18 contract system calls in contract order,
  in both the debug/test profile and under release codegen. The release proof is **probative, not
  DCE-only**.
* **Two new IMPORTANT findings are accepted.** **P3R-10**: the report fixture emits `pgid_is_self`
  while the parser handles only `tgid`, so the direct-child identity assertion compares `0` to a real
  pid and the load-bearing descriptor-isolation case cannot pass on Linux. **P3R-11**: the
  child-closure checker recognises an indirect `call` but silently discards an indirect `jmp`, so a
  function-escaping control transfer can pass the gate unrecorded, against the required fail-closed
  model.
* **The intended correction is test-level and checker-level.** The fixture reports `tgid` and
  `pgid_is_self` as two explicit, mandatory, separately asserted facts, with a missing or malformed
  required field rejecting the producer report; the checker reasons about **function-escaping control
  transfers**, traversing a resolvable direct tail `jmp` as a call-graph edge and failing closed on an
  indirect `call`, an indirect `jmp` and an unresolvable direct `jmp`. **If either fix required
  changing normal product unsafe or backend semantics, the work stops and returns
  `OWNER DECISION REQUIRED`.**
* **No contract amendment.** ADR-0024 and the productization plan are **not** modified. PID, TGID and
  PGID must not be conflated, and no target resolver for indirect transfers is authorised.
* **P3R-12 stays MINOR and open; P3R-13 and P3R-14 are backlog.** P3R-03 to P3R-09 stay open, were
  re-evaluated by the independent review and none was promoted. The Linux runtime and `strace` gates
  stay **pending publication CI**.
* **Re-review.** If the correction stays strictly test-level and checker-level, one **bounded
  independent correction re-review** follows, and the session that authored `4c834415` may perform it
  provided it did not author the correction. If normal product backend or unsafe semantics changed, a
  **full fresh independent unsafe review** is required again.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P4 AND P5 ARE NOT AUTHORISED.**

<a id="helm-launch-p3-correction-required"></a>

## HELM-LAUNCH P3 author-review dispositioned, 2026-09-18 — correction required, publication blocked, independent review still owed

The owner [dispositioned the author self-review findings](DECISIONS.md#helm-launch-p3-author-review-disposition)
for the HELM-LAUNCH P3 implementation candidate. This supersedes the "next gate" of the sections
below, which are left as written. P1 and P2 stay **accepted**; P3 stays **authorised**; P4 and P5
stay **not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** (unchanged by this disposition) |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **AUTHORISED / IMPLEMENTED CANDIDATE / CORRECTION REQUIRED** |
| P3 authority record | `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60` |
| P3 implementation candidate | `afe8922bebd0c85ead7e58d146b738b84141f096` |
| P3 **author self-review** | `168fe1339dd20bdecc8d5c0f111ef5f185493e9f` — **NOT INDEPENDENT** |
| P3 independent unsafe review | **STILL OWED** |
| Publication | **BLOCKED** pending the bounded correction |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **BOUNDED P3 CORRECTION, THEN ONE GENUINELY FRESH INDEPENDENT UNSAFE REVIEW** |

* **The review that exists is an author self-review.** The agent session that produced the P3
  authority and implementation commits also produced `168fe133`, so the independence precondition was
  not met and the required independent P3 unsafe-review gate is **not satisfied**. The artifact is
  named `HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md` so it cannot be mistaken for that gate. Its
  technical work is accepted as **diagnostic evidence only**.
* **Two IMPORTANT findings are accepted.** **P3R-01**: the committed release injection-absence CI
  proof is defective — it selects no real cargo artifact and would fail the publication run with a
  misleading message. **P3R-02**: the actual test and debug child machine code contains a
  `memcpy@PLT` call and compiler-emitted panic paths.
* **P3R-02 is not resolved by documentation, and no contract is weakened.** ADR-0024 and the
  productization plan are **not amended**. The required target is that normal P3 child machine code
  must call no glibc, libstd, allocator, panic/unwind or other external runtime helper between child
  entry and `execveat` / `exit_group`; internal helm-launch child helpers and the raw syscall shim are
  acceptable if their own closures satisfy the same contract.
* **Two independent gates, neither substituting for the other.** A **machine-code gate** (no
  forbidden userspace runtime helper in the child closure) and a **`strace` gate** (no forbidden
  syscall in the runtime child window). `strace` cannot see `memcpy`, panic helpers or allocator
  calls that issue no syscall.
* **S6 needs no correction.** The deliberately retained parent stdin writer makes the pre-exec stall
  deterministic rather than race-dependent.
* **P3R-03 stays open and no numeric-PID fallback is authorised.** P3R-04 to P3R-08 stay open and are
  out of scope for the bounded correction. The Linux runtime and `strace` gates stay **pending**.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P4 AND P5 ARE NOT AUTHORISED.**

<a id="helm-launch-p3-authorised"></a>

## HELM-LAUNCH P3 authorised, 2026-09-18 — unsafe Linux x86_64 backend and the closed child contract; P4/P5 not authorised

The owner [authorised HELM-LAUNCH P3](DECISIONS.md#helm-launch-p3-authorised), the unsafe Linux
x86_64 process-creation backend and the closed post-clone child contract, as the third product
implementation slice of the accepted [ADR-0024](adr/ADR-0024-launch-authority.md). This supersedes
the "next gate" and the P3 authority rows of the sections below, which are left as written. P1 and
P2 stay **accepted**; P4 and P5 stay **not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **AUTHORISED** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Process creation | **AUTHORISED INTERNALLY IN P3** |
| Process execution attempt | **AUTHORISED INTERNALLY IN P3** |
| Public process-execution API | **NONE** |
| Unsafe | **AUTHORISED ONLY UNDER `crates/helm-launch/src/backend/`** |
| Host privilege acquisition | **NONE** |
| Process-group sweep | **NOT AUTHORISED IN P3** |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |
| LAUNCH-EXEC-01 | **FORMAL TRIAL LINE CLOSED**; Trial #3 remains `MECHANISM_REJECTED` |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **IMPLEMENT P3, THEN ONE FRESH INDEPENDENT UNSAFE REVIEW OF THE P3 CANDIDATE** |

* **The safety transition.** P2 ended at an `AuthorizedLaunch` with **no consumer**. P3 authorises a
  **crate-private** Linux x86_64 consumer that may create **one direct child** and attempt to
  execute through the authorised descriptor. There is still **no public `launch()`**, no public
  process handle and no public pidfd, child pid or raw descriptor: an external caller has no way to
  make the crate create a process.
* **P3 may implement** `src/backend/{mod,spawn,child}.rs` and one private syscall module, one raw
  x86_64 `asm!` syscall shim, parent preparation from a consumed `AuthorizedLaunch`, raw
  `rt_sigprocmask` around `clone3`, `clone3(CLONE_PIDFD)`, parent-side `setpgid(child, child)`,
  pidfd ownership, the complete accepted child stage sequence, the exec-status pipe, crate-private
  spawn and result structures, a bounded non-blocking direct-child reap, direct-child pidfd
  `SIGKILL` only for the fixed pre-exec timeout and bounded test cleanup, an internal
  `launch_minimal` for tests, P3 structural, trace and Linux integration tests, and a non-default
  `test-fault-injection` feature.
* **P3 must not implement** public `launch()`, `LaunchOutcome`, any public execution entry point or
  process handle, receipt emission from a real launch, the P4 observation loop, plan-driven run
  timeouts, the `SIGTERM`/grace lifecycle, general stream drain policy, the process-group `SIGKILL`
  sweep, process-tree containment, Wine, orchestration or sandboxing.
* **Group authority without a sweep.** A successful parent `setpgid(child, child)` is the only
  positive group-authority event, recorded internally for a later P4. P3 issues **no**
  `kill(-child_pid, …)` and no other negative-pid group signal.
* **Fixed internal bounds only.** `SPAWN_CONFIRM_TIMEOUT_MS = 5000` and `POST_KILL_REAP_MS = 5000`
  are the two bounds P3's own contract requires. They are not public run-timeout semantics.
* **Unsafe confinement.** The crate root stays `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`;
  exactly `src/backend/mod.rs` may carry the scoped `#![allow(unsafe_code)]`. Six operations are on
  the closed authorised list; anything beyond it stops and returns `OWNER DECISION REQUIRED`.
* **Test-process execution is expected in P3** and is ordinary product validation, **not**
  LAUNCH-EXEC-01, Trial #3, Trial #4 or D-7 activity. The frozen spike, runner and helper ELFs are
  not used.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P4 AND P5 ARE NOT AUTHORISED.**

<a id="helm-launch-p2-accepted"></a>

## HELM-LAUNCH P2 accepted, 2026-09-18 — capability admission accepted, P3+ not authorised

The owner [accepted HELM-LAUNCH P2](DECISIONS.md#helm-launch-p2-accepted) as the second helm-launch
product implementation slice, after an independent review with **0 BLOCKER and 0 IMPORTANT** and a
green publication CI gate whose Linux jobs **actually executed** the cohort-gated admission tests.
This supersedes the "next gate" and the P2 status rows of the sections below, which are left as
written. P1 stays **accepted**; P3, P4 and P5 stay **not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| P2 accepted authority record | `a3a8d999a6bfa59ce27b1515525e6a7578dad7fe` |
| P2 accepted implementation | `c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` |
| P2 independent review | `94ce8dd34cd6694ea3b528a9dad95d2a71b4ad70` (0 BLOCKER, 0 IMPORTANT) |
| P2 Linux runtime CI | **PASSED** — `tests/linux_admission.rs` 28 passed, `authority.rs` Linux-specific unit tests 13 passed, cohort doctests executed and passed |
| Portable model | **PASSED LINUX / WINDOWS / MACOS** |
| P2 authority API off the cohort | **ABSENT** |
| HELM-LAUNCH P3 / P4 / P5 | **NOT AUTHORISED** |
| Process creation | **NONE** |
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |
| helm-launch complete 0.1 module | **NOT YET PRODUCT-ACCEPTED** |
| LAUNCH-EXEC-01 | **FORMAL TRIAL LINE CLOSED**; Trial #3 remains `MECHANISM_REJECTED` |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **OWNER DECISION ON HELM-LAUNCH P3 AUTHORITY — UNSAFE BACKEND AND CHILD CONTRACT** |

* **Current helm-launch capability.** Portable plan, receipt and lifecycle model **plus** safe
  Linux x86_64 capability admission **plus** single-use authorisation composition. Concretely:
  `parse_launch_plan` and the portable receipt model everywhere; and on the Linux x86_64 cohort
  only, `admit_executable`, `admit_working_directory`, `authorize`, `ExecutableCapability`,
  `WorkingDirectoryCapability` and `AuthorizedLaunch`, with pre-execution executable measurement,
  ELF64 x86_64 cohort admission and detected-instability refusal.
* **`AuthorizedLaunch` has no consumer that can create a process.** `launch()` does not exist on any
  platform, so the edge from an authorisation to a process does not exist.
* **P2 acceptance authorises nothing further.** It does **not** authorise process creation and does
  **not** activate the ADR-0024 section E `unsafe` exception, which stays reserved and inactive. No
  `clone3`, `execveat`, `backend/`, syscall shim, inline assembly, pidfd, signal manipulation or
  child creation is authorised.
* **CI success is not execution evidence.** The P2 admission tests execute no admitted program.
* **Non-claims carried forward.** Measurement is a pre-execution measurement of the pinned object,
  never executed-body identity. Detected instability means only that the protocol detected
  instability; not detecting it proves nothing. `NoClaimContradicted` is not permission, and
  `asserted_context` digests stay inert caller assertions that `authorize` does not inspect.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P3+ IS NOT AUTHORISED.**

<a id="helm-launch-p2-authorised"></a>

## HELM-LAUNCH P2 authorised, 2026-09-18 — capability admission only, P3+ not authorised

The owner [authorised HELM-LAUNCH P2](DECISIONS.md#helm-launch-p2-authorised), the bounded
capability-admission and authorisation-composition slice of the accepted
[productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md). This supersedes the "next
gate" and the P2 authority rows of the sections below, which are left as written. P1 stays
**accepted**; P3, P4 and P5 stay **not authorised**.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** |
| HELM-LAUNCH P1 | **ACCEPTED** (`cd5db27964dc10593bd8856d2d331b907a4b608e`) |
| HELM-LAUNCH P2 | **AUTHORISED** |
| HELM-LAUNCH P3 / P4 / P5 | **NOT AUTHORISED** |
| Current helm-launch capability before P2 implementation | **PORTABLE MODEL ONLY** |
| P2 authorised capability | **SAFE LINUX X86_64 CAPABILITY ADMISSION AND COMPOSITION** |
| Process creation | **NONE** |
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |
| helm-launch complete module | **NOT YET PRODUCT-ACCEPTED** |
| LAUNCH-EXEC-01 | **FORMAL TRIAL LINE CLOSED**; Trial #3 remains `MECHANISM_REJECTED` |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **IMPLEMENT P2, THEN ONE FRESH INDEPENDENT REVIEW OF THE P2 CANDIDATE** |

* **P2 boundary.** `ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`, the
  admission error and authorisation refusal vocabularies, `admit_executable`,
  `admit_working_directory`, `authorize`, safe Linux x86_64 descriptor inspection and positional
  reads, pre-execution executable measurement, ELF cohort admission, and the admission, refusal,
  capability and type-boundary tests. Nothing else.
* **Not in P2.** `LaunchOutcome`, `launch()`, process creation, process execution, `clone3`,
  `execveat`, pidfd acquisition or signalling, `waitid`, `pidfd_send_signal`, `close_range`,
  `fchdir` execution, signal or process-group manipulation, the `PR_SET_NO_NEW_PRIVS` call, child
  pipes, polling lifecycle, timeout execution, `backend/`, a syscall shim, inline assembly, libc
  calls and `unsafe` code.
* **P2 may perform read-only I/O through caller-supplied descriptors.** Executable measurement may
  update atime and populate the page cache; `O_NOATIME` is not used.
* **P2 creates authority-bearing in-process values, but no function can execute them.**
  `AuthorizedLaunch` exists and is inert, because `launch()` does not exist. **P2 does not
  authorise process creation.**
* **Platform.** The P2 APIs exist only under `cfg(all(target_os = "linux", target_arch = "x86_64"))`
  and must not exist in the public API elsewhere; portable P1 stays available on Linux x86_64, other
  Linux architectures, Windows and macOS.
* **Non-claims carried forward.** Measurement is a pre-execution measurement of the pinned object,
  never executed-body identity. Detected instability means only that the protocol detected
  instability; not detecting it proves nothing. `NoClaimContradicted` is not permission, and
  `asserted_context` digests stay inert caller assertions that `authorize` must not inspect.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P3+ IS NOT AUTHORISED.**

<a id="helm-launch-p1-accepted"></a>

## HELM-LAUNCH P1 accepted, 2026-09-17 — portable model only, P2+ not authorised

The owner [accepted HELM-LAUNCH P1](DECISIONS.md#helm-launch-p1-accepted) as the first helm-launch
product implementation slice, after the
[independent review](implementation/HELM-LAUNCH-P1-INDEPENDENT-REVIEW.md) (0 BLOCKER, 0 IMPORTANT)
and green publication CI. This supersedes the "next gate" and the `crates/helm-launch` state of the
2026-09-17 authorisation section below, which is left as written.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED** |
| HELM-LAUNCH P1 | **PRODUCT SLICE ACCEPTED** |
| Accepted implementation | `427b1af092db29c619b7f7a4c0d40b72efaacc65` |
| Accepted correction | `d3914ab95fb253abd8ac10462a4e2de1cb2185de` |
| Independent review | `e32b2e1768b0b0d11f9b02115b107b6a40f60ecb` |
| Cross-platform CI | **PASSED LINUX / WINDOWS / MACOS** (runs `35244879485`, `35244879475`, `35244879552`) |
| helm-launch complete module | **NOT YET PRODUCT-ACCEPTED** |
| P2+ | **NOT AUTHORISED** |
| Current helm-launch capability | **PORTABLE MODEL ONLY** |
| Process execution | **NONE** |
| LAUNCH-EXEC-01 | **FORMAL TRIAL LINE CLOSED**; Trial #3 remains `MECHANISM_REJECTED` |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **OWNER DECISION ON HELM-LAUNCH P2 AUTHORITY** |

* **Accepted P1 contents.** Portable launch-plan parsing and validation, `ValidatedLaunchPlan`,
  `Digest`, the portable receipt and fact model, deterministic receipt serialisation, the plan
  error vocabulary, pure fd-layout planning, pure lifecycle state modelling, portable tests and the
  P1 CI plumbing. **Unsafe: none. Host privilege: none. Experiment execution: none.** No executable
  or working-directory capability and no `launch()` exist; accepted P1 cannot create a process.
* **Review findings.** P1-DOC-01 is resolved by syncing the productization plan's T40 wording
  (`EndNotObserved` is latched). P1-DOC-02 and P1-TEST-01 remain MINOR, open and nonblocking; the
  other findings are backlog.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P2+ IS NOT AUTHORISED.**

<a id="adr-0024-accepted-helm-launch-p1-authorised"></a>

## ADR-0024 accepted, 2026-09-17 — HELM-LAUNCH P1 authorised, P2+ not authorised

The owner [accepted the revised ADR-0024 and authorised HELM-LAUNCH P1 only](DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised).
[ADR-0024](adr/ADR-0024-launch-authority.md) is Accepted as revised, read together with the
owner-reviewed [productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md). This
supersedes the "next gate" and the ADR-0024 status of the 2026-09-16 section below, which is left as
written.

| Item | State |
|---|---|
| ADR-0024 | **ACCEPTED 2026-09-17** (approver Djomla83) |
| HELM-LAUNCH PRODUCT CONTRACT | **ACCEPTED** (helm-launch 0.1 architecture only; evidence classes unchanged) |
| LAUNCH-EXEC-01 | **FORMAL TRIAL LINE CLOSED** |
| Trial #3 | **COMPLETED / `MECHANISM_REJECTED` / immutable history** — D-7 consumed, valid count one, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |
| helm-launch implementation — P1 | **AUTHORISED** — crate skeleton and portable, pure model |
| helm-launch implementation — P2+ | **NOT AUTHORISED** |
| crates/helm-launch | **NOT YET CREATED** — may be created only within P1 scope |
| Next gate | **IMPLEMENT HELM-LAUNCH P1 — PORTABLE MODEL ONLY** |

* **P1 boundary.** Validated plan model, deterministic parsing and validation, the `Digest` value
  type, the portable receipt model, closed non-verdict enums, the public error vocabulary, pure
  fd-layout planning, a pure lifecycle state model, the serialisation the contract requires,
  compile-fail/type-boundary tests, portable unit and property tests, and the crate README.
  **Process execution: none. Unsafe: none. Host privilege: none. Experiment execution: none.**
  No `clone3`, `execveat`, syscall shim, Linux backend, pidfd, `waitid`, `close_range`, `fchdir`,
  signal or process-group manipulation, `PR_SET_NO_NEW_PRIVS` call, descriptor admission, ELF or
  measurement I/O, `launch()`, or reuse of the experimental runner.
* **Trial history unchanged.** The X2c engineering disposition stays `PRODUCT_MECHANISM:
  MECHANISM_NOT_IMPLICATED`, and Trial #3 is not rewritten as `MECHANISM_ACCEPTED`.
* **Freeze test conflict resolved.** The open validation conflict recorded on 2026-09-16 is closed by
  the accepted test correction `03285d9d13f53c2d97d78bd4f50552c201941c8f`: the 17 frozen experiment
  sources bind to the current tree, and the 3 Trial #3 definition inputs bind to their historical
  bytes at freeze commit `bebd8a5`. `SOURCE-HASHES.json` and the Trial #3 freeze are unchanged.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED. HELM-LAUNCH P2+ IS NOT AUTHORISED.**

<a id="helm-launch-productization-plan-owner-review"></a>

## helm-launch productization plan owner-reviewed, 2026-09-16 — ADR-0024 revision prepared, still PROPOSED

The owner [reviewed the productization plan](DECISIONS.md#helm-launch-productization-plan-owner-review)
and passed it with bounded amendments. The amendments are applied in the
[plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md#19-owner-review-amendments-2026-09-16), and
[ADR-0024](adr/ADR-0024-launch-authority.md) is revised in place to the current intended product
contract. This supersedes the "next gate" of the X2c section below, which is left as written.

| Item | State |
|---|---|
| HELM-LAUNCH PRODUCTIZATION PLAN | **OWNER-REVIEWED** (`HELM_LAUNCH_PRODUCTIZATION_PLAN_OWNER_REVIEW_PASSED_WITH_BOUNDED_AMENDMENTS`) |
| Plan proposal commit | `930ec14b940da9b136c7d2ad024b441d47ceba6c` |
| Q1 / Q2 / Q3 | **APPROVED** / **APPROVED WITH EXACT NARROWING** / **APPROVED WITH A GROUP-AUTHORITY GUARD** |
| ADR-0024 | **REVISION PREPARED / STILL PROPOSED** |
| crates/helm-launch | **NOT CREATED** |
| Implementation | **NOT AUTHORISED** |
| Trial #3 frozen result | **`MECHANISM_REJECTED`**, 70 PASS / 1 FAIL / 1 BLOCKED |
| LAUNCH-EXEC-01 formal trial line | **CLOSED** |
| Trial #4 | **NOT AUTHORISED** |
| Next gate | **OWNER REVIEW OF REVISED ADR-0024** |

* **The contract as proposed.** Execution authority is an already-open, admitted executable
  descriptor; a plan, specification, observation or binding report grants none. The target is one
  regular ELF64 x86_64 object on Linux x86_64. The mechanism is `clone3(CLONE_PIDFD)` followed by
  `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`, with no procfs fallback. Clean exec-status EOF
  alone is not positive exec proof. Lifecycle is direct-child only, with one guarded `SIGKILL`
  group sweep before the reap as best-effort cleanup, never containment. One `cfg`-gated backend
  module holds all `unsafe`, and the crate has zero HELM crate dependencies.
* **Approved refinements are obligations, not evidence.** They are to be validated by ordinary
  product tests. LAUNCH-EXEC-01 did not validate them, and no formal trial is authorised for them.
* **Frozen ADR bytes stay addressable.** The Trial #3 manifest bound the pre-revision ADR-0024 bytes
  (SHA-256 `c1f3cce88438439aab6632a9c56450ae98d1c64adb31b13c742e2febb1476351`), which remain at
  freeze commit `bebd8a5`. No freeze manifest, experiment source, LAUNCH-EXEC-01 definition or
  evidence file changed.
* **Open validation conflict — owner decision needed.** `Trial3Freeze` in
  `tools/tests/test_launch_exec_01_trial3.py` compares the **working-tree** ADR-0024 with that frozen
  hash. With the ordered in-place revision, `test_the_exact_frozen_bytes_verify` and
  `test_drift_fails_on_a_disposable_copy` report definition drift for exactly that path: 783 tests,
  2 failures, 74 skipped. Changing the test lies outside this revision's file scope, so reconciling
  the binding with the revision is an owner decision. `--verify-freeze` checks source hashes only and
  still passes.

**None of this is acceptance.** ADR-0024 is **PROPOSED**, `crates/helm-launch` is **NOT CREATED**,
implementation is **NOT AUTHORISED**, **TRIAL #3 MUST NOT BE RERUN**, and **NO TRIAL #4 IS
AUTHORISED**.

<a id="launch-exec-01-trial-003-x2c-disposition"></a>

## Trial #3 X2c postmortem accepted, 2026-09-15 — formal LAUNCH-EXEC-01 trial line CLOSED

The owner [accepted the X2c postmortem disposition](DECISIONS.md#trial-003-x2c-postmortem-owner-disposition).
The [X2c postmortem](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-X2C-POSTMORTEM.md) is commit
`585caf924454faf5fb92e1e1dbd54675a1b7c0bf`. This supersedes the "next gate" of the Trial #3
completion section below, which is left as written. No frozen byte, evidence file, review or
postmortem changed.

| Item | State |
|---|---|
| Trial #3 | **COMPLETED** |
| Frozen result | **`MECHANISM_REJECTED`**, 70 PASS / 1 FAIL / 1 BLOCKED (sole FAIL `X2c`, sole BLOCKED `N3`) |
| D-7 | **CONSUMED** |
| Valid Trial #3 count | **ONE** |
| X2c postmortem | **ACCEPTED** (`TRIAL_3_X2C_POSTMORTEM_ACCEPTED`) |
| Engineering disposition | **PRODUCT MECHANISM NOT IMPLICATED BY X2c** |
| Trial #4 | **NOT AUTHORISED** |
| Current formal trial line | **CLOSED** |
| Next gate | **HELM-LAUNCH PRODUCTIZATION PLAN** |
| ADR-0024 | **PROPOSED** |
| helm-launch product crate | **NOT YET CREATED** |

* **Classification.** Product mechanism `MECHANISM_NOT_IMPLICATED`; frozen expectation
  `EXPECTATION_EVIDENCE_MISMATCH`; observability `OBSERVABILITY_INCOMPLETE`; harness
  `STATIC_POSABILITY_DEFECT` + `FIXTURE_DEFECT`.
* **Not an acceptance.** The formal result stays `MECHANISM_REJECTED`. X2c no longer blocks
  productization planning of the supported ELF launch path, but Trial #3 is not reclassified as
  `MECHANISM_ACCEPTED`.
* **S4 policy unchanged.** Clean exec-status EOF alone is not positive exec proof.
* **Scripts.** `#!` script execution stays outside the 0.1 supported product path.
* **N3.** Still BLOCKED on `unprivileged_runner`; no real privilege transition is validated.
* **Backlog, not implemented.** R3-M1 (future evidence contract: integrity digests must commit to
  reproducible published bytes) and a report-producer / static-posability audit of
  report-declaring plans.

**TRIAL #3 MUST NOT BE RERUN. NO TRIAL #4 IS AUTHORISED.**

**Next gate:** one bounded helm-launch productization plan — API boundaries, Linux backend
boundary, safe/unsafe split, lifecycle and result model, integration with helm-bind, helm-observe
and helm-evidence, supported versus unsupported 0.1 behaviour, product test strategy, and migration
from experiment code without copying the harness wholesale. No `crates/helm-launch` is created and
ADR-0024 is not accepted.

<a id="launch-exec-01-trial-003-completed"></a>

## Trial #3 completed and independently reviewed, 2026-09-14 — `MECHANISM_REJECTED`

GitHub Actions run `34883316368`, job `104107679380`, performed the one authorised execution of
freeze `bebd8a5f83d4d0daebe9b068050cb5436289c75e`. The run details:

* workflow `357912663`, `workflow_dispatch` on `main` at `501a7fa`;
* run number 1, attempt 1;
* the D-7 dispatcher blob `64ce3d433a47eaae3eb28b8bb28f7300a330d762`.

Trial #3 executed exactly once. This supersedes the "not yet dispatchable" state of the D-7 section
below, which is left as written.

* **Evidence preserved.** The five artifact files are preserved byte-for-byte under
  [`docs/experiments/evidence/LAUNCH-EXEC-01-TRIAL-003-2026-09-14/`](experiments/evidence/LAUNCH-EXEC-01-TRIAL-003-2026-09-14/)
  in local commit `11bddc5ca8a56f5b71f817f6aa28c01a7f26c93b`. The source is artifact `10364580293`;
  its ZIP SHA-256 `92ee47033513761ec5947ae10e3175bb78bd0a444799cc4fb3c3f5fbb3bcaeee` is transport
  only.
* **Independently reviewed.** The [independent result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-RESULT-REVIEW.md)
  is committed locally with this section. Offline replay with the frozen `bebd8a5` checker
  reproduces the aggregate, the counts and all 72 case statuses and reasons exactly.
* **Frozen result: 70 PASS / 1 FAIL / 1 BLOCKED, `MECHANISM_REJECTED`.**
  * Sole FAIL: `X2c` (`ExecStatusIndeterminate`, not the frozen prediction
    `interpreter_ran_with_devfd`).
  * Sole BLOCKED: `N3` (`unprivileged_runner`, never posed).
* **Journal.** The first durable `case_pose_started` is `E1` at `n = 4`, and the journal ends in one
  valid `trial_end` at `n = 218`.
* **No diagnosis.** Nothing here diagnoses why X2c failed.

**TRIAL #3 D-7 IS CONSUMED**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ONE**

**TRIAL #3 MUST NOT BE RERUN**

**NO TRIAL #4 IS AUTHORISED**

**Next gate:** one bounded X2c postmortem and owner decision.

**Trial #2:** completed, D-7 consumed, valid count one, `MECHANISM_REJECTED`, never to be rerun.
**Trial #3:** completed, D-7 consumed, valid count one, `MECHANISM_REJECTED`, never to be rerun.

<a id="launch-exec-01-trial-003-d7-authorised"></a>

## Trial #3 D-7 authorised, 2026-09-14 — LAUNCH-EXEC-01 Trial #3 still NOT_RUN and not yet dispatchable

The owner granted [D-7 for Trial #3](DECISIONS.md#d-7-authorised-trial-003) prospectively: exactly
one valid Trial #3 execution, bound to the exact freeze, manifest, freeze review, accepted Build 8
evidence and reviewed dispatcher blob below. This replaces the "no D-7 and no dispatcher" state of
the freeze section below. That section is left as written.

* **Freeze published.** Freeze `bebd8a5f83d4d0daebe9b068050cb5436289c75e`, manifest blob
  `8cd290b573408510f8c16cd8dafe676354140a38`, SHA-256
  `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70`. The
  [independent freeze review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md)
  `69f13a7` passed.
* **Build 8 evidence published and accepted.** Formal run `34754901079`, job `103717520698`,
  attempt 1, `BUILD_8_FORMAL_COMPILE_ONLY_SUCCESS`, with durable job-log SHA-256
  `6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15`. It was accepted at `5a6be59`
  by the [evidence-correction re-review](implementation/HELM-LAUNCH-EXEC-01-BUILD-008-EVIDENCE-CORRECTION-REREVIEW.md).
  It is pretrial compile evidence only.
* **Dispatcher reviewed and published on the milestone branch only.** Candidate `f197386` passed
  its [independent review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-DISPATCHER-REVIEW.md)
  `4f19897`. Both were pushed to `docs/helm-launch-architecture` in one fast-forward,
  `5a6be59..4f19897`.
  * Path: `.github/workflows/launch-exec-01-trial-003.yml`.
  * Blob: `64ce3d433a47eaae3eb28b8bb28f7300a330d762`.
  * SHA-256: `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8`.
  * After publication GitHub showed zero Trial #3 runs and no Trial #3 workflow registered.
* **Dispatcher absent from `main`.** `main` is still `d8a6887d508be3b4bb5707c40e6433cee21411b5`.
* **D-7 authorised and not consumed.** It is consumed by the first fsynced `case_pose_started`.
  While it is live:
  * the dispatcher must not be edited, renamed, copied, recreated or replaced;
  * authority-bearing history must not be force-pushed;
  * the milestone branch must not be deleted, because the dispatcher's identity gates need
    `bebd8a5`, `69f13a7`, `5a6be59`, `f197386` and `4f19897` to stay reachable.
* **No frozen byte changed.** `SOURCE-HASHES.json` keeps `status: NOT_RUN` and
  `d7_execution_authorised: false` as freeze-time fields.

**Human dispatch is NOT yet permitted.** The next gate is:

1. byte-identical publication of the dispatcher to `main`;
2. confirmation that GitHub registers the workflow at that path;
3. proof that it still has zero runs and an unused run number 1.

**Trial #2:** D-7 consumed, valid count one, `MECHANISM_REJECTED`, never to be rerun.
**Trial #3:** D-7 authorised and not consumed, NOT_RUN, valid count zero, not yet dispatchable.

<a id="launch-exec-01-trial-003-freeze"></a>

## Trial #3 freeze cut locally, 2026-09-13 — FROZEN, NOT_RUN, NOT AUTHORISED

After the [final RR-I1 micro review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-RRI1-MICRO-REVIEW.md)
closed the correction review, the LAUNCH-EXEC-01 **Trial #3 freeze** (`trial-003`) was cut locally
and is not pushed. It covers the reviewed correction sequence `7f98d4d`, `53ad8bf` and `f1e7eea`,
reviewed at `ee8cd91`, `db45336` and `030d88a`.

* **Manifest.** `SOURCE-HASHES.json` is now the Trial #3 manifest. It binds 17 source hashes and
  3 definition hashes, and has SHA-256 `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70`.
  The Trial #2 manifest stays addressable at `ba41a3f`.
* **Identity.** The runner's trial identifier is `trial-003`.
* **Freeze step.** The freeze step also records three wording corrections in
  [definition section 10](experiments/LAUNCH-EXEC-01-DEFINITION.md#10-trial-3-freeze--frozen-not-authorised-not-run):
  R-M2, RR-M1 and RR-M3.
* **Superseded record.** The NOT_FROZEN
  [correction record](experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json) is superseded
  provenance and never freeze authority.
* **Shape.** 72 cases (54 mandatory, 11 conditional, 7 recorded), traced `E1 E7 F4 F7 M1 M2 M3 M4`,
  72 handlers, 72 posable, 0 unposable.
* **Build.** Build 7 does not bind the changed C sources: **Build 8 is required** and does not
  exist yet.

**Trial #2:** completed, D-7 consumed, valid count one, `MECHANISM_REJECTED`, never to be rerun.
**Trial #3:** frozen locally, NOT_RUN, valid count zero, no D-7 and no dispatcher. The freeze is
not published. One independent Trial #3 freeze review is required before publication and Build 8.

<a id="launch-exec-01-trial-003-correction-candidate"></a>

## Trial #3 correction candidate, 2026-09-12 — NOT FROZEN, NOT AUTHORISED, NOT RUN

On the owner's instruction after the [postmortem decisions](DECISIONS.md#trial-002-postmortem-decisions)
and the [postmortem diagnostics](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-POSTMORTEM-DIAGNOSTICS.md),
a bounded prospective correction is implemented locally and is not pushed. It is recorded in
[definition section 10](experiments/LAUNCH-EXEC-01-DEFINITION.md#10-trial-3-correction-candidate--not-frozen-not-authorised-not-run)
and in the NOT_FROZEN record
[`TRIAL-3-CORRECTION-CANDIDATE.json`](experiments/launch-exec-01/TRIAL-3-CORRECTION-CANDIDATE.json).

Its scope is X2b, X2c, X4, T1, S4, M2, E4, E6, E6c, O6, O7 and R3, plus liveness revalidation
support for P1, P2 and P4. N3 is unchanged. `SOURCE-HASHES.json` remains the Trial #2 freeze byte
for byte. Trial #2's evidence, statuses and `MECHANISM_REJECTED` aggregate are unchanged, and its
replay with the `ba41a3f` checker still reproduces them. C sources changed, so Build 7 no longer
binds: **BUILD_8_REQUIRED_FOR_FUTURE_FREEZE**.

The candidate needs one bounded independent correction review, limited to these findings, before
any freeze. **No Trial #3 freeze, Trial #3 D-7, Trial #3 dispatcher or Trial #3 execution exists.**

<a id="launch-exec-01-trial-002-postmortem-decisions"></a>

## Trial #2 postmortem decisions, 2026-09-12 — bounded diagnostics only

The owner accepted the [Trial #2 result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md)
and recorded the prospective [postmortem decisions](DECISIONS.md#trial-002-postmortem-decisions).
Trial #2 remains immutable at 59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED and
`MECHANISM_REJECTED`; D-7 remains consumed and the valid Trial #2 count remains one.

Future correction scope is bounded to the demonstrated Trial #2 issues and the non-probative
`P1`/`P2`/`P4` liveness evidence. Clean exec-status EOF remains insufficient proof of exec; E6c
remains recorded; `P1`/`P2`/`P4` require valid liveness re-observation; N3 privileged testing is
deferred. One pre-correction diagnostic pass is authorised only for E6/E6c marker construction and
R3 `waitid`/`CLD_DUMPED` semantics. It is not Trial #3 and uses no D-7.

**No correction implementation, Trial #3 freeze, Trial #3 execution or Trial #3 D-7 is
authorised.**

<a id="launch-exec-01-trial-002-completed"></a>

## Trial #2 completed and independently reviewed, 2026-09-11 — `MECHANISM_REJECTED`

GitHub Actions run `34640280964`, job `103397999925`, performed the one authorised execution of
freeze `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`. The complete journal has one valid
`trial_end`, 72 entered and completed cases, and the frozen result **59 PASS / 6 FAIL / 6 INVALID /
1 BLOCKED**. Offline replay with the frozen checker reproduces the aggregate
**`MECHANISM_REJECTED`** and every case status and reason exactly; the
[independent result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md) keeps
that frozen result separate from its post-trial engineering diagnoses.

The five published evidence files are preserved byte-for-byte under
[`docs/experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/`](experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/)
in commit `8a9dc7721751d2c94a91f109959b8d6416184843`. The first durable
`case_pose_started` is `E1` at journal record `n = 4`; therefore:

**TRIAL #2 D-7 IS CONSUMED**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE**

**TRIAL #2 MUST NOT BE RERUN**

**NO TRIAL #3 IS AUTHORISED**

<a id="launch-exec-01-trial-002-d7-authorised"></a>

## Trial #2 D-7 authorised, 2026-09-11 — LAUNCH-EXEC-01 Trial #2 still NOT_RUN

The [final bounded independent review of freeze `ba41a3f`](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-BA41A3F-FINAL-REVIEW.md)
(`5da397c`) returned **`TRIAL_2_READY_FOR_NEW_D7_DECISION`**. The owner then granted
[D-7 for Trial #2](DECISIONS.md#d-7-authorised-trial-002): exactly one valid Trial #2 execution,
bound to:

* freeze `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`;
* `SOURCE-HASHES.json` blob `6f000fac9a48625d3c9def18e16ae7ce1b61efa8`, SHA-256
  `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94`;
* the final review `5da397c6a79aea7c9a626789480903d50df0b7b4`.

D-7 is consumed when the first durable `case_pose_started` record is fsynced. An abort after that
boundary leaves it consumed, and there is no rerun or resume. A rejection before the boundary poses
no case and manufactures no trial result. No frozen byte changed.

**Authorised is not executed.** Trial #2 is NOT_RUN and the valid trial count is ZERO. It runs
only through a new one-shot `workflow_dispatch` dispatcher, and that dispatcher is published only
after one bounded independent infrastructure review.

<a id="launch-exec-01-trial-002-ab7-correction-freeze"></a>

## Trial #2 AB7 correction frozen, 2026-09-11 — LAUNCH-EXEC-01 still NOT_RUN

The [final bounded independent review of freeze `ab74356`](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-AB74356-FINAL-REVIEW.md)
(`898a31c`) returned **`TRIAL_2_NEEDS_OWNER_DECISION`**. It found:
* AB7-B1: O7's fixture could never be established, because the launcher's group sweep killed the
  descendant before the harness's post-launch liveness read;
* AB7-I1: S2/S7 scored a correct `ExecFailed:CHDIR:EACCES` INVALID whenever stdout was not
  drained to its end;
* AB7-M1: a report sentinel beside a report that did not parse scored INVALID instead of FAIL;
* AB7-M2: O6's frozen `CompleteAtEof` FAIL could only ever score INVALID.

That review is preserved unamended. The owner accepted all four and decided the correction
without changing C. The harness now arms O6's and O7's fixture signal before the launcher is
spawned, with no `--setsid` and the group sweep untouched. S2/S7's explicit CHDIR status is
authoritative, and a report sentinel is decisive. The correction (`1db4347`) and
[definition section 9.7](experiments/LAUNCH-EXEC-01-DEFINITION.md#97-ab7-correction--the-pre-armed-fixture-signal-and-the-authoritative-chdir-status)
record it, and a new freeze in `SOURCE-HASHES.json` supersedes `ab74356`. No case membership,
class, prediction or safe set moved. No C or helper source changed, so Build 7 still binds them.

**Trial #2 is NOT_RUN, D-7 is NOT granted, and the valid trial count is ZERO.** Before any new D-7
decision the freeze needs one bounded independent review, limited to:
* AB7-B1, AB7-I1, AB7-M1 and AB7-M2;
* the fixture signal's descriptor hygiene;
* the freeze's integrity;
* Build 7's binding.

<a id="launch-exec-01-trial-002-final-classification-freeze"></a>

## Trial #2 final classification freeze, 2026-09-11 — LAUNCH-EXEC-01 still NOT_RUN

The [bounded independent review of freeze `f417984`](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-F417-BOUNDED-REVIEW.md)
(`7f97100`) returned **`TRIAL_2_NEEDS_PRETRIAL_FIXES`**. It found four classification defects:
* a first-trial mismatch in O8 or S7 could PASS;
* a posed short O8 repetition scored INVALID where its row says FAIL;
* an S2/S7 image that ran scored INVALID where its rows say FAIL;
* F7's own EINVAL scored INVALID.

That review is preserved unamended. The owner accepted its findings and decided that the section 3
expectations win.

The classification correction (`f9bbf39`) scores every launcher invocation by one rule and reduces
repeated cases FAIL > INVALID > PASS. The owner's O7 decision (`82b8746`) runs O7 on the unchanged
`helper_fork` retained-writer fixture: its stderr capture failure is the receipt's
`WriterRetainedAfterChildExit` beside `Exited:42`. [Definition section 9.6](experiments/LAUNCH-EXEC-01-DEFINITION.md#96-final-classification-semantics--posed-versus-result)
preregisters both, and a new freeze follows in `SOURCE-HASHES.json`, superseding `f417984`. No case
membership, class, prediction or safe set moved. No C or helper source changed, so Build 7 still
binds them.

**Trial #2 is NOT_RUN, D-7 is NOT granted, and the valid trial count is ZERO.** Before any new D-7
decision the freeze needs one final bounded independent review, limited to:
* the classification delta;
* O7;
* the freeze's integrity;
* Build 7's binding.

<a id="launch-exec-01-trial-002-delta-correction"></a>

## Trial #2 delta correction frozen, 2026-09-11 — LAUNCH-EXEC-01 still NOT_RUN

The [bounded independent delta review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-DELTA-REVIEW.md)
of the corrected Trial #2 candidate `e4f49f2` returned **`TRIAL_2_NEEDS_BOUNDED_CORRECTION`**: parent
states were declared and never applied, executed-body identity was never asserted, E5 and O5 posed
checks read keys nothing produced, S2/S7's working-directory state was inert, and the proof that a
forced state landed never reached the durable record. That review is preserved unamended.

The correction (`5e04642`) creates every forced state in the launcher and proves it at the post-pin
barrier, asserts what executed, measures O5's two bounds, and writes normalised posing evidence into
every durable case record; [definition section 9.5](experiments/LAUNCH-EXEC-01-DEFINITION.md#95-trial-2-delta-correction--what-posed-means-and-what-a-posed-case-showed)
preregisters it. A new freeze follows it in `SOURCE-HASHES.json`. No case membership, class,
prediction or safe set moved.

**Trial #2 is NOT_RUN, D-7 is NOT granted, and the valid trial count is ZERO.** Before any new D-7
decision the candidate needs one bounded independent review of this correction and fresh Linux
compile-only evidence of `launcher_spike.c`, which changed.

<a id="launch-exec-01-trial-001-closed"></a>

## Trial #1 closed, 2026-09-10 — LAUNCH-EXEC-01 **TRIAL_ABORTED_AFTER_BOUNDARY**

The first and only execution authorised under D-7 has run and is closed. GitHub run `34500901306`
executed freeze `89c923a147ff16182d4d0ae14a0bd7bb6e62723d`, and the
[independent result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-001-RESULT-REVIEW.md) is
accepted; see [DECISIONS.md](DECISIONS.md#trial-001-accepted).

**Trial #1 started.** Every dispatch gate held, the checkout resolved to the authorised bytes rather
than a branch tip, and the mandatory preflight **passed** on Linux 6.17 x86_64 with strace 6.8,
`ptrace_scope 1` and `clone3` available.

**The immutability boundary was crossed**, and then the harness aborted during E5b's fixture setup
with `OSError: [Errno 9] Bad file descriptor`: it read back through a write-only descriptor, which
Linux refuses categorically. E1 through E5 were genuinely posed and scored, but their records existed
only in memory and were lost, so their statuses are permanently
**`UNKNOWN_FROM_PRESERVED_EVIDENCE`**. The remaining 66 cases were never entered.

**No frozen aggregate is derivable and none was manufactured.** Scoring the missing records INVALID
would assert that E1–E5 could not be posed, which is false. LAUNCH-EXEC-01 has produced **no**
`MECHANISM_ACCEPTED`, `MECHANISM_REJECTED` or `MECHANISM_INCONCLUSIVE`.

**D-7 for Trial #1 is consumed.** Trial #1 must not be re-run, resumed or patched, its freeze is
unchanged, and its dispatcher is retired from the default branch. **Trial #2 is a new preregistered
trial with its own freeze and its own D-7, which is NOT granted.**

<a id="launch-exec-01-d7"></a>

## Owner decision, 2026-09-10 — **D-7 AUTHORISED**, LAUNCH-EXEC-01 still NOT_RUN

The owner has granted **D-7** for **exactly one** LAUNCH-EXEC-01 trial, bound to freeze commit
`89c923a147ff16182d4d0ae14a0bd7bb6e62723d`, whose `SOURCE-HASHES.json` has SHA-256
`9fb861602d477a00f014f14f0a31b5979947af18fa2b620b95ac803ab5bfc365` and Git blob
`4561daf32acb398ec6a601acbe7e986bce0b1105`. The full record is in
[DECISIONS.md](DECISIONS.md#d-7-authorised).

The authorisation is recorded **outside** the freeze. No frozen source, definition or manifest byte
changed, and `d7_execution_authorised: false` inside `SOURCE-HASHES.json` is deliberately left as it
was: it describes the state at which the immutable freeze was cut, not the owner's later authority.

**LAUNCH-EXEC-01 is still NOT_RUN and the valid trial count is still ZERO.** Authorising a trial is
not running one. The preregistered environment is a GitHub-hosted `ubuntu-24.04` runner and the
definition requires the experiment workflow to run **only** on `workflow_dispatch`, so the trial
begins when that workflow is deliberately dispatched — see
`.github/workflows/launch-exec-01-trial.yml`, which verifies the
freeze identity against the authorised SHA-256 before it will pose anything, runs the non-posing
preflight first, and refuses to start if either check fails.

The first execution of the first preregistered case is the immutability boundary. A preflight HALT
poses zero cases and does not consume it. Any re-run is a new trial. The result requires an
independent review before ADR-0024 or `crates/helm-launch` may advance.

<a id="launch-exec-01-frozen"></a>

## Owner decisions and pre-trial freeze, 2026-09-09 — LAUNCH-EXEC-01 at 72 cases, still NOT_RUN

The owner resolved every open decision from the
[pre-execution review](implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md) and authorised
**pre-trial experiment implementation only**. The
[preregistered definition](experiments/LAUNCH-EXEC-01-DEFINITION.md) is re-frozen at **72 cases**
— 56 mandatory, 9 conditional, 7 recorded — and the disposable experiment sources now exist under
[`docs/experiments/launch-exec-01/`](experiments/launch-exec-01/).

**Decisions.** **D-1 arm (i)**: pursue the tiny isolated unsafe Linux backend, and do **not**
weaken the exact FD-inheritance invariant merely to preserve crate-wide `forbid(unsafe_code)`.
**D-9**: refuse `S_ISUID`/`S_ISGID` objects at admission as `SetIdBitsPresent`, never
record-and-permit. **D-10**: the child environment is **exactly empty** and `explicit` is removed
from the 0.1 contract — no key/value list, no `PATH`, no `HOME`, no `LD_*`, no `GCONV_PATH`, no
locale or ambient inheritance. **D-11**, new: the child sets `PR_SET_NO_NEW_PRIVS` before exec,
because D-9 refuses set-id bits but **file capabilities are a separate mechanism admission
metadata does not carry**. D-2 to D-6 and D-8 are accepted, D-4 as corrected by the review.
**D-7 — authorisation to run — is NOT granted.**

**`frozen_cases.py` is the machine source of truth.** Membership, the class partition, the frozen
D-1 arm, schedules and per-case `traced` flags live there; the definition's prose tables are
**generated from it**, and `tools/tests/test_launch_exec_01.py` enforces that they match, along
with the manifest's own invariants and the totality of the aggregate precedence — using
fabricated records that never invoke the spike. That self-test caught a real defect in the
corrected aggregate rule: as written after the review, a conditional case BLOCKED with a recorded
cause would have fired `MECHANISM_INCONCLUSIVE`, which would have made the class meaningless and
the whole run inconclusive by construction, since **N3 is BLOCKED by design on any unprivileged
runner**.

**What the experiment still cannot show, recorded rather than papered over.** D-11's guarantee
rests on three separable things that must never be collapsed: the kernel semantics, taken from
primary sources; the **directly observed** `NoNewPrivs: 1` state, which N1 establishes and N2
controls for; and the **untested** privileged transition, which is N3, is BLOCKED because no
privileged fixture is created, and is never presented as demonstrated.

**The C sources have not been compiled on Linux.** They were authored and reviewed on a Windows
host, where a Linux build cannot honestly be attempted; faking one, or reaching for WSL merely to
obtain a green result, would manufacture evidence. First compilation is a preflight step of the
first authorised trial, and a failure there is a preflight finding under the halt rule.

**No trial has been executed and no case has been posed.** `crates/helm-launch` was not created,
no experiment code entered the Cargo workspace, and the runner refuses to pose a case without
both a verified source freeze and explicit owner authorisation. **ADR-0024 remains Proposed** and
**A0-7ZIP remains experimental FAIL.**

<a id="helm-launch-pre-execution-review"></a>

## Pre-execution review, 2026-09-09 — helm-launch 0.1 and LAUNCH-EXEC-01

Three independent reviewers audited the proposed architecture and the preregistration in
**isolated contexts, in parallel**, before any experiment is authorised: A on executable identity
and `execveat`, B on the process boundary and lifecycle, C on the preregistration as a protocol.
None could write to the repository and none saw the others' work; the synthesis resolved conflicts
by technical evidence rather than by vote. The result is the
[pre-execution review](implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md).

**Classification: NEEDS_ARCHITECTURE_OWNER_REVIEW. Sixteen BLOCKER findings** among 53 in total, two of them reached
independently by two workstreams. The core mechanism survives — `execveat` on a retained
descriptor does pin the inode — but the frozen definition could have returned
`MECHANISM_ACCEPTED` while four load-bearing claims were false, and the documented child-setup
sequence could not have executed a single successful launch, because it closed the
working-directory descriptor before using it and the parent never closed its own copies of the
pipe write ends.

**What was corrected.** The executable digest is a **pre-execution measurement** and was
described as the identity of the body that ran; it is renamed `pre_exec_body_sha256`, and
`ETXTBSY` does not cover the measure-to-exec window — the ordinary open-write-close writer is
never refused. The drain loop had no termination condition surviving a descendant that holds a
stdio write end, so a bounded `POST_EXIT_DRAIN_MS` and a `WriterRetainedAfterChildExit`
disposition now bound the launcher's own tail without claiming containment. Inherited signal mask
and ignored dispositions were an unclosed ambient input. The credentials claim was
unconditionally false for a set-user-ID object. Clean EOF does not prove exec. The receipt becomes
a product of one process disposition and one per-stream completeness, because those facts are
orthogonal. ELF admission now pins the header cohort, so no `binfmt_misc` entry can route a HELM
capability to a pathname-resolved interpreter.

<a id="launch-exec-01-reframed"></a>

**LAUNCH-EXEC-01 is re-frozen at 71 cases and remains NOT_RUN**, up from 43: two deleted as
unposable, thirty added, eleven reformulated — 58 mandatory, 5 conditional, 8 recorded, verified
mechanically. Its aggregate precedence was neither total nor disjoint and is replaced by an
ordered total precedence following OBS-FS-01; E6's safe set was illegitimate and becomes a single
prediction; F2 is a mandatory PASS under the owner's D-1 arm (i) and its FAIL now rejects; the
helper report moves to descriptor 1 so the instrumentation stops violating the invariant it
measures; and a launcher-side non-return is an unconditional rejection, including in the negative
controls.

**D-1 through D-6 and D-8 are provisionally recorded; D-7 is NOT authorised.** Two new owner
decisions the review surfaced are open and both change case membership: **D-9**, whether to refuse
set-user-ID objects at admission, and **D-10**, whether the environment is `empty`-only in 0.1.
A definition whose membership depends on an open decision cannot be frozen, so the recommended
next step is to rule on D-9 and D-10 and re-read the corrected case table.

`helm-app-spec`, `helm-observe`, `helm-bind` and `helm-evidence` are untouched; no crate was
created; **ADR-0024 remains Proposed**; the experiment was **not executed**; main was not changed;
and **A0-7ZIP remains experimental FAIL.**

<a id="helm-launch-architecture-review"></a>

## Design under owner review, 2026-09-09 — helm-launch 0.1 architecture and LAUNCH-EXEC-01

The [helm-launch architecture and falsification plan](research/HELM-LAUNCH-ARCHITECTURE.md)
designs HELM's fifth product module, the first whose purpose involves **actual process
execution**, and proposes [ADR-0024](adr/ADR-0024-launch-authority.md). **Design only.** There
is no `crates/helm-launch`, no product API change, no execution code, no Wine or 7-Zip
execution, no A0 access and no lab change. ADR-0024 is **Proposed, not Accepted**, and
authorises no implementation.

**Recommended 0.1 scope:** one crate that executes exactly one explicitly authorized,
already-open, regular ELF object on Linux x86_64 (kernel 5.9 or newer, the floor set by
`close_range`), via `fork` plus `execveat(fd, "", AT_EMPTY_PATH)`. A Wine-specific launcher,
a path-based `std::process::Command` launcher, a two-crate split and a non-executing validator
were all compared and rejected, with reasons recorded.

**The authority boundary is the core of the design.** Parsing a `LaunchPlan` grants zero
execution authority, and a `BindingReport` grants zero execution authority:
`NoClaimContradicted` does **not** mean ready, safe, permitted, compatible or launchable, and
is reachable with `unsupported_binding >= 4` and almost nothing mapped. Execution authority is
an already-open executable descriptor passed by value by a trusted caller. The recommended
design depends on **no HELM crate** — context travels as opaque digests — so the launcher
contains no code path that can read `Contradiction` or `Coverage` at all.

**Stated non-claims.** It is **not a sandbox**: no filesystem, network, process, registry,
device or user-data isolation, and the child runs with the caller's own OS credentials. It
provides **no process-tree containment**, only direct-child lifecycle. Exit code 0 means only
that the direct process exited with status 0. A launch result is never compatibility, and
execution receipt is never rollback.

**Two findings from the current tree shape the design.** The workspace declares
`unsafe_code = "forbid"`, which cannot be relaxed by a local `#[allow]`; and `rustix` 1.1.4
provides no `close_range` and exposes `execveat` only as an `unsafe fn` in a `doc(hidden)`
module documented as unstable. The exact descriptor-inheritance invariant therefore cannot be
implemented in safe Rust today, which is recorded as owner decision **D-1** together with its
alternative of keeping `forbid` and publishing a weaker inheritance claim. Eight owner
decisions in total are listed in the design report and none is decided.

> **Superseded in part, 2026-09-09**, by the
> [pre-execution review](#helm-launch-pre-execution-review). This section records the design as
> submitted, and is kept unedited for provenance. Since then the design report has been corrected
> under sixteen BLOCKER findings, the owner decisions have grown from eight to **ten** with D-9 and
> D-10, and D-1 is provisionally recorded at arm (i).

<a id="launch-exec-01-proposed"></a>

**LAUNCH-EXEC-01 is proposed and NOT_RUN.** Unlike helm-bind, a system experiment **is**
warranted: every load-bearing claim is about kernel behaviour that pure tests cannot settle.
The [preregistered definition](experiments/LAUNCH-EXEC-01-DEFINITION.md) freezes 43 cases
across executable identity and TOCTOU, argv literalness, environment, descriptor inheritance,
exec failure, output draining, exit and signal, timeout, spawn/exec confirmation, and a
process-tree negative control designed to demonstrate a limitation rather than to pass. It
uses only synthetic helpers: **no Wine, no 7-Zip, no proprietary software, no A0 lab and no
privileged operation.** The recommended first environment is a GitHub-hosted `ubuntu-24.04`
runner. `crates/helm-launch` must not be created before it has run and been reviewed.

> **Superseded, 2026-09-09.** The 43-case definition described here was audited and
> [re-frozen at 71 cases](#launch-exec-01-reframed), then at
> [72 after the owner decisions](#launch-exec-01-frozen); the count, the case classes, the
> aggregate verdict rules and the instrumentation all changed. It remains **NOT_RUN**.

`helm-app-spec`, `helm-observe`, `helm-bind` and `helm-evidence` are untouched; ADR-0021,
ADR-0022 and ADR-0023 are unchanged; `helm-launch` remains unimplemented and unaccepted; and
**A0-7ZIP remains experimental FAIL.**

<a id="helm-bind-owner-acceptance"></a>

## Owner acceptance, 2026-09-09 — experimental helm-bind 0.1 is merged to main

The owner accepted the independently reviewed implementation and **fast-forwarded main** from
`60a0e16962ac4fcd0af8e545a33bc7ded9bcc4b6` to the reviewed tip
`4f51c1b2bc7e59b8142ccc4c328e2641c89327d3`. Strict fast-forward: the merge base equalled main
exactly, the lineage is linear with no merge commit, and no squash, rebase, cherry-pick, amend
or force-push occurred. The whole reviewed history is preserved — the implementation `706e276`,
the author review `932631a`, the author test correction `86ab17f`, the submitted candidate
`832a112`, the independent adversarial suites `e9daf4e` committed before any correction, the
two reviewer-test corrections `e42e1ac` and `fe0ce62`, the independent first-pass findings
`564cbc4` recorded **before** any product correction, the corrections `78e26de`, and the
published review `4f51c1b`. The independently corrected **product-code tip** is
`78e26de4ca952b7125032e5c9fa468e6dc85af7c`; `4f51c1b` adds the review document only and
changes no product code. Verified CI on that exact product-code SHA: runs `34355861835`
(HELM Rust workspace Linux) and `34355861658` (helm-bind cross-platform purity), both success.

**Experimental helm-bind 0.1 is now owner-merged on main.** It is **not a release**: the
schema, API, numeric limits and internal layout stay unstabilised and `publish = false`.
Accepted [ADR-0023](adr/ADR-0023-binding-authority.md) remains the architectural authority and
this acceptance changes none of its semantics, widens no scope and stabilises nothing.

**What was independently verified**
([review](implementation/HELM-BIND-INDEPENDENT-REVIEW-0.1.md)): no BLOCKER, and no unresolved
IMPORTANT — both IMPORTANT findings were claims about the code rather than defects in it, and
each is corrected and now enforced by a test that establishes the property it states. The
universal coverage theorem `unsupported_binding >= 4` is established structurally and by a
generator over the schema boundaries, with the maximum specification reaching exactly **39**
semantic claims. Refusal atomicity, the typed comparison domains, the body truth table, the
entry-point root and path rules, the optional entry-point digest precedence, the ten distinct
observation and binding states, `Contradicted` **if and only if** at least one claim is
`Mismatch`, the serializer's injectivity and size bound, and the four-digest acyclic report
identity were all confirmed independently. The crate performs **no I/O and holds no
authority**, and the cross-platform evidence scope is stated honestly.

**Retained limits, unchanged by acceptance.** There is **no satisfaction, compatibility or
readiness verdict** and no global success token of any kind. Unsupported semantic coverage
remains **at least four** — source architecture, runtime family, Windows architecture and
prefix role have no comparator in 0.1 — so a binding report can never claim completeness.
`asserted_prefix_root_id` remains a **caller assertion, never an attestation**, and no report
upgrades it. A refusal produces **no `BindingReport`**: no bytes, no claim outcomes and no
report identity. `bind` remains a pure, authority-free function. End-to-end `bind` runs on
Linux only, because `ObservationArtifact` has no public constructor elsewhere; the parser, the
claim universe, the algebra and the serializer run on Ubuntu, Windows and macOS.
`helm-launch` remains **unaccepted and unimplemented**, so no comparison confers launch
permission. No A0 rerun, no Wine or 7-Zip execution, no lab change, and **A0-7ZIP remains
experimental FAIL.**

## Independent review, 2026-09-09 — helm-bind 0.1 candidate reviewed

The bounded implementation candidate `832a112decaa333ee3b618d52e966a20c443513e` on
`product/helm-bind` was independently reviewed on `review/helm-bind-independent`. The
[independent review](implementation/HELM-BIND-INDEPENDENT-REVIEW-0.1.md) records the
reconstructed state, the dataflow trace, the findings, the corrections and the evidence. This
entry records review status only; the candidate is **not owner-merged**.

**No BLOCKER.** Eighteen independent adversarial tests, with reviewer-chosen seeds and
independent oracles, passed against the **unmodified** candidate: the binding-plan parser and
its ceilings, exact mapping identity, refusal atomicity and precedence, typed-domain
separation, the size and digest truth table, entry-point root and path rules including a
case-only difference, the optional entry-point-digest precedence matrix, observation-state
distinctness, the contradiction algebra and the four-digest report identity. The universal
coverage theorem was re-established with a generator spanning the schema boundaries rather
than the two committed fixtures, confirming `unsupported_binding >= 4` for every valid
specification and the exact claim-instance formula with its ceiling of 39.

**Two IMPORTANT findings, both claims about the code rather than defects in it, corrected.**
Caller role and DLL selectors are legal `helm-app-spec` identifiers and may be verdict words,
so they reach report bytes verbatim, which the README and one author test said could not
happen. The contract question resolves in the implementation's favour: ADR-0023 forbids
verdict vocabulary in the binder's **own** terms, not caller data echoed under a `role` key,
and a selector can never reach a slot the binder controls. The property is now stated
precisely and enforced by a test whose selectors deliberately are verdict words. Separately,
the three-platform run executed the parser, the claim universe, the algebra and the serializer
on all three runners, not `bind` over four genuine inputs, because `ObservationArtifact` has
no public constructor off Linux; the determinism claim stands on that execution plus verified
purity, and the README now says exactly what was run where.

One MINOR collapse was corrected: an unreachable branch that returned `not_observed` for a
mapped target missing from the artifact now returns the conservative
`observation_not_interpretable`. `helm-app-spec`, `helm-observe` and `helm-evidence` remain
byte-identical to main, no `helm-observe` API was added to make the review pass, no lab or VM
ran, no A0 access occurred and **A0-7ZIP remains experimental FAIL.**

## Implementation candidate, 2026-09-09 — helm-bind 0.1 awaits independent review

Experimental [helm-bind](../crates/helm-bind/README.md) 0.1 is implemented on
`product/helm-bind` under Accepted [ADR-0023](adr/ADR-0023-binding-authority.md) and
**awaits independent review**. The [author implementation review](implementation/HELM-BIND-REVIEW.md)
maps the code to every accepted invariant. Disposition: **READY_FOR_INDEPENDENT_REVIEW**.
This is a branch candidate, not an owner-accepted merge or a release, and the author must not
merge it.

`bind` is a pure function of four already-validated values, with no filesystem, descriptor,
network, subprocess, environment, registry, clock, randomness, write or OS backend, no
product-semantic `cfg` branch and `unsafe` still forbidden. It emits **no satisfaction,
compatibility or readiness verdict** and no global success token: one typed outcome per
semantic desired claim, plus contradiction, which is true only when two known values
disagree, and coverage counts over ten distinct states. **Coverage can never be complete in
0.1**, because four mandatory desired requirements have no comparator, and that is a property
test rather than a promise.

The mapping is an inert exact-byte document with a closed five-kind vocabulary and no expected
values, predicates or paths; nothing is inferred from a target ID's spelling.
`asserted_prefix_root_id` is required exactly when an entry-point claim is mapped and every
mapped entry-point target must sit under it. Both entry-point claims require the regular-file
observable and bind the desired relative path byte for byte, so identical bytes at another
path never satisfy either, and a specification stating no entry-point digest yields
`desired_value_unspecified` rather than a manufactured requirement. Every cross-document
precondition is checked before any claim exists, so a refusal produces no report bytes, no
claim outcomes and no report digest.

Neither `helm-app-spec` nor `helm-observe` was changed, in semantics, API or dependency
features; the `sha2` feature-unification effect is recorded as performance and build
composition only, and the separate package builds and standalone evidence tests are retained.
No new system experiment was run: the module is pure, so adversarial and property tests are
the falsification mechanism. No `helm-launch`, no lab or VM execution, no A0 rerun, no Wine
and no 7-Zip. **A0-7ZIP remains experimental FAIL.** The next decision is to assign
independent review of this bounded candidate.

<a id="helm-bind-architecture-acceptance"></a>

## Owner acceptance, 2026-09-09 — ADR-0023 Accepted for helm-bind 0.1 architecture

The owner accepted the [helm-bind design](research/HELM-BIND-ARCHITECTURE.md) subject to
bounded pre-implementation corrections, and
[ADR-0023](adr/ADR-0023-binding-authority.md) moved from Proposed to **Accepted** with a
bounded 0.1 architecture scope. Acceptance settles overall-verdict semantics, mapping
semantics, cross-crate dependencies and the identity graph. It **does not authorise
implementation**: there is no `crates/helm-bind`, no product API changed, no comparison code,
no launch work, no lab access and no A0 rerun. `helm-launch` remains unaccepted and undesigned.

**Accepted core.** A pure, authority-free comparison library over four already-validated
inputs, emitting **no satisfaction, compatibility or readiness verdict**. Per-claim outcomes
plus two axes: `Contradicted` only when two known values disagree, and a coverage summary.
`Absent`, observation rejection, observation failure, observation omission and unsupported
binding are **never** promoted to contradiction in 0.1, so entry-point absence stays the
explicit `Absent` state — deliberately conservative, because the prefix-root association is
caller asserted rather than attested.

**Bounded corrections applied.** Both entry-point comparators require `regular_file_sha256`,
because `directory_metadata` rejects a regular file as `WrongKind`; helm-observe is not
changed and no new observable is added. Path binding applies to **both** entry-point claims,
byte for byte with no normalisation, so a same-content file at another relative path cannot
satisfy the entry-point body claim. The caller assertion is renamed `asserted_prefix_root_id`,
is required exactly when an entry-point claim is mapped, and every mapped entry-point target
must sit under it or the binding is refused. The claim universe is defined precisely: the spec
digest, application ID and version, artifact labels and role selector keys receive no claim
outcome and do not enter coverage. The coverage theorem is corrected to **at least four**
unavoidable unsupported claims — source architecture, runtime family, Windows architecture and
prefix role — which holds even when `disabled_dlls` is empty, and becomes a property-test
obligation. The taxonomy is a single closed set of **ten** states, and a report carries up to
**39** semantic claim instances although a binding plan carries at most 27 mapping entries.

**Also accepted unchanged.** The inert exact-byte `BindingPlan`; `ValidatedPlan` as a required
input because the observation artifact deliberately carries no target paths; subject-spec and
observation-plan identity checks that work against the current public APIs, so **no change to
helm-observe is required**; direct dependencies on `helm-app-spec` and `helm-observe` and not
`helm-evidence`, with the `sha2` unification effect classified as performance and build
composition only; the four-digest acyclic report identity; and typed refusals that produce no
report bytes, no claim outcomes and no report digest. No new system experiment is required:
the module is pure, so adversarial, property, exact-identity and cross-platform tests are the
falsification mechanism. **A0-7ZIP remains experimental FAIL.**

## Design under owner review, 2026-09-09 — helm-bind 0.1 architecture

The architecture for the fourth product module is proposed and **awaits owner review**. The
[design report](research/HELM-BIND-ARCHITECTURE.md) and Proposed
[ADR-0023](adr/ADR-0023-binding-authority.md) describe a pure, authority-free comparison
library that answers only what follows from comparing desired claims with the observations
that were actually made. **Nothing is accepted and nothing is implemented:** no
`crates/helm-bind`, no product API change, no comparison code, no launch work, no lab access
and no A0 rerun.

The central proposal is that `helm-bind` emits **no satisfaction, compatibility or readiness
verdict**. It reports per-claim outcomes plus two axes: contradiction, which is true only when
two known values disagree, and coverage, which counts states and lists the claim classes with
no comparator. Because `runtime.family`, `environment.windows_architecture` and
`environment.prefix_role` are mandatory desired claims that helm-observe 0.1 cannot establish,
**coverage can never be complete in 0.1**, which makes a false global success structurally
unreachable rather than merely discouraged.

The design was derived from the current public Rust types, not from historical sketches. It
records that the identifier, path and digest grammars of helm-app-spec and helm-observe
already coincide, so no translation layer is needed; that the observation artifact carries no
target paths, so the `ValidatedPlan` must be an input; that observation target IDs are
deliberately neutral, so an explicit identity-bearing binding plan is required rather than any
naming heuristic; and that the subject-spec and observation-plan identity checks work against
the **current** public APIs, so **no change to helm-observe is required**. No new system
experiment is proposed: the module performs no syscall, so adversarial and property tests are
the right falsification mechanism.

Eight owner decisions are listed in the design report. ADR-0021 and ADR-0022 are unchanged,
`helm-launch` remains unaccepted and undesigned, and **A0-7ZIP remains experimental FAIL.**

<a id="helm-observe-owner-acceptance"></a>

## Owner acceptance, 2026-09-09 — experimental helm-observe 0.1 is merged to main

The owner accepted the independently reviewed implementation and **fast-forwarded main** from
`0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b` to the reviewed tip
`626de914000263cc3206d28479db692ad0b40724`. Strict fast-forward: merge base equalled main
exactly, the lineage is linear with no merge commit, and no squash, rebase, cherry-pick,
amend or force-push occurred. The full history is preserved, including the initial
implementation `53ade006`, the author candidate `e2a62081`, the independent first-pass
findings `a1726f3` recorded **before** any correction, and every correction commit. Verified
CI on that exact SHA: runs `34316001530` (helm-observe independent review) and `34316001469`
(HELM Rust workspace Linux), both success.

**Experimental helm-observe 0.1 is now owner-merged on main.** It is **not a release**: the
schema, API, numeric limits and internal layout stay unstabilised and `publish = false`.
Accepted [ADR-0022](adr/ADR-0022-observation-authority.md) remains the architectural
authority and this acceptance changes none of its semantics.

**What was independently verified** ([review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW-0.1.md)):
no unresolved BLOCKER or IMPORTANT finding; exact plan SHA-256 binding, root-set binding by
logical ID, and plan-substitution prevention by construction, all three obligations
implemented and independently confirmed; procfs admission using the corrected fresh `memfd`
self-identity probe, with controlled foreign-process descriptor directories refused and the
real current-process capability admitted; strict rejection of duplicate decoded JSON keys at
every object depth; the observation backend gated to Linux x86_64; special files, symlinks and
over-limit metadata proved at syscall level never to reach the regular-file data reopen; no
pathname fallback; bounded hashing and budget behaviour.

**Retained limits, unchanged by acceptance.** The supported and evidenced cohort stays
**Linux x86_64 and ext4**. Cohort membership is an **external support precondition, not
something helm-observe attests**: the trusted caller and the provisioning environment supply
in-cohort roots. `0xEF53` is only the necessary ext-family admission guard, never proof of
ext4; **ext2 and ext3 remain unsupported and unevidenced even though they mechanically pass
it**; storage locality is not attested; and no artifact makes a filesystem-identity claim, now
enforced by test. Descendant bind mounts remain outside the validated 0.1 cohort. There is no
desired-versus-observed comparison and no execution anywhere in helm-observe: a future
`helm-bind` owns comparison and a future `helm-launch` owns execution, and **neither is
implemented or accepted**. No A0 rerun, no Wine or 7-Zip execution, no lab change, and
**A0-7ZIP remains experimental FAIL.**

## Owner decision, 2026-09-09 — ADR-0022 cohort attestation clarified; helm-observe 0.1 ready for owner merge

The owner resolved the architecture question the
[independent review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW-0.1.md) raised, by
[clarifying Accepted ADR-0022](adr/ADR-0022-observation-authority.md#cohort-attestation-clarification)
rather than by expanding support. ADR-0022 **remains Accepted** and its scope is unchanged.
The operative clarification is: **filesystem cohort membership is an external support
precondition, not an observation attestation; root admission applies only necessary mechanism
guards available inside the explicit capability boundary.**

**The supported and empirically validated 0.1 cohort stays Linux, x86_64, ext4.** It is
**not** broadened to ext2, ext3 or "ext-family". `helm-observe` does **not** attest that a
supplied root is ext4 and does **not** attest storage locality; the trusted caller and the
provisioning environment supply in-cohort roots, established outside the crate. The retained
`0xEF53` check means only "this descriptor is on a filesystem reporting the ext-family
superblock magic" — a necessary sanity guard, never verified ext4, a validated cohort, local
storage, a supported environment or provenance. An ext2 or ext3 root may mechanically pass it;
such an observation is outside the validated cohort and inherits no support or safety claim.
No new authority was added to enforce the label: no mountinfo read, no block device, no sysfs,
no mount scan, no superblock read. The descendant bind-mount exclusion and every other
ADR-0022 semantic limit are preserved unchanged.

The IMPORTANT-3 finding is preserved, not erased: it was valid, the implementation could not
solve it inside ADR-0022, and the owner resolved it by decision. The cohort tests now print
two statements that must not be collapsed — runner evidence that the *test* ran on ext4, and
the *product guard* seeing `f_type = 0xEF53`.

With that decision, no BLOCKER or IMPORTANT finding remains unresolved. The independent review
classifies the corrected tip **READY_FOR_OWNER_MERGE**. That is a reviewer recommendation:
the candidate is **still not owner-merged**, main is unchanged at
`0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b`, and the merge remains a separate owner action. No
A0 evidence was opened, no WSL or Hyper-V lab was touched, no Wine or 7-Zip ran, `helm-bind`
and `helm-launch` remain unimplemented, and **A0-7ZIP remains experimental FAIL.**

## Independent review, 2026-09-08 — helm-observe 0.1 needs an owner architecture decision

The bounded implementation candidate `e2a62081ece52391b30ede153eee139103c98e31` on
`product/helm-observe` was independently reviewed on `review/helm-observe-independent`. The
[independent review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW-0.1.md) records the
reconstructed state, an independent source trace, the findings, the corrections and the
evidence. Disposition: **NEEDS_ARCHITECTURE_OWNER_REVIEW**. The candidate is **not
owner-merged**, and this entry records review status only.

One BLOCKER and three IMPORTANT findings were raised, and the first-pass findings were
committed before any correction so the candidate as submitted is preserved. The BLOCKER was
**reproduced on hosted Ubuntu CI**: the procfs self-identity probe used the caller-supplied
capability as its own probe object, so a foreign process holding its own `/proc/self/fd`
across low descriptor numbers had its descriptor directory admitted as the trusted
current-process capability. The later device, inode and kind re-verification still prevented
any unauthorized read, so the damage is invariant violation and an availability surface, not
disclosure. Admission now creates a fresh anonymous memory object as the probe, which is what
the frozen OBS-FS-01 definition required. Duplicate JSON keys were also undetected inside
array-nested objects and across escape spellings, and the observation backend was gated on the
operating system only, so it carried full authority outside the accepted Linux x86_64 cohort.
Both are corrected with regression tests, and the backend is now gated to the cohort.

**The open question is the ext4 cohort itself.** `0xEF53` is the shared ext2, ext3 and ext4
superblock magic, so root admission establishes an ext-family **necessary but not sufficient**
condition and cannot establish ext4; "local" is likewise assumed rather than established. No
mechanism inside the accepted explicit-capability, no-ambient-scan boundary can close that
gap. Admission was **not** weakened and the cohort was **not** renamed: only the claims were
corrected to state what the mechanism establishes, and the previous
`HELM-OBSERVE-COHORT: ext4 PASS` line is withdrawn as unsupported by its own evidence. The
owner must choose whether to restate the cohort, require a stronger discriminator with new
evidence, or accept in writing that admission does not enforce ext4.

The special-file, symlink and no-fallback negatives are now proved on the **compiled Rust
implementation** at syscall level, by a review-only driver over the public API traced with the
frozen OBS-FS-01 tracer imported read-only: zero data reopens, zero reads, zero `connect` and
zero `getdents64` for the trailing symlink, non-final symlink, FIFO, socket and over-limit
cases, with `resolve = 15` and `O_PATH|O_NOFOLLOW|O_CLOEXEC` on every resolution. A FIFO whose
writer was blocked in `open(2)` stayed blocked throughout. Evidence cohort: hosted Ubuntu
24.04, kernel `6.17.0-1022-azure`, x86_64.

No OBS-FS-01 case was re-run and no frozen file changed; no A0 evidence was opened; no WSL or
Hyper-V lab was touched; no Wine or 7-Zip ran; ADR-0022 was not broadened; `helm-bind` and
`helm-launch` remain unimplemented; main was not modified. **A0-7ZIP remains experimental
FAIL.** The next decision is the owner's, on the ext4 cohort question.

## Implementation candidate, 2026-09-08 — helm-observe 0.1 awaits independent review

Experimental [helm-observe](../crates/helm-observe/README.md) 0.1 is implemented on
`product/helm-observe` under Accepted
[ADR-0022](adr/ADR-0022-observation-authority.md) and **awaits independent review**. The
[author implementation review](implementation/HELM-OBSERVE-REVIEW.md) maps the code to
every ADR-0022 requirement and to each merge-blocking obligation. Disposition:
**READY_FOR_INDEPENDENT_REVIEW**. This is a branch candidate, not an owner-accepted merge
or a release, and the author must not merge it.

The crate answers only "what actually exists at explicitly authorised targets?". The three
merge-blocking obligations are implemented and tested: the scope binds the complete plan
SHA-256 and the artifact carries it; roots bind by logical ID, set and count rather than
position; and plan substitution is **impossible by construction**, because `authorize`
consumes the validated plan and `observe` takes only the authorised scope.

It depends on neither HELM crate, adds no shared utility crate, and keeps the workspace
`unsafe_code = "forbid"`: rustix's safe wrappers express the whole accepted mechanism, so
no libc or raw-syscall unsafe block was needed. Pure plan and model code compiles
cross-platform; the observation backend is gated to Linux with no fake Windows semantics.

Hosted Ubuntu CI passed `cargo fmt --check`, workspace Clippy with warnings denied, and the
full workspace test suite, including 12 pure contract tests and 16 Linux tests. The
supported-cohort check reported **`HELM-OBSERVE-COHORT: ext4 PASS`** on an ext-family
(`0xEF53`) fixture filesystem, so the ext4 cohort execution is recorded as PASS rather than
BLOCKED. Root admission refuses a non-cohort filesystem instead of degrading.

Descendant bind mounts remain outside the validated 0.1 cohort: `RESOLVE_NO_XDEV` stays
mandatory and such crossings are conservatively rejected, with no validation claimed. No
`helm-bind`, no `helm-launch`, no A0 access or rerun, no Wine or 7-Zip, no privilege
change, and no modification to helm-app-spec or helm-evidence sources or features. The
recorded `sha2/force-soft` combined-graph limitation is documented, not silently changed.
**A0-7ZIP remains experimental FAIL.** The next decision is to assign independent review of
this bounded candidate.

## Owner acceptance, 2026-09-08 — ADR-0022 Accepted with bounded 0.1 scope

The repository owner accepted [ADR-0022](adr/ADR-0022-observation-authority.md) on evidence
from [OBS-FS-01 PASS](experiments/OBS-FS-01-EXECUTION-REPORT.md). Main was fast-forwarded
from `63ac6796296894945dff520423cb1a93351e8524` through the
[execution definition](experiments/obs-fs-01/) `2496f68cccbf70ac04288992a9945cb34f046b98`
to evidence tip `1bd9c0a75abb0c4acccae352960f42ead96fcacb`, preserving Amendment 1, the
pre-trial freeze and the evidence as separate commits. No squash, rebase, cherry-pick or
force-push occurred.

**Accepted decision:** build `helm-observe` 0.1 as an explicit-target, Linux-only
observation library that reports actual facts only and does not interpret desired state or
execute applications. The evidence basis is the original architecture candidate, the
independent review, Amendment 1, the frozen execution definition and the OBS-FS-01 PASS
with its exact conditional BLOCKED bind-mount claim.

**Accepted cohort:** Linux x86_64 and local ext4, anchored by the tested mechanism and
kernel cohort. Experimental evidence is not generalised into arbitrary Linux filesystem
support.

**Excluded and unverified:** target paths whose claim depends on rejecting a bind mount
created as a descendant of an authorized ext4 root. `RESOLVE_NO_XDEV` stays mandatory and
mount crossings stay conservatively rejectable, but 0.1 claims no empirical validation of
that case, and future support depending on it needs new evidence.

**Three mandatory implementation obligations** block any future owner merge of
helm-observe 0.1: exact plan SHA-256 binding; root logical-ID, set and count binding by ID
rather than position; and prevention of plan substitution after authorisation, with
explicit adversarial tests for authorise-A-then-execute-B and for root
addition, removal and substitution.

**helm-observe is not implemented and its implementation is not accepted.** Acceptance
implies no package provenance, archive-to-loader binding, prefix dedication, effective DLL
policy, desired-state satisfaction, general compatibility, safe launch, atomic snapshot,
network or FUSE safety, arbitrary kernel safety, or Windows semantics. Future `helm-bind`
owns comparison; future `helm-launch` owns execution; neither is accepted. No product code
or Cargo member was added in this acceptance, and **A0-7ZIP remains experimental FAIL**.

The next product task is to implement experimental helm-observe 0.1 under accepted
ADR-0022, which requires a separate owner instruction.

## Experiment executed, 2026-09-08 — OBS-FS-01 PASS, bind-mount case BLOCKED

The owner authorised OBS-FS-01 execution against Amendment 1. The
[execution report](experiments/OBS-FS-01-EXECUTION-REPORT.md) records **PASS**: 41 of 41
mandatory trials satisfied their frozen safe outcome sets and trace invariants, and all
ten mandatory race injections landed, so no trial was INVALID and none fell to
INCONCLUSIVE. The
[execution definition](experiments/obs-fs-01/) was committed and pushed at
`2496f68cccbf70ac04288992a9945cb34f046b98` **before the first preregistered trial**.

Amendment 1 was confirmed on both symlink variants: a trailing symlink is pinned as an
`O_PATH` descriptor and rejected at classification, while a non-final symlink is rejected
at resolution with `ELOOP` and yields no descriptor. Special files were classified with
zero data reopens, zero reads and zero connects. The D1-D3 arm measured why the mechanism
was chosen: direct `O_RDONLY` on a writerless FIFO blocked until the supervisor deadline,
direct `O_RDONLY|O_NONBLOCK` returned a descriptor and so was already a data-open, and the
`O_PATH` route classified the same object without reading it.

The conditional descendant bind-mount case is **BLOCKED** by Ubuntu's AppArmor
user-namespace restriction and was not bypassed. Exactly one claim stays unverified:
**that `RESOLVE_NO_XDEV` rejects a bind mount created as a descendant of an authorized
ext4 root.** General mount-traversal rejection is evidenced by the mandatory non-namespace
fallback and its control arm.

The result is evidence for one kernel, filesystem, toolchain and mechanism cohort only. It
establishes no arbitrary Linux filesystem safety, no network or FUSE behaviour, no Windows
semantics, no atomic snapshot, no provenance and no safe execution. **A PASS does not
accept ADR-0022 and does not authorise helm-observe implementation; both remain separate
owner decisions.** No product code exists, no Cargo member was added, no A0 file was read,
no Wine or 7-Zip ran, no privilege or system configuration changed, and the VM was booted
and shut down normally with 0 checkpoints. [ADR-0022](adr/ADR-0022-observation-authority.md)
remains **Proposed** and **A0-7ZIP remains experimental FAIL**.

<a id="obs-fs-01-amendment-1"></a>

## Owner approval, 2026-09-08 — OBS-FS-01 preregistration Amendment 1

The original frozen preregistration `b4ed2e108134eb58a2561403f5da3509aed955ee`
encountered a **pre-execution factual mechanism defect**: it predicted that a symbolic-link
target is rejected at path resolution with `ELOOP` and that no descriptor is produced.
That holds only for a non-final component. For a **trailing** symlink under the mandated
`O_PATH|O_NOFOLLOW` policy, `openat2` succeeds and returns an `O_PATH` descriptor to the
link itself, so rejection is a classification step.

Preflight and build validation found this **before any preregistered case, frozen fixture
set, race run or experiment verdict existed**, and execution halted at
`90e025890a32b36ccc55a4dc223bbb85046ba152`. The owner approved **Amendment 1**, a
pre-execution correction limited to four symlink items: the section 7 object-kind row, the
withdrawal of finding M3, the section 12 static-symlink outcome, and the section 8 and 12
symlink-race outcomes. Two sentences restating the same falsified claim were corrected for
coherence and add no expectation.

The [amended independent review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md) is now
the **authoritative frozen OBS-FS-01 definition** and supersedes `b4ed2e1` for all future
execution. `b4ed2e1` remains immutable in history with its original role intact, the
preflight observation is preserved, and the error is recorded rather than erased.

Verified unchanged by the amendment: mandatory and optional case membership; the D1–D3
direct-open arm; procfs admission rules; mount preflight and fallback rules; resource
ceilings; instrumentation requirements; oracle definitions; repetition and seed policy;
and the verdict rules and their precedence. The measured preflight fact is not
reinterpreted: **the unprivileged descendant bind-mount case remains a BLOCKED
prerequisite on this lab**, and the non-namespace `NO_XDEV` fallback arm remains mandatory.

**OBS-FS-01 remains NOT_RUN and execution again requires a separate owner authorisation;
this task authorises none.** No VM was booted or accessed, no experiment case ran, no
helm-observe code exists, [ADR-0022](adr/ADR-0022-observation-authority.md) remains
**Proposed**, and **A0-7ZIP remains experimental FAIL**.

## Execution halted, 2026-09-08 — OBS-FS-01 preflight complete, no trial run

OBS-FS-01 execution was authorised and **halted at the preregistration before the first
trial**; see the [preflight and halt record](experiments/OBS-FS-01-PREFLIGHT-AND-HALT.md).
Preflight passed on the recorded lab and measured two things: the unprivileged
descendant bind-mount case is **BLOCKED** by Ubuntu's AppArmor user-namespace restriction
(confirming review finding I8, not bypassed), and a **material frozen expectation is
factually wrong** — a trailing symlink under the mandated flag set returns an `O_PATH`
descriptor to the link rather than failing with `ELOOP`, so rejection is a classification
step, not a resolution step. The safety property held in every observation: no link was
resolved, opened or read, and nothing escaped the authorised root.

The error originates in the independent review (finding M3 and the expectations derived
from it), **not** in the architecture proposal, which stated the correct behaviour. No
preregistered case was executed, so no goalpost moved. Correcting a material expectation
requires a new owner review **before** execution; that decision is pending.

No A0 file was read, no Wine or 7-Zip ran, no sudo, sysctl or AppArmor change was made,
no package was installed, and no helm-observe product code exists. The VM was booted and
shut down normally with 0 checkpoints. ADR-0022 remains Proposed, OBS-FS-01 remains
NOT_RUN, and **A0-7ZIP remains experimental FAIL**.

<a id="obs-fs-01-preregistration-acceptance"></a>

## Owner acceptance, 2026-09-08 — OBS-FS-01 preregistration frozen

The repository owner accepted the corrected OBS-FS-01 preregistration at exact commit
`b4ed2e108134eb58a2561403f5da3509aed955ee`. Main was fast-forwarded from
`e69abdfc55646dff6073befaeb843a18d6c7975f` to that exact tip and published with a
normal push; no squash, rebase, cherry-pick, history rewrite or force-push occurred.
That commit's [independent review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md)
is now the owner's **frozen reference experiment definition**.

**This acceptance covers the experiment definition only.** It accepts no architecture
decision and authorises no execution.

| Question | Status after this acceptance |
|---|---|
| ADR-0022 | **Proposed.** Not accepted, not superseded, unmodified |
| OBS-FS-01 | **NOT_RUN.** Execution requires a separate owner authorisation |
| VM boot/access, harness execution, syscall spike | Not authorised |
| helm-observe implementation | Not authorised |
| A0 access or rerun | Not authorised; **A0-7ZIP remains experimental FAIL** |
| helm-launch, binder, licence, release | Not authorised |

Expectations are frozen **before** execution. Material expectations are: mandatory and
optional case membership; expected safe outcome sets; race schedules and
proof-of-injection requirements; the D1–D3 direct-open comparison arm; the special-file
zero-data-open policy; instrumentation requirements; independent oracles; procfs
admission checks; mount preflight and fallback behaviour; resource budgets; the
repetition policy; and the PASS/FAIL/INCONCLUSIVE/BLOCKED rules. **No material
expectation may change after this acceptance without a new owner review before
execution, and post-execution goalpost changes are forbidden.** Changing a material
expectation after execution invalidates the original preregistration and requires a new
experiment definition, not an amended verdict.

A later OBS-FS-01 **PASS would not** automatically accept ADR-0022 and would **not**
authorise helm-observe implementation; both remain separate owner decisions. A PASS is
evidence for one kernel/filesystem/toolchain/mechanism cohort only. If the unprivileged
descendant bind-mount case is BLOCKED, exactly one claim remains explicitly unverified —
**that `RESOLVE_NO_XDEV` rejects a bind mount created as a descendant of an authorized
ext4 root** — and it must be carried as unverified rather than softened.

The separately tracked implementation-test obligations (exact plan SHA binding, root
logical-ID and count binding, and rejection of an authorized-then-substituted plan) are
**not** OBS-FS-01 execution cases and are not authorised by this acceptance. The next
owner decision is whether to authorise OBS-FS-01 execution against this frozen
definition. Earlier entries retain their then-current authorisation status.

## Independent review, 2026-09-08 — helm-observe experiment definition

An [independent experiment-definition review](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md)
of candidate `6d9e4d8c0dacb09c7f4833a4fda6c4ba50d6e9dc` exists and **awaits owner
authorisation**. It found one BLOCKER and eight IMPORTANT issues in the preregistered
expectations, none of which invalidates the architecture, and supplies a corrected
preregistration. Disposition: **NEEDS_EXPECTATION_FIXES**. Nothing was executed:
ADR-0022 remains Proposed, OBS-FS-01 remains NOT_RUN, no observer code exists, no VM
was booted, and **A0-7ZIP remains experimental FAIL**.

## Architecture analysis, 2026-09-08 — helm-observe awaits owner review

The [helm-observe architecture analysis](research/HELM-OBSERVE-ARCHITECTURE.md) exists
with [ADR-0022 Proposed](adr/ADR-0022-observation-authority.md) and awaits owner review.
It includes an observability matrix and a proposed falsification experiment; no
observer implementation or experiment execution is authorised by this analysis.

## Owner acceptance, 2026-09-08 — helm-app-spec 0.1 merged

The repository owner approved reviewed tip
`8ada80a286b91251998d8a35487dc5f020b2473c` for merge. Main was fast-forwarded
from `e32ab269fbe7c4151186d9257e0aff67c0c70197` to that exact tip and published
with a normal push. A subsequent fetch verified origin/main, and
[main CI run 34229008234](https://github.com/Djomla83/helm-os/actions/runs/34229008234)
completed successfully. **helm-app-spec 0.1 is merged as HELM's second
experimental product module.**

[ADR-0021](adr/ADR-0021-second-product-module.md) remains the architectural
authority. The schema and API remain experimental. Exact-byte document identity
semantics also remain experimental, but are accepted for 0.1: semantically
equivalent byte-different documents may have different SHA-256 identities.
helm-app-spec represents desired state only. It performs no observation or
execution and establishes no runtime, installation, entry-point, compatibility or
verification outcome.

[Independent review](implementation/HELM-APP-SPEC-INDEPENDENT-REVIEW.md) found one
BLOCKER (parser error-path CPU discovery) and one IMPORTANT issue (cross-crate SHA
feature coupling). Both were corrected before merge in
`c9d6c42b331c65805a256514dcd3e0d83a7189f3`. Independent Windows and hosted Linux
validation passed for that correction, and the approved tip is its direct
documentation/evidence-only child.

One build-composition limitation remains open. When helm-app-spec and
helm-evidence participate in the same Cargo build graph, `sha2/force-soft` feature
unification can select the software SHA backend for both crates. This is currently
a documented performance/build-composition limitation, not an evidence-semantic
failure. Standalone release builds preserve the reviewed helm-evidence baseline.
The issue must be revisited before HELM depends on a performance-sensitive
executable that links both modules into one Cargo dependency graph; it is not
permanently accepted or solved.

**A0-7ZIP remains experimental FAIL and was not rerun.** No new architecture ADR
was accepted. `helm-observe` remains unauthorised, and no observation, execution or
further subsystem work is included in this acceptance.

## Independent review, 2026-09-08 — helm-app-spec 0.1

The [independent review](implementation/HELM-APP-SPEC-INDEPENDENT-REVIEW.md) checked
candidate implementation `7d1de4cd02268cf29ea1e4f13136336ce0c2c36f` and documentation
tip `18c7aa5e80617483b6f5afd0fea4affeb62f926c` against authoritative base
`e32ab269fbe7c4151186d9257e0aff67c0c70197`. It found and corrected parser error-path
CPU discovery and the unintended evidence release-build SHA performance change.
The in-memory reader preserves purity; separate release builds preserve the
standalone evidence backend. Combined consumers still use software SHA, now
explicitly documented and measured. No evidence semantic regression was found.

Correction `c9d6c42b331c65805a256514dcd3e0d83a7189f3` passed Windows validation and
[independent hosted Linux CI](https://github.com/Djomla83/helm-os/actions/runs/34224397648).
Original/corrected Windows/Linux results match for 20,000 independent seeded cases,
exact-byte identities and complete evidence reports. Disposition:
**READY_FOR_OWNER_MERGE** on `review/helm-app-spec-independent`; no merge or release
has occurred. Main and the authoring branch remain untouched, and historical
**A0-7ZIP experimental FAIL remains unchanged**. No helm-observe or further module
is implemented or authorized. Next decision: owner review and merge decision for
this corrected branch. Earlier entries preserve their then-current status.

## Implementation candidate, 2026-09-08 — helm-app-spec 0.1

The owner separately authorised the pure application-specification validation
library after accepting [ADR-0021](adr/ADR-0021-second-product-module.md). Starting
local main, origin/main and HEAD were fetched and verified clean at
`e32ab269fbe7c4151186d9257e0aff67c0c70197`. Work uses `product/helm-app-spec`.

[helm-app-spec](../crates/helm-app-spec/README.md) now implements a bounded JSON
bytes-to-immutable-desired-model API with exact-byte document SHA-256 and
deterministic errors. Its schema/API remain experimental. Desired runtime artifact
requirements, dedicated win64 prefix intent, typed DLL disables, inert entry point
and frozen pre-execution definition references establish no observed state.
The [implementation review](implementation/HELM-APP-SPEC-REVIEW.md) records the
fixtures, validation, dependency changes, measurements and independent-review boundary.
Implementation commit `7d1de4cd02268cf29ea1e4f13136336ce0c2c36f` passed local Windows
checks and [hosted Ubuntu CI](https://github.com/Djomla83/helm-os/actions/runs/34219988167).
Disposition: **READY_FOR_INDEPENDENT_REVIEW**; independent review has not occurred.

This is a branch candidate, not an owner-accepted merge or production release.
The author must not merge it. helm-evidence source and A0 historical evidence are
unchanged; **A0-7ZIP experimental FAIL remains unchanged**. No helm-observe,
execution, discovery, lifecycle, recovery, App Forge, licence or further subsystem
is implemented or authorised. Next decision: assign independent review of this
bounded candidate. Earlier entries retain their then-current authorisation status.

<a id="helm-app-spec-selection-acceptance"></a>

## Owner acceptance, 2026-09-08 — bounded helm-app-spec selection

Repository owner Djomla83 explicitly accepted [ADR-0021](adr/ADR-0021-second-product-module.md)
only for: **"Build a pure, non-executable application specification/validation library before
observation or lifecycle execution."** HELM's second selected product module is `helm-app-spec`.
The eventual schema/API remain unsettled and **implementation is not authorised**.

The [selection report](research/SECOND-PRODUCT-MODULE-SELECTION.md) now records the owner's
requirements: raw-byte SHA-256 document identity, desired state only, bounded runtime artifact
identities, acyclic frozen verification references, pure validation, minimal typed environment
fields, Wine-only initial scope and parser safety. Observations belong to a later module,
currently expected to be `helm-observe`. No later module is authorised.

Original analysis commit `08f29c53095a51947e9662ad2d0d7931ccb606ae` is preserved as history.
Main was fetched and verified at `0b72e14f5d6101c281a8d9823168407da3e71b9a` before this refinement.
Only documentation changes are included; `helm-evidence` remains the sole implemented product
module, and **A0-7ZIP experimental FAIL is unchanged**. No other ADR, licence, production release,
data/permission policy or recovery design is accepted. Validation is recorded in the report's
owner-refinement section. The next owner decision is whether to authorise the bounded implementation.

Earlier entries below are historical snapshots, including their then-current acceptance status.

> **Original decision analysis, 2026-09-08:** the report was submitted for owner review with
> ADR-0021 Proposed; the bounded acceptance above supersedes that pending status.

<a id="helm-evidence-owner-acceptance"></a>

## Owner acceptance, 2026-09-08 — helm-evidence 0.1 merged

The repository owner explicitly approved the corrected reviewed tip
`ab2e0d1c46933467dd1d9f9dedbcedeba8a37163` for merge. Main was fast-forwarded from
`ae3f012fb1bfd7b20018c3faf6c71a6740a041fe` to that exact tip and published with a
normal push; a subsequent fetch verified origin/main. **helm-evidence 0.1 is merged
as HELM's first experimental product-code module.** Its schema and API remain
experimental, not stable. This is owner acceptance of the bounded module, not a
production release, licence decision or acceptance of any architecture ADR.

[Independent review](implementation/HELM-EVIDENCE-INDEPENDENT-REVIEW.md) found and
fixed three IMPORTANT issues (symlink-open races, blocking FIFO-open races and
duplicate decoded content expectations) and one MINOR issue (reserved section IDs).
The corrective code/test/CI commit is
`83907b0c1d7f305b9271292e7582b4de831e2469`; the approved tip is its direct child,
changing documentation/evidence only. Product code, manifests, lockfile, tools and
workflow are identical between those two commits. All five commits after the old
main were preserved; the original unfixed candidate was not merged by itself.

[Correction CI run 34187102166](https://github.com/Djomla83/helm-os/actions/runs/34187102166)
was verified completed/success for the corrective commit, without rerunning it.
After publication, [main CI run 34192021221](https://github.com/Djomla83/helm-os/actions/runs/34192021221)
completed successfully for the approved tip: fmt, Clippy, tests and release build
on GitHub-hosted Ubuntu 24.04 with Rust 1.95.0. The workflow remains module-scoped.
Pre-merge Windows checks passed with Rust/Cargo 1.95.0, Python 3.14.3 and the Ryzen
9 5950X host: 45 Rust tests, 36 Python tests, documentation validation, the 22-file
A0 fixture check, whitespace checks and the bounded privacy/forbidden-artifact
audit, including all 153 historical publication hashes. Commands and local receipts
are retained under ignored `target/helm-evidence-merge-admin/`.

**A0-7ZIP experimental FAIL remains unchanged**, including W1's missing required
destination despite correct content and W2's scoped success. No application rerun
occurred. The [security model](../crates/helm-evidence/README.md#security-model) and
[review limitations](implementation/HELM-EVIDENCE-INDEPENDENT-REVIEW.md#hostile-filesystem-outcomes-and-limits)
remain limitations, not newly accepted risks: authentication/freshness/truthful
capture, atomic snapshots, hardlink origin, hostile filesystems, untested platform
and race cases, and comprehensive security/advisory coverage are not established.
No data/permission policy changed. Review branches, VM/WSL labs and private
provenance are retained. Work stops at this acceptance boundary; any next subsystem
requires a separate owner decision. Earlier entries below are historical snapshots.

> **Independent module review, 2026-09-08:** [source findings and Linux validation](implementation/HELM-EVIDENCE-INDEPENDENT-REVIEW.md)
> identified and corrected three IMPORTANT findings and one MINOR finding in
> helm-evidence. Windows/Ubuntu checks and the bounded Linux CI job pass for the
> correction. The corrected review branch is recommended for owner merge review;
> no merge, application rerun, ADR/licence acceptance or new subsystem occurred.

> **Implementation update, 2026-09-08:** the first bounded Rust module,
> [helm-evidence](../crates/helm-evidence/README.md), implements read-only verification
> of declared evidence contracts. [Review and validation](implementation/HELM-EVIDENCE-REVIEW.md)
> supersede earlier statements that no product module exists. The owner accepts
> A0-7ZIP as a completed experimental baseline with its overall FAIL preserved.
> No application rerun, larger Evidence Loop PoC, architecture ADR acceptance,
> licence selection or other subsystem is included in this authorisation.

> Current status: the [application-baseline execution below](#application-baseline) supersedes earlier dated
> snapshots, including their aggregate counts and publication/isolation claims. Historical text is
> retained; the [Gate 0 report](experiments/EXP-009-GATE0-REPORT.md#current-assessment) governs Gate 0,
> and the [application report](experiments/EXP-009-APP-BASELINE-REPORT.md) governs A0-7ZIP.

**Datum:** 2026-09-06. **Faza:** G0, dokumentacija i priprema istraživanja.

| Oblast | Stanje |
|---|---|
| Vizija i zahtevi | Nacrt pripremljen; principi izdvojeni iz razgovora. |
| Arhitektonske odluke | Predlozi; imenovana ljudska odobrenja nisu unesena. |
| Master dokument | Postoji u ovom paketu. |
| AI uputstva i backlog | Postoje u ovom paketu. |
| Osam eksperimenata | Planovi, bez izvršenih app testova. |
| Kohorta aplikacija | Kandidati; svi `not_tested`; verzije i hardver nisu fiksirani. |
| OS / ISO / App Forge | Nisu implementirani. |
| Developer SDK / novi jezik | Nisu implementirani. |
| Windows VM integracija | Nije testirana. |
| Kompatibilnost i performanse | Nepoznato; nema projektnih merenja. |
| Lokalni validator | Samo dokumentacija i strukturirani primeri. |
| Udaljeni GitHub repo | Nije kreiran kroz dostupne akcije ove sesije. |
| Open-source licenca | Predlog, čeka odluku vlasnika. |

GitHub veza je proverena čitanjem naloga i dostupnih repozitorijuma. Dostupni skup
akcija nije pružio kreiranje novog repozitorijuma, a lokalno nije dostupan
autorizovan GitHub CLI. Nijedan postojeći projekat nije menjan.

Ovaj zapis je vremenski snapshot. Nakon stvarne objave dodati provereni URL,
commit SHA i datum; ne izmišljati ih unapred. Nakon eksperimenta dodati evidence
referencu i promeniti odgovarajući status, bez brisanja prvobitne istorije.

Izvršene dokumentacione provere opisane su u [VALIDATION.md](VALIDATION.md).

---

## Update 2026-09-06 — foundation audit completed (EXP-001, desk research)

Written in English by explicit owner instruction; this deviates from the single-language rule in
`CONTRIBUTING.md`, which the owner should either amend or override deliberately. The Serbian
snapshot above is unchanged and is retained as project history.

### What changed

| Area | State |
|---|---|
| Prior-art review (HELM-003 / EXP-001) | **Done as desk research.** Result: [FOUNDATION_AUDIT.md](research/FOUNDATION_AUDIT.md) — twelve domains reviewed against primary sources, plus two adversarial cross-checks. |
| Reuse matrix | Done. USE AS-IS / EXTEND / REPLACE LATER / BUILD NEW across roughly sixty components, in [audit §4](research/FOUNDATION_AUDIT.md#s04). |
| Novelty analysis | Done. Four of five claimed pillars already ship elsewhere; six genuinely unoccupied areas identified, in [audit §5](research/FOUNDATION_AUDIT.md#s05). |
| Risk register | Extended. Ten highest-risk assumptions with a cheap falsifying test each, in [audit §8](research/FOUNDATION_AUDIT.md#s08). |
| PoC proposal | Drafted. [EXP-009](experiments/EXP-009.md) with a Gate 0 falsification stage and pre-registered pass and fail criteria. |
| New architecture decisions | Seven drafts, ADR-0013 to ADR-0019, all `Proposed`. No existing ADR was modified. |
| Applications, ISO, App Forge, SDK, VM, language | **Still not implemented.** Unchanged. |
| Application tests, benchmarks, hardware matrix, sandbox audit | **Still not executed.** Unchanged. |
| Compatibility rate | **Still unknown.** No test has been run. |
| Remote GitHub repository | Still not created through the actions available in these sessions. |
| Open-source licence | Still a proposal awaiting the owner. |

### What was actually verified

Two facts were re-fetched by hand and are first-hand verified: the Wine 11.0 release statements on
WoW64 parity, the removed `wine64` loader, deprecated `WINEARCH=win32` prefixes, NTSync and the
"experimental Wayland driver"; and the state of Wine's `uiautomationcore.spec`, where the pattern
provider entry point is a stub while element discovery and property reads are implemented. The
documentation validator and its twenty unit tests pass. **Nothing else in the audit was verified by
running software**, and [audit §11](research/FOUNDATION_AUDIT.md#s11) lists the claims that did not
survive cross-checking, including one load-bearing measurement that is graded LIKELY rather than
VERIFIED.

### Findings that change the plan

1. The compatibility substrate is mature and must not be rebuilt.
2. Four of the five named pillars — per-application environments, dependency management,
   confinement plumbing and automated environment derivation — are already shipped by an incumbent
   under an open licence.
3. The only defensible differentiator is reproducible, version-scoped, expiring evidence, plus
   recovery that attributes a failure to one axis and never reverts user documents.
4. **The motivating example does not support the proposed architecture.** Communication-application
   unreliability on Linux is, on the available evidence, not a Wine problem. See
   [audit §6](research/FOUNDATION_AUDIT.md#s06).
5. The real thesis under test is economic: whether automation can substitute for the QA labour the
   funded incumbents pay for. It is currently unevidenced.

### Decisions now needed from the owner

Base distribution and exact hardware; the language rule for documentation; a named owner, reviewer
and ADR approver; whether the project funds or performs upstream Wine work; the licence policy, now
with additional inputs recorded in the audit; and the Track A versus Track B first-product question,
which [ADR-0019](adr/ADR-0019-scope-boundary.md) proposes to settle with the Gate 0 result rather
than by argument.

### Next step

Run Gate 0 of [EXP-009](experiments/EXP-009.md) — three to five days, one machine, no new spend —
and publish the result whatever it says. Do not begin catalogue, GUI, OS-image or SDK work before
that.

---

## Update 2026-09-07 — audit corrected, Gate 0 partially executed

### Product mission, restated because the audit does not replace it

HELM's goal is a free, open-source, user-controlled desktop with reliable application experiences.
**Windows compatibility is one mechanism toward that goal, not the goal**, and the proposed Evidence
Loop is an enabling subsystem, not a substitute for the product vision. Nothing below changes the
mission; it changes what is claimed to be known.

### Audit corrected to revision 2

Ten corrections are recorded in [audit §13](research/FOUNDATION_AUDIT.md#s13), written **before** any
result was collected. The systematic error was grading "I read this in a source" and "this was
observed to behave this way" identically; the audit now separates **VERIFIED_SOURCE** from
**VERIFIED_EXECUTION**. The substantive corrections:

- A factual error is fixed and, more importantly, the inference built on it is withdrawn:
  **bug-report counts do not measure usage**. The cause of the reported Viber experience is
  **unestablished** — which is not the same as the earlier claim that it "is not a Wine problem".
- The claim that a 2026 tray regression explained the user's instability is **withdrawn**; it was
  never reproduced, and the affected code path is not present in that client.
- "Four of five pillars already shipped" is replaced by the accurate form: **the capabilities exist,
  the verification loop does not**.
- The evidence pillar's novelty is roughly **halved**. openQA already ships traceability, artifact
  collection, a closed result vocabulary, reproducible rerun, non-pixel oracles and last-good
  attribution. Two genuine absences remain, plus two schema-shape fixes.
- The UI Automation conclusion is scoped to a pinned tag, to pattern-based actuation, and is
  **refuted as a universal claim** by a shipping counter-example.
- Snapshot **contamination** is separated from **corruption**; one backup tool was wrongly accused
  and is withdrawn.

### Gate 0: 1 PASS, 4 BLOCKED, 0 FAIL

Full report: [EXP-009-GATE0-REPORT.md](experiments/EXP-009-GATE0-REPORT.md). Environment record:
[`G0-environment-2026-09-07.json`](experiments/evidence/G0-environment-2026-09-07.json).

The available environment is Windows 11 with WSL2 Ubuntu 24.04 and **no Wine, no compiler, no
sandbox tooling, no desktop environment and no portal backend**. Four checks are therefore BLOCKED
on an approved lab, and are recorded as blocked rather than answered by inference.

The one executed check produced two results worth acting on:

1. **Hardlink farms are confirmed unusable as prefix snapshots** — the copy silently tracks live
   data, deterministically, with no crash required.
2. **Reflink copies were unsupported on both filesystems tested**, including a default Ubuntu
   install. **ADR-0017 proposed reflink as the safe mechanism**, so that ADR is contradicted by
   execution and must be amended.

No software was installed, no money spent, no persistent host change made, and zero manual
interventions were required.

### Decision status

| Item | State |
|---|---|
| ADR-0013 to ADR-0020 | **All `Proposed`. None accepted.** ADR-0015 and ADR-0017 carry recorded corrections that must be applied before review. |
| Evidence Loop PoC | **Not authorised.** Four of five Gate 0 checks are blocked on the same precondition. |
| Track A / Track B first-product decision | **Still open**, and must not be settled by argument — its deciding input (G0-1) is blocked. |
| Documentation language | [ADR-0020](adr/ADR-0020-documentation-language.md) now **proposes** a `CONTRIBUTING.md` amendment instead of the current silent override. |
| Application tests, benchmarks, compatibility rate | **Still none. Still unknown.** |

### Next step

One decision from the maintainer: **whether to authorise a lab**, and where it runs. That single
item unblocks four of the five Gate 0 checks. WSL2 can unblock the Wine-mechanics checks; it cannot
unblock the integration or graphics questions, and results obtained there must not be presented as
if it could.


---

## Update 2026-09-07 (later) — bounded lab phase executed

Owner authorised a bounded Gate 0 lab phase. Full detail:
[Gate 0 report](experiments/EXP-009-GATE0-REPORT.md) and
[lab runbook](experiments/LAB-G0-RUNBOOK.md).

### Language policy — decided

[ADR-0020](adr/ADR-0020-documentation-language.md) is **Accepted** with named owner approval, and is
the authoritative specification for the topic. English is primary for new technical specifications,
ADRs, RFCs, experiment reports and developer-facing instructions; existing Serbian material is
preserved as history and originating requirements; the repository is **not** translated now; and
each topic has exactly one authoritative document that others link to. `CONTRIBUTING.md` now
summarises this and points at the ADR. **No architecture ADR was accepted.**

### Lab provisioned

The preferred VM path was **not available**: Hyper-V's role is enabled but the account lacks
permission, and creating a VM would require a host administrator change that the authorisation
excludes. The authorised fallback was used — a single disposable WSL2 distribution `helm-lab-g0`,
isolated from Windows drives and interop, running as an unprivileged user. Host changes: one new
directory and one new WSL distribution. Nothing else.

### Results: 3 PASS, 1 PARTIAL, 3 BLOCKED, 0 FAIL

| Check | Outcome |
|---|---|
| G0-3a filesystem mechanics | PASS (round 1; reported as a newly added subtest, **not** as satisfying the originally registered criterion) |
| G0-3b Wine registry behaviour | **PASS with real Wine 11.17** — hardlink copy contaminated, quiesced full copy independent, both controls correct |
| G0-2 DirectComposition | **PARTIAL** — vanilla Wine returns E_NOTIMPL; the staging and Proton arms were not tested, so the comparison is unresolved |
| G0-1, G0-4, G0-5 | BLOCKED — need authorised accounts, a desktop session, and a reviewed containment boundary respectively |

**The most consequential result:** a synthetic user document created after the recovery point did
**not** survive a whole-prefix restore. That is the failure ADR-0017 rule 2 exists to prevent, now
demonstrated rather than argued.

### Decision status

| Item | State |
|---|---|
| ADR-0020 | **Accepted** 2026-09-07 (documentation language only) |
| ADR-0013 to ADR-0019 | **All still `Proposed`.** ADR-0017 amended on owner instruction and is now the best-evidenced; ADR-0015 still needs its novelty claim narrowed. |
| Evidence Loop PoC | **Still not authorised.** Its central thesis needs real applications and a desktop session. |
| Track A / Track B | **Still open.** G0-1 stayed blocked; a lab does not supply authorised accounts or a reproduction. |
| Application compatibility rate | **Still unknown.** No Windows application was installed or run. |

### Next step

Finish G0-2's remaining two runtime arms in a second disposable lab (about an hour), take amended
ADR-0017 to review, and decide the environment for application-level work — a desktop session on
real hardware, or the Hyper-V group approval recorded in the runbook.

---

<a id="publication-review"></a>

## Update 2026-09-07 — publication/provenance review

The owner authorised deterministic privacy redaction of exactly the three unpublished Gate 0
commits. A private bundle and original evidence were archived outside the repository and verified
before editing. Public base `e09a52fa2ed95c759e3e9370d4b0f0e95ce1fa20` is unchanged. The actual
repository is [Djomla83/helm-os](https://github.com/Djomla83/helm-os); older statements that no remote
exists are historical. The private publication receipt records the old/new SHAs and remote
verification after a normal fast-forward push. No force-push is authorised.

The two G0-3a evidence files now identify their redacted fields and raw hashes; their publication
hashes are distinct. [Provenance and artifact identities](experiments/EXP-009-GATE0-REPORT.md#publication-provenance)
are recorded without exposing the private archive or operator account component.

Current Gate 0 status is defined in the [report](experiments/EXP-009-GATE0-REPORT.md#current-assessment):
the original five checks remain BLOCKED. G0-3a and G0-3b separately retain PASS for their recorded
filesystem/recovery mechanics; neither completes original G0-3. Vanilla G0-2 preserves
`0x80004001` (`E_NOTIMPL`) but is INCONCLUSIVE because required controls are not evidenced. Staging
and Proton/UMU remain NOT_RUN. No application-class compatibility inference follows.

The lab configuration intends to disable interoperability, while the observed `WSLInterop`
registration reports `enabled`. Effective execution blocking is UNVERIFIED; no direct negative
execution test was run. See the [lab observation](experiments/LAB-G0-RUNBOOK.md#interop-observation).

ADR-0020 remains Accepted for documentation language; ADR-0013 through ADR-0019 remain Proposed.
No probe, lab package, global WSL setting or Hyper-V permission was changed. The authorised work
stops at publication/provenance repair. Any further experiment or full desktop VM needs a separate
owner decision; the Evidence Loop PoC remains unauthorised.

<a id="g0-2-completion"></a>

## Update 2026-09-07 — bounded G0-2 completion

Starting public baseline `ca0aa5ae26a1d3b630f63eda8f87626d125248d1` matched local main and
origin/main after fetching, with a clean tree and passing validation. The owner authorised only
G0-2 completion. Controls were committed before execution; the target source and executable
remained unchanged. Full [results and evidence](experiments/EXP-009-GATE0-REPORT.md#current-assessment)
are authoritative.

| Runtime | Controlled observation | Arm verdict |
|---|---|---|
| Vanilla WineHQ `11.17~noble-1` | `0x80004001` (`E_NOTIMPL`); both controls correct | PASS, valid measurement |
| WineHQ staging `11.16~noble-1` | `0x00000000` (`S_OK`); both controls correct | PASS, valid measurement |
| UMU 1.4.4 / UMU-Proton-10.0-4 / sniper `3.0.20260805.254768` | `0x80004001`; both controls correct in the documented DOS-path repetition | PASS, valid measurement |

Overall G0-2 is **PASS for the narrow comparison**, not application compatibility or rendering.
Historical vanilla stays INCONCLUSIVE. The first Proton sequence also stays INCONCLUSIVE because
console JSON was absent; its failed capture and the preregistered methodological repetition are
both retained. No result was retried merely because its HRESULT was undesirable.

Direct Windows-executable launch attempts in the original lab and both authorised clones failed
with interop connection error/exit 1 and no marker. Configuration stayed disabled despite the
shared binfmt registration reading enabled. This is a specific execution observation, not a
containment certification. `helm-lab-g0`, `helm-lab-g0-staging` and `helm-lab-g0-proton` are retained
and stopped. Normal Ubuntu, global WSL configuration and Hyper-V permissions were unchanged.

The original G0-3 remains BLOCKED; its separate mechanics subtests remain narrowly PASS.
G0-1/G0-4/G0-5 remain BLOCKED and were not attempted. ADR-0013 through ADR-0019 remain Proposed;
ADR-0020 remains Accepted for documentation language only. No OS, desktop, application catalogue,
SDK, language, App Forge or Evidence Loop PoC development was started.

Retained VHD allocation grew by 46.33 GB, including the common private baseline export. Probe
execution totalled about 49 seconds across four sequences; engineering/debugging and publication
work are accounted separately. Public copies redact UMU's host-name diagnostic and local account
paths, preserving private originals and separate digests. A local privacy-checkpoint mistake was
repaired before publication; no public commit was rewritten.

Recommended owner decision: review G0-2, then separately authorise a full Linux desktop VM
environment. Rendering and application experiments require their own bounded approval and
preregistration. Work stops at the Gate 0 review boundary.

<a id="application-baseline-preparation"></a>

## Update 2026-09-07 — G0-2 accepted narrowly; application baseline prepared

The owner accepted G0-2 only for its controlled HRESULT comparison at public commit
`3f947888067243ba3cedcc3f77916344d9d10a25`. This accepts no architecture ADR, rendering result or
larger Evidence Loop PoC. Fetch found that same clean local/remote state; no history was reset
or rewritten. Work uses the dedicated `experiment/exp009-app-baseline` branch.

The separately authorised [EXP-009 A0-7ZIP baseline](experiments/EXP-009.md#application-baseline)
covers one Ubuntu 24.04 desktop VM and two planned Windows x64 7-Zip GUI workflows, before/after
a guest restart. **Current overall result: BLOCKED on Hyper-V permissions.** The service exists
and runs, but the current account lacks group membership and VM/switch queries are denied.
An exact owner-only administrator action is prepared privately, not executed by the agent.

[Preparation report](experiments/EXP-009-APP-BASELINE-REPORT.md): official 7-Zip 26.03 installer
downloaded/hashed, Ubuntu 24.04.4 desktop and vanilla Wine `11.17~noble-1` artifacts pinned for
later verification, conservative 50 GiB storage plan on D:, synthetic fixtures and a tested ZIP
oracle. No VM was created; no application, guest GUI workflow or restart ran. No host security,
Hyper-V group, global WSL setting or prior lab was changed. The three WSL labs stay stopped.

The host helper's checks do not count as guest application results. Installation, GUI coverage,
output correctness and post-restart operation remain unevidenced. G0-1, original G0-3, G0-4 and
G0-5 remain BLOCKED and were not attempted. All architecture ADRs remain Proposed; ADR-0020 stays
Accepted for documentation language. Stop at non-invasive preparation pending actual VM access.

Definition/helper commit `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae` precedes the labelled host oracle
run: known-good accepted, corrupted payload rejected. No application workflow has been executed;
the planned coverage remains 0 of 2 GUI executions. The verifier and all 20 documentation-validator
tests pass, with 15 additional helper tests. These structural/helper checks do not unblock Hyper-V.

<a id="application-baseline"></a>

## Update 2026-09-07 — A0-7ZIP desktop execution completed for review

The owner performed the previously prepared Hyper-V group action. Resumption fetched and verified
clean local main/origin/main at `7b9ec6f91d33d44578179da467cb01f4da55ea2c`, preserving the published
definition at `5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae`. Effective membership and read-only VM/switch
queries passed in the non-elevated session. The agent changed no host permissions.

**Current A0-7ZIP result: FAIL against the registered two-workflow protocol.** In W1 the agent
prematurely submitted the GUI Add dialog and created a correct ZIP in `inputs/fixture.zip`, leaving
the required before-restart destination absent. That failure was preserved and not retried or
relabelled. The originally planned W2, after a normal guest restart, passed at its fresh destination.
Both actual ZIP contents matched the frozen synthetic tree, and good/corrupted guest controls
behaved correctly before and after restart. This does not establish application incompatibility;
it leaves the complete two-workflow compatibility claim inconclusive because of the actuation error.

One Ubuntu 24.04.4 Generation 2 VM was created: 4 vCPUs, fixed 8 GiB RAM, dynamic 32 GiB VHDX,
Secure Boot enabled and the existing Default Switch. Verified WineHQ vanilla `11.17~noble-1` and
official Windows x64 7-Zip 26.03 were used under unprivileged `helmlab`. Installed executable hashes
were unchanged after restart. Actual GUI coverage was GNOME 46 Wayland console with Wine's X11
driver through XWayland, llvmpipe software rendering, 1024×768 and scale 1.0.

The [current report](experiments/EXP-009-APP-BASELINE-REPORT.md#current-assessment) and
[execution evidence](experiments/evidence/app-baseline-execution-2026-09-07/INDEX.md) preserve all
failures, raw/public hashes, package inventories and measured effort. The VM is stopped and retained;
all three old project WSL labs and normal Ubuntu remain stopped. D: project file lengths total
20.175 GiB; C:/D: retain 79.992/513.920 GiB free. No checkpoint, disk-cap increase, new switch,
host sharing, security downgrade, runtime fallback or host restart was performed.

G0-1, original G0-3, G0-4 and G0-5 remain BLOCKED; G0-2 remains accepted only for the controlled
HRESULT comparison. Architecture ADRs remain Proposed; ADR-0020 remains Accepted for documentation
language. No product module or larger PoC was implemented. Stop at A0 review. A proposed next bounded
module is a read-only evidence-bundle completeness verifier for this exact workflow; owner
authorisation is still required before implementing it.
