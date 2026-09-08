"""Reproduce library/workspace checks, measurements and bounded publication audit.

Development tooling only. No Wine, installer, runtime artifact or lab is accessed.
The existing evidence publication audit is reused with this task's starting main.
"""
from pathlib import Path
import argparse
import hashlib
import json
import platform
import re
import subprocess
import time
import tomllib

import check_helm_evidence as evidence_checks
import helm_app_spec_fixture

ROOT = Path(__file__).resolve().parents[1]
BASE = "e32ab269fbe7c4151186d9257e0aff67c0c70197"


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT).decode().strip()


def audit():
    evidence_checks.BASE = BASE
    result = evidence_checks.audit()
    if git("diff", "--name-only", BASE, "--", "crates/helm-evidence"):
        raise ValueError("Existing evidence crate changed")
    files = sorted((ROOT / "crates/helm-app-spec/src").glob("*.rs"))
    source = "\n".join(p.read_text(encoding="utf-8") for p in files)
    if re.search(r"std::(?:fs|net|process|env|time)|Command::|SystemTime|Instant::|File::|canonicalize|current_dir|read_dir|unsafe\s*\{|getrandom|rand::", source):
        raise ValueError("Unexpected authority API in app-spec product source")
    manifest = tomllib.loads((ROOT / "crates/helm-app-spec/Cargo.toml").read_text())
    if "force-soft" not in manifest["dependencies"]["sha2"]["features"]:
        raise ValueError("SHA-256 must not discover CPU features")
    baseline = tomllib.loads(subprocess.check_output(["git", "show", f"{BASE}:Cargo.lock"], cwd=ROOT).decode())
    current = tomllib.loads((ROOT / "Cargo.lock").read_text())
    prior_packages = {p["name"]: p for p in baseline["package"]}
    current_packages = {p["name"]: p for p in current["package"]}
    if any(current_packages.get(name) != p for name, p in prior_packages.items()):
        raise ValueError("Existing dependency resolution changed")
    if set(current_packages) - set(prior_packages) != {"helm-app-spec"}:
        raise ValueError("Unexpected lockfile package addition")
    all_paths = files + sorted((ROOT / "crates/helm-app-spec/tests").rglob("*.rs"))
    all_paths += sorted((ROOT / "crates/helm-app-spec/tests/fixtures").glob("*.json"))
    all_paths += [ROOT / p for p in ["Cargo.toml", "Cargo.lock", "crates/helm-app-spec/Cargo.toml", "crates/helm-app-spec/benches/parse.rs", ".github/workflows/helm-evidence.yml", "tools/helm_app_spec_fixture.py", "tools/check_helm_app_spec.py", "tools/tests/test_helm_app_spec_fixture.py"]]
    result.update({"app_spec_source_sha256": {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in all_paths},
                   "app_spec_source_authority_scan": "No listed I/O/environment/time/random/unsafe API patterns; software SHA feature required. Static bounded check, not formal proof.",
                   "dependency_delta": {"new_workspace_crates": ["helm-app-spec"], "new_third_party_packages": [], "changed_existing_lock_entries": [], "new_feature": "sha2/force-soft"},
                   "fixture_projection": helm_app_spec_fixture.check()})
    return result


def measurements():
    groups = {}
    for label, folder in [("product", "src"), ("test", "tests"), ("benchmark", "benches")]:
        paths = sorted((ROOT / "crates/helm-app-spec" / folder).glob("*.rs"))
        lines = [line for p in paths for line in p.read_text(encoding="utf-8").splitlines()]
        groups[label] = {"physical_rust_lines_including_comments_and_blanks": len(lines), "nonblank_rust_lines": sum(bool(line.strip()) for line in lines)}
    artifact = ROOT / "target/release/helm_app_spec.rlib"
    if not artifact.exists():
        artifact = ROOT / "target/release/libhelm_app_spec.rlib"
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--locked", "--offline"], cwd=ROOT))
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    package = next(p for p in metadata["packages"] if p["name"] == "helm-app-spec")
    closure = set()
    pending = [package["id"]]
    while pending:
        node = pending.pop()
        if node in closure:
            continue
        closure.add(node)
        pending.extend(d["pkg"] for d in nodes[node]["deps"])
    deps = [{"name": p["name"], "version": p["version"], "license_metadata": p["license"], "features_in_workspace_resolution": nodes[p["id"]]["features"]}
            for p in metadata["packages"] if p["id"] in closure and p["id"] != package["id"]]
    return {"loc": groups, "direct_dependencies": len(package["dependencies"]),
            "release_rlib": {"file": artifact.name, "bytes": artifact.stat().st_size, "meaning": "Rust archive including metadata, excludes transitive artifacts; not a deployed footprint"},
            "resolved_transitive_inventory_including_platform_and_build_packages": deps,
            "peak_memory": "NOT_MEASURED; no reliable per-parse peak facility used"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if not args.output.exists():
        args.output.write_text('{"status":"recording"}\n', encoding="utf-8")
    commands = [
        ["rustc", "-Vv"], ["cargo", "-V"],
        ["cargo", "fmt", "--check"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"],
        ["cargo", "test", "--workspace", "--locked"],
        ["cargo", "build", "--workspace", "--release", "--locked"],
        ["cargo", "check", "-p", "helm-app-spec", "--all-targets", "--all-features", "--locked"],
        ["cargo", "bench", "-p", "helm-app-spec", "--bench", "parse", "--locked"],
        ["python", "-m", "unittest", "discover", "-s", "tools/tests", "-v"],
        ["python", "tools/validate_docs.py"],
        ["python", "tools/helm_evidence_fixture.py"],
        ["python", "tools/helm_app_spec_fixture.py"],
        ["git", "diff", "--check"], ["git", "diff", "--cached", "--check"],
    ]
    results = []
    for command in commands:
        start = time.perf_counter()
        run = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=240)
        streams = {}
        for name, raw in [("stdout", run.stdout), ("stderr", run.stderr)]:
            text = raw.decode("utf-8")
            for path in [str(ROOT), ROOT.as_posix(), str(Path.home()), Path.home().as_posix()]:
                text = text.replace(path, "<repo>" if path in [str(ROOT), ROOT.as_posix()] else "<home>")
            streams[name] = text
        results.append({"argv": command, "exit_code": run.returncode, "wall_seconds": time.perf_counter() - start, **streams})
        print(json.dumps({"argv": command, "exit_code": run.returncode}), flush=True)
    report = {"scope": "App-spec declaration validation and unchanged evidence/repository regressions; no application execution",
              "starting_main": BASE, "head_at_check": git("rev-parse", "HEAD"), "branch": git("branch", "--show-current"),
              "platform": {"system": platform.system(), "release": platform.release(), "version": platform.version(), "machine": platform.machine(), "python": platform.python_version()},
              "commands": results, "measurements": measurements(), "audit": audit(),
              "log_redaction": "Repository root and home directory replaced; no hostname collected"}
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    audit()  # Scan the finished receipt as well as other changed/unignored files.
    return int(any(r["exit_code"] for r in results))


if __name__ == "__main__":
    raise SystemExit(main())
