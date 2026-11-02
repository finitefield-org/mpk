"""Private source-contract capture, binding and loop identities; not a Go importer."""
import copy
import json
import re
import unicodedata
from closed_carriers import fixture as type_fixture,TYPE_DOMAIN,sealed
from carrier_cases import plain_error_candidate
from foundation_limits_vectors import linked_contexts
from identity_wire import seal as source_seal,validate_graph,canonical,digest,parse
from identity_catalog import GO,SHARED
from context_protocol import go_selection
from overlay_contracts import SOURCE_SCHEMA,Types,normalize
from run_proof_pilot import ROOT,HERE,sha

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/contract-wire"
FACTS=ROOT/"develop/migrations/go-practical/t01-w04/overlay-contracts/native-facts.json"
SCOPE_SCHEMA="mpk.go.contract_scope.v1";SCOPE_DOMAIN="MPK-GO-CONTRACT-SCOPE-1.0"
BINDING_SCHEMA="mpk.go.binding.v1";BINDING_DOMAIN="MPK-GO-BINDING-1.0"


def need(ok,message):
    if not ok:raise ValueError(message)


def exact(value,fields):need(type(value) is dict and set(value)==set(fields),"closed required fields")


def source_overlay(old):
    """Translate retained private vectors, never accept their old schema as new."""
    facts=json.loads(FACTS.read_text());types=Types(facts,["fixture/policy.RejectCode"])
    value=copy.deepcopy(old);value["schema"]=GO["source_overlay"][0]
    selector=value["function"];receiver=selector.get("receiver")
    value["function"]={"package":"fixture/policy","function":selector["name"],"receiver":None if receiver is None else {"package":"fixture/policy","type_name":receiver.split(".")[-1]}}
    value["modifies"]=value.get("modifies",[])
    f=next((f for f in facts["functions"] if f["name"]==selector["name"] and f.get("receiver")==facts["aliases"].get(receiver,receiver)),None)
    if f:
        for row in value["error_inputs"]:
            key=f["inputs"][row["input"]]["type"];path=[]
            for step in row["path"]:
                r=types.base(key)
                if step=="elements":path.append({"kind":"elements"});key=r["element"]
                else:
                    field=r["fields"][int(step.split(":")[1])];path.append({"kind":"field","name":field["name"]});key=field["type"]
            row["path"]=path
    # Optional source variant stays a closed tagged value, never a missing/null payload.
    for loop in value["loops"]:
        loop["variant"]={"kind":"absent"} if "variant" not in loop else {"kind":"candidate","expression":loop["variant"]}
    return value


class Scope:
    def __init__(self,raw,universe=("fixture/policy.RejectCode",),overlay_path="contracts/policy.json",raw_bytes=None):
        exact(raw,{"schema","source","function","termination","requires","ensures","error_inputs","loops","modifies"})
        need(raw["schema"]==GO["source_overlay"][0],"new exact source overlay family")
        need(type(raw["source"]) is str and type(raw["termination"]) is str and raw["termination"] in {"partial","total"},"source path/termination mode")
        for key in ("requires","ensures","error_inputs","loops","modifies"):need(type(raw[key]) is list,"exact source clause/entry/loop arrays")
        for loop in raw["loops"]:
            exact(loop,{"selector","invariants","variant"});exact(loop["selector"],{"kind","start_byte","end_byte","ordinal"})
            need(type(loop["variant"]) is dict,"closed source variant object")
            v=loop["variant"]
            if v.get("kind")=="absent":exact(v,{"kind"})
            else:exact(v,{"kind","expression"});need(v["kind"]=="candidate","source variant tag")
        for row in raw["error_inputs"]:
            exact(row,{"input","path","cases"});need(type(row["path"]) is list and all(type(p) is dict for p in row["path"]),"typed raw entry path")
        go_selection({"schema":GO["selection"][0],"value":{"roots":[raw["function"]]}})
        need(raw["function"]["package"]=="fixture/policy","captured exact package; private fixture only")
        self.raw=copy.deepcopy(raw);self.facts=json.loads(FACTS.read_text());self.types=Types(self.facts,universe)
        self.overlay_path=overlay_path
        receiver=raw["function"]["receiver"]
        receiver=None if receiver is None else "fixture/policy."+receiver["type_name"]
        receiver=self.facts["aliases"].get(receiver,receiver)
        matches=[f for f in self.facts["functions"] if f["name"]==raw["function"]["function"] and f.get("receiver")==receiver]
        need(len(matches)==1,"one exact source function/value receiver")
        self.function=matches[0];self.context=linked_contexts()["contexts"]["go_practical"]
        tf=type_fixture();self.ids=tf["ids"];self.names={v:k for k,v in self.ids.items()};self.source=(HERE/"overlay-fixture.go.txt").read_bytes()
        defs=copy.deepcopy(tf["types"]);nodes=defs["identity_nodes"]
        old_u=next(n for n in nodes if n["id"]==defs["error_universe_id"])
        u=source_seal("error_universe",{"source_error_type_id":old_u["source_error_type_id"],"cases":sorted(self.ids[self.types.resolve(t)] for t in universe)})
        nodes=[n for n in nodes if n["id"]!=old_u["id"]];nodes.append(u);defs["error_universe_id"]=u["id"]
        self.raw_bytes=canonical(raw)+b"\n" if raw_bytes is None else raw_bytes
        need(type(self.raw_bytes) is bytes and len(self.raw_bytes)<=1048576,"captured contract candidate byte ceiling")
        need(canonical(parse(self.raw_bytes))==canonical(raw),"exact original raw overlay bytes; duplicate names never disappear")
        overlay=source_seal("source_object",{"capability":"overlays","path":overlay_path,"kind":"overlay","raw_sha256":sha(self.raw_bytes),"bytes":len(self.raw_bytes)})
        old_capture=next(n for n in nodes if n["schema"]=="mpk.go.capture_context.v1")
        capture=source_seal("capture_context",{k:v for k,v in old_capture.items() if k not in {"schema","id"}}|{"objects":sorted(old_capture["objects"]+[overlay["id"]])})
        nodes=[n for n in nodes if n["id"]!=old_capture["id"]]+[overlay,capture]
        self.capture=capture["id"];self.overlay_id=overlay["id"]
        self.source_id=next(n["id"] for n in nodes if n["schema"]=="mpk.go.source_object.v1" and n["kind"]=="go_source")
        package=next(n["id"] for n in nodes if n["schema"]=="mpk.go.package_instance.v1")
        ordinals={m.start():i for i,m in enumerate(re.finditer(rb"(?m)^(?:type|var|const|func) ",self.source))}
        f=self.function;decl=source_seal("declaration",{"package_id":package,"source_object_id":self.source_id,"kind":"method" if f.get("receiver") else "function","start_byte":f["start_byte"],"end_byte":f["end_byte"],"ordinal":ordinals[f["start_byte"]],"name":f["name"],"receiver_type_id":self.ids[f.get("receiver")] if f.get("receiver") else None})
        self.declaration=decl["id"]
        def tid(t):return self.ids[self.types.resolve(t)]
        signature=source_seal("signature",{"declaration_id":decl["id"],"receiver":self.ids[f.get("receiver")] if f.get("receiver") else None,"params":[tid(b["type"]) for b in f["inputs"][(1 if f.get("receiver") else 0):]],"results":[tid(b["type"]) for b in f["results"]]})
        self.signature=signature["id"]
        self.bindings={};self.native_bindings={};self.inputs=[];self.results=[];self.loop_ids={};self.header_ids={}
        def binding(b,kind,position=None):
            native=b["id"]
            if native in self.bindings:return self.bindings[native]["id"]
            start=int(native.split(":")[1]);suffix=self.source[start:].decode("utf-8");name=b["name"] or None
            token=name if name is not None else "".join(__import__("itertools").takewhile(lambda c:c=="_" or unicodedata.category(c).startswith("L") or unicodedata.category(c)=="Nd",suffix))
            end=start+len((token or suffix[0]).encode("utf-8"))
            row=sealed(BINDING_SCHEMA,BINDING_DOMAIN,{"capture_context_id":self.capture,"source_object_id":self.source_id,"function_decl_id":self.declaration,"kind":kind,"position":position,"name":name,"start_byte":start,"end_byte":end,"source_type_id":tid(b["type"])},"id")
            self.bindings[native]=row;self.native_bindings[row["id"]]=b;return row["id"]
        for i,b in enumerate(f["inputs"]):self.inputs.append(binding(b,"receiver" if i==0 and f.get("receiver") else "input",i))
        for i,b in enumerate(f["results"]):self.results.append(binding(b,"result",i))
        extra_nodes={decl["id"]:decl,signature["id"]:signature}
        for loop in f["loops"]:
            ref=source_seal("loop_ref",{"capture_context_id":self.capture,"source_object_id":self.source_id,"function_decl_id":self.declaration,"ast_kind":loop["kind"],"start_byte":loop["start_byte"],"end_byte":loop["end_byte"],"function_preorder_ordinal":loop["ordinal"],"effective_language":self.facts["language"]})
            self.loop_ids[loop["ordinal"]]=ref["id"];extra_nodes[ref["id"]]=ref
            ids=[binding(b,"local") for b in loop["bindings"]];need(len(ids)==len(set(ids)),"unique live lexical header bindings")
            self.header_ids[loop["ordinal"]]=sorted(ids)
        defs["identity_nodes"]=sorted(nodes,key=lambda n:n["id"])
        defs=sealed(defs["schema"],TYPE_DOMAIN,{k:v for k,v in defs.items() if k not in {"schema","content_sha256"}})
        graph={n["id"]:n for n in nodes};graph.update(extra_nodes)
        roots=set(tf["identity_roots"])-{old_u["id"],old_capture["id"]}|{u["id"],capture["id"]}|set(extra_nodes)
        self.graph=sorted(graph.values(),key=lambda n:n["id"]);self.graph_roots=sorted(roots)
        validate_graph(self.graph,self.graph_roots)
        self.value=sealed(SCOPE_SCHEMA,SCOPE_DOMAIN,{"semantic_context":self.context,"capture_context_id":self.capture,"source_overlay_object_id":self.overlay_id,"source_object_id":self.source_id,"function_decl_id":self.declaration,"signature_id":self.signature,"error_universe_id":u["id"],"type_definitions":defs,"identity_nodes":self.graph,"identity_roots":self.graph_roots,"bindings":sorted(self.bindings.values(),key=lambda b:b["id"]),"inputs":self.inputs,"results":self.results,"headers":[{"loop_ref_id":self.loop_ids[l["ordinal"]],"bindings":self.header_ids[l["ordinal"]]} for l in f["loops"]]})

    def legacy_overlay(self):
        r=copy.deepcopy(self.raw);r["schema"]=SOURCE_SCHEMA
        r["function"]={"name":self.function["name"],**({"receiver":self.function.get("receiver")} if self.function.get("receiver") else {})}
        for row in r["error_inputs"]:
            exact(row,{"input","path","cases"});need(type(row["input"]) is int and 0<=row["input"]<len(self.function["inputs"]),"entry input position")
            key=self.function["inputs"][row["input"]]["type"];path=[]
            for step in row["path"]:
                base=self.types.base(key)
                if step.get("kind")=="elements":exact(step,{"kind"});need(base["kind"] in {"array","slice"},"entry element path");path.append("elements");key=base["element"]
                else:
                    exact(step,{"kind","name"});need(step["kind"]=="field" and base["kind"]=="struct","entry field path")
                    matches=[f for f in base["fields"] if f["name"]==step["name"] and f["name"]!="_"];need(len(matches)==1,"exact entry field")
                    path.append(matches[0]["key"]);key=matches[0]["type"]
            row["path"]=path
        for row in r["loops"]:
            exact(row,{"selector","invariants","variant"});v=row.pop("variant")
            if v.get("kind")=="absent":exact(v,{"kind"})
            else:exact(v,{"kind","expression"});need(v["kind"]=="candidate","closed source variant");row["variant"]=v["expression"]
        return r

    def legacy_normalized(self):return normalize(self.legacy_overlay(),self.facts,self.capture,self.types.universe)

    def assert_integrity(self):
        # Rebuild from original buffers; mutable prototype maps never become an authority.
        fresh=Scope(parse(self.raw_bytes),self.types.universe,self.overlay_path,self.raw_bytes)
        fields=("raw","facts","function","context","ids","names","capture","overlay_id","source_id","declaration","signature","bindings","native_bindings","inputs","results","loop_ids","header_ids","value")
        for name in fields:
            left=json.loads(json.dumps(getattr(self,name),sort_keys=True));right=json.loads(json.dumps(getattr(fresh,name),sort_keys=True))
            need(canonical(left)==canonical(right),"immutable source/scope reconstruction: "+name)
        need(canonical(self.types.rows)==canonical(fresh.types.rows) and canonical(self.types.aliases)==canonical(fresh.types.aliases),"exact immutable captured type facts")
        return True


def validate_scope(scope,value):
    scope.assert_integrity();need(canonical(value)==canonical(scope.value),"complete source scope/context/capture/binding/loop hash reconstruction")
    return True
