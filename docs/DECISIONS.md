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
[decision of 2026-09-18](#helm-launch-p2-authorised).

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
| ADR-0024 | [Execute one explicitly authorized object without granting authority from comparison](adr/ADR-0024-launch-authority.md) — [design report and falsification plan](research/HELM-LAUNCH-ARCHITECTURE.md), [LAUNCH-EXEC-01 preregistered definition](experiments/LAUNCH-EXEC-01-DEFINITION.md), **NOT_RUN**. A single-crate, Linux x86_64, capability-driven launcher for exactly one already-open regular ELF object, with no satisfaction, compatibility, readiness or success verdict. Parsing a LaunchPlan and holding a BindingReport both grant **zero** execution authority; `NoClaimContradicted` is never permission. Direct-child lifecycle only, **no process-tree containment**, and **not a sandbox** — the child runs with the caller's own credentials. Depends on no HELM crate; context travels as opaque digests. Requires a scoped `unsafe` backend or a weaker descriptor claim (owner decision D-1), because the workspace `forbid(unsafe_code)` cannot be locally relaxed and rustix provides no `close_range`. **Narrowed on 2026-09-09 by the [three-workstream pre-execution review](implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md)**, sixteen BLOCKERs among 53 findings, classification NEEDS_ARCHITECTURE_OWNER_REVIEW: the executable digest is a pre-execution measurement of the main file body (`pre_exec_body_sha256`) and never the identity of the body that ran, `ETXTBSY` does not cover the measure-to-exec window, the child runs with the caller's credentials **except** for a set-user-ID or capability-bearing object, clean EOF does not prove exec, direct-child lifecycle does not imply direct-child liveness, and the receipt becomes a product of one process disposition and one per-stream completeness. LAUNCH-EXEC-01 re-frozen at **71 cases** with a total aggregate precedence. **Owner decisions of 2026-09-09**: D-1 **arm (i)** (scoped unsafe backend; FD isolation not weakened to keep crate-wide `forbid`), D-2 to D-6 and D-8 accepted (D-4 as corrected), **D-9** refuse `S_ISUID`/`S_ISGID` at admission, **D-10** environment **exactly empty** with `explicit` removed from 0.1, and new **D-11** `PR_SET_NO_NEW_PRIVS` before exec because D-9 does not cover file capabilities. Definition re-frozen at **72 cases** (56 mandatory / 9 conditional / 7 recorded) against a machine-readable manifest, with the disposable [experiment sources](experiments/launch-exec-01/) committed and hashed. **Pre-trial implementation only: D-7 is NOT granted, LAUNCH-EXEC-01 remains NOT_RUN, and `crates/helm-launch` must not be created before it has run and been reviewed**. The text of this row up to here records the state of 2026-09-09. **[Owner decision 2026-09-17](#adr-0024-accepted-helm-launch-p1-authorised): the revised ADR-0024 is Accepted** for the helm-launch 0.1 architecture only; Trial #3 remains `MECHANISM_REJECTED`, no Trial #4; **HELM-LAUNCH P1 only** is authorised, P2+ is not | **Proposed 2026-09-09; revised 2026-09-16; Accepted 2026-09-17** |


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
