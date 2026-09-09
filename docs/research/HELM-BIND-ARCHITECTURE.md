# helm-bind 0.1 architecture and design report

**Status: architecture accepted by the owner on 2026-09-09, with bounded corrections
applied in place. Design only — no implementation, no crate, no product API change.**\
**Authoritative base:** `9b09e7d3f6d216fb0258d23963d70bb889793567`.\
**Task:** design the fourth HELM product module, which answers exactly one question —
*what can we conclude when explicitly comparing desired claims with the observations that
were actually made?*\
**Not authorised here:** `helm-launch`, any implementation, any change to
`helm-app-spec`, `helm-observe` or `helm-evidence`.

Everything below is derived from the **current** public Rust types on main, read directly,
not from historical sketches.

## 1. Reconstructed current contracts

### 1.1 `helm-app-spec` — desired claims

`parse_spec(&[u8]) -> Result<ValidatedAppSpec, SpecErrors>` is the only constructor. Identity
is SHA-256 of the exact supplied bytes; there is no canonicalisation and **no model
serializer**. The complete desired surface, from `crates/helm-app-spec/src/model.rs`:

| Path | Type | Notes from the code |
|---|---|---|
| `spec_sha256()` | `Digest` | exact input bytes, whitespace and field order included |
| `application().id()` | `Identifier` | local ID; "no application-specific branches depend on its value" |
| `application().version()` | `&str` | "human metadata, never used as immutable identity" |
| `application().source().size()` | `u64` | required source byte length |
| `application().source().sha256()` | `Digest` | required source digest |
| `application().source().architecture()` | `SourceArchitecture::X86_64` | "declared, never discovered from the source artifact" |
| `runtime().family()` | `RuntimeFamily::Wine` | "no runtime discovery or selection" |
| `runtime().artifacts()` | `[ArtifactRequirement]`, 1–16 | each `role: Identifier`, `size: u64`, `sha256: Digest`, `label: Option<String>` |
| `environment().windows_architecture()` | `WindowsArchitecture::Win64` | "not a host-architecture observation" |
| `environment().prefix_role()` | `PrefixRole::Dedicated` | "by intent; grants no creation/deletion authority" |
| `environment().disabled_dlls()` | `[String]`, 0–8 | "effective configuration is not observed" |
| `entry_point().path()` | `RelativeEntryPoint` | begins `drive_c/`, forward slashes, inert |
| `entry_point().sha256()` | `Option<Digest>` | **optional**; `None` leaves identity unspecified |
| `verification()` | `[VerificationDefinitionRef]`, 1–8 | each `role`, `size`, `sha256`; "reference to frozen pre-execution definition bytes, never to resulting evidence" |

`Digest` is a private-field wrapper over a `String` of exactly 64 lowercase hex characters,
readable only through `as_str()`. `Identifier` is 1–80 bytes, first byte lowercase ASCII or
digit, remainder lowercase ASCII, digits, `.`, `_`, `-`.

### 1.2 `helm-observe` — actual facts

Public surface relevant here:

| Item | Shape |
|---|---|
| `parse_plan(&[u8]) -> Result<ValidatedPlan, PlanErrors>` | only constructor |
| `ValidatedPlan::sha256()` | `Digest` over exact plan bytes |
| `ValidatedPlan::subject_spec_sha256()` | `Digest`, opaque caller context |
| `ValidatedPlan::root_ids()` | `&[String]` |
| `ValidatedPlan::targets()` | `&[Target]`, each `id()`, `root()`, `path() -> Option<&str>`, `observable()` |
| `Observable` | `DirectoryMetadata` \| `RegularFileSha256`, `#[non_exhaustive]` |
| `ObservationArtifact` | `exact_bytes()`, `sha256()`, `record()` |
| `ObservationRecord` | public fields `subject_spec_sha256`, `plan_sha256`, `roots: Vec<(String, Option<ObjectTuple>)>`, `targets: Vec<TargetObservation>` |
| `TargetObservation` | public `target_id: String`, `root_id: String`, `outcome: TargetOutcome` |
| `TargetOutcome` | `ObservedFile(FileFacts)` \| `ObservedDirectory(DirectoryFacts)` \| `Absent` \| `Rejected{rejection, observed_kind}` \| `Failed{failure, observed_kind, partial_bytes}` \| `NotObserved(BudgetReason)`, `#[non_exhaustive]` |
| `FileFacts` | public `tuple: ObjectTuple`, `link_count: u64`, `bytes_read: u64`, `sha256: Digest` |
| `helm_observe::Digest` | `as_bytes() -> &[u8; 32]`, `to_hex() -> String` |

Three consequences the design must respect:

1. **The artifact contains no target paths.** Only logical `target_id` and `root_id`. Paths
   live in the `ValidatedPlan` only.
2. **Target IDs are semantically neutral by design**, and roots carry no path at all.
3. **Every observation enum is `#[non_exhaustive]`.** A downstream matcher must have a
   catch-all arm, and that arm must not fall into a benign state.

`bytes_read` on `ObservedFile` equals the observed length, because `Streamed::Complete`
requires the streamed total to equal the pre-read length. Desired `size` is therefore
bindable, not only desired `sha256`.

### 1.3 Path and identifier grammar compatibility, checked not assumed

| Property | `helm-app-spec` entry point | `helm-observe` target path | Compatible? |
|---|---|---|---|
| total bytes | ≤ 1024 | ≤ 1024 | yes |
| components | ≤ 32 | ≤ 32 | yes |
| component bytes | 1–255 | 1–255 | yes |
| alphabet | ASCII alphanumeric plus `.` `_` `-` space | printable ASCII `0x20..0x7f`, no backslash | **strict subset** |
| `.`/`..`, leading space, trailing space or dot | rejected | rejected | yes |
| extra constraints | must start `drive_c/`, Windows device names rejected | none | one-directional, harmless |

**Every valid desired entry-point path is expressible as a valid observation target path.**
The identifier grammars are identical in both crates, and both digest grammars are exactly 64
lowercase hex characters. No translation layer is needed anywhere, which removes a whole
class of would-be adapter code.

### 1.4 `helm-evidence`

`verify(&Path) -> Report` opens a bundle directory and checks declared evidence
completeness and validity. It has filesystem authority and its own contract vocabulary. It is
**not** an input to binding and `helm-bind` must not depend on it; see section 12.

## 2. Fundamental separation, restated for the new module

| Module | Answers | Authority |
|---|---|---|
| `helm-app-spec` | what is desired | none, pure |
| `helm-observe` | what actually exists at authorised targets | caller-granted descriptors, Linux x86_64 |
| **`helm-bind`** | **what follows from comparing the two** | **none** |
| `helm-evidence` | is the declared evidence complete and valid | bundle filesystem read |
| `helm-launch` | execution | not accepted, not designed |

`helm-bind` gets **no** filesystem authority, no network, no process execution, no
environment lookup, no registry, no Wine, no package lookup, no discovery and no capability
descriptors. Given identical validated inputs its output must be byte-identical on Windows,
Linux and macOS, which is achievable because none of its inputs is a host object: they are
all already-validated in-memory values plus one inert document.

## 3. The overall-result question

This is the highest-priority semantic decision, so it is settled first and everything else is
shaped to it.

### 3.1 The dangerous case, stated precisely

> A caller supplies a spec, maps only `application.source` and one runtime artifact to two
> observation targets whose digests happen to match, omits every claim the observer cannot
> establish, and obtains a global "application satisfied" result.

Under the current schema that caller would have omitted: runtime family, Windows
architecture, dedicated-prefix intent, disabled DLLs, source architecture and the entry
point. A design that returns success there is a design that lies.

### 3.2 Options

**A. `SATISFIED` / `UNSATISFIED` / `INDETERMINATE`.** Any proven mismatch is `UNSATISFIED`,
every required claim proven is `SATISFIED`, otherwise `INDETERMINATE`. The trap is the word
"required". If the unobservable claims are required, `SATISFIED` is unreachable in 0.1 and
the variant is decorative. If they are not required, the dangerous case returns `SATISFIED`.
There is no third reading. **Rejected.**

**B. No global verdict at all; per-claim results only.** Safe and honest, and it makes the
dangerous case unexpressible. But it pushes aggregation to every caller, and a caller that
aggregates badly reproduces the danger outside the module, where nobody reviews it.
**Rejected as insufficient, though its safety property is kept.**

**C. Two axes, consistency and coverage, with no success token.** Contradiction is a fact
about values that were actually compared. Coverage is a fact about how much of the desired
document was reached. Neither axis alone, and no combination of them, spells success.
**Recommended.**

### 3.3 Recommended shape

```text
BindingReport
  contradiction : Contradicted { mismatches: u32 } | NoClaimContradicted
  coverage      : per-state counts over every claim in the spec,
                  plus the explicit list of claim classes that have
                  no comparator in 0.1
  claims        : one typed outcome per desired claim, in spec declaration order
```

There is **no** `SATISFIED`, `UNSATISFIED`, `COMPATIBLE`, `READY`, `COMPLETE`, `PASS` or
`FAIL` anywhere in the report vocabulary, exactly as `helm-observe` forbids them in an
observation artifact. `NoClaimContradicted` is deliberately negative in form: it says what
did *not* happen.

### 3.4 Why the dangerous case is impossible, and it is provable

*Owner correction of 2026-09-09: the original proposal said "at least three". That was valid
but not exact, because `application.source.architecture` is mandatory and unsupported too.
The exact bound is four.*

Every valid `ValidatedAppSpec` necessarily carries four mandatory semantic requirements that
have no comparator in 0.1, verified against `validate::object`, whose optional-key list is
empty for each of these containers:

1. `application.source.architecture`
2. `runtime.family`
3. `environment.windows_architecture`
4. `environment.prefix_role`

Therefore:

> **Theorem (0.1).** For every `ValidatedAppSpec` and every input combination, the binding
> report contains **at least four** claims in state `UnsupportedBinding`, so its coverage is
> never complete. The bound holds even when `disabled_dlls` is empty.

This is a property-test obligation, not a hope: enumerate any accepted spec, bind it against
any artifact, and assert `unsupported_count >= 4`. The dangerous case yields
`NoClaimContradicted` with a visibly incomplete coverage summary and every unreached claim
named — a result no reasonable reader mistakes for success, because **no global success token
exists at all**.

## 4. Complete desired-claim bindability matrix

Derived field by field from the current model, not from names.

| Desired field | Classification | Comparator in 0.1 | Why |
|---|---|---|---|
| `spec_sha256` | contextual identity only | none | it identifies the document being bound; comparing it to an observed object is meaningless. It is bound into the report and used for the subject check of section 6 |
| `application.id` | semantically inappropriate to compare | none | there is no observable "application identity"; it is a local label |
| `application.version` | semantically inappropriate to compare | none | explicitly human metadata in the model |
| `application.source.size` | **directly bindable with an explicit mapping** | `SourceBody` | `FileFacts::bytes_read` |
| `application.source.sha256` | **directly bindable with an explicit mapping** | `SourceBody` | `FileFacts::sha256` |
| `application.source.architecture` | not bindable in 0.1 | none | declared, not discovered; the observer parses no PE header |
| `runtime.family` | not bindable in 0.1 | none | no observable establishes a runtime family |
| `runtime.artifacts[i].role` | mapping key only | — | names the claim; never compared to anything |
| `runtime.artifacts[i].size` | **bindable with an explicit mapping** | `RuntimeArtifactBody(role)` | `FileFacts::bytes_read` |
| `runtime.artifacts[i].sha256` | **bindable with an explicit mapping** | `RuntimeArtifactBody(role)` | `FileFacts::sha256` |
| `runtime.artifacts[i].label` | semantically inappropriate to compare | none | human metadata |
| `environment.windows_architecture` | not bindable in 0.1 | none | intent, not a host observation |
| `environment.prefix_role` | not bindable in 0.1 | none | intent; ownership and dedication are unobservable, and `helm-observe` never enumerates a prefix |
| `environment.disabled_dlls[i]` | not bindable in 0.1 | none | effective DLL policy is not a filesystem fact the observer exposes |
| `entry_point.path` | **bindable with an explicit mapping, plus a caller root assertion** | `EntryPointPresence` | the mapped target's path in the `ValidatedPlan` must equal it, and the outcome must be `ObservedFile` |
| `entry_point.sha256` when `Some` | **bindable with an explicit mapping** | `EntryPointBody` | `FileFacts::sha256` |
| `entry_point.sha256` when `None` | desired value unspecified | `EntryPointBody` returns `DesiredValueUnspecified` | the spec declines to state it; that is neither a match nor a mismatch |
| `verification[i].role` | mapping key only | — | names the claim |
| `verification[i].size` | **bindable with an explicit mapping** | `VerificationDefinitionBody(role)` | `FileFacts::bytes_read` |
| `verification[i].sha256` | **bindable with an explicit mapping** | `VerificationDefinitionBody(role)` | `FileFacts::sha256`; identity of the frozen definition body **only** |

Five comparators, covering at most `1 + 16 + 8 + 2 = 27` **mappable** claims.

### 4.1 The claim universe, defined precisely

*Owner correction of 2026-09-09.* Coverage must **not** mean "every serialized field in
`helm-app-spec`". Three kinds of field are distinguished, and only the first enters the
coverage denominator.

**A. Semantic desired claims — these receive a claim outcome and enter coverage.**

| Bindable in 0.1 | Instances |
|---|---|
| application source body | exactly 1 |
| runtime artifact body, per role | 1–16 |
| entry-point presence | exactly 1 |
| entry-point body | exactly 1 |
| verification-definition body, per role | 1–8 |

| Unsupported in 0.1 | Instances |
|---|---|
| application source architecture | exactly 1 |
| runtime family | exactly 1 |
| Windows architecture | exactly 1 |
| prefix role, dedication intent | exactly 1 |
| disabled-DLL requirement, per entry | 0–8 |

**B. Contextual and human metadata — no claim outcome, not in the denominator.**
`spec_sha256`, `application.id`, `application.version`, `runtime.artifacts[i].label`.
Comparing them to an observed object is meaningless; `spec_sha256` is used only for the
subject check of section 6 and is bound into the report as an input identity.

**C. Selector keys — no claim outcome, not in the denominator.**
`runtime.artifacts[i].role` and `verification[i].role` name claim instances; they are the key
under which a claim appears, never an independently compared value.

Maximum semantic claim instances in one report:
`1 + 16 + 1 + 1 + 8` bindable `+ 1 + 1 + 1 + 1 + 8` unsupported `= 39`.

## 5. Inputs

### 5.1 Options

- **A. `ValidatedAppSpec` + `ObservationArtifact`.** Impossible for the entry point: the
  artifact carries no paths, so the binder could only compare a digest against a target
  chosen by name, which is the naming heuristic that section 11 forbids. **Rejected.**
- **B. `ValidatedAppSpec` + `ValidatedPlan` + `ObservationArtifact`.** Recovers paths and
  observables, but still needs to decide *which* target answers *which* claim, and the only
  material available is the target ID. Deriving that from spelling is a heuristic.
  **Rejected.**
- **C. `ValidatedAppSpec` + `BindingPlan` + `ValidatedPlan` + `ObservationArtifact`.** The
  mapping becomes an explicit, inert, identity-bearing document. **Recommended.**
- **D. Caller-supplied projection types.** Moves the semantics outside the reviewed module
  and lets a caller hand the binder fabricated "facts". **Rejected.**

### 5.2 Conceptual signature

```rust
pub fn bind(
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> Result<BindingReport, BindingRefusal>;
```

A pure function of four already-validated values. No `&mut`, no capability, no I/O, no clock.

## 6. Subject-spec identity

`ObservationPlan` and `ObservationArtifact` carry `subject_spec_sha256` as opaque context,
and `helm-observe` deliberately does not attest that the context is correct. Binding is where
it acquires meaning, so the binder must check it:

```text
observation.record().subject_spec_sha256.to_hex() == spec.spec_sha256().as_str()
```

**Disposition: a typed refusal, not a claim result.** `bind` returns
`Err(BindingRefusal::SubjectSpecMismatch { spec, observed_subject })` and produces **no**
claim outcomes at all. Reasons: no desired claim was compared, so calling it a contradiction
would let it be counted beside genuine mismatches and averaged away; and the two inputs are
each individually valid, so "invalid input" understates that it is the *combination* that is
refused. Refusing early also guarantees that a report never contains results computed against
another application's observation.

**What this check does and does not establish.** It establishes that the plan's declared
subject is this exact spec document. It does **not** establish that the observed objects are
relevant to that spec, and it is not an authentication of whoever wrote the plan. It is a
consistency check on caller-declared context; the report must say so in those words.

## 7. Observation-plan identity, and the target-path attack

The attack in the instruction is real and reachable today:

> An artifact contains `target_id = "entry"`, but the `ObservationPlan` that produced it
> pointed at a completely different relative path. A binder comparing only target ID and
> digest compares the wrong object.

Mitigation, in two parts.

1. **Pair verification.** `observation.record().plan_sha256 == plan.sha256()`. Both are
   `helm_observe::Digest` with derived equality over all 32 bytes, so this needs **no
   helm-observe API change at all**. A failure is
   `Err(BindingRefusal::ObservationPlanMismatch { .. })`, refused before any claim is bound,
   for the same reasons as section 6.
2. **Path binding for path-relevant claims.** For `EntryPointPresence` the binder resolves
   the mapped `target_id` inside the verified `ValidatedPlan`, reads `Target::path()`, and
   requires it to equal `entry_point.path().as_str()` exactly, byte for byte, with no
   normalisation. A difference is `Mismatch` on the path claim, not a refusal, because it is
   a genuine disagreement between two authorised documents.

No absolute path enters any artifact anywhere, and the observation artifact stays
path-free: paths are read from the plan document the caller already holds.

**The residual gap, stated plainly.** `Target::path()` is relative to a *logical root*, and
roots carry no path and no meaning. That the root in question is the application's dedicated
prefix is a **caller assertion**, recorded in the `BindingPlan` as `asserted_prefix_root_id`
and reported as an assertion. `helm-bind` cannot verify it and must never phrase an
entry-point conclusion as though it had. This is the honest analogue of `helm-observe`'s
cohort clarification: the association is an external precondition, not something the module
attests.

*Owner corrections of 2026-09-09.*

1. **The field is named `asserted_prefix_root_id`**, so the wire vocabulary itself makes the
   unattested nature visible. It is **required** when the plan maps `EntryPointPresence` or
   `EntryPointBody`, and must be **absent** when neither is mapped.
2. **Every mapped entry-point claim must sit under that asserted root.** For each mapped
   `EntryPointPresence` and `EntryPointBody`, `Target::root()` must equal
   `asserted_prefix_root_id`; otherwise `Err(BindingRefusal::EntryPointRootMismatch)`. That is
   a broken mapping, not a desired-versus-observed contradiction, so it is a refusal and not a
   claim outcome.
3. **Path binding applies to both entry-point claims, not only to presence.** For each of
   them the mapped target's `ValidatedPlan` path is compared with
   `spec.entry_point().path().as_str()` exactly, byte for byte, with no normalisation, no case
   folding and no target-ID heuristic. If the path differs the claim outcome is `Mismatch`
   **before** the target is treated as the desired entry point at all, so a same-content file
   at another relative path can never satisfy `EntryPointBody`.
4. No absolute path enters a `BindingReport`.

## 8. The `BindingPlan`

### 8.1 Is it needed?

Yes. Without it the binder must decide which observation answers which claim, and the only
material available is a target ID whose neutrality is a deliberate, documented property of
`helm-observe`. Any rule over spellings — `"entry"`, `"wine"`, `"source"` — is a heuristic
that a caller can trip accidentally and an attacker can arrange deliberately.

### 8.2 Shape

An inert JSON document, parsed by `helm-bind` from untrusted bytes with the same discipline
as the other two documents: closed schema, unknown fields rejected, duplicate decoded keys
rejected at every object depth, bounded sizes, exact-byte identity, no I/O on any path.

```jsonc
{
  "schema": "helm-binding-plan",
  "version": "0.1",
  "subject_spec_sha256": "<64 hex>",       // must equal the spec being bound
  "asserted_prefix_root_id": "<observation root id>", // caller assertion, see section 7
                                             // required iff an entry-point claim is mapped
  "claims": [
    { "claim": "source_body",                  "target": "src-body" },
    { "claim": "runtime_artifact_body", "role": "wine-devel-archive", "target": "rt-1" },
    { "claim": "entry_point_presence",         "target": "entry" },
    { "claim": "entry_point_body",             "target": "entry" },
    { "claim": "verification_definition_body", "role": "protocol", "target": "def-1" }
  ]
}
```

Closed vocabulary: exactly five `claim` values, one optional `role` whose presence is fixed
per claim kind, and one `target` identifier. **No** expression language, **no** predicates,
**no** scripts, **no** filesystem paths, **no** expected hashes and **no** caller-supplied
satisfaction constants. The expected value always comes from the app-spec; the plan chooses
only *where to look*.

Ceilings: ≤ 16 KiB document, ≤ 27 claim entries, identifiers reusing the shared 1–80 byte
grammar, nesting ≤ 4.

### 8.3 Does the `BindingPlan` need exact-byte identity? — hypothesis tested, confirmed

The hypothesis was: *if it materially controls which desired claim is compared to which
observed object, it should have an exact identity and the report should bind it.* Tested by
asking what breaks without it:

- **Reproducibility.** Two different mappings over the same spec, plan and artifact produce
  different reports. Without a mapping identity in the report, an auditor holding the report
  and the three other documents cannot tell which mapping produced it, so the report is not
  reproducible from its own recorded inputs. That alone is decisive.
- **Authority.** The mapping is a semantic choice, not a formatting one: it decides that
  *this* observed body answers *that* desired requirement. A caller may re-map until a match
  appears; recording the mapping identity makes that visible instead of invisible.
- **Counter-argument considered.** One could argue the per-claim outcomes already encode the
  mapping. They do not: they record which claim got which outcome, not which target answered
  it, and two different targets can yield the same outcome.

Confirmed: SHA-256 over exact `BindingPlan` bytes, bound into the report.

### 8.4 Cross-document validation, at bind time

| Condition | Disposition | Why |
|---|---|---|
| plan `subject_spec_sha256` ≠ spec identity | `Err(SubjectSpecMismatch)` | same rule as section 6, applied to the mapping document |
| `target` names an ID absent from the `ValidatedPlan` | `Err(UnknownTarget)` | a dangling reference means the mapping is broken, not that a claim is unobservable |
| `asserted_prefix_root_id` absent from `plan.root_ids()` | `Err(UnknownRoot)` | same |
| `asserted_prefix_root_id` missing while an entry-point claim is mapped, or present while none is | rejected at `BindingPlan` validation | the field's presence is conditional on the mapping, so an inconsistent document is malformed |
| a mapped entry-point target whose `root()` differs from `asserted_prefix_root_id` | `Err(EntryPointRootMismatch)` | the mapping points outside the root the caller asserted to be the prefix; a broken mapping, not a contradiction |
| mapped target's `Observable` incompatible with the claim kind | `Err(IncompatibleObservable)` | this is the section 9 protection, and a broken mapping must not degrade into a claim outcome |
| `role` names a role absent from the spec | `Err(UnknownRole)` | the mapping refers to a requirement that does not exist |
| the same claim key appears twice | rejected at `BindingPlan` validation | duplicate claim, exactly as duplicate IDs are rejected elsewhere |
| two claims map to the same target | **allowed** | `entry_point_presence` and `entry_point_body` legitimately share one target |
| a claim with a comparator has no mapping | `NotObserved` claim outcome | a real, expected state, not an error |

## 9. Typed claim compatibility

A generic `expected SHA == actual SHA` is not architecture; it is a way to compare a Wine
archive with an installed loader and call the result agreement. Compatibility is therefore
enforced structurally by the claim kind, which fixes three things at once:

| Claim kind | Desired field it reads | Required target `Observable` | Comparison domain |
|---|---|---|---|
| `SourceBody` | `application.source.{size, sha256}` | `RegularFileSha256` | immutable application-source body identity |
| `RuntimeArtifactBody(role)` | `runtime.artifacts[role].{size, sha256}` | `RegularFileSha256` | immutable runtime-artifact body identity |
| `VerificationDefinitionBody(role)` | `verification[role].{size, sha256}` | `RegularFileSha256` | frozen verification-definition body identity |
| `EntryPointPresence` | `entry_point.path` | `RegularFileSha256` **only** | existence and regular-file kind at a caller-asserted location |
| `EntryPointBody` | `entry_point.sha256` | `RegularFileSha256` | installed entry-point body identity |

*Owner correction of 2026-09-09.* `EntryPointPresence` previously also permitted
`DirectoryMetadata`. That was wrong: under `directory_metadata` a regular file is rejected as
`WrongKind`, so the observable could never establish a regular entry-point file. Both
entry-point claims therefore require `RegularFileSha256`. helm-observe is **not** changed and
no regular-file-metadata observable is added; the binder simply uses the `ObservedFile` fact
for presence and ignores the digest when the desired claim does not require content identity —
which is exactly the case when `entry_point.sha256` is `None`.

The caller cannot say "compare field X to target Y": the claim kind is a closed enumeration
and each kind hard-wires which desired field it reads. Cross-domain mapping is therefore
inexpressible rather than merely discouraged. The domains stay distinct in the report, so a
reader never sees a runtime-archive body match presented as an installed-loader fact.

## 10. Semantic boundaries that must survive

**Entry point.** Permitted conclusions: the mapped target's plan path equals the desired
path, byte for byte; the observed object is a regular file under the asserted prefix root;
the observed digest equals the desired digest when the spec states one. Forbidden conclusions: executable permission, PE validity, Wine
loadability, successful launch, correct installation. Also forbidden: presenting the
caller-asserted `asserted_prefix_root_id` as a proven prefix.

**Application source and runtime artifacts.** Identical observed bytes establish the identity
of *that body* and nothing else: not that it was installed, not that it produced the current
prefix, not that a current loader came from it, not that any process loaded it. If the mapped
object is absent, the correct conclusion is that the claim could not be established — **not**
that the installed runtime is wrong. The runtime and provenance gap recorded in
[ADR-0022](../adr/ADR-0022-observation-authority.md) survives binding unchanged.

**Verification definitions.** Comparing a definition body establishes definition-body
identity. It must never be phrased as verification executed, verification passed or
verification fresh; those belong to the evidence and execution layers.

## 11. Result taxonomy

**Ten** distinct claim states. None collapses into another. *Owner correction of 2026-09-09:
the original text said "nine states plus" a tenth, which double-counted informally; the
taxonomy is a single closed set of ten.*

| State | Meaning |
|---|---|
| `Match` | both sides known and equal |
| `Mismatch` | both sides known and different |
| `DesiredValueUnspecified` | the spec declines to state the value, e.g. `entry_point.sha256 == None` |
| `NotObserved` | a comparator exists but the `BindingPlan` supplied no mapping for this claim |
| `UnsupportedBinding` | no comparator exists in 0.1 for this claim class |
| `Absent` | the observer established that the mapped target is absent |
| `ObservationRejected { rejection }` | the observer refused the object: symlink, special file, wrong kind, scope violation. A fact about the object, not a failure |
| `ObservationFailed { failure }` | the attempt could not complete: permission, I/O, race, changed during read, reopen unavailable, unsupported platform |
| `ObservationOmitted { reason }` | bounded away by the observer's budget: per-file limit, aggregate limit, not attempted |
| `ObservationNotInterpretable` | the artifact carried a `TargetOutcome` variant this build does not know |

The last state exists because every `helm-observe` result enum is `#[non_exhaustive]`. A
catch-all arm that silently produced `NotObserved` would let a future observer variant
degrade into a benign state; this one is explicit and is never counted as agreement.

**Size and digest are two facts, not one.** For body claims the binder reports both, and a
matching digest with a differing `bytes_read` is a `Mismatch`, not a `Match`.

## 12. Overall verdict algebra

`contradiction = Contradicted` if and only if at least one claim is `Mismatch`; otherwise
`NoClaimContradicted`. Nothing else is promoted to contradiction. Coverage is a set of counts
over the ten states plus the explicit unsupported-claim list. Refusals happen strictly
before any claim is evaluated, so they are outside the algebra.

| Situation | Result |
|---|---|
| one mismatch, ten matches | `Contradicted { mismatches: 1 }`; coverage incomplete; the ten matches are still listed |
| zero mismatches, one unsupported required claim | `NoClaimContradicted`; coverage incomplete; **never** a success |
| zero mismatches, one failed observation | `NoClaimContradicted`; coverage incomplete; `observation_failed = 1` |
| all bindable claims match, prefix intent unobservable | `NoClaimContradicted`; coverage incomplete; unsupported list contains `prefix_role`, `windows_architecture`, `runtime_family` and any DLL claims |
| wrong subject-spec identity | `Err(SubjectSpecMismatch)`; **no** report, no claim outcomes |
| wrong observation-plan identity | `Err(ObservationPlanMismatch)`; **no** report, no claim outcomes |

Precedence is total and unambiguous: refusal beats everything; within a report, contradiction
is a monotone OR over claim states; coverage is independent of contradiction and is never
used to soften it.

**Open judgement for the owner, section 19 item 3.** Should `Absent` on
`EntryPointPresence` count as a contradiction? It is the one case where absence arguably
contradicts a desired claim. The recommendation here is **no**: absence is only meaningful
relative to the caller-asserted prefix root, which the binder cannot verify, so promoting it
would let an incorrect root assertion manufacture a contradiction. It is reported as `Absent`
and counted separately so no reader misses it.

## 13. Purity

A pure function over four immutable values. No OS backend, no `cfg` on product semantics, no
descriptors, no clock, no randomness, no environment, no timestamps in identity, no writes,
no execution, no network. Determinism follows from the same discipline `helm-observe` already
uses: fixed field order, no map iteration, integers rendered decimally, enumerations rendered
from a closed set. The same four inputs give byte-identical reports on Windows, Linux and
macOS, and the whole test suite runs anywhere.

## 14. Cross-crate dependency decision

| Option | Assessment |
|---|---|
| **A. depend directly on `helm-app-spec` and `helm-observe`** | **Recommended.** No semantic drift, full type safety, identities already computed and comparable, downstream tests can construct real inputs through the real constructors |
| B. reparse their serialized bytes | Not viable at all for the spec: `helm-app-spec` has **no serializer**, and its identity is the exact input bytes. A reparsing binder would have to re-implement the validator, which is the definition of semantic drift. For the artifact it would mean writing a parser that shadows the deterministic serializer |
| C. shared contract or utility crate | Nothing to share. The grammars already coincide (section 1.3) and the digests are 64 lowercase hex on both sides. Creating a crate to avoid two dependencies is exactly the premature abstraction the instruction warns against |
| D. caller-supplied projections | Loses type safety and lets a caller hand the binder fabricated facts. The binder would be verifying its own input's shape rather than a validated model |

API stability cost is real and accepted: `helm-bind` pins two experimental APIs and will move
when they move. That is preferable to a copy of their semantics that can silently diverge.
`#[non_exhaustive]` on the observation enums means a new variant is a compile-time
non-breaking change with a defined runtime state, which is the right trade.

## 15. `sha2` feature unification

`helm-app-spec` deliberately enables `sha2/force-soft` to avoid runtime CPU feature discovery.
Cargo unifies features per graph, so any binary linking `helm-bind` gets the software backend
for `sha2` throughout, including `helm-observe` and `helm-bind` itself.

Classification, unchanged from the existing record: **performance and build composition
only**. The feature selects an implementation, not a digest; SHA-256 outputs are identical
either way, so no semantic correctness question arises and hashing is not redesigned here.

Two consequences to record honestly:

1. A binder-containing binary **necessarily** creates the combined graph. There is no
   arrangement in which it links both crates and avoids unification.
2. Separate release binaries remain useful and should be kept. The existing CI already builds
   `helm-app-spec` and `helm-evidence` separately and runs `cargo test -p helm-evidence`
   on its own precisely to keep the un-unified backend exercised. Adding `helm-bind` does not
   change that and must not be allowed to quietly remove it.

`helm-bind` needs `sha2` in its own right, for the report identity of section 17.

## 16. Input authenticity versus type validity

A `ValidatedAppSpec` proves that the supplied desired bytes are syntactically and
semantically valid under schema 0.1. An `ObservationArtifact` proves what that observer
recorded under its capability authority within its documented limits. Neither proves anything
about who supplied them.

`helm-bind` must not upgrade validity into authentication. Concretely: the subject and plan
identity checks of sections 6 and 7 are **consistency** checks between caller-supplied
documents, and the report must describe them that way. The report carries no signature, no
author, no operator identity and no trust statement, and its digests identify documents, not
people.

## 17. `BindingReport` identity

Yes, the report needs exact deterministic bytes and a SHA-256 over exactly those bytes, for
the same reasons the observation artifact does: evidence packaging, reproducibility, a future
launch policy that must reference an exact comparison rather than a recomputed one, and
auditability.

Identity graph, acyclic by construction — every arrow points forward and nothing consumes the
report's own digest:

```text
app-spec bytes ─────sha256──► spec_sha256 ────────────────┐
observation-plan bytes ──sha256──► plan_sha256 ───────────┤
binding-plan bytes ──────sha256──► binding_plan_sha256 ───┼──► BindingReport
observation artifact bytes ──sha256──► artifact_sha256 ───┘        exact bytes
                                                                       │
                                                                    sha256
                                                                       ▼
                                                                report_sha256
```

The artifact already binds `plan_sha256` and `subject_spec_sha256` internally, so the four
recorded digests plus the claim outcomes let an auditor holding the four input documents
recompute the report byte for byte.

Report contents: schema and version, the four digests, the ordered claim outcomes keyed by
claim kind and role, the counts, and the contradiction axis. **Not** in the report: any
filesystem path, any timestamp, any signature, any random or host identifier, any pointer,
any success vocabulary. Paths are deliberately omitted even though the desired path is
caller-supplied rather than host-derived, because the four digests already let an auditor
recover it and omitting it keeps the report small and consistent with the artifact's
path-free rule.

**Size, corrected 2026-09-09.** The `BindingPlan` carries at most **27** mapping entries —
1 source, up to 16 runtime artifacts, up to 8 verification definitions, entry-point presence
and entry-point body. A `BindingReport` carries more than that, because unsupported semantic
claims receive outcomes without ever being mapped: up to **39** claim instances, the 27
mappable ones plus the 4 unavoidable unsupported classes plus up to 8 disabled-DLL
requirements. Each is a short fixed record of a claim kind, an optional role and a state, so
a deterministic report of a few kilobytes remains plausible — but the ceiling is 39 claims,
not 27.

## 18. Relationship to evidence and to launch

**Evidence.** `helm-evidence` may later package or reference an exact `BindingReport` by its
digest. `helm-bind` must not grow evidence-completeness logic, bundle reading or workflow
vocabulary; it produces an artifact and stops. The dependency direction stays one-way and
`helm-bind` does not depend on `helm-evidence`.

**Launch.** `helm-launch` does not exist and is not designed here. The only thing this design
must guarantee is that it does not paint the project into a corner: the report contains **no**
`READY_TO_LAUNCH` and no readiness vocabulary of any kind, and a successful comparison
confers no launch permission. A future launcher will need specific binding states, an
execution policy, runtime authority and sandbox or recovery state, all of which live outside
`helm-bind`. Because the report is an exact artifact with a digest, a future launcher can
require "a binding report with digest X exhibiting states Y" without `helm-bind` ever knowing
what a launch is.

## 19. Falsifiers

Reject any design or implementation that permits:

1. omitted unobservable claims yielding a global success;
2. an observation for spec A bound to spec B;
3. an artifact from observation plan A paired with plan B;
4. arbitrary caller-supplied expected hashes;
5. a runtime archive compared to an installed loader as equivalent identity;
6. an entry-point content match treated as a successful launch;
7. verification-definition identity treated as verification success;
8. absence collapsed into I/O failure;
9. a failed observation collapsed into a mismatch;
10. an unsupported comparator collapsed into satisfaction;
11. any target-ID naming heuristic;
12. nondeterministic output;
13. any I/O or execution inside the binder;
14. an identity cycle;
15. a caller-asserted prefix root presented as a proven prefix.

Items 1, 4, 5, 10 and 11 are structurally impossible in the recommended design rather than
merely tested: the coverage theorem of section 3.4, the absence of any expected-value field
in the `BindingPlan`, the closed typed claim vocabulary of section 9, the two-axis report of
section 3.3, and the mandatory explicit mapping of section 8.

## 20. A0 paper mapping — from committed artifacts only

Paper exercise using the committed fixture `crates/helm-app-spec/tests/fixtures/a0-7zip.json`
and its `provenance.json`. **No lab was accessed, no observation was produced, and
A0-7ZIP remains experimental FAIL.**

| A0 desired claim | Bindable today? | Notes |
|---|---|---|
| `application.source` size 1661239, sha256 `0859c5…` | yes, with a mapping to a `regular_file_sha256` target over the retained installer body | establishes that body's identity only; not that it was installed |
| four `runtime.artifacts` roles `wine-devel-archive`, `wine-devel-amd64-archive`, `winehq-devel-archive`, `wine-devel-i386-archive` | yes, each with its own mapping | establishes archive-body identity; says nothing about the loader that actually ran |
| `runtime.family = wine` | **no** | `UnsupportedBinding` |
| `environment.windows_architecture = win64` | **no** | `UnsupportedBinding` |
| `environment.prefix.role = dedicated` | **no** | `UnsupportedBinding`; dedication is intent |
| `environment.disabled_dlls = [mscoree, mshtml]` | **no** | `UnsupportedBinding`; effective DLL policy is not a filesystem fact |
| `entry_point.path = drive_c/Program Files/7-Zip/7zFM.exe` | yes for presence and kind, **relative to a caller-asserted prefix root** | the path is expressible as an observation target path under section 1.3 |
| `entry_point.sha256` | **absent from the A0 spec** | `DesiredValueUnspecified`; A0 declares no entry-point digest |
| three `verification.definitions` roles `protocol`, `oracle`, `content-fixture` | yes, definition-body identity only | never verification executed, passed or fresh |
| `application.id = 7zip-x64`, `version = 26.03` | not compared | labels and human metadata |

So of A0's claim classes, at most **9 of roughly 15** could be bound today, and the four
environment and runtime-family claims remain `UnsupportedBinding` no matter what is observed.
An A0 binding report would read `NoClaimContradicted` with a conspicuously incomplete
coverage summary — which is the honest description of what HELM can currently establish about
A0, and is precisely why the two-axis result is the right shape.

## 21. Test and falsification strategy

**No new system experiment is warranted.** `helm-bind` performs no syscall, opens nothing and
depends on no kernel, filesystem or hardware behaviour; there is no real-world behaviour a VM
could settle that a property test cannot. The environment-dependent facts this module relies
on — `openat2` resolution semantics, procfs reopen, special-file classification — belong to
`helm-observe` and are already evidenced by OBS-FS-01 and by the independent syscall
regression. Inventing an experiment because the previous module needed one would be
cargo-culting. The falsification mechanism here is adversarial and property testing.

Planned suites, all cross-platform, no VM:

1. **Identity refusals.** Subject mismatch, plan mismatch, a valid artifact paired with a
   foreign plan, a mapping whose declared subject differs from the spec.
2. **Every desired claim class**, one test each, over both fixtures.
3. **Every `TargetOutcome` variant**, including a synthetic unknown-variant path exercised
   through the catch-all, asserting `ObservationNotInterpretable`.
4. **Match, mismatch, size-differs-digest-agrees, digest-differs-size-agrees.**
5. **Absence, observer rejection, observer failure, budget omission**, each producing its own
   distinct state and none collapsing into another.
6. **Unsupported binding**, asserting the coverage theorem as a property over every accepted
   spec: `unsupported_count >= 4`, holding even when `disabled_dlls` is empty, with the four
   unavoidable classes named individually.
7. **Missing mapping, duplicate mapping, unknown target, unknown root, unknown role,
   incompatible observable.**
8. **Entry-point path binding**, including a plan whose target path differs from the desired
   path while the digest matches — the section 7 attack — asserting `Mismatch`.
9. **Determinism**: identical inputs give identical bytes and digest; claim order follows spec
   declaration order; no map iteration.
10. **Vocabulary**: the serialized report contains none of `PASS`, `FAIL`, `SATISFIED`,
    `UNSATISFIED`, `COMPATIBLE`, `READY`, `INSTALLED`, `snapshot`, `verified`, and no `/`.
11. **Verdict truth table**, property-tested over generated combinations of claim states,
    asserting the section 12 algebra exactly.
12. **Seeded adversarial corpus** over `BindingPlan` bytes, with a reviewer-chosen seed,
    asserting no panic and deterministic diagnostics.
13. **Neutral fixtures**: `synthetic-notes.json` is already committed, is a fictional second
    application and carries an `entry_point.sha256`, so nothing needs 7-Zip.

## 22. Crate and API sketch

One crate, `crates/helm-bind`, library only, `publish = false`, no `cfg`, no OS backend.

```text
crates/helm-bind/
  Cargo.toml            serde, serde_json, sha2, helm-app-spec, helm-observe
  src/lib.rs            re-exports, module docs, the bind() entry point
  src/plan.rs           BindingPlan document: closed schema, strict duplicate-key
                        rejection, ceilings, exact-byte identity
  src/claim.rs          the closed claim vocabulary and its desired-field/observable
                        compatibility rules
  src/model.rs          BindingReport, claim outcomes, counts, deterministic serializer
  src/error.rs          BindingPlanError codes and BindingRefusal
```

```rust
pub fn parse_binding_plan(bytes: &[u8]) -> Result<ValidatedBindingPlan, BindingPlanErrors>;

pub fn bind(
    spec: &helm_app_spec::ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    plan: &helm_observe::ValidatedPlan,
    observation: &helm_observe::ObservationArtifact,
) -> Result<BindingReport, BindingRefusal>;

impl BindingReport {
    pub fn exact_bytes(&self) -> &[u8];
    pub fn sha256(&self) -> Digest;
    pub fn contradiction(&self) -> Contradiction;
    pub fn claims(&self) -> &[ClaimBinding];
    pub fn coverage(&self) -> &Coverage;
}
```

No public constructor for a report, no `Deserialize` for any validated type, no expected-value
input anywhere.

**Estimated size.** Product ≈ 850–1,100 lines: roughly 300 for the plan document, 200 for the
claim vocabulary and compatibility rules, 200 for the comparison, 260 for the model and
serializer, 120 for errors. Tests ≈ 900–1,400 lines across the thirteen suites. For scale,
`helm-observe` is about 1,850 product lines and 1,800 test lines, and about a third of its
product code is the Linux backend that `helm-bind` does not have.

Dependencies: `serde`, `serde_json`, `sha2`, `helm-app-spec`, `helm-observe`. No `rustix`, no
new third-party crate, no utility crate, no adapter crate.

## 23. Owner decisions — all eight decided on 2026-09-09

The owner reviewed this report and **accepted the core architecture subject to bounded
pre-implementation corrections**, which are applied above and marked in place. Acceptance is
recorded in [ADR-0023](../adr/ADR-0023-binding-authority.md), now **Accepted**.

| # | Question | Owner decision |
|---|---|---|
| 1 | the two-axis result, and no global satisfaction verdict | **accepted**; no satisfaction, compatibility or readiness verdict in 0.1 |
| 2 | the `BindingPlan` as an inert identity-bearing document with a closed five-value vocabulary | **accepted** unchanged |
| 3 | should `Absent` on `EntryPointPresence` be a contradiction | **no**, as recommended. It stays the explicit `Absent` state; `Absent`, rejection, failure, omission and unsupported binding are **never** promoted to contradiction in 0.1 |
| 4 | recording an unverifiable caller prefix assertion | **accepted, hardened**: the field is renamed `asserted_prefix_root_id`, its presence is conditional on an entry-point mapping, and every mapped entry-point target must sit under it or the binding is refused |
| 5 | direct dependencies on both HELM crates, with the `sha2` consequence | **accepted**; the unification effect stays classified as performance and build composition only, hashing is not changed, and separate package builds remain tested |
| 6 | report identity and the four-digest acyclic graph | **accepted** unchanged |
| 7 | is ADR-0023 the right vehicle | **yes**; ADR-0021 and ADR-0022 are **not** amended to absorb this responsibility |
| 8 | refusals produce no artifact | **accepted**: no report bytes, no claim outcomes, no report digest. A later evidence or reporting layer may record that an attempt was refused; the binder must not manufacture a comparison artifact when no valid comparison occurred |

Four further corrections were directed and are applied above: the entry-point observable rule
of section 9, the entry-point path binding on **both** entry claims in section 7, the claim
universe of section 4.1, and the corrected counts in sections 3.4 and 17.

**Still not authorised.** Acceptance settles the architecture only. It does **not** authorise
implementation, which needs a separate owner instruction. No crate was created, no product API
was changed, no comparison code was written, no lab was booted and A0 was not rerun.
