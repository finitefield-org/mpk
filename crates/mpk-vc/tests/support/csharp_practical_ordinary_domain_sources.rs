//! Additional original-source requests for recursive domain boundaries.
use super::*;

pub(super) fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"DomainCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn source_type(name: &str) -> Value {
    json!({"kind":"source","id":source_id(name)})
}
fn root() -> String {
    let envelope = source_id("Envelope");
    csharp_practical_declaration_id(&json!({"kind":"method","namespace":"DomainCases","owner":source_id("Entry"),"name":"Run","parameter_type_ids":[envelope.clone()],"result_type_id":envelope})).unwrap()
}
fn binding(
    source: &str,
    source_hash: &str,
    role: &str,
    members: Vec<(&str, &str, Value)>,
    arguments: Vec<String>,
) -> SemanticBindingInput {
    let id = source_id(source);
    SemanticBindingInput {
        source_type_id: id.clone(),
        source_content_sha256: source_hash.into(),
        role: role.into(),
        member_map: members
            .into_iter()
            .map(|(role, name, ty)| SemanticBindingMember {
                role: role.into(),
                member_id: csharp_practical_stored_member_id(&id, name, &ty, "readonly_field")
                    .unwrap(),
            })
            .collect(),
        inferred_argument_ids: arguments,
        tag_arms: vec![],
        default_arm: "ineligible".into(),
        bounds: if matches!(role, "ordered_map" | "ordered_set") {
            vec![SemanticBound {
                id: "length".into(),
                maximum: 4096,
            }]
        } else {
            vec![]
        },
        operation_map: vec![],
        enum_arms: BTreeMap::new(),
    }
}
fn requests() -> Vec<Value> {
    let bundle = b();
    let mut requests = vec![];
    let integer = primitive("i32");
    for (id, cs_type, key, extra, map) in [
        ("map-string", "string", primitive("string"), "", true),
        ("map-decimal", "decimal", primitive("decimal"), "", true),
        ("set-decimal", "decimal", primitive("decimal"), "", false),
        (
            "map-compound",
            "Key",
            source_type("Key"),
            "public readonly struct Key{public readonly string Name;public readonly int Code;}",
            true,
        ),
    ] {
        let body = if map {
            format!("public readonly struct Pair{{public readonly {cs_type} Key;public readonly int Value;}}public readonly struct Envelope{{public readonly Pair[] Items;}}")
        } else {
            format!("public readonly struct Envelope{{public readonly {cs_type}[] Items;}}")
        };
        let code=format!("namespace DomainCases;{extra}{body}public static class Entry{{public static Envelope Run(Envelope value){{return value;}}}}\n");
        let (plain_context, plain_captures) = support::context(&bundle, &root(), code.as_bytes());
        let source_hash = plain_captures.entries()[0].raw_sha256();
        let key_id = if key["kind"] == "primitive" {
            ty(key["id"].as_str().unwrap())
        } else {
            key["id"].as_str().unwrap().to_owned()
        };
        let mut bindings = vec![];
        if map {
            bindings.push(binding(
                "Pair",
                source_hash,
                "ordered_entry",
                vec![
                    ("key", "Key", key.clone()),
                    ("value", "Value", integer.clone()),
                ],
                vec![key_id.clone(), ty("i32")],
            ));
            bindings.push(binding(
                "Envelope",
                source_hash,
                "ordered_map",
                vec![(
                    "entries",
                    "Items",
                    instance("bounded_sequence", vec![source_type("Pair")]),
                )],
                vec![key_id, ty("i32")],
            ));
        } else {
            bindings.push(binding(
                "Envelope",
                source_hash,
                "ordered_set",
                vec![("elements", "Items", instance("bounded_sequence", vec![key]))],
                vec![key_id],
            ));
        }
        let sidecar = build_semantic_bindings(&plain_context, &plain_captures, bindings)
            .unwrap()
            .canonical_bytes()
            .to_vec();
        let (context, captures) =
            support::context_with_sidecar(&bundle, &root(), code.as_bytes(), |_| sidecar);
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source {"source"} else {"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    let fields = (0..15)
        .map(|i| format!("public readonly int F{i};"))
        .collect::<String>();
    let code=format!("namespace DomainCases;public readonly struct Item{{{fields}public readonly bool? Flag;}}public readonly struct Envelope{{public readonly Item[] Items;}}public static class Entry{{public static Envelope Run(Envelope value){{return value;}}}}\n");
    let (context, captures) = support::context(&bundle, &root(), code.as_bytes());
    requests.push(json!({"id":"total-cells","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":"source","path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    requests
}
fn exception_requests() -> Vec<Value> {
    let bundle = b();
    let mut requests = vec![];
    let integer_root = csharp_practical_declaration_id(&json!({"kind":"method","namespace":"DomainCases","owner":source_id("Entry"),"name":"Run","parameter_type_ids":[ty("i32")],"result_type_id":ty("i32")})).unwrap();
    for (id, declaration, construct) in [
        ("exception-empty", "public sealed class Fault:System.Exception{}", "new Fault()"),
        ("exception-payload", "public sealed class Fault:System.Exception{public readonly int Code;public readonly bool Flag;public readonly char Unit;public Fault(int code){Code=code;Flag=true;Unit='x';}}", "new Fault(value)"),
        ("exception-nested", "public sealed class Fault:System.Exception{public readonly System.Exception Error;public Fault(){Error=new System.ArgumentException();}}", "new Fault()"),
    ] {
        let code = format!("namespace DomainCases;{declaration}public static class Entry{{public static int Run(int value){{if(value<0)throw {construct};return value;}}}}\n");
        let (context, captures) = support::context(&bundle, &integer_root, code.as_bytes());
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":"source","path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    requests
}
#[test]
fn csharp_03_t06_w09_domain_boundary_source_requests() {
    let requests = requests();
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/csharp-03/ordinary-foundation/domain-sources/requests.json",
    );
    if let Some(out) = std::env::var_os("MPK_W09_DOMAIN_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(fs::read(fixture).unwrap(), bytes);
    }
}

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    load_sources("domain-sources", 5)
        .into_iter()
        .chain(exception_sources())
        .collect()
}
pub(super) fn exception_sources() -> Vec<(String, Value, Value)> {
    let sources = load_sources("exception-domain-sources", 3);
    assert_eq!(sources.len(), 2);
    sources
}
#[test]
fn csharp_03_t06_w09_domain_exception_source_requests() {
    let bytes = serde_json::to_vec_pretty(&exception_requests()).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/csharp-03/ordinary-foundation/exception-domain-sources/requests.json",
    );
    if let Some(out) = std::env::var_os("MPK_W09_EXCEPTION_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(fs::read(fixture).unwrap(), bytes);
    }
}
fn load_sources(directory: &str, count: usize) -> Vec<(String, Value, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation")
        .join(directory);
    let requests: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("requests.json")).unwrap()).unwrap();
    let responses: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.len(), count);
    assert_eq!(responses.len(), count);
    requests
        .into_iter()
        .filter_map(|row| {
            let response = responses.iter().find(|r| r["id"] == row["id"]).unwrap();
            if row["id"] == "exception-nested" {
                assert_eq!(
                    response["reject"],
                    "CSHARP_PRACTICAL_TYPE/exception_value_api"
                );
                assert_eq!(response["artifact_count"], 0);
                assert!(response.get("facts").is_none());
                return None;
            }
            assert!(response.get("reject").is_none(), "{}", row["id"]);
            Some((
                format!("boundary-{}", row["id"].as_str().unwrap()),
                row,
                response["facts"].clone(),
            ))
        })
        .collect()
}
