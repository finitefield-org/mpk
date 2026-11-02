"""Reject weakened guards, dropped signed domains and overstated evidence."""

import copy
import json
from unittest import TestCase, main

from check_signed_prefix_report import check
from run_signed_prefix_probes import RECORD


class SignedPrefixReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 32)

    def test_cannot_drop_boundary(self):
        self.reject(lambda r: r["cases"].pop(), "case coverage")

    def test_cannot_claim_completion(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_source_guard_pending(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_claim_go_application(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go application verified"), "candidate scope")

    def test_cannot_weaken_actual_guard(self):
        self.reject(lambda r: r["cases"][4]["case"].update(premise="nonnegative inputs only"), "sign-carry premise")

    def test_cannot_replace_signed_payload_by_small_values(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_storage_bits=64), "payload quantification")

    def test_cannot_ignore_padding(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(storage_capacity=129), "capacity identity")

    def test_cannot_admit_129(self):
        self.reject(lambda r: r.update(boundary_probe_scope="129 admitted"), "profile admission")

    def test_vacuous_empty_guard_is_not_negative(self):
        self.reject(lambda r: r["cases"][2]["case"].update(expected="rejected"), "candidate identity")

    def test_cannot_replace_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_checker_hashes(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_relabel_fuel(self):
        self.reject(lambda r: r["cases"][1].update(classification="core_type_mismatch"), "counted as semantic negative")

    def test_cannot_cross_source(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_resource_ceiling(self):
        self.reject(lambda r: r.update(ceiling_scope="frozen budget"), "resource budget")


if __name__ == "__main__":
    main()
