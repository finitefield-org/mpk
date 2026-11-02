"""Reject overstated, crossed or incomplete balanced arithmetic evidence."""

import copy
from unittest import TestCase, main

from entry_audit import read_json
from check_balanced_report import check
from run_balanced_probes import RECORD, ROOT


class BalancedReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 48)

    def test_cannot_drop_large_width(self):
        self.reject(lambda r: r["cases"].pop(), "case coverage")

    def test_cannot_claim_w03_complete(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_prefix_sums(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_promote_case_to_application_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go application verified"), "candidate scope")

    def test_cannot_claim_128_bit_go_admission(self):
        self.reject(lambda r: r.update(internal_width_128_scope="Go uint128 admitted"), "source-width admission")

    def test_cannot_weaken_signed_guard(self):
        self.reject(lambda r: r["cases"][24]["case"].update(required_premise="none"), "precondition binding")

    def test_cannot_ground_operands(self):
        self.reject(lambda r: r["cases"][18]["case"].update(quantified_operand_bits=1), "operand quantification")

    def test_cannot_replace_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_checker_hashes(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_relabel_fuel_as_semantic_negative(self):
        def change(report):
            next(row for row in report["cases"] if row["classification"] == "candidate_fuel_failure_not_false_claim_evidence")["classification"] = "core_type_mismatch"
        self.reject(change, "counted as semantic negative")

    def test_cannot_cross_source(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_probe_ceiling_as_profile_budget(self):
        self.reject(lambda r: r.update(ceiling_scope="frozen profile budget"), "resource budget")


if __name__ == "__main__":
    main()
