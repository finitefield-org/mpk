"""Focused rejection controls for the retained-source export boundary."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import retained_csharp_epoch as subject


class RetainedEpochControls(unittest.TestCase):
    def fixture(self):
        return json.loads((subject.RECORDS / "private-epoch.json").read_bytes())

    def rejects_before_git(self, value):
        with patch.object(subject.json, "loads", return_value=value), patch.object(
            subject, "git", side_effect=AssertionError("invalid input reached Git")
        ):
            with self.assertRaises(ValueError):
                subject.load("private")

    def test_foreign_commit_rejects(self):
        value = self.fixture()
        value["commit"] = "0" * 40
        self.rejects_before_git(value)

    def test_unknown_field_rejects(self):
        value = self.fixture()
        value["proof_authority"] = True
        self.rejects_before_git(value)

    def test_unsafe_paths_and_modes_reject(self):
        for path in ("../escape", "/absolute", "a\\b", "C:/drive", "a\x00b"):
            value = self.fixture()
            value["files"][0]["path"] = path
            self.rejects_before_git(value)
        value = self.fixture()
        value["files"][0]["mode"] = "120000"
        self.rejects_before_git(value)

    def test_changed_raw_bytes_reject(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "payload"
            path.write_bytes(b"b")
            path.chmod(0o644)
            record = {"files": [{"path": "payload", "bytes": 1, "mode": "100644",
                                 "sha256": hashlib.sha256(b"a").hexdigest()}]}
            with self.assertRaises(ValueError):
                subject.verify_files(record, root)


if __name__ == "__main__":
    unittest.main()
