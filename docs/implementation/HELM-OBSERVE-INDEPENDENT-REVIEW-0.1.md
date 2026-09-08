# helm-observe 0.1 independent review

**Role:** independent adversarial review of the first Rust implementation, by a
reviewer who is not the implementation author.\
**Reviewed candidate:** `e2a62081ece52391b30ede153eee139103c98e31` on
`product/helm-observe`.\
**Review branch:** `review/helm-observe-independent`, descending from the candidate.\
**Authority:** Accepted [ADR-0022](../adr/ADR-0022-observation-authority.md), bounded 0.1
scope. **Classification: pending — first pass recorded, corrections not yet applied.**

This first section is deliberately committed **before** any correction, so the state of
the candidate as submitted is preserved in history rather than tidied away.
`e2a62081` itself is unchanged.

## 1. Independently reconstructed state

| Claim | Method | Result |
|---|---|---|
| `origin/main` is the accepted architecture base | `git rev-parse origin/main` | `0f93431d3bb9285a71e9a7f2c2db3dc3d0bfb45b`, matches the instruction |
| `53ade006` descends from main | `git merge-base --is-ancestor` | yes |
| `e2a62081` descends from `53ade006` | `git merge-base --is-ancestor` | yes |
| `product/helm-observe` is the reviewed candidate | `git rev-parse` | `e2a62081…`, equal to `origin/product/helm-observe` |
| main does not contain helm-observe | `git ls-tree origin/main crates/` | only `helm-app-spec`, `helm-evidence`; main carries the architecture and OBS-FS-01 documents only |
| no rewrite in candidate history | `git log 0f93431..e2a62081` | two linear commits, both reachable, no force-push evidence, evidence commits `63ac679`, `2496f68`, `1bd9c0a` all ancestors of main |
| dependency graph | `git diff Cargo.lock` | one new member and its `rustix` edges; no new third-party version |

Read in full: `AGENTS.md`, `docs/PROJECT_STATE.md`, [DECISIONS](../DECISIONS.md),
[ADR-0022](../adr/ADR-0022-observation-authority.md),
[architecture report](../research/HELM-OBSERVE-ARCHITECTURE.md),
[independent review with Amendment 1](HELM-OBSERVE-INDEPENDENT-REVIEW.md),
[OBS-FS-01 preflight and halt](../experiments/OBS-FS-01-PREFLIGHT-AND-HALT.md),
[OBS-FS-01 execution report](../experiments/OBS-FS-01-EXECUTION-REPORT.md), the
[execution definition sources](../experiments/obs-fs-01/), the
[author review](HELM-OBSERVE-REVIEW.md), every `crates/helm-observe` source and test file,
the Cargo manifests, lock and workspace lints, and the `helm-app-spec` and `helm-evidence`
dependency configuration relevant to `sha2` feature unification.

## 2. Independent source trace

Traced from the Rust source, not from the documentation.

| Transition | Input | Owned authority | Filesystem operation | Failure modes | Alternate path |
|---|---|---|---|---|---|
| bytes → `ValidatedPlan` | `&[u8]` | none | none | `PlanErrors` only | none: `plan.rs` links no I/O API |
| descriptor → `RootCapability` | `OwnedFd` | that descriptor | `statx(AT_EMPTY_PATH)`, `fstatfs` | not a directory, metadata unavailable, unsupported filesystem | none: no path-based constructor exists |
| descriptor → `ProcFdCapability` | `OwnedFd` | that descriptor | `fstatfs`, `statx`, one `openat` of a decimal name | wrong filesystem, foreign or unusable | none |
| plan + capabilities → `AuthorizedScope` | by value | plan and all root descriptors | none | missing, extra, duplicate root | none |
| scope → target pin | authorised scope | root descriptor | one `openat2` with `O_PATH│O_NOFOLLOW│O_CLOEXEC` and `BENEATH│NO_SYMLINKS│NO_MAGICLINKS│NO_XDEV` | `ENOENT`→`Absent`, `ENOTDIR`→`wrong_kind`, `EXDEV`→`scope_violation`, `ELOOP`→`symlink_forbidden`, `EACCES`/`EPERM`→`permission_denied`, `EAGAIN`→`resolution_race`, `ENOSYS`/`EOPNOTSUPP`→`unsupported_platform`, other→`io_failure` | none: single call, no retry, no `openat` fallback |
| pin → classification | pinned `OwnedFd` | that pin | `statx(AT_EMPTY_PATH)` | `io_failure` | none |
| classification → data reopen | pin classified `Regular` only | pin plus procfs capability | one `openat` naming the pin's decimal descriptor number under the procfs capability | `EACCES`/`EPERM`→`permission_denied`, other→`reopen_unavailable` | none, and no pathname is ever reopened |
| reopen → stream | reopened descriptor | that descriptor | `statx` identity re-check, then bounded `read` loop, then `statx` | identity mismatch→`io_failure`, read error→`io_failure`/`permission_denied`, change→`changed_during_read` | none |
| results → artifact | owned record | none | none | none | `ObservationArtifact::new` is crate-private |

Whole-crate searches confirm the author's no-hidden-authority claim on every point:

| Searched for | Result |
|---|---|
| `Path`, `PathBuf`, `std::path` | **none** |
| `std::fs` | one occurrence, `File::from(OwnedFd)`, which wraps an already-open descriptor and opens no name |
| `read_dir`, `readdir`, `getdents`, `walkdir`, `glob` | **none** |
| subprocess, `Command`, network, socket APIs | **none** |
| environment, `HOME`, `PATH`, cwd | **none** in code; only in prose and error-code spellings |
| raw descriptor conversions | two `as_raw_fd()` calls, both only to render a decimal descriptor number; an `i32` from an `OwnedFd` cannot contain path syntax |
| `unsafe`, `libc`, raw syscalls | **none**; the workspace forbids `unsafe_code` and the crate needs no exception |
| filesystem writes | **none**; the only `truncate` calls act on in-memory `Vec`/`String` |
| `openat2` fallback inside `rustix` 1.1.4 | none: both backends issue the syscall directly and surface `ENOSYS` |

## 3. Findings

Severity as defined by the review instruction. Every finding below is stated with its
exact location, the concrete failure, whether it was reproduced or derived from source,
the consequence, the minimum correction and the regression test required.

### BLOCKER-1 — the procfs self-identity probe can admit a foreign process

**Location:** [`crates/helm-observe/src/linux.rs`](../../crates/helm-observe/src/linux.rs),
`admit_procfs`, the probe construction (`let number = … as_raw_fd(&fd.as_fd())`).

**What the accepted mechanism requires.** The frozen OBS-FS-01 definition, section 13,
"Procfs acceptance test (I4)", requires that *"the observer **creates a descriptor it
already controls**, opens that decimal name under the supplied directory with `O_PATH`, and
requires the device and inode to match the known object."* ADR-0022 accepts reading only
through an *"admission-validated **trusted current-process** procfs descriptor capability"*.

**What the implementation does.** It creates no probe object. It uses the caller-supplied
capability descriptor **as its own probe**: it takes that descriptor's number `N`, opens
the name `"N"` under the same directory, and compares the result to the directory itself.
The probe object is therefore an object whose identity a foreign process can arrange.

**Concrete attack.** A foreign process that opens its own `/proc/self/fd` at every
descriptor number in a modest range makes `/proc/<child>/fd/<N>` resolve, for any such `N`,
to the child's own descriptor directory — which is exactly the object the caller supplied.
Device and inode then match and admission succeeds for a **foreign** descriptor namespace.
In POSIX shell the whole attack is `exec 3</proc/self/fd 4</proc/self/fd …`.

**Consequence.** A false admission of a foreign process's descriptor directory, violating
the accepted "trusted current-process" invariant. It does **not** by itself yield an
unauthorized read: `reopen_and_stream` re-verifies device, inode and kind against the pin
before the first byte, and two distinct objects cannot share a device and inode pair, so a
foreign namespace produces `io_failure` rather than foreign data. The realised damage is
therefore invariant violation plus an availability and race surface, not disclosure — but
the invariant is an accepted security invariant, which is the BLOCKER criterion.

**Minimum correction.** Create a fresh process-private probe object at admission time and
compare against that object instead of against the supplied capability. `memfd_create` is
the smallest sufficient primitive: it is a safe `rustix` call already inside the enabled
`fs` feature, it needs no new dependency and no `unsafe`, it touches no observed
filesystem, and a process that already existed cannot possess the resulting inode.

**Regression test required.** A foreign child that saturates its descriptor table with its
own `/proc/self/fd`, whose descriptor directory must be refused; plus the inherited-object
and dead-process variants, and a positive test that the genuine capability still works.

### IMPORTANT-2 — duplicate-key rejection is bypassable, in two independent ways

**Location:** [`crates/helm-observe/src/plan.rs`](../../crates/helm-observe/src/plan.rs),
`scan_raw`.

**Reproduced**, on the candidate, by
`crates/helm-observe/tests/independent_review_plan.rs`.

1. **Array-nested objects are never scanned.** `expect_key` is recomputed as
   `depth == stack.len()`, but `depth` counts `{` *and* `[` while `stack` counts only `{`.
   Inside any object nested in an array — that is, inside every `roots[]` and `targets[]`
   entry — the two never agree after the first key, so duplicates there are invisible.
   `{"targets":[{"id":"decoy",…,"id":"real"}]}` is **accepted** and silently resolves to
   `real`. The same holds for a duplicated `path`: a target carrying
   `"path":"harmless"` and later `"path":"other/elsewhere"` validates and observes the
   second spelling.
2. **Escaped key spellings are not decoded.** The hand-rolled scanner drops escape
   sequences instead of decoding them, so `"targets"` is recorded as the key
   `0074argets` and does not collide with `targets`, while `serde_json` decodes both to
   `targets` and keeps the last. A document carrying a visible decoy `targets` array and an
   escaped real one is **accepted**, and the decoy is discarded.

**Consequence.** The crate documents, in code comments, in the README and in the author
review, that duplicate decoded keys are rejected *before* a map can hide one. That control
does not hold. This is not a capability escape: targets still resolve only beneath
caller-granted roots, and the plan digest is still over the exact bytes. It is a
plan-interpretation ambiguity, which directly weakens the auditability that obligation A's
exact-plan identity exists to provide: a reviewer, or any other JSON reader, can disagree
with the observer about what the authorised bytes said.

**Minimum correction.** Detect duplicates on **decoded** keys at every object depth. Track
container kinds on a stack instead of comparing two different depth counters, and decode
each key literal with the JSON decoder already in the dependency graph.

**Regression test required.** The three array-nested cases, the escaped-spelling case, and
negative controls proving sibling objects may legitimately reuse key names and that
key-shaped text inside a string value is not a duplicate.

### IMPORTANT-3 — `0xEF53` does not establish ext4, but the claims say it does

**Location:** [`crates/helm-observe/src/linux.rs`](../../crates/helm-observe/src/linux.rs)
(`EXT_SUPER_MAGIC`, `admit_root`),
[`crates/helm-observe/README.md`](../../crates/helm-observe/README.md) "Supported cohort",
`AdmissionErrorCode::RootUnsupportedFilesystem` documentation, the author suite's
`ext4_cohort_admission_is_reported_not_weakened` test and its
`HELM-OBSERVE-COHORT: ext4 PASS` line, and the corresponding paragraph of
`docs/PROJECT_STATE.md`.

**Primary-source analysis.** The Linux UAPI header `include/uapi/linux/magic.h` defines
`EXT2_SUPER_MAGIC`, `EXT3_SUPER_MAGIC` and `EXT4_SUPER_MAGIC` as the **same** value,
`0xEF53`; the ext4 on-disk documentation gives `s_magic = 0xEF53` for the same superblock
that ext2 and ext3 use, and `statfs(2)` lists a single `EXT2_SUPER_MAGIC` entry for the
whole family. The implementation's own comment says so: *"Filesystem magic for ext4 (shared
with ext2/ext3)"*. `fstatfs` therefore **cannot** distinguish ext2, ext3 and ext4, and no
other check in the crate attempts to.

**Consequence.** Root admission accepts an ext2 or an ext3 root, while the README states
the supported cohort is "local **ext4**" and that "root admission refuses anything else
rather than degrading", the author suite prints `ext4 PASS`, and `PROJECT_STATE` records
"the ext4 cohort execution is recorded as PASS". Admission is *stricter* than nothing and
*weaker* than the claim: `0xEF53` is a necessary but not a sufficient condition for ext4.
"Local" is likewise assumed rather than established — the same magic is reported for an ext
filesystem on network-backed block storage.

**This is not correctable inside the accepted boundary.** The only honest discriminators
are outside it: the mounted filesystem *type name* in `/proc/self/mountinfo` would need a
new pathname-based procfs read and still only reports the name a mount was requested with,
and the on-disk feature set that actually distinguishes ext4 needs block-device access far
outside an explicit-target observer. Renaming the cohort to "ext-family" would broaden
ADR-0022 and is forbidden to this review.

**Minimum correction available to a reviewer.** Correct the *claims* to state exactly what
the mechanism establishes, keep admission exactly as strict, and refer the cohort question
to the owner. **Classification: NEEDS_ARCHITECTURE_OWNER_REVIEW** for the ext4 cohort
claim itself.

**Regression test required.** A cohort test that reports the superblock magic and the
mounted type name from primary sources and asserts admission behaviour, without asserting
an ext4 identity the mechanism cannot support.

### IMPORTANT-4 — the Linux x86_64 half of the cohort is not enforced at all

**Location:** [`crates/helm-observe/src/lib.rs`](../../crates/helm-observe/src/lib.rs), the
`#[cfg(target_os = "linux")]` gates on `authority`, `linux` and `observe`.

**Source-derived.** The accepted cohort is "Linux x86_64 and local ext4". The crate gates
the observation backend on the operating system only. On Linux aarch64, riscv64 or any
other Linux architecture the full capability API, target resolution and data reopen compile
and operate, with no compile error, no admission refusal and no runtime signal — while the
filesystem half of the same cohort *is* enforced at admission, and the README says
"root admission refuses anything else rather than degrading". The asymmetry is undocumented
and unexercised. `AdmissionErrorCode::UnsupportedPlatform` exists but is never constructed.

**Consequence.** Full observation authority outside the empirically anchored cohort,
presented as if the cohort were enforced. No memory-safety or disclosure consequence.

**Minimum correction.** Narrow the backend gate to
`all(target_os = "linux", target_arch = "x86_64")`, so an unsupported Linux architecture
gets the same treatment as Windows — pure plan and model code, and no observation backend
at all — instead of unvalidated authority. This narrows to the accepted cohort and
broadens nothing; a single `cfg` edit reverses it if the owner extends the cohort later.

**Regression test required.** The Linux suites carry the same architecture gate, so an
unsupported architecture no longer compiles a test that asserts supported behaviour.

### MINOR findings

| # | Location | Finding |
|---|---|---|
| M1 | `model.rs`, `ObservationArtifact::new` | `bytes.truncate(MAX_ARTIFACT_BYTES)` would silently emit invalid JSON whose `sha256` still matches, if the ceiling were ever reachable. It is not reachable today, but that is nowhere proved in-tree. Worst case from the actual schema and limits is ≈32 KiB against a 256 KiB ceiling; the proof belongs in a test |
| M2 | `observe.rs`, `stream_regular` | A file at exactly `MAX_FILE_BYTES` that grows may have `MAX_FILE_BYTES + 1` bytes **read** because of the growth sentinel. The public constants are still honest — `MAX_FILE_BYTES` documents bytes *hashed*, and never more than `size` bytes enter the digest, while the aggregate ceiling is exact — but the one-byte read overrun deserves one explicit sentence rather than only a code comment |
| M3 | `observe.rs`, `observe` | Aggregate exhaustion suppresses **every** later target, including `directory_metadata` targets that would read no bytes at all. Deterministic and explicitly reported, but more conservative than the budget requires |
| M4 | `README.md` | Says "Observation code performs no filesystem writes", which is true, and never states the effect that ADR-0022 does record: a permitted read is not side-effect-free, and may touch atime, page cache and I/O. The absence invites the stronger reading |
| M5 | `tests/linux_authority.rs`, `root_admission_requires_a_directory_and_rejects_a_file` | `assert!(is_ext(&base) \|\| !is_ext(&base))` is a tautology that cannot fail |
| M6 | `tests/linux_authority.rs`, `obligation_b_…` | The order-independence half compares only `exact_bytes().len()` of two artifacts, which two different root bindings could also satisfy |
| M7 | `tests/plan_contract.rs`, `every_digest_byte_participates_in_identity` | Flips bytes in a **copy of the digest array** and compares it to the original array. It tests `[u8; 32]` equality, not that the authorisation or the artifact binds all 32 bytes |
| M8 | `docs/implementation/HELM-OBSERVE-REVIEW.md` §9 | States "Linux, 13 tests"; the file contains 16, as `PROJECT_STATE` correctly says |
| M9 | `error.rs` | `AdmissionErrorCode::UnsupportedPlatform` is unconstructible: the module that would produce it is itself gated to the supported platform |

### NO FINDING

Obligations A, B and C as such; resolve-flag policy and the absence of any fallback;
absence-versus-failure mapping; aggregate budget arithmetic; artifact determinism, result
vocabulary and privacy; dependency boundary and the absence of `unsafe`; descriptor
lifetime. Section 4 onwards records the evidence for each.

## 4. Review method and instrumentation

The review adds, on this branch only:

- `crates/helm-observe/tests/independent_review_plan.rs` — adversarial plan contract,
  exact ceilings on both sides, and a **new** seeded corpus with a reviewer-chosen seed
  (`0x123456789ABCDEF0`) and structure-aware mutations, not a re-run of the author's seed.
- `crates/helm-observe/tests/independent_review_linux.rs` — foreign-process procfs
  admission attacks, a FIFO with a writer blocked in `open(2)`, deadline-guarded special
  files, errno discipline, exact budget boundaries, descriptor-leak checks and independent
  cohort identification.
- `tools/helm-observe-review-driver/` — a driver around the **public API only**, outside
  the product workspace, so the compiled implementation can be traced.
- `tools/helm_observe_syscall_review.py` — a syscall-level regression that imports the
  frozen OBS-FS-01 ptrace tracer **read-only** and derives the negative claims from the
  trace rather than from the returned enum. It re-runs no OBS-FS-01 case, changes no frozen
  file and touches no A0 environment.
- `.github/workflows/helm-observe-independent-review.yml` — records the hosted runner's
  actual tool inventory before drawing any conclusion from it, and runs the checks above
  next to the unchanged repository validation.

Nothing in this review runs Wine or 7-Zip, opens A0 evidence, modifies a WSL or Hyper-V
lab, implements `helm-bind` or `helm-launch`, or broadens ADR-0022.
