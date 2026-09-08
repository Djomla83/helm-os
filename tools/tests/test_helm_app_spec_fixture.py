"""Compare desired pins with independent historical Git blobs, not product logic."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import helm_app_spec_fixture


class AppSpecFixtureTests(unittest.TestCase):
    def test_frozen_a0_and_synthetic_projection(self):
        result = helm_app_spec_fixture.check()
        self.assertEqual(result["fixture_files_checked"], 3)
        self.assertEqual([x["size"] for x in result["frozen_definition_identities"]], [34585, 9093, 1290])


if __name__ == "__main__":
    unittest.main()
