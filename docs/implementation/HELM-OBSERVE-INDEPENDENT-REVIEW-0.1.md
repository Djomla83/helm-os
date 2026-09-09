# helm-observe 0.1 independent review

**Role:** independent adversarial review of the first Rust implementation, by a reviewer
who is not the implementation author.\
**Reviewed candidate:** `e2a62081ece52391b30ede153eee139103c98e31` on
`product/helm-observe`.\
**Review branch:** `review/helm-observe-independent`, descending from the candidate, which
is preserved unchanged.\
**Authority:** Accepted [ADR-0022](../adr/ADR-0022-observation-authority.md), bounded 0.1
scope. Nothing here broadens it.

**Classification: READY_FOR_OWNER_MERGE**, following the owner's architecture decision of
2026-09-09. The earlier disposition of this review was
**NEEDS_ARCHITECTURE_OWNER_REVIEW**; that is recorded below rather than erased, and the
finding that produced it stands as a valid finding.

One BLOCKER and three IMPORTANT findings were raised. Three of the four were corrected on
this branch with regression tests. The fourth, IMPORTANT-3, could not be resolved by any
implementation change: the accepted cohort names **ext4**, and no mechanism inside the
accepted explicit-capability, no-ambient-scan boundary can establish ext4 rather than
ext-family. It was referred to the owner, who resolved it by
[clarifying ADR-0022](../adr/ADR-0022-observation-authority.md#cohort-attestation-clarification)
rather than by broadening scope: **cohort membership is an external support precondition,
not an observation attestation.** Section 6 records the analysis, the decision and the
resulting disposition.

The findings in section 3 were committed **before** any correction
(`a1726f3`), so the candidate as submitted is preserved in history rather than tidied away.

---

## 1. Independently reconstructed state

| Claim | Method | Result |
|---|---|---|
| `origin/main` is the accepted architecture base | `git rev-parse origin/main` | `0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b`, as instructed; no newer owner work |
| `53ade006` descends from main | `git merge-base --is-ancestor` | yes |
| `e2a62081` descends from `53ade006` | `git merge-base --is-ancestor` | yes |
| `product/helm-observe` is the reviewed candidate | `git rev-parse` | `e2a62081…`, equal to `origin/product/helm-observe` |
| main does not contain helm-observe | `git ls-tree origin/main crates/` | only `helm-app-spec` and `helm-evidence`; main carries the architecture and OBS-FS-01 documents only |
| no force-push or rewrite in candidate history | `git log 0f93431..e2a62081` | two linear commits; evidence commits `63ac679`, `2496f68`, `1bd9c0a` are all ancestors of main, in the recorded order |
| dependency graph | `git diff Cargo.lock` | one new member plus its `rustix` edges; no new third-party version |

Read in full: `AGENTS.md`, `docs/PROJECT_STATE.md`, [DECISIONS](../DECISIONS.md),
[ADR-0022](../adr/ADR-0022-observation-authority.md), the
[architecture report](../research/HELM-OBSERVE-ARCHITECTURE.md), the
[independent review carrying Amendment 1](HELM-OBSERVE-INDEPENDENT-REVIEW.md), the
[OBS-FS-01 preflight and halt](../experiments/OBS-FS-01-PREFLIGHT-AND-HALT.md), the
[OBS-FS-01 execution report](../experiments/OBS-FS-01-EXECUTION-REPORT.md), the
[frozen execution definition sources](../experiments/obs-fs-01/), the
[author review](HELM-OBSERVE-REVIEW.md), every `crates/helm-observe` source and test file,
the Cargo manifests, lock and workspace lints, and the `helm-app-spec` and `helm-evidence`
dependency configuration relevant to `sha2` feature unification.

## 2. Independent source trace

Traced from the Rust source, not from the documentation.

| Transition | Input | Owned authority | Filesystem operation | Failure modes | Alternate path |
|---|---|---|---|---|---|
| bytes → `ValidatedPlan` | `&[u8]` | none | none | `PlanErrors` only | none: `plan.rs` links no I/O API |
| descriptor → `RootCapability` | `OwnedFd` | that descriptor | `statx(AT_EMPTY_PATH)`, `fstatfs` | not a directory, metadata unavailable, unsupported filesystem | none: no path-based constructor exists |
| descriptor → `ProcFdCapability` | `OwnedFd` | that descriptor | `fstatfs`, `memfd_create`, `statx`, one `openat` of a decimal name | wrong filesystem, foreign or unusable | none |
| plan + capabilities → `AuthorizedScope` | by value | plan and all root descriptors | none | missing, extra, duplicate root | none |
| scope → target pin | authorised scope | root descriptor | one `openat2`, `O_PATH│O_NOFOLLOW│O_CLOEXEC` with `BENEATH│NO_SYMLINKS│NO_MAGICLINKS│NO_XDEV` | `ENOENT`→`Absent`, `ENOTDIR`→`wrong_kind`, `EXDEV`→`scope_violation`, `ELOOP`→`symlink_forbidden`, `EACCES`/`EPERM`→`permission_denied`, `EAGAIN`→`resolution_race`, `ENOSYS`/`EOPNOTSUPP`→`unsupported_platform`, other→`io_failure` | none: one call, no retry, no `openat` fallback |
| pin → classification | pinned `OwnedFd` | that pin | `statx(AT_EMPTY_PATH)` | `io_failure` | none |
| classification → data reopen | a pin classified `Regular` **only** | pin plus procfs capability | one `openat` naming the pin's decimal descriptor number under the procfs capability | `EACCES`/`EPERM`→`permission_denied`, other→`reopen_unavailable` | none, and no pathname is ever reopened |
| reopen → stream | reopened descriptor | that descriptor | `statx` identity re-check, bounded `read` loop, `statx` | identity mismatch→`io_failure`, read error→`io_failure`/`permission_denied`, change→`changed_during_read` | none |
| results → artifact | owned record | none | none | none | `ObservationArtifact::new` is crate-private |

Whole-crate searches confirm the author's no-hidden-authority claim on every point.

| Searched for | Result |
|---|---|
| `Path`, `PathBuf`, `std::path` | **none** |
| `std::fs` | one occurrence, `File::from(OwnedFd)`, which wraps an already-open descriptor and opens no name |
| `read_dir`, `readdir`, `getdents`, `walkdir`, `glob`, recursive search | **none**, and the syscall trace confirms zero `getdents64` in every case |
| subprocess, `Command`, network or socket APIs | **none**, and the trace confirms zero `connect` |
| environment, `HOME`, `PATH`, cwd | **none** in code; only in prose and in error-code spellings |
| raw descriptor conversions | two `as_raw_fd()` calls, both only to render a decimal descriptor number; an `i32` taken from an `OwnedFd` cannot contain path syntax |
| `unsafe`, `libc`, raw syscalls | **none**; the workspace forbids `unsafe_code` and the crate needs no exception |
| filesystem writes | **none**; the only `truncate` calls act on an in-memory `Vec`/`String` |
| an `openat2` fallback inside `rustix` 1.1.4 | none: both backends issue the syscall directly and surface `ENOSYS` |

**Verdict: the claim of no hidden authority and no alternate path is confirmed**, by source
and by trace.

## 3. Findings

Severity as defined by the review instruction.

### BLOCKER-1 — the procfs self-identity probe admitted a foreign process. **Reproduced. Corrected.**

**Location:** `crates/helm-observe/src/linux.rs`, `admit_procfs`, as submitted.

**What the accepted mechanism requires.** The frozen OBS-FS-01 definition, section 13,
"Procfs acceptance test (I4)", requires that *"the observer **creates a descriptor it
already controls**, opens that decimal name under the supplied directory with `O_PATH`, and
requires the device and inode to match the known object."* ADR-0022 accepts reading only
through an *"admission-validated **trusted current-process** procfs descriptor capability"*.

**What the candidate did.** It created no probe object. It used the caller-supplied
capability descriptor **as its own probe**: it took that descriptor's number `N`, opened the
name `"N"` under the same directory, and compared the result with the directory itself. The
probe object was therefore one whose identity a foreign process can arrange.

**Concrete attack, reproduced.** A foreign process that keeps its own `/proc/self/fd` open
across the low descriptor numbers makes `/proc/<foreign>/fd/<N>` resolve, for any such `N`,
to the foreign process's own descriptor directory — exactly the object the caller supplied.
Device and inode then match and admission succeeds for a **foreign** descriptor namespace.

*Reproduced*, not merely derived: on hosted Ubuntu 24.04, run `34280059548`,
`procfs_admission_refuses_a_foreign_self_fd_hoarding_process` failed with
`a foreign process descriptor directory was admitted as the trusted current-process procfs
capability (probe descriptor 4)`.

**Consequence.** A false admission of a foreign process's descriptor directory, violating an
accepted security invariant. It does **not** by itself yield an unauthorized read:
`reopen_and_stream` re-verifies device, inode and kind against the pin before the first
byte, and two distinct objects cannot share a device and inode pair, so a foreign namespace
yields `io_failure` rather than foreign data. The realised damage is invariant violation
plus an availability and race surface, not disclosure. The BLOCKER criterion is the
invariant, and the invariant was broken.

**Correction applied.** Admission now creates a fresh anonymous memory object
(`memfd_create`) as the probe and compares device, inode **and kind** against it. The object
is created after any process that already exists, is never shared or inherited, lives on an
internal kernel mount, needs no new dependency, needs no `unsafe`, and writes nothing to any
observed filesystem. This restores the reviewed mechanism rather than inventing one.

**Regression tests.** `procfs_admission_refuses_a_foreign_self_fd_hoarding_process`,
`…_refuses_a_foreign_process_sharing_the_callers_objects`,
`…_refuses_a_dead_foreign_process`, `…_refuses_a_tmpfs_decoy` and
`…_accepts_the_real_current_process_capability`.

### IMPORTANT-2 — duplicate-key rejection was bypassable, two ways. **Reproduced. Corrected.**

**Location:** `crates/helm-observe/src/plan.rs`, `scan_raw`, as submitted.

1. **Array-nested objects were never scanned.** `expect_key` was recomputed as
   `depth == stack.len()`, but `depth` counted `{` *and* `[` while `stack` counted only `{`.
   Inside any object nested in an array — that is, inside every `roots[]` and `targets[]`
   entry — the two never agreed after the first key.
   `{"targets":[{"id":"decoy",…,"id":"real"}]}` was **accepted** and silently resolved to
   `real`; a target carrying `"path":"harmless"` and later `"path":"other/elsewhere"`
   validated and would have observed the second spelling.
2. **Escaped key spellings were not decoded.** The scanner dropped escape sequences instead
   of decoding them, so a `"targets"` key written with a `t` escape was recorded under
   a different name and did not collide, while `serde_json` decoded both to `targets` and
   kept the last. A document with a visible decoy `targets` array and an escaped real one
   was **accepted**, and the decoy was discarded.

Both were reproduced first locally and then on CI by
`crates/helm-observe/tests/independent_review_plan.rs`.

**Consequence.** The crate documents, in code comments, in the README and in the author
review, that duplicate decoded keys are rejected *before* a map can hide one. That control
did not hold. It is not a capability escape — targets still resolve only beneath
caller-granted roots, and the plan digest is still over the exact bytes — but it is a
plan-interpretation ambiguity, and it weakens exactly the auditability that obligation A's
exact-plan identity exists to provide: a reviewer, or any other JSON reader, could disagree
with the observer about what the authorised bytes said.

**Correction applied.** Structural admission now uses a seeded strict visitor that rejects
duplicate **decoded** keys at every object depth and bounds nesting, the same way
`helm-app-spec` already bounds its own untrusted JSON. Duplicate and depth failures return
one document-level finding, consistently with that sibling crate.

**Regression tests.** `duplicate_keys_inside_array_nested_objects_are_rejected`,
`duplicate_keys_written_with_json_escapes_are_rejected`, and the negative control
`distinct_objects_may_reuse_the_same_key_names`, which proves sibling objects may reuse key
names and that key-shaped text inside a string value is data, not a key.

### IMPORTANT-3 — `0xEF53` does not establish ext4. **Confirmed from primary sources. Referred to the owner and RESOLVED BY OWNER DECISION on 2026-09-09.**

The finding was and remains **valid**: an `fstatfs` magic check cannot attest ext4. The
implementation could not solve it within ADR-0022, so it was referred rather than patched
over. The owner resolved it by clarification, not by expanding support. Full analysis, the
decision and its consequences are in section 6.

### IMPORTANT-4 — the Linux x86_64 half of the cohort was not enforced. **Source-derived. Corrected.**

**Location:** `crates/helm-observe/src/lib.rs`, the `#[cfg(target_os = "linux")]` gates on
`authority`, `linux` and `observe`, as submitted.

The accepted cohort is "Linux x86_64 and local ext4". The candidate gated the observation
backend on the operating system only. On Linux aarch64, riscv64 or any other Linux
architecture the whole capability API, target resolution and data reopen compiled and
operated, with no compile error, no admission refusal and no runtime signal — while the
*filesystem* half of the same cohort **was** enforced at admission and the README said
"root admission refuses anything else rather than degrading". The asymmetry was undocumented
and unexercised, and `AdmissionErrorCode::UnsupportedPlatform` was never constructed.

**Consequence.** Full observation authority outside the empirically anchored cohort,
presented as if the cohort were enforced. No memory-safety or disclosure consequence.

**Correction applied.** The backend is gated to `all(target_os = "linux", target_arch =
"x86_64")`, so an unsupported Linux architecture now gets the same treatment as Windows:
pure plan and model code, and no observation backend at all, instead of unvalidated
authority. This **narrows** to the accepted cohort and broadens nothing; one `cfg` edit
reverses it if the owner later extends the cohort on new evidence.

### MINOR findings

| # | Location | Finding | Disposition |
|---|---|---|---|
| M1 | `model.rs` | `bytes.truncate(MAX_ARTIFACT_BYTES)` would silently emit invalid JSON whose `sha256` still matched, if the ceiling were ever reachable. Boundedness was only implicit | Corrected: a unit test builds the widest artifact the schema and limits can express and asserts it fits. Section 12 gives the number |
| M2 | `observe.rs` | A file at exactly `MAX_FILE_BYTES` that grows may have one further byte **read** as the growth sentinel. The constants stay honest — `MAX_FILE_BYTES` bounds bytes *hashed*, and the aggregate ceiling is exact — but only a code comment said so | Corrected in the README; behaviour unchanged |
| M3 | `observe.rs` | Aggregate exhaustion suppresses every later target, including `directory_metadata` targets that would read no bytes | Recorded, not changed: deterministic, explicitly reported, and changing it is a product decision, not an ADR-0022 requirement |
| M4 | `README.md` | Said "performs no filesystem writes" and never stated the read side effects ADR-0022 does record | Corrected: atime, page cache and I/O are now stated explicitly, with the distinction from "no writes" |
| M5 | `tests/linux_authority.rs` | `assert!(is_ext(&base) \|\| !is_ext(&base))` is a tautology that cannot fail | Corrected to a real admission check |
| M6 | `tests/linux_authority.rs` | The obligation-B order-independence half compared only `exact_bytes().len()`, which two different root bindings could also satisfy | Corrected to compare the bytes and the bound root identities. This exposed that the artifact listed roots in the caller's vector order; see M10 |
| M7 | `tests/plan_contract.rs` | `every_digest_byte_participates_in_identity` flipped bytes in a copy of the digest array and compared it with the original array, testing `[u8; 32]` equality rather than the crate | Corrected to an independent SHA-256 oracle, a positional hex check, and a one-byte mutation sweep over the document |
| M8 | `HELM-OBSERVE-REVIEW.md` §9 | States "Linux, 13 tests"; the file contains 16, as `PROJECT_STATE` correctly says | Recorded only. An independent reviewer does not edit the author's signed review |
| M9 | `error.rs` | `AdmissionErrorCode::UnsupportedPlatform` is unconstructible, because the module that would produce it is itself gated to the supported platform | Recorded, not changed: harmless, and it becomes useful if a future backend reports the cohort at runtime |
| M10 | `observe.rs` | The artifact listed roots in the **caller's capability vector order**, so authorising one plan with the same root set supplied in a different order produced different artifact bytes and a different artifact digest, even though binding is by ID | Corrected: the artifact now follows the plan's declared root order, so a semantically meaningless input ordering no longer changes exact-byte identity |

### NO FINDING

Obligations A, B and C as such; the resolve-flag policy and the absence of any fallback;
absence-versus-failure mapping; aggregate budget arithmetic; artifact determinism, result
vocabulary and privacy; the dependency boundary and the absence of `unsafe`; descriptor
lifetime. Sections 4 onward record the evidence.

---

## 4. Obligations A, B and C

**A — exact plan identity. Confirmed.** `ValidatedPlan::sha256` is SHA-256 over
`bytes.to_vec()` of the exact accepted input, computed after validation and before any
authority exists. No canonicalisation happens anywhere: the retained bytes are the input
bytes, verified by an independent test asserting `exact_bytes() == input` for every accepted
mutation of a 1024-case corpus. `Digest` is `[u8; 32]` with derived `PartialEq`, so equality
is over all 32 bytes; there is no prefix, truncation, pointer or `Debug` comparison anywhere
in the authority path — `Debug` and `Display` render the digest but participate in nothing.
`authorize` copies `plan.sha256()` into `AuthorizedScope::authorized_plan_sha256`, and
`observe` writes exactly that value into the record and into the serialized bytes.

Attacked independently: whitespace difference, field-order difference, escaping difference
(`targets` — which, once IMPORTANT-2 is corrected, is rejected as a duplicate rather
than silently reinterpreted), and a one-byte XOR sweep across every position of the document
where the mutant still parses, each of which yields a different identity. The digest is
checked byte for byte against an independent `sha2` oracle computed by the test, and each of
the 32 bytes is checked to appear at its own position in the 64-character rendering, so
truncation would fail. `Digest::from_raw` is `pub(crate)`, `ValidatedPlan` has no public
constructor other than `parse_plan`, and no `Deserialize`, `Default` or `From` impl exists
for it, so a digest or a plan cannot be forged through the public API.

**B — root set binding. Confirmed.** `authorize` compares supplied capability IDs against
`plan.root_ids()` in both directions and rejects a missing root, an extra root and a
duplicate logical ID; a substituted ID surfaces as both extra and missing. Nothing in the
comparison uses a vector index. Attacked: missing, extra, duplicate, unknown, reversed
vector, same count with one substituted ID, and a target referencing an undeclared root
(rejected earlier still, at parse time, as `UNKNOWN_ROOT_REFERENCE`). After authorisation
`AuthorizedScope` exposes no way to add, remove or replace a capability: the fields are
private, the only accessors return `&ValidatedPlan` and crate-private borrows, and there is
no `Clone`, `Deserialize`, `Default`, public constructor or conversion trait on the scope or
on `RootCapability`.

The distinction the instruction draws is preserved: a trusted caller deliberately choosing
*which* directory descriptor stands for an authorised logical ID is authority by design, and
the crate cannot and should not second-guess it. What must be impossible is silent
substitution **after** the scope exists, and that is impossible by construction.

With M10 corrected, root order is now irrelevant to the artifact as well as to the binding,
so the regression test can assert byte equality between a forward and a reversed capability
vector rather than merely equal lengths.

**C — plan substitution. Confirmed, by construction.** `authorize` takes
`plan: ValidatedPlan` **by value** and moves it into the scope; `observe` takes only
`&AuthorizedScope`. There is no second plan parameter to supply, `plan()` hands out a borrow
only, and `ValidatedPlan` cannot be constructed except by parsing. An external integration
program that performs `authorize(plan A)` then `observe(plan B)` is therefore
**unexpressible** without private-field access or `unsafe`, both of which are unavailable
to a downstream crate and the latter of which the workspace forbids outright. The runtime
half of the invariant is also asserted: the artifact carries plan A's identity and only plan
A's target IDs, and never plan B's digest.

## 5. Target resolution, flags and fallbacks

Confirmed by source and by syscall trace. Every target resolution is a single `openat2` on
the retained root descriptor with flags `0o12400000` — `O_PATH | O_NOFOLLOW | O_CLOEXEC` —
and `resolve = 15`, that is `BENEATH | NO_SYMLINKS | NO_MAGICLINKS | NO_XDEV`. The constant
is applied unconditionally; there is no error-specific retry with fewer flags, no `openat`
fallback and no direct-open fallback. `rustix` 1.1.4 was read: both its backends issue the
`openat2` syscall directly and neither emulates it, so `ENOSYS` surfaces as
`unsupported_platform` rather than silently degrading.

A target with no `path` denotes the supplied root object itself and performs **no pathname
resolution at all**: it `dup`s the retained root descriptor. That is justified by explicit
root authority — the caller already granted that exact descriptor — and it cannot reach any
other object.

The descendant bind-mount case remains outside the validated cohort, exactly as ADR-0022
records. `NO_XDEV` is mandatory and crossings are conservatively rejected as
`scope_violation`; this review constructed no mount and claims no validation.

## 6. Does `0xEF53` prove ext4? No — and this is the open question

**Primary sources.** The Linux UAPI header `include/uapi/linux/magic.h` defines
`EXT2_SUPER_MAGIC`, `EXT3_SUPER_MAGIC` and `EXT4_SUPER_MAGIC` as the **same** value,
`0xEF53`. The ext4 on-disk documentation gives `s_magic = 0xEF53` for the same superblock
layout ext2 and ext3 use, and `statfs(2)` lists one `EXT2_SUPER_MAGIC` entry for the whole
family. The implementation's own comment already said so.

**Therefore root admission cannot honestly establish "ext4".** What `admit_root` establishes
is: the supplied descriptor is a directory, on a filesystem whose superblock magic is the
ext-family value. That is a **necessary but not sufficient** condition. An ext2 or an ext3
root is admitted and is not distinguished from an ext4 one.

**Was the claim overstated? Yes.** As submitted, the README said the cohort is "local ext4"
and that "root admission refuses anything else rather than degrading"; the cohort test
printed `HELM-OBSERVE-COHORT: ext4 PASS` on the strength of the magic alone; and
`PROJECT_STATE` recorded "the ext4 cohort execution is recorded as PASS". Admission is
*stricter than nothing* and *weaker than the claim*.

**Independent corroboration from the runner itself.** The author's cohort test asks
coreutils for the human name of the fixture filesystem type. On the hosted runner it answers
**`ext2/ext3`** for magic `ef53` — coreutils will not name that magic "ext4" either — while
`/proc/self/mountinfo` separately reports the mounted type name `ext4`. Two independent
sources on the same directory, disagreeing exactly where the magic stops carrying
information. The crate sees only the first of them.

**"Local" is likewise assumed, not established.** The same magic is reported for an ext
filesystem on network-backed block storage — iSCSI, NBD, a loop device over a network file.
Nothing in the crate distinguishes those.

**Is there a safe in-scope way to distinguish ext4?** No.

- `fstatfs` cannot, by the definition above.
- The mounted filesystem *type name* in `/proc/self/mountinfo` is strictly more information,
  and this review used it as *evidence about the runner*. It is not a fix: reading it needs a
  new pathname-based procfs read, which the accepted boundary does not grant, and it reports
  the name a mount was requested with rather than the on-disk feature set. On a kernel built
  with `CONFIG_EXT4_USE_FOR_EXT23` the ext4 driver serves `ext2` and `ext3` mounts too, and
  an ext2 image can be mounted as `-t ext4`.
- The feature flags that genuinely distinguish ext4 live in the superblock and need
  block-device access, which is far outside an explicit-target observer.

**Conclusion.** This is the third classification the instruction offers: an
**accepted-scope evidence and admission mismatch requiring owner architecture review**. It is
not an implementation defect with a safe fix, and it is not "no finding".

**What this review did, and deliberately did not do.** Admission is **unchanged** and not
weakened: a non-ext-family filesystem is still refused. The *claims* are corrected to state
exactly what the mechanism establishes, in the README, in the crate documentation, in the
`RootUnsupportedFilesystem` documentation, in the internal constant's name and comment, and
in the cohort test's printed line. The cohort is **not** renamed to "ext-family", ADR-0022 is
**not** broadened, and no new provenance is invented.

**The question put to the owner.** Choose one, with evidence:

1. accept that 0.1's admission enforces an ext-family necessary condition and restate the
   supported cohort accordingly, which is a scope change only an owner may make; or
2. require a stronger ext4 discriminator, which needs new authority beyond ADR-0022 and its
   own falsification evidence; or
3. keep the cohort as ext4 and accept, explicitly and in writing, that admission does not
   enforce it and that an ext2 or ext3 root will be observed.

<a id="owner-decision"></a>

### Owner decision, 2026-09-09: option 3, recorded as an ADR-0022 clarification

The repository owner chose **option 3** by explicit written instruction and recorded it as
an owner-approved clarification to Accepted ADR-0022, which remains **Accepted**. The
clarification is
[in the ADR itself](../adr/ADR-0022-observation-authority.md#cohort-attestation-clarification);
its operative sentence is:

> Filesystem cohort membership is an external support precondition, not an observation
> attestation. Root admission applies only necessary mechanism guards available inside the
> explicit capability boundary.

What follows from it, and what this review verified in the code and documents:

| Point | State on this branch |
|---|---|
| Supported and evidenced cohort | **Linux, x86_64, ext4**, unchanged. **Not** broadened to ext2, ext3 or "ext-family" |
| What helm-observe attests about the cohort | **Nothing.** It does not attest ext4 and does not attest storage locality |
| Who establishes cohort membership | The trusted caller and the provisioning environment, **outside** the crate, from provisioning, controlled deployment or separate evidence |
| Meaning of the `0xEF53` guard | Only *"this descriptor is on a filesystem reporting the ext-family superblock magic."* A necessary sanity and admission guard. Not verified ext4, not a validated cohort, not local storage, not a supported environment, not provenance |
| ext2 or ext3 roots | May mechanically pass the guard. Such an observation is **outside** the validated cohort; no support or safety claim transfers. Recorded, not supported |
| New authority to distinguish ext4 | **None added.** No mountinfo read, no implicit procfs beyond the accepted reopen capability, no block device, no sysfs, no mount scan, no superblock read, no provisioning discovery |
| Artifact claims | No artifact claims ext4 from the magic. The serializer emits no filesystem-type field at all |
| Descendant bind-mount exclusion | Preserved unchanged |
| Other ADR-0022 semantic limits | Preserved unchanged |

**Disposition of this finding.** Valid finding; not solvable inside ADR-0022 by the
implementation; referred to the owner; resolved by owner decision rather than by weakening
admission, renaming the cohort or inventing provenance. The finding text above is preserved
in full, and the pre-correction state is preserved at `a1726f3`.

**Consequences for the documents**, all applied on this branch: ADR-0022 carries the
clarification and an annotation on the historical word "local" so it cannot be read as a
property root admission establishes; the crate README, the crate documentation, the
`RootUnsupportedFilesystem` documentation and the magic constant's comment all state the
guard's exact meaning; and the cohort tests print **two separate statements** that must not
be collapsed — a runner-side line recording the mounted filesystem type as evidence that the
*test* ran on ext4, and a product-side line recording that the *guard* saw `f_type = 0xEF53`.

## 7. Linux x86_64 gating

As submitted: documented as unsupported, not enforced, while the sibling half of the same
cohort was enforced. Classified above as IMPORTANT-4 and corrected by narrowing the gate.
After the correction the crate offers, outside Linux x86_64, exactly what it offers on
Windows: pure plan and model types with no observation backend, no capability types and no
fake semantics. Nothing was broadened, and the change is one `cfg` predicate to undo.

## 8. Procfs admission, with controlled foreign processes

The self-identity probe was reconstructed exactly, then attacked. The probe object as
submitted was the caller-supplied capability descriptor itself; it is now a freshly created
anonymous memory object. In both cases the probe deliberately **follows** the procfs magic
link — no `NO_SYMLINKS`, no `NOFOLLOW` — so it observes the pinned object rather than the
link, and it compares device, inode and (after the correction) kind.

Shell redirection cannot express the attack portably, because POSIX shells need only honour
single-digit descriptor numbers; the first attempt silently held two descriptors instead of
the needed range and produced a false negative. The helper is now this same test binary
re-executed in helper mode, holding 320 descriptors on a chosen object.

| Case | Foreign helper | As submitted | After correction |
|---|---|---|---|
| A, foreign child created **before** the admission probe | 320 descriptors on its own `/proc/self/fd` | **ADMITTED** — invariant violated | refused, `PROCFS_FOREIGN_OR_UNUSABLE` |
| B/C, foreign child holding the **same object** the caller holds | 320 descriptors on a shared regular file | refused | refused |
| D, same numeric descriptor, different object | implied by A and B | refused when the object differs | refused |
| E, child that exits after the capability is opened | helper killed and reaped first | refused | refused |
| F, the real `/proc/self/fd` | none | admitted | admitted |
| tmpfs decoy, `/dev/shm` | none | refused, `PROCFS_WRONG_FILESYSTEM` | refused |
| `/proc/1/fd` | none | refused where the kernel lets an unprivileged process open it at all | refused |

Case A is BLOCKER-1. The answer to the instruction's question —
*could `/proc/<child>/fd` pass admission merely because the selected probe descriptor was
inherited by that child?* — is: not by inheritance, which case B/C shows is refused, but by
the foreign process presenting **its own descriptor directory** at that number, which case A
shows was admitted. Consequence as stated in BLOCKER-1: a false admission, no unauthorized
read, an availability and race surface.

There is no pathname fallback after any admission failure: `proc_fd_from_trusted_current_process`
returns an error and the caller has no other way to obtain the capability.

## 9. Special files and symlinks, from syscall evidence

Proved on the **compiled Rust implementation**, not on the OBS-FS-01 C spike, by a driver
around the public API traced with the frozen OBS-FS-01 ptrace tracer, imported read-only.
Runner: Ubuntu 24.04, kernel `6.17.0-1022-azure`, `x86_64`, fixture on a filesystem whose
mounted type name is `ext4`. Every `openat2` in every case carried flags `0o12400000` and
`resolve = 15`; every case had zero `getdents64` and zero `connect`.

The evidence was collected on the corrected tip. It carries over to the candidate itself
without qualification, because none of the corrections touches this path: the whole
`e2a62081..41f6c4e` diff of `linux.rs` is confined to imports, the filesystem-magic
constant's name and comment, and `admit_procfs`, leaving `resolve`, `describe` and
`reopen_and_stream` byte-identical, and the only change in `observe.rs` is the root ordering
of M10, leaving `observe_one`, `reject_for` and `stream_regular` byte-identical. That is
checkable with `git diff e2a62081..41f6c4e -- crates/helm-observe/src/`. No syscall trace was
captured against the candidate before correction, because the reproduction runs aborted at
the failing test step before reaching that stage; the review workflow now runs the trace
first for exactly that reason.

| Case | `openat2` result | procfs data reopen | target reads | statx | outcome | seconds |
|---|---|---|---|---|---|---|
| regular file | fd 5 | 1, naming `"5"` → fd 6 | 2 reads, 17 bytes | 3 | `observed_file` | 0.05 |
| directory metadata | fd 5 | **0** | **0** | 1 | `observed_directory` | 0.046 |
| trailing symlink | fd 5, pinned | **0** | **0** | 1 | `symlink_forbidden` | 0.046 |
| non-final symlink | `-ELOOP`, no descriptor | **0** | **0** | 0 | `symlink_forbidden` | 0.046 |
| FIFO | fd 5, pinned | **0** | **0** | 1 | `special_file` | 0.047 |
| Unix socket | fd 5, pinned | **0** | **0** | 1 | `special_file` | 0.046 |
| metadata over the per-file ceiling | fd 5, pinned | **0** | **0** | 1 | `file_limit` | 0.047 |
| permission-denied regular file | fd 5, pinned | 1 attempt, `-EACCES` | **0** | 1 | `permission_denied` | 0.047 |
| absent | `-ENOENT`, no descriptor | **0** | **0** | 0 | `absent` | 0.045 |

The correlation is by descriptor, not by counting syscalls: the analysis takes the descriptor
each `openat2` returned, requires every procfs reopen to name exactly such a descriptor
number under exactly the admitted capability, and counts reads only on descriptors the
reopen produced. Dynamic-loader activity is excluded by starting the analysis at the first
`openat2`. It also asserts that after resolution begins there is **no** pathname open on any
directory other than the procfs capability, which is the no-fallback property, and that no
read is ever issued against an `O_PATH` pin.

Two independent checks back the FIFO result at the API level. A writerless FIFO and a bound
socket complete under a 20-second deadline instead of blocking. And a FIFO whose writer is
blocked in `open(2)` stays blocked across the whole observation: the writer only prints
`opened` after the test itself opens the read end afterwards, which is the control proving it
really was waiting. So the FIFO case does not merely avoid blocking; it demonstrably never
opened the FIFO for data.

**Amendment 1 is confirmed on the Rust product, with the destination untouched.** A trailing
symlink is pinned as `O_PATH`, classified `symlink`, and rejected, with zero reopens and zero
reads — so no destination descriptor is ever produced. A non-final symlink is rejected at
resolution in the `ELOOP` class and yields no descriptor at all, which the trace shows as an
`openat2` returning `-40` with no following `statx`. The escaping-symlink canary outside the
authorised root is byte-identical afterwards. No assertion in this review checks only the
result enum.

Hazardous device nodes were **not** data-opened, and none was created.

## 10. Reopen identity

Only the `Regular` arm of the classification `match` can call `reopen_and_stream`; that is
structural, and the trace confirms it for every non-regular kind. The reopen names a decimal
descriptor number, which an `i32` from an `OwnedFd` cannot turn into path syntax, under the
admitted capability only; the original target pathname is never reopened. The magic link is
followed on purpose. The reopened descriptor is compared with the pin — kind, device major,
device minor, inode — **before** the first read, and a mismatch returns `io_failure` with
zero bytes.

Descriptor-number reuse is not a hazard here: the `O_PATH` pin is still held while the reopen
runs, so neither its descriptor number nor the inode it references can be recycled. A
hardlink alias is the same inode and so is legitimately the same object, and the artifact
says `"origin":"unestablished"` rather than treating the link count as provenance. A target
removed after the pin keeps its inode alive through the pin, and any resulting length or
timestamp change is reported as `changed_during_read`. Even under BLOCKER-1's false
admission, this identity check is what kept a foreign descriptor namespace from yielding
foreign bytes.

## 11. Absence versus failure

Confirmed against the source and, for six of the cases, against the trace.

| Condition | Reported as | Confirmed |
|---|---|---|
| `ENOENT` at constrained target lookup, missing leaf | `absent` | trace: `openat2` → `-2` |
| `ENOENT` at constrained target lookup, missing parent | `absent` | API test |
| `EACCES`/`EPERM` at resolution | `permission_denied` | API test |
| `ENOTDIR` | `wrong_kind` | API test |
| `ELOOP` | `symlink_forbidden` | trace: `openat2` → `-40` |
| `EXDEV` | `scope_violation` | source; not constructible without a mount, and outside the validated cohort |
| procfs-stage failure of any kind | `reopen_unavailable` | source |
| reopen permission denial | `permission_denied`, zero bytes | trace: reopen → `-13`, zero reads |
| I/O error | `io_failure` | source |
| changed file | `changed_during_read` | source |
| budget exhaustion | `file_limit` / `total_limit` / `not_attempted_total_limit` | API tests |
| unsupported platform | `unsupported_platform` | source |

**No path other than a constrained lookup miss can produce `absent`.** Missing parent and
missing leaf are separately exercised and both are genuine absence, since both come from the
same constrained resolution.

## 12. Hashing, budgets and artifact size

Audited line by line and exercised at the boundaries.

- The per-file ceiling is checked on **metadata**, before any reopen: `size > MAX_FILE_BYTES`
  returns `file_limit`. The trace confirms zero reopens and zero reads for that case.
- Allocation is never proportional to an observed length: one fixed 64 KiB buffer,
  `vec![0u8; READ_BUFFER_BYTES]`, allocated once per observation.
- Actual returned bytes are charged, including on the partial and error paths.
- The aggregate ceiling is **exact**. Before reading, `stream_regular` requires
  `size + 1 <= MAX_TOTAL_BYTES - spent`; the stream ceiling is then `min(size + 1,
  remaining)`, which equals `size + 1`; so `spent` can never exceed `MAX_TOTAL_BYTES`. Every
  addition is saturating, so no overflow is reachable.
- A per-file refusal does not exhaust the batch; a genuine aggregate refusal marks the target
  `total_limit` and every later target `not_attempted_total_limit`. Every declared target
  always receives exactly one explicit outcome.

**The growth sentinel, examined as the instruction requires.** `ceiling = size + 1` means a
file sitting at exactly `MAX_FILE_BYTES` that grows can have one byte **read** beyond the
per-file constant. That byte *is* counted — `spent` is increased by the bytes actually
returned — and it can never exceed the aggregate ceiling, because the `size + 1` reservation
is made before the read. It never enters a digest: only bytes below the pre-read length are
hashed. So the documented contracts hold exactly as written — `MAX_FILE_BYTES` is documented
as the maximum bytes *hashed*, and `MAX_TOTAL_BYTES` as the maximum bytes read — and the
one-byte read overrun of the per-file figure is a real, bounded, charged effect that was
described only in a code comment. It is now stated in the README. **Not waved away, and not
a violation of either stated ceiling.**

Exercised, with sparse fixtures so no ceiling-sized file is ever materialised: ceiling plus
one refused on metadata alone; exactly ceiling accepted and streamed to `bytes_read ==
MAX_FILE_BYTES`; aggregate ceiling exact, so a second ceiling-sized file is refused with
`total_limit`; and the target after exhaustion explicitly `not_attempted_total_limit`. The
heavy case is `#[ignore]`d and runs in release mode in the review workflow, where it streams
a real 512 MiB and passes, so no public constant was reduced to make a test cheap and no
internal budget was substituted.

**Artifact boundedness, proved numerically rather than assumed.** The widest artifact the
schema and the declared limits can express — 8 roots and 64 targets, every identifier at
`MAX_ID_BYTES`, every numeric field at its type maximum, every outcome the widest
(`observed_file`) — is **37,312 bytes** against the 256 KiB ceiling, about 14 % of it. A unit
test constructs exactly that record and asserts it fits, so the silent `truncate` branch is
unreachable and stays unreachable if a limit is edited.

## 13. Concurrent mutation

The implementation never upgrades sampled metadata into a snapshot guarantee, and the
documents say so.

Metadata is sampled before and after the stream, and a change in size, link count, mtime or
ctime, or a short or growing stream, yields `changed_during_read` with the partial count and
**no** complete digest. Replacement before the pin observes the new object, which is
correct and unremarkable; replacement after the pin cannot affect the retained descriptor,
which the author's rename test demonstrates for the root and the pin does for the target. A
same-length in-place overwrite is detected through mtime and ctime in practice, but nothing
guarantees it: ctime cannot be forged by an unprivileged writer, yet a write inside the
timestamp granularity is not excluded in principle. A hardlink alias mutation is a mutation
of the same inode and is detected on the same terms.

The vocabulary is honest about exactly this. The artifact says
`"consistency":"sequential_objects"`, an observed file says
`"change_check":"no_change_detected"` rather than "unchanged", and the string `snapshot`
appears nowhere in an artifact, which a test asserts. The README states that undetected
concurrent mutation remains a real limitation. **No output calls a digest a stable file
version.**

## 14. Parser review

Beyond IMPORTANT-2, the parser was attacked independently and no further finding was raised.

Rejected as required: unknown fields at document, root and target level, including
descriptor-shaped ones; unknown schema and version; duplicate root and target IDs; invalid
root references; absolute paths; `.`; `..`; repeated separators; empty components; NUL;
backslash; non-portable bytes; leading spaces; trailing spaces and dots. Every documented
ceiling was tested on **both** sides and each is exact: 1024 and 1025 path bytes, 255 and 256
component bytes, 32 and 33 components, 80 and 81 identifier bytes, 8 and 9 roots, 64 and 65
targets, the document-size limit, and the nesting bound.

A **new** independent corpus was run: reviewer-chosen seed `0x123456789ABCDEF0`, 1024 cases,
structure-aware mutations that splice JSON metacharacters, escape sequences, invalid UTF-8,
a lone surrogate and `/../` as well as flipping, deleting and inserting bytes. It asserts no
panic, deterministic acceptance, deterministic error ordering, and — for every mutant that
still parses — that the retained bytes are exactly the mutated input. No failing seed was
found, so none needed preserving. The author's seed was not reused as the only corpus.

Parsing performs no filesystem, process, network or environment lookup on any path,
including every error path: `plan.rs` links no such API, which the whole-crate search in
section 2 confirms, and the pure suite runs on a non-Linux host where no backend exists at
all.

## 15. Artifact serialization, privacy and vocabulary

The serializer is a fixed sequence of literal field names in a fixed order, with no map
iteration, so it is deterministic; there is no `HashMap`, no timestamp, no hostname, no
random identifier, no pointer address and no absolute path. `sha256()` is SHA-256 over
exactly `exact_bytes()`, and equal inputs and state give equal bytes, which a test asserts by
observing twice. Since M10 was corrected, the bytes no longer depend on the caller's
capability vector order.

It is unambiguous and injective enough for its typed schema: every value is either a JSON
number rendered from an integer, a fixed enumeration spelling from a closed set, a
64-character lowercase hex digest, or a validated logical identifier restricted to lowercase
ASCII alphanumerics, dot, underscore and hyphen. `push_id` filters to exactly that alphabet,
so no delimiter or escape collision is expressible; the filter is total rather than lossy
because the parser has already guaranteed the alphabet. It is bounded, numerically, by
section 12.

**Target paths never appear in the artifact.** Only logical IDs and fixed codes do. The
privacy tests use a distinctively named sibling and a distinctively named directory child:
neither name nor content appears, no `/` appears at all, and only declared targets appear.
The syscall trace independently shows the sibling is never opened and no directory is ever
enumerated. Diagnostics are separately bounded: `PlanError` carries a `&'static str` locator
from the closed schema plus an index, `AdmissionError` carries at most a validated logical
root ID, and neither can echo a host path — a test asserts no `/` in a rendered error. The
distinction between product artifacts and developer or test output is real and worth keeping
explicit: this review's own tests print fixture paths in failure messages, which is
appropriate for a test harness and would not be appropriate in an `ObservationArtifact`.

**Result vocabulary.** No `PASS`, `FAIL`, `SATISFIED`, `UNSATISFIED`, `COMPATIBLE`,
`READY_TO_LAUNCH` or `INSTALLED_CORRECTLY` appears in any artifact string or public result
enum, and no synonym implies the same conclusion: the outcomes are `observed_file`,
`observed_directory`, `absent`, four rejection codes, six failure codes and three budget
codes, all descriptive of what was attempted. The one word that could be misread,
`change_check`, is answered `no_change_detected`, which states the observation rather than a
guarantee. The subject spec digest is carried as opaque context and is never parsed,
compared or presented as attestation that the observed root belongs to that spec.

## 16. Read side effects

ADR-0022 already records that reading is not effect-free. As submitted, the README said only
"Observation code performs no filesystem writes", which is true and which invites the
stronger reading. That is M4 and it is corrected: the README now states that a permitted read
may update atime depending on mount options, populates the page cache, causes real I/O, and
can block in the kernel despite finite byte limits. The correction also notes the one new
object the corrected admission creates — an anonymous memory file, which touches no observed
filesystem.

## 17. Dependencies and `sha2` feature unification

Confirmed: no dependency on `helm-app-spec`, none on `helm-evidence`, no new utility crate,
no `unsafe`, no subprocess or network dependency. `rustix` is `default-features = false` with
only `std` and `fs`, which is the minimum for `openat2`, `statx`, `openat`, `fstatfs`,
`memfd_create` and `PROC_SUPER_MAGIC`. `Cargo.lock` gained one member and its `rustix` edges
and no new third-party version. The correction added no dependency: `memfd_create` is already
inside the enabled `fs` feature.

The recorded `sha2/force-soft` limitation was reproduced by reasoning over the manifests:
`helm-app-spec` enables `sha2/force-soft` deliberately, to avoid runtime CPU feature
discovery, and Cargo feature unification is per-graph, so every crate in the workspace graph
— including this one — gets the software backend. It remains **performance and build
composition only**: the feature selects an implementation, not a digest, so outputs are
identical either way, and the workflow keeps a separate `cargo test -p helm-evidence`
invocation so the standalone backend is still exercised. It was **not** changed here.

## 18. Cross-platform contract

Pure plan and model code builds and its contract tests pass on Windows, which is where this
review's parser reproductions were first obtained, and on Linux. Observation authority is
unavailable outside the accepted backend: after the IMPORTANT-4 correction the capability
types, `authorize` and `observe` do not exist at all outside Linux x86_64. Windows has no
observation backend and no simulated filesystem semantics. Support was not broadened merely
because `rustix` compiles elsewhere — the opposite was done.

## 19. Test quality

The author's 12 pure and 16 Linux tests were reviewed for false confidence. Findings M5, M6
and M7 are the three that mattered and are corrected. Beyond those: no helper is shared
between the implementation and an oracle — the digest oracles call `sha2` directly, and the
independent suites compute expectations from the documented contract; there is no mock whose
return value is the expected result; there are no retries and no test-order dependence; and
the fixtures are neutrally named throughout, with no 7-Zip or A0 naming anywhere.

Two environment-dependent skips exist and both are honest rather than a pass in disguise:
`/proc/1/fd` is skipped when the kernel refuses an unprivileged open, and the `/dev/shm`
tmpfs decoy is skipped where `/dev/shm` is absent. Each is stated in this report rather than
counted as evidence. The cohort test does **not** skip: on a non-ext-family filesystem it
asserts that admission refuses.

The independent suites do not take their expected answers from the code under test. Where a
property could only be asserted as an enum, syscall evidence was added instead, and the FIFO
writer control exists precisely so that "did not block" is distinguished from "did not open".

## 20. Corrections made on this branch

| Commit | Content |
|---|---|
| `a1726f3` | First-pass findings recorded **before** any correction, with the failing reproductions, the review-only driver, the syscall regression and the review workflow |
| `81aec01` | Test-harness fix: the shell-based foreign helper held two descriptors rather than the attacked range, because POSIX shells need only honour single-digit descriptor numbers in redirections. The helper is now this test binary re-executed in helper mode |
| `834322b` | BLOCKER-1, IMPORTANT-2, IMPORTANT-3 claims, IMPORTANT-4, M1, M2, M4, M5, M6, M7, M10 |
| `41f6c4e` | Dead-code removal exposed by the M10 correction, the built-commit field in the syscall evidence, this report and the `PROJECT_STATE` entry |
| `c4e292b`, `585016d` | Validation record for the corrected tip, and the review's own residual limits stated precisely |
| this commit | The owner's 2026-09-09 cohort attestation clarification applied to ADR-0022, the crate, the cohort tests and this report. **No product mechanism changed**: admission, resolution, classification, reopen, budgets and the artifact are byte-identical to `585016d` apart from documentation comments |

`e2a62081` is preserved unchanged, as are the first-pass finding commits. No history was
rewritten, nothing was force-pushed, main was not touched and no merge of main was
performed. Provenance stays linear.

## 20a. Validation on the corrected tip

Both workflows are green on `41f6c4e8371324abd2dc4983694b843d0f9b378d`, hosted
`ubuntu-24.04`, kernel `6.17.0-1022-azure`, `x86_64`: the unchanged product workflow
(run `34281472783`) and the review workflow (run `34281472724`).

| Check | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | pass |
| `cargo test --workspace --locked` | pass |
| `cargo test -p helm-evidence --locked` | pass, standalone `sha2` backend still exercised |
| `cargo build -p helm-app-spec --release --locked`, `-p helm-evidence` | pass |
| `python3 -m unittest discover -s tools/tests` | pass |
| `python3 tools/validate_docs.py` | pass |
| app-spec frozen fixture, `tools/helm_app_spec_fixture.py` | pass |
| evidence frozen fixture, `tools/helm_evidence_fixture.py` | pass |
| `git diff --check` | pass |
| independent app-spec properties and cross-crate comparison | pass |
| independent plan-contract adversarial suite, 7 tests | pass |
| independent Linux adversarial suite, 13 tests plus 1 opt-in | pass |
| author suites, 12 pure and 16 Linux | pass |
| release-mode budget ceiling test, streams 512 MiB | pass |
| syscall-level regression, 9 cases | `HELM-OBSERVE-SYSCALL-REVIEW: PASS`, **0 violations** |
| privacy and artifact checks | pass, inside the suites above |
| supported-cohort check | admission behaves correctly; the *claim* was the open question of section 6, since resolved by owner decision |

The same set was re-run on the clarification tip, section 20b.

**Runner tool inventory**, recorded by the workflow before anything is concluded from it:
`rustc`, `cargo`, `gcc`, `cc`, `python3`, `stat`, `mkfifo` and `mkfs.ext4` present;
**`strace` and `ltrace` absent**; `yama/ptrace_scope` permitting a traced own child. The
syscall obligation was therefore met by importing the frozen OBS-FS-01 ptrace tracer
read-only, not by installing anything and not by weakening the requirement.

## 21. Residual limitations

1. **The crate attests no cohort membership**, by owner decision, section 6. Admission
   applies an ext-family necessary guard only; the supported cohort stays ext4 and is a
   caller precondition. Supplying an in-cohort root is the deploying environment's
   responsibility, and an ext2 or ext3 root will be observed without support.
2. **"Local" is not established by anything in the crate.** Network-backed block storage
   under an ext filesystem is indistinguishable to this mechanism, and nothing claims
   otherwise.
3. **Descendant bind mounts remain unvalidated**, exactly as ADR-0022 records. This review
   constructed no mount namespace and claims nothing new. `EXDEV` mapping is therefore
   source-derived, not reproduced.
4. **Undetected concurrent mutation remains possible.** A same-length overwrite inside
   timestamp granularity is not excluded in principle.
5. **Mutation-during-read races were reviewed from source, not injected.** Replacement,
   truncation or growth between the pre-read `statx`, the reopen and the post-read `statx`
   is analysed in section 13 and covered by the implementation's own comparison, but this
   review built no deterministic injector for it. OBS-FS-01 exercised such races against the
   C spike; nobody has yet exercised them against this Rust code.
6. **Three failure mappings are source-derived only**, because none is injectable on a
   hosted runner without privileges or an old kernel: `EXDEV` from a mount crossing,
   `ENOSYS`/`EOPNOTSUPP` from a kernel without `openat2`, and a procfs capability that
   becomes unusable after a successful admission.
7. **One kernel, one runner.** All Linux evidence comes from hosted Ubuntu 24.04, kernel
   `6.17.0-1022-azure`, x86_64. Other kernels and mount topologies are unevidenced.
8. **`strace` and `ltrace` are absent from the hosted runner.** The syscall obligation was
   met by adapting the frozen OBS-FS-01 ptrace tracer, read-only, rather than by weakening
   the requirement; the runner inventory is recorded by the workflow's first step so this is
   checkable rather than asserted.
9. **No WSL or Hyper-V lab was used or modified**, and no A0 evidence was opened or re-run.
10. **The 512 MiB and 1 GiB budget cases** run in release mode as opt-in tests, so an ordinary
   `cargo test` does not exercise them.
11. `AdmissionErrorCode::UnsupportedPlatform` stays unconstructible, M9, and aggregate
   exhaustion still suppresses zero-cost later targets, M3.

## 22. Recommendation

The implementation is, after the corrections on this branch and under the owner's 2026-09-09
clarification, a faithful and disciplined realisation of the accepted mechanism. Obligations
A, B and C hold. The special-file, symlink and no-fallback negatives are proved on the
compiled product at syscall level. There is no hidden authority and no alternate path. The
result vocabulary, the privacy posture and now the cohort language are honest about exactly
what is and is not established.

Every BLOCKER and IMPORTANT finding is resolved: BLOCKER-1 and IMPORTANT-2 and IMPORTANT-4 by
correction with regression tests, IMPORTANT-3 by owner decision recorded as an ADR-0022
clarification that expands no support and adds no authority. The MINOR findings are either
corrected or recorded with a stated reason for leaving the behaviour alone. The residual
limitations in section 21 stay exactly what they are — source-derived limitations and
environment bounds — and none of them is inflated into reproduced evidence.

**READY_FOR_OWNER_MERGE.**

This is a reviewer recommendation. The merge itself remains the owner's action; nothing in
this review performs or authorises it, and main is untouched.
