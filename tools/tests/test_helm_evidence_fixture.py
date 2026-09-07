"""Independent cross-check of the product fixture against the published experiment."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import helm_evidence_fixture


class PublishedFixtureTests(unittest.TestCase):
    def test_publication_hashes_and_exact_projections(self):
        result = helm_evidence_fixture.check()
        self.assertEqual(result["application_executions"], 0)
        self.assertEqual(result["historical_files_modified"], 0)
        self.assertGreater(result["fixture_files_checked"], 0)
