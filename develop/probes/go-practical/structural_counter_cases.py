"""Independent exact-weight recipes and representative accounting states."""
import json
from admission_vectors import CASES as ADMISSION
from structural_counters import Shape, Shapes, VALUE_UNITS, CURRENT_UNITS, ALL_UNITS, checked_charge, CounterError, CounterLimit, measure_world
from structural_limit_catalog import generate
from storage_lifetimes import LiveWorld, Value, BYTES, Atom, ErrorValue, error_layout
from storage_views import INT, array, structure, slice_layout
from run_proof_pilot import ROOT, HERE


def boundary_cases():
    rows=[]
    for limit in generate()["counters"]:
        maximum=limit["maximum"]
        for position,count in (("below",maximum-1),("at",maximum),("above",maximum+1)):
            try:
                value=checked_charge(0,count,maximum,limit["id"])
                outcome="counter_within_limit"
            except CounterLimit:
                value=None;outcome=limit["above"]
            except CounterError:
                value=None;outcome="invalid_counter_input"
            expected="invalid_counter_input" if count<0 else limit["above"] if count>maximum else "counter_within_limit"
            if outcome!=expected:raise ValueError("isolated unsigned counter boundary")
            rows.append({"counter":limit["id"],"position":position,"input":count,"maximum":maximum,"outcome":outcome,"retained_counter":value,
                         "scope":"isolated counter preflight only; not a source/application/proof or allocator acceptance"})
    return rows


def exact_weight_shapes(target):
    """Construct any target near2Mi without materializing its values.

    block = 1+256*(1+256) = 65793; outer has <=33 fields.
    A remainder r>=3 is 3+q*257+n with0<=q,n<=256.
    """
    rows=[Shape("word","word"),Shape("row","array","word",256),Shape("block","array","row",256)]
    block=65793;q,r=divmod(target-1,block)
    fields=[("b"+str(i),"block") for i in range(q)]
    if r==1:fields.append(("remainder","word"))
    elif r==2:
        rows.append(Shape("remainder","array","word",1));fields.append(("remainder","remainder"))
    elif r>=3:
        a,b=divmod(r-3,257)
        rows.extend([Shape("rows","array","row",a),Shape("tail","array","word",b),Shape("remainder","struct",fields=(("a","rows"),("b","tail")))])
        fields.append(("remainder","remainder"))
    rows.append(Shape("BudgetValue","struct",fields=tuple(fields)))
    return rows


def source_for_shape(rows):
    shapes={r.key:r for r in rows}
    def type_source(key):
        r=shapes[key]
        if r.kind=="word":return "int64"
        if r.kind=="array":return f"[{r.length}]"+type_source(r.element)
        if r.kind=="struct":return "struct{"+";".join(name+" "+type_source(t) for name,t in r.fields)+"}"
        raise ValueError("closed generated native layout")
    return "package policy\ntype BudgetValue "+type_source("BudgetValue")+"\nfunc Root(v BudgetValue)int{return 0}\n"


def source_inputs():
    rows=[{"id":c["id"],"source":c["source"],"language":c["language"]} for c in ADMISSION]
    rows.append({"id":"original-overlay-quote","source":(HERE/"overlay-fixture.go.txt").read_text(),"language":"go1.25"})
    for n in (VALUE_UNITS-1,VALUE_UNITS,VALUE_UNITS+1):
        rows.append({"id":"weighted-value-"+str(n),"source":source_for_shape(exact_weight_shapes(n)),"language":"go1.25"})
    return rows


def shape_observations():
    rows=[]
    for target in (VALUE_UNITS-1,VALUE_UNITS,VALUE_UNITS+1):
        shapes=Shapes(exact_weight_shapes(target));height,plain,weight=shapes.inspect("BudgetValue")
        if weight!=target:raise ValueError("independent exact boundary weight")
        rows.append({"id":"exact-value-"+str(target),"target_units":target,"weight":weight,"depth":height,"plain":plain,"within_value_candidate":target<=VALUE_UNITS,"value_materialized":False})
    graph=Shapes([Shape("word","word"),Shape("string","string"),Shape("strings","array","string",256),Shape("string_fields","struct",fields=tuple((str(i),"string") for i in range(64))),Shape("both_maxima","array","string_fields",256),Shape("result_product","array","strings",16)])
    for name,expected in (("strings",65793),("string_fields",16449),("both_maxima",VALUE_UNITS+1),("result_product",1052689)):
        weight=graph.weight(name)
        if weight!=expected:raise ValueError("independent aggregate-bound interaction")
        rows.append({"id":name,"weight":weight,"units_exact":name!="both_maxima","true_weight_if_known":4210945 if name=="both_maxima" else expected,"within_value_candidate":weight<=VALUE_UNITS})
    return rows


def quote_type_weights():
    facts=json.loads((ROOT/"develop/migrations/go-practical/t01-w04/overlay-contracts/native-facts.json").read_text())
    shapes=Shapes.from_source_facts(facts,["fixture/policy.RejectCode"])
    quote=next(f for f in facts["functions"] if f["name"]=="Quote")
    inputs=[shapes.weight(v["type"]) for v in quote["inputs"]]
    outputs=[shapes.weight(v["type"]) for v in quote["results"]]
    if inputs!=[257,37,1] or outputs!=[2,2]:raise ValueError("original Quote type/fixed carrier weights")
    return {"inputs":inputs,"results":outputs,"joint_result_units_with_header":1+sum(outputs),"error_universe":["fixture/policy.RejectCode"],"scope":"retained original Go type facts; no source/entry/ordinary shape binding proof"}


def state_observations():
    out={}
    w=LiveWorld();w.begin_scope("caller")
    line=structure("Line",[("Amount",INT)]);decision=structure("Decision",[("Total",INT)]);code=Atom("RejectCode","word",width=8,signed=False);error=error_layout([("RejectCode",code)])
    d=w.allocate("arg","caller",line,128,access="borrowed")
    entry=[Value(BYTES,b"customer"),Value(slice_layout(line),d),Value(INT,100)]
    w.begin_scope("Quote","caller",entry=entry,result_token="call",result_types=(decision,error));w.release("arg")
    for name,v in zip(("customerID","lines","limit"),entry):w.declare("Quote",name,v.layout,v.data)
    w.declare("Quote","total",INT,0);w.declare("Quote","line",line,{"Amount":0})
    out["quote_shaped_active_frame"]=measure_world(w)
    w.put_result("Quote",[Value(decision,{"Total":0}),Value(error,ErrorValue())]);w.end_scope("Quote")
    out["quote_shaped_return_evidence"]=measure_world(w)
    w=LiveWorld();w.begin_scope("caller");w.begin_scope("callee","caller",entry=(),result_token="call",result_types=(slice_layout(INT),slice_layout(INT)))
    a=w.declare("callee","a",array(INT,256));left=w.view(a,0,2,2);right=w.view(a,254,256,256)
    w.put_result("callee",[Value(slice_layout(INT),left),Value(slice_layout(INT),right)]);w.end_scope("callee")
    out["packed_windows_keep_full_post_store"]=measure_world(w)
    out["packed_windows_current_cells"]=sum(o.layout.length for o in w.owners.values() if o.kind=="heap")
    out["packed_windows_post_source_cells"]=w.packets["call"].evidence.post.store.owners[a.owner].layout.length
    w=LiveWorld();w.begin_scope("root");total=w.declare("root","sum",INT,0);peak=0;final=None
    for i in range(1000):
        scope="iteration:"+str(i);w.begin_scope(scope,"root");d=w.allocate("eval",scope,INT,1);w.declare(scope,"tmp",slice_layout(INT),d);w.release("eval")
        w.write(w.element_location(d,0),i);w.write(total,w.read(total)+i)
        peak=max(peak,measure_world(w)["total_units"]);w.end_scope(scope)
    final=measure_world(w)
    out["repeated_allocation"]={"iterations":1000,"sum":w.read(total),"peak_carrier_units":peak,"final_carrier_units":final["total_units"],"ghost_allocation_events":len(w.slot_events),"canonical_finite_id_proved":False}
    return out
