"""Exact named-DAG inventory of full original component inputs; no proof admission."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
verification=root.parent.parent
reader=verification/"word-signed-division/prerequisite-analysis/measure.py"
ns={"__file__":str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]",1)[0],str(reader),"exec"),ns)
Tables=ns["Tables"]


class ExactArena:
    def __init__(self):
        self.levels=[];self.level_index={};self.terms=[];self.term_index={}
    def level(self,key):
        if key not in self.level_index:self.level_index[key]=len(self.levels);self.levels.append(key)
        return self.level_index[key]
    def term(self,key):
        if key not in self.term_index:self.term_index[key]=len(self.terms);self.terms.append(key)
        return self.term_index[key]
    def load(self,t):
        levels=[]
        for tag,args in t.levels:
            value=() if tag==0 else (t.names[args[0]],) if tag==3 else tuple(levels[i] for i in args)
            levels.append(self.level((tag,value)))
        terms=[]
        for tag,args in t.terms:
            if tag==0:value=(levels[args[0]],)
            elif tag==1:value=(args[0],)
            elif tag==2:value=(t.declarations[args[0]][0],tuple(levels[i] for i in args[1]))
            elif tag==3:value=(terms[args[0]],tuple(terms[i] for i in args[1]))
            else:value=tuple(terms[i] for i in args)
            terms.append(self.term((tag,*value)))
        declarations={}
        for name,tag,args in t.declarations:
            if tag in {1,2}:value=(terms[args[0]],terms[args[1]],*args[2:])
            elif tag in {4,5}:value=(terms[args[0]],t.declarations[args[1]][0],args[2])
            else:
                assert tag==3;value=(terms[args[0]],)
            declarations[name]=(tag,*value)
        return declarations,set(terms)
    def used_terms(self,declarations,names):
        selected=set();pending=[]
        for name in names:
            d=declarations[name];pending.extend(d[1:3] if d[0] in {1,2} else d[1:2])
        while pending:
            index=pending.pop()
            if index in selected:continue
            selected.add(index);tag,*args=self.terms[index]
            if tag==3:pending.extend([args[0],*args[1]])
            elif tag in {4,5,6}:pending.extend(args)
        return selected


inputs=[]
for checkpoint in sorted(verification.glob("*/verification.json")):
    data=json.loads(checkpoint.read_text())
    if "fresh_component" not in data:continue
    component=data["fresh_component"]
    inputs.append((checkpoint.parent.name,checkpoint,component["certificate"],component))
for directory,key in [("core-interfaces","ordinary_basis"),("ordinary-boolean","positive_certificate")]:
    checkpoint=verification/directory/"verification.json";data=json.loads(checkpoint.read_text())
    certificate=data[key]["raw_certificate"] if key=="ordinary_basis" else data[key]
    inputs.append((directory,checkpoint,certificate,data[key]))
product_checkpoint=root.parent/"generic-component.json";product=json.loads(product_checkpoint.read_text())
product_raw=root.parent/"explicit-result-types/construction/certificates/product-foundation.mpcert"
inputs.append(("product-foundation",product_checkpoint,seal(product_raw),{"axioms":0}))
assert len({name for name,_,_,_ in inputs})==len(inputs)
arena=ExactArena();declarations={};origins={};rows=[];conflicts=[];by_component={}
for label,checkpoint,source,metadata in inputs:
    before=seal(source["path"]);assert before==source
    table=Tables(source);current,terms=arena.load(table)
    names=set(current);by_component[label]=names
    shared=0;local=[]
    for name,value in current.items():
        if name in declarations:
            shared+=1
            if declarations[name]!=value:
                conflict={"name":name,"earlier_components":list(origins[name]),"current_component":label,"earlier_exact_node_metadata":declarations[name],"current_exact_node_metadata":value}
                conflicts.append(conflict);local.append(conflict)
        else:declarations[name]=value;origins[name]=[]
        origins[name].append(label)
    assert seal(source["path"])==source
    used=arena.used_terms(current,names)
    rows.append({"component":label,"complete_original_checkpoint":seal(checkpoint),"complete_original_certificate":source,"module":table.module,"whole_declarations":len(table.declarations),"whole_terms":len(table.terms),"unique_exact_terms":len(terms),"terms_used_by_all_original_declarations":len(used),"shared_original_named_declarations":shared,"exact_metadata_conflicts":local,"all_original_names":sorted(names),"all_original_declarations_retained_for_this_measurement":True})
    print(json.dumps({"component":label,"declarations":len(names),"shared":shared,"conflicts":len(local)}),flush=True)
    del table,current,terms
assert {"word-shift-count-bounds","word-signed-equality","product-foundation"}.issubset(by_component)
scenarios=[]
for label,components in [("whole_full64_shift_plus_whole_product",["word-shift-count-bounds","product-foundation"]),("whole_full64_shift_signed_equality_and_product",["word-shift-count-bounds","word-signed-equality","product-foundation"]),("whole_all_completed_mathematical_components",list(by_component))]:
    names=set().union(*(by_component[name] for name in components))
    bad=[c for c in conflicts if c["name"] in names]
    used=arena.used_terms(declarations,names)
    scenarios.append({"name":label,"complete_components":components,"complete_named_declaration_union":len(names),"shared_exact_term_nodes_used_by_every_original_declaration":len(used),"declaration_slots_remaining":4096-len(names),"declaration_cap_met_structurally":len(names)<=4096,"term_cap_met_structurally":len(used)<=131072,"exact_named_metadata_conflicts":bad,"all_original_declarations_and_their_full_type_body_proof_terms_retained":True,"all_original_names":sorted(names),"encoded_bytes":"unmeasured; no assembled certificate exists","actual_kernel_check_performed":False})
record={"schema":"mpk.go_practical.ordinary_component_assembly_planning.v1","status":"exact_full_original_named_DAG_measurement_not_checker_authority","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"original_table_reader":seal(reader),"format_sources":[seal("crates/mpk-cert/src/decode.rs"),seal("crates/mpk-cert/src/binary_tags.rs")],"frozen_foundation_design":seal("develop/specs/GO_PRACTICAL_FOUNDATION_V1.md"),"complete_original_components":len(rows),"components":rows,"whole_name_union":len(declarations),"shared_exact_term_node_universe":len(arena.terms),"shared_named_type_body_proof_level_family_conflicts":conflicts,"exact_node_identity_not_only_fingerprint_hashes":True,"scenarios":scenarios,"frozen_limits":{"declarations":4096,"terms":131072,"bytes":2097152,"conversion_fuel":1024,"changed":False},"actual_new_kernel_executions":0,"CSharp_tests_executed":False,"T02_W01_complete":False,"public_profile_activated":False,"remaining_application_demands":169,"full_T02_gate":"deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp","scope":"Every complete original input and every declaration/type/body/proof/level/generated-family identity is retained. Cross-input DAG interning names globals by their exact original names, keeps exact levels/de Bruijn indices/kind/reducibility/family members and compares exact node IDs; it performs no beta/eta/delta/type-alias rewrite. Conflicting same-name metadata is recorded and cannot be silently merged. These are structural counts of mathematical prerequisites only, not an assembled canonical certificate, encoded byte bound, production core verdict, complete56-family source/nominal/type/storage/lifetime/state/program foundation, private registry/context/retained-policy binding or immutable original application replay. No omitted operation/claim/source or free proposition, nonempty-import workaround, axiom/fuel/cap widening or source/JSON normalization is used. Full56 operations and all169 application demands/T02-T11 remain required."}
p=root/"result.json";assert not p.exists();p.write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"components":len(rows),"whole_name_union":len(declarations),"exact_conflicts":len(conflicts),"scenarios":[{k:s[k] for k in ["name","complete_named_declaration_union","shared_exact_term_nodes_used_by_every_original_declaration","declaration_slots_remaining"]} for s in scenarios]}))
