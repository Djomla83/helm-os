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
closure, a missing syscall shim, a failed positive control, and **any
function-escaping control transfer whose target this parser cannot resolve** are
all errors, so "zero matches" can never mean "clean" because the parser found
nothing.

Control transfers are analysed as a closed model, not as a `call` pattern
(owner disposition of P3R-11, 2026-09-19). `call`, `jmp`, **every conditional
`Jcc`** and **every `loop` branch** can leave a function on x86_64, so all of
them are classified against one explicit, reviewable branch vocabulary, and
every operand form is placed in exactly one of four buckets:

    named target        `callq foo`, `jmp foo`, `jno foo@PLT`,
                        `callq *foo@GOTPCREL(%rip)`
                        -> a call-graph edge, traversed transitively whether it
                           is a call, a tail jump or a conditional branch
    local label         `jmp .LBB0_3`, `je .LBB0_3`, where `.LBB0_3` is defined
                        in the body being analysed
                        -> intra-function control flow
    indirect            `jmp *%rax`, `callq *(%rax)`, `jmpq *.LJTI0_0(,%rax,8)`
                        -> UNRESOLVED, FAIL CLOSED
    anything else       an operand form this parser does not understand, or a
                        local label the body does not define
                        -> UNRESOLVED, FAIL CLOSED

No resolver for an indirect target is authorised. A tail jump to `memcpy`, a
panic helper or an allocator therefore fails exactly as the corresponding call
does, and a jump through a register or a jump table fails because the target is
unknown rather than because it was recognised as forbidden.

**A conditional branch is a function-escaping transfer too** (owner disposition
of P3R-15, 2026-09-19). The earlier model assumed rustc emits `Jcc` only to a
label of the same function; that assumption is false, and this crate's own
release-codegen assembly contains

    jno  _ZN4core3ptr56drop_in_place$LT$…Report$GT$17h…E

a conditional **tail branch to another function**. A conditional edge is a
possible execution edge, so it enters the closure and is traversed exactly like
a call, and `jno memcpy@PLT` fails exactly like `callq memcpy@PLT`.

The branch vocabulary below is explicit and enumerated rather than inferred, so
it can be reviewed against the ISA. A mnemonic that is recognisably a branch but
is **not** in that vocabulary is a **failure**, never an ordinary instruction:
"the pattern did not match" must not mean "nothing is there".

Outside the model, by construction: `ret`, `ud2`, `int3` and `hlt` end the
current flow without naming another function, so they extend no closure. A
control transfer hand-assembled out of `push`/`ret` would be invisible to any
mnemonic-level model; the crate's only `asm!` block is the reviewed syscall
shim, which contains no branch at all, and `tests/p3_boundary.rs` pins that.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

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

# --------------------------------------------------------------------------
# The branch vocabulary
# --------------------------------------------------------------------------
#
# Every mnemonic that can move control out of the current function on x86_64,
# enumerated explicitly so the set can be reviewed against the ISA rather than
# inferred from a pattern (owner disposition of P3R-15).

CALL_MNEMONICS = frozenset({"call", "callq", "calll", "callw", "lcall"})

UNCONDITIONAL_JUMP_MNEMONICS = frozenset({"jmp", "jmpq", "jmpl", "jmpw", "ljmp"})

# The canonical Jcc vocabulary, including every documented alias, plus the three
# counter-register branches. A conditional branch to another function is a real
# execution edge (P3R-15), so these are classified exactly like `jmp`.
CONDITIONAL_JUMP_MNEMONICS = frozenset(
    {
        "ja", "jae", "jb", "jbe", "jc", "je", "jg", "jge", "jl", "jle",
        "jna", "jnae", "jnb", "jnbe", "jnc", "jne", "jng", "jnge", "jnl", "jnle",
        "jno", "jnp", "jns", "jnz", "jo", "jp", "jpe", "jpo", "js", "jz",
        "jcxz", "jecxz", "jrcxz",
    }
)

LOOP_MNEMONICS = frozenset({"loop", "loope", "loopz", "loopne", "loopnz"})

#: Every mnemonic the model classifies. Anything outside it that still looks
#: like a branch fails closed instead of being read as an ordinary instruction.
ESCAPING_MNEMONICS = (
    CALL_MNEMONICS
    | UNCONDITIONAL_JUMP_MNEMONICS
    | CONDITIONAL_JUMP_MNEMONICS
    | LOOP_MNEMONICS
)

#: Instruction-prefix tokens that may precede a branch and carry no target.
PREFIX = (
    r"(?:(?:bnd|notrack|rep|repe|repz|repne|repnz|lock|data16|addr32|rex64"
    r"|ds|es|cs|ss|fs|gs)\s+)*"
)
#: One instruction: its mnemonic and the rest of the line. Directives are not
#: matched, because a mnemonic may not begin with `.`.
INSTRUCTION = re.compile(rf"^\s+{PREFIX}([A-Za-z][\w.]*)\b[ \t]*(.*)$")

SYSCALL = re.compile(r"^\s*(?:syscall|sysenter)\b")


def is_branch_like(mnemonic: str) -> bool:
    """Whether a mnemonic could be a branch, used only to fail closed.

    This is the conservative backstop, **not** the authority: the authority is
    `ESCAPING_MNEMONICS` above. No x86 mnemonic that is not a jump begins with
    `j`, and none that is not a call or a jump contains `call` or `jmp`, so a
    mnemonic matching this predicate and absent from the vocabulary is an
    unsupported control transfer and therefore a failure (P3R-15).
    """
    return (
        mnemonic.startswith("j")
        or mnemonic.startswith("loop")
        or "call" in mnemonic
        or "jmp" in mnemonic
    )

# `foo`, `foo@PLT`: a target named directly.
NAMED_TARGET = re.compile(r"^([A-Za-z_$.][\w$.@]*)$")
# `*foo@GOTPCREL(%rip)`, `*foo(%rip)`: still a **named** target, reached through
# the GOT rather than a relocation against the symbol itself.
GOT_TARGET = re.compile(r"^\*([A-Za-z_$.][\w$.@]*)\(%rip\)$")


def instruction_of(line: str) -> str:
    """One assembly line without its end-of-line comment.

    `#APP` / `#NO_APP` and ordinary `#` comments are dropped, so a comment can
    neither hide nor invent a control transfer.
    """
    for marker in ("#", "//"):
        at = line.find(marker)
        if at >= 0:
            line = line[:at]
    return line.rstrip()


class Edges(NamedTuple):
    """Every control transfer leaving one function body, classified."""

    #: Function-escaping transfers whose target is named, call and tail jump
    #: alike. Each is followed transitively.
    targets: list[str]
    #: `syscall` instructions in this body.
    syscalls: int
    #: Transfers through a register, memory or a jump table. Never resolvable
    #: here, so each one fails the gate.
    indirect: list[str]
    #: Escaping transfers this parser cannot resolve at all, including a local
    #: label the body does not define. Each one fails the gate.
    unresolved: list[str]
    #: Intra-function branches to a label this body defines.
    local: int
    #: Branch-like mnemonics outside the explicit vocabulary. Each one fails the
    #: gate rather than being read as an ordinary instruction (P3R-15).
    unsupported: list[str]


class Assembly:
    """Function bodies and their outgoing edges, from one `--emit=asm` file."""

    def __init__(self, text: str) -> None:
        self.lines = text.splitlines()
        self.bodies: dict[str, list[str]] = {}
        # The local labels each body defines. A `jmp` to one of them is
        # intra-function control flow; a `jmp` to a local label the body does
        # not define is an unresolved escape, not a branch to ignore.
        self.local_labels: dict[str, set[str]] = {}
        current: str | None = None
        for line in self.lines:
            label = LABEL.match(line)
            if label:
                name = label.group(1)
                if not name.startswith(".L"):
                    current = name
                    self.bodies.setdefault(current, [])
                    self.local_labels.setdefault(current, set())
                else:
                    # A local label inside the current body.
                    if current is not None:
                        self.bodies[current].append(line)
                        self.local_labels[current].add(name)
                continue
            if current is None:
                continue
            if SIZE_DIRECTIVE.match(line):
                current = None
                continue
            self.bodies[current].append(line)

    def defined(self, symbol: str) -> bool:
        return symbol in self.bodies

    def edges(self, symbol: str) -> Edges:
        """Classify every control transfer leaving one function body."""
        targets: list[str] = []
        indirect: list[str] = []
        unresolved: list[str] = []
        unsupported: list[str] = []
        syscalls = 0
        local = 0
        labels = self.local_labels.get(symbol, set())
        for line in self.bodies.get(symbol, []):
            if SYSCALL.match(line):
                syscalls += 1
                continue
            instruction = INSTRUCTION.match(instruction_of(line))
            if not instruction:
                continue
            mnemonic = instruction.group(1)
            if mnemonic not in ESCAPING_MNEMONICS:
                # A branch this vocabulary does not model may still escape the
                # function, so it is reported rather than read as ordinary code.
                if is_branch_like(mnemonic):
                    unsupported.append(line.strip())
                continue
            operand = instruction.group(2).strip()
            if not operand:
                # A transfer with no operand at all. It may still escape, so it
                # is never dropped.
                unresolved.append(line.strip())
                continue
            got = GOT_TARGET.match(operand)
            if got:
                targets.append(got.group(1).split("@")[0])
                continue
            named = NAMED_TARGET.match(operand)
            if named:
                target = named.group(1)
                if target.startswith(".L"):
                    if target in labels:
                        local += 1
                    else:
                        unresolved.append(line.strip())
                    continue
                targets.append(target.split("@")[0])
                continue
            # A register, memory or jump-table operand. `*` marks it indirect
            # explicitly; any other shape is an operand form this parser does
            # not understand. Both fail closed, and neither is authorised to be
            # resolved here.
            if operand.startswith("*"):
                indirect.append(line.strip())
            else:
                unresolved.append(line.strip())
        return Edges(targets, syscalls, indirect, unresolved, local, unsupported)

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
    unresolved_sites: list[str] = []
    unsupported_sites: list[str] = []
    syscall_sites: dict[str, int] = {}
    internal_edges: list[tuple[str, str]] = []
    local_branches = 0

    while pending:
        symbol = pending.pop(0)
        if symbol in seen:
            continue
        seen.append(symbol)
        edges = asm.edges(symbol)
        if edges.syscalls:
            syscall_sites[symbol] = edges.syscalls
        local_branches += edges.local
        for instruction in edges.indirect:
            indirect_sites.append(f"{symbol}: {instruction}")
        for instruction in edges.unresolved:
            unresolved_sites.append(f"{symbol}: {instruction}")
        for instruction in edges.unsupported:
            unsupported_sites.append(f"{symbol}: {instruction}")
        # A named target is a call-graph edge whether it was reached by `call`
        # or by a tail `jmp`, so both are followed transitively.
        for target in edges.targets:
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
        "indirect_transfer_sites": sorted(set(indirect_sites)),
        "unresolved_transfer_sites": sorted(set(unresolved_sites)),
        "unsupported_transfer_sites": sorted(set(unsupported_sites)),
        "local_branches": local_branches,
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
    # Every function-escaping control transfer the parser could not resolve is a
    # failure, whether it was a `call` or a tail `jmp`. No resolver is
    # authorised, so there is no path by which one of these is accepted.
    if result["indirect_transfer_sites"]:
        problems.append(
            "indirect control transfer(s) in the child closure, target unresolvable: "
            + "; ".join(result["indirect_transfer_sites"])
        )
    if result["unresolved_transfer_sites"]:
        problems.append(
            "unresolved control transfer(s) in the child closure: "
            + "; ".join(result["unresolved_transfer_sites"])
        )
    if result["unsupported_transfer_sites"]:
        problems.append(
            "branch mnemonic(s) outside the modelled vocabulary in the child closure: "
            + "; ".join(result["unsupported_transfer_sites"])
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
        for target in asm.edges(symbol).targets:
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
        print(f"local branches : {result['local_branches']} intra-function")
        print(f"indirect transfers  : {len(result['indirect_transfer_sites'])}")
        for site in result["indirect_transfer_sites"]:
            print(f"    {site}")
        print(f"unresolved transfers: {len(result['unresolved_transfer_sites'])}")
        for site in result["unresolved_transfer_sites"]:
            print(f"    {site}")
        print(f"unsupported branches : {len(result['unsupported_transfer_sites'])}")
        for site in result["unsupported_transfer_sites"]:
            print(f"    {site}")
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
    print(
        "\nCHILD CLOSURE CHECK PASSED: every control transfer in the closure resolves, "
        "and no external runtime helper is reachable."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
