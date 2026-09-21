# FRESH BOUNDED INDEPENDENT REREVIEW OF P5R-01

> **Scope: the `P5R-01` correction delta only.** This is **not** a second whole-crate review, not a
> new P5 implementation review and not a publication task. The whole-crate independent review at
> `6e6057c13762759ad3e90ce96dcc8709263d337b` remains authoritative for every area outside the delta.

---

## 1. Session provenance and independence

| Item | Value |
|---|---|
| Review date | 2026-09-20 |
| Host | Windows 11 Pro 10.0.26200, `x86_64-pc-windows-msvc` |
| Toolchain | `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` — the CI-pinned version |
| Python | 3.14.3 |
| Repository | `D:\HELM\helm-os` |
| Branch | `docs/helm-launch-architecture` |

**This session did not author or modify `57459628c4da5fba43c4be83aa6a579e30a08430`.** It opened on
the correction as an already-committed artifact and read it for the first time here. Independence is
asserted from the session's own history, not inferred from Git metadata.

### Starting state — all preconditions met

| Requirement | Observed | |
|---|---|---|
| Clean worktree | clean | PASS |
| `git branch --show-current` | `docs/helm-launch-architecture` | PASS |
| `git rev-parse HEAD` | `57459628c4da5fba43c4be83aa6a579e30a08430` | PASS |
| `origin/docs/helm-launch-architecture` | `06223035828bfc6fec249bad7a5f2b5d520a7d59` | PASS |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` | PASS |
| Ahead / behind | 4 ahead, 0 behind | PASS |
| Merge commits in `0622303..HEAD` | 0 | PASS |

Ancestry verified commit by commit through `git rev-list --parents`; every commit has exactly one
parent and the chain is exactly:

```
0622303 -> 9466c9f -> 67a4502 -> 6e6057c -> 5745962
```

`origin` was fetched read-only. Nothing was pushed.

All mutation experiments in this review were run in **throwaway `git worktree` checkouts** under the
session scratchpad, never in `D:\HELM\helm-os`. Both were removed; `git worktree list` shows only the
primary worktree and `git status` is clean.

---

## 2. The original finding, re-established independently

`P5R-01` as recorded in the whole-crate review (section 54) has two limbs. Both were re-established
here from the pre-correction tree, without relying on the review's own text.

**Limb A — committed receipt vectors: machine-checked, but not naturally triggerable.**

At `6e6057c`, `docs/implementation/helm-launch-receipt-0.1-test-vectors.json` is consumed by
`include_str!` at `crates/helm-launch/src/receipt.rs:949` and
`crates/helm-launch/tests/p5_regressions.rs:331`, and held to exact-byte and digest agreement with the
production serializer. I enumerated the `pull_request`/`push` path filters of **every** workflow in the
repository at that commit:

| Workflow | Covers the vectors? |
|---|---|
| `helm-bind.yml` | no |
| `helm-evidence.yml` | no — `crates/**` and `tools/tests/**` only, no `docs/` |
| `helm-launch.yml` | no |
| `helm-observe-independent-review.yml` | no — branch-triggered on `review/helm-observe-**` |
| `launch-exec-01-compile-only.yml` | no — `docs/experiments/launch-exec-01/**` only |
| `launch-exec-01-trial-002.yml`, `-003.yml` | no — `workflow_dispatch` only |

No workflow runs unconditionally on push. A commit changing only the vectors triggered **nothing**.
**Limb A confirmed.**

**Limb B — the published schema document: neither triggerable nor machine-checked.**

Same path-filter result as above: uncovered. Additionally, at `6e6057c` there is **no `include_str!`
of `HELM-LAUNCH-RECEIPT-0.1.md` anywhere in the crate**, and `tools/validate_docs.py` contains **zero**
references to the receipt schema (`grep -n "RECEIPT\|receipt" tools/validate_docs.py` returns nothing) —
and `validate_docs.py` itself runs only inside `helm-evidence.yml` and
`helm-observe-independent-review.yml`, whose filters exclude `docs/`. So the document had no machine
validator of any kind, anywhere, and no trigger that would have run one. **Limb B confirmed.**

**`P5R01_ROOT_CAUSE_CONFIRMED`.**

---

## 3. Correction scope

`git diff --name-status 6e6057c 5745962` returns exactly four modified files, no additions, no
deletions, no renames:

| File | Hunk classification |
|---|---|
| `.github/workflows/helm-launch.yml` | workflow path-filter only (+16 lines: 2 path entries + 6 comment lines, twice) |
| `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` | schema normative block + documentation (+86/-1: new section 7, `## 7. Stability` renumbered to `## 8.`) |
| `crates/helm-launch/src/receipt.rs` | test-only (+633/-1: one `use` line inside `mod tests`, and 633 lines appended inside the same `#[cfg(test)]` module) |
| `crates/helm-launch/tests/p2_boundary.rs` | test-only boundary assertion (the owner-ratified fourth file) |

**No product runtime hunk exists in the delta.** `Cargo.toml`, `Cargo.lock` and every other source
file under `crates/helm-launch/src/` are untouched.

**`P5R01_SCOPE_SOUND`.**

---

## 4. Product identity — load-bearing

Mechanically verified, not read by eye. The product region of `receipt.rs` is everything before the
first `#[cfg(test)]` marker — the same definition `p2_boundary::product_code()` uses, and the crate
separately asserts that test-only items come last.

```
6e6057c product region sha256: 70d1f395a9834ac410a23aa6680754f72f092d63518fde1cafc23d2a1c8ac27a
5745962 product region sha256: 70d1f395a9834ac410a23aa6680754f72f092d63518fde1cafc23d2a1c8ac27a
diff: empty
```

**Byte-identical.** Therefore unchanged, by construction: `LaunchReceipt`, `ReceiptRecord`,
`serialize()`, `sha256()`, `MAX_RECEIPT_BYTES`, the public API, every receipt field semantic and every
enum semantic. `Cargo.toml`, `Cargo.lock`, `backend`, `launch`, `lifecycle` and every `unsafe` site
are outside the delta entirely.

**`P5R01_PRODUCT_SEMANTICS_UNCHANGED`.**

---

## 5. The schema normative block — structure

The block lives between two HTML-comment markers. Marker counts in the committed document: **exactly
one `:BEGIN`, exactly one `:END`** (`grep -c`, confirmed).

`contract()` in `receipt.rs` enforces the grammar and fails on:

* a missing or repeated `BEGIN` marker (`matches(...).count() == 1`);
* a missing or repeated `END` marker (same);
* a line that is not `key=value` (`split_once('=')` with a panic message);
* an empty key or empty value;
* a key outside the closed `CONTRACT_KEYS` set;
* a duplicate key (`map.insert(...).is_none()`);
* a missing required key (post-pass over `CONTRACT_KEYS`).

Whitespace around `=` is refused by consequence: a padded key no longer equals a member of
`CONTRACT_KEYS`, and a padded value no longer equals the production truth it is compared against.
`trim_end_matches('\r')` makes the parse CRLF-safe, which matters on a Windows checkout.

Text outside the block cannot become a contract key, because only the span between the two markers is
parsed and the key set is closed.

**The 34-key set was reviewed key by key, not counted.** Every key has a specific contract purpose and
a named source of truth — 10 byte/encoding and identity facts, 9 closed vocabularies, 15 field-order
and nullability facts. None is decorative and none is a claim about meaning.

*Observation, not a finding:* the grammar prose says "no blank lines", while the parser **skips** empty
lines rather than rejecting them. A blank line carries no key, so this opens no hole; the parser is
merely more lenient than the prose on one point.

**`P5R01_SCHEMA_BLOCK_STRUCTURE_SOUND`.**

---

## 6. No shadow serializer

I read the complete new checker. It is a **reader** throughout.

`members()` walks compact receipt bytes and returns `(key, value-offset)` pairs plus the offset past
the object; `keys_at()` and `raw_value()` are built on it. Every byte it inspects comes from
`LaunchReceipt::from_record(record).exact_bytes()` — the production serializer — or from
`exact_bytes_base16` decoded out of the committed vector artifact.

There is **no code path in the new section that emits a receipt byte**. No string is assembled into
receipt shape, no field is formatted, no alternate encoder exists. `contract_witnesses()` constructs
`ReceiptRecord` values only, and hands them to production code.

The reader is deliberately narrow and fails loudly rather than guessing: an escape sequence, an array
or an unexpected byte after a member each panic instead of being tolerated. Compact-ASCII-with-no-array
is not assumed on trust — the same test asserts it from the same bytes.

**`P5R01_NO_SHADOW_SERIALIZER`.**

---

## 7. Product-truth linkage

For every normative category I identified the actual source of truth and confirmed the comparison
reaches it. Nothing load-bearing is compared against a handwritten mirror.

| Contract category | Source of truth | Direct? |
|---|---|---|
| `max_receipt_bytes` | `MAX_RECEIPT_BYTES` production constant | yes |
| `schema`, `version` | `raw_value(bytes, …)` — the values the production serializer emits | yes |
| `backend_values` | `Backend::LinuxX8664Clone3PidfdExecveat.as_str()` | yes |
| `environment_mode_values` | `EnvironmentMode::Empty.as_str()` | yes |
| `elf_type_values` | `all_elf_types()` → `ElfType::as_str()` | yes |
| `child_stage_values` | `all_stages()` → `ChildStage::as_str()` | yes |
| `indeterminate_reason_values` | `all_exec_statuses()` → `IndeterminateReason::as_str()` | yes |
| `exec_status_kinds` | `ExecStatus::as_str()` of both constructed variants | yes |
| `child_end_kinds` | `all_child_ends()` → `ChildEnd::as_str()`, first-occurrence dedup | yes |
| `group_sweep_values` | `all_sweeps()` → `GroupSweep::as_str()` | yes |
| `stream_completeness_values` | `all_completeness()` → `Completeness::as_str()` | yes |
| every `*_order*` | key order read back out of production bytes, and out of committed vector bytes | yes |
| `asserted_context_nullable` | production bytes of an absent-digest and a present-digest record | yes |
| byte/encoding keys | the production bytes themselves | yes |

The encoding-style keys (`encoding=utf8`, `formatting_whitespace=none`, `trailing_newline=absent`,
`digest_encoding=lowercase_hex_64`, `argument_count_encoding=decimal_u32`) are each a **label pinned to
a spelling** *plus* a product-derived assertion of the property it labels — for example
`bytes.iter().all(u8::is_ascii)` alongside `assert_eq!(contract["encoding"], "utf8")`. Changing either
side alone fails the test. That is a sound pairing, not a handwritten mirror.

**`P5R01_SCHEMA_PRODUCT_TRUTH_LINKED`.**

---

## 8. Enum closed-world check

The author's claim — wildcard-free `all_*()` lists, so a new model variant fails compilation until
represented — was **verified by experiment, not accepted on report.**

Every `all_*()` helper ends in a `match` over its own elements with no `_ =>` arm. Note that
`#[non_exhaustive]` on these enums constrains *downstream* crates only; in-crate matches must still be
exhaustive, so the guards bite.

* **Control P6** — added a variant to `ChildStage` in the model. Result: **compilation fails**. The
  closed-world guarantee holds for `child_stage_values`, and by the same construction for
  `elf_type_values`, `indeterminate_reason_values`, `exec_status_kinds`, `child_end_kinds`,
  `group_sweep_values` and `stream_completeness_values` — seven of the nine vocabularies.

* **Controls P7 and P8** — added a variant to `Backend`, and separately to `EnvironmentMode`, in each
  case also adding the `as_str()` arm the compiler demands. Result: **the contract test still passes.**

`backend_values` and `environment_mode_values` are handwritten single-element `vec![…]` expressions with
no exhaustiveness guard. Both enums have exactly one variant today, so the published lists are currently
**complete and correct**, and both are still compared against production `as_str()` output. The gap is
purely forward-looking: a future variant would leave those two published vocabularies silently
under-declared. Recorded as **`P5R01R-M1` (MINOR)**.

Ordering is enforced wherever it is contractually relevant: `listed()` splits on `,` and the comparison
is against an ordered `Vec`, so `elf_type_values`, `child_stage_values` (child-sequence order),
`indeterminate_reason_values`, `child_end_kinds`, `group_sweep_values` and
`stream_completeness_values` are all order-sensitive. Control C12 (a value inserted mid-list) and
control C2 (a value omitted) both fail the test.

**`P5R01_ENUM_VOCABULARY_SOUND`** — with `P5R01R-M1` carried as MINOR.

---

## 9. Field-order proof

Order is established from **actual production serialized bytes**, not from a list-versus-list
comparison. For each witness the test calls `LaunchReceipt::from_record(record).exact_bytes()` and
reads the key order back out with `keys_at()`.

No order-discarding JSON parser is used as proof of order anywhere. `serde_json` appears in the new
code only to enumerate the committed vector array and pull out the `exact_bytes_base16` string; the
order proof is then taken from the **decoded receipt bytes**, not from the parsed JSON.

Shapes covered, all read from production bytes:

| Shape | Contract key |
|---|---|
| top-level receipt | `top_level_order` (15 fields) |
| `asserted_context` | `asserted_context_order` |
| `executable` | `executable_order` |
| `termination` | `termination_order` |
| `stdout` / `stderr` | `stream_order`, `stream_order_read_failed` |
| `exec_status`, 3 variant shapes | `exec_status_order_pre_exec_failure`, `…_indeterminate`, `…_indeterminate_status_read_failed` |
| `child_end`, 4 variant shapes | `…_exited`, `…_signaled`, `…_end_unobservable`, `…_end_not_observed` |

I also checked all fifteen declared orders by eye against `serialize()`, `push_exec_status()`,
`push_child_end()` and `push_stream()` in the product region. All fifteen agree.

**Control P1** — reordered two fields inside `executable` in the **production serializer**. Result:
the contract test fails. The link runs in the direction that matters.

**`P5R01_FIELD_ORDER_PROOF_SOUND`.**

---

## 10. Production + vector three-way link

The link is genuine rather than a pair of circular checks, because the three legs are independently
anchored:

1. **Schema declaration** — parsed from the committed `.md`.
2. **Production serializer output** — `exact_bytes()` of records the test constructs in-crate.
3. **Committed vector bytes** — decoded from `exact_bytes_base16` in the committed `.json`, i.e. read
   out of the published artifact, not re-serialised.

Step 5 of the new test checks the committed bytes against the declared top-level, `asserted_context`,
`executable` and `termination` orders — so leg 1 is checked against leg 3 without going through leg 2.
Legs 1↔2 are checked in step 3. Leg 2↔3 is established by the pre-existing, unchanged
`published_receipt_vectors_match_the_production_serializer`, which requires exact byte equality between
the production serializer's output and the committed hex.

The vector test's guarantees are intact and were re-run here:

* vectors loaded from the committed artifact (`include_str!`);
* production serializer used to rebuild every record;
* `exact_bytes_base16` decodes exactly;
* `exact_bytes_utf8` encodes to the same bytes;
* exact byte equality; exact published length;
* SHA-256 **recomputed from the committed bytes**;
* product digest equals the committed and recomputed digest;
* every published digest distinct; names and order match the in-crate set, so no vector may be withheld.

**`P5R01_THREE_WAY_CONTRACT_LINK_SOUND`**, **`P5R01_VECTOR_VALIDATOR_PRESERVED`**.

---

## 11. Variant shapes and nullability

`contract_witnesses()` yields all 11 published vector records, plus `vector_base()`, plus
`widest_base()`, plus `widest_base()` varied across `all_exec_statuses(i32::MIN)` (9 pre-exec stages +
4 indeterminate reasons), `all_child_ends(i32::MIN)` (5, including both `core_dumped` states),
`all_completeness(i32::MIN)` (4), `all_sweeps()` (3) and `all_elf_types()` (2).

| Required shape | Covered by |
|---|---|
| asserted-context null digest fields | step 6: an all-`None` record and an all-`Some` record, both serialised; null literal asserted from production bytes; the two byte strings asserted different |
| pre-exec failure | `all_exec_statuses` — every one of the 9 stages |
| clean-status EOF indeterminate | `status_eof_without_record` |
| other indeterminate reasons | `status_record_malformed`, `pre_exec_status_timeout`, `status_read_failed` (its own order key, with `errno`) |
| ordinary exited child | `ChildEnd::Exited` |
| signalled child, `core_dumped` both ways | `ChildEnd::Signaled` with `false` and `true` |
| `end_unobservable`, `end_not_observed` | both, each with its own order key |
| group sweep issued / not-issued dispositions | all 3 `GroupSweep` values |
| stream completeness variants | all 4, including `read_failed` with its extra `errno` field |

All witnesses are inert `ReceiptRecord` values; no host execution is involved, as the owner allowed.

**`P5R01_VARIANT_SHAPES_SOUND`.**

---

## 12. Byte contract

Every byte-level check inspects `receipt.exact_bytes()` — production output — not an independently
formatted string.

| Declared | Checked against production bytes |
|---|---|
| compact JSON, no formatting whitespace | `!bytes.iter().any(u8::is_ascii_whitespace)` |
| UTF-8 / ASCII-compatible | `bytes.iter().all(u8::is_ascii)` |
| no trailing newline | `assert_ne!(bytes.last(), Some(&b'\n'))` |
| literal `null` | read from the `asserted_context` members of an absent-digest record |
| literal `true` / `false` | each `termination_bool_fields` value read out of the bytes and required to be a declared boolean literal |
| decimal integer representation | `argument_count` raw text parsed as `u32` |
| lowercase hex digests | `plan_sha256` raw text: exactly 64 chars, every byte in `[0-9a-f]` |
| fixed object order | section 9 above |

**`P5R01_BYTE_CONTRACT_SOUND`.**

---

## 13. Negative controls

**The ten negative controls the author reported are not committed artifacts** — no negative-control
test exists in the delta (the only two `#[test]` functions added are the contract test and the
machine-block/prose separation test), and the correction's commit message is a single subject line.
They were therefore transient session evidence, which I could not review as written.

I did not accept them on report. I **built and ran my own control battery** instead — 22 controls, all
in throwaway worktrees, the repository artifacts never mutated. Every control was applied to a
pristine copy and reverted before the next.

**Document-side controls, against the committed schema document (14):**

| # | Control | Result |
|---|---|---|
| C1 | top-level order corruption (`schema`/`version` swapped) | CAUGHT |
| C2 | enum omission (`et_dyn` dropped from `elf_type_values`) | CAUGHT |
| C3 | wrong bound (`max_receipt_bytes=8193`) | CAUGHT |
| C4 | variant order corruption (`child_end_order_signaled`) | CAUGHT |
| C5 | unknown key (`containment=none`) | CAUGHT |
| C6 | duplicate key (`version=0.1` repeated) | CAUGHT |
| C7 | missing key (`stream_order` removed) | CAUGHT |
| C8 | byte-contract lie (`trailing_newline=present`) | CAUGHT |
| C9 | smuggled semantic key (`success=true`) | CAUGHT |
| C10 | missing `BEGIN` marker | CAUGHT |
| C11 | duplicate `BEGIN` marker | CAUGHT |
| C12 | vocabulary addition (`fabricated` inserted into `group_sweep_values`) | CAUGHT |
| C13 | smuggled key vs. the closed-set test (`authenticity=proven`) | CAUGHT |
| C14 | schema identity lie (`schema=helm-launch-receipt-v2`) | CAUGHT |

**Product-side controls — the direction that proves the link is not circular (8):**

| # | Control | Result |
|---|---|---|
| P1 | production serializer nested field order swapped | CAUGHT (test failed) |
| P2 | `MAX_RECEIPT_BYTES` changed to 8193 | CAUGHT (test failed) |
| P3 | production `ElfType::as_str()` spelling changed | CAUGHT (test failed) |
| P4 | third `include_str!` in the **test** region | see section 15 — `P5R01R-M2` |
| P5 | `include_str!` in the **product** region | CAUGHT (test failed) |
| P6 | new `ChildStage` variant | CAUGHT (compile error) |
| P7 | new `Backend` variant | **not caught** — `P5R01R-M1` |
| P8 | new `EnvironmentMode` variant | **not caught** — `P5R01R-M1` |

Every control in the categories the owner named at minimum — top-level order, enum omission, wrong
bound, variant order, unknown key, duplicate key, missing key, byte-contract lie, extra/smuggled
contract key, missing marker — is probative.

**`P5R01_NEGATIVE_CONTROLS_PROBATIVE`** (established by this session's own battery, not by the
author's report).

---

## 14. Closed key set versus nonclaims

The closed-key-set design is sound and is the right resolution of the earlier vocabulary collision.

`contract()` rejects any key outside `CONTRACT_KEYS`, so `containment=`, `sandbox=`, `authenticity=`,
`success=` or any other undocumented semantic key **fails rather than being ignored** — demonstrated by
controls C5, C9 and C13. Growth is impossible in both directions: an extra key fails as unknown, a
removed key fails as missing, and `the_schema_document_separates_its_machine_block_from_its_prose`
independently asserts the parsed key set equals the declared set exactly, by size and by membership.

The nonclaims of sections 1, 4 and 5 remain ordinary Markdown prose, outside the machine block, and the
document says plainly why: they are claims about meaning that no test could check. That is the honest
arrangement — it does not dress prose up as machine-verified.

**`P5R01_CONTRACT_MEANING_SCOPE_SOUND`.**

---

## 15. `include_str!` test boundary and the `p2_boundary` fourth file

### 15.1 Placement

Both evidence includes are inside the test-only region. `receipt.rs` has its single `#[cfg(test)]` at
line 235, `mod tests` opens at 237 and closes at 1914; `VECTORS_JSON` is at line 949 and the new
`SCHEMA_DOC` at line 1304. The crate separately asserts that test-only items come last in every
source. There is **no `include_str!` in the product region** — asserted by `p2_boundary`, and control
P5 confirms that assertion bites.

The permitted targets are exactly two exact filenames — `HELM-LAUNCH-RECEIPT-0.1.md` and
`helm-launch-receipt-0.1-test-vectors.json`. There is no directory allowance, no glob, no arbitrary
`.md` or `.json`.

**`P5R01_TEST_ONLY_INCLUDE_BOUNDARY_SOUND`.**

### 15.2 The owner-ratified fourth-file change, before and after

The diff is narrow. The **only** conceptual relaxation is precisely the ratified one:

```rust
-   argument.contains("helm-launch-receipt-0.1-test-vectors.json")
+   P5_EVIDENCE_ARTIFACTS.iter().any(|artifact| argument.contains(artifact))
```

with `P5_EVIDENCE_ARTIFACTS` a `[&str; 2]` of the two exact P5 artifact filenames. One specifically
permitted target became exactly two specifically permitted targets. The rest of the hunk is comment
text explaining why.

Unchanged in strength, and verified as such:

* the product-region `include_str!` prohibition — intact, control P5 CAUGHT;
* `#[path`, `include!(`, `include_bytes!(`, `cfg_if!` — the forbidden list is byte-identical and still
  applied to the **whole** source, product and test region alike;
* no `docs/**`, `implementation/**`, `*.json` or `*.md` acceptance anywhere.

### 15.3 `P5R01R-M2` — the allowlist is a proximity window, and this pre-dates the correction

The scan reads a **240-character window** after each `include_str!(` occurrence and requires one of the
permitted filenames to appear somewhere in it. It is a proximity check, not a path parse. An
unauthorised `include_str!` placed immediately before a permitted one can therefore borrow the
permitted name from the window and pass.

I tested this on **both** trees:

| Tree | Placement | Result |
|---|---|---|
| `6e6057c` (before, one permitted target) | short sneak `include_str!("../lib.rs")` immediately before the vectors include | **not caught** |
| `6e6057c` (before) | long sneak (`docs/DECISIONS.md`) immediately before the vectors include | caught |
| `5745962` (after, two permitted targets) | short sneak before the vectors include | **not caught** |
| `5745962` (after) | short sneak before the schema include | **not caught** |
| `5745962` (after) | long sneak before the schema include | **not caught** |
| `5745962` (after) | any sneak isolated from both permitted includes (4 placements tried) | caught |

**The weakness is pre-existing** — it is a property of the 240-character window in the already
whole-crate-reviewed scan, reproducible at `6e6057c`, and the correction did not create it. What the
correction does is widen the aperture modestly: there are now two shadow zones instead of one, and
because `HELM-LAUNCH-RECEIPT-0.1.md` is a shorter name than the vectors filename, more of the window
remains available after it, so a longer sneak also slips through there.

I classify this **MINOR**, not IMPORTANT, on three grounds: the allowlist is still two exact filenames
with no broad acceptance, so this is not a broad weakening; the identical evasion exists before the
correction, so it is not a weakening *introduced* here; and it is confined to the test region, where an
extra include confers no product authority. The obvious hardening is to match the include argument
against the full expected path suffix rather than scanning a fixed window.

**`P5R01_P2_BOUNDARY_REFINEMENT_SOUND`** — with `P5R01R-M2` carried as MINOR.

### 15.4 P2 contract preservation

The original P2 reason for confining source inclusion is intact. Test data may be embedded into a test
binary; product authority and runtime data may not be imported through source inclusion. The schema
document and the vector artifact confer zero authority, are compiled into no product binary — the
product-region prohibition is absolute and enforced — and remain evidence only.

**`P5R01_P2_BOUNDARY_PRESERVED`.**

---

## 16. CI path filters

Both triggers received the identical two entries:

```yaml
- 'docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md'
- 'docs/implementation/helm-launch-receipt-0.1-test-vectors.json'
```

Both spellings match the on-disk filenames exactly, including case — which matters, because GitHub path
filters are case-sensitive and the two artifacts differ in case. All nine pre-existing entries are
preserved, in order. There is no `docs/**` and no `paths-ignore`. `pull_request` and `push` carry the
same twelve patterns in the same order, with no `branches` restriction on either, so they remain
semantically equivalent.

I did **not** use the author's scratch matcher. I implemented GitHub's documented `paths` glob
semantics independently (`*` = any run of non-`/`, `**` = any run including `/`, a wildcard-free pattern
matches that exact path, case-sensitive) and ran the required cases plus four of my own:

| Case | PR | Push | Matched solely by a P5R-01 entry? |
|---|---|---|---|
| schema-only | MATCH | MATCH | yes |
| vectors-only | MATCH | MATCH | yes |
| `crates/helm-launch/src/receipt.rs` | MATCH | MATCH | no — `crates/helm-launch/**` |
| `crates/helm-launch/tests/p2_boundary.rs` | MATCH | MATCH | no — `crates/helm-launch/**` |
| `tools/helm_launch_release_reachability.py` | MATCH | MATCH | no |
| `tools/tests/test_helm_launch_release_reachability.py` | MATCH | MATCH | no |
| `docs/DECISIONS.md` | none | none | — |
| `docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md` | none | none | — |
| `docs/implementation/HELM-LAUNCH-0.1-WHOLE-CRATE-INDEPENDENT-REVIEW.md` | none | none | — |
| lower-case variant of the schema filename | none | none | — |
| sibling `docs/implementation/…-P4-CORRECTION-REREVIEW.md` | none | none | — |

Every case matched the owner's expected outcome. The two new entries are surgical: they pull in exactly
the two evidence artifacts and nothing else in `docs/`.

**`P5R01_PATH_FILTERS_SOUND`**, **`P5R01_TRIGGER_CASES_SOUND`**.

---

## 17. Actual validator reachability

A path match alone is not enough, so I traced through to the assertion.

**Schema-only push** → matches `push.paths` → the `purity` job → matrix
`os: [ubuntu-24.04, windows-2025, macos-15]` with `fail-fast: false` → step
`cargo test -p helm-launch --locked --no-fail-fast`, which carries **no `if:` condition** and therefore
runs on all three → `--lib` unit tests → `receipt::tests::the_published_schema_contract_matches_the_production_serializer`
→ `SCHEMA_DOC` (`include_str!` of the committed document, resolved at compile time) → `contract()` and
the full normative validator.

**Vector-only push** → same filter, same job, same unconditional step →
`receipt::tests::published_receipt_vectors_match_the_production_serializer` (lib) **and**
`p5_regressions::level4_every_published_receipt_digest_is_recomputable_from_the_artifact`. Neither
`p5_regressions.rs` nor the new tests carry any `cfg` gate — the file contains no `cfg(` at all — so
both run on all three operating systems.

No Linux-only `cfg` sits on schema or vector validation anywhere. Reachable on **Linux, Windows and
macOS**.

**`P5R01_VALIDATOR_REACHABILITY_SOUND`.**

---

## 18. Workflow semantics unchanged

With comments and blank lines stripped, `diff` between the two versions of `helm-launch.yml` returns
**only** the four added path lines (two in `pull_request`, two in `push`). Nothing else differs.

Unchanged: the matrix, every step, every `run` command, every `if:` condition, `timeout-minutes: 30`,
`permissions: contents: read`, the `concurrency` group and `cancel-in-progress`, both `--no-fail-fast`
uses, the machine-code proof, the release reachability proof, `strace`, injection, the Level-4 gate and
repository confinement.

**`P5R01_WORKFLOW_SEMANTICS_UNCHANGED`.**

---

## 19. Portability

The schema test ran on this Windows host and passed. Reviewing the code for host dependence: it uses
`include_str!` (a compile-time evidence input), `BTreeMap`/`BTreeSet`, byte and `&str` comparisons, and
`serde_json` only to enumerate the committed vector array. There is no Linux API, no descriptor, no
process, no `strace`, no backend, no filesystem mutation and no network. Digest and byte comparisons
are ASCII-exact and locale-independent. The include path is spelled with `/` inside `concat!` with
`CARGO_MANIFEST_DIR`, which resolves correctly on Windows, and `trim_end_matches('\r')` absorbs a CRLF
checkout. This is a portable Level-1 design.

**`P5R01_SCHEMA_VALIDATOR_PORTABLE`.**

---

## 20. Dependencies

`git diff 6e6057c 5745962 -- Cargo.toml Cargo.lock crates/helm-launch/Cargo.toml` is empty. No parser
dependency was added; `serde_json` was already a dev-dependency used by the accepted vector test.

**`P5R01_DEPENDENCY_GRAPH_UNCHANGED`.**

---

## 21. Nonclaims and prose consistency

The correction introduces no exec-success claim, no application-success verdict, no compatibility
verdict, no authenticity claim, no provenance claim, no containment claim, no sandbox claim, and confers
no authority from the receipt, the schema or the vectors. The new document section restates the
separation explicitly and keeps every nonclaim out of the machine block. The pre-existing
`no_emitted_spelling_or_schema_key_is_a_verdict` test and the vector test's verdict-term scan
(`success`, `succeeded`, `verified`, `authentic`, `trusted`) both still pass.

I read the complete schema document and found **no contradiction** between the new block and the
surrounding prose. Section 2's envelope table, section 3's fifteen-row ordered field table, and every
vocabulary table in sections 4.1 through 4.9 agree with the block line for line — including the
child-sequence ordering of `stage`, the `errno`-only-on-`status_read_failed` shape, the retention of
`signal` regardless of `core_dumped`, and the nullability of both `asserted_context` members. The
renumbering of `## 7. Stability` to `## 8.` leaves no dangling cross-reference: no file in the
repository cites this document by section number.

**`P5R01_NONCLAIMS_PRESERVED`**, **`P5R01_SCHEMA_PROSE_CONSISTENT`**, **`P5R01_VECTORS_UNCHANGED`**
(vector blob `60501abd3c535b5fae81dce0387aba128c19546a` is identical across the correction).

---

## 22. Local validation

All run on this Windows host at `5745962`.

| Command | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS |
| `cargo test -p helm-launch --locked` | PASS |
| `cargo test -p helm-launch --locked --features test-fault-injection` | PASS |
| `cargo clippy -p helm-launch --all-targets --all-features --locked --target x86_64-unknown-linux-gnu -- -D warnings` | PASS |
| `python -m unittest discover -s tools/tests` | `Ran 841 tests … OK (skipped=74)` |
| `python tools/validate_docs.py` | PASS — 147 markdown files, 256 JSON files, 1684 link targets |
| `git diff --check` | clean |

Named surfaces, explicitly:

| Test | Result |
|---|---|
| `receipt::tests::the_published_schema_contract_matches_the_production_serializer` (new) | ok |
| `receipt::tests::the_schema_document_separates_its_machine_block_from_its_prose` (new) | ok |
| `receipt::tests::published_receipt_vectors_match_the_production_serializer` | ok |
| `receipt::tests::the_published_vectors_respect_the_accepted_receipt_bound` | ok |
| `p5_regressions::level4_every_published_receipt_digest_is_recomputable_from_the_artifact` | ok |
| `p2_boundary` | 20 passed |
| `p3_boundary` | 17 passed |
| `p4_boundary` | 15 passed |
| `p5_regressions` | 7 passed |
| `plan_contract` | 19 passed |

No Linux runtime was installed for this review. The Linux Level-4 surface — real children, `strace`,
the machine-code gate, the release reachability proof — did not run here, as expected.

---

## 23. Findings

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P5R01R-M1` | MINOR | schema enforcement, closed world | no — needs a future model change | `backend_values` and `environment_mode_values` are handwritten single-element lists with no wildcard-free guard. Adding a `Backend` or `EnvironmentMode` variant leaves the contract test passing and the published vocabulary silently under-declared. Both lists are correct and product-derived today; the other seven vocabularies **are** guarded. | Carry. Remedy is an `all_backends()` / `all_environment_modes()` helper mirroring the existing seven. |
| `P5R01R-M2` | MINOR | `p2_boundary` source-inclusion scan | no — requires a deliberate future edit | The `include_str!` allowlist matches a 240-character window rather than parsing the path, so an unauthorised include placed immediately before a permitted one passes. **Reproduced at `6e6057c`: pre-existing, not introduced by this correction.** The correction widens the aperture slightly (two shadow zones; the shorter schema filename leaves more headroom). | Carry. Remedy is to match the full expected path suffix instead of a fixed window. |
| `P5R01R-M3` | MINOR | test hygiene | no | `raw_value()` in `receipt.rs` contains a vestigial computation — `let end = members(bytes, 0).0; let _ = end;` — that recomputes the top-level members and discards the result. Harmless and clippy-clean, but dead. | Carry, or delete in a later pass. |
| `P5R01R-B1` | BACKLOG_NONBLOCKING | review evidence | n/a | The ten negative controls reported by the author are not committed and could not be reviewed as written. Probativeness was instead established by this session's own 22-control battery. Nothing in-tree guards the checker against being hollowed out by a future refactor. | Backlog. Optional hardening: refactor `contract()` to take the document text as a parameter so a committed control can feed it mutated text. |

**BLOCKER: 0. IMPORTANT: 0.**

No finding meets an IMPORTANT trigger: product receipt semantics are unchanged; the schema block agrees
with the serializer; the checker is compared against production truth, not a handwritten mirror; field
order is linked to production bytes; no test include can enter product code; `p2_boundary` is not
weakened broadly; schema and vector changes both trigger the workflow **and** reach the validator on all
three operating systems; workflow semantics are unchanged beyond the path filters; and no dependency
was added.

---

## 24. Carried findings

Carried unchanged; no new evidence makes any of them load-bearing:

`P5R-02` MINOR · `P5R-03` MINOR · `P5R-04` MINOR · `P5R-05` MINOR · `P5R-06` MINOR ·
`P4A-03` MINOR / OPEN · `P1-TEST-01` · `P3R-03` / `F-P4-M2` · `P3R-04` · `P3R-05` · `P3R-06` ·
`P3R-07` · `P3R-08` · `P3R-12` · `P3R-16` · `P3R-17` · `P3R-18` · `P3R-19` · `P3R-13` BACKLOG ·
`P3R21-M1`

Plus the new `P5R01R-M1`, `P5R01R-M2`, `P5R01R-M3` (MINOR) and `P5R01R-B1` (BACKLOG_NONBLOCKING).

---

## 25. Whole-crate review carry-forward

This rereview found **no product semantic change and no material boundary weakening**. The whole-crate
independent review at `6e6057c13762759ad3e90ce96dcc8709263d337b` therefore **remains accepted evidence
for every untouched area**. A second whole-crate review is not required: a schema enforcement test was
added, path filters were added, and the P2 test-only include allowlist grew from one exact P5 artifact
to two exact P5 artifacts — none of which meets the bar the owner set.

---

## 26. Pending gate

**`P5 LINUX LEVEL-4 VALIDATION: PENDING FINAL REVIEWED PUBLICATION CI`.**

The Linux-only surface — real child creation, the observation loop, `strace`, the machine-code gate, the
release reachability proof and the Level-4 lifecycle gate — has not run for this chain. It runs
naturally on the hosted matrix after publication.

---

## 27. Outcome

`P5R-01` is **FIXED**. Both limbs are closed: the committed vectors and the published schema document
are now each named exactly in both the `pull_request` and `push` path filters, and the schema document —
which previously had no machine validator anywhere — is now enforced against the production model and
the production serializer's own bytes by a portable Level-1 test that runs on all three operating
systems.

`BLOCKER: 0`, `IMPORTANT: 0`, product `UNCHANGED`, schema `ENFORCED`, vectors `UNCHANGED + ENFORCED`,
CI `REACHABLE`, P2 boundary `PRESERVED`.

**THE P5 WHOLE-CRATE REVIEW + P5R-01 CORRECTION ARE READY FOR ONE FINAL FAST-FORWARD PUBLICATION AND
NATURAL HOSTED P1–P5 VALIDATION.**

**P5 IS NOT YET ACCEPTED.**

`HELM_LAUNCH_P5R01_REREVIEW_PASSED_READY_FOR_FINAL_PUBLICATION_CI`
