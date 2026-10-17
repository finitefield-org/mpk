"""Exact70 expression arms, source preflight and definedness-member inventory."""
from contract_wire import ARITIES,HEAP_OPS,NORMAL_OPS
from contract_scope import GO,SCOPE_SCHEMA,SCOPE_DOMAIN,BINDING_SCHEMA,BINDING_DOMAIN,exact,need
from overlay_contracts import OPERATORS,QUANTIFIERS


def source_fields(op):
    if op in {"bool.literal","nat.literal","int.literal"}:return ["op","value"]
    if op in {"word.literal","bytes.literal"}:return ["op","type","value"]
    if op=="source.zero":return ["op","type"]
    if op in {"input","result"}:return ["op","position"]
    if op in {"local","bound"}:return ["op","name"]
    if op in {"range.index","range.length"}:return ["op"]
    if op=="old":return ["op","value"]
    if op=="word.convert":return ["op","type","value"]
    if op=="field":return ["op","value","field"]
    if op in {"error.is","error.payload"}:return ["op","value","case"]
    if op in QUANTIFIERS:return ["op","name","lower","upper","body"]
    return ["op","args"]


def payload_fields(op):
    if op in {"bool.literal","nat.literal","int.literal","word.literal","bytes.literal"}:return ["value"]
    if op=="source.zero":return []
    if op in {"input","result"}:return ["position","binding_id","namespace"]
    if op=="binding":return ["binding_id","loop_ref_id","namespace"]
    if op=="bound":return ["depth"]
    if op in {"range.index","range.length"}:return ["loop_ref_id","namespace"]
    if op=="old":return ["namespace","value"]
    if op=="word.convert":return ["value"]
    if op=="field":return ["value","owner_type_id","field_ordinal"]
    if op in {"error.is","error.payload"}:return ["value","case_type_id"]
    if op in QUANTIFIERS:return ["lower","upper","body"]
    return ["args","namespace"] if op in HEAP_OPS else ["args"]


def preflight(expressions):
    """Complete shape pass precedes numeric limits, then actual semantic typing."""
    stack=[(e,0) for e in expressions];count=0;max_depth=0
    while stack:
        e,depth=stack.pop();need(type(e) is dict and type(e.get("op")) is str and e["op"] in OPERATORS,"closed70 source expression heads")
        op=e["op"];exact(e,source_fields(op));count+=1;max_depth=max(max_depth,depth)
        if "args" in e:
            need(type(e["args"]) is list and len(e["args"])==ARITIES[op],"exact source expression arity")
            stack.extend((c,depth+1) for c in e["args"])
        for key in ("value","lower","upper","body"):
            if key in e and type(e[key]) is dict:stack.append((e[key],depth+1))
    need(count<=1024 and max_depth<=32,"registered per-function expression count/depth")
    return count,max_depth


def generate():
    definitions={
        "nat.from_signed":["nonnegative source word before conversion to Nat"],
        "word.div":["nonzero divisor; Go min/-1 wraps, with no invented overflow panic"],
        "word.rem":["nonzero divisor; Go min%-1 is zero"],
        "word.shl":["nonnegative signed count; full unsigned count allowed without a width ceiling"],
        "word.shr":["nonnegative signed count; full unsigned count allowed without masking"],
        "byte_at":["index less than logical byte length"],
        "element":["index less than actual array/slice length","same-snapshot live typed slice owner and permitted observation"],
        "bytes.slice":["ordered endpoints within logical byte length"],
        "bytes.concat":["result byte length within registered256-byte domain; not a Go panic"],
        "slice.view2":["ordered endpoints within capacity","same-snapshot descriptor validity and permission; no copied-list semantics"],
        "slice.view3":["ordered lo<=hi<=max<=capacity","same-snapshot descriptor validity and permission"],
        "error.payload":["exact matching dynamic case in common global U"],
        "sequence.equal":["both actual lengths and all guarded same-snapshot element observations","value comparison only; no slice identity equality"],
    }
    arms=[]
    for op in sorted(NORMAL_OPS):
        arms.append({"op":op,"required_fields":["op","type","payload"],"payload_required_fields":payload_fields(op),"optional_fields":[],"unknown_fields":"reject","args_count":ARITIES.get(op),"definedness_members":definitions.get(op,[]),"proof_boundary":"typing is not definedness/value/source/frame proof; empty member list introduces no axiom"})
    roots={
        "source_overlay":(GO["source_overlay"][0],None,None,{"schema":"literal source_overlay.v1","source":"captured logical source path","function":"closed Go selector with required nullable value receiver","termination":"partial or total","requires":"ordered logical Bool expressions,0..64","ensures":"ordered logical Bool expressions,1..64","error_inputs":"complete typed entry paths and explicit case selectors","loops":"unique exact physical loop selectors with invariants/tagged variants","modifies":"empty array"}),
        "source_scope":(SCOPE_SCHEMA,SCOPE_DOMAIN,"content_sha256",{"schema":"literal contract_scope.v1","semantic_context":"complete validated Go practical context.v3","capture_context_id":"same complete raw-source/overlay capture id","source_overlay_object_id":"exact raw overlay object id","source_object_id":"exact captured Go source object id","function_decl_id":"exact same-package function/value method id","signature_id":"exact receiver/parameter/result signature id","error_universe_id":"one same global-U id","type_definitions":"complete source type definition package","identity_nodes":"complete source/capture/function/signature/loop identity DAG","identity_roots":"complete private supplied identity roots","bindings":"id-sorted closed binding records","inputs":"signature-ordered binding ids,receiver first","results":"signature-ordered result binding ids","headers":"loop-preorder exact headers with sorted live binding ids","content_sha256":"lowercase sha256"}),
        "contract":(GO["normalized_contract"][0],GO["normalized_contract"][1],"contract_hash",{"schema":"literal practical.contract.v1","semantic_context":"scope-equal full typed context","source_scope_sha256":"complete independently reconstructed scope hash","source_overlay_object_id":"scope-equal raw overlay object id","capture_context_id":"scope-equal capture id","function_decl_id":"scope-equal source function id","signature_id":"scope-equal source signature id","termination":"partial or total","requires":"ordered typed logical Bool nodes","ensures":"nonempty ordered typed logical Bool nodes","error_inputs":"complete binding/typed-path/sorted case-id records","modifies":"empty array","loops":"source-preorder complete loop refs/invariants/tagged variants","entry_storage":"fixed read_only/forbidden external mutation conditions","pending_candidates":"derived missing total variant candidates,not a proof verdict","contract_hash":"lowercase sha256"})}
    schemas=[{"name":name,"schema":s,"required_fields":list(fields),"field_types":fields,"optional_fields":[],"unknown_fields":"reject","hash_domain":d,"hash_field":h,"hash_preimage_fields":[k for k in fields if k!=h]} for name,(s,d,h,fields) in roots.items()]
    return {"schema":"mpk.go_practical.contract_wire_draft.v0","roots":schemas,"source_schema":GO["source_overlay"][0],"contract_schema":GO["normalized_contract"][0],"scope_schema":SCOPE_SCHEMA,"binding_schema":BINDING_SCHEMA,"new_identities":[{"schema":SCOPE_SCHEMA,"domain":SCOPE_DOMAIN},{"schema":BINDING_SCHEMA,"domain":BINDING_DOMAIN}],"source_arms":[{"op":op,"required_fields":source_fields(op),"optional_fields":[],"unknown_fields":"reject"} for op in sorted(OPERATORS)],"normalized_arms":arms,"expression_type_union":{"logical":["kind","name"],"source":["kind","source_type_id"]},"logical_types":["Bool","Nat","Int"],"source_path_union":{"field":["kind","name"],"elements":["kind"]},"normalized_path_union":{"field":["kind","owner_type_id","ordinal"],"elements":["kind","owner_type_id","extent"]},"normalized_variant_union":{"absent":["kind"],"candidate":["kind","origin","expression"]},"binding_required_fields":["schema","capture_context_id","source_object_id","function_decl_id","kind","position","name","start_byte","end_byte","source_type_id","id"],"remaining":["independent complete source/capture/alias/type/lexical binding/global-U root derivation and all source/definedness/value/permission/frame/loop proofs","shared VIR/source-map/manifest/VC/policy/API/release roots/unions, reviewed four-profile migration, W05/T01 gate and T02–T11"]}
