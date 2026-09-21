# HELM-LAUNCH P3 — BOUNDED INDEPENDENT RE-REVIEW OF P3R-15

> # 🔎 BOUNDED INDEPENDENT RE-REVIEW OF `P3R-15` — NOT A FULL P3 UNSAFE REVIEW
>
> **Session provenance.** **THIS REVIEW SESSION AUTHORED OR MODIFIED NONE OF:**
> `5c577d45f62e2e3adc35a6c04a5ed0d8465a4366` (the bounded independent correction re-review that
> raised `P3R-15`), `ea803739ad74ca72e9d0a988659dc1fac7e1bd3a` (the owner disposition of that
> re-review) and `d07d48b5522e05e34dd419e2252bd44ead4a4d0a` (the `P3R-15` checker correction).
> It authored no part of the P3 implementation chain. It read those commits, re-derived the
> corrected evidence from source, from freshly emitted Linux x86_64 machine code and from fixtures
> and probes written in this session, and reached its own conclusions.
>
> Repository policy records normal commits under the owner identity and adds no agent attribution
> trailer, so `git log` cannot establish reviewer independence. The statement above is a
> session-provenance fact supplied by the repository owner, not an inference from commit metadata.

> **This document does NOT supersede
> [`HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-INDEPENDENT-UNSAFE-REVIEW.md)
> (`4c834415e8e0224b1eb1ce6c6546245cdfda0962`), the genuinely independent full P3 unsafe review,
> and it does NOT supersede
> [`HELM-LAUNCH-P3-CORRECTION-REREVIEW.md`](HELM-LAUNCH-P3-CORRECTION-REREVIEW.md)
> (`5c577d45f62e2e3adc35a6c04a5ed0d8465a4366`), the bounded independent re-review of the
> `P3R-10` / `P3R-11` correction.** Both remain authoritative for every area this correction did not
> touch: the unsafe confinement, the `clone3` ABI, the signal ABI, the borrowed `ChildPlan`, the
> closed child sequence, descriptor isolation, exec authority, direct-child cleanup, and the
> `P3R-10` report contract.
>
> **`4c834415` + `5c577d45` + this re-review together form the P3 pre-publication independent
> review record.** None of the three is complete alone.
>
> [`HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md) (`168fe133`)
> is an **AUTHOR SELF-REVIEW**, is **NOT INDEPENDENT**, and is not an approval artifact.

> **P1 ACCEPTED. P2 ACCEPTED. P3 AUTHORISED / CORRECTED CANDIDATE, NOT ACCEPTED. P4 AND P5 NOT
> AUTHORISED. TRIAL #3 STAYS `MECHANISM_REJECTED`. TRIAL #4 NOT AUTHORISED.**

## 0. Verdict

| Item | Result |
|---|---|
| BLOCKER | **0** |
| IMPORTANT | **0** |
| `P3R-15` (conditional branch fail-closedness) | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-01` (artifact selection / injection proof) | **REMAINS FIXED** |
| `P3R-02` (child closed world at machine level) | **REMAINS FIXED** |
| `P3R-10` (report contract) | **REMAINS FIXED** |
| `P3R-11` (indirect transfer fail-closedness) | **REMAINS FIXED** |
| Backend product semantics | **BYTE-UNCHANGED BY THE `P3R-15` CORRECTION** |
| MINOR | 3 new — `P3R-17`, `P3R-18`, `P3R-19`, none reachable in the gate's actual input; `P3R-03` … `P3R-09`, `P3R-12`, `P3R-16` carried forward unchanged |
| Classification | **`HELM_LAUNCH_P3_P3R15_REREVIEW_PASSED_READY_FOR_PUBLICATION_CI`** |

The correction is genuinely and decisively made, and this re-review establishes it from evidence it
produced itself rather than from the correction author's report. The checker now classifies every
function-escaping x86 control transfer against one explicit, enumerated vocabulary; a conditional
edge is traversed transitively exactly like a call; indirect transfers still fail closed with no
resolver added; and a recognisably control-flow mnemonic outside the vocabulary fails rather than
being read as ordinary code.

The decisive evidence is not synthetic. On **freshly regenerated** Linux x86_64 release-codegen
assembly the corrected parser finds the real `jno <function symbol>` that `P3R-15` was raised on,
records it as a call-graph edge, and **traverses it** — a closure rooted at its containing function
reaches `__rust_dealloc`, `_Unwind_Resume` and `core::option::unwrap_failed`, all of which the
previous checker could not have seen from that site. The same instruction form placed in
`child_main` now fails the gate. Inside the child closure itself, **29 (debug), 22
(release-codegen) and 47 (injection) conditional branches that the previous checker did not see at
all are now each accounted for as intra-function control flow**, with zero escaping to a function
symbol.

No memory-safety, authority or boundary defect was found. Nothing in this correction touches the P3
safety case: the whole `crates/` tree is byte-identical across it.

## 1. Starting state, verified

| Requirement | Observed | Result |
|---|---|---|
| Branch | `docs/helm-launch-architecture` | ✅ |
| Worktree | clean (`git status --short` empty) | ✅ |
| Local `HEAD` | `d07d48b5522e05e34dd419e2252bd44ead4a4d0a` | ✅ |
| `origin/docs/helm-launch-architecture` | `9fb0f8cabd5b7dd4f8df3ee5d15cf127702fcb5b` | ✅ |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` | ✅ |
| Branch relation | **11 ahead, 0 behind** | ✅ |

Exact local chain, verified commit by commit with `git rev-list --reverse`:

```text
9fb0f8ca  accepted P2 base, published
 └─ 7bb016f  docs(launch): authorise p3 unsafe backend            P3 authority
     └─ afe8922  feat(helm-launch): implement p3 child backend    original implementation
         └─ 168fe13  author self-review — NOT INDEPENDENT
             └─ ac823cd  owner disposition of author-review findings
                 └─ 672228b  P3R-01 / P3R-02 correction
                     └─ 4c83441  GENUINELY INDEPENDENT full P3 unsafe review
                         └─ 7b749b8  owner disposition of independent findings
                             └─ f144d30  P3R-10 / P3R-11 correction
                                 └─ 5c577d4  bounded independent correction re-review
                                     └─ ea80373  P3R-15 owner disposition
                                         └─ d07d48b  P3R-15 checker correction   ← HEAD
```

`origin` was fetched read-only. Nothing was pushed, amended, rebased, squashed or dispatched. No
implementation, test, checker or workflow file was modified by this review. No Trial history file
was touched; Trial #3 remains `MECHANISM_REJECTED`.

### 1.1 Lineage read

Read in full, line by line: [`AGENTS.md`](../../AGENTS.md),
[`tools/helm_launch_child_closure.py`](../../tools/helm_launch_child_closure.py) (676 lines) and
[`tools/tests/test_helm_launch_machine_proofs.py`](../../tools/tests/test_helm_launch_machine_proofs.py)
(535 lines) — the two files the correction changed, and the authority this review evaluates.

Read for the sections bearing on this bounded scope:
[`docs/adr/ADR-0024-launch-authority.md`](../adr/ADR-0024-launch-authority.md) (acceptance block,
sections D, E and the authority header), [`docs/DECISIONS.md`](../DECISIONS.md) (the `P3R-15`
disposition in full, including the required correction and bounded-scope clauses, plus the
`P3R-01`/`P3R-02`/`P3R-10`/`P3R-11` dispositions),
[`docs/PROJECT_STATE.md`](../PROJECT_STATE.md) (the current status block),
[`HELM-LAUNCH-P3-CORRECTION-REREVIEW.md`](HELM-LAUNCH-P3-CORRECTION-REREVIEW.md) (verdict,
scope, the `P3R-15` and `P3R-16` findings, the differential and the findings table), and
[`HELM-LAUNCH-PRODUCTIZATION-PLAN.md`](HELM-LAUNCH-PRODUCTIZATION-PLAN.md) /
[`HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md`](HELM-LAUNCH-P3-AUTHOR-UNSAFE-REVIEW.md) for the
referenced contract clauses. **Stated as a limitation:** ADR-0024 and the productization plan were
not re-read end to end, because both are proven byte-unchanged below and their accepted contract is
not under review in a bounded checker re-review.

## 2. Correction scope, verified against the owner's boundary

`ea803739` changes exactly two files and deletes nothing:

| File | Change |
|---|---|
| [`docs/DECISIONS.md`](../DECISIONS.md) | **+121 / −0** |
| [`docs/PROJECT_STATE.md`](../PROJECT_STATE.md) | **+60 / −0** |

`d07d48b5` changes exactly two files:

| File | Change |
|---|---|
| [`tools/helm_launch_child_closure.py`](../../tools/helm_launch_child_closure.py) | **+117 / −19** |
| [`tools/tests/test_helm_launch_machine_proofs.py`](../../tools/tests/test_helm_launch_machine_proofs.py) | **+180 / −0** |

`git diff --name-status 5c577d4..d07d48b` over the whole correction span returns those four files
and **nothing else**.

## 3. Backend product semantics — byte-unchanged proof

Git blob and tree identity compared across the whole correction span, `5c577d4` → `d07d48b`, so both
the disposition and the checker correction are covered:

| Path | `5c577d4` object | `d07d48b` object | Result |
|---|---|---|---|
| `crates/` (whole tree) | `aeea0f7a52a1cb5be5be32ae00bb15cf383883ee` | same | **IDENTICAL** |
| `crates/helm-launch/src/` (whole tree) | `c3fe76629f9bc5890dff439dd4ac350e92d6144e` | same | **IDENTICAL** |
| `crates/helm-launch/src/backend/` | `fdff1be9d1586a897141dd421b25f146ac2c81f1` | same | **IDENTICAL** |
| `crates/helm-launch/src/authority.rs` | `3a877d61faf44c81460031b2bf92d07d9f6f4927` | same | **IDENTICAL** |
| `crates/helm-launch/src/lib.rs` | `c3b3b5243e404eba97b3c276025dd3334e8928a0` | same | **IDENTICAL** |
| `crates/helm-launch/Cargo.toml` | `1254408f1b050365766d6afd7a9d86538bd4654f` | same | **IDENTICAL** |
| `Cargo.lock` | `b2df4153d01ce87f1beee8715cb7eb0cdaa63a11` | same | **IDENTICAL** |
| `.github/workflows/` | `3945bec5dc77cb35e87d98626e1b96907c587419` | same | **IDENTICAL** |
| `docs/adr/ADR-0024-launch-authority.md` | `97a76f9ca95a71859787ac0384a5e2971012a1fd` | same | **IDENTICAL** |
| `docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md` | `c366ad7f1f198668c9d77625ee2de29575d3e847` | same | **IDENTICAL** |

The whole `crates/` tree is one identical tree object, which covers every backend product file,
every crate test and the public API in a single hash rather than file by file.

> **`BACKEND PRODUCT SEMANTICS: BYTE-UNCHANGED BY P3R-15 CORRECTION.`**

The full unsafe review at `4c834415` therefore remains valid for all of it, and no full P3 unsafe
review is required again.

**Public `launch()`: ABSENT.** `grep -rn "pub fn launch" crates/` returns nothing.
[`lib.rs`](../../crates/helm-launch/src/lib.rs) declares `mod backend;` privately and re-exports only
the authority, error, model, plan and receipt items.

**Process-group sweep: ABSENT.** The only pid-valued group call is the parent's
`setpgid(child, child)`, whose success is recorded as `pgid_is_self` and, as
[`spawn.rs`](../../crates/helm-launch/src/backend/spawn.rs) states, "is recorded for a future P4
sweep and is used for nothing in P3". `killpg` appears in the repository exactly once — inside
[`p3_boundary.rs`](../../crates/helm-launch/tests/p3_boundary.rs) as a token the boundary suite
**forbids**. This matches ADR-0024's authority header: the P3 authorisation "adds **no public
execution API**, no process-group sweep and no host privilege acquisition".

## 4. The control-transfer vocabulary, extracted mechanically

The vocabulary was extracted from the committed code two independent ways — by importing the module
and measuring the `frozenset` objects, and by parsing the file's AST and counting the string
literals in each `frozenset({…})` expression, which detects a duplicate the set would silently
absorb. The correction handoff's count was **not** trusted.

| Group | Source | Source literals | Unique | Silent duplicates |
|---|---|---|---|---|
| `CALL_MNEMONICS` | [line 155](../../tools/helm_launch_child_closure.py#L155) | 5 | **5** | 0 |
| `UNCONDITIONAL_JUMP_MNEMONICS` | [line 157](../../tools/helm_launch_child_closure.py#L157) | 5 | **5** | 0 |
| `CONDITIONAL_JUMP_MNEMONICS` | [line 162](../../tools/helm_launch_child_closure.py#L162) | 33 | **33** | 0 |
| `LOOP_MNEMONICS` | [line 171](../../tools/helm_launch_child_closure.py#L171) | 5 | **5** | 0 |

> ```text
> UNIQUE CALL MNEMONICS:                      5
> UNIQUE UNCONDITIONAL JUMP MNEMONICS:        5
> UNIQUE CONDITIONAL/JCC MNEMONICS:          33
> UNIQUE LOOP-BRANCH MNEMONICS:               5
> TOTAL UNIQUE CONTROL-TRANSFER MNEMONICS:   48
> ```

`len(ESCAPING_MNEMONICS)` is **48**, exactly the sum of the four group sizes, so no mnemonic appears
in two groups; pairwise intersection of all four groups is empty.

- call: `call`, `callq`, `calll`, `callw`, `lcall`
- unconditional jump: `jmp`, `jmpq`, `jmpl`, `jmpw`, `ljmp`
- conditional: `ja`, `jae`, `jb`, `jbe`, `jc`, `je`, `jg`, `jge`, `jl`, `jle`, `jna`, `jnae`, `jnb`,
  `jnbe`, `jnc`, `jne`, `jng`, `jnge`, `jnl`, `jnle`, `jno`, `jnp`, `jns`, `jnz`, `jo`, `jp`, `jpe`,
  `jpo`, `js`, `jz`, `jcxz`, `jecxz`, `jrcxz`
- loop: `loop`, `loope`, `loopz`, `loopne`, `loopnz`

### 4.1 Reconciling the handoff's reported "46"

The owner's instruction reports the correction handoff as claiming **46** canonical branch
mnemonics while listing group counts of 5 + 5 + 33 + 5, which sum to **48**.

**The code's actual unique vocabulary is 48.** The arithmetic in the handoff is what is wrong, not
the code, and **no missing branch family accounts for the difference** — the difference is 48 − 46 = 2
with no group short by two and no alias collapsed.

**No committed artifact states 46.** `grep` over `docs/DECISIONS.md` and `docs/PROJECT_STATE.md`
finds no mnemonic count at all. The `d07d48b5` commit message states the vocabulary as "call,
callq, calll, callw, lcall; jmp, jmpq, jmpl, jmpw, ljmp; the canonical thirty-three Jcc mnemonics
including the three counter-register branches; and loop, loope, loopz, loopne, loopnz", and later
"the thirty-eight conditional and loop mnemonics" — **5, 5, 33, 5 and 38, every one of which this
review independently confirms**. The committed test
[`test_the_branch_vocabulary_is_the_canonical_one`](../../tools/tests/test_helm_launch_machine_proofs.py#L313)
pins `len(CONDITIONAL_JUMP_MNEMONICS) == 33` and the exact `LOOP_MNEMONICS` set.

The "46" is therefore an **out-of-band handoff arithmetic slip with no repository consequence**, not
a defect in the committed correction. It is recorded here for the record and carries no finding id,
because nothing in the repository is wrong.

### 4.2 Required conditional coverage, verified against the owner's list

Every one of the 38 mnemonics the owner's section 8 enumerated — the 30 flag-condition `Jcc`
spellings and their documented aliases, the three counter-register branches `jcxz` / `jecxz` /
`jrcxz`, and `loop` / `loope` / `loopz` / `loopne` / `loopnz` — is present in the committed
vocabulary. Aliases (`je`/`jz`, `jne`/`jnz`, `jnbe`/`ja`, `jpe`/`jp`, `jpo`/`jnp`, …) map to the same
classification rule rather than to duplicate internal representations, which is what the owner
allowed: the rule is per-transfer, not per-condition, so an alias needs no separate path.

## 5. The direct control-transfer rule, verified in code

[`Assembly.edges`](../../tools/helm_launch_child_closure.py#L286) applies one rule to **every**
mnemonic in `ESCAPING_MNEMONICS`, with no branch on family after the vocabulary test. Read against
the owner's section 9:

| Operand shape | Classification | Bucket |
|---|---|---|
| `.Lxxx` the body itself defines | intra-function control flow | `local` counter |
| `.Lxxx` the body does not define | escape with unknown target | `unresolved` → **FAIL** |
| `foo`, `foo@PLT` resolving to another body | call-graph edge, pushed onto `pending` | `targets` → traversed |
| `foo`, `foo@PLT` resolving to no body | external reference | `external_edges` → **FAIL** |
| `*foo@GOTPCREL(%rip)`, `*foo(%rip)` | named edge through the GOT | `targets` → traversed |
| any other `*…` form | indirect | `indirect` → **FAIL** |
| any other shape | operand form not understood | `unresolved` → **FAIL** |
| no operand at all | transfer with unknown target | `unresolved` → **FAIL** |

The traversal itself is in [`closure`](../../tools/helm_launch_child_closure.py#L370): for every
entry of `edges.targets`, a target defined in the unit is appended to `internal_edges` **and** to
`pending`, so the walk continues through it. Because `targets` is populated identically for a
`callq`, a tail `jmp`, a `Jcc` and a `loop`, **conditionality cannot make an escaping function edge
ignorable, and cannot make it recorded-but-unvisited**. This was then verified by execution rather
than left as a reading (section 6).

Every entry of `indirect`, `unresolved` and `unsupported` reaching the closure is appended to
`problems` in [`check`](../../tools/helm_launch_child_closure.py#L476), each with its own message,
so a failure names which bucket it came from.

## 6. `P3R-15` reproduction, independently executed

Fixtures for this section were **written in this session**, not imported from the correction's own
test helpers, so a fixture that flatters the implementation cannot be inherited. Each scene is a
`child_main` with a real prologue, an out-of-line syscall shim, a local-label branch, and a separate
witness function reaching `memset` so the positive control has something to find.

| Probe | Result | Reason reported |
|---|---|---|
| **`jno memcpy@PLT` in `child_main`** — the exact `P3R-15` defect | **FAIL** | `child_main -> memcpy [libc string/memory helper]` |
| `jne malloc@PLT` | **FAIL** | `child_main -> malloc [allocator]` |
| `je <internal helper>`, helper clean | **PASS** | helper **in closure**, `(child_main, helper)` internal edge recorded |
| `jrcxz free@PLT` | **FAIL** | `child_main -> free [allocator]` |
| `loop memmove@PLT` | **FAIL** | `child_main -> memmove [libc string/memory helper]` |

> **`jno memcpy@PLT` FAILS FOR THE INTENDED REASON** — an external forbidden memory/runtime helper
> edge, categorised as a libc string/memory helper, **not** an unrelated parser failure. The
> `indirect`, `unresolved` and `unsupported` buckets are all empty for that probe; the single
> problem is the external edge.

### 6.1 Conditional-edge transitivity

The owner's section 11 asks specifically whether a conditional edge is *traversed* or merely
recorded. Two chains, both written here:

| Chain | Result | Where the forbidden edge was found |
|---|---|---|
| `child_main --jg--> helper --callq--> __rust_alloc@PLT` | **FAIL** | at `helper`, which is **in the closure** |
| `child_main --jbe--> helper --jle--> deeper --jmp--> _Unwind_Resume@PLT` | **FAIL** | at `deeper`, three hops in, two of them conditional |

The second chain is the strong form: the closure grew to four functions
(`child_main`, `syscall6`, `helper`, `deeper`), and the failure is reported against `deeper`, which
can only be reached by following a conditional edge and then a second conditional edge. A checker
that recorded a conditional edge without traversing it would have reported zero problems here.

### 6.2 Indirect transfers — `P3R-11` behaviour preserved

Ten spellings, all written here, all required to fail closed with no resolver:

| Spelling | Result | Bucket |
|---|---|---|
| `callq *%rax` | **FAIL** | `indirect` |
| `call *%r11` | **FAIL** | `indirect` |
| `jmp *%rax` | **FAIL** | `indirect` |
| `jmpq *%rax` | **FAIL** | `indirect` |
| `callq *(%rdi)` | **FAIL** | `indirect` |
| `jmpq *16(%rsp)` | **FAIL** | `indirect` |
| `jmpq *.LJTI0_0(,%rax,8)` (jump table) | **FAIL** | `indirect` |
| `jmpq *(%rax,%rcx,8)` (jump table, no label) | **FAIL** | `indirect` |
| `notrack jmpq *%r11` | **FAIL** | `indirect` |
| `bnd callq *%rax` | **FAIL** | `indirect` |

Conditional indirect forms fail the same way: `jne *%rax` and `jno *(%rbx)` both land in `indirect`.
**No resolver for an indirect target exists in the corrected checker** — the only code path for an
operand beginning `*` that is not the `(%rip)`-named GOT form is `indirect.append(...)`.

### 6.3 Unsupported branch-like mnemonics, and the terminal-form boundary

[`is_branch_like`](../../tools/helm_launch_child_closure.py#L194) is documented in the code as "the
conservative backstop, **not** the authority", and the code matches the documentation: it is
consulted only in the `mnemonic not in ESCAPING_MNEMONICS` arm, and its only effect is to append to
`unsupported`, which fails the gate.

| Probe | Result |
|---|---|
| `jmpabs $memcpy` (APX form, absent from the vocabulary) | **FAIL**, `unsupported` = 1 |
| `jfoo memcpy` | **FAIL**, `unsupported` = 1 |
| `jmpx %rax` | **FAIL**, `unsupported` = 1 |
| `loopz2 .LBB7_9` | **FAIL**, `unsupported` = 1 |
| `calll2 malloc` | **FAIL**, `unsupported` = 1 |

"Not in the known table ⇒ ordinary instruction" is therefore not reachable for anything that looks
like a branch. The reverse direction holds too — the backstop does not swallow normal code:

| Probe | Result |
|---|---|
| `retq`, `ret`, `ud2`, `int3`, `hlt` | **PASS**, `unsupported` = 0 |
| `leave`, `nop`, `lfence`, `cpuid`, `repz retq` | **PASS**, `unsupported` = 0 |
| `movq`, `cmpq`, `leaq`, `lock cmpxchgq` (committed negative control) | **PASS**, `unsupported` = 0 |

The docstring's statement of the boundary is **accurate**: "`ret`, `ud2`, `int3` and `hlt` end the
current flow without naming another function, so they extend no closure." It also states the honest
limit of any mnemonic-level model — a transfer hand-assembled out of `push`/`ret` would be invisible
— and closes it by reference to a **separate committed gate** rather than by assertion. That
reference checks out: [`p3_boundary.rs`](../../crates/helm-launch/tests/p3_boundary.rs#L410) asserts
assembly exists only in `src/backend/syscall.rs`, that exactly one `asm!(` invocation exists, and
that its template is literally `asm!("syscall",` — a shim containing no branch at all. That suite
runs on all three platforms.

No broader P3 contract is invented here, per the owner's section 13.

## 7. Differential against the pre-P3R-15 checker

### 7.1 The author's own new cases, run against the old checker

The correction's test file was copied unchanged next to the checker **as of `5c577d4`** and executed
there. All 37 methods ran:

- **11 unique test methods fail or error**, and all 11 are exactly the 11 methods the correction
  added — `test_the_p3r15_reproduction_fails`, `test_the_branch_vocabulary_is_the_canonical_one`,
  `test_every_conditional_branch_to_a_forbidden_external_fails`,
  `test_every_conditional_branch_to_an_own_label_is_intra_function`,
  `test_a_conditional_branch_to_an_internal_helper_is_traversed`,
  `test_a_clean_conditional_branch_to_an_internal_helper_passes`,
  `test_a_conditional_branch_keeps_every_other_fail_closed_rule`,
  `test_a_conditional_branch_through_the_got_resolves_to_its_symbol`,
  `test_a_prefixed_conditional_branch_is_still_classified`,
  `test_an_unsupported_branch_mnemonic_fails_closed`,
  `test_an_ordinary_instruction_is_not_a_branch`.
- **All 26 prior methods pass unchanged on the old checker**, and all 37 pass on the new one.

The new tests therefore genuinely expose the old fail-open, and no prior test became dependent on an
unrelated new failure.

**No prior machine-proof negative is weakened.** The test file's diff is a **pure addition**:
`git diff` from `5c577d4` to `d07d48b` over that file shows **+180 / −0**, with **zero** deleted or
modified lines, and set comparison of `def test_*` names shows all 26 prior names retained and 11
added.

### 7.2 This session's own fixtures, old checker versus corrected

Six probes written here, run against both versions, to avoid relying on the author's fixtures for
the load-bearing differential:

| Probe | `5c577d4` checker | `d07d48b` checker |
|---|---|---|
| `jno memcpy@PLT` — **the `P3R-15` defect** | **PASS** (silently discarded) | **FAIL** |
| `je malloc@PLT` | **PASS** | **FAIL** |
| `jrcxz __rust_dealloc` | **PASS** | **FAIL** |
| `loopne free@PLT` | **PASS** | **FAIL** |
| `jne .LBB404_1` (another function's local label) | **PASS** | **FAIL** |
| `jno *%rdx` (conditional indirect) | **PASS** | **FAIL** |

Six fail-open holes, all closed. The original defect is reproduced on the old checker and closed on
the new one, from fixtures the correction author never saw.

### 7.3 The differential on real compiler output

The strongest differential available: both checker versions run over the **identical, freshly
generated** assembly files, so the comparison is on rustc's real output rather than on synthetic
lines.

| Profile | Checker | closure | internal edges | **local branches** | ext | ind | unres | unsup |
|---|---|---|---|---|---|---|---|---|
| debug / test | `5c577d4` | 9 | 12 | **33** | 0 | 0 | 0 | 0 |
| debug / test | `d07d48b` | 9 | 12 | **62** | 0 | 0 | 0 | 0 |
| release codegen | `5c577d4` | 2 | 1 | **3** | 0 | 0 | 0 | 0 |
| release codegen | `d07d48b` | 2 | 1 | **25** | 0 | 0 | 0 | 0 |
| debug + injection | `5c577d4` | 10 | 14 | **36** | 0 | 0 | 0 | 0 |
| debug + injection | `d07d48b` | 10 | 14 | **83** | 0 | 0 | 0 | 0 |

Two facts follow, and both matter:

1. **Closure membership and every failure bucket are unchanged.** The correction introduces no new
   false failure and does not alter which functions the child reaches, so the previously reported
   clean result is preserved rather than re-litigated.
2. **The `local branches` counter jumps by +29, +22 and +47.** Those are precisely the conditional
   branches *inside the child closure* that the previous checker discarded without record. They were
   invisible; they are now each classified.

## 8. Every conditional branch inside the child closure, accounted for

The owner's section 18 requires that each conditional branch reachable inside the child closure be
accounted for as local CFG or as a known traversed function edge. Counted directly over the closure
bodies:

| Profile | conditional/loop branches in closure | intra-function CFG | traversed function edge | **unaccounted** |
|---|---|---|---|---|
| debug / test | 29 | **29** | 0 | **0** |
| release codegen | 22 | **22** | 0 | **0** |
| debug + injection | 47 | **47** | 0 | **0** |

Closure mnemonic census — debug: `callq` 70, `jmp` 33, `jne` 20, `je` 5, `jl` 2, `ja` 1, `jbe` 1.
Release codegen: `callq` 9, `je` 7, `jae` 12, `jmp` 3, `ja` 2, `jb` 1. Injection: `callq` 88,
`jmp` 36, `je` 23, `jne` 20, `jl` 2, `ja` 1, `jbe` 1. The `local_branches` counter reconciles
exactly: 62 = 29 + 33, 25 = 22 + 3, 83 = 47 + 36.

## 9. The real rustc conditional function edge

Regenerated fresh Linux x86_64 assembly with the crate's already-installed
`x86_64-unknown-linux-gnu` std target. **No WSL Rust toolchain was installed.** The
release-codegen profile still emits the instruction form `P3R-15` was raised on:

```text
in   _ZN4core3ptr131drop_in_place$LT$core..result..Result$LT$helm_launch..backend..tests..Report$C$
     helm_launch..backend..tests..ReportRejection$GT$$GT$17hb62a47ac0e5c9a14E

     jno  _ZN4core3ptr56drop_in_place$LT$helm_launch..backend..tests..Report$GT$17hde2e620c4e652740E
```

A conditional tail branch to another function, target defined in the same unit. It is the only such
transfer in the whole file: of 7 326 conditional transfers in the release-codegen profile, 7 325 are
intra-function and **this one is a function edge**.

**Recognised:** `edges()` on the containing body returns that symbol in `targets`, with
`local` = 2, `indirect` = 0, `unresolved` = 0, `unsupported` = 0.

**Traversed, not merely recorded:** a closure rooted at the containing function has **3 members**,
**includes the `jno` target**, records the `(containing, target)` pair in `internal_edges`, and
through it reaches **5 forbidden external edges** — `__rust_dealloc` (allocator, three sites),
`_Unwind_Resume` (unwinder) and `core::option::unwrap_failed` (panic runtime). Under the previous
checker none of those was reachable from that site, because the edge that leads to them did not
exist in its model.

**It lies outside the child closure** (`child_main`'s closure is 2 functions and does not contain the
containing function), so it correctly does not fail the gate.

> **`REAL/SYNTHETIC CONDITIONAL FUNCTION EDGE: RECOGNISED AND TRAVERSED.`** A real one was found in
> this fresh build, so the synthetic fallback the owner permitted was not needed.

## 10. File-wide control-transfer inventory

Fresh debug, release-codegen and injection assembly, aggregated through the checker's own
`Assembly.edges` so the numbers are the checker's classification and not a re-implementation.

| | debug / test | release codegen | debug + injection |
|---|---|---|---|
| function bodies | 5 769 | 723 | 5 794 |
| **TOTAL MODELLED CONTROL TRANSFERS** | **37 071** | **16 712** | **37 347** |
|  · call family | 16 097 | 5 677 | 16 222 |
|  · unconditional jump family | 16 021 | 3 709 | 16 136 |
| **TOTAL CONDITIONAL/JCC/LOOP TRANSFERS** | **4 953** | **7 326** | **4 989** |
|  · intra-function CFG | 4 953 | 7 325 | 4 989 |
|  · **to a function symbol** | **0** | **1** | **0** |
| **TOTAL INDIRECT TRANSFERS** | **1 809** | **450** | **1 821** |
| **TOTAL UNSUPPORTED BRANCH-LIKE** | **0** | **0** | **0** |
| **TOTAL UNRESOLVED** | **0** | **0** | **0** |
| distinct branch mnemonics observed | 16 | 18 | 16 |

The **4 953** and **7 326** conditional transfers, and the **5 769** and **723** body counts,
reproduce the correction's reported figures exactly, from a build made in this session. The previous
parser modelled **none** of those conditional transfers.

The load-bearing property is met: **real conditional transfers are no longer invisible**, file-wide
`unresolved` is 0 across 5 769 and 723 function bodies, and file-wide `unsupported` is 0 — so the
48-mnemonic model covers everything rustc emitted here without ever falling back on the backstop.
Indirect transfers in unrelated functions outside the child closure are expected and acceptable; the
point is that the parser sees them.

Hygiene checks over the same three files: **zero** uppercase mnemonic tokens, and **zero**
occurrences of `int`, `into`, `int1`, `xbegin`, `xabort`, `iret`, `iretq`, `sysenter`, `sysexit`,
`sysret`, `bound` or `jmpabs`. This bounds `P3R-17` and `P3R-18` below to unreachable in the gate's
actual input.

## 11. Fresh machine closures

### 11.1 Debug / test profile

```text
root     _ZN11helm_launch7backend5child10child_main17haee81f8dae299a2eE
closure  9 functions      internal edges 12      local branches 62 (29 conditional + 33 jmp)
syscalls 1 instruction, shim OUT-OF-LINE (exactly one, in the reviewed shim, none elsewhere)
external edges 0   indirect 0   unresolved 0   unsupported 0
positive control 4 witness functions; named control reaches 143 external helpers
CHILD CLOSURE CHECK PASSED
```

Closure: `child_main`, `syscall6`, `is_error`, `errno_of`, `fail`, and four `core` numeric/result
helpers (`i64::unsigned_abs`, `TryFrom<u64> for i32`, `Result::unwrap_or_default`,
`i32::to_le_bytes`) — all defined in the same unit, none an external runtime helper. Identical
membership to the two prior independent reviews, re-derived rather than copied.

### 11.2 Release codegen

```text
root     _ZN11helm_launch7backend5child10child_main17haee81f8dae299a2eE
closure  2 functions      internal edges 1       local branches 25 (22 conditional + 3 jmp)
syscalls 18 instructions, shim INLINED  (child_main 16, fail 2)
root body 217 real instructions
external edges 0   indirect 0   unresolved 0   unsupported 0
positive control 4 witness functions
CHILD CLOSURE CHECK PASSED
```

Generated by the already-established probative mechanism —
`--profile test --rustc-flag=-Copt-level=3 --rustc-flag=-Cdebug-assertions=off` — the same one
[`helm-launch.yml`](../../.github/workflows/helm-launch.yml#L143) uses, **not** a plain release
library build.

> **`RELEASE CHILD BACKEND ACTUALLY INSTANTIATED: YES`** — the root resolves uniquely, its body is
> 217 real instructions, and it issues 16 inline `syscall` instructions with 2 more in `fail`.
>
> **`RELEASE PROOF NOT DCE-ONLY: YES`** — proven by contrast, executed here. A plain
> `cargo rustc -p helm-launch --lib --release --all-features --target x86_64-unknown-linux-gnu`
> emits assembly in which `backend5child10child_main` occurs **0** times (eliminated, since P3 adds
> no public consumer), while the release-codegen path emits it **7** times with the body above. The
> clean result therefore describes real optimised child code, not an empty artifact. Ordinary DCE of
> the private release library is **not** used as evidence anywhere in this review.

### 11.3 Injection-enabled

```text
root     _ZN11helm_launch7backend5child10child_main17h41c688be4c50452cE
closure  10 functions     internal edges 14      local branches 83 (47 conditional + 36 jmp)
syscalls 1 instruction, shim OUT-OF-LINE
external edges 0   forbidden 0   indirect 0   unresolved 0   unsupported 0
CHILD CLOSURE CHECK PASSED
```

No regression: the closure grows by one function relative to debug and stays clean on every bucket.
The `P3R-15` correction does not make the fault-injection closure proof weaker — it strengthens it,
by newly accounting for 47 conditional branches inside it.

## 12. The committed proof suite

`python -m unittest tools.tests.test_helm_launch_machine_proofs -v` — **37 tests, OK**, no skips.
The suite was read in full rather than counted, and the count alone is not accepted as evidence.

Two of the 37 are **table-driven over the entire conditional and loop vocabulary**, giving
38 × 2 = **76 sub-cases** in both required directions:

| Test | Direction | Cases |
|---|---|---|
| [`test_every_conditional_branch_to_a_forbidden_external_fails`](../../tools/tests/test_helm_launch_machine_proofs.py#L325) | forbidden escaping target → failure, asserted on the **category string** `libc string/memory helper` | 38 |
| [`test_every_conditional_branch_to_an_own_label_is_intra_function`](../../tools/tests/test_helm_launch_machine_proofs.py#L345) | own-function label → recognised intra-function CFG, asserted on `edges.local == 2` **and** `problems == []` **and** the label absent from `targets` | 38 |

So no mnemonic is modelled in one direction only, and the positive direction is asserted strongly
enough that "fail on every conditional branch" could not satisfy it.

Coverage against the owner's section 14 list, each verified by reading the assertion and not the
name:

| Required case | Present | How it is asserted |
|---|---|---|
| `jno memcpy@PLT` | ✅ [L301](../../tools/tests/test_helm_launch_machine_proofs.py#L301) | the `(memcpy, libc string/memory helper)` pair is in `external_edges` |
| direct conditional internal-helper traversal | ✅ [L362](../../tools/tests/test_helm_launch_machine_proofs.py#L362) | helper in `closure` **and** `(child, helper)` in `internal_edges` **and** `malloc` found through it |
| indirect jump | ✅ | 6 spellings, each asserting `len(indirect_transfer_sites) == 1` |
| indirect call | ✅ | asserts `len(indirect_transfer_sites) == 1` |
| unresolved direct target | ✅ | foreign local label, `memcpy+0x20`, absolute numeric, missing operand, each asserting its bucket is exactly 1 |
| unsupported branch-like mnemonic | ✅ [L429](../../tools/tests/test_helm_launch_machine_proofs.py#L429) | 4 spellings, each asserting `len(unsupported_transfer_sites) == 1` and the message text |
| ordinary non-branch control | ✅ [L451](../../tools/tests/test_helm_launch_machine_proofs.py#L451) | `unsupported_transfer_sites == []` and `problems == []` over real instruction shapes including `ud2` |

Each added case asserts the **specific bucket or category**, not merely that `problems` is
non-empty, so none can pass because of an unrelated earlier parser error. The gap the previous
re-review recorded — "there is no test for an unknown control-transfer **mnemonic**" — is closed.

**All 26 prior proof tests remain semantically intact**, established structurally in section 7.1: the
file's diff deletes and modifies nothing.

## 13. Prior finding regression checks

Minimal, as the owner's section 21 bounds them. No unaffected unsafe or backend reasoning was
reopened.

### `P3R-01` — REMAINS FIXED

`python tools/helm_launch_injection_proof.py` rerun with fresh roots, on a tool that is byte-identical
across this correction:

- Exact artifact selection from cargo's own `compiler-artifact` record — **4** named artifacts, no
  wildcard.
- debug + `test-fault-injection`: **128** and **1** archive members inspected, marker **PRESENT**,
  with hits in `lib.rmeta` and in a real `.rcgu.o` object member (`marker=True, symbol=True`).
- release + `--all-features`: **11** and **1** members inspected, marker **ABSENT**.
- `INJECTION PROOF PASSED`.

### `P3R-02` — REMAINS FIXED

Regenerated debug, release-codegen and injection child closures each report **0 forbidden runtime
edges**, 0 external, 0 indirect, 0 unresolved and 0 unsupported (section 11). The release proof is
probative, not DCE-only.

### `P3R-10` — REMAINS FIXED

The report contract lives in byte-identical product test code, and the structural fix is intact by
inspection: [`REPORT_SCALARS`](../../crates/helm-launch/src/backend/tests.rs#L569) is 12 scalars,
plus 1 optional and 3 repeated keys — **sixteen** schema keys, matching the emitter;
[`struct Report`](../../crates/helm-launch/src/backend/tests.rs#L528) derives `Debug, PartialEq, Eq`
and **no `Default`**, so no missing value can leave a zero behind; `tgid` ("the executed image's own
process identity … **Not** a group id") and `pgid_is_self` are separate fields, separately parsed and
separately asserted. `UnknownKey`, `Missing`, `Duplicate` and `Malformed` are all distinct refusals
and a refused report is never partially used. The suite's portable cases pass; the Linux-gated cases
type-check under cross-`clippy`.

### `P3R-11` — REMAINS FIXED

Independently re-established in sections 6.2 and 7: indirect `call` and `jmp` fail closed across ten
spellings including jump tables and prefixed forms; a resolvable direct tail `jmp` to an internal
helper is traversed as an edge; an unresolved direct target fails closed. File-wide indirect
visibility is unchanged between `5c577d4` and `d07d48b` on identical assembly (1 809 / 450 / 1 821),
confirming the `P3R-11` fix is preserved rather than re-derived.

## 14. `P3R-16` — inspected as the owner required, NOT promoted

The owner's section 22 asks whether the missing `sig_blk` / `sig_ign` / `sig_cgt` presence validation
can actually falsify a load-bearing P3 assertion. It cannot, and here is the concrete evidence.

The premise is accurate. The three keys are emitted by the fixture producer from
`/proc/self/status`, and they are members of `REPORT_SCALARS`, so a **renamed** key would still be
refused loudly as `UnknownKey` — which is the `P3R-10` defect class, and it is closed. But
`try_parse_report`'s presence enforcement runs through the local `scalar(key)` closure, which raises
`Missing` only for keys it is **asked** for. No `Report` field reads these three, so `scalar` is
never called for them and their **absence** is never detected.

Why that falsifies nothing load-bearing:

1. **No assertion reads their values.** `Report` has no field for them, so no P3 claim is made about
   signal dispositions on the basis of these keys, and none can silently pass.
2. **No default path exists.** The structural `P3R-10` fix — `Report` derives no `Default`, and every
   field it does have is obtained through `scalar(...)` or a checked count — means a missing key
   cannot produce a zero that an assertion then compares against a real value. That was the actual
   danger in `P3R-10`, and it is closed independently of these three keys.
3. The defect is that the docstring on `REPORT_SCALARS` overstates the contract for 3 of its 12
   entries. Documentation and test hygiene, not a product defect.

> **`P3R-16`: MINOR / OPEN / NONBLOCKING, carried unchanged, not fixed, not promoted.**

## 15. New MINOR findings

Three hardening observations found by adversarial probing beyond the correction's own test set. All
three are **unreachable in the gate's actual input**, evidenced in section 10, and none can change a
current closure result. None is promoted; none is to be fixed in this review.

### `P3R-17` — MINOR — the vocabulary and the backstop are case-sensitive

`JNE memcpy@PLT` (uppercase) is classified as an **ordinary instruction**: `JNE` is absent from the
lowercase `ESCAPING_MNEMONICS`, and `is_branch_like("JNE")` is `False` because the predicate tests
`startswith("j")`, not a case-folded form. Verified by execution: the probe reports 0 external, 0
indirect, 0 unresolved, 0 unsupported and **PASS**.

Why it is MINOR and not IMPORTANT: the checker's only input is `rustc --emit=asm` output, LLVM's
AT&T printer emits lowercase mnemonics exclusively, and **zero uppercase mnemonic tokens occur across
all 5 769 + 723 + 5 794 function bodies** of the three fresh profiles. It is also **not a `P3R-15`
regression** — the pre-correction `TRANSFER` regex was equally case-sensitive, so this predates the
correction and survived the two earlier reviews. Cheap hardening if the owner ever wants it: fold the
mnemonic before the lookup and the predicate.

### `P3R-18` — MINOR — `int` and `xbegin` are outside both the model and the backstop

`int $0x80` and `xbegin .L…` are read as ordinary instructions: neither starts with `j` or `loop`,
and neither contains `call` or `jmp`. Verified by execution; both **PASS**.

Neither names another function, so neither can hide a userspace runtime-helper edge — the contract
this gate enforces. The one consequence worth recording is narrower: because
[`SYSCALL`](../../tools/helm_launch_child_closure.py#L191) matches only `syscall` and `sysenter`, a
system call issued through `int $0x80` would not be counted, which could weaken the **shim-shape**
assertion ("the child issues at least one syscall, and when the shim is out of line no other function
issues one") rather than the runtime-helper closure contract. Bounded to unreachable by two
independent facts: **zero** occurrences of `int`, `into`, `int1`, `xbegin`, `xabort`, `iret`,
`sysenter`, `sysexit`, `sysret` or `bound` in any of the three fresh profiles; and
[`p3_boundary.rs`](../../crates/helm-launch/tests/p3_boundary.rs#L425) pins the crate's only `asm!`
to the literal template `asm!("syscall",` in one file, so no second hand-written instruction sequence
can exist to contain one.

### `P3R-19` — MINOR — `is_branch_like("syscall")` is `True`, and correctness depends on guard ordering

`"syscall"` contains the substring `"call"`, so the backstop predicate returns `True` for it. The
checker is **correct as written** because
[`edges()`](../../tools/helm_launch_child_closure.py#L286) tests `SYSCALL.match(line)` and
`continue`s **before** reaching the vocabulary test, so `syscall` never reaches the predicate. The
committed suite covers it: the clean-closure and inlined-shim tests would both fail if `syscall` were
bucketed as an unsupported branch.

Recorded because the coupling is implicit and easy to break: this review's own first file-wide
inventory omitted the `SYSCALL` guard and consequently mis-reported 10 and 102 "unsupported" sites,
which is direct evidence that a reimplementation or a refactor that reorders those two checks would
silently fail every closure. A one-line exclusion in the predicate, or a comment at the guard stating
the dependency, would remove the coupling. No current result is affected.

## 16. Carried forward, not re-examined

Unchanged by this correction, carried at their dispositioned severity, **not promoted and not
fixed**: `P3R-03`, `P3R-04`, `P3R-05`, `P3R-06`, `P3R-07`, `P3R-08`, `P3R-09`, `P3R-12`, `P3R-16`
(MINOR); `P3R-13`, `P3R-14` (BACKLOG_NONBLOCKING). `P3R-00` remains closed by `4c834415`. No
numeric-PID fallback is authorised. Runtime gates `P3R-G1` and `P3R-G2` remain `GATE_PENDING`.

## 17. Local validation executed

| Command | Result |
|---|---|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | ✅ |
| `cargo clippy -p helm-launch --target x86_64-unknown-linux-gnu --all-targets --all-features --locked -- -D warnings` | ✅ fresh target dir; **type-checks the Linux-gated backend and its tests** |
| `cargo test --workspace --locked` | ✅ all suites pass |
| `cargo test -p helm-launch --locked` | ✅ 42 + 19 + 17 + 19 + 2 + 23 pass |
| `cargo test -p helm-launch --locked --features test-fault-injection` | ✅ |
| `cargo build -p helm-launch --release --locked` | ✅ |
| `python -m unittest discover -s tools/tests` | ✅ **831 tests, OK (74 skipped)** |
| `python -m unittest tools.tests.test_helm_launch_machine_proofs -v` | ✅ **37 tests, OK**, no skips |
| `python tools/validate_docs.py` | ✅ 137 markdown files, 1 491 link targets, 255 JSON files |
| `git diff --check` | ✅ |
| fresh debug / release-codegen / injection child closures | ✅ all **PASSED** |
| fresh injection proof | ✅ **PASSED** |
| release-library backend-absence contrast | ✅ `child_main` absent from the plain release library, present in the probative build |
| old-versus-new checker differential, synthetic and real codegen | ✅ executed both versions |

The backend suite is `cfg`-gated to Linux x86_64 and therefore **does not execute on this host**;
cross-`clippy` type-checks it without running it. No WSL toolchain was installed; the fresh assembly
was produced with the already-installed `x86_64-unknown-linux-gnu` std target, which needs no linker
for `--emit=asm`.

> **`P3 LINUX RUNTIME VALIDATION: PENDING PUBLICATION CI`.**
> **`P3 STRACE CHILD-WINDOW VALIDATION: PENDING PUBLICATION CI`.**

These are the expected `GATE_PENDING` items (`P3R-G1`, `P3R-G2`), not review failures. The
machine-code gate is one of two gates and does not replace the strace gate.

## 18. Findings table

| ID | Severity | Area | Reachable? | Summary | Disposition |
|---|---|---|---|---|---|
| `P3R-15` | — | machine-code gate | — | 48-mnemonic explicit vocabulary; every direct transfer of every family classified as intra-function CFG, traversed call-graph edge, or fail-closed; `jno memcpy@PLT` fails as an external libc helper edge; conditional edges traversed transitively to 3 hops; the one real emitted `jno <function symbol>` is recognised **and** traversed; indirect still fail-closed with no resolver; unsupported branch-like fails closed; 76 table-driven sub-cases in both directions; prior 26 negatives byte-unchanged and passing | **INDEPENDENTLY VERIFIED FIXED** |
| `P3R-01` | — | CI proof / artifact selection | — | Tool byte-identical; rerun with fresh roots, exact `compiler-artifact` selection, 128 / 11 members inspected, PRESENT / ABSENT as required | **REMAINS FIXED** |
| `P3R-02` | — | child closed world | — | 0 external, 0 forbidden, 0 indirect, 0 unresolved, 0 unsupported in freshly regenerated debug, release-codegen and injection closures; release proof probative, not DCE-only | **REMAINS FIXED** |
| `P3R-10` | — | Linux backend test contract | — | Sixteen schema keys matching the emitter; `Report` derives no `Default`; `tgid` and `pgid_is_self` separate, separately parsed and asserted; all four refusal classes distinct. Byte-unchanged product code | **REMAINS FIXED** |
| `P3R-11` | — | machine-code gate | — | Ten indirect spellings fail closed; resolvable tail `jmp` traversed; unresolved direct target fails closed; file-wide indirect visibility identical across the correction | **REMAINS FIXED** |
| `P3R-16` | MINOR | test schema documentation | n/a | `sig_blk` / `sig_ign` / `sig_cgt` documented "required exactly once" but absence is never detected, because no `Report` field reads them. Cannot falsify a load-bearing assertion: no assertion reads their values and no default path exists | **OPEN, NONBLOCKING, not promoted** |
| `P3R-17` | MINOR | machine-code gate | **no** (0 uppercase mnemonics in 12 286 bodies) | Vocabulary and backstop are case-sensitive, so `JNE memcpy@PLT` reads as ordinary code. Pre-dates the `P3R-15` correction | **OPEN, NONBLOCKING, not promoted** |
| `P3R-18` | MINOR | machine-code gate | **no** (0 occurrences in 12 286 bodies; `asm!` pinned to one branch-free shim) | `int $0x80` and `xbegin` are outside both the model and the backstop; neither names a function, but an `int $0x80` would evade the syscall count and so the shim-shape assertion | **OPEN, NONBLOCKING, not promoted** |
| `P3R-19` | MINOR | machine-code gate | n/a (correct as written) | `is_branch_like("syscall")` is `True`; correctness depends on `edges()` testing the `SYSCALL` guard first. Implicit coupling, covered by committed tests, easy to break in a refactor | **OPEN, NONBLOCKING, not promoted** |

Handoff reconciliation, no finding id: the out-of-band claim of "46 canonical branch mnemonics" is
arithmetically wrong — the committed vocabulary is **48** unique mnemonics — but **no repository
artifact states 46**, and every count the commit message does state (5 call, 5 jmp, 33 Jcc, 38
conditional-and-loop) is correct. No branch family is missing.

## 19. Recommendation

**`P3 CANDIDATE MAY BE PUBLISHED FOR REAL LINUX BACKEND, MACHINE-CODE AND STRACE VALIDATION.`**

The correction the owner ordered is made in full and within its boundary. The checker's
control-transfer model is now closed over an explicit, reviewable 48-mnemonic vocabulary; a
conditional edge is treated as the possible execution edge it is; and the fail-closed rule the owner
made load-bearing for `P3R-02` has no remaining unclassified mnemonic family in anything rustc emits
for this crate. The evidence rests on executing both checker versions over freshly generated real
compiler output, not on reading the diff.

The P3 safety case is untouched — the whole `crates/` tree is byte-identical, so `4c834415` remains
valid for all of it. No BLOCKER and no IMPORTANT finding is open. The three new MINOR findings are
hardening items, each proven unreachable in the gate's actual input.

**P4 REMAINS NOT AUTHORISED. No Trial #4 is authorised. Trial #3 stays `MECHANISM_REJECTED` and must
not be rerun. No public `launch()` and no process-group sweep exists.**

**Next gate: ONE FAST-FORWARD PUBLICATION OF THE COMPLETE P3 CHAIN AND REVIEW RECORD, FOLLOWED BY
LOAD-BEARING HOSTED LINUX P3 CI.**

## 20. Classification

**`HELM_LAUNCH_P3_P3R15_REREVIEW_PASSED_READY_FOR_PUBLICATION_CI`**
