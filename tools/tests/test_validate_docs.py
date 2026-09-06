"""Testovi pomoćnog validatora, nikada testovi Windows aplikacija."""
from __future__ import annotations

import copy
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("validate_docs", ROOT / "tools" / "validate_docs.py")
assert SPEC and SPEC.loader
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)


class RecordTests(unittest.TestCase):
    def setUp(self) -> None:
        self.record = json.loads((ROOT / "examples/app-test-record.example.json").read_text(encoding="utf-8"))

    def test_untested_example(self) -> None:
        self.assertEqual(validator.check_app_record(self.record), [])

    def test_fake_verified_rejected(self) -> None:
        self.record["status"] = "verified"
        errors = validator.check_app_record(self.record)
        self.assertTrue(any("sintetički" in e for e in errors))
        self.assertTrue(any("evidence" in e for e in errors))
        self.assertTrue(any("source_sha256" in e for e in errors))

    def test_untested_with_evidence_rejected(self) -> None:
        self.record["evidence"] = ["UNIT-TEST-ONLY"]
        self.assertTrue(validator.check_app_record(self.record))

    def test_untested_with_date_rejected(self) -> None:
        self.record["tested_at"] = "2026-09-06T18:00:00Z"
        self.assertTrue(validator.check_app_record(self.record))

    def test_invalid_hash_rejected(self) -> None:
        self.record["application"]["source_sha256"] = "not-a-hash"
        self.assertTrue(validator.check_app_record(self.record))

    def test_unknown_field_rejected(self) -> None:
        self.record["invented_success_rate"] = 0.95
        self.assertTrue(validator.check_app_record(self.record))

    def test_bad_workflow_element_does_not_crash(self) -> None:
        self.record["mandatory_workflows"] = [{"bad": "type"}]
        self.assertTrue(validator.check_app_record(self.record))

    def test_non_object_rejected(self) -> None:
        self.assertTrue(validator.check_app_record([]))

    def test_timestamp_requires_timezone(self) -> None:
        self.assertFalse(validator.valid_timestamp("2026-09-06T18:00:00"))
        self.assertTrue(validator.valid_timestamp("2026-09-06T18:00:00+02:00"))

    def test_draft_cohort_cannot_claim_rate(self) -> None:
        cohort = json.loads((ROOT / "planning/app-cohort.json").read_text(encoding="utf-8"))
        self.assertEqual(validator.check_cohort(cohort), [])
        cohort["compatibility_rate"] = 0.95
        self.assertTrue(validator.check_cohort(cohort))

    def test_duplicate_cohort_identifier(self) -> None:
        cohort = json.loads((ROOT / "planning/app-cohort.json").read_text(encoding="utf-8"))
        cohort["applications"].append(copy.deepcopy(cohort["applications"][0]))
        self.assertTrue(validator.check_cohort(cohort))


class MarkdownTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "README.md"

    def write(self, text: str) -> None:
        self.source.write_text(text, encoding="utf-8")

    def test_existing_link_and_explicit_anchor(self) -> None:
        (self.root / "target.md").write_text('<a id="known"></a>\n# Naslov\n', encoding="utf-8")
        self.write("[opis](target.md#known)\n")
        errors, count = validator.check_markdown(self.source, self.root)
        self.assertEqual(errors, [])
        self.assertEqual(count, 1)

    def test_missing_target(self) -> None:
        self.write("[opis](missing.md)\n")
        self.assertTrue(validator.check_markdown(self.source, self.root)[0])

    def test_missing_anchor(self) -> None:
        self.write("# Naslov\n\n[opis](#missing)\n")
        self.assertTrue(validator.check_markdown(self.source, self.root)[0])

    def test_reference_definition_required(self) -> None:
        self.write("[izvor][S99]\n")
        self.assertTrue(validator.check_markdown(self.source, self.root)[0])

    def test_code_links_ignored(self) -> None:
        self.write("```md\n[primer](missing.md)\n```\n")
        self.assertEqual(validator.check_markdown(self.source, self.root)[0], [])

    def test_unclosed_fence(self) -> None:
        self.write("```text\nnezatvoreno\n")
        self.assertTrue(validator.check_markdown(self.source, self.root)[0])

    def test_repo_escape_rejected(self) -> None:
        self.write("[opis](../outside.md)\n")
        self.assertTrue(validator.check_markdown(self.source, self.root)[0])

    def test_external_url_not_fetched(self) -> None:
        self.write("[external](https://example.invalid/no-network)\n")
        self.assertEqual(validator.check_markdown(self.source, self.root)[0], [])

    def test_json_syntax_error_reported(self) -> None:
        self.write("# Dobar fajl\n")
        (self.root / "broken.json").write_text("{bad", encoding="utf-8")
        errors, _ = validator.validate_repository(self.root)
        self.assertTrue(any("broken.json" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
