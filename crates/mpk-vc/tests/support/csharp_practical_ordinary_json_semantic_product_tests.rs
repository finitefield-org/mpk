//! Original-source bindings for two Money instances and an ordered entry.
use super::*;
#[path = "csharp_practical_ordinary_json_semantic_product_runtime.rs"]
mod runtime;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/csharp-03/ordinary-foundation/json-semantic-product-sources",
    )
}
fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn requests() -> Value {
    let code="namespace Boundary;public enum Currency{None=0,Usd=1,Jpy=7}public readonly struct EnumMoney{public readonly decimal Amount;public readonly Currency Currency;}public readonly struct TextMoney{public readonly decimal Amount;public readonly string Currency;}public readonly struct Pair{public readonly int Key;public readonly bool Value;}public readonly struct Payload{public readonly EnumMoney EnumValue;public readonly TextMoney TextValue;public readonly Pair Entry;}public static class Entry{public static Payload Run(Payload p){return p;}}\n";
    let mut requests = super::source_tests::requests_for(&[("money-entry", code)]);
    let bundle = b();
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source_hash = captures
        .entries()
        .iter()
        .find(|e| e.kind() == OriginalInputKind::Source)
        .unwrap()
        .raw_sha256();
    let mut bindings = vec![];
    for (source, role, members, arguments) in [
        (
            "EnumMoney",
            "money",
            vec![
                (
                    "amount",
                    "Amount",
                    json!({"kind":"primitive","id":"decimal"}),
                ),
                (
                    "currency",
                    "Currency",
                    json!({"kind":"source","id":source_id("Currency")}),
                ),
            ],
            vec![source_id("Currency")],
        ),
        (
            "TextMoney",
            "money",
            vec![
                (
                    "amount",
                    "Amount",
                    json!({"kind":"primitive","id":"decimal"}),
                ),
                (
                    "currency",
                    "Currency",
                    json!({"kind":"primitive","id":"string"}),
                ),
            ],
            vec!["mpk.csharp.value.string.v1".into()],
        ),
        (
            "Pair",
            "ordered_entry",
            vec![
                ("key", "Key", json!({"kind":"primitive","id":"i32"})),
                ("value", "Value", json!({"kind":"primitive","id":"bool"})),
            ],
            vec![
                "mpk.csharp.value.i32.v1".into(),
                "mpk.csharp.value.bool.v1".into(),
            ],
        ),
    ] {
        let id = source_id(source);
        bindings.push(SemanticBindingInput {
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
            bounds: vec![],
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        });
    }
    let sidecar = build_semantic_bindings(&context, &captures, bindings).unwrap();
    let text = std::str::from_utf8(sidecar.canonical_bytes()).unwrap();
    let inputs = requests[0]["inputs"].as_array_mut().unwrap();
    inputs.push(json!({"kind":"sidecar","path":"contracts/data.json","utf8":text}));
    inputs.sort_by_key(|i| i["path"].as_str().unwrap().to_owned());
    requests
}
#[test]
fn csharp_03_t06_w09_json_semantic_product_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_SEMANTIC_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}

fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_SEMANTIC_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-semantic-products"))
}

#[test]
fn csharp_03_t06_w09_json_semantic_product_source() {
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    let bundle = b();
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let facts = &responses[0]["facts"];
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
    assert_eq!(p.products().len(), 7);
    assert_eq!(p.enums().len(), 1);
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let mut semantic = 0;
    for product in p.products() {
        assert_eq!(
            &product.carrier,
            layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == product.carrier.type_id)
                .unwrap()
        );
        assert!(!p.deferred_type_ids().contains(&product.carrier.type_id));
        if let Some(template) = &product.template_id {
            semantic += 1;
            let entry = emitted
                .closure()
                .closed()
                .entries()
                .iter()
                .find(|e| e["instance_id"] == product.carrier.type_id)
                .unwrap();
            assert_eq!(&entry["template_id"], template);
            let expected = if template.ends_with(".money.v1") {
                vec!["amount", "currency"]
            } else {
                assert!(template.ends_with(".ordered_entry.v1"));
                vec!["key", "value"]
            };
            assert_eq!(product.member_names, expected);
            assert_eq!(product.member_ids, product.member_names);
            let repr = &entry["type_definition"]["representation"];
            assert_eq!(repr["kind"], "product");
            assert_eq!(
                repr["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|f| f[0].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        } else {
            let ty = facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == product.carrier.type_id)
                .unwrap();
            assert_eq!(
                product.member_names,
                ty["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m["name"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            );
        }
    }
    assert_eq!(semantic, 3);
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
    let index = p
        .products()
        .iter()
        .position(|d| d.template_id.is_some())
        .unwrap();
    for (key, value) in [
        ("template_id", json!("forged")),
        ("member_names", json!(["currency", "amount"])),
        ("member_ids", json!(["forged"])),
    ] {
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        forged["products"][index][key] = value;
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let row = json!({"id":"money-entry","program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
    let bytes = serde_json::to_vec_pretty(&row).unwrap();
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    let out = pins();
    if std::env::var_os("MPK_W09_JSON_SEMANTIC_OUT").is_some() {
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("certificate.json"), bytes).unwrap();
        fs::write(out.join("money-entry.hex"), hex).unwrap();
    } else {
        assert_eq!(fs::read(out.join("certificate.json")).unwrap(), bytes);
        assert_eq!(
            fs::read_to_string(out.join("money-entry.hex")).unwrap(),
            hex
        );
    }
    eprintln!("JSON semantic products: two Money, one ordered entry and four source products; {} terms, {} declarations",cert.term_table.len(),cert.declarations.len());
}
