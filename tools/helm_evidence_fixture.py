"""Derive/check the small public A0 fixture from immutable published machine records.

This development-only adapter never runs an experiment or opens private evidence.
Default: compare bytes only. --write: write derived fixture files, never historical sources.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/experiments/evidence/app-baseline-execution-2026-09-07"
DEST = ROOT / "crates/helm-evidence/tests/fixtures/a0-7zip"
PUBLICATION = "ae3f012fb1bfd7b20018c3faf6c71a6740a041fe"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return (json.dumps(value, ensure_ascii=True, indent=2) + "\n").encode()


def derive():
    """Expectations come from EXP-009's preregistration, never from verifier output."""
    publication = json.loads((SOURCE / "publication-manifest.json").read_bytes())
    pins = {a["file"]: a["publication_sha256"] for a in publication["artifacts"]}
    files, artifacts, origins = {}, [], []

    def source(name):
        raw = (SOURCE / name).read_bytes()
        if digest(raw) != pins[name]:
            raise ValueError(f"Published source identity changed: {name}")
        return raw

    def add(name, raw, origin=None):
        filename = name + ".json"
        files[filename] = raw
        artifacts.append({"id": name, "path": filename, "sha256": digest(raw)})
        if origin:
            origins.append({"fixture": filename, "source": origin, "sha256": digest(raw)})
        return name

    copies = {
        "summary": "result-summary.json",
        "deviation": "workflow-w1-deviation.json",
        "before-result": "guest/records/verify-before-accidental.json",
        "before-missing": "guest/records/verify-before-required.json",
        "after-result": "guest/records/verify-after.json",
        "after-command": "guest/records/verify-after-command.json",
        "before-good": "guest/controls/before-good.json",
        "before-bad": "guest/controls/before-bad.json",
        "after-good": "guest/controls/after-good.json",
        "after-bad": "guest/controls/after-bad.json",
        "before-gui": "guest/records/workflow-before.json",
        "after-gui": "guest/records/workflow-after.json",
        "installation": "guest/records/7zip-installation.json",
        "identities-before": "guest/records/installed-before.json",
        "identities-after": "guest/records/installed-after.json",
        "restart-command": "guest-r1-restart.json",
    }
    for name, origin in copies.items():
        add(name, source(origin), (SOURCE / origin).relative_to(ROOT).as_posix())
    manifest_path = ROOT / "tools/fixtures/exp009-7zip.json"
    manifest = manifest_path.read_bytes()
    # Frozen before application installation, also recorded by each original oracle.
    assert digest(manifest) == "75d78bc6bc0f539df5ca8d54df34f406e44f2aa5ceb8a07bf3d856ce72747c8f"
    add("content-manifest", manifest, manifest_path.relative_to(ROOT).as_posix())
    summary = json.loads(files["summary.json"])
    deviation = json.loads(files["deviation.json"])
    after = json.loads(files["after-result.json"])
    command = json.loads(files["after-command.json"])
    assert summary["overall_verdict"] == "FAIL"
    assert summary["workflow_W1"] == "FAIL" and summary["workflow_W2"] == "PASS"
    assert deviation["registered_destination"] == "outputs/before-restart/workflow.zip"
    assert deviation["observed_destination"] == "inputs/fixture.zip"
    assert command["argv"][:3] == ["python3", "tools/app_baseline.py", "verify"]
    assert command["argv"][3] == "outputs/after-restart/workflow.zip" and command["exit_code"] == 0
    assert summary["before_boot_id"] != summary["after_boot_id"]
    workflows = []
    for label, wid, phase, steps, path, output_hash in [
        ("before", "W1", "before_restart", [
            ("A1", "installation_A1", ["summary", "installation"]),
            ("A2", "installed_identities_A2", ["summary", "identities-before"]),
            ("W1", "workflow_W1", ["summary", "before-gui", "deviation"]),
            ("V1", "registered_output_V1", ["before-missing", "before-result"]),
        ], deviation["observed_destination"], deviation["observed_archive_sha256"]),
        ("after", "W2", "after_restart", [
            ("W2", "workflow_W2", ["summary", "after-gui", "identities-after"]),
            ("V2", "output_V2", ["after-result", "after-command"]),
        ], command["argv"][3], after["archive_sha256"]),
    ]:
        record_id = add(label + "-workflow", encoded({
            "version": "0.1", "workflow": wid, "boot_id": summary[label + "_boot_id"],
            "steps": [{"id": sid, "status": summary[key]} for sid, key, _ in steps],
            "output": {"path": path, "sha256": output_hash},
        }))
        workflows.append({
            "id": wid, "phase": phase, "record": record_id,
            "steps": [{"id": sid, "evidence": refs} for sid, _, refs in steps],
            "output": {
                "id": "V1" if wid == "W1" else "V2",
                "expected_path": f"outputs/{label}-restart/workflow.zip",
                "sha256": summary["output_archive_sha256"], "content_result": label + "-result",
                "content_manifest": "content-manifest",
                "known_good": {"result": label + "-good", "sha256": json.loads(files[label + "-good.json"])["archive_sha256"]},
                "deliberately_broken": {"result": label + "-bad", "sha256": json.loads(files[label + "-bad.json"])["archive_sha256"]},
                "local_artifact": None,
            },
        })
    restart = add("restart", encoded({
        "version": "0.1", "before_boot_id": summary["before_boot_id"], "after_boot_id": summary["after_boot_id"],
    }))
    files["bundle.json"] = encoded({
        "schema": "helm-evidence", "version": "0.1", "experiment": "EXP-009-A0-7ZIP",
        "application": {"id": "7zip-x64", "version": "26.03", "source_sha256":
                        "0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd"},
        "experimental_verdict": summary["overall_verdict"], "artifacts": artifacts,
        "workflows": workflows, "restart": {"record": restart, "evidence": ["restart-command", "summary"]},
    })
    files["provenance.json"] = encoded({
        "kind": "Post-experiment adapter of published machine evidence, not a new execution",
        "publication_commit": PUBLICATION,
        "copies": origins,
        "projections": {
            "before-workflow.json": "summary step/boot fields; deviation observed_destination and observed_archive_sha256",
            "after-workflow.json": "summary step/boot fields; after-command argv[3]; after-result archive_sha256",
            "restart.json": "summary before_boot_id and after_boot_id",
        },
        "boundary": "No ZIP bodies, installers, private provenance or generated application evidence. Output location is an experiment-relative recorded observation, not a local-file existence claim.",
    })
    return files


def check():
    files = derive()
    for name, raw in files.items():
        if (DEST / name).read_bytes() != raw:
            raise ValueError(f"Fixture does not match published projection: {name}")
    return {"fixture_files_checked": len(files), "fixture_bytes": sum(map(len, files.values())),
            "historical_files_modified": 0, "application_executions": 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="Write only the derived public fixture")
    args = parser.parse_args()
    if args.write:
        DEST.mkdir(parents=True, exist_ok=True)
        for name, raw in derive().items():
            (DEST / name).write_bytes(raw)
    print(json.dumps(check(), sort_keys=True))


if __name__ == "__main__":
    main()
