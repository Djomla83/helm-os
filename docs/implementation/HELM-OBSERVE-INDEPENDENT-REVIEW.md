# Independent review: helm-observe observation authority and OBS-FS-01 definition

**Scope:** review only. No experiment was executed, no VM was booted, no observer code
was written, and ADR-0022 was not accepted.\
**Date:** 2026-09-08.\
**Reviewed candidate:** `6d9e4d8c0dacb09c7f4833a4fda6c4ba50d6e9dc`
(`docs/helm-observe-architecture`).\
**Authoritative base:** `e69abdfc55646dff6073befaeb843a18d6c7975f`.\
**Disposition:** **NEEDS_EXPECTATION_FIXES.** The architectural boundary is coherent;
the preregistered experiment is not yet closed against post-hoc goalpost movement.

This review checks the [architecture proposal](../research/HELM-OBSERVE-ARCHITECTURE.md)
and [ADR-0022](../adr/ADR-0022-observation-authority.md) as an *experiment definition*.
It does not review an implementation, because none exists.

## 1. Reconstructed state, independently verified

| Check | Command | Actual result |
|---|---|---|
| Remote refs fetched | `git fetch --all --prune` | Succeeded; no ref moved |
| Main unchanged | `git rev-parse origin/main main` | Both `e69abdfc55646dff6073befaeb843a18d6c7975f`, equal to the instructed base |
| Candidate ancestry | `git merge-base --is-ancestor e69abdf 6d9e4d8` | True; `6d9e4d8^` is exactly `e69abdf`, so the candidate is a direct child of main |
| Documentation-only | `git diff --name-only e69abdf 6d9e4d8` | 4 files, all Markdown: DECISIONS, PROJECT_STATE, ADR-0022, the architecture report. No crate, manifest, lockfile, fixture, evidence or CI change |
| ADR-0022 status | source line | `**Status:** Proposed`; approver "Not assigned"; acceptance date "Not accepted" |
| OBS-FS-01 status | report section 14 | `**OBS-FS-01: NOT_RUN; requires a separate owner authorisation.**` |
| Working tree | `git status --porcelain` | Empty before this review branch was created |

Read in full: [AGENTS.md](../../AGENTS.md), [PROJECT_STATE](../PROJECT_STATE.md),
[DECISIONS](../DECISIONS.md), [ADR-0021](../adr/ADR-0021-second-product-module.md),
[ADR-0022](../adr/ADR-0022-observation-authority.md), the complete
[architecture report](../research/HELM-OBSERVE-ARCHITECTURE.md), both crate READMEs
and product sources, all four existing implementation reviews, the
[A0 report](../experiments/EXP-009-APP-BASELINE-REPORT.md),
[EXP-009 Gate 0](../experiments/EXP-009-GATE0-REPORT.md), and the cited
[Linux syscall-boundary probe](../../tools/helm_evidence_linux_probe.py).

Two report claims were independently reproduced rather than accepted:

- The A0 app-spec fixture digest asserted in report section 13 recomputes to
  `b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e`. Matches.
- The lab facts asserted in report sections 13 and 14 (Ubuntu 24.04.4, Linux
  `7.0.0-31-generic`, ext4, UID/GID 1001 `helmlab` with no supplementary groups,
  Rust 1.95.0, GCC 13.3.0) match the preserved
  [A0 report](../experiments/EXP-009-APP-BASELINE-REPORT.md) and
  [evidence independent review](HELM-EVIDENCE-INDEPENDENT-REVIEW.md). Match.

Nothing was modified before this review pass completed.

## 2. Findings summary

| # | Severity | Location | Subject | Invalidates |
|---|---|---|---|---|
| B1 | **BLOCKER** | report section 14 | No mandatory/optional case partition, no experiment-level verdict rule, undefined "sufficient coverage" success prerequisite, undefined repetition scoring | Experiment only |
| I1 | IMPORTANT | report sections 7 and 9 | `mount_crossing` and `symlink_forbidden` are not derivable from the mandated `openat2` flag set; `EXDEV` and `ELOOP` are each ambiguous | Experiment only |
| I2 | IMPORTANT | report section 14 | Mechanism choice is asserted, not falsified: no direct-open comparison arm exists, so the experiment cannot show why procfs reopening is required | Experiment only |
| I3 | IMPORTANT | report section 15 | `observe(plan, scope)` re-admits a caller-supplied plan; plan-to-scope binding is stated in prose but never as a required digest equality, and is untested | Architecture text |
| I4 | IMPORTANT | report sections 3 and 14 | "Reject synthetic fake proc directory" is preregistered with no stated acceptance test, so the case has no pass criterion | Experiment only |
| I5 | IMPORTANT | report section 7 step 5 versus section 16 | Ordering ambiguity: an over-limit file may be data-opened before the length/budget check rejects it | Experiment only |
| I6 | IMPORTANT | report section 14 | "In-place same-length overwrite: rejected **where samples detect it**" is unfalsifiable as written; every outcome passes | Experiment only |
| I7 | IMPORTANT | report section 14 | No instrumentation artifact is declared authoritative per security property; the cited probe cannot currently observe `open_how.resolve`, reads, or non-injected syscalls | Experiment only |
| I8 | IMPORTANT | report section 14 | The mandatory mount case is very likely BLOCKED on the chosen lab for a reason this repository already documents; no preflight, fallback arm or residual-claim statement exists | Experiment only |
| M1-M8 | MINOR | various | Documentation accuracy and preflight completeness; see section 9 | Neither |

Section anchors for the reviewed locations:
[3](../research/HELM-OBSERVE-ARCHITECTURE.md#3-plan-and-exact-authority),
[7](../research/HELM-OBSERVE-ARCHITECTURE.md#7-linux-filesystem-design-and-reuse-assessment),
[9](../research/HELM-OBSERVE-ARCHITECTURE.md#9-minimal-immutable-record-and-error-taxonomy),
[14](../research/HELM-OBSERVE-ARCHITECTURE.md#14-proposed-pre-implementation-falsification-experiment),
[15](../research/HELM-OBSERVE-ARCHITECTURE.md#15-candidate-rust-api-not-implementation),
[16](../research/HELM-OBSERVE-ARCHITECTURE.md#16-proposed-resource-ceilings).

No finding invalidates the architecture. B1 and I1 to I8 are experiment-definition or
architecture-text corrections; all are addressed by the corrected preregistration in
sections 10 to 14 of this document.

## 3. Core separation: NO FINDING

The proposal preserves the required separation. This was checked by looking for
comparison semantics, not by accepting the report's own claim.

| Surface | Checked for | Result |
|---|---|---|
| Plan fields | Expected hashes, sizes, DLL values, runtime family, pass criteria | None present. Fields are `schema`, `version`, `subject_spec_sha256`, `roots[].id`, `targets[].{id,root,path,observable}` |
| `subject_spec_sha256` | Whether it is an expected value | It is the identity of the *desired-state document*, carried opaquely and never compared to anything observed. The report states the observer cannot detect a wrong subject. This is provenance labelling, not an expectation |
| `observable` vocabulary | Hidden comparison | `directory_metadata`, `regular_file_sha256`. Both name a measurement, not a criterion |
| Result vocabulary | SATISFIED/PASS/COMPATIBLE-style conclusions | Absent, and explicitly forbidden: "No PASS, SATISFIED, COMPATIBLE, INSTALLED_CORRECTLY or global compatibility boolean" |
| Target operations | Recursion, globbing, search, fallback, expansion | All explicitly rejected; declaration order retained; empty plans rejected |
| A0 mapping | Historical digests leaking into the plan as expectations | The mapping is paper-only; the plan carries no digests, and the report requires a later caller to use the identity of the bytes it actually supplies |
| Error and status names | Comparative naming | `absent`, `observed_file`, `observed_directory`, `symlink_forbidden`, `special_file`, `wrong_kind`, `mount_crossing`, `permission_denied`, `io_failure`, `resolution_race`, `changed_during_read`, `reopen_unavailable`, `file_limit`, `total_limit`, `not_attempted_total_limit` |

The identity-reference graph in report section 10 was checked for cycles by hand and is
acyclic. `helm-observe` depends on neither HELM crate. The desired/actual/binder/launcher
split is preserved: the observer never receives an expected value, and comparison is
deferred to a component that does not exist.

One vocabulary note is recorded as M7 below: `wrong_kind` is the only outcome whose name
reads comparatively. Its justification in the report — that a requested observable
constrains *safe measurement mechanics*, not a desired outcome — is correct and
sufficient, but is stated in report section 3 and not repeated where the code is defined.

## 4. Direct open versus O_PATH plus procfs reopen

The instruction was to compare strategies on their merits rather than accept the
report's proposal. The comparison below is grounded in primary kernel documentation.

| Property | A: `O_PATH` pin, `statx`, procfs reopen | B: direct `openat2` with `O_RDONLY`, then `fstat` | C: any other unprivileged Linux-native route |
|---|---|---|---|
| Symlink race | `RESOLVE_NO_SYMLINKS` rejects before any object is opened | Same resolution guarantee, but the leaf is opened before it is classified | Same as A or B |
| Parent replacement | Resolution is `RESOLVE_BENEATH` from a held root fd; a replaced parent yields rejection or resolves under the retained root | Identical | Identical |
| Mount crossing | `RESOLVE_NO_XDEV` rejects, "including all bind mounts" | Identical | Identical |
| Magic links | `RESOLVE_NO_SYMLINKS` "implies `RESOLVE_NO_MAGICLINKS`" | Identical for target lookup | Identical |
| FIFO blocking | `O_PATH` never blocks: "The file itself is not opened" | `O_RDONLY` on a FIFO blocks until a writer opens it. With `O_NONBLOCK` it returns immediately, but the FIFO **has been data-opened**, and that open unblocks any writer waiting in `open` for writing: an externally visible side effect on another process | — |
| Socket and device open side effects | None: no driver open method is invoked | `open` on a Unix socket fails with `ENXIO`, but a character or block device node reached this way enters the driver's open path. Some device classes have real side effects on open alone | — |
| Permission semantics | `O_PATH` pinning needs only search permission along the path; the procfs reopen re-checks access to the underlying file and may legitimately fail where the pin succeeded | One check at open time | — |
| Object pinning | The `O_PATH` fd pins the inode; classification and reopen refer to the same object | `fstat` describes an object that was already opened, so classification is post hoc | — |
| Hash exactly the classified object | Yes: the reopen resolves the pinned descriptor, not the pathname | Only by re-verifying `st_dev` and `st_ino` after an open that already happened | — |
| Kernel and filesystem assumptions | `openat2` (Linux 5.6), `STATX_MNT_ID` (5.8), a mounted procfs for the current process | `openat2` only | — |

**Conclusion, stated explicitly as instructed: strategy B cannot satisfy the intended
"no special-file data-open" policy.** Linux has no open flag that restricts a successful
open to regular files; `O_DIRECTORY` restricts to directories and there is no counterpart
for `S_IFREG`. Classification must therefore happen either before the open — which
requires `O_PATH`, the only documented way to obtain a handle where "the file itself is
not opened" and where `read`, `mmap` and `ioctl` fail with `EBADF` — or after it, which
is exactly the prohibited data-open. A name-based `fstatat` with `AT_SYMLINK_NOFOLLOW`
before opening does not fix this, because the name can be replaced between the stat and
the open; `O_PATH` removes that window by pinning the object.

**Strategy C does not exist for this policy without privilege.** An `O_PATH` descriptor
cannot be upgraded in place: `openat` does not accept an empty pathname, so there is no
`AT_EMPTY_PATH` reopen; `open_by_handle_at` and `linkat` with `AT_EMPTY_PATH` require
`CAP_DAC_READ_SEARCH`; `F_DUPFD` yields another `O_PATH` descriptor. The only remaining
unprivileged route is opening the process's own `/proc/self/fd/N`, which the kernel
resolves to the pinned file description rather than by re-walking a pathname. Procfs is
therefore **not a convenience: it is the consequence of requiring both metadata-first
classification and readable bytes from exactly the pinned object without privilege.**

The report reaches the same design, so the recommendation is not to change it. The defect
is evidential (I2): the report calls this route unproven and demands a falsification
experiment, but the experiment as defined exercises only route A. Running A successfully
falsifies nothing about B. The corrected preregistration adds a small mandatory
comparison arm (D1 to D3 in section 12) so the mechanism choice is recorded as measured
behaviour rather than as an assumption.

Two consequences must be written into the architecture instead of left implicit:

1. **The mandated `RESOLVE_*` set applies to target resolution only.** The deliberate
   procfs reopen traverses a magic link on purpose and therefore must not carry
   `RESOLVE_NO_SYMLINKS` or `RESOLVE_NO_MAGICLINKS`. Any frozen syscall policy that does
   not distinguish the two open classes would score the legitimate reopen as a violation.
   This distinction is load-bearing for I7.
2. **Procfs is a prerequisite for the file-bytes operation only.** Directory metadata
   targets need no reopen. If procfs is unavailable the correct outcome is admission
   failure or `reopen_unavailable`, never a pathname reopen — which the report already
   states and falsifier 12 already forbids.

## 5. Authority binding

The three-boundary design (untrusted plan bytes to `ValidatedPlan`; trusted caller to
`AuthorizedScope`; scope to observation) is sound and does prevent the listed attacks,
with one gap.

| Attack | Prevented? | Mechanism |
|---|---|---|
| Ambient scanning | Yes | No `AT_FDCWD` lookup, no HOME/PATH/registry/package/network capability, no recursion or enumeration, no directory listing |
| Plan-supplied numeric FD authority | Yes, structurally | The plan schema has no descriptor field; roots are logical IDs only. Authority is an owned descriptor passed in Rust. An unknown field is rejected by the closed schema. "a numeric FD in JSON is never authority" |
| Root substitution | Out of scope, honestly | The observer cannot authenticate a root it did not open. The report says so and claims nothing more; a hostile authority broker is excluded by the stated trust boundary, not solved |
| Plan mutation after authorization | Yes | `ValidatedPlan` is immutable with private fields and retains its exact bytes; identity is the digest of those bytes, so equal digest implies equal plan content |
| Target-set mutation after authorization | Yes, by the same property | Targets are a borrowed view of the immutable plan |
| Authorize plan P1, observe plan P2 | **Only by an unstated runtime check** | See below |

`authorize(&plan, roots, reopen)` and `observe(&plan, &scope)` each take a plan
reference. The report says `observe` "verifies scope/plan identity before reads", that
`authorize` should "retain ... the approved plan identity", and that replacing a plan
requires a new authorisation. That is the right intent, but it is prose: the required
check is never written as an equality, and no test obligation is attached to it. Because
plan identity is the digest of exact accepted bytes, digest equality *is* structurally
sufficient for the plan itself — pointer or object identity is not, and must not be
relied on.

**Required correction (I3), minimum form.** State in the architecture that `observe`
must reject unless

```text
scope.authorized_plan_sha256 == plan.sha256()
```

compared over the full 32 bytes, and that the scope additionally binds the set of root
logical IDs and their count, verified by ID rather than by position. Root *order* need
not be bound: targets reference roots by unique ID, so the ID-to-capability mapping
carries the whole meaning and binding order would add a constraint with no security
content.

**Recommended stronger form.** Have `authorize` consume the `ValidatedPlan` into the
`AuthorizedScope`, and give `observe(&AuthorizedScope)` no separate plan parameter. Plan
substitution then becomes impossible by construction instead of being prevented by a
comparison a later refactor could drop. This is the structurally stronger equivalent and
is preferred; the digest equality remains the fallback if the owner keeps the current
signature for API reasons.

Neither form is exercised by OBS-FS-01, and that is acceptable: the spike is a syscall
experiment that need not implement the full API. The binding therefore belongs on the
*implementation* test list, and section 13 records it there so it cannot be lost.

## 6. Path resolution policy

The mandated set is `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS`, `RESOLVE_NO_MAGICLINKS` and
`RESOLVE_NO_XDEV`, with `O_PATH|O_NOFOLLOW|O_CLOEXEC`. Verified against the `openat2`
documentation:

- **Absolute paths are never accepted.** The plan grammar rejects them at validation, and
  `RESOLVE_BENEATH` independently "causes absolute symbolic links (and absolute values of
  path) to be rejected". Two layers, correctly ordered.
- **`..` never reaches the kernel.** The plan path grammar excludes empty, `.` and `..`
  components. This matters more than the report says: the documented `EAGAIN` case is
  specifically "the kernel could not ensure that a `..` component didn't escape", so
  excluding `..` at validation removes the main source of that error. `EAGAIN` remains
  reachable through ancestor renames and must still map to `resolution_race`.
- **The starting root may itself be a mount point.** `RESOLVE_NO_XDEV` forbids
  *traversing* a mount point during resolution; beginning at one is not a traversal.
- **Descendant bind mounts are rejected**, "including all bind mounts", even when the bind
  source is on the same device. This is why `RESOLVE_NO_XDEV` is the correct primitive and
  an `st_dev` comparison is not: a same-device bind mount would pass a device check.
- **Mount topology changing during resolution** yields `EXDEV` or `EAGAIN`. The report's
  rule — return an unresolved observation rather than weakening flags or re-searching — is
  the right response and is consistent with falsifier 12.
- `RESOLVE_NO_SYMLINKS` "implies `RESOLVE_NO_MAGICLINKS`", so listing both is redundant.
  Keeping both is harmless and self-documenting; no change required (M2).

**Finding I1.** The report's result vocabulary asks for more attribution than the kernel
supplies. Under this flag set:

- `EXDEV` is returned both for "an escape from the root during path resolution was
  detected" (`RESOLVE_BENEATH`) and for "a path component crosses a mount point"
  (`RESOLVE_NO_XDEV`).
- `ELOOP` is returned for "one of the path components was a symbolic link (or magic
  link)", with no indication of *which* component, so a leaf symlink and a parent symlink
  are indistinguishable.

The observer therefore cannot honestly emit `mount_crossing` as distinct from a
beneath-escape, and cannot attribute `symlink_forbidden` to a component. It could only do
so by re-resolving with weakened flags, which the report forbids. As written, an
implementation reporting one honest rejection code would be scored FAIL against the
preregistered per-case codes, while an implementation inventing attribution would violate
the report's own rule that "EXDEV does not invent the destination".

The correction is in section 12: expectations are defined over **errno classes plus
observable properties**, and the two rejection codes are declared interchangeable wherever
the kernel cannot separate them. This must be fixed **before** execution, because it
changes what counts as a pass.

**Path-resolution rejection versus post-open classification rejection** must be separable
in the record and in the trace, and they are: resolution rejection is an `openat2`
returning `-1`, so no descriptor exists; classification rejection is a successful
`openat2` returning an `O_PATH` descriptor, followed by `statx` and no reopen. The
corrected preregistration makes that an explicit trace property rather than an inference
from the final result.

## 7. Object-kind safety

Permitted operations before classification are exactly two, and both are non-data:
`openat2` with `O_PATH|O_NOFOLLOW|O_CLOEXEC` and the mandated resolve set, then `statx`
with an empty path and `AT_EMPTY_PATH`, which the documentation confirms targets "the one
referred to by the file descriptor". `O_PATH` guarantees the object is not opened and that
`read`, `mmap` and `ioctl` on that descriptor fail with `EBADF`, so classification cannot
accidentally become a data access.

| Object kind | Permitted before classification | Required outcome |
|---|---|---|
| Regular file | `openat2` with `O_PATH`, `statx` | Pin, then size and budget check, then procfs reopen, then stream |
| Directory | `openat2` with `O_PATH`, `statx` | Metadata only; never listed |
| Symlink | Nothing beyond the failed `openat2` | Rejected at resolution with `ELOOP`; no descriptor is produced |
| FIFO | `openat2` with `O_PATH`, `statx` | Metadata-only rejection. `O_PATH` cannot block and cannot wake a waiting writer |
| Unix socket | `openat2` with `O_PATH`, `statx` | Metadata-only rejection; no connect |
| Character device | `openat2` with `O_PATH`, `statx` | Metadata-only rejection; the driver open path is never entered |
| Block device | `openat2` with `O_PATH`, `statx` | Metadata-only rejection; no media access or exclusive-open attempt |
| Sparse regular file | As regular file | Logical bytes hashed including holes; no hole skipping and no allocation from length |
| Hardlinked regular file | As regular file | Permitted inside an authorized root; link count sampled, origin unestablished |

The report's threat table states that "`O_PATH` link metadata can identify rejection
without reading its destination". Under the mandated flag set this is not what happens:
`RESOLVE_NO_SYMLINKS` fails the call, so no link handle is produced and the record cannot
carry an observed kind of symlink. The behaviour is safe — safer, in fact — but the
sentence describes a different flag set. Recorded as M3; the record schema's existing
hedge, "kind included only if actually established", already accommodates the truth.

**Metadata-only rejection is testable from the trace**, and the corrected preregistration
requires exactly that: for every special-file case the complete child syscall trace must
contain the `O_PATH` `openat2` and its `statx`, and must contain no further `openat` or
`openat2` naming that object and no `read`, `pread64`, `readv`, `preadv` or `mmap` on its
descriptor. A final JSON result is not accepted as proof.

## 8. Symlink, replacement and race expectations

Each case is listed separately with a safe outcome *set* rather than a single errno, as
instructed. "No escape" below means the complete trace contains no `openat`, `openat2` or
read reaching any object outside the authorized root subtree, and no absolute pathname
argument other than the procfs reopen path.

| Race | Preregistered safe outcome set | Invariant that must hold in every trial |
|---|---|---|
| Leaf regular and symlink swap | Rejected in the `ELOOP` class if the swap lands before resolution; observed file with the digest of the pinned regular object if it lands after the pin | No escape; the link destination is never opened |
| Parent directory and symlink swap | Rejected in the `ELOOP` class, or observed file from the pinned object if the swap lands after that component resolved | No escape |
| Regular file replacement **before** pin | Observed file with the digest of the new object; `absent` if the replacement window left no entry | No escape; the observer must not judge the bytes |
| Regular file replacement **after** pin | Observed file with the digest of the retained object; `changed_during_read`; `io_failure` with a partial byte count | No complete digest after a known short read; no claim that the pathname still names the object |
| Root directory rename or replacement after the caller supplied its fd | Observed outcomes from the retained root; `absent`; rejected | No escape; no live-path-ancestry claim in the record |

Passing is defined by the property, not the errno: never follow an unauthorised link or
escape; either reject safely or operate on the already-pinned object; never claim
persistent pathname binding after rename or replacement.

**Proof of injection is mandatory.** A race trial whose scheduled mutation did not land
inside its syscall window is **INVALID**, not PASS, and must be reported with its seed.
The cited probe already records an injection flag and forces the mutation at the actual
syscall stop rather than relying on timing luck; the adapted harness must preserve that
property and must additionally record *which* syscall stop was used, because the new
sequence has more than one open per target.

## 9. Minor findings

| # | Location | Finding and minimum correction |
|---|---|---|
| M1 | report section 7 | Candidate minimum Linux 5.8 gives `STATX_MNT_ID`, which is not guaranteed unique; `STATX_MNT_ID_UNIQUE` (Linux 6.8) "is guaranteed to not be reused while the system is running". Record which was used, prefer the unique ID where available, and never treat a plain mount ID as a discriminator across an unmount. The chosen lab kernel supplies the unique ID |
| M2 | report section 7 | `RESOLVE_NO_SYMLINKS` implies `RESOLVE_NO_MAGICLINKS`; listing both is redundant but harmless. No change required, noted so a reviewer does not read it as two independent guarantees |
| M3 | report section 7 threat table | The `O_PATH` symlink-metadata sentence does not hold under the mandated flag set. Delete it, or condition it on a flag set that is not used |
| M4 | report sections 7 and 9 | State explicitly that an `ENOENT` arising from the procfs reopen stage is `reopen_unavailable`, never target `absent`. The existing definition already ties `absent` to lookup; the trap deserves one sentence because both stages can produce the same errno |
| M5 | report section 16 | Record the deterministic consequence of the ceilings: reserving 512 MiB plus one byte per maximum file against a strict 1 GiB aggregate means a batch admits **at most one** maximum-sized file, and a second one deterministically returns a budget outcome. This is a clarification, not a defect; no constant should change without a concrete flaw |
| M6 | report section 14 | The over-limit fixture must be **sparse**, or it cannot coexist with the 1 GiB fixture allocation. Record that the small sparse case and the over-limit case have different purposes: hole hashing versus never opening data |
| M7 | report section 9 | `wrong_kind` is the only outcome whose name reads comparatively. Repeat the section 3 rationale — a requested observable constrains safe measurement mechanics, not a desired outcome — beside the code definition |
| M8 | report section 14 | The preflight list is prose, "verify current tools/kernel/permissions". Replace it with the enumerated list in section 13, so a missing item produces BLOCKED rather than an improvised judgement |

## 10. Corrected preregistration: what changes before execution

The following supersede the corresponding expectations in report section 14. They correct
*expectations*, not the architecture, and none weakens a claim.

1. Every case carries an explicit **mandatory** or **optional** marking (section 11).
2. Expected outcomes are defined as **errno classes plus trace properties**, and rejection
   codes the kernel cannot separate are declared interchangeable (section 12).
3. A direct-open comparison arm is added and is mandatory (D1 to D3).
4. The in-place overwrite case is split into a deterministic arm and an explicitly
   undetectable arm.
5. The metadata length and aggregate budget check is fixed at the `statx` step, before any
   reopen, and the over-limit case predeclares **zero data-opens**.
6. Each security property names its authoritative instrumentation artifact (section 13).
7. The experiment gains a single aggregate verdict rule (section 14).
8. The mount case gains a preflight probe, a non-namespace fallback arm and an explicit
   statement of what remains unverified if it blocks (section 13).

## 11. Mandatory and optional cases

| Case | Status |
|---|---|
| Regular nonempty and empty file | Mandatory |
| Missing leaf and missing parent | Mandatory |
| Wrong file identity with same name | Mandatory |
| Static internal, escaping and dangling symlink | Mandatory |
| Leaf and parent symlink swap | Mandatory |
| FIFO, including swap with no writer | Mandatory |
| Unix socket | Mandatory |
| Directory in file position; regular file in parent position | Mandatory |
| Concurrent file replacement, before and after pin | Mandatory |
| Directory and root replacement after pin | Mandatory |
| Growing and truncating file | Mandatory |
| In-place same-length overwrite, deterministic arm | Mandatory |
| In-place same-length overwrite, undetectable arm | Mandatory |
| Hardlink to synthetic canary, including alias mutation | Mandatory |
| Permission denied on leaf and on parent search | Mandatory |
| Sparse file, and metadata-over-limit file | Mandatory |
| Budget exhaustion and partial I/O failure | Mandatory |
| Procfs unavailable, and fake procfs directory | Mandatory |
| Two-file changing environment | Mandatory |
| Private marker, unlisted file, descriptor-shaped plan field | Mandatory |
| **D1 to D3: direct-open comparison arm** | **Mandatory** (new) |
| **Mount crossing, non-namespace fallback arm** | **Mandatory** (new) |
| Nested bind mount inside an unprivileged namespace | Conditional: mandatory *claim*, executable only if preflight permits; otherwise BLOCKED |
| `/dev/null` and `/dev/zero` metadata-only classifier subtest | Optional |

A conditional case that blocks does not reduce the mandatory set and cannot be waived by
reinterpreting "sufficient coverage" after execution. Optional cases may not be promoted
to mandatory after execution, and mandatory cases may not be demoted. The list freezes
with the fixture recipes.

## 12. Expected outcome sets

`R` is the set of acceptable *result* outcomes; `T` are trace properties that must hold in
every trial regardless of which result occurred.

| Case | R | T |
|---|---|---|
| Regular nonempty and empty | Observed file with the independently computed digest | Exactly one `O_PATH` `openat2`, one `statx`, one procfs reopen, the reads, one close |
| Missing leaf and missing parent | `absent` | No sibling or alternative path is attempted |
| Wrong file, same name | Observed file with the *actual* digest | No comparison and no mismatch code anywhere |
| Static symlink: internal, escaping, dangling | Rejected, `ELOOP` class | No descriptor for the link; destination never opened; no escape |
| Leaf and parent symlink swap | Rejected `ELOOP` class, or observed file from the pinned object | Injection landed; no escape |
| FIFO, socket, character device, block device | Rejected as a special file | `O_PATH` `openat2` and `statx` only; **zero** further opens naming the object and zero reads on its descriptor |
| Directory in file position | Rejected as wrong kind, with observed kind directory | No directory enumeration of any kind |
| Regular file in parent position | Rejected, `ENOTDIR` class — never `absent` | — |
| Replacement before and after pin | See section 8 | Injection landed |
| Growing file | `changed_during_read`, or `io_failure` with the partial count | At most the ceiling plus one byte charged; no complete digest after a short read |
| Truncating file | `changed_during_read`, or `io_failure` with the partial count | Same |
| In-place overwrite, deterministic arm | `changed_during_read`, **or** observed file whose digest is in the frozen set computable from the scheduled interleaving | Injection landed at the declared read boundary |
| In-place overwrite, undetectable arm | Observed file whose digest is in the frozen interleaving set, and a record that claims no snapshot consistency | Demonstrates the documented limit; non-detection here is a PASS, not a FAIL |
| Hardlink | Observed file with sampled link count of at least two | No alias enumeration; no path outside the root opened |
| Alias mutation | Observed file with a digest in the frozen set, or `changed_during_read` | — |
| Permission denied, leaf and parent search | `permission_denied` — never `absent`, never a guessed digest | No retry under another identity |
| Sparse file | Observed file whose digest hashes holes as zeros | No `SEEK_HOLE` or `SEEK_DATA` skipping |
| Metadata-over-limit file | `file_limit` | **Zero** procfs reopens and zero reads for that target |
| Budget exhaustion | `total_limit` for the target that cannot fit; `not_attempted_total_limit` for every later target | Earlier successful results present and unchanged; every declared target has an explicit entry |
| Procfs unavailable | Admission failure, or `reopen_unavailable` per file target | **Zero** pathname reopens; directory targets still succeed |
| Fake procfs directory | Admission failure | Rejected before any target read; acceptance test in section 13 |
| Private marker and unlisted file | Unlisted objects absent from the record | Zero opens naming them; no absolute path in any output stream |
| Descriptor-shaped plan field | Plan validation error | No I/O at all |
| D1: FIFO direct open read-only, no writer | Child killed at its deadline | Demonstrates the blocking hazard |
| D2: FIFO direct open read-only with `O_NONBLOCK` | A descriptor is returned on a FIFO object | Demonstrates that direct open **is** a data-open |
| D3: the same objects via `O_PATH` | Descriptor returned, `statx` classifies, zero reads | The contrast that justifies the chosen mechanism |
| Mount fallback arm | Rejected, `EXDEV` class | Resolution failed; no descriptor produced |
| Bind mount inside a namespace | Rejected `EXDEV` class, or **BLOCKED** | — |

**Interchangeable rejection codes.** `EXDEV` may be reported as `mount_crossing` *or* as a
beneath-escape code; both are accepted and neither is a FAIL. `ELOOP` may be reported as
`symlink_forbidden` without component attribution. The pass criterion is the property — no
escape, no data-open, no invented attribution — not the label.

**Hazard limit for the direct-open arm.** D1 and D2 use only FIFOs the harness created.
Hazardous device classes are never opened by any arm, including the optional classifier
subtest, which is limited to `/dev/null` and `/dev/zero` and is metadata-only.

## 13. Instrumentation, oracles and prerequisites

**Authoritative artifact per property.** A final JSON result is not accepted as proof of
any of these.

| Security property | Authoritative artifact |
|---|---|
| Which resolution flags were used | Complete syscall trace decoding both `open_how.flags` and `open_how.resolve` for every `openat2` |
| No data-open occurred for a special file | Complete trace showing, for that object, only the `O_PATH` `openat2` and its `statx`. Completeness must be asserted: single-threaded child, no fork, no untraced descendant |
| Which object descriptor was classified | Trace correlation of the `openat2` return value with the `statx` descriptor argument, tracking closes so a reused descriptor number is not confused |
| Which descriptor supplied hashed bytes | Trace correlation of the reopened descriptor with every read on it, plus the spike's own `statx` tuple for that descriptor; the two must agree |
| No escape outside the authorized root | Trace contains no open naming an absolute path other than the procfs reopen, and no open whose directory argument is neither an authorized root nor the procfs capability |
| Race injection timing | The harness's recorded injection flag, the syscall stop it fired at, and the seed |
| Byte accounting | Sum of read return values in the trace, reconciled against the spike's reported counts |
| Process timeout and termination | Supervisor record of deadline, signal and reaped status |

The cited [probe](../../tools/helm_evidence_linux_probe.py) is the right starting point:
unprivileged, in-repository, needing no new packages, and forcing the mutation at the real
syscall stop. It is **not** sufficient as-is, and the report's warning against assuming it
understands the new sequence is correct. Concretely it currently reads only the first
eight bytes of the `open_how` structure, so it sees the flags but never the resolve field;
records a result only for the single injected syscall; traces no reads; and filters leaf
opens on the absence of `O_PATH`, which is precisely inverted for a spike whose target
opens are all `O_PATH`. Proving a negative needs a complete trace, so the adaptation must
log every open and read rather than one selected syscall. `strace` is an acceptable
alternative only if preflight finds it already installed; installing packages is out of
scope.

**Independent oracles, frozen before execution.** Expected digests come from Python
`hashlib` over preregistered byte recipes, never from the spike. Frozen artifacts: fixture
byte recipes; expected digests including the enumerated interleaving digest sets; the
mandatory and optional case list; the safe outcome sets; the race schedule and seeds; and
the expected syscall policy as a machine-readable file. The policy checker must neither
import nor link the spike, so the experiment's classifier cannot define its own
expectation.

**Repetition policy.** One execution per deterministic static case; one forced execution
per deterministic race schedule; at most 20 preregistered stress repetitions where
justified. All seeds and failures preserved. Retry-until-green is forbidden: a required
injection that did not occur is INVALID, and if every repetition of a case is INVALID the
case is INCONCLUSIVE, never PASS.

**Preflight; each item recorded as a fact, and a missing item yields BLOCKED for the cases
that depend on it.** VM identity; kernel release; ext4 confirmed for the fixture
filesystem together with its mount options; current UID, GID and supplementary groups,
expected to be 1001 with none; `rustc`, `cargo` and `gcc` presence and versions;
`kernel.yama.ptrace_scope`, which must permit tracing an own child; procfs mounted, its
`hidepid` setting, and whether the process's own descriptor directory is reachable;
`STATX_MNT_ID_UNIQUE` support; `strace` presence; free disk against the 1 GiB fixture and
5 GiB total budgets.

**Mount prerequisite (I8).** The report proposes the nested bind-mount case on the Ubuntu
24.04.4 lab without noting an obstacle this repository has already recorded:
[ADR-0016](../adr/ADR-0016-host-side-sandbox.md) and the
[foundation audit](../research/FOUNDATION_AUDIT.md) both state that Ubuntu 24.04 and later
restrict unprivileged user namespaces through AppArmor. Creating a private user and mount
namespace to build a bind mount is therefore likely to fail for the unprivileged `helmlab`
user. Preflight must probe this — record the AppArmor userns restriction sysctl and
attempt one throwaway unshare of a user and mount namespace — and must **not** relax it:
changing that sysctl, adding an AppArmor profile, or using `sudo` are all forbidden host
privilege changes.

Because of that, a **non-namespace fallback arm is mandatory**: resolve a target across an
already-existing mount boundary the unprivileged user can reach without creating anything,
and require `EXDEV`. That exercises the `RESOLVE_NO_XDEV` code path itself. It does **not**
substitute for the bind-mount case.

If the namespace case blocks, exactly one claim remains unverified and must be recorded as
such: **that `RESOLVE_NO_XDEV` rejects a bind mount created as a descendant of an
authorized ext4 root.** General mount-traversal rejection is still evidenced by the
fallback arm. The unverified claim must not be softened into "mount crossing was tested",
and implementation approval for any use that depends on descendant bind mounts stays
pending, as report section 14 already requires.

**Procfs acceptance test (I4).** "Reject a synthetic fake proc directory" needs a stated
criterion. Two metadata-only checks suffice, and neither probes a plan-controlled path:

1. A filesystem-statistics call on the supplied directory must report the procfs
   filesystem type. A tmpfs or ext4 decoy fails here.
2. A self-identity probe: the observer creates a descriptor it already controls, opens
   that decimal name under the supplied directory with `O_PATH`, and requires the device
   and inode to match the known object. A directory belonging to another process's
   descriptor namespace fails here.

Preregistered outcome: both decoys are rejected at admission with **zero** target reads.
`hidepid` does not interfere, since it restricts other processes' directories rather than
the caller's own. If procfs is absent entirely the outcome is admission failure, never a
pathname fallback.

**Implementation-test obligations, recorded here so they are not lost.** These are not
OBS-FS-01 cases: the plan-to-scope digest binding of section 5; root ID and count binding;
rejection of an authorized-then-substituted plan; and the plan, artifact and diagnostic
size ceilings.

## 14. Experiment verdict rules

One verdict for OBS-FS-01, distinct from the observer's result vocabulary. The observer
never emits these words, and the experiment never emits observer codes as conclusions.

| Verdict | Condition |
|---|---|
| **PASS** | Every mandatory case produced a result inside its preregistered safe set, every mandatory injection is proven to have occurred, and every authoritative instrumentation artifact was collected and interpretable |
| **FAIL** | At least one valid mandatory trial violated a required safety or correctness property |
| **INCONCLUSIVE** | Execution occurred but a required property could not be interpreted, including a mandatory case whose every repetition was INVALID |
| **BLOCKED** | A mandatory prerequisite or case could not be executed |

Precedence: any FAIL makes the experiment FAIL. Otherwise an unexecuted mandatory
prerequisite makes it BLOCKED. Otherwise an uninterpretable mandatory property makes it
INCONCLUSIVE. PASS requires the complete mandatory set. A conditional case that blocks
yields BLOCKED for the claim it carries, recorded alongside PASS for everything else,
rather than being absorbed into a general pass.

**A PASS does not accept ADR-0022 and does not authorise implementation.** It is evidence
for one cohort only: Linux `7.0.0-31-generic` on x86_64, local ext4 with the recorded
mount options, the pinned Rust and GCC toolchain, unprivileged UID 1001, and the specific
`O_PATH` plus procfs-reopen mechanism. It does **not** establish arbitrary Linux
filesystem safety, network or FUSE behaviour, future kernel behaviour, Windows semantics,
atomic environment snapshots, runtime provenance, or safe execution.

**Sufficient for the owner to consider bounded acceptance** of ADR-0022 for a Linux and
ext4 0.1 design would be: an OBS-FS-01 PASS across the complete mandatory set above; the
bind-mount claim either verified or explicitly carried as unverified with its dependent
uses excluded from 0.1; the corrected expectations in this document accepted *before*
execution; and a separate implementation instruction. Anything less leaves ADR-0022
Proposed.

## 15. Residual limitations of this review

This is a documentation review. No syscall was executed, no VM was booted, no observer
exists, and no A0 artifact was read or re-observed. The kernel semantics used here come
from the `openat2`, `open`, `statx`, `fifo` and `proc_pid_fd` manual pages consulted on
2026-09-08; they are documentation, not behaviour measured on the lab kernel, and preflight
must confirm them there. The AppArmor user-namespace prediction is a documented
distribution property and an expectation, not a measurement of that VM. Whether inode
change-cookie or i_version metadata is usable was deliberately left open: it is not assumed
available, and must be a recorded preflight fact if used at all. The direct-open analysis
establishes what the kernel documentation guarantees; D1 to D3 exist to convert that into
measured behaviour. Nothing here validates the proposed API, the future binder, launch
safety, effective DLL configuration, dedication semantics, or the unresolved shared
Cargo-graph `sha2/force-soft` limitation.

A0-7ZIP remains experimental FAIL and was not rerun. ADR-0022 remains Proposed.
OBS-FS-01 remains NOT_RUN.
