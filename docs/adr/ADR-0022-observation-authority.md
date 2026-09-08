# ADR-0022: Observe explicit targets without interpreting desired state

**Status:** Proposed\
**Draft date:** 2026-09-08\
**Approver:** Not assigned\
**Acceptance date:** Not accepted

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

## Proposed decision

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
