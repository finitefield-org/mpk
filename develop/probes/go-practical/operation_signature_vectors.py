"""All56 concrete signature candidates and adversarial signature/member edits."""
import copy
from contract_scope import Scope
from contract_wire_vectors import source_cases
from operation_signatures import Signatures, PARAMETERS, SCHEMA, DOMAIN, COMMON, SYNTAX_MAX, canonical, digest


def model():
    row = next(r for r in source_cases() if r["id"] == "missing-invariants-partial")
    return Signatures(Scope(row["raw"], row["universe"]))


def enclosing(m, ordinal=0):
    loop = next(l for l in m.scope.function["loops"] if l["ordinal"] == ordinal)
    return {"kind": "loop", "source_object_id": m.scope.source_id,
            "start_byte": loop["start_byte"], "end_byte": loop["end_byte"], "ordinal": ordinal}


def parameters(m):
    t = m.id
    a,b,w,s,arr,sl,bs,line,flag = (t(k) for k in ("fixture/policy.Amount", "bool", "uint64", "string", "fixture/policy.LineArray", "[]fixture/policy.Line", "[]uint8", "fixture/policy.Line", "fixture/policy.Flag"))
    i = t("int"); code = t("fixture/policy.RejectCode")
    callee = next(d for d,sig in m.callees.items() if sig["receiver"] == code)
    return {
        1:{"type":a}, 2:{"type":w,"value":"18446744073709551615"}, 3:{"from":a,"to":t("uint8")},
        4:{"type":a,"operator":"negate"},5:{"type":a,"operator":"add"},6:{"type":a,"operator":"divide"},
        7:{"type":a,"count_type":w,"operator":"left"},8:{"type":flag,"operator":"and"},
        9:{"type":a,"operator":"less"},10:{"type":arr,"initializers":[1]},11:{"type":line,"field":0},
        12:{"type":arr},13:{"type":arr,"index_type":w},14:{"type":arr,"operator":"equal"},
        15:{"condition_type":flag},16:{"type":a,"case_arities":[2,1],"default_index":0},
        17:{"types":[a,m.id("error")]},18:{"callee":callee,"index":0},19:{"callee":callee},20:{},
        21:{"targets":[a,None],"types":[a,m.id("error")]},22:{"type":a,"rhs_type":w,"operator":"left"},
        23:{},24:{"type":code},25:{},26:{},27:{"type":s,"bytes_hex":"00ff"},
        28:{"type":s,"operator":"less"},29:{"type":s},30:{"type":s,"index_type":w},
        31:{"type":s,"lo_type":i,"hi_type":w},32:{"type":s},33:{},34:{},
        35:{"type":sl},36:{"type":sl},37:{"type":sl,"size":"capacity"},38:{"type":sl},
        39:{"type":sl,"index_type":w},40:{"type":sl,"lo_type":i,"hi_type":w},
        41:{"type":sl,"lo_type":i,"hi_type":w,"max_type":w},
        42:{"type":arr,"slice_type":sl,"lo_type":i,"hi_type":w,"max_type":None},
        43:{"type":sl,"len_type":i,"cap_type":w},44:{"type":sl,"initializers":[1,0]},
        45:{"type":sl,"index_type":w},46:{"destination":sl,"source":sl},47:{"type":bs},
        48:{"type":sl,"count":2},49:{"destination":sl,"source":sl},50:{"type":bs},
        51:{"loop":0,"condition_type":flag,"has_test":True},
        52:{"loop":2,"type":arr,"index":True,"value":True,"evaluation":"evaluate"},
        53:{"loop":2,"type":sl,"index":True,"value":True},54:{"enclosing":[enclosing(m)]},
        55:{"enclosing":[enclosing(m)]},56:{"type":arr,"size":"length"}}


def definitions(m): return [m.derive("GP-OP-"+str(n).zfill(2),p) for n,p in sorted(parameters(m).items())]


def reseal(value):
    value["operation_hash"] = digest(DOMAIN,{k:v for k,v in value.items() if k!="operation_hash"})
    return value


def mutate(row, kind):
    row = copy.deepcopy(row)
    if kind == "extra_parameter": row["parameters"]["trusted"] = True
    elif kind == "alter_signature": row["inputs"].append({"kind":"state"})
    elif kind == "omit_obligation": row["obligations"].pop(0)
    else: raise ValueError("unknown vector mutation")
    return reseal(row)


def vectors():
    return [{"id":"GP-OP-"+str(n).zfill(2)+"-"+kind,"op":"GP-OP-"+str(n).zfill(2),"mutation":kind,
             "expected":"typed_unproved" if kind=="none" else "configuration_rejection"}
            for n in range(1,57) for kind in ("none","extra_parameter","alter_signature","omit_obligation")]


def catalogue(m):
    rows = []
    for n, fields in sorted(PARAMETERS.items()):
        op = "GP-OP-"+str(n).zfill(2); old = m.operations[op]
        field_types = {key:"captured_source_type_id" for key in fields}
        field_types.update({k:v for k,v in {
            "operator":"closed operation-specific subtag", "size":"length|capacity", "value":"type-directed Bool|canonical signed decimal|lowercase bytes hex",
            "initializers":"ordered unique array<uint>, bounded by exact fields/length/storage", "field":"uint exact nonblank field ordinal",
            "case_arities":"ordered positive uint groups,total within complete source syntax budget", "default_index":"required nullable uint source clause position",
            "types":"ordered source_type_id array; pack0..16, assignment within source syntax budget",
            "targets":"ordered nullable source_type_id array,exact assignment arity", "callee":"captured function/method declaration id",
            "index":"uint result projection ordinal" if n==18 else "Bool effective range index",
            "bytes_hex":"lowercase even hex,0..256 bytes", "count":"uint within complete source syntax budget",
            "loop":"uint exact captured loop ordinal", "has_test":"Bool", "evaluation":"evaluate|constant_length",
            "enclosing":"nonempty outer-to-inner control_target records"}.items() if k in fields})
        if n in {52,53}:field_types["value"]="Bool effective nonblank second range variable"
        if n==42:field_types["max_type"]="required nullable captured integer type id; null selects two-index view"
        rows.append({"op":op,"parameter_required_fields":list(fields),"parameter_field_types":field_types,"optional_fields":[],"unknown_fields":"reject",
                     "source_signature":old["signature"],"evaluation_rule":old["evaluation_rule"],
                     "producer":old["primary_production_executor"],"consumer":old["independent_validator"],
                     "vc_owner":old["vc_owner"],"test_owner":old["primary_test_owner"],
                     "ordinary_members":list(COMMON),"source_model_derived":False,"ordinary_operation_proofs_checked":False})
    example = definitions(m)[0]
    root_types={"schema":"literal mpk.go.operation_definition.v1","semantic_context":"full validated Go practical semantic_context.v3",
                "capture_context_id":"scope-equal capture id","source_scope_sha256":"complete contract_scope hash",
                "function_decl_id":"scope-equal function declaration id","signature_id":"scope-equal signature id",
                "type_definitions_sha256":"scope-equal complete type-definition hash","error_universe_id":"same complete global-U id",
                "operation_table_sha256":"exact original logical operation-table raw hash","op":"one of the exact56 GP-OP tags",
                "parameters":"closed op-directed concrete parameter record","inputs":"ordered concrete operation_type array",
                "outputs":"ordered concrete operation_type array","regions":"ordered region interface records",
                "obligations":"complete derived rule-member records,not checked terms","binding_requirements":"closed op-directed binding requirement records",
                "operation_hash":"lowercase sha256"}
    assert set(root_types)==set(example)
    return {"schema":"mpk.go_practical.operation_signature_draft.v0","operation_schema":SCHEMA,"hash_domain":DOMAIN,
            "root_required_fields":list(example),"root_field_types":root_types,"optional_fields":[],"unknown_fields":"reject",
            "hash_field":"operation_hash","hash_preimage_fields":[k for k in example if k!="operation_hash"],
            "type_union":{"source":["kind","source_type_id"],"location":["kind","source_type_id"],
                          "joint":["kind","result_types"],"values":["kind","types"],"state":["kind"],"control":["kind","result_types"]},
            "region_required_fields":["role","inputs","outputs"],
            "obligation_required_fields":["category","ordinal","predicate"],
            "control_target_required_fields":["kind","source_object_id","start_byte","end_byte","ordinal"],
            "control_target_kinds":["loop","switch"], "complete_source_syntax_ceiling":SYNTAX_MAX,
            "binding_requirement_arms":{"static_callee":["function_decl_id","signature_id"],
                                        "error_target":["case_type_id","signature_id","function_decl_id"],
                                        "adapter":["symbol","compiler_release","compiler_inventory_sha256","sources"],
                                        "control":["target"]},
            "adapter_source_fields":["path","raw_sha256","bytes"],
            "obligation_categories":["ordinary","runtime_safety","profile_domain","permission"],
            "predicate_boundary":"closed6 ordinary member names plus exact indexed operation-table statements; implementations must construct/check ordinary terms,not treat these strings as proofs",
            "operations":rows,
            "remaining":["actual independently reconstructed source/callee/adapter/control/SSA/region bindings and all ordinary operation/definedness/value/effect/frame proofs",
                         "complete shared VIR/VC/source-map/manifest/API/release roots and consumer/migration review; W05/T01 gate and full T02–T11"]}
