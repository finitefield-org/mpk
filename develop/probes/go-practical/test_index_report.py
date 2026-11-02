"""Reject truncated index domains, hidden premises and false proof readiness."""

import copy
import json
from unittest import TestCase, main

from check_index_report import check
import index_differential
from run_index_probes import RECORD


class IndexReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 27)

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_source_binding(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_promote_to_application_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go indexing proved"), "candidate scope")

    def test_cannot_admit_public_capacity(self):
        self.reject(lambda r: r.update(boundary_probe_scope="Go profile capacity frozen at 256"), "capacity admission")

    def test_cannot_freeze_private_timeout(self):
        self.reject(lambda r: r.update(ceiling_scope="profile checker budget"), "resource budget")

    def test_cannot_omit_empty_case(self):
        self.reject(lambda r: r["cases"].pop(4), "case coverage")

    def test_cannot_omit_high_capacity(self):
        self.reject(lambda r: r["construction_groups"].pop(), "construction coverage")

    def test_cannot_shadow_files(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_replace_full_index_with_low_bits(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(full_index_bits=8), "full source domain")

    def test_cannot_replace_source_length_with_small_bound(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(full_length_bits=8), "full source domain")

    def test_cannot_shorten_source_word(self):
        self.reject(lambda r: r["cases"][0]["case"].update(source_word_bits=32), "full source domain")

    def test_cannot_add_fit_oracle(self):
        self.reject(lambda r: r["cases"][10]["case"].update(premises="caller certifies index already fits into zero bits"), "fit oracle")

    def test_cannot_hide_erased_guard(self):
        self.reject(lambda r: r["cases"][12]["case"].update(premises=r["cases"][10]["case"]["premises"]), "actual premises")

    def test_cannot_accept_erased_bound(self):
        self.reject(lambda r: r["cases"][11]["case"].update(expected="accepted"), "candidate identity")

    def test_cannot_cross_certificate_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_independent_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axioms(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_export_hash(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_replace_both_checker_observations(self):
        self.reject(lambda r: r["cases"][0].update(observations=copy.deepcopy(r["cases"][2]["observations"])), "checker/certificate identity")

    def test_cannot_count_fuel_as_false_claim_evidence(self):
        def change(r):
            row = r["cases"][1]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source_identity(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_hide_native_failure(self):
        self.reject(lambda r: r["index_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_cannot_promote_native_observations_to_proof(self):
        self.reject(lambda r: r["index_observations"].update(scope="source-bound proof"), "observation scope")

    def test_cannot_omit_false_claim_witness(self):
        self.reject(lambda r: r["index_observations"]["negative_claim_witnesses"].pop(), "false-claim witnesses")

    def test_cannot_substitute_type_failure_for_counterexample(self):
        self.reject(lambda r: r["index_observations"].update(negative_witness_scope="checker rejection proves falsity"), "false-claim witnesses")

    def native(self):
        return index_differential.inputs(), json.loads((RECORD / "index-output.json").read_text())

    def test_beyond_private_bound_can_be_valid_go(self):
        cases, outputs = self.native()
        i = cases.index({"capacity": 32, "length": 33, "index": "32"})
        self.assertTrue(outputs[i]["actual_defined"])
        self.assertFalse(outputs[i]["bound"])
        self.assertFalse(outputs[i]["covered"])
        outputs[i]["actual_defined"] = False
        with self.assertRaisesRegex(ValueError, "index mismatch"):
            index_differential.compare(cases, outputs)

    def test_single_cell_embedding_is_not_modulo_equality(self):
        cases, outputs = self.native()
        i = cases.index({"capacity": 1, "length": 2, "index": "1"})
        self.assertTrue(outputs[i]["actual_defined"])
        self.assertEqual(outputs[i]["unsigned_index"], "1")
        self.assertEqual(outputs[i]["low_index"], "0")
        self.assertEqual(outputs[i]["actual_byte"], 48)
        self.assertFalse(outputs[i]["covered"])

    def test_erasing_zero_bound_has_real_counterexample(self):
        cases, outputs = self.native()
        i = cases.index({"capacity": 0, "length": 1, "index": "0"})
        self.assertTrue(outputs[i]["actual_defined"])
        self.assertFalse(outputs[i]["bound"])
        witnesses = self.report["index_observations"]["negative_claim_witnesses"]
        witness = next(w for w in witnesses if w["case_id"] == "index-bound-0-empty-no-index-erased-bound")
        self.assertEqual(witness["input_position"], i)
        self.assertIs(witness["left"], True)
        self.assertIs(witness["right"], False)

    def test_negative_and_large_positive_indexes_panic(self):
        cases, outputs = self.native()
        for row, actual in zip(cases, outputs):
            if row["index"] in (str(-(1 << 63)), str((1 << 63) - 1)):
                self.assertFalse(actual["source_guard"])
                self.assertFalse(actual["actual_defined"])

    def test_native_flags_require_actual_booleans(self):
        cases, outputs = self.native()
        outputs[0]["source_guard"] = 0
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            index_differential.compare(cases, outputs)

    def test_high_bits_cannot_disappear_in_native_observation(self):
        cases, outputs = self.native()
        i = cases.index({"capacity": 256, "length": 257, "index": "256"})
        outputs[i]["covered"] = True
        with self.assertRaisesRegex(ValueError, "index mismatch"):
            index_differential.compare(cases, outputs)


if __name__ == "__main__":
    main()
