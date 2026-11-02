use mpk_vc::go_practical_type_keys::{build_go_source_scopes,evaluate_go_constant_binding,import_go_type_key_components,check_go_function_body,GoSourceScopes,GoTypeSourceBuffer};
use mpk_vc::{canonical_json_bytes,hash_canonical_json,parse_strict_json,HashDomain,StrictJsonLimits};
use serde_json::{json,Value};
use std::path::PathBuf;
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/aggregate-constants")}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_AGGREGATE_CONSTANTS_RUST_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let mut raw = serde_json::to_vec(value).unwrap();
        raw.push(b'\n');
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
}
fn scopes(input: &Value) -> std::result::Result<GoSourceScopes, String> {
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&v["object"]).unwrap(),
                &bytes(&v["raw_hex"]),
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| format!("{e:?}"))?;
    let keys = import_go_type_key_components(&bytes(&input["components_hex"]), &sources)
        .map_err(|e| format!("{e:?}"))?;
    build_go_source_scopes(&keys).map_err(|e| format!("{e:?}"))
}
fn canonical(v: &Value) -> Vec<u8> {
    canonical_json_bytes(
        &parse_strict_json(
            &serde_json::to_vec(v).unwrap(),
            StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
        )
        .unwrap(),
    )
    .unwrap()
}
fn seal(v: &mut Value, domain: &'static str, field: &str) {
    v.as_object_mut().unwrap().remove(field);
    let b = canonical(v);
    let x = parse_strict_json(
        &b,
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
    )
    .unwrap();
    v[field] = json!(hash_canonical_json(HashDomain::new(domain), &x)
        .unwrap()
        .to_hex())
}
fn sort(v: &mut Value, key: &str) {
    v.as_array_mut()
        .unwrap()
        .sort_by(|a, b| a[key].as_str().cmp(&b[key].as_str()))
}

fn control(label: &str, text: &str) -> (Value, GoSourceScopes) {
    let cap = format!("scalar-controls/{label}");
    let file = GoTypeSourceBuffer::new(&cap, "policy.go", "go_file", text.as_bytes()).unwrap();
    let module = GoTypeSourceBuffer::new(
        &cap,
        "go.mod",
        "go_mod",
        b"module example.com/scalar\ngo 1.25.0\n",
    )
    .unwrap();
    let mut p = json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/scalar","package_name":"policy","source_objects":[file.object().id]});
    seal(&mut p, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut t = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});
    seal(&mut t, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut v = json!({"schema":"mpk.go.type_components.v2","source_objects":[file.object(),module.object()],"packages":[p.clone()],"declarations":[],"types":[t.clone()],"underlying":[],"aliases":[],"package_roots":[p["id"]],"type_roots":[t["id"]]});
    sort(&mut v["source_objects"], "id");
    seal(&mut v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    let input = json!({"components_hex":canonical(&v).iter().map(|b|format!("{b:02x}")).collect::<String>(),"sources":[{"object":file.object(),"raw_hex":file.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()},{"object":module.object(),"raw_hex":module.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()}]});
    retain(&format!("{label}-input.json"), &input);
    let graph = scopes(&input).unwrap();
    (input, graph)
}


#[test]
fn original_array_constants_and_contextual_iota(){
 let(input,graph)=control("aggregate-constant-basic",r#"package policy
type W uint16
type A=[3]W
var array A
var inferred=[4]W{}
func Probe(a A){const(Result=len(a);_);_=Result}
const(I=len([iota+1]W{});J;K)
const N=cap(inferred)
"#);
 let mut output=vec![];
 for(name,want)in[("Result","3"),("I","1"),("J","2"),("K","3"),("N","4")]{let b=graph.bindings().iter().find(|b|b.name()==name).unwrap();let value=evaluate_go_constant_binding(&graph,b).unwrap();assert_eq!(value.integer_decimal().as_deref(),Some(want));output.push(json!({"name":name,"value":value.integer_decimal()}));}
 retain("aggregate-basic-actual.json",&json!(output));drop(input);
}
#[test]
fn local_array_reads_and_cross_dependency_cycles(){
 let(input,graph)=control("aggregate-local-read","package policy\nfunc Probe(){var a [2]byte;const N=len(a);var _ [N]byte}\n");let source=input["sources"][0]["object"]["id"].as_str().unwrap();let node=graph.source(source).unwrap().nodes().iter().position(|n|n.kind()=="FuncDecl").unwrap();let body=check_go_function_body(&graph,source,node).unwrap();assert_eq!(body.local_variables().len(),1);
 retain("aggregate-local-actual.json",&json!({"locals":body.local_variables().len()}));
 let(_,graph)=control("aggregate-cycle","package policy\nconst N=len(a)\nvar a [N]byte\n");let b=graph.bindings().iter().find(|b|b.name()=="N").unwrap();assert!(evaluate_go_constant_binding(&graph,b).is_err());
}
