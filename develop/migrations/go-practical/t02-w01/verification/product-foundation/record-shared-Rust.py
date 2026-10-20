"""Record completed full shared Rust matrix without promoting ongoing Go checks."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
execution=root/"alias-consumers/initial/tests/result.json";ex=json.loads(execution.read_text())
assert ex["status"]=="passed" and not ex["changed_inputs_during_execution"]
for key in ["stdout","stderr"]:assert seal(ex[key]["path"])==ex[key]
assert "1 passed; 0 failed" in Path(ex["stdout"]["path"]).read_text()
assert "2080 pack +2080 reconstructed +2080 updated +87360 unchanged" in Path(ex["stdout"]["path"]).read_text()
for source in ex["sources"]:
    assert seal(source["path"])==source
    stored=seal(Path(ex["source_snapshot"])/source["path"])
    assert (stored["sha256"],stored["bytes"])==(source["sha256"],source["bytes"])
types_path=root/"complete-type-sharing-analysis/result.json";types=json.loads(types_path.read_text())
assert types["status"]=="all65_complete_original_goal_type_sets_match" and types["identical_named_goal_types"]==2210
resource_path=root/"expanded-resource/verification.json";resource=json.loads(resource_path.read_text())
assert resource["status"]=="all65_complete_original_constructions_retained_fixed_budget_preserved"
rows=[]
for arity,pair in enumerate(types["pairs"]):
    assert pair["arity"]==arity
    candidate=pair["complete_shared_checked_certificate"];report=pair["shared_actual_Rust_report"]
    assert seal(candidate["path"])==candidate and seal(report["path"])==report
    r=json.loads(Path(report["path"]).read_text())
    assert r["verdict"]=="accepted" and r["axiom_count"]==0 and not r["axiom_report"]["entries"]
    assert r["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    assert r["declaration_count"]==80+arity<=4096 and pair["terms"][1]<=131072 and candidate["bytes"]<=2097152
    rows.append({"arity":arity,"complete_certificate":candidate,"actual_complete_Rust_report":report,"full_Rust_observation":r,"declarations":80+arity,"terms":pair["terms"][1],"bytes":candidate["bytes"],"every_original_named_goal_type_preserved":pair["identical_goal_types"]})
assert len(rows)==65 and rows[-1]["terms"]==32198 and rows[-1]["bytes"]==305189
checks=[]
for relative in ["final/clippy/result.json","final/format/result.json","final/diagnostic-list/result.json","explicit-helper-list/result.json","alias-consumers/initial/clippy/result.json","alias-consumers/initial/format/result.json","expanded-resource/collected/clippy/result.json","expanded-resource/collected/format/result.json","expanded-resource/collected/listing/result.json"]:
    path=root/relative;d=json.loads(path.read_text());assert d["status"]=="passed" and not d["changed_inputs_during_execution"]
    for key in ["stdout","stderr"]:assert seal(d[key]["path"])==d[key]
    checks.append(seal(path))
frozen_path=root/"frozen-design-preservation.json";frozen=json.loads(frozen_path.read_text())
assert len(frozen["inputs"])==255
for original in frozen["inputs"]:assert seal(original["path"])==original
record={"schema":"mpk.go_practical.shared_product_Rust_checkpoint.v1","status":"full_shared_Rust_and_goal_preservation_passed_Go_still_running","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"entry_commit":"7154ec0b2ac2ab56ac21e87e018d2f2c02545366","runner":seal(__file__),"actual_terminal_Rust_execution":seal(execution),"Rust_seconds":ex["wall_seconds"],"shared_arities":65,"leaf_obligations":{"pack":2080,"reconstruct":2080,"updated":2080,"unchanged":87360,"total":93600},"independent_goal_type_preservation":seal(types_path),"identical_goal_types":2210,"original_complete_resource_inputs":seal(resource_path),"all_original78_basis_identities_preserved":True,"selected_lint_format_and_ignored_listings":checks,"frozen255_inputs":seal(frozen_path),"rows":rows,"paired_Go_status":"running; only completed actual complete-input cases count","original_expanded_Go_handle":59906,"shared_Go_handle":67201,"all_actual_validation_handles_terminal":False,"current_expanded_prototype_active_root_consolidation":"pending until both original actual Go handles are terminal; retain original full source/failed guard/raw outputs, keep the complete successful shared matrix active, retire only its redundant failing expanded prototype root","all_current_active_test_targets_green":False,"strict_changed_input_review":"pending after final consolidation and every relevant actual handle is terminal","staged_or_committed":False,"T02_W01_complete":False,"CSharp_tests_executed":False,"public_profile_activated":False,"whole_source_program_proof_claimed":False,"immutable_application_replay_claimed":False,"native_Linux_acceptance_claimed":False,"remaining_application_demands":169,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp","scope":"All65 full shared ordinary inputs really pass unchanged Rust kernel with zero axioms and frozen4096decl/131072term/2MiB/1024fuel;93,600 arbitrary-type/value per-field obligations and all2210 original goal types remain. This does not claim completion of still-running same-byte Go validation, source-bound Go operations or full56-family assembly/context/descriptor/private registry/retained-policy delegation/original package/export/selected-closure immutable loading and application replay. Original expanded producer bound failure and all diagnostic failures/raws remain separate; expanded60..64 construction-only inputs are not admitted. All169 demands and T02-T11 remain required."}
p=root/"shared-Rust-checkpoint.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"Rust_arities":65,"leaf_obligations":93600,"identical_goal_types":2210,"max_terms":32198,"max_bytes":305189}))
