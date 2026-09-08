# Stanje projekta

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
