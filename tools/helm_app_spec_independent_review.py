"""Bounded independent review checks; all generated projects/data remain in target/.

Run on Windows and the existing hosted Ubuntu workflow. Uses public historical Git
blobs and synthetic bytes only. No installer, runtime, application or lab executes.
"""
from pathlib import Path
import argparse
import hashlib
import io
import json
import platform
import shutil
import statistics
import subprocess
import tarfile
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]
BASE = "e32ab269fbe7c4151186d9257e0aff67c0c70197"
CANDIDATE = "18c7aa5e80617483b6f5afd0fea4affeb62f926c"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    work = ROOT / "target/helm-app-spec-independent"
    work.mkdir(parents=True, exist_ok=True)
    receipt = {"candidate": CANDIDATE, "base": BASE, "platform": {
        "system": platform.system(), "release": platform.release(),
        "machine": platform.machine(), "python": platform.python_version()}, "commands": []}

    def save():
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8", newline="\n")

    def run(argv, cwd=ROOT, expected=0, timeout=240):
        start = time.perf_counter()
        result = subprocess.run([str(a) for a in argv], cwd=cwd, capture_output=True, timeout=timeout)
        out = result.stdout.decode("utf-8", errors="replace")
        err = result.stderr.decode("utf-8", errors="replace")
        def redact(text):
            return text.replace(str(ROOT), "<repo>").replace(ROOT.as_posix(), "<repo>").replace(str(Path.home()), "<home>")
        receipt["commands"].append({"argv": [redact(str(a)) for a in argv], "cwd": redact(str(cwd)),
            "exit_code": result.returncode, "expected_exit": expected,
            "wall_seconds": time.perf_counter() - start, "stdout": redact(out), "stderr": redact(err)})
        save()
        print(json.dumps({"command": [redact(str(a)) for a in argv], "exit_code": result.returncode}), flush=True)
        assert result.returncode == expected, redact(out + err)
        return out

    receipt["head_at_check"] = run(["git", "rev-parse", "HEAD"]).strip()
    receipt["rustc"] = run(["rustc", "-Vv"])
    if platform.system() == "Linux":
        receipt["linux_cpu"] = run(["lscpu"])
    receipt["workspace_sha2_features"] = run(["cargo", "tree", "--workspace", "--locked", "-e", "features", "-i", "sha2"])
    receipt["standalone_sha2_features"] = run(["cargo", "tree", "-p", "helm-evidence", "--locked", "-e", "features", "-i", "sha2"])
    assert 'feature "force-soft"' in receipt["workspace_sha2_features"]
    assert 'feature "force-soft"' not in receipt["standalone_sha2_features"]
    tests = run(["cargo", "test", "-p", "helm-app-spec", "--test", "independent_review", "--locked", "--", "--nocapture"], timeout=180)
    for label in ["IDENTITIES", "PROPERTIES"]:
        line = next(line.split("=", 1)[1] for line in tests.splitlines() if line.startswith("REVIEW_" + label + "="))
        receipt[label.lower()] = json.loads(line)
    notes = (ROOT / "crates/helm-app-spec/tests/fixtures/synthetic-notes.json").read_bytes().decode("utf-8")
    variants = [notes, notes.replace("  ", "\t"), notes.replace('"schema": "helm-app-spec",\n  "version": "0.1"', '"version": "0.1",\n  "schema": "helm-app-spec"', 1),
                notes.replace("synthetic-notes", "synthetic-\\u006eotes"), notes.rstrip("\n")]
    assert receipt["identities"] == [hashlib.sha256(s.encode()).hexdigest() for s in variants]
    receipt["exact_byte_python_hashlib_comparison"] = "PASS: whitespace, field order, escaped string, final newline"

    api = work / "api-rejection"
    (api / "src").mkdir(parents=True, exist_ok=True)
    (api / "Cargo.toml").write_text('[package]\nname="review-api-rejection"\nversion="0.0.0"\nedition="2024"\npublish=false\n[workspace]\n[dependencies]\n'
        f'helm-app-spec={{path="{(ROOT / "crates/helm-app-spec").as_posix()}"}}\nserde_json="=1.0.149"\n', encoding="utf-8")
    shutil.copyfile(ROOT / "Cargo.lock", api / "Cargo.lock")
    attempts = [
        ("private validated fields and empty collections", "E0451", '''use helm_app_spec::*;
fn forge(s: ValidatedAppSpec) -> ValidatedAppSpec { ValidatedAppSpec {
spec_sha256:s.spec_sha256().clone(), application:s.application().clone(),
runtime:RuntimeRequirement {family:RuntimeFamily::Wine,artifacts:vec![]},
environment:s.environment().clone(),entry_point:s.entry_point().clone(),verification:vec![] } }
fn main() {}'''),
        ("unchecked newtypes", "E0423", 'use helm_app_spec::*; fn main() { let _ = Digest("!".into()); let _ = Identifier("!".into()); let _ = RelativeEntryPoint("../escape".into()); }'),
        ("mutation through borrowed views", "E0599", 'use helm_app_spec::*; fn mutate(s: &mut ValidatedAppSpec) { s.runtime().artifacts().clear(); s.verification().clear(); } fn main() {}'),
        ("deserialization bypass", "E0277", 'fn main() { let _ = serde_json::from_str::<helm_app_spec::ValidatedAppSpec>("{}"); }'),
        ("raw tree exposure", "E0603", 'fn main() { let _ = helm_app_spec::parse::Value::Other; }'),
    ]
    receipt["public_api_rejections"] = []
    for name, expected_code, source in attempts:
        (api / "src/main.rs").write_text(source, encoding="utf-8")
        run(["cargo", "check", "--offline"], cwd=api, expected=101)
        assert expected_code in receipt["commands"][-1]["stderr"]
        receipt["public_api_rejections"].append({"attempt":name,"rustc_code":expected_code,"result":"rejected"})

    # Base extraction is confined to a new review directory; never checkout/reset main.
    base = work / "base"
    if not base.exists():
        raw = subprocess.check_output(["git", "archive", BASE, "Cargo.toml", "Cargo.lock", "crates/helm-evidence", "tools/helm_evidence_linux_probe.py"], cwd=ROOT)
        base.mkdir()
        with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
            archive.extractall(base, filter="data")
    run(["cargo", "test", "--workspace", "--locked"], cwd=base)
    # The same executable source links either baseline evidence, candidate evidence
    # alone, or both candidate libraries. Pins/registry packages remain unchanged.
    large = work / "synthetic-large"
    if not large.exists():
        shutil.copytree(ROOT / "crates/helm-evidence/tests/fixtures/synthetic", large)
    payload = bytes(i % 251 for i in range(4 * 1024 * 1024))
    (large / "payload.dat").write_bytes(payload)
    contract = json.loads((ROOT / "crates/helm-evidence/tests/fixtures/synthetic/bundle.json").read_bytes())
    contract["artifacts"].append({"id": "review-payload", "path": "payload.dat", "sha256": hashlib.sha256(payload).hexdigest()})
    (large / "bundle.json").write_text(json.dumps(contract), encoding="utf-8")
    registry_expected = {(p["name"], p["version"]): p for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"] if "source" in p}
    measurements = {}
    for mode, repo, combined in [("base", base, False), ("candidate-combined", ROOT, True), ("candidate-standalone", ROOT, False)]:
        folder = work / mode
        if folder == base:
            folder = work / "base-probe"
        (folder / "src").mkdir(parents=True, exist_ok=True)
        manifest = ('[package]\nname = "review-evidence-bench"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n'
                    '[workspace]\n[dependencies]\nserde_json = "=1.0.149"\nsha2 = "=0.10.9"\n'
                    f'helm-evidence = {{ path = "{(repo / "crates/helm-evidence").as_posix()}" }}\n')
        if combined:
            manifest += f'helm-app-spec = {{ path = "{(ROOT / "crates/helm-app-spec").as_posix()}" }}\n'
        (folder / "Cargo.toml").write_text(manifest, encoding="utf-8", newline="\n")
        shutil.copyfile(ROOT / "tools/helm_app_spec_review_bench.rs", folder / "src/main.rs")
        shutil.copyfile(ROOT / "Cargo.lock", folder / "Cargo.lock")
        run(["cargo", "build", "--release", "--offline"], cwd=folder)
        actual = {(p["name"], p["version"]): p for p in tomllib.loads((folder / "Cargo.lock").read_text())["package"] if "source" in p}
        assert all(registry_expected.get(k) == v for k, v in actual.items())
        features = run(["cargo", "tree", "--locked", "-e", "features", "-i", "sha2"], cwd=folder)
        assert ('feature "force-soft"' in features) == combined
        binary = folder / "target/release" / ("review-evidence-bench.exe" if platform.system() == "Windows" else "review-evidence-bench")
        output = run([binary, ROOT / "crates/helm-evidence/tests/fixtures/a0-7zip", ROOT / "crates/helm-evidence/tests/fixtures/synthetic", large], timeout=180)
        measurements[mode] = [json.loads(line) for line in output.splitlines()]
        for item in measurements[mode]:
            item["median_ns"] = statistics.median(item["ns_per_call"])
    for mode in ["candidate-combined", "candidate-standalone"]:
        for baseline, candidate in zip(measurements["base"], measurements[mode]):
            identity = "report_sha256" if baseline["kind"] == "verify" else "sha256"
            assert baseline[identity] == candidate[identity]
            if baseline["kind"] == "verify":
                assert baseline["report"] == candidate["report"]
            else:
                data = bytes(i % 251 for i in range(baseline["bytes"]))
                assert candidate[identity] == hashlib.sha256(data).hexdigest()
    receipt["measurements"] = measurements
    receipt["evidence_semantic_comparison"] = "PASS: complete reports identical across base/combined/standalone; source unchanged"
    source_paths = sorted((ROOT / "crates/helm-app-spec").rglob("*.rs")) + sorted((ROOT / "crates/helm-app-spec/tests/fixtures").glob("*.json")) + [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "crates/helm-app-spec/Cargo.toml", Path(__file__), ROOT / "tools/helm_app_spec_review_bench.rs", ROOT / ".github/workflows/helm-evidence.yml"]
    receipt["source_sha256"] = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
    receipt["status"] = "PASS"
    save()
    print("INDEPENDENT_REVIEW_RESULT=" + json.dumps({k: receipt[k] for k in ["platform", "identities", "properties", "evidence_semantic_comparison", "status"]}), flush=True)
    print("INDEPENDENT_REVIEW_MEASUREMENTS=" + json.dumps({mode: [{k:v for k,v in item.items() if k != "report"} for item in items] for mode,items in measurements.items()}), flush=True)
    # Preserve the complete receipt in hosted logs without another action or secret.
    # JSON is chunked so individual log lines remain small; the owner can reassemble it.
    encoded = json.dumps(receipt)
    for offset in range(0, len(encoded), 2048):
        print("INDEPENDENT_REVIEW_RECEIPT_CHUNK=" + json.dumps(encoded[offset:offset + 2048]), flush=True)


if __name__ == "__main__":
    main()
