"""Reconstruct complete entry equations and compare every original declaration."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("source_entry_binary_reader",ROOT.parent/"source-invocations/structural-oracle.py")
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
f=m.f
freeze=m.m.frozen_ast


def seal(path):
    raw=path.read_bytes()
    return {"path":str(path),"bytes":len(raw),"sha256":hashlib.sha256(raw).hexdigest()}


def named(table,term):
    names=list(table)
    def value(row):
        _,kind,ty,*body=row
        if kind==1:return kind,term(ty),term(body[0]),body[1]
        if kind==2:return kind,term(ty),term(body[0])
        if kind in [4,5]:return kind,term(ty),names[body[0]],body[1]
        assert kind==3
        return kind,term(ty)
    return {name:value(row) for name,row in table.items()}


def inspect(path,meta):
    raw,module,table,term=m.decode(path)
    assert module==meta["module"]
    source=meta["source_module"]
    carrier=lambda tid:f.const(source+".T"+tid+".Carrier")
    zero=lambda tid:f.const(source+".T"+tid+".Zero")
    def own(name,*args):return f.app(f.const(module+"."+name),*args)
    checked=set()
    def check(name,args,result,body,law=False):
        row=table[module+"."+name]
        assert row[1]==(2 if law else 1),(module,name,"kind")
        assert term(row[2])==freeze(f.bind("pi",args,result)),(module,name,"complete type")
        assert term(row[3])==freeze(f.bind("lam",args,body)),(module,name,"complete body/proof")
        checked.add(module+"."+name)
    input_types=[carrier(t) for t,_ in meta["inputs"]]
    active=[(i,b) for i,b in enumerate(meta["bindings"]) if b["origin"][0]!="local"]
    env_types=[carrier(b["type"]) for _,b in active]
    arg_type,env_type=m.pt(input_types),m.pt(env_types)
    check("Arguments",[],{"kind":"sort0"},arg_type)
    check("Environment",[],{"kind":"sort0"},env_type)
    args=[("arguments",arg_type)]
    initial=[]
    for _,binding in active:
        origin=binding["origin"]
        initial.append(m.get(input_types,f.var("arguments"),origin[1]) if origin[0]=="input" else zero(binding["type"]))
    check("Initialize",args,env_type,m.make(env_types,initial))
    if not active:
        check("EmptyEnvironment",args,f.eq(f.boolean(),own("Initialize",f.var("arguments")),f.bit(False)),f.refl(f.boolean(),f.bit(False)),True)
    for i,binding in enumerate(meta["bindings"]):
        present=f.bit(binding["origin"][0]!="local")
        check("ExistsAtEntry"+str(i),[],f.boolean(),present)
        check("EntryPresence"+str(i),[],f.eq(f.boolean(),own("ExistsAtEntry"+str(i)),present),f.refl(f.boolean(),present),True)
        positions=[j for j,(index,_) in enumerate(active) if index==i]
        if positions:
            position=positions[0];ty=carrier(binding["type"])
            check("Binding"+str(i),[("environment",env_type)],ty,m.get(env_types,f.var("environment"),position))
            check("InitializedBinding"+str(i),args,f.eq(ty,own("Binding"+str(i),own("Initialize",f.var("arguments"))),initial[position]),f.refl(ty,initial[position]),True)
        else:
            assert module+".Binding"+str(i) not in table,"inactive local has no fabricated entry cell"
    parameters=[("argument"+str(i),ty) for i,ty in enumerate(input_types)]
    values=[f.var(name) for name,_ in parameters]
    check("PackArguments",[],f.bind("pi",parameters,arg_type),f.bind("lam",parameters,m.make(input_types,values)))
    if len(input_types)>1:
        predicate=f.pi("value",arg_type,{"kind":"sort0"})
        application=f.app(f.var("predicate"),own("PackArguments",*values))
        check("ForAllArguments",[("predicate",predicate)],{"kind":"sort0"},f.bind("pi",parameters,application))
        evidence=f.pi("value",arg_type,f.app(f.var("predicate"),f.var("value")))
        check("ForAllArgumentsProof",[("predicate",predicate),("evidence",evidence)],own("ForAllArguments",f.var("predicate")),f.bind("lam",parameters,f.app(f.var("evidence"),own("PackArguments",*values))))
    if not input_types:
        check("EmptyArguments",[],f.eq(f.boolean(),own("PackArguments"),f.bit(False)),f.refl(f.boolean(),f.bit(False)),True)
    for i,ty in enumerate(input_types):
        check("Argument"+str(i),args,ty,m.get(input_types,f.var("arguments"),i))
        claim=f.bind("pi",parameters,f.eq(ty,own("Argument"+str(i),own("PackArguments",*values)),values[i]))
        if len(input_types)>1:
            projection=own("Argument"+str(i),f.var("values"))
            predicate=f.lam("values",arg_type,f.eq(ty,projection,projection))
            evidence=f.lam("values",arg_type,f.refl(ty,projection))
            proof=own("ForAllArgumentsProof",predicate,evidence)
        else:proof=f.bind("lam",parameters,f.refl(ty,values[i]))
        check("PackedArgument"+str(i),[],claim,proof,True)
    actual={name for name in table if name.startswith(module+".")}
    assert checked==actual,"missing or extra entry declaration"
    # Every bound input/result keeps its original position and exact type.
    for role,slots in [("input",meta["inputs"]),("named_result",meta["results"])]:
        for position,(typ,index) in enumerate(slots):
            if index is not None:
                b=meta["bindings"][index]
                assert b["origin"]==[role,position] and b["type"]==typ
    return named(table,term),len(checked)


def main():
    certs=ROOT/(sys.argv[1] if len(sys.argv)>1 else "rust-factored")/"certificates"
    units=[]
    total_members=total_declarations=0
    for unit in ["pure","wide","supplement"]:
        inventory=json.loads((certs/(unit+"-inventory.json")).read_bytes())
        expected={};comparisons=members=0
        rows=sorted((certs/"rows").glob(unit+"-*.mpcert"))
        assert len(rows)==inventory["all_selected_bodies"]==len(inventory["rows"])
        for path,metadata in zip(rows,inventory["rows"]):
            actual,checked=inspect(path,metadata)
            assert metadata==json.loads(path.with_suffix(".entry.json").read_bytes())
            for name,value in actual.items():
                if name in expected:assert expected[name]==value
                else:expected[name]=value
                comparisons+=1
            members+=checked
        combined={};functions=[];parts=[]
        for part in inventory["parts"]:
            path=certs/part["file"]
            raw,_,table,term=m.decode(path)
            actual=named(table,term)
            for name,value in actual.items():
                if name in combined:assert combined[name]==value
                combined[name]=value
            functions.extend(part["functions"])
            report=json.loads(path.with_suffix(".rust.json").read_bytes())
            assert report["verdict"]=="accepted" and report["axiom_count"]==0
            assert report["hashes"]["certificate"]==hashlib.sha256(b"MPK-MODULE-CERT-0.1\0"+raw).hexdigest()
            assert part["raw_sha256"]==hashlib.sha256(raw).hexdigest()
            own_modules={name.rsplit(".",1)[0] for name in actual if name.startswith("Mpk.Go.Practical.SourceEntry.")}
            assert own_modules=={row[2] for row in part["functions"]}
            parts.append({"certificate":seal(path),"declarations":part["declarations"],"terms":part["terms"]})
        assert combined==expected,"entire original entry union across all complete parts"
        assert functions==[row["function"]+[row["module"]] for row in inventory["rows"]],"every full original function appears exactly once"
        assert len(combined)==inventory["complete_named_declarations"]
        for part in inventory["parts"]:
            _,_,table,term=m.decode(certs/part["file"])
            actual=named(table,term)
            assert {k:v for k,v in expected.items() if not k.startswith("Mpk.Go.Practical.SourceEntry.")}=={k:v for k,v in actual.items() if not k.startswith("Mpk.Go.Practical.SourceEntry.")},"each part retains the entire common source/type/U foundation"
        original_checks=0
        if unit!="supplement":
            old=ROOT.parent/"source-invocations/generation-linked/certificates"/(unit+"-000.mpcert")
            _,_,old_table,old_term=m.decode(old)
            current=combined
            for name,value in named(old_table,old_term).items():
                if not name.startswith("Mpk.Go.Practical.SourceInvocation."):
                    assert current[name]==value,"whole previous source/type/foundation declaration"
                    original_checks+=1
        total_members+=members;total_declarations+=comparisons
        units.append({"unit":unit,"complete_bodies":len(rows),"complete_entry_members":members,"whole_original_declaration_comparisons":comparisons,"previous_source_recipe_comparisons":original_checks,"all_source_cases":len(inventory["cases"]),"source_types":len(inventory["source_types"]),"max_inputs":inventory["max_inputs"],"max_results":inventory["max_results"],"parts":parts,"declarations":inventory["complete_named_declarations"],"incomplete_packing_states":inventory["incomplete_packing_states"]})
    record={"status":"passed","driver":seal(Path(__file__)),"complete_entry_members":total_members,"whole_original_declaration_comparisons":total_declarations,"units":units,"scope":"Independent complete type/body/proof reconstruction and full original declaration preservation. Entry-value equations only; source program, storage/frame/capture/context validity remains due."}
    with (ROOT/"entry-review.json").open("x") as output:json.dump(record,output,indent=2);output.write("\n")
    print(json.dumps({"status":"passed","units":len(units),"entry_members":total_members,"whole_declaration_comparisons":total_declarations}))


if __name__=="__main__":main()
