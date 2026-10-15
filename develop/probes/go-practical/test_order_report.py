"""Reject weakened path domains, crossed evidence and empirical proof claims."""

import copy
import json
from unittest import TestCase, main

from check_order_report import check
import order_differential
from run_order_probes import RECORD


class OrderReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 40)

    def test_cannot_drop_64bit_path(self):
        def change(r):
            index = next(i for i, row in enumerate(r["cases"]) if row["case"]["width"] == 64 and row["case"]["family"] == "path" and row["case"]["mutation"] == 0)
            r["cases"].pop(index)
        self.reject(change, "case coverage")

    def test_cannot_shadow_candidate_file(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_claim_completion(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_loop_pending(self):
        self.reject(lambda r: r["pending"].pop(1), "pending scope")

    def test_cannot_promote_to_application_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go application verified"), "candidate scope")

    def test_cannot_ground_operands(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_operand_bits=1), "operand quantification")

    def test_cannot_drop_exact_natural_observer(self):
        def change(r):
            next(row for row in r["cases"] if row["case"]["family"] == "natural")["case"]["initial_natural_quantified"] = False
        self.reject(change, "observer quantification")

    def test_cannot_weaken_comparison_premises(self):
        self.reject(lambda r: r["cases"][0]["case"].update(premises="arbitrary success flag"), "path premises")

    def test_cannot_admit_uint128(self):
        self.reject(lambda r: r.update(internal_width_128_scope="Go uint128 admitted"), "source width")

    def test_cannot_cross_certificate(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_hashes(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_count_fuel_as_type_rejection(self):
        def change(r):
            row = r["cases"][1]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_probe_timeout(self):
        self.reject(lambda r: r.update(ceiling_scope="frozen application budget"), "resource budget")

    def test_cannot_promote_empirical_to_universal_proof(self):
        self.reject(lambda r: r["differential"].update(scope="Go universal theorem"), "differential scope")

    def test_cannot_hide_actual_go_failure(self):
        self.reject(lambda r: r["differential"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_empirical_model_cannot_ignore_negative_inputs(self):
        cases = order_differential.inputs()
        observations = json.loads((RECORD / "go-reference-output.json").read_text())
        cases = [c for c, o in zip(cases, observations) if o["continues"]]
        observations = [o for o in observations if o["continues"]]
        with self.assertRaisesRegex(ValueError, "input coverage"):
            order_differential.compare(cases, observations)

    def test_empirical_comparison_detects_wrong_signs(self):
        cases = order_differential.inputs()
        observations = json.loads((RECORD / "go-reference-output.json").read_text())
        observations[0]["item_nonnegative"] = True
        with self.assertRaisesRegex(ValueError, "operation mismatch"):
            order_differential.compare(cases, observations)

    def test_empirical_observation_requires_booleans(self):
        cases = order_differential.inputs()
        observations = json.loads((RECORD / "go-reference-output.json").read_text())
        observations[0]["item_nonnegative"] = 0
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            order_differential.compare(cases, observations)


if __name__ == "__main__":
    main()
