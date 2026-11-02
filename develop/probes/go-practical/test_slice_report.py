"""Reject lossy slice domains, copied-list aliases and overstated evidence."""

import copy
import json
from unittest import TestCase, main

from check_slice_report import check, check_initial
from run_slice_probes import RECORD
from run_proof_pilot import ROOT, sha
import slice_differential


class SliceReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / "observations.json").read_text())

    def reject(self, change, pattern):
        report = copy.deepcopy(self.report)
        change(report)
        with self.assertRaisesRegex(ValueError, pattern):
            check(report)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 90)

    def test_original_failed_positive_diagnostics(self):
        self.assertEqual(check_initial(), 3)

    def test_actual_resource_host_receipt(self):
        receipt = json.loads((RECORD.parent / "local-resource-host.json").read_text())
        self.assertEqual(receipt["schema"], "mpk.go_practical.local_resource_host.v0")
        self.assertEqual(receipt["provider"], "read-only /usr/sbin/sysctl outside workspace sandbox")
        self.assertEqual(receipt["scope"], "adjacent identity of the local Darwin development host for the referenced measurement; original report model/RAM remain null; no isolated-machine guarantee, profile budget or native Linux release receipt")
        self.assertEqual(receipt["observed_host"], {k: self.report["host"][k] for k in ("system", "release", "machine", "cpu_count")})
        target = ROOT / receipt["measurement_ref"]["path"]
        self.assertEqual(target, RECORD / "observations.json")
        self.assertEqual(receipt["measurement_ref"]["sha256"], sha(target.read_bytes()))
        raw = dict(line.split(": ", 1) for line in receipt["raw_stdout"].splitlines())
        hardware = receipt["hardware"]
        self.assertEqual(receipt["command"], ["/usr/sbin/sysctl", *raw.keys()])
        for key in ("physical_memory_bytes", "physical_cpu_count", "logical_cpu_count"):
            self.assertIs(type(hardware[key]), int)
        self.assertEqual(hardware["model"], raw["hw.model"])
        self.assertEqual(hardware["processor"], raw["machdep.cpu.brand_string"])
        self.assertEqual(hardware["physical_memory_bytes"], int(raw["hw.memsize"]))
        self.assertEqual(hardware["physical_cpu_count"], int(raw["hw.physicalcpu"]))
        self.assertEqual(hardware["logical_cpu_count"], int(raw["hw.logicalcpu"]))
        self.assertEqual(hardware["logical_cpu_count"], self.report["host"]["cpu_count"])

    def test_cannot_promote_failed_attempts_to_false_claims(self):
        report = json.loads((RECORD.parent / "slice-initial/observations.json").read_text())
        report["scope"] = "three false model claims"
        with self.assertRaisesRegex(ValueError, "initial slice scope"):
            check_initial(report)

    def test_cannot_reclassify_initial_fuel_as_type_error(self):
        report = json.loads((RECORD.parent / "slice-initial/observations.json").read_text())
        report["groups"][1]["cases"][0]["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_TYPE_MISMATCH"}'
        with self.assertRaisesRegex(ValueError, "initial failure class"):
            check_initial(report)

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status="complete"), "W03 completion")

    def test_cannot_hide_absolute_validity(self):
        self.reject(lambda r: r["pending"].pop(0), "pending scope")

    def test_cannot_promote_to_application_proof(self):
        self.reject(lambda r: r["cases"][0]["case"].update(scope="Go slice source proof"), "candidate scope")

    def test_cannot_admit_capacity_129(self):
        self.reject(lambda r: r.update(boundary_probe_scope="Go now admits capacity129"), "capacity admission")

    def test_cannot_freeze_probe_timeout(self):
        self.reject(lambda r: r.update(ceiling_scope="profile budget"), "resource budget")

    def test_cannot_omit_empty_metadata_case(self):
        self.reject(lambda r: r["cases"].pop(2), "case coverage")

    def test_cannot_omit_boundary(self):
        self.reject(lambda r: r["construction_groups"].pop(), "construction coverage")

    def test_cannot_shadow_candidates(self):
        self.reject(lambda r: r["cases"][1]["case"].update(id=r["cases"][0]["case"]["id"]), "shadowed candidate")

    def test_cannot_use_small_source_words(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(source_word_bits=8), "full source/descriptor/element")

    def test_cannot_drop_descriptor_fields(self):
        self.reject(lambda r: r["cases"][0]["case"].update(descriptor_bits=128), "full source/descriptor/element")

    def test_cannot_keep_only_amount_payload(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(element_bits=64), "full source/descriptor/element")

    def test_cannot_discard_padding_or_other_region(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(quantified_storage_bits=129 * 128), "complete backing/padding")

    def test_cannot_collapse_region_names(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(logical_region_count=1), "region/pointer scope")

    def test_cannot_admit_source_pointers(self):
        self.reject(lambda r: r["cases"][-1]["case"].update(source_pointer_bits_admitted=True), "region/pointer scope")

    def test_cannot_accept_copied_list_alias_model(self):
        self.reject(lambda r: r["cases"][13]["case"].update(expected="accepted"), "candidate identity")

    def test_cannot_cross_certificate_bytes(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "byte identity")

    def test_cannot_omit_independent_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checker")

    def test_cannot_use_axiom(self):
        self.reject(lambda r: r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1), "zero-axiom")

    def test_cannot_cross_export_hash(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_cannot_replace_both_accepted_observations(self):
        self.reject(lambda r: r["cases"][0].update(observations=copy.deepcopy(r["cases"][2]["observations"])), "checker/certificate identity")

    def test_cannot_count_fuel_as_semantic_rejection(self):
        def change(r):
            row = r["cases"][1]
            row["observations"]["rust"]["report"]["error_detail"] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            row["observations"]["go"]["report"]["error_detail"] = "core reduction fuel exhausted"
        self.reject(change, "intended type boundary")

    def test_cannot_cross_source_identity(self):
        self.reject(lambda r: r["inputs"][0].update(sha256="0" * 64), "source identity")

    def test_cannot_substitute_host_toolchain(self):
        self.reject(lambda r: r["host_toolchain_inventory"].update(files=14495), "toolchain inventory")

    def test_cannot_hide_native_failure(self):
        self.reject(lambda r: r["slice_observations"]["go_measurement"].update(exit_code=1), "did not execute")

    def test_cannot_promote_native_to_source_proof(self):
        self.reject(lambda r: r["slice_observations"].update(scope="captured-source theorem"), "observation scope")

    def test_cannot_omit_false_claim_witness(self):
        self.reject(lambda r: r["slice_observations"]["negative_claim_witnesses"].pop(), "false-claim witnesses")

    def test_witness_boolean_is_not_numeric_one(self):
        self.reject(lambda r: r["slice_observations"]["negative_claim_witnesses"][0].update(left=1), "false-claim witnesses")

    def native(self):
        return slice_differential.inputs(), json.loads((RECORD / "slice-output.json").read_text())

    def test_nonnil_empty_does_not_become_nil(self):
        cases, outputs = self.native()
        i = next(i for i, row in enumerate(cases) if not row["nil"] and row["length"] == 0)
        outputs[i]["nil"] = True
        with self.assertRaisesRegex(ValueError, "slice mismatch"):
            slice_differential.compare(cases, outputs)

    def test_view_copy_must_observe_shared_write(self):
        cases, outputs = self.native()
        i = next(i for i, actual in enumerate(outputs) if actual["write_defined"])
        outputs[i]["alias_after"] = outputs[i]["wrong_fresh_view"]
        with self.assertRaisesRegex(ValueError, "slice mismatch"):
            slice_differential.compare(cases, outputs)

    def test_array_value_copy_remains_independent(self):
        cases, outputs = self.native()
        i = next(i for i, actual in enumerate(outputs) if actual["write_defined"])
        self.assertTrue(outputs[i]["array_differs_after_write"])
        outputs[i]["array_copy_preserved"] = False
        with self.assertRaisesRegex(ValueError, "slice mismatch"):
            slice_differential.compare(cases, outputs)

    def test_full_reslice_exposes_nonzero_tail(self):
        cases, outputs = self.native()
        i = next(i for i, actual in enumerate(outputs) if actual["full_defined"] and not actual["read_defined"])
        self.assertNotEqual(outputs[i]["full_before"], slice_differential.ZERO)
        outputs[i]["full_before"] = slice_differential.ZERO
        with self.assertRaisesRegex(ValueError, "slice mismatch"):
            slice_differential.compare(cases, outputs)

    def test_capacity_restriction_survives_full_reslice(self):
        cases, outputs = self.native()
        i = next(i for i, row in enumerate(cases) if row["storage"] == 128 and row["offset"] == 0 and row["capacity"] == 1)
        self.assertEqual(outputs[i]["capacity"], 1)
        self.assertTrue(outputs[i]["full_metadata"])
        outputs[i]["full_metadata"] = False
        with self.assertRaisesRegex(ValueError, "slice mismatch"):
            slice_differential.compare(cases, outputs)

    def test_internal_129_boundary_is_not_a_go_panic(self):
        cases, outputs = self.native()
        i = cases.index({"storage": 129, "offset": 0, "length": 129, "capacity": 129,
                         "index": "128", "nil": False, "seed": "0"})
        self.assertTrue(outputs[i]["read_defined"])
        self.assertTrue(outputs[i]["write_defined"])
        self.assertTrue(outputs[i]["array_copy_preserved"])

    def test_native_flags_require_actual_booleans(self):
        cases, outputs = self.native()
        outputs[0]["read_defined"] = 0
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            slice_differential.compare(cases, outputs)

    def test_extreme_indexes_panic_without_mutation(self):
        cases, outputs = self.native()
        for row, actual in zip(cases, outputs):
            if row["index"] in (str(-(1 << 63)), str((1 << 63) - 1)):
                self.assertFalse(actual["read_defined"])
                self.assertFalse(actual["write_defined"])
                self.assertFalse(actual["array_differs_after_write"])
                self.assertTrue(actual["other_region_frame"] and actual["same_region_frame"])


if __name__ == "__main__":
    main()
