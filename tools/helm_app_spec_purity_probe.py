"""Instrument an isolated copy of memchr to detect reachable CPU dispatch.

The cached dependency is never edited. The original candidate must trip the
sentinel; the corrected current source must not. All generated code is in target/.
This proves a particular dependency call-path regression, not arbitrary purity.
"""
from pathlib import Path
import argparse
import hashlib
import io
import json
import platform
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
CANDIDATE = "18c7aa5e80617483b6f5afd0fea4affeb62f926c"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert platform.machine().lower() in ("amd64", "x86_64"), "Probe targets the reviewed x86_64 dispatch"
    work = ROOT / "target/helm-app-spec-independent/purity"
    work.mkdir(parents=True, exist_ok=True)
    version = subprocess.check_output(["rustc", "-Vv"], cwd=ROOT).decode()
    host = next(line.split(": ", 1)[1] for line in version.splitlines() if line.startswith("host: "))
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", host], cwd=ROOT))
    package = next(p for p in metadata["packages"] if p["name"] == "memchr")
    original = Path(package["manifest_path"]).parent
    vendor = work / "memchr"
    shutil.copytree(original, vendor, dirs_exist_ok=True)
    instrument = vendor / "src/arch/x86_64/memchr.rs"
    source = instrument.read_text(encoding="utf-8")
    marker = source.index("unsafe fn detect(")
    insertion = source.index("{", marker) + 1
    source = source[:insertion] + '\n            panic!("MEMCHR_RUNTIME_DISPATCH_REACHED");\n' + source[insertion:]
    instrument.write_text(source, encoding="utf-8", newline="\n")
    candidate = work / "original"
    if not candidate.exists():
        raw = subprocess.check_output(["git", "archive", CANDIDATE, "Cargo.toml", "Cargo.lock", "crates/helm-app-spec", "crates/helm-evidence", "tools/helm_evidence_linux_probe.py"], cwd=ROOT)
        candidate.mkdir()
        with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
            archive.extractall(candidate, filter="data")
    # Older local probe directories may predate inclusion of the Linux test helper.
    helper = candidate / "tools/helm_evidence_linux_probe.py"
    helper.parent.mkdir(exist_ok=True)
    helper.write_bytes(subprocess.check_output(["git", "show", CANDIDATE + ":tools/helm_evidence_linux_probe.py"], cwd=ROOT))
    results = []
    for name, repo in [("original-candidate", candidate), ("current", ROOT)]:
        project = work / name
        (project / "src").mkdir(parents=True, exist_ok=True)
        manifest = '[package]\nname="review-purity-probe"\nversion="0.0.0"\nedition="2024"\npublish=false\n[workspace]\n[dependencies]\n'
        manifest += f'helm-app-spec={{path="{(repo / "crates/helm-app-spec").as_posix()}"}}\n'
        manifest += f'[patch.crates-io]\nmemchr={{path="{vendor.as_posix()}"}}\n'
        (project / "Cargo.toml").write_text(manifest, encoding="utf-8")
        shutil.copyfile(ROOT / "Cargo.lock", project / "Cargo.lock")
        fixture = (repo / "crates/helm-app-spec/tests/fixtures/a0-7zip.json").as_posix()
        rust = '''fn main() {
let valid = include_bytes!("FIXTURE").as_slice();
assert!(helm_app_spec::parse_spec(valid).is_ok());
for input in [b"{".as_slice(), b"{\\\"x\\\":0,\\\"x\\\":1}", b"[[[[[[[[[0]]]]]]]]]", b"1e99999", b"null null", b"\\\"\\xff\\\"", b"", b"{}"] {
assert!(helm_app_spec::parse_spec(input).is_err());
}
println!("PURITY_PROBE_COMPLETED");
}'''.replace("FIXTURE", fixture)
        (project / "src/main.rs").write_text(rust, encoding="utf-8")
        build = subprocess.run(["cargo", "build", "--offline"], cwd=project, capture_output=True, timeout=180)
        assert build.returncode == 0, build.stderr.decode(errors="replace")
        binary = project / "target/debug" / ("review-purity-probe.exe" if platform.system() == "Windows" else "review-purity-probe")
        run = subprocess.run([str(binary)], cwd=ROOT, capture_output=True, timeout=20)
        results.append({"case":name, "exit_code":run.returncode,
            "dispatch_sentinel_reached":b"MEMCHR_RUNTIME_DISPATCH_REACHED" in run.stderr,
            "completed":b"PURITY_PROBE_COMPLETED" in run.stdout,
            "parse_rs_sha256":hashlib.sha256((repo / "crates/helm-app-spec/src/parse.rs").read_bytes()).hexdigest()})
    receipt = {"candidate":CANDIDATE,"platform":platform.system(),"memchr_version":package["version"],
        "instrumented_function":"memchr::arch::x86_64::memchr::unsafe_ifunc!::detect",
        "source_copy_sha256":hashlib.sha256((original / "src/arch/x86_64/memchr.rs").read_bytes()).hexdigest(),
        "instrumented_sha256":hashlib.sha256(instrument.read_bytes()).hexdigest(),"results":results}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2)+"\n", encoding="utf-8", newline="\n")
    print("PURITY_PROBE=" + json.dumps(receipt), flush=True)
    assert results[0]["dispatch_sentinel_reached"], "Original must reproduce the defect"
    assert results[1]["exit_code"] == 0 and results[1]["completed"], "Current must avoid the dispatch path"


if __name__ == "__main__":
    main()
