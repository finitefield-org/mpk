//! Original source map/set keys, canonical order and parser linkage.
use super::*;
#[path = "csharp_practical_ordinary_json_ordered_runtime.rs"]
mod runtime;
fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-ordered-sources")
}
fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_ORDERED_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-ordered"))
}
fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn requests() -> Value {
    let bundle = b();
    let mut all = vec![];
    for (id, key_cs, key_ty, key_id, extra) in [
        ("integer", "int", primitive("i32"), ty("i32"), ""),
        (
            "decimal",
            "decimal",
            primitive("decimal"),
            ty("decimal"),
            "",
        ),
        ("string", "string", primitive("string"), ty("string"), ""),
        (
            "compound",
            "Key",
            json!({"kind":"source","id":source_id("Key")}),
            source_id("Key"),
            "public readonly struct Key{public readonly string Name;public readonly int Code;}",
        ),
    ] {
        let code = format!("namespace Boundary;{extra}public readonly struct Pair{{public readonly {key_cs} Key;public readonly bool Value;}}public readonly struct MapRep{{public readonly Pair[] Items;}}public readonly struct SetRep{{public readonly {key_cs}[] Items;}}public readonly struct Payload{{public readonly MapRep Map;public readonly SetRep Set;}}public static class Entry{{public static Payload Run(Payload p){{return p;}}}}\n");
        let mut request = source_tests::requests_for(&[(id, &code)])[0].clone();
        let (context, captures) = support::replay_context(&bundle, &request);
        let source_hash = captures
            .entries()
            .iter()
            .find(|e| e.kind() == OriginalInputKind::Source)
            .unwrap()
            .raw_sha256();
        let mut bindings = vec![];
        for (name, role, members, arguments) in [
            (
                "Pair",
                "ordered_entry",
                vec![
                    ("key", "Key", key_ty.clone()),
                    ("value", "Value", primitive("bool")),
                ],
                vec![key_id.clone(), ty("bool")],
            ),
            (
                "MapRep",
                "ordered_map",
                vec![(
                    "entries",
                    "Items",
                    instance(
                        "bounded_sequence",
                        vec![json!({"kind":"source","id":source_id("Pair")})],
                    ),
                )],
                vec![key_id.clone(), ty("bool")],
            ),
            (
                "SetRep",
                "ordered_set",
                vec![(
                    "elements",
                    "Items",
                    instance("bounded_sequence", vec![key_ty.clone()]),
                )],
                vec![key_id.clone()],
            ),
        ] {
            let id = source_id(name);
            bindings.push(SemanticBindingInput {
                source_type_id: id.clone(),
                source_content_sha256: source_hash.into(),
                role: role.into(),
                member_map: members
                    .into_iter()
                    .map(|(role, name, ty)| SemanticBindingMember {
                        role: role.into(),
                        member_id: csharp_practical_stored_member_id(
                            &id,
                            name,
                            &ty,
                            "readonly_field",
                        )
                        .unwrap(),
                    })
                    .collect(),
                inferred_argument_ids: arguments,
                tag_arms: vec![],
                default_arm: "ineligible".into(),
                bounds: if role == "ordered_entry" {
                    vec![]
                } else {
                    vec![SemanticBound {
                        id: "length".into(),
                        maximum: 4096,
                    }]
                },
                operation_map: vec![],
                enum_arms: BTreeMap::new(),
            });
        }
        let sidecar = build_semantic_bindings(&context, &captures, bindings).unwrap();
        let inputs = request["inputs"].as_array_mut().unwrap();
        inputs.push(json!({"kind":"sidecar","path":"contracts/data.json","utf8":std::str::from_utf8(sidecar.canonical_bytes()).unwrap()}));
        inputs.sort_by_key(|i| i["path"].as_str().unwrap().to_owned());
        all.push(request);
    }
    json!(all)
}
#[test]
fn csharp_03_t06_w09_json_ordered_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_ORDERED_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}
#[test]
fn csharp_03_t06_w09_json_ordered_sources() {
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 4);
    assert_eq!(responses.as_array().unwrap().len(), 4);
    let bundle = b();
    let mut rows = vec![];
    let selected = std::env::var("MPK_W09_JSON_ORDERED_SELECT").ok();
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        if selected.as_deref().is_some_and(|s| s != id) {
            continue;
        }
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none(), "{id}: {response}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_json_products(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(p.collections().len(), 2, "{id}");
        assert_eq!(p.key_relations().len(), 1, "{id}");
        for c in p.collections() {
            assert!(!p.deferred_type_ids().contains(&c.sequence.carrier.type_id));
            assert_eq!(c.sequence.capacity, 4096);
            assert_eq!(c.key_type_id, p.key_relations()[0].carrier.type_id);
            assert_eq!(
                Some(&c.key_compare_definition),
                p.key_relations()[0].compare_definition.as_ref()
            );
        }
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_json_products(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for (pointer, value) in [
            ("/collections/0/key_compare_definition", json!("forged")),
            ("/collections/0/join_definition", json!("forged")),
            ("/collections/0/sequence/capacity", json!(4097)),
        ] {
            let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(import_csharp_practical_ordinary_json_products(
                &serde_json::to_vec(&bad).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
        let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
        assert_eq!(p.primitives(), values.definitions());
        for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            super::super::super::super::structural_equivalence_tests::same_definition_closure(
                &old, &cert, &names,
            )
            .unwrap();
        }
        let row = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
        eprintln!(
            "JSON ordered {id}: {} terms,{} declarations,{} static transformers",
            cert.term_table.len(),
            cert.declarations.len(),
            row["program"]["static_transformers"]
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        let out = pins();
        if std::env::var_os("MPK_W09_JSON_ORDERED_OUT").is_some() {
            fs::create_dir_all(&out).unwrap();
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(out.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(row);
    }
    assert_eq!(rows.len(), if selected.is_some() { 1 } else { 4 });
    if std::env::var_os("MPK_W09_JSON_ORDERED_OUT").is_some() {
        fs::write(
            pins().join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    } else {
        let pinned: Value =
            serde_json::from_slice(&fs::read(pins().join("certificates.json")).unwrap()).unwrap();
        let expected = pinned
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| selected.as_deref().is_none_or(|id| r["id"] == id))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(rows, expected);
    }
}
