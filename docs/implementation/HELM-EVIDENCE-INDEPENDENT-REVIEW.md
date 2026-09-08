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

Pending. Do not merge on this initial source pass.
