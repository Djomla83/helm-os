# helm-observe 0.1 implementation review (author)

**Disposition: READY_FOR_INDEPENDENT_REVIEW.** Implemented on `product/helm-observe`.
Not owner-merged, not released, and not independently reviewed.\
**Date:** 2026-09-08. **Authoritative base:** `0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b`.\
**Authority:** Accepted [ADR-0022](../adr/ADR-0022-observation-authority.md), bounded 0.1 scope.

This is the author's own review. It maps the implementation back to every ADR-0022
requirement and to each merge-blocking obligation, and records what was **not** done.

## 1. Evidence and authority chain

| Role | Commit |
|---|---|
| Amendment 1, operative expectations | `63ac6796296894945dff520423cb1a93351e8524` |
| OBS-FS-01 execution definition, frozen pre-trial | `2496f68cccbf70ac04288992a9945cb34f046b98` |
| OBS-FS-01 PASS evidence | `1bd9c0a75abb0c4acccae352960f42ead96fcacb` |
| Bounded ADR-0022 acceptance | `0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b` |

The OBS-FS-01 spike under [docs/experiments/obs-fs-01/](../experiments/obs-fs-01/) was
treated as **evidence and reference**, not as code to copy. The product re-implements the
mechanism in safe Rust; it shares no source with the disposable C spike.

## 2. Crate boundary

One new workspace member, `crates/helm-observe`, library only, `publish = false`.

It depends on **neither** `helm-app-spec` nor `helm-evidence`, and no shared utility crate
was created. Direct dependencies are `serde`, `serde_json`, `sha2`, and on Linux `rustix`
with only the `std` and `fs` features. All are versions already present in `Cargo.lock`;
the lock gained the new member and its rustix edges only.

`#![forbid(unsafe_code)]` is inherited from the workspace and **holds**. The accepted
mechanism needed `openat2` with `open_how`, `statx` with `AT_EMPTY_PATH`, `openat` and
`fstatfs`, plus the `PROC_SUPER_MAGIC` constant. `rustix` 1.1.4 exposes all of them as safe
wrappers returning `OwnedFd`, so no `libc` call, no raw syscall and no unsafe block was
introduced, and there was no need to stop for owner review on that point.

## 3. Public API

```rust
pub fn parse_plan(bytes: &[u8]) -> Result<ValidatedPlan, PlanErrors>;

// Linux only
pub fn root_from_fd(id: &str, fd: OwnedFd) -> Result<RootCapability, AdmissionError>;
pub fn proc_fd_from_trusted_current_process(fd: OwnedFd)
    -> Result<ProcFdCapability, AdmissionError>;
pub fn authorize(plan: ValidatedPlan, roots: Vec<RootCapability>, reopen: ProcFdCapability)
    -> Result<AuthorizedScope, AdmissionErrors>;
pub fn observe(scope: &AuthorizedScope) -> ObservationArtifact;
```

There is deliberately no path-based root constructor, no `Deserialize` for a validated
plan, an authorised scope or an artifact, and no public constructor for a successful
observation.

## 4. ObservationPlan 0.1 schema

Closed JSON object; every unknown field is rejected.

| Field | Meaning |
|---|---|
| `schema` | exactly `helm-observation-plan` |
| `version` | exactly `0.1` |
| `subject_spec_sha256` | 64 lowercase hex characters, opaque context, never parsed or compared |
| `roots[].id` | unique logical root ID; no locator, path or descriptor |
| `targets[].id` | unique logical target ID, no built-in role semantics |
| `targets[].root` | one declared root ID |
| `targets[].path` | exact inert relative spelling; omitted means the root directory itself |
| `targets[].observable` | `directory_metadata` or `regular_file_sha256` |

The plan carries **no** expected digest, expected size, runtime family, DLL value,
satisfaction criterion, numeric descriptor or absolute host capability. Identifiers are
1–80 bytes of lowercase ASCII alphanumerics, dot, underscore and hyphen, starting
alphanumeric.

Enforced ceilings: ≤ 128 KiB document, ≤ 8 roots, ≤ 64 targets, ≤ 1024 path bytes,
≤ 32 components, ≤ 255 bytes per component, ≤ 8 JSON nesting levels, ≤ 64 retained
findings. These match the reviewed values; none was changed.

Rejected: unknown schema or version, unknown fields, duplicate decoded JSON keys,
duplicate IDs, unknown root references, empty plans, unknown observables, absolute paths,
`.` and `..` components, empty and repeated separators, backslashes, NUL, non-portable
bytes, leading spaces, trailing spaces or dots, and every limit violation.

Exact input bytes are retained and `sha256()` is computed over them. No canonicalisation:
whitespace or field-order changes produce a different plan identity.

**Purity.** `parse_plan` performs no filesystem, process, network, environment, registry,
package or Wine lookup on any path, including error paths. It is reachable and testable on
non-Linux hosts, which is how the contract suite runs cross-platform.

**Duplicate keys.** `serde_json` silently keeps the last duplicate, which would let two
documents with different meaning validate identically. A raw structural scan therefore
detects duplicate keys and over-deep nesting *before* any map insertion.

## 5. Merge-blocking obligations

### A — exact plan identity binding

`authorize` stores `plan.sha256()` in `AuthorizedScope::authorized_plan_sha256`, and
`observe` writes exactly that value into the record and the serialized artifact. `Digest`
is a 32-byte array compared in full; there is no prefix or pointer comparison anywhere.

Tested: whitespace difference changes identity; field reordering changes identity; each of
the 32 bytes participates in equality; the artifact contains the authorised identity.

### B — root set binding

`authorize` matches supplied capabilities against `plan.root_ids()` **by ID**. It rejects a
missing root, an extra root, a duplicate root ID, and a substituted ID (which surfaces as
both an extra and a missing root). Root order is irrelevant by design, because targets
reference roots by unique ID; a test authorises the same plan with the capability vector
reversed and gets an equivalent result.

No plan field can become a descriptor: the closed schema has no descriptor-shaped field,
and a `root_fd`-style key is rejected as an unknown field with no I/O at all.

### C — plan substitution

**Impossible by construction.** `authorize` takes `plan: ValidatedPlan` **by value** and
moves it into the scope; `observe` takes only `&AuthorizedScope`. The signature offers no
second plan to substitute, so "authorise plan A, observe plan B" cannot be written against
this API. `AuthorizedScope::plan()` returns a borrow only, and `ValidatedPlan` has no
public constructor other than `parse_plan`, so the retained plan cannot be replaced or
mutated. The target set is consequently fixed at authorisation time.

A runtime test documents the observable half of the invariant: the artifact carries plan
A's identity and only plan A's target IDs, and never plan B's digest.

## 6. Requirement map

| ADR-0022 requirement | Where |
|---|---|
| Explicit bounded plan | `plan.rs`, closed schema and ceilings |
| No ambient scanning, no recursive inventory | No `readdir`/`getdents` call exists anywhere in the crate; `directory_metadata` stops at `statx` |
| Caller supplies opened roots | `root_from_fd`, no path constructor |
| Validation alone grants no authority | `ValidatedPlan` holds no descriptor; authority begins at `authorize` |
| Constrained `openat2`/`O_PATH` resolution | `linux::resolve`, `RESOLVE` constant |
| Classify pinned object before data access | `observe_one` classifies, then only the `Regular` arm calls `stream_regular` |
| Amended symlink and special-file rejection | `reject_for`, plus trailing/non-final split |
| Admission-validated procfs reopen | `linux::admit_procfs`, `linux::reopen_and_stream` |
| No pathname fallback | The reopen only ever names a descriptor number under the supplied capability |
| No desired/observed comparison | No expected value exists in any type |
| No execution | No process, command or network API is linked |
| No snapshot guarantee | `changed_during_read`, and the artifact says `sequential_objects` |

## 7. Result model and artifact identity

Outcomes are `ObservedFile`, `ObservedDirectory`, `Absent`, `Rejected`, `Failed` and
`NotObserved(budget)`. Rejections are `symlink_forbidden`, `special_file`, `wrong_kind`
and `scope_violation`. Failures are `permission_denied`, `io_failure`, `resolution_race`,
`changed_during_read`, `reopen_unavailable` and `unsupported_platform`.

There is no `PASS`, `FAIL`, `SATISFIED`, `UNSATISFIED`, `COMPATIBLE`,
`INSTALLED_CORRECTLY` or `READY_TO_LAUNCH`, and a test asserts those strings never appear
in a serialized artifact.

`ObservationArtifact::exact_bytes()` is produced by a fixed deterministic serializer within
schema 0.1 — not a canonicalisation standard — and `sha256()` hashes exactly those bytes.
The artifact contains no timestamp, signature, random identifier, hostname, absolute path
or machine inventory, and is bounded by a 256 KiB ceiling. `ObservationArtifact::new` is
crate-private, so an ordinary caller cannot manufacture an observation.

`EXDEV` is deliberately reported as a single `scope_violation`, because the kernel returns
it both for a mount crossing and for a root escape and the observer must not invent
attribution it cannot have.

## 8. Consistency and budgets

Per-file ceiling 512 MiB; aggregate 1 GiB including partial reads; fixed 64 KiB streaming
buffer, never allocated from an observed length. Budgets are checked **before** any data
reopen, so an over-limit file is rejected on metadata alone with zero reopens and zero
reads. Actual returned bytes are charged. A per-file limit does not exhaust the batch; a
genuine aggregate exhaustion marks later targets `not_attempted_total_limit`, and every
declared target always receives an explicit outcome.

The sequence is pre-read metadata, bounded stream, explicit EOF and growth check via one
sentinel byte, post-read metadata. Any size, link-count, mtime or ctime change, or a short
or growing stream, yields `changed_during_read` with the partial count and **no** complete
digest.

## 9. Tests

**Pure contract, 12 tests, cross-platform.** Exact-byte identity including whitespace and
field-order differences; every digest byte participating; duplicate decoded keys; unknown
schema, version and fields including descriptor-shaped ones; identifier and digest grammar;
nine unsafe path-grammar cases plus NUL; all resource ceilings; missing root references and
duplicate IDs; deterministic and private error ordering; malformed and deep JSON without
panic; and a 512-case seeded mutation corpus asserting determinism, with fixed seeds so any
failure reproduces.

**Linux, 13 tests.** Obligations A, B and C; artifact determinism and forbidden vocabulary;
root admission rejecting a non-directory; procfs decoy and foreign-namespace rejection;
regular and empty files against independent digests; absence distinguished from wrong kind,
`ENOTDIR` and permission denial on both leaf and parent; all three trailing symlink
variants plus a non-final one; FIFO and Unix socket; hardlink and sparse; directory
metadata without enumeration; unlisted-sibling privacy with a distinctive marker; budgets;
retained root descriptor across pathname replacement; and the ext4 cohort check.

Fixtures are synthetic and neutrally named — a "notes" application, never 7-Zip. Nothing
touches A0.

**Special-file regression.** The FIFO test is itself the traceable proof: a writerless FIFO
would block a direct `O_RDONLY` open indefinitely, so the test completing promptly with
`special_file` demonstrates that classification happened on an `O_PATH` pin and the reopen
was never called. No mocking framework was introduced. Independent review is expected to
add syscall-level adversarial checks.

## 10. Self-review

Traced by hand from `parse_plan` through `authorize`, `resolve`, `describe`,
`stream_regular` and the artifact:

| Question | Answer |
|---|---|
| Can untrusted JSON cause a descriptor to be opened? | **No.** `parse_plan` has no I/O call in its module |
| Can validation alone access a root? | **No.** `ValidatedPlan` holds no capability |
| Can a symlink reach data reopen? | **No.** Only the `Regular` arm calls `stream_regular` |
| Can a FIFO, socket or device reach data reopen? | **No.** Same single arm |
| Can plan B execute under plan A's scope? | **No.** Not expressible: `authorize` consumes the plan |
| Can an unlisted sibling be enumerated? | **No.** The crate contains no directory-reading call |
| Can procfs failure trigger pathname fallback? | **No.** Failure returns `reopen_unavailable` |
| Can permission denial become absence? | **No.** `EACCES`/`EPERM` map to `permission_denied`; only `ENOENT` from constrained lookup is `absent` |
| Can a changed file produce an unjustified stable claim? | **No.** Any detected change or length mismatch yields `changed_during_read` |
| Can an absolute host path enter the artifact? | **No.** Only logical IDs and fixed codes are serialized, and a test asserts no `/` appears |

No question answered YES, so no blocker was found by this pass.

## 11. Known limitations

- **Descendant bind mounts** stay outside the validated cohort; conservatively rejected,
  never claimed.
- **Undetected concurrent mutation** remains possible; metadata sampling is best-effort
  and inode timestamps are coarse. No snapshot is claimed.
- **`STATX_MNT_ID`** is recorded when the kernel supplies it and is not treated as durable;
  a plain mount ID can be reused after unmount.
- **Combined `sha2` graph.** With `helm-app-spec` in the same graph, feature unification
  selects the software backend for this crate too. Documented, semantically neutral, and
  deliberately not "fixed" here.
- **No independent review yet**, and no syscall-level adversarial verification of the Rust
  implementation. OBS-FS-01 evidences the mechanism, not this code.
- Schema, API and limits are experimental and unstabilised.

## 12. Not done, deliberately

No `helm-bind`, no `helm-launch`, no A0 access or rerun, no Wine or 7-Zip, no broad
scanning, no new architecture decision, no descendant-bind-mount claim, no merge to main,
no change to `helm-app-spec` or `helm-evidence` sources or features, and no dependency
feature edit to optimise this crate. **A0-7ZIP remains experimental FAIL.**
