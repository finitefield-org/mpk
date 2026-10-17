"""Adversarial tests for real epoch identity, path and byte/mode checks."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from source_epoch import RECORD, validate_shape, verify_files


class EpochTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.epoch = json.loads((RECORD / "historical-source-epoch.json").read_text())

    def test_wrong_epoch_and_duplicate_or_unsafe_files_reject(self):
        for kind in ("commit", "tree", "duplicate", "escape", "absolute", "boolean_bytes", "symlink_mode"):
            value = copy.deepcopy(self.epoch)
            if kind in ("commit", "tree"): value[kind] = "0" * 40
            elif kind == "duplicate": value["files"].append(copy.deepcopy(value["files"][0]))
            elif kind == "escape": value["files"][0]["path"] = "../outside"
            elif kind == "absolute": value["files"][0]["path"] = "/private/tmp/outside"
            elif kind == "boolean_bytes": value["files"][0]["bytes"] = True
            else: value["files"][0]["mode"] = "120000"
            with self.subTest(kind=kind), self.assertRaises(ValueError): validate_shape(value)

    def test_same_length_modified_source_and_executable_mode_reject(self):
        value = copy.deepcopy(self.epoch); value["files"] = value["files"][:1]
        row = value["files"][0]; data = b"historical input\n"
        row["sha256"] = hashlib.sha256(data).hexdigest(); row["bytes"] = len(data); row["mode"] = "100644"
        with tempfile.TemporaryDirectory() as td:
            root = Path(td); p = root / row["path"]; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(data); p.chmod(0o644)
            verify_files(value, root)
            p.write_bytes(b"X" + data[1:])
            with self.assertRaisesRegex(ValueError, "raw bytes"): verify_files(value, root)
            p.write_bytes(data); p.chmod(0o755)
            with self.assertRaisesRegex(ValueError, "mode"): verify_files(value, root)

    def test_pinned_source_replaced_by_symlink_rejects(self):
        value = copy.deepcopy(self.epoch); value["files"] = value["files"][:1]
        with tempfile.TemporaryDirectory() as td:
            root = Path(td); p = root / value["files"][0]["path"]; p.parent.mkdir(parents=True, exist_ok=True)
            other = root / "target"; other.write_bytes(b"same source is still not an immutable input handle")
            p.symlink_to(other)
            with self.assertRaisesRegex(ValueError, "regular epoch input"): verify_files(value, root)


if __name__ == "__main__": unittest.main()
