"""Reject evidence changes which could turn a partial pilot into false support."""

import copy
from unittest import TestCase, main

import check_proof_report
from entry_audit import read_json
from run_proof_pilot import RECORD, ROOT


class ProofReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check_proof_report.check(report)

    def test_retained_report(self):
        self.assertEqual(check_proof_report.check(self.report), 27)

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status="complete"), "W03 status")

    def test_cannot_hide_arithmetic(self):
        self.reject(lambda r: r["pending"].pop(2), "pending requirements")

    def test_cannot_drop_workload(self):
        self.reject(lambda r: r["cases"].pop(), "coverage differs")

    def test_cannot_change_certificate(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_substitute_non_core_rejection(self):
        self.reject(lambda r: r["cases"][2]["observations"]["rust"]["report"].update(error_code="KERNEL_DECODE"), "core checking")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_rust_zero_count_is_required(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].pop("axiom_count"), "zero-axiom")

    def test_cannot_cross_theorem_module(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(module="Other"), "module/declaration identity")

    def test_cannot_cross_checker_hashes(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_omit_resource_measurement(self):
        self.reject(lambda r: r["cases"][0]["construction"].update(peak_rss_bytes=0), "memory measurement")

    def test_cannot_cross_source_inventory(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source inventory")


if __name__ == "__main__":
    main()
