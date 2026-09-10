//! Original-source bounded quantifiers, including universal body definedness.
use super::*;
fn variable(token: &str, name: &str) -> J {
    let name = match name {
        "lower" => "parameter:0",
        "upper" => "parameter:1",
        other => other,
    };
    J::object(vec![
        ("tag", J::string("variable")),
        ("type_id", J::string(ty(token))),
        ("binding_id", J::string(name)),
    ])
}
fn literal(token: &str, n: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", J::string(n)),
    ])
}
fn binary(token: &str, op: &str, left: J, right: J) -> J {
    J::object(vec![
        ("tag", J::string("binary")),
        (
            "type_id",
            J::string(ty(if op == "divide" { token } else { "bool" })),
        ),
        (
            "operation_id",
            J::string(format!("integer.{token}.{op}.checked")),
        ),
        ("left", left),
        ("right", right),
    ])
}
fn quantified(tag: &str, name: &str, lower: J, upper: J, body: J) -> J {
    J::object(vec![
        ("tag", J::string(tag)),
        ("type_id", J::string(ty("bool"))),
        ("binding_id", J::string(name)),
        ("lower", lower),
        ("upper", upper),
        ("body", body),
    ])
}
fn clauses(token: &str, oversized: bool) -> Vec<J> {
    let q = variable(token, "q");
    let lower = variable(token, "lower");
    let upper = variable(token, "upper");
    let quant = |tag: &str, body: J| quantified(tag, "q", lower.clone(), upper.clone(), body);
    if oversized {
        return vec![quantified(
            "bounded_forall",
            "q",
            literal(token, "0"),
            literal(token, "16385"),
            truth(),
        )];
    }
    vec![
        quant(
            "bounded_forall",
            binary(token, "less", q.clone(), upper.clone()),
        ),
        quant(
            "bounded_exists",
            binary(token, "equal", q.clone(), lower.clone()),
        ),
        quant(
            "bounded_exists",
            J::object(vec![
                ("tag", J::string("let")),
                ("type_id", J::string(ty("bool"))),
                ("binding_id", J::string("ignored")),
                (
                    "value",
                    binary(token, "divide", literal(token, "1"), q.clone()),
                ),
                ("body", truth()),
            ]),
        ),
        quant(
            "bounded_forall",
            quantified(
                "bounded_exists",
                "k",
                lower.clone(),
                upper.clone(),
                binary(token, "equal", variable(token, "k"), q),
            ),
        ),
        quantified(
            "bounded_forall",
            "q",
            literal(token, "7"),
            literal(token, "3"),
            truth(),
        ),
    ]
}
fn quantifier_requests() -> Value {
    let originals = read("exception-vc/requests.json");
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::MethodContract,
        originals[0]["inputs"][0]["utf8"]
            .as_str()
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    let mut rows = vec![];
    for (token, keyword, oversized) in [
        ("i32", "int", false),
        ("u32", "uint", false),
        ("i64", "long", false),
        ("u64", "ulong", false),
        ("i32", "int", true),
    ] {
        let source=format!("namespace Quantifiers;public static class Entry{{public static {keyword} Run({keyword} lower,{keyword} upper){{return lower;}}}}\n");
        let owner=csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Quantifiers","owner":"","name":"Entry","parameter_type_ids":[],"result_type_id":""})).unwrap();
        let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"Quantifiers","owner":owner,"name":"Run","parameter_type_ids":[ty(token),ty(token)],"result_type_id":ty(token)})).unwrap();
        let (context, captures) =
            support::context_with_sidecar(&b(), &root, source.as_bytes(), |context| {
                let J::Object(mut fields) = template.clone() else {
                    panic!()
                };
                fields.retain(|(k, _)| k != "contract_sha256");
                for (key, value) in &mut fields {
                    match key.as_str() {
                        "semantic_context" => *value = context.semantic_context().clone(),
                        "callable_id" => *value = J::string(&root),
                        "source_content_sha256" => {
                            *value = J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                        }
                        "ensures" => *value = J::Array(clauses(token, oversized)),
                        "requires" | "exceptional_cases" | "modifies" | "loops" => {
                            *value = J::Array(vec![])
                        }
                        _ => {}
                    }
                }
                let hash = mpk_vc::hash_domain_separated_raw(
                    a::METHOD_CONTRACT_HASH_DOMAIN,
                    &a::canonical_practical_json_bytes(&J::Object(fields.clone())).unwrap(),
                )
                .unwrap()
                .to_hex();
                fields.push(("contract_sha256".into(), J::string(hash)));
                a::canonical_practical_json_bytes(&J::Object(fields)).unwrap()
            });
        rows.push(json!({"id":if oversized{"oversized".into()}else{format!("quantifier-{token}")},"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    let public = total_clause_tests::requests_with_expressions(
        vec![
            quantified(
                "bounded_forall",
                "q",
                literal("i32", "0"),
                literal("i32", "2"),
                truth(),
            ),
            quantified(
                "bounded_exists",
                "q",
                literal("i32", "0"),
                literal("i32", "2"),
                truth(),
            ),
            quantified(
                "bounded_forall",
                "q",
                literal("i32", "7"),
                literal("i32", "3"),
                truth(),
            ),
        ],
        "quantifier-public",
    );
    rows.extend(public.as_array().unwrap().iter().cloned());
    json!(rows)
}
#[test]
fn csharp_03_t06_w09_quantifier_requests() {
    let requests = quantifier_requests();
    if let Some(path) = std::env::var_os("MPK_W09_QUANTIFIER_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/quantifiers/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_QUANTIFIER_SOURCE_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/quantifiers");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn word(value: i128, width: usize) -> V {
    V::Cube(
        (0..width)
            .map(|i| (value as u128) & (1 << i) != 0)
            .collect(),
    )
}
#[test]
fn csharp_03_t06_w09_quantifiers_original_source() {
    let bundle = b();
    let requests = quantifier_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_QUANTIFIER_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/quantifiers/responses.json")
    };
    let mut observations = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
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
        if id == "oversized" {
            assert_eq!(
                generate_csharp_practical_ordinary_contract_expressions(vir).unwrap_err(),
                OrdinaryCarrierError::Limit
            );
            continue;
        }
        if id == "quantifier-public" {
            let p = generate_csharp_practical_ordinary_source_clauses(vir).unwrap();
            assert_eq!(p.definitions().len(), 3);
            assert!(p
                .definitions()
                .iter()
                .all(|d| d.definedness_definition.is_some()));
            assert!(generate_csharp_practical_ordinary_public_domains(vir).is_err());
            assert!(generate_csharp_practical_ordinary_public_defaults(vir).is_err());
            assert!(generate_csharp_practical_ordinary_structural_public(vir).is_err());
            assert_eq!(
                import_csharp_practical_ordinary_source_clauses(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    vir
                )
                .unwrap(),
                p
            );
            let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            changed["definitions"][0]
                .as_object_mut()
                .unwrap()
                .remove("definedness_definition");
            assert!(import_csharp_practical_ordinary_source_clauses(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
            output(&format!("{id}.json"), &p.canonical_bytes());
            output(
                &format!("{id}.hex"),
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
                    .as_bytes(),
            );
            continue;
        }
        let token = id.strip_prefix("quantifier-").unwrap();
        let width = if token.ends_with("32") { 32 } else { 64 };
        let signed = token.starts_with('i');
        let min = if signed { -(1i128 << (width - 1)) } else { 0 };
        let max = if signed {
            (1i128 << (width - 1)) - 1
        } else {
            (1i128 << width) - 1
        };
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let p = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
        metadata(&p, &vc);
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let expressions = clauses(token, false);
        for (index, expression) in expressions.iter().enumerate() {
            let mut hash = Sha256::new();
            hash.update(b"MPK-CSHARP-CONTRACT-EXPRESSION-1.0\0");
            hash.update(a::canonical_practical_json_bytes(expression).unwrap());
            let hash = format!("{:x}", hash.finalize());
            let d = p
                .definitions()
                .iter()
                .find(|d| d.expression_sha256 == hash)
                .unwrap();
            for (lower, upper) in [
                (0, 0),
                (0, 2),
                (3, 5),
                (7, 3),
                (min, min + 2),
                (max - 2, max),
                (if signed { -1 } else { 0 }, 2),
                (0, 16385),
            ] {
                let valid = lower <= upper && upper - lower <= 16384;
                let defined = index != 4 && valid && (index != 2 || !(lower <= 0 && 0 < upper));
                let args = d
                    .subjects
                    .iter()
                    .map(|(name, _)| match name.as_str() {
                        "current:parameter:0" | "entry:parameter:0" => word(lower, width),
                        "current:parameter:1" | "entry:parameter:1" => word(upper, width),
                        "result" | "current:result" => word(lower, width),
                        _ => panic!("unexpected subject {name}"),
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    bit(run(&c, &d.definedness_definition, args.clone())),
                    defined,
                    "{id} clause{index} {lower}..{upper} definedness"
                );
                if defined {
                    assert_eq!(
                        bit(run(&c, &d.value_definition, args)),
                        if matches!(index, 1 | 2) {
                            lower < upper
                        } else {
                            true
                        },
                        "{id} clause{index} {lower}..{upper}"
                    );
                }
                observations += 1;
            }
        }
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        changed["definitions"][0]["subjects"][0][0] = json!("changed");
        assert!(import_csharp_practical_ordinary_contract_expressions(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
    }
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    eprintln!("original quantifier clause observations: {observations}");
}
