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
reader = root.parent / "word-signed-division/prerequisite-analysis/measure.py"
ns = {"__file__": str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]", 1)[0], str(reader), "exec"), ns)
Tables = ns["Tables"]
current_path = root / "final/tests/certificates/word-signed-order.mpcert"
current = Tables(seal(current_path))
fingerprints = current.fingerprints()
old = Tables(comparison["actual_certificate"])
old_order_record = read(root.parent / "word-order/verification.json")["fresh_component"]["certificate"]
old_order = Tables(old_order_record)
for certificate in [old, old_order]:
    for name, original in certificate.fingerprints().items():
        assert fingerprints[name] == original, name
assert len(old.index) == 1892 and len(old_order.index) == 3437
rows = []
branches = {"00": 0, "01": 0, "10": 0, "11": 0}
widths = set()
for query, observation in zip(queries, native_rows, strict=True):
    assert query["id"] == observation["id"]
    if query["operation"] not in {"SignedLess", "SignedLessEqual"}:
        continue
    width = query["width"]
    widths.add(width)
    modulus = 1 << width
    a, b = (int(query[name], 16) for name in ["x_hex", "y_hex"])
    assert 0 <= a < modulus and 0 <= b < modulus
    sa, sb = a >= modulus // 2, b >= modulus // 2
    ia, ib = a - modulus * int(sa), b - modulus * int(sb)
    left, right = a + modulus * int(sb), b + modulus * int(sa)
    ordinary = a < b
    circuit_choice = sa if sa != sb else ordinary
    assert circuit_choice == (left < right) == (ia < ib)
    expected = ia < ib if query["operation"] == "SignedLess" else ia <= ib
    assert observation == {"id": query["id"], "kind": "boolean", "value": expected}
    gap_a, gap_b = modulus - a - 1, modulus - b - 1
    assert modulus == a + gap_a + 1 == b + gap_b + 1
    assert b + modulus == a + (b + gap_a) + 1
    assert a < b + modulus and not b + modulus < a
    branches[str(int(sa)) + str(int(sb))] += 1
    rows.append({"original_query": query, "original_actual_Bool_and_native_result": observation,
                 "input_sign_bits_from_full_payloads": [sa, sb], "signed_integers": [str(ia), str(ib)],
                 "balanced_cross_sums": [str(left), str(right)], "strict_modulus_gaps": [str(gap_a), str(gap_b)],
                 "source_sign_choice": circuit_choice, "signed_less": ia < ib, "signed_less_equal": ia <= ib})
assert len(rows) == 96 and widths == {8, 16, 32, 64} and all(branches.values())
out = root / "finite-counterparts"
out.mkdir(exist_ok=False)
(out / "counterparts.json").write_text(json.dumps(rows, indent=2) + "\n")
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "original_native_execution": seal(native_path), "original_actual_Bool_comparison": seal(comparison_path),
          "original_queries": native["original_queries"], "current_certificate": seal(current_path),
          "dependency_reader": seal(reader), "all_original_Bool_named_identities_unchanged": 1892,
          "all_original_order_named_identities_unchanged": 3437, "counterparts": seal(out / "counterparts.json"),
          "original_signed_queries": len(rows), "sign_branch_counts": branches, "widths": sorted(widths),
          "whole_original_result_rows_cross_checked": 1640, "original_native_runtime_outcomes_retained": runtime_outcomes,
          "fresh_native_build_or_Bool_model_rerun": False, "Nat_Int_or_new_theorem_bodies_executed": False,
          "scope": "Retain every complete original signed comparison row, actual decoded-certificate Bool result and two identical native Go outputs. Their complete circuit identities and original source/query/output/prepared-image evidence are exact. New Python integers check actual payload signs, branch choices, balanced cross sums and unbounded-law finite gap counterparts. Finite observations supply no universal proof authority; both unchanged cores separately inspect the complete new proof component. This is no application/source/context/profile/immutable replay or native Linux release claim."}
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "original_signed_queries", "sign_branch_counts", "all_original_Bool_named_identities_unchanged", "all_original_order_named_identities_unchanged"]}))
