"""Check retrospective desired-state fixtures against frozen A0 inputs, without artifacts.

Development tooling only: reads Git blobs, never fetches/opens an installer or Wine
artifact, never starts a lab. --write regenerates only the new app-spec fixtures.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "crates/helm-app-spec/tests/fixtures"
DEFINITION = "5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae"
PINS = "docs/experiments/evidence/app-baseline-2026-09-07/artifact-pins.json"


def blob(path):
    return subprocess.check_output(["git", "show", f"{DEFINITION}:{path}"], cwd=ROOT)


def identity(raw):
    return {"size": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def fixtures():
    pins = json.loads(blob(PINS))
    references = []
    origins = []
    for role, path in [
        ("protocol", "docs/experiments/EXP-009.md"),
        ("oracle", "tools/app_baseline.py"),
        ("content-fixture", "tools/fixtures/exp009-7zip.json"),
    ]:
        reference = {"role": role, **identity(blob(path))}
        references.append(reference)
        origins.append({"path": path, **reference})
    application = pins["application"]
    a0 = {
        "schema": "helm-app-spec", "version": "0.1",
        "application": {"id": "7zip-x64", "version": application["version"],
                        "source": {"size": application["bytes"], "sha256": application["sha256"], "architecture": "x86_64"}},
        "runtime": {"family": "wine", "artifacts": [
            {"role": p["package"] + "-archive", "size": p["expected_bytes"], "sha256": p["expected_sha256"],
             "label": f'{p["package"]} {p["version"]} ({p["architecture"]})'} for p in pins["wine"]["packages"]]},
        "environment": {"windows_architecture": "win64", "prefix": {"role": "dedicated"}, "disabled_dlls": ["mscoree", "mshtml"]},
        "entry_point": {"path": "drive_c/Program Files/7-Zip/7zFM.exe"},
        "verification": {"definitions": references},
    }
    synthetic = {
        "schema": "helm-app-spec", "version": "0.1",
        "application": {"id": "synthetic-notes", "version": "fictional v2",
                        "source": {**identity(b"Synthetic application source; not an executable.\n"), "architecture": "x86_64"}},
        "runtime": {"family": "wine", "artifacts": [{"role": "synthetic-runtime", **identity(b"Synthetic runtime artifact; not Wine.\n")}]},
        "environment": {"windows_architecture": "win64", "prefix": {"role": "dedicated"}, "disabled_dlls": []},
        "entry_point": {"path": "drive_c/Notes/notes.exe", "sha256": identity(b"Synthetic entry point; not executable.\n")["sha256"]},
        "verification": {"definitions": [{"role": "notes-contract", **identity(b"Synthetic pre-execution definition: preserve note bytes.\n")}]},
    }
    provenance = {
        "scope": "Retrospective desired-state A0 fixture and fictional second application; no execution or observation claims",
        "definition_commit": DEFINITION, "source_pins": PINS, "definitions": origins,
        "a0_environment_and_entry_point": "EXP-009 registered A0 definition; prefix-relative spelling from the original Wine C-drive intent",
        "excluded": ["installed file identities", "loader/server observations", "workflow verdicts", "output ZIP identity", "boot IDs", "evidence bundle", "host/prefix instance paths"],
        "synthetic_identity_recipe": "SHA-256 and lengths of the literal UTF-8 byte strings in tools/helm_app_spec_fixture.py; no real executable or runtime claimed",
    }
    return {"a0-7zip.json": a0, "synthetic-notes.json": synthetic, "provenance.json": provenance}


def check():
    result = fixtures()
    for name, value in result.items():
        expected = (json.dumps(value, indent=2) + "\n").encode()
        if (FOLDER / name).read_bytes() != expected:
            raise ValueError("Fixture differs from frozen input projection: " + name)
    return {"fixture_files_checked": len(result), "definition_commit": DEFINITION,
            "frozen_definition_identities": result["provenance.json"]["definitions"],
            "scope": "Declared inputs only; no installer, runtime, entry point or definition executed"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        FOLDER.mkdir(parents=True, exist_ok=True)
        for name, value in fixtures().items():
            (FOLDER / name).write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(check(), indent=2))


if __name__ == "__main__":
    main()
