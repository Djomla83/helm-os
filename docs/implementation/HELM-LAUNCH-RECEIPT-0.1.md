# helm-launch receipt 0.1 — published evidence contract

> **Status: published evidence contract for the accepted P1–P4 serializer.**
> Added by [HELM-LAUNCH P5](../DECISIONS.md#helm-launch-p5-authorised). It **documents** the
> accepted model and serializer of `crates/helm-launch/src/receipt.rs` and
> `crates/helm-launch/src/model.rs`. It defines no new field, no new value and no new behaviour,
> and it is not a schema-stability promise for any later version.

Machine-readable vectors: [`helm-launch-receipt-0.1-test-vectors.json`](helm-launch-receipt-0.1-test-vectors.json).

---

## 1. What a receipt is, and is not

**A receipt is data.** It is a byte string plus the SHA-256 of exactly those bytes.

| A receipt **is** | A receipt **is not** |
|---|---|
| deterministic bytes from a fixed-order serializer | signed, sealed or attested |
| recomputable: `sha256(exact_bytes)` is reproducible by anyone | proof that HELM produced it |
| a record of **facts the launcher observed** | proof that a particular process produced it |
| copyable, quotable and diffable | proof that `exec` succeeded |
| bounded by `MAX_RECEIPT_BYTES` | proof that the application succeeded |
| free of captured output bytes and of host detail | proof of compatibility, safety, containment or sandboxing |

**A receipt carries ZERO execution authority.** Holding one, or holding bytes that parse as one,
grants nothing. Nothing in HELM consumes a receipt as permission to launch anything.

**Receipt bytes may be copied. Receipt bytes may be fabricated.** The serializer is documented here,
so anyone can emit a byte-identical string without having launched anything. That is expected and it
is not a defect: **this crate makes no receipt-authenticity claim.**

**The digest proves exactly one thing: the identity of an exact byte string.** It does not prove
provenance, issuance, authorship or truth. Two parties agreeing on a digest have agreed only that
they are looking at the same bytes.

What the in-process API *does* prevent is forging a `LaunchReceipt` **value** inside a Rust program:
there is no public constructor, no `Default` and no `Deserialize` for `LaunchReceipt` or
`ReceiptRecord`. That is an in-process API property only. It says nothing about any serialised
receipt.

**No semantic verifier is added to `helm-evidence`.** `helm-launch` 0.1 has zero HELM crate
dependencies, and nothing in the workspace parses a receipt semantically or trusts one. Composition
above `helm-launch` is by exact-byte identity, not by receipt interpretation.

## 2. Envelope

| Property | Value |
|---|---|
| `schema` | `helm-launch-receipt` |
| `version` | `0.1` |
| Encoding | UTF-8, in practice pure ASCII |
| Form | one JSON object, no whitespace anywhere |
| Field order | **fixed**, exactly as listed in section 3 — not alphabetical, never reordered |
| Duplicate keys | none; the bytes satisfy the crate's own strict-JSON reader |
| Digest | SHA-256 over the exact bytes, lowercase hex |
| `MAX_RECEIPT_BYTES` | **8192** — a proven ceiling, **not** a truncation point |

Reformatting a receipt — pretty-printing, reordering, re-encoding — produces **different bytes with
a different digest**. Store the exact bytes unchanged.

### Digest encoding

Every `*_sha256` field is a **64-character lowercase hexadecimal** string, or `null` where the field
is explicitly nullable. No other digest encoding appears.

### Numeric encoding

All numbers are JSON integers in decimal, with no exponent and no fraction. `pre_exec_mode_bits` is
the **decimal** value of the permission bits, so octal `0o755` is written `493`.

## 3. Fields, in serialised order

| # | Field | Type | Notes |
|---|---|---|---|
| 1 | `schema` | string | always `helm-launch-receipt` |
| 2 | `version` | string | always `0.1` |
| 3 | `backend` | enum | see [4.1](#41-backend) |
| 4 | `plan_sha256` | digest | SHA-256 of the exact plan bytes |
| 5 | `asserted_context` | object | see [4.2](#42-asserted_context) |
| 6 | `working_directory_id` | string | caller-chosen identifier, see [4.3](#43-working_directory_id) |
| 7 | `executable` | object | pre-exec measurement, see [4.4](#44-executable) |
| 8 | `argument_count` | u32 | count only; **no argument value is recorded** |
| 9 | `environment_mode` | enum | see [4.5](#45-environment_mode) |
| 10 | `exec_status` | object | see [4.6](#46-exec_status) |
| 11 | `child_end` | object | see [4.7](#47-child_end) |
| 12 | `run_deadline_expired` | bool | the application run deadline expired |
| 13 | `termination` | object | see [4.8](#48-termination) |
| 14 | `stdout` | object | stream facts, see [4.9](#49-stdout-and-stderr) |
| 15 | `stderr` | object | stream facts, see [4.9](#49-stdout-and-stderr) |

Nothing else appears. There is **no** timestamp, no duration, no pid, no descriptor number, no
pathname, no hostname, no user, no environment value, no argument value and no captured output byte.

## 4. Value vocabularies

Every vocabulary below is **closed**: the serializer emits these spellings and no others.

### 4.1 `backend`

| Value | Meaning |
|---|---|
| `linux_x86_64_clone3_pidfd_execveat` | the one accepted 0.1 mechanism: `clone3(CLONE_PIDFD)` and `execveat` of the admitted descriptor |

### 4.2 `asserted_context`

| Field | Type | Meaning |
|---|---|---|
| `subject_spec_sha256` | digest or `null` | a digest the **caller asserted**; the launcher neither verifies nor interprets it |
| `binding_report_sha256` | digest or `null` | as above |

These are carried through unchanged. They confer nothing, and they never affect authorisation.

### 4.3 `working_directory_id`

The caller's identifier for the admitted working directory. Grammar: at most **80** bytes; first
byte `a`–`z` or `0`–`9`; remaining bytes `a`–`z`, `0`–`9`, `.`, `_` or `-`. The grammar admits only
JSON-safe ASCII, so no escaping ever occurs in this field.

**It is an identifier, not a path.** No pathname reaches a receipt.

### 4.4 `executable`

| Field | Type | Meaning |
|---|---|---|
| `pre_exec_body_size` | u64 | body size measured **before** the execution attempt |
| `pre_exec_body_sha256` | digest | SHA-256 of the measured body, **before** the attempt |
| `pre_exec_mode_bits` | u16, decimal | permission bits from the first accepted metadata sample |
| `elf_type` | enum | `et_exec` or `et_dyn` |

**Pre-exec means pre-exec.** The measurement describes the object as admitted, through the exact
descriptor the caller moved in. It is not a claim about what ran, and it is not re-measured
afterwards.

### 4.5 `environment_mode`

| Value | Meaning |
|---|---|
| `empty` | the child was given `envp = [NULL]` |

### 4.6 `exec_status`

Always an object with `kind`. There is **no success variant anywhere in the model**.

| `kind` | Extra fields | Meaning |
|---|---|---|
| `pre_exec_failure` | `stage` (enum), `errno` (i32) | the closed child sequence reported a structured failure before or at `execveat` |
| `indeterminate` | `reason` (enum), `errno` (i32, only when `reason` is `status_read_failed`) | no exec-stage failure was recorded, and no positive exec evidence exists either |

`stage` vocabulary, in child-sequence order: `dup2`, `clear_cloexec`, `chdir`, `close_range`,
`setpgid`, `sigaction`, `sigmask`, `no_new_privs`, `exec`.

`reason` vocabulary:

| Value | Meaning |
|---|---|
| `status_eof_without_record` | the exec-status channel closed cleanly with no record. **This is the ordinary shape of a run that appears to have worked, and it is still not exec success.** |
| `status_record_malformed` | a record arrived that is not the accepted eight-byte shape |
| `pre_exec_status_timeout` | no record and no clean end-of-file within the fixed pre-exec bound |
| `status_read_failed` | reading the channel failed; `errno` carries the raw number |

### 4.7 `child_end`

Always an object with `kind`.

| `kind` | Extra fields | Meaning |
|---|---|---|
| `exited` | `code` (i32) | the direct child exited with this raw status. **`code` 0 means only that.** |
| `signaled` | `signal` (i32), `core_dumped` (bool) | terminated by this signal. The signal number is retained **regardless** of `core_dumped` |
| `end_unobservable` | — | the end could not be classified, for example because something else reaped the child |
| `end_not_observed` | — | no end was observed within the bound after `SIGKILL`. **No claim is made that the child is running at any later moment**, and this value is latched: a later observation never revises it |

### 4.8 `termination`

| Field | Type | Meaning |
|---|---|---|
| `sigterm_sent` | bool | this crate sent `SIGTERM` to the direct child through its pidfd |
| `sigkill_sent` | bool | this crate sent `SIGKILL` to the direct child through its pidfd |
| `group_sweep` | enum | the guarded cleanup sweep disposition |

`group_sweep` vocabulary:

| Value | Meaning |
|---|---|
| `issued` | one process-group signal **call was made**. It says nothing more — see below |
| `not_issued_group_not_established` | the parent's own `setpgid(child, child)` did not succeed, so no group authority existed and no sweep was issued |
| `not_issued_child_already_reaped` | the child had already been reaped elsewhere, so no sweep was issued |

**`issued` is best-effort cleanup, not containment.** It records that one call was made. It does
**not** claim that any descendant received the signal, that any descendant died, or that a process
tree was contained. Descendants may outlive the launch.

Group authority is **never inferred** from an observed process group; only the parent's own
successful `setpgid` establishes it.

### 4.9 `stdout` and `stderr`

Identical shape.

| Field | Type | Meaning |
|---|---|---|
| `bytes_drained` | u64 | total bytes read from the stream, **counted and hashed in full** even when the in-memory prefix bound is smaller |
| `drained_sha256` | digest | SHA-256 over **every** drained byte |
| `completeness` | enum | how draining ended |

`completeness` vocabulary:

| Value | Meaning |
|---|---|
| `complete_at_eof` | the stream reached end-of-file and every byte was drained |
| `writer_retained_after_child_exit` | a writer outlived the child and the bounded post-exit drain ended first |
| `read_stopped_child_end_not_observed` | draining stopped because the child's end was not observed |
| `read_failed` | a read failed; `errno` carries the raw number. **A read error is its own fact and is never reported as end-of-file** |

**No captured byte ever reaches the receipt.** The receipt carries counts and digests only. The
in-memory output prefix is a separate, memory-only convenience on `LaunchOutcome`; it is bounded,
it is not persisted by this crate, and it is not part of this schema.

## 5. Nonclaims restated

* **No `ExecSucceeded` value exists anywhere in this crate.** A clean exec-status end-of-file is
  `indeterminate` / `status_eof_without_record` and nothing stronger.
* **`exited` with `code` 0 is not application success.** It is a raw status.
* **`group_sweep: issued` is not containment**, not a descendant-killed claim, and not a process-tree
  guarantee.
* **`end_not_observed` is not "still running".** It is the absence of an observation, latched.
* **No authenticity, provenance, issuer or signature** is expressed or implied. The digest is byte
  identity only.
* **No privileged transition is claimed or validated.** Nothing here says a privilege change was
  attempted, prevented or observed.
* **No containment, sandboxing, cgroup, namespace, Wine or Proton semantics** are expressed. None
  exist in 0.1.

## 6. Published test vectors

[`helm-launch-receipt-0.1-test-vectors.json`](helm-launch-receipt-0.1-test-vectors.json) carries 11
vectors. It is **not** a signed or trusted manifest; it is a portable fixture so that an independent
implementation can check its reading of this document.

Each vector carries:

| Key | Meaning |
|---|---|
| `name` | stable vector name |
| `exact_byte_length` | length of the receipt bytes |
| `sha256` | lowercase hex SHA-256 **of those exact bytes** |
| `exact_bytes_base16` | the receipt bytes, lowercase hex — **the authoritative encoding** |
| `exact_bytes_utf8` | the same bytes as a readable JSON string, for human review |

Two encodings are published deliberately. A JSON string of a JSON document needs escaping, and
escaping is exactly where a near-miss hides; the hexadecimal copy is unambiguous, and the in-crate
test asserts that **both decode to the same bytes** and that both equal what the production
serializer emits.

`receipt::tests::published_receipt_vectors_match_the_production_serializer` is the gate. For every
vector it rebuilds the record in-crate, serialises it through **production code**, requires exact
byte equality against the published hex, requires the readable copy to be the same bytes, recomputes
SHA-256 **from the published bytes** and requires it to equal both the published digest and the
product digest, requires the length to be within `MAX_RECEIPT_BYTES`, and requires every published
digest to be distinct. There is no shadow serializer and no withheld field.

The vectors are **Level 1**: no process, no descriptor, no host state. They run on Linux, Windows
and macOS and must produce **identical bytes and identical digests** on all three. A
platform-dependent result is a defect, not a platform expectation.

**There is deliberately no exec-success vector**, because no such value exists in the model.

### Covered shapes

| Vector | Shape |
|---|---|
| `normal_direct_child_exit` | ordinary run, clean streams |
| `pre_exec_failure_exec_eacces` | `pre_exec_failure` at stage `exec` |
| `status_eof_without_record_indeterminate` | clean status EOF, non-zero exit |
| `run_deadline_sigterm_then_sigkill` | deadline expiry with both signals |
| `group_sweep_issued` | sweep issued under established authority |
| `group_sweep_not_issued_group_not_established` | same termination facts, sweep withheld for want of authority |
| `foreign_reaped_end_unobservable` | foreign reap, sweep withheld |
| `end_not_observed_latched` | `end_not_observed` with a stopped stream |
| `stream_completeness_variants` | retained writer and `read_failed` together |
| `signaled_child_keeps_core_flag` | signal retained with `core_dumped: true` |
| `widest_near_max_receipt` | widest numeric and identifier values |

## 7. Machine-checkable normative contract

Sections 1 to 6 are written for people. This section is written for a test.

**Three authorities, each with its own scope, and no overlap between them:**

1. **The normative contract block below pins the exact structural and vocabulary facts** — the
   schema name and version, the receipt bound, every closed value vocabulary, the top-level field
   order, the field order of every nested object and of every variant shape, which fields are
   nullable, and the byte-level encoding rules that follow from compact JSON.
   `receipt::tests::the_published_schema_contract_matches_the_production_serializer` reads **this
   document**, parses the block, and checks every line against the production model and the
   production serializer's own output. A line that stops being true fails that test.
2. **The production serializer and the published vectors pin the exact bytes.** No prose can do
   that, and none is asked to:
   `receipt::tests::published_receipt_vectors_match_the_production_serializer` and
   `p5_regressions::level4_every_published_receipt_digest_is_recomputable_from_the_artifact` are
   where byte identity and digest recomputability are established.
3. **The explanatory prose and the nonclaims of sections 1, 4 and 5 remain documentation.** They
   are held by review and by the crate's own vocabulary scans, **not** by the block below. Claims
   such as "carries zero execution authority", "is not containment" or "no authenticity is claimed"
   are deliberately **absent** from the machine block, because they are not schema vocabulary and a
   key asserting them would be a claim about meaning that no test could check.

Do not read the block as a second, competing definition of the receipt. Where it names a fact, that
fact is also true in sections 1 to 6; the block exists so the two cannot drift apart unnoticed.

**Grammar.** One `key=value` per line between the two markers, no whitespace around `=`, no blank
lines, no comments. A value is either a single token or a comma-separated list in the exact order
the contract fixes. A duplicate key, an unknown key or a missing key fails the test, so the block
cannot be quietly extended, reordered or hollowed out.

<!-- HELM-LAUNCH-RECEIPT-CONTRACT:BEGIN -->
schema=helm-launch-receipt
version=0.1
max_receipt_bytes=8192
encoding=utf8
formatting_whitespace=none
trailing_newline=absent
bool_literals=true,false
null_literal=null
digest_encoding=lowercase_hex_64
argument_count_encoding=decimal_u32
backend_values=linux_x86_64_clone3_pidfd_execveat
environment_mode_values=empty
elf_type_values=et_exec,et_dyn
child_stage_values=dup2,clear_cloexec,chdir,close_range,setpgid,sigaction,sigmask,no_new_privs,exec
indeterminate_reason_values=status_eof_without_record,status_record_malformed,pre_exec_status_timeout,status_read_failed
exec_status_kinds=pre_exec_failure,indeterminate
child_end_kinds=exited,signaled,end_unobservable,end_not_observed
group_sweep_values=issued,not_issued_group_not_established,not_issued_child_already_reaped
stream_completeness_values=complete_at_eof,writer_retained_after_child_exit,read_stopped_child_end_not_observed,read_failed
top_level_order=schema,version,backend,plan_sha256,asserted_context,working_directory_id,executable,argument_count,environment_mode,exec_status,child_end,run_deadline_expired,termination,stdout,stderr
asserted_context_order=subject_spec_sha256,binding_report_sha256
asserted_context_nullable=subject_spec_sha256,binding_report_sha256
executable_order=pre_exec_body_size,pre_exec_body_sha256,pre_exec_mode_bits,elf_type
termination_order=sigterm_sent,sigkill_sent,group_sweep
termination_bool_fields=sigterm_sent,sigkill_sent
stream_order=bytes_drained,drained_sha256,completeness
stream_order_read_failed=bytes_drained,drained_sha256,completeness,errno
exec_status_order_pre_exec_failure=kind,stage,errno
exec_status_order_indeterminate=kind,reason
exec_status_order_indeterminate_status_read_failed=kind,reason,errno
child_end_order_exited=kind,code
child_end_order_signaled=kind,signal,core_dumped
child_end_order_end_unobservable=kind
child_end_order_end_not_observed=kind
<!-- HELM-LAUNCH-RECEIPT-CONTRACT:END -->

**What each key is checked against.** Nothing here is checked against prose.

| Key group | Checked against |
|---|---|
| `max_receipt_bytes` | the production constant `MAX_RECEIPT_BYTES` |
| every `*_values`, `*_kinds` | the production `as_str()` of every variant, enumerated by a wildcard-free `match` so a new variant fails compilation before it can go unlisted |
| every `*_order*` | the key order read back out of **production serializer output**, for a record of each shape, and again out of the **committed vector bytes** |
| `asserted_context_nullable` | a record with both digests absent and a record with both present |
| `encoding`, `formatting_whitespace`, `trailing_newline`, `bool_literals`, `null_literal`, `digest_encoding`, `argument_count_encoding` | the production bytes themselves |
| `schema`, `version` | the values the production serializer emits for those two fields |

The reader the test uses walks compact receipt bytes and reports the order of the keys it finds. It
is a reader, not a writer: **no second serializer exists**, and nothing in the test can produce
receipt bytes.

## 8. Stability

This document describes **version `0.1`**. It is a description of the accepted serializer, not a
compatibility promise. A later version may change field order, vocabulary or bounds, and would
change `version` when it does. Any consumer should treat an unexpected `schema` or `version` as
unreadable rather than guessing.
