"""Independent exact full-table identity observation for the actual assembled certificate."""
from pathlib import Path
import datetime,hashlib,json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
reader=root.parent/"foundation-assembly-analysis/measure.py"
ns={"__file__":str(reader.resolve())}
exec(compile(reader.read_text().split("inputs=[]",1)[0],str(reader),"exec"),ns)
Tables,ExactArena=ns["Tables"],ns["ExactArena"]
execution=root/"initial/tests/result.json";ex=json.loads(execution.read_text())
assert ex["status"]=="passed" and not ex["changed_inputs_during_execution"]
for source in ex["sources"]:assert seal(source["path"])==source
folder=execution.parent/"certificates"
originals=[folder/"original-shift.mpcert",folder/"original-product.mpcert"]
expected=["119c384ddefd5afc6c42de2171df608d4b5b6329304912d3e9ce29860751e1b2","06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e"]
arena=ExactArena();names={};terms=set();all_names=set();sources=[];shared=set()
for p,raw_sha in zip(originals,expected,strict=True):
    record=seal(p);assert record["sha256"]==raw_sha
    t=Tables(record);d,nodes=arena.load(t)
    for name,value in d.items():
        if name in names:assert names[name]==value;shared.add(name)
        names[name]=value
    terms.update(nodes);all_names.update(t.names)
    sources.append({"complete_certificate":record,"declarations":len(d),"source_term_nodes":len(t.terms),"exact_unique_source_term_nodes":len(nodes),"all_original_names":sorted(d)})
merged=folder/"full-shift-product.mpcert";table=Tables(seal(merged));actual,nodes=arena.load(table)
assert actual==names and len(actual)==4096 and len(shared)==35
assert nodes==terms and len(nodes)==len(table.terms)==101705
assert set(table.names)==all_names and len(table.names)==len(all_names)
assert len(table.levels)==len(arena.levels)
report_path=merged.with_suffix(".rust.json");report=json.loads(report_path.read_text())
assert report["verdict"]=="accepted" and report["axiom_count"]==0 and not report["axiom_report"]["entries"]
assert report["declaration_count"]==4096
for original in sources:assert seal(original["complete_certificate"]["path"])==original["complete_certificate"]
record={"status":"all_original_named_declarations_levels_and_every_term_preserved_exactly","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"exact_graph_reader":seal(reader),"actual_terminal_Rust_execution":seal(execution),"originals":sources,"assembled_complete_certificate":seal(merged),"assembled_complete_actual_Rust_report":seal(report_path),"whole_original_named_union":4096,"shared_original_core_identities":35,"exact_original_level_nodes_preserved":len(arena.levels),"every_original_term_node_preserved_modulo_exact_structural_sharing":101705,"all_original_name_table_entries_preserved":len(all_names),"exact_node_equality_not_only_fingerprint_hashes":True,"type_body_proof_level_kind_reducibility_generated_family_identity_conflicts":[],"source_term_nodes_dropped":0,"delta_beta_eta_substitution_or_type_alias_expansion_performed":False,"all_original_declarations_and_full_source_bytes_retained":True,"remaining_decl_slots":0,"scope":"Independent exact DAG comparison of all complete original source tables and the actual assembled4096-declaration input, with no term/declaration pruning, rewriting, substitution or identity-by-type-only. All original type/body/proof/level/generated-family metadata and even unused source term/name/level entries are retained modulo exact interning. Actual Rust and Go proof verdicts remain separately required; this is not the full23-component12249-name library, full56-family/source/context/program foundation or immutable application replay. No imports/axioms/fuel/cap widening, omitted operation or free source proposition. All169 demands and T02-T11 remain due."}
p=root/"original-identity.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"declarations":4096,"shared_original_core_identities":35,"all_original_term_nodes_retained":101705,"remaining_decl_slots":0}))
