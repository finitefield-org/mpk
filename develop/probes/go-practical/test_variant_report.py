import copy
from unittest import TestCase, main
from check_scope_report import strict_json
from check_variant_report import check
from run_variant_probes import RECORD


class VariantReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = strict_json((RECORD / "observations.json").read_bytes())

    def reject(self, edit, reason):
        report = copy.deepcopy(self.report)
        edit(report)
        with self.assertRaisesRegex((ValueError, RuntimeError), reason):
            check(report)

    def test_actual(self):
        self.assertEqual(check(self.report), 13)

    def test_no_source_totality(self):
        self.reject(lambda r: r.update(scope="all Go loops terminate"), "not source/public")

    def test_no_completion(self):
        self.reject(lambda r: r.update(status="W04 complete"), "not source/public")

    def test_remaining_work(self):
        self.reject(lambda r: r["pending"].pop(), "remaining source/freeze")

    def test_source_pins(self):
        self.reject(lambda r: r["inputs"].pop(), "source identities")

    def test_unchanged_kernels(self):
        self.reject(lambda r: r["mirrored_checker_inputs"].pop(), "mirrored checker")

    def test_exact_go_binary(self):
        self.reject(lambda r: r["checker_binaries"].update(go="0" * 64), "unchanged Go checker")

    def test_bound_is_not_caller_oracle(self):
        self.reject(lambda r: r["cases"].pop(7), "premise/corollary coverage")

    def test_strict_decrease_present(self):
        self.reject(lambda r: r["cases"].pop(2), "premise/corollary coverage")

    def test_zero_rank_exiting_edge_present(self):
        self.reject(lambda r: r["cases"].pop(9), "premise/corollary coverage")

    def test_candidate_identity(self):
        self.reject(lambda r: r["cases"][0].update(raw_sha256="0" * 64), "candidate bytes")

    def test_no_single_checker(self):
        self.reject(lambda r: r["cases"][0]["observations"].pop("go"), "independent checkers")

    def test_no_axioms(self):
        self.reject(lambda r: r["cases"][7]["observations"]["rust"]["report"].update(axiom_count=1), "not zero-axiom")

    def test_hash_agreement(self):
        self.reject(lambda r: r["cases"][0]["observations"]["go"]["report"]["hashes"].update(export="0" * 64), "hashes disagree")

    def test_fuel_is_not_falsity(self):
        self.reject(lambda r: r["cases"][1]["observations"]["rust"]["report"].update(error_detail='{"code":"CORE_FUEL_EXHAUSTED"}'), "fuel/other")

    def test_no_finite_unrolling_claim(self):
        self.reject(lambda r: r["cases"][0]["case"].update(iteration_domain="enumerated eight steps"), "exact case index")

    def test_full_frame_required(self):
        self.reject(lambda r: r["cases"][0]["case"].update(frame_bits=64), "exact case index")

    def test_private_budget(self):
        self.reject(lambda r: r.update(ceiling_scope="normative profile budget"), "private resource")

    def test_json_boolean_not_numeric_parallelism(self):
        self.reject(lambda r: r.update(direct_child_parallelism=True), "private resource")

    def test_required_regressions(self):
        self.reject(lambda r: r["regressions"].pop(), "regression coverage")

    def test_regression_identity(self):
        self.reject(lambda r: r["regressions"][0].update(raw_sha256="0" * 64), "regression bytes")

    def test_independent_regression_checker(self):
        self.reject(lambda r: r["regressions"][0]["observations"].pop("go"), "regression checkers")

    def test_outcome_counts(self):
        self.reject(lambda r: r["outcomes"].update(ordinary_accepted=9), "outcome counts")

    def test_closed_shape(self):
        self.reject(lambda r: r.update(source_binding_proved=True), "closed report")


if __name__ == "__main__":
    main()
