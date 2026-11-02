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
        .join("../../develop/migrations/go-practical/t02-w01/verification/slice-builtins")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_SLICE_BUILTIN_RUST_OUTPUT") {
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

fn output(value: &GoValueTyping) -> Value {
    json!({"accepted":true,"values":value.values().iter().map(|a|{let c=a.constant();json!({"type_key":a.source_type_key(),"underlying_builtin":a.scalar_type().map(|t|t.underlying_builtin()),"constant":c.is_some(),"integer_decimal":c.and_then(|c|c.integer_decimal()),"boolean":c.and_then(|c|c.boolean()),"bytes_hex":c.and_then(|c|c.bytes()).map(|b|b.iter().map(|b|format!("{b:02x}")).collect::<String>()),"addressable":a.addressable(),"untyped_nil":a.is_untyped_nil()})}).collect::<Vec<_>>(),"initializer_body_or_invocation_state_or_program_authority":false})
}
#[test]
fn matches_original_go_slice_builtin_types() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases = corpus.as_array().unwrap();
    assert!(cases.len() > 1500);
    let (mut accepted, mut rejected, mut outside) = (0, 0, 0);
    let mut failures = vec![];
    let mut native_panics = 0;
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
        if row["native"]["native_panic"].is_object() {
            assert!(
                actual.is_err(),
                "actual native panic case {n} must not gain a typing verdict"
            );
            native_panics += 1;
        } else if input["outside_value_typing"] == true && row["native"]["accepted"] == true {
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
        &json!({"accepted":accepted,"rejected":rejected,"outside":outside,"native_panics":native_panics,"failures":failures}),
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
fn original_builtin_headers_remain_owned_and_do_not_prove_access_or_callee_body() {
    let(input,graph)=control("slice-builtins-owned","package policy\ntype Word uint16\ntype Vec []Word\nfunc Opaque()Vec\nfunc Probe(u uint64){var Result=copy(make(Vec,1,2),append(Opaque(),1<<u));_=Result}\n");
    let binding = graph
        .bindings()
        .iter()
        .find(|b| b.name() == "Result")
        .unwrap();
    let source = binding.source_object_id().to_string();
    let result = infer_go_binding(&graph, binding).unwrap();
    let scalar = mpk_vc::go_practical_type_keys::infer_go_scalar_binding(&graph, binding).unwrap();
    assert_eq!(
        result.slice_builtin_headers(),
        scalar.slice_builtin_headers()
    );
    assert_eq!(result.slice_builtin_headers().len(), 3);
    assert_eq!(result.call_headers().len(), 1);
    assert!(result.call_headers()[0].body_node().is_none());
    let mut headers = vec![];
    for h in result.slice_builtin_headers() {
        assert_eq!(h.source_object_id(), source);
        let parsed = result.source(h.source_object_id()).unwrap();
        let node = &parsed.nodes()[h.call_node()];
        assert_eq!(node.kind(), "CallExpr");
        assert_eq!(
            h.argument_nodes(),
            node.edges()
                .iter()
                .find(|e| e.role() == "Args")
                .unwrap()
                .targets()
        );
        assert_eq!(
            h.function_node(),
            node.edges()
                .iter()
                .find(|e| e.role() == "Fun")
                .unwrap()
                .targets()[0]
        );
        assert_eq!(
            parsed.spelling(h.function_node()).unwrap(),
            h.name().as_bytes()
        );
        assert!(!h.ellipsis());
        assert_eq!(h.explicit_capacity(), h.name() == "make");
        assert_eq!(
            h.argument_nodes().len(),
            if h.name() == "make" { 3 } else { 2 }
        );
        assert!(result.types().any(|t| t.id() == h.slice_type_id()));
        assert!(result.types().any(|t| t.id() == h.element_type_id()));
        assert!(result.types().any(|t| t.id() == h.result_type_id()));
        headers.push(json!({"source":h.source_object_id(),"call":h.call_node(),"function":h.function_node(),"name":h.name(),"arguments":h.argument_nodes(),"slice_type":h.slice_type_id(),"element_type":h.element_type_id(),"result_type":h.result_type_id(),"ellipsis":h.ellipsis(),"explicit_capacity":h.explicit_capacity()}));
    }
    assert_eq!(
        result.values()[0]
            .scalar_type()
            .unwrap()
            .underlying_builtin(),
        "int"
    );
    assert!(result.values()[0].constant().is_none());
    assert!(result.values()[0].addressable());
    assert!(infer_go_value(&graph, "missing", 0).is_err());
    let (_, foreign) = control(
        "foreign-builtin-owner",
        "package policy\nfunc F(x []byte){}\n",
    );
    assert!(infer_go_binding(
        &graph,
        foreign.bindings().iter().find(|b| b.name() == "x").unwrap()
    )
    .is_err());
    drop(input);
    drop(graph);
    assert!(!result.source(&source).unwrap().source().bytes().is_empty());
    retain(
        "owned-headers-actual.json",
        &json!({"values":output(&result),"headers":headers,"allocation_access_effect_or_body_authority":false}),
    );
}
