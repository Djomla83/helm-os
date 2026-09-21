"""Positive and negative controls for the P5 release-reachability checker.

`P4PUB05-R2`. The checker in `tools/helm_launch_release_reachability.py` claims
that the public `launch` reaches the private `child_main` by direct control
transfers in an emitted release assembly. A checker that cannot fail proves
nothing, so every case below is paired: the shape it must accept, and the shape
it must refuse.

The assemblies here are synthetic and minimal. They exercise the checker, not
the compiler, and nothing in this file builds, links or runs anything.
"""

from __future__ import annotations

import unittest
from pathlib import Path
import sys

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

from helm_launch_child_closure import Assembly  # noqa: E402
from helm_launch_release_reachability import (  # noqa: E402
    ReachabilityError,
    direct_path,
    resolve,
)

ROOT = "_ZN11helm_launch6launch6launch17habcE"
MIDDLE = "_ZN11helm_launch7backend5spawn18clone_and_dispatch17hdefE"
TARGET = "_ZN11helm_launch7backend5child10child_main17h123E"
UNRELATED = "_ZN11helm_launch5plan5parse17h456E"

ROOT_NEEDLE = "11helm_launch6launch6launch"
TARGET_NEEDLE = "11helm_launch7backend5child10child_main"


def body(symbol: str, *instructions: str, index: int = 0) -> str:
    """One emitted function body in the shape `--emit=asm` produces."""
    lines = [
        f'\t.section\t.text.{symbol},"ax",@progbits',
        f"\t.globl\t{symbol}",
        f"\t.type\t{symbol},@function",
        f"{symbol}:",
    ]
    lines.extend(f"\t{instruction}" for instruction in instructions)
    lines.append("\tretq")
    lines.append(f"\t.size\t{symbol}, .Lfunc_end{index}-{symbol}")
    return "\n".join(lines)


def assembly(*bodies: str) -> Assembly:
    return Assembly("\n".join(bodies) + "\n")


class ResolveTests(unittest.TestCase):
    """A symbol needle must name exactly one emitted function."""

    def test_a_unique_needle_resolves(self) -> None:
        asm = assembly(body(ROOT), body(TARGET, index=1))
        self.assertEqual(resolve(asm, ROOT_NEEDLE, "root"), ROOT)
        self.assertEqual(resolve(asm, TARGET_NEEDLE, "target"), TARGET)

    def test_a_needle_matching_nothing_fails(self) -> None:
        asm = assembly(body(ROOT))
        with self.assertRaises(ReachabilityError) as raised:
            resolve(asm, TARGET_NEEDLE, "target")
        self.assertIn("no defined function symbol", str(raised.exception))

    def test_an_ambiguous_needle_fails_rather_than_choosing(self) -> None:
        # Two monomorphisations of the same path would be exactly this shape.
        asm = assembly(
            body("_ZN11helm_launch7backend5child10child_main17hAAAE"),
            body("_ZN11helm_launch7backend5child10child_main17hBBBE", index=1),
        )
        with self.assertRaises(ReachabilityError) as raised:
            resolve(asm, TARGET_NEEDLE, "target")
        message = str(raised.exception)
        self.assertIn("matches 2 defined symbols", message)
        self.assertIn("cannot choose one", message)


class DirectPathTests(unittest.TestCase):
    """The search follows real transfers and refuses to invent one."""

    def test_a_direct_call_chain_is_found(self) -> None:
        asm = assembly(
            body(ROOT, f"callq\t{MIDDLE}"),
            body(MIDDLE, f"callq\t{TARGET}", index=1),
            body(TARGET, index=2),
        )
        self.assertEqual(direct_path(asm, ROOT, TARGET), [ROOT, MIDDLE, TARGET])

    def test_a_tail_jump_is_a_transfer_too(self) -> None:
        # A tail call is how an optimised release build often reaches a
        # diverging function, and `child_main` never returns.
        asm = assembly(
            body(ROOT, f"jmp\t{TARGET}"),
            body(TARGET, index=1),
        )
        self.assertEqual(direct_path(asm, ROOT, TARGET), [ROOT, TARGET])

    def test_an_unrelated_neighbour_is_not_a_path(self) -> None:
        asm = assembly(
            body(ROOT, f"callq\t{UNRELATED}"),
            body(UNRELATED, index=1),
            body(TARGET, index=2),
        )
        with self.assertRaises(ReachabilityError) as raised:
            direct_path(asm, ROOT, TARGET)
        self.assertIn("no chain of direct control transfers", str(raised.exception))

    def test_an_emitted_but_unreachable_target_fails(self) -> None:
        # This is the case the plain marker grep cannot distinguish: the symbol
        # is present, and nothing calls it.
        asm = assembly(body(ROOT), body(TARGET, index=1))
        with self.assertRaises(ReachabilityError):
            direct_path(asm, ROOT, TARGET)

    def test_an_indirect_only_edge_is_never_followed(self) -> None:
        # A call through a register is unresolvable, so the checker must report
        # no path rather than assume the edge exists.
        asm = assembly(
            body(ROOT, "callq\t*%rax"),
            body(TARGET, index=1),
        )
        with self.assertRaises(ReachabilityError):
            direct_path(asm, ROOT, TARGET)

    def test_a_cycle_does_not_hang_the_search(self) -> None:
        asm = assembly(
            body(ROOT, f"callq\t{MIDDLE}"),
            body(MIDDLE, f"callq\t{ROOT}", index=1),
            body(TARGET, index=2),
        )
        with self.assertRaises(ReachabilityError):
            direct_path(asm, ROOT, TARGET)

    def test_a_longer_chain_is_still_found(self) -> None:
        extra = "_ZN11helm_launch7backend5spawn5spawn17h789E"
        asm = assembly(
            body(ROOT, f"callq\t{extra}"),
            body(extra, f"callq\t{MIDDLE}", index=1),
            body(MIDDLE, f"callq\t{TARGET}", index=2),
            body(TARGET, index=3),
        )
        self.assertEqual(
            direct_path(asm, ROOT, TARGET), [ROOT, extra, MIDDLE, TARGET]
        )


if __name__ == "__main__":
    unittest.main()
