"""Summarize actual completed checks; no model/source/program authority."""
from pathlib import Path
import datetime
import hashlib
import json

OUT=Path(__file__).resolve().parent
BASE="8814646e76c9dd75d2666e4610e8c091ea44fc8b"

def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}

pair=json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"]=="passed" and len(pair["cases"])==339
assert pair["zero_axiom_acceptances"]==2 and pair["genuine_core_type_mismatches"]==337
checks={}
for epoch,mode in [("semantic-controls","selected"),("lint-corrected","foundation"),("lint-corrected","lint"),("lint-corrected","format")]:
    path=OUT/"epochs"/epoch/mode/"result.json";row=json.loads(path.read_bytes())
    assert row["status"]=="passed" and row["exit_code"]==0 and row["whole_source_inputs_unchanged"]
    checks[epoch+"/"+mode]={"result":seal(path),"stdout":seal(path.parent/"stdout.log"),"stderr":seal(path.parent/"stderr.log")}
old=OUT/"epochs/semantic-controls/selected/candidates/order-foundation.mpcert"
new=OUT/"epochs/lint-corrected/foundation/candidates/order-foundation.mpcert"
assert old.read_bytes()==new.read_bytes()
fixtures=old.parent
byte_steps=sum(json.loads(path.read_bytes())["actual_AST_steps"]for path in fixtures.glob("byte-*.observation.json"))
string_steps=sum(json.loads(path.read_bytes())["actual_AST_steps"]for path in fixtures.glob("string-*.observation.json"))
assert byte_steps==113660694 and string_steps==147304477
assert len(list(fixtures.glob("byte-*.observation.json")))==256
assert len(list(fixtures.glob("string-*.observation.json")))==150
assert len(list(fixtures.glob("operation-*.counterexample.json")))==25
prefixes=json.loads((fixtures/"definition-controls.prefix.json").read_bytes())
assert len(prefixes)==25 and all(row["actual_definition_prefix_typechecks"] and not row["full_certificate_acceptance_claimed"] for row in prefixes)
archives=[json.loads(path.read_bytes())for path in sorted((OUT/"epochs").glob("*/inputs.json"))]
executions=[]
for path in sorted((OUT/"epochs").glob("*/*/result.json")):
    row=json.loads(path.read_bytes());assert row["status"]in {"passed","failed"} and row["whole_source_inputs_unchanged"]
    executions.append({"result":seal(path),"stdout":seal(path.parent/"stdout.log"),"stderr":seal(path.parent/"stderr.log"),"status":row["status"],"exit_code":row["exit_code"]})
record={
 "schema":"mpk.go_practical.string_order_checkpoint.v1","status":"scoped_checks_passed","entry_commit":BASE,
 "observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"source_language_scope":["Go","Rust"],
 "generic_component":{"declarations":1706,"terms":52523,"bytes":679227,"new_definitions":26,"new_ordinary_laws":190,"all1215_prior_string_identities_exact":True,"old_string_equality_and_full_word_order_bytes_exact":True,"input":seal(new)},
 "proof_structure":"Actual unsigned Word8 comparison/numeric bridges; independent D0..8 Nat-vector equality and lex order; Less true eliminates to a real first-difference address, equality at every lower index and a strict rank difference; actual witnesses conversely prove true. Rank is0 for end and unsigned-byte-value+1 inside the full64-size guard. Four order operations, complete pointwise respects/normalization, Nat asymmetry, actual vector total-rank equality and reflexivity use ordinary zero-axiom proofs without function extensionality, operand tables, core rules or limit changes.",
 "affected_Rust_test_roots":6,"affected_checks":checks,"lint_correction":"Remove unused receiver from lifted_prefix. Current producer regenerates the identical full certificate; all retained semantic/proof controls consume those exact same bytes. No proof recipe/observer/consumer behavior changes.",
 "complete_same_byte_inputs":339,"zero_axiom_acceptances":2,"genuine_core_type_mismatches":337,"independent_comparison":seal(OUT/"paired/result.json"),"original_same_byte_failure_rejections":2,"original_failure_independent_comparison":seal(OUT/"failed-paired/result.json"),
 "negative_controls":{"changed_actual_conclusions":190,"replaced_typed_witness_domains":122,"type_correct_wrong_definitions":25,"all25_full_actual_definition_counterexamples":True,"all25_canonical_tail_inputs":True,"partial_certificate_acceptance_claimed":False,"empty_prefix_witness_note":"The depth0 prefix is vacuous but remains part of the exact typed contract; domain mutation tests signature/proof binding, not logical independence."},
 "finite_observations":{"complete_unsigned_byte_pairs":65536,"four_comparisons":True,"byte_AST_steps":byte_steps,"whole_string_cases":75,"raw_and_normalized_observations":150,"both_operand_orders":True,"string_AST_steps":string_steps,"full_length_word_bits":64,"all_byte_bits":8,"complete_input_arrays_and_available_partial_results_retained":True,"definition_callbacks":False,"theorem_bodies_executed":False,"supplies_universal_proof":False,"native_Go_source_or_runtime_receipt":False,"observer_ceilings_unchanged":[50000000,2000000,2000000]},
 "complete_source_epochs":len(archives),"complete_source_preimages":sum(len(row["sources"])for row in archives),"actual_executions":executions,
 "first_record_review":{"result":seal(OUT/"input-review.initial.json"),"original_available_stdout":seal(OUT/"input-review.initial.stdout.log"),"source_or_proof_failure":False,"finding":"Only the prospective link to the review output itself was missing before initial creation; original result/output remain, with no code or checker change."},
 "original_failures":"Original borrow and module-import compile errors, two rejected nested-Boolean reflexivity candidates, and strict Clippy argument-count failure remain complete. Original 16-case actual-DAG diagnostic shows the failed Boolean endpoints agree; explicit Eq.congr/trans proofs repair conversion without changing the core or fuel. Original partial format coverage remains and final format checks all six changed files.",
 "selection_reason":"Select actual new producer, old1215 identities/golden producer bytes, exact owning import, every new conclusion, exact typed premises,25 typed semantic changes and actual-DAG65536 bytepair/75 full string observations. After the internal lint-only receiver change, recheck current complete positive certificate byte identity and current strict target Clippy/all changed-file format. Unchanged Go/frontend/native capture/core/ref/protocol and unrelated suites are retained without repeat. All verification is local.",
 "all_candidates_and_actual_drivers_saved_before_verification":True,"all_actual_test_checker_handles_terminal":True,
 "frozen_design_inputs_unchanged":255,"Go_frontend_build_inputs_unchanged":186,"checker_limits_or_frozen_caps_changed":False,
 "public_profile_activated":False,"source_program_or_registered_context_proof":False,"native_release_acceptance":False,
 "R01_complete":False,"R06_complete":False,"W01_complete":False,"full_T_gate_passed":False,"full_T_gate":"deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
 "remaining":"Universal bounded-string bridge to existing full-size String.Equality and actual original Go source/evaluation/nominal/signed-domain binding; prefix/suffix/substring/concat/source preservation/array/storage/state/program; full ordinary foundation/private descriptor/registry/context/native kernel/general namespace/selected closure/all169/T02-T11. Raw lengths beyond256 are not admitted-source evidence; TotalRanksEqual concerns rank vectors only."
}
(OUT/"verification.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({k:record[k]for k in ["status","affected_Rust_test_roots","complete_same_byte_inputs","zero_axiom_acceptances","genuine_core_type_mismatches","complete_source_epochs","complete_source_preimages","R01_complete","R06_complete","W01_complete"]}))
