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
        targets, _syscalls, _indirect = self.asm.edges(CHILD)
        self.assertNotIn(".LBB0_3", targets)
        self.assertEqual(sorted(set(targets)), sorted({SHIM, FAIL}))

    def test_plt_and_got_suffixes_are_stripped(self):
        targets, _s, _i = self.asm.edges(NOISY)
        self.assertEqual(targets, ["memcpy"])

    def test_the_syscall_instruction_is_counted(self):
        _t, syscalls, _i = self.asm.edges(SHIM)
        self.assertEqual(syscalls, 1)

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
