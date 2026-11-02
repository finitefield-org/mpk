import copy
from unittest import TestCase,main
from check_scope_report import strict_json
from check_termination_report import check
from run_termination_probes import RECORD


class TerminationReportTests(TestCase):
    @classmethod
    def setUpClass(cls):cls.report=strict_json((RECORD/"observations.json").read_bytes())
    def reject(self,edit,reason):
        r=copy.deepcopy(self.report);edit(r)
        with self.assertRaisesRegex((ValueError,RuntimeError),reason):check(r)
    def test_actual(self):self.assertEqual(check(self.report),39)
    def test_no_full_variant_claim(self):self.reject(lambda r:r.update(scope="all Go loops terminate"),"not source/public/variant")
    def test_no_completion(self):self.reject(lambda r:r.update(status="W04 complete"),"not source/public/variant")
    def test_remaining_work(self):self.reject(lambda r:r["pending"].pop(),"remaining termination")
    def test_source_pins(self):self.reject(lambda r:r["inputs"].pop(),"source identities")
    def test_mirrored_sources(self):self.reject(lambda r:r["mirrored_checker_inputs"].pop(),"mirrored checker")
    def test_go_distribution(self):self.reject(lambda r:r["host_toolchain_inventory"].update(sha256="0"*64),"exact toolchain")
    def test_no_frozen_budget(self):self.reject(lambda r:r.update(ceiling_scope="normative profile budget"),"private resource")
    def test_strong_induction_present(self):self.reject(lambda r:r["cases"].pop(8),"ordinary coverage")
    def test_isolated_prior_opaque_present(self):self.reject(lambda r:r["interface_cases"].pop(16),"interface coverage")
    def test_candidate_bytes(self):self.reject(lambda r:r["cases"][0].update(raw_sha256="0"*64),"candidate bytes")
    def test_no_one_checker_acceptance(self):self.reject(lambda r:r["cases"][0]["observations"].pop("go"),"independent checkers")
    def test_no_axioms(self):self.reject(lambda r:r["cases"][0]["observations"]["rust"]["report"].update(axiom_count=1),"not zero-axiom")
    def test_no_crossed_hashes(self):self.reject(lambda r:r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0"*64),"hashes disagree")
    def test_fuel_not_false(self):self.reject(lambda r:r["cases"][1]["observations"]["rust"]["report"].update(error_detail='{"code":"CORE_FUEL_EXHAUSTED"}'),"fuel/other")
    def test_interface_not_semantic_negative(self):self.reject(lambda r:r["interface_cases"][3].update(classification="core_type_mismatch"),"classification")
    def test_no_dropped_regressions(self):self.reject(lambda r:r["regressions"].pop(),"legacy coverage")
    def test_legacy_bytes(self):self.reject(lambda r:r["regressions"][0].update(raw_sha256="0"*64),"legacy bytes")
    def test_counts(self):self.reject(lambda r:r["outcomes"].update(ordinary_accepted=16),"outcome counts")
    def test_closed_shape(self):self.reject(lambda r:r.update(source_proved=True),"closed report")
    def test_binary_pin(self):self.reject(lambda r:r["checker_binaries"].update(go="unknown"),"binary identities")


if __name__=="__main__":main()
