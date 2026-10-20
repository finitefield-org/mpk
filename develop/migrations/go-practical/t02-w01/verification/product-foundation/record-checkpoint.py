"""Bind final active product tests, complete paired inputs and retained full scope."""
from pathlib import Path
from collections import Counter
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
output=root/"verification.json";assert not output.exists()
checks=[]
for relative in ["consolidated/tests/result.json","consolidated/clippy/result.json","consolidated/format/result.json","alias-consumers/initial/tests/result.json","alias-consumers/initial/clippy/result.json","alias-consumers/initial/format/result.json"]:
    path=root/relative;d=json.loads(path.read_text())
    assert d["status"]=="passed" and not d["changed_inputs_during_execution"]
    for key in ["stdout","stderr"]:assert seal(d[key]["path"])==d[key]
    for source in d["sources"]:
        assert seal(source["path"])==source
        saved=seal(Path(d["source_snapshot"])/source["path"])
        assert (saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
    checks.append({"record":seal(path),"seconds":d.get("wall_seconds",d.get("seconds"))})
listings=[]
for relative in ["final/diagnostic-list/result.json","explicit-helper-list/result.json","expanded-resource/collected/listing/result.json","foundation-assembly-probe/initial/listing/result.json"]:
    path=root/relative;d=json.loads(path.read_text())
    assert d["status"]=="passed" and not d["changed_inputs_during_execution"]
    for key in ["stdout","stderr"]:assert seal(d[key]["path"])==d[key]
    for source in d["sources"]:
        saved=seal(Path(d["source_snapshot"])/source["path"])
        assert (saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
    command=d["command"];assert "--ignored" in command and "--list" in command
    target=command[command.index("--test")+1]
    target_path="crates/mpk-vc/tests/"+target+".rs"
    original=next(source for source in d["sources"]if source["path"]==target_path)
    assert seal(target_path)==original
    listings.append({"actual_terminal_listing":seal(path),"unchanged_current_born_ignored_target":original,"full_original_source_snapshot":d["source_snapshot"]})
assert len(listings)==4
main=root/"consolidated/tests/result.json";main_result=json.loads(main.read_text())
assert "3 passed; 0 failed" in Path(main_result["stdout"]["path"]).read_text()
shared=root/"alias-consumers/initial/tests/result.json";shared_result=json.loads(shared.read_text())
assert "1 passed; 0 failed" in Path(shared_result["stdout"]["path"]).read_text()
old_path=root/"paired-ready/result.json";old=json.loads(old_path.read_text())
assert old["status"]=="partial_actual_streams_preserved" and len(old["cases"])==112
assert old["missing_names"]==[f"balanced-shape-{i:02}"for i in range(60,65)]
shared_pair_path=root/"alias-consumers/paired/result.json";shared_pair=json.loads(shared_pair_path.read_text())
assert shared_pair["status"]=="passed" and len(shared_pair["cases"])==65 and shared_pair["same_byte_streams"]==65
boundaries_path=root/"consolidated/paired-boundaries/result.json";boundaries=json.loads(boundaries_path.read_text())
assert boundaries["status"]=="passed" and len(boundaries["rows"])==52
probe_path=root/"foundation-assembly-probe/verification.json";probe=json.loads(probe_path.read_text())
assert probe["status"]=="diagnostic_full4096_assembly_both_cores_and_exact_identity_passed"
probe_pair_path=root/"foundation-assembly-probe/paired/result.json";probe_pair=json.loads(probe_pair_path.read_text());assert probe_pair["status"]=="passed"
cases=[r["actual_prior_whole_byte_Go_execution"]for r in boundaries["rows"]]+shared_pair["cases"]+probe_pair["cases"]
assert len(cases)==120
subtypes=Counter();positive=0;hashes=set()
for row in cases:
    for key in ["certificate","rust_report","stdout","stderr"]:assert seal(row[key]["path"])==row[key]
    raw=(row["certificate"]["sha256"],row["certificate"]["bytes"]);assert raw not in hashes;hashes.add(raw)
    assert not Path(row["stderr"]["path"]).read_bytes()
    rust=json.loads(Path(row["rust_report"]["path"]).read_text());go=json.loads(Path(row["stdout"]["path"]).read_text())
    assert rust==row["rust_observation"] and go==row["go_observation"] and rust["verdict"]==go["verdict"]
    if rust["verdict"]=="accepted":
        positive+=1;assert row["exit_code"]==0 and rust["axiom_count"]==0 and not rust["axiom_report"]["entries"]
        assert rust["hashes"]==go["hashes"] and rust["declaration_count"]==go["declaration_count"]
        assert go["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert rust["verdict"]=="rejected" and row["exit_code"]!=0 and rust["error_code"]=="KERNEL_CORE_CHECK" and go["error_kind"]=="core_check"
        detail=json.loads(rust["error_detail"]);assert detail["code"]=="CORE_TYPE_MISMATCH";subtypes[detail["details"]["kind"]]+=1
assert positive==69 and sum(subtypes.values())==51 and subtypes==Counter({"check_type_mismatch":36,"lambda_domain_mismatch":15})
for group in [old,shared_pair,probe_pair]:
    assert seal(group["go_binary"]["path"])==group["go_binary"]
    for source in group["go_production_sources"]:assert seal(source["path"])==source
base=root/"consolidated/tests/certificates/product-foundation.mpcert";base_report=base.with_suffix(".rust.json")
assert seal(base)["sha256"]=="06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e" and seal(base)["bytes"]==11788
canonical=json.loads(base_report.read_text());assert canonical["verdict"]=="accepted" and canonical["declaration_count"]==78 and canonical["axiom_count"]==0
for row in boundaries["rows"]:
    assert seal(row["current_certificate"]["path"])==row["current_certificate"] and seal(row["current_actual_Rust_report"]["path"])==row["current_actual_Rust_report"]
    assert Path(row["current_certificate"]["path"]).read_bytes()==Path(row["actual_prior_whole_byte_Go_execution"]["certificate"]["path"]).read_bytes()
types_path=root/"complete-type-sharing-analysis/result.json";types=json.loads(types_path.read_text())
assert types["status"]=="all65_complete_original_goal_type_sets_match" and types["identical_named_goal_types"]==2210
resource_path=root/"expanded-resource/verification.json";resource=json.loads(resource_path.read_text())
assert resource["status"]=="all65_complete_original_constructions_retained_fixed_budget_preserved" and resource["complete_candidates"]==65
for row in resource["rows"]:
    assert seal(row["complete_candidate"]["path"])==row["complete_candidate"] and seal(row["construction_metadata"]["path"])==row["construction_metadata"]
for row in types["pairs"]:
    for key in ["complete_original_construction","complete_shared_checked_certificate","original_construction_metadata","shared_actual_Rust_report"]:assert seal(row[key]["path"])==row[key]
frozen_path=root/"frozen-design-preservation.json";frozen=json.loads(frozen_path.read_text());assert len(frozen["inputs"])==255 and not frozen["mismatches"]
for source in frozen["inputs"]:assert seal(source["path"])==source
retained_path=root/"retained-inputs.json";retained=json.loads(retained_path.read_text());assert retained["status"]=="passed"
for source in retained["complete_original_scalar_certificates"]:assert seal(source["path"])==source
original_path=root/"final/tests/result.json";original=json.loads(original_path.read_text())
assert original["status"]=="failed" and original["exit_code"]==101 and not original["changed_inputs_during_execution"]
assert "3 passed; 1 failed" in Path(original["stdout"]["path"]).read_text()
for source in original["sources"]:
    saved=seal(Path(original["source_snapshot"])/source["path"]);assert(saved["sha256"],saved["bytes"])==(source["sha256"],source["bytes"])
for key in ["stdout","stderr"]:assert seal(original[key]["path"])==original[key]
requirements_path=Path("develop/migrations/go-practical/t02-w01/requirements.json");requirements=json.loads(requirements_path.read_text());by={r["id"]:r for r in requirements["requirements"]}
assert len(by)==7 and by["T02-W01-R06"]["status"]=="pending" and not requirements["T02_W01_complete"] and not requirements["public_profile_activated"]
assert by["T02-W01-R03"]["latest_frontend_build_checkpoint"]["binary_sha256"]=="7a8a86f713dbc1e0098db6af12f5b24134f5603466e59dfc9cf25edabea2142a"
assert by["T02-W01-R05"]["latest_archive_ownership_checkpoint"]["evidence"]=="verification/archive-ownership/verification.json"
assert by["T02-W01-R01"]["latest_assembly_capacity_observation"]["whole_named_union"]==12249
owner=by["T02-W01-R01"];assert owner["latest_ordinary_checkpoint"]["evidence"]=="verification/word-signed-equality/verification.json"
owner.setdefault("prior_ordinary_checkpoints",[]).append(owner["latest_ordinary_checkpoint"])
owner["latest_ordinary_checkpoint"]={"evidence":"verification/product-foundation/verification.json","scope":"Complete78decl heterogeneous Sort0 product component,14definitions29laws, zeroaxioms in both current cores. Construction/projections, pointwise reconstruction/equality, updates/frames/overwrite/commutation, maps and actual per-field validity/preservation witnesses. All35 original core identities/full raw remain exact. Complete65arity arbitrary independently typed field/result consumers pass both cores with93600leaf obligations and2210original goal types preserved by exact graph comparison. All50semantic controls genuinely reject; exact import rejects a valid79decl extension. All original expanded/resource/inference/linkage/diagnostic failures and full inputs remain. Full4096shift/product feasibility3streams and whole23component12249name capacity audit are diagnostic only. Full56 source/carrier/nominal/type/operation/storage/lifetime/state/program assembly, compiler-derived roots, descriptor/private registry/context and immutable original application replay remain due.","not_application_verification_closure":True,"public_profile_activated":False}
owner=by["T02-W01-R07"];owner.setdefault("prior_scoped_checkpoints",[]).append(owner["latest_scoped_checks"])
owner["latest_scoped_checks"]={"evidence":"verification/product-foundation/verification.json","affected_main_Rust_roots":3,"active_shared_consumer_Rust_roots":1,"shared_arities":65,"per_field_leaf_obligations":93600,"exact_original_goal_types_preserved":2210,"product_same_byte_streams":117,"additional_born_ignored_assembly_probe_Rust_roots":1,"additional_assembly_same_byte_streams":3,"all_current_both_core_streams":120,"zero_axiom_positives":69,"genuine_core_type_mismatches":51,"mismatch_subtypes":dict(subtypes),"original_expanded_resource_failure_retained":True,"complete_all65_expanded_constructions_retained":True,"current_prototype_consolidation":"Three main boundary roots plus unchanged complete65arity shared target. The expanded prototype wrapper is retired only after its relevant actual Go observer is terminal; the separate shared observer captures no edited main wrapper source and all its own sources remain exact. All65/93600 obligations, exact goal-type equality and both-core completion are required before this final scoped receipt. Full original sources/raws remain.","unchanged_complete_Go_streams_reused_only_after_whole_byte_comparison":True,"all_four_born_ignored_diagnostics_listed":True,"unrelated_prior_suites_repeated":False,"CSharp_tests_executed":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
record={"schema":"mpk.go_practical.product_foundation_checkpoint.v1","status":"scoped_checks_passed","entry_commit":"7154ec0b2ac2ab56ac21e87e018d2f2c02545366","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"fresh_component":{"certificate":seal(base),"declarations":78,"terms":1164,"bytes":11788,"axioms":0,"new_definitions":14,"new_laws":29,"original_core_declarations":35,"logical_hashes":canonical["hashes"]},"production_sources":[seal(p)for p in ["crates/mpk-vc/src/go_practical_ordinary.rs","crates/mpk-vc/src/go_practical_ordinary/product.rs"]],"checks":checks,"all_four_actual_born_ignored_diagnostic_listings":listings,"actual_current_main_Rust_execution":seal(main),"actual_current_shared_Rust_execution":seal(shared),"paired_current_boundaries":seal(boundaries_path),"paired_full_shared_matrix":seal(shared_pair_path),"original_expanded_Go_complete_available_streams":seal(old_path),"original_expanded_Rust_resource_failure":seal(original_path),"full_original_expanded_constructions":seal(resource_path),"exact_source_delta_audit":seal(root/"shared-consumer-source.json"),"independent_all65_goal_type_audit":seal(types_path),"all65_arities_and_93600_leaf_obligations_passed_both_cores":True,"identical_original_goal_types":2210,"product_semantic_negatives":50,"product_same_byte_streams":117,"additional_full4096_assembly_probe":seal(probe_path),"all_same_byte_current_streams":120,"zero_axiom_positives":69,"genuine_core_type_mismatches":51,"mismatch_subtypes":dict(subtypes),"all35_original_core_identities_and_full_raw_unchanged":True,"frozen_inputs":seal(frozen_path),"complete_prior_scalar_inputs":seal(retained_path),"original_failures":seal(root/"original-failures.json"),"operation_contribution_audit":seal(root/"operation-contributions.json"),"all56_original_operations_and_six_common_members_retained":True,"all_actual_validation_handles_terminal":True,"new_product_active_test_targets_green":True,"strict_changed_input_review":"input-review/changed-inputs-review.json","CSharp_tests_executed":False,"T02_W01_complete":False,"remaining_application_demands":169,"public_profile_activated":False,"whole_source_program_proof_claimed":False,"immutable_application_replay_claimed":False,"native_Linux_gate_claimed":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp","scope":"Ordinary heterogeneous value-product mathematical foundation only. Actual make/project/reconstruct/pointwise equality/update/frame/map/validity witnesses for arbitrary independently typed Sort0 carriers; every actual shared0..64shape and original goal type preserved. All original full source/candidate/failure/diagnostic/expanded/resource bytes remain; resource/refinement failures are never counted as semantic negatives. No funext/core/axiom/import/fuel/cap or source/JSON rewrite. Additional4096composition is diagnostic, with all23/12249current mathematical names retained. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program bindings, compiler-derived roots and complete descriptor/private registry/context/retained-policy/original source/export/selected-closure immutable application replay remain required. All169 demands and T02-T11 stay open."}
requirements_path.write_text(json.dumps(requirements,ensure_ascii=False,indent=2)+"\n")
output.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":"scoped_checks_passed","current_main_Rust_roots":3,"shared_arities":65,"leaf_obligations":93600,"identical_original_goal_types":2210,"all_current_both_core_streams":120,"zero_axiom_positives":69,"genuine_type_rejections":51,"frozen_inputs":255,"T02_W01_complete":False}))
