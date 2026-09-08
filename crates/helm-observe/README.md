# helm-observe (experimental 0.1)

Explicit-target observation of **actual** Linux filesystem facts, under Accepted
[ADR-0022](../../docs/adr/ADR-0022-observation-authority.md).

It answers exactly one question: **what actually exists at explicitly authorised
targets?**

It does **not** answer what should exist, whether desired state is satisfied, whether an
application is compatible, whether a runtime is correct, whether installation succeeded,
whether a prefix is dedicated, whether DLL policy is effective, or whether launch is safe.
Comparison belongs to a future `helm-bind`; execution to a future `helm-launch`. Neither
exists, and neither is authorised.

**Status: implemented on a product branch, awaiting independent review. Not owner-merged
and not a release.**

## What it will not do

No process execution, no Wine invocation, no registry semantics, no PATH/HOME/cwd/system
lookup, no installation discovery, no recursive inventory, no directory enumeration, no
file search, no network, and no interpretation of a `helm-app-spec` requirement. The crate
depends on neither HELM crate; artifact identities, not Cargo types, connect the modules.
Observation code performs no filesystem writes.

## Supported cohort

Linux x86_64 on **local ext4**, anchored by the
[OBS-FS-01](../../docs/experiments/OBS-FS-01-EXECUTION-REPORT.md) evidence cohort. Root
admission refuses anything else rather than degrading. Other filesystems, architectures
and kernels are not claimed and need their own evidence.

Pure plan and model code compiles on any platform so the contract can be tested and
reviewed anywhere. The observation backend and the capability types are gated to Linux;
there are no fake Windows observation semantics.

### Descendant bind mounts are excluded

Target paths whose correctness claim depends specifically on rejecting a **bind mount
created as a descendant of an authorised ext4 root** are outside the empirically validated
0.1 cohort. `RESOLVE_NO_XDEV` remains mandatory and such crossings are conservatively
rejected, but no validation of that scenario is claimed: OBS-FS-01 recorded it BLOCKED.
The crate never scans a root trying to prove absence of descendant mounts.

## Authority

```text
untrusted plan bytes  --parse_plan-->  ValidatedPlan            no authority at all
ValidatedPlan + opened roots + procfs  --authorize-->  AuthorizedScope
AuthorizedScope  --observe-->  ObservationArtifact
```

Validating a plan grants nothing. Authority arrives only when a trusted caller hands over
already-open descriptors together with that exact plan.

- **Exact plan binding.** The scope binds the complete plan SHA-256 over all 32 bytes, and
  the artifact carries that identity. Identity is over exact bytes: reformatting or
  reordering fields yields a different plan.
- **Root set binding.** Roots are matched by logical ID, set and count — never by vector
  position. A missing, extra, duplicate or substituted root is refused.
- **Plan substitution.** Impossible by construction: `authorize` consumes the
  `ValidatedPlan`, and `observe` takes only `&AuthorizedScope`, so a second plan cannot be
  supplied. The target set is fixed at authorisation time.

A numeric descriptor never appears in a plan and can never become authority: the closed
schema has no such field, and capabilities are owned descriptors passed in Rust.

## Mechanism

Constrained `openat2` with `O_PATH | O_NOFOLLOW | O_CLOEXEC` and
`BENEATH | NO_SYMLINKS | NO_MAGICLINKS | NO_XDEV` pins the target. `O_PATH` means the
object is *not* opened, so classification precedes any data access — a FIFO cannot block
and a device driver is never entered. `statx` on the pinned descriptor classifies it.
Per-file and aggregate budgets are enforced **before** any reopen. Only a descriptor
already classified as a regular file can reach the procfs reopen, which re-verifies device,
inode and type against the pin before a single byte is read.

Flags are never weakened and there is no `openat` fallback for target resolution. If
`openat2` is unavailable the outcome is explicit, never a degraded resolver. The procfs
capability is supplied by the caller; the crate never opens `/proc/self/fd` itself, never
searches for another procfs mount, and never falls back to reopening a pathname.

**Symlinks.** A trailing symlink may be pinned and is rejected at classification with its
kind established; a non-final symlink is rejected at resolution and yields no descriptor.
No target symlink destination is ever followed.

**Scope violations.** `EXDEV` covers both a mount crossing and a root escape, so one
`scope_violation` outcome is reported rather than inventing an attribution the kernel does
not provide.

## Consistency

Sequential per-object capture. **No atomic file or environment snapshot.** A digest
identifies the bytes actually supplied through the retained descriptor during that observed
read sequence, subject to this consistency model. Metadata is sampled before and after the
stream; a detected size, link-count, mtime or ctime change, or a short or growing stream,
yields `changed_during_read` rather than a complete digest. Undetected concurrent mutation
remains a real limitation, and no output claims otherwise.

A permission denial, unsafe object, wrong kind, scope violation, procfs failure, I/O error,
unsupported platform, budget exhaustion or detected mutation is **never** reported as
absence. `absent` means constrained authorised lookup legitimately found nothing at that
attempt.

## Privacy

Only declared targets are touched. Unlisted siblings are never opened or enumerated.
Diagnostics and artifacts use logical root and target IDs and bounded fixed codes; no host
absolute path, hostname, timestamp, signature or random identifier enters an artifact.

## Build composition

`sha2` is used with its default backend here. The repository already records a
`sha2/force-soft` feature-unification limitation: when `helm-app-spec` participates in the
same Cargo graph, feature unification selects the software backend for every crate in that
graph, including this one. That is a documented performance and build-composition
limitation, not a semantic one — digests are identical either way. This crate does not
change that dependency arrangement, and no hashing utility crate is introduced.

## Not stabilised

The schema, API, numeric limits and internal layout are experimental. `publish = false`.
