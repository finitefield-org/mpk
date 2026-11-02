"""Reject lost full-width/finite-byte domains and overstated string evidence."""

import copy
import json
from unittest import TestCase, main

import byte_differential
from check_byte_report import check, check_initial
from run_byte_probes import RECORD


class ByteReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 46)

    def test_original_true_claim_attempts(self):
        self.assertEqual(check_initial(), 2)

    def test_one_cell_control_is_not_negative(self):
        self.reject(lambda r: r["cases"][3]["case"].update(expected="rejected"), "candidate identity")

    def test_cannot_omit_finite_embedding(self):
        self.reject(lambda r: r["cases"].pop(-5), "case coverage")

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_remaining_byte_laws(self):
        self.reject(lambda r: r["pending"].pop(1), "pending scope")

    def test_cannot_promote_to_source_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go string operations verified"), "candidate scope")

    def test_cannot_use_only_low_index_bits(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(full_index_bits=8), "source or byte domain")

    def test_cannot_use_only_small_length_type(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(full_length_bits=8), "source or byte domain")

    def test_cannot_discard_backing_bytes(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_storage_bits=8), "complete finite backing")

    def test_cannot_replace_bytes_with_boolean_cells(self):
        self.reject(lambda r: r["cases"][0]["case"].update(payload_bits=1), "source or byte domain")

    def test_comparisons_quantify_both_stores(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(two_stores=False), "complete finite backing")

    def test_cannot_admit_profile_capacity(self):
        self.reject(lambda r: r.update(boundary_probe_scope="profile capacity is now 256"), "capacity admission")

    def test_cannot_label_guard_erasure_accepted(self):
        self.reject(lambda r: r["cases"][13]["case"].update(expected="accepted"), "candidate identity")

    def test_cannot_shadow_files(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_cross_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_independent_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axiom(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_logical_hash(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_cross_both_accepted_observations(self):
        self.reject(lambda r: r["cases"][0].update(observations=copy.deepcopy(r["cases"][2]["observations"])), "checker/certificate identity")

    def test_cannot_count_fuel_as_false_claim(self):
        def change(r):
            row = r["cases"][1]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source_identity(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_freeze_probe_ceiling(self):
        self.reject(lambda r: r.update(ceiling_scope="profile checker budget"), "resource budget")

    def test_cannot_hide_native_go_failure(self):
        self.reject(lambda r: r["byte_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_cannot_promote_observations_to_proof(self):
        self.reject(lambda r: r["byte_observations"].update(scope="source-bound proof"), "observation scope")

    def test_padding_must_not_change_logical_order(self):
        cases = byte_differential.inputs()
        outputs = json.loads((RECORD / "byte-output.json").read_text())
        i = next(i for i, row in enumerate(outputs) if row["less"] != row["raw_less"])
        outputs[i]["less"] = outputs[i]["raw_less"]
        with self.assertRaisesRegex(ValueError, "byte mismatch"):
            byte_differential.compare(cases, outputs)

    def test_negative_index_must_remain_undefined(self):
        cases = byte_differential.inputs()
        outputs = json.loads((RECORD / "byte-output.json").read_text())
        i = next(i for i, row in enumerate(cases) if int(row["index"]) < 0)
        outputs[i]["index_defined"] = True
        with self.assertRaisesRegex(ValueError, "byte mismatch"):
            byte_differential.compare(cases, outputs)

    def test_native_observations_require_actual_booleans(self):
        cases = byte_differential.inputs()
        outputs = json.loads((RECORD / "byte-output.json").read_text())
        outputs[0]["input_unchanged"] = 1
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            byte_differential.compare(cases, outputs)

    def test_invalid_utf8_and_nul_are_in_observed_domain(self):
        values = [bytes.fromhex(r["left"])[:r["left_length"]] for r in byte_differential.inputs()]
        self.assertTrue(any(b"\x00" in value for value in values))
        self.assertTrue(any(b"\xff" in value for value in values))
        self.assertTrue(any(value == "あ".encode() for value in values))


if __name__ == "__main__":
    main()
