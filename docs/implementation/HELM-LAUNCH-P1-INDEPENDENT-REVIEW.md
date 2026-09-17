# helm-launch P1 independent review

> **THIS IS AN INDEPENDENT REVIEW OF THE HELM-LAUNCH P1 TWO-COMMIT CANDIDATE.**
>
> **IT DOES NOT AUTHORISE P2.**
>
> **IT DOES NOT ADD PROCESS EXECUTION OR UNSAFE CODE.**

**Role:** fresh independent adversarial review of the P1 candidate. This review session authored
neither `427b1af` nor `d3914ab`; both commits carry the repository owner's identity and were
produced before this review began.\
**Reviewed candidate:** `076c2b046554ba9a9f401d7fcf4861821831f8e3` (published P1 authority base)
→ `427b1af092db29c619b7f7a4c0d40b72efaacc65` (P1 implementation) →
`d3914ab95fb253abd8ac10462a4e2de1cb2185de` (pre-review correction), on
`docs/helm-launch-architecture`.\
**Authority:** Accepted [ADR-0024](../adr/ADR-0024-launch-authority.md), the owner-reviewed
[productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md), and the
[owner decision of 2026-09-17](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised):
**P1 authorised, P2+ not authorised.** The owner's P1 finding rulings F1–F8 and the owner's T40
clarification given with this review instruction are applied as current owner authority.\
**Scope of this document:** review only. No implementation file, plan, ADR, decision record,
project state, experiment, evidence or workflow is changed. Nothing was pushed and no CI was
dispatched.

**Classification: `HELM_LAUNCH_P1_INDEPENDENT_REVIEW_PASSED_READY_FOR_PUBLICATION_CI`.**
No BLOCKER and no IMPORTANT finding. Three MINOR findings and six non-blocking backlog items are
recorded in section 20.

## 1. Starting state, independently reconstructed

| Claim | Method | Result |
|---|---|---|
| worktree clean | `git status --short` | empty |
| branch | `git branch --show-current` | `docs/helm-launch-architecture` |
| local HEAD | `git rev-parse HEAD` | `d3914ab95fb253abd8ac10462a4e2de1cb2185de` |
| remote milestone | `git fetch origin`; `git rev-parse origin/docs/helm-launch-architecture` | `076c2b046554ba9a9f401d7fcf4861821831f8e3` |
| main | `git rev-parse origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |
| exact chain | `git log --oneline -4` | `076c2b0` → `427b1af` → `d3914ab` |
| divergence | `git rev-list --left-right --count HEAD...origin/docs/helm-launch-architecture` | 2 ahead, 0 behind |

## 2. Authority read before code

Read in full before the diff: `AGENTS.md`,
[ADR-0024](../adr/ADR-0024-launch-authority.md), the
[productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md), the P1 decision in
[`DECISIONS.md`](../DECISIONS.md#adr-0024-accepted-helm-launch-p1-authorised) and the current
sections of [`PROJECT_STATE.md`](../PROJECT_STATE.md). Then the complete candidate diff
`076c2b0..d3914ab` (15 files, 6 522 insertions, 0 deletions), every source and test file in
full, and each commit separately. The implementation was judged against the accepted contract,
not against the LAUNCH-EXEC-01 spike.

The binding P1 boundary is the owner decision's list: no `unsafe`; no `clone3`, `execveat`,
syscall shim, Linux backend, process creation or execution, pidfd, `pidfd_send_signal`,
`waitid`, `close_range`, `fchdir`, process-group signalling, signal manipulation,
`PR_SET_NO_NEW_PRIVS` call, executable or working-directory descriptor admission, ELF or
measurement I/O, `launch()`, or reuse of the experimental runner.

## 3. Commit and file scope

| Commit | Files | Assessment |
|---|---|---|
| `427b1af` | `Cargo.toml`, `Cargo.lock`, `crates/helm-launch/**` (11 files), `.github/workflows/helm-evidence.yml`, `.github/workflows/helm-launch.yml` | the P1 implementation candidate; nothing outside the authorised areas |
| `d3914ab` | `crates/helm-launch/{Cargo.toml, README.md, src/lib.rs, src/lifecycle.rs, tests/p1_boundary.rs}` | bounded: T40 latch in `lifecycle.rs` with its tests; crate-root `deny` instead of `forbid` in `lib.rs` with its boundary test; the matching manifest comment and README wording. No dependency, lockfile, workflow or public API change |

`git diff --name-status 076c2b0..d3914ab` touches only `Cargo.toml`, `Cargo.lock`,
`crates/helm-launch/**`, `.github/workflows/helm-evidence.yml` and
`.github/workflows/helm-launch.yml`. `DECISIONS.md`, `PROJECT_STATE.md`, ADR-0024, the
productization plan, every LAUNCH-EXEC-01 experiment file, preserved evidence and every trial
workflow are unchanged. Both commit messages carry no AI attribution and no `Co-Authored-By`.

**`P1_COMMIT_SCOPE_SOUND`.**

## 4. Absolute P1 authority boundary

Independent scans of `crates/helm-launch/src` and `tests`, with comments included where stated:

| Check | Result |
|---|---|
| `ExecutableCapability`, `WorkingDirectoryCapability`, `AuthorizedLaunch`, `LaunchOutcome`, `AdmissionError`, `AuthorizationRefusal`, `LaunchError` | absent; the only occurrences of `authorize`, `admit_executable`, `admit_working_directory` and `launch` are inside `compile_fail` doctests proving their absence |
| files | exactly `Cargo.toml`, `README.md`, six `src` modules plus `lib.rs`, two integration tests; no `backend/`, `authority.rs`, `launch.rs` or Linux module |
| the word `unsafe` in `src`, comments included | zero |
| `extern`, `asm!`, `#[link]`, `no_mangle`, `std::fs`, `std::process`, `std::os`, `std::env`, `std::net`, `std::thread`, `std::time`, `std::io`, `OwnedFd`, `RawFd`, `libc`, `rustix` | zero |
| `cfg(target_os …)` gating | none: the crate is uniformly portable |
| dependencies able to reach the OS on the crate's behalf | `serde`, `serde_json`, `sha2` only, used for parsing and hashing in memory |
| tests | pure; `include_str!` inspection is compile-time. No test spawns a process |

Compile-time confirmation from outside the crate (section 17): `use helm_launch::launch`,
`authorize`, `admit_executable` and `admit_working_directory` fail `E0432`, and naming
`helm_launch::ExecutableCapability` or `helm_launch::LaunchOutcome` fails with "cannot find type".

```text
PROCESS EXECUTION: NONE
UNSAFE: NONE
HOST PRIVILEGE: NONE
EXPERIMENT EXECUTION: NONE
P2+ IMPLEMENTATION: NONE
```

## 5. Dependency and workspace

* `crates/helm-launch` is a workspace member (root `Cargo.toml`, one added line).
* Package: `name = "helm-launch"`, `version = "0.1.0"`, `edition = "2024"`,
  `rust-version = "1.95"`, `publish = false`.
* Direct dependencies exactly `serde = "=1.0.228"`, `serde_json = "=1.0.149"`,
  `sha2 = "=0.10.9"`. No dev-, build- or target-specific dependencies, no features, no
  `build.rs`, no HELM crate, no `rustix`, no `libc`.
* `git diff 076c2b0..d3914ab -- Cargo.lock` adds exactly one local `[[package]]` entry,
  `helm-launch 0.1.0`, depending on `serde`, `serde_json`, `sha2`, with no `source`. No other
  package or version changes.

**`P1_DEPENDENCY_GRAPH_SOUND`.**

## 6. Lint and unsafe policy

* The manifest does not inherit `[lints] workspace = true`. It restates
  `[lints.rust] unsafe_code = "deny"`, `unsafe_op_in_unsafe_fn = "deny"` and
  `[lints.clippy] unwrap_used`, `expect_used`, `panic`, `undocumented_unsafe_blocks`,
  `multiple_unsafe_ops_per_block`, all `deny` — exactly the plan section 7.4 table.
* Every workspace restriction is preserved: `clippy::unwrap_used`, `expect_used` and `panic` at
  `deny`. The only difference from the workspace is the accepted `unsafe_code` `deny` in place of
  `forbid`.
* `src/lib.rs` carries `#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]`, not `forbid`
  (owner ruling F4), plus `#![deny(missing_docs)]`.
* No `allow`, `expect`, `warn` or `forbid` of either unsafe lint exists anywhere in `src`,
  including through `cfg_attr`. The only local lint relaxations are `dead_code` for the
  test-only producer, planner and model, and `clippy::unwrap_used`/`expect_used`/`panic` inside
  `#[cfg(test)]` modules and integration tests.
* Actual unsafe blocks, functions, traits or impls: **zero**.
* The other four members still inherit the workspace table; `tests/p1_boundary.rs` pins this, the
  exact crate table, the exact root attribute, and that each unsafe lint is named as code once, in
  `lib.rs`.

**`P1_LINT_POLICY_SOUND`.**

## 7. Public API inventory

Inventoried independently from `pub` items in `src` and the `lib.rs` re-exports.

| Item | Kind | Public surface | Plan §5.2 | Assessment |
|---|---|---|---|---|
| `parse_launch_plan(&[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors>` | fn | the only constructor of a plan | yes | intent only |
| `ValidatedLaunchPlan` | struct, private fields; `Clone`, `Debug`, `PartialEq`, `Eq` | `exact_bytes`, `sha256`, `argv`, `working_directory_id`, `capture_prefix_bytes`, `timeout_ms`, `grace_ms`, `asserted_subject_spec_sha256`, `asserted_binding_report_sha256`; **extra:** `execution_kind`, `environment_mode`, `stdin_mode`, `termination_signal` | yes, plus four accessors | extras return single-member closed enums the parser already fixed; inert |
| `Digest` | tuple struct, private `[u8; 32]`; `Copy`, `Eq`, `Ord`, `Hash`, `Debug`/`Display` as hex | `as_bytes`, `to_hex` | yes | no public constructor or parser |
| `LaunchReceipt` | struct, private fields; `Clone`, `Eq`, `Hash`, custom `Debug` | `exact_bytes`, `sha256`, `record` | yes | no public producer |
| `ReceiptRecord` | struct, `pub(crate)` fields; `Clone`, `Debug`, `Eq`, `Hash` | read accessors for every schema field, `stream(Stream)` | yes | read-only |
| `AssertedContext`, `ExecutableMeasurement`, `Termination`, `StreamFacts` | structs, `pub(crate)` fields | read accessors | implied by `record()` | read-only, no constructor |
| `ExecStatus`, `IndeterminateReason`, `ChildEnd`, `GroupSweep`, `Completeness`, `ChildStage`, `ElfType`, `Backend`, `EnvironmentMode`, `Stream` | `#[non_exhaustive]` enums | `as_str` only | yes (§10.2, §11.2, §11.3) | accepted receipt vocabulary |
| `ExecutionKind`, `StdinMode`, `TerminationSignal` | `#[non_exhaustive]` enums | `as_str` only | implied by §6.1 | extra, inert |
| `LaunchPlanErrors` | struct, private `Vec`; `std::error::Error` | `as_slice`, `codes`, `contains` | yes | read-only |
| `LaunchPlanError` | struct, private fields | `code`, `locator`, `index` | implied | read-only |
| `LaunchPlanErrorCode` | `#[non_exhaustive]` enum, 25 variants | `as_str`, `Display` | yes (§13.2 family) | fixed codes |
| `MAX_PLAN_BYTES`, `MAX_ARGS`, `MAX_ARG_BYTES`, `MAX_ARGV_TOTAL_BYTES`, `MAX_CAPTURE_BYTES`, `MIN_TIMEOUT_MS`, `MAX_TIMEOUT_MS`, `MAX_GRACE_MS`, `MAX_JSON_DEPTH`, `MAX_PLAN_ERRORS`, `MAX_ID_BYTES`, `MAX_RECEIPT_BYTES` | consts | values of §8.1 and the owner's F3 ruling | yes, plus the depth, error-cap and id bounds | inert |

**F7, the extra read-only surface.** Every extra item exposes inert P1 data, creates no
authority, has no constructor, and names only values the accepted plan schema already fixes. None
forces a backend design: `Backend` and `ChildStage` are the accepted receipt vocabulary of plan
sections 8.4 and 11.2, and all enums are `#[non_exhaustive]`. No `pub` field exists on any type.
The `pub(crate)` constructors (`LaunchReceipt::from_record`, `Digest::from_raw`, `Digest::of`,
`Digest::parse_hex`, `LaunchPlanErrors::new`) are unreachable from outside the crate, confirmed
by `E0624` probes.

No public P2-shaped authority or execution API exists.

**`P1_PUBLIC_API_MINIMAL_AND_WITHIN_AUTHORITY`**, with the F7 extras accepted as inert.

## 8. Plan parser

**Structure.** `parse_launch_plan` checks `MAX_PLAN_BYTES` (32 768) first, then builds a private
bounded tree through a `serde` `DeserializeSeed` visitor over `serde_json::Deserializer::from_slice`,
never through `serde_json::Value`. Each object key is decoded with `next_key::<String>` and checked
against a `BTreeMap` before insertion, so duplicates are compared as **decoded** text at every
depth, including inside unknown fields and arrays. Container depth is checked on entry, so a
10 000-deep array is refused after nine frames. `parser.end()` refuses trailing content. The
exact input bytes are kept, and `Digest::of(bytes)` hashes them unchanged. No I/O exists on any
path.

**Reviewer probes** (temporary, uncommitted, run against the public API from a separate crate
outside the repository):

| Input | Result |
|---|---|
| `"schema"` beside `"schema"`; `"schema"` beside `"schema"`; `"mode"` beside `"mode"` nested; `"k"` beside `"k"` inside an unknown array; `"a\/b"` beside `"a/b"`; a surrogate-pair escape beside the literal character | `DUPLICATE_KEY at plan` for all six |
| `timeout_ms`: `-0`, `0.0`, `-0.0`, `1E3`, `3e4`, `30000.0`, `-9223372036854775809` | `TYPE_MISMATCH at timeout_ms` |
| `timeout_ms`: `030000`, `+1`, `1_0`, `NaN`, `Infinity` | `MALFORMED_JSON at plan` |
| `timeout_ms`: `18446744073709551615` | `TIMEOUT_OUT_OF_RANGE` (see P1-PARSE-01) |
| `asserted_context`: `null`, `[]`; member `null` | `TYPE_MISMATCH` at the exact locator |
| `asserted_context`: `{}` | accepted, no assertion (see P1-PARSE-02) |
| capability id `"work\u0000dir"` | `ID_GRAMMAR` |
| key `"sche\u0000ma"` | `UNKNOWN_FIELD at plan`, key text not echoed |
| overlong `C0 80` and raw surrogate bytes inside a string | `MALFORMED_JSON` |
| one argv element of 1 024 escaped surrogate pairs (4 096 decoded bytes) / 1 025 pairs | accepted / `ARG_TOO_LONG at argv[0]`: the bound is on decoded bytes |
| structural precedence: duplicate then malformed; malformed then duplicate; over-deep then duplicate | first structural failure in stream order, one finding |
| 64 non-string argv elements plus schema, version and timeout errors | 64 findings, last `ERROR_LIMIT`; `Display` 1 710 bytes |
| whitespace-only respelling | same argv, different digest; digest equals an independent SHA-256 of the exact bytes |

**Contract checks.** Unknown keys at the root and in every nested object are refused without
echoing the key. `argv` must be a non-empty array of at most 64 strings; each element is checked
for decoded NUL and a 4 096-byte decoded length, with a 131 072-byte total rule that is
unreachable through a 32 768-byte document and unit-tested directly. `argv[0]` is caller data.
`environment.mode` is only `empty`, `stdin.mode` only `closed_pipe_eof`, `termination.signal`
only `SIGTERM` (decoded equality, so an escaped spelling is the same string). Capture bounds
`0..=65 536`, `timeout_ms` `1..=600 000`, `grace_ms` `0..=60 000`, all inclusive and integer-only.
The id grammar is exactly `[a-z0-9][a-z0-9._-]{0,79}`. Asserted digests are exactly 64 lowercase
hex characters; absence is by omission; `null` is refused (owner ruling F3).
`MAX_JSON_DEPTH = 8` counts containers including the root. No executable, path, shell, command,
environment-entry, descriptor, success-condition, Wine, prefix or runtime field exists.

**Errors.** Findings are sorted by code, locator and index, de-duplicated, and capped at 64 with a
final `ERROR_LIMIT`. Structural failures return exactly one document-level finding. Locators are
`&'static str` chosen by the crate and indices are integers, so no document text can reach an
error or its `Display`. The committed 40 000-round arbitrary-byte and mutation test found no panic
or inconsistency, and the reviewer's own probes above added none.

**Test weakness.** The committed duplicate-key test claims "literal and escaped spellings" at top
level, but its first two vectors are byte-identical literal duplicates, so no committed test
proves escaped-key collision (P1-TEST-01). The implementation behaviour is correct, as shown above.

**`P1_PLAN_PARSER_SOUND`**, with P1-TEST-01 (MINOR) and P1-PARSE-01, P1-PARSE-02 (backlog).

## 9. Digest

* Representation: private `[u8; 32]`; no public constructor, `From`, `Default` or parser.
  Construction outside the crate fails (`E0423`, `E0624`, `E0277` probes).
* Parsing (crate-private, used for asserted digests): exactly 64 bytes from `[0-9a-f]`;
  uppercase, prefixes, whitespace, short, long and non-ASCII spellings are refused (unit test and
  parser probes).
* `Display`, `Debug` and `to_hex` emit 64 lowercase hex characters from a `const fn` nibble table,
  so output is platform-independent. `Ord` and `Hash` are identity operations on bytes, not an
  ordering of outcomes.
* Documentation states that a digest identifies bytes and carries no authenticity, trust or
  authority meaning.

**Independent recomputation.** The fixed plan was rebuilt in Python from the plan section 6.1
field order with `json.dumps(..., separators=(",", ":"))`, compared with the Rust test literal
(identical), and hashed with `hashlib`:

| Vector | Bytes | SHA-256 |
|---|---|---|
| fixed plan | **380** | `3597cffad48d156d87af28ccb956c4253fc74f8ad56709e5fdaf1fb12a074032` |

This matches the candidate's claim and is not taken from the Rust test.

**`P1_DIGEST_CONTRACT_SOUND`.**

## 10. Receipt model and authority boundary

* `LaunchReceipt { bytes, sha256, record }` has private fields. Its only constructor is
  `pub(crate) fn from_record`, marked `expect(dead_code)` outside tests: **P1 has no producer.**
* No public `new`, `from_record`, `From<ReceiptRecord>`, `TryFrom<&[u8]>`, `Deserialize`, receipt
  parser or equivalent exists. Probes: struct update fails `E0451`; `serde_json::from_slice` into
  `LaunchReceipt` or `ReceiptRecord` fails with the missing `Deserialize` bound; `from_record`
  fails `E0624`; `ReceiptRecord { .. }` and `Termination { .. }` literals fail `E0451`;
  `LaunchReceipt::try_from(&[u8])` fails `E0277`.
* `ReceiptRecord` is inert read-only data and also has no public constructor, so neither type can
  be obtained outside the crate in P1 at all. No function turns a record into authority.
* A receipt carries **zero execution authority**: no type in the crate carries any.
* No output payload can enter a receipt: `StreamFacts` holds only `u64` count, `Digest` and
  `Completeness`; no field of any record type is a byte buffer or free string except the
  grammar-restricted working-directory identifier.
* No timestamp, duration, pid, descriptor number, host path, inode, device, hostname or kernel
  field exists. The committed test pins the closed key set across all variant combinations.
* The code, rustdoc and README make **no authenticity, signature or provenance claim**; they
  state that receipt bytes may be copied or fabricated and that API non-constructibility is not
  authenticity.
* Observation: the Rust field types are wider than the contract domain in places
  (`pre_exec_mode_bits: u16` against `st_mode & 0o7777`, `argument_count: u32` against at most 64).
  In P1 no producer exists, the widest-value receipt is still bounded, and this is not a finding.

**`P1_RECEIPT_AUTHORITY_BOUNDARY_SOUND`.**

## 11. Receipt vocabulary and serialisation

**Shape.** The serializer is hand-written, pushes keys in a fixed order, iterates no map and uses
no floating-point or locale-dependent formatting. Integers are formatted by `to_string`, digests
by the fixed hex routine, and every token is a `&'static str` or a grammar-checked identifier
that needs no JSON escaping. Output is ASCII with no whitespace. One `ReceiptRecord` therefore has
exactly one byte representation, and field order is fixed by construction.

**Data-carrying variants (F5).**

| Object | `kind` / discriminant | Sibling fields present |
|---|---|---|
| `exec_status` | `pre_exec_failure` | `stage`, `errno` (both always) |
| `exec_status` | `indeterminate` | `reason`; `errno` if and only if `reason = status_read_failed` |
| `child_end` | `exited` | `code` |
| `child_end` | `signaled` | `signal`, `core_dumped` |
| `child_end` | `end_unobservable`, `end_not_observed` | none |
| stream | `completeness = read_failed` | `errno` after `completeness`; absent for every other value |
| `termination.group_sweep` | plain token | none |
| `asserted_context` members | always present | digest string or `null` |

Each optional sibling is determined by exactly one discriminant value that precedes it, no two
variants share a discriminant spelling, and no sibling is shared across discriminants with
different meaning. Field presence is closed. The committed test serialises all
13 × 5 × 3 × 4 × 4 = 3 120 variant combinations at the widest numeric values and finds every
byte string distinct; it also checks the remaining boolean and optional axes individually. The
encoding is unambiguous. P1 has no receipt parser, so the per-kind presence rule is currently
documented only by the serializer and a short README summary (P1-SER-01, backlog).

**Bound.** `MAX_RECEIPT_BYTES = 8 192` is a proven bound, not a truncation point: the exhaustive
widest-value test asserts every combination stays below it. With no producer in P1 there is no
runtime path to enforce; the bound is real and tested as a property of the model.

**Digest over exact bytes.** `from_record` serialises first and hashes exactly the resulting
bytes, stores both, and never hashes a richer internal form. Every receipt the tests build is
rehashed independently with `sha2`.

**Independent vector reconstruction.** Both records were rebuilt in Python from the plan
section 11.2 field order and the record values, serialised with `json.dumps(...,
separators=(",", ":"))`, compared with the Rust test literals (identical) and hashed with
`hashlib`:

| Vector | Bytes | SHA-256 |
|---|---|---|
| fixture receipt | **1091** | `c2e58e8825b8398fd4b2492f13109caa39a52b461de01c43abe804498525e949` |
| data-variant receipt | **1229** | `19771d9cd6216735a4d9145ad4e06b6f2ba070b365dfc46004a50436f7bfc52a` |

Both match the candidate's claims and neither was taken from the Rust serializer.

**`P1_RECEIPT_SERIALISATION_SOUND`**, with P1-SER-01 (backlog).

## 12. Non-verdict vocabulary

Every emitted spelling was read semantically: schema and version constants, `backend`,
`environment_mode`, all `ChildStage`, `ExecStatus`, `IndeterminateReason`, `ChildEnd`,
`GroupSweep`, `Completeness`, `ElfType`, `ExecutionKind`, `StdinMode` and `TerminationSignal`
spellings, all 34 receipt keys, and all 25 error codes.

* No emitted type, value or key means pass, overall fail, success, ready, compatible, verified,
  sandboxed or contained.
* `pre_exec_failure`, `status_read_failed` and `read_failed` name an observed failed operation,
  not a verdict, and are correctly allowed. Error codes such as `MALFORMED_JSON` describe the
  input, not an outcome.
* No `is_success`, `ok`, `into_bool`, `From<_> for bool`, `Result<(), _>` mapping or outcome
  ordering exists. The only public `-> bool` functions are the three receipt facts of the same
  name (`run_deadline_expired`, `sigterm_sent`, `sigkill_sent`) and `LaunchPlanErrors::contains`.
  Fact enums derive neither `PartialOrd` nor `Ord`.
* The committed vocabulary guard checks whole terms and each `_`/`-`/`.` word against a list
  that extends plan section 10.4 with `passed`, `successful` and `trusted`.

**`P1_NON_VERDICT_VOCABULARY_SOUND`.**

## 13. Pure fd-layout planner

* `layout.rs` is arithmetic over `u32` descriptor numbers. It owns, opens, closes and inspects
  nothing, has no `std` I/O import, and is crate-private and consumed only by tests.
* `relocations` relocates all six child-side roles unconditionally to at least 3 (T19).
* `plan_child_layout` refuses any number below 3 and any shared number, maps stdin, stdout and
  stderr to 0, 1 and 2, preserves exactly the executable and status descriptors, and emits at most
  three ascending ranges from 3 to `u32::MAX` around them, skipping inverted gaps with checked
  arithmetic. Adjacent preserved numbers (F7), preserved numbers at 3 and 4 in either order, a
  single-number gap, and preserved `u32::MAX - 1`/`u32::MAX` are fixed vectors.
* 20 000 generated layouts check non-inversion, ordering, exclusion of the preserved numbers,
  coverage at every boundary, closure of the four non-preserved roles, and `dup2` source never
  equal to target. 5 000 simulated descriptor tables — host 0/1/2 randomly closed, random
  close-on-exec host descriptors, lowest-free allocation, caller descriptors of either
  close-on-exec state — end with exactly `{0, 1, 2}` bound to the three pipe objects, the working
  directory still open when used, and no host or caller object in the image.
* Generality for later work: the planner takes numbers and makes no claim about any operating
  system. `u32` admits numbers above `INT_MAX`, which a later backend would have to convert
  checked; that is inert here and not a finding.

**`P1_LAYOUT_MODEL_SOUND`.**

## 14. Pure lifecycle state machine

**Purity.** `lifecycle.rs` imports only model types. It reads no clock (time is a caller
`u64`), polls, waits, signals, spawns, reads or writes nothing, and makes no OS call. Inputs are
`Event` values and outputs are `Action` values and a `Facts` record.

**Scenario review against plan section 8.5 and the owner rulings.**

| Scenario | Model behaviour | Verdict |
|---|---|---|
| S5 | EOF without record, then a SIGKILL end: `indeterminate(status_eof_without_record)`, identical to a normal run; `sigkill_sent = false` | matches |
| T1 | run deadline at EOF + `timeout_ms` → `SendSigterm`, grace deadline | matches |
| T2 | grace expiry → `SendSigkill`, kill deadline + `POST_KILL_REAP_MS` | matches |
| T3 | end during grace: no `SIGKILL` | matches |
| T4, T5 | end one tick before the run deadline, stale deadline delivered anyway: no expiry, no signal | matches |
| end before EOF | no run deadline is armed | matches |
| S6 | no status by 5 000 ms → `pre_exec_status_timeout`, immediate `SIGKILL`, no `SIGTERM`, bounded kill wait | matches |
| record / malformed | wait for the end until the pre-exec deadline, then `SIGKILL` and the kill wait | matches |
| **F1** | a status read failure classifies `status_read_failed`, starts **no** run deadline even if EOF follows, keeps the pre-exec bound, and `SIGKILL`s at that bound if no end was observed | **matches the owner disposition** |
| O6 | drain bound 2 000 ms after the end; still-open streams become `writer_retained_after_child_exit` | matches |
| **T40** | kill-deadline expiry without an observed end latches `child_end = end_not_observed` immediately and marks open streams `read_stopped_child_end_not_observed` | **matches** |
| T41 | a stream read error is `read_failed { errno }`, is never overwritten by a later EOF, and stops further reads | matches |
| POLLIN+POLLHUP | readiness with hangup only requests a read; only a zero-byte read is EOF | matches |
| T30 | with authority: `ProbeReaped` → `Unreaped` → exactly one `SweepGroup` → `Reap`, on all six completion paths | matches |
| T31 | without authority: no probe and no sweep on any path, recorded `not_issued_group_not_established`; with authority and `AlreadyReaped`: no sweep, `not_issued_child_already_reaped` | matches |
| exec success | `ExecStatus` has no success variant; no transition produces one | matches |

**Exactly-one sweep and ordering.** The sweep can be emitted only from the probe handler, which
runs once because the phase advances to `AwaitingReap`; a second probe or deadline after cleanup
produces nothing. The sweep always precedes the reap request. Authority comes only from the
constructor argument; nothing infers it.

**Reviewer-owned probes.** The committed, unmodified `lifecycle.rs` and `model.rs` were compiled
into a temporary crate outside the repository and driven by scenarios and a generator written
for this review:

* F1 + T40 + T31 end to end: read failure at 100 ms, EOF at 200 ms, `SIGKILL` at 5 000 ms, latch at
  10 000 ms, late end readiness, `AlreadyReaped` probe, late `Exited` reap → `end_not_observed`,
  `not_issued_child_already_reaped`, `status_read_failed`, no `SIGTERM`, both streams
  `read_stopped_child_end_not_observed`.
* Record path without authority, latch against late `Killed`, `Dumped`, `Unclassifiable` and
  `NothingAvailable` reaps: latch stands, no probe, no sweep.
* End observed while the status is still awaited and EOF never arrives: waits only to the pre-exec
  bound, records `pre_exec_status_timeout` without `SIGKILL`, one sweep.
* 50 000 generated runs with random authority, timeout, grace, event order, probe answer and reap
  result: **0 violations** of total bound, at most one of each signal, exactly one reap, sweep
  discipline per authority and probe, latch if and only if final `end_not_observed`,
  `run_deadline_expired` if and only if `sigterm_sent`, and **no observing state without a
  pending deadline**. 35 171 runs latched, 5 878 of them with a later `AlreadyReaped` probe.

**Driver-ordering contract.** The model assumes its driver delivers `DeadlineReached` when a
deadline is due, before observations stamped later. Delivered out of order, an end readiness
stamped after the kill deadline prevents the latch, and a status EOF stamped after the pre-exec
deadline arms a run deadline. This is consistent with the plan's rule that readiness observed
before a deadline is never a timeout, and no P1 consumer exists, so it is recorded only as
P1-LIFE-01 (backlog) for the later loop.

**`P1_LIFECYCLE_MODEL_SOUND`.**

## 15. T40 owner clarification and documentation sync

Owner authority for P1: once `POST_KILL_REAP_MS` expires without an observed direct-child end,
receipt-facing `ChildEnd::EndNotObserved` is latched; no later `Exited`, `Signaled`, core-dumped
result or `ECHILD` revises it; a later independent cleanup fact such as
`not_issued_child_already_reaped` may still be recorded. `EndNotObserved` means that the end was
not observed within the bound, not that the child never ended.

Implementation (`d3914ab`): the kill-deadline branch sets `child_end = Some(EndNotObserved)` at
the bound; `after_probe(AlreadyReaped)` uses `get_or_insert(EndUnobservable)`, so a latched value
stands while `group_sweep` still records `not_issued_child_already_reaped`; `after_reap` keeps
any latched value and only fills an unset one. A late `ChildEndReadable` is ignored once latched.
Without a latch, `ECHILD` still yields `end_unobservable`, as the plan requires.

**`END_NOT_OBSERVED_LATCHING_MATCHES_OWNER_AUTHORITY`.**

Plan section 8.5 Phase C step 2 still reads "`ECHILD` → … child end `EndUnobservable`" without
excepting an already latched `EndNotObserved`. That text predates the owner clarification; it is
the only discrepancy found, and the implementation matches current owner authority.

**P1-DOC-01 — MINOR — PRODUCTIZATION PLAN NEEDS LATER SYNC TO OWNER-CLARIFIED T40 LATCHING.**
The plan is not edited in this review.

## 16. Error model

* P1 implements only `LaunchPlanErrors`, `LaunchPlanError` and `LaunchPlanErrorCode`.
  `AdmissionError`, `AuthorizationRefusal` and `LaunchError` do not exist, not even as inert
  enums; the module documentation says so.
* Codes are fixed `SCREAMING_SNAKE` strings; `Display` is `CODE at locator` or
  `CODE at locator[index]`, joined by `; `, bounded by the 64-finding cap (1 710 bytes in the
  widest probe).
* No `anyhow`, no `Box<dyn Error>`, no string error, no OS message, path, pid, descriptor number or
  document text. The internal `serde` custom error string never leaves `strict_tree`.
* `LaunchPlanErrors` implements `std::error::Error`.

**`P1_ERROR_MODEL_SOUND`.**

## 17. Type-boundary tests

The 13 `compile_fail` doctests and 2 positive doctests pass. Each `compile_fail` snippet was
also compiled as a separate example against the public crate to see why it fails:

| Doctest | Failure | Probative? |
|---|---|---|
| `use helm_launch::launch` / `authorize` / `admit_executable` / `admit_working_directory` | `E0432` unresolved import | yes: absence is the claim |
| `ValidatedLaunchPlan { argv: …, ..real }` | `E0451` private fields | yes |
| `ValidatedLaunchPlan::default()` | `E0599` no `default` | yes |
| `LaunchReceipt { bytes, ..real }` | `E0451` private fields | yes |
| `serde_json::from_slice::<LaunchReceipt>` / `<ReceiptRecord>` | `E0277` missing `Deserialize` | yes; a positive doctest deserialising `serde_json::Value` shows `serde_json` resolves |
| `let _: bool = status.into()` | `E0277` no `From<ExecStatus> for bool` | yes |
| `ChildEnd::Exited { code: 0 }.is_success()` | `E0599` no method | yes |
| `ExecStatus::ExecSucceeded` | `E0599` no variant | yes |
| `if status {}` | `E0308` mismatched types | weak: any non-`bool` type fails; the two tests above prove the property independently (P1-TEST-02, backlog) |

A positive control compiled the same prefixes (`ExecStatus` construction, `as_str`, `Clone` of
plan and receipt, `parse_launch_plan`), so no snippet fails for an incidental import or syntax
reason. No test requires a fake authority type to exist. The source-token boundary test adds
non-compile checks: forbidden OS, execution and P2+ identifiers as code, no public `-> bool`
beyond the four named accessors, no `impl … for bool`, exactly the six modules, and no `#[path]`
or `include!` in `src`.

**`P1_TYPE_BOUNDARY_TESTS_SOUND`**, with P1-TEST-02 (backlog).

## 18. README, CI and hygiene

**README.** The banner states P1 implementation only, no execution backend, no process can be
created by this crate, no executable or working-directory capability yet, and no `unsafe` code
with the root `deny` and no `allow`. The accepted backend is labelled "Accepted future behaviour —
NOT IMPLEMENTED". Non-claims cover no execution, no sandbox or containment, no Wine, no `PATH` or
shell, no authority from parsing and no receipt authenticity, and add that cross-platform
determinism is only as strong as the platforms the tests actually ran on. Two wording defects are
recorded as P1-DOC-02: the sentence "it has not been independently reviewed" becomes stale once
this review is published, and the one-line receipt shape `exec_status{kind, stage|reason, errno?}`
and `child_end{kind, code|signal, core_dumped}` is looser than the closed per-kind presence of
section 11. **`P1_README_SOUND`**, with P1-DOC-02 (MINOR).

**CI.**

* `helm-evidence.yml` adds `crates/helm-launch/**` to both the `push` and the `pull_request`
  path filters, and two steps: `cargo test -p helm-launch --locked` and
  `cargo build -p helm-launch --release --locked`.
* `helm-launch.yml` is a P1 portable-model job modelled on `helm-bind.yml`: path filters for
  `Cargo.toml`, `Cargo.lock`, `crates/helm-launch/**` and the workflow on both events;
  `workflow_dispatch` for deliberate re-runs, as in repository precedent; matrix
  `ubuntu-24.04`, `windows-2025`, `macos-15` with `fail-fast: false`; checkout pinned to the
  repository's commit `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09` with `persist-credentials: false`;
  Rust 1.95.0 through `rustup`; `cargo fmt --check`,
  `cargo clippy -p helm-launch --all-targets --all-features --locked -- -D warnings`,
  `cargo test -p helm-launch --locked`, then an identity-printing step; `permissions:
  contents: read`; `timeout-minutes: 20`; concurrency grouped by workflow and ref with
  `cancel-in-progress`, identical to precedent.
* Commands are plain executables with arguments, valid in `bash` and `pwsh`. On Windows a
  multi-line `run` block reports only its last command's status; this affects the toolchain,
  runner-record and identity-printing blocks only, as in `helm-bind.yml`, while every asserting
  step is one command (P1-CI-01, backlog).
* No experiment runner, helper, Wine, backend, privilege or artifact publication.

**`P1_CI_DESIGN_SOUND`**, with P1-CI-01 (backlog). The workflows were not dispatched.

**Hygiene.** Across all 15 committed files: zero NUL bytes, zero carriage returns, no generated
build output, target artifacts or logs, no private host path, credential or key, no
`Co-Authored-By` or AI attribution in files or commit messages, and no unrelated change. Non-ASCII
content is limited to typographic dashes and ellipses and one `é` test literal.
`git diff --check 076c2b0..d3914ab` is clean. **`P1_CANDIDATE_HYGIENE_SOUND`.**

## 19. Validation

Fresh run on the candidate tip with an empty `CARGO_TARGET_DIR` outside the repository, so no
earlier build state could mask or invent a result.

| Command | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | pass |
| `cargo test --workspace --locked` | pass; helm-launch: 38 unit, 10 `p1_boundary`, 19 `plan_contract`, 15 doctests |
| `cargo test -p helm-launch --locked` | pass, same counts |
| `cargo build -p helm-launch --release --locked` | pass |
| `python -m unittest discover -s tools/tests` | pass, 784 tests, 74 skipped |
| `python tools/validate_docs.py` | pass |
| `git diff --check` | pass |
| reviewer parser, compile-boundary and lifecycle probes | as reported in sections 8, 14 and 17; all temporary and outside the repository |

| Platform | Status |
|---|---|
| Windows | **run locally**: Windows 11 Pro 10.0.26200, `rustc 1.95.0 (59807616e 2026-04-14)` `x86_64-pc-windows-msvc`, `cargo 1.95.0`, Python 3.14.3 |
| Linux | **not run locally**: the default WSL Ubuntu distribution has no Rust toolchain, and none was installed for this review |
| macOS | **not run locally**: no macOS host was available |

**P1 CROSS-PLATFORM MATRIX: PENDING PUBLICATION CI.**

## 20. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| P1-DOC-01 | MINOR | plan / T40 | documentation only | Productization plan section 8.5 Phase C step 2 says `ECHILD` yields `EndUnobservable` without excepting an already latched `EndNotObserved`; the text predates the owner's T40 clarification, which the implementation matches | later documentation sync of the plan under owner instruction; not edited here |
| P1-DOC-02 | MINOR | README | documentation only | "it has not been independently reviewed" becomes stale on publication of this review; the one-line receipt shape understates per-kind field presence (`errno` is mandatory with `pre_exec_failure`, `core_dumped` exists only with `signaled`) | wording correction at the next owner-authorised touch of the crate |
| P1-TEST-01 | MINOR | parser tests | test gap; behaviour correct | the duplicate-key test's "escaped spellings" vector is a byte-identical literal duplicate, so no committed test proves decoded-key collision for escaped keys, which plan section 14.1 and the README name; reviewer probes show six escaped variants all refused `DUPLICATE_KEY` | add one escaped-key vector in a bounded test-only correction; owner decides whether before or after publication |
| P1-PARSE-01 | BACKLOG_NONBLOCKING | parser errors | yes, refused either way | an integer outside the `u64`/`i64` range, and `-0`, classify `TYPE_MISMATCH`, while an in-range integer outside the field bound classifies `*_OUT_OF_RANGE`; every case is refused and deterministic | none required; revisit if error codes are stabilised |
| P1-PARSE-02 | BACKLOG_NONBLOCKING | plan schema | yes | `"asserted_context": {}` is accepted as a second spelling of "no assertion" beside omission; this conforms to plan section 6.1 ("each member optional") and ruling F3 (which refuses `null`) | owner may decide whether omission should be the only spelling |
| P1-LIFE-01 | BACKLOG_NONBLOCKING | lifecycle model | model API only; no P1 consumer | the model relies on its driver to deliver `DeadlineReached` before observations stamped after a due deadline; out of order, a late end readiness prevents the T40 latch and a late status EOF arms a run deadline | state and test the driver-ordering contract when a real loop is authorised |
| P1-SER-01 | BACKLOG_NONBLOCKING | receipt schema (F5) | documentation only | per-kind field presence is closed and unambiguous in the serializer but not yet written as a receipt schema | publish with the receipt schema document planned for P5 / B-02 |
| P1-CI-01 | BACKLOG_NONBLOCKING | CI | Windows runner | multi-line `pwsh` `run` blocks report only the last command's exit status; only the toolchain, runner-record and identity-printing blocks are multi-line, as in `helm-bind.yml` | optional hardening across both workflows |
| P1-TEST-02 | BACKLOG_NONBLOCKING | type-boundary tests | test strength | the `if status {}` doctest fails for any non-`bool` type; the `into()` and `is_success` doctests prove the property specifically | none required |

**BLOCKER: none. IMPORTANT: none.**

## 21. Recommendation and next gate

Verdicts: `P1_COMMIT_SCOPE_SOUND`, `P1_DEPENDENCY_GRAPH_SOUND`, `P1_LINT_POLICY_SOUND`,
`P1_PUBLIC_API_MINIMAL_AND_WITHIN_AUTHORITY`, `P1_PLAN_PARSER_SOUND`, `P1_DIGEST_CONTRACT_SOUND`,
`P1_RECEIPT_AUTHORITY_BOUNDARY_SOUND`, `P1_RECEIPT_SERIALISATION_SOUND`,
`P1_NON_VERDICT_VOCABULARY_SOUND`, `P1_LAYOUT_MODEL_SOUND`, `P1_LIFECYCLE_MODEL_SOUND`,
`P1_ERROR_MODEL_SOUND`, `P1_TYPE_BOUNDARY_TESTS_SOUND`, `P1_README_SOUND`, `P1_CI_DESIGN_SOUND`,
`P1_CANDIDATE_HYGIENE_SOUND`; `END_NOT_OBSERVED_LATCHING_MATCHES_OWNER_AUTHORITY`.

> **P1 CANDIDATE MAY BE PUBLISHED FOR THE AUTHORISED THREE-PLATFORM CI GATE.**
> **P2 REMAINS NOT AUTHORISED.**

**Next gate:** one fast-forward publication of the P1 candidate and this independent review,
followed by the authorised three-platform CI gate. The three MINOR findings do not block that
gate; the owner decides whether P1-TEST-01 and P1-DOC-02 are corrected before or after it.

This review changes no implementation file, authorises nothing beyond P1, adds no process
execution and no `unsafe` code, dispatched no workflow, pushed nothing, and leaves `main`
unchanged. No Trial #4 is authorised.

**Classification: `HELM_LAUNCH_P1_INDEPENDENT_REVIEW_PASSED_READY_FOR_PUBLICATION_CI`.**
