"""Closed source-type specialization and ordinary carrier terms, no source proof."""
import copy
import json
import re
from identity_wire import seal as source_seal, validate_node, validate_graph, digest, canonical
from identity_catalog import GO, SHARED
from foundation_limits_vectors import linked_contexts
from structural_counters import Shapes,Shape,VALUE_UNITS
from run_proof_pilot import ROOT,HERE,sha
from carrier_terms import *

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/closed-carriers"
FACTS=ROOT/"develop/migrations/go-practical/t01-w04/overlay-contracts/native-facts.json"
SOURCE=HERE/"overlay-fixture.go.txt"
SEED=ROOT/"develop/migrations/go-practical/t01-w04/nat-termination/type-select-zero-valid.mpcert"
TYPE_SCHEMA="mpk.go.type_definitions.v1";TYPE_DOMAIN="MPK-GO-TYPE-DEFINITIONS-1.0"
INSTANCE_SCHEMA="mpk.go.semantic_instance.v1";INSTANCE_DOMAIN="MPK-GO-SEMANTIC-INSTANCE-1.0"
MODULE="Probe.GoPractical.ClosedCarriers"


def need(ok,message):
    if not ok:raise ValueError(message)


def sealed(schema,domain,body,field="content_sha256"):
    value={"schema":schema,**copy.deepcopy(body)};value[field]=digest(domain,value);return value


def fixture():
    facts=json.loads(FACTS.read_text());raw=SOURCE.read_bytes();nodes=[];ids={};rows={}
    # This fixture has one declaration/spec per top-level line. It is not a general Go parser.
    declaration_ordinals={m.start():i for i,m in enumerate(re.finditer(rb"(?m)^(?:type|var|const|func) ",raw))}
    def add(kind,value):
        n=source_seal(kind,value);nodes.append(n);return n["id"]
    modobj=add("source_object",{"capability":"main","path":"go.mod","kind":"go_mod","raw_sha256":sha(b"module fixture/policy\n\ngo 1.25.0\n"),"bytes":len(b"module fixture/policy\n\ngo 1.25.0\n")})
    obj=add("source_object",{"capability":"main","path":"policy.go","kind":"go_source","raw_sha256":sha(raw),"bytes":len(raw)})
    module=add("module_instance",{"kind":"main","module_path":"fixture/policy","version":"","replacement":{"kind":"none","path":None,"version":None},"objects":sorted([modobj,obj]),"go_mod":modobj})
    package=add("package_instance",{"module_id":module,"import_path":"fixture/policy","package_name":"policy","objects":[obj]})
    for r in facts["types"]:
        if r["kind"]=="named":
            name=r["id"].split(".")[-1];start=raw.index(("type "+name+" ").encode());end=raw.index(b"\n",start)
            decl=add("declaration",{"package_id":package,"source_object_id":obj,"kind":"type","start_byte":start,"end_byte":end,"ordinal":declaration_ordinals[start],"name":name,"receiver_type_id":None})
            ids[r["id"]]=add("source_type",{"kind":"nominal","declaration_id":decl})
    by_name={r["id"]:r for r in facts["types"]}
    def build(key):
        if key in rows:return ids[key]
        r=by_name[key];kind=r["kind"]
        if kind=="named":
            child=build(r["underlying"]);rows[key]={"id":ids[key],"kind":"named","underlying":child}
        elif kind in {"bool","word","string","error"}:
            width=r.get("width");signed=r.get("signed",False) if kind=="word" else None
            sid=add("source_type",{"kind":"builtin","name":key,"width":width,"signed":signed})
            ids[key]=sid;rows[key]={"id":sid,"kind":kind,**({"width":width,"signed":signed} if kind=="word" else {})}
        elif kind in {"array","slice"}:
            child=build(r["element"]);fields={"kind":kind,"element":child}
            if kind=="array":fields["length"]=r["length"]
            sid=add("source_type",fields);ids[key]=sid;rows[key]={"id":sid,**fields}
        elif kind=="struct":
            fields=[{"name":f["name"],"unexported_package_id":None if f["name"][0].isupper() else package,"type_id":build(f["type"]),"tag_bytes_hex":""} for f in r["fields"]]
            sid=add("source_type",{"kind":"struct","fields":fields});ids[key]=sid;rows[key]={"id":sid,"kind":"struct","fields":[{"key":f["key"],"type":build(f["type"])} for f in r["fields"]]}
        else:raise ValueError("closed source type shape")
        return ids[key]
    for key in by_name:build(key)
    aliases=[]
    for key,target in facts["aliases"].items():
        if "/" not in key:continue
        name=key.split(".")[-1];start=raw.index(("type "+name+" ").encode());end=raw.index(b"\n",start)
        decl=add("declaration",{"package_id":package,"source_object_id":obj,"kind":"alias_type","start_byte":start,"end_byte":end,"ordinal":declaration_ordinals[start],"name":name,"receiver_type_id":None})
        aliases.append({"declaration_id":decl,"target_type_id":ids[target]})
    for n in (0,1,3,256):
        key=f"fixture-array-{n}";sid=add("source_type",{"kind":"array","length":n,"element":ids["bool"]});ids[key]=sid;rows[key]={"id":sid,"kind":"array","length":n,"element":ids["bool"]}
    for n in (0,64):
        key=f"fixture-struct-{n}";fields=[{"name":"F"+str(i),"unexported_package_id":None,"type_id":ids["bool"],"tag_bytes_hex":""} for i in range(n)]
        sid=add("source_type",{"kind":"struct","fields":fields});ids[key]=sid;rows[key]={"id":sid,"kind":"struct","fields":[{"key":"field:"+str(i),"type":ids["bool"]} for i in range(n)]}
    cases=sorted(ids[r["id"]] for r in facts["types"] if r.get("implements_error"))
    implementers=[]
    for fn in facts["functions"]:
        if fn["name"]!="Error" or fn["receiver"] is None:continue
        receiver=ids[fn["receiver"]]
        decl=add("declaration",{"package_id":package,"source_object_id":obj,"kind":"method","start_byte":fn["start_byte"],"end_byte":fn["end_byte"],"ordinal":declaration_ordinals[fn["start_byte"]],"name":"Error","receiver_type_id":receiver})
        sig=add("signature",{"declaration_id":decl,"receiver":receiver,"params":[],"results":[ids["string"]]})
        implementers.append({"type_id":receiver,"signature_id":sig})
    universe=source_seal("error_universe",{"source_error_type_id":ids["error"],"cases":cases});nodes.append(universe)
    build_config=add("build_config",{"goos":"linux","goarch":"amd64","goamd64":"v1","compiler_release":"go1.25.0","tags":[]})
    context=linked_contexts()["contexts"]["go_practical"]
    capture=add("capture_context",{"semantic_context_sha256":digest(SHARED["context"][1],context),"compiler_inventory_sha256":sha((HERE/"host-toolchain.json").read_bytes()),"build_config_id":build_config,"objects":sorted([modobj,obj]),"modules":[module],"packages":[package]})
    graph_roots=sorted({r["id"] for r in rows.values()}|{a["declaration_id"] for a in aliases}|{a["signature_id"] for a in implementers}|{universe["id"],capture})
    validate_graph(nodes,graph_roots)
    types=sealed(TYPE_SCHEMA,TYPE_DOMAIN,{"identity_nodes":sorted(nodes,key=lambda n:n["id"]),"definitions":sorted(rows.values(),key=lambda r:r["id"]),"aliases":sorted(aliases,key=lambda a:a["declaration_id"]),"error_implementers":sorted(implementers,key=lambda a:a["type_id"]),"error_universe_id":universe["id"]})
    roots=[{"role":"source","source_type_id":value} for key,value in sorted(ids.items()) if key!="[]error"]
    roots += [{"role":"joint","result_types":r} for r in ([],[ids["int64"]],[ids["fixture/policy.Decision"],ids["error"]],[ids["string"]]*16)]
    return {"types":types,"roots":roots,"ids":ids,"identity_roots":graph_roots,"scope":"retained native type facts and supplied source/type roots; literal declaration locators and synthetic anonymous boundary shapes; no independent capture, root derivation or source proof"}


class Specializer:
    def __init__(self,types,context):
        model=__import__("foundation_limits_vectors").model();model.contexts.validate_context(context)
        need(context["semantic_profile"]=="mpk.go.practical.v1" and context["source_language"]=="go","Go practical specialization never dispatches by language alone")
        need(set(types)=={"schema","identity_nodes","definitions","aliases","error_implementers","error_universe_id","content_sha256"} and types["schema"]==TYPE_SCHEMA,"closed type definition root")
        need(types["content_sha256"]==digest(TYPE_DOMAIN,{k:v for k,v in types.items() if k!="content_sha256"}),"complete type definition hash")
        self.types=types;self.context=context;self.context_hash=digest(SHARED["context"][1],context)
        need([n["id"] for n in types["identity_nodes"]]==sorted({n["id"] for n in types["identity_nodes"]}) and [r["id"] for r in types["definitions"]]==sorted({r["id"] for r in types["definitions"]}),"canonical unique source node/definition order")
        self.nodes={n["id"]:n for n in types["identity_nodes"]};need(len(self.nodes)==len(types["identity_nodes"]),"unique identity nodes")
        for node in self.nodes.values():validate_node(node)
        alias_ids=[]
        for a in types["aliases"]:
            need(type(a) is dict and set(a)=={"declaration_id","target_type_id"},"closed source alias")
            need(a["declaration_id"] in self.nodes and self.nodes[a["declaration_id"]].get("kind")=="alias_type" and a["target_type_id"] in self.nodes and self.nodes[a["target_type_id"]]["schema"]=="mpk.go.source_type.v1","alias preserves a real target source type")
            alias_ids.append(a["declaration_id"])
        need(len(alias_ids)==len(set(alias_ids)),"unique alias declarations")
        need(alias_ids==sorted(alias_ids),"canonical source alias order")
        captures=[n for n in self.nodes.values() if n["schema"]=="mpk.go.capture_context.v1"]
        need(len(captures)==1 and captures[0]["semantic_context_sha256"]==self.context_hash,"complete context binds capture before specialization")
        self.rows={r["id"]:r for r in types["definitions"]};need(len(self.rows)==len(types["definitions"]),"unique complete type definitions")
        need(set(self.rows)=={n["id"] for n in self.nodes.values() if n["schema"]=="mpk.go.source_type.v1"},"one exact definition for every captured source type")
        u=self.nodes[types["error_universe_id"]];need(u["schema"]=="mpk.go.error_universe.v1","one exact closed error universe")
        implementers={}
        for e in types["error_implementers"]:
            need(type(e) is dict and set(e)=={"type_id","signature_id"} and e["type_id"] not in implementers,"closed unique error method signatures")
            sig=self.nodes[e["signature_id"]];decl=self.nodes[sig["declaration_id"]]
            need(sig["schema"]=="mpk.go.signature.v1" and sig["receiver"]==e["type_id"] and sig["params"]==[] and len(sig["results"])==1 and self.nodes[sig["results"][0]].get("name")=="string" and decl.get("kind")=="method" and decl.get("name")=="Error","exact captured Error() string signature; boxing does not call the body")
            implementers[e["type_id"]]=e["signature_id"]
        need(set(u["cases"])<=set(implementers),"every global error payload has its exact Error method signature")
        need(list(implementers)==sorted(implementers),"canonical error implementer order")
        validate_graph(list(self.nodes.values()),sorted(set(self.rows)|set(alias_ids)|set(implementers.values())|{types["error_universe_id"],captures[0]["id"]}))
        self.universe=u;shapes=[]
        for key,r in self.rows.items():
            node=self.nodes[key];kind=r["kind"]
            fields={"named":{"id","kind","underlying"},"bool":{"id","kind"},"word":{"id","kind","width","signed"},"string":{"id","kind"},"error":{"id","kind"},"array":{"id","kind","element","length"},"slice":{"id","kind","element"},"struct":{"id","kind","fields"}}
            need(kind in fields and set(r)==fields[kind],"closed type shape arm")
            if kind=="named":
                need(node["kind"]=="nominal","nominal source identity")
                need(r["underlying"] in self.rows and self.rows[r["underlying"]]["kind"]!="error","predeclared error/aliases only; no named interface substitution")
            elif kind in {"array","slice"}:
                need(node["kind"]==kind and node["element"]==r["element"] and (kind!="array" or type(r["length"]) is int and node["length"]==r["length"]),"source array/slice exact element and length")
            elif kind=="struct":
                need(node["kind"]==kind and [f["type"] for f in r["fields"]]==[f["type_id"] for f in node["fields"]] and all(set(f)=={"key","type"} and f["key"]=="field:"+str(i) for i,f in enumerate(r["fields"])),"ordered complete struct field definitions")
            else:
                need(node["kind"]=="builtin","builtin source identity")
                need((kind=="word" and type(r["width"]) is int and type(r["signed"]) is bool and r["width"]==node["width"] and r["signed"]==node["signed"]) or (kind!="word" and node["name"]==kind),"builtin width/signedness/kind")
            fs=tuple((f["key"],f["type"]) for f in r.get("fields",[]))
            if kind=="error":fs=tuple((x,x) for x in u["cases"])
            shapes.append(Shape(key,kind,r.get("underlying",r.get("element","")),r.get("length",0),fs,r.get("width",64),r.get("signed",True)))
        self.shapes=Shapes(shapes);self.entries={};self.carriers={}

    def dependencies(self,key):
        r=self.rows[key];kind=r["kind"]
        if kind in {"named","array","slice"}:return [r.get("underlying",r.get("element"))]
        if kind=="struct":return [f["type"] for f in r["fields"]]
        if kind=="error":return self.universe["cases"]
        return []

    def source(self,key):
        if key in self.entries:return self.entries[key]
        depth,plain,weight=self.shapes.inspect(key);need(weight<=VALUE_UNITS,"complete per-value carrier domain obligation")
        deps=[self.source(k)["id"] for k in self.dependencies(key)]
        entry=sealed(INSTANCE_SCHEMA,INSTANCE_DOMAIN,{"semantic_context_sha256":self.context_hash,"type_definitions_sha256":self.types["content_sha256"],"global_error_universe_id":self.universe["id"],"role":"source","source_type_id":key,"dependencies":deps,"depth":depth,"plain":plain,"weight_units":weight},"id")
        self.entries[key]=entry;return entry

    def carrier(self,key):
        if key in self.carriers:return self.carriers[key]
        self.source(key);r=self.rows[key];kind=r["kind"]
        if kind=="named":value=self.carrier(r["underlying"])
        elif kind=="bool":value=boolean(),bit(False)
        elif kind=="word":value=word(r["width"])
        elif kind=="string":value=product([word(64),vector(8,*word(8))],[0])
        elif kind=="array":value=(boolean(),bit(False)) if r["length"]==0 else vector((r["length"]-1).bit_length(),*self.carrier(r["element"]))
        elif kind=="slice":
            # Step kind0=NoStep,1=field,2=array index; remaining tags/padding require validity.
            path_cell=product([word(8),word(64)],[0]);path=vector(4,*path_cell)
            value=product([(boolean(),bit(True)),(nat(),const("Std.Nat.zero")),path,word(64),word(64),word(64)],[0])
        elif kind=="struct":value=product([self.carrier(f["type"]) for f in r["fields"]],[0])
        else:value=product([word(64)]+[self.carrier(k) for k in self.universe["cases"]],[0])
        self.carriers[key]=value;return value

    def specialize(self,roots):
        need(type(roots) is list and roots,"explicit private type/result roots")
        need(len({canonical(r) for r in roots})==len(roots),"unique alias-resolved type/result roots")
        root_ids=[];joints={}
        for r in roots:
            need(type(r) is dict and r.get("role") in {"source","joint"},"closed specialization root")
            if r["role"]=="source":
                need(set(r)=={"role","source_type_id"},"closed source root");root_ids.append(self.source(r["source_type_id"])["id"])
            else:
                need(set(r)=={"role","result_types"} and type(r["result_types"]) is list and len(r["result_types"])<=16,"closed ordered result product0..16")
                members=[self.source(k) for k in r["result_types"]];weight=1+sum(m["weight_units"] for m in members);need(weight<=VALUE_UNITS,"joint product domain obligation")
                entry=sealed(INSTANCE_SCHEMA,INSTANCE_DOMAIN,{"semantic_context_sha256":self.context_hash,"type_definitions_sha256":self.types["content_sha256"],"global_error_universe_id":self.universe["id"],"role":"joint","result_types":r["result_types"],"dependencies":[m["id"] for m in members],"depth":1+max([m["depth"] for m in members],default=0),"plain":all(m["plain"] for m in members),"weight_units":weight},"id")
                joints[entry["id"]]=entry;root_ids.append(entry["id"])
        entries=sorted(list(self.entries.values())+list(joints.values()),key=lambda e:e["id"])
        instances=sealed(GO["foundation_instances"][0],GO["foundation_instances"][1],{"semantic_context":self.context,"foundation_ref":self.context["foundation_descriptor"],"limits_sha256":self.context["semantic_parameters"]["value"]["limits_sha256"],"type_definitions_sha256":self.types["content_sha256"],"global_error_universe_id":self.universe["id"],"roots":sorted(root_ids),"entries":entries})
        declarations=[]
        # A deterministic dependency order; source identity and every result order remain in names/instances.
        pending={e["id"]:e for e in entries};done=set()
        while pending:
            ready=sorted(k for k,e in pending.items() if set(e["dependencies"])<=done);need(ready,"acyclic specialization dependencies")
            for k in ready:
                e=pending.pop(k);ty,zero=self.carrier(e["source_type_id"]) if e["role"]=="source" else product([self.carrier(x) for x in e["result_types"]],[0])
                prefix=MODULE+".I"+e["id"]
                declarations += [{"name":prefix+".Carrier","kind":"definition","ty":{"kind":"sort0"},"value":ty},{"name":prefix+".Zero","kind":"definition","ty":ty,"value":zero}]
                declarations.append({"name":prefix+".ZeroRecipe","kind":"theorem","ty":eq(ty,const(prefix+".Zero"),zero),"value":refl(ty,zero)})
                done.add(k)
        # Arbitrary values, not value enumeration: two heterogeneous construction/projection laws.
        for kind,prefix,left in (("string","PairWordString",word(64)[0]),("error","PairBoolRoot",boolean())):
            keys=[k for k in self.entries if self.rows[k]["kind"]==kind]
            if keys:declarations += pair_library(MODULE+"."+prefix,left,self.carrier(keys[0])[0])
        for d in declarations:validate_expr(d["ty"]);validate_expr(d["value"])
        expansion=sealed(GO["foundation_expansion"][0],GO["foundation_expansion"][1],{"semantic_context_sha256":self.context_hash,"foundation_ref":self.context["foundation_descriptor"],"instances_sha256":instances["content_sha256"],"type_definitions_sha256":self.types["content_sha256"],"seed_sha256":sha(SEED.read_bytes()),"module":MODULE,"declarations":declarations})
        return instances,expansion


def candidate():
    f=fixture();context=linked_contexts()["contexts"]["go_practical"]
    s=Specializer(f["types"],context);instances,expansion=s.specialize(f["roots"])
    return f,instances,expansion


def validate_expansion(types,context,roots,instances,expansion):
    expected=Specializer(types,context).specialize(roots)
    need(canonical(instances)==canonical(expected[0]) and canonical(expansion)==canonical(expected[1]),"field-complete derived instances/ordinary expansion, not a caller-provided equation/allowlist")
    return True


def schemas():
    f,i,e=candidate();roots=[]
    types={"type_definitions":{"schema":"literal type_definitions.v1","identity_nodes":"id-sorted exact source identity DAG with one capture/context","definitions":"id-sorted complete closed source type definition union","aliases":"declaration-id sorted exact target refs","error_implementers":"type-id sorted exact Error() string signature refs","error_universe_id":"same captured closed error universe id","content_sha256":"lowercase sha256"},"instances":{"schema":"literal closed_instances.v1","semantic_context":"full validated Go practical context.v3","foundation_ref":"context-equal exact compiled reference","limits_sha256":"context-equal exact compiled limits hash","type_definitions_sha256":"complete input type definition hash","global_error_universe_id":"same captured U","roots":"sorted unique derived instance ids","entries":"id-sorted complete dependency closure of source/joint union","content_sha256":"lowercase sha256"},"expansion":{"schema":"literal foundation_expansion.v1","semantic_context_sha256":"complete typed context hash","foundation_ref":"context-equal exact compiled reference","instances_sha256":"complete closed instance set hash","type_definitions_sha256":"complete input type definition hash","seed_sha256":"exact retained zero-axiom core seed bytes","module":"fixed private module","declarations":"complete dependency-ordered ordinary definitions/theorems with closed AST","content_sha256":"lowercase sha256"}}
    for name,node,domain in (("type_definitions",f["types"],TYPE_DOMAIN),("instances",i,GO["foundation_instances"][1]),("expansion",e,GO["foundation_expansion"][1])):
        roots.append({"name":name,"schema":node["schema"],"required_fields":list(node),"field_types":types[name],"optional_fields":[],"unknown_fields":"reject","hash_domain":domain,"hash_field":"content_sha256","hash_preimage_fields":[k for k in node if k!="content_sha256"]})
    return {"schema":"mpk.go_practical.closed_carriers_draft.v0","roots":roots,"new_identities":[{"schema":TYPE_SCHEMA,"domain":TYPE_DOMAIN},{"schema":INSTANCE_SCHEMA,"domain":INSTANCE_DOMAIN}],"instance_union":{"source":["schema","semantic_context_sha256","type_definitions_sha256","global_error_universe_id","role","source_type_id","dependencies","depth","plain","weight_units","id"],"joint":["schema","semantic_context_sha256","type_definitions_sha256","global_error_universe_id","role","result_types","dependencies","depth","plain","weight_units","id"]},"ordinary_ast":{"sort0":["kind"],"const":["kind","name"],"var":["kind","name"],"app":["kind","function","arguments"],"pi":["kind","name","domain","body"],"lam":["kind","name","domain","body"]},"status":"closed specialization/ordinary-term draft; not source/finite-id/validity/operation foundation completion","remaining":["full independent source/capture/root/alias/nominal/underlying and error-U completeness proofs; finite identity/zero-access normalization and validity predicates","all operation/contract/control/source-binding laws and downstream schema/migration review; W05/T01 gate and complete T02–T11"]}
