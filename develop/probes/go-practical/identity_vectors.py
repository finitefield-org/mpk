"""Source-identity fixtures and independent canonical/hash boundary vectors."""
import copy
import hashlib
import json
from identity_wire import seal,canonical,digest,parse,validate_graph,IdentityError
from identity_catalog import IDENTITY_SHAPES

SOURCE='''package policy
// 元のUTF-8ソース。位置は物理バイト。
type Amount int64
type Other int64
type Code uint8
type Money = Amount
func(Code) Error() string { return "code" }
func Root(xs []Amount) (Amount,error) { total:=Amount(0); for _,v:=range xs { total+=v }; if total<0 { return 0,Code(1) }; return total,nil }
'''


def fixture():
    nodes=[]
    def add(kind,fields):
        n=seal(kind,fields);nodes.append(n);return n["id"]
    def blob(path,kind,raw,capability="main"):
        return add("source_object",{"capability":capability,"path":path,"kind":kind,"raw_sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)})
    gomod=blob("go.mod","go_mod",b"module example.com/policy\n\ngo 1.25.0\n")
    source=blob("policy.go","go_source",SOURCE.encode())
    overlay=blob("contracts/root.json","overlay",b'{"function":{"name":"Root"}}\n',"overlays")
    module=add("module_instance",{"kind":"main","module_path":"example.com/policy","version":"","replacement":{"kind":"none","path":None,"version":None},"objects":sorted([gomod,source]),"go_mod":gomod})
    package=add("package_instance",{"module_id":module,"import_path":"example.com/policy","package_name":"policy","objects":[source]})
    types={}
    for name,width,signed in (("int64",64,True),("uint8",8,False),("string",None,None),("error",None,None)):
        types[name]=add("source_type",{"kind":"builtin","name":name,"width":width,"signed":signed})
    declarations={}
    for ordinal,(name,kind,prefix) in enumerate((("Amount","type","type Amount"),("Other","type","type Other"),("Code","type","type Code"),("Money","alias_type","type Money"))):
        start=SOURCE.encode().index(prefix.encode());end=SOURCE.encode().index(b"\n",start)
        declarations[name]=add("declaration",{"package_id":package,"source_object_id":source,"kind":kind,"start_byte":start,"end_byte":end,"ordinal":ordinal,"name":name,"receiver_type_id":None})
        if kind=="type":types[name]=add("source_type",{"kind":"nominal","declaration_id":declarations[name]})
    for ordinal,(name,prefix,receiver) in enumerate((("Error","func(Code)",types["Code"]),("Root","func Root",None)),4):
        start=SOURCE.encode().index(prefix.encode());end=SOURCE.encode().index(b"\n",start)
        declarations[name]=add("declaration",{"package_id":package,"source_object_id":source,"kind":"method" if receiver else "function","start_byte":start,"end_byte":end,"ordinal":ordinal,"name":name,"receiver_type_id":receiver})
    types["lines"]=add("source_type",{"kind":"slice","element":types["Amount"]})
    types["struct"]=add("source_type",{"kind":"struct","fields":[{"name":"Value","unexported_package_id":None,"type_id":types["int64"],"tag_bytes_hex":""},{"name":"code","unexported_package_id":package,"type_id":types["uint8"],"tag_bytes_hex":"ff"}]})
    signature=add("signature",{"declaration_id":declarations["Root"],"receiver":None,"params":[types["lines"]],"results":[types["Amount"],types["error"]]})
    method_signature=add("signature",{"declaration_id":declarations["Error"],"receiver":types["Code"],"params":[],"results":[types["string"]]})
    universe=add("error_universe",{"source_error_type_id":types["error"],"cases":[types["Code"]]})
    build=add("build_config",{"goos":"linux","goarch":"amd64","goamd64":"v1","compiler_release":"go1.25.0","tags":[]})
    capture=add("capture_context",{"semantic_context_sha256":hashlib.sha256(b"opaque private Go semantic-context fixture; not registry evidence").hexdigest(),"compiler_inventory_sha256":hashlib.sha256(b"opaque private compiler inventory fixture").hexdigest(),"build_config_id":build,"objects":sorted([gomod,source,overlay]),"modules":[module],"packages":[package]})
    start=SOURCE.encode().index(b"for _,v:=range xs");end=SOURCE.encode().index(b"}",start)+1
    loop=add("loop_ref",{"capture_context_id":capture,"source_object_id":source,"function_decl_id":declarations["Root"],"ast_kind":"range","start_byte":start,"end_byte":end,"function_preorder_ordinal":0,"effective_language":"go1.25"})
    roots=[signature,method_signature,universe,loop,types["Other"],types["struct"],declarations["Money"]]
    result=validate_graph(nodes,roots)
    return {"nodes":nodes,"roots":roots,"result":result,"types":types,"declarations":declarations,"aliases":{declarations["Money"]:types["Amount"]},"raw_source_sha256":hashlib.sha256(SOURCE.encode()).hexdigest(),"scope":"supplied private roots and opaque external context/inventory hashes; no source/type/registry proof or production root derivation"}


def hash_vectors():
    rows=[]
    def add(identity,raw,accepted=True,domain="MPK-GO-IDENTITY-PILOT-1.0"):
        row={"id":identity,"domain":domain,"raw_json_hex":raw.hex(),"expected_accepted":accepted}
        if accepted:
            value=parse(raw);row.update(canonical_hex=canonical(value).hex(),digest=digest(domain,value))
        rows.append(row)
    for node in fixture()["nodes"]:
        family=next(k for k,(schema,_,_) in IDENTITY_SHAPES.items() if schema==node["schema"])
        preimage={k:v for k,v in node.items() if k!="id"}
        add("preimage-"+str(len(rows)),json.dumps(preimage,ensure_ascii=False,indent=2).encode(),domain=IDENTITY_SHAPES[family][1])
        if rows[-1]["digest"]!=node["id"]:raise ValueError("exact complete field projection")
    for identity,value in (("utf16-key-order",{"\ue000":1,"😀":2}),("controls-and-separators",{"text":"\x00\b\t\n\f\r\x1f\u2028\u2029<>&/\\\""}),("raw-word-decimal",{"value":"18446744073709551615"}),("nfc",{"name":"é"}),("nfd",{"name":"e\u0301"}),("safe-integers",[-9007199254740991,0,9007199254740991])):
        add(identity,json.dumps(value,ensure_ascii=False).encode())
    add("negative-zero",b"-0")
    add("escaped-surrogate-pair",b'"\\ud83d\\ude00"')
    for identity,raw in (("duplicate",b'{"x":1,"x":2}'),("nested-duplicate",b'{"x":{"y":0,"y":1}}'),("fraction",b"1.0"),("exponent",b"1e0"),("unsafe-integer",b"9007199254740992"),("lone-high-surrogate",b'"\\ud800"'),("lone-low-surrogate",b'"\\udfff"'),("invalid-utf8",b'"\xff"'),("second-root",b"{} {}")):
        add(identity,raw,False)
    add("invalid-domain",b"{}",False,domain="MPK-BAD\x00-1.0")
    return rows
