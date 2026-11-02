"""Reject oracle premises, partial frames and overstated halting-return evidence."""

import copy
import json
from unittest import TestCase, main

from check_returned_report import check
import quote_differential
from run_returned_probes import RECORD


class ReturnedReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 108)

    def test_cannot_drop_first_failure(self):
        self.reject(lambda r: r["cases"].pop(-1), "case coverage")

    def test_cannot_claim_w03_complete(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_source_binding(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_promote_model_to_go_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go function verified"), "candidate scope")

    def test_cannot_assume_success_checks(self):
        self.reject(lambda r: r["cases"][4]["case"].update(premises="every item already passes all checks"), "return premises")

    def test_cannot_observe_only_payload(self):
        self.reject(lambda r: r["cases"][0]["case"].update(frame_bits=64), "full frame")

    def test_cannot_hide_arbitrary_initial_frame(self):
        self.reject(lambda r: r["cases"][0]["case"].update(root_entry=True), "root entry binding")

    def test_cannot_ground_payload(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_storage_bits=64), "payload quantification")

    def test_cannot_ignore_padding(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(storage_capacity=129), "capacity identity")

    def test_cannot_admit_129(self):
        self.reject(lambda r: r.update(boundary_probe_scope="129 admitted"), "capacity admission")

    def test_altered_claims_are_semantic_negatives(self):
        self.reject(lambda r: r["cases"][1]["case"].update(expected="accepted"), "candidate identity")

    def test_cannot_shadow_candidate_file(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_cross_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_go_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_checker_hashes(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_count_fuel_as_type_failure(self):
        def change(r):
            row = r["cases"][1]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_probe_budget(self):
        self.reject(lambda r: r.update(ceiling_scope="frozen budget"), "resource budget")

    def test_cannot_drop_natural_quantifier(self):
        self.reject(lambda r: r["cases"][6]["case"].update(initial_natural_quantified=False), "observer quantification")

    def test_cannot_promote_observed_go_to_proof(self):
        self.reject(lambda r: r["quote_observations"].update(scope="whole Go function proved"), "observation scope")

    def test_cannot_hide_actual_go_failure(self):
        self.reject(lambda r: r["quote_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_failure_result_really_is_zero(self):
        cases = quote_differential.inputs()
        observations = json.loads((RECORD / "quote-output.json").read_text())
        index = next(i for i, o in enumerate(observations) if o["code"] != 0)
        observations[index]["total"] = "1"
        with self.assertRaisesRegex(ValueError, "Quote mismatch"):
            quote_differential.compare(cases, observations)

    def test_closed_error_reasons_are_distinct(self):
        cases = quote_differential.inputs()
        observations = json.loads((RECORD / "quote-output.json").read_text())
        index = next(i for i, o in enumerate(observations) if o["code"] == 2)
        observations[index]["code"] = 1
        with self.assertRaisesRegex(ValueError, "Quote mismatch"):
            quote_differential.compare(cases, observations)


if __name__ == "__main__":
    main()
