"""Reject promotion of a partial diagnostic run into a complete word gate."""

import copy
import json
from unittest import TestCase, main

from entry_audit import read_json
from check_word_checkpoint import RECORD, ROOT, check


class WordCheckpointTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix())

    def reject(self, mutate, pattern):
        report = copy.deepcopy(self.report)
        mutate(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_checkpoint(self):
        self.assertEqual(check(self.report), 2)

    def test_cannot_promote_to_full_gate(self):
        self.reject(lambda r: r.update(scope="W03 complete"), "checkpoint scope")

    def test_cannot_omit_64_bit_claim(self):
        self.reject(lambda r: r["observations"].pop("word-64-nat-conservation"), "case scope")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["observations"]["word-64-nat-conservation"]["checkers"].pop("go"), "independent checker")

    def test_cannot_call_fuel_semantic_negative(self):
        def mutate(report):
            result = report["observations"]["word-32-nat-conservation-wrong"]["checkers"]["rust"]["report"]
            detail = json.loads(result["error_detail"])
            detail["code"] = "CORE_TYPE_MISMATCH"
            result["error_detail"] = json.dumps(detail)
        self.reject(mutate, "counted as semantic negative")

    def test_cannot_change_certificate(self):
        self.reject(lambda r: r["observations"]["word-64-nat-conservation"].update(raw_sha256="0" * 64), "certificate identity")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["observations"]["word-64-nat-conservation"]["checkers"]["rust"]["report"].update(axiom_count=1), "unknown axioms")

    def test_cannot_cross_checker_hashes(self):
        self.reject(lambda r: r["observations"]["word-64-nat-conservation"]["checkers"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_cross_source_snapshot(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")


if __name__ == "__main__":
    main()
