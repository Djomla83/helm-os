"""Tests for the two HELM-LAUNCH P3 machine-level proof tools.

The tools are load-bearing gates, so the gates themselves are tested. What
matters most is the **negative** direction: each test below proves the checker
actually fails on the defect it exists to catch, so that a clean result on the
real candidate is evidence rather than a checker that finds nothing.

Nothing here builds, links or executes the crate; every case runs against a
synthetic fixture.
"""

from pathlib import Path
import io
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import helm_launch_child_closure as closure_tool
import helm_launch_injection_proof as injection_tool


def function(name: str, body: str) -> str:
    """One assembler function definition in the shape rustc emits."""
    return (
        f"\t.globl\t{name}\n"
        f"\t.type\t{name},@function\n"
        f"{name}:\n"
        f"\t.cfi_startproc\n"
        f"{body}"
        f"\t.cfi_endproc\n"
        f"\t.size\t{name}, .-{name}\n"
    )


CHILD = "_ZN11helm_launch7backend5child10child_main17habcdE"
FAIL = "_ZN11helm_launch7backend5child4fail17habcdE"
SHIM = "_ZN11helm_launch7backend7syscall8syscall617habcdE"
NOISY = "_ZN11helm_launch7backend5spawn7prepare17habcdE"


def clean_assembly(extra_child: str = "") -> str:
    return (
        function(
            CHILD,
            "\tmovq\t%rdi, %rax\n"
            f"\tcallq\t{SHIM}\n"
            "\tjmp\t.LBB0_3\n"
            ".LBB0_3:\n"
            f"\tcallq\t{FAIL}\n" + extra_child,
        )
        + function(FAIL, f"\tcallq\t{SHIM}\n")
        + function(SHIM, "\t#APP\n\tsyscall\n\t#NO_APP\n\tretq\n")
        # A function that legitimately reaches libc, so the positive control
        # has something to find.
        + function(NOISY, "\tcallq\tmemcpy@PLT\n\tretq\n")
    )


class AssemblyParserTests(unittest.TestCase):
    def setUp(self):
        self.asm = closure_tool.Assembly(clean_assembly())

    def test_every_function_body_is_extracted(self):
        for symbol in (CHILD, FAIL, SHIM, NOISY):
            self.assertTrue(self.asm.defined(symbol), symbol)

    def test_local_branches_are_not_call_edges(self):
        edges = self.asm.edges(CHILD)
        self.assertNotIn(".LBB0_3", edges.targets)
        self.assertEqual(sorted(set(edges.targets)), sorted({SHIM, FAIL}))
        # The local branch is counted as intra-function control flow rather than
        # silently discarded, so "no edges" cannot mean "nothing was read".
        self.assertEqual(edges.local, 1)
        self.assertEqual(edges.indirect, [])
        self.assertEqual(edges.unresolved, [])

    def test_plt_and_got_suffixes_are_stripped(self):
        self.assertEqual(self.asm.edges(NOISY).targets, ["memcpy"])

    def test_the_syscall_instruction_is_counted(self):
        self.assertEqual(self.asm.edges(SHIM).syscalls, 1)

    def test_symbol_resolution_is_anchored_and_rejects_ambiguity(self):
        # A compiler-generated closure sibling must not be mistaken for the
        # function itself.
        text = clean_assembly() + function(
            "_ZN11helm_launch7backend5spawn7prepare28_$u7b$$u7b$closure$u7d$$u7d$17hfeedE",
            "\tretq\n",
        )
        asm = closure_tool.Assembly(text)
        self.assertEqual(asm.find_unique("backend5spawn7prepare"), NOISY)
        with self.assertRaises(closure_tool.CheckerError):
            asm.find_unique("backend5child")  # matches two functions


class ChildClosureGateTests(unittest.TestCase):
    def test_a_clean_child_closure_passes(self):
        asm = closure_tool.Assembly(clean_assembly())
        result = closure_tool.check(asm, control_needle=None)
        self.assertEqual(result["problems"], [])
        self.assertEqual(result["external_edges"], [])
        self.assertIn(SHIM, result["closure"])
        self.assertEqual(result["shim_shape"], "out-of-line")

    def test_a_memcpy_edge_in_the_child_fails(self):
        # The exact defect the gate exists to catch.
        asm = closure_tool.Assembly(clean_assembly(extra_child="\tcallq\tmemcpy@PLT\n"))
        result = closure_tool.check(asm, control_needle=None)
        self.assertTrue(result["problems"])
        self.assertTrue(
            any("memcpy" in problem for problem in result["problems"]), result["problems"]
        )

    def test_a_panic_edge_in_the_child_fails(self):
        asm = closure_tool.Assembly(
            clean_assembly(
                extra_child="\tcallq\t*_RNvNtCs_4core9panicking18panic_bounds_check@GOTPCREL(%rip)\n"
            )
        )
        result = closure_tool.check(asm, control_needle=None)
        self.assertTrue(
            any("panic runtime" in problem for problem in result["problems"]),
            result["problems"],
        )

    def test_an_edge_reached_only_transitively_fails(self):
        # A helper is acceptable only if its own closure is clean too.
        text = clean_assembly().replace(
            function(FAIL, f"\tcallq\t{SHIM}\n"),
            function(FAIL, f"\tcallq\t{SHIM}\n\tcallq\tmalloc@PLT\n"),
        )
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertTrue(
            any("malloc" in problem for problem in result["problems"]), result["problems"]
        )

    def test_an_unresolved_indirect_call_fails(self):
        asm = closure_tool.Assembly(clean_assembly(extra_child="\tcallq\t*%rax\n"))
        result = closure_tool.check(asm, control_needle=None)
        self.assertTrue(
            any("indirect" in problem for problem in result["problems"]), result["problems"]
        )
        self.assertEqual(len(result["indirect_transfer_sites"]), 1)

    def test_an_unresolved_indirect_jmp_fails(self):
        # P3R-11. An indirect tail jump escapes the function exactly as an
        # indirect call does, so it must fail rather than be discarded.
        for spelling in (
            "\tjmp\t*%rax\n",
            "\tjmpq\t*%rax\n",
            "\tjmpq\t*(%rax)\n",
            "\tjmpq\t*8(%rbp)\n",
            "\tjmpq\t*.LJTI0_0(,%rax,8)\n",
            "\tnotrack jmpq\t*%r11\n",
        ):
            with self.subTest(spelling=spelling.strip()):
                asm = closure_tool.Assembly(clean_assembly(extra_child=spelling))
                result = closure_tool.check(asm, control_needle=None)
                self.assertEqual(
                    len(result["indirect_transfer_sites"]),
                    1,
                    f"{spelling.strip()} was not recorded as an indirect transfer",
                )
                self.assertTrue(
                    any("indirect" in problem for problem in result["problems"]),
                    result["problems"],
                )

    def test_a_direct_tail_jmp_to_a_forbidden_external_symbol_fails(self):
        # A tail jump to a runtime helper must fail exactly like a call to it.
        for spelling in ("\tjmp\tmemcpy\n", "\tjmpq\tmemcpy@PLT\n", "\tjmp\t__rust_alloc\n"):
            with self.subTest(spelling=spelling.strip()):
                asm = closure_tool.Assembly(clean_assembly(extra_child=spelling))
                result = closure_tool.check(asm, control_needle=None)
                self.assertTrue(
                    any(
                        "libc string/memory helper" in problem or "allocator" in problem
                        for problem in result["problems"]
                    ),
                    result["problems"],
                )

    def test_a_direct_tail_jmp_to_an_internal_helper_is_traversed(self):
        # The other half of the same rule: a resolvable tail jump to an internal
        # helper is a call-graph edge, so the helper enters the closure and its
        # own closure is checked. Here the helper itself tail-jumps to `malloc`,
        # which must therefore be found.
        helper = "_ZN11helm_launch7backend5child6tailed17habcdE"
        text = (
            function(
                CHILD,
                f"\tcallq\t{SHIM}\n\tjmp\t{helper}\n",
            )
            + function(helper, "\tjmp\tmalloc@PLT\n")
            + function(FAIL, f"\tcallq\t{SHIM}\n")
            + function(SHIM, "\t#APP\n\tsyscall\n\t#NO_APP\n\tretq\n")
            + function(NOISY, "\tcallq\tmemcpy@PLT\n\tretq\n")
        )
        asm = closure_tool.Assembly(text)
        result = closure_tool.check(asm, control_needle=None)
        self.assertIn(helper, result["closure"])
        self.assertIn(
            (CHILD, helper),
            [tuple(edge) for edge in result["internal_edges"]],
        )
        self.assertTrue(
            any("malloc" in problem for problem in result["problems"]), result["problems"]
        )

    def test_a_direct_jmp_to_a_label_the_body_does_not_define_fails(self):
        # A local label belonging to another function is not an intra-function
        # branch, and must not be treated as one.
        asm = closure_tool.Assembly(clean_assembly(extra_child="\tjmp\t.LBB99_7\n"))
        result = closure_tool.check(asm, control_needle=None)
        self.assertEqual(len(result["unresolved_transfer_sites"]), 1)
        self.assertTrue(
            any("unresolved control transfer" in problem for problem in result["problems"]),
            result["problems"],
        )

    def test_an_unknown_control_transfer_spelling_fails_closed(self):
        # An operand form this parser does not understand may still escape the
        # function, so it is reported rather than ignored.
        for spelling in (
            "\tjmpq\t0x1234(%rbx)\n",
            "\tcallq\t(%rax)\n",
            "\tjmp\t*\n",
            "\tcallq\n",
        ):
            with self.subTest(spelling=spelling.strip()):
                asm = closure_tool.Assembly(clean_assembly(extra_child=spelling))
                result = closure_tool.check(asm, control_needle=None)
                unresolved = len(result["unresolved_transfer_sites"])
                indirect = len(result["indirect_transfer_sites"])
                self.assertEqual(
                    unresolved + indirect,
                    1,
                    f"{spelling.strip()} was neither resolved nor reported",
                )
                self.assertTrue(result["problems"], result)

    def test_a_comment_can_neither_hide_nor_invent_a_transfer(self):
        # The instruction is read without its comment, so a commented-out call
        # is not an edge and a real call followed by a comment still is.
        asm = closure_tool.Assembly(
            clean_assembly(extra_child="\t# callq\tmemcpy@PLT\n\tcallq\tmalloc@PLT # tail\n")
        )
        result = closure_tool.check(asm, control_needle=None)
        reached = {symbol for _origin, symbol, _category in result["external_edges"]}
        self.assertEqual(reached, {"malloc"})

    def test_a_missing_root_fails_loudly_rather_than_passing(self):
        asm = closure_tool.Assembly(function(NOISY, "\tretq\n"))
        with self.assertRaises(closure_tool.CheckerError):
            closure_tool.check(asm, control_needle=None)

    def test_a_child_with_no_syscall_at_all_fails(self):
        text = function(CHILD, "\tretq\n") + function(NOISY, "\tcallq\tmemcpy@PLT\n")
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertTrue(
            any("no syscall instruction" in problem for problem in result["problems"]),
            result["problems"],
        )

    def test_the_positive_control_fails_when_nothing_forbidden_exists(self):
        # If the assembly contains no forbidden-category reference anywhere, the
        # checker cannot claim to be able to detect one.
        text = (
            function(CHILD, f"\tcallq\t{SHIM}\n")
            + function(SHIM, "\tsyscall\n\tretq\n")
        )
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertTrue(
            any("POSITIVE CONTROL FAILED" in problem for problem in result["problems"]),
            result["problems"],
        )

    def test_an_inlined_shim_is_accepted_with_its_own_rule(self):
        text = function(CHILD, "\tsyscall\n\tsyscall\n\tretq\n") + function(
            NOISY, "\tcallq\tmemcpy@PLT\n"
        )
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertEqual(result["shim_shape"], "inlined")
        self.assertEqual(result["problems"], [])


HELPER = "_ZN11helm_launch7backend5child6helper17habcdE"


class ConditionalBranchGateTests(unittest.TestCase):
    """P3R-15 — a conditional branch is a function-escaping transfer too.

    The earlier model assumed rustc emits `Jcc` only to a label of the same
    function, so a conditional branch out of a function was discarded without
    record. This crate's own release-codegen assembly disproved the assumption
    with a real `jno <function symbol>`, and a synthetic `jno memcpy@PLT` in
    `child_main` passed the gate. These cases are table-driven over the whole
    branch vocabulary so a mnemonic cannot be modelled in one direction only.
    """

    def test_the_p3r15_reproduction_fails(self):
        # The exact instruction that passed the previous checker.
        result = closure_tool.check(
            closure_tool.Assembly(clean_assembly(extra_child="\tjno\tmemcpy@PLT\n")),
            control_needle=None,
        )
        self.assertIn(
            ("memcpy", "libc string/memory helper"),
            [(symbol, category) for _o, symbol, category in result["external_edges"]],
        )
        self.assertTrue(result["problems"], result)

    def test_the_branch_vocabulary_is_the_canonical_one(self):
        # The vocabulary is the authority, so it is pinned rather than inferred.
        self.assertEqual(len(closure_tool.CONDITIONAL_JUMP_MNEMONICS), 33)
        self.assertEqual(
            closure_tool.LOOP_MNEMONICS,
            frozenset({"loop", "loope", "loopz", "loopne", "loopnz"}),
        )
        for mnemonic in ("jno", "je", "jne", "jrcxz", "jecxz", "jcxz", "jpo", "jnle"):
            self.assertIn(mnemonic, closure_tool.CONDITIONAL_JUMP_MNEMONICS)
        for mnemonic in closure_tool.ESCAPING_MNEMONICS:
            self.assertTrue(closure_tool.is_branch_like(mnemonic), mnemonic)

    def test_every_conditional_branch_to_a_forbidden_external_fails(self):
        for mnemonic in sorted(
            closure_tool.CONDITIONAL_JUMP_MNEMONICS | closure_tool.LOOP_MNEMONICS
        ):
            with self.subTest(mnemonic=mnemonic):
                result = closure_tool.check(
                    closure_tool.Assembly(
                        clean_assembly(extra_child=f"\t{mnemonic}\tmemcpy@PLT\n")
                    ),
                    control_needle=None,
                )
                self.assertTrue(
                    any(
                        "libc string/memory helper" in problem
                        for problem in result["problems"]
                    ),
                    f"{mnemonic} memcpy@PLT was not reported as a libc helper edge: "
                    f"{result['problems']}",
                )

    def test_every_conditional_branch_to_an_own_label_is_intra_function(self):
        # The other half of the rule: modelling the mnemonic must not turn
        # ordinary intra-function control flow into a false edge.
        for mnemonic in sorted(
            closure_tool.CONDITIONAL_JUMP_MNEMONICS | closure_tool.LOOP_MNEMONICS
        ):
            with self.subTest(mnemonic=mnemonic):
                asm = closure_tool.Assembly(
                    clean_assembly(extra_child=f"\t{mnemonic}\t.LBB0_7\n.LBB0_7:\n")
                )
                result = closure_tool.check(asm, control_needle=None)
                edges = asm.edges(CHILD)
                self.assertEqual(result["problems"], [], mnemonic)
                self.assertNotIn(".LBB0_7", edges.targets)
                # `jmp .LBB0_3` in the baseline plus this one.
                self.assertEqual(edges.local, 2, mnemonic)

    def test_a_conditional_branch_to_an_internal_helper_is_traversed(self):
        # `je helper` is a call-graph edge: the helper enters the closure and
        # its own branches are inspected, rather than the branch being marked
        # seen and dropped.
        text = (
            function(CHILD, f"\tcallq\t{SHIM}\n\tje\t{HELPER}\n\tretq\n")
            + function(HELPER, "\tjne\tmalloc@PLT\n\tretq\n")
            + function(SHIM, "\t#APP\n\tsyscall\n\t#NO_APP\n\tretq\n")
            + function(NOISY, "\tcallq\tmemcpy@PLT\n\tretq\n")
        )
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertIn(HELPER, result["closure"])
        self.assertIn(
            (CHILD, HELPER), [tuple(edge) for edge in result["internal_edges"]]
        )
        self.assertTrue(
            any("malloc" in problem for problem in result["problems"]),
            result["problems"],
        )

    def test_a_clean_conditional_branch_to_an_internal_helper_passes(self):
        # The positive direction, so the rule above is not satisfied by failing
        # on every conditional branch.
        text = (
            function(CHILD, f"\tcallq\t{SHIM}\n\tjg\t{HELPER}\n\tretq\n")
            + function(HELPER, "\tretq\n")
            + function(SHIM, "\t#APP\n\tsyscall\n\t#NO_APP\n\tretq\n")
            + function(NOISY, "\tcallq\tmemcpy@PLT\n\tretq\n")
        )
        result = closure_tool.check(closure_tool.Assembly(text), control_needle=None)
        self.assertIn(HELPER, result["closure"])
        self.assertEqual(result["problems"], [])

    def test_a_conditional_branch_keeps_every_other_fail_closed_rule(self):
        for spelling, bucket in (
            ("\tjne\t*%rax\n", "indirect_transfer_sites"),
            ("\tjno\t*(%rbx)\n", "indirect_transfer_sites"),
            ("\tjno\t.LBB99_7\n", "unresolved_transfer_sites"),
            ("\tjs\tmemcpy+0x20\n", "unresolved_transfer_sites"),
            ("\tje\n", "unresolved_transfer_sites"),
        ):
            with self.subTest(spelling=spelling.strip()):
                result = closure_tool.check(
                    closure_tool.Assembly(clean_assembly(extra_child=spelling)),
                    control_needle=None,
                )
                self.assertEqual(len(result[bucket]), 1, result)
                self.assertTrue(result["problems"], result)

    def test_a_conditional_branch_through_the_got_resolves_to_its_symbol(self):
        result = closure_tool.check(
            closure_tool.Assembly(
                clean_assembly(extra_child="\tjno\t*memcpy@GOTPCREL(%rip)\n")
            ),
            control_needle=None,
        )
        reached = {symbol for _origin, symbol, _category in result["external_edges"]}
        self.assertIn("memcpy", reached)

    def test_a_prefixed_conditional_branch_is_still_classified(self):
        result = closure_tool.check(
            closure_tool.Assembly(clean_assembly(extra_child="\tbnd jne\tmemcpy@PLT\n")),
            control_needle=None,
        )
        reached = {symbol for _origin, symbol, _category in result["external_edges"]}
        self.assertIn("memcpy", reached)

    def test_an_unsupported_branch_mnemonic_fails_closed(self):
        # "Not in the vocabulary" must never mean "ordinary instruction".
        for spelling in (
            "\tjfoo\tmemcpy@PLT\n",
            "\tjqq\t%rax\n",
            "\tloopfoo\t.LBB0_3\n",
            "\tcallfoo\tmemcpy\n",
        ):
            with self.subTest(spelling=spelling.strip()):
                result = closure_tool.check(
                    closure_tool.Assembly(clean_assembly(extra_child=spelling)),
                    control_needle=None,
                )
                self.assertEqual(len(result["unsupported_transfer_sites"]), 1, result)
                self.assertTrue(
                    any(
                        "outside the modelled vocabulary" in problem
                        for problem in result["problems"]
                    ),
                    result["problems"],
                )

    def test_an_ordinary_instruction_is_not_a_branch(self):
        # The conservative backstop must not swallow normal code, or every
        # closure would fail and the gate would prove nothing.
        body = (
            "\tmovq\t8(%rdi), %rsi\n"
            "\tcmpq\t$-4095, %rax\n"
            "\tleaq\t(%rax,%rbx,8), %rcx\n"
            "\tlock cmpxchgq\t%rcx, (%rdx)\n"
            "\tud2\n"
        )
        asm = closure_tool.Assembly(clean_assembly(extra_child=body))
        result = closure_tool.check(asm, control_needle=None)
        self.assertEqual(result["unsupported_transfer_sites"], [])
        self.assertEqual(result["problems"], [])


def synthetic_archive(members: list[tuple[str, bytes]]) -> bytes:
    out = io.BytesIO()
    out.write(b"!<arch>\n")
    for name, body in members:
        header = (
            f"{name:<16}".encode()
            + b"0           "
            + b"0     "
            + b"0     "
            + b"100644  "
            + f"{len(body):<10}".encode()
            + b"`\n"
        )
        assert len(header) == 60, len(header)
        out.write(header)
        out.write(body)
        if len(body) % 2:
            out.write(b"\n")
    return out.getvalue()


class InjectionProofTests(unittest.TestCase):
    def test_every_archive_member_is_inspected_not_only_the_first(self):
        data = synthetic_archive(
            [("first.o", b"nothing here at all"), ("second.o", b"x" + injection_tool.MARKER)]
        )
        members = injection_tool.archive_members(data)
        self.assertEqual([name for name, _ in members], ["first.o", "second.o"])
        self.assertIn(injection_tool.MARKER, members[1][1])

    def test_a_marker_in_a_late_member_is_found(self):
        path = Path(self.enterContext(__import__("tempfile").TemporaryDirectory())) / "a.rlib"
        path.write_bytes(
            synthetic_archive(
                [("a.o", b"clean"), ("b.o", b"clean"), ("c.o", injection_tool.MARKER)]
            )
        )
        report = injection_tool.inspect(path)
        self.assertTrue(report["present"])
        self.assertEqual(report["members"], 3)
        self.assertEqual(report["hits"][0]["member"], "c.o")

    def test_a_clean_archive_reports_absent(self):
        path = Path(self.enterContext(__import__("tempfile").TemporaryDirectory())) / "a.rlib"
        path.write_bytes(synthetic_archive([("a.o", b"clean"), ("b.o", b"also clean")]))
        report = injection_tool.inspect(path)
        self.assertFalse(report["present"])
        self.assertEqual(report["members"], 2)

    def test_a_non_archive_artifact_fails_loudly(self):
        path = Path(self.enterContext(__import__("tempfile").TemporaryDirectory())) / "a.rlib"
        path.write_bytes(b"this is not an ar archive")
        with self.assertRaises(injection_tool.ProofError):
            injection_tool.inspect(path)

    def test_an_empty_artifact_fails_loudly(self):
        path = Path(self.enterContext(__import__("tempfile").TemporaryDirectory())) / "a.rlib"
        path.write_bytes(b"")
        with self.assertRaises(injection_tool.ProofError):
            injection_tool.inspect(path)

    def test_a_missing_artifact_fails_loudly(self):
        path = Path(self.enterContext(__import__("tempfile").TemporaryDirectory())) / "gone.rlib"
        with self.assertRaises(injection_tool.ProofError):
            injection_tool.inspect(path)


if __name__ == "__main__":
    unittest.main()
