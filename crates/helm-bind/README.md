# helm-bind (experimental 0.1)

Pure comparison of **desired** application claims with the **actual** observations that
were made, under Accepted
[ADR-0023](../../docs/adr/ADR-0023-binding-authority.md).

It answers exactly one question: **what follows from comparing explicitly mapped desired
claims with the observations that were actually made?**

**Status: implementation candidate on a product branch, awaiting independent review. Not
owner-merged and not a release.** Schema, API and numeric limits are experimental;
`publish = false`.

## There is no verdict

This crate emits **no satisfaction, compatibility or readiness verdict**, and no global
success token of any kind. It does not answer whether an application is satisfied,
compatible, correctly installed, ready or launchable. Comparison stops here; execution
belongs to a future `helm-launch`, which does not exist and is not authorised. Evidence
completeness belongs to `helm-evidence`, which this crate does not depend on.

A report carries one typed outcome per semantic desired claim, plus two axes:

- **contradiction** — `Contradicted` if and only if at least one claim is a `Mismatch` of two
  known values. Absence, observer rejection, observer failure, budget omission, unsupported
  binding, an unmapped claim, an unspecified desired value and an uninterpretable outcome are
  never promoted into contradiction.
- **coverage** — deterministic counts over the ten claim states, plus the claim classes that
  have no comparator at all.

## Coverage is necessarily incomplete in 0.1

Every valid `helm-app-spec` document carries four mandatory semantic requirements that
`helm-observe` 0.1 cannot establish: source architecture, runtime family, Windows
architecture and prefix role. So `unsupported_binding >= 4` always, and a caller cannot map a
couple of easy digests, omit everything unobservable and obtain success — because no success
token exists. That is a property test, not a promise.

## Authority

```text
untrusted mapping bytes  --parse_binding_plan-->  ValidatedBindingPlan     inert
ValidatedAppSpec + ValidatedBindingPlan + ValidatedPlan + ObservationArtifact
                         --bind-->  BindingReport
```

`bind` is a **pure function**. No filesystem, no descriptors, no network, no subprocess, no
environment, no cwd, `HOME` or `PATH`, no registry, no Wine, no package lookup, no discovery,
no clock, no randomness, no writes and no OS backend. There is no product-semantic `cfg`
branch anywhere, and `#![forbid(unsafe_code)]` holds. Given the same four validated inputs
the exact `BindingReport` bytes are therefore identical on Linux, Windows and macOS.

**What CI actually executes, precisely.** The three-platform job runs `cargo fmt --check`,
clippy and `cargo test -p helm-bind` on Ubuntu, Windows and macOS. On all three that covers
the binding-plan parser, the claim universe, the verdict algebra and the report serializer,
including one fixed record whose exact bytes and digest are identical on every runner. It
does **not** run `bind` over four genuine inputs on Windows or macOS, because
`ObservationArtifact` has no public constructor outside `helm-observe`'s Linux backend; those
end-to-end tests run on Linux. The determinism claim rests on that three-platform execution
of the only platform-sensitive step, serialization, together with the absence of any
platform-dependent construct in the comparison itself.

## The mapping is explicit, and carries no expected values

Observation target IDs are deliberately semantically neutral, so nothing is inferred from a
target's spelling. An inert `BindingPlan` document maps typed desired claims to target IDs
using a closed five-value vocabulary: `source_body`, `runtime_artifact_body` with a role,
`verification_definition_body` with a role, `entry_point_presence` and `entry_point_body`.

It carries **no** expected size, expected digest, predicate, operator, field path, filesystem
path, expression, script or satisfaction constant. Expected values always come from the
application specification. The mapping has exact-byte SHA-256 identity, and the report binds
it, because the mapping decides which desired claim is compared with which observed object.

## The prefix root is an assertion, never an attestation

`asserted_prefix_root_id` names the observation root the **caller asserts** is the
application's prefix. It proves neither dedication, nor ownership, nor that the root belongs
to this application, and no report upgrades it. It is required exactly when an entry-point
claim is mapped, absent otherwise, and every mapped entry-point target must sit under it or
the binding is refused.

## What a match does not mean

An agreeing body establishes the identity of **that body** and nothing else: not that it was
installed, not that it produced the current prefix, not that a current loader came from it,
not that any process loaded it. An absent body does not mean the installed runtime is wrong;
it means that claim could not be established.

An agreeing entry point establishes a byte-exact path binding under the asserted root, a
regular-file kind and, when the specification states one, a digest. Never executable
permission, PE validity, Wine loadability, correct installation or launchability.

A verification-definition body match is definition-body identity. Never verification
executed, verification passed or verification fresh.

## Refusals produce nothing

When the four documents cannot form a valid comparison — a mapping or observation declaring
another specification, an artifact paired with a foreign observation plan, an unknown target,
root or role, an incompatible observable, or an entry-point target outside the asserted root
— `bind` returns a typed refusal and **no** report bytes, **no** claim outcomes and **no**
report identity. Nothing is partly bound first. A later evidence or reporting layer may record
that an attempt was refused; this crate does not manufacture an artifact for a comparison that
did not happen.

## Validity is not authenticity

A `ValidatedAppSpec` proves the supplied desired bytes are valid under schema 0.1. An
`ObservationArtifact` proves what that observer recorded under its capability authority within
its documented limits. A `ValidatedBindingPlan` proves the caller's mapping is well formed.
None of them proves who supplied it. The identity checks here are **consistency** checks
between supplied documents, never authentication or attestation.

## Report identity

Deterministic exact bytes plus SHA-256 over exactly those bytes, binding four input
identities in an acyclic graph:

```text
app-spec SHA + observation-plan SHA + binding-plan SHA + observation-artifact SHA
        -> BindingReport exact bytes -> report SHA
```

No timestamp, hostname, random identifier, signature, host path, desired relative path or
pointer address. The whole report is lowercase by construction, so an uppercase token cannot
appear at all.

**No term the binder emits is a verdict word.** Not a claim class, not a state, not the
contradiction value, not a coverage key, not the schema name. That is the property, and it is
the one ADR-0023 requires.

It is deliberately **not** a claim that the bytes never contain such a word. Validated role
and DLL selectors are caller data, written verbatim, and `ready`, `pass`, `fail`, `verified`,
`complete`, `snapshot`, `compatible`, `satisfied` and `installed` are all legal
`helm-app-spec` identifiers, so a valid specification can put any of them in a report under a
`"role"` key. That asserts nothing, exactly as a file named `passed.txt` asserts nothing, and
rejecting such names is not this module's business. A selector can never reach a slot the
binder controls: identifier output cannot emit a quote, so a role cannot close its string and
impersonate a claim class or a state.

## Build composition

Linking this crate necessarily brings `helm-app-spec` and `helm-observe` into one Cargo
graph, which unifies `sha2` features and selects the software backend that `helm-app-spec`
requests. That is a **performance and build-composition** effect only: the feature selects an
implementation, not a digest, so outputs are identical either way. Hashing is not redesigned
here and no existing crate's dependency features were changed. The repository keeps separate
package builds and a standalone `cargo test -p helm-evidence` so the un-unified backend stays
exercised.

## Known limitation

`ObservationArtifact` has no constructor outside `helm-observe`'s observation backend, which
exists only on Linux x86_64. End-to-end `bind` tests therefore run there. Everything they
exercise is platform-independent, and the binding-plan parser, the claim universe, the verdict
algebra and the report serializer — including its exact fixture digest — are tested on all
three platforms.
