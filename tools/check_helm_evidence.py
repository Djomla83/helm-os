"""Run the bounded module's offline checks and publication audit; retain a JSON receipt.

Development tooling only. Does not run Wine, change labs, or execute bundle contents.
"""
from __future__ import annotations
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
BASE = "ae3f012fb1bfd7b20018c3faf6c71a6740a041fe"


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT).decode("utf-8").strip()


def audit():
    unchanged = ["docs/experiments", "docs/adr", "tools/app_baseline.py", "tools/fixtures/exp009-7zip.json"]
    if git("diff", "--name-only", BASE, "--", *unchanged):
        raise ValueError("Historical experiment, ADR or oracle files changed")
    source = ROOT / "docs/experiments/evidence/app-baseline-execution-2026-09-07"
    manifest = json.loads((source / "publication-manifest.json").read_bytes())
    for item in manifest["artifacts"]:
        raw = (source / item["file"]).read_bytes()
        if hashlib.sha256(raw).hexdigest() != item["publication_sha256"]:
            raise ValueError("Historical published artifact hash mismatch")
    names = sorted(set(git("diff", "--name-only", BASE).splitlines())
                   | set(git("ls-files", "--others", "--exclude-standard").splitlines()))
    forbidden_suffixes = {".exe", ".dll", ".pdb", ".rlib", ".rmeta", ".obj", ".zip", ".tar", ".iso", ".vhdx", ".key", ".pem"}
    privacy = [str(ROOT), str(Path.home())]
    account = os.environ.get("USERNAME", "")
    if account:
        privacy.extend(["C:\\Users\\" + account, "C:/Users/" + account])
    markers = []
    for value in privacy:
        for variant in [value, value.replace("\\", "/"), value.replace("\\", "\\\\"), value.replace("\\", "\\\\\\\\")]:
            markers.append(variant.casefold())
    checked = []
    decoded_streams = 0

    def decoded(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key in {"stdout_base64", "stderr_base64"} and isinstance(item, str):
                    yield base64.b64decode(item, validate=True)
                else:
                    yield from decoded(item)
        elif isinstance(value, list):
            for item in value:
                yield from decoded(item)

    for name in names:
        path = ROOT / name
        if not path.is_file():
            continue
        if any(part in {"target", "private", "secrets"} for part in path.relative_to(ROOT).parts) or path.suffix.lower() in forbidden_suffixes:
            raise ValueError("Generated/private/binary file entered the publication set: " + name)
        raw = path.read_bytes()
        texts = [raw]
        if path.suffix == ".json":
            streams = list(decoded(json.loads(raw)))
            decoded_streams += len(streams)
            texts.extend(streams)
        for data in texts:
            text = data.decode("utf-8").casefold()
            if any(marker in text for marker in markers):
                raise ValueError("Private host path found in: " + name)
            if re.search(r"-----begin (?:rsa |ec |openssh )?private key-----|gh[pousr]_[a-z0-9]{30,}|sk-proj-[a-z0-9_-]{20,}", text):
                raise ValueError("Credential-like marker found in: " + name)
        checked.append(name)
    product = "\n".join(p.read_text(encoding="utf-8") for p in (ROOT / "crates/helm-evidence/src").glob("*.rs"))
    if re.search(r"Command::|std::net|reqwest|\bunsafe\s*\{|File::create|fs::write|\.write\(true\)", product):
        raise ValueError("Product source contains an unexpected side-effect API")
    if git("ls-files", "target"):
        raise ValueError("Cargo target output is tracked")
    source_paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "crates/helm-evidence/Cargo.toml"]
    source_paths.extend(sorted((ROOT / "crates/helm-evidence/src").glob("*.rs")))
    source_hashes = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
    index_fixtures = 0
    if git("diff", "--cached", "--name-only"):
        for fixture in ["a0-7zip", "synthetic"]:
            folder = f"crates/helm-evidence/tests/fixtures/{fixture}"
            contract = json.loads((ROOT / folder / "bundle.json").read_bytes())
            for artifact in contract["artifacts"]:
                raw = subprocess.check_output(["git", "show", f":{folder}/{artifact['path']}"], cwd=ROOT)
                if hashlib.sha256(raw).hexdigest() != artifact["sha256"]:
                    raise ValueError("Git index changes fixture artifact bytes")
                index_fixtures += 1
    return {"historical_publication_hashes_checked": len(manifest["artifacts"]),
            "historical_sources_unchanged_against": BASE, "publication_files_scanned": checked,
            "decoded_command_streams_scanned": decoded_streams,
            "git_index_fixture_artifacts_checked": index_fixtures,
            "product_source_sha256": source_hashes,
            "privacy_scan_scope": "Changed and new unignored files, UTF-8 and decoded stdout/stderr; current host root/home/account paths plus common credential markers; not a comprehensive secret detector",
            "product_side_effect_api_scan": "No command/network/file-write/unsafe-block API pattern in product src"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    # Ensure the documentation scan sees the same output filename/count on its first run.
    if not args.output.exists():
        args.output.write_text('{"status":"recording"}\n', encoding="utf-8")
    commands = [
        ["cargo", "fmt", "--all", "--", "--check"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"],
        ["cargo", "test", "--workspace", "--locked", "--offline"],
        ["python", "tools/validate_docs.py"],
        ["python", "-m", "unittest", "discover", "-s", "tools/tests", "-v"],
        ["python", "tools/helm_evidence_fixture.py"],
        ["git", "diff", "--check"],
        ["git", "diff", "--cached", "--check"],
    ]
    results = []
    for command in commands:
        start = time.perf_counter()
        run = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
        streams = {}
        for name, raw in [("stdout", run.stdout), ("stderr", run.stderr)]:
            value = raw.decode("utf-8")
            for root in [str(ROOT), ROOT.as_posix()]:
                value = value.replace(root, "<repo>")
            streams[name] = value
        results.append({"argv": command, "exit_code": run.returncode,
                        "wall_seconds": time.perf_counter() - start, **streams})
        print(json.dumps({"argv": command, "exit_code": run.returncode}))
    report = {"scope": "Verifier implementation and repository checks; no application rerun",
              "head_at_check": git("rev-parse", "HEAD"), "branch": git("branch", "--show-current"),
              "log_redaction": "Absolute repository root replaced with <repo>; output otherwise retained",
              "commands": results, "audit": audit()}
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    sys.exit(main())
