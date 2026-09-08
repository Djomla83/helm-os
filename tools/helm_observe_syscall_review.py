#!/usr/bin/env python3
"""Independent syscall-level regression for the compiled helm-observe 0.1 code.

This is review instrumentation for candidate `e2a62081`, not an OBS-FS-01 rerun.
It reuses the frozen OBS-FS-01 ptrace tracer read-only, by import, and modifies
no experiment file, no frozen case set and no evidence. It runs the **Rust
product** through its public API and proves the negative claims from the syscall
trace instead of from the returned enum alone.

Unprivileged. Traces only its own child. Synthetic fixtures only: no A0 file, no
Wine, no 7-Zip, no application, no network, no privilege change.

Usage:
    python3 tools/helm_observe_syscall_review.py --driver <path> [--out <json>]
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import sys
import tempfile

REPO = Path(__file__).resolve().parent.parent
FROZEN_HARNESS = REPO / "docs" / "experiments" / "obs-fs-01" / "harness.py"

SUBJECT = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e"

O_PATH = 0o10000000
O_NOFOLLOW = 0o400000
O_CLOEXEC = 0o2000000
RESOLVE_REQUIRED = 0x0F  # NO_XDEV | NO_MAGICLINKS | NO_SYMLINKS | BENEATH
DATA_READS = {"read", "pread64", "readv", "preadv"}
MAX_FILE_BYTES = 512 * 1024 * 1024


def load_frozen_tracer():
    """Import the frozen OBS-FS-01 tracer without copying or changing it."""
    spec = importlib.util.spec_from_file_location("obs_fs_01_harness", FROZEN_HARNESS)
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load frozen tracer at {FROZEN_HARNESS}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_fixtures(root: Path) -> socket.socket:
    """Neutral synthetic fixtures. Nothing here names a real application."""
    root.mkdir(parents=True, exist_ok=True)
    (root / "plain").write_bytes(b"observed payload\n")
    (root / "adir").mkdir(exist_ok=True)
    (root / "realdir").mkdir(exist_ok=True)
    (root / "realdir" / "child").write_bytes(b"child payload\n")
    with open(root / "over", "wb") as handle:      # sparse, metadata over ceiling
        handle.truncate(MAX_FILE_BYTES + 4096)
    perm = root / "permfile"
    perm.write_bytes(b"unreadable payload\n")
    os.chmod(perm, 0o000)
    for name, dest in (("linkint", "plain"), ("linkdir", "realdir")):
        link = root / name
        if link.is_symlink() or link.exists():
            link.unlink()
        link.symlink_to(dest)
    fifo = root / "fifo"
    if fifo.exists():
        fifo.unlink()
    os.mkfifo(fifo)
    sockp = root / "sock"
    if sockp.exists():
        sockp.unlink()
    server = socket.socket(socket.AF_UNIX)
    server.bind(str(sockp))
    return server                                   # keep the socket bound


def plan_json(target: dict) -> str:
    fields = [f'"id":"{target["id"]}"', '"root":"r"']
    if target.get("path") is not None:
        fields.append(f'"path":"{target["path"]}"')
    fields.append(f'"observable":"{target["observable"]}"')
    body = "{" + ",".join(fields) + "}"
    return (
        '{"schema":"helm-observation-plan","version":"0.1",'
        f'"subject_spec_sha256":"{SUBJECT}","roots":[{{"id":"r"}}],'
        f'"targets":[{body}]}}'
    )


CASES = [
    # name, target, expected outcome code, expected reopen count, reads allowed
    ("regular", {"id": "t", "path": "plain", "observable": "regular_file_sha256"},
     "observed_file", 1, True),
    ("directory", {"id": "t", "path": "adir", "observable": "directory_metadata"},
     "observed_directory", 0, False),
    ("trailing_symlink", {"id": "t", "path": "linkint", "observable": "regular_file_sha256"},
     "symlink_forbidden", 0, False),
    ("nonfinal_symlink", {"id": "t", "path": "linkdir/child",
                          "observable": "regular_file_sha256"},
     "symlink_forbidden", 0, False),
    ("fifo", {"id": "t", "path": "fifo", "observable": "regular_file_sha256"},
     "special_file", 0, False),
    ("socket", {"id": "t", "path": "sock", "observable": "regular_file_sha256"},
     "special_file", 0, False),
    ("metadata_over_limit", {"id": "t", "path": "over", "observable": "regular_file_sha256"},
     "file_limit", 0, False),
    ("permission_denied", {"id": "t", "path": "permfile", "observable": "regular_file_sha256"},
     "permission_denied", 1, False),
    ("absent", {"id": "t", "path": "nothing-here", "observable": "regular_file_sha256"},
     "absent", 0, False),
]


def analyse(case, record, root_fd, proc_fd):
    """Derive the safety properties from the trace, not from the returned enum."""
    trace = record["trace"]
    findings = []
    first_openat2 = next((i for i, e in enumerate(trace) if e["syscall"] == "openat2"), None)
    post = trace if first_openat2 is None else trace[first_openat2:]

    openat2s = [e for e in post if e["syscall"] == "openat2"]
    for event in openat2s:
        if event["dirfd"] != root_fd:
            findings.append(f"openat2 used dirfd {event['dirfd']}, not the authorised root")
        flags = event.get("flags", 0)
        for bit, label in ((O_PATH, "O_PATH"), (O_NOFOLLOW, "O_NOFOLLOW"), (O_CLOEXEC, "O_CLOEXEC")):
            if not flags & bit:
                findings.append(f"target resolution missing {label}")
        if event.get("resolve", 0) != RESOLVE_REQUIRED:
            findings.append(
                f"resolve flags {event.get('resolve')} != required {RESOLVE_REQUIRED}"
            )

    pinned = {e["result"] for e in openat2s if e["result"] >= 0}
    reopens = [e for e in post if e["syscall"] == "openat" and e["dirfd"] == proc_fd]
    for event in reopens:
        name = event.get("path", "")
        if not name.isdigit():
            findings.append(f"procfs reopen named {name!r}, which is not a descriptor number")
        elif int(name) not in pinned:
            findings.append(f"procfs reopen named descriptor {name}, which was never pinned")
    data_fds = {e["result"] for e in reopens if e["result"] >= 0}
    reads = [e for e in post if e["syscall"] in DATA_READS and e.get("fd") in data_fds]
    pinned_reads = [e for e in post if e["syscall"] in DATA_READS and e.get("fd") in pinned]

    # No pathname fallback: after resolution begins, the only pathname opens are
    # descriptor numbers under the procfs capability.
    for event in post:
        if event["syscall"] == "openat" and event["dirfd"] != proc_fd:
            findings.append(
                f"pathname open {event.get('path')!r} on dirfd {event['dirfd']} after resolution"
            )
    for event in trace:
        if event["syscall"] == "getdents64":
            findings.append("directory enumeration occurred")
        if event["syscall"] == "connect":
            findings.append("connect occurred")
    if pinned_reads:
        findings.append("a read was issued against the O_PATH pin itself")

    name, _target, expect_code, expect_reopens, reads_allowed = case
    if record["timeout"]:
        findings.append("the observation did not complete: it blocked")
    observation = next((r for r in record["results"] if r.get("kind") == "observation"), None)
    if observation is None:
        findings.append(f"no observation result; driver output {record['results']}")
    else:
        got = observation["targets"][0]["code"]
        if got != expect_code:
            findings.append(f"outcome {got!r} != expected {expect_code!r}")
    if len(reopens) != expect_reopens:
        findings.append(f"{len(reopens)} procfs data reopens, expected {expect_reopens}")
    if not reads_allowed and reads:
        findings.append(f"{len(reads)} data reads reached the target, expected none")
    if reads_allowed and not reads:
        findings.append("expected at least one data read on the reopened descriptor")

    return {
        "case": name,
        "openat2_calls": len(openat2s),
        "openat2_flags_octal": [oct(e.get("flags", 0)) for e in openat2s],
        "openat2_resolve": [e.get("resolve") for e in openat2s],
        "openat2_results": [e["result"] for e in openat2s],
        "procfs_reopens": [
            {"name": e.get("path"), "result": e["result"]} for e in reopens
        ],
        "target_reads": len(reads),
        "target_read_bytes": sum(max(e["result"], 0) for e in reads),
        "getdents64": sum(1 for e in trace if e["syscall"] == "getdents64"),
        "connect": sum(1 for e in trace if e["syscall"] == "connect"),
        "statx_calls": sum(1 for e in post if e["syscall"] == "statx"),
        "seconds": record["seconds"],
        "results": record["results"],
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--driver", required=True, help="compiled review driver binary")
    parser.add_argument("--out", default=None, help="write the evidence JSON here")
    parser.add_argument("--timeout", type=float, default=25.0)
    args = parser.parse_args()

    if sys.platform != "linux":
        print("helm-observe syscall review runs on Linux only", file=sys.stderr)
        return 3
    driver = Path(args.driver).resolve()
    if not driver.is_file() or not os.access(driver, os.X_OK):
        print(f"driver {driver} is not executable", file=sys.stderr)
        return 3

    harness = load_frozen_tracer()
    workdir = Path(tempfile.mkdtemp(prefix="helm-observe-review-"))
    root = workdir / "root"
    keepalive = build_fixtures(root)

    mounted_type = "unknown"
    for line in Path("/proc/self/mountinfo").read_text(encoding="utf-8").splitlines():
        left, _, right = line.partition(" - ")
        parts = left.split()
        if len(parts) > 4 and str(root).startswith(parts[4]):
            mounted_type = right.split()[0]

    report = {
        "role": "independent review syscall regression",
        "candidate": "e2a62081ece52391b30ede153eee139103c98e31",
        "driver": str(driver),
        "kernel": os.uname().release,
        "machine": os.uname().machine,
        "fixture_mounted_fstype": mounted_type,
        "tracer": "docs/experiments/obs-fs-01/harness.py (imported read-only)",
        "cases": [],
    }

    violations = 0
    for case in CASES:
        name, target, *_ = case
        record = harness.run_traced(
            [str(driver), str(root), plan_json(target)], timeout=args.timeout
        )
        caps = next((r for r in record["results"] if r.get("kind") == "capabilities"), None)
        if caps is None:
            report["cases"].append({
                "case": name,
                "findings": [f"driver produced no capability line: {record['results']}"],
            })
            violations += 1
            continue
        summary = analyse(case, record, caps["root_fd"], caps["proc_fd"])
        report["cases"].append(summary)
        violations += len(summary["findings"])

    os.chmod(root / "permfile", 0o600)
    keepalive.close()
    shutil.rmtree(workdir, ignore_errors=True)

    report["violations"] = violations
    text = json.dumps(report, indent=2, sort_keys=True)
    if args.out:
        out = Path(args.out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text + "\n", encoding="utf-8")
    print(text)
    for case in report["cases"]:
        for finding in case["findings"]:
            print(f"VIOLATION [{case['case']}] {finding}", file=sys.stderr)
    print(
        "HELM-OBSERVE-SYSCALL-REVIEW: "
        + ("PASS" if violations == 0 else f"FAIL ({violations} violations)")
    )
    return 0 if violations == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
