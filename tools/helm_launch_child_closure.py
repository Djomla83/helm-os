#!/usr/bin/env python3
"""HELM-LAUNCH P3 machine-code gate for the post-clone child window.

Source-token scanning cannot see code the compiler inserts, and `strace` cannot
see userspace helper calls that issue no system call. This checker reads the
**machine code the compiler actually emitted** for the Linux x86_64 child path,
builds the call closure rooted at `child_main`, and fails if that closure
reaches any external runtime helper.

Required target (owner disposition of P3R-02, 2026-09-18):

    NORMAL P3 CHILD MACHINE CODE MUST NOT CALL GLIBC, LIBSTD, ALLOCATOR,
    PANIC/UNWIND OR OTHER EXTERNAL RUNTIME HELPERS BETWEEN CHILD ENTRY AND
    execveat / exit_group.

Internal helm-launch child helpers and the raw syscall shim are acceptable only
if their own closures satisfy the same contract, which is why the walk is
transitive.

This is one of **two** gates and does not replace the other:

    MACHINE-CODE GATE  no forbidden userspace runtime helper in the child closure
    STRACE GATE        no forbidden syscall in the runtime child window

It analyses `rustc --emit=asm` output rather than a linked image, so it runs
identically on the Linux runner and on a cross-compiling developer host, and it
sees helper references by name before the linker turns them into PLT stubs.

The checker fails loudly rather than silently passing: a missing root, an empty
closure, an unresolved indirect call, a missing syscall shim or a failed
positive control are all errors, so "zero matches" can never mean "clean"
because the parser found nothing.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

# --------------------------------------------------------------------------
# Classification
# --------------------------------------------------------------------------

# An external reference whose name matches one of these is reported with its
# category. Anything external that matches none of them is still a failure —
# the contract is "no external runtime helper", not "none of a known list" —
# but naming the category makes a failure readable.
FORBIDDEN_CATEGORIES: list[tuple[str, str]] = [
    ("memcpy", "libc string/memory helper"),
    ("memmove", "libc string/memory helper"),
    ("memset", "libc string/memory helper"),
    ("memcmp", "libc string/memory helper"),
    ("bcmp", "libc string/memory helper"),
    ("strlen", "libc string/memory helper"),
    ("malloc", "allocator"),
    ("calloc", "allocator"),
    ("realloc", "allocator"),
    ("free", "allocator"),
    ("__rust_alloc", "allocator"),
    ("__rust_dealloc", "allocator"),
    ("__rust_realloc", "allocator"),
    ("__rust_alloc_error_handler", "allocator error handler"),
    ("handle_alloc_error", "allocator error handler"),
    ("panicking", "panic runtime"),
    ("panic", "panic runtime"),
    ("unwrap_failed", "panic runtime"),
    ("expect_failed", "panic runtime"),
    ("_Unwind", "unwinder"),
    ("rust_eh_personality", "unwinder"),
    ("personality", "unwinder"),
    ("pthread", "thread runtime"),
    ("futex", "thread runtime"),
    ("__tls_get_addr", "TLS runtime"),
    ("errno_location", "libc errno"),
    ("abort", "libc abort"),
    ("exit", "libc exit"),
]


def categorise(symbol: str) -> str:
    for needle, category in FORBIDDEN_CATEGORIES:
        if needle in symbol:
            return category
    return "unclassified external reference"


# --------------------------------------------------------------------------
# Assembly parsing
# --------------------------------------------------------------------------

LABEL = re.compile(r"^([A-Za-z_$.][\w$.]*):\s*$")
SIZE_DIRECTIVE = re.compile(r"^\s*\.size\s")
# `callq  sym`, `callq  *sym@GOTPCREL(%rip)`, `jmp  sym` (tail call).
DIRECT_CALL = re.compile(r"^\s*(?:call|callq|jmp|jmpq)\s+\*?([A-Za-z_$.][\w$.@]*)")
INDIRECT_CALL = re.compile(r"^\s*(?:call|callq)\s+\*(?![A-Za-z_$.])")
SYSCALL = re.compile(r"^\s*syscall\b")


class Assembly:
    """Function bodies and their outgoing edges, from one `--emit=asm` file."""

    def __init__(self, text: str) -> None:
        self.lines = text.splitlines()
        self.bodies: dict[str, list[str]] = {}
        current: str | None = None
        for line in self.lines:
            label = LABEL.match(line)
            if label:
                name = label.group(1)
                if not name.startswith(".L"):
                    current = name
                    self.bodies.setdefault(current, [])
                else:
                    # A local label inside the current body.
                    if current is not None:
                        self.bodies[current].append(line)
                continue
            if current is None:
                continue
            if SIZE_DIRECTIVE.match(line):
                current = None
                continue
            self.bodies[current].append(line)

    def defined(self, symbol: str) -> bool:
        return symbol in self.bodies

    def edges(self, symbol: str) -> tuple[list[str], int, int]:
        """Outgoing call targets, syscall count and unresolved indirect count."""
        targets: list[str] = []
        syscalls = 0
        indirect = 0
        for line in self.bodies.get(symbol, []):
            if SYSCALL.match(line):
                syscalls += 1
            if INDIRECT_CALL.match(line):
                indirect += 1
                continue
            call = DIRECT_CALL.match(line)
            if not call:
                continue
            target = call.group(1)
            if target.startswith(".L"):
                # A local branch inside the same function.
                continue
            targets.append(target.split("@")[0])
        return targets, syscalls, indirect

    def find_unique(self, needle: str) -> str:
        """Resolve one function by name.

        The needle is anchored to the Itanium mangling hash marker (`17h`) that
        immediately follows a path component, so `…prepare17h…` is found and the
        compiler-generated `…prepare28_$u7b$$u7b$closure…` siblings are not.
        Ambiguity is an error, never a silent first match.
        """
        anchored = needle + "17h"
        matches = sorted(s for s in self.bodies if anchored in s)
        if not matches:
            loose = sorted(s for s in self.bodies if needle in s)
            raise CheckerError(
                f"no function named {needle!r} is defined in the assembly"
                + (f"; similar symbols: {loose[:6]}" if loose else "")
            )
        if len(matches) > 1:
            raise CheckerError(f"function {needle!r} is ambiguous: {matches}")
        return matches[0]


class CheckerError(RuntimeError):
    """A condition that must fail loudly rather than pass quietly."""


# --------------------------------------------------------------------------
# Closure
# --------------------------------------------------------------------------


def closure(asm: Assembly, root: str) -> dict:
    seen: list[str] = []
    pending = [root]
    external: list[tuple[str, str, str]] = []  # (from, symbol, category)
    indirect_sites: list[str] = []
    syscall_sites: dict[str, int] = {}
    internal_edges: list[tuple[str, str]] = []

    while pending:
        symbol = pending.pop(0)
        if symbol in seen:
            continue
        seen.append(symbol)
        targets, syscalls, indirect = asm.edges(symbol)
        if syscalls:
            syscall_sites[symbol] = syscalls
        if indirect:
            indirect_sites.append(f"{symbol} ({indirect} indirect call site(s))")
        for target in targets:
            if asm.defined(target):
                internal_edges.append((symbol, target))
                pending.append(target)
            else:
                external.append((symbol, target, categorise(target)))

    return {
        "root": root,
        "closure": seen,
        "internal_edges": sorted(set(internal_edges)),
        "external_edges": sorted(set(external)),
        "indirect_call_sites": sorted(set(indirect_sites)),
        "syscall_sites": syscall_sites,
    }


# --------------------------------------------------------------------------
# Building the assembly
# --------------------------------------------------------------------------


def emit_assembly(
    manifest_dir: Path,
    target: str,
    target_dir: Path,
    profile: str,
    features: list[str],
    rustc_flags: list[str],
) -> Path:
    """Compile the crate to assembly and return the one emitted file."""
    if shutil.which("cargo") is None:
        raise CheckerError("TEST ENVIRONMENT FAILURE: cargo is not on PATH")

    command = [
        "cargo",
        "rustc",
        "-p",
        "helm-launch",
        "--lib",
        "--locked",
        "--target",
        target,
        "--profile",
        profile,
        "--target-dir",
        str(target_dir),
    ]
    for feature in features:
        command += ["--features", feature]
    command += ["--", "--emit=asm", *rustc_flags]

    finished = subprocess.run(
        command, cwd=manifest_dir, capture_output=True, text=True, check=False
    )
    emitted = sorted(target_dir.rglob("helm_launch-*.s"))
    if not emitted:
        raise CheckerError(
            "TEST ENVIRONMENT FAILURE: no assembly was emitted.\n"
            f"command: {' '.join(command)}\n"
            f"stderr:\n{finished.stderr[-4000:]}"
        )
    if len(emitted) > 1:
        raise CheckerError(
            "ambiguous assembly output; a fresh target directory must contain exactly one:\n  "
            + "\n  ".join(str(path) for path in emitted)
        )
    return emitted[0]


# --------------------------------------------------------------------------
# The gate
# --------------------------------------------------------------------------


def check(asm: Assembly, control_needle: str | None) -> dict:
    root = asm.find_unique("backend5child10child_main")
    result = closure(asm, root)

    problems: list[str] = []

    # "The walk is inspecting something." A fully inlined child can legitimately
    # be one function, so the check is on the root's body, not on a function
    # count: an empty body would mean the extraction failed.
    instructions = [
        line
        for line in asm.bodies.get(root, [])
        if line.strip() and not line.strip().startswith((".", "#"))
    ]
    result["root_instructions"] = len(instructions)
    if not instructions:
        problems.append(
            f"the child entry point {root} has an empty body, so the extraction failed"
        )
    # The child must actually issue system calls, or the closure being clean
    # would be meaningless. How they appear depends on the optimisation level:
    # unoptimised, the reviewed shim is a separate function the closure calls;
    # optimised, that one shim is inlined into the child's own bodies. Both
    # shapes are accepted, and each has its own rule; neither is inferred.
    total_syscalls = sum(result["syscall_sites"].values())
    shim = [s for s in result["closure"] if "backend7syscall8syscall6" in s]
    result["shim_shape"] = "out-of-line" if shim else "inlined"
    if total_syscalls == 0:
        problems.append(
            "the child closure contains no syscall instruction at all, so either the "
            "root is wrong or the extraction failed"
        )
    if shim:
        if result["syscall_sites"].get(shim[0], 0) != 1:
            problems.append(
                f"the syscall shim {shim[0]} does not contain exactly one syscall instruction"
            )
        for symbol, _count in result["syscall_sites"].items():
            if "backend7syscall8syscall6" not in symbol:
                problems.append(
                    f"{symbol} issues a syscall although the reviewed shim is out of line here"
                )
    if result["indirect_call_sites"]:
        problems.append(
            "unresolved indirect call site(s) in the child closure: "
            + ", ".join(result["indirect_call_sites"])
        )
    for origin, symbol, category in result["external_edges"]:
        problems.append(f"{origin} -> {symbol}  [{category}]")

    # Positive control. A clean child closure is only evidence if the extractor
    # and the classifier can find a forbidden-class edge where one legitimately
    # exists; otherwise "zero external edges" might mean the parser found
    # nothing. The control scans every function defined in the same file for a
    # direct external reference in a forbidden category. It is deliberately not
    # tied to one named function, because at higher optimisation levels any
    # particular helper may be inlined away.
    control: dict = {"witnesses": [], "witness_count": 0, "named_root": None}
    if control_needle:
        try:
            named = asm.find_unique(control_needle)
        except CheckerError:
            named = None
        if named:
            named_closure = closure(asm, named)
            control["named_root"] = named
            control["named_external_edge_count"] = len(named_closure["external_edges"])

    for symbol in sorted(asm.bodies):
        targets, _, _ = asm.edges(symbol)
        for target in targets:
            if asm.defined(target):
                continue
            if categorise(target) != "unclassified external reference":
                control["witnesses"].append((symbol, target, categorise(target)))
                break
        if len(control["witnesses"]) >= 4:
            break
    control["witness_count"] = len(control["witnesses"])
    if control["witness_count"] == 0:
        problems.append(
            "POSITIVE CONTROL FAILED: no function anywhere in this assembly reaches a "
            "forbidden-category external helper, so the checker cannot be trusted to "
            "detect one in the child closure"
        )

    result["problems"] = problems
    result["control"] = control
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asm", type=Path, help="an already emitted .s file")
    parser.add_argument("--manifest-dir", type=Path, default=Path.cwd())
    parser.add_argument("--target", default="x86_64-unknown-linux-gnu")
    parser.add_argument("--target-dir", type=Path)
    parser.add_argument("--profile", default="test")
    parser.add_argument("--features", action="append", default=[])
    parser.add_argument("--rustc-flag", action="append", default=[])
    parser.add_argument(
        "--control",
        default="backend5spawn7prepare",
        help="symbol needle expected to reach an external helper; empty disables",
    )
    parser.add_argument("--label", default="child closure")
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args()

    try:
        if arguments.asm:
            path = arguments.asm
        else:
            if not arguments.target_dir:
                raise CheckerError("--target-dir is required when building")
            path = emit_assembly(
                arguments.manifest_dir,
                arguments.target,
                arguments.target_dir,
                arguments.profile,
                arguments.features,
                arguments.rustc_flag,
            )
        asm = Assembly(path.read_text(encoding="utf-8", errors="replace"))
        result = check(asm, arguments.control or None)
    except CheckerError as error:
        print(f"CHILD CLOSURE CHECK ERROR: {error}", file=sys.stderr)
        return 2

    result["assembly"] = str(path)
    result["label"] = arguments.label
    if arguments.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"=== P3 child machine-code closure: {arguments.label} ===")
        print(f"assembly : {path}")
        print(f"root     : {result['root']}")
        print(f"closure  : {len(result['closure'])} function(s)")
        for symbol in result["closure"]:
            print(f"    {symbol}")
        print(f"internal edges : {len(result['internal_edges'])}")
        for origin, target in result["internal_edges"]:
            print(f"    {origin} -> {target}")
        print(
            f"syscall sites  : {sum(result['syscall_sites'].values())} instruction(s), "
            f"shim {result['shim_shape']}"
        )
        for symbol, count in sorted(result["syscall_sites"].items()):
            print(f"    {symbol}: {count}")
        print(f"external edges : {len(result['external_edges'])}")
        for origin, target, category in result["external_edges"]:
            print(f"    {origin} -> {target}  [{category}]")
        control = result["control"]
        print(
            f"positive control: {control['witness_count']} witness function(s) in this "
            "assembly do reach a forbidden-category helper"
        )
        for origin, target, category in control["witnesses"]:
            print(f"    witness {origin} -> {target}  [{category}]")
        if control.get("named_root"):
            print(
                f"    named control {control['named_root']} reaches "
                f"{control['named_external_edge_count']} external helper(s)"
            )

    if result["problems"]:
        print("\nCHILD CLOSURE CHECK FAILED:", file=sys.stderr)
        for problem in result["problems"]:
            print(f"  {problem}", file=sys.stderr)
        return 1
    print("\nCHILD CLOSURE CHECK PASSED: no external runtime helper is reachable.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
