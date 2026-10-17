"""Reconstructible complete inputs and adversarial mutations for foundation packaging."""
import copy
from foundation_limits_protocol import package, FoundationProtocol
from context_protocol import raw_request
from context_vectors import contexts, selections, assign, remove
from identity_catalog import GO, SHARED
from identity_wire import canonical, digest
from run_context_protocol import RECORD as PRIOR, policy_from_outputs
from context_vectors import legacy_queries
from check_scope_report import strict_json


def model():
    outputs=strict_json((PRIOR/"legacy-observations.json").read_bytes())
    return FoundationProtocol(policy_from_outputs(legacy_queries(),outputs))


def linked_contexts():
    p=model();cs=p.contextual_cases()
    return {"registry":p.contexts.compiled,"contexts":cs,"requests":{name:raw_request(c,selections()[name]) for name,c in cs.items()}}


def cases():
    data=package();rows=[]
    def add(name,kind,action="same",path=(),value=None,accepted=False,rehash=False):
        rows.append({"id":name,"kind":kind,"action":action,"path":list(path),"value":value,"rehash":rehash,"expected_accepted":accepted})
    for kind,node in data.items():
        add(kind+"-compiled",kind,accepted=True)
        for field in node:add(kind+"-missing-"+field,kind,"remove",[field])
        add(kind+"-extra",kind,"set",["caller_authorized"],True,rehash=True)
        add(kind+"-bool-version",kind,"set",["version"],True,rehash=True)
        add(kind+"-wrong-own-hash",kind,"set",["content_sha256"],"0"*64)
    for name,path,value in (
        ("max-override",["counters",0,"maximum"],999999999),
        ("unit-override",["counters",0,"unit"],"characters"),
        ("outcome-override",["counters",0,"exceeded"],"accepted"),
        ("negative",["counters",0,"maximum"],-1),
        ("bool-maximum",["counters",0,"maximum"],True),
        ("lighter-string",["weights","byte_string"],1),
        ("lighter-error",["weights","closed_error"],"only_active_payload"),
        ("unknown-counter",["counters",0,"id"],"caller.counter"),
    ):add("limits-"+name,"limits","set",path,value,rehash=True)
    add("limits-duplicate-counter","limits","set",["counters"],data["limits"]["counters"]+[data["limits"]["counters"][0]],rehash=True)
    add("limits-missing-counter","limits","set",["counters"],data["limits"]["counters"][:-1],rehash=True)
    add("limits-order","limits","set",["counters"],list(reversed(data["limits"]["counters"])),rehash=True)
    for name,path,value in (
        ("basis",["limits_ref","content_sha256"],"0"*64),
        ("universe",["core_interfaces",0,"signature"],"Nat -> Sort1"),
        ("equation",["core_interfaces",0,"iota",1],"induct P z s (succ n) = z"),
        ("type-selection",["core_interfaces",1,"signature"],"Bool -> Bool"),
        ("guard",["core_interfaces",1,"guards"],[]),
        ("source-hash",["core_interfaces",0,"sources",0,"sha256"],"0"*64),
        ("mirror-hash",["mirrored_checker_inputs",0,"sha256"],"0"*64),
        ("callable",["source_callable_members"],["caller.Helper"]),
        ("extensions",["caller_extension_points"],["caller.recipe"]),
        ("operation",["operation_families",0,"operations",0,"result_relation"],"caller says true"),
        ("common-proof",["common_required_members"],[]),
    ):add("definitions-"+name,"definitions","set",path,value,rehash=True)
    for name,path,value in (
        ("limits-ref",["limits_ref","content_sha256"],"0"*64),
        ("definitions-ref",["definitions_ref","content_sha256"],"0"*64),
        ("active-label",["activation"],"installed"),
        ("caller-bundle",["caller_extension_points"],["caller"]),
        ("member-path",["members",0,"path"],"../../outside.json"),
        ("member-size",["members",0,"size_bytes"],0),
        ("member-sha",["members",0,"sha256"],"0"*64),
    ):add("descriptor-"+name,"descriptor","set",path,value,rehash=True)
    add("package-compiled","package",accepted=True)
    for action in ("missing","extra","changed-byte","no-final-lf","extra-final-lf","null-bytes"):
        add("raw-members-"+action,"members",action)
    for name in contexts():
        add("new-context-"+name,"context",value=name,accepted=True)
        add("new-request-"+name,"request",value=name,accepted=True)
        add("old-private-context-"+name,"old_context",value=name)
    add("context-old-limits","context","set",["semantic_parameters","value","limits_sha256"],contexts()["go_practical"]["semantic_parameters"]["value"]["limits_sha256"])
    add("context-old-foundation","context","set",["foundation_descriptor"],contexts()["go_practical"]["foundation_descriptor"])
    add("context-caller-override","context","set",["semantic_parameters","value","limits"],{})
    assert len(rows)==len({r["id"] for r in rows})
    return rows


def apply_row(row,p):
    kind=row["kind"];data=p.compiled
    if kind in data:
        value=copy.deepcopy(data[kind])
        if row["action"]=="set":value=assign(value,row["path"],row["value"])
        elif row["action"]=="remove":value=remove(value,row["path"])
        if row["rehash"]:
            domain=GO["foundation"][1] if kind=="descriptor" else GO["limits" if kind=="limits" else "foundation_definitions"][1]
            value["content_sha256"]=digest(domain,{k:v for k,v in value.items() if k!="content_sha256"})
        return p.validate_record(kind,value)
    if kind in {"package","members"}:
        from foundation_limits_protocol import ROOT
        provided={m["path"]:(ROOT/m["path"]).read_bytes() for m in data["descriptor"]["members"]}
        if kind=="members":
            key=next(k for k in provided if k.endswith("limits.json"));action=row["action"]
            if action=="missing":del provided[key]
            elif action=="extra":provided["caller.json"]=b"{}\n"
            elif action=="changed-byte":provided[key]=provided[key].replace(b'"version": 1',b'"version": 2')
            elif action=="no-final-lf":provided[key]=provided[key][:-1]
            elif action=="extra-final-lf":provided[key]+=b"\n"
            elif action=="null-bytes":provided[key]=None
        return p.validate_package(data,provided)
    if kind=="old_context":return p.contexts.validate_context(contexts()[row["value"]])
    cs=p.contextual_cases()
    if kind=="request":return p.contexts.validate_request(raw_request(cs[row["value"]],selections()[row["value"]]))
    value=cs[row["value"]] if row["action"]=="same" else assign(cs["go_practical"],row["path"],row["value"])
    return p.contexts.validate_context(value)


def hash_vectors():
    data=package();linked=linked_contexts();rows=[]
    def add(name,domain,node,omit=None):
        body={k:v for k,v in node.items() if k!=omit};raw=canonical(body)
        rows.append({"id":name,"domain":domain,"raw_json_hex":raw.hex(),"canonical_hex":raw.hex(),"digest":digest(domain,body),"expected_accepted":True})
    for kind,node in data.items():add(kind,GO["foundation"][1] if kind=="descriptor" else GO["limits" if kind=="limits" else "foundation_definitions"][1],node,"content_sha256")
    add("registry",SHARED["registry"][1],linked["registry"],"registry_sha256")
    for i,entry in enumerate(linked["registry"]["profiles"]):add("entry-"+str(i),SHARED["registry_entry"][1],entry,"entry_sha256")
    for name,c in linked["contexts"].items():add("context-"+name,SHARED["context"][1],c)
    for name,r in linked["requests"].items():add("request-"+name,SHARED["validated_request"][1],r,"request_sha256")
    return rows
