//! Original exceptional contracts reuse finite exception functions and guards.
use super::*;
const BUILTINS: [&str; 9] = [
    "System.DivideByZeroException",
    "System.OverflowException",
    "System.IndexOutOfRangeException",
    "System.ArgumentException",
    "System.ArgumentOutOfRangeException",
    "System.ArgumentNullException",
    "System.InvalidOperationException",
    "System.NullReferenceException",
    "System.Runtime.CompilerServices.SwitchExpressionException",
];
fn source_id() -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"DomainCases","owner":"","name":"Fault","parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn exception() -> J {
    J::object(vec![
        ("tag", J::string("variable")),
        ("type_id", J::string(ty("exception"))),
        ("binding_id", J::string("exception")),
    ])
}
fn is_type(id: &str) -> J {
    J::object(vec![
        ("tag", J::string("exception_is")),
        ("type_id", J::string(ty("bool"))),
        ("value", exception()),
        ("exception_type_id", J::string(id)),
    ])
}
fn clauses(payload: bool) -> Vec<(String, J)> {
    let mut rows = BUILTINS
        .iter()
        .map(|id| (id.to_string(), is_type(id)))
        .collect::<Vec<_>>();
    if payload {
        let id = source_id();
        rows.push((id.clone(), is_type(&id)));
        for (name, primitive) in [("Code", "i32"), ("Flag", "bool"), ("Unit", "char")] {
            let member = csharp_practical_stored_member_id(
                &id,
                name,
                &json!({"kind":"primitive","id":primitive}),
                "readonly_field",
            )
            .unwrap();
            let read = J::object(vec![
                ("tag", J::string("exception_payload")),
                ("type_id", J::string(ty(primitive))),
                ("value", exception()),
                ("member_id", J::string(member)),
            ]);
            let clause = J::object(vec![
                ("tag", J::string("let")),
                ("type_id", J::string(ty("bool"))),
                ("binding_id", J::string("observed")),
                ("value", read),
                ("body", truth()),
            ]);
            rows.push((name.into(), clause.clone()));
            if name == "Code" {
                rows.push((
                    "guarded".into(),
                    J::object(vec![
                        ("tag", J::string("conditional")),
                        ("type_id", J::string(ty("bool"))),
                        ("condition", is_type(&id)),
                        ("when_true", clause),
                        ("when_false", truth()),
                    ]),
                ));
            }
        }
    }
    rows
}
fn exception_requests() -> Value {
    let originals = read("exception-vc/requests.json");
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::MethodContract,
        originals[0]["inputs"][0]["utf8"]
            .as_str()
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    let domain = read("ordinary-foundation/exception-domain-sources/requests.json");
    let mut rows = vec![];
    for payload in [false, true] {
        let request = if payload {
            domain
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == "exception-payload")
                .unwrap()
        } else {
            &originals[0]
        };
        let source = request["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
            .unwrap()["utf8"]
            .as_str()
            .unwrap();
        let root = request["roots"][0].as_str().unwrap();
        let scope = if payload {
            source_id()
        } else {
            "System.ArgumentException".into()
        };
        let (context, captures) =
            support::context_with_sidecar(&b(), root, source.as_bytes(), |context| {
                let J::Object(mut fields) = template.clone() else {
                    panic!()
                };
                fields.retain(|(k, _)| k != "contract_sha256");
                for (key, value) in &mut fields {
                    match key.as_str() {
                        "semantic_context" => *value = context.semantic_context().clone(),
                        "callable_id" => *value = J::string(root),
                        "source_content_sha256" => {
                            *value = J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                        }
                        "exceptional_cases" => {
                            *value = J::Array(vec![J::object(vec![
                                ("exception_type_id", J::string(&scope)),
                                ("path_condition", truth()),
                                (
                                    "ensures",
                                    J::Array(
                                        clauses(payload).into_iter().map(|(_, e)| e).collect(),
                                    ),
                                ),
                            ])])
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
        rows.push(json!({"id":if payload{"exception-payload"}else{"exception-builtins"},"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    json!(rows)
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_EXCEPTION_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/exception-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_exception_clause_requests() {
    let requests = exception_requests();
    if let Some(path) = std::env::var_os("MPK_W09_EXCEPTION_CLAUSES_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/exception-clauses/requests.json"),
            requests
        );
    }
}
fn matches(tag: u32, target: &str, payload: bool) -> bool {
    let exact = BUILTINS.get(tag as usize).is_some_and(|id| *id == target)
        || (payload && tag == 9 && target == source_id());
    exact
        || target == "System.ArgumentException" && matches!(tag, 4 | 5)
        || target == "System.InvalidOperationException" && tag == 8
}
#[test]
fn csharp_03_t06_w09_exception_clauses_original_source() {
    let bundle = b();
    let requests = exception_requests();
    let responses: Value =
        if let Some(path) = std::env::var_os("MPK_W09_EXCEPTION_CLAUSES_RESPONSES") {
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
        } else {
            read("ordinary-foundation/exception-clauses/responses.json")
        };
    let mut observations = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let payload = id == "exception-payload";
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
        let finite = generate_csharp_practical_ordinary_finite_operations(vir).unwrap();
        let fc = mpk_cert::decode_canonical_certificate(finite.certificate_bytes()).unwrap();
        let names = fc
            .declarations
            .iter()
            .map(|d| fc.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&fc, &c, &names).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let depth = types[&ty("exception")].depth;
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| d.tag == "exception_payload")
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(recipes.len(), if payload { 3 } else { 0 });
        for seed in [0, 1, 17] {
            let value = payload.then(|| {
                relation_tests::sample(
                    &source_id(),
                    seed,
                    &types,
                    &response["facts"],
                    emitted.closure().closed(),
                )
            });
            for tag in (0..11).chain([1 << 31, u32::MAX]) {
                let exception = MonomorphicValue::ClosedException {
                    type_id: ty("exception"),
                    tag,
                    source_type_id: payload.then(source_id),
                    payload: value.clone().map(Box::new),
                };
                let raw = relation_tests::storage(&exception, &types);
                let input = V::Cube(raw);
                for (label, expression) in clauses(payload) {
                    let mut hash = Sha256::new();
                    hash.update(b"MPK-CSHARP-CONTRACT-EXPRESSION-1.0\0");
                    hash.update(a::canonical_practical_json_bytes(&expression).unwrap());
                    let hash = format!("{:x}", hash.finalize());
                    let d = p
                        .definitions()
                        .iter()
                        .find(|d| d.expression_sha256 == hash)
                        .unwrap();
                    assert_eq!(
                        d.exception_scope.as_deref(),
                        Some(if payload {
                            source_id()
                        } else {
                            "System.ArgumentException".into()
                        })
                        .as_deref()
                    );
                    assert!(d
                        .subjects
                        .iter()
                        .any(|(name, _)| name == "current:exception"));
                    let args = d
                        .subjects
                        .iter()
                        .map(|(name, t)| {
                            if name == "current:exception" {
                                input.clone()
                            } else {
                                sparse_cube(types[t].depth, BTreeSet::new())
                            }
                        })
                        .collect::<Vec<_>>();
                    let partial = matches!(label.as_str(), "Code" | "Flag" | "Unit");
                    let defined = !partial || tag == 9;
                    assert_eq!(
                        bit(run(&c, &d.definedness_definition, args.clone())),
                        defined,
                        "{id} {label} tag{tag} definedness"
                    );
                    if defined {
                        let expected =
                            partial || label == "guarded" || matches(tag, &label, payload);
                        assert_eq!(
                            bit(run(&c, &d.value_definition, args)),
                            expected,
                            "{id} {label} tag{tag}"
                        );
                    }
                    observations += 1;
                }
                if let Some(MonomorphicValue::Product { fields, .. }) = &value {
                    for d in recipes.values() {
                        let params: Value = serde_json::from_str(&d.parameters).unwrap();
                        let member = params["member_id"].as_str().unwrap();
                        let OrdinaryShape::Product { fields: stored } = &types[&source_id()].shape
                        else {
                            panic!()
                        };
                        let field = &fields[stored.iter().position(|f| f.id == member).unwrap()];
                        let expected = relation_tests::storage(&field.value, &types);
                        let name = |s: &str| {
                            format!(
                                "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
                                s.as_bytes()
                                    .iter()
                                    .map(|b| format!("{b:02x}"))
                                    .collect::<String>()
                            )
                        };
                        // Definedness is checked through the full W03 clauses
                        // above; its symbol aliases the existing active-arm
                        // predicate rather than adding a second global.
                        let result = run(&c, &name(&d.name), vec![input.clone()]);
                        for (index, expected) in expected.into_iter().enumerate() {
                            let mut leaf = result.clone();
                            for k in 0..types[&d.result_type].depth {
                                leaf = apply(&c, leaf, V::Bit(index & (1 << k) != 0));
                            }
                            assert_eq!(
                                bit(leaf),
                                tag == 9 && expected,
                                "{id} {member} tag{tag} bit{index}"
                            );
                            observations += 1;
                        }
                    }
                }
                assert!(depth >= 5);
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
        let row = changed["definitions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|d| !d["exception_scope"].is_null())
            .unwrap();
        row["exception_scope"] = json!("changed");
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
    eprintln!("exception contract scope/guard/storage observations: {observations}");
}
