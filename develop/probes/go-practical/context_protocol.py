"""Private closed registry/context/request draft; no installed registration."""
import copy
import json
import unicodedata
from pathlib import Path
from identity_wire import canonical,digest
from identity_catalog import SHARED,GO
from structural_limit_catalog import generate as limits_catalog

ROOT=Path(__file__).resolve().parents[3]
ACTIVE_VECTOR=ROOT/"develop/specs/vectors/semantic-profile-registry-v4.json"
GO_PROFILE="mpk.go.practical.v1"
CONTRACTS=("ai","evidence","frontend","manifest","policy","release","source_map","vc","vir")


class ContextError(ValueError):pass


def need(ok,message):
    if not ok:raise ContextError(message)


def exact(value,fields):need(type(value) is dict and set(value)==set(fields),"closed required fields")


def same(a,b):return canonical(a)==canonical(b)


def sealed(kind,fields,hash_field):
    node={"schema":SHARED[kind][0],**copy.deepcopy(fields)}
    node[hash_field]=digest(SHARED[kind][1],node)
    return node


def active_registry():return json.loads(ACTIVE_VECTOR.read_text())["registry"]


def registration_fixture():
    """Compiled private fixture metadata, not a complete Go foundation bundle."""
    rules=limits_catalog()
    limits_value={"schema":"mpk.go_practical.numeric_registration_fixture.v0","counters":{r["id"]:r["maximum"] for r in rules["counters"]},"weights":rules["weights"]}
    limits_hash=digest(GO["limits"][1],limits_value)
    foundation_data={"schema":"mpk.go_practical.foundation_registration_fixture.v0","semantic_profile":GO_PROFILE,"limits_sha256":limits_hash,
                     "required_core_names":["Std.Nat.induct","Std.Bool.recType"],"complete_definition_bundle":False}
    foundation={"schema":GO["foundation_descriptor"][0],"id":GO["foundation"][0],"content_sha256":digest(GO["foundation"][1],foundation_data)}
    params={"target_id":"linux/amd64","pointer_width":64,"goos":"linux","goarch":"amd64","goamd64":"v1","compiler_release":"go1.25.0",
            "effective_languages":["go1.22","go1.23","go1.24","go1.25"],"limits_id":GO["limits"][0],"limits_sha256":limits_hash}
    entries=[]
    for old in active_registry()["profiles"]:
        suffix=old["semantic_profile"].removeprefix("mpk.").removesuffix(".v0").replace(".","_")
        entries.append(sealed("registry_entry",{"source_language":old["source_language"],"semantic_profile":old["semantic_profile"],"semantic_parameters_schema":old["semantic_parameters_schema"],"selection_schema":old["selection_schema"],"foundation_descriptor":None,
                                             "contracts":{k:f"mpk.profile.{k}.{suffix}.v2" for k in CONTRACTS}},"entry_sha256"))
    entries.append(sealed("registry_entry",{"source_language":"go","semantic_profile":GO_PROFILE,"semantic_parameters_schema":GO["parameters"][0],"selection_schema":GO["selection"][0],"foundation_descriptor":foundation,
                                         "contracts":{k:f"mpk.profile.{k}.go_practical.v2" for k in CONTRACTS}},"entry_sha256"))
    entries.sort(key=lambda e:(e["source_language"],e["semantic_profile"]))
    registry=sealed("registry",{"id":SHARED["registry"][0],"revision":1,"profiles":entries},"registry_sha256")
    return {"registry":registry,"go_parameters":params,"foundation_fixture":foundation_data,"limits_fixture":limits_value,
            "scope":"private compiled registration fixture; not registered foundation definitions, profile preservation proof or installed registry"}


def registry_ref(registry):return {k:copy.deepcopy(registry[k]) for k in ("schema","id","revision","registry_sha256")}


def raw_context(profile,parameters):
    registry=registration_fixture()["registry"];entry=next(e for e in registry["profiles"] if e["semantic_profile"]==profile)
    return {"schema":SHARED["context"][0],"profile_registry":registry_ref(registry),"profile_entry_sha256":entry["entry_sha256"],"source_language":entry["source_language"],"semantic_profile":profile,
            "semantic_parameters":{"schema":entry["semantic_parameters_schema"],"value":copy.deepcopy(parameters)},"foundation_descriptor":copy.deepcopy(entry["foundation_descriptor"])}


def raw_request(context,selection):return sealed("validated_request",{"semantic_context":context,"selection":selection},"request_sha256")


def legacy_projection(context,selection=None):
    """Project only into the actual old compiled validator, never accept it as v3."""
    old=active_registry();profile=context.get("semantic_profile")
    entries=[e for e in old["profiles"] if e["semantic_profile"]==profile]
    need(len(entries)==1,"retained compiled profile")
    e=entries[0]
    out={"profile_registry":registry_ref(old),"profile_entry_sha256":e["entry_sha256"],"source_language":context.get("source_language"),"semantic_profile":profile,"semantic_parameters":copy.deepcopy(context.get("semantic_parameters"))}
    return {"context":out,"selection":copy.deepcopy(selection)}


def identifier(name):
    need(unicodedata.unidata_version=="15.0.0","fixed Go1.25 Unicode15 identifier tables")
    if type(name) is not str or not name or name=="_" or name in "break default func interface select case defer go map struct chan else goto package switch const fallthrough if range type continue for import return var".split():return False
    def first(c):return c=="_" or unicodedata.category(c).startswith("L")
    return first(name[0]) and all(first(c) or unicodedata.category(c)=="Nd" for c in name[1:])


def go_selection(selection):
    exact(selection,{"schema","value"});need(selection["schema"]==GO["selection"][0],"Go practical selection schema")
    exact(selection["value"],{"roots"});roots=selection["value"]["roots"]
    need(type(roots) is list and 1<=len(roots)<=8192,"explicit selected roots")
    for r in roots:
        exact(r,{"package","function","receiver"})
        need(type(r["package"]) is str and r["package"] and not any(c in r["package"] for c in "\\\x00\n\r") and not r["package"].startswith("/"),"logical package selector")
        need(identifier(r["function"]),"Go function identifier, no keyword/blank selection")
        need(r["function"]!="init" or r["receiver"] is not None,"package init is outside function-entry proof")
        if r["receiver"] is not None:
            exact(r["receiver"],{"package","type_name"})
            need(r["receiver"]["package"]==r["package"] and identifier(r["receiver"]["type_name"]),"local value receiver name/alias; source resolution remains required")
    keys=[canonical(r) for r in roots]
    need(keys==sorted(set(keys)),"unique canonical root selector order; alias-resolved duplication checked after capture")


class Protocol:
    def __init__(self,legacy_policy):
        self.fixture=registration_fixture();self.compiled=self.fixture["registry"];self.legacy_policy=legacy_policy

    def validate_registry(self,registry):
        exact(registry,{"schema","id","revision","profiles","registry_sha256"})
        need(registry["schema"]==SHARED["registry"][0] and registry["id"]==SHARED["registry"][0] and type(registry["revision"]) is int and registry["revision"]==1,"one exact private registry family/revision")
        need(type(registry["profiles"]) is list,"entry collection")
        for e in registry["profiles"]:
            exact(e,{"schema","source_language","semantic_profile","semantic_parameters_schema","selection_schema","foundation_descriptor","contracts","entry_sha256"})
            need(e["schema"]==SHARED["registry_entry"][0],"entry schema")
            exact(e["contracts"],CONTRACTS)
            need(e["entry_sha256"]==digest(SHARED["registry_entry"][1],{k:v for k,v in e.items() if k!="entry_sha256"}),"complete entry hash")
        need(registry["registry_sha256"]==digest(SHARED["registry"][1],{k:v for k,v in registry.items() if k!="registry_sha256"}),"complete registry hash")
        need(same(registry,self.compiled),"compiled exact entries/contracts/foundation; a rehashed caller registry grants no authority")
        return registry

    def validate_context(self,context,registry=None):
        registry=self.validate_registry(self.compiled if registry is None else registry)
        exact(context,{"schema","profile_registry","profile_entry_sha256","source_language","semantic_profile","semantic_parameters","foundation_descriptor"})
        need(context["schema"]==SHARED["context"][0],"context schema")
        need(same(context["profile_registry"],registry_ref(registry)),"field-complete registry reference")
        entries=[e for e in registry["profiles"] if e["entry_sha256"]==context["profile_entry_sha256"]]
        need(len(entries)==1,"registered exact entry hash")
        e=entries[0]
        need(context["source_language"]==e["source_language"] and context["semantic_profile"]==e["semantic_profile"],"profile/language bound to the selected entry, never language-only dispatch")
        exact(context["semantic_parameters"],{"schema","value"})
        need(context["semantic_parameters"]["schema"]==e["semantic_parameters_schema"],"entry parameter schema")
        need(same(context["foundation_descriptor"],e["foundation_descriptor"]),"null for retained profiles; exact compiled Go descriptor reference for practical")
        if e["semantic_profile"]==GO_PROFILE:
            need(same(context["semantic_parameters"]["value"],self.fixture["go_parameters"]),"fixed registered Go parameters/limits; no feature or numeric overrides")
        else:
            need(self.legacy_policy(legacy_projection(context))["context_accepted"],"actual retained compiled parameter policy")
        return context

    def validate_request(self,request,registry=None):
        exact(request,{"schema","semantic_context","selection","request_sha256"})
        need(request["schema"]==SHARED["validated_request"][0],"request schema")
        c=self.validate_context(request["semantic_context"],registry)
        exact(request["selection"],{"schema","value"})
        entry=next(e for e in self.compiled["profiles"] if e["entry_sha256"]==c["profile_entry_sha256"])
        need(request["selection"]["schema"]==entry["selection_schema"],"entry selection schema")
        if c["semantic_profile"]==GO_PROFILE:go_selection(request["selection"])
        else:need(self.legacy_policy(legacy_projection(c,request["selection"]))["selection_accepted"],"actual retained compiled selection policy")
        need(request["request_sha256"]==digest(SHARED["validated_request"][1],{k:v for k,v in request.items() if k!="request_sha256"}),"complete context/selection request preimage")
        return request

    def linkage(self,left,right):
        self.validate_context(left);self.validate_context(right)
        need(same(left,right),"field-complete typed context equality, not hash/projection equality")
        return True


def schemas():
    fields={
        "registry":["schema","id","revision","profiles","registry_sha256"],
        "registry_entry":["schema","source_language","semantic_profile","semantic_parameters_schema","selection_schema","foundation_descriptor","contracts","entry_sha256"],
        "context":["schema","profile_registry","profile_entry_sha256","source_language","semantic_profile","semantic_parameters","foundation_descriptor"],
        "validated_request":["schema","semantic_context","selection","request_sha256"],
        "parameters":["schema","value"],"selection":["schema","value"],
    }
    roots=[]
    types={
        "registry":{"schema":"literal registry.v3","id":"literal registry.v3","revision":"uint exact1","profiles":"ordered exact3 entry.v3 records","registry_sha256":"sha256"},
        "registry_entry":{"schema":"literal entry.v3","source_language":"registered language","semantic_profile":"registered exact profile","semantic_parameters_schema":"registered parameters schema","selection_schema":"registered selection schema","foundation_descriptor":"null or exact registered Go foundation_ref","contracts":"exact nine compiled profile contract IDs","entry_sha256":"sha256"},
        "context":{"schema":"literal context.v3","profile_registry":"registry_ref","profile_entry_sha256":"registered entry sha256","source_language":"entry language","semantic_profile":"entry profile","semantic_parameters":"entry-bound parameters envelope","foundation_descriptor":"entry-equal nullable foundation_ref"},
        "validated_request":{"schema":"literal validated_request.v3","semantic_context":"context.v3","selection":"entry-bound selection envelope","request_sha256":"sha256"},
        "parameters":{"schema":"literal go_practical parameters.v1","value":"exact compiled go_parameter_values"},
        "selection":{"schema":"literal go_functions.v1","value":"exact roots array of go_selector,1..8192 canonical unique order"},
    }
    for name,keys in fields.items():
        table=GO if name in {"parameters","selection"} else SHARED
        hash_field={"registry":"registry_sha256","registry_entry":"entry_sha256","validated_request":"request_sha256"}.get(name)
        roots.append({"name":name,"schema":table[name][0],"ordered_fields":keys,"required_fields":keys,"optional_fields":[],"field_types":types[name],"unknown_fields":"reject","hash_domain":table[name][1],"hash_field":hash_field,"hash_preimage_fields":[k for k in keys if k!=hash_field]})
    return {"schema":"mpk.go_practical.context_protocol_draft.v0","status":"private closed context/registry/request rules; no complete foundation or activation","roots":roots,
            "nested":{"registry_ref":["schema","id","revision","registry_sha256"],"foundation_ref":["schema","id","content_sha256"],"envelope":["schema","value"],"go_selector":["package","function","receiver"],"receiver_selector":["package","type_name"],"go_parameter_values":list(registration_fixture()["go_parameters"])},
            "foundation_rule":"retained two profiles require null; Go practical requires the exact compiled Go reference; no caller bundle/descriptor/allowlist",
            "legacy_rule":"parameter/selection policy delegated to unchanged actual v1 compiled Rust validators; not a hand-copied subset",
            "remaining":["complete registered Go foundation definitions/limits schemas and proofs, compiled profile payloads, independent production import","full source/snapshot/alias resolution, downstream common roots/unions/preimages and reviewed migration/activation/release gates"]}
