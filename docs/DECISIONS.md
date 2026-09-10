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
decision.

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
| ADR-0024 | [Execute one explicitly authorized object without granting authority from comparison](adr/ADR-0024-launch-authority.md) — [design report and falsification plan](research/HELM-LAUNCH-ARCHITECTURE.md), [LAUNCH-EXEC-01 preregistered definition](experiments/LAUNCH-EXEC-01-DEFINITION.md), **NOT_RUN**. A single-crate, Linux x86_64, capability-driven launcher for exactly one already-open regular ELF object, with no satisfaction, compatibility, readiness or success verdict. Parsing a LaunchPlan and holding a BindingReport both grant **zero** execution authority; `NoClaimContradicted` is never permission. Direct-child lifecycle only, **no process-tree containment**, and **not a sandbox** — the child runs with the caller's own credentials. Depends on no HELM crate; context travels as opaque digests. Requires a scoped `unsafe` backend or a weaker descriptor claim (owner decision D-1), because the workspace `forbid(unsafe_code)` cannot be locally relaxed and rustix provides no `close_range`. **Narrowed on 2026-09-09 by the [three-workstream pre-execution review](implementation/HELM-LAUNCH-PRE-EXECUTION-REVIEW.md)**, sixteen BLOCKERs among 53 findings, classification NEEDS_ARCHITECTURE_OWNER_REVIEW: the executable digest is a pre-execution measurement of the main file body (`pre_exec_body_sha256`) and never the identity of the body that ran, `ETXTBSY` does not cover the measure-to-exec window, the child runs with the caller's credentials **except** for a set-user-ID or capability-bearing object, clean EOF does not prove exec, direct-child lifecycle does not imply direct-child liveness, and the receipt becomes a product of one process disposition and one per-stream completeness. LAUNCH-EXEC-01 re-frozen at **71 cases** with a total aggregate precedence. **Owner decisions of 2026-09-09**: D-1 **arm (i)** (scoped unsafe backend; FD isolation not weakened to keep crate-wide `forbid`), D-2 to D-6 and D-8 accepted (D-4 as corrected), **D-9** refuse `S_ISUID`/`S_ISGID` at admission, **D-10** environment **exactly empty** with `explicit` removed from 0.1, and new **D-11** `PR_SET_NO_NEW_PRIVS` before exec because D-9 does not cover file capabilities. Definition re-frozen at **72 cases** (56 mandatory / 9 conditional / 7 recorded) against a machine-readable manifest, with the disposable [experiment sources](experiments/launch-exec-01/) committed and hashed. **Pre-trial implementation only: D-7 is NOT granted, LAUNCH-EXEC-01 remains NOT_RUN, and `crates/helm-launch` must not be created before it has run and been reviewed** | **Proposed 2026-09-09** |


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
