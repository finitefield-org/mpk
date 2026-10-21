use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, check_go_scalar_assignment, import_go_type_key_components,
    infer_go_scalar_binding, infer_go_scalar_value, GoScalarTyping, GoSourceScopes,
    GoTypeSourceBuffer,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/scalar-typing")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_SCALAR_TYPING_RUST_OUTPUT") {
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

fn output(value: &GoScalarTyping) -> Value {
    json!({"accepted":true,"values":value.values().iter().map(|a|{let c=a.constant();json!({"type_key":a.scalar_type().source_type_key(),"underlying_builtin":a.scalar_type().underlying_builtin(),"constant":c.is_some(),"integer_decimal":c.and_then(|c|c.integer_decimal()),"boolean":c.and_then(|c|c.boolean()),"bytes_hex":c.and_then(|c|c.bytes()).map(|b|b.iter().map(|b|format!("{b:02x}")).collect::<String>()),"addressable":a.addressable()})}).collect::<Vec<_>>(),"initializer_body_or_invocation_state_or_program_authority":false})
}
#[test]
fn matches_original_go_scalar_expression_and_binding_types() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases = corpus.as_array().unwrap();
    assert!(cases.len() > 1000);
    let (mut accepted, mut rejected, mut outside) = (0, 0, 0);
    for (n, row) in cases.iter().enumerate() {
        let input = &row["input"];
        retain(&format!("case-{n:05}-input.json"), input);
        let graph = scopes(input);
        let actual = graph.as_ref().map_err(Clone::clone).and_then(|graph| {
            let source = input["query_source_object_id"].as_str().unwrap();
            let parsed = graph.source(source).ok_or("source".to_string())?;
            let binding = graph
                .bindings()
                .iter()
                .find(|b| b.source_object_id() == source && b.name() == "Result");
            let result = if input["query_mode"] == "binding" {
                infer_go_scalar_binding(graph, binding.ok_or("binding".to_string())?)
            } else {
                let node = parsed
                    .nodes()
                    .iter()
                    .position(|n| {
                        (n.kind().ends_with("Expr") || matches!(n.kind(),"Ident"|"BasicLit"|"FuncLit"|"CompositeLit")) && n.start() as u64 == input["expression_start_byte"].as_u64().unwrap()
                            && n.end() as u64 == input["expression_end_byte"].as_u64().unwrap()
                    })
                    .ok_or("expression".to_string())?;
                if input["target_type"] != "" {
                    let spec =
                        &parsed.nodes()[binding.ok_or("binding".to_string())?.declaration_node()];
                    let typ = spec
                        .edges()
                        .iter()
                        .find(|e| e.role() == "Type")
                        .and_then(|e| e.targets().first())
                        .ok_or("annotation".to_string())?;
                    check_go_scalar_assignment(graph, source, node, source, *typ)
                } else {
                    infer_go_scalar_value(graph, source, node)
                }
            };
            result.map_err(|e| format!("{e:?}"))
        });
        let outcome = match &actual {
            Ok(v) => output(v),
            Err(e) => {
                json!({"accepted":false,"error":e,"initializer_body_or_invocation_state_or_program_authority":false})
            }
        };
        retain(&format!("case-{n:05}-actual.json"), &outcome);
        if input["outside_scalar_typing"] == true && row["native"]["accepted"] == true {
            assert!(row["native"]["accepted"] == true);
            assert!(actual.is_err(), "outside {n}");
            outside += 1;
        } else if row["native"]["accepted"] == true {
            assert_eq!(outcome, row["native"], "case {n} {}", input["label"]);
            accepted += 1;
        } else {
            assert!(
                actual.is_err(),
                "case {n} {} falsely accepted: {outcome}",
                input["label"]
            );
            rejected += 1;
        }
    }
    retain(
        "summary.json",
        &json!({"accepted":accepted,"rejected":rejected,"outside":outside}),
    );
}
