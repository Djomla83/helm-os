"""Output oracle checks, not tests of 7-Zip, Wine, Hyper-V or a desktop."""

from pathlib import Path
import stat
import struct
import subprocess
import sys
import tempfile
import unittest
import warnings
import zipfile
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import app_baseline as baseline


class ArchiveVerifierTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest = baseline.load_manifest()
        self.work = self.root / "inputs"
        baseline.prepare(self.work, self.manifest)

    def verify(self, filename):
        return baseline.verify_archive(filename, self.manifest)

    def candidate(self, edit=None, omitted=None, extra=None, compression=zipfile.ZIP_DEFLATED):
        path = self.root / "candidate.zip"
        with zipfile.ZipFile(path, "x", compression=compression) as archive:
            for name in self.manifest["required_directories"]:
                archive.writestr(name, b"")
            for name, data in baseline.fixture_bytes().items():
                if name != omitted:
                    archive.writestr(name, edit(name, data) if edit else data)
            if extra:
                with warnings.catch_warnings():
                    warnings.simplefilter("ignore", UserWarning)
                    archive.writestr(*extra)
        return path

    def test_good_control_covers_all_literal_files(self):
        result = self.verify(self.work / "oracle-good.zip")
        self.assertEqual(result["verdict"], "PASS")
        self.assertEqual(len(result["files"]), 6)
        self.assertIn("fixture/names/caf\u00e9-\u6f22\u5b57.txt", {f["path"] for f in result["files"]})

    def test_valid_zip_with_corrupted_payload_is_rejected(self):
        path = self.work / "oracle-corrupted.zip"
        with zipfile.ZipFile(path) as archive:
            self.assertIsNone(archive.testzip(), "Seed keeps ZIP checksums valid")
        result = self.verify(path)
        self.assertEqual(result["verdict"], "FAIL")
        self.assertIn("Decompressed content mismatch: fixture/binary/payload.bin", result["errors"])

    def test_independently_encoded_stored_zip(self):
        # Construct a minimal ZIP with struct/zlib, independent of ZipFile's writer.
        name, data = b"reference.txt", b"abc"
        crc = zlib.crc32(data)
        local = struct.pack("<I5H3I2H", 0x04034B50, 20, 0, 0, 0, 33, crc, 3, 3, len(name), 0) + name + data
        central = struct.pack("<I6H3I5H2I", 0x02014B50, 20, 20, 0, 0, 0, 33, crc, 3, 3, len(name), 0, 0, 0, 0, 0, 0) + name
        end = struct.pack("<I4H2IH", 0x06054B50, 0, 0, 1, 1, len(central), len(local), 0)
        path = self.root / "independent.zip"
        path.write_bytes(local + central + end)
        manifest = {"files": {"reference.txt": {"bytes": 3, "sha256": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}}, "required_directories": [], "allowed_directories": []}
        self.assertEqual(baseline.verify_archive(path, manifest)["verdict"], "PASS")

    def test_deflate_and_implicit_ancestor_directories_are_valid(self):
        self.assertEqual(self.verify(self.candidate())["verdict"], "PASS")

    def test_missing_empty_file_is_rejected(self):
        self.assertEqual(self.verify(self.candidate(omitted="fixture/empty.dat"))["verdict"], "FAIL")

    def test_missing_empty_directory_is_rejected(self):
        self.manifest["required_directories"] = []
        path = self.candidate()
        self.manifest = baseline.load_manifest()
        self.assertEqual(self.verify(path)["verdict"], "FAIL")

    def test_same_size_wrong_text_is_rejected(self):
        path = self.candidate(edit=lambda name, data: b"X" * len(data) if name.endswith("plain.txt") else data)
        self.assertEqual(self.verify(path)["verdict"], "FAIL")

    def test_extra_and_traversal_path_rejected_without_extraction(self):
        result = self.verify(self.candidate(extra=("../outside.txt", b"synthetic")))
        self.assertEqual(result["verdict"], "FAIL")
        self.assertFalse((self.root / "outside.txt").exists())

    def test_duplicate_member_is_rejected(self):
        result = self.verify(self.candidate(extra=("fixture/plain.txt", baseline.fixture_bytes()["fixture/plain.txt"])))
        self.assertEqual(result["verdict"], "FAIL")
        self.assertIn("Duplicate archive paths", result["errors"])

    def test_symlink_entry_is_rejected(self):
        info = zipfile.ZipInfo("fixture/link")
        info.create_system = 3
        info.external_attr = (stat.S_IFLNK | 0o777) << 16
        self.assertEqual(self.verify(self.candidate(extra=(info, b"plain.txt")))["verdict"], "FAIL")

    def test_truncated_archive_is_rejected(self):
        path = self.root / "truncated.zip"
        path.write_bytes((self.work / "oracle-good.zip").read_bytes()[:-22])
        self.assertEqual(self.verify(path)["verdict"], "FAIL")

    def test_crc_corruption_is_rejected(self):
        path = self.root / "bad-crc.zip"
        raw = (self.work / "oracle-good.zip").read_bytes()
        path.write_bytes(raw.replace(b"HELM EXP-009 application baseline", b"XELM EXP-009 application baseline", 1))
        self.assertEqual(self.verify(path)["verdict"], "FAIL")

    def test_archive_size_limit_is_enforced(self):
        path = self.root / "oversized.zip"
        path.write_bytes(b"X" * (baseline.MAX_ARCHIVE_BYTES + 1))
        self.assertEqual(self.verify(path)["verdict"], "FAIL")

    def test_existing_fixtures_and_records_are_preserved(self):
        with self.assertRaises(FileExistsError):
            baseline.prepare(self.work, self.manifest)
        path = self.root / "record.json"
        baseline.write_record(path, {"first": True})
        with self.assertRaises(FileExistsError):
            baseline.write_record(path, {"second": True})

    def test_capture_preserves_bytes_and_nonzero_process_status(self):
        import base64
        import json
        path = self.root / "capture.json"
        command = [sys.executable, str(Path(baseline.__file__).resolve()), "capture",
                   "--cwd", str(self.root), "--timeout", "5", str(path), "--",
                   sys.executable, "-c", "import sys;sys.stdout.buffer.write(b'OK\\r\\n');sys.stderr.buffer.write(b'ERR\\n');sys.exit(7)"]
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 7, result.stderr)
        record = json.loads(path.read_text())
        self.assertEqual(record["exit_code"], 7)
        self.assertEqual(base64.b64decode(record["stdout_base64"]), b"OK\r\n")
        self.assertEqual(base64.b64decode(record["stderr_base64"]), b"ERR\n")


if __name__ == "__main__":
    unittest.main()
