import copy
import tempfile
from pathlib import Path
from unittest import TestCase, main

from check_scope_report import check, json_bytes, sha, strict_json
from scope_cases import CASES, RECORD


class ScopeReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = strict_json((RECORD / "observations.json").read_bytes())
        cls.native = strict_json((RECORD / "native-output.json").read_bytes())

    def reject(self, edit, reason):
        report = copy.deepcopy(self.report)
        edit(report)
        with self.assertRaisesRegex(ValueError, reason):
            check(report)

    def native_reject(self, edit, reason):
        native = copy.deepcopy(self.native)
        edit(native)
        report = copy.deepcopy(self.report)
        with tempfile.TemporaryDirectory(prefix="mpk-w04-test-", dir="/private/tmp") as temporary:
            temp = Path(temporary)
            (temp / "native-input.json").write_bytes((RECORD / "native-input.json").read_bytes())
            raw = json_bytes(native)
            (temp / "native-output.json").write_bytes(raw)
            report["artifacts"]["native-output.json"] = {"bytes": len(raw), "sha256": sha(raw)}
            with self.assertRaisesRegex(ValueError, reason):
                check(report, temp)

    def test_actual_native_report(self):
        self.assertEqual(check(self.report), len(CASES))

    def test_draft_expectation_is_not_admission(self):
        self.reject(lambda r: r.update(scope="practical profile accepted all positive cases"),
                    "not practical-profile support")

    def test_no_false_completion(self):
        self.reject(lambda r: r.update(status="W04 complete"), "not practical-profile support")

    def test_no_hidden_pending_production(self):
        self.reject(lambda r: r["pending"].pop(), "remaining source/proof/freeze")

    def test_no_dropped_source_pin(self):
        self.reject(lambda r: r["inputs"].pop(), "source identity")

    def test_no_toolchain_fallback(self):
        self.reject(lambda r: r["toolchain_inventory"].update(sha256="0"*64),
                    "exact Go toolchain")

    def test_no_linux_release_claim(self):
        self.reject(lambda r: r["build"].update(host="linux/amd64"), "native build/target")

    def test_no_missing_raw_source(self):
        self.reject(lambda r: r["artifacts"].pop("native-input.json"), "artifact coverage")

    def test_no_crossed_raw_output_hash(self):
        self.reject(lambda r: r["artifacts"]["native-output.json"].update(sha256="0"*64),
                    "artifact identity")

    def test_original_source_order(self):
        self.native_reject(lambda r: r["sources"].reverse(), "source order/identity")

    def test_original_type_errors_not_deleted(self):
        self.native_reject(lambda r: next(x for x in r["sources"] if x["id"]=="unused-type-error").update(status="ok"),
                           "original Go status")

    def test_amd64_int_is_not_host_guess(self):
        self.native_reject(lambda r: r["sources"][0].update(int_bits=32), "target-sized")

    def test_nominal_alias_is_retained(self):
        self.native_reject(lambda r: next(t for x in r["sources"] if x["id"]=="named-and-alias-types"
                                        for t in x["types"] if t["name"]=="Alias").update(alias=False),
                           "original nominal/error type")

    def test_boxing_is_not_method_execution(self):
        self.native_reject(lambda r: r["runtime"].update(conversion_error_body_calls=1),
                           "independent runtime")

    def test_direct_call_is_not_interface_dispatch(self):
        self.native_reject(lambda r: next(c for x in r["sources"] if x["id"]=="direct-error-body-call"
                                        for c in x["calls"]).update(kind="interface_invoke"),
                           "call distinction")

    def test_interface_invocation_is_not_conversion(self):
        self.native_reject(lambda r: next(c for x in r["sources"] if x["id"]=="closed-error-body-call"
                                        for c in x["calls"]).update(kind="conversion"),
                           "call distinction")

    def test_call_target_is_bound_to_original_declaration(self):
        self.native_reject(lambda r: next(c for x in r["sources"] if x["id"]=="direct-error-body-call"
                                        for c in x["calls"]).update(target="(other.Code).Error"),
                           "exact original call target")

    def test_struct_copy_does_not_deep_copy_slices(self):
        self.native_reject(lambda r: r["runtime"].update(root_copy_shares_backing=False),
                           "independent runtime")

    def test_non_nil_zero_slice_does_not_become_nil(self):
        self.native_reject(lambda r: r["runtime"]["zero_slices"][1].update(nil=True),
                           "independent runtime")

    def test_nil_zero_slice_does_not_become_non_nil(self):
        self.native_reject(lambda r: r["runtime"]["zero_slices"][0].update(append_empty_nil=False),
                           "independent runtime")

    def test_zero_operations_have_no_backing_writes(self):
        self.native_reject(lambda r: r["runtime"].update(zero_operations_preserve_all_backing=False),
                           "independent runtime")

    def test_positive_append_is_not_a_go_panic(self):
        self.native_reject(lambda r: r["runtime"].update(positive_append_to_zero_succeeds_in_go=False),
                           "independent runtime")

    def test_zero_source_copy_has_no_writes(self):
        self.native_reject(lambda r: r["runtime"].update(zero_source_copy_preserves_all_backing=False),
                           "independent runtime")

    def test_positive_capacity_empty_tail_is_not_discarded(self):
        self.native_reject(lambda r: r["runtime"].update(positive_capacity_empty_exposes_full_tail=False),
                           "independent runtime")

    def test_positive_capacity_empty_alias_is_not_canonicalized(self):
        self.native_reject(lambda r: r["runtime"].update(positive_capacity_empty_shares_backing=False),
                           "independent runtime")

    def test_zero_error_is_not_nil(self):
        self.native_reject(lambda r: r["runtime"].update(zero_value_error_is_nil=True),
                           "independent runtime")

    def test_typed_nil_is_not_nil_interface(self):
        self.native_reject(lambda r: r["runtime"].update(typed_nil_interface_is_nil=True),
                           "independent runtime")

    def test_noncomparable_error_is_not_safely_comparable(self):
        self.native_reject(lambda r: r["runtime"].update(error_comparison_panics=False),
                           "independent runtime")

    def test_runtime_types_are_exact(self):
        # Python's ordinary equality equates True with 1; the checker must not.
        self.native_reject(lambda r: r["runtime"].update(conversion_error_body_calls=False),
                           "independent runtime")

    def test_no_dropped_counts(self):
        self.reject(lambda r: r["draft_outcome_counts"].update(accepted_model=1), "draft outcomes")

    def test_count_booleans_are_not_integers(self):
        self.reject(lambda r: r["native_type_counts"].update(source_parse_error=True), "classification counts")

    def test_build_flag_is_boolean(self):
        self.reject(lambda r: r["build"].update(trimpath=1), "native build/target")

    def test_closed_report_shape(self):
        self.reject(lambda r: r.update(admitted=True), "closed shape")

    def test_json_duplicate_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            strict_json('{"x":1,"x":2}')

    def test_json_nonfinite_rejected(self):
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            strict_json('{"x":NaN}')


if __name__ == "__main__":
    main()
