"""Keep actual validity, source outcomes and complete address coverage distinct."""

import copy
import json
from unittest import TestCase, main

import access_differential as native
from check_access_report import check
from run_access_probes import RECORD


class AccessReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())
        cls.inputs = native.inputs()
        cls.outputs = json.loads((RECORD / "access-output.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    @staticmethod
    def case(report, capacity, family, mutation=0):
        return next(r for r in report["cases"] if (r["case"]["capacity"], r["case"]["family"], r["case"]["mutation"]) == (capacity, family, mutation))

    def output(self, wanted):
        return self.outputs[self.inputs.index(wanted)]

    def reject_native(self, wanted, field, value, pattern="access mismatch"):
        outputs = copy.deepcopy(self.outputs)
        outputs[self.inputs.index(wanted)][field] = value
        with self.assertRaisesRegex(ValueError, pattern):
            native.compare(self.inputs, outputs)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 104)

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_source_ownership_obligations(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_promote_to_application_theorem(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="captured Go function theorem"), "candidate scope")

    def test_cannot_admit_padding_as_capacity(self):
        self.reject(lambda r: r.update(boundary_probe_scope="Go accepts storage up to the padded capacity"), "padding promoted")

    def test_cannot_round_127_semantic_bound_to_128(self):
        self.reject(lambda r: self.case(r, 127, "finite-read")["case"].update(capacity=128), "case coverage")

    def test_cannot_round_129_semantic_bound_to_256(self):
        self.reject(lambda r: self.case(r, 129, "full-value-coverage")["case"].update(capacity=256), "case coverage")

    def test_cannot_freeze_private_timeout(self):
        self.reject(lambda r: r.update(ceiling_scope="Go profile budget"), "resource budget")

    def test_cannot_omit_zero_storage_boundary(self):
        self.reject(lambda r: r["construction_groups"].pop(0), "construction coverage")

    def test_cannot_omit_erased_validity_case(self):
        self.reject(lambda r: r["cases"].remove(self.case(r, 1, "physical-bounds", 1)), "case coverage")

    def test_cannot_shadow_candidate_identity(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_truncate_source_words(self):
        self.reject(lambda r: self.case(r, 129, "full-value-coverage")["case"].update(source_word_bits=8), "full source/descriptor/element")

    def test_cannot_use_wrapped_extent_as_validity(self):
        self.reject(lambda r: r["cases"][0]["case"].update(extent_intermediate_bits=64), "computed widened extent")

    def test_cannot_erase_descriptor_metadata(self):
        self.reject(lambda r: r["cases"][0]["case"].update(descriptor_bits=128), "full source/descriptor/element")

    def test_cannot_keep_only_one_scalar(self):
        self.reject(lambda r: self.case(r, 128, "finite-read")["case"].update(element_bits=64), "full source/descriptor/element")

    def test_cannot_drop_padding_or_other_region(self):
        self.reject(lambda r: self.case(r, 129, "finite-read")["case"].update(quantified_storage_bits=129 * 128), "complete backing/padding")

    def test_cannot_conflate_logical_regions(self):
        self.reject(lambda r: r["cases"][0]["case"].update(logical_region_count=1), "region/source pointer")

    def test_cannot_admit_source_pointer_operations(self):
        self.reject(lambda r: r["cases"][0]["case"].update(source_pointer_bits_admitted=True), "region/source pointer")

    def test_cannot_accept_erased_storage_bound(self):
        self.reject(lambda r: self.case(r, 128, "full-value-coverage", 1)["case"].update(expected="accepted"), "candidate identity")

    def test_cannot_cross_raw_certificate(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_independent_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_introduce_axiom(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_export_hash(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_cross_both_accepted_observations(self):
        self.reject(lambda r: r["cases"][0].update(observations=copy.deepcopy(r["cases"][2]["observations"])), "checker/certificate identity")

    def test_cannot_count_fuel_as_false_claim(self):
        def change(r):
            row = self.case(r, 128, "full-value-coverage", 1)
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source_pins(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_substitute_toolchain(self):
        self.reject(lambda r: r["host_toolchain_inventory"].update(files=14495), "toolchain inventory")

    def test_cannot_hide_observer_failure(self):
        self.reject(lambda r: r["access_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_cannot_treat_raw_metadata_as_incoming_go(self):
        self.reject(lambda r: r["access_observations"].update(scope="every tuple is an incoming Go slice"), "observation scope/byte")

    def test_cannot_omit_false_claim_witness(self):
        self.reject(lambda r: r["access_observations"]["false_claim_witnesses"].pop(), "false-claim witnesses")

    def test_cannot_reclassify_invalid_representation_as_legal_go(self):
        def change(r):
            witness = next(w for w in r["access_observations"]["false_claim_witnesses"] if w["domain"] == "raw_invalid_representation")
            witness["domain"] = "bounded_source"
        self.reject(change, "false-claim witnesses")

    def test_cannot_reclassify_forged_live_binding_as_go_panic(self):
        def change(r):
            witness = next(w for w in r["access_observations"]["false_claim_witnesses"] if w["domain"] == "raw_invalid_binding")
            witness["domain"] = "source_index_panic"
        self.reject(change, "false-claim witnesses")

    def test_boolean_witness_is_not_numeric_one(self):
        self.reject(lambda r: r["access_observations"]["false_claim_witnesses"][0].update(left=1), "false-claim witnesses")

    def test_invalid_representation_is_construction_failure(self):
        actual = self.output(native.row(1, 1, 0, 2, 1, 1))
        self.assertTrue(actual["guard"])
        self.assertFalse(actual["valid"] or actual["constructed"] or actual["physical_bound"])
        self.assertTrue(actual["construction_panicked"])
        self.assertFalse(actual["read_panicked"])

    def test_forged_dead_binding_can_still_be_a_valid_go_read(self):
        wanted = native.row(1, 1, 0, 1, 1, 0, dead=True)
        actual = self.output(wanted)
        self.assertFalse(actual["valid"] or actual["selected_live"])
        self.assertTrue(actual["constructed"] and actual["read_defined"])
        self.reject_native(wanted, "valid", True)

    def test_padding_does_not_raise_127_or_129_bounds(self):
        for b in (127, 129):
            actual = self.output(native.row(b, b + 1, 0, b + 1, b + 1, b))
            self.assertTrue(actual["valid"] and actual["guard"] and actual["covered"] and actual["read_defined"])
            self.assertFalse(actual["storage_bound"])
            self.reject_native(native.row(b, b + 1, 0, b + 1, b + 1, b), "storage_bound", True)

    def test_erased_bound_loses_actual_high_value_bits(self):
        for b in (1, 8, 32, 127, 128, 129):
            p = native.padding(b)
            actual = self.output(native.row(b, p + 1, 0, p + 1, p + 1, p))
            self.assertTrue(actual["valid"] and actual["guard"] and actual["read_defined"])
            self.assertFalse(actual["covered"] or actual["storage_bound"])
            self.assertEqual(actual["absolute"], str(p))
            self.assertEqual(actual["reconstructed"], "0")
            self.assertNotEqual(actual["actual_value"], actual["modeled_value"])

    def test_modulo_heap_read_alone_cannot_witness_value_coverage(self):
        # The total finite-heap lift reads low address bits even outside B.
        # Its self-equality can be true while the full numeric address differs.
        a = self.output(native.row(128, 129, 0, 129, 129, 128))
        self.assertFalse(a["covered"])
        self.assertEqual(a["modeled_value"], self.output(native.row(128, 128, 0, 128, 128, 0))["actual_value"])
        witnesses = self.report["access_observations"]["false_claim_witnesses"]
        self.assertTrue(all(w["case_id"].endswith("zeroed-read") for w in witnesses if "finite-read" in w["case_id"]))

    def test_physical_offset_reconstructs_complete_cell(self):
        wanted = native.row(128, 128, 1, 127, 127, 126, region=True)
        actual = self.output(wanted)
        self.assertTrue(actual["valid"] and actual["guard"] and actual["covered"])
        self.assertFalse(actual["absolute_carry"])
        self.assertEqual(actual["absolute"], "127")
        self.assertEqual(actual["actual_value"], actual["modeled_value"])
        amount, tag = actual["actual_value"].split(":")
        self.assertNotEqual(amount, "0" * 16)
        self.assertNotEqual(tag, "0" * 16)
        self.reject_native(wanted, "modeled_value", amount + ":" + "0" * 16)

    def test_nil_and_non_nil_empty_both_have_no_access(self):
        for nil_tag in (False, True):
            actual = self.output(native.row(0, 0, 0, 0, 0, 0, nil=nil_tag))
            self.assertTrue(actual["valid"] and actual["constructed"] and actual["storage_bound"])
            self.assertFalse(actual["guard"] or actual["read_defined"])
            self.assertTrue(actual["read_panicked"])
        self.assertFalse(any(a["guard"] for r, a in zip(self.inputs, self.outputs) if r["profile"] == 0 and a["valid"] and a["storage_bound"]))

    def test_erased_zero_storage_bound_is_nonvacuous(self):
        actual = self.output(native.row(0, 1, 0, 1, 1, 0))
        self.assertTrue(actual["valid"] and actual["guard"] and actual["read_defined"])
        self.assertFalse(actual["storage_bound"])

    def test_copy_and_full_preserve_validity(self):
        for actual in self.outputs:
            if actual["valid"]:
                self.assertTrue(actual["copy_valid"] and actual["full_valid"])
        wanted = native.row(128, 128, 0, 1, 128, 0)
        self.reject_native(wanted, "full_valid", False)

    def test_extreme_indexes_are_actual_go_index_panics(self):
        for r, actual in zip(self.inputs, self.outputs):
            if actual["constructed"] and r["index"] in (str(-(1 << 63)), str((1 << 63) - 1)):
                self.assertFalse(actual["guard"] or actual["read_defined"])
                self.assertTrue(actual["read_panicked"])

    def test_extreme_descriptor_fields_are_not_truncated(self):
        for bad in (-(1 << 63), -1, (1 << 63) - 1):
            for wanted in (native.row(128, 128, bad, 0, 0, 0),
                           native.row(128, 128, 0, bad, 1, 0),
                           native.row(128, 128, 0, 0, bad, 0)):
                actual = self.output(wanted)
                self.assertFalse(actual["valid"] or actual["constructed"])
                self.assertTrue(actual["construction_panicked"])
        wanted = native.row(128, 128, -1, 1, 1, 1)
        actual = self.output(wanted)
        self.assertTrue(actual["extent_carry"] and actual["absolute_carry"])
        self.assertEqual(actual["extent"], "0")
        self.assertFalse(actual["valid"] or actual["constructed"])
        self.reject_native(wanted, "extent_carry", False)

    def test_native_flags_require_booleans(self):
        self.reject_native(self.inputs[0], "valid", 1, "field/type mismatch")

    def test_observation_domains_all_have_nonvacuous_cases(self):
        outcomes = native.compare(self.inputs, self.outputs)
        self.assertEqual(len(self.inputs), 1307)
        self.assertEqual(outcomes, {"proof_domain_reads": 157, "source_reads": 174, "construction_rejections": 98,
                                    "index_panics": 1035, "invalid_metadata_with_local_guard": 21,
                                    "dead_binding_constructible": 7, "out_of_profile_reads": 11,
                                    "padding_not_admission": 4, "uncovered_source_reads": 7})


if __name__ == "__main__":
    main()
