"""EXP-009 A0-7ZIP synthetic fixtures, bounded ZIP oracle and command capture.

This verifies output content only. It does not establish GUI coverage, application
provenance, restart completion, compatibility, or a security boundary.
"""

import argparse
import hashlib
import json
from pathlib import Path
import stat
import sys
import zipfile
import zlib

from gate0.probe_wsl_interop import capture, now


MANIFEST = Path(__file__).parent / "fixtures" / "exp009-7zip.json"
MAX_ARCHIVE_BYTES = 1024 * 1024
MAX_ENTRIES = 32


def digest(data):
    return hashlib.sha256(data).hexdigest()


def fixture_bytes():
    """Literal synthetic input definition, independent of application output."""
    return {
        "fixture/plain.txt": b"HELM EXP-009 application baseline\nLine two.\n",
        "fixture/binary/payload.bin": bytes(range(256)) * 16,
        "fixture/empty.dat": b"",
        "fixture/nested/deeper/data.txt": b"nested content\n",
        "fixture/names/with spaces.txt": b"spaces remain intact\n",
        "fixture/names/caf\u00e9-\u6f22\u5b57.txt": "UTF-8 content: caf\u00e9 \u6f22\u5b57\n".encode("utf-8"),
    }


def load_manifest():
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def prepare(workdir, manifest):
    """Create once; never replace a previous fixture or workflow destination."""
    files = fixture_bytes()
    assert set(files) == set(manifest["files"])
    for name, data in files.items():
        assert manifest["files"][name] == {"bytes": len(data), "sha256": digest(data)}
    workdir.mkdir(parents=True, exist_ok=False)
    for name in manifest["allowed_directories"]:
        (workdir / name).mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        with (workdir / name).open("xb") as stream:
            stream.write(data)
    controls = []
    for label, corrupt in [("oracle-good", False), ("oracle-corrupted", True)]:
        path = workdir / (label + ".zip")
        with zipfile.ZipFile(path, "x", compression=zipfile.ZIP_STORED) as archive:
            for name in sorted(manifest["allowed_directories"]):
                info = zipfile.ZipInfo(name, (2026, 9, 7, 0, 0, 0))
                info.external_attr = (stat.S_IFDIR | 0o755) << 16 | 0x10
                archive.writestr(info, b"")
            for name, original in sorted(files.items()):
                data = original
                if corrupt and name == "fixture/binary/payload.bin":
                    data = bytes([data[0] ^ 1]) + data[1:]
                info = zipfile.ZipInfo(name, (2026, 9, 7, 0, 0, 0))
                info.external_attr = (stat.S_IFREG | 0o644) << 16
                archive.writestr(info, data)
        controls.append({"file": path.name, "bytes": path.stat().st_size,
                         "sha256": digest(path.read_bytes()),
                         "expected_verifier_verdict": "FAIL" if corrupt else "PASS"})
    return {"scope": "Synthetic harness fixtures, not application output", "created_utc": now(),
            "fixture_manifest_sha256": digest(MANIFEST.read_bytes()), "controls": controls,
            "corruption": "Flip bit 0 of the first binary payload byte; ZIP CRC remains valid."}


def verify_archive(path, manifest):
    """Decompress members in memory and check exact paths, sizes and byte hashes.

    No archive-controlled filesystem path is written. The committed input manifest
    supplies expectations; the candidate archive cannot redefine them.
    """
    result = {"scope": "ZIP output-content verification only", "started_utc": now(),
              "verdict": "FAIL", "errors": [], "files": []}
    errors = result["errors"]
    expected = manifest["files"]
    try:
        with path.open("rb") as stream:
            raw = stream.read(MAX_ARCHIVE_BYTES + 1)
        if len(raw) > MAX_ARCHIVE_BYTES:
            raise ValueError("Archive exceeds the 1 MiB synthetic-fixture limit")
        result["archive_sha256"] = digest(raw)
        result["archive_bytes"] = len(raw)
        import io
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            entries = archive.infolist()
            if len(entries) > MAX_ENTRIES:
                raise ValueError("Archive exceeds the 32-entry fixture limit")
            names = [entry.filename for entry in entries]
            if len(names) != len(set(names)):
                errors.append("Duplicate archive paths")
            actual_files = {entry.filename for entry in entries if not entry.is_dir()}
            actual_dirs = {entry.filename for entry in entries if entry.is_dir()}
            if actual_files != set(expected):
                errors.append("File paths differ from the frozen manifest")
            if not set(manifest["required_directories"]).issubset(actual_dirs):
                errors.append("Required empty directory is missing")
            if not actual_dirs.issubset(manifest["allowed_directories"]):
                errors.append("Unexpected directory path")
            for entry in entries:
                name = entry.filename
                parts = name.rstrip("/").split("/")
                if entry.orig_filename != name or "\\" in name or ":" in name or any(part in ["", ".", ".."] for part in parts):
                    errors.append("Noncanonical archive path")
                    continue
                if entry.flag_bits & 1:
                    errors.append("Encrypted entry: " + name)
                    continue
                kind = stat.S_IFMT(entry.external_attr >> 16)
                expected_kind = stat.S_IFDIR if entry.is_dir() else stat.S_IFREG
                if kind not in [0, expected_kind]:
                    errors.append("Unsupported entry type: " + name)
                    continue
                if entry.compress_type not in [zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED]:
                    errors.append("Expected ordinary stored/deflated ZIP: " + name)
                    continue
                if entry.is_dir():
                    if entry.file_size != 0:
                        errors.append("Directory carries file content: " + name)
                    continue
                if name not in expected:
                    continue
                requirement = expected[name]
                if entry.file_size != requirement["bytes"]:
                    errors.append("File size mismatch: " + name)
                    continue
                with archive.open(entry) as stream:
                    data = stream.read(requirement["bytes"] + 1)
                observed = {"path": name, "bytes": len(data), "sha256": digest(data)}
                result["files"].append(observed)
                if observed["bytes"] != requirement["bytes"] or observed["sha256"] != requirement["sha256"]:
                    errors.append("Decompressed content mismatch: " + name)
    except (OSError, ValueError, zipfile.BadZipFile, RuntimeError, NotImplementedError, EOFError, zlib.error) as error:
        errors.append(type(error).__name__ + ": " + str(error))
    result["verdict"] = "FAIL" if errors else "PASS"
    result["ended_utc"] = now()
    return result


def write_record(path, record):
    with path.open("xb") as stream:
        stream.write((json.dumps(record, ensure_ascii=True, indent=2) + "\n").encode())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    fixture = sub.add_parser("prepare")
    fixture.add_argument("directory", type=Path)
    fixture.add_argument("record", type=Path)
    verify = sub.add_parser("verify")
    verify.add_argument("archive", type=Path)
    verify.add_argument("record", type=Path)
    command = sub.add_parser("capture", help="Reuse the existing byte-preserving command helper")
    command.add_argument("record", type=Path)
    command.add_argument("--cwd", required=True)
    command.add_argument("--timeout", type=int, default=180)
    command.add_argument("argv", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.record.exists():
        raise FileExistsError("Refusing to overwrite prior evidence")
    if args.operation == "prepare":
        record = prepare(args.directory, load_manifest())
        exit_code = 0
    elif args.operation == "verify":
        record = verify_archive(args.archive, load_manifest())
        record["fixture_manifest_sha256"] = digest(MANIFEST.read_bytes())
        exit_code = 0 if record["verdict"] == "PASS" else 1
    else:
        argv = args.argv[1:] if args.argv[:1] == ["--"] else args.argv
        if not argv or not 1 <= args.timeout <= 600:
            parser.error("An explicit command and a timeout of 1–600 seconds are required")
        record = capture(argv, cwd=args.cwd, timeout=args.timeout)
        record["scope"] = "Command capture only; exit status is not a workflow verdict"
        exit_code = record["exit_code"] if record["exit_code"] is not None else 125
    write_record(args.record, record)
    print(json.dumps(record, ensure_ascii=True))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
