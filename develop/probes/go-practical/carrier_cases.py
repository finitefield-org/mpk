"""Two complete specializations and six altered ordinary definition/proof controls."""
import copy
from closed_carriers import candidate,Specializer,TYPE_DOMAIN,sealed,GO,MODULE
from identity_wire import seal as source_seal
from carrier_terms import bit,word
from foundation_limits_vectors import linked_contexts

CASES=[{"id":"complete-root-types","expected":"accepted","kind":"complete_carriers"},{"id":"plain-error-cell-universe","expected":"accepted","kind":"complete_carriers"}]+[
    {"id":name,"expected":"rejected","kind":kind} for name,kind in (
        ("word-zero-wrong-type","definition_type_mismatch"),("empty-array-noncanonical","altered_zero_law"),
        ("empty-struct-noncanonical","altered_zero_law"),("empty-result-noncanonical","altered_zero_law"),
        ("pair-left-forced-zero","altered_universal_projection"),("pair-right-forced-zero","altered_universal_projection"))]


def plain_error_candidate():
    f,_,_=candidate();types=copy.deepcopy(f["types"])
    old=next(n for n in types["identity_nodes"] if n["id"]==types["error_universe_id"])
    new=source_seal("error_universe",{"source_error_type_id":old["source_error_type_id"],"cases":[f["ids"]["fixture/policy.RejectCode"]]})
    types["identity_nodes"]=sorted([n for n in types["identity_nodes"] if n["id"]!=old["id"]]+[new],key=lambda n:n["id"])
    types["error_universe_id"]=new["id"];types=sealed(types["schema"],TYPE_DOMAIN,{k:v for k,v in types.items() if k not in {"schema","content_sha256"}})
    roots=f["roots"]+[{"role":"source","source_type_id":f["ids"]["[]error"]}]
    i,e=Specializer(types,linked_contexts()["contexts"]["go_practical"]).specialize(roots)
    return types,roots,i,e


def plan(case):
    f,i,e=candidate();name=case["id"]
    if name=="plain-error-cell-universe":_,_,i,e=plain_error_candidate()
    rows=copy.deepcopy(e["declarations"])
    if name in {"word-zero-wrong-type","empty-array-noncanonical","empty-struct-noncanonical","empty-result-noncanonical"}:
        if name=="empty-result-noncanonical":entry=next(x for x in i["entries"] if x["role"]=="joint" and not x["result_types"])
        else:
            source=f["ids"][{"word-zero-wrong-type":"int64","empty-array-noncanonical":"fixture-array-0","empty-struct-noncanonical":"fixture-struct-0"}[name]]
            entry=next(x for x in i["entries"] if x.get("source_type_id")==source)
        next(d for d in rows if d["name"]==MODULE+".I"+entry["id"]+".Zero")["value"]=bit(name!="word-zero-wrong-type")
    elif name.startswith("pair-"):
        left=name=="pair-left-forced-zero";suffix="LeftOfMake" if left else "RightOfMake"
        d=next(d for d in rows if d["name"]==MODULE+".PairWordString."+suffix)
        goal=d["ty"]["body"]["body"]
        zero=word(64)[1] if left else Specializer(f["types"],i["semantic_context"]).carrier(f["ids"]["string"])[1]
        goal["arguments"][2]=zero
    return {"module":e["module"],"declarations":rows}
