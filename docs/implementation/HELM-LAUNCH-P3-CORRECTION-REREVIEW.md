# HELM-LAUNCH P3 — BOUNDED INDEPENDENT CORRECTION RE-REVIEW

> # 🔎 BOUNDED INDEPENDENT CORRECTION RE-REVIEW — NOT A FULL P3 UNSAFE REVIEW
>
> **Session provenance.** **THIS REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `7b749b8116b53ed07a8758b9feec50334546b7cd` (owner disposition of the independent findings) and
> `f144d3004826276a5f2281ffea146c2e99645033` (the bounded correction of `P3R-10` and `P3R-11`).
> It read them, re-derived the corrected evidence from source, from freshly emitted Linux x86_64
> machine code and from independently executed parsers, and reached its own conclusions.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact supplied by the repository owner, not an inference from commit metadata.

> **This document does NOT supersede
> [`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md)
> (`4c834415e8e0224b1eb1ce6c6546245cdfda0962`), the genuinely independent full P3 unsafe review.**
> That review remains authoritative for every P3 area this correction did not touch: the unsafe
> confinement, the `clone3` ABI, the signal ABI, the borrowed `ChildPlan`, the closed child
> sequence, descriptor isolation, exec authority and direct-child cleanup.
>
> **`4c834415` + this bounded rereview together form the P3 pre-publication independent review
> record.** Neither is complete alone.
>
> [`HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md) (`168fe133`)
> is an **AUTHOR SELF-REVIEW**, is **NOT INDEPENDENT**, and is not an approval artifact.

> **P1 ACCEPTED. P2 ACCEPTED. P3 AUTHORISED / CORRECTED CANDIDATE, NOT ACCEPTED. P4 AND P5 NOT
> AUTHORISED. TRIAL #3 STAYS `MECHANISM_REJECTED`. TRIAL #4 NOT AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| BLOCKER | **0** |
| IMPORTANT | **1** — `P3R-15` (new) |
| MINOR | 1 new — `P3R-16`; `P3R-03` … `P3R-09`, `P3R-12` carried forward unchanged |
| `P3R-10` (report contract) | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-11` (indirect `jmp` fail-closedness) | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-01` (artifact selection / injection proof) | **REMAINS FIXED** |
| `P3R-02` (child closed world at machine level) | **REMAINS FIXED** |
| Backend product semantics | **BYTE-UNCHANGED BY CORRECTION** |
| Classification | **`HELM_LAUNCH_P3_CORRECTION_REREVIEW_NEEDS_FIX`** |

Both corrections the owner required are genuinely made, and this rereview confirms both from
evidence it produced itself rather than from the author's report. `P3R-10` is fixed structurally,
not merely by renaming a key. `P3R-11` is fixed decisively, and a differential against the
pre-correction checker reproduces the original fail-open and shows it closed, along with three
further fail-open holes the correction closed as a side effect.

The candidate nevertheless cannot be published as it stands. Regenerating fresh Linux x86_64
assembly and scanning it file-wide — rather than reading only the corrected lines — found that the
checker's control-transfer model still omits one mnemonic family: **a conditional branch whose
target is a function symbol**. That is the same fail-open class as `P3R-11`, in the same
load-bearing gate, and unlike `P3R-11` it is not hypothetical: **rustc emits exactly that
instruction in this crate's own release-codegen output.** It is recorded as `P3R-15`.

No memory-safety, authority or boundary defect was found. Nothing in this correction touches the
P3 safety case.

## 1. Starting state, verified

| Requirement | Observed | Result |
|---|---|---|
| Branch | `docs/helm-launch-architecture` | ✅ |
| Worktree | clean (`git status --short` empty) | ✅ |
| Local `HEAD` | `f144d3004826276a5f2281ffea146c2e99645033` | ✅ |
| `origin/docs/helm-launch-architecture` | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` | ✅ |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` | ✅ |
| Branch relation | **8 ahead, 0 behind** | ✅ |

Exact local chain, verified commit by commit:

```text
9fb0f8ca  accepted P2 base
 └─ 7bb016f  docs(launch): authorise p3 unsafe backend          P3 authority
     └─ afe8922  feat(helm-launch): implement p3 child backend  original implementation
         └─ 168fe13  author self-review — NOT INDEPENDENT
             └─ ac823cd  owner disposition of author-review findings
                 └─ 672228b  first bounded P3 correction
                     └─ 4c83441  GENUINELY INDEPENDENT full P3 unsafe review
                         └─ 7b749b8  owner disposition of independent findings
                             └─ f144d30  bounded correction for P3R-10 / P3R-11   ← HEAD
```

`origin` was fetched read-only. Nothing was pushed, amended, rebased, squashed or dispatched. No
implementation, test, checker or workflow file was modified by this review.

## 2. Correction scope, verified against the owner's boundary

`7b749b8` changes exactly two files — `docs/DECISIONS.md` and `docs/PROJECT_STATE.md` — and adds
177 lines of status record with no product change.

`f144d30` changes exactly three files, all of them test-level or checker-level as the disposition
required:

| File | Change |
|---|---|
| [`crates/helm-launch/src/backend/tests.rs`](../../crates/helm-launch/src/backend/tests.rs) | +537 / −80 |
| [`tools/helm_launch_child_closure.py`](../../tools/helm_launch_child_closure.py) | +190 / −80 |
| [`tools/tests/test_helm_launch_machine_proofs.py`](../../tools/tests/test_helm_launch_machine_proofs.py) | +125 |

`git diff --name-only 4c834415 f144d30` returns those three plus the two status documents, and
**nothing else** — no workflow, no Cargo file, no ADR, no plan.

## 3. Backend product semantics — byte-unchanged proof

Blob identity compared across the whole correction span, `4c834415` → `f144d30`, so both the
disposition and the correction are covered:

| File | Blob at `4c834415` | Blob at `f144d30` | Result |
|---|---|---|---|
| [`backend/child.rs`](../../crates/helm-launch/src/backend/child.rs) | `d8b75c67…` | `d8b75c67…` | **identical** |
| [`backend/spawn.rs`](../../crates/helm-launch/src/backend/spawn.rs) | `5575639e…` | `5575639e…` | **identical** |
| [`backend/syscall.rs`](../../crates/helm-launch/src/backend/syscall.rs) | `38d95fb4…` | `38d95fb4…` | **identical** |
| [`backend/mod.rs`](../../crates/helm-launch/src/backend/mod.rs) | `12467e87…` | `12467e87…` | **identical** |
| [`backend/injection.rs`](../../crates/helm-launch/src/backend/injection.rs) | `fa3704d0…` | `fa3704d0…` | **identical** |
| [`src/authority.rs`](../../crates/helm-launch/src/authority.rs) | `3a877d61…` | `3a877d61…` | **identical** |
| `Cargo.toml` | `953ca8a3…` | `953ca8a3…` | **identical** |
| `Cargo.lock` | `b2df4153…` | `b2df4153…` | **identical** |
| [`ADR-0024`](../adr/ADR-0024-launch-authority.md) | `97a76f9c…` | `97a76f9c…` | **identical** |
| [productization plan](HELM-LAUNCH-PRODUCTIZATION-PLAN.md) | `c366ad7f…` | `c366ad7f…` | **identical** |
| `.github/workflows/**` | — | — | **0 files changed** |

**`BACKEND PRODUCT SEMANTICS: BYTE-UNCHANGED BY CORRECTION`.**

No unsafe block, no syscall number, no ABI, no stage order, no authority check and no public item
is touched. The bounded-rereview premise therefore holds, and the full unsafe review at `4c834415`
remains valid for all of it.

Boundary invariants re-confirmed at `HEAD`: public `launch()` **ABSENT**, `LaunchOutcome`
**ABSENT**, process-group sweep **ABSENT** (the only `kill`/`tgkill` tokens in the crate are inside
a test assertion that *forbids* those syscalls), and the `pub use` export list **byte-identical to
the accepted P2 base** `9fb0f8ca`.

## 4. `P3R-10` — the report contract, re-established independently

### 4.1 Emitter schema, inventoried from source

Every key the fixture can emit, extracted mechanically from
[`REPORT_FIXTURE_SOURCE`](../../crates/helm-launch/src/backend/tests.rs#L392) rather than taken
from the author's list:

```text
marker line   HELM-LAUNCH-P3-FIXTURE/1
scalars       name  argv_count  env_count  cwd_dev  cwd_ino  fd_count
              no_new_privs  sig_blk  sig_ign  sig_cgt  tgid  pgid_is_self
alternative   cwd_error            (replaces cwd_dev + cwd_ino when stat fails)
repeated      argv  env  fd
terminator    end
```

Sixteen distinct keys. The parser's accepted schema is
[`REPORT_SCALARS`](../../crates/helm-launch/src/backend/tests.rs#L569) (12) +
[`REPORT_OPTIONAL`](../../crates/helm-launch/src/backend/tests.rs#L587) (1, `cwd_error`) +
[`REPORT_REPEATED`](../../crates/helm-launch/src/backend/tests.rs#L590) (3). **Sixteen.**

**`EMITTED KEY SET == PARSER ACCEPTED SCHEMA KEY SET`**, exactly, with `cwd_error` the one
explicitly documented optional field. No producer key falls into a catch-all arm; no consumer key
is impossible for the producer to emit. The defect shape — a key emitted under one spelling and
read under another — is now structurally impossible, because any key outside the schema is refused
as `UnknownKey` rather than ignored.

### 4.2 No missing-value path exists

[`Report`](../../crates/helm-launch/src/backend/tests.rs#L528) derives `Debug, PartialEq, Eq` and
**no `Default`**; no `impl Default for Report` exists anywhere in the crate. Every field is filled
from `scalar(key)?`, which returns `Missing(key)` when absent. There is therefore **no
`Default → tgid = 0` path and no equivalent**: a `Report` value cannot exist carrying a number the
producer never sent. That is a structural fix, not a spelling fix.

### 4.3 Identity semantics — PID, TGID and PGID are not conflated

| Fact | Producer | Meaning | Asserted as |
|---|---|---|---|
| `tgid` | `std::process::id()` | the executed image's own process identity; on Linux the thread-group id of a single-threaded process, preserved across `execveat` | `i64::from(report.tgid) == completed.launch.child.pid()` |
| `pgid_is_self` | field 5 (`pgrp`) of `/proc/self/stat`, compared to `std::process::id()` | whether the image **leads its own process group** — a statement about the group, not about identity | `report.pgid_is_self` is `true`, **separately** |

The `/proc/self/stat` scan starts after the **last** `)`, then skips `state` and `ppid` to reach
`pgrp`, which is correct for a `comm` field that may itself contain spaces and parentheses.

`child.pid()` is the `clone3` return value in the parent, and `execveat` preserves the pid, so
`report.tgid == child.pid()` is genuinely probative of direct-child identity. The two facts are
carried in two fields, parsed by two rules and checked by two assertions.

The group-leader assertion is **satisfiable and not a tautology**: the parent issues
`setpgid(child, child)` at [`spawn.rs:415`](../../crates/helm-launch/src/backend/spawn.rs#L415) and
the child issues `setpgid(0, 0)` at
[`child.rs:226`](../../crates/helm-launch/src/backend/child.rs#L226) before exec, so the executed
image leads its own group whichever ran first — while the producer self-test proves the same field
reads `false` when the fixture runs outside the backend.

**`P3R10_IDENTITY_EVIDENCE_SOUND`.**

### 4.4 Closed parser — executed, not only read

The backend is `cfg(all(target_os = "linux", target_arch = "x86_64"))`, so these tests do not
compile on this host. `try_unhex`, `Report`, `ReportRejection`, the schema constants and
`try_parse_report` were therefore extracted verbatim into a throwaway crate outside the repository,
together with the committed test body, and **executed**:

```text
COMMITTED SCHEMA TEST BODY: PASSED
```

Every refusal the owner required was then re-exercised by probes written for this rereview, not
copied from the committed suite:

| Class | Probe | Result |
|---|---|---|
| unknown key | `pgid_is_leader=true` | `UnknownKey("pgid_is_leader")` |
| producer/parser spelling disagreement | `tgid` → `pgid` | `UnknownKey("pgid")` |
| missing mandatory scalar | each of the 12 scalars removed | `Missing(k)` for 9; **3 exceptions → `P3R-16`** |
| bad integer | `not-a-number`, `-1`, empty, ` 42`, `4294967296` | `Malformed("tgid")` |
| bad boolean | `yes`, `1`, `True`, `unavailable` | `Malformed("pgid_is_self")` |
| wrong / absent marker | `…/2`, `""`, `not a marker at all` | `Marker` |
| missing terminator | `end` removed | `Truncated` |
| non-text | `[0xff, 0xfe, 0xfd]` | `NotText` |
| declared count mismatch | `argv_count`, `env_count`, `fd_count`, duplicate `env` | `Malformed(count key)` |
| argv hole | indices `0, 2` with `argv_count=2` | `Malformed("argv_count")` |
| duplicate scalar / repeated | `tgid`, `name`, `argv`, `fd`, `pgid_is_self` | `Duplicate(k)` |
| malformed repeated entry | `argv=1:6`, `argv=1:zz`, `argv=one:62`, `argv=1`, `fd=2:7069706` | `Malformed(k)` |
| line with no separator, empty line | — | `UnknownKey(line)` |
| trailing key after `end` | `rogue=1` | `UnknownKey("rogue")` |
| `cwd_error` path | replaces `cwd_dev`/`cwd_ino` | `Missing("cwd_dev")` — refused, reason visible |

Two probe expectations of this review were wrong rather than the parser: `tgid=+1` parses because
Rust's `u32::from_str` accepts a leading `+`, and CRLF input parses because `str::lines()` strips a
trailing `\r`. Both are benign and neither weakens an assertion.

`pgid_is_self=unavailable` — exactly what the fixture emits when it cannot read its group —
is **refused**, so an image that cannot determine its group produces no evidence rather than a
plausible-looking guess. That is the correct fail-closed direction.

**`P3R10_REPORT_SCHEMA_SOUND`**, subject to the MINOR `P3R-16` below.

### 4.5 Producer self-test

[`report_fixture_reports_without_the_backend`](../../crates/helm-launch/src/backend/tests.rs#L1084)
validates the contract directly, before any launcher evidence is trusted:

* the fixture is **`spawn`ed, not `output()`-ed**, so the harness holds `running.id()` — the pid it
  observed itself;
* `parse_report` refuses a malformed report with its exact reason, failing this test;
* the marker must name **this** fixture;
* `report.tgid == observed_identity` — so `tgid` is proven to be the fixture's own process identity
  against a value the harness knows independently, rather than "some number the producer printed";
* `!report.pgid_is_self` — a **negative control** proving the field tracks the real group and is not
  a constant, which is what makes the launched `true` a result;
* argv, a non-empty inherited environment (so the later empty-environment assertion is meaningful)
  and the stderr marker are checked too.

**Caching does not bypass validation.**
[`fixture_binary`](../../crates/helm-launch/src/backend/tests.rs#L352) caches compiled *bytes* by a
SHA-256 prefix of the source text, but the producer self-test **runs the cached binary and
re-validates its report on every execution**. Validation is per-run, not per-build. (That the cache
returns without verifying file contents is the separate, already-open `P3R-12`; it is not affected
by this correction and is not promoted here.)

One structural note, not a finding: cargo's harness does not order tests, so "before" is not a
sequencing guarantee. It does not need to be — a failing producer self-test makes the whole suite
red, so no launcher evidence is ever accepted while the contract is broken.

**`P3R10_PRODUCER_CONTRACT_SOUND`.**

### 4.6 The corrected tests compile for the real target

`cargo clippy -p helm-launch --all-targets --all-features --locked --target
x86_64-unknown-linux-gnu -- -D warnings` is **clean**. The corrected `tests.rs` type-checks for
Linux x86_64 with warnings denied. Real execution remains the publication CI gate.

**`P3R-10`: INDEPENDENTLY VERIFIED FIXED.**

## 5. `P3R-11` — function-escaping control transfers

### 5.1 The model, read whole

[`tools/helm_launch_child_closure.py`](../../tools/helm_launch_child_closure.py) was read in full
(578 lines), not only at the changed lines. The abstraction is now **function-escaping control
transfer**, not `call`: `TRANSFER` matches `call`/`callq`/`calll`/`lcall` **and**
`jmp`/`jmpq`/`jmpl`/`ljmp`, after optional `bnd`/`notrack`/`rep`/`lock`/segment prefixes, and every
operand lands in exactly one of four buckets — named target, own local label, indirect, unresolved.
The last two both fail the gate, and no resolver for an indirect target exists.

Two design points matter and are correct. `local_labels` is built over the **whole file** before
any body is classified, so a forward branch resolves; and the label set is **per function**, so a
`.L` target belonging to a different function is an unresolved escape rather than an ignored
branch.

### 5.2 Independent probes — 37 of 37

Synthetic assemblies written for this rereview, fed to the committed checker:

| Class | Forms probed | Result |
|---|---|---|
| direct call → forbidden | `memcpy@PLT`, `core::panicking::panic`, `__rust_alloc` | **FAIL** |
| **direct tail jump → forbidden** | `jmp memcpy@PLT`, `jmpq memmove`, `jmp panic_fmt` | **FAIL** |
| **indirect call** | `call/callq *%rax`, `*(%rax)` | **FAIL** |
| **indirect jump** | `jmp/jmpq *%rax`, `*(%rax)`, `*0x10(%rsp)` | **FAIL** |
| jump table | `jmpq *.LJTI0_0(,%rax,8)` | **FAIL** |
| prefixed indirect | `bnd jmpq *%rax`, `notrack jmp *%r11` | **FAIL** |
| far transfer | `lcall *%rax`, `ljmp *(%rbx)` | **FAIL** |
| GOT-named (resolvable) | `callq/jmpq *memcpy@GOTPCREL(%rip)` | edge to `memcpy` → **FAIL** |
| own local label | `jmp .LBB0_1` where defined | **PASS**, counted intra-function |
| undefined local label | `jmp .LBB9_9` | **FAIL** — unresolved |
| another function's local label | `jmp .LBB7_7` defined elsewhere | **FAIL** — unresolved |
| **tail jump → allowed internal helper** | `jmp helper` | **PASS**; helper **enters the closure** |
| transitive via tail jump | `jmp helper` → helper `callq memcpy@PLT` | **FAIL** |
| transitive tail from helper | `callq helper` → helper `jmp malloc@PLT` | **FAIL** |
| no operand | `callq`, `jmp` | **FAIL** — unresolved |
| unresolvable spelling | `jmp memcpy+0x20` | **FAIL** — unresolved |
| absolute numeric | `callq 0x401000` | **FAIL** — unresolved |
| missing child root | — | **ERROR**, loud |
| ambiguous root | two `child_main` symbols | **ERROR**, loud |
| syscall-free child | shim with no `syscall` | **FAIL** |
| positive control absent | nothing forbidden anywhere in file | **FAIL** |
| comment cannot hide a transfer | `callq memcpy@PLT # inlined` | **FAIL** |
| comment cannot invent one | `# callq memcpy@PLT` | **PASS** |

**37 of 37 behaved as the owner disposition requires.**

### 5.3 Differential against the pre-correction checker

The same probes, run against `tools/helm_launch_child_closure.py` **as of `4c834415`**:

| Probe | `4c834415` | `f144d30` |
|---|---|---|
| `jmpq *%rax` — **the `P3R-11` defect** | **PASS** (silently dropped) | **FAIL** |
| `jmp *(%rax)` | **PASS** | **FAIL** |
| `jmpq *.LJTI0_0(,%rax,8)` | **PASS** | **FAIL** |
| `jmp .LBB9_9` (undefined local label) | **PASS** | **FAIL** |
| `callq` with no operand | **PASS** | **FAIL** |
| `callq 0x401000` | **PASS** | **FAIL** |
| `callq *%rax` (already caught) | FAIL | FAIL |
| `jmp memcpy@PLT` (already caught) | FAIL | FAIL |

The original fail-open is reproduced on the old checker and closed on the new one, and three
further fail-open holes are closed with it. This is the decisive evidence for `P3R-11`, and it
rests on executing both versions rather than on reading the diff.

### 5.4 The committed negative tests pass for the right reasons

`python -m unittest discover -s tools/tests` — **820 tests, OK (74 skipped)**; the machine-proof
file contributes **26**. Each was inspected rather than counted. The added cases assert the
**specific bucket** — `len(result["indirect_transfer_sites"]) == 1`,
`len(result["unresolved_transfer_sites"]) == 1`, or the external edge's category string — not
merely that `problems` is non-empty, so none can pass because of an unrelated earlier parser error.
`test_a_direct_tail_jmp_to_an_internal_helper_is_traversed` additionally asserts the helper is in
`result["closure"]` **and** that the `(child, helper)` internal edge exists **and** that `malloc` is
found through it, which proves traversal rather than mere rejection.

Coverage against the owner's list: memcpy edge ✅, panic edge ✅, transitive allocator edge ✅,
indirect call ✅, indirect jmp ✅ (6 spellings), direct tail jump to forbidden external ✅,
direct tail jump to allowed internal helper traversed ✅, unresolved direct jump ✅, unknown
operand form ✅, missing root ✅, syscall-free child ✅, late archive member ✅.

**One gap:** there is no test for an unknown control-transfer **mnemonic**, only for unknown
**operand forms**. The suite mirrors the implementation's remaining blind spot — see `P3R-15`.

### 5.5 File-wide indirect-transfer visibility

Freshly generated Linux x86_64 assembly, not the author's saved output:

| Profile | file functions | **total indirect transfers seen** | file-wide unresolved | child-closure indirect | child-closure unresolved |
|---|---|---|---|---|---|
| debug / test | 5 769 | **1 809** | **0** | **0** | **0** |
| release codegen | 723 | **450** | **0** | **0** | **0** |
| debug + injection | 5 794 | **1 821** | **0** | **0** | **0** |

The pre-correction checker saw **1 771** and **421** on the identical files — a blind spot of
**38** and **29** transfers, matching the raw `jmp *…` counts once GOT-named forms (which are
resolvable named edges, correctly traversed rather than rejected) are excluded.

**`jmpq *…` is no longer invisible.** Indirect transfers in unrelated functions outside the child
closure are expected and acceptable; the point is that the parser now sees them. That file-wide
**unresolved count is 0** across 5 769 function bodies is itself strong evidence that the four-bucket
model covers rustc's real output — with the single exception recorded as `P3R-15`.

**`P3R-11`: INDEPENDENTLY VERIFIED FIXED.**

## 6. Fresh machine closures

Generated with the crate's already-installed `x86_64-unknown-linux-gnu` std target. **No WSL Rust
toolchain was installed.** Each used a fresh target directory.

### 6.1 Debug / test profile

```text
root     _ZN11helm_launch7backend5child10child_main17haee81f8dae299a2eE
closure  9 functions      internal edges 12      local branches 33
syscalls 1 instruction, shim OUT-OF-LINE (exactly one, in the reviewed shim)
external edges 0   indirect 0   unresolved 0
positive control 4 witness functions; named control reaches 143 external helpers
CHILD CLOSURE CHECK PASSED
```

Closure: `child_main`, `syscall6`, `is_error`, `errno_of`, `fail`, and four `core` numeric/result
helpers (`i64::unsigned_abs`, `TryFrom<u64> for i32`, `Result::unwrap_or_default`,
`i32::to_le_bytes`) — all defined in the same unit, none an external runtime helper. This matches
the 9 functions the full independent review reported, re-derived rather than copied.

### 6.2 Release codegen

```text
root     _ZN11helm_launch7backend5child10child_main17haee81f8dae299a2eE
closure  2 functions      internal edges 1       local branches 3
syscalls 18 instructions, shim INLINED  (child_main 16, fail 2)
external edges 0   indirect 0   unresolved 0
positive control 4 witness functions
CHILD CLOSURE CHECK PASSED
```

**`RELEASE CHILD BACKEND ACTUALLY INSTANTIATED: YES`** — the root resolves uniquely, its body is
real machine code, and it issues 16 `syscall` instructions inline.

**`RELEASE PROOF NOT DCE-ONLY: YES`** — proven by contrast. A plain
`cargo rustc --lib --release --all-features` emits assembly in which `backend5child10child_main`
is **absent** (eliminated, since P3 adds no public consumer), while the release-codegen path
(`--profile test -Copt-level=3 -Cdebug-assertions=off`) **instantiates the same child with 18
syscalls**. The clean result therefore describes real optimised child code, not an empty artifact.

### 6.3 Debug + `test-fault-injection`

```text
closure 10 functions   internal edges 14   syscalls 1 (out-of-line)
external edges 0   indirect 0   unresolved 0      CHECK PASSED
```

Not a CI step today (that is the open backlog item `P3R-13`); verified here.

**`P3R-02`: REMAINS FIXED.** No `memcpy`, `memmove`, `memset`, allocator, panic, unwind, libstd or
glibc helper is reachable from `child_main` in any of the three profiles, and no escaping transfer
is unresolved in any of them.

## 7. `P3R-01` — injection proof regression

[`tools/helm_launch_injection_proof.py`](../../tools/helm_launch_injection_proof.py) is
**byte-identical** across the correction (`861512d8…` at both `4c834415` and `f144d30`). The
established evidence was rerun rather than redesigned:

```text
=== debug + test-fault-injection (POSITIVE CONTROL) ===
  libhelm_launch.rlib    4 323 136 bytes, 128 members inspected, marker PRESENT
      hit in lib.rmeta                                   (marker=True, symbol=True)
      hit in helm_launch-…rcgu.o                         (marker=True, symbol=True)
  libhelm_launch-….rmeta   720 040 bytes,   1 member  inspected, marker PRESENT

=== release + --all-features (NEGATIVE PROOF) ===
  libhelm_launch.rlib    1 037 276 bytes,  11 members inspected, marker ABSENT
  libhelm_launch-….rmeta   773 195 bytes,   1 member  inspected, marker ABSENT

INJECTION PROOF PASSED
```

Exact artifact selection still depends on cargo's machine-readable record: the tool filters
`--message-format=json-render-diagnostics` output for `reason == "compiler-artifact"` and takes the
`filenames` it reports. **No wildcard, no `head`, no `-print -quit`.** Roots are removed before each
case. Every archive member is inspected and counted, not only the first. Failure semantics are
covered by the committed tests — missing artifact, empty artifact, non-archive artifact each fail
loudly, and a marker placed in a **late** archive member is still found.

**`P3R-01`: REMAINS FIXED.**

## 8. New findings

### `P3R-15` — IMPORTANT — a conditional branch to a function symbol escapes the checker's model

**Reachable: NO in the child closure today; latent in the load-bearing gate. Realisable: YES —
rustc emits this instruction form in this crate.**

`TRANSFER` enumerates `call`/`jmp` mnemonics only. The module docstring justifies the omission
explicitly:

> A conditional branch cannot [move control out of the current function]: rustc emits one only to a
> label of the same function.

**That premise is false, and this crate's own emitted code disproves it.** In freshly generated
release-codegen assembly, LLVM emitted a conditional **tail branch to a function symbol**:

```asm
_ZN4core3ptr131drop_in_place$LT$core..result..Result$LT$…Report$C$…ReportRejection$GT$$GT$…E:
        xorl    %eax, %eax
        cmpq    (%rdi), %rax
        jno     _ZN4core3ptr56drop_in_place$LT$helm_launch..backend..tests..Report$GT$17hde2e620c…E
```

A line of that shape matches neither `TRANSFER` nor `LABEL`, so it is appended to the body and then
**discarded without record** by `edges()` — not an edge, not indirect, not unresolved. This is the
`P3R-11` defect one mnemonic family over, and it contradicts the checker's own promise that
"zero matches" can never mean "the parser found nothing".

Demonstrated directly against the committed checker, with the branch placed in `child_main`:

| Instruction in `child_main` | Gate verdict | external | indirect | unresolved |
|---|---|---|---|---|
| `jno memcpy@PLT` — the real LLVM form | **PASSED** | 0 | 0 | 0 |
| `js __rust_alloc` | **PASSED** | 0 | 0 | 0 |
| `je core::panicking::panic` | **PASSED** | 0 | 0 | 0 |
| `jne *%rax` (indirect conditional) | **PASSED** | 0 | 0 | 0 |
| `loop malloc@PLT` | **PASSED** | 0 | 0 | 0 |
| control: `jmp memcpy@PLT` | FAILED | 1 | 0 | 0 |

A tail branch out of the child window straight to `memcpy` is reported as a clean closure.

Bounded today: across all three freshly generated profiles, the number of escaping conditional
branches **inside the child closure is 0** (debug 0, release codegen 0, injection 0). File-wide
there is exactly **1**, in test-type drop glue outside the closure. So no current evidence is
wrong — every closure result in section 6 stands.

Rated IMPORTANT by parity with `P3R-11`, which the owner accepted as IMPORTANT at the same
reachability (`NO`, latent) and on weaker evidence: that review found **zero** such instructions
anywhere, whereas here the compiler demonstrably emits the form for this crate. The gate is the
sole evidence for the corrected `P3R-02` property and its durability depends on failing closed.
The owner may re-rate it MINOR on the strength of the zero-in-closure result; this review is not
entitled to close it against the explicit fail-closed instruction.

**Not fixed here: this review makes no code change.** The shape of a fix — recorded for the owner,
not applied — is to treat every `j*`/`loop*` mnemonic as a possible escaping transfer, classify a
`.L` target the body defines as intra-function and anything else as an edge or unresolved, and add
the missing unknown-**mnemonic** negative test.

### `P3R-16` — MINOR — three scalar keys are at-most-once, not exactly-once as documented

[`REPORT_SCALARS`](../../crates/helm-launch/src/backend/tests.rs#L569) is documented as "the scalar
keys the schema defines, each required **exactly once**". For nine of the twelve that holds. For
`sig_blk`, `sig_ign` and `sig_cgt` it does not: no `Report` field reads them, so `scalar(key)?` is
never called for them and `Missing` can never be raised. Removing all three from an otherwise
well-formed report **parses successfully**, confirmed by execution.

The practical guarantee that matters is intact — the three keys *are* schema members, so a producer
that renamed one would be refused loudly as `UnknownKey`, which is the `P3R-10` defect class — and
no assertion depends on their values. If a future change adds a field reading one of them, presence
checking becomes automatic. The defect is that the docstring overstates the contract. Test hygiene
and documentation, not a product defect; the committed suite's missing-key loop also omits these
three.

## 9. Carried forward, not re-examined

Unchanged by this correction, carried at their dispositioned severity, **not promoted and not
fixed**: `P3R-03`, `P3R-04`, `P3R-05`, `P3R-06`, `P3R-07`, `P3R-08`, `P3R-09`, `P3R-12` (MINOR);
`P3R-13`, `P3R-14` (BACKLOG_NONBLOCKING). `P3R-00` remains closed by `4c834415`. Runtime gates
`P3R-G1` and `P3R-G2` remain `GATE_PENDING`.

## 10. Local validation executed

| Command | Result |
|---|---|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | ✅ |
| `cargo clippy -p helm-launch --all-targets --all-features --locked --target x86_64-unknown-linux-gnu -- -D warnings` | ✅ **type-checks the corrected backend tests** |
| `cargo test --workspace --locked` | ✅ |
| `cargo test -p helm-launch --locked` | ✅ |
| `cargo test -p helm-launch --locked --features test-fault-injection` | ✅ |
| `cargo build -p helm-launch --release --locked` | ✅ |
| `python -m unittest discover -s tools/tests` | ✅ 820 tests, 74 skipped |
| `python tools/validate_docs.py` | ✅ |
| `git diff --check` | ✅ |
| fresh debug / release-codegen / injection child closures | ✅ all PASSED |
| fresh injection proof | ✅ PASSED |
| release-library backend-absence contrast | ✅ `child_main` absent |

The backend suite is `cfg`-gated to Linux x86_64 and therefore **does not execute on this host**;
cross-`clippy` type-checks it without running it.

**`P3 LINUX RUNTIME VALIDATION: PENDING PUBLICATION CI`.**
**`P3 STRACE CHILD-WINDOW VALIDATION: PENDING PUBLICATION CI`.**

These are the expected `GATE_PENDING` items, not review failures.

## 11. Findings table

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P3R-10` | — | Linux backend test contract | — | `tgid` and `pgid_is_self` are two mandatory, separately parsed, separately asserted facts; `Report` has no `Default`; unknown/missing/duplicate/malformed all refused; producer self-test proves `tgid` against a harness-observed pid and `pgid_is_self` against a negative control | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-11` | — | machine-code gate | — | Model is function-escaping control transfer; indirect `call`/`jmp`, jump tables, unresolvable direct targets and foreign local labels all fail closed; resolvable direct tail jumps traverse as edges; differential reproduces the old fail-open and shows it closed | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-01` | — | CI proof / artifact selection | — | Tool byte-identical; rerun with fresh roots, exact `compiler-artifact` selection, 128/11 members inspected, PRESENT/ABSENT as required | **REMAINS FIXED** |
| `P3R-02` | — | child closed world | — | 0 external, 0 indirect, 0 unresolved in freshly regenerated debug, release-codegen and injection closures | **REMAINS FIXED** |
| `P3R-15` | **IMPORTANT** | machine-code gate | **no** (latent; 0 in closure, 1 in file) | A conditional branch to a function symbol (`jno <symbol>`, which rustc **does** emit here) matches no `TRANSFER` pattern and is discarded without record; `jno memcpy@PLT` in `child_main` passes the gate | **OPEN — owner decision required before publication** |
| `P3R-16` | MINOR | test schema documentation | n/a | `sig_blk`, `sig_ign`, `sig_cgt` are documented "required exactly once" but are never presence-checked, because no `Report` field reads them | **OPEN, not promoted** |

## 12. Recommendation

**`P3 CANDIDATE REQUIRES OWNER REVIEW / CORRECTION BEFORE PUBLICATION.`**

Both corrections the owner ordered are genuinely and verifiably made, and the P3 safety case is
untouched — the backend is byte-identical, so `4c834415` remains valid for all of it. The single
obstacle is `P3R-15`: the fail-closed model the owner made load-bearing for `P3R-02` still has one
unclassified mnemonic family, and this rereview holds a real instance of that instruction form
emitted by rustc for this crate.

`P3R-15` needs an owner rating decision and, if upheld, a bounded checker fix plus the missing
unknown-mnemonic negative test. Nothing else in this correction blocks publication.

**P4 REMAINS NOT AUTHORISED. No Trial #4 is authorised. Trial #3 stays `MECHANISM_REJECTED`.**

**Next gate: OWNER REVIEW OF THE BOUNDED INDEPENDENT P3 FINDINGS.**

## 13. Classification

**`HELM_LAUNCH_P3_CORRECTION_REREVIEW_NEEDS_FIX`**
