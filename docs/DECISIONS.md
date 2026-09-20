# Indeks arhitektonskih odluka

ADR-0020 is Accepted for documentation language. ADR-0022 is Accepted on 2026-09-08
with a bounded 0.1 scope on evidence from OBS-FS-01; it accepts an architecture boundary,
not an implementation, and excludes the descendant bind-mount case as unverified. On
2026-09-09 the owner clarified it, without changing its status or widening its scope:
filesystem cohort membership is an external support precondition, not an observation
attestation, and root admission applies only the necessary mechanism guards available
inside the explicit capability boundary. The supported cohort stays Linux x86_64 ext4.
On the same day the owner accepted the independently reviewed experimental helm-observe 0.1
implementation and fast-forwarded main to it; that records product-module acceptance under
ADR-0022, not another architecture decision, and expands none of its limits. ADR-0021 is Accepted on 2026-09-08
by repository owner Djomla83 only for building a pure, non-executable application
specification/validation library before observation or lifecycle execution.
Its schema/API remain experimental; a later separate owner instruction approved the
[independently corrected implementation](implementation/HELM-APP-SPEC-INDEPENDENT-REVIEW.md),
which is now merged as helm-app-spec 0.1. This records product-module acceptance,
not another architecture decision.
ADR-0023 is Accepted on 2026-09-09 for desired-versus-observed binding, with bounded
pre-implementation corrections and a 0.1 architecture scope. It fixes overall-verdict
semantics with no satisfaction, compatibility or readiness verdict, mapping semantics,
cross-crate dependencies and the identity graph, and it authorised no implementation.
Implementation was authorised separately, and on 2026-09-09 the owner accepted the
independently reviewed experimental helm-bind 0.1 and fast-forwarded main to it. That records
product-module acceptance under ADR-0023, not another architecture decision; it widens no
scope, stabilises no schema or API, and issues no satisfaction, compatibility or readiness
verdict. ADR-0024 is **Proposed only** on 2026-09-09 for launch/execution authority; it
authorises no implementation, no experiment execution and no crate, and it is the first
proposed decision that would introduce actual process execution. On the same day a bounded
three-workstream **pre-execution review** audited it, the design report and the LAUNCH-EXEC-01
preregistration in isolated parallel contexts, and returned sixteen BLOCKER findings among 53 without
changing its status: it narrowed four false claims — the executable digest is a **pre-execution
measurement** and not the identity of the body that ran, the caller-credentials claim fails for a
set-user-ID object, clean EOF does not prove exec, and direct-child lifecycle does not imply
direct-child liveness — and re-froze the experiment at **71 cases, still NOT_RUN**. Later the same
day the owner **decided ten of the eleven design questions** — D-1 arm (i) (scoped unsafe backend,
FD isolation not weakened), D-2 to D-6, D-8, **D-9** (refuse set-user-ID and set-group-ID objects
at admission), **D-10** (the child environment is exactly empty; `explicit` removed from 0.1) and
a new **D-11** (`PR_SET_NO_NEW_PRIVS` before exec, because D-9 does not cover file capabilities)
— and authorised **pre-trial experiment implementation only**. The definition is re-frozen at
**72 cases** with a machine-readable manifest as its source of truth, and the disposable
experiment sources are committed and hashed. **ADR-0024 stays Proposed**, no implementation of
`crates/helm-launch` is authorised, **D-7 (execution) is not granted**, and **LAUNCH-EXEC-01
remains NOT_RUN**: deciding the design questions an ADR depends on is not accepting the ADR.
ADR-0001 through ADR-0019 remain Proposed. No other architecture acceptance follows from this
decision. On 2026-09-17, after the LAUNCH-EXEC-01 trial line had closed and the owner had reviewed
the productization plan, the owner **accepted the revised ADR-0024** for the helm-launch 0.1
architecture only and authorised **HELM-LAUNCH P1 only**, a portable, pure model with no process
execution and no `unsafe`; see the
[decision of 2026-09-17](#adr-0024-accepted-helm-launch-p1-authorised). P1 was implemented,
independently reviewed and [accepted the same day](#helm-launch-p1-accepted). On 2026-09-18 the
owner authorised the bounded **HELM-LAUNCH P2** slice — safe Linux x86_64 capability admission and
authorisation composition, with no process creation, no process execution and no `unsafe` — while
**P3, P4 and P5 remain not authorised**; see the
[decision of 2026-09-18](#helm-launch-p2-authorised). P2 was implemented, independently reviewed,
published against hosted CI and [accepted the same day](#helm-launch-p2-accepted). On the same day
the owner authorised **HELM-LAUNCH P3** — the scoped-`unsafe` Linux x86_64 process-creation backend
and the closed post-clone child contract, internally only; see the
[decision of 2026-09-18](#helm-launch-p3-authorised). P3 was implemented, reviewed by a full
independent unsafe review and four bounded independent correction reviews, and published three
times: the [first](#helm-launch-p3-publication-failure-disposition) and
[second](#helm-launch-p3-second-publication-failure-disposition) hosted validations **failed**, on
`P3R-20` and then on `P3R-21`, and both failures stay as permanent evidence. The third publication,
at `8a359ee4215b6c803dcc5b527010612dabbd110b`, passed every load-bearing hosted gate on its first
natural run, and P3 was [accepted on 2026-09-19](#helm-launch-p3-accepted). On the same day the
owner authorised **HELM-LAUNCH P4** — the lifecycle, termination, public `launch` and real receipt
slice; see the [decision of 2026-09-19](#helm-launch-p4-authorised). P4 was implemented as a
candidate and independently reviewed on 2026-09-20; that review returned **0 BLOCKER and 7
IMPORTANT**, so **P4 publication is blocked** and a bounded correction is ordered; see the
[disposition of 2026-09-20](#helm-launch-p4-independent-findings-disposition). **P4 is authorised,
not accepted and under correction, P5 is still not authorised**, and the complete helm-launch 0.1
module is **not yet product-accepted**.

| ID | Odluka | Status |
|---|---|---|
| ADR-0001 | [Početi od postojećeg Linux user-space okruženja](adr/ADR-0001-linux-userspace-first.md) | Proposed |
| ADR-0002 | [Odvojiti binarnu kompatibilnost, native dostupnost i VM](adr/ADR-0002-compatibility-metrics.md) | Proposed |
| ADR-0003 | [Ponovno koristiti upstream komponente i ograničiti fork](adr/ADR-0003-upstream-first.md) | Proposed |
| ADR-0004 | [App Forge prvo kao priprema i validacija okruženja](adr/ADR-0004-appforge-orchestration.md) | Proposed |
| ADR-0005 | [Prefix nije sigurnosna granica](adr/ADR-0005-sandbox-boundary.md) | Proposed |
| ADR-0006 | [Verzionisani runtime uz održavanje i revokaciju](adr/ADR-0006-versioned-runtime-lifecycle.md) | Proposed |
| ADR-0007 | [Koristiti postojeći desktop u prvoj fazi](adr/ADR-0007-reuse-desktop-first.md) | Proposed |
| ADR-0008 | [SDK graditi na standardnom toolchain-u i Linux ABI-ju](adr/ADR-0008-sdk-standard-target.md) | Proposed |
| ADR-0009 | [Novi jezik i generalni semantički IR odložiti](adr/ADR-0009-language-deferred.md) | Proposed |
| ADR-0010 | [Korisnički AI je opcion i ne određuje autorizaciju](adr/ADR-0010-ai-optional-policy.md) | Proposed |
| ADR-0011 | [Windows VM ostaje posebna opciona putanja](adr/ADR-0011-vm-separate.md) | Proposed |
| ADR-0012 | [Javna dokumentacija mora odvojiti plan od rezultata](adr/ADR-0012-public-docs-evidence.md) | Proposed |

Sledeći zapisi su predloženi na osnovu [foundation audit-a](research/FOUNDATION_AUDIT.md)
i pisani su na engleskom po uputstvu vlasnika. Nijedan ne menja prethodne ADR-ove;
tenzije sa postojećim odlukama navedene su u
[poglavlju 10 audit-a](research/FOUNDATION_AUDIT.md#s10).

| ID | Odluka | Status |
|---|---|---|
| ADR-0013 | [Build, pin and ship HELM's own Wine runtime](adr/ADR-0013-pinned-wine-runtime.md) | Proposed |
| ADR-0014 | [Profiles are keyed on a version-scoped triple](adr/ADR-0014-version-scoped-profiles.md) | Proposed |
| ADR-0015 | [Evidence must be reproducible, gating, and must expire](adr/ADR-0015-evidence-expiry.md) | Proposed |
| ADR-0016 | [The application boundary runs on the host](adr/ADR-0016-host-side-sandbox.md) | Proposed |
| ADR-0017 | [Recovery is tiered, quiesced and reflink-based](adr/ADR-0017-data-safety-rules.md) | Proposed |
| ADR-0018 | [Integration via a Win32-to-portal bridge](adr/ADR-0018-win32-portal-bridge.md) | Proposed |
| ADR-0019 | [Publish the unsupportable class; decide the first product with evidence](adr/ADR-0019-scope-boundary.md) | Proposed |
| ADR-0020 | [Documentation language policy](adr/ADR-0020-documentation-language.md) | **Accepted 2026-09-07** |

Historical annotation, 2026-09-07: ADR-0017 was amended but remains Proposed; ADR-0015 still needs
review of its recorded corrections. ADR-0020 alone is Accepted, for documentation language.
The original G0-3 criterion remains BLOCKED; the separate mechanics subtest does not complete it.
See the [current Gate 0 assessment](experiments/EXP-009-GATE0-REPORT.md#current-assessment).

Proces je u [master planu](../HELM_MASTER_PLAN.md#s31). Licencna odluka ostaje zaseban uslov javnog open-source izdanja.

| ID | Decision | Status |
|---|---|---|
| ADR-0021 | [Make the second product module an inert application contract](adr/ADR-0021-second-product-module.md) — [selection analysis and owner refinements](research/SECOND-PRODUCT-MODULE-SELECTION.md); bounded architectural authority, with experimental helm-app-spec 0.1 now accepted on main | **Accepted 2026-09-08** |
| ADR-0022 | [Observe explicit targets without interpreting desired state](adr/ADR-0022-observation-authority.md) — [architecture analysis](research/HELM-OBSERVE-ARCHITECTURE.md), [independent review with Amendment 1](implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md), [execution definition](experiments/obs-fs-01/) and [OBS-FS-01 PASS](experiments/OBS-FS-01-EXECUTION-REPORT.md). **Bounded acceptance**: Linux x86_64 and ext4 cohort only; descendant bind mounts excluded and unverified; three implementation-binding obligations, all met and independently verified on the review branch. [Owner clarification 2026-09-09](adr/ADR-0022-observation-authority.md#cohort-attestation-clarification): cohort membership is a caller precondition, not an observer attestation; `0xEF53` is a necessary ext-family guard only; no new authority added; scope unchanged. **Experimental helm-observe 0.1 is implemented, independently reviewed and owner-merged to main on 2026-09-09** at reviewed tip `626de914000263cc3206d28479db692ad0b40724`; schema and API unstabilised, `publish = false`, not a release | **Accepted 2026-09-08, clarified 2026-09-09** |
| ADR-0023 | [Compare desired claims with actual observations without producing a satisfaction verdict](adr/ADR-0023-binding-authority.md) — [design report](research/HELM-BIND-ARCHITECTURE.md). A pure, authority-free `helm-bind` 0.1 with no satisfaction, compatibility or readiness verdict, a two-axis result whose coverage can never be complete in 0.1, an explicit identity-bearing binding plan instead of target-ID heuristics, and no change to any existing crate. Owner corrections of 2026-09-09: both entry-point comparators require `regular_file_sha256` and bind the desired path byte for byte; the caller assertion is named `asserted_prefix_root_id` and is conditional; the claim universe excludes contextual metadata and selector keys; the coverage theorem is at least four unsupported claims. **Experimental helm-bind 0.1 is implemented, [independently reviewed](implementation/HELM-BIND-INDEPENDENT-REVIEW-0.1.md) and owner-merged to main on 2026-09-09** at reviewed tip `4f51c1b2bc7e59b8142ccc4c328e2641c89327d3`, product-code tip `78e26de4ca952b7125032e5c9fa468e6dc85af7c`; no BLOCKER and no unresolved IMPORTANT; unsupported coverage stays at least four; `asserted_prefix_root_id` stays an assertion, never an attestation; schema and API unstabilised, `publish = false`, not a release | **Accepted 2026-09-09** |
| ADR-0024 | [Execute one explicitly authorized object without granting authority from comparison](adr/ADR-0024-launch-authority.md) — [design report and falsification plan](research/HELM-LAUNCH-ARCHITECTURE.md), [LAUNCH-EXEC-01 preregistered definition](experiments/LAUNCH-EXEC-01-DEFINITION.md), **NOT_RUN**. A single-crate, Linux x86_64, capability-driven launcher for exactly one already-open regular ELF object, with no satisfaction, compatibility, readiness or success verdict. Parsing a LaunchPlan and holding a BindingReport both grant **zero** execution authority; `NoClaimContradicted` is never permission. Direct-child lifecycle only, **no process-tree containment**, and **not a sandbox** — the child runs with the caller's own credentials. Depends on no HELM crate; context travels as opaque digests. Requires a scoped `unsafe` backend or a weaker descriptor claim (owner decision D-1), because the workspace `forbid(unsafe_code)` cannot be locally relaxed and rustix provides no `close_range`. **Narrowed on 2026-09-09 by the [three-workstream pre-execution review](implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md)**, sixteen BLOCKERs among 53 findings, classification NEEDS_ARCHITECTURE_OWNER_REVIEW: the executable digest is a pre-execution measurement of the main file body (`pre_exec_body_sha256`) and never the identity of the body that ran, `ETXTBSY` does not cover the measure-to-exec window, the child runs with the caller's credentials **except** for a set-user-ID or capability-bearing object, clean EOF does not prove exec, direct-child lifecycle does not imply direct-child liveness, and the receipt becomes a product of one process disposition and one per-stream completeness. LAUNCH-EXEC-01 re-frozen at **71 cases** with a total aggregate precedence. **Owner decisions of 2026-09-09**: D-1 **arm (i)** (scoped unsafe backend; FD isolation not weakened to keep crate-wide `forbid`), D-2 to D-6 and D-8 accepted (D-4 as corrected), **D-9** refuse `S_ISUID`/`S_ISGID` at admission, **D-10** environment **exactly empty** with `explicit` removed from 0.1, and new **D-11** `PR_SET_NO_NEW_PRIVS` before exec because D-9 does not cover file capabilities. Definition re-frozen at **72 cases** (56 mandatory / 9 conditional / 7 recorded) against a machine-readable manifest, with the disposable [experiment sources](experiments/launch-exec-01/) committed and hashed. **Pre-trial implementation only: D-7 is NOT granted, LAUNCH-EXEC-01 remains NOT_RUN, and `crates/helm-launch` must not be created before it has run and been reviewed**. The text of this row up to here records the state of 2026-09-09. **[Owner decision 2026-09-17](#adr-0024-accepted-helm-launch-p1-authorised): the revised ADR-0024 is Accepted** for the helm-launch 0.1 architecture only; Trial #3 remains `MECHANISM_REJECTED`, no Trial #4; **HELM-LAUNCH P1 only** is authorised, P2+ is not. **[2026-09-18/19](#helm-launch-p3-accepted):** P2 and P3 were each authorised, implemented, independently reviewed and **accepted**; **P4 and P5 remain not authorised** and the complete helm-launch 0.1 module is not yet product-accepted | **Proposed 2026-09-09; revised 2026-09-16; Accepted 2026-09-17** |


<a id="d-7-authorised"></a>

### Owner decision 2026-09-10 — **D-7 AUTHORISED** for exactly one LAUNCH-EXEC-01 trial

**D-7 — execution authorisation — is GRANTED**, bound to one exact frozen object and to **exactly
one valid trial**.

| Binding | Value |
|---|---|
| Freeze commit | `89c923a147ff16182d4d0ae14a0bd7bb6e62723d` |
| `SOURCE-HASHES.json` SHA-256 | `9fb861602d477a00f014f14f0a31b5979947af18fa2b620b95ac803ab5bfc365` |
| `SOURCE-HASHES.json` Git blob | `4561daf32acb398ec6a601acbe7e986bce0b1105` |
| Final independent review tip | `64f7d94225e17fac3d1cd0c7aedf564f9ed9295b` |
| Pretrial classification | `READY_FOR_OWNER_D7` |
| Compile-only validation | run `34491770021` SUCCESS; Rust workspace run `34491769999` SUCCESS |
| Scope | **exactly one** valid LAUNCH-EXEC-01 trial |

**The frozen source, definition and manifest remain immutable.** This authorisation is recorded
here, outside the freeze, precisely so that granting it changes no frozen byte.
`d7_execution_authorised: false` inside `SOURCE-HASHES.json` is left untouched: that field records
the state at which the immutable freeze was cut, and this external decision supersedes it for
execution authority without mutating the artefact it authorises.

Authorised only for: the exact frozen candidate above, the exact preregistered definition, the
exact 72-case membership (54 mandatory / 11 conditional / 7 recorded), the eight traced cases
E1 E7 F4 F7 M1 M2 M3 M4, the preregistered trial environment and preflight contract, and one valid
trial.

**Not authorised:** modifying the frozen experiment before or during the trial; a different freeze
SHA; retries to obtain a better result; rerunning failed, invalid or blocked cases; Wine, Proton,
A0, 7-Zip or another experiment; creating `crates/helm-launch`; accepting ADR-0024; or changing
checker or verdict semantics after observing results.

The first execution of the first preregistered case is the **immutability boundary**. A preflight
HALT poses zero cases and does not consume it. Any re-run is a **new trial** with its own report.
After the trial, the result requires an independent review before ADR-0024 or `crates/helm-launch`
may advance.


<a id="trial-001-accepted"></a>

### Owner decision 2026-09-10 — **LAUNCH-EXEC-01 Trial #1 accepted and CLOSED**

The owner accepts the
[independent Trial #1 result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-001-RESULT-REVIEW.md).

| | |
|---|---|
| Trial | **trial-001**, GitHub run `34500901306`, run number 1, attempt 1 |
| Frozen candidate executed | `89c923a147ff16182d4d0ae14a0bd7bb6e62723d` |
| Result-review commit | `f7e5ca07a33ca3ed8365ee0f015bcc5c2a8affa6` |
| Preflight | **PREFLIGHT_PASSED** |
| Immutability | **IMMUTABILITY_BOUNDARY_CROSSED** |
| Aggregate | **AGGREGATE_NOT_DERIVABLE_FROM_FROZEN_EVIDENCE** |
| Status | **TRIAL_ABORTED_AFTER_BOUNDARY** |
| D-7 | **D7_AUTHORIZATION_CONSUMED** |

Trial #1 started, passed its mandatory preflight, crossed the immutability boundary, and aborted
during E5b's fixture setup with `OSError: [Errno 9] Bad file descriptor` — the harness read back
through a write-only descriptor. E1 through E5 were genuinely posed and scored, but their records
lived only in memory and were lost with the runner, so their statuses are permanently
**`UNKNOWN_FROM_PRESERVED_EVIDENCE`**.

**The owner explicitly rejects manufacturing a frozen aggregate by treating every missing record as
INVALID.** INVALID means *could not be posed*; E1–E5 were posed. No aggregate verdict exists for
Trial #1, and none may be reconstructed after the fact.

**Trial #1 is closed and is historical evidence.** It must not be re-run, resumed or patched in
place, and its freeze and `SOURCE-HASHES.json` are unchanged. Its D-7 authorisation is consumed and
must not be reused; the dispatcher bound to it is retired from the default branch.

Trial #2 is a **new preregistered trial** under a new descendant freeze — not a retry — and requires
its own D-7 authorisation, which is **not granted**.


<a id="d-7-authorised-trial-002"></a>

### Owner decision 2026-09-11 — **Trial #2 D-7 AUTHORISED** for exactly one valid LAUNCH-EXEC-01 Trial #2 execution

**D-7 — execution authorisation — is GRANTED for LAUNCH-EXEC-01 Trial #2**, bound to one exact
frozen object and to **exactly one valid Trial #2 execution**. It is a new authorisation. Trial #1's
consumed D-7 is not reused, and the Trial #1 section above is unchanged.

| Binding | Value |
|---|---|
| Trial | **trial-002** |
| Freeze commit | `ba41a3f12be411058ed50e78bcd1c7e22afb7ae4` |
| `SOURCE-HASHES.json` Git blob | `6f000fac9a48625d3c9def18e16ae7ce1b61efa8` |
| `SOURCE-HASHES.json` SHA-256 | `616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94` |
| Final independent review | `5da397c6a79aea7c9a626789480903d50df0b7b4` — [record](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-BA41A3F-FINAL-REVIEW.md) |
| Review classification | `TRIAL_2_READY_FOR_NEW_D7_DECISION` |
| Compile-only evidence | Build 7, run `34575558065` at `f417984`; every C source of this freeze is byte-identical to what it compiled. It is pretrial evidence only: the trial builds and hashes its own binaries |
| Scope | **exactly one** valid Trial #2 execution |

**The frozen source, definition and manifest remain immutable.** This authorisation is recorded
here, outside the freeze, so that granting it changes no frozen byte. `status: NOT_RUN` and
`d7_execution_authorised: false` inside `SOURCE-HASHES.json` are left untouched. They record the
state at which the immutable freeze was cut, and this external decision supersedes them for
execution authority without mutating the artefact it authorises.

**When D-7 is consumed.**

* D-7 is consumed when the **first durable `case_pose_started` record is successfully fsynced** —
  the Trial #2 immutability boundary of definition section 9.3 and of the manifest's
  `durable_evidence.journal.boundary`. That record is written before the launcher process exists,
  so a death between the record and the process still counts as the execution.
* If the trial aborts after that boundary, **D-7 remains consumed**. The trial is then read from
  its preserved journal as section 9.3 fixes: `TRIAL_ABORTED_AFTER_BOUNDARY`, with no aggregate
  derived after the fact.
* **No rerun, no resume and no retry.** No case, no failed or invalid case, and no aborted trial is
  run again under this authorisation. Any later execution is a new trial that needs a new owner
  decision.
* **A pre-boundary rejection does not manufacture a trial result.** A refused dispatch, a failed
  identity, freeze or completeness gate, `HALT_PREFLIGHT` or `HALT_BUILD_IDENTITY` poses zero
  cases. It produces no case status, no aggregate and no change to the valid trial count, and it
  does not consume D-7. The one-shot dispatcher, however, accepts only its own run number 1, so
  after such a rejection it cannot be dispatched again. Any further step is an owner decision.

**Authorised only for:**

* the exact freeze, manifest and final review above;
* the exact preregistered definition;
* the exact 72-case membership (54 mandatory / 11 conditional / 7 recorded);
* the eight traced cases E1 E7 F4 F7 M1 M2 M3 M4;
* the preregistered `ubuntu-24.04` trial environment and preflight contract;
* one run of the frozen runner through one reviewed one-shot dispatcher.

**Not authorised:**

* another freeze, or a different freeze SHA;
* arbitrary HELM execution;
* Trial #1, or any rerun, resume or retry to obtain a better result;
* rerunning failed, invalid or blocked cases;
* Wine, Proton, A0, 7-Zip or another experiment;
* privileged host operations;
* modifying frozen experiment bytes before or during the trial;
* changing checker or verdict semantics after observing results;
* creating `crates/helm-launch`, or accepting ADR-0024.

After the trial, its result needs an independent result review before ADR-0024 or
`crates/helm-launch` may advance. A red or green workflow is not a verdict: the result comes only
from the frozen durable evidence.


<a id="trial-002-postmortem-decisions"></a>

### Owner decision 2026-09-12 — Trial #2 result accepted; bounded postmortem scope selected

The owner accepts the
[independent Trial #2 result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md).
This acceptance preserves the historical result exactly: run `34640280964`, freeze
`ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`, 59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED, aggregate
`MECHANISM_REJECTED`, one valid Trial #2, and consumed D-7. Nothing here changes a frozen status,
reason, aggregate, checker, case definition, evidence file or result review.

These are **prospective correction decisions**. They authorise only the bounded static/process
diagnostics named below. They do not authorise a correction implementation, a Trial #3 freeze, a
Trial #3 execution or a Trial #3 D-7.

1. **Correction scope.** A future correction may address only Trial #2's demonstrated issues:
   `X2b`, `X2c`, `X4`, `T1`, `S4`, `M2`, `E4`, `E6`, `E6c`, `O6`, `O7`, `R3`, and the shared
   liveness-evidence defect affecting interpretation of `P1`, `P2` and `P4`. `N3` remains a
   separate conditional case. No unrelated case or backlog is reopened unless a correction
   demonstrates a new reachable dependency.
2. **S4 product semantics.** HELM launch 0.1 remains conservative: clean exec-status EOF alone is
   not positive proof that an executable image ran. Independent test evidence may pose a future
   S4 case, but it cannot make the launcher receipt claim an observation the launcher did not make.
   The future S4 contract must be rewritten prospectively around this rule; the launcher must not
   infer exec success from EOF alone.
3. **E6c remains recorded and in scope.** Its shared-writable-mapping question remains useful.
   Future work corrects its posing machinery and does not delete or pre-answer the case.
4. **P1/P2/P4 evidence standing.** Their Trial #2 statuses remain historical PASS, but those records
   are not positive architectural evidence for descendant survival or death because the relative
   liveness-FIFO path made the observation non-probative. A future valid trial must re-observe all
   three with the corrected fixture. This path defect alone does not invalidate P3, whose frozen
   result is the launcher's sweep observation.
5. **N3 remains conditional.** No privileged GitHub or self-hosted execution is introduced now.
   Privileged-transition testing is deferred to a future controlled privileged environment; the
   Trial #2 BLOCKED status remains valid historical evidence.
6. **One bounded pre-correction diagnostic pass is authorised.** It is limited to static inspection
   of the E6/E6c marker-location mechanics and an isolated `waitid`/`CLD_DUMPED` process probe for
   R3. It is not Trial #3, uses no D-7, poses no LAUNCH-EXEC case, and may not execute
   `launcher_spike` or any HELM helper.

Settled prospective correction requirements:

* `X2b/X2c/X4_FIXTURE_EXEC_MODE_CORRECTION_REQUIRED`
* `T1_FROZEN_EXPECTATION_CORRECTION_REQUIRED`
* `S4_CONSERVATIVE_EXEC_EVIDENCE_POLICY_SELECTED`
* `M2_PROCESS_CLONE_CORRELATION_CORRECTION_REQUIRED`
* `E4_PATH_CORRECTION_REQUIRED`
* `O6_O7_ABSOLUTE_FIFO_CORRECTION_REQUIRED`
* `P1_P2_P4_LIVENESS_REVALIDATION_REQUIRED`
* `N3_PRIVILEGED_TEST_DEFERRED`

**Trial #2 is immutable and closed. No Trial #3 exists, and no D-7 exists for Trial #3.**


<a id="d-7-authorised-trial-003"></a>

### Owner decision 2026-09-14 — **Trial #3 D-7 AUTHORISED** for exactly one valid LAUNCH-EXEC-01 Trial #3 execution

**D-7 — execution authorisation — is GRANTED PROSPECTIVELY for LAUNCH-EXEC-01 Trial #3.** It is
bound to one exact frozen object, one exact reviewed dispatcher blob and **exactly one valid Trial #3
execution**. It is a new authorisation. The consumed D-7 authorisations of Trial #1 and Trial #2 are
not reused. Every earlier section is left as written, including the 2026-09-12 statement above that
no D-7 existed for Trial #3.

**AUTHORISED IS NOT CONSUMED.** At this record Trial #3 D-7 is **AUTHORISED** and **NOT CONSUMED**,
Trial #3 is **NOT_RUN**, and its valid trial count is **ZERO**. **Trial #3 must not yet be
dispatched.** The dispatcher is published on the milestone branch only. It is not on `main`, and
its zero-run state on `main` has not been proven.

| Binding | Value |
|---|---|
| Trial | **trial-003** |
| D-7 status | **AUTHORISED PROSPECTIVELY — NOT CONSUMED** |
| Scope | **exactly one** valid Trial #3 execution |
| Freeze commit | `bebd8a5f83d4d0daebe9b068050cb5436289c75e` |
| `SOURCE-HASHES.json` Git blob | `8cd290b573408510f8c16cd8dafe676354140a38` |
| `SOURCE-HASHES.json` SHA-256 | `ea482c6feaf77abac1edcc23d26afbc4f249638170f60ed897088d5cda79ba70` |
| Independent freeze review | `69f13a78810e1d270e4540b1fa169c8a8e9eb5fd` — [record](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-FREEZE-REVIEW.md), `TRIAL_3_FREEZE_REVIEW_PASSED_READY_FOR_PUBLICATION_AND_BUILD_8` |
| Formal Build 8 | run `34754901079`, job `103717520698`, attempt 1 |
| Formal Build 8 result | **`BUILD_8_FORMAL_COMPILE_ONLY_SUCCESS`** — pretrial compile evidence only; the trial builds and hashes its own binaries |
| Accepted Build 8 evidence authority | `5a6be5959c5a9131afa1154c015369df46a5deb8` — [re-review](implementation/HELM-LAUNCH-EXEC-01-BUILD-008-EVIDENCE-CORRECTION-REREVIEW.md), `BUILD_8_EVIDENCE_CORRECTION_REREVIEW_PASSED_READY_FOR_TRIAL3_AUTHORITY_DECISIONS` |
| Build 8 durable job-log SHA-256 | `6c16e23ea03a7593e09911ca968c4313ab7764a05b98fd5b19c4756da9930a15` |
| Dispatcher candidate commit | `f1973867a709735b7c7e967f7f4b320774de2aa2` |
| Independent dispatcher review | `4f1989737631b66f8cd4ebc8f7fdfc71ec1a5165` — [record](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-DISPATCHER-REVIEW.md), `TRIAL_3_DISPATCHER_REVIEW_PASSED_READY_FOR_MILESTONE_PUBLICATION` |
| Dispatcher path | [`.github/workflows/launch-exec-01-trial-003.yml`](../.github/workflows/launch-exec-01-trial-003.yml) |
| Authorised dispatcher Git blob | `64ce3d433a47eaae3eb28b8bb28f7300a330d762` |
| Authorised dispatcher file SHA-256 | `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8` |
| Required human confirmation | `RUN-TRIAL-003-ONE-VALID-TRIAL` |
| D-7 consumption boundary | the **first fsynced `case_pose_started`** record |

**The frozen source, definition and manifest remain immutable, and so does the dispatcher.** This
authorisation is recorded here, outside the freeze, so that granting it changes no frozen byte and
no dispatcher byte. `status: NOT_RUN`, `d7_execution_authorised: false` and the `trial_3` block inside
`SOURCE-HASHES.json` are left untouched. They record the state at which the immutable freeze was
cut. This external decision supersedes them for execution authority without mutating the artefact
it authorises.

**Operator invariants.** They are part of this authority.

1. This D-7 authorises **only** the exact dispatcher path, Git blob and file SHA-256 above.
2. Before dispatch, that exact workflow must be published **byte-identically** to `main`.
3. Immediately before dispatch, `main` must contain `.github/workflows/launch-exec-01-trial-003.yml`
   with blob `64ce3d433a47eaae3eb28b8bb28f7300a330d762` and SHA-256
   `9158e2932cc4f639c9e9be30ab445d9bcf1797dd7526274b7f96f1948fb9e4c8`.
4. Before the authorised human dispatch, the Trial #3 workflow run count must still be **zero**, and
   run number 1 must be unused.
5. Between this authorisation and completion of the authorised run:
   * do not edit the Trial #3 dispatcher;
   * do not rename it;
   * do not copy it to another executable workflow path;
   * do not delete and recreate it;
   * do not replace its bytes;
   * do not force-push authority-bearing history.
6. **The remote milestone branch `docs/helm-launch-architecture` must not be deleted while this D-7
   remains live.** The authority commits `bebd8a5`, `69f13a7`, `5a6be59`, `f197386` and `4f19897`
   must remain reachable from remote repository history, because the dispatcher's identity gates
   bind them. Any of the following means **STOP, NO DISPATCH, OWNER DECISION REQUIRED**:
   * deleting the milestone branch;
   * force-moving it so that those commits become unreachable;
   * otherwise breaking their required reachability.
7. The dispatcher may be manually dispatched **exactly once**, from `main`, using exactly
   `RUN-TRIAL-003-ONE-VALID-TRIAL`.
8. No rerun, retry, resume, replacement dispatch, second dispatch or rerun-to-green is authorised.
9. After dispatch, the exact GitHub run commit must be verified to carry
   `.github/workflows/launch-exec-01-trial-003.yml` at blob
   `64ce3d433a47eaae3eb28b8bb28f7300a330d762`.
10. Any dispatcher path, blob or SHA-256 mismatch means **STOP — NO EXECUTION AUTHORITY**. This D-7
    gives no authority to execute changed dispatcher bytes.
11. **Before the boundary.** If the first dispatch is rejected or fails before the first fsynced
    `case_pose_started`:
    * D-7 remains unconsumed;
    * the valid Trial #3 count remains zero;
    * run number 1 and this dispatcher instance are spent.

    This D-7 does **not** transfer to another dispatcher or a second run. A new owner decision is
    required.
12. **At the boundary.** Once the first fsynced `case_pose_started` exists, **TRIAL #3 D-7 IS
    CONSUMED**. That holds regardless of the eventual aggregate, a harness failure, a timeout or an
    abort. An abort after the boundary is read from the preserved journal under the manifest's
    frozen `partial_reading` rules, with no aggregate derived after the fact.
13. **After the boundary:** **no retry, no resume and no second Trial #3 under this D-7.**

**Scope limits carried from the Trial #2 record.** They restrict this authorisation and add no
authority. It does not extend to:

* another freeze, freeze SHA or dispatcher blob;
* arbitrary HELM execution;
* Trial #1 or Trial #2;
* any rerun, resume or retry to obtain a better result, or rerunning failed, invalid or blocked
  cases;
* modifying frozen experiment bytes or the dispatcher before or during the trial;
* changing checker or verdict semantics after observing results;
* creating `crates/helm-launch`, or accepting ADR-0024.

After the trial, its result needs an independent result review before ADR-0024 or
`crates/helm-launch` may advance. A red or green workflow is not a verdict: the result comes only
from the frozen durable evidence.

**TRIAL #3 D-7 IS AUTHORISED.**

**TRIAL #3 D-7 IS NOT CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ZERO.**

**TRIAL #3 IS NOT_RUN.**

**TRIAL #3 MUST NOT YET BE DISPATCHED.** The next gates are:

1. byte-identical publication of the dispatcher to `main`;
2. confirmation that GitHub registers the workflow at that path;
3. proof that it still has zero runs and an unused run number 1.


<a id="trial-003-x2c-postmortem-owner-disposition"></a>

### Owner decision 2026-09-15 — Trial #3 X2c postmortem accepted; formal LAUNCH-EXEC-01 trial line **CLOSED**; **no Trial #4**

The owner accepts the
[Trial #3 X2c postmortem](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-X2C-POSTMORTEM.md) at
`585caf924454faf5fb92e1e1dbd54675a1b7c0bf`, which follows the
[independent Trial #3 result review](implementation/HELM-LAUNCH-EXEC-01-TRIAL-003-RESULT-REVIEW.md)
at `8dc9e96c64c2e590434e6e210875ce6d13fefc32`. The postmortem is engineering diagnosis made after
the frozen result. This decision changes no frozen source, definition, manifest, checker, fixture,
launcher, dispatcher, evidence file, review or postmortem. Every earlier section is left as written,
including the prospective Trial #3 D-7 record above.

**`TRIAL_3_X2C_POSTMORTEM_ACCEPTED`**

#### 1. X2c disposition

**X2C_PRIMARY_DIAGNOSIS:** X2c's frozen success rule required a structured helper report behind the
frozen report sentinel, but the frozen `script_fixture.sh` could never emit that report. Static
posability certified X2c because the launcher supplied the report **transport channel**, without
establishing that the **executed object** could produce the report required by the success rule.

| Layer | Classification |
|---|---|
| Product mechanism | **`MECHANISM_NOT_IMPLICATED`** |
| Frozen expectation | **`EXPECTATION_EVIDENCE_MISMATCH`** |
| Observability | **`OBSERVABILITY_INCOMPLETE`** |
| Harness | **`STATIC_POSABILITY_DEFECT`** + **`FIXTURE_DEFECT`** |

The conservative [Trial #2 / S4 policy](#trial-002-postmortem-decisions) remains in force and is not
weakened: **clean exec-status EOF alone is not positive exec proof.**

#### 2. Frozen history

Trial #3 remains exactly as frozen and reviewed: **70 PASS / 1 FAIL / 1 BLOCKED**, aggregate
**`MECHANISM_REJECTED`**, sole FAIL `X2c`, sole BLOCKED `N3`. X2c is not changed to PASS and is not
removed from Trial #3; the aggregate is not recomputed under a corrected rule; Trial #3 was **not**
`MECHANISM_ACCEPTED`.

#### 3. No Trial #4

**No Trial #4 is authorised.** No Trial #4 candidate, freeze, Build 9 for an acceptance trial or
D-7 is created or prepared. A formal trial being rejected does not make another trial an automatic
consequence. Any future formal trial requires a **new explicit owner decision** after a new
preregistration and review process.

#### 4. Product engineering disposition

* **Formal experimental result:** `MECHANISM_REJECTED`.
* **Engineering disposition:** the sole reject-causing case does not demonstrate a defect in the
  supported product launch mechanism.

The supported ELF launch mechanism is therefore **not blocked by X2c** from moving into
productization planning. This is **not** a statement that Trial #3 is `MECHANISM_ACCEPTED`, and it
must not be read or cited as one.

#### 5. Script scope

**`#!` script execution is not a supported LAUNCH-EXEC-01 0.1 product path.** The 0.1 mechanism
admits the declared ELF cohort and refuses scripts. X2b and X2c were counterfactual control arms
used to understand the reason for script refusal. The X2c observability defect does not require
changing the product mechanism to support scripts, and 0.1 scope is not expanded.

#### 6. N3

N3 remains **BLOCKED** on `unprivileged_runner`. No real privileged identity was manufactured, so
Trial #3 establishes no new claim about a real privilege transition. Existing conservative product
policy stands, including refusal of set-ID objects at admission (D-9). N3 is not reopened.

#### 7. R3-M1

R3-M1 is **future evidence-contract work**. `aggregate_input_digest` was calculated from
pre-sanitisation records, while the sanctioned preserved evidence intentionally withholds eight
`child_pid` values, so the digest cannot be independently recomputed from the public preserved
evidence.

* **MINOR**; **nonblocking** for Trial #3 result authority; **not related** to X2c.
* Trial #3 history is **not** repaired.
* For a future evidence contract, any integrity digest intended for independent reproduction must
  commit to sanctioned preserved bytes or another reproducible published representation. That
  correction is not implemented by this decision.

#### 8. Future X2c / posability work — backlog, not a trial

* A report-declaring case must prove that its planned **executed object** can actually produce the
  evidence its rule requires. Global channel availability is not enough.
* X2c's fixture and evidence construction must be corrected before X2c is used in any future formal
  acceptance claim.
* The postmortem noted `helper_setid` / N3 as a possible audit target of the same unchecked
  property. N3 was outside the X2c diagnosis and remains blocked; the X2c diagnosis is **not**
  generalised to N3.
* A later bounded test-contract audit may inspect all report-declaring plans.

#### 9. ADR and product crate boundary

* **ADR-0024 remains Proposed.** This decision does not accept it.
* This decision does **not** create `crates/helm-launch`.
* The next phase is **productization planning**, not implementation.

#### 10. Next gate

**The current LAUNCH-EXEC-01 formal trial line is CLOSED.** No additional formal trial is
authorised.

**Next gate: one bounded helm-launch productization plan.** It must translate the experimentally
supported invariants into:

* product API boundaries;
* the Linux backend boundary;
* the safe/unsafe split;
* the lifecycle and result model;
* integration points with helm-bind, helm-observe and helm-evidence;
* supported versus unsupported 0.1 behaviour;
* a product test strategy;
* migration from experiment code to product code.

It must not copy the experiment harness wholesale into the product.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED.**

**TRIAL #3 D-7 IS CONSUMED.**

**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ONE.**

**TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**


<a id="helm-launch-productization-plan-owner-review"></a>

### Owner decision 2026-09-16 — helm-launch productization plan owner review **PASSED WITH BOUNDED AMENDMENTS**; ADR-0024 revision prepared, **still Proposed**

The owner reviewed the
[helm-launch 0.1 productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md) as proposed
at `930ec14b940da9b136c7d2ad024b441d47ceba6c`. This decision changes no frozen experiment source,
LAUNCH-EXEC-01 definition, manifest, checker, fixture, launcher, dispatcher, evidence file, review or
postmortem. Its effect on ADR-0024, a file the Trial #3 freeze bound as a definition input, is stated
in section 6. Every earlier section is left as written, including the ADR-0024 index row, every D-7
record and every Trial #3 record.

**`HELM_LAUNCH_PRODUCTIZATION_PLAN_OWNER_REVIEW_PASSED_WITH_BOUNDED_AMENDMENTS`**

#### 1. Q1 — ADR-0024 disposition: **APPROVED**

ADR-0024 must be revised before it can be considered for acceptance. Its Proposed text of 2026-09-09
is stale and must not be accepted as-is.

#### 2. Q2 — post-experiment product refinements: **APPROVED WITH AMENDMENTS, AS AN EXACT NARROWING**

Approved as 0.1 design obligations, to be validated by normal product tests and **not** by another
formal D-7 trial:

* refuse writable executable capabilities;
* relocate all child-side preserved descriptors to `>= 3` before final mapping;
* block all blockable signals across `clone3`, and restore or reset child signal state in the closed
  child sequence;
* guard the process-group sweep on positively established group authority;
* bound the reap after `SIGKILL`, and represent a child that still cannot be observed ended honestly;
* distinguish read failure from EOF;
* record timeout and termination actions as facts, without unsupported causal claims;
* `POST_KILL_REAP_MS = 5000` as the initial 0.1 bound;
* a maximum admitted executable size of 512 MiB as the initial 0.1 product bound.

These are product design choices. They are **not** claims that LAUNCH-EXEC-01 directly validated
them.

#### 3. Q3 — process-group sweep: **APPROVED WITH A GROUP-AUTHORITY GUARD**

* Exactly one `SIGKILL` process-group sweep is the fixed 0.1 cleanup policy on every return or
  completion path **after** the launcher has positively established the direct child's dedicated
  process group. This includes a normal direct-child exit.
* The sweep **must** occur before the direct child is reaped.
* If dedicated process-group establishment was not positively established, the launcher **must not**
  infer or guess a group id and **must not** issue a group sweep.
* Issuance is recorded as a fact. No descendant-killed claim and no containment claim follows.
* **Accepted consequence.** Same-group descendants may be killed when `launch()` completes, including
  after a normal direct-child exit: a same-group background process may be terminated even when the
  direct child exited normally. That is intentional 0.1 cleanup policy. A descendant that leaves the
  group may survive.
* This is **best-effort cleanup** and must not be described as process-tree containment.

#### 4. Required amendments

**A. Measurement instability.** `helm-launch` refuses admission when the defined measurement protocol
**detects** instability during measurement. Failure to detect instability does not prove that no
concurrent mutation occurred, immutability, a snapshot, or that the measured bytes are the bytes
later executed. The executable digest remains only a **pre-execution measurement of the pinned
object**. Any detection method is described by what it actually observes, and no attestation claim
is introduced.

**B. Receipt authenticity.** `LaunchReceipt` is not globally "unforgeable". The safe Rust API may
prevent an external caller from constructing an authority-bearing capability or `AuthorizedLaunch`
value through ordinary public constructors. That does not make serialised receipt bytes authentic. A
durable receipt is deterministic data, carries zero execution authority, may be copied, may be
fabricated outside the crate, is not cryptographically signed, and is not proof of provenance by
itself. `helm-launch` provides no receipt-authenticity claim. Provenance and bundle validation belong
above `helm-launch`, and `helm-evidence` does not become launch authority.

**C. "Executes exactly one".** The product contract authorises and **attempts** execution of exactly
one admitted ELF object through the exact authorised descriptor. No prose may imply that the receipt
always proves that the measured image executed successfully. **Clean exec-status EOF alone is not
positive exec proof.** The durable model contains no unconditional `ExecSucceeded` fact, and where
positive exec cannot be independently established, `ExecStatusIndeterminate` is preserved.

#### 5. Execution mechanism and signal contract

* Direct-child creation and atomic pidfd acquisition use `clone3(CLONE_PIDFD)`, which replaces the
  stale `fork` plus `pidfd_open` mechanism. `clone3` avoids the post-fork `pidfd_open` preconditions
  and the `pthread_atfork` surface identified during the experiment line.
* The execution target is `execveat(exec_fd, "", argv, envp, AT_EMPTY_PATH)`. The `/proc/self/fd`
  fallback is not a 0.1 product path. `std::process::Command`, `posix_spawn` and a helper or
  trampoline are rejected alternatives, never fallbacks.
* **Signals.** Before `clone3`, the calling thread saves its signal mask and blocks all blockable
  signals for the clone window. The child starts with that blocked mask, establishes its setup,
  restores the required dispositions while delivery is blocked, restores the intended final mask only
  after the dispositions are ready, then applies `no_new_privs` and reaches `execveat` in the
  approved sequence. The parent restores its original mask after `clone3`. `SIGKILL` and `SIGSTOP`
  are not blockable. No process-wide signal control is claimed in a multithreaded caller: the
  operation controls the calling thread and the child's inheritance boundary.

#### 6. ADR-0024 and the plan

* The amendments are applied in place in the plan, and its section 19 lists every amended passage.
* [ADR-0024](adr/ADR-0024-launch-authority.md) is **revised in place** to the current intended
  product contract and **stays Proposed**. No acceptance date and no approver are entered.
* The Trial #3 freeze bound the pre-revision ADR-0024 bytes as a definition input (SHA-256
  `c1f3cce88438439aab6632a9c56450ae98d1c64adb31b13c742e2febb1476351`). Those bytes remain
  addressable at freeze commit `bebd8a5f83d4d0daebe9b068050cb5436289c75e`. The revision changes no
  freeze manifest, experiment source, LAUNCH-EXEC-01 definition or evidence file.

#### 7. Trial history

Trial #3's frozen result stays **`MECHANISM_REJECTED`**, 70 PASS / 1 FAIL / 1 BLOCKED. The sole FAIL,
X2c, was subsequently accepted by owner postmortem as `PRODUCT_MECHANISM: MECHANISM_NOT_IMPLICATED`,
and the formal result is **not** rewritten as accepted. N3 remains unvalidated for a real privilege
transition. R3-M1 is future evidence-contract work. The LAUNCH-EXEC-01 formal trial line is closed.

#### 8. Boundary and next gate

* **ADR-0024 is not accepted** and remains Proposed.
* **`crates/helm-launch` remains unauthorised** and is not created. No implementation is authorised.
* **No Trial #4** is authorised, prepared or implied.
* `main` is unchanged.

**Next gate: owner review of revised ADR-0024.**

**ADR-0024 REMAINS PROPOSED.**

**`crates/helm-launch` IS NOT AUTHORISED.**

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED.**

**NO TRIAL #4 IS AUTHORISED.**

<a id="adr-0024-accepted-helm-launch-p1-authorised"></a>

### Owner decision 2026-09-17 — **ADR-0024 ACCEPTED**; **HELM-LAUNCH P1 AUTHORISED**, P2+ not authorised

The repository owner, Djomla83, completed the final architecture review of the revised
[ADR-0024](adr/ADR-0024-launch-authority.md) carried by the chain
`174caad1a3980a35b2ea841f49e05555753710b5` → `930ec14b940da9b136c7d2ad024b441d47ceba6c` →
`a810f50f3ef12177d5bdcc2825dd81fc124ccb9b` → `03285d9d13f53c2d97d78bd4f50552c201941c8f`. This
decision changes no frozen experiment source, LAUNCH-EXEC-01 definition, freeze manifest,
`SOURCE-HASHES.json`, test, workflow, evidence file, review or postmortem. Every earlier section is
left as written, including every D-7 record and every Trial #1, #2 and #3 record.

#### 1. ADR-0024: **ACCEPTED 2026-09-17**

* **Approver:** repository owner Djomla83. **Acceptance date:** 2026-09-17.
* What is accepted is the **revised** ADR-0024, interpreted together with the
  [owner-reviewed productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md). It is not a
  retroactive approval of the stale Proposed text of 2026-09-09.
* **Scope: the helm-launch 0.1 architecture boundary only.** That boundary is: Linux x86_64 product
  backend only; an already-open descriptor as the sole execution authority; no authority from a
  plan, AppSpec, observation, binding or evidence; `NoClaimContradicted` never permission; one
  regular admitted ELF64 x86_64 object only; scripts refused; no `PATH`, no shell, no pathname
  execution, no `/proc` fallback; no Wine, PWA, MicroVM or prefix orchestration in 0.1;
  `clone3(CLONE_PIDFD)` as the planned process-creation primitive; `execveat(fd, "", …,
  AT_EMPTY_PATH)`; an explicit working-directory capability; UTF-8 argv; an exactly empty
  environment; `PR_SET_NO_NEW_PRIVS`; the exact descriptor-inheritance contract; the conservative
  exec-confirmation model, in which **clean exec-status EOF alone is not positive exec proof** and
  `ExecStatusIndeterminate` is first-class; direct-child lifecycle only, with a guarded exactly-once
  process-group sweep before the reap; no process-tree containment; no sandbox claim; a durable
  receipt with no verdict vocabulary and no authenticity claim; zero HELM crate dependencies; and a
  scoped Linux `unsafe` backend in a later implementation slice.
* **Evidence classes are unchanged.** Acceptance does not claim that every post-experiment product
  refinement was experimentally validated. The distinction between `EXPERIMENTALLY_SUPPORTED`,
  `OWNER_POLICY`, `DOCUMENTATION_DERIVED` and `UNVALIDATED` product obligations stays as the plan's
  section 3 records it.
* **HELM-LAUNCH 0.1 product contract: ACCEPTED**, as architecture. No schema or API is stabilised.

#### 2. Trial history: unchanged

* **Trial #3 frozen result remains `MECHANISM_REJECTED`** (70 PASS / 1 FAIL / 1 BLOCKED). It is not
  rewritten as `MECHANISM_ACCEPTED`.
* The engineering disposition remains `PRODUCT_MECHANISM: MECHANISM_NOT_IMPLICATED` by X2c.
* Trial #3 D-7 is **consumed**; the valid Trial #3 count is **one**; **Trial #3 must not be rerun**.
* **No Trial #4 is authorised.** The LAUNCH-EXEC-01 formal trial line is **closed**.
* The test correction at `03285d9d13f53c2d97d78bd4f50552c201941c8f` is accepted: the 17 frozen
  experiment source files are bound to the current tree, and the 3 Trial #3 definition files to the
  historical freeze at `bebd8a5`. The freeze and its manifest are unchanged.

#### 3. Implementation authority: **HELM-LAUNCH P1 ONLY**

**P1 purpose: create the product crate skeleton and the portable, pure model.** P1 may create
`crates/helm-launch`, add it to the workspace, and implement only portable, pure surfaces such as the
validated launch-plan model; deterministic parsing and validation; the `Digest` value type; the
portable receipt data model; closed non-verdict enums; the public error vocabulary; pure fd-layout
planning; a pure lifecycle state-transition model that performs no OS operation; the serialisation
and deserialisation the accepted contract requires; compile-fail and type-boundary tests; portable
unit and property tests; and a crate README documenting scope. P1 may use ordinary safe dependencies
already approved by the plan where these surfaces need them.

**P1 must not implement:** `unsafe` code; `clone3`; `execveat`; raw syscall shims; the Linux execution
backend; process creation or execution; pidfd acquisition; `pidfd_send_signal`; `waitid`;
`close_range`; `fchdir`; process-group signalling; signal manipulation; the `PR_SET_NO_NEW_PRIVS`
call; executable descriptor admission; working-directory descriptor admission; ELF filesystem I/O;
executable measurement I/O; `launch()`; any function that can cause a child process to exist; or any
reuse of the experimental runner as product code.

| P1 property | Value |
|---|---|
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |

**P2, P3, P4, P5 and any equivalent later slice are NOT authorised.** A later explicit owner decision
is required before moving beyond P1.

#### 4. Boundary and next gate

* This decision is recorded in documentation only. `crates/helm-launch` is **not yet created**;
  `Cargo.toml`, `Cargo.lock`, tests, experiments, evidence and workflows are unchanged. `main` is
  unchanged.

**Next gate: IMPLEMENT HELM-LAUNCH P1 — PORTABLE MODEL ONLY.**

**ADR-0024 IS ACCEPTED.**

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 IS AUTHORISED. HELM-LAUNCH P2+ IS NOT AUTHORISED.**


<a id="helm-launch-p1-accepted"></a>

### Owner decision 2026-09-17 — **HELM-LAUNCH P1 ACCEPTED**; P2+ not authorised

**`HELM_LAUNCH_P1_ACCEPTED`.** The repository owner, Djomla83, accepts HELM-LAUNCH P1 as the
**first helm-launch product implementation slice**, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md) and the
[P1-only authorisation](#adr-0024-accepted-helm-launch-p1-authorised). This is **not** product
acceptance of the complete helm-launch 0.1 module. Every earlier section, including every ADR, D-7
and Trial #1, #2 and #3 record, is left as written.

| Item | Value |
|---|---|
| Accepted implementation | `427b1af092db29c619b7f7a4c0d40b72efaacc65` |
| Accepted correction | `d3914ab95fb253abd8ac10462a4e2de1cb2185de` |
| Independent review | `e32b2e1768b0b0d11f9b02115b107b6a40f60ecb` — [review](implementation/HELM-LAUNCH-P1-INDEPENDENT-REVIEW.md) |
| Independent review findings | **0 BLOCKER**, **0 IMPORTANT** |
| Publication CI | **PASSED** — HELM Rust workspace Linux run `35244879485`; helm-launch P1 portable model cross-platform purity run `35244879475`; helm-bind cross-platform purity run `35244879552`; all SUCCESS |
| P1 cross-platform matrix | **PASSED ON LINUX / WINDOWS / MACOS** (`ubuntu-24.04`, `windows-2025`, `macos-15`) |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |

#### 1. Accepted P1 boundary

Accepted P1 contains only: portable launch-plan parsing and validation; `ValidatedLaunchPlan`;
`Digest`; the portable receipt and fact model; deterministic receipt serialisation; the launch-plan
error vocabulary; pure fd-layout planning; pure lifecycle state modelling; portable tests; and the
P1 CI plumbing.

| P1 property | Value |
|---|---|
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |
| P2+ implementation | **NONE** |

No executable or working-directory capability exists in accepted P1. No `launch()` exists. No
process can be created by accepted P1.

#### 2. Authority

| Slice | Authority |
|---|---|
| P2 | **NOT AUTHORISED** |
| P3 | **NOT AUTHORISED** |
| P4 | **NOT AUTHORISED** |
| P5 | **NOT AUTHORISED** |

P1 acceptance does **not** authorise the next slice. The next owner decision is whether to authorise
the bounded P2 capability-admission slice defined by the accepted
[productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md). That decision is not made
here.

#### 3. Independent-review findings

None blocks P1 acceptance.

| Finding | Severity | Disposition |
|---|---|---|
| P1-DOC-01 | MINOR | **RESOLVED BY OWNER-ACCEPTANCE SYNC**: the productization plan's T40 wording (section 8.5 and the T40 row) now states that `EndNotObserved` is latched once `POST_KILL_REAP_MS` expires without an observed end, and no later `Exited`, `Signaled`, core-dumped result or `ECHILD` changes it; `not_issued_child_already_reaped` may still be recorded. `EndNotObserved` means the end was not observed within the approved bound, not that the child never ended |
| P1-DOC-02 | MINOR | **open, nonblocking**; crate README not edited |
| P1-TEST-01 | MINOR | **open, nonblocking**; tests not edited |
| P1-PARSE-01, P1-PARSE-02, P1-LIFE-01, P1-SER-01, P1-CI-01, P1-TEST-02 | BACKLOG_NONBLOCKING | unchanged |

#### 4. Historical authority and next gate

This decision changes no product code, test, workflow, Cargo file, ADR, experiment, evidence or
the independent review, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 IS ACCEPTED. HELM-LAUNCH P2+ IS NOT AUTHORISED.**

**Next gate: OWNER DECISION ON WHETHER TO AUTHORISE HELM-LAUNCH P2 CAPABILITY ADMISSION.**


<a id="helm-launch-p2-authorised"></a>

### Owner decision 2026-09-18 — **HELM-LAUNCH P2 AUTHORISED**: capability admission and authorisation composition only; P3+ not authorised

**`HELM_LAUNCH_P2_AUTHORISED`.** The repository owner, Djomla83, authorises the **bounded P2
capability-admission slice** of `helm-launch`, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md), the owner-reviewed
[productization plan](implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md) section 16 P2 row, and the
[P1 acceptance of 2026-09-17](#helm-launch-p1-accepted). This is **not** product acceptance of
P2 and **not** product acceptance of the complete helm-launch 0.1 module. Every earlier section,
including every ADR, D-7 and Trial #1, #2 and #3 record, is left as written.

| Item | Value |
|---|---|
| ADR-0024 | **ACCEPTED 2026-09-17** |
| HELM-LAUNCH P1 | **ACCEPTED** (`cd5db27964dc10593bd8856d2d331b907a4b608e`) |
| HELM-LAUNCH P2 | **AUTHORISED BY THIS DECISION** |
| HELM-LAUNCH P3, P4, P5 | **NOT AUTHORISED** |
| Current capability before P2 implementation | **PORTABLE MODEL ONLY** |
| P2 authorised capability | **SAFE LINUX X86_64 CAPABILITY ADMISSION AND COMPOSITION** |
| Trial #4 | **NOT AUTHORISED** |
| Starting state | branch `docs/helm-launch-architecture` at `cd5db27964dc10593bd8856d2d331b907a4b608e`; `main` at `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |

#### 1. P2 scope

**Capability admission and authorisation composition only.** P2 may introduce
`ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`,
`AdmissionError`/`AdmissionErrorCode`, `AuthorizationRefusal` with a refusal code vocabulary if a
separate enum is useful, `admit_executable`, `admit_working_directory`, `authorize`, safe Linux
x86_64 descriptor inspection and positional reads, pre-execution executable measurement, ELF cohort
admission, admission and refusal tests, and capability and type-boundary tests.

**P2 must not introduce** `LaunchOutcome`; `launch()`; process creation; process execution;
`clone3`; `execveat`; pidfd acquisition or signalling; `waitid`; `pidfd_send_signal`;
`close_range`; `fchdir` execution; signal manipulation; process-group manipulation; the
`PR_SET_NO_NEW_PRIVS` call; pipes for a child; polling lifecycle; timeout execution; a `backend/`
directory; a syscall shim; inline assembly; libc calls; or `unsafe` code.

| P2 property | Value |
|---|---|
| Process creation | **NONE** |
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |
| P3+ implementation | **NONE** |

#### 2. What P2 may do to the host, stated

* **P2 may perform read-only I/O through caller-supplied descriptors.** Admission inspects
  descriptor flags, samples metadata twice and reads the object's bytes positionally, all through
  the one descriptor the trusted caller moved in. It resolves no pathname and opens nothing.
* **Executable measurement may update atime and populate the page cache.** `O_NOATIME` is not used,
  because it requires file ownership or `CAP_FOWNER` (ADR-0024 section C).
* **P2 creates authority-bearing in-process Rust values, but there is still no function that can
  execute them.** `AuthorizedLaunch` exists after P2 and is inert: `launch()` does not exist, so no
  P2 code path can create a process.
* **P2 does not authorise process creation.**

#### 3. Product boundary after P2

```text
untrusted plan bytes ──▶ ValidatedLaunchPlan          no authority
caller-owned executable fd ──admit_executable──▶ ExecutableCapability
caller-owned cwd fd + logical id ──admit_working_directory──▶ WorkingDirectoryCapability
plan + executable + working directory ──authorize──▶ AuthorizedLaunch
AuthorizedLaunch ──╳──▶ process        no edge exists: launch() does not exist in P2
```

The capability, authorisation and admission APIs exist only under
`cfg(all(target_os = "linux", target_arch = "x86_64"))`. The portable P1 model stays available on
Linux x86_64, other Linux architectures, Windows and macOS, and off the cohort the P2 types and
functions must not exist in the public API. No support for another platform is advertised.

`NoClaimContradicted` remains **not permission**, plan `asserted_context` digests remain **inert
caller assertions** that `authorize` must not inspect, and no function may turn bytes, a string, a
path, a `ValidatedAppSpec`, an `ObservationArtifact`, a `RootCapability`, a `BindingReport`, a
`Contradiction`, a `Coverage`, any `serde` input, receipt bytes or a `LaunchReceipt` into a
capability or an authorisation. `helm-launch` keeps **zero HELM crate dependencies**.

#### 4. Measurement is not an attestation

The second metadata sample may only ever support the statement **the measurement protocol detected
instability**, or that it did not. It must never be described as proof that no mutation occurred,
that the inode is immutable, that a snapshot exists, or that the measured bytes are the bytes later
executed. Measurement is a **pre-execution measurement of the pinned object**, never executed-body
identity.

#### 5. Dependencies and safety

`rustix` may become a direct `helm-launch` dependency on the Linux x86_64 cohort at the
repository-vetted pin `=1.1.4`, with `default-features = false` and only the safe features P2
actually needs. No direct `libc` dependency, no new package version, and no unrelated `Cargo.lock`
change. P2 contains **zero** `unsafe` operations; the accepted future `unsafe` exception stays
reserved for P3 and is **not active**, and no `backend` directory may exist.

#### 6. Boundary and next gate

This decision is recorded in documentation only and changes no product code, test, workflow, Cargo
file, ADR, experiment, evidence or independent review, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 IS ACCEPTED. HELM-LAUNCH P2 IS AUTHORISED. HELM-LAUNCH P3+ IS NOT AUTHORISED.**

**Next gate: IMPLEMENT HELM-LAUNCH P2 — CAPABILITY ADMISSION AND AUTHORISATION COMPOSITION ONLY,
then ONE FRESH INDEPENDENT REVIEW OF THE P2 CANDIDATE.**


<a id="helm-launch-p2-accepted"></a>

### Owner decision 2026-09-18 — **HELM-LAUNCH P2 ACCEPTED**; P3+ not authorised

**`HELM_LAUNCH_P2_ACCEPTED`.** The repository owner, Djomla83, accepts HELM-LAUNCH P2 as the
**second helm-launch product implementation slice**, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md), the
[P2 authorisation of 2026-09-18](#helm-launch-p2-authorised) and the
[P1 acceptance of 2026-09-17](#helm-launch-p1-accepted). **P1 remains accepted.** This is **not**
product acceptance of the complete helm-launch 0.1 module. Every earlier section, including every
ADR, D-7 and Trial #1, #2 and #3 record, is left as written.

| Item | Value |
|---|---|
| Accepted P2 authority record | `a3a8d999a6bfa59ce27b1515525e6a7578dad7fe` |
| Accepted P2 implementation | `c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` |
| Independent review | `94ce8dd34cd6694ea3b528a9dad95d2a71b4ad70` — [review](implementation/HELM-LAUNCH-P2-INDEPENDENT-REVIEW.md) |
| Independent review findings | **0 BLOCKER**, **0 IMPORTANT** |
| Accepted P1 base | `cd5db27964dc10593bd8856d2d331b907a4b608e` |
| Publication CI | **PASSED** — HELM Rust workspace Linux run `35323497391`; helm-launch matrix run `35323497452`; helm-bind cross-platform purity run `35323497482`; all SUCCESS, all `push`, all attempt 1 |
| P2 Linux runtime gate | **PASSED** — `ubuntu-24.04`: `tests/linux_admission.rs` **28 passed**, `authority.rs` Linux-specific unit tests **13 passed**, the Linux cohort doctest surface executed and passed |
| Cross-platform matrix | **PASSED ON LINUX / WINDOWS / MACOS** (`ubuntu-24.04`, `windows-2025`, `macos-15`) |
| P2 authority API off the cohort | **ABSENT** — Windows and macOS report `0 tests` for the admission suite and run the off-cohort `compile_fail` absence proofs |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |

#### 1. Accepted P2 capability

Accepted P2 adds `ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`,
`admit_executable`, `admit_working_directory`, `authorize`, safe Linux x86_64 descriptor admission,
pre-execution executable measurement, ELF64 x86_64 cohort admission, detected-instability refusal,
working-directory capability admission, and zero-I/O composition into a single-use authorisation.
These exist only under `cfg(all(target_os = "linux", target_arch = "x86_64"))`.

Accepted authority flow:

```text
caller-owned executable fd    --admit_executable--------▶ ExecutableCapability
caller-owned cwd fd + id      --admit_working_directory-▶ WorkingDirectoryCapability
plan + executable + cwd       --authorize---------------▶ AuthorizedLaunch
AuthorizedLaunch              --╳-----------------------▶ process
```

**The last edge does not exist.** `AuthorizedLaunch` still has no consumer that can create a
process, because `launch()` does not exist on any platform.

| P2 property | Value |
|---|---|
| Process creation | **NONE** |
| Process execution | **NONE** |
| Unsafe | **NONE** |
| Host privilege | **NONE** |
| Experiment execution | **NONE** |

The P2 admission tests execute no admitted program. Publication CI success is **not** evidence of
process execution, and none occurred.

#### 2. Authority after this acceptance

| Slice | Authority |
|---|---|
| P3 | **NOT AUTHORISED** |
| P4 | **NOT AUTHORISED** |
| P5 | **NOT AUTHORISED** |

P2 acceptance does **not** authorise process creation and does **not** activate the ADR-0024
section E `unsafe` exception, which stays reserved and inactive. It authorises no `clone3`, no
`execveat`, no `backend/` directory, no syscall shim, no inline assembly, no pidfd, no signal
manipulation and no child creation. The next owner decision is whether to authorise the P3 unsafe
backend and child contract. **That decision is not made here.**

#### 3. Verification gates

| Gate | State | Evidence |
|---|---|---|
| **P2-VERIFY-01** | **CLOSED — PASSED** | hosted Linux x86_64 execution: `tests/linux_admission.rs` 28 passed; `authority.rs` Linux-specific unit tests 13 passed; the Linux cohort doctest surface executed and passed. The suite ran in both the helm-launch `ubuntu-24.04` job and the workspace Linux job |
| **P2-VERIFY-02** | **CLOSED — PASSED** | hosted `macos-15` portable matrix passed |
| Windows portable matrix | **PASSED** | `windows-2025` |
| Off-cohort P2 authority absence | **PASSED** | `0 tests` for the admission suite on Windows and macOS; the crate root's off-cohort `compile_fail` doctests ran there |

#### 4. Independent-review findings

None blocks P2 acceptance.

| Finding | Severity | Disposition |
|---|---|---|
| P2-DOC-01 | MINOR | **RESOLVED BY P2 ACCEPTANCE SYNC**: ADR-0024's "Implementation authority" header, its Consequences paragraph and its owner-decision tables now state P1 **accepted**, P2 **accepted** and P3+ **not authorised**, and reference this decision. The accepted architectural contract of sections A to N is unchanged, and no historical text was rewritten |
| P2-RISK-01, P2-MIN-03, P2-MIN-04, P2-MIN-05, P2-MIN-06, P2-MIN-07 | MINOR | **open, nonblocking**; no crate, test or workflow file edited |
| P2-BL-01, P2-BL-02, P2-BL-03 | BACKLOG_NONBLOCKING | unchanged |
| P1-DOC-02 | MINOR | **RESOLVED** by the P2 implementation's README edition |
| P1-TEST-01 | MINOR | **open, nonblocking**; P2 does not touch the plan parser |
| P1-PARSE-01, P1-PARSE-02, P1-LIFE-01, P1-SER-01, P1-CI-01, P1-TEST-02 | BACKLOG_NONBLOCKING | unchanged |

One documentation-only hygiene item is carried for a later authorised touch: the comment above
`helm-evidence.yml`'s `cargo test -p helm-launch` step still reads "Portable P1 model only", which
is now stale because that step also executes the P2 admission suite. No workflow was edited during
owner acceptance.

#### 5. Historical authority and next gate

This decision changes no product code, test, workflow, Cargo file, experiment, evidence or the
independent review, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 IS ACCEPTED. HELM-LAUNCH P2 IS ACCEPTED. HELM-LAUNCH P3+ IS NOT AUTHORISED.**

**Next gate: OWNER DECISION ON WHETHER TO AUTHORISE HELM-LAUNCH P3 — UNSAFE BACKEND AND CHILD
CONTRACT.**


<a id="helm-launch-p3-authorised"></a>

### Owner decision 2026-09-18 — **HELM-LAUNCH P3 AUTHORISED**: unsafe Linux x86_64 backend and the closed child contract; P4/P5 not authorised

**`HELM_LAUNCH_P3_AUTHORISED`.** The repository owner, Djomla83, authorises HELM-LAUNCH P3 as the
**third helm-launch product implementation slice**, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md), the
[P1 acceptance of 2026-09-17](#helm-launch-p1-accepted) and the
[P2 acceptance of 2026-09-18](#helm-launch-p2-accepted). **P1 and P2 remain accepted.** This is an
**implementation authorisation only**: it is not acceptance of P3, and not product acceptance of the
complete helm-launch 0.1 module. Every earlier section, including every ADR, D-7 and Trial #1, #2
and #3 record, is left as written.

| Item | Value |
|---|---|
| Accepted P2 implementation base | `c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` |
| P2 acceptance record | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| ADR-0024 | **ACCEPTED** |
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **AUTHORISED** by this decision |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |

#### 1. The safety transition this decision makes

P2 ended at an `AuthorizedLaunch` **with no consumer**: no function in the crate could turn an
authorisation into a process. P3 authorises the **first crate-private consumer** of that value, on
the Linux x86_64 cohort only. After P3 the crate can create **one direct child** and attempt to
execute the **exact descriptor the caller admitted**, internally.

| Property | After P2 | Authorised in P3 |
|---|---|---|
| **Process creation** | none | **AUTHORISED INTERNALLY IN P3** |
| **Process execution attempt** | none | **AUTHORISED INTERNALLY IN P3** |
| **Public process-execution API** | none | **NONE** — unchanged |
| **Unsafe** | none | **AUTHORISED ONLY UNDER `crates/helm-launch/src/backend/`** |
| **Host privilege acquisition** | none | **NONE** |
| **Process-group sweep** | none | **NOT AUTHORISED IN P3** |

External callers still have **no way at all** to make `helm-launch` create a process: no public
`launch`, no public spawn, no public process handle, no public pidfd, child pid or raw descriptor.

#### 2. P3 scope — what may be implemented

Unsafe Linux x86_64 process creation, the closed post-clone child contract, and a **crate-private,
test-only minimal execution path**:

* `src/backend/mod.rs`, `src/backend/spawn.rs`, `src/backend/child.rs`, and — if it makes the unsafe
  boundary smaller and clearer — one private `src/backend/syscall.rs`. There must still be **one**
  reviewed raw syscall implementation;
* one raw x86_64 `core::arch::asm!` syscall shim, decoding `-errno` without host errno or TLS;
* parent preparation from a consumed `AuthorizedLaunch` (plan section 8.2);
* raw `rt_sigprocmask` signal blocking around `clone3`, and the restore;
* `clone3(CLONE_PIDFD)` with `exit_signal = SIGCHLD`, and immediate pidfd ownership;
* parent-side `setpgid(child, child)` as the first system call after `clone3`;
* the complete accepted post-clone child sequence (plan section 8.4);
* the exec-status pipe and its fixed 8-byte failure record;
* crate-private minimal spawn and result structures;
* bounded, non-blocking direct-child reap;
* direct-child pidfd `SIGKILL` **only** for the fixed pre-exec timeout and for bounded P3 test
  cleanup that must not leak a child;
* an internal, non-public `launch_minimal` for P3 tests;
* P3 structural, trace and Linux integration tests;
* a non-default `test-fault-injection` feature.

#### 3. P3 scope — what must not be implemented

Public `launch()`; `LaunchOutcome`; any public execution entry point or process handle; receipt
emission from a real launch; the P4 observation loop; plan-driven run timeouts; `SIGTERM`/grace
lifecycle; general stdout/stderr drain policy; the process-group `SIGKILL` sweep; process-tree
containment; Wine; orchestration; sandboxing.

**No new public execution API.** `launch`, `launch_minimal`, `spawn`, `SpawnedChild`,
`PreparedLaunch`, `Backend`, `LaunchOutcome`, pidfd access, child pid access and raw descriptor
values must **not** be exported. `backend` stays a **private, `cfg`-gated module**, and no backend
item is publicly re-exported. If the implementation appears to require a new **public** process or
error API it must **stop and return `OWNER DECISION REQUIRED`**, not silently enter P4.

#### 4. P3 versus P4 — group authority without a sweep

P3 **establishes** potential group-sweep authority and **must not use it**.

* The parent's `setpgid(child_pid, child_pid)` returning success is the **only** positive
  group-authority event. It is recorded internally as a boolean so P4 can later consume it.
* The child still performs `setpgid(0, 0)` as stage 5 of its closed sequence.
* P3 **must not** issue `kill(-child_pid, SIGKILL)` or any other negative-pid group signal. The
  guarded every-path group sweep belongs to **P4**.

#### 5. Fixed internal bounds, not run-timeout semantics

The slice table's "timeouts absent" means the **P4 application run lifecycle**. P3 is authorised to
implement exactly the two fixed internal bounds its own contract requires:

| Constant | Value | Why P3 needs it |
|---|---|---|
| `SPAWN_CONFIRM_TIMEOUT_MS` | 5 000 | S6 pre-exec stall handling and exec-status observation |
| `POST_KILL_REAP_MS` | 5 000 | bounded non-blocking reap after a direct-child `SIGKILL` |

These are **not** public run-timeout semantics. `plan.timeout_ms` execution, `SIGTERM`, `grace_ms`
execution, `POST_EXIT_DRAIN_MS` lifecycle and general run deadlines stay with **P4**.

#### 6. Unsafe confinement

The crate root stays `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`. Exactly `src/backend/mod.rs`
may carry the scoped `#![allow(unsafe_code)]`, covering only its backend descendants. No other
source module and no Rust test source may relax `unsafe_code`. The backend additionally denies
`clippy::indexing_slicing`, `clippy::arithmetic_side_effects`, `clippy::as_conversions` and
`clippy::missing_safety_doc`, and keeps `clippy::undocumented_unsafe_blocks`,
`clippy::multiple_unsafe_ops_per_block` and `unsafe_op_in_unsafe_fn` at `deny`. Every unsafe block
carries a specific `// SAFETY:` comment describing the actual invariant; one giant block is refused.

**Closed list of authorised unsafe operations.** (1) raw `rt_sigprocmask` around `clone3` in the
parent; (2) raw `clone3`; (3) immediate `OwnedFd::from_raw_fd(pidfd)` after a successful `clone3`;
(4) crossing into the child entry with the prepared `ChildPlan` pointer; (5) raw child syscalls of
the closed post-clone sequence; (6) the x86_64 `asm!` syscall shim itself. **Anything else stops and
returns `OWNER DECISION REQUIRED`.** In particular, parent-side operations that safe `rustix` can
perform must not be reimplemented as raw unsafe.

**Forbidden even inside the backend:** additional FFI; `libc::syscall`; `fork`; `vfork`;
`CLONE_VM`; `CLONE_VFORK`; `CLONE_FILES`; `CLONE_THREAD`; `pidfd_open`; `mmap`; `transmute`;
`static mut`; unchecked slice or string construction; arbitrary raw pointer arithmetic; and
allocation, locks, formatting, printing, panic or unwind, destructor-dependent state, `std`, `alloc`
or `rustix` calls **in the child**. No `/proc/self/fd` execution. No `std::process::Command` in
product backend code.

#### 7. Dependencies

`rustix` stays pinned at `=1.1.4` on the Linux x86_64 cohort and may expand only as P3 genuinely
requires — expected `std`, `fs`, `process`, `pipe`. The `event` feature must **not** be enabled
unless a concrete P3 requirement proves it unavoidable; P4 owns the poll and event observation loop.
A target-specific `libc = "=0.2.189"` — the already locked, vetted version — may be added **for
constants only**: product Rust must not call `libc::syscall`, any `libc` function or any `extern`
`libc` function. No direct `linux-raw-sys` without a prior owner decision. No `build.rs`. No new
dependency version unless unavoidable and reviewed.

#### 8. Platform boundary

All P3 implementation exists only under
`cfg(all(target_os = "linux", target_arch = "x86_64"))`. Windows, macOS and other architectures keep
the portable P1 model and, where P2 policy already defines it, **no authority or backend API off the
cohort**. There is no generic Unix backend, no Linux aarch64 backend and no fallback.

#### 9. Test-process execution is expected in P3

Unlike P1 and P2, P3 Linux tests **are** expected to create and execute purpose-built test
processes. That is ordinary product validation of the newly authorised backend. It is **not**
LAUNCH-EXEC-01, Trial #3, Trial #4 or D-7 activity. `launcher_spike`, the frozen Python runner and
the frozen helper ELFs must not be used; test ideas are reimplemented independently. A report
fixture must pass a **producer self-test** before any launcher test consumes its report (the
Trial #3 X2c rule). A tiny, newly written, test-only C host-condition helper is permitted only where
Rust cannot express the host condition without adding unsafe outside `src/backend` — for example
`pthread_atfork` host setup — and never as a C implementation of the launcher.

#### 10. Boundary and next gate

This decision is recorded in documentation only. The authority commit changes no product code, test,
workflow, Cargo file, ADR contract, experiment, evidence or independent review, and does not touch
`main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 IS ACCEPTED. HELM-LAUNCH P2 IS ACCEPTED. HELM-LAUNCH P3 IS AUTHORISED.
HELM-LAUNCH P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: IMPLEMENT HELM-LAUNCH P3 — UNSAFE LINUX X86_64 BACKEND AND THE CLOSED CHILD CONTRACT
ONLY, then ONE FRESH INDEPENDENT UNSAFE REVIEW OF THE P3 CANDIDATE.**


<a id="helm-launch-p3-author-review-disposition"></a>

### Owner disposition 2026-09-18 — **HELM-LAUNCH P3 AUTHOR-REVIEW FINDINGS**: bounded correction required, publication blocked, independent review still owed

**`HELM_LAUNCH_P3_AUTHOR_REVIEW_DISPOSITIONED`.** The repository owner, Djomla83, dispositions the
findings of the **author self-review** of the HELM-LAUNCH P3 implementation candidate, recorded in
[HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md](implementation/HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md).
**P1 and P2 remain accepted, P3 remains authorised, and P4 and P5 remain not authorised.**

| Item | Value |
|---|---|
| P3 authority record | `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60` |
| P3 implementation candidate | `afe8922bebd0c85ead7e58d146b738b84141f096` |
| Author self-review | `168fe1339dd20bdecc8d5c0f111ef5f185493e9f` |
| HELM-LAUNCH P3 | **AUTHORISED / IMPLEMENTED CANDIDATE / CORRECTION REQUIRED** |
| Publication | **BLOCKED** pending the bounded correction |
| Independent P3 unsafe review | **STILL OWED** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The review that was performed was not independent

The agent session that produced the P3 authority and implementation commits also produced
`168fe133`. **That commit is therefore an AUTHOR SELF-REVIEW, not an independent review**, and it
**does not satisfy** the independent P3 unsafe-review gate that ADR-0024 and the
[P3 authorisation](#helm-launch-p3-authorised) require. Its technical work is accepted as **useful
diagnostic evidence only**. The artifact is named
`HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md` so that it cannot be mistaken for the missing gate.

#### 2. Finding dispositions

| Finding | Severity | Disposition |
|---|---|---|
| **P3R-00** | IMPORTANT / process | **ACCEPTED.** A genuinely fresh independent unsafe review remains mandatory, **after** the correction below |
| **P3R-01** | IMPORTANT / CI proof | **ACCEPTED.** The committed release injection-absence proof is defective and **must be corrected before publication** |
| **P3R-02** | IMPORTANT / child closed world | **ACCEPTED.** See the ruling below |
| **P3R-03** | MINOR | **ACCEPTED AS OPEN.** No numeric-PID fallback is authorised |
| **P3R-04 … P3R-08** | MINOR | remain **OPEN**, carried forward; not in scope for the bounded correction |
| **P3R-G1 / P3R-G2** | GATE_PENDING | unchanged: real Linux backend execution and real `strace` child-window evidence |

#### 3. Ruling on P3R-02

The actual test and debug child machine code contains a `memcpy@PLT` call and compiler-emitted panic
paths. **This does not satisfy the intended closed child contract merely because those paths are
believed safe or unreachable.**

* **ADR-0024 and the productization plan are NOT amended.** No owner contract change is made by this
  disposition, and the accepted technical contract of those documents is untouched.
* **The finding is NOT resolved by documentation.** The implementation is corrected, and
  machine-code evidence is added.

**Required target.**

> **NORMAL P3 CHILD MACHINE CODE MUST NOT CALL GLIBC, LIBSTD, ALLOCATOR, PANIC/UNWIND OR OTHER
> EXTERNAL RUNTIME HELPERS BETWEEN CHILD ENTRY AND `execveat` / `exit_group`.**

Internal helm-launch child helpers and the raw syscall shim are acceptable if they themselves satisfy
the same closed contract.

#### 4. The evidence model now has two independent gates

`strace` proves syscalls and **cannot** detect `memcpy`, panic-helper calls, allocator calls that
issue no syscall in that invocation, or userspace PLT helper code. The final P3 evidence model must
therefore contain **both**:

| Gate | Proves |
|---|---|
| **MACHINE-CODE GATE** | no forbidden userspace runtime helper in the child closure |
| **STRACE GATE** | no forbidden syscall in the runtime child window |

**Neither substitutes for the other.**

#### 5. Bounded correction scope

The correction may change only `crates/helm-launch/src/backend/**`, `crates/helm-launch/tests/**`,
the crate manifest if strictly required, the helm-launch workflow, the workspace workflow if the P3
plumbing genuinely requires it, `tools/tests/**`, and small test-support scripts the new proof needs.

It must **not** publish, start P4, add a public `launch()`, add `LaunchOutcome`, add a process-group
sweep, add a numeric-pid signalling fallback, add a `pidfd_open` fallback, change the accepted P3
mechanism, weaken unsafe confinement, weaken the closed child contract, modify frozen LAUNCH-EXEC
evidence, rerun Trial #3 or authorise Trial #4.

**S6 needs no correction.** The owner accepts the author-review analysis that the deliberately
retained parent stdin writer makes the pre-exec stall deterministic rather than race-dependent; the
correction reconfirms only that the retention cannot affect a normal product build.

#### 6. Boundary and next gate

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR, experiment or evidence, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 AND P2 ARE ACCEPTED. P3 IS AUTHORISED, IS AN IMPLEMENTED CANDIDATE, AND REQUIRES
CORRECTION. P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: BOUNDED P3 CORRECTION OF P3R-01 AND P3R-02, then ONE GENUINELY FRESH INDEPENDENT UNSAFE
REVIEW OF THE CORRECTED P3 CANDIDATE BEFORE PUBLICATION.**


<a id="helm-launch-p3-independent-review-disposition"></a>

### Owner disposition 2026-09-19 — **HELM-LAUNCH P3 INDEPENDENT-REVIEW FINDINGS**: independence satisfied, P3R-01 and P3R-02 verified fixed, two new IMPORTANT findings accepted, publication still blocked

**`HELM_LAUNCH_P3_INDEPENDENT_REVIEW_DISPOSITIONED`.** The repository owner, Djomla83, dispositions
the findings of the **genuinely independent unsafe review** of the corrected HELM-LAUNCH P3
candidate, recorded in
[HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md](implementation/HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md).
**P1 and P2 remain accepted, P3 remains authorised, and P4 and P5 remain not authorised.**

| Item | Value |
|---|---|
| Accepted P2 base | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| P3 authority record | `7bb016f5597c91a2aeb53ee0fb6c3eb38c3abe60` |
| P3 implementation candidate | `afe8922bebd0c85ead7e58d146b738b84141f096` |
| P3 **author self-review** | `168fe1339dd20bdecc8d5c0f111ef5f185493e9f` — **NOT INDEPENDENT**, diagnostic only |
| Author-review disposition | `ac823cd87e38ad5af5393ff3436c5c55b1f496df` |
| P3 bounded correction | `672228b8eeeef95cf72bb07051c0ccdec1ae261f` |
| **P3 independent unsafe review** | **`4c834415e8e0224b1eb1ce6c6546245cdfda0962`** |
| Review classification | **`HELM_LAUNCH_P3_INDEPENDENT_UNSAFE_REVIEW_NEEDS_FIX`** |
| HELM-LAUNCH P3 | **AUTHORISED / CORRECTED CANDIDATE / CORRECTION REQUIRED** |
| Publication | **BLOCKED** pending the bounded correction of P3R-10 and P3R-11 |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The independence precondition is satisfied

The review at `4c834415` was produced by a session that authored **none** of `7bb016f`, `afe8922`,
`168fe13`, `ac823cd` or `672228b`. It is therefore the **independent P3 unsafe-review gate artifact**
that [P3R-00](#helm-launch-p3-author-review-disposition) recorded as still owed, and the author
self-review at `168fe133` remains diagnostic evidence only. **P3R-00 is closed.**

The review re-derived the P3 safety case from source, from freshly emitted Linux x86_64 machine code
and from freshly built artifacts, and it did not take the author report conclusions on trust.

#### 2. The two previously required corrections are verified fixed

| Finding | Disposition |
|---|---|
| **P3R-01** — release fault-injection absence proof | **INDEPENDENTLY VERIFIED FIXED.** The proof now selects the exact artifact from the `compiler-artifact` record cargo itself emits, uses fresh build roots, inspects every archive member, and has a positive control that hits real object code. The independent review additionally built the **release** profile with `-Cdebug-assertions=on` and observed the `#[used]` marker present, which isolates the release absence to the `cfg` gate rather than to dead-code elimination |
| **P3R-02** — closed child world at machine level | **INDEPENDENTLY VERIFIED FIXED.** The child entry borrows the `ChildPlan`; regenerated assembly shows a child closure of 9 functions in the debug/test profile and 2 under release codegen, with 0 external runtime edges, 0 indirect call sites and exactly the 18 contract system calls in contract order. The release proof is **probative, not DCE-only** |

#### 3. Two new IMPORTANT findings are accepted

| Finding | Severity | Disposition |
|---|---|---|
| **P3R-10** | **IMPORTANT / Linux test correctness** | **ACCEPTED. MUST FIX BEFORE PUBLICATION.** The report fixture emits `pgid_is_self` while the parser handles only `tgid`, so `report.tgid` is always its default `0` and the direct-child identity assertion compares `0` to a real pid. The load-bearing descriptor-isolation case cannot pass on Linux x86_64, and the direct-child identity claim is unproven |
| **P3R-11** | **IMPORTANT / machine-proof fail-closedness** | **ACCEPTED. MUST FIX BEFORE PUBLICATION.** The child-closure checker recognises an indirect `call` but silently discards an indirect `jmp`, so a function-escaping control transfer can pass the gate unrecorded. That violates the owner-required fail-closed model for the gate this disposition line made load-bearing |
| **P3R-12** | MINOR | **OPEN.** Cached fixtures and the preloaded `pthread_atfork` helper live in a shared temporary directory and are reused without content verification |
| **P3R-13** | BACKLOG_NONBLOCKING | CI does not machine-prove the injection-enabled child; the independent review verified it clean |
| **P3R-14** | BACKLOG_NONBLOCKING | the release-library backend-absence step reads only the first emitted assembly |
| **P3R-03 … P3R-09** | MINOR | remain **OPEN**, carried forward, re-evaluated by the independent review and none promoted; no numeric-PID fallback is authorised |
| **P3R-G1 / P3R-G2** | GATE_PENDING | unchanged: real Linux backend execution and real `strace` child-window evidence |

#### 4. Required corrections

**P3R-10 — the report schema.** `pgid_is_self` must **not** be reinterpreted as `tgid`. The fixture
reports both facts explicitly: `tgid` as the executed image actual process identity, and
`pgid_is_self` as a boolean. Both are parsed explicitly, both are **mandatory** for the
report-capable fixture schema, and a missing or malformed required field **rejects the producer
report**. The direct-child identity assertion `report.tgid == child.pid()` must become genuinely
probative, and any group-leader claim must be a **separate** assertion. PID, TGID and PGID must not
be conflated. The producer self-test validates the new schema **before** any launcher consumer test
uses it, and a cached fixture must not bypass that validation.

**P3R-11 — function-escaping control transfers.** The checker must reason about
**function-escaping control transfers**, not only `call`. A direct `jmp` whose target is inside the
current function is ordinary intra-function control flow; a direct `jmp` that resolves to another
function or symbol is a **call-graph edge** and is traversed transitively exactly like a direct call,
so a tail jump to `memcpy`, a panic helper or an allocator fails exactly like a call. An **indirect
`call`**, an **indirect `jmp`** and an **unresolvable direct `jmp` target** must all **FAIL CLOSED**.
No target resolver for indirect transfers is authorised. A zero unresolved-edge result must not be
reportable while one exists.

#### 5. Bounded correction scope

**The intended correction is test-level and checker-level.** The expected changed files are the
backend test module and its report support, `tools/helm_launch_child_closure.py`,
`tools/tests/test_helm_launch_machine_proofs.py`, and strictly necessary test support.

The correction must **not** publish, start P4, add a public `launch()`, add `LaunchOutcome`, add a
process-group sweep, add a numeric-pid fallback, add a `pidfd_open` fallback, alter the `clone3`,
syscall or signal ABI, change the child stage order, change execution authority, weaken unsafe
confinement, weaken the closed child contract, or authorise Trial #4. **If either fix required
changing normal product unsafe or backend semantics, the work stops and returns
`OWNER DECISION REQUIRED`.** The minor and backlog findings of section 3 are **not** in scope.

**No P3 contract amendment is made.** ADR-0024 and the productization plan are **not** modified by
this disposition, and the accepted technical contract of sections A to N is untouched.

#### 6. Re-review gate

If both IMPORTANT findings are corrected **without** backend semantic changes, the next gate is
**ONE BOUNDED INDEPENDENT CORRECTION RE-REVIEW**. The session that authored `4c834415` may perform
it **provided it did not author or modify the correction**. A third full unsafe review is **not**
required while the correction stays strictly test-level and checker-level. **If normal product
backend or unsafe semantics changed, a full fresh independent unsafe review is required again.**

#### 7. Boundary and next gate

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR, experiment or evidence, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 AND P2 ARE ACCEPTED. P3 IS AUTHORISED, IS A CORRECTED CANDIDATE, AND REQUIRES THE
BOUNDED CORRECTION OF P3R-10 AND P3R-11. P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: BOUNDED P3 CORRECTION OF P3R-10 AND P3R-11, then ONE BOUNDED INDEPENDENT CORRECTION
RE-REVIEW BEFORE PUBLICATION.**


<a id="helm-launch-p3-conditional-branch-disposition"></a>

### Owner disposition 2026-09-19 — **HELM-LAUNCH P3 CONDITIONAL-BRANCH FINDING**: bounded rereview completed, P3R-10 and P3R-11 verified fixed, P3R-15 accepted, publication still blocked

**`HELM_LAUNCH_P3_CONDITIONAL_BRANCH_DISPOSITIONED`.** The repository owner, Djomla83, dispositions
the findings of the **bounded independent correction re-review** recorded in
[HELM-LAUNCH-P3-CORRECTION-REREVIEW.md](implementation/HELM-LAUNCH-P3-CORRECTION-REREVIEW.md).
**P1 and P2 remain accepted, P3 remains authorised, and P4 and P5 remain not authorised.**

| Item | Value |
|---|---|
| Accepted P2 base | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| P3 **full independent unsafe review** | `4c834415e8e0224b1eb1ce6c6546245cdfda0962` |
| Disposition of the independent findings | `7b749b8116b53ed07a8758b9feec50334546b7cd` |
| P3 bounded evidence correction | `f144d3004826276a5f2281ffea146c2e99645033` |
| **P3 bounded independent correction re-review** | **`5c577d45f62e2e3adc35a6c04a5ed0d8465a4366`** |
| Re-review classification | **`HELM_LAUNCH_P3_CORRECTION_REREVIEW_NEEDS_FIX`** |
| HELM-LAUNCH P3 | **AUTHORISED / CORRECTED CANDIDATE / CORRECTION REQUIRED** |
| Publication | **BLOCKED** pending the bounded correction of P3R-15 |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The bounded rereview was performed independently

`5c577d45` was produced by a session that authored **neither** `7b749b8` nor `f144d30`, the bounded
evidence correction it reviewed. It re-derived the corrected evidence from source, from freshly
emitted Linux x86_64 machine code and from parsers it executed itself, rather than from the author
report.

It is a **bounded** rereview and does **not** supersede `4c834415`. Together the two documents form
the **P3 pre-publication independent review record**; neither is complete alone.

#### 2. The two previously required corrections are verified fixed

| Finding | Disposition |
|---|---|
| **P3R-10** — report contract | **INDEPENDENTLY VERIFIED FIXED.** The emitted key set and the parser schema are both sixteen keys and match exactly. `Report` derives no `Default`, so no missing-value path can leave a zero behind. `tgid` is the executed image own process identity and `pgid_is_self` a separate group fact, each parsed and asserted separately; PID, TGID and PGID are not conflated. The producer self-test proves `tgid` against a pid the harness observed itself and `pgid_is_self` against a negative control. The parser was extracted and **executed** outside the repository, where the committed schema test and a further independent battery of refusal probes all behaved as required |
| **P3R-11** — indirect `jmp` fail-closedness | **INDEPENDENTLY VERIFIED FIXED.** Thirty-seven of thirty-seven synthetic probes behaved as the predecessor disposition required, and a **differential against the pre-correction checker** reproduces the original fail-open on `jmpq *%rax` and shows it closed, together with three further fail-open holes. Freshly generated assembly shows 1809 and 450 indirect transfers now seen where the old parser saw 1771 and 421, with zero unresolved transfers file-wide |
| **P3R-01** | **REMAINS FIXED.** The proof tool is byte-identical across the correction and was rerun: fresh roots, exact `compiler-artifact` selection, 128 and 11 archive members inspected, marker PRESENT in the debug feature build and ABSENT in release `--all-features` |
| **P3R-02** | **REMAINS FIXED.** Regenerated debug, release-codegen and injection child closures each report 0 external, 0 indirect and 0 unresolved edges. The release proof is **probative, not DCE-only**: the release-codegen path instantiates `child_main` with eighteen system calls while a plain release library eliminates it |

**Backend product semantics are byte-unchanged by `f144d30`**, confirmed by blob identity across
`4c834415` to `f144d30` for every backend file, both Cargo files, ADR-0024 and the plan, with no
workflow change. The full unsafe review at `4c834415` therefore remains valid for all of it.

#### 3. One new IMPORTANT finding is accepted

| Finding | Severity | Disposition |
|---|---|---|
| **P3R-15** | **IMPORTANT / machine-proof fail-closedness** | **ACCEPTED. MUST FIX BEFORE PUBLICATION.** The machine-code checker can **silently discard a conditional branch whose target escapes the current function**. The reviewer demonstrated a real emitted form from this crate — `jno <function symbol>`, in freshly generated release-codegen assembly — and proved that a synthetic `jno memcpy@PLT` placed in `child_main` **passes the current checker** with zero external, zero indirect and zero unresolved edges reported. That violates the owner-required fail-closed machine-code proof. Zero such branches occur inside the child closure today, so no current closure result is wrong |
| **P3R-16** | MINOR | **OPEN, NONBLOCKING.** `sig_blk`, `sig_ign` and `sig_cgt` are documented as required exactly once but are never presence-checked, because no `Report` field reads them. **Not to be fixed in the P3R-15 correction** |
| **P3R-12** | MINOR | **OPEN**, unchanged |
| **P3R-13 / P3R-14** | BACKLOG_NONBLOCKING | unchanged |
| **P3R-03 … P3R-09** | MINOR | remain **OPEN**, carried forward, none promoted; no numeric-PID fallback is authorised |
| **P3R-G1 / P3R-G2** | GATE_PENDING | unchanged: real Linux backend execution and real `strace` child-window evidence |

#### 4. Required correction

**P3R-15 — all function-escaping control transfers.** The checker must model every function-escaping
x86 control transfer, not only `call` and `jmp`. An explicit, reviewable branch vocabulary is
required — `call`/`callq`; `jmp`/`jmpq`; the canonical `Jcc` family; and `loop`, `loope`, `loopz`,
`loopne`, `loopnz` — rather than a loose "starts with `j`" heuristic as the primary authority.

For **every** direct transfer with a target, including `Jcc`, `jcxz`/`jecxz`/`jrcxz` and `loop*`:
a target inside the current function own body is **intra-function control flow**; a target that
resolves uniquely to another function or symbol is a **call-graph edge** that enters the closure and
is **traversed transitively**; anything else **FAILS CLOSED**. A conditional edge to another
function is still a possible execution edge. A conditional or tail branch to `memcpy`, `memmove`,
`memset`, an allocator, a panic or unwind helper or any glibc/runtime helper must fail exactly as
the equivalent call, so `jno memcpy@PLT` must fail. A conditional branch to a permitted internal
helper must add that helper to the closure and inspect its own branches, not merely mark the first
branch as seen.

The existing fail-closed rule for **indirect** `call` and `jmp` is preserved, and **no speculative
resolver is authorised**. A recognisably control-flow mnemonic that the explicit model does not
support must **FAIL CLOSED**: "not in the transfer pattern" must never mean "ordinary instruction".

Coverage must be proven by **table-driven** tests over the canonical conditional-jump vocabulary,
explicitly including `jno memcpy@PLT`, which reproduced P3R-15, together with representative
`je internal_helper`, `jne external_forbidden`, `jrcxz external_forbidden` and
`loop external_forbidden` cases. **No prior machine-proof negative may be weakened.**

#### 5. Bounded correction scope

**The intended correction is checker-level and test-level only.** The expected changed files are
`tools/helm_launch_child_closure.py` and `tools/tests/test_helm_launch_machine_proofs.py`, plus this
disposition in the status documents as a separate commit.

The correction must **not** touch `crates/helm-launch/src/backend/**`, the crate manifest,
`Cargo.lock`, any workflow, ADR-0024, the productization contract, the public API, any unsafe code
or any process semantics. It must not publish, start P4, add a public `launch()`, add
`LaunchOutcome`, add a process-group sweep, add a numeric-pid fallback, or authorise Trial #4.
**P3R-16 is explicitly out of scope.** **If the correction required changing product backend
semantics, the work stops and returns `OWNER DECISION REQUIRED`.**

**No P3 contract amendment is made.** ADR-0024 and the productization plan are **not** modified by
this disposition, and the accepted technical contract of sections A to N is untouched.

#### 6. Re-review gate

If the correction touches only the checker, its tests and this owner disposition, the next gate is
**ONE BOUNDED INDEPENDENT RE-REVIEW OF P3R-15**. A full unsafe review is **not** required. The
reviewer **must not have authored the correction**. **If product backend semantics changed, a full
fresh independent unsafe review is required again.**

#### 7. Boundary and next gate

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR, experiment or evidence, and does not touch `main`.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 AND P2 ARE ACCEPTED. P3 IS AUTHORISED, IS A CORRECTED CANDIDATE, AND REQUIRES THE
BOUNDED CORRECTION OF P3R-15. P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: BOUNDED CHECKER CORRECTION OF P3R-15, then ONE BOUNDED INDEPENDENT RE-REVIEW OF P3R-15
BEFORE PUBLICATION.**

<a id="helm-launch-p3-publication-failure-disposition"></a>

### Owner disposition 2026-09-19 — **HELM-LAUNCH P3 FIRST PUBLICATION FAILURE**: chain published, hosted validation failed, P3R-20 accepted

**`HELM_LAUNCH_P3_PUBLICATION_FAILURE_DISPOSITIONED`.** The repository owner, Djomla83, records the
**first hosted publication validation of the HELM-LAUNCH P3 candidate** and dispositions its
failure. The publication itself succeeded exactly as instructed; the hosted validation that the
publication existed to obtain did **not**. **P1 and P2 remain accepted, P3 remains authorised, and
P4 and P5 remain not authorised.**

| Item | Value |
|---|---|
| Previous remote milestone | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` |
| **Published head** | **`3a9368f845452110afc859ed899acae9384c7d9b`** |
| Publication | **ONE FAST-FORWARD PUSH**, `9fb0f8c..3a9368f`, twelve linear commits, no force, no tags |
| `main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED**; no merge to `main` is authorised |
| Pre-publication review record | `4c834415` + `5c577d45` + `3a9368f8` |
| **Hosted validation** | **FAILED** |
| **P3R-20** | **IMPORTANT / ACCEPTED / MUST FIX** |
| Classification of the failure | **TEST / EVIDENCE DEFECT** |
| Product launcher mechanism | **NOT IMPLICATED BY THIS FAILURE** |
| HELM-LAUNCH P3 | **PUBLISHED CANDIDATE / HOSTED VALIDATION FAILED** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The publication

The complete P3 chain was published to `docs/helm-launch-architecture` in a single fast-forward
push. Nothing was amended, rebased, squashed, cherry-picked or force-pushed, no publication commit
was created, no tag was pushed and `main` was not touched. The pre-publication review record was
complete at the published head: the author self-review, the full independent unsafe review, the
bounded P3R-10/P3R-11 re-review and the bounded P3R-15 re-review, the last classified
`HELM_LAUNCH_P3_P3R15_REREVIEW_PASSED_READY_FOR_PUBLICATION_CI` with zero BLOCKER and zero
IMPORTANT findings.

#### 2. The first hosted runs, preserved as they are

Three workflows triggered naturally from the push. **All three are attempt 1, and all three are
permanently preserved.**

| Workflow | Run | Attempt | Event | Conclusion |
|---|---|---|---|---|
| helm-launch portable model, Linux capability admission and Linux backend | `35442641728` | 1 | `push` | **FAILURE** |
| HELM Rust workspace Linux | `35442641743` | 1 | `push` | **FAILURE** |
| helm-bind cross-platform purity | `35442641707` | 1 | `push` | SUCCESS |

No LAUNCH-EXEC-01 workflow triggered, nothing was dispatched, and **no trial was run**.

**Nothing was retried.** No workflow was rerun, no job was rerun, no replacement run was dispatched
and no correction was pushed to the published head. A corrected run must arrive naturally from a
**new** SHA; it does not replace the historical result above.

#### 3. What the failure was

Both failures are the same root cause, in the P3 test harness's fixture builder, reached for the
**first time ever** on a hosted Linux runner. The backend suite is `cfg`-gated to Linux x86_64 and
the two earlier green runs of that workflow (`e32b2e17`, `94ce8dd3`) predate the P3 backend
entirely, so no earlier CI and no developer host could have reached it.

`fixture_binary` in [`crates/helm-launch/src/backend/tests.rs`](../crates/helm-launch/src/backend/tests.rs)
builds its fixtures in one shared temporary directory, with a staging name made unique only by
`std::process::id()`. **Parallel test threads share that pid.** Seventeen call sites request the
same content-addressed `report` fixture, so several threads invoke `rustc` concurrently with the
same source pathname and the same `-o` pathname; `rustc` derives its intermediate `.rcgu.o` names
from that output pathname, so the concurrent invocations delete and overwrite one another's
objects.

The two runs show the same race with different timing, and **different failing test sets**, which
is what proves it is a race and not a deterministic defect:

| Run | Symptom | Result |
|---|---|---|
| `35442641728` | `rust-lld: error: undefined hidden symbol: …` for the fixture's own CGUs | 76 passed, **4 failed** |
| `35442641743` | `rust-lld: error: cannot open …-cgu.0.rcgu.o: No such file or directory` | 78 passed, **2 failed** |

Downstream symptoms follow from the same cause: the loser of the staging rename gets `NotFound`,
and admission observing a fixture being replaced reports `MeasurementInstabilityDetected` — **that
last one is product code correctly refusing a genuinely unstable object**, not a defect.

#### 4. What the failure was not

The runner had every required tool: `strace 6.8`, `cc` and `rustc 1.95.0` were all verified usable
by the job's own tool check before any test ran. `rustc` was found and did run. The failure is
therefore **not** a `TEST ENVIRONMENT FAILURE` in the sense the harness's own message used, even
though the harness prints that wording for any `rustc` non-zero exit.

**No product backend code is implicated.** The failing code path is the test harness's fixture
builder. Real hosted Linux evidence was obtained for the launcher itself before the abort: the
backend suite was **not** `cfg`-skipped — twenty-six `backend::tests::` cases were present, and the
traced child-window cases passed on the hosted runner from both a single-threaded and a
multithreaded allocating parent, together with the `pthread_atfork` host condition, the `clone3`
UAPI record, the stage vocabulary, the full signal mask, the raw `rt_sigaction` layout,
`no_new_privs`, the empty environment, the admitted-directory identity, execution of the admitted
descriptor after its pathname is replaced, `ETXTBSY`, `ENOEXEC`, the eight-byte failure record and
the rule that a clean end of file is never success, and the parent establishing group authority
while issuing **no group signal**.

#### 5. What the failure blocked

Because the first test step failed, every later step of both Linux jobs was skipped. The following
required hosted evidence therefore **did not run at all** and remains owed:

* the debug and release-codegen child machine-code closure gates, and the release-library
  dead-code-elimination contrast;
* the fault-injection positive control and the release absence proof;
* the S5 and S6 bounded-cleanup cases;
* the Linux capability-admission suite and the `--nocapture` traced-window record;
* the Linux run of `p3_boundary` and `p2_boundary`;
* the P3R-15 conditional-branch machine-proof tests on Linux, skipped in **both** failing runs.

**P3 hosted validation is incomplete.** `P3R-G1` and `P3R-G2` remain `GATE_PENDING`.

#### 6. P3R-20 — accepted

| Finding | Severity | Disposition |
|---|---|---|
| **P3R-20** | **IMPORTANT / test and evidence harness** | **ACCEPTED. MUST FIX.** `fixture_binary` is not safe for concurrent construction. Its source and staging pathnames are not unique per compiler invocation, because `std::process::id()` is shared by every parallel test thread, so concurrent `rustc` invocations collide on the source pathname, the `-o` pathname and the intermediate object basenames derived from it. Classified **TEST / EVIDENCE DEFECT**; the product launcher mechanism is **not implicated** |

P3R-20 is adjacent to, but distinct from, the already-open P3R-12. P3R-12 is about a
content-addressed cache returning a file **without verifying its contents**, and was rated on the
basis that "on a fresh CI runner this is immaterial". P3R-20 is about the **construction and
publication** step not being concurrency-safe, and it manifests **precisely** on a fresh CI runner.
P3R-12 stays MINOR, open and **out of scope**.

#### 7. Required correction

Every actual compiler invocation must have its own source pathname and its own staging output
pathname — process id **plus** a process-local monotonic nonce, no randomness and no new
dependency — so that two concurrent invocations can never share a source pathname, an `-o`
pathname or a `rustc` intermediate object basename.

The final fixture stays content-addressed at `{fixture_root}/{stem}`, and publication must be
**atomic and no-replace**: a successful build publishes its unique staged executable by a link that
fails with `AlreadyExists` if another builder already published the same content-addressed fixture,
in which case this builder deletes its own staged file and uses the existing published path. An
unconditional `rename` is **not** acceptable, because it may replace an already-published inode,
and an existence test followed by a rename is **not** an acceptable solution to the race. A
process-local mutex may be an auxiliary optimisation only, never the correctness basis, and no
global build lock may be held across launcher execution.

A deterministic regression is required: several `Barrier`-synchronised threads calling the **real**
`fixture_binary` for the **same previously uncached** source, with real `rustc` compilation, proving
that every call succeeds, that every caller returns the same final path, that the published fixture
is executable and actually runnable, that the published object is not subsequently replaced, and
that no source or staging collision occurred. Sleep-based timing is not acceptable.

`atfork_helper` must be **inspected** for the same pattern. If no concurrent construction path is
reachable, that fact is documented and the function is left untouched; if the same race is
reachable, the work stops and returns `OWNER DECISION REQUIRED` rather than broadening the
correction.

A `rustc` that is missing or unusable remains a genuine `TEST ENVIRONMENT FAILURE`. A `rustc` that
runs and returns failure during fixture construction is a **fixture build failure**, and should say
so. Unrelated environment diagnostics are not to be changed.

#### 8. Bounded correction scope

**The intended correction is test-harness-level only.** The expected changed file is
`crates/helm-launch/src/backend/tests.rs`, plus this disposition in the status documents as a
separate commit.

The correction must **not** touch `child.rs`, `spawn.rs`, `syscall.rs`, `mod.rs`, `injection.rs`,
`authority.rs`, the public API, the crate manifest, `Cargo.lock`, any workflow, the machine-code
checker, the machine-proof tests, ADR-0024 or the productization contract. It must not publish,
rerun CI, start P4, add a public `launch()`, add a process-group sweep or authorise Trial #4.
**If normal product semantics must change, the work stops and returns `OWNER DECISION REQUIRED`.**

**No P3 contract amendment is made.** ADR-0024 and the productization plan are **not** modified by
this disposition, and the accepted technical contract of sections A to N is untouched.

#### 9. Re-review gate

If the correction touches only the status documents and the test harness, and normal backend
semantics remain unchanged, the next gate is **ONE BOUNDED INDEPENDENT REVIEW OF P3R-20**. A full
unsafe review is **not** required. The reviewer **must not have authored the correction**. **If the
product backend changes, a full fresh independent unsafe review is required again.**

#### 10. Boundary and next gate

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR, experiment or evidence, and does not touch `main`. The historical failed runs
`35442641728` and `35442641743` stay as they are: **no retry, no rerun, no replacement run.**

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 AND P2 ARE ACCEPTED. P3 IS A PUBLISHED CANDIDATE WHOSE HOSTED VALIDATION FAILED
AND IS INCOMPLETE. P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: BOUNDED HARNESS CORRECTION OF P3R-20, then ONE BOUNDED INDEPENDENT REVIEW OF P3R-20,
then a corrected publication and a new hosted Linux P3 CI run from a NEW SHA.**

<a id="helm-launch-p3-second-publication-failure-disposition"></a>

### Owner disposition 2026-09-19 — **HELM-LAUNCH P3 SECOND PUBLICATION FAILURE**: P3R-20 hosted-verified corrected, P3R-21 accepted

**`HELM_LAUNCH_P3_SECOND_PUBLICATION_FAILURE_DISPOSITIONED`.** The repository owner, Djomla83,
records the **second hosted publication validation of the HELM-LAUNCH P3 candidate**, on the
`P3R-20` correction chain, and dispositions its failure. **`P3R-20` is verified corrected on real
hosted Linux.** A different, previously masked defect — **`P3R-21`** — failed the run. **P1 and P2
remain accepted, P3 remains authorised, and P4 and P5 remain not authorised.**

| Item | Value |
|---|---|
| **Published head** | **`80ea89b8eef40dc1de68993eac3e140a4925d9b3`** |
| Publication | **ONE FAST-FORWARD PUSH**, `3a9368f..80ea89b`, three linear commits, no force, no tags |
| Chain | `86d798e` disposition + `f15d19d` correction + `80ea89b` bounded independent review |
| Pre-publication review record | `4c834415` + `5c577d45` + `3a9368f8` + `80ea89b8` |
| **`P3R-20`** | **HOSTED LINUX VERIFIED CORRECTED** |
| **`P3R-21`** | **IMPORTANT / ACCEPTED / MUST FIX** |
| `P3R-21` classification | **TEST / EVIDENCE CONTRACT DEFECT** |
| Product launcher mechanism | **NOT IMPLICATED BY THIS FAILURE** |
| P3 hosted validation | **INCOMPLETE AND NOT ACCEPTED** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The two publication results are both permanent

Two hosted validations now exist, on two different published heads. Neither replaces the other, and
neither was retried.

| Publication | Head | helm-launch | Workspace | helm-bind |
|---|---|---|---|---|
| **First** | `3a9368f8` | `35442641728` attempt 1 **FAILURE** | `35442641743` attempt 1 **FAILURE** | `35442641707` attempt 1 SUCCESS |
| **`P3R-20` correction** | `80ea89b8` | `35461333887` attempt 1 **FAILURE** | `35461333920` attempt 1 **SUCCESS** | not triggered — path filter unmatched |

**Nothing was retried.** No workflow was rerun, no job was rerun, no replacement run was dispatched,
and no fix was pushed to either published head. Both runs on `80ea89b` are **attempt 1**, event
`push`, naturally triggered. `helm-bind` and every `launch-exec-01` workflow correctly did **not**
trigger, because the published range touched none of their filtered paths — so **no trial workflow
ran and no trial was dispatched**.

#### 2. `P3R-20` is verified corrected on hosted Linux

The defect that failed the first publication is fixed, and the fix is proven by the run itself
rather than inferred from a green job:

* `backend::tests::concurrent_builders_of_one_fixture_publish_exactly_one_object` **executed on real
  Linux x86_64 and passed** — it was not `cfg`-skipped;
* every historical fixture-build collision signature is **absent** from the log: no
  `undefined hidden symbol`, no `cannot open …rcgu.o`, no `FIXTURE BUILD FAILURE`, no
  `TEST ENVIRONMENT FAILURE: rustc could not build fixture`, no failed staging rename;
* all seventeen `report_fixture` consumers passed, where four failed on fixture construction in
  `35442641728`;
* the workspace workflow `35461333920`, which failed from the same defect in `35442641743`,
  **completed successfully end to end**, including `cargo test -p helm-launch`, the
  `tools/tests` machine-proof suite (831 tests, `OK`) and `validate_docs.py`.

`P3R-20` therefore stands as **independently reviewed** (`80ea89b8`) **and hosted-verified
corrected**. Its correction is not reopened by this disposition.

#### 3. `P3R-21` — accepted

| Finding | Severity and area | Disposition |
|---|---|---|
| **`P3R-21`** | **IMPORTANT / test and evidence contract** | **ACCEPTED. MUST FIX.** `backend::tests::the_parent_establishes_group_authority_and_issues_no_group_signal` asserts `group_authority_established == true` for an ordinary, uncoordinated launch. That is **not an accepted P3 guarantee**. Classified **TEST / EVIDENCE CONTRACT DEFECT**; the product launcher mechanism is **not implicated** |

The exact failure, run `35461333887`, job `105945631799`, step `Run cargo test -p helm-launch
--locked`, exit code 101:

```
---- backend::tests::the_parent_establishes_group_authority_and_issues_no_group_signal stdout ----
thread '...' (4695) panicked at crates/helm-launch/src/backend/tests.rs:1426:5:
the parent's own setpgid(child, child) did not succeed, so no later slice could sweep

test result: FAILED. 80 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.79s
```

**Root cause.** The accepted P3 semantics are deliberately conservative and are unchanged by this
disposition:

* the parent, immediately after `clone3`, issues `setpgid(child, child)` as its **first** system
  call, and **only that call's success** sets `group_authority_established`;
* any parent-side error sets it `false`. There is no retry, no inference from the child, and no
  further interpretation;
* P3 consumes this fact for nothing — there is no process-group sweep and no negative-pid signal
  anywhere in the crate;
* separately, the child issues its own `setpgid(0, 0)` as stage 5 of its closed sequence, before
  `execveat`.

The executed image therefore leads its own process group **whichever of the two calls ran first**,
exactly as `backend/child.rs` already documents. Linux permits the parent's `setpgid(child, child)`
to fail with `EACCES` once the child has already executed. When the child wins that race, the group
state is still correct and the parent's authority fact is legitimately `false`. **The test treated a
permitted scheduling outcome as a failure.**

**This is a race, and the evidence proves it rather than assuming it.** On the same head
`80ea89b8`, on the same `ubuntu-24.04` image, the workspace run `35461333920` executed the same test
binary and the same test **passed**, while `35461333887` failed it. In the first publication run
`35442641728` the same test **passed**; every failure there was a `P3R-20` fixture-build failure.

#### 4. Hosted validation is incomplete

Because the default `cargo test -p helm-launch --locked` step failed, every later step of the
helm-launch job was **skipped**: the fault-injection suite, the release build, both child
machine-code closure proofs, the fault-injection positive control and release absence proof, the
Linux capability-admission cases, the P3 backend and traced-window cases, the S5/S6 fault-injection
cases, the off-cohort emptiness checks, the unsafe-confinement and boundary suites, the
repository-level confinement checks and the deterministic receipt identities.

`P3R-G1` and `P3R-G2` remain `GATE_PENDING`. **P3 hosted validation is not accepted.** A green job
colour is not accepted as evidence for any of those gates, and none of them may be inferred from the
successful workspace run.

#### 5. Correction authorised, bounded

A bounded correction of `P3R-21` is authorised. It may change **only** the status documents and
`crates/helm-launch/src/backend/tests.rs`. It must not change `child.rs`, `spawn.rs`, `syscall.rs`,
`mod.rs`, `injection.rs`, `authority.rs`, the public API, the crate manifest, `Cargo.lock`, any
workflow, the machine-code checker, the machine-proof tests, ADR-0024 or the productization
contract.

**The accepted group-authority rule is not amended and must not be weakened:**

> **ONLY A SUCCESSFUL PARENT-SIDE `setpgid(child, child)` ESTABLISHES GROUP-SWEEP AUTHORITY.**

The correction must not retry `setpgid`, must not infer parent authority from the child's
`setpgid(0, 0)`, must not read `EACCES` as authority, must not promote an observed process-group
identity into authority, must not add a group sweep and must not change P4 semantics. **If fixing
the test requires changing product semantics, the work stops and returns `OWNER DECISION
REQUIRED`.**

The ordinary-launch test must assert the deterministic contract — the executed image leads its own
process group, and P3 issues no group signal — and must **not** require either value of
`group_authority_established`. The positive parent-authority fact must not be lost: it is to be
asserted deterministically under the existing test-only **S6** pre-exec stall, which holds the child
before `execveat` so the parent's `setpgid` cannot lose the race. **Process-group state and
group-sweep authority stay two separate facts, separately asserted.**

The existing trace contract is not weakened: it must continue to prove that
`setpgid(child, child)` is the first parent syscall after `clone3`, and it must **not** be made to
require that call to return `0` on an uncoordinated run.

**No P3 contract amendment is made.** ADR-0024 and the productization plan are **not** modified by
this disposition, and the accepted technical contract of sections A to N is untouched.

#### 6. Re-review gate

If the correction touches only the status documents and the test harness, and normal backend
semantics remain unchanged, the next gate is **ONE BOUNDED INDEPENDENT REVIEW OF `P3R-21`**. A full
unsafe review is **not** required. The reviewer **must not have authored the correction**. **If the
product backend changes, a full fresh independent unsafe review is required again.**

#### 7. Boundary and next gate

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR, experiment or evidence, and does not touch `main`. The historical failed runs
`35442641728`, `35442641743` and `35461333887` stay as they are: **no retry, no rerun, no
replacement run.**

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1 AND P2 ARE ACCEPTED. P3 IS A PUBLISHED CORRECTED CANDIDATE WHOSE SECOND HOSTED
VALIDATION FAILED AND IS INCOMPLETE. P4 AND P5 ARE NOT AUTHORISED.**

**Next gate: BOUNDED HARNESS CORRECTION OF `P3R-21`, then ONE BOUNDED INDEPENDENT REVIEW OF
`P3R-21`, then a corrected publication and a new hosted Linux P3 CI run from a NEW SHA.**

<a id="helm-launch-p3-accepted"></a>

### Owner decision 2026-09-19 — **HELM-LAUNCH P3 ACCEPTED**; P4 and P5 not authorised

**`HELM_LAUNCH_P3_ACCEPTED`.** The repository owner, Djomla83, accepts HELM-LAUNCH P3 as the
**third helm-launch product implementation slice**, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md), the
[P3 authorisation of 2026-09-18](#helm-launch-p3-authorised), the
[P2 acceptance of 2026-09-18](#helm-launch-p2-accepted) and the
[P1 acceptance of 2026-09-17](#helm-launch-p1-accepted). **P1 and P2 remain accepted.** This is
**not** product acceptance of the complete helm-launch 0.1 module. Every earlier section, including
every ADR, D-7, Trial #1, #2 and #3 record, and both failed P3 publications, is left as written.

| Item | Value |
|---|---|
| **Accepted P3 implementation head** | **`8a359ee4215b6c803dcc5b527010612dabbd110b`** |
| Accepted P3 authority record | [P3 authorised 2026-09-18](#helm-launch-p3-authorised) |
| Full independent unsafe review | `4c834415` — [review](implementation/HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md) |
| Bounded independent re-reviews | `5c577d45` (correction), `3a9368f8` ([`P3R-15`](implementation/HELM-LAUNCH-P3-P3R15-REREVIEW.md)), `80ea89b8` ([`P3R-20`](implementation/HELM-LAUNCH-P3-P3R20-REVIEW.md)), `8a359ee4` ([`P3R-21`](implementation/HELM-LAUNCH-P3-P3R21-REVIEW.md)) |
| Review findings, every review | **0 BLOCKER**, **0 IMPORTANT** |
| Accepted P2 base | `c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` |
| Publication | **ONE FAST-FORWARD PUSH**, `80ea89b..8a359ee`, no force, no tags |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` — **UNCHANGED** |
| helm-launch hosted run | **`35467256138`**, run 5, attempt 1, `push`, workflow `360677034` — **SUCCESS** |
| Workspace hosted run | **`35467255952`**, run 37, attempt 1, `push`, workflow `352797925` — **SUCCESS** |
| helm-bind hosted run | not triggered — path filter unmatched, verified from workflow source |
| LAUNCH-EXEC-01 trial workflow | **NONE RAN**; no trial was dispatched |
| Retry / rerun / replacement | **NONE** |
| helm-launch 0.1 complete module | **NOT YET PRODUCT-ACCEPTED** |
| HELM-LAUNCH P4 / P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The hosted gates this acceptance rests on

**A green job colour is not the evidence.** Each gate below was read from the executed log of the
first natural run of `8a359ee`.

| Gate | Result |
|---|---|
| **`P3R-20` Linux concurrency regression** | **PASSED** — `concurrent_builders_of_one_fixture_publish_exactly_one_object` executed and passed on Linux in **both** workflows; no historical collision signature present |
| **`P3R-21` ordinary Linux regression** | **PASSED** — `the_executed_image_leads_its_group_and_p3_issues_no_group_signal` executed and passed in both workflows; the default lib suite moved from **80 passed / 1 failed** to **81 passed / 0 failed** |
| **`P3R-21` S6 positive-authority regression** | **PASSED** — `injected::a_child_that_stalls_before_exec_is_bounded_killed_and_reaped` executed and passed with `group_authority_established == true` |
| **P3 Linux runtime validation** | **PASSED** — **29 backend tests passed, 0 failed**, 1 ignored by design; not `cfg`-skipped |
| **P3 machine-code closed-world validation** | **PASSED** — debug closure 9 functions and release closure 2 functions, each **0 external, 0 indirect, 0 unresolved, 0 unsupported**; the release child is **instantiated**, with 18 inlined syscall sites, so the proof is not dead-code elimination; both runs carry a live positive control |
| **P3 injection-confinement validation** | **PASSED** — marker **PRESENT** in the debug feature build, **ABSENT** from the release `--all-features` build, by exact artifact selection |
| **P3 `strace` child-window validation** | **PASSED** — single-threaded, multithreaded-allocating and `pthread_atfork` cases under real `strace` 6.8 |
| **P3 S5 validation** | **PASSED** — `injected::a_child_that_dies_before_exec_without_a_record_is_indeterminate_never_success` |
| **P3 S6 bounded-cleanup validation** | **PASSED** — `PreExecStatusTimeout`, one pidfd `SIGKILL`, bounded reap, `Signaled { signal: 9 }`, no zombie, no sweep |
| **P3 public API boundary** | **UNCHANGED / NO PUBLIC `launch()`** — `test_no_public_launch_api_exists` and `the_crate_root_proves_the_absence_of_a_public_execution_path` both green |
| **Process-group sweep** | **ABSENT** — `test_no_process_group_signal_exists_anywhere_in_the_crate` green |
| **N3 real privilege transition** | **UNVALIDATED / NONCLAIM** — nothing attempts a privilege transition and no test claims one was prevented |

Off the Linux x86_64 cohort, `windows-2025` and `macos-15` both ran the off-cohort proofs
successfully: the admission suite is empty and **no backend test exists**. No Linux backend result
is inferred from either.

#### 2. Accepted P3 product boundary

**P3 includes:** Linux x86_64 internal process creation; `clone3(CLONE_PIDFD)` with
`exit_signal = SIGCHLD`; a pidfd-owned direct-child lifecycle; an internal authorised-object
execution attempt; `execveat(fd, "", ..., AT_EMPTY_PATH)`; the closed raw post-clone child contract;
scoped `unsafe` **only** under `crates/helm-launch/src/backend/`; the parent `setpgid` attempt and
the conservative group-authority fact; the fixed internal pre-exec confirmation bound; bounded
direct-child pidfd `SIGKILL` cleanup; the test-only fault injection; and the machine-code and
syscall-window evidence.

**P3 does NOT include:** a public `launch()`; `LaunchOutcome`; a public process handle; the P4 run
lifecycle; a general run timeout; `SIGTERM` or grace policy; a process-group sweep; a general
stdout/stderr drain policy; real launch receipt emission; any sandbox or containment; Wine
integration; and N3 real privileged-transition validation.

A clean exec-status end-of-file remains **`ExecStatus::Indeterminate`**. It is **not** positive exec
success, and **no `ExecSucceeded` product state is introduced** — its absence is proven by a
`compile_fail` doctest.

#### 3. The accepted group-authority semantics are unchanged

> **ONLY A SUCCESSFUL PARENT-SIDE `setpgid(child, child)` ESTABLISHES GROUP-SWEEP AUTHORITY.**

The parent issues `setpgid(child, child)` as its **first** system call after `clone3`. Only that
call's success sets `group_authority_established`; any error establishes nothing, is not retried and
is not interpreted further. Nothing is inferred from the child's own `setpgid(0, 0)`, which the
child issues independently as stage 5 of its closed sequence, before `execveat`.

**Executed-image group leadership and parent-side sweep authority are two separate facts.**
`pgid_is_self` is an observation the executed image makes about itself and is never promoted into
authority. **P3 performs no process-group sweep**, and no negative-pid signal exists anywhere in the
crate.

#### 4. Both failed publications stay as permanent evidence

| Publication | Head | helm-launch | Workspace | helm-bind |
|---|---|---|---|---|
| **First** | `3a9368f8` | `35442641728` attempt 1 **FAILURE** | `35442641743` attempt 1 **FAILURE** | `35442641707` attempt 1 SUCCESS |
| **`P3R-20` correction** | `80ea89b8` | `35461333887` attempt 1 **FAILURE** | `35461333920` attempt 1 **SUCCESS** |  not triggered |
| **`P3R-21` correction** | **`8a359ee4`** | **`35467256138` attempt 1 SUCCESS** | **`35467255952` attempt 1 SUCCESS** | not triggered |

**NO RETRY. NO RERUN. NO REPLACEMENT.** No workflow or job was rerun, no replacement run was
dispatched, and no fix was pushed to either failed head. Every historical run remains **attempt 1**
with its original conclusion. The successful validation at `8a359ee` is **new evidence obtained
after reviewed corrections**; it does **not** convert either earlier publication into a pass.

#### 5. Findings carried forward

`P3R-20` and `P3R-21` are **CLOSED / VERIFIED CORRECTED** — each independently reviewed and then
verified on hosted Linux. `P3R-10`, `P3R-11` and `P3R-15` remain **CLOSED / VERIFIED FIXED**.

These remain **open and nonblocking**, and this acceptance invents no fix for any of them:

| Finding | Class | Carried state |
|---|---|---|
| `P3R-03` | MINOR | **OPEN.** `PidfdNotProvided` is unreachable under the `CLONE_PIDFD` kernel contract; **no numeric-PID fallback is authorised** |
| `P3R-04` | MINOR | **OPEN.** No traced exec-failure child window; narrowed by the machine-code gate covering `fail`'s own closure |
| `P3R-05` | MINOR | **OPEN, not promoted.** No handler-fire positive control for `pthread_atfork`; the registration control keeps the test probative |
| `P3R-06` | MINOR | **OPEN.** The trace pins the mask restore as the second parent syscall after `clone3`, tighter than the contract requires |
| `P3R-07` | MINOR | **OPEN.** The backend suite runs three times per job, each paying the intentional 5 s S6 bound and the tracer executions |
| `P3R-08` | MINOR | **OPEN.** `require_tool("env", …)` is GNU-specific; satisfied on `ubuntu-24.04` and a loud environment failure elsewhere |
| `P3R-09` | MINOR | **OPEN.** The crate README still describes the superseded grep-based injection proof and does not mention the machine-code gate; both **understate** the evidence |
| `P3R-12` | MINOR | **OPEN.** Cached fixtures and the preloaded `pthread_atfork` helper are reused from a shared temporary directory without content verification |
| `P3R-16` | MINOR | **OPEN.** `sig_blk`, `sig_ign` and `sig_cgt` are documented as required exactly once but are never presence-checked, because no `Report` field reads them |
| `P3R-17` | MINOR | **OPEN.** The machine-code vocabulary and backstop are case-sensitive; unreachable in the gate's actual input |
| `P3R-18` | MINOR | **OPEN.** `int` and `xbegin` are outside both the model and the backstop; unreachable in the gate's actual input |
| `P3R-19` | MINOR | **OPEN.** `is_branch_like("syscall")` is `True`, so correctness depends on guard ordering; implicit coupling covered by committed tests |
| `P3R-13` | BACKLOG_NONBLOCKING | CI does not machine-prove the injection-enabled child; the independent review verified it clean |
| `P3R-14` | BACKLOG_NONBLOCKING | The release-library backend-absence step reads only the first emitted assembly |
| `P3R21-M1` | MINOR | **OPEN.** The ordinary group test's name claims "no group signal", but its only signal assertion is the direct-child `!sigkill_sent`; the group-signal-absence claim is carried structurally by the traced-window scan |
| `P3R21-M2` | MINOR | **OPEN, with a concrete hosted manifestation.** The only reader of `group_authority_established` now lives behind `test-fault-injection`, so **default-feature Linux builds warn that the field is never read**. The warning is **nonblocking**: it invalidates no hosted P3 semantics or evidence, and `cargo clippy --all-features … -D warnings` stays green because `--all-features` compiles the reader. **No code is changed to remove it in this acceptance** |

#### 6. Authority after this acceptance

| Slice | Authority |
|---|---|
| HELM-LAUNCH P1 | **ACCEPTED** |
| HELM-LAUNCH P2 | **ACCEPTED** |
| HELM-LAUNCH P3 | **ACCEPTED** |
| HELM-LAUNCH P4 | **NOT AUTHORISED** |
| HELM-LAUNCH P5 | **NOT AUTHORISED** |
| Complete helm-launch 0.1 module | **NOT YET PRODUCT-ACCEPTED** |

#### 7. Boundary of this decision

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, experiment or evidence, and does not touch `main`. It modifies no review artifact. **Accepting
P3 is status synchronisation against the already accepted ADR-0024 contract, not a new architecture
decision**, and it promotes no traceability row to a stronger evidence class.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 AND P5 ARE NOT AUTHORISED. THE COMPLETE HELM-LAUNCH
0.1 MODULE IS NOT YET PRODUCT-ACCEPTED.**

**Next gate: OWNER DECISION ON WHETHER TO AUTHORISE HELM-LAUNCH P4.**

<a id="helm-launch-p4-authorised"></a>

### Owner decision 2026-09-19 — **HELM-LAUNCH P4 AUTHORISED**: lifecycle, termination, public launch and real receipt; P5 not authorised

**`HELM_LAUNCH_P4_AUTHORISED`.** The repository owner, Djomla83, authorises HELM-LAUNCH P4 as the
**fourth helm-launch product implementation slice**, under Accepted
[ADR-0024](adr/ADR-0024-launch-authority.md) and the
[P3 acceptance of 2026-09-19](#helm-launch-p3-accepted). **P1, P2 and P3 remain accepted. P4 is
authorised and NOT accepted. P5 remains not authorised.** This decision authorises **no** formal
trial: **no Trial #4 is authorised**, and P4 validation is ordinary product testing.

| Item | Value |
|---|---|
| Authority token | **`HELM_LAUNCH_P4_AUTHORISED`** |
| Accepted P3 base | `8a359ee4215b6c803dcc5b527010612dabbd110b` |
| P4 | **AUTHORISED / NOT YET ACCEPTED** |
| P5 | **NOT AUTHORISED** |
| Complete helm-launch 0.1 | **NOT YET PRODUCT-ACCEPTED** |
| Formal trial required for P4 | **NO** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |
| `unsafe` boundary | **UNCHANGED** — `src/backend/` only; P4 adds none |
| Closed child syscall contract | **UNCHANGED** |

#### 1. What P4 may add

`src/launch.rs`; the public Linux x86_64 `launch(AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError>`
and `LaunchOutcome`; the real parent observation loop; the plan-driven run deadline; the
`SIGTERM` → grace → `SIGKILL` lifecycle with a bounded post-`SIGKILL` observation and reap;
concurrent stdout/stderr draining with bounded in-memory prefixes; the guarded process-group
cleanup sweep; real launch classification; deterministic `LaunchReceipt` emission from an actual
launch; the Level 3 O/R/S/T/P product tests; and the CI those tests require.

#### 2. What P4 must not add

The P5 adversarial and evidence-contract slice; helm-evidence semantic receipt integration; Wine;
Proton; orchestration; sandboxing; cgroups; process-tree containment; an async launch API; an N3
privileged-transition claim; receipt signing, verification or provenance; any positive
exec-success claim; and Trial #4.

#### 3. The rules P4 does not get to weaken

* **`Err` versus receipt.** Before a direct child exists, every failure is `Err(LaunchError)` and
  **no receipt exists**. Once `clone3` has returned a child, `launch` returns `Ok(LaunchOutcome)`
  **with a receipt, whatever happened** — child setup failure, `execveat` failure, indeterminate
  exec evidence, timeout, a signal sent, a stream read failure, or an unobservable end. **A child
  attempt is never lost behind an error.**
* **No exec-success claim.** A clean exec-status end-of-file means
  `ExecStatus::Indeterminate(StatusEofWithoutRecord)` and nothing else. The run deadline starts at
  the `exec_status_eof` event, **not** at a confirmed exec. No API, receipt field, variant,
  `Display` text or document may introduce a positive exec-success claim.
* **`EndNotObserved` is latched.** Once the post-`SIGKILL` bound expires with no observed end, the
  receipt-facing `child_end` is `EndNotObserved` and **no later pidfd readiness, exit, signal, core
  dump, `ECHILD` or cleanup step may replace it**. It means only that no end was observed within the
  bound — never that the child is definitely still running.
* **The group sweep is guarded.** Without established group authority, **no sweep is issued** and
  the disposition is `not_issued_group_not_established`; authority is never inferred or rediscovered
  from an observed PGID. With authority, a **non-consuming** `waitid(P_PIDFD, WNOWAIT)` probe runs
  first: on `ECHILD` no sweep is issued and the disposition is `not_issued_child_already_reaped`.
  Otherwise **exactly one** `SIGKILL` group sweep is issued, **strictly before the reap**, on every
  completion path.
* **The sweep is not containment.** `group_sweep = issued` means only that the one call was issued.
  It does not mean a descendant received it, died, or that the process tree was contained. An
  escaped descendant may survive. P4 is **not** a sandbox.
* **Direct-child identity stays pidfd-based.** No `kill` by numeric pid, no `waitpid` by numeric
  pid, no numeric-pid fallback.
* **The receipt carries facts, not verdicts**, no raw stream bytes, no host path, no pid, no
  descriptor number, no timestamp, no duration and no authenticity claim. A digest identifies bytes
  and nothing more.
* **The `unsafe` boundary does not move.** If P4 needs new `unsafe` outside `src/backend/`, a change
  to the closed child syscall contract, a numeric-pid lifecycle fallback, an unbounded wait, a
  containment mechanism, a new public execution or error semantic beyond the accepted plan, or a new
  crate dependency, the work **stops** and returns `OWNER DECISION REQUIRED`.

#### 4. Acceptance gate

P4 is **not** accepted by this decision. Acceptance requires, in order: **one fresh independent P4
lifecycle and receipt review** by a reviewer who authored neither P4 commit; **real hosted Linux
x86_64 P4 validation**; and a separate **owner acceptance decision**. The accepted P3 evidence —
the fixture-concurrency regression, the group-test semantics, the machine-code closed-world proof,
the injection-confinement proof, the `strace` child-window proof, S5, S6 and unsafe confinement —
must remain green.

#### 5. Boundary of this decision

This decision is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR contract, experiment or evidence, and does not touch `main`. Authorising P4 is status
synchronisation against the already accepted ADR-0024 contract, not a new architecture decision, and
it promotes no traceability row to a stronger evidence class.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED AND NOT YET ACCEPTED. P5 IS NOT
AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1 MODULE IS NOT YET PRODUCT-ACCEPTED.**

**Next gate: P4 IMPLEMENTATION, then ONE FRESH INDEPENDENT P4 LIFECYCLE / RECEIPT REVIEW.**

<a id="helm-launch-p4-independent-findings-disposition"></a>

### Owner disposition 2026-09-20 — **HELM-LAUNCH P4 INDEPENDENT FINDINGS**: publication blocked, bounded correction ordered

**`HELM_LAUNCH_P4_INDEPENDENT_REVIEW_NEEDS_FIX`.** The repository owner, Djomla83, accepts
[`HELM-LAUNCH-P4-INDEPENDENT-LIFECYCLE-REVIEW.md`](implementation/HELM-LAUNCH-P4-INDEPENDENT-LIFECYCLE-REVIEW.md)
at `351f985bb239496c1bd336e39279e8ff742de5cc` as the **authoritative pre-correction P4 review**. It
returned **0 BLOCKER and 7 IMPORTANT**. **P4 publication is blocked** until a bounded correction and
one bounded independent re-review return **0 BLOCKER and 0 IMPORTANT**.

| Item | Value |
|---|---|
| Accepted P3 base | `ef50e8865a4f14965a115c5dc26c72e45d4af2c9` |
| P4 authority | `41ac4f90e85da0688c9e76cdeec92ba904fcfe1d` |
| P4 implementation candidate | `3d152ad0b7a422eb04160ba70697a457efd390c5` |
| Independent P4 lifecycle / receipt review | `351f985bb239496c1bd336e39279e8ff742de5cc` |
| Review result | **0 BLOCKER / 7 IMPORTANT / 8 MINOR / 1 BACKLOG / 1 GATE_PENDING** |
| P4 | **AUTHORISED / IMPLEMENTED CANDIDATE / CORRECTION REQUIRED** |
| P4 publication | **BLOCKED** |
| P5 | **NOT AUTHORISED** |
| Complete helm-launch 0.1 | **NOT PRODUCT-ACCEPTED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |
| ADR-0024 product contract | **UNCHANGED** by this disposition |
| Accepted total-bound formula | **UNCHANGED** |

#### 1. Owner dispositions, finding by finding

| Finding | Author rating | **Owner disposition** |
|---|---|---|
| `P4A-01` — spawn mask-restore semantic transition | reported | **NOT A FINDING.** The independent verdict is adopted: public `launch` cannot turn a post-child mask-restore failure into `Err`, and the P3 compatibility path is intact. This path is **not** to be rewritten for activity |
| `P4A-02` / `F-P4-02` — second `POST_KILL_REAP_MS` through `ChildHandle::Drop` | `MINOR` | **IMPORTANT / ACCEPTED / MUST FIX** |
| `P4A-03` — missing real-loop cases | `MINOR` | **SPLIT.** `EndNotObserved` real-loop integration: **MINOR / OPEN**, carried; foreign-reap / `ECHILD` real-loop gap (`F-P4-06`): **IMPORTANT / MUST FIX** |
| `P4A-04` / `F-P4-05` — scheduler-dependent, disjunctive real sweep test | `MINOR` | **IMPORTANT / ACCEPTED / MUST FIX** |
| `P4A-05` — pre-commit busy-loop defect | reported fixed | **CONFIRMED FIXED.** A deterministic regression guard is **required** as part of `F-P4-07` |
| `F-P4-01` — unbounded stream drain starves deadline processing | `IMPORTANT` | **IMPORTANT / ACCEPTED / MUST FIX** |
| `F-P4-03` — privacy canary cannot pass on the cohort | `IMPORTANT` | **IMPORTANT / ACCEPTED / MUST FIX.** It is a **test** defect; the accepted mechanism identifier and the receipt vocabulary do not change |
| `F-P4-04` — receipt determinism asserted over a scheduler-dependent fact | `IMPORTANT` | **IMPORTANT / ACCEPTED / MUST FIX.** It is a **test** defect; durable receipt semantics do not change |
| `F-P4-06` — no real foreign-reap / `ECHILD` case | `IMPORTANT` | **IMPORTANT / ACCEPTED / MUST FIX** |
| `F-P4-07` — accepted Level 3 O/R/S/T/P rows not delivered | `IMPORTANT` | **IMPORTANT / ACCEPTED / MUST FIX** |
| `F-P4-M1` … `F-P4-M8`, `F-P4-B1` | `MINOR` / backlog | **CARRIED, NON-BLOCKING.** Not part of this bounded correction |

#### 2. What the correction may not do

The correction is bounded to the seven `IMPORTANT` findings. It **must not** publish, start P5, alter
the accepted P4 architecture, **widen the accepted total bound**, add public API or public error
semantics, add `unsafe` outside `crates/helm-launch/src/backend/`, alter the raw child syscall
contract, add a numeric-pid direct-child lifecycle fallback, add containment or cgroups, add receipt
authenticity, change `helm-evidence` semantics, or authorise Trial #4. A correction that would need a
new child-window system call or a new `unsafe` operation **stops** and returns
`OWNER DECISION REQUIRED`.

#### 3. The accepted total bound is not renegotiated

`launch` returns within

`SPAWN_CONFIRM_TIMEOUT_MS + timeout_ms + grace_ms + POST_KILL_REAP_MS + POST_EXIT_DRAIN_MS`

plus finite scheduling slack, with **exactly one** `POST_KILL_REAP_MS` contribution. The correction
fixes the real adapter and the direct-child ownership hand-off so that the implementation meets that
formula; it does not add a second contribution to the model, the documentation or the tests.

#### 4. The pure lifecycle model stays the policy

`src/lifecycle.rs` was independently found sound. The correction is an **adapter, ownership, fairness
and evidence** correction. Any semantic change to the pure model **stops** and returns
`OWNER DECISION REQUIRED`.

#### 5. Boundary of this disposition

This disposition is recorded in documentation only. It changes no product code, test, workflow, Cargo
file, ADR contract, experiment or evidence, and does not touch `main`. It promotes no traceability row
to a stronger evidence class and accepts no P4 gate.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, IMPLEMENTED AS A CANDIDATE AND REQUIRES
CORRECTION. P4 PUBLICATION IS BLOCKED. P5 IS NOT AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1 MODULE IS
NOT PRODUCT-ACCEPTED.**

**Next gate: BOUNDED P4 CORRECTION, then ONE BOUNDED INDEPENDENT P4 CORRECTION RE-REVIEW.**

<a id="helm-launch-p4-first-publication-failure"></a>

### Owner disposition 2026-09-20 — **HELM-LAUNCH P4 FIRST PUBLICATION**: hosted validation FAILED, bounded test/evidence correction authorised

**`HELM_LAUNCH_P4_FIRST_PUBLICATION_HOSTED_VALIDATION_FAILED`.** The repository owner, Djomla83,
records that the first publication of the corrected P4 head failed its hosted Linux validation. The
two natural runs are **permanent historical evidence** and are preserved exactly as they stand.
**P4 hosted validation is NOT ACCEPTED.** A bounded **test / evidence** correction is authorised;
the P4 product contract is not reopened.

| Item | Value |
|---|---|
| Published P4 head | `94ee48da0cc412f0d8043e34b914c198615b923e` |
| `helm-launch` workflow — first natural run | `35499943908`, attempt 1, event `push` — **FAILURE** |
| its Linux job | `106049803287` — **FAILURE** |
| its Windows job | **SUCCESS** |
| its macOS job | **SUCCESS** |
| `HELM Rust workspace Linux` — first natural run | `35499943903`, attempt 1, event `push` — **FAILURE** |
| its Linux job | `106049803383` — **FAILURE** |
| P4 hosted validation | **NOT ACCEPTED** |
| P4 product mechanism | **NOT IMPLICATED** by any preserved evidence |
| Historical runs | **PRESERVED** — **NO RETRY, NO RERUN, NO REPLACEMENT** |
| ADR-0024 product contract | **UNCHANGED** by this disposition |
| P4 product contract | **UNCHANGED** by this disposition |
| Accepted total-bound formula | **UNCHANGED** |
| `unsafe` boundary | **UNCHANGED** — `crates/helm-launch/src/backend/` only |
| HELM-LAUNCH P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The three failures, classified

All three are **IMPORTANT** and all three are **test / evidence** defects. None is a product defect,
and no preserved evidence implicates the P4 product mechanism.

| Id | Case | Owner classification |
|---|---|---|
| `P4PUB-01` | `launch::tests::a_signalled_child_keeps_its_signal_number_and_its_core_flag` | **IMPORTANT — TEST / EVIDENCE CONTRACT DEFECT** |
| `P4PUB-02` | `backend::tests::a_disarmed_drop_guard_neither_signals_nor_waits` | **IMPORTANT — TEST HARNESS CLEANUP DEFECT** |
| `P4PUB-03` | `launch::tests::the_outcome_owns_no_descriptor_and_consumes_its_authorisation` | **IMPORTANT — TEST EVIDENCE ISOLATION / RACE DEFECT** |

**`P4PUB-01`.** The `segv-nocore` shape observed `Signaled { signal: 11, core_dumped: true }` where
the case expected `core_dumped: false`. The fixture set `RLIMIT_CORE.rlim_cur = 0` and the case
treated that as a universal Linux guarantee that the wait status must report no core. That
assumption is **invalid** on a Linux host whose core dumps are piped to a userspace handler:
`RLIMIT_CORE` need not control that path. The product preserved the kernel-reported wait
classification, which is exactly its obligation.

**`P4PUB-02`.** Both load-bearing assertions **passed** before the failure: the child survived the
disarmed `Drop`, and `Drop` returned in under 250 ms. No hidden second `SIGKILL` and no hidden second
`POST_KILL_REAP_MS` were observed. The later **harness cleanup** failed, because `kill -9` plus a
poll of `/proc/<pid>` with **no reap** can leave a zombie visible in `/proc` indefinitely.

**`P4PUB-03`.** The workspace run observed `before = 11`, `after = 10`. A descriptor count that
**falls** is not a descriptor leak. The same case passed on the same head in the `helm-launch`
workflow. The oracle reads the **process-global** descriptor table while the Rust test harness runs
cases in parallel, so its equality is not isolated.

#### 2. What the preserved evidence does and does not say

On real Linux, before the default-suite failure, these executed and **passed**: continuous-output
fairness; post-exit non-spin; the accepted total bound on the run-timeout path; the accepted total
bound on the retained-writer path; armed `Drop` cleanup; a reaped child not re-signalled; receipt
privacy; receipt determinism; 8 MiB concurrent dual streams; `POLLIN`+`POLLHUP` over 200
repetitions; S3 exit 127; guarded sweep disposition; the same-group and escaped-descendant ordinary
case; and the other Phase A/B and default cases.

These remain evidence **from a failed publication**. They **do not** make the overall P4 validation
pass: later named gates were skipped when the default suite failed.

#### 3. The authorised correction is bounded to tests and evidence

The correction **must not** change the public `launch` API, `LaunchOutcome`, the lifecycle policy,
the receipt schema or serializer, group-sweep semantics, `ChildHandle` product semantics, stream
fairness product code, the deadline implementation, the closed child contract or the `unsafe`
boundary. It must not weaken an oracle merely to obtain green: `RLIMIT_CORE == 0` must not be
encoded as a fixed `core_dumped == false` invariant, `/proc` absence must not be the sole reap
oracle, and the descriptor equality must not be relaxed to `after <= before`. A correction that
would need a product change **stops** and returns `OWNER DECISION REQUIRED`.

#### 4. The historical runs are permanent

Runs `35499943908` and `35499943903` both remain **attempt 1, FAILURE**. Neither is rerun, retried,
cancelled, restarted or replaced by a dispatch, no published history is amended and nothing is force
pushed. **NO RETRY. NO RERUN. NO REPLACEMENT.** A corrected head receives a new SHA and new natural
run identifiers, which will be new evidence rather than a revision of this one.

#### 5. Boundary of this disposition

This disposition is recorded in documentation only. It changes no product code, test, workflow,
Cargo file, ADR contract, experiment or evidence, does not touch `main`, promotes no traceability
row and accepts no P4 gate.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, PUBLISHED ONCE AND ITS HOSTED
VALIDATION FAILED. P4 IS NOT ACCEPTED. P5 IS NOT AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1 MODULE IS
NOT PRODUCT-ACCEPTED.**

**Next gate: BOUNDED P4 TEST/EVIDENCE CORRECTION, then ONE BOUNDED INDEPENDENT RE-REVIEW OF
`P4PUB-01`, `P4PUB-02` AND `P4PUB-03`.**

<a id="helm-launch-p4-unpublished-attribution-cleanup"></a>

### Owner disposition 2026-09-20 — **HELM-LAUNCH P4**: `R-P4PUB-I1` closed by a metadata-only rewrite of the unpublished chain

**`R-P4PUB-I1` is CLOSED / METADATA-ONLY BEFORE PUBLICATION.** The bounded independent re-review of
the P4 first-publication corrections returned one IMPORTANT finding that was neither a product, a
test nor an evidence defect: the two unpublished correction commits carried an AI-agent
`Co-Authored-By` trailer, which [commit authorship](../AGENTS.md) forbids and which states that the
repository rule is stronger than a tool's default instruction. Because both commits were still
**unpublished**, the repository owner, Djomla83, required the trailers to be removed **before**
publication rather than becoming permanent. This disposition records that rewrite and proves, by
exact tree identity, that nothing else changed.

| Item | Value |
|---|---|
| Finding | `R-P4PUB-I1` — **CLOSED / METADATA-ONLY BEFORE PUBLICATION** |
| Class | repository policy / commit metadata — **not** a product, test or evidence defect |
| Published base | `94ee48da0cc412f0d8043e34b914c198615b923e` — **NOT REWRITTEN** |
| Published history | **NOT REWRITTEN, NOT FORCE PUSHED, NOTHING PUSHED** |
| `main` | **UNCHANGED** |
| ADR-0024 product contract | **UNCHANGED** |
| P4 product contract | **UNCHANGED** |
| P4 hosted Linux runtime | **PENDING NEW PUBLICATION CI** |
| HELM-LAUNCH P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

#### 1. Commit mapping

Only the unpublished descendants of the published base were rewritten. Each commit was rebuilt
against its original tree object, so the mapping is provable rather than asserted.

| Role | Old SHA | New attribution-clean SHA | Tree SHA (both) | Tree |
|---|---|---|---|---|
| Owner disposition | `cfe2537ba638c536d2edd2a9660998713a443b89` | `59f15c3d59c9e4c496bcaca1065076be88ea5301` | `5bb327622ad2b7f64e6e6f45c7ebb2d1ab616d03` | **IDENTICAL** |
| Test/evidence correction | `aed3fd2564f47d2b0a999b553133d057b68783e1` | `eb673f5728e4e721427d3c0c73892e4990607f8a` | `52b8a86a58c63d05858838db0dd9ca03ec2bbf64` | **IDENTICAL** |
| Independent re-review | `5aca7d593c9152b0252f2ed38be782fade0b4dc3` | `3ce7eb31fcbb4471245ee10b7763d465184f7ada` | `aa18dff18912576cffe3b998155c6bd451cd2a33` | **IDENTICAL** |

The commit SHAs necessarily changed, because a commit object commits to its message and its parent.
The **tree** SHAs did not change at all, which is the load-bearing fact.

#### 2. What changed, and what did not

Exactly two commit messages lost exactly one trailer line each, together with the blank line that
separated it from the body. Nothing else in either message was touched: subject, body, findings,
blockers, run identifiers and authority statements are byte-identical to what was reviewed. The
independent re-review commit's message was replayed **byte-identical**; it carried no trailer and
needed no change. Author and committer identity and both timestamps were preserved exactly on all
three commits, so the only metadata deltas are the removed trailers and the new parent identities.

* **No product change.** No file under `crates/` differs from the reviewed state.
* **No test change.** The corrected oracles are exactly the reviewed oracles.
* **No documentation-content change** inside the three replayed commits.
* **No review artifact change.** `HELM-LAUNCH-P4-FIRST-PUBLICATION-CORRECTION-REREVIEW.md` is
  byte-identical, and **no review finding was altered, softened or removed.**
* **No published history rewritten.** `94ee48da0cc412f0d8043e34b914c198615b923e` and every ancestor
  are untouched and the base is still an ancestor of `HEAD`.

The re-review commit's body still quotes the `Co-Authored-By` trailer inside its statement of
`R-P4PUB-I1`. That occurrence is the **finding itself**, not an attribution, and it was deliberately
preserved: altering it would have altered a review finding.

#### 3. Why no new technical re-review is required

The independent re-review reviewed the **content** of the disposition and the correction, which is
exactly what a tree object names. Since the old and new tree SHAs are identical for all three
commits, the reviewed content is bit-for-bit the content now on the branch, and the re-review
carries forward unchanged. Had any tree differed, the required response was to stop and return
`TECHNICAL CONTENT CHANGED — NEW INDEPENDENT REREVIEW REQUIRED`. No tree differed. This disposition
is the provenance bridge between the old and new SHAs; the re-review was **not** reopened merely
because commit identities changed.

#### 4. Attribution state of the unpublished range

No commit in `94ee48da0cc412f0d8043e34b914c198615b923e..HEAD` carries a `Co-Authored-By` trailer, a
"Generated with" signature, an agent signature in the body or any other AI-agent attribution.
`git interpret-trailers --parse` returns an empty trailer set for every commit in the range, and the
author and committer of every commit is the human owner who approves the change and takes
responsibility for it. No commit outside this unpublished range was inspected for modification or
modified.

#### 5. Boundary of this disposition

This disposition is recorded in documentation only. It changes no product code, test, workflow,
Cargo file, ADR contract, experiment or evidence, does not touch `main`, promotes no traceability
row and accepts no P4 gate. Nothing was pushed and no historical CI run was rerun: runs
`35499943908` and `35499943903` both remain **attempt 1, FAILURE**. **NO RETRY. NO RERUN. NO
REPLACEMENT.**

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, CORRECTED, INDEPENDENTLY RE-REVIEWED AND
ATTRIBUTION-CLEAN, AND ITS LINUX RUNTIME REMAINS UNVALIDATED. P4 IS NOT ACCEPTED. P5 IS NOT
AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1 MODULE IS NOT PRODUCT-ACCEPTED.**

**Next gate: ONE FAST-FORWARD PUBLICATION OF THE ATTRIBUTION-CLEAN REVIEWED CHAIN, then NEW NATURAL
HOSTED P4 CI.**

<a id="helm-launch-p4-second-publication-failure"></a>

### Owner disposition 2026-09-20 — **HELM-LAUNCH P4 SECOND PUBLICATION**: hosted validation FAILED on a stale P3-era doctest, `P4PUB-01/02/03` verified fixed

**`HELM_LAUNCH_P4_SECOND_PUBLICATION_HOSTED_VALIDATION_FAILED`.** The repository owner, Djomla83,
records that the attribution-clean corrected P4 head was published once and its hosted Linux
validation failed. The two natural runs are **permanent historical evidence** and are preserved
exactly as they stand. **P4 hosted validation is NOT ACCEPTED.** A narrowly bounded
**documentation / doctest** correction is authorised inside `crates/helm-launch/src/authority.rs`
only; the P4 product contract is not reopened.

| Item | Value |
|---|---|
| Published P4 head | `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1` |
| `helm-launch` — first natural run | `35507479482`, run #7, attempt 1, event `push` — **FAILURE** |
| its Linux job | `106069632303` (`ubuntu-24.04`) — **FAILURE** at step 8, `cargo test -p helm-launch --locked` |
| its Windows job | `106069632268` (`windows-2025`) — **SUCCESS** |
| its macOS job | `106069632184` (`macos-15`) — **SUCCESS** |
| `HELM Rust workspace Linux` — first natural run | `35507479458`, run #39, attempt 1, event `push` — **FAILURE** |
| its Linux job | `106069631924` (`ubuntu-24.04`) — **FAILURE** at step 6, `cargo test --workspace --locked` |
| P4 hosted validation | **NOT ACCEPTED** |
| P4 product mechanism | **NOT IMPLICATED** by any preserved evidence |
| Historical runs | **PRESERVED** — **NO RETRY, NO RERUN, NO REPLACEMENT** |
| ADR-0024 product contract | **UNCHANGED** by this disposition |
| P4 product contract | **UNCHANGED** by this disposition |
| `unsafe` boundary | **UNCHANGED** — `crates/helm-launch/src/backend/` only |
| HELM-LAUNCH P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

#### 1. The three dispositioned first-publication failures are closed on real Linux

All three corrections authorised earlier on 2026-09-20 executed on real Linux and **passed**, each
on **two independent `ubuntu-24.04` runners** — the `helm-launch` Linux job and the workspace
`verify` job:

| Id | Case | Hosted result |
|---|---|---|
| `P4PUB-01` | `launch::tests::a_signalled_child_keeps_its_signal_number_and_its_core_flag` | **HOSTED VERIFIED FIXED** |
| `P4PUB-02` | `backend::tests::a_disarmed_drop_guard_neither_signals_nor_waits` | **HOSTED VERIFIED FIXED** |
| `P4PUB-03` | `launch::tests::the_outcome_owns_no_descriptor_and_consumes_its_authorisation` | **HOSTED VERIFIED FIXED** |

`P4PUB-01` passed without invoking its named `TEST ENVIRONMENT PRECONDITION` path, so the positive
`CLD_DUMPED` producer gate was satisfied on both runners. `P4PUB-03`'s inner case reported
`ignored, driven by the_outcome_owns_no_descriptor_and_consumes_its_authorisation, in a dedicated
process`, exactly as designed. The `helm-launch` Linux library target reported **107 passed, 0
failed, 2 ignored**, and six boundary suites then passed (28, 20, 17, 15, 19, 6). In the workspace
run every crate suite passed before the failure.

#### 2. `P4PUB-04`, the one new failure

| Id | Severity | Class | Reachable | Product mechanism |
|---|---|---|---|---|
| `P4PUB-04` | **IMPORTANT** | **TEST / EVIDENCE DOCUMENTATION DEFECT** | **YES** | **NOT IMPLICATED** |

Both runs failed identically, in the **doctest** target and in no lifecycle, sweep, stream or
receipt case:

```text
test crates/helm-launch/src/authority.rs - authority::AuthorizedLaunch (line 500) - compile fail ... FAILED
test crates/helm-launch/src/authority.rs - authority::AuthorizedLaunch (line 506) - compile fail ... FAILED
Test compiled successfully, but it is marked `compile_fail`.
test result: FAILED. 40 passed; 2 failed; 0 ignored
error: doctest failed, to rerun pass `-p helm-launch --doc`
```

Those two P3-era doctests assert that `helm_launch::launch(..)` cannot be named and that
`helm_launch::LaunchOutcome` does not exist, under a prose claim that nothing in the crate can
consume an `AuthorizedLaunch` to create a process because no `launch` function exists. Accepted P4
made both public on the cohort — `#[cfg(all(target_os = "linux", target_arch = "x86_64"))] pub use
launch::{LaunchOutcome, launch};` — so on Linux the snippets now compile and their stale
`compile_fail` expectation correctly reports FAILED. **The product is correct and the doctest is
wrong.**

#### 3. Why it was not seen before

The defect is **pre-existing and newly exposed**, not a regression of the authorised correction.
`authority.rs` was not touched by any published commit of this chain and was last modified in
`afe8922`, before P4; the contradiction dates from `3d152ad`, the P4 implementation commit, which
added the public export without retiring the P3-era negative proofs. It was unobservable until now
because `authority.rs` is compiled only on the cohort, so off-cohort and on the Windows development
host the snippets correctly fail to compile and pass, while on Linux — the only place the defect is
observable — the first publication's library target failed first, so `cargo` never reached the
doctest target. The log of run `35499943908` contains no doctest line at all. This publication is
the first time the Linux doctest target has ever executed on a P4 head.

#### 4. Later load-bearing gates are INCOMPLETE, not passed

`cargo test` stops at the first failing target, so every later step was skipped and **no skipped
gate may be treated as success**: the fault-injection suite and with it `F-P4-05` positive parent
group authority and the foreign-reaper `ECHILD` case; the three named P4 gate steps; machine-code
closure in both profiles; the P3 regression steps; the repository-level confinement checks; and, in
the workspace run, the Python tool tests, `validate_docs.py` and the release builds. Windows and
macOS remained **green**, including the off-cohort absence proofs, but no Linux runtime claim
derives from them.

#### 5. The authorised correction is bounded to documentation

The correction is confined to `crates/helm-launch/src/authority.rs` and to doc comments, doctests,
ordinary comments and lint-reason strings. It **must not** change the public API, a signature, a
body, a type or field layout, visibility, a `cfg` boundary, a trait implementation, runtime
behaviour, `unsafe` or any dependency, and it must not touch `launch.rs`, `lifecycle.rs`,
`receipt.rs`, `model.rs`, the backend or any workflow. Still-valid authority-boundary negative
proofs must be preserved rather than deleted to obtain green, no off-cohort absence claim may be
encoded in this cohort-only module, and no new wording may claim exec success: a clean status EOF
remains `Indeterminate(StatusEofWithoutRecord)`. Adding `--no-fail-fast` is **not** authorised here.
A correction that would need an executable change **stops** and returns `OWNER DECISION REQUIRED`.

#### 6. Both publication phases are permanent

Phase 1 at `94ee48da0cc412f0d8043e34b914c198615b923e` — runs `35499943908` and `35499943903`, both
attempt 1, **FAILURE** — and Phase 2 at `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1` — runs
`35507479482` and `35507479458`, both attempt 1, **FAILURE** — are immutable. Neither is rerun,
retried, cancelled, restarted or replaced by a dispatch, no published history is amended and nothing
is force pushed. **NO RETRY. NO RERUN. NO REPLACEMENT.** A corrected head receives a new SHA and new
natural run identifiers, which will be new evidence rather than a revision of either phase.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, PUBLISHED TWICE AND ITS HOSTED
VALIDATION HAS NOT PASSED. P4 IS NOT ACCEPTED. P5 IS NOT AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1
MODULE IS NOT PRODUCT-ACCEPTED.**

**Next gate: BOUNDED `P4PUB-04` DOCUMENTATION CORRECTION, then ONE BOUNDED INDEPENDENT RE-REVIEW OF
`P4PUB-04`.**

<a id="helm-launch-p4-third-publication-failure"></a>

### Owner disposition 2026-09-20 — **HELM-LAUNCH P4 THIRD PUBLICATION**: hosted validation FAILED on a stale P3 release-library CI oracle, `P4PUB-01` through `P4PUB-04` verified fixed

**`HELM_LAUNCH_P4_THIRD_PUBLICATION_HOSTED_VALIDATION_FAILED`.** The repository owner, Djomla83,
records that the corrected P4 head was published a third time and that its hosted `helm-launch`
Linux validation failed, while the workspace run **passed**. Both natural runs are **permanent
historical evidence** and are preserved exactly as they stand. **P4 hosted validation is NOT
ACCEPTED.** A narrowly bounded **workflow / CI evidence** correction is authorised inside
`.github/workflows/helm-launch.yml` only; the P4 product contract is not reopened and no crate
source, manifest, test or tool is touched.

| Item | Value |
|---|---|
| Published P4 head | `6f9c73dfdde2e2321722519baa8fdf2764ca825f` |
| `helm-launch` — natural run | `35511973984`, attempt 1 — **FAILURE** at step 17, `Confirm a release library instantiates no backend` |
| `HELM Rust workspace Linux` — natural run | `35511973812`, attempt 1 — **SUCCESS** |
| `P4PUB-01` | **HOSTED VERIFIED FIXED** |
| `P4PUB-02` | **HOSTED VERIFIED FIXED** |
| `P4PUB-03` | **HOSTED VERIFIED FIXED** |
| `P4PUB-04` | **HOSTED VERIFIED FIXED** |
| `P4PUB-05` | **IMPORTANT — CI / WORKFLOW EVIDENCE DEFECT**, reachable, product mechanism **NOT IMPLICATED** |
| P4 hosted validation | **NOT ACCEPTED** |
| P4 product mechanism | **NOT IMPLICATED** by any preserved evidence |
| Historical runs | **PRESERVED** — **NO RETRY, NO RERUN, NO REPLACEMENT** |
| ADR-0024 product contract | **UNCHANGED** by this disposition |
| P4 product contract | **UNCHANGED** by this disposition |
| `unsafe` boundary | **UNCHANGED** — `crates/helm-launch/src/backend/` only |
| HELM-LAUNCH P5 | **NOT AUTHORISED** |
| Trial #3 | frozen **`MECHANISM_REJECTED`**, unchanged, must not be rerun |
| Trial #4 | **NOT AUTHORISED** |

#### 1. All four dispositioned defects are closed on real Linux

Every correction authorised earlier in this chain executed on hosted Linux and **passed**:

| Id | Class | Hosted result |
|---|---|---|
| `P4PUB-01` | `launch::tests::a_signalled_child_keeps_its_signal_number_and_its_core_flag` | **HOSTED VERIFIED FIXED** |
| `P4PUB-02` | `backend::tests::a_disarmed_drop_guard_neither_signals_nor_waits` | **HOSTED VERIFIED FIXED** |
| `P4PUB-03` | `launch::tests::the_outcome_owns_no_descriptor_and_consumes_its_authorisation` | **HOSTED VERIFIED FIXED** |
| `P4PUB-04` | the two stale P3-era `compile_fail` doctests on `AuthorizedLaunch` | **HOSTED VERIFIED FIXED** |

`P4PUB-04` is the decisive new fact: the Linux **doctest** target, which had never completed on a P4
head before, executed and passed. The default Linux library, integration and doctest targets, the
fault-injection suite, the three named P4 gate steps, the pure lifecycle model, machine-code closure
in **both** profiles and the fault-injection positive control with its release absence proof all
passed before the failing step was reached. Windows and macOS were **SUCCESS**, and the whole
workspace run `35511973812` was **SUCCESS**.

#### 2. `P4PUB-05` — IMPORTANT, CI / workflow evidence defect

Step 17 of the `helm-launch` workflow still encodes a **P3** invariant in both its prose and its
oracle: *"a release library contains no backend at all, because P3 adds no public consumer for it"*.
It builds a real release `x86_64-unknown-linux-gnu` library with `--emit=asm` and **fails when it
finds** `backend5child10child_main`.

That invariant was true in P3 and is **false in P4**. Accepted P4 exports
`#[cfg(all(target_os = "linux", target_arch = "x86_64"))] pub use launch::{LaunchOutcome, launch};`,
and the accepted call path is `launch` -> `backend::spawn_for_lifecycle` -> `spawn::spawn` ->
`child::child_main`. None of that path is behind `cfg(test)` or behind the `test-fault-injection`
feature, which is additionally gated on `debug_assertions` and is therefore compiled out of a
release build even under `--all-features`. A P4 production release library is consequently
**expected to instantiate the private backend**, and the old negative oracle is **superseded**: it
now fails on exactly the evidence that proves P4 works.

The defect is therefore in the **evidence**, not in the mechanism. The product is correct.

#### 3. The product is not the defect and is not changed

No public API is withdrawn, no `LaunchOutcome` is hidden, the release `launch` is not made inert,
the backend is not eliminated by dead-code elimination, no visibility, `cfg` boundary, `child_main`,
lifecycle, receipt, `unsafe` boundary or P3 child contract is altered. The backend remains
**private** while being **reachable internally** from the public P4 `launch`. That is the accepted
architecture, and the authorised correction is confined to the workflow file.

#### 4. Step 24 — substitute Python evidence, accurately scoped

`helm-launch` step 24, `Repository-level confinement checks, independent of cargo`, was **SKIPPED**
because step 17 failed. It is **not** separately defective and receives **no** correction.

The workspace run `35511973812` nevertheless completed `python3 -m unittest discover -s tools/tests
-v` and its log shows actual **PASS** execution of
`test_helm_launch_confinement.UnsafeConfinementTests` and of `test_helm_launch_machine_proofs.*`.
Repository-level confinement and machine-proof evidence is therefore **not wholly absent** in this
phase. This substitute evidence **does not** convert the failed `helm-launch` run into a success:
the next natural `helm-launch` run must still proceed through and pass its **own** explicit step 24.

#### 5. Skipped is not passed

Everything after step 17 was skipped and no skipped gate may be treated as success: the named Linux
x86_64 capability-admission cases, the P3 backend and traced-window cases, the P3 fault-injection
cases, the unsafe-confinement and boundary suites, step 24, and the deterministic receipt and plan
identities. Windows and macOS remained green, including the off-cohort absence proofs, but no Linux
runtime claim derives from them.

#### 6. Bounded correction authority

Correction authority is limited to `.github/workflows/helm-launch.yml` plus this documentation
disposition. The stale P3 negative release-library oracle is **replaced**, not deleted, by a
P4-correct **positive** release-library reachability gate that keeps the same real release build and
requires the backend marker to be **present**. The marker is a **CI evidence marker under the
pinned Rust 1.95.0 toolchain**; it is neither public API nor a product contract, and P5 may harden
or replace it. The stale neighbouring comment above `Child machine-code closure — release codegen`
is corrected in the same commit. The distinct machine-code closure gates and the fault-injection
positive control with its release absence proof are **preserved unchanged**, step order is
**unchanged**, no `continue-on-error` is introduced and `set -euo pipefail` is not weakened. Adding
`--no-fail-fast` is **not** authorised here and remains potential P5 CI-hardening work. `P4DOC-01`
remains **MINOR / NONBLOCKING** and is **carried**, not fixed, in this task.

#### 7. All three publication phases are permanent

Phase 1 at `94ee48da0cc412f0d8043e34b914c198615b923e` — runs `35499943908` and `35499943903`, both
attempt 1, **FAILURE** — Phase 2 at `84b49ab9d7bf4f7c9d10c7e55f327778f2855da1` — runs `35507479482`
and `35507479458`, both attempt 1, **FAILURE** — and Phase 3 at
`6f9c73dfdde2e2321722519baa8fdf2764ca825f` — run `35511973984`, attempt 1, **FAILURE**, and run
`35511973812`, attempt 1, **SUCCESS** — are immutable. None is rerun, retried, cancelled, restarted
or replaced by a dispatch, no published history is amended and nothing is force pushed. **NO RETRY.
NO RERUN. NO REPLACEMENT.** A corrected head receives a new SHA and new natural run identifiers,
which will be new evidence rather than a revision of any phase.

**TRIAL #3 FROZEN RESULT REMAINS `MECHANISM_REJECTED`. TRIAL #3 MUST NOT BE RERUN.**

**NO TRIAL #4 IS AUTHORISED.**

**HELM-LAUNCH P1, P2 AND P3 ARE ACCEPTED. P4 IS AUTHORISED, PUBLISHED THREE TIMES AND ITS HOSTED
VALIDATION HAS NOT PASSED. P4 IS NOT ACCEPTED. P5 IS NOT AUTHORISED. THE COMPLETE HELM-LAUNCH 0.1
MODULE IS NOT PRODUCT-ACCEPTED.**

**Next gate: BOUNDED `P4PUB-05` WORKFLOW CORRECTION, then ONE BOUNDED INDEPENDENT RE-REVIEW OF
`P4PUB-05`.**
