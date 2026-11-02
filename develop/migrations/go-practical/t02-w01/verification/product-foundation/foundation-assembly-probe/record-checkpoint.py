"""Bind actual maximal closed assembly proof and exact full-original identity checks."""
from pathlib import Path
import datetime,hashlib,json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
checks=[]
for mode in ["tests","clippy","format","listing"]:
    path=root/"initial"/mode/"result.json";d=json.loads(path.read_text())
    assert d["status"]=="passed" and not d["changed_inputs_during_execution"]
    for key in ["stdout","stderr"]:assert seal(d[key]["path"])==d[key]
    for source in d["sources"]:
        assert seal(source["path"])==source
        stored=seal(Path(d["source_snapshot"])/source["path"])
        assert (stored["sha256"],stored["bytes"])==(source["sha256"],source["bytes"])
    assert seal(root/"initial/runner.py")["sha256"]==d["runner"]["sha256"]
    checks.append({"mode":mode,"record":seal(path),"seconds":d["wall_seconds"]})
paired_path=root/"paired/result.json";paired=json.loads(paired_path.read_text());assert paired["status"]=="passed" and len(paired["cases"])==3
for row in paired["cases"]:
    for key in ["certificate","rust_report","stdout","stderr"]:assert seal(row[key]["path"])==row[key]
    assert row["entire_original_bytes_sealed_before_and_after_actual_Go_execution"]
assert [r["go_observation"]["verdict"] for r in paired["cases"]]==["accepted","accepted","rejected"]
for row in paired["cases"][:2]:
    assert row["rust_observation"]["axiom_count"]==0 and row["rust_observation"]["axiom_report"]["entries"]==[]
    assert row["rust_observation"]["hashes"]==row["go_observation"]["hashes"]
    assert row["go_observation"]["hashes"]["axiom_report"]=="0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
negative=paired["cases"][2];assert json.loads(negative["rust_observation"]["error_detail"])["code"]=="CORE_TYPE_MISMATCH"
identity_path=root/"original-identity.json";identity=json.loads(identity_path.read_text());assert identity["status"]=="all_original_named_declarations_levels_and_every_term_preserved_exactly"
assert identity["whole_original_named_union"]==4096 and identity["every_original_term_node_preserved_modulo_exact_structural_sharing"]==101705
assert identity["source_term_nodes_dropped"]==0 and identity["exact_node_equality_not_only_fingerprint_hashes"]
analysis_path=root.parent/"foundation-assembly-analysis/result.json";analysis=json.loads(analysis_path.read_text())
assert analysis["complete_original_components"]==23 and analysis["whole_name_union"]==12249 and not analysis["shared_named_type_body_proof_level_family_conflicts"]
positive=paired["cases"][0];assert positive["certificate"]["bytes"]==1428859 and positive["go_observation"]["declaration_count"]==4096
conflict_path=root/"initial/tests/certificates/nominal-conflict.json";conflict=json.loads(conflict_path.read_text())
assert conflict["actual_assembly_rejection"]=="nominal identity conflict: Std.Eq.symm"
frozen_path=root.parent/"frozen-design-preservation.json";frozen=json.loads(frozen_path.read_text());assert len(frozen["inputs"])==255
for source in frozen["inputs"]:assert seal(source["path"])==source
record={"status":"diagnostic_full4096_assembly_both_cores_and_exact_identity_passed","entry_commit":"7154ec0b2ac2ab56ac21e87e018d2f2c02545366","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"fresh_closed_assembly":{"complete_certificate":positive["certificate"],"declarations":4096,"terms":101705,"bytes":1428859,"axioms":0,"logical_hashes":positive["rust_observation"]["hashes"],"remaining_declaration_slots":0},"complete_original_full64_shift_declarations":4053,"complete_original_product_declarations":78,"shared_original_core_declarations":35,"new_product_declarations":43,"exact_full_original_identity_audit":seal(identity_path),"full23_component_inventory":seal(analysis_path),"full23_component_name_union":12249,"checks":checks,"actual_same_byte_both_core_checks":seal(paired_path),"same_byte_streams":3,"zero_axiom_positives":2,"genuine_core_type_mismatches":1,"valid_alternate_named_proof_rejected_by_exact_assembly":seal(conflict_path),"frozen255_inputs":seal(frozen_path),"all_this_probe_validation_handles_terminal":True,"unrelated_product_Go_matrices_still_running":[59906,67201],"production_assembly_registry_context_implemented":False,"complete56_family_foundation_claimed":False,"source_bound_operations_claimed":0,"remaining_application_demands":169,"T02_W01_complete":False,"CSharp_tests_executed":False,"public_profile_activated":False,"whole_source_program_proof_claimed":False,"immutable_application_replay_claimed":False,"native_Linux_acceptance_claimed":False,"strict_changed_input_review":"pending after current two actual original Go handles finish and final prototype consolidation","staged_or_committed":False,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp","scope":"Actual two unchanged kernels recheck the whole self-contained4096-declaration/101705-term composition with no imports or new kernel rules/axioms/fuel/caps. Every source term/name/level and all4053+43 original type/body/proof/generated-family identities survive exact sharing; no pruning/rewriting/type-only merge. A valid same-name Std.Eq.symm alternate proof passes both cores but strict assembly rejects its changed original identity; a false UnitZeroValid inside the full merged closure genuinely rejects. This is a born-ignored diagnostic feasibility input, not an installed/public assembler or all56 source-family/descriptor/private registry/context/retained-policy/immutable original application replay proof. Whole23 current mathematical inputs contain12249 distinct names and remain complete; none are omitted or relabeled as a completed foundation. All169 application demands and T02-T11 remain required."}
p=root/"verification.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"declarations":4096,"terms":101705,"bytes":1428859,"both_core_streams":3,"zero_axiom_positives":2,"genuine_type_rejections":1,"whole_original_component_union":12249}))
