"""Record actual scoped outcomes, retaining the failed finite-observer command."""
from pathlib import Path
import datetime
import hashlib
import json

OUT = Path(__file__).resolve().parent


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


pair = json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 65
assert pair["zero_axiom_acceptances"] == 2 and pair["genuine_core_type_mismatches"] == 63
assert pair["production_and_core_consumer_sources_unchanged"] and len(pair["observer_only_source_delta"]) == 1
selected = json.loads((OUT/"epochs/typed-controls/selected/result.json").read_bytes())
assert selected["status"] == "failed" and selected["exit_code"] == 101 and selected["whole_source_inputs_unchanged"]
assert len(pair["Rust_complete_core_test_roots_passed"]) == 4
checks = {"retained_four_core_roots": seal(OUT/"epochs/typed-controls/selected/result.json")}
for mode in ["observations", "lint", "format"]:
    path = OUT/"epochs/trimmed-observer"/mode/"result.json"
    row = json.loads(path.read_bytes())
    assert row["status"] == "passed" and row["exit_code"] == 0 and row["whole_source_inputs_unchanged"]
    checks[mode] = {"result": seal(path), "stdout": seal(path.parent/"stdout.log"), "stderr": seal(path.parent/"stderr.log")}
old = OUT/"epochs/typed-controls/selected/candidates"
current = OUT/"epochs/trimmed-observer/observations/candidates"
old_inputs, old_outputs = list(old.glob("raw-*.input.json")), list(old.glob("raw-*.observation.json"))
assert len(old_inputs) == 31 and len(old_outputs) == 30
assert len(list(current.glob("raw-*.input.json"))) == len(list(current.glob("raw-*.observation.json"))) == 54
for path in old_inputs:
    assert path.read_bytes() == (current/path.name).read_bytes()
for path in old_outputs:
    a, b = json.loads(path.read_bytes()), json.loads((current/path.name).read_bytes())
    a.pop("actual_AST_steps")
    b.pop("actual_AST_steps")
    assert a == b
assert (old/"raw-63-0.input.json").exists() and not (old/"raw-63-0.observation.json").exists()
foundation = next(row for row in pair["cases"] if row["name"] == "normalization-foundation")
record = {
    "schema": "mpk.go_practical.string_normalization_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "a874633cef63dcfa10965d0b702789937969f98c",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "source_language_scope": ["Go", "Rust"],
    "affected_checks": checks, "affected_Rust_test_roots": 5,
    "root_selection": "Four unchanged certificate roots pass in the original five-root command. That actual command fails only in its finite observer. Only the observer source changes; its full54-input root passes separately. Final Clippy/five-file format cover the actual last epoch; passing production/core roots are retained without repetition.",
    "complete_same_byte_inputs": 65, "zero_axiom_acceptances": 2, "genuine_core_type_mismatches": 63,
    "independent_comparison": seal(OUT/"paired/result.json"),
    "generic_component": {"declarations": 1143, "terms": 36820, "bytes": 479936, "new_definitions": 6, "new_ordinary_laws": 28,
                          "all1109_prior_string_identities_exact": True, "old_generic_producer_bytes_exact": True,
                          "input": foundation["input"], "hashes": foundation["Rust"]["hashes"]},
    "actual_laws": "Exact entire Word64 length and all64 projections; canonical whole Word8 padding; Valid consumes real full upper bound; logical size/observable-slot equivalence with actual symmetry/transitivity/guard transport; full64 guarded reads/sequence observations; normalization content preservation and pointwise stability/idempotence/empty/full laws. No extensionality or source access authority.",
    "negative_controls": {"actual_generic_conclusions": 28, "actual_erased_witnesses": 28, "type_correct_wrong_operations": 7,
                          "operation_definition_prefixes_typecheck": True, "partial_certificate_acceptance_claimed": False},
    "finite_observer": {"complete_raw_inputs": 54, "each_full_length_bits": 64, "each_full_data_bits": 2048,
                        "actual_original_normalized_and_twice_values_observed": True, "actual_AST_steps": 28685384,
                        "lengths": [0,1,2,3,7,8,15,16,31,32,63,64,127,128,129,254,255,256], "patterns": 3,
                        "actual_certificate_definitions_executed": True, "arithmetic_or_string_definition_callbacks": False,
                        "theorem_bodies_executed": False, "supplies_universal_proof": False,
                        "unchanged_step_value_environment_ceilings": [50000000,2000000,2000000],
                        "retained_old_complete_inputs": 31, "retained_old_available_results": 30,
                        "original_uncompleted_case": "raw-63-0", "original_detailed_counters_available": False,
                        "missing_original_output_reconstructed": False,
                        "repair": "Only de Bruijn free-variable environment trimming, variable-thunk sharing and already-forced argument sharing. All original input bytes and30 available outputs match except step counts; no ceiling or high-level primitive added."},
    "original_failed_epochs": [
        {"epoch": "initial", "kind": "CORE_NOT_A_FUNCTION", "meaning": "Insufficient explicit content-function view"},
        {"epoch": "contents-function", "kind": "CORE_FUEL_EXHAUSTED", "meaning": "Implicit MakeSize conversion needs explicit length congruence"},
        {"epoch": "typed-controls", "kind": "finite_observer_resource_ceiling", "meaning": "Four certificate roots pass; observer retains31 inputs/30 outputs"},
    ],
    "complete_source_epochs": 5, "all_candidates_and_drivers_saved_before_verification": True,
    "all_actual_test_checker_handles_terminal": True, "frozen_design_inputs_unchanged": 255,
    "Go_frontend_inputs_unchanged": 186, "checker_limits_or_frozen_caps_changed": False,
    "public_profile_activated": False, "source_program_or_registered_context_proof": False,
    "native_release_acceptance": False, "R01_complete": False, "R06_complete": False, "W01_complete": False,
    "full_T_gate_passed": False, "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "remaining": "Complete string compare/substring/concat compatibility and source-to-carrier preservation; ordinary original Go source/constant evaluation, nominal/alias identity/signed domains/array/storage/state/program; complete ordinary foundation/private descriptor/registry/context/native kernel/general namespace/selected closure/all169/T02-T11.",
}
(OUT/"verification.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:record[k] for k in ["status", "affected_Rust_test_roots", "complete_same_byte_inputs", "zero_axiom_acceptances", "genuine_core_type_mismatches", "R01_complete", "R06_complete", "W01_complete"]}))
