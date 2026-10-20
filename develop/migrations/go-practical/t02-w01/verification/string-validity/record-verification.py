"""Actual ordinary whole-string validity outcomes; no source authority."""
from pathlib import Path
import datetime
import hashlib
import json

OUT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


pair = json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 120
assert pair["zero_axiom_acceptances"] == 38 and pair["genuine_core_type_mismatches"] == 82
checks = {}
for mode in ["selected", "lint", "format"]:
    path = OUT/"epochs/native-controls"/mode/"result.json"
    row = json.loads(path.read_bytes())
    assert row["status"] == "passed" and row["exit_code"] == 0 and row["whole_source_inputs_unchanged"]
    checks[mode] = {"result": seal(path), "stdout": seal(path.parent/"stdout.log"), "stderr": seal(path.parent/"stderr.log")}
foundation = next(row for row in pair["cases"] if row["name"] == "validity-foundation")
full = next(row for row in pair["cases"] if row["name"] == "valid-full")
assert foundation["Rust"]["declaration_count"] == 1109 and full["Rust"]["declaration_count"] == 3338
assert len([row for row in pair["cases"] if row["name"].startswith("native-")]) == 18
record = {
    "schema": "mpk.go_practical.whole_string_validity_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "08fe5c78de6d61117fe1c7c1440f9015c5ce8a57",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "source_language_scope": ["Go", "Rust"],
    "affected_checks": checks, "affected_Rust_test_roots": 7,
    "independent_comparison": seal(OUT/"paired/result.json"),
    "complete_same_byte_inputs": 120, "zero_axiom_acceptances": 38, "genuine_core_type_mismatches": 82,
    "rejections": {"changed_actual_generic_conclusions": 56, "erased_actual_witnesses": 21, "type_correct_bad_carriers_without_old_byte_assertions": 5},
    "generic_component": {"declarations": 1109, "terms": 36114, "bytes": 470742, "new_definitions": 0, "new_ordinary_laws": 56,
                          "all1053_original_string_identities_exact": True, "old_string_foundation_bytes_exact": True,
                          "input": foundation["input"], "hashes": foundation["Rust"]["hashes"]},
    "full256_value": {"declarations": 3338, "terms": 42970, "bytes": 748187,
                      "old3166_data_and_all2112_bit_identities_exact": True, "whole_String_Valid_proved": True,
                      "input": full["input"], "hashes": full["Rust"]["hashes"]},
    "initial_empty_value": {"declarations": 3282, "terms": 41104, "bytes": 724026},
    "initial_three_byte_value": {"declarations": 3282, "terms": 41330, "bytes": 726144},
    "actual_original_Go_observations": {"native_inputs": 19, "whole_original_source_expression_and_native_value_retained": True,
                                        "actual_whole_validity_and_all2112_bits": 18, "distinct_257_byte_domain_errors": 1,
                                        "original_evidence": "../string-constants/input-review.json", "source_evaluation_or_nominal_identity_proved": False},
    "additional_literal_boundaries": [1,7,8,15,16,31,32,63,64,127,128,129,254,255],
    "generic_proof_structure": "All-depth Bool-address Index low/high/zero and same/cross comparisons use actual IndexPaddedBound and unbounded Nat cancellation. Full64 high-clear bounds use CountLowBalance/CountNoLessExtension. Full-capacity size uses actual weighted high bits and low digits, never truncated length.",
    "padding_proof_structure": "Proper dependent Bool eliminators follow only the prefix boundary. Entire inside subtrees contradict the real outside assumption through actual index comparisons; entire padded subtrees return exact canonical whole Word8 zero. Full256 uses IndexBound with an actual Size=Capacity equality. No operand enumeration, extensionality, primitive rule or semantic axiom.",
    "bad_carrier_controls": {"cases": ["padding-empty", "padding-partial", "length257", "length-high32", "length-high63"],
                              "all2048_old_slot_observation_proofs_removed": True,
                              "real64_length_bit_assertions_match_modified_length": True,
                              "retained_carrier_and_length_prefix_typechecks": True,
                              "whole_validity_rejects_by_genuine_type_mismatch": True,
                              "partial_certificate_acceptance_claimed": False},
    "selection_reason": "The new source-free producer extends exact existing string/bit components with actual String.Valid witnesses. Select the new generic laws, all original actual Go observations, binary carry/boundary lengths, exact owners and malformed mathematical carriers. Removing old byte observations and repairing real length observations ensures rejection comes from whole validity. The seven affected roots plus current Clippy/four-file format cover producer and consumer changes. Existing APIs retain exact bytes; untouched Go/capture/core/ref/protocol and unrelated workspace suites are not repeated.",
    "all_candidates_and_actual_drivers_saved_before_verification": True, "all_actual_test_and_checker_handles_terminal": True,
    "original_source_epochs": [str(path.relative_to(OUT)) for path in sorted((OUT/"epochs").glob("*/inputs.json"))],
    "frozen_design_inputs_unchanged": 255, "current_Go_frontend_build_inputs_unchanged": 186,
    "checker_or_frozen_limit_changes": False, "public_profile_activated": False, "source_program_or_registered_context_proof": False,
    "native_release_acceptance": False, "R01_complete": False, "R06_complete": False, "W01_complete": False, "full_T_gate_passed": False,
    "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "remaining": "Ordinary original Go source/constant evaluation and nominal/alias identity; signed source-index domain/full string operations/array/padding/empty-dummy/storage/state/program correspondence; complete ordinary foundation/private descriptor/registry/context; production native kernel/general namespace/selected closure/all169/T02-T11.",
}
(OUT/"verification.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({key:record[key] for key in ["status", "affected_Rust_test_roots", "complete_same_byte_inputs", "zero_axiom_acceptances", "genuine_core_type_mismatches", "R01_complete", "R06_complete", "W01_complete"]}))
