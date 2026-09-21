#!/usr/bin/env python3
"""HELM-LAUNCH P3 proof that the test-only fault injection cannot reach a release build.

The injection is gated on **two** conditions — the non-default
`test-fault-injection` feature **and** `debug_assertions` — so that a release
build contains none of it even if the feature is enabled by accident. This
script is the evidence for that, and it is deliberately built so that a wrong
answer is impossible rather than merely unlikely:

* **fresh build roots.** Each build uses its own empty `CARGO_TARGET_DIR`, so no
  artifact from an earlier build with different features can be inspected.
* **exact artifact selection.** The artifact is taken from cargo's own
  `--message-format=json-render-diagnostics` `compiler-artifact` record for the
  `helm-launch` **lib** target of *that* invocation. There is no wildcard, no
  `head -n 1` and no guessing from a directory listing.
* **a positive control.** A debug build with the feature must show the marker
  PRESENT. Without that, "absent" in the release build could just mean the
  checker was looking at the wrong file or for the wrong spelling.
* **every archive member is inspected**, not the first one.
* **loud failure.** A missing artifact, an unreadable archive, an ambiguous
  selection or a failed control is an error, never a quiet pass.

An rlib is a standard `ar` archive; it is parsed here directly, so no external
`ar`, `nm` or `objdump` is required and there is no tool whose absence could be
mistaken for a mechanism result.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

MARKER = b"helm_launch_p3_fault_injection_present"
SYMBOL_HINT = b"PRESENCE_MARKER_KEPT"


class ProofError(RuntimeError):
    """A condition that must fail loudly."""


# --------------------------------------------------------------------------
# `ar` archive reading
# --------------------------------------------------------------------------


def archive_members(data: bytes) -> list[tuple[str, bytes]]:
    """Every member of a GNU `ar` archive, as (name, bytes)."""
    if not data.startswith(b"!<arch>\n"):
        raise ProofError("the artifact is not an `ar` archive; cannot inspect its members")
    members: list[tuple[str, bytes]] = []
    long_names = b""
    offset = 8
    while offset + 60 <= len(data):
        header = data[offset : offset + 60]
        if header[58:60] != b"`\n":
            raise ProofError(f"malformed archive header at offset {offset}")
        raw_name = header[0:16].decode("ascii", "replace").strip()
        try:
            size = int(header[48:58].decode("ascii", "replace").strip())
        except ValueError as error:
            raise ProofError(f"malformed archive member size at offset {offset}") from error
        body = data[offset + 60 : offset + 60 + size]
        if raw_name == "//":
            long_names = body
        elif raw_name.startswith("/") and raw_name[1:].isdigit():
            start = int(raw_name[1:])
            end = long_names.find(b"/\n", start)
            if end < 0:
                end = long_names.find(b"/", start)
            name = long_names[start : end if end >= 0 else len(long_names)].decode(
                "ascii", "replace"
            )
            members.append((name, body))
        elif raw_name not in ("/", "/SYM64/"):
            members.append((raw_name.rstrip("/"), body))
        offset += 60 + size + (size % 2)
    if not members:
        raise ProofError("the archive contains no inspectable member")
    return members


def inspect(artifact: Path) -> dict:
    """Search every member of one artifact for the injection marker."""
    if not artifact.exists():
        raise ProofError(f"the selected artifact does not exist: {artifact}")
    data = artifact.read_bytes()
    if not data:
        raise ProofError(f"the selected artifact is empty: {artifact}")

    if artifact.suffix == ".rlib":
        members = archive_members(data)
    else:
        members = [(artifact.name, data)]

    hits = []
    for name, body in members:
        found_marker = MARKER in body
        found_symbol = SYMBOL_HINT in body
        if found_marker or found_symbol:
            hits.append(
                {"member": name, "marker": found_marker, "symbol": found_symbol}
            )
    return {
        "artifact": str(artifact),
        "bytes": len(data),
        "members": len(members),
        "hits": hits,
        "present": bool(hits),
    }


# --------------------------------------------------------------------------
# Building and selecting
# --------------------------------------------------------------------------


def build(
    manifest_dir: Path, target_dir: Path, profile_release: bool, features: list[str], target: str | None
) -> list[Path]:
    """Build, and return the exact lib artifacts cargo says it produced."""
    if shutil.which("cargo") is None:
        raise ProofError("TEST ENVIRONMENT FAILURE: cargo is not on PATH")
    if target_dir.exists():
        shutil.rmtree(target_dir)

    command = [
        "cargo",
        "build",
        "-p",
        "helm-launch",
        "--lib",
        "--locked",
        "--target-dir",
        str(target_dir),
        "--message-format=json-render-diagnostics",
    ]
    if profile_release:
        command.append("--release")
    for feature in features:
        command.append(feature)
    if target:
        command += ["--target", target]

    finished = subprocess.run(
        command, cwd=manifest_dir, capture_output=True, text=True, check=False
    )
    if finished.returncode != 0:
        raise ProofError(
            "TEST ENVIRONMENT FAILURE: the build failed.\n"
            f"command: {' '.join(command)}\n"
            f"stderr:\n{finished.stderr[-4000:]}"
        )

    selected: list[Path] = []
    for line in finished.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        target_info = message.get("target") or {}
        # Cargo names the lib target after the crate, with the dash replaced, and
        # identifies the package by path. Both are required, so a same-named
        # dependency could not be mistaken for it.
        if target_info.get("name") != "helm_launch":
            continue
        if "lib" not in (target_info.get("kind") or []):
            continue
        package_id = message.get("package_id") or ""
        if "crates/helm-launch" not in package_id.replace("\\", "/"):
            continue
        for filename in message.get("filenames") or []:
            path = Path(filename)
            if path.suffix in (".rlib", ".rmeta"):
                selected.append(path)

    if not selected:
        raise ProofError(
            "cargo reported no compiler-artifact record for the helm-launch lib target; "
            "the artifact cannot be identified, so nothing is proven"
        )
    return selected


def run_case(
    label: str,
    manifest_dir: Path,
    target_dir: Path,
    release: bool,
    features: list[str],
    target: str | None,
) -> dict:
    artifacts = build(manifest_dir, target_dir, release, features, target)
    reports = [inspect(path) for path in artifacts]
    return {
        "label": label,
        "release": release,
        "features": features,
        "artifacts": reports,
        "present": any(report["present"] for report in reports),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-dir", type=Path, default=Path.cwd())
    parser.add_argument("--root", type=Path, required=True, help="a scratch directory for fresh build roots")
    parser.add_argument("--target", default=None, help="an explicit --target triple")
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args()

    arguments.root.mkdir(parents=True, exist_ok=True)

    try:
        control = run_case(
            "debug + test-fault-injection (POSITIVE CONTROL)",
            arguments.manifest_dir,
            arguments.root / "debug-feature",
            release=False,
            features=["--features", "test-fault-injection"],
            target=arguments.target,
        )
        proof = run_case(
            "release + --all-features (NEGATIVE PROOF)",
            arguments.manifest_dir,
            arguments.root / "release-all",
            release=True,
            features=["--all-features"],
            target=arguments.target,
        )
    except ProofError as error:
        print(f"INJECTION PROOF ERROR: {error}", file=sys.stderr)
        return 2

    result = {"control": control, "proof": proof}
    if arguments.json:
        print(json.dumps(result, indent=2))
    else:
        for case in (control, proof):
            print(f"=== {case['label']} ===")
            for report in case["artifacts"]:
                print(
                    f"  artifact {report['artifact']}\n"
                    f"    {report['bytes']} bytes, {report['members']} member(s) inspected, "
                    f"marker {'PRESENT' if report['present'] else 'ABSENT'}"
                )
                for hit in report["hits"]:
                    print(
                        f"      hit in member {hit['member']}"
                        f" (marker={hit['marker']}, symbol={hit['symbol']})"
                    )
            print()

    problems = []
    if not control["present"]:
        problems.append(
            "POSITIVE CONTROL FAILED: a debug build with test-fault-injection does not "
            "contain the marker, so the negative proof below would pass for the wrong reason"
        )
    if proof["present"]:
        problems.append(
            "FAILURE: a release build with --all-features contains the test-only fault injection"
        )

    if problems:
        print("INJECTION PROOF FAILED:", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1

    print(
        "INJECTION PROOF PASSED: present in the debug feature build, absent from the "
        "release --all-features build."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
