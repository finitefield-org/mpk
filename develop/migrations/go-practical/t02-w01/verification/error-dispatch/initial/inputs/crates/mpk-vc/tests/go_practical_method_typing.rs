use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, check_go_assignment, import_go_type_key_components, infer_go_binding,
    infer_go_results, infer_go_value, GoSourceScopes, GoTypeSourceBuffer, GoValueTyping,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/method-typing")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_METHOD_TYPING_RUST_OUTPUT") {
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

fn method_calls(value: &GoValueTyping) -> Vec<Value> {
    let key = |id: &str| value.types().find(|k| k.id() == id).unwrap().clone();
    let mut calls = vec![];
    for h in value.method_call_headers() {
        let parsed = value.source(h.call_source_object_id()).unwrap();
        let call = &parsed.nodes()[h.call_node()];
        let function = &parsed.nodes()[h.function_node()];
        let receiver = &parsed.nodes()[h.receiver_node()];
        let method = h.method();
        let declaration = method
            .declaration_source_object_id()
            .zip(method.declaration_node())
            .map(|(s, n)| &value.source(s).unwrap().nodes()[n]);
        let body = method
            .declaration_source_object_id()
            .zip(method.body_node())
            .map(|(s, n)| &value.source(s).unwrap().nodes()[n]);
        calls.push(json!({"source":h.call_source_object_id(),"call_start":call.start(),"call_end":call.end(),"function_start":function.start(),"function_end":function.end(),"receiver_start":receiver.start(),"receiver_end":receiver.end(),"method_expression":h.method_expression(),"selected_receiver":key(h.selected_receiver_type_id()),"name":method.name(),"declaration_source":method.declaration_source_object_id(),"declaration_start":declaration.map(|n|n.start()),"declaration_end":declaration.map(|n|n.end()),"body_start":body.map(|n|n.start()),"body_end":body.map(|n|n.end()),"declared_receiver":key(method.receiver_type_id()),"path":method.receiver_path(),"parameters":h.parameter_type_ids().iter().map(|id|key(id)).collect::<Vec<_>>(),"results":h.result_type_ids().iter().map(|id|key(id)).collect::<Vec<_>>() }));
    }
    calls.sort_by(|a, b| {
        (
            a["source"].as_str(),
            a["call_start"].as_u64(),
            a["call_end"].as_u64(),
        )
            .cmp(&(
                b["source"].as_str(),
                b["call_start"].as_u64(),
                b["call_end"].as_u64(),
            ))
    });
    calls
}
fn output(value: &GoValueTyping) -> Value {
    json!({"accepted":true,"method_calls":method_calls(value),"values":value.values().iter().map(|a|{let c=a.constant();json!({"type_key":a.source_type_key(),"underlying_builtin":a.scalar_type().map(|t|t.underlying_builtin()),"constant":c.is_some(),"integer_decimal":c.and_then(|c|c.integer_decimal()),"boolean":c.and_then(|c|c.boolean()),"bytes_hex":c.and_then(|c|c.bytes()).map(|b|b.iter().map(|b|format!("{b:02x}")).collect::<String>()),"addressable":a.addressable(),"untyped_nil":a.is_untyped_nil()})}).collect::<Vec<_>>(),"initializer_body_or_invocation_state_or_program_authority":false})
}
#[test]
fn matches_original_go_method_and_error_types() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus-calls.json")).unwrap()).unwrap();
    let cases = corpus.as_array().unwrap();
    assert!(cases.len() > 400);
    let (mut accepted, mut rejected, mut outside) = (0, 0, 0);
    let mut failures = vec![];
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
                infer_go_binding(graph, binding.ok_or("binding".to_string())?)
            } else {
                let node = parsed
                    .nodes()
                    .iter()
                    .position(|n| {
                        (n.kind().ends_with("Expr")
                            || matches!(
                                n.kind(),
                                "Ident" | "BasicLit" | "FuncLit" | "CompositeLit"
                            ))
                            && n.start() as u64 == input["expression_start_byte"].as_u64().unwrap()
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
                    check_go_assignment(graph, source, node, source, *typ)
                } else if input["query_mode"] == "tuple" {
                    infer_go_results(graph, source, node)
                } else {
                    infer_go_value(graph, source, node)
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
        // The original corpus predates finite error invocation headers. Keep
        // its bytes intact while promoting exactly the actual direct builtin
        // error calls; promoted/interface/pointer method values stay outside.
        let direct_error_call = row["native"]["method_calls"].as_array().is_some_and(|calls| {
            calls.len() == 1 && calls[0]["declaration_source"].is_null()
                && calls[0]["selected_receiver"]["kind"] == "builtin"
                && calls[0]["selected_receiver"]["name"] == "error"
                && calls[0]["path"].as_array().is_some_and(Vec::is_empty)
        });
        if input["outside_value_typing"] == true && row["native"]["accepted"] == true && !direct_error_call {
            assert!(row["native"]["accepted"] == true);
            if actual.is_ok() {
                failures.push(format!("outside case {n} {} accepted", input["label"]));
            }
            outside += 1;
        } else if row["native"]["accepted"] == true {
            if outcome != row["native"] {
                failures.push(format!(
                    "case {n} {} expected {} actual {outcome}",
                    input["label"], row["native"]
                ));
            }
            accepted += 1;
        } else {
            if actual.is_ok() {
                failures.push(format!(
                    "case {n} {} falsely accepted: {outcome}",
                    input["label"]
                ));
            }
            rejected += 1;
        }
    }
    retain(
        "summary.json",
        &json!({"accepted":accepted,"rejected":rejected,"outside":outside,"failures":failures}),
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
fn original_error_signature_dependency_and_method_calls_remain_distinct_and_owned() {
    let(input,graph)=control("method-owned","package policy\ntype E struct{N uint16}\nfunc(E)Error()string{_=make(map[int]int);return \"opaque body\"}\nfunc(E)Make(uint16)(uint16,error)\ntype Wrap struct{E}\nfunc Probe(){var Result error=Wrap{};_=Result;Wrap{}.Make(1);Wrap{}.Error()}\n");
    let binding = graph
        .bindings()
        .iter()
        .find(|b| b.name() == "Result")
        .unwrap();
    let source = binding.source_object_id().to_string();
    let assigned = infer_go_binding(&graph, binding).unwrap();
    assert!(assigned.call_headers().is_empty());
    assert!(assigned.method_call_headers().is_empty());
    assert!(assigned.slice_builtin_headers().is_empty());
    let implementation = assigned
        .error_implementations()
        .iter()
        .find(|i| i.method().receiver_path() == [0] && i.method().body_node().is_some())
        .unwrap();
    assert_eq!(implementation.method().name(), "Error");
    assert!(implementation.method().parameter_type_ids().is_empty());
    assert_eq!(
        implementation.method().declaration_source_object_id(),
        Some(source.as_str())
    );
    let parsed = graph.source(&source).unwrap();
    let call = |text: &[u8]| {
        parsed
            .nodes()
            .iter()
            .enumerate()
            .find(|(n, node)| node.kind() == "CallExpr" && parsed.spelling(*n) == Some(text))
            .unwrap()
            .0
    };
    let joint = infer_go_results(&graph, &source, call(b"Wrap{}.Make(1)")).unwrap();
    assert_eq!(joint.values().len(), 2);
    assert_eq!(joint.method_call_headers().len(), 1);
    let h = &joint.method_call_headers()[0];
    assert_eq!(h.method().name(), "Make");
    assert_eq!(h.method().receiver_path(), [0]);
    assert!(h.method().body_node().is_none());
    assert!(!h.method_expression());
    assert_eq!(h.parameter_type_ids().len(), 1);
    assert_eq!(h.result_type_ids().len(), 2);
    let scalar = mpk_vc::go_practical_type_keys::infer_go_scalar_value(
        &graph,
        &source,
        call(b"Wrap{}.Error()"),
    )
    .unwrap();
    assert_eq!(
        scalar.values()[0].scalar_type().underlying_builtin(),
        "string"
    );
    assert!(scalar.values()[0].constant().is_none());
    assert_eq!(scalar.method_call_headers().len(), 1);
    assert!(scalar.method_call_headers()[0]
        .method()
        .body_node()
        .is_some());
    assert!(scalar.slice_builtin_headers().is_empty());
    let headers=joint.method_call_headers().iter().map(|h|json!({"source":h.call_source_object_id(),"call":h.call_node(),"function":h.function_node(),"receiver_node":h.receiver_node(),"selected_receiver":h.selected_receiver_type_id(),"method_expression":h.method_expression(),"declaration_source":h.method().declaration_source_object_id(),"declaration":h.method().declaration_node(),"body":h.method().body_node(),"receiver_type":h.method().receiver_type_id(),"path":h.method().receiver_path(),"parameters":h.parameter_type_ids(),"results":h.result_type_ids()})).collect::<Vec<_>>();
    assert!(infer_go_value(&graph, "missing", 0).is_err());
    let (_, foreign) = control(
        "foreign-method-owner",
        "package policy\nfunc F(x uint16){}\n",
    );
    assert!(infer_go_binding(
        &graph,
        foreign.bindings().iter().find(|b| b.name() == "x").unwrap()
    )
    .is_err());
    drop(input);
    drop(graph);
    assert!(!assigned
        .source(&source)
        .unwrap()
        .source()
        .bytes()
        .is_empty());
    retain(
        "owned-method-actual.json",
        &json!({"assigned":output(&assigned),"joint":output(&joint),"method_calls":headers,"source_body_evaluation_or_invocation_or_program_proof":false}),
    );
}
#[test]
fn original_method_receiver_work_is_flat() {
    let text=format!("package policy\ntype E uint16\nfunc(e E)Id()E{{return e}}\nfunc Probe(){{var Result=E(1){};_=Result}}\n",".Id()".repeat(200));
    let (_, graph) = control("method-chain", &text);
    let b = graph
        .bindings()
        .iter()
        .find(|b| b.name() == "Result")
        .unwrap();
    let result = infer_go_binding(&graph, b).unwrap();
    assert_eq!(result.method_call_headers().len(), 200);
    assert!(result.values()[0].constant().is_none());
    retain("method-chain-actual.json", &output(&result));
}
