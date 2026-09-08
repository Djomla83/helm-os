# helm-evidence independent review and Linux validation

## Initial written source review — before corrections

Review date: 2026-09-08. Candidate `product/helm-evidence` at
`a4d0a0b120d88ec41c3343f4f622cecc79a0d4f4`. Fetch verified this remote tip;
its sole parent and fetched main are
`ae3f012fb1bfd7b20018c3faf6c71a6740a041fe`. The working tree was clean.
Review work uses `review/helm-evidence-linux`, descended from the candidate.
No author history, historical evidence, ADR, licence or implementation has been
changed at this initial pass. The author's passing-test report is not proof.

Source traced: all five product Rust files and all Rust tests, workspace manifests
and lockfile, the A0 adapter/fixture and preregistered A0 contract, README, project
state/master plan, decision index, RFC-0001/0002, ADR-0012/0015 and implementation
starting/review records. Dependency source was inspected from Cargo's pinned cache.

### Findings at candidate locations

| Severity / ID | Exact location | Scenario and reproducibility | Minimum correction |
|---|---|---|---|
| IMPORTANT R1 | `crates/helm-evidence/src/bundle.rs:49-78`, especially `:74` | Metadata rejects links, but `Dir::open` subsequently resolves the entire path with following enabled. Replace a checked regular artifact or parent directory with a relative symlink to matching bytes inside the bundle. The returned handle is regular and its bytes hash correctly, permitting COMPLETE despite the no-symlink policy. The same gap affects `bundle.json`. Source-established interleaving; execution reproduction pending. This is policy violation B, not evidence of containment escape A. | Open every component without following links, pin directory handles, and validate the opened object; preserve cap-std containment. Reject links during the open itself, not just a path-based precheck. |
| IMPORTANT R2 | `crates/helm-evidence/src/bundle.rs:74-76` | Replace a checked regular file with a FIFO with no writer. Blocking read-only open waits before the later regular-file check or any byte bound. cap-primitives defaults `nonblock` to false. Source-established interleaving; bounded execution reproduction pending. | Nonblocking leaf open followed by handle-based regular-file/reparse validation, without reading a special object. Retain bounded child-process regression for FIFO replacement. |
| IMPORTANT R3 | `crates/helm-evidence/src/model.rs:173-175`; `src/oracle.rs:26-34,98-109` | `ContentManifest.files` deserializes directly into a BTreeMap. Serde inserts entries, silently overwriting duplicate decoded keys. A manifest can contain conflicting expectations for the same member; placing the matching one last can yield COMPLETE. Typed struct duplicate rejection does not protect map entries. Reproduce using duplicate JSON member paths (including escaped equivalent spellings) and update the synthetic manifest/oracle bindings. Execution pending. | Reject duplicate decoded file keys during deserialization, before insertion; keep the existing schema. |
| MINOR R4 | `crates/helm-evidence/src/lib.rs:193-196`; `src/model.rs:265-268` | Workflow IDs may be `contract`, `artifacts` or `restart`, producing duplicate section IDs with different verdicts. A consumer selecting a section by ID can select the wrong result. Deterministic synthetic rename reproducer pending. | Reserve the three built-in section IDs in workflow declaration validation. |
| BLOCKER | No established finding | No containment escape established by source inspection. | Execution remains required before a merge recommendation. |

### NO FINDING and explicit limits

- Verdict aggregation makes INVALID dominate INCOMPLETE. Experiment FAIL and
  INCONCLUSIVE remain executed observations; BLOCKED/NOT_RUN leave evidence
  incomplete. Content identity does not substitute for the declared destination.
  Workflow sections include their own references/restart dependency. A0 expectations
  come from preregistration and immutable records, not the Rust implementation.
- Contract/workflow/restart objects reject unknown/duplicate typed fields; oracle
  auxiliary fields are deliberately ignored. u64 lengths, input limits, saturating
  read accounting and the normal JSON recursion limit avoid obvious integer wrap
  and unbounded input reads. Parsed allocation/output can exceed input byte limits;
  these are not memory or wall-clock guarantees.
- Storage path spelling excludes traversal, absolute paths, separators/streams,
  non-ASCII, trailing dot/space and Windows devices. Duplicate storage paths are
  ASCII case-folded; actual lookup follows filesystem case behavior. Output path
  comparison is exact. Unicode member names are bounded data, never host paths.
- The root is explicitly caller-selected. Hardlink origin, mount policy, atomic
  snapshots and truthful capture are explicitly outside the contract. Hashes bind
  the retained bytes, not the inode initially observed or a durable filesystem
  snapshot. Ordinary replacement/deletion need not always be rejected if valid
  bytes were already opened. These limits do not excuse following a symlink.
- Product filesystem access is centralized; no authored unsafe block, execution,
  networking, privilege request or bundle write path was found. CLI output handles
  write failure. Diagnostics use fixed explanations and validated IDs, suppressing
  raw parser/OS errors and filesystem paths. Untrusted data can declare an ID but
  cannot cause a raw private absolute path to be echoed through verification.
- Static link/reparse/type tests cover useful states but do not cover the open
  interleavings above. Windows device/reparse behavior and Unix special objects
  require execution; no unsupported platform audit is claimed.

### Upstream basis

Pinned `cap-std 4.0.2` / `cap-primitives 4.0.3` source: `fs/open_options.rs`
defaults to following links and blocking opens; `rustix/linux/fs/open_impl.rs`
uses BENEATH plus NO_MAGICLINKS, not NO_SYMLINKS, with a manual confined fallback;
`rustix/fs/oflags.rs` adds NONBLOCK only when requested. These are distinct
containment and no-symlink policies. Primary references:
[Dir API](https://docs.rs/cap-std/4.0.2/cap_std/fs/struct.Dir.html) and
[Windows device advisory](https://github.com/bytecodealliance/cap-std/security/advisories/GHSA-hxf5-99xg-86hw).
The advisory is fixed before the pinned version; this is not an exhaustive advisory audit.

The four direct dependencies each serve a bounded purpose: cap-std confines I/O,
Serde supplies typed records, serde_json parses/serializes JSON, and sha2 supplies
SHA-256. No replacement or upgrade sweep is proposed. Lockfile/licence inventory,
execution receipts and the final assessment will be appended after validation.

## Execution and final disposition

**Corrected module: READY_FOR_OWNER_MERGE**, subject to owner review of this report
and the correction. The original candidate alone **NEEDS_FIXES**. No merge was
performed. Corrective code/test/CI commit:
`83907b0c1d7f305b9271292e7582b4de831e2469`. Later report/receipt-only commits do not
change those tested product bytes. Main and the author's candidate remain intact.

### Reproduced findings and corrections

The initial source pass was committed as
`d0705a3fccd19bd96b7c4c11424b6cf97789aeff`. The next commit,
`ed49358b279c1ba8bcfe4e025f5fc3ac96f4125a`, preserved failing expectations and pre-fix execution before product
corrections. No failing test was removed or relaxed to obtain a pass.

| ID | Independent pre-fix execution | Correction and post-fix result |
|---|---|---|
| R1 IMPORTANT, resolved | Unmodified Linux release binary: forced internal file, manifest and directory symlink swaps all returned COMPLETE. Escaping equivalents returned INVALID and the selected open returned an error rather than a file descriptor. | `bundle.rs` opens each parent with `open_dir_nofollow`, keeps its capability, and opens the leaf with `FollowSymlinks::No`. Opened directory/file metadata is checked for reparse/type policy. All forced symlink opens now return INVALID. |
| R2 IMPORTANT, resolved | FIFO swap reached the real blocking open and timed out at five seconds; child was killed/reaped. Static FIFO was already INVALID. | Unix leaf opens request nonblocking mode; opened FIFO is rejected as PATH_UNSAFE without reading its stream or waiting for a writer. The same forced swap now returns INVALID promptly. |
| R3 IMPORTANT, resolved | New Windows regression returned COMPLETE for contradictory duplicate content expectations. | `model.rs::unique_content_files` rejects duplicate decoded keys and the existing 32-entry bound while parsing. Ordinary/escaped duplicate cases now return INVALID/RECORD_FORMAT on both platforms. No schema redesign. |
| R4 MINOR, resolved | New Windows regression produced two `contract` sections and COMPLETE. | `validate_contract` reserves the three built-in section names; all three now return INVALID/WORKFLOW_DECLARATION. |

Evidence: [original Linux matrix](evidence/helm-evidence-independent-2026-09-08/candidate-linux-validation.json),
[original races](evidence/helm-evidence-independent-2026-09-08/candidate-linux-filesystem.json),
[failing Windows regressions](evidence/helm-evidence-independent-2026-09-08/candidate-windows-regressions.json),
and [corrected filesystem results](evidence/helm-evidence-independent-2026-09-08/fixed-linux-filesystem.json).
The panics in the failing regression receipt are test assertions demonstrating a
wrong verdict, not malformed-input panics in the verifier.

### Linux environment and tests

Used the existing `helm-lab-desktop-7zip` Hyper-V VM with 4 vCPUs and fixed 8 GiB RAM
on the Ryzen 9 5950X host. Guest: Ubuntu 24.04.4 LTS, Linux `7.0.0-31-generic`, ext4;
Rust/Cargo 1.95.0, LLVM 22.1.2, target `x86_64-unknown-linux-gnu`, GCC 13.3.0.
Rust's release/commit matched Windows; only the target differs from
`x86_64-pc-windows-msvc`. All builds/verifier tests ran as UID/GID 1001 (`helmlab`),
with no supplementary groups. No root-based rerun made a test pass.

Setup installed Ubuntu's `build-essential` and `git` with their 35 new packages,
zero package upgrades/removals, then official rustup with minimal Rust 1.95.0,
rustfmt and Clippy. Rust/cache/source trees are under a new review directory;
shell profiles and A0 working files were not changed. Guest administrative setup
and orderly shutdown are separate from test execution. No GUI, Wine or application
workflow was run. Normal WSL Ubuntu and the three WSL labs were not started.
See [setup and preservation](evidence/helm-evidence-independent-2026-09-08/lab-preservation.json).

| Check | Original candidate, Linux | Corrected commit, Linux | Corrected commit, Windows |
|---|---|---|---|
| fmt | PASS | PASS | PASS |
| Clippy, all targets/features, warnings denied | PASS | PASS | PASS |
| cargo test | 42 PASS | 45 PASS | 45 PASS |
| Release build, locked dependencies | PASS | PASS | PASS |
| Synthetic contract | COMPLETE, exit 0 | COMPLETE, exit 0 | COMPLETE, exit 0 |
| A0 contract | INCOMPLETE, exit 1 | INCOMPLETE, exit 1 | INCOMPLETE, exit 1 |
| A0 W1 / W2 / experiment | INCOMPLETE / COMPLETE / FAIL | Same | Same |

The author reported 43 Windows tests. Linux has no Windows-junction test; corrected
Linux adds one integration test exercising 24 filesystem cases, and both platforms
add two review regression tests. These explain the counts; none were ignored.
The W1 failure remains exactly OUTPUT_DESTINATION with CONTENT_RESULT_PASS satisfied.

Reproducible commands: `cargo fmt --check`;
`cargo clippy --all-targets --all-features --locked -- -D warnings`;
`cargo test --locked`; `cargo build --release --locked`; then
`target/release/helm-evidence verify crates/helm-evidence/tests/fixtures/synthetic --json`
and the corresponding `a0-7zip` invocation. `cargo fetch --locked` preceded offline
dependency use. On Windows the binary has `.exe` suffix.

Final receipts: [Linux](evidence/helm-evidence-independent-2026-09-08/final-linux-validation.json)
and [Windows/repository/privacy](evidence/helm-evidence-independent-2026-09-08/final-windows-validation.json).
Both identify the corrective commit; all Cargo manifests/lockfile and product
source SHA-256 values agree across their receipts. Validation was also rerun from
a clean Linux worktree of that exact commit.

### Hostile filesystem outcomes and limits

The [Linux probe](../../tools/helm_evidence_linux_probe.py) uses unprivileged
parent-to-child ptrace on Linux x86_64. It stops the unmodified binary at the actual
selected open syscall, changes only synthetic fixture objects, and resumes without
modifying code, registers or syscall results. It records whether injection occurred,
the selected open result, verdict/codes, deadline and exit. Every child is bounded
to five seconds and killed/reaped if needed. Static cases also have bounded invocations.

| Objects/interleaving | Corrected behavior |
|---|---|
| Internal/escaping file link; internal/escaping directory link; dangling link | INVALID, PATH_UNSAFE for static links |
| FIFO, Unix socket, directory in place of file | INVALID, PATH_UNSAFE |
| Device reference | Symlink to harmless `/dev/null` rejected; actual device creation was not attempted because it needs privilege |
| Hardlink to synthetic outside file | COMPLETE with matching bytes; external alias mutation gives INVALID/ARTIFACT_HASH. Origin is deliberately not certified. |
| File/manifest/directory symlink swap before open | INVALID/ARTIFACT_UNREADABLE; no followed symlink accepted |
| FIFO swap before open | INVALID/PATH_UNSAFE, no timeout |
| Delete before open | INCOMPLETE/ARTIFACT_MISSING |
| Wrong regular file or directory replacement | INVALID/ARTIFACT_HASH |
| Matching regular file or manifest replacement | COMPLETE; retained bytes match the contract |
| Directory replaced after its capability was opened | COMPLETE from the pinned original directory, despite different bytes at the replacement pathname; no symlink traversal or snapshot claim |

**A — containment:** no outside-root symlink escape observed in candidate or fix.
The original escape tests produced failed opens, not canary descriptors. This is
execution evidence for these interleavings, not a proof for every OS/filesystem.
**B — strict no-symlink policy:** positively violated by the candidate's internal
swaps, corrected without weakening policy. **C — benign mutation:** missing/wrong
bytes reject as above; an already-opened capability or matching replacement may
legitimately complete. Precheck inode equality and an atomic whole-tree snapshot
are neither promised nor inferred from a digest match.

No verifier panic was observed. The only verifier hang was the pre-fix FIFO open.
Manual cap-std fallback on older Linux, Windows race scheduling, arbitrary Windows
reparse types, actual block/character devices, hostile mounts/FUSE and non-x86_64
Linux remain unexecuted. Windows static symlink/junction/device-name regressions
passed. Tests do not establish a universal wall-clock guarantee or hardlink origin.

### Cross-platform semantics, parsing and publication

[Windows reports](evidence/helm-evidence-independent-2026-09-08/windows-semantics.json)
and [Linux reports](evidence/helm-evidence-independent-2026-09-08/linux-semantics.json)
were collected by [one script](../../tools/helm_evidence_compare.py). Entire canonical
JSON reports match for 21 of 22 cases, including fixture verdicts, sections and
machine failure codes. The sole difference is native filesystem case lookup:
`GOOD.JSON` names existing `good.json` on tested Windows NTFS but is missing on
Linux ext4. This is explicitly documented; case-folded duplicate declarations and
case changes in recorded output destinations have identical semantics.

Malformed UTF-8, lone surrogate, duplicate typed fields and overflowing u64 values
return INVALID. Deep **ignored** oracle metadata completes: serde_json's
`de.rs::ignore_value` uses an iterative scratch stack, rather than recursive typed
deserialization. The README now explains this distinction. Byte limits still apply.
Unicode content names are exact decoded strings, not normalized host paths.
Human and JSON outputs did not echo the synthetic private marker or absolute
fixture root. No network, process-execution or bundle-write API was added to product code.

Repository validation passed on Windows and Linux: documentation checks, all 36
Python tests and the exact 22-file A0 adapter. The existing privacy/publication audit
passed on Windows, checked all 153 historical publication hashes, decoded command
streams, protected historical paths and indexed fixture bytes. It remains a bounded
scan, not a comprehensive secret detector. The preserved guest A0 tree has the same
2,214-entry inventory digest before and after (regular contents, names, types/modes,
lengths and link targets; atime excluded). Generated target binaries are untracked.
The VM was shut down normally after preservation checks and returned to Off.

### Dependencies and CI

The [dependency inventory](evidence/helm-evidence-independent-2026-09-08/dependencies.json)
checks 60 cached crate archives against Cargo.lock checksums and records licence
metadata/build-script/proc-macro flags. All original 59 dependency pins are unchanged.
Only `cap-fs-ext =4.0.2` was added, using already-locked dependencies. Its matching
no-follow/nonblocking API avoids a new OS wrapper or replacement of cap-std.
Its build script probes compiler features; this is build-time process execution,
not execution from bundle contents. The dependency runtime call path remains file I/O.

cap-std and cap-fs-ext declare `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`;
serde, serde_json and sha2 declare `MIT OR Apache-2.0`. These are package metadata,
not acceptance of a HELM licence. No broad upgrade, vendoring or production release.
Advisory checking was targeted at cap-std's documented device fix, not exhaustive.

The minimal [Linux workflow](../../.github/workflows/helm-evidence.yml) uses only
GitHub-hosted Ubuntu 24.04, immutable checkout SHA, non-persisted checkout credentials,
read-only contents permission and pinned stable Rust 1.95.0. It checks fmt, Clippy,
tests and release build, with a 15-minute job deadline and module path filters.
No `pull_request_target`, secret input, privileged/self-hosted runner, cache,
deployment, signing, release or artifact publication was added. This follows
[GitHub's untrusted-code guidance](https://docs.github.com/en/actions/reference/security/secure-use).
[CI run 34187102166](https://github.com/Djomla83/helm-os/actions/runs/34187102166)
passed for corrective commit `83907b0c1d7f305b9271292e7582b4de831e2469`;
[machine receipt](evidence/helm-evidence-independent-2026-09-08/ci-result.json).

### Failures retained and owner boundary

Development failures beyond the actual findings: SSH was initially attempted while
the VM was still booting; an early clone found git not yet installed; rustup warned
that the linker installation was still pending. The first ptrace probe had a Python
bytearray-to-path TypeError, corrected to bytes before producing the retained race
receipt. A README patch context mismatch changed no files and was reapplied.
None changed the expected outcomes or the application's historical evidence.

No unresolved BLOCKER or IMPORTANT finding remains for this bounded module.
Authentication, freshness, truth of capture, application correctness and broader
HELM subsystems remain outside scope. The owner is the final approver of the
correction; the reviewer who wrote it is not independently approving its release.
Recommended next decision: review and merge the **corrected review branch**, if
accepted, preserving the original candidate ancestry. Do not merge the uncorrected
candidate alone. No further HELM module is authorised or started by this work.
