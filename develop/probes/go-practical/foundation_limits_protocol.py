"""Closed private limits/definition-inventory/descriptor packaging, no activation."""
import copy
import json
from pathlib import Path
from context_protocol import Protocol as ContextProtocol, registration_fixture, registry_ref
from context_vectors import contexts, rehash_registry
from identity_catalog import GO
from identity_wire import canonical, digest, parse, path as logical_path
from structural_limit_catalog import generate as counter_catalog
from run_proof_pilot import ROOT, HERE, json_bytes, sha
import termination_foundation_build as core_build

RECORD=ROOT/"develop/migrations/go-practical/t01-w04/foundation-limits"
RULES=RECORD/"semantic-rules.md"
OP_TABLE=ROOT/"develop/migrations/go-practical/t01-w04/operation-table.json"
FAILURES={"source_capture":"capture_rejected","structural_artifact":"model_rejected_limit","capture_manifest":"model_rejected_limit","semantic_domain":"domain_obligation_unproved","carrier_accounting":"domain_obligation_unproved","proof_resources":"proof_pending_resource"}


class FoundationError(ValueError):pass


def need(ok,message):
    if not ok:raise FoundationError(message)


def exact(value,fields):need(type(value) is dict and set(value)==set(fields),"closed required fields")


def equal(a,b):return canonical(a)==canonical(b)


def seal(kind,body):
    value={"schema":GO[kind][0],**copy.deepcopy(body)}
    value["content_sha256"]=digest(GO["foundation"][1] if kind=="foundation_descriptor" else GO[kind][1],value)
    return value


def reference(value):return {k:copy.deepcopy(value[k]) for k in ("schema","id","content_sha256")}


def member(path,schema,raw):
    return {"path":path.relative_to(ROOT).as_posix(),"schema":schema,"sha256":sha(raw),"size_bytes":len(raw)}


def counter_unit(row):
    key=row["id"].split(".",1)[1]
    if row["category"]=="carrier_accounting":return "abstract_carrier_units"
    if "bytes" in key:return "bytes"
    if key in {"construction_wall_ns","checker_wall_ns_each"}:return "monotonic_nanoseconds"
    if key=="source_word_bits":return "bits"
    if "nesting" in key:return "depth"
    return "count"


def limits():
    rows=counter_catalog()["counters"]
    need(len(rows)==64 and all(type(r["maximum"]) is int and 0<=r["maximum"]<1<<53 and FAILURES[r["category"]]==r["above"] for r in rows),"exact64 typed numeric/category/outcome candidates")
    counters=[{"id":r["id"],"maximum":r["maximum"],"unit":counter_unit(r),"category":r["category"],"counting_rule":r["counting"],"exceeded":r["above"]} for r in rows]
    counters.sort(key=lambda c:c["id"])
    return seal("limits",{"id":GO["limits"][0],"version":1,"semantic_profile":"mpk.go.practical.v1","counters":counters,"weights":counter_catalog()["weights"]})


def core_interfaces():
    def sources(*names):return [member(HERE/name,None,(HERE/name).read_bytes()) for name in names]
    guards=["generated reserved recursor name","family at Sort0","exact two generated constructors in declaration order","canonical constructor names/types","exact eliminator type","no universe arguments even in partial/neutral uses","scan all prior types/opaque values/theorem proofs","reject later constructors without environment mutation","legacy rec/cases keep their existing rules"]
    return [
        {"name":"Std.Nat.induct","family":"Std.Nat","constructors":["zero : Nat","succ : Nat -> Nat"],"signature":"(P : Nat -> Sort0) -> P zero -> ((n : Nat) -> P n -> P (succ n)) -> (n : Nat) -> P n",
         "iota":["induct P z s zero = z","induct P z s (succ n) = s n (induct P z s n)"],"guards":guards,
         "sources":sources("control-foundation/rust/nat_induct.rs","control-foundation/go/nat_induct.go")},
        {"name":"Std.Bool.recType","family":"Std.Bool","constructors":["false : Bool","true : Bool"],"signature":"Sort0 -> Sort0 -> Bool -> Sort0",
         "iota":["recType A B false = A","recType A B true = B"],"guards":guards,
         "sources":sources("termination-foundation/rust/bool_rec_type.rs","termination-foundation/go/bool_rec_type.go")},
    ]


def definitions(limit):
    table=json.loads(OP_TABLE.read_text());groups={}
    for op in table["operations"]:groups.setdefault(op["component"],[]).append(op)
    mirror=[{"path":p,"sha256":sha(raw)} for p,raw in sorted(core_build.files().items())]
    receipt=ROOT/"develop/migrations/go-practical/t01-w04/nat-termination/observations.json"
    need(mirror==json.loads(receipt.read_text())["mirrored_checker_inputs"],"exact retained joint77-file private core proposal")
    receipts=[ROOT/"develop/migrations/go-practical"/p for p in (
        "t01-w03/completion-audit.json","t01-w04/nat-termination/observations.json","t01-w04/variant-termination/observations.json",
        "t01-w04/storage-views/observations.json","t01-w04/storage-lifetimes/observations.json","t01-w04/structural-limits/observations.json")]
    return seal("foundation_definitions",{
        "id":GO["foundation_definitions"][0],"version":1,"semantic_profile":"mpk.go.practical.v1","representation":"operation_recipe_inventory_draft",
        "limits_ref":reference(limit),"core_interfaces":core_interfaces(),"mirrored_checker_inputs":mirror,
        "operation_families":[{"id":"mpk.go.recipe."+name+".v1","operations":ops} for name,ops in sorted(groups.items())],
        "common_required_members":table["common_required_members"],"operation_table":member(OP_TABLE,table["schema"],OP_TABLE.read_bytes()),
        "evidence_receipts":[member(p,json.loads(p.read_text())["schema"],p.read_bytes()) for p in sorted(receipts)],
        "source_callable_members":[],"caller_extension_points":[],
    })


def package():
    limit=limits();defs=definitions(limit)
    members=[member(RECORD/"limits.json",limit["schema"],json_bytes(limit)),member(RECORD/"definitions.json",defs["schema"],json_bytes(defs)),member(RULES,None,RULES.read_bytes())]
    descriptor=seal("foundation_descriptor",{"id":GO["foundation"][0],"version":1,"semantic_profile":"mpk.go.practical.v1","activation":"candidate_only","limits_ref":reference(limit),"definitions_ref":reference(defs),"members":sorted(members,key=lambda m:m["path"]),"source_callable_members":[],"caller_extension_points":[]})
    return {"limits":limit,"definitions":defs,"descriptor":descriptor}


def contextual_fixture(data=None):
    data=package() if data is None else data;fixture=registration_fixture();registry=fixture["registry"]
    entry=next(e for e in registry["profiles"] if e["semantic_profile"]=="mpk.go.practical.v1")
    entry["foundation_descriptor"]=reference(data["descriptor"])
    fixture["go_parameters"]["limits_sha256"]=data["limits"]["content_sha256"]
    fixture["registry"]=rehash_registry(registry)
    return fixture


class FoundationProtocol:
    def __init__(self,legacy_policy):
        self.compiled=package();fixture=contextual_fixture(self.compiled)
        self.contexts=ContextProtocol(legacy_policy);self.contexts.fixture=fixture;self.contexts.compiled=fixture["registry"]

    def validate_record(self,kind,value):
        fixed=self.compiled[kind]
        exact(value,fixed)
        domain=GO["foundation"][1] if kind=="descriptor" else GO["limits" if kind=="limits" else "foundation_definitions"][1]
        need(value["content_sha256"]==digest(domain,{k:v for k,v in value.items() if k!="content_sha256"}),"complete typed record hash")
        need(equal(value,fixed),"compiled exact limits/recipes/core/descriptor; rehashing caller changes grants no authority")
        return value

    def validate_members(self,descriptor,provided=None):
        self.validate_record("descriptor",descriptor)
        if provided is not None:need(type(provided) is dict and set(provided)=={m["path"] for m in descriptor["members"]},"complete exact raw member set")
        for m in descriptor["members"]:
            logical_path(m["path"])
            if provided is None:
                file=ROOT
                for part in Path(m["path"]).parts:
                    file=file/part;need(not file.is_symlink(),"no symlink in registered member capability")
                raw=file.read_bytes()
            else:raw=provided[m["path"]]
            need(type(raw) is bytes and len(raw)==m["size_bytes"] and sha(raw)==m["sha256"],"complete original raw bytes/size, including file LF")
            if m["schema"] is not None:need(parse(raw)["schema"]==m["schema"] and json_bytes(parse(raw))==raw,"exact canonical JSON member transport")
        return True

    def validate_package(self,data,provided=None):
        exact(data,{"limits","definitions","descriptor"})
        for kind,value in data.items():self.validate_record(kind,value)
        need(equal(data["descriptor"]["limits_ref"],reference(data["limits"])) and equal(data["descriptor"]["definitions_ref"],reference(data["definitions"])) and equal(data["definitions"]["limits_ref"],reference(data["limits"])),"acyclic exact limits/definitions/descriptor refs")
        return self.validate_members(data["descriptor"],provided)

    def contextual_cases(self):
        fixture=self.contexts.fixture;out=contexts()
        for name,c in out.items():
            c["profile_registry"]=registry_ref(fixture["registry"])
            e=next(e for e in fixture["registry"]["profiles"] if e["semantic_profile"]==c["semantic_profile"])
            c["profile_entry_sha256"]=e["entry_sha256"];c["foundation_descriptor"]=copy.deepcopy(e["foundation_descriptor"])
            if name=="go_practical":c["semantic_parameters"]["value"]=copy.deepcopy(fixture["go_parameters"])
        return out


def schema_draft():
    roots=[]
    types={
        "limits":{"schema":"literal limits.v1","id":"literal limits.v1","version":"uint exact1","semantic_profile":"literal go.practical.v1","counters":"id-sorted exact64 counter records","weights":"exact compiled named weight equations","content_sha256":"lowercase sha256"},
        "definitions":{"schema":"literal foundation_definitions.v1","id":"literal foundation_definitions.v1","version":"uint exact1","semantic_profile":"literal go.practical.v1","representation":"literal operation_recipe_inventory_draft","limits_ref":"exact compiled reference","core_interfaces":"ordered exact2 core_interface records","mirrored_checker_inputs":"path-sorted exact77 path/hash records","operation_families":"ordered exact8 operation_family records covering56 operations","common_required_members":"ordered exact6 proof member descriptions","operation_table":"exact raw member","evidence_receipts":"path-sorted exact6 raw members","source_callable_members":"empty array","caller_extension_points":"empty array","content_sha256":"lowercase sha256"},
        "descriptor":{"schema":"literal foundation_descriptor.v1","id":"literal go.practical.foundation.v1","version":"uint exact1","semantic_profile":"literal go.practical.v1","activation":"literal candidate_only","limits_ref":"exact compiled reference","definitions_ref":"exact compiled reference","members":"path-sorted exact3 raw members","source_callable_members":"empty array","caller_extension_points":"empty array","content_sha256":"lowercase sha256"},
    }
    for kind,node in package().items():
        domain=GO["foundation"][1] if kind=="descriptor" else GO["limits" if kind=="limits" else "foundation_definitions"][1]
        roots.append({"name":kind,"schema":node["schema"],"required_fields":list(node),"optional_fields":[],"field_types":types[kind],"unknown_fields":"reject","hash_field":"content_sha256","hash_domain":domain,"hash_preimage_fields":[k for k in node if k!="content_sha256"]})
    return {"schema":"mpk.go_practical.foundation_limits_protocol_draft.v0","roots":roots,"nested":{"reference":["schema","id","content_sha256"],"member":["path","schema","sha256","size_bytes"],"counter":["id","maximum","unit","category","counting_rule","exceeded"],"core_interface":["name","family","constructors","signature","iota","guards","sources"],"operation_family":["id","operations"]},"status":"three private closed packaging roots; operation recipes are not complete closed kernel definitions or source proofs","remaining":["complete ordinary carrier/operation/contract specialization/expansion/binding schemas and all source obligations","production joint core integration and four-profile migration/preservation, complete foundation bundle review/freeze, W05/T01 gate and T02–T11"]}
