# helm-launch P2 independent review

> **THIS IS AN INDEPENDENT REVIEW OF THE HELM-LAUNCH P2 CANDIDATE.**
>
> **P2 ADDS AUTHORITY-BEARING FD CAPABILITIES BUT NO PROCESS CREATION OR EXECUTION.**
>
> **P3+ REMAINS NOT AUTHORISED.**
>
> **THIS REVIEW ADDS NO `unsafe`, NO PROCESS CREATION AND NO `launch()`.**

**Role:** fresh independent adversarial review of the P2 two-commit candidate. This review session
authored neither `a3a8d99` nor `c74e906`; both commits carry the repository owner's identity and
were produced before this review began.\
**Reviewed candidate:** `cd5db27964dc10593bd8856d2d331b907a4b608e` (published, accepted P1 base)
→ `a3a8d999a6bfa59ce27b1515525e6a7578dad7fe` (P2 authority record) →
`c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` (P2 implementation), on
`docs/helm-launch-architecture`.\
**Authority:** Accepted [ADR-0024](../adr/ADR-0024-launch-authority.md), the owner-reviewed
[productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md), the
[P1 acceptance of 2026-09-17](../DECISIONS.md#helm-launch-p1-accepted) and the
[P2 authorisation of 2026-09-18](../DECISIONS.md#helm-launch-p2-authorised):
**P1 accepted, P2 authorised, P3/P4/P5 not authorised, no Trial #4.** The owner's clarification
that the absence of a local Linux Rust toolchain is `GATE_PENDING` and not a defect is applied as
current owner authority.\
**Scope of this document:** review only. No implementation file, plan, ADR, decision record,
project state, experiment, evidence or workflow is changed. Nothing was pushed and no CI was
dispatched.

**Classification: `HELM_LAUNCH_P2_INDEPENDENT_REVIEW_PASSED_READY_FOR_PUBLICATION_CI`.**
No BLOCKER and no IMPORTANT finding. Seven MINOR findings, three non-blocking backlog items and
two `GATE_PENDING` items are recorded in section 21.

## 1. Starting state, independently reconstructed

| Claim | Method | Result |
|---|---|---|
| worktree clean | `git status --porcelain` | empty |
| branch | `git rev-parse --abbrev-ref HEAD` | `docs/helm-launch-architecture` |
| local HEAD | `git rev-parse HEAD` | `c74e9064f4a852688b1a13dc3d3d31b93b61b0aa` |
| remote milestone | `git fetch origin --prune`; `git rev-parse origin/docs/helm-launch-architecture` | `cd5db27964dc10593bd8856d2d331b907a4b608e` |
| main | `git rev-parse origin/main`, `git rev-parse main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` (both) |
| exact chain | `git log --format='%H %P' -3` | `cd5db27` → `a3a8d99` → `c74e906`, each the sole parent of the next |
| divergence | `git rev-list --left-right --count origin/docs/helm-launch-architecture...HEAD` | **2 ahead, 0 behind** |

The fetch was read-only. Nothing was pushed at any point.

## 2. Authority read before code

Read in full before the diff: `AGENTS.md`; [ADR-0024](../adr/ADR-0024-launch-authority.md) in its
entirety, including sections A to N and the owner-decision tables; the
[productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md) sections 6.2 to 6.4, 7.1 to 7.4, 13.2,
13.3 and 16; the P1 and P2 decisions in [`DECISIONS.md`](../DECISIONS.md); the current sections of
[`PROJECT_STATE.md`](../PROJECT_STATE.md); and the accepted
[P1 independent review](HELM-LAUNCH-P1-INDEPENDENT-REVIEW.md) with its finding table. Then the
complete candidate diff `cd5db27..c74e906` (12 files, 3 585 insertions, 198 deletions), every
changed source, test, manifest, README and workflow file in full, and each commit separately.

The implementation was judged against the accepted P2 contract, not against P3 assumptions and not
against the LAUNCH-EXEC-01 spike. The binding boundary is the owner's P2 decision list: no
`LaunchOutcome`, `launch()`, process creation or execution, `clone3`, `execveat`, pidfd acquisition
or signalling, `waitid`, `pidfd_send_signal`, `close_range`, `fchdir` execution, signal or
process-group manipulation, `PR_SET_NO_NEW_PRIVS` call, child pipe, polling lifecycle, timeout
execution, `backend/` directory, syscall shim, inline assembly, libc call or `unsafe` operation.

## 3. Commit and file scope

| Commit | Files | Assessment |
|---|---|---|
| `a3a8d99` | `docs/DECISIONS.md`, `docs/PROJECT_STATE.md`, `docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md` | exactly the three permitted documents; authorises P2 and states P3/P4/P5 **not authorised**; no ADR, experiment, evidence, code, test, Cargo or workflow file touched |
| `c74e906` | `.github/workflows/helm-launch.yml`, `Cargo.lock`, `crates/helm-launch/Cargo.toml`, `crates/helm-launch/README.md`, `crates/helm-launch/src/{authority.rs (new), error.rs, lib.rs}`, `crates/helm-launch/tests/{linux_admission.rs (new), p1_boundary.rs → p2_boundary.rs}` | bounded P2 implementation, test, Cargo, CI and README changes only |

The authority commit **precedes** the implementation commit and is its sole parent. No frozen
experiment definition, freeze manifest, evidence file, ADR or `main` content is touched by either
commit. Neither commit message carries a `Co-Authored-By` trailer, a "Generated with" signature or
any agent attribution; `git log cd5db27..c74e906 --format='%B'` contains none.

The authority record's own content was checked against the boundary it claims. `DECISIONS.md`
§1 enumerates what P2 may introduce and an explicit "must not introduce" list; §2 states the
host-visible effects (read-only I/O through caller-supplied descriptors; atime may be updated; the
page cache is populated); §3 restates the cohort gate and the non-conversion rule; §4 restates that
measurement is not an attestation; §5 bounds the dependency; §6 fixes the next gate as
implementation followed by one fresh independent review. `PROJECT_STATE.md` carries the same
boundary as a state table. The plan's header moves from "IMPLEMENTATION AUTHORITY: P1 ONLY" to
"P1 AND P2" with "P3, P4, P5: NOT AUTHORISED". Nothing in the record authorises P3 or a Trial #4.

**`P2_AUTHORITY_AND_COMMIT_SCOPE_SOUND`**, with P2-DOC-01 (MINOR) on ADR-0024's now-stale
"Implementation authority" header line.

## 4. The absolute P2 boundary, independently proved

| Boundary | Method | Result |
|---|---|---|
| process creation | `grep -rnoE '\b(clone3\|fork\|vfork\|posix_spawn\|Command::new\|\.spawn\()'` over `src/` and `tests/` | zero occurrences as code; the only textual hits are string literals inside the `FORBIDDEN_EVERYWHERE` list of `tests/p2_boundary.rs` |
| process execution | same scan for `execve`, `execveat`, `fexecve`, `std::process`, `CommandExt`, `Stdio`, shell invocation | none as code; `StdioMapping` in `src/layout.rs` is a pre-existing P1 pure type whose identifier tokenises as one word |
| `unsafe` | `tests/p2_boundary.rs::the_source_contains_no_unsafe_token_anywhere` scans every source **and test** file for the bare word, comments included; reproduced independently by grep | absent everywhere; `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]` at the crate root and `unsafe_code = "deny"` in the manifest, with no `allow` of either |
| host privilege | scan for `sudo`, `setuid`, `capset`, `prctl`, `PR_SET_NO_NEW_PRIVS`, namespace, seccomp, cgroup and MAC vocabulary | none; the only `prctl` hit is a forbidden-list literal |
| experiment execution | scan for `launcher_spike` and any LAUNCH-EXEC-01 asset | none; the crate tree is 7 source files, 3 test files, `Cargo.toml` and `README.md`, listed by `find` |
| P3+ implementation | required-absent list of section 4 of the review instruction | `LaunchOutcome`, `launch()`, `backend/`, `pidfd`, `waitid`, `pidfd_send_signal`, `close_range` as an OS operation, `fchdir` as an OS operation, signal and process-group manipulation, raw syscall shim, `asm`, FFI (`extern`), direct `libc` — **all absent**. `close_range` and `setpgid` appear only as string spellings of the P1 `ChildStage` receipt enum in `src/model.rs`, which is inert data that existed before this candidate; `fchdir` appears only in one explanatory comment in `tests/linux_admission.rs` |

Required-present on Linux x86_64 only: `ExecutableCapability`, `WorkingDirectoryCapability`,
`AuthorizedLaunch`, `admit_executable`, `admit_working_directory`, `authorize` — all present,
all gated (section 5).

The decisive structural fact is that **`AuthorizedLaunch` has no consumer**. No function in the
crate takes one by value or by reference except its own three inert accessors, so the value cannot
be turned into a process by any code path in this tree. The crate-root `compile_fail` doctests pin
`use helm_launch::launch;` and `use helm_launch::LaunchOutcome;` as non-resolving on every
platform.

**Nothing in this candidate is a P3 implementation.**

## 5. Platform gating

`src/lib.rs` declares `mod authority;` and re-exports the six P2 names under exactly
`#[cfg(all(target_os = "linux", target_arch = "x86_64"))]`, together with the cohort-only constant
`MAX_EXECUTABLE_BYTES`. `src/authority.rs` carries **no** platform `cfg` of its own —
`tests/p2_boundary.rs` asserts that the module names neither `target_os` nor `target_arch` as
code — so the module cannot be compiled off the cohort by a second route. The same test pins the
exact predicate text on both the module declaration and the re-export, and counts every
`target_os` occurrence in every source file (4 in `lib.rs`, 1 prose occurrence in `authority.rs`,
0 elsewhere).

Off-cohort absence is proved by a `cfg_attr` documentation block in the crate root holding six
`compile_fail` doctests, one per P2 name; a sibling on-cohort block holds the positive `Send`
assertions. Both blocks are pinned by `tests/p2_boundary.rs`. Locally, the Windows run collected
and passed 17 doctests, which includes the off-cohort absence set, and
`tests/linux_admission.rs` compiled to an empty binary (`0 passed`) exactly as its file-level
`#![cfg(...)]` requires.

The exported error vocabulary was reviewed for authority content. `AdmissionError`,
`AdmissionErrorCode`, `AuthorizationRefusal` and `AuthorizationRefusalCode` are portable data:
private fields, a fixed code and an `Option<i32>` error number. They carry no descriptor, no
`rustix` type, no platform authority and no producer off the cohort. Their portability follows
plan §7.1, which places "errors" in the portable set on every target. See P2-BL-01 for the wording
question this raises against the P2 decision's §3 sentence; no authority is exposed either way.

**`P2_PLATFORM_BOUNDARY_SOUND`.**

## 6. Dependencies

`crates/helm-launch/Cargo.toml` keeps its three direct normal dependencies unchanged at
`serde = "=1.0.228"`, `serde_json = "=1.0.149"`, `sha2 = "=0.10.9"`, and adds exactly one
cohort-gated line:

```toml
[target.'cfg(all(target_os = "linux", target_arch = "x86_64"))'.dependencies]
rustix = { version = "=1.1.4", default-features = false, features = ["std", "fs"] }
```

Verified against the vendored `rustix 1.1.4` manifest at
`~/.cargo/registry/src/index.crates.io-*/rustix-1.1.4/Cargo.toml`:

* `fs = []` — the feature pulls in **no** other feature.
* `process = ["linux-raw-sys/prctl"]`, `pipe`, `event`, `thread`, `mm`, `net`,
  `runtime` — **none is enabled**, directly or transitively, and none is reachable from `std` or
  `fs`.
* `rustix::io`, which carries `pread` and `retry_on_intr`, is not feature-gated, so the manifest
  comment asserting that is accurate.

The target-specific **dev**-dependency is the identical line: same package, same pin, same
`default-features = false`, same two features, same cohort gate. It therefore adds no package, no
version and no feature to the product graph under feature unification, and it is genuinely needed:
the Linux tests are the trusted caller and must open descriptors in `O_PATH`, `O_WRONLY`, `O_RDWR`
and `O_RDONLY`, set and clear `FD_CLOEXEC` with `F_GETFD`/`F_SETFD`, and build sparse fixtures with
`ftruncate` — none of which the product API offers.

`Cargo.lock` changes by exactly one line: `rustix` is added to the `helm-launch` dependency list.
No package, version or checksum elsewhere changes. `rustix 1.1.4` was already in the lockfile for
`helm-observe`; `tests/p2_boundary.rs` asserts there is exactly one `rustix` entry and exactly one
`libc` entry. There is no direct `libc` dependency, no HELM crate dependency, no build script, no
`[features]` table, and exactly two `[target...]` tables. The Linux cross-check compiled
`linux-raw-sys` and **not** `libc`, confirming the raw backend.

**`P2_DEPENDENCY_GRAPH_SOUND`.**

## 7. Safe descriptor ownership

Both admission functions take `fd: OwnedFd` **by value**. Every internal check receives
`fd.as_fd()`, a `BorrowedFd` tied to the owned value's lifetime; on success the same `OwnedFd` is
moved into the capability's private `fd` field, so the capability owns exactly the object the
caller supplied — never a duplicate, never a reopened path. `tests/linux_admission.rs` and the
`authority.rs` unit test `each_capability_owns_the_exact_descriptor_the_caller_moved_in` prove this
by comparing `st_ino` and `st_dev` of `capability.descriptor()` with the caller's own pre-admission
`fstat`, and by renaming the object afterwards without changing what the descriptor names.

No `from_raw_fd`, `into_raw_fd`, `borrow_raw`, `AsRawFd`, `RawFd`, `mem::forget`, `ManuallyDrop` or
`dup` appears anywhere in the crate; `tests/p2_boundary.rs::forbidden_vocabulary_appears_nowhere_in_the_crate`
enforces every one of those tokens across sources **and** tests, and an independent grep
reproduces the result. The only `dup` in the tree is `rustix::io::dup` inside two tests, used to
observe the shared file offset — and it is reached through the `rustix::io::` path, not as a bare
token.

RAII is preserved on every path. `fd` is a moved-in local, so each of the eight early `return
Err(...)` arms in `admit_executable`, both in `admit_working_directory`, and the mismatch arm of
`authorize` drops it and closes the descriptor naturally. No refusal value carries a descriptor:
`AdmissionError` is `{ code, errno: Option<i32> }` and `AuthorizationRefusal` is `{ code }`, both
`Copy`.

The two crate-private accessors are `pub(crate) fn descriptor(&self) -> BorrowedFd<'_>`. They
return a borrow, never a number, and `tests/p2_boundary.rs` asserts there are exactly two of them,
both `pub(crate)`, and that no `pub fn descriptor` exists. No public method of any type returns a
descriptor or a descriptor number.

Leak behaviour is tested on both the success and the refusal path:
`refusals_and_drops_leak_no_descriptor` runs 64 rounds of four refusals, one refused authorisation
and one successful admit-authorise-drop cycle, and compares `/proc/self/fd` counts with a
`PARALLEL_ALLOWANCE` of 16 against a per-round leak of at least eight. The margin is sound.

**`P2_FD_OWNERSHIP_SOUND`.**

## 8. Authority types and auto-traits

All three authority-bearing types obtain `Send` and not `Sync` **safely**, with no marker `impl`:

* `ExecutableCapability { fd: OwnedFd, measurement: ExecutableMeasurement, not_sync: PhantomData<Cell<()>> }`
* `WorkingDirectoryCapability { fd: OwnedFd, id: CapabilityId, not_sync: PhantomData<Cell<()>> }`
* `AuthorizedLaunch { plan, executable, working_directory }` — no marker of its own, because
  `ExecutableCapability` already removes `Sync`

`PhantomData<Cell<()>>` is the ordinary safe idiom: `Cell<T>` is `Send` when `T` is and is never
`Sync`, so the field removes exactly `Sync` and changes neither ownership nor layout. No
`unsafe impl Send`/`Sync` exists anywhere. Each property is pinned by a doctest pair — a positive
`assert_send::<T>()` and a `compile_fail` `assert_sync::<T>()` — and `AuthorizedLaunch` carries the
equivalent pair.

No `Clone`, `Copy`, `Default`, `Serialize`, `Deserialize`, `From<OwnedFd>`, `From<LaunchReceipt>`,
`From<&[u8]>`, `From<&Path>`, public constructor or setter exists for any of the three. The fields
are private, so no struct literal works from outside. `tests/p2_boundary.rs::no_capability_type_derives_a_forbidden_trait`
pins that the whole product region of `authority.rs` contains exactly two `#[derive(` attributes
(the two internal sample structs), exactly one hand-written `Debug` per capability type, and no
`impl Clone/Copy/Default/From/PartialEq/Hash for` any capability nor any generic form. A grep for
`impl .* From<` across the crate returns nothing.

`AuthorizedLaunch` is single-use in the only sense available in P2: it cannot be cloned, and no
function consumes it at all.

`Debug` output is admitted facts only. `ExecutableCapability` prints `pre_exec_body_size`,
`pre_exec_body_sha256`, `pre_exec_mode_bits` and `elf_type` with `finish_non_exhaustive`;
`WorkingDirectoryCapability` prints the caller's logical identifier; `AuthorizedLaunch` prints the
plan digest, the identifier and the nested capability. No descriptor, descriptor number, inode,
device, host path, argv byte or output byte appears.

**`P2_AUTHORITY_TYPE_BOUNDARY_SOUND`.**

## 9. Executable admission order

`admit_executable` was traced line by line against ADR-0024 §C and plan §6.2. The order is exact
and each step short-circuits with `?` or an early `return`, so no later step can run after an
earlier refusal:

| Step | Code | Refusal |
|---|---|---|
| 1 | `require_read_only(fd.as_fd())?` — `fcntl_getfl` | `DescriptorModeUnsuitable`, or `MetadataUnavailable` with errno |
| 2 | `metadata_sample(fd.as_fd())?` → **first sample**; `FileType::from_raw_mode(raw_mode) != RegularFile` | `NotRegularFile` |
| 3 | `Mode::from_raw_mode(raw_mode).intersects(SUID ∪ SGID)` | `SetIdBitsPresent` |
| 4 | `first.size > MAX_EXECUTABLE_BYTES` | `ExecutableTooLarge` |
| 5 | `classify_elf_header(&read_elf_header(fd.as_fd())?)?` | `NotElf` / `ElfNotInCohort` |
| 6 | `measure_body(fd.as_fd(), first.size)?` | `ReadFailed` |
| 7 | second `metadata_sample`, then `Observation::instability_detected()` | `MeasurementInstabilityDetected` |
| 8 | `ExecutableCapability { fd, measurement, not_sync }` | — |

**The size bound is proved to precede any body read by construction**, not only by test: step 4 is
a plain `if … return Err` on the first sample's `size`, and the first `pread` in the function is
inside `read_elf_header`, called at step 5. No read of any kind — header or body — is reachable
above step 4.

Every step uses `fd.as_fd()`, the same descriptor throughout. No pathname, name, `PATH`, `open`,
`openat`, `statx`, `/proc` path or environment lookup occurs; `tests/p2_boundary.rs::product_code_names_no_host_state_and_no_pathname_api`
enforces a 38-token list including `Path`, `PathBuf`, `CStr`, `CString`, `OsStr`, `env`, `open`,
`openat`, `statx`, `canonicalize`, `read_link` and `current_dir` across the product region of every
source file, and proves the scan is real by asserting the test region of `authority.rs` does use
`open` and `unlink`.

**`P2_EXECUTABLE_ADMISSION_ORDER_SOUND`.**

## 10. Descriptor mode semantics

```rust
let flags = fcntl_getfl(fd).map_err(|errno| os_refusal(A::MetadataUnavailable, errno))?;
if flags.contains(OFlags::PATH) || flags.intersection(OFlags::RWMODE) != OFlags::RDONLY {
    return Err(AdmissionError::new(A::DescriptorModeUnsuitable));
}
```

Checked against Linux semantics and against the vendored `rustix 1.1.4` source:

* `open(2)` specifies that `F_GETFL` is one of the operations permitted on an `O_PATH` descriptor
  and that the returned flags **include** `O_PATH`, so the `contains(OFlags::PATH)` test is the
  correct discriminator. `O_PATH` is refused.
* `rustix`'s `OFlags::RWMODE` is `O_RDONLY | O_WRONLY | O_RDWR` (`backend/linux_raw/fs/types.rs`),
  i.e. the `O_ACCMODE` mask. `intersection(RWMODE) != RDONLY` therefore requires the access-mode
  bits to be exactly zero: `O_WRONLY` and `O_RDWR` are refused, `O_RDONLY` is admitted.
* Unrelated status bits are masked away and cannot cause a false refusal. This is tested directly:
  `an_unrelated_status_flag_does_not_make_the_mode_unsuitable` admits an `O_RDONLY | O_NONBLOCK`
  descriptor, and `a_writable_descriptor_is_refused_in_either_writable_mode` includes an
  `O_RDWR | O_APPEND` arm.
* `FD_CLOEXEC` is a **descriptor** flag read with `F_GETFD`, not a status flag, and `F_GETFL` does
  not report it. The implementation never inspects it, and the crate says so in three places.

`close_on_exec_is_irrelevant_to_admission_in_either_state` covers all three states and is
**probative**, which I verified at the source rather than assuming: `rustix`'s `open`/`openat` add
only `O_LARGEFILE` and never `O_CLOEXEC`, so the test's assertion that a descriptor opened by
`rustix::fs::open(path, OFlags::RDONLY, …)` has no `FD_CLOEXEC` is a real observation. The other
two arms open with `OFlags::CLOEXEC` and assert the flag is present, and then clear it with
`fcntl_setfd` and assert it is absent. All three are admitted.

**`P2_DESCRIPTOR_MODE_SOUND`.** This closes reported finding P2-IMPL-01: the committed form does
not assume automatic `O_CLOEXEC`.

## 11. Metadata, file type, set-ID and size

`metadata_sample` calls `fstat` once and returns both the five observed fields and the raw mode
word, so mode bits and file type come from **one** syscall result rather than a second sample.

* Regular-file check is exact: `FileType::from_raw_mode(raw_mode) != FileType::RegularFile`, an
  equality against the decoded type rather than a bitmask test. A directory and `/dev/null` are
  both tested and both refused `NotRegularFile`.
* Set-ID uses `Mode::from_raw_mode(raw_mode).intersects(Mode::SUID.union(Mode::SGID))` — the
  correct `S_ISUID`/`S_ISGID` bits, decoded by `rustix` rather than by a hand-written constant.
* `pre_exec_mode_bits` is `mode_bits(raw_mode)` taken from **sample #1 only**; sample #2's mode word
  is discarded with `let (second, _) = …`. `recorded_mode_bits_come_from_the_first_accepted_metadata_sample`
  additionally chmods the object after admission and asserts the recorded value does not move.
* Size conversion is safe: `u64::try_from(stat.st_size)` refuses a negative `st_size` with
  `MetadataUnavailable` rather than wrapping it into a huge accepted value. I confirmed against the
  vendored source that `rustix`'s x86_64 `Stat` has `st_size: c_long`, `st_mtime: c_long`,
  `st_mtime_nsec: c_ulong`, `st_ctime: c_long`, `st_ctime_nsec: c_ulong` and `st_mode: c_uint` —
  exactly the `i64`/`u64`/`u32` shapes the code's `MetadataSample` and `RawMode` use, so the
  comment claiming a type change would become a compile error is accurate.
* `mode_bits` masks with `0o7777` before the `as u16`, so the cast cannot truncate; unit assertions
  cover `0o100_755 → 0o755`, `0o104_755 → 0o4755` and `0o40_700 → 0o700`.

The 512 MiB boundary is tested on both sides with a **sparse** `ftruncate` fixture: `MAX + 1` is
refused `ExecutableTooLarge`, and exactly `MAX` passes the bound and is then refused `NotElf` by the
next step — which is the sharpest available proof that the comparison is `>` and not `>=`. The
over-limit test does not itself observe that no `pread` occurred (P2-MIN-05); the code ordering in
section 9 establishes that independently.

**`P2_METADATA_ADMISSION_SOUND`**, with P2-MIN-05 (MINOR).

## 12. ELF header

Offsets and decoding were checked against the ELF64 header layout rather than against the
implementation's own constants:

| Field | Offset | Width | Code | Correct |
|---|---|---|---|---|
| `EI_MAG0..3` | 0 | 4 | `[0x7f, b'E', b'L', b'F']` | yes |
| `EI_CLASS` | 4 | 1 | `ELFCLASS64 = 2` | yes |
| `EI_DATA` | 5 | 1 | `ELFDATA2LSB = 1` | yes |
| `e_type` | 16 (`0x10`) | 2, LE | `ET_EXEC = 2`, `ET_DYN = 3` | yes |
| `e_machine` | 18 (`0x12`) | 2, LE | `EM_X86_64 = 62` | yes |

Both 16-bit fields are decoded with `u16::from_le_bytes`, which is correct for `ELFDATA2LSB` and is
reached only after `EI_DATA` has been confirmed little-endian. `EI_VERSION` (offset 6) is not
checked. That is **consistent with the accepted ADR**: ADR-0024 §B and §C and plan §6.2 name the
cohort as magic plus `ELFCLASS64`, `ELFDATA2LSB`, `EM_X86_64` and `e_type ∈ {ET_EXEC, ET_DYN}` and
name nothing else, so no requirement was invented here. The test fixture sets
`header[6] = 1` with a comment marking it deliberately outside the check.

Refusal classification is correct and distinguishable: absent magic or a short object is `NotElf`;
magic present with any one cohort field wrong is `ElfNotInCohort`. Coverage: short object (0, 1, 4,
20, 63 bytes), non-ELF ≥ 64 bytes (4 096 bytes of `0x5a`), wrong class (`ELFCLASS32`), wrong endian
(`ELFDATA2MSB`), wrong machine (`EM_AARCH64 = 183`), wrong `e_type` (`ET_REL`, `ET_CORE`, and in
the unit test also `0`, `1`, `4`, `5`, `0xfe00`, `0xffff`).

**The script test is probative for absent magic, not merely for short length.** The committed form
uses two fixtures: `#!/bin/sh\nexit 0\n` padded to **128 bytes**, which passes the length rule and
is refused for the absent magic, and the unpadded 16-byte form, which the length rule stops first.
The test asserts `long.len() > 64 && short.len() < 64` before running, so the distinction cannot
silently collapse. This closes reported finding P2-IMPL-02.

**`P2_ELF_COHORT_CHECK_SOUND`.**

## 13. Positional reads and the shared offset

Both readers use `rustix::io::pread` exclusively, wrapped in `retry_on_intr`. No `read`, `seek`,
`lseek`, `readv` or `read_at_end` appears in product code, so the shared file offset of the open
file description is never consulted or moved.

`read_elf_header` fills a fixed `[u8; 64]` from offset `filled`, treats `read == 0` as `NotElf`
(fewer than 64 bytes exist) and advances with `saturating_add`. `measure_body` computes
`limit = initial_size.saturating_add(1)`, allocates one fixed `vec![0u8; 65_536]`, and loops
`want = min(limit - total, 65_536)` at offset `total` until `read == 0` or `total >= limit`.

Arithmetic reviewed against the section 9 bound (`initial_size ≤ 512 MiB` before measurement is
reachable):

* `size = 0` cannot reach measurement, because a zero-length object is refused `NotElf` at step 5.
* short reads are handled by the loop, which simply re-`pread`s at the new offset;
* `EINTR` is retried by `retry_on_intr`;
* EOF ends the loop through `read == 0`;
* growth past the bound is capped at `initial_size + 1`, which then fails the byte-count
  comparison of section 15 rather than reading unboundedly;
* `initial_size + 1` cannot overflow `u64` from a bound of 2^29, and `saturating_add` is used
  regardless;
* every `usize`/`u64` conversion is a `try_from` mapped to `ReadFailed`, never an `as` cast;
* the slice index `buffer[..want]` is in range because `want ≤ MEASUREMENT_BUFFER_BYTES = capacity`,
  and the hashed chunk uses `buffer.get(..read)` with an `ok_or_else` rather than an index.

Allocation is one 64 KiB buffer, independent of object size.

Offset immutability is tested on both paths and both scopes: the integration test seeks a
duplicated descriptor to 4 097 before admitting a 70 064-byte object and asserts the offset is
still 4 097 afterwards; the unit test does the same at offset 11. Because `dup` shares one open
file description, the watcher genuinely observes the offset admission would have used.

**`P2_POSITIONAL_MEASUREMENT_SOUND`.**

## 14. Measurement facts

`measure_body` feeds `hasher.update(chunk)` exactly the `read` bytes each `pread` returned, so the
SHA-256 covers exactly the admitted measurement bytes — never the first `st_size`, never a padded
buffer. The capability records `pre_exec_body_size: observation.bytes_read`, the count actually
read, not `first.size`. On a successful admission the two necessarily agree, because
`bytes_read != first.size` is itself an instability condition (section 15).

I recomputed all three committed digest vectors independently, in Python with `hashlib`, from the
byte construction the fixtures describe rather than from the crate:

| Fixture | Bytes | Independent SHA-256 | Committed expectation |
|---|---|---|---|
| 64-byte `ET_DYN` cohort header | 64 | `d58548872dbb5a72d6a518237a170b8173e6a91212356f7d9b249cbf591dc713` | identical |
| 64-byte `ET_EXEC` cohort header | 64 | `fac1ecd4990500ce3d68a41947d21e042f592eb9007dddd2d636e8f514605ca0` | identical |
| `ET_EXEC` header + 200 KiB `i % 251` pattern | 204 864 | `5889a02db0414b3b91c1fc2b681a3001d223afbbd3f01b80152c26ae9e288347` | identical |

Nothing was executed to obtain these; the probe only hashed bytes. The third vector is 204 864
bytes, which is more than three full 64 KiB rounds, so it proves each round contributes exactly
once and in order.

**`P2_MEASUREMENT_FACTS_SOUND`.**

## 15. Instability detection

`Observation::instability_detected` compares exactly six things and nothing else:

```rust
first.size != second.size
    || first.mtime_sec != second.mtime_sec
    || first.mtime_nsec != second.mtime_nsec
    || first.ctime_sec != second.ctime_sec
    || first.ctime_nsec != second.ctime_nsec
    || self.bytes_read != first.size
```

`MetadataSample` has exactly those five fields, so no stronger comparison is even representable.
Seconds and nanoseconds are separate fields for both timestamps, as the accepted amendment
requires. No stronger claim is made anywhere: the method's own doc, the `MeasurementInstabilityDetected`
doc comment, the module docs, the crate root docs and the README each state that returning `false`
proves nothing about mutation, immutability, snapshots or executed bytes.

**The deterministic seam.** `perturb_observation` has two definitions: `#[cfg(not(test))]` an empty
function, and `#[cfg(test)]` one that applies a thread-local `Cell<Option<fn(&mut Observation)>>`.
The seam:

* is not public — the function, the `thread_local!` and `with_instability_seam` are all module-
  private and none is re-exported;
* does not exist in any non-unit-test build — `cfg(test)` is set only when the crate is compiled as
  its own test harness, so neither `cargo build`, nor `cargo build --release`, nor the lib unit
  that `tests/linux_admission.rs` links against contains any hook; the non-test definition compiles
  to nothing;
* cannot be enabled by a normal product caller — there is no feature, no environment read and no
  public entry point;
* is pinned by `tests/p2_boundary.rs::the_test_only_region_of_every_source_is_last`, which asserts
  `INSTABILITY_SEAM` appears in the test region of `authority.rs` and **not** in its product region.

Deterministic tests exist for every observed field:
`detected_instability_refuses_admission_for_every_observed_field` drives seven perturbations —
size grew, size shrank, mtime seconds, mtime nanoseconds, ctime seconds, ctime nanoseconds, byte
count — each through a real `admit_executable` call, and asserts
`MeasurementInstabilityDetected` each time; `a_short_byte_count_is_detected_as_instability` covers
the short-count direction; `the_protocol_observes_exactly_five_fields_and_one_count` covers the
pure comparison; `the_seam_is_per_thread_and_leaves_admission_untouched_by_default` shows an
unseamed admission behaves exactly as a release build.

`real_metadata_changes_are_detected_by_the_same_comparison` is treated, correctly, as supporting
evidence: it uses an append (size + mtime) and a chmod (ctime only), both of which move
whole-second or coarse fields, so it does not depend on any filesystem exposing fine-grained
timestamps.

**`P2_INSTABILITY_DETECTION_SOUND`**, with P2-MIN-04 (MINOR) on the seam helper's lack of a
panic-safe drop guard.

## 16. `O_PATH`, writable and close-on-exec test probity

Covered in section 10. Summary of the committed forms:

| Case | Test | Probative |
|---|---|---|
| `O_PATH` executable | `an_o_path_descriptor_is_refused` | yes |
| `O_PATH` directory | `an_o_path_directory_is_refused` | yes |
| `O_WRONLY`, `O_RDWR`, `O_RDWR\|O_APPEND` | `a_writable_descriptor_is_refused_in_either_writable_mode` | yes |
| executable fd **without** `FD_CLOEXEC` | `close_on_exec_is_irrelevant_to_admission_in_either_state`, arm 1 | yes — `rustix::fs::open` adds only `O_LARGEFILE`, verified at the vendored source; the test asserts the flag's absence with `fcntl_getfd` before admitting |
| executable fd **with** `FD_CLOEXEC` | arm 2 | yes |
| caller-**cleared** `FD_CLOEXEC` | arm 3 | yes — opened with `OFlags::CLOEXEC`, cleared with `fcntl_setfd`, re-checked, then admitted |

`FD_CLOEXEC` is nowhere treated as an admission property.

**`P2_CLOEXEC_TESTS_SOUND`.**

## 17. Set-ID fixture probity

`a_set_id_object_is_refused_without_any_privilege_transition` builds `0o4755` and `0o2755`
fixtures. The shared `fixture()` helper chmods **after** writing, so no umask interferes, and then
re-stats and asserts:

```rust
assert_eq!(observed.st_mode & 0o7777, mode, "the host did not keep fixture mode {mode:o}");
```

This is exactly what section 17 of the review instruction requires: if the host or filesystem
strips `S_ISUID` or `S_ISGID`, the test fails **in the fixture, with a message naming the host**,
and cannot be mistaken for a launcher that failed to refuse a set-ID object. No privileged
transition is attempted anywhere: the crate never executes the object, and the admission path
refuses on a mode-bit test with no syscall beyond the `fstat` it had already performed.

P2-RISK-01 therefore remains **MINOR / environment-sensitive** and is the only issue here.

**`P2_SETID_REFUSAL_SOUND`.**

## 18. Working-directory admission

```rust
let id = CapabilityId::parse(id).ok_or_else(|| AdmissionError::new(A::WorkingDirectoryIdInvalid))?;
require_read_only(fd.as_fd())?;
let (_, raw_mode) = metadata_sample(fd.as_fd())?;
if FileType::from_raw_mode(raw_mode) != FileType::Directory { return Err(…NotDirectory); }
```

Identifier validation happens **before** any descriptor inspection, as required. It reuses
`CapabilityId::parse` from `src/model.rs` — the same crate-private validator the plan parser uses;
there is no second, divergent grammar, and `tests/p2_boundary.rs` prevents a copy by keeping every
identifier-grammar symbol out of `authority.rs`. The grammar `[a-z0-9][a-z0-9._-]{0,79}` is covered
by twelve rejection vectors (empty, three leading-character violations, uppercase, space, slash,
colon, tab, embedded NUL, non-ASCII, 81 bytes) and five acceptance vectors including the 80-byte
maximum.

`O_PATH` yields `DescriptorModeUnsuitable`; only `O_RDONLY` is admitted; the `fstat` is on the same
descriptor. No enumeration, path resolution, path recording, search-permission pre-check or
`fchdir` occurs — `fchdir` does not exist as code anywhere in the crate, and
`working_directory_admission_neither_enumerates_nor_resolves_anything` asserts that neither the
directory path nor a known entry name appears in the capability's `Debug` output.

The 0o400 fixture: the test creates an entry, admits once, unlinks the entry **while the directory
is still writable**, chmods to `0o400` (readable, **not** searchable), opens it `O_RDONLY` — which
succeeds for the owner, since opening a directory for reading needs only the `r` bit — and asserts
admission still succeeds. That is the right shape: it shows admission does not anticipate the
kernel's `fchdir` decision, while still letting the trusted test caller obtain the descriptor. Its
one limitation is recorded as P2-MIN-03: run as `root`, permission checks are bypassed and the
assertion holds vacuously rather than falsely. GitHub's `ubuntu-24.04` runner executes as an
unprivileged user, so the case is probative in the authorised CI gate.

**`P2_WORKING_DIRECTORY_ADMISSION_SOUND`**, with P2-MIN-03 (MINOR).

## 19. `authorize` — zero I/O

The whole function body is:

```rust
if plan.working_directory_id() != working_directory.id() {
    return Err(AuthorizationRefusal::new(R::WorkingDirectoryIdMismatch));
}
Ok(AuthorizedLaunch { plan, executable, working_directory })
```

Exactly one semantic relation. There is no `fstat`, `pread`, `fcntl`, `dup`, `open`, path,
metadata access or re-measurement — no syscall of any kind, and no call into any function that
performs one. `asserted_context` is not read: `plan.asserted_context()` is never called here, which
`asserted_context_digests_never_affect_authorisation` corroborates by authorising five plans whose
only difference is the asserted context (proved distinct by five distinct plan digests) with
identical outcomes, and then refusing all five on a mismatching identifier.

No binding, observation or `AppSpec` type can enter, for the structural reason ADR-0024 §M
requires: the crate links no HELM crate, so `Contradiction`, `Coverage`, `BindingReport`,
`ObservationArtifact`, `RootCapability` and `ValidatedAppSpec` are not nameable in it. A grep for
`helm_` across `src/` returns nothing but `helm_launch` itself.

All three inputs are consumed by value, so the mismatch arm drops the plan and both capabilities
and closes both descriptors before returning; nothing is handed back. On success `AuthorizedLaunch`
owns the plan and both capabilities unchanged —
`authorisation_succeeds_exactly_when_the_identifiers_match` captures the measurement before
`authorize`, asserts equality afterwards, then chmods the object and asserts the recorded
measurement still does not move, which proves no re-measurement and no live view.

**`P2_AUTHORIZE_ZERO_IO_SOUND`.**

## 20. No HELM authority conversion, error model, privacy, side effects, tests, CI, hygiene

### 20.1 Non-conversion

`Cargo.toml` declares no HELM crate dependency; `Cargo.lock`'s `helm-launch` entry lists exactly
`rustix`, `serde`, `serde_json`, `sha2`. The crate contains **no `From` or `Into` implementation at
all** (`grep -rn 'impl .*From<'` over `src/` returns nothing), so no conversion exists from
`ValidatedAppSpec`, `ObservationArtifact`, `RootCapability`, `BindingReport`, `Contradiction`,
`Coverage`, `LaunchReceipt`, receipt bytes, any `serde` input, `Path`, `PathBuf` or `String` into
`ExecutableCapability`, `WorkingDirectoryCapability` or `AuthorizedLaunch`. Three `compile_fail`
doctests pin the `From` and `Deserialize` absences per type, and `Deserialize`/`Serialize` are in
the crate-wide forbidden-token list. `NoClaimContradicted` remains impossible to inspect because
`helm-bind` is not linked.

`admit_working_directory` takes a `&str`, but that is the caller's **logical identifier**, which
ADR-0024 §C and plan §6.3 require; the authority is the `OwnedFd`, and the identifier can name
nothing on the host.

**`P2_AUTHORITY_NON_CONVERSION_SOUND`.**

### 20.2 Error model

The admission family is exactly the eleven codes plan §13.2 lists — `DescriptorModeUnsuitable`,
`NotRegularFile`, `NotDirectory`, `SetIdBitsPresent`, `ExecutableTooLarge`, `NotElf`,
`ElfNotInCohort`, `MeasurementInstabilityDetected`, `MetadataUnavailable`, `ReadFailed`,
`WorkingDirectoryIdInvalid` — and the refusal family is the single `WorkingDirectoryIdMismatch`.
**No new policy category appeared**, and the process-creation family of plan §13.2 is deliberately
absent because `launch` does not exist. Both enums are `#[non_exhaustive]`.

Mapping: `F_GETFL` and `fstat` failures become `MetadataUnavailable` with the errno, which matches
the code's stated rationale (the mode and object kind are then unknown, so refusing beats
guessing); `pread` failures become `ReadFailed` with the errno; policy refusals carry no errno, and
`a_policy_refusal_carries_no_error_number` asserts that for every one of the eleven codes.

`rustix::io::Errno` never reaches the public API. `os_refusal` converts with
`errno.raw_os_error()` into `Option<i32>` — a narrowing relative to plan §13.2's
`errno: Option<Errno>` that strengthens the accepted privacy rule rather than weakening it.

I verified the 28-entry symbolic table independently against the Linux `asm-generic` errno values:
EPERM 1, ENOENT 2, EINTR 4, EIO 5, ENXIO 6, EBADF 9, EAGAIN 11, ENOMEM 12, EACCES 13, EFAULT 14,
EBUSY 16, ENODEV 19, ENOTDIR 20, EISDIR 21, EINVAL 22, ENFILE 23, EMFILE 24, ETXTBSY 26, EFBIG 27,
ENOSPC 28, ESPIPE 29, EROFS 30, ENOSYS 38, ELOOP 40, EOVERFLOW 75, EOPNOTSUPP 95, ESTALE 116,
EDQUOT 122. All 28 are correct, strictly ascending and unique; `the_errno_table_is_closed_and_ascending`
pins the count, the ordering, the spelling shape and the uniqueness.

An unknown number stays representable: `errno_name()` returns `None` while `errno_number()` still
reports it and `Display` renders `METADATA_UNAVAILABLE errno 4242`. `errno_name()` is a `const fn`
over a closed `match` with no host input, so it is fully deterministic. No `strerror`, no OS message
and no locale string is consulted anywhere.

**`P2_ERROR_MODEL_SOUND`.**

### 20.3 Privacy

Reviewed every public `Debug` and `Display`. None can emit a descriptor number, inode, device, host
path, body byte, argv byte, username or locale string: `AdmissionError` holds only a code and an
`Option<i32>`, `AuthorizationRefusal` only a code, and the three capability `Debug`s print the inert
admitted facts listed in section 8. The executable measurement digest, size, mode bits and ELF type
are admitted facts the accepted contract permits a capability to expose. The logical caller
identifier appears only where ADR-0024 §L already records it in the receipt.

`no_refusal_or_capability_ever_prints_a_path_or_a_descriptor_number` exercises this at run time
against real fixtures under `/tmp`. Its `!rendered.contains("fd")` assertion is deterministic for
the fixed fixture bytes but is fragile to a future fixture change (P2-MIN-06).

**`P2_PRIVACY_BOUNDARY_SOUND`**, with P2-MIN-06 (MINOR).

### 20.4 Side-effect claims

The module docs, the crate-root docs, the README and the CI workflow header all state the same
thing and nothing stronger: reading the body **may update atime** under the host's mount policy and
**populates the page cache**; `O_NOATIME` is not used, because it requires file ownership or
`CAP_FOWNER`. No document claims admission is observationally pure, side-effect-free or read-only
in the observational sense. The README lists the side effects in its own subsection immediately
after the admission table, and the P2 authority record states them too.

**`P2_SIDE_EFFECT_DOCUMENTATION_SOUND`.**

### 20.5 Type-boundary tests

Every new `compile_fail` doctest was read for its intended failure reason:

| Snippet | Intended failure | Sound |
|---|---|---|
| `assert_sync::<T>()` for each of the three types | unsatisfied `Sync` bound | yes; the sibling positive `assert_send::<T>()` proves the item resolves |
| `T { fd: todo!() }` / `AuthorizedLaunch { plan: todo!() }` | private fields (E0451) | yes |
| `T::default()` | no `Default` | yes |
| `c.clone()` | no `Clone` | yes |
| `T::from(fd)` / `T::from(bytes)` / `T::from(path)` / `AuthorizedLaunch::from(receipt)` | no `From` | yes |
| `serde_json::from_slice::<T>(…)` | unsatisfied `Deserialize` (E0277) | yes — and the risk of failing for an unresolved crate is excluded by the **positive** P1 doctest in `src/receipt.rs` that deserialises a `serde_json::Value` successfully, which ran and passed locally |
| `admit_executable("/usr/bin/example")` | type mismatch: no pathname form | yes |
| `helm_launch::launch(a)` / `-> LaunchOutcome` | unresolved item | yes |
| the six off-cohort `use helm_launch::…;` blocks | unresolved item off the cohort | yes; and the on-cohort block proves the same names resolve on Linux x86_64 |

The renamed `tests/p1_boundary.rs → tests/p2_boundary.rs` was checked for stale assertions. It no
longer claims any P2 API is absent; the P1 assertion that `authorize`, `admit_executable`,
`admit_working_directory`, `ExecutableCapability`, `WorkingDirectoryCapability` and
`AuthorizedLaunch` do not exist has been correctly replaced by
`the_crate_root_proves_the_off_cohort_absence_of_every_p2_name`, which asserts those names are
absent **off the cohort only**. The README's corresponding stale paragraph was removed. Continued
proof of `launch()`, `LaunchOutcome` and `backend/` absence is retained through
`FORBIDDEN_EVERYWHERE` and `the_crate_declares_exactly_the_p2_modules` (which also pins the module
set at exactly seven and forbids `#[path`, `include!`, `include_bytes!`, `include_str!` and
`cfg_if!` in sources). `tests/plan_contract.rs` is unchanged by this candidate.

**`P2_TYPE_BOUNDARY_TESTS_SOUND`.**

### 20.6 Linux test probity and the re-audit of reported findings

Every Linux-gated test was read for fixtures that would fail before reaching the intended
assertion. The shared `fixture()` helper writes in a loop (so a short `write` cannot truncate a
fixture), chmods after writing, and re-stats with an explicit assertion; `scratch_path()` tolerates
`EEXIST`; `directory_fixture()` does the same. No test depends on a pre-existing host file except
`/dev/null`, used only as a non-regular descriptor.

| Reported finding | Committed form | State |
|---|---|---|
| **P2-IMPL-01** CLOEXEC assumption | the test asserts the flag's actual state with `fcntl_getfd` in all three arms, and `rustix::fs::open`'s no-`O_CLOEXEC` behaviour was verified at the vendored source | **fixed** — not carried |
| **P2-IMPL-02** script length | a 128-byte `#!` fixture plus the short one, with `assert!(long.len() > 64 && short.len() < 64)` | **fixed** — not carried |
| **P2-IMPL-03** cwd permission fixture | entry unlinked before the chmod, `0o400` (readable, not searchable), admission asserted `is_ok()` with an explanatory message | **fixed** — not carried; residual environment sensitivity recorded separately as P2-MIN-03 |
| **P2-IMPL-04** parallel fd-count assumption | `ROUNDS = 64`, `PARALLEL_ALLOWANCE = 16`, a warm-up call before the baseline, and a stated per-round leak floor of eight | **fixed** — not carried |
| **P2-IMPL-05** vacuous non-claim assertion | `a_successful_admission_claims_only_that_nothing_was_detected` now performs a real length-preserving `pwrite` after admission and asserts `assert_ne!` between the first capability's digest and a fresh admission's digest at the same size | **fixed** — not carried |

### 20.7 No test executes an admitted object

`std::process::Command`, `CommandExt`, `exec`, `spawn`, `fork`, `posix_spawn`, `system` and any
shell invocation are absent from product and test code alike, enforced by
`forbidden_vocabulary_appears_nowhere_in_the_crate` over `SOURCES` **and** `TESTS` and reproduced by
independent grep. `launcher_spike` is in the forbidden list and appears nowhere. No helper ELF is
built, shipped or referenced; every ELF fixture is a 64-byte synthetic header, optionally padded,
and none is ever executed. The only build tooling in the candidate is the CI workflow, which runs
`cargo fmt`, `cargo clippy` and `cargo test` and nothing else.

**`P2_TESTS_EXECUTE_NO_ADMITTED_OBJECT_SOUND`.**

### 20.8 CI design

`.github/workflows/helm-launch.yml` is renamed from "helm-launch P1 portable model cross-platform
purity" to "helm-launch portable model and Linux capability admission", and its header comment is
rewritten, so nothing in the file still says P1-only. The portable matrix is unchanged:
`ubuntu-24.04`, `windows-2025`, `macos-15`, `fail-fast: false`, 20-minute timeout, pinned
`actions/checkout` by SHA with `persist-credentials: false`, and `permissions: contents: read` —
least privilege, unchanged.

On every platform the job runs `cargo fmt --check`, `cargo clippy -p helm-launch --all-targets
--all-features --locked -- -D warnings` and `cargo test -p helm-launch --locked`. On `ubuntu-24.04`
that `cargo test` compiles `tests/linux_admission.rs` with its `#![cfg(…)]` satisfied and runs the
`authority.rs` unit tests and the on-cohort doctests, so the Linux-gated P2 cases execute in the
ordinary run. An added Linux-only step lists and re-runs the admission binary so the case names
appear in the log; an added non-Linux step lists the same binary to show it is empty off the
cohort. No root, no `sudo`, no privilege transition, no LAUNCH-EXEC-01 asset, no helper, no
`workflow_dispatch` call and no matrix expansion was added. The existing `workflow_dispatch`
trigger is pre-existing and is the mechanism `AGENTS.md` prescribes for re-running an exact ref.

The new Linux step is a multi-line `run:`, but it executes under `bash -e` on the Linux runner, so
it does not extend the P1-CI-01 backlog item, which concerns `pwsh` blocks on the Windows runner.
The step asserts no minimum case count (P2-BL-03).

**`P2_CI_DESIGN_SOUND`.**

### 20.9 Candidate hygiene

Scanned the whole two-commit candidate: no NUL byte in any changed file, no binary blob
(`git diff --numstat` reports no `-`/`-` pair), no `target/`, build artifact, generated ELF, helper
binary or log, no private host path, no credential or key, no `Co-Authored-By` trailer, no
"Generated with" signature, no AI attribution in any commit message or file, and no unrelated
change. `git diff --check cd5db27 c74e906` is clean. `cargo fmt --check` passes.

**`P2_CANDIDATE_HYGIENE_SOUND`.**

## 21. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| P2-VERIFY-01 | GATE_PENDING | Linux runtime validation | n/a | no local Linux Rust toolchain, so `tests/linux_admission.rs`, the `src/authority.rs` unit tests and the on-cohort doctests have never executed; they compile and lint clean under `--target x86_64-unknown-linux-gnu --all-targets -D warnings` | owner-declared non-defect; hosted Linux CI at publication |
| P2-VERIFY-02 | GATE_PENDING | macOS validation | n/a | no macOS host available; Windows was validated locally in full | hosted CI at publication |
| P2-DOC-01 | MINOR | ADR-0024 header | documentation only | the ADR's "Implementation authority: **HELM-LAUNCH P1 only** … P2 and every later slice are **not authorised**" line is superseded by the 2026-09-18 decision but still reads as current; the ADR's own closing table defers to a separate owner decision, which `DECISIONS.md` and `PROJECT_STATE.md` now supply | owner decides whether to refresh the ADR header at the next authorised ADR touch; the authority commit was correctly forbidden from editing the ADR |
| P2-RISK-01 | MINOR | set-ID fixtures | test environment | `0o4755`/`0o2755` fixtures depend on the host keeping the bit; the helper asserts `st_mode & 0o7777` before admission, so a stripping host fails loudly as a fixture precondition | none required; the loud-failure shape is what the contract asks for |
| P2-MIN-03 | MINOR | working-directory 0o400 fixture | test environment | run as `root`, the not-searchable arm holds vacuously because permission checks are bypassed; it never misreports | none required; the authorised `ubuntu-24.04` runner is unprivileged, so the case is probative there |
| P2-MIN-04 | MINOR | instability seam helper | unit tests only | `with_instability_seam` has no drop guard, so a panic inside `body()` leaves the thread-local seam installed; harmless under libtest's default thread-per-test, but under `--test-threads=1` a panicking seam test could perturb a later admission on the same thread | optional: wrap the reset in a guard at the next authorised touch |
| P2-MIN-05 | MINOR | size-bound tests | test strength | both over-limit tests assert only the refusal code; neither observes that no `pread` occurred, although their names say "before any body read". The ordering is proved by code inspection and the fixtures are sparse | none required; optionally strengthen with a read-visibility probe |
| P2-MIN-06 | MINOR | privacy test | test fragility | `!rendered.contains("fd")` is asserted against a `Debug` string embedding a 64-character hex digest; deterministic for the committed fixture (`fac1ecd4…` contains no `fd`) but a future fixture change could fail it for a non-privacy reason | none required; optionally assert on the structured fields instead |
| P2-MIN-07 | MINOR | README | documentation only | the executable-admission table numbers two consecutive rows "5" (the `NotElf` and `ElfNotInCohort` arms of the same step) | cosmetic; fix at the next authorised touch |
| P2-BL-01 | BACKLOG_NONBLOCKING | error vocabulary portability | no authority either way | `AdmissionError`, `AdmissionErrorCode`, `AuthorizationRefusal` and `AuthorizationRefusalCode` are exported on every platform with no off-cohort producer. This follows plan §7.1 ("errors" portable on every target) and ADR-0024 §B; the P2 decision's §3 sentence "off the cohort the P2 types and functions must not exist in the public API" can be read more strictly. The types carry no descriptor, no `rustix` type and no authority | owner may state which reading governs when the API is stabilised |
| P2-BL-02 | BACKLOG_NONBLOCKING | CI coverage | build only | the matrix has no Linux non-x86_64 runner, so "portable P1 still builds on Linux on another architecture" remains asserted rather than tested. Plan §7.1 classes that target "Level 1 only" and advertises no support | optional: add a check-only cross job, or leave the claim explicitly untested |
| P2-BL-03 | BACKLOG_NONBLOCKING | CI step strength | Linux runner | the "Name the Linux x86_64 capability-admission cases" step prints `--list` for the log but asserts no minimum case count, so a suite that became silently empty would still pass the job | optional: assert a floor on the listed case count |

**BLOCKER: none. IMPORTANT: none.**

### Existing findings carried from the P1 independent review

| ID | State after this candidate |
|---|---|
| **P1-DOC-02** | **RESOLVED and verified.** The README's stale "it has not been independently reviewed" sentence is replaced by a per-slice review-state table, and the receipt section now states per-kind field presence exactly (`errno` mandatory with `pre_exec_failure` and absent otherwise; `reason` for `indeterminate` only; `code` for `exited`; `signal` and `core_dumped` for `signaled`; a stream `errno` only with `read_failed`) |
| **P1-TEST-01** | **OPEN, untouched.** P2 does not change the plan parser, so the escaped duplicate-key vector is unchanged. The commit message and the README both state this correctly |
| **P1-PARSE-01, P1-PARSE-02, P1-LIFE-01, P1-SER-01, P1-CI-01, P1-TEST-02** | unchanged `BACKLOG_NONBLOCKING`. P1-CI-01 is not extended: the one new multi-line `run:` block is Linux-only and runs under `bash -e` |

## 22. Validation actually performed

Fresh `CARGO_TARGET_DIR` outside the repository. Host: Windows 11, `rustc 1.95.0 (59807616e
2026-04-14)`, `cargo 1.95.0`, host triple `x86_64-pc-windows-msvc`, with the
`x86_64-unknown-linux-gnu` std target already installed.

| Command | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | pass, no warning |
| `cargo test --workspace --locked` | pass, all suites green |
| `cargo test -p helm-launch --locked` | 42 lib + 19 `p2_boundary` + 19 `plan_contract` + 17 doctests pass; `linux_admission` 0 tests, as its cohort gate requires off Linux |
| `cargo build -p helm-launch --release --locked` | pass |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | **pass, no warning** — so `src/authority.rs`, its unit tests and `tests/linux_admission.rs` all type-check and lint clean for the cohort, and `linux-raw-sys` compiled while `libc` did not |
| `python -m unittest discover -s tools/tests` | `Ran 784 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | `PASS`; 133 markdown files, 255 JSON files, 1 409 link targets |
| `git diff --check cd5db27 c74e906` | clean |
| independent SHA-256 recomputation of the three committed measurement vectors (Python `hashlib`) | all three identical (section 14) |

No Rust toolchain was installed into WSL, no toolchain component was added, no workflow was
dispatched, nothing was pushed and `main` was not touched.

> **P2 LINUX RUNTIME VALIDATION: PENDING PUBLICATION CI.**
> **macOS: PENDING PUBLICATION CI.**

These are `GATE_PENDING`, not defects, under the owner's clarification of 2026-09-18.

## 23. Recommendation and next gate

Verdicts: `P2_AUTHORITY_AND_COMMIT_SCOPE_SOUND`, `P2_PLATFORM_BOUNDARY_SOUND`,
`P2_DEPENDENCY_GRAPH_SOUND`, `P2_FD_OWNERSHIP_SOUND`, `P2_AUTHORITY_TYPE_BOUNDARY_SOUND`,
`P2_EXECUTABLE_ADMISSION_ORDER_SOUND`, `P2_DESCRIPTOR_MODE_SOUND`, `P2_METADATA_ADMISSION_SOUND`,
`P2_ELF_COHORT_CHECK_SOUND`, `P2_POSITIONAL_MEASUREMENT_SOUND`, `P2_MEASUREMENT_FACTS_SOUND`,
`P2_INSTABILITY_DETECTION_SOUND`, `P2_CLOEXEC_TESTS_SOUND`, `P2_SETID_REFUSAL_SOUND`,
`P2_WORKING_DIRECTORY_ADMISSION_SOUND`, `P2_AUTHORIZE_ZERO_IO_SOUND`,
`P2_AUTHORITY_NON_CONVERSION_SOUND`, `P2_ERROR_MODEL_SOUND`, `P2_PRIVACY_BOUNDARY_SOUND`,
`P2_SIDE_EFFECT_DOCUMENTATION_SOUND`, `P2_TYPE_BOUNDARY_TESTS_SOUND`,
`P2_TESTS_EXECUTE_NO_ADMITTED_OBJECT_SOUND`, `P2_CI_DESIGN_SOUND`, `P2_CANDIDATE_HYGIENE_SOUND`.

Scope, restated as proved in section 4:

```text
PROCESS CREATION: NONE
PROCESS EXECUTION: NONE
UNSAFE: NONE
HOST PRIVILEGE: NONE
EXPERIMENT EXECUTION: NONE
P3+ IMPLEMENTATION: NONE
```

> **P2 CANDIDATE MAY BE PUBLISHED FOR THE AUTHORISED LINUX CI GATE.**
> **P3 REMAINS NOT AUTHORISED.**

**Next gate:** one fast-forward publication of the P2 authority record, the P2 implementation and
this independent review, followed by Linux runtime CI. The seven MINOR findings and three backlog
items do not block that gate. If hosted Linux CI fails, the first failure is to be preserved and
returned to the owner rather than iterated against.

This review changes no implementation file, authorises nothing beyond P2, adds no process creation,
no `launch()` and no `unsafe` code, dispatched no workflow, pushed nothing, and leaves `main`
unchanged. Trial #3 remains `MECHANISM_REJECTED` and must not be rerun. No Trial #4 is authorised.

**Classification: `HELM_LAUNCH_P2_INDEPENDENT_REVIEW_PASSED_READY_FOR_PUBLICATION_CI`.**
