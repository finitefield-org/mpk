"""Reject oracle premises, missing zero entry and overstated Go prefix evidence."""

import copy
import json
from unittest import TestCase, main

from check_continuation_report import check, check_initial
import quote_differential
from run_continuation_probes import RECORD


class ContinuationReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 72)

    def test_original_finding(self):
        self.assertEqual(check_initial(), 1)

    def test_cannot_drop_zero_entry(self):
        self.reject(lambda r: r["cases"].pop(7), "case coverage")

    def test_cannot_claim_w03_complete(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_error_binding(self):
        self.reject(lambda r: r["pending"].pop(1), "pending scope")

    def test_cannot_promote_to_application_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go function verified"), "candidate scope")

    def test_cannot_assume_all_intermediate_bounds(self):
        self.reject(lambda r: r["cases"][0]["case"].update(premises="every intermediate accumulator already satisfies the invariant"), "actual item checks")

    def test_cannot_replace_zero_with_unknown_initial(self):
        self.reject(lambda r: r["cases"][5]["case"].update(quantified_accumulator_bits=64), "accumulator binding")

    def test_cannot_ground_payload(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_storage_bits=64), "payload quantification")

    def test_cannot_ignore_padding(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(storage_capacity=129), "capacity identity")

    def test_cannot_admit_129(self):
        self.reject(lambda r: r.update(boundary_probe_scope="129 admitted"), "capacity admission")

    def test_empty_controls_are_not_negative(self):
        self.reject(lambda r: r["cases"][1]["case"].update(expected="rejected"), "candidate identity")

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
            row = r["cases"][4]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_probe_budget(self):
        self.reject(lambda r: r.update(ceiling_scope="frozen budget"), "resource budget")

    def test_cannot_promote_observed_go_to_proof(self):
        self.reject(lambda r: r["quote_observations"].update(scope="whole Go function proved"), "observation scope")

    def test_cannot_hide_actual_go_failure(self):
        self.reject(lambda r: r["quote_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_observations_preserve_error_paths(self):
        cases = quote_differential.inputs()
        observations = json.loads((RECORD / "quote-output.json").read_text())
        with self.assertRaisesRegex(ValueError, "input coverage"):
            quote_differential.compare([c for c, o in zip(cases, observations) if o["code"] == 0], [o for o in observations if o["code"] == 0])

    def test_observations_reject_frame_mutation(self):
        cases = quote_differential.inputs()
        observations = json.loads((RECORD / "quote-output.json").read_text())
        observations[0]["input_unchanged"] = False
        with self.assertRaisesRegex(ValueError, "Quote mismatch"):
            quote_differential.compare(cases, observations)

    def test_observations_require_bool_frame(self):
        cases = quote_differential.inputs()
        observations = json.loads((RECORD / "quote-output.json").read_text())
        observations[0]["nil_preserved"] = 1
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            quote_differential.compare(cases, observations)


if __name__ == "__main__":
    main()
