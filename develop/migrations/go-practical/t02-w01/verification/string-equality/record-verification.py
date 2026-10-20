"""Actual scoped equality outcomes; finite observations grant no source proof."""
from pathlib import Path
import datetime
import hashlib
import json

OUT = Path(__file__).resolve().parent


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


pair = json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 123
assert pair["zero_axiom_acceptances"] == 2 and pair["genuine_core_type_mismatches"] == 121
checks = {}
for mode in ["selected","lint","format"]:
    path = OUT/"epochs/semantic-controls"/mode/"result.json"
    row = json.loads(path.read_bytes())
    assert row["status"] == "passed" and row["exit_code"] == 0 and row["whole_source_inputs_unchanged"]
    checks[mode] = {"result":seal(path),"stdout":seal(path.parent/"stdout.log"),"stderr":seal(path.parent/"stderr.log")}
root = OUT/"epochs/semantic-controls/selected/candidates"
assert len(list(root.glob("byte-*.observation.json"))) == 256
assert len(list(root.glob("string-*.observation.json"))) == 75
assert len(list(root.glob("operation-*.counterexample.json"))) == 15
foundation = next(row for row in pair["cases"] if row["name"] == "equality-foundation")
record = {
    "schema":"mpk.go_practical.string_equality_checkpoint.v1","status":"scoped_checks_passed",
    "entry_commit":"3720e9d0a582d37fcc2ad21634949e35817b8380",
    "observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"source_language_scope":["Go","Rust"],
    "affected_Rust_test_roots":6,"affected_checks":checks,
    "complete_same_byte_inputs":123,"zero_axiom_acceptances":2,"genuine_core_type_mismatches":121,
    "independent_comparison":seal(OUT/"paired/result.json"),
    "generic_component":{"declarations":1215,"terms":38100,"bytes":496415,"new_definitions":13,"new_ordinary_laws":59,
                         "all1143_prior_identities_exact":True,"old_normalization_bytes_exact":True,
                         "input":foundation["input"],"hashes":foundation["Rust"]["hashes"]},
    "proof_structure":"Proper Bool-depth D0..8 All step/reflection/completion proves actual all8-bit ByteEqual and full64-size guarded-prefix String Equal. The actual And major checks length first. True reflection/completion matches real size equality and every observable bit. Bitwise equivalence, actual strong-word equality conversion, result symmetry/transitivity, respects and normalization invariance avoid function extensionality/operand enumeration/axioms/core rules/limit changes.",
    "negative_controls":{"changed_actual_conclusions":59,"erased_real_witnesses":47,"type_correct_wrong_definitions":15,
                         "every_actual_definition_prefix_typechecks":True,"all15_actual_definition_counterexamples_saved":True,
                         "partial_certificate_acceptance_claimed":False},
    "counterexample_scope":"Unrestricted mathematical raw carrier contract, not source-program/profile acceptance. The unguarded-tail case deliberately has dirty raw padding; the other14 examples have canonical tail bytes. All full input bytes and actual wrong-definition results remain, including0-vs256 low8-length and each omitted bit0..7.",
    "finite_observations":{"byte_input_batches":256,"complete_byte_pairs":65536,"byte_AST_steps":13144961,
                           "complete_string_cases":75,"string_AST_steps":23729207,
                           "raw_normalized_equal_and_not_equal_results":True,"full_length_word_bits":64,"all_byte_bits":8,
                           "boundary_lengths":[0,1,2,3,7,8,15,16,31,32,63,64,127,128,129,254,255,256],
                           "complete_original_arrays_retained":True,"definition_callbacks":False,"theorem_bodies_executed":False,
                           "supplies_universal_proof":False,"native_Go_source_or_runtime_receipt":False,
                           "observer_ceilings_unchanged":[50000000,2000000,2000000]},
    "selection_reason":"The new producer and six affected roots cover actual equality/reflection, all original1143 prerequisite identities and byte-exact old producer, exact ownership, every new conclusion/real premise and15 typed semantic changes with actual definition counterexamples. Finite actual-DAG observation independently covers every byte pair and75 whole raw/normalized string cases. Select current strict target Clippy/five-file format. Go/frontend/native capture/core/ref/protocol and unrelated suites are unchanged and not repeated.",
    "complete_source_epochs":2,"complete_source_preimages":745,"failed_verification_execution_in_this_checkpoint":False,
    "all_candidates_and_drivers_saved_before_verification":True,"all_actual_test_checker_handles_terminal":True,
    "frozen_design_inputs_unchanged":255,"Go_frontend_build_inputs_unchanged":186,"checker_limits_or_frozen_caps_changed":False,
    "public_profile_activated":False,"source_program_or_registered_context_proof":False,"native_release_acceptance":False,
    "R01_complete":False,"R06_complete":False,"W01_complete":False,"full_T_gate_passed":False,
    "full_T_gate":"deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "remaining":"Actual original Go source/evaluation/nominal/signed-domain binding; all four bytewise order comparisons/prefix/suffix/substring/concat/source preservation/array/storage/state/program; full ordinary foundation/private descriptor/registry/context/native kernel/general namespace/selected closure/all169/T02-T11.",
}
(OUT/"verification.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:record[k] for k in ["status","affected_Rust_test_roots","complete_same_byte_inputs","zero_axiom_acceptances","genuine_core_type_mismatches","R01_complete","R06_complete","W01_complete"]}))
