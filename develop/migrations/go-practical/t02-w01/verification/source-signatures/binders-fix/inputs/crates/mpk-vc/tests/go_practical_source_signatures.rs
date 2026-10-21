use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, derive_go_declared_binding_type, derive_go_source_signature,
    import_go_type_key_components, GoSourceScopes, GoSourceSignature, GoSourceSignatureError,
    GoSourceTypeError, GoTypeKeyComponents, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-signatures")
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
fn retain(name: &str, v: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_SIGNATURES_RUST_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let mut raw = canonical(v);
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
fn candidate(label: &str, text: &str) -> GoTypeKeyComponents {
    let file = GoTypeSourceBuffer::new("root", "policy.go", "go_file", text.as_bytes()).unwrap();
    let module = GoTypeSourceBuffer::new(
        "root",
        "go.mod",
        "go_mod",
        b"module example.com/policy\ngo 1.25.0\n",
    )
    .unwrap();
    let mut p = json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/policy","package_name":"policy","source_objects":[file.object().id]});
    seal(&mut p, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut t = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});
    seal(&mut t, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut v = json!({"schema":"mpk.go.type_components.v2","source_objects":[file.object(),module.object()],"packages":[p.clone()],"declarations":[],"types":[t.clone()],"underlying":[],"aliases":[],"package_roots":[p["id"]],"type_roots":[t["id"]]});
    sort(&mut v["source_objects"], "id");
    seal(&mut v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    retain(
        &format!("{label}-before.json"),
        &json!({"components":v,"sources":[{"object":file.object(),"whole_original_bytes":file.bytes()},{"object":module.object(),"whole_original_bytes":module.bytes()}]}),
    );
    import_go_type_key_components(&canonical(&v), &[file, module]).unwrap()
}
fn slots(signature: &GoSourceSignature) -> Value {
    let parsed = signature
        .types()
        .source(signature.source_object_id())
        .unwrap();
    json!(signature.slots().iter().map(|s|{
  let field=&parsed.nodes()[s.field_node()];
  let name=s.name_node().map(|n|&parsed.nodes()[n]);
  json!({"role":s.role().as_str(),"field_start_byte":field.start(),"field_end_byte":field.end(),"name":s.name(),"name_start_byte":name.map(|n|n.start()),"name_end_byte":name.map(|n|n.end()),"type_id":s.type_id()})
 }).collect::<Vec<_>>())
}
#[test]
fn matches_actual_original_signatures_and_declared_bindings() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus-binders.json")).unwrap()).unwrap();
    retain("original-before.json", &corpus);
    let wire = canonical(&corpus["components"]);
    let sources = corpus["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &canonical(&s["object"]),
                &s["whole_original_bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_u64().unwrap() as u8)
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(&wire, &sources).unwrap();
    let graph = build_go_source_scopes(&keys).unwrap();
    let mut ok = 0;
    let mut outside = 0;
    for (i, row) in corpus["observations"]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let source = row["source_object_id"].as_str().unwrap();
        let parsed = graph.source(source).unwrap();
        let kind = row["syntax_kind"]
            .as_str()
            .unwrap()
            .strip_prefix("*ast.")
            .unwrap();
        let node = parsed
            .nodes()
            .iter()
            .position(|n| {
                n.kind() == kind
                    && n.start() as u64 == row["start_byte"].as_u64().unwrap()
                    && n.end() as u64 == row["end_byte"].as_u64().unwrap()
            })
            .unwrap();
        let actual = derive_go_source_signature(&graph, source, node);
        let expected = row["expected_reader_error"].as_str().unwrap();
        let result = if expected.is_empty() {
            let sig = actual.unwrap();
            assert_eq!(sig.name(), row["name"].as_str());
            assert_eq!(slots(&sig), row["slots"], "function {i}: {}", row["name"]);
            ok += 1;
            assert_eq!(sig.function_node(), node);
            assert_eq!(
                sig.types().source(source).unwrap().source().object().id,
                source
            );
            json!({"slots":slots(&sig),"body_node":sig.body_node(),"signature_type_node":sig.type_node(),"whole_body_or_program_authority":false})
        } else {
            let error = actual.unwrap_err();
            assert_eq!(
                format!("{error:?}"),
                expected,
                "function {i}: {}",
                row["name"]
            );
            outside += 1;
            json!({"error":format!("{error:?}")})
        };
        retain(&format!("function-{i:03}-actual.json"), &result);
    }
    let mut declared = 0;
    let mut inferred = 0;
    let mut unsupported = 0;
 let mut type_only=0;
    for (i, row) in corpus["observations"]["variables"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let source = row["source_object_id"].as_str().unwrap();
        let parsed = graph.source(source).unwrap();
        let binding = graph
            .bindings()
            .iter()
            .find(|b| {
                b.source_object_id() == source
                    && parsed.nodes()[b.name_node()].start() as u64
                        == row["start_byte"].as_u64().unwrap()
                    && b.name() == row["name"].as_str().unwrap()
            });
        if row["lexical_binding"]==false {assert!(binding.is_none());type_only+=1;retain(&format!("variable-{i:03}-actual.json"),&json!({"type_only_function_parameter":true,"lexical_value_binding":false}));continue;}
        let binding=binding.unwrap_or_else(||panic!("missing original binding {i} {row}"));
        let actual = derive_go_declared_binding_type(&graph, binding);
        let expected = row["expected_reader_error"].as_str().unwrap();
        let result = if expected.is_empty() {
            let typ = actual.unwrap();
            assert_eq!(
                typ.type_id(),
                row["type_id"].as_str().unwrap(),
                "variable {i}: {}",
                row["name"]
            );
            assert_eq!(typ.binding(), binding);
            assert!(!typ
                .types()
                .source(source)
                .unwrap()
                .spelling(typ.type_node())
                .unwrap()
                .is_empty());
            declared += 1;
            json!({"type_id":typ.type_id(),"type_node":typ.type_node(),"initializer_or_body_or_program_authority":false})
        } else {
            let error = actual.unwrap_err();
            assert_eq!(
                format!("{error:?}"),
                expected,
                "variable {i}: {}",
                row["name"]
            );
            if error == GoSourceSignatureError::InferenceRequired {
                inferred += 1
            } else {
                unsupported += 1
            }
            json!({"error":format!("{error:?}")})
        };
        retain(&format!("variable-{i:03}-actual.json"), &result);
    }
    retain(
        "summary.json",
        &json!({"accepted_function_signatures":ok,"explicit_valid_Go_outside_signature_domain":outside,"declared_bindings":declared,"type_only_parameter_names":type_only,"inferred_bindings_require_expression_typing":inferred,"explicit_valid_Go_outside_value_type_domain":unsupported}),
    );
    assert_eq!(type_only,1);
    assert!(ok > 20 && outside >= 3 && declared > 200 && inferred >= 5);
    drop(graph);
    drop(keys);
    drop(sources);
}
fn named_signature(
    graph: &GoSourceScopes,
    name: &str,
) -> Result<GoSourceSignature, GoSourceSignatureError> {
    let b = graph.bindings().iter().find(|b| b.name() == name).unwrap();
    derive_go_source_signature(graph, b.source_object_id(), b.declaration_node())
}
#[test]
fn declared_binding_annotation_is_separate_from_body_and_other_slots() {
    let keys=candidate("annotation-boundary","package policy\ntype Word uint16\nfunc F(keep Word,p *Word)(out Word){var wrong Word=\"bad\"; var noType=1; short:=keep;_,_,_=wrong,noType,short;return}\n");
    let graph = build_go_source_scopes(&keys).unwrap();
    assert_eq!(
        named_signature(&graph, "F"),
        Err(GoSourceSignatureError::Type(GoSourceTypeError::Unsupported))
    );
    for name in ["keep", "out", "wrong"] {
        let b = graph.bindings().iter().find(|b| b.name() == name).unwrap();
        let actual = derive_go_declared_binding_type(&graph, b).unwrap();
        retain(
            &format!("annotation-{name}-actual.json"),
            &json!({"type_id":actual.type_id(),"initializer_or_body_checked":false}),
        );
    }
    for name in ["noType", "short"] {
        let b = graph.bindings().iter().find(|b| b.name() == name).unwrap();
        assert_eq!(
            derive_go_declared_binding_type(&graph, b),
            Err(GoSourceSignatureError::InferenceRequired)
        );
    }
    let nonvariable = graph
        .bindings()
        .iter()
        .find(|b| b.name() == "Word")
        .unwrap();
    assert_eq!(
        derive_go_declared_binding_type(&graph, nonvariable),
        Err(GoSourceSignatureError::NotVariable)
    );
    let other = candidate(
        "foreign-annotation-owner",
        "package policy\nfunc G(x byte){}\n",
    );
    let other_graph = build_go_source_scopes(&other).unwrap();
    let binding = other_graph
        .bindings()
        .iter()
        .find(|b| b.name() == "x")
        .unwrap();
    assert_eq!(
        derive_go_declared_binding_type(&graph, binding),
        Err(GoSourceSignatureError::Locator)
    );
    retain(
        "annotation-boundary-actual.json",
        &json!({"invalid_initializer_is_not_certified":true,"unrelated_pointer_slot_does_not_change_own_annotation":true,"inferred_bindings_remain_untyped":true,"foreign_bindings_rejected":true}),
    );
}
#[test]
fn original_signature_limits_and_owned_slots() {
    for count in [0, 16, 17] {
        let results = (0..count)
            .map(|i| format!("r{i} byte"))
            .collect::<Vec<_>>()
            .join(",");
        let suffix = if count == 0 {
            String::new()
        } else {
            format!("({results})")
        };
        let text = format!("package policy\nfunc F(){suffix}{{return}}\n");
        let label = format!("results-{count}");
        let keys = candidate(&label, &text);
        let graph = build_go_source_scopes(&keys).unwrap();
        let actual = named_signature(&graph, "F");
        assert_eq!(actual.is_ok(), count <= 16);
        if count <= 16 {
            let sig = actual.unwrap();
            drop(graph);
            drop(keys);
            assert_eq!(sig.slots().len(), count);
            assert!(!sig
                .types()
                .source(sig.source_object_id())
                .unwrap()
                .source()
                .bytes()
                .is_empty());
            retain(&format!("{label}-actual.json"), &slots(&sig));
        } else {
            assert_eq!(actual.unwrap_err(), GoSourceSignatureError::Limit);
        }
    }
    let params = (0..70)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let keys = candidate(
        "params-70",
        &format!("package policy\nfunc F({params} byte){{}}\n"),
    );
    let graph = build_go_source_scopes(&keys).unwrap();
    let sig = named_signature(&graph, "F").unwrap();
    assert_eq!(sig.slots().len(), 70);
    assert_eq!(sig.types().roots().len(), 1);
    retain("params-70-actual.json", &slots(&sig));
    assert_eq!(
        derive_go_source_signature(&graph, "missing", 0),
        Err(GoSourceSignatureError::Locator)
    );
    let source = sig.source_object_id();
    let parsed = graph.source(source).unwrap();
    let not_function = parsed.root();
    assert_eq!(
        derive_go_source_signature(&graph, source, not_function),
        Err(GoSourceSignatureError::NotFunction)
    );
    for (label, text, error) in [
        (
            "variadic",
            "package policy\nfunc F(x ...byte){}\n",
            GoSourceSignatureError::Unsupported,
        ),
        (
            "generic",
            "package policy\nfunc F[T any](x T){}\n",
            GoSourceSignatureError::Unsupported,
        ),
        (
            "pointer",
            "package policy\nfunc F(x *byte){}\n",
            GoSourceSignatureError::Type(GoSourceTypeError::Unsupported),
        ),
    ] {
        let keys = candidate(label, text);
        let scopes = build_go_source_scopes(&keys).unwrap();
        let actual = named_signature(&scopes, "F").unwrap_err();
        assert_eq!(actual, error);
        retain(
            &format!("{label}-actual.json"),
            &json!({"error":format!("{actual:?}")}),
        );
    }
}

#[test]
fn bare_signature_names_and_missing_bodies_supply_no_call_authority() {
    let keys = candidate(
        "bare-duplicate",
        "package policy\nvar Fn func(x byte)(x byte)\n",
    );
    let scopes = build_go_source_scopes(&keys).unwrap();
    let source = scopes
        .bindings()
        .iter()
        .find(|b| b.name() == "Fn")
        .unwrap()
        .source_object_id();
    let parsed = scopes.source(source).unwrap();
    let typ = parsed
        .nodes()
        .iter()
        .position(|n| n.kind() == "FuncType")
        .unwrap();
    assert_eq!(
        derive_go_source_signature(&scopes, source, typ),
        Err(GoSourceSignatureError::Duplicate)
    );
    let keys = candidate("opaque-body", "package policy\nfunc F(x byte) byte\n");
    let scopes = build_go_source_scopes(&keys).unwrap();
    let sig = named_signature(&scopes, "F").unwrap();
    assert!(sig.body_node().is_none());
    assert_eq!(sig.slots().len(), 2);
    retain(
        "opaque-body-actual.json",
        &json!({"slots":slots(&sig),"body_available":false,"call_or_program_authority":false}),
    );
}

#[test]
fn receiver_base_requires_an_original_local_nominal_type() {
 let keys=candidate("builtin-receiver","package policy\nfunc (x uint16) Bad(){}\n");let graph=build_go_source_scopes(&keys).unwrap();let binding=graph.bindings().iter().find(|b|b.name()=="x").unwrap();let error=derive_go_source_signature(&graph,binding.source_object_id(),binding.declaration_node()).unwrap_err();assert_eq!(error,GoSourceSignatureError::Unsupported);let annotation=derive_go_declared_binding_type(&graph,binding).unwrap();retain("builtin-receiver-actual.json",&json!({"signature_error":format!("{error:?}"),"own_declared_annotation":annotation.type_id(),"method_or_program_authority":false}));
}
