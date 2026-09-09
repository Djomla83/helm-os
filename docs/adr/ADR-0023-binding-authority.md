# ADR-0023: Compare desired claims with actual observations without producing a satisfaction verdict

**Status:** **Accepted — bounded 0.1 architecture scope, implementation not authorised**\
**Draft date:** 2026-09-09\
**Approver:** Repository owner (Djomla83), by explicit written instruction of 2026-09-09\
**Acceptance date:** 2026-09-09\
**Authoritative base:** `9b09e7d3f6d216fb0258d23963d70bb889793567`\
**Design basis:** [helm-bind architecture report](../research/HELM-BIND-ARCHITECTURE.md)

## Context

Three experimental product modules are owner-merged on main.
[helm-app-spec](../../crates/helm-app-spec/README.md) validates **desired** claims and
establishes nothing observed. [helm-observe](../../crates/helm-observe/README.md), under
Accepted [ADR-0022](ADR-0022-observation-authority.md), reports **actual** facts at explicitly
authorised targets and interprets no desired state.
[helm-evidence](../../crates/helm-evidence/README.md) checks declared evidence-bundle
completeness and validity. Nothing compares desired with observed, and both accepted ADRs say
so deliberately: ADR-0022 records that "a future `helm-bind` owns desired-versus-observed
comparison".

The comparison is where a project of this kind usually starts lying. The observer already
refuses satisfaction vocabulary; a binder that accepts a partial comparison and answers
"satisfied" would reintroduce, in one module, every claim the previous three were built to
avoid. The current schema makes that risk concrete: `application.source.architecture`,
`runtime.family`, `environment.windows_architecture` and `environment.prefix_role` are four
mandatory desired requirements that helm-observe 0.1 cannot establish at all.

## Options

**A. A three-valued satisfaction verdict** — `SATISFIED`, `UNSATISFIED`, `INDETERMINATE`.

**B. No overall verdict** — per-claim binding results only.

**C. Two axes** — a contradiction axis over values actually compared, and a coverage summary
over how much of the desired document was reached, with no success token anywhere.

The [design report](../research/HELM-BIND-ARCHITECTURE.md#3-the-overall-result-question)
scores all three. A collapses on the word "required": if the unobservable mandatory claims
are required, `SATISFIED` is unreachable and decorative; if they are not, a caller who omits
them obtains a false success. B is safe but exports aggregation to unreviewed callers. C
keeps B's safety property and adds a reviewed structure.

## Accepted decision, 2026-09-09

Accepted **subject to bounded pre-implementation corrections**, which are recorded in the
design report and reflected below. Acceptance settles the architecture only: overall-verdict
semantics, mapping semantics, cross-crate dependencies and the identity graph. It stabilises
no schema or API and **does not authorise implementation**, which needs a separate owner
instruction.

> Build `helm-bind` 0.1 as a **pure, deterministic, authority-free** comparison library that
> reports per-claim binding outcomes plus a two-axis summary, and that **emits no satisfaction,
> compatibility or readiness verdict of any kind**.

### Accepted authority boundary

No filesystem authority, no network, no process execution, no environment lookup, no registry,
no Wine, no package lookup, no discovery, no capability descriptors, no clock, no randomness.
A pure function of four already-validated inputs, byte-identical on every platform.

### Accepted inputs

`ValidatedAppSpec`, an inert `ValidatedBindingPlan`, the `ValidatedPlan` that produced the
observation, and the `ObservationArtifact`. The plan is required because the artifact
deliberately carries no target paths; the binding plan is required because observation target
IDs are deliberately semantically neutral and no naming heuristic over them is acceptable.

### Accepted result semantics

- **Ten** distinct claim states, none collapsing into another: `Match`, `Mismatch`,
  `DesiredValueUnspecified`, `NotObserved`, `UnsupportedBinding`, `Absent`,
  `ObservationRejected`, `ObservationFailed`, `ObservationOmitted`, and
  `ObservationNotInterpretable` for a future `#[non_exhaustive]` observation variant.
- `Contradicted` if and only if at least one claim is `Mismatch`. `Absent`, observation
  rejection, observation failure, observation omission and unsupported binding are **never**
  promoted to contradiction in 0.1. In particular `Absent` on `EntryPointPresence` stays the
  explicit `Absent` state, deliberately, because the prefix-root association is caller
  asserted rather than attested.
- Coverage counts semantic desired claims only. Contextual and human metadata — the spec
  digest, application ID, application version, artifact labels — and role selector keys
  receive no claim outcome and do not enter the coverage denominator.
- **Coverage can never be complete in 0.1**, because every valid spec carries **four**
  mandatory semantic requirements that helm-observe cannot establish: source architecture,
  runtime family, Windows architecture and prefix role. The bound holds even when
  `disabled_dlls` is empty. This is a property-test obligation, `unsupported_count >= 4`, and
  it is what makes a false global success structurally unreachable: no global success token
  exists, and mandatory coverage is visibly incomplete.

### Accepted identity rules

- The observation's `subject_spec_sha256` must equal the spec identity, and the artifact's
  `authorized_plan_sha256` must equal the supplied plan identity. Either mismatch is a typed
  **refusal** that produces no claim outcomes at all, never a claim-level contradiction.
  Both comparisons work against the current public APIs; **no change to helm-observe is
  required**. The declared subject of the binding plan must equal the spec identity too. A
  refusal produces **no report bytes, no claim outcomes and no report digest**; a later
  evidence or reporting layer may record that an attempt was refused, but the binder must
  never manufacture a comparison artifact when no valid comparison occurred.
- The binding plan carries exact-byte SHA-256 identity, because it materially decides which
  desired claim is compared to which observed object, and without it a report is not
  reproducible from its own recorded inputs.
- The report has exact deterministic bytes and a SHA-256 over exactly those bytes, binding
  four input digests in an acyclic graph. No timestamp, signature, random identifier,
  hostname or path.

### Accepted typed comparison domains

Five comparators and no generic "compare field X to target Y" language: application-source
body, runtime-artifact body, verification-definition body, entry-point presence and
entry-point body. Each fixes which desired field it reads and which observation `Observable`
it requires, so mapping a runtime archive onto an installed-loader claim is inexpressible
rather than merely discouraged. Expected values always come from the app-spec; the binding
plan chooses only where to look.

**Both entry-point comparators require `RegularFileSha256`.** `DirectoryMetadata` cannot
establish a regular entry-point file, because a regular file under that observable is rejected
as `WrongKind`. helm-observe is **not** changed and no regular-file-metadata observable is
added; the binder uses the `ObservedFile` fact for presence and ignores the digest when the
desired claim does not require content identity.

**Both entry-point comparators bind the path.** The mapped target path in the `ValidatedPlan`
must equal the desired entry-point path exactly, byte for byte, with no normalisation, no case
folding and no target-ID heuristic; a difference is `Mismatch` before the target is treated as
the desired entry point at all, so a same-content file at another relative path can never
satisfy `EntryPointBody`. Both must also sit under the caller-declared
`asserted_prefix_root_id`, whose name is deliberately explicit about being an assertion: it is
required when an entry-point claim is mapped, absent otherwise, and a mapped entry-point
target under a different root is a typed refusal, not a contradiction.

### Accepted retained limits

Binding a body establishes the identity of **that body** and nothing else: not installation,
not that it produced the current prefix, not that a loader came from it, not that a process
loaded it. Entry-point conclusions may state path binding, observed kind and digest agreement,
and may never imply executable permission, PE validity, Wine loadability, launch or correct
installation. A verification-definition body match is never verification executed, passed or
fresh. Absence of a mapped body does not imply the installed runtime is wrong.

The association between an observation root and the application's dedicated prefix is a
**caller assertion** that the binder records and cannot verify. It proves neither dedication,
nor ownership, nor that the root is truly the prefix of the application, in the same spirit as
the 2026-09-09 cohort attestation clarification to ADR-0022: an external precondition, never
an attestation. No binding report may upgrade the assertion into attestation.

### Accepted dependency decision

Depend directly on `helm-app-spec` and `helm-observe`. Reparsing their bytes is impossible for
the spec, which has no serializer and whose identity is its exact input bytes; a shared crate
would abstract nothing, since the identifier, path and digest grammars already coincide; and
caller-supplied projections would let a caller hand the binder fabricated facts. The `sha2`
`force-soft` feature unification that follows is **performance and build composition only**,
already recorded, and hashing is not redesigned here.

### Accepted relation to other modules

`helm-evidence` may later reference an exact binding report by digest; `helm-bind` must not
grow evidence-completeness logic and does not depend on it. `helm-launch` remains unaccepted
and undesigned, and the report contains no readiness vocabulary, so a successful comparison
confers no launch permission.

## Consequences

The project gains a reviewable place for comparison, with the semantic traps enumerated and
mostly made structurally unreachable rather than merely tested. The cost is honest and
visible: in 0.1 a binding report can never claim completeness, and roughly a third of the
current desired schema has no comparator at all. That is a true statement about what HELM can
establish today, and it belongs in the output rather than in a footnote.

This acceptance settles overall-verdict semantics, mapping semantics, cross-crate dependencies
and the identity graph. It does **not** authorise implementation, which needs a separate owner
instruction, and it stabilises no schema or API.

## Falsification and approval boundary

Apply the design report's
[falsifiers](../research/HELM-BIND-ARCHITECTURE.md#19-falsifiers). Reject any design that
lets omitted unobservable claims yield success, binds an observation for one spec to another,
pairs an artifact with a foreign plan, accepts caller-supplied expected hashes, equates a
runtime archive with an installed loader, treats an entry-point match as a launch, treats a
definition identity as verification success, collapses absence into failure or failure into
mismatch, uses target-ID naming heuristics, produces nondeterministic output, performs I/O, or
creates an identity cycle.

**No system experiment is proposed.** The module performs no syscall and depends on no kernel
or filesystem behaviour, so there is nothing a virtual machine could settle that adversarial
and property tests cannot. The environment-dependent facts it relies on belong to
helm-observe and are already evidenced by OBS-FS-01.

This ADR accepts no licence, production release, privileges, receipt infrastructure, binder
implementation, launcher, recovery or larger HELM subsystem, and supersedes no accepted ADR.
ADR-0021 and ADR-0022 are deliberately **not** amended to absorb this responsibility. **A0-7ZIP remains experimental FAIL.**

## Owner decisions, all recorded

All eight questions the design raised were decided on 2026-09-09 and are tabulated in the
design report's
[owner decisions](../research/HELM-BIND-ARCHITECTURE.md#23-owner-decisions--all-eight-decided-on-2026-09-09).
The bounded corrections directed alongside acceptance are applied in the design report and
reflected above: the entry-point observable rule, path binding on both entry claims, the
renamed and conditional `asserted_prefix_root_id`, the precise claim universe, the
four-claim coverage theorem, and the corrected state and size counts.
