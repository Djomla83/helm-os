# ADR-0022: Observe explicit targets without interpreting desired state

**Status:** **Accepted — bounded 0.1 scope, implementation not yet performed**\
**Draft date:** 2026-09-08\
**Approver:** Repository owner (Djomla83), by explicit written instruction of 2026-09-08\
**Acceptance date:** 2026-09-08\
**Evidence basis:** [OBS-FS-01 PASS](../experiments/OBS-FS-01-EXECUTION-REPORT.md)

## Context

Authoritative main is `e69abdfc55646dff6073befaeb843a18d6c7975f`.
The accepted experimental helm-app-spec 0.1 describes desired state only;
helm-evidence 0.1 checks declared evidence-bundle completeness/validity only.
[ADR-0021](ADR-0021-second-product-module.md) does not authorise observation or
execution. Historical suggestions combining observation and comparison were
advisory, not accepted architecture.

The next authority must answer what was actually observed. Desired archive hashes
do not identify installed Wine loader bytes, a prefix path does not prove dedication,
and read-only filesystem access still has security and consistency consequences.

## Options

A: directly consume ValidatedAppSpec and interpret requirements into observations.
B: receive an explicit bounded observation plan and caller-granted root capabilities.
C: scan a prefix/system/runtime into a general inventory.

The [design report](../research/HELM-OBSERVE-ARCHITECTURE.md#2-three-designs-and-recommendation)
scores all three on separation, authority, testability, security, determinism,
coupling, usefulness and premature abstraction. B is recommended for its explicit
authority boundary. A imports desired semantics and build coupling; C lacks a
requirement justifying broad discovery/privacy exposure. No fourth design is needed.

## Accepted decision, 2026-09-08

> Build `helm-observe` 0.1 as an explicit-target, Linux-only observation library that
> reports actual facts only and does not interpret desired state or execute applications.

Acceptance is **bounded**. It settles the architectural boundary and the acquisition
mechanism; it does not stabilise the schema or API, and **it does not authorise
implementation**, which needs a separate owner instruction.

**Accepted authority and mechanism boundary.** An explicit bounded `ObservationPlan`;
no ambient scanning; no recursive inventory; the caller supplies already-open root
capabilities; validation alone grants no authority; target resolution uses the reviewed
constrained `openat2`/`O_PATH` model; the pinned object is classified before any
regular-file data access; symlinks and special files are rejected under the amended
semantics; permitted regular-file content is read through an admission-validated trusted
current-process procfs descriptor capability; no pathname fallback; no desired/observed
comparison; no execution; and no atomic-file or environment-snapshot guarantee.

**Amended symlink semantics, as measured.** A trailing symlink may be pinned as an
`O_PATH` descriptor and is rejected at classification; a non-final symlink is rejected at
resolution in the `ELOOP` class and yields no descriptor. Post-open handle classification
is therefore mandatory, not defensive. See
[Amendment 1](../implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md#amendment-1).

**Accepted initial evidence cohort.** Linux x86_64, local ext4, anchored by the tested
mechanism and kernel cohort of OBS-FS-01. This experimental evidence is **not** generalised
into arbitrary Linux filesystem support; other filesystems, architectures, kernels and
mount topologies need their own evidence.

<a id="bind-mount-exclusion"></a>

### Excluded from the validated 0.1 cohort: descendant bind mounts

Target paths whose security or correctness claim depends specifically on rejecting a
**bind mount created as a descendant of an authorized ext4 root** are **outside** the
validated and supported 0.1 cohort.

`RESOLVE_NO_XDEV` remains mandatory, and observed mount crossings remain conservatively
rejectable. But HELM 0.1 **must not claim empirical validation of the descendant
bind-mount case**: OBS-FS-01 recorded it BLOCKED, because Ubuntu's AppArmor restriction on
unprivileged user namespaces prevented constructing the fixture, and it was not bypassed.
General mount-traversal rejection *is* evidenced, by the mandatory non-namespace
`NO_XDEV` arm and its control. Any future support or claim that depends on the descendant
bind-mount case requires new evidence. **The experiment did not prove arbitrary mount
topology safety.**

<a id="implementation-obligations"></a>

### Mandatory implementation obligations before 0.1 may be owner-merged

These are blockers, not advice. Each needs an explicit adversarial test.

**A. Exact plan identity binding.** The authorised observation authority must bind the
complete exact validated plan SHA-256, compared over all 32 bytes. Object or pointer
identity is not sufficient.

**B. Root set binding.** Authorisation must bind the logical root IDs and the exact root
set and count **by ID**, never by positional coincidence.

**C. Plan substitution prevention.** After authorisation it must be impossible by
construction, or else reliably rejected, to execute a different plan. An API equivalent to
`authorize(plan, capabilities) -> AuthorizedScope` followed by `observe(&AuthorizedScope)`
is preferred where it cleanly removes the re-supplied-plan substitution surface. This ADR
specifies the **invariant**, not that exact Rust signature.

Tests must explicitly attack: authorising plan A and then attempting execution with plan
B; and root addition, removal and substitution.

## Original proposed decision, preserved

Create no product code in this task. Recommend, for separate later authorisation,
one Linux-only library called helm-observe that:

1. Takes an inert, bounded ObservationPlan inside that crate: exact subject app-spec
   digest as context, logical roots and explicit target IDs/relative paths/observable
   operations. It takes no expected artifact hashes or app-spec requirement types.
2. Separates untrusted plan validation from the trusted caller's explicit grant of
   that exact plan and already-open root descriptors. No ambient discovery or recursion.
3. Observes only directory metadata and regular-file byte streams in initial 0.1.
   Effective DLL policy, runtime-family/architecture interpretation, package metadata
   parsing, dedication and archive-to-installation provenance remain unestablished.
4. Pins objects before reading, rejects target symlinks, mount crossings and special
   files, and hashes one retained read handle. The report's Linux O_PATH/native
   procfs-FD capability route requires a separate falsification experiment; no
   pathname-reopen or weakened fallback is accepted merely to make it work.
5. Reports sequential per-object observations, valid absence, rejection, unavailable/
   failed attempts and explicit budget omissions. It promises no atomic environment
   snapshot or stable file version under undetected concurrent mutation.
6. Returns immutable observation bytes with exact-byte identity, bound to exact
   plan and subject identities. No canonicalization, signatures or identity timestamps.
   External capture metadata supplies provenance/time, without becoming attestation.
7. Depends on neither helm-app-spec nor helm-evidence. Identity artifacts connect
   them; future pure comparison/binding and execution stay outside the observer.

The report is the detailed proposal for API, authority, output semantics, full
observability matrix, limits, dependency diagrams, A0 mapping and OBS-FS-01. These
details remain reviewable proposals. No existing accepted ADR is superseded or
other ADR accepted by this document.

## Consequences and evidence

The approach supplies useful actual file identities while leaving unsupported
semantics explicit. It cannot make a future launch safe by itself. A caller needs
separate root/installation binding, trust and coherent-state evidence where required.
Reading may affect atime/cache/I/O and can block in the OS despite finite byte limits.
Hardlinks do not establish provenance or disposable ownership.

The report traces preserved A0 source/archive/installed-file identities, unchanged
experimental FAIL, G0-3b post-copy document loss, the independent evidence review's
symlink/FIFO races, and primary Linux/Wine/pinned dependency sources. It also retains
the unresolved shared-Cargo-graph sha2/force-soft performance limitation. No new
benchmark, application run, observer test or security certification occurred.

## Retained semantic limits

Acceptance implies **none** of the following, and no observer output upgrades them:
installed package provenance; Wine archive-to-loader binding; dedicated-prefix proof;
effective DLL-policy interpretation; desired-state satisfaction; general compatibility;
safe launch; atomic file or environment snapshots; network or FUSE filesystem safety;
arbitrary Linux or kernel safety; Windows observer semantics.

`helm-observe` remains **actual-facts-only**. A future `helm-bind` owns desired-versus-
observed comparison. A future `helm-launch` owns execution. Neither is accepted here.

## Falsification and approval boundary

Apply the report's [falsifiers](../research/HELM-OBSERVE-ARCHITECTURE.md#17-falsifiers-and-owner-decision-boundary)
and [proposed experiment](../research/HELM-OBSERVE-ARCHITECTURE.md#14-proposed-pre-implementation-falsification-experiment).
Reject conflated desired/actual semantics, false provenance/dedication, implicit
scanning/execution, special-file data access, misleading consistency, identity
cycles, uncontrolled coupling or 7-Zip-specific core behavior.

Owner review and independent reference-expectation review precede separate experiment
authorisation. Passing that experiment would still require a separate implementation
instruction. This Proposed ADR accepts no licence, production release, privileges,
receipt infrastructure, binder, launcher, recovery or larger HELM subsystem.
