"""Closed typed contract expressions with nominal ids, namespaces and lexical bindings."""
import copy
from dataclasses import replace
from contract_scope import Scope,exact,need,GO,SHARED,SCOPE_SCHEMA,SCOPE_DOMAIN,BINDING_SCHEMA,BINDING_DOMAIN
from overlay_contracts import (BOOL,NAT,INT,Context,Expressions,UNARY,BINARY,WORD_UNARY,WORD_BINARY,COMPARISONS,QUANTIFIERS,OPERATORS)
from closed_carriers import sealed
from identity_wire import canonical,digest

NORMAL_OPS=(OPERATORS-{"local"})|{"binding"}
UNARY_OPS=set(UNARY)|WORD_UNARY|{"bool.value","nat.from_unsigned","nat.from_signed","int.from_unsigned","int.from_signed","length","capacity","is_nil"}
BINARY_OPS=set(BINARY)|WORD_BINARY|COMPARISONS|{"byte_at","element","bytes.prefix","bytes.suffix","bytes.concat","sequence.equal"}
ARITIES={**{o:1 for o in UNARY_OPS},**{o:2 for o in BINARY_OPS},"if":3,"bytes.slice":3,"slice.view2":3,"slice.view3":4}
HEAP_OPS={"element","sequence.equal"}


def type_ref(scope,t):return {"kind":"logical","name":{BOOL:"Bool",NAT:"Nat",INT:"Int"}[t]} if t in {BOOL,NAT,INT} else {"kind":"source","source_type_id":scope.ids[scope.types.resolve(t)]}


def state(ctx):return "entry" if ctx.entry or ctx.phase=="requires" else "return" if ctx.phase=="ensures" else "header"


def context(scope,phase,ordinal=None):
    f=scope.function;loop=None if ordinal is None else next(l for l in f["loops"] if l["ordinal"]==ordinal)
    return Context(phase,tuple(f["inputs"]),tuple(f["results"]),tuple(loop["bindings"]) if loop else (),loop["kind"] if loop else "")


def check_source_budget(expressions):
    from contract_catalog import preflight
    return preflight(expressions)[0]


class WireExpressions:
    def __init__(self,scope):self.scope=scope

    def encode(self,e,ctx,ordinal=None,stack=()):
        ctx=replace(ctx,variables=tuple((name,NAT) for name in stack))
        normal,t=Expressions(self.scope.types).check(e,ctx);op=normal["op"];p={}
        rec=lambda child,c=ctx,ss=stack:self.encode(child,c,ordinal,ss)
        if op in {"bool.literal","nat.literal","int.literal","word.literal","bytes.literal"}:p={"value":normal["value"]}
        elif op=="source.zero":p={}
        elif op in {"input","result"}:
            n=normal["position"];ids=self.scope.inputs if op=="input" else self.scope.results
            p={"position":n,"binding_id":ids[n],"namespace":"entry_arguments" if op=="input" else "return_values"}
        elif op=="binding":
            need(ordinal in self.scope.loop_ids,"one current header for local binding")
            p={"binding_id":self.scope.bindings[normal["id"]]["id"],"loop_ref_id":self.scope.loop_ids[ordinal],"namespace":"header"}
        elif op=="bound":p={"depth":len(stack)-1-stack.index(e["name"])}
        elif op in {"range.index","range.length"}:p={"loop_ref_id":self.scope.loop_ids[ordinal],"namespace":"header"}
        elif op=="old":p={"namespace":"entry","value":rec(e["value"],replace(ctx,entry=True))}
        elif op=="word.convert":p={"value":rec(e["value"])}
        elif op=="field":p={"value":rec(e["value"]),"owner_type_id":self.scope.ids[normal["owner_type"]],"field_ordinal":int(normal["field"].split(":")[1])}
        elif op in {"error.is","error.payload"}:p={"value":rec(e["value"]),"case_type_id":self.scope.ids[normal["case"]]}
        elif op in QUANTIFIERS:
            p={"lower":rec(e["lower"]),"upper":rec(e["upper"]),"body":rec(e["body"],ctx,stack+(e["name"],))}
        else:
            need(op in ARITIES,"registered exact expression head")
            p={"args":[rec(c) for c in e["args"]]}
            if op in HEAP_OPS:p["namespace"]=state(ctx)
        return {"op":op,"type":type_ref(self.scope,t),"payload":p}

    def decode(self,node,ctx,ordinal=None,stack=()):
        exact(node,{"op","type","payload"});op=node["op"];need(type(op) is str and op in NORMAL_OPS,"closed70 expression heads")
        tr=node["type"];need(type(tr) is dict and type(tr.get("kind")) is str and tr.get("kind") in {"logical","source"},"closed expression type union")
        if tr["kind"]=="logical":exact(tr,{"kind","name"});need(type(tr["name"]) is str and tr["name"] in {"Bool","Nat","Int"},"logical type")
        else:exact(tr,{"kind","source_type_id"});need(type(tr["source_type_id"]) is str and tr["source_type_id"] in self.scope.names,"captured nominal type id")
        p=node["payload"];raw={"op":op};rec=lambda n,c=ctx,ss=stack:self.decode(n,c,ordinal,ss)
        if op in {"bool.literal","nat.literal","int.literal","word.literal","bytes.literal"}:
            exact(p,{"value"});raw["value"]=p["value"]
            if op in {"word.literal","bytes.literal"}:need(tr["kind"]=="source","typed source literal");raw["type"]=self.scope.names[tr["source_type_id"]]
        elif op=="source.zero":exact(p,set());need(tr["kind"]=="source","source zero type");raw["type"]=self.scope.names[tr["source_type_id"]]
        elif op in {"input","result"}:
            exact(p,{"position","binding_id","namespace"});slots=self.scope.inputs if op=="input" else self.scope.results
            need(type(p["position"]) is int and 0<=p["position"]<len(slots) and p["binding_id"]==slots[p["position"]],"exact same-function positional binding")
            need(p["namespace"]==("entry_arguments" if op=="input" else "return_values"),"exact value observation namespace");raw["position"]=p["position"]
        elif op=="binding":
            exact(p,{"binding_id","loop_ref_id","namespace"})
            need(ordinal in self.scope.loop_ids and p["loop_ref_id"]==self.scope.loop_ids[ordinal] and p["binding_id"] in self.scope.header_ids[ordinal] and p["namespace"]=="header","own exact live lexical header binding")
            b=self.scope.native_bindings[p["binding_id"]];raw={"op":"local","name":b["name"]}
        elif op=="bound":
            exact(p,{"depth"});need(type(p["depth"]) is int and 0<=p["depth"]<len(stack),"lexical de Bruijn bound reference");raw["name"]=stack[-1-p["depth"]]
        elif op in {"range.index","range.length"}:
            exact(p,{"loop_ref_id","namespace"});need(ordinal in self.scope.loop_ids and p["loop_ref_id"]==self.scope.loop_ids[ordinal] and p["namespace"]=="header","own current range header")
        elif op=="old":exact(p,{"namespace","value"});need(p["namespace"]=="entry","freeze whole entry observation");raw["value"]=rec(p["value"],replace(ctx,entry=True))
        elif op=="word.convert":exact(p,{"value"});need(tr["kind"]=="source","source conversion target");raw.update(type=self.scope.names[tr["source_type_id"]],value=rec(p["value"]))
        elif op=="field":
            exact(p,{"value","owner_type_id","field_ordinal"});raw["value"]=rec(p["value"])
            child_type=p["value"]["type"];need(child_type.get("kind")=="source" and p["owner_type_id"]==child_type["source_type_id"],"exact nominal field owner")
            fields=self.scope.types.base(self.scope.names[p["owner_type_id"]]).get("fields",[])
            need(type(p["field_ordinal"]) is int and 0<=p["field_ordinal"]<len(fields),"direct ordered field");raw["field"]=fields[p["field_ordinal"]]["name"]
        elif op in {"error.is","error.payload"}:
            exact(p,{"value","case_type_id"});need(type(p["case_type_id"]) is str and p["case_type_id"] in self.scope.names,"captured error case id");raw.update(value=rec(p["value"]),case=self.scope.names[p["case_type_id"]])
        elif op in QUANTIFIERS:
            exact(p,{"lower","upper","body"});name="q"+str(len(stack));raw.update(name=name,lower=rec(p["lower"]),upper=rec(p["upper"]),body=rec(p["body"],ctx,stack+(name,)))
        else:
            exact(p,{"args","namespace"} if op in HEAP_OPS else {"args"});need(type(p["args"]) is list and len(p["args"])==ARITIES[op],"exact expression arity")
            if op in HEAP_OPS:need(p["namespace"]==state(ctx),"descriptor and heap observation share the full snapshot namespace")
            raw["args"]=[rec(c) for c in p["args"]]
        old_ctx=replace(ctx,variables=tuple((name,NAT) for name in stack));_,actual=Expressions(self.scope.types).check(raw,old_ctx)
        need(canonical(type_ref(self.scope,actual))==canonical(tr),"independently inferred complete typed expression result")
        return raw


def normalize_contract(scope):
    scope.assert_integrity()
    raw=scope.raw;source_exprs=list(raw["requires"])+list(raw["ensures"])
    need(len(raw["requires"])<=64 and 1<=len(raw["ensures"])<=64 and len(raw["loops"])<=1024,"contract clause/loop counts")
    for loop in raw["loops"]:
        need(type(loop["invariants"]) is list and 1<=len(loop["invariants"])<=64,"explicit invariant count")
        source_exprs+=loop["invariants"]
        if loop["variant"]["kind"]=="candidate":source_exprs.append(loop["variant"]["expression"])
    check_source_budget(source_exprs);old=scope.legacy_normalized();expr=WireExpressions(scope)
    requires=[expr.encode(e,context(scope,"requires")) for e in raw["requires"]];ensures=[expr.encode(e,context(scope,"ensures")) for e in raw["ensures"]]
    loops=[];supplied={r["selector"]["ordinal"]:r for r in raw["loops"]}
    for row in old["loops"]:
        ordinal=row["loop_ref"]["function_preorder_ordinal"];r=supplied.get(ordinal);ctx=context(scope,"header",ordinal)
        invariants=[expr.encode(e,ctx,ordinal) for e in r["invariants"]] if r else [expr.encode({"op":"bool.literal","value":True},ctx,ordinal)]
        variant=row["variant"]
        if variant["kind"]=="candidate":
            source=r["variant"]["expression"] if r and r["variant"]["kind"]=="candidate" else {"op":"nat.sub","args":[{"op":"range.length"},{"op":"range.index"}]}
            variant={"kind":"candidate","origin":variant["origin"],"expression":expr.encode(source,ctx,ordinal)}
        loops.append({"loop_ref_id":scope.loop_ids[ordinal],"invariants":invariants,"invariant_origin":row["invariant_origin"],"variant":variant})
    entries=[]
    for row in old["error_inputs"]:
        t=scope.function["inputs"][row["input"]]["type"];path=[]
        for step in row["path"]:
            base=scope.types.base(t);owner=scope.ids[scope.types.resolve(t)]
            if step=="elements":path.append({"kind":"elements","owner_type_id":owner,"extent":"capacity" if base["kind"]=="slice" else "length"});t=base["element"]
            else:
                n=int(step.split(":")[1]);path.append({"kind":"field","owner_type_id":owner,"ordinal":n});t=base["fields"][n]["type"]
        entries.append({"input_binding_id":scope.inputs[row["input"]],"path":path,"case_type_ids":sorted(scope.ids[t] for t in row["cases"])})
    return sealed(GO["normalized_contract"][0],GO["normalized_contract"][1],{"semantic_context":scope.context,"source_scope_sha256":scope.value["content_sha256"],"source_overlay_object_id":scope.overlay_id,"capture_context_id":scope.capture,"function_decl_id":scope.declaration,"signature_id":scope.signature,"termination":raw["termination"],"requires":requires,"ensures":ensures,"error_inputs":entries,"modifies":[],"loops":loops,"entry_storage":old["entry_storage"],"pending_candidates":[{"loop_ref_id":scope.loop_ids[p["ordinal"]],"reason":"missing checked variant candidate"} for p in old["pending_candidates"]]},"contract_hash")


def validate_contract(scope,contract):
    expected=normalize_contract(scope)
    need(canonical(contract)==canonical(expected),"field-complete source/context/type/scope/loop/entry/contract linkage and complete hash")
    wire=WireExpressions(scope)
    for phase in ("requires","ensures"):
        for e in contract[phase]:wire.decode(e,context(scope,phase))
    for i,l in enumerate(contract["loops"]):
        ordinal=scope.function["loops"][i]["ordinal"];ctx=context(scope,"header",ordinal)
        for e in l["invariants"]:wire.decode(e,ctx,ordinal)
        if l["variant"]["kind"]=="candidate":wire.decode(l["variant"]["expression"],ctx,ordinal)
    return True
