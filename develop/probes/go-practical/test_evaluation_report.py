import copy
import tempfile
from pathlib import Path
from unittest import TestCase, main

from check_evaluation_report import check, json_bytes, sha, strict_json
from evaluation_cases import RECORD, inputs


class EvaluationTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = strict_json((RECORD/"observations.json").read_bytes())
        cls.output = strict_json((RECORD/"native-output.json").read_bytes())

    def reject(self, edit, reason):
        r = copy.deepcopy(self.report)
        edit(r)
        with self.assertRaisesRegex(ValueError, reason):
            check(r)

    def wrong_fact(self, identity, edit, n=0, flag=False):
        output = copy.deepcopy(self.output)
        index = inputs().index({"id": identity, "n": n, "flag": flag})
        edit(output[index])
        r = copy.deepcopy(self.report)
        with tempfile.TemporaryDirectory(prefix="mpk-eval-negative-", dir="/private/tmp") as tmp:
            temp = Path(tmp)
            (temp/"native-input.json").write_bytes((RECORD/"native-input.json").read_bytes())
            raw = json_bytes(output)
            (temp/"native-output.json").write_bytes(raw)
            r["artifacts"]["native-output.json"] = {"bytes": len(raw), "sha256": sha(raw)}
            with self.assertRaisesRegex(ValueError, "independent evaluation facts"):
                check(r, temp)

    def test_actual(self): self.assertEqual(check(self.report), len(inputs()))
    def test_no_support_promotion(self): self.reject(lambda r:r.update(scope="profile supported"), "not proof/admission")
    def test_no_completion(self): self.reject(lambda r:r.update(status="W04 complete"), "not proof/admission")
    def test_no_hidden_work(self): self.reject(lambda r:r["pending"].pop(), "remaining evaluation")
    def test_source_pins(self): self.reject(lambda r:r["inputs"].pop(), "source identity")
    def test_toolchain(self): self.reject(lambda r:r["toolchain_inventory"].update(sha256="0"*64), "pinned Go")
    def test_raw_hash(self): self.reject(lambda r:r["artifacts"]["native-output.json"].update(sha256="0"*64), "artifact identity")
    def test_no_linux_receipt(self): self.reject(lambda r:r["build"].update(host="linux/amd64"), "build/scope")
    def test_no_term_proof(self): self.reject(lambda r:r.update(termination_proved=True), "closed shape")
    def test_full_counts(self): self.reject(lambda r:r.update(cases=1), "counts")
    def test_parallel_target(self): self.wrong_fact("parallel-index",lambda r:r.update(s=[3,2,7]))
    def test_duplicate_target(self): self.wrong_fact("duplicate-target",lambda r:r.update(s=[1]))
    def test_slice_target_captured(self): self.wrong_fact("slice-rebind-target",lambda r:r.update(s=[9,4],old=[1,2]))
    def test_array_location_retained(self): self.wrong_fact("array-rebind-target",lambda r:r.update(a=[3,4]))
    def test_array_view_location_survives_assignment(self): self.wrong_fact("array-view-assignment",lambda r:r.update(view=[1,2]))
    def test_array_assignment_copies_values(self): self.wrong_fact("array-view-assignment",lambda r:r.update(a=[8,4],view=[8,4]))
    def test_struct_location_retained(self): self.wrong_fact("struct-rebind-target",lambda r:r.update(v=3))
    def test_nested_slice_target(self): self.wrong_fact("struct-slice-rebind-target",lambda r:r.update(s=[9,4],old=[1,2]))
    def test_merge_current_cell(self): self.wrong_fact("cell-field-merge",lambda r:r.update(a=1))
    def test_joint_once(self): self.wrong_fact("joint-call-once",lambda r:r.update(trace=[1,1]))
    def test_joint_projection(self): self.wrong_fact("joint-call-once",lambda r:r.update(a=9,b=7))
    def test_discard_does_not_reexecute(self): self.wrong_fact("discarded-result",lambda r:r.update(trace=[1,1]))
    def test_forward_does_not_reexecute(self): self.wrong_fact("forwarded-return",lambda r:r.update(trace=[1,1]))
    def test_compound_target_once(self): self.wrong_fact("compound-once",lambda r:r.update(trace=[1,1,2]))
    def test_store_panic_after_first_write(self): self.wrong_fact("assignment-calls-before-store-panic",lambda r:r.update(s=[1,2]))
    def test_rhs_panic_before_any_write(self): self.wrong_fact("rhs-panic-before-writes",lambda r:r.update(s=[7,2]))
    def test_short_circuit(self): self.wrong_fact("short-circuit",lambda r:r.update(trace=[1,2]),flag=True)
    def test_case_order(self): self.wrong_fact("switch-case-order",lambda r:r.update(trace=[1,2,3,4]))
    def test_default_position_not_priority(self): self.wrong_fact("switch-default-first",lambda r:r.update(v=11,trace=[1]))
    def test_break_consumed_by_switch(self): self.wrong_fact("switch-break-target",lambda r:r.update(iterations=1),n=5)
    def test_continue_runs_post(self): self.wrong_fact("switch-continue-target",lambda r:r.update(posts=2),n=5)
    def test_nested_break_target(self): self.wrong_fact("nested-loop-break",lambda r:r.update(outer=1),n=5)
    def test_array_range_snapshot(self): self.wrong_fact("range-array-copy",lambda r:r.update(seen=[1,20,3]))
    def test_array_descriptor_not_deep_copied(self): self.wrong_fact("range-array-descriptor-copy",lambda r:r.update(seen=[1,1]))
    def test_array_descriptor_container_copied(self): self.wrong_fact("range-array-descriptor-copy",lambda r:r.update(seen=[7,9]))
    def test_slice_range_live_cells(self): self.wrong_fact("range-slice-capture",lambda r:r.update(seen=[1,2,3]))
    def test_slice_range_length_once(self): self.wrong_fact("range-slice-capture",lambda r:r.update(seen=[1]))
    def test_nil_range(self): self.wrong_fact("range-slice-nil",lambda r:r.update(iterations=1))
    def test_range_assignment_two_phase(self): self.wrong_fact("range-assignment-target",lambda r:r.update(s=[3,5,7]))
    def test_array_function_call_not_elided(self): self.wrong_fact("range-array-call-once",lambda r:r.update(trace=[]))
    def test_blank_second_is_not_extra_call(self): self.wrong_fact("range-blank-second-variable",lambda r:r.update(trace=[1,1]))
    def test_array_size_expression_not_elided(self): self.wrong_fact("array-size-call",lambda r:r.update(trace=[]))
    def test_return_skips_post(self): self.wrong_fact("early-return-skips-post",lambda r:r.update(posts=1))
    def test_named_result_zero(self): self.wrong_fact("named-return-zero",lambda r:r.update(value=1))
    def test_string_copy_full_bytes(self): self.wrong_fact("byte-string-copy",lambda r:r.update(bytes=[0,0,120]))
    def test_string_append_full_bytes(self): self.wrong_fact("byte-string-append",lambda r:r.update(bytes=[7,0,0,120]))
    def test_string_append_capacity(self): self.wrong_fact("byte-string-append",lambda r:r.update(capacity=8))
    def test_min_division_defined(self): self.wrong_fact("integer-defined-edges",lambda r:r.update(div=0))
    def test_huge_shift_not_modulo(self): self.wrong_fact("integer-defined-edges",lambda r:r.update(left=1))
    def test_signed_shift(self): self.wrong_fact("integer-defined-edges",lambda r:r.update(signed_right=0))
    def test_aggregate_error_comparison_not_safe(self): self.wrong_fact("aggregate-error-equality",lambda r:r.update(panicked=False))
    def test_exact_json_types(self): self.wrong_fact("named-return-zero",lambda r:r.update(ok=0))


if __name__ == "__main__":
    main()
