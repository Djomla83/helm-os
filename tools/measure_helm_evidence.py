"""Baseline the already-built verifier CLI; never run an application experiment.

Alternates checked-in fixtures after three warmups each. Wall time includes startup
and piped JSON output. Windows peak working set is queried on the child process handle.
"""
from __future__ import annotations
import argparse
import ctypes
import json
import platform
from pathlib import Path
import statistics
import subprocess
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def output(args):
    return subprocess.check_output(args, cwd=ROOT, text=True, encoding="utf-8").strip()


def peak_working_set(process):
    if platform.system() != "Windows":
        return None
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in ["PeakWorkingSetSize", "WorkingSetSize",
                "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
                "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage"]]

    query = ctypes.WinDLL("kernel32", use_last_error=True).K32GetProcessMemoryInfo
    query.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
    query.restype = wintypes.BOOL
    counters = Counters()
    counters.cb = ctypes.sizeof(counters)
    if query(wintypes.HANDLE(int(process._handle)), ctypes.byref(counters), counters.cb):
        return counters.PeakWorkingSetSize
    return None


def run(binary, fixture, expected):
    start = time.perf_counter_ns()
    process = subprocess.Popen([str(binary), "verify", f"crates/helm-evidence/tests/fixtures/{fixture}", "--json"],
                               cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate(timeout=30)
    elapsed_ms = (time.perf_counter_ns() - start) / 1_000_000
    peak = peak_working_set(process)
    report = json.loads(stdout)
    if process.returncode != expected or stderr or report["verdict"] != ("COMPLETE" if expected == 0 else "INCOMPLETE"):
        raise RuntimeError("Measured verifier returned an unexpected result")
    return {"wall_ms": elapsed_ms, "peak_working_set_bytes": peak, "exit_code": process.returncode}


def measure(binary, samples):
    rustc = output(["rustc", "-Vv"])
    host = next(line.removeprefix("host: ") for line in rustc.splitlines() if line.startswith("host: "))
    metadata = json.loads(output(["cargo", "metadata", "--format-version", "1", "--locked", "--offline", "--filter-platform", host]))
    active = {node["id"] for node in metadata["resolve"]["nodes"]}
    inventory = [{k: p[k] for k in ["name", "version", "license", "repository", "source"]}
                 for p in metadata["packages"] if p["id"] in active and p["name"] != "helm-evidence"]
    inventory.sort(key=lambda p: (p["name"], p["version"]))
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    fixture_samples = {"synthetic": [], "a0-7zip": []}
    for _ in range(3):
        for fixture, expected in [("synthetic", 0), ("a0-7zip", 1)]:
            run(binary, fixture, expected)
    for _ in range(samples):
        for fixture, expected in [("synthetic", 0), ("a0-7zip", 1)]:
            fixture_samples[fixture].append(run(binary, fixture, expected))
    results = {}
    for fixture, measurements in fixture_samples.items():
        values = [m["wall_ms"] for m in measurements]
        peaks = [m["peak_working_set_bytes"] for m in measurements if m["peak_working_set_bytes"] is not None]
        results[fixture] = {"median_wall_ms": statistics.median(values), "min_wall_ms": min(values),
                            "max_wall_ms": max(values), "max_peak_working_set_bytes": max(peaks) if peaks else None,
                            "samples": measurements}
    loc = {}
    for path in sorted((ROOT / "crates/helm-evidence").rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").splitlines()
        loc[path.relative_to(ROOT).as_posix()] = {"physical_lines": len(lines),
            "nonblank_non_line_comment_lines": sum(bool(line.strip()) and not line.lstrip().startswith("//") for line in lines)}
    return {"measurement_kind": "Baseline, not a performance target or application measurement",
            "method": "Default release CLI wall time including startup, verification, JSON formatting and piped stdout; warm filesystem cache; 3 warmups per fixture; alternating measured runs; no affinity or background-load control",
            "memory_method": "Windows K32GetProcessMemoryInfo on child handle after communicate; peak working set, not private heap; null if unavailable",
            "platform": platform.platform(), "machine": platform.machine(), "rustc": rustc,
            "cargo": output(["cargo", "-V"]), "python": platform.python_version(),
            "build_mode": "cargo build --release --locked --offline; default release profile",
            "binary_bytes": binary.stat().st_size, "direct_dependencies": 4,
            "active_target_dependencies_including_transitive_and_build": len(inventory),
            "lockfile_dependencies_all_targets": len(lock["package"]) - 1,
            "dependency_inventory": inventory, "rust_loc": loc,
            "rust_physical_lines": sum(v["physical_lines"] for v in loc.values()),
            "rust_nonblank_non_line_comment_lines": sum(v["nonblank_non_line_comment_lines"] for v in loc.values()),
            "results": results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.samples <= 1000:
        parser.error("samples must be 1..1000")
    report = measure(args.binary.resolve(), args.samples)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"binary_bytes": report["binary_bytes"], "rust_physical_lines": report["rust_physical_lines"],
        "rust_nonblank_non_line_comment_lines": report["rust_nonblank_non_line_comment_lines"],
        "active_dependencies": report["active_target_dependencies_including_transitive_and_build"],
        "lockfile_dependencies": report["lockfile_dependencies_all_targets"],
        "fixtures": {k: {key: value for key, value in v.items() if key != "samples"} for k, v in report["results"].items()}}, indent=2))


if __name__ == "__main__":
    main()
