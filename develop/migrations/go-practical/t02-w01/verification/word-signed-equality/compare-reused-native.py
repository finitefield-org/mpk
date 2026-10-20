"""Reuse exact original signed circuit/native rows; check new Int counterparts."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def read(path):
    return json.loads(Path(path).read_text())


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
previous = root.parent / "word-foundation"
native_path = previous / "native/result.json"
native = read(native_path)
comparison_path = previous / "native-comparison/result.json"
comparison = read(comparison_path)
assert native["status"] == comparison["status"] == "passed"
assert native["native_rows"] == comparison["rows"] == 1640 and native["two_native_outputs_byte_identical"]
for key in ["original_source", "owned_source", "original_queries", "owned_queries", "complete_original_prepared_image"]:
    assert seal(native[key]["path"]) == native[key]
for command in native["commands"]:
    assert command["exit_code"] == 0
    for key in ["stdout", "stderr"]:
        assert seal(command[key]["path"]) == command[key]
for image in native["complete_original_image_checks"]:
    assert image["status"] == "passed" and image["files"] == 5577
for key in ["actual_certificate", "model_certificate", "actual_decoded_certificate_model", "independent_integer_oracle", "model_metrics"]:
    assert seal(comparison[key]["path"]) == comparison[key]
native_rows = read(previous / "native/run1.stdout.log")
assert (previous / "native/run1.stdout.log").read_bytes() == (previous / "native/run2.stdout.log").read_bytes()
model_rows = read(comparison["actual_decoded_certificate_model"]["path"])
oracle_rows = read(comparison["independent_integer_oracle"]["path"])
assert model_rows == oracle_rows
queries = read(native["original_queries"]["path"])
assert len(queries) == len(native_rows) == len(model_rows) == 1640
runtime_outcomes = 0
for query, native_row, model_row in zip(queries, native_rows, model_rows, strict=True):
    assert query["id"] == native_row["id"] == model_row["id"]
    assert native_row["kind"] == model_row["kind"]
    if native_row["kind"] == "runtime_safety_failure":
        assert native_row["detail"] == {"negative_shift": "runtime error: negative shift amount", "division_by_zero": "runtime error: integer divide by zero"}[model_row["reason"]]
        runtime_outcomes += 1
    else:
        assert native_row == model_row
assert runtime_outcomes == 40
fresh_path = root / "native/result.json"
fresh = read(fresh_path)
assert fresh["status"] == "passed" and fresh["native_rows"] == 96 and fresh["two_native_outputs_byte_identical"]
for key in ["original_source", "owned_source", "original_queries", "owned_queries", "complete_original_prepared_image", "compiler", "observer_binary"]:
    assert seal(fresh[key]["path"]) == fresh[key]
assert fresh["original_queries"] == native["original_queries"]
for command in fresh["commands"]:
    assert command["exit_code"] == 0
    for key in ["stdout", "stderr"]:
        assert seal(command[key]["path"]) == command[key]
for image in fresh["complete_original_image_checks"]:
    assert image["status"] == "passed" and image["files"] == 5577 and image["original_bytes"] == 163457770
fresh_rows = read(root / "native/run1.stdout.log")
assert (root / "native/run1.stdout.log").read_bytes() == (root / "native/run2.stdout.log").read_bytes()
reader = root.parent / "word-signed-division/prerequisite-analysis/measure.py"
ns = {"__file__": str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]", 1)[0], str(reader), "exec"), ns)
Tables = ns["Tables"]
current_path = root / "final/tests/certificates/word-signed-equality.mpcert"
current = Tables(seal(current_path))
fingerprints = current.fingerprints()
old = Tables(comparison["actual_certificate"])
old_order_record = read(root.parent / "word-signed-order/verification.json")["fresh_component"]["certificate"]
old_order = Tables(old_order_record)
for certificate in [old, old_order]:
    for name, original in certificate.fingerprints().items():
        assert fingerprints[name] == original, name
assert len(old.index) == 1892 and len(old_order.index) == 3457
rows = []
branches = {"00": 0, "01": 0, "10": 0, "11": 0}
widths = set()
fresh_by_id = {r["id"]: r for r in fresh_rows}
assert len(fresh_by_id) == 96
for query, observation in zip(queries, native_rows, strict=True):
    assert query["id"] == observation["id"]
    if query["operation"] not in {"Equal", "NotEqual"}:
        continue
    width = query["width"]
    widths.add(width)
    modulus = 1 << width
    a, b = (int(query[name], 16) for name in ["x_hex", "y_hex"])
    assert all(len(query[name]) == width // 4 for name in ["x_hex", "y_hex"])
    assert 0 <= a < modulus and 0 <= b < modulus
    sa, sb = a >= modulus // 2, b >= modulus // 2
    ia, ib = a - modulus * int(sa), b - modulus * int(sb)
    left, right = a + modulus * int(sb), b + modulus * int(sa)
    actual_bits = [bool((a >> bit) & 1) == bool((b >> bit) & 1) for bit in range(width)]
    def folded(flags):
        if not flags:
            return True
        if len(flags) == 1:
            return flags[0]
        mid = len(flags) // 2
        return folded(flags[:mid]) and folded(flags[mid:])
    source_equal = folded(actual_bits)
    assert source_equal == (a == b) == (ia == ib) == (left == right)
    right_steps = []
    cursor = actual_bits
    while len(cursor) > 1:
        mid = len(cursor) // 2
        right_steps.append({"whole_equal": folded(cursor), "right_equal": folded(cursor[mid:]), "whole_width": len(cursor)})
        assert not folded(cursor) or folded(cursor[mid:])
        cursor = cursor[mid:]
    assert len(right_steps) == width.bit_length() - 1
    assert not source_equal or (cursor[0] and sa == sb)
    expected = source_equal if query["operation"] == "Equal" else not source_equal
    assert observation == {"id": query["id"], "kind": "boolean", "value": expected}
    fresh_row = fresh_by_id.pop(query["id"])
    assert fresh_row == {"id": query["id"], "kind": "boolean", "operation": query["operation"], "width": width,
                         "value": expected, "unsigned_value": expected, "signed_x": str(ia), "signed_y": str(ib)}
    branches[str(int(sa)) + str(int(sb))] += 1
    rows.append({"original_query": query, "original_actual_Bool_and_native_result": observation,
                 "fresh_actual_signed_unsigned_Go_observation": fresh_row,
                 "input_sign_bits_from_full_payloads": [sa, sb], "signed_integers": [str(ia), str(ib)],
                 "balanced_cross_sums": [str(left), str(right)], "actual_balanced_bit_equalities": actual_bits,
                 "actual_right_tree_projection_steps": right_steps, "source_equal": source_equal,
                 "signed_equal": ia == ib, "signed_not_equal": ia != ib, "sign_coherence_when_equal": not source_equal or sa == sb})
assert len(rows) == 96 and not fresh_by_id and widths == {8, 16, 32, 64} and all(branches.values())
out = root / "finite-counterparts"
out.mkdir(exist_ok=False)
(out / "counterparts.json").write_text(json.dumps(rows, indent=2) + "\n")
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "original_native_execution": seal(native_path), "original_actual_Bool_comparison": seal(comparison_path),
          "fresh_signed_unsigned_native_execution": seal(fresh_path), "original_queries": native["original_queries"],
          "current_certificate": seal(current_path), "dependency_reader": seal(reader),
          "all_original_Bool_named_identities_unchanged": 1892, "all_original_signed_order_named_identities_unchanged": 3457,
          "counterparts": seal(out / "counterparts.json"), "original_equality_queries": len(rows),
          "sign_branch_counts": branches, "widths": sorted(widths), "whole_original_result_rows_cross_checked": 1640,
          "original_native_runtime_outcomes_retained": runtime_outcomes, "unchanged_original_Bool_model_rerun": False,
          "fresh_actual_signed_unsigned_native_rows": 96, "Nat_Int_or_new_theorem_bodies_executed": False,
          "scope": "Preserve every original1640 result and40 runtime outcomes, retain all96 full Equal/NotEqual rows and exact old decoded Bool results. Fresh actual typed signed/unsigned Go execution reads the entire original query document and is compared by exact original IDs, widths, operations and signed decimal payload values. The independent Python counterpart checks all actual bit comparisons/right-tree projections, sign coherence, unbounded signed equality and balanced cross sums. No query/source payload is rewritten. Finite evidence supplies no universal proof authority; both unchanged cores inspect the full new component. Full56-family/source/program/context/profile/immutable replay and native Linux release remain open."}
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "original_equality_queries", "sign_branch_counts", "all_original_Bool_named_identities_unchanged", "all_original_signed_order_named_identities_unchanged"]}))
