"""Private strict source-identity DAG/hash prototype; not source acceptance."""
import hashlib
import json
import re
import unicodedata
from identity_catalog import IDENTITY_SHAPES,TYPE_ARMS

SAFE=(1<<53)-1
SHA=re.compile(r"[0-9a-f]{64}\Z")


class IdentityError(ValueError):pass


def need(ok,message):
    if not ok:raise IdentityError(message)


def parse(raw):
    def pairs(items):
        out={}
        for key,value in items:
            need(key not in out,"duplicate JSON member");out[key]=value
        return out
    def no_float(value):raise IdentityError("narrow integral JSON only")
    try:value=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs,parse_float=no_float,parse_constant=no_float)
    except (UnicodeError,json.JSONDecodeError) as e:raise IdentityError("strict UTF-8/JSON") from e
    canonical(value)
    return value


def canonical(value,depth=0):
    need(depth<=64,"private hash construction depth")
    if value is None:return b"null"
    if type(value) is bool:return b"true" if value else b"false"
    if type(value) is int:
        need(-SAFE<=value<=SAFE,"safe integral JSON; full Go words use typed decimal strings")
        return str(value).encode()
    if type(value) is str:
        try:value.encode("utf-8")
        except UnicodeError as e:raise IdentityError("Unicode scalar string") from e
        return json.dumps(value,ensure_ascii=False,separators=(",",":")).encode("utf-8")
    if type(value) is list:return b"["+b",".join(canonical(v,depth+1) for v in value)+b"]"
    need(type(value) is dict and all(type(k) is str for k in value),"JSON object keys")
    try:keys=sorted(value,key=lambda s:s.encode("utf-16-be"))
    except UnicodeError as e:raise IdentityError("Unicode scalar object key") from e
    return b"{"+b",".join(canonical(k,depth+1)+b":"+canonical(value[k],depth+1) for k in keys)+b"}"


def digest(domain,value):
    need(type(domain) is str and re.fullmatch(r"[A-Z][A-Z0-9-]*-[0-9]+\.[0-9]+",domain),"exact ASCII domain")
    return hashlib.sha256(domain.encode("ascii")+b"\x00"+canonical(value)).hexdigest()


def path(value):
    need(type(value) is str and value and len(value.encode("utf-8"))<=1024 and "\\" not in value and not value.startswith("/") and all(p not in {"",".",".."} for p in value.split("/")) and not any(ord(c)<32 for c in value),"normalized logical path, never a host path")


def exact(value,fields):need(type(value) is dict and set(value)==set(fields),"closed required record fields")


def validate_value(kind,value):
    enums={"source_kind":{"go_source","go_mod","go_sum","overlay","metadata","export_data","excluded_source","auxiliary"},"module_kind":{"main","dependency","local_replacement","stdlib"},"declaration_kind":{"function","method","type","alias_type","const"},"loop_kind":{"for","range"},"language":{"go1.22","go1.23","go1.24","go1.25"},"builtin_name":{"bool","int","int8","int16","int32","int64","uint","uint8","uint16","uint32","uint64","string","error"}}
    if kind.startswith("nullable_"):
        if value is not None:validate_value(kind[len("nullable_"):],value)
    elif kind=="sha256":need(type(value) is str and SHA.fullmatch(value),"lowercase SHA256")
    elif kind=="uint":need(type(value) is int and 0<=value<=SAFE,"unsigned safe metadata integer")
    elif kind=="bool":need(type(value) is bool,"Boolean metadata")
    elif kind=="text":need(type(value) is str,"text metadata");canonical(value)
    elif kind=="logical_path":path(value)
    elif kind in enums:need(type(value) is str and value in enums[kind],"closed "+kind)
    elif kind in {"linux","amd64","v1","go1.25.0"}:need(value==kind,"fixed registered target/compiler candidate")
    elif kind in {"sha256s","sorted_sha256s","sorted_texts"}:
        need(type(value) is list,"ordered reference/text array")
        for v in value:validate_value("text" if kind=="sorted_texts" else "sha256",v)
        if kind.startswith("sorted_"):need(value==sorted(set(value),key=lambda s:s.encode("utf-8")),"unique UTF-8 byte order")
    elif kind=="replacement":
        exact(value,{"kind","path","version"});need(value["kind"] in {"none","versioned","local"},"closed replacement tag")
        if value["kind"]=="none":need(value["path"] is None and value["version"] is None,"no inactive replacement payload")
        elif value["kind"]=="versioned":need(type(value["path"]) is str and value["path"] and type(value["version"]) is str and value["version"],"versioned replacement")
        else:path(value["path"]);need(value["version"] is None,"logical local replacement capability, no host path/version")
    elif kind=="type_fields":
        need(type(value) is list and len(value)<=64,"ordered direct fields")
        for f in value:
            exact(f,{"name","unexported_package_id","type_id","tag_bytes_hex"})
            need(type(f["name"]) is str and f["name"],"Go field name")
            validate_value("nullable_sha256",f["unexported_package_id"]);validate_value("sha256",f["type_id"])
            need(type(f["tag_bytes_hex"]) is str and re.fullmatch(r"(?:[0-9a-f]{2})*",f["tag_bytes_hex"]),"arbitrary source tag bytes, no Unicode normalization")
            exported=unicodedata.category(f["name"][0])=="Lu"
            need((f["unexported_package_id"] is None)==exported,"Go exported/unexported field identity")
    else:raise IdentityError("unknown field type: "+kind)


def record_fields(family,node):
    schema,domain,fields=IDENTITY_SHAPES[family]
    if family=="source_type":
        need(node.get("kind") in TYPE_ARMS,"closed source type identity union")
        fields={"kind":"text",**TYPE_ARMS[node["kind"]]}
    return schema,domain,fields


def validate_node(node):
    need(type(node) is dict,"identity record")
    families=[family for family,(schema,_,_) in IDENTITY_SHAPES.items() if schema==node.get("schema")]
    need(len(families)==1,"exact source identity schema/version")
    family=families[0];schema,domain,fields=record_fields(family,node)
    exact(node,{"schema","id",*fields})
    for field,kind in fields.items():validate_value(kind,node[field])
    if family in {"declaration","loop_ref"}:need(node["start_byte"]<node["end_byte"],"nonempty physical source span")
    if family=="declaration":need((node["receiver_type_id"] is not None)==(node["kind"]=="method"),"receiver only on a method declaration")
    if family=="signature":need(len(node["results"])<=16 and len(node["params"])+(node["receiver"] is not None)<=256,"fixed signature/product arity")
    if family=="source_type":
        if node["kind"]=="builtin":
            name=node["name"]
            if name in {"bool","string","error"}:need(node["width"] is None and node["signed"] is None,"nonword builtin identity")
            else:
                width=64 if name in {"int","uint"} else int(name.lstrip("uint"))
                need(node["width"]==width and node["signed"]==(not name.startswith("u")),"complete target word identity")
        if node["kind"]=="array":need(node["length"]<=256,"candidate array extent")
    validate_value("sha256",node["id"])
    need(node["id"]==digest(domain,{k:node[k] for k in ["schema",*fields]}),"complete typed self-hash preimage")
    return family


def seal(family,fields):
    schema,domain,_=IDENTITY_SHAPES[family];node={"schema":schema,**fields}
    _,_,types=record_fields(family,node);exact(node,{"schema",*types})
    node["id"]=digest(domain,node);validate_node(node);return node


def references(family,node):
    table={"source_object":[],"module_instance":[("source_object",v) for v in node.get("objects",[])]+([("source_object",node["go_mod"])] if node.get("go_mod") else []),"package_instance":[("module_instance",node.get("module_id"))]+[("source_object",v) for v in node.get("objects",[])],"declaration":[("package_instance",node.get("package_id")),("source_object",node.get("source_object_id"))]+([("source_type",node["receiver_type_id"])] if node.get("receiver_type_id") else []),"signature":[("declaration",node.get("declaration_id"))]+[("source_type",v) for v in node.get("params",[])+node.get("results",[])]+([("source_type",node["receiver"])] if node.get("receiver") else []),"error_universe":[("source_type",node.get("source_error_type_id"))]+[("source_type",v) for v in node.get("cases",[])],"build_config":[],"capture_context":[("build_config",node.get("build_config_id"))]+[("source_object",v) for v in node.get("objects",[])]+[("module_instance",v) for v in node.get("modules",[])]+[("package_instance",v) for v in node.get("packages",[])],"loop_ref":[("capture_context",node.get("capture_context_id")),("source_object",node.get("source_object_id")),("declaration",node.get("function_decl_id"))]}
    if family!="source_type":return table[family]
    kind=node["kind"]
    if kind=="nominal":return [("declaration",node["declaration_id"])]
    if kind in {"array","slice"}:return [("source_type",node["element"])]
    if kind=="struct":return [("source_type",f["type_id"]) for f in node["fields"]]+[("package_instance",f["unexported_package_id"]) for f in node["fields"] if f["unexported_package_id"]]
    return []


def validate_graph(nodes,roots):
    rows={n["id"]:n for n in nodes};need(len(rows)==len(nodes),"unique identity nodes")
    kinds={key:validate_node(n) for key,n in rows.items()};active=set();done=set()
    def visit(key):
        need(key in rows,"complete reference closure")
        need(key not in active,"acyclic hash/reference DAG")
        if key in done:return
        active.add(key)
        for kind,child in references(kinds[key],rows[key]):
            need(child in rows and kinds[child]==kind,"typed identity reference")
            visit(child)
        active.remove(key);done.add(key)
    for root in roots:visit(root)
    need(done==set(rows),"no unowned/unreachable identity nodes")
    captures=[n for n in nodes if kinds[n["id"]]=="capture_context"]
    need(len(captures)==1,"one complete capture context")
    capture=captures[0]
    members={"source_object":set(capture["objects"]),"module_instance":set(capture["modules"]),"package_instance":set(capture["packages"])}
    for kind,ids in members.items():need(ids=={n["id"] for n in nodes if kinds[n["id"]]==kind},"complete capture membership")
    paths=set()
    for n in nodes:
        kind=kinds[n["id"]]
        if kind=="source_object":
            fold=str.maketrans("ABCDEFGHIJKLMNOPQRSTUVWXYZ","abcdefghijklmnopqrstuvwxyz")
            logical=(n["capability"].translate(fold),n["path"].translate(fold))
            need(logical not in paths,"unique captured logical path/ASCII case fold");paths.add(logical)
        elif kind=="module_instance":
            need(n["go_mod"] is None if n["kind"]=="stdlib" else n["go_mod"] in n["objects"] and rows[n["go_mod"]]["kind"]=="go_mod","module manifest linkage")
        elif kind=="package_instance":
            need(set(n["objects"])<=set(rows[n["module_id"]]["objects"]) and all(rows[v]["kind"]=="go_source" for v in n["objects"]),"package source/module membership")
        elif kind=="declaration":
            need(n["source_object_id"] in rows[n["package_id"]]["objects"] and n["end_byte"]<=rows[n["source_object_id"]]["bytes"],"physical declaration/package/source linkage")
        elif kind=="source_type" and n["kind"]=="nominal":need(rows[n["declaration_id"]]["kind"]=="type","nominal identity is a defined type, not an alias")
        elif kind=="signature":need(rows[n["declaration_id"]]["kind"] in {"function","method"} and n["receiver"]==rows[n["declaration_id"]]["receiver_type_id"],"same declaration receiver/signature")
        elif kind=="error_universe":need(rows[n["source_error_type_id"]].get("name")=="error" and all(rows[v]["kind"]=="nominal" for v in n["cases"]),"predeclared error and concrete nominal cases; full U derivation remains unproved")
        elif kind=="loop_ref":
            fn=rows[n["function_decl_id"]]
            need(n["capture_context_id"]==capture["id"] and n["source_object_id"]==fn["source_object_id"] and fn["kind"] in {"function","method"} and fn["start_byte"]<=n["start_byte"]<n["end_byte"]<=fn["end_byte"],"same capture/function/physical loop binding")
    return {"identity_nodes":len(nodes),"roots":len(roots),"capture_context_id":capture["id"],"scope":"identity shape/hash/membership only; no registry/source/compiler/typing/U/loop semantic proof"}


def resolve_alias(reference,aliases):
    active=set()
    while reference in aliases:
        need(reference not in active,"acyclic captured alias resolution");active.add(reference);reference=aliases[reference]
    validate_value("sha256",reference)
    return reference
