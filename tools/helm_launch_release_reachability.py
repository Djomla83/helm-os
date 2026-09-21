#!/usr/bin/env python3
"""Prove that the P4 production release library really *reaches* the private backend.

`P4PUB05-R2`. The release-library CI gate greps one mangled symbol, which proves
the backend child entry was **emitted**. Emission of a crate-private, non-generic
function already implies the monomorphisation collector reached it, so the grep is
probative — but it is not, on its own, a statement about the call graph. This tool
makes the stronger claim explicit: there is a chain of **direct** control transfers
from the public ``launch`` to ``backend::child::child_main`` inside the emitted
release assembly.

It is deliberately a **separate** checker. ``tools/helm_launch_child_closure.py`` is
a load-bearing P3 gate with its own accepted semantics; this tool imports its
assembly parser and changes nothing in it. If that parser ever stops resolving a
transfer, this tool reports no path and fails — it never guesses.

Fail-closed everywhere:

* a root or target substring that matches zero defined symbols is a failure;
* a substring that matches more than one defined symbol is a failure, because the
  tool must not choose;
* no direct-transfer path is a failure.

Inlining is expected and is not a problem: the search follows whatever out-of-line
functions survive. ``spawn_for_lifecycle`` and ``spawn`` are normally inlined into
``launch``, and ``clone_and_dispatch`` survives because it is ``#[inline(never)]``,
so the observed chain is ``launch -> clone_and_dispatch -> child_main``.

This reads an already emitted ``.s`` file. It compiles nothing and runs nothing.
"""

from __future__ import annotations

import argparse
import sys
from collections import deque
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from helm_launch_child_closure import Assembly  # noqa: E402  (path set above)

#: The public P4 entry point, in Rust's legacy mangling.
DEFAULT_ROOT = "11helm_launch6launch6launch"
#: The private backend child entry point.
DEFAULT_TARGET = "11helm_launch7backend5child10child_main"


class ReachabilityError(RuntimeError):
    """A structural failure. Never a statement about the product's behaviour."""


def resolve(asm: Assembly, needle: str, role: str) -> str:
    """The one defined symbol containing ``needle``, or a failure."""
    found = sorted(symbol for symbol in asm.bodies if needle in symbol)
    if not found:
        raise ReachabilityError(
            f"no defined function symbol contains the {role} {needle!r}. "
            f"The assembly holds {len(asm.bodies)} function bodies."
        )
    if len(found) > 1:
        listed = "\n  ".join(found)
        raise ReachabilityError(
            f"the {role} {needle!r} matches {len(found)} defined symbols, so this "
            f"tool cannot choose one:\n  {listed}"
        )
    return found[0]


def direct_path(asm: Assembly, root: str, target: str) -> list[str]:
    """A shortest chain of direct control transfers from ``root`` to ``target``."""
    if root == target:
        return [root]
    seen = {root}
    queue: deque[list[str]] = deque([[root]])
    while queue:
        path = queue.popleft()
        for next_symbol in asm.edges(path[-1]).targets:
            if next_symbol == target:
                return [*path, next_symbol]
            if next_symbol in seen or not asm.defined(next_symbol):
                continue
            seen.add(next_symbol)
            queue.append([*path, next_symbol])
    raise ReachabilityError(
        f"no chain of direct control transfers reaches {target} from {root}. "
        f"{len(seen)} function bodies were explored. Either the public launch path "
        f"no longer instantiates the backend, or every remaining edge into it is "
        f"indirect and therefore unprovable here."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asm", type=Path, required=True, help="an emitted .s file")
    parser.add_argument("--root", default=DEFAULT_ROOT, help="root symbol substring")
    parser.add_argument("--target", default=DEFAULT_TARGET, help="target substring")
    parser.add_argument("--label", default="P4 release library")
    args = parser.parse_args()

    print(f"=== {args.label}: public launch reaches the private backend ===")
    try:
        text = args.asm.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        print(f"BUILD EVIDENCE FAILURE: cannot read {args.asm}: {error}")
        return 1

    asm = Assembly(text)
    print(f"assembly : {args.asm}")
    print(f"bodies   : {len(asm.bodies)} function(s)")

    try:
        root = resolve(asm, args.root, "root")
        target = resolve(asm, args.target, "target")
        path = direct_path(asm, root, target)
    except ReachabilityError as error:
        print(f"REACHABILITY CHECK FAILED: {error}")
        return 1

    print(f"root     : {root}")
    print(f"target   : {target}")
    print(f"path     : {len(path)} function(s)")
    for depth, symbol in enumerate(path):
        print(f"  {'  ' * depth}-> {symbol}")
    print(
        "REACHABILITY CHECK PASSED: the public launch reaches the private child "
        "entry point by direct control transfers in this release assembly."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
