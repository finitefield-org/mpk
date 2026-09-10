//! Every admitted sum constructor through an original method contract.
use super::*;
fn scalar(token: &str, value: J) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", value),
    ])
}
fn make(instance: &str, arm: &str, payload: J) -> J {
    J::object(vec![
        ("tag", J::string("tagged_make")),
        ("type_id", J::string(instance)),
        ("semantic_instance_id", J::string(instance)),
        ("arm", J::string(arm)),
        ("payload", payload),
    ])
}
fn makers() -> Vec<J> {
    let bundle = b();
    let id = |role: &str, args: Vec<Value>| {
        csharp_practical_closed_instance_id(&bundle, &instance(role, args)).unwrap()
    };
    let mut makers = vec![];
    for (role, args, arms) in [
        (
            "option",
            vec![primitive("i32")],
            vec![("none", None), ("some", Some("i32"))],
        ),
        (
            "lookup",
            vec![primitive("i32")],
            vec![("missing_key", None), ("found", Some("i32"))],
        ),
        (
            "result",
            vec![primitive("i32"), primitive("bool")],
            vec![("ok", Some("i32")), ("error", Some("bool"))],
        ),
        (
            "validation",
            vec![primitive("i32"), primitive("i32")],
            vec![("valid", Some("i32")), ("invalid", Some("errors"))],
        ),
        (
            "boundary_field",
            vec![primitive("i32")],
            vec![("missing", None), ("null", None), ("value", Some("i32"))],
        ),
    ] {
        let instance = id(role, args);
        for (arm, payload) in arms {
            let payload = match payload {
                None => J::Null,
                Some("bool") => scalar("bool", J::Bool(true)),
                Some("errors") => J::object(vec![
                    ("tag", J::string("literal")),
                    (
                        "type_id",
                        J::string(id("bounded_sequence", vec![primitive("i32")])),
                    ),
                    ("value", J::Array(vec![J::string("1"), J::string("-7")])),
                ]),
                Some("i32") => scalar("i32", J::string("-7")),
                _ => unreachable!(),
            };
            makers.push(make(&instance, arm, payload));
        }
    }
    let option = id("option", vec![primitive("i32")]);
    let nested = id(
        "result",
        vec![
            instance("option", vec![primitive("i32")]),
            primitive("bool"),
        ],
    );
    for payload in [
        make(&option, "none", J::Null),
        make(&option, "some", scalar("i32", J::string("2147483647"))),
    ] {
        makers.push(make(&nested, "ok", payload));
    }
    makers
}
fn clauses() -> Vec<J> {
    makers()
        .into_iter()
        .map(|value| {
            let arm = value.get("arm").unwrap().clone();
            J::object(vec![
                ("tag", J::string("tagged_is")),
                ("type_id", J::string(ty("bool"))),
                ("value", value),
                ("arm", arm),
            ])
        })
        .collect()
}
fn requests() -> Value {
    let original = read("ordinary-foundation/json-sum-sources/requests.json")[0].clone();
    let source = original["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["path"] == "src/Entry.cs")
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let sidecar = |path: &str| {
        original["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["path"] == path)
            .unwrap()["utf8"]
            .as_str()
            .unwrap()
    };
    let binding = sidecar("contracts/data.json");
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::MethodContract,
        sidecar("contracts/method.json").as_bytes(),
    )
    .unwrap();
    let (context, captures) = support::context_with_sidecars(
        &b(),
        original["roots"][0].as_str().unwrap(),
        source.as_bytes(),
        vec!["contracts/data.json".into(), "contracts/method.json".into()],
        |context| {
            let J::Object(mut fields) = template.clone() else {
                panic!()
            };
            fields.retain(|(key, _)| key != "contract_sha256");
            for (key, value) in &mut fields {
                match key.as_str() {
                    "semantic_context" => *value = context.semantic_context().clone(),
                    "ensures" => *value = J::Array(clauses()),
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
            vec![
                binding.as_bytes().to_vec(),
                a::canonical_practical_json_bytes(&J::Object(fields)).unwrap(),
            ]
        },
    );
    json!([{"id":"tagged-make","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}
#[test]
fn csharp_03_t06_w09_tagged_make_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_TAGGED_MAKE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/tagged-make/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_TAGGED_MAKE_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/tagged-make");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn core_name(name: &str) -> String {
    format!(
        "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
#[test]
fn csharp_03_t06_w09_tagged_make_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_TAGGED_MAKE_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/tagged-make/responses.json")
    };
    let request = &requests[0];
    let response = &responses[0];
    assert_eq!(request["id"], response["id"]);
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
    let p = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
    let vc = generate_csharp_practical_vc(PracticalVcSource {
        artifact_context: &context,
        captured_inputs: &captures,
        vir,
    })
    .unwrap();
    metadata(&p, &vc);
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let types = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect::<BTreeMap<_, _>>();
    for d in p.definitions() {
        let args = d
            .subjects
            .iter()
            .map(|(_, ty)| sparse_cube(types[ty].depth, BTreeSet::new()))
            .collect::<Vec<_>>();
        assert!(bit(run(&c, &d.definedness_definition, args.clone())));
        assert!(bit(run(&c, &d.value_definition, args)));
    }
    assert_eq!(p.definitions().len(), 13);
    let recipes = vir
        .contract_expressions()
        .iter()
        .flat_map(|e| e.definitions())
        .filter(|d| d.tag == "tagged_make")
        .map(|d| (d.name.clone(), d.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(recipes.len(), 12);
    let structures = generate_csharp_practical_ordinary_structural(vir).unwrap();
    let storage_cert =
        mpk_cert::decode_canonical_certificate(structures.certificate_bytes()).unwrap();
    let mut dependencies = BTreeSet::new();
    let mut observations = 0;
    for d in recipes.values() {
        let params: Value = serde_json::from_str(&d.parameters).unwrap();
        let arm = params["arm"].as_str().unwrap();
        let structure = structures
            .definitions()
            .iter()
            .find(|s| s.carrier.type_id == d.result_type)
            .unwrap();
        let OrdinaryStructuralOperations::Sum { arms, .. } = &structure.operations else {
            panic!()
        };
        let constructor = &arms
            .iter()
            .find(|a| a.arm_id == arm)
            .unwrap()
            .make_definition;
        dependencies.insert(constructor.clone());
        let declaration = c
            .declarations
            .iter()
            .find(|row| c.name_table[row.name as usize] == core_name(&d.name))
            .unwrap();
        let mpk_cert::encode::DeclarationKind::Def { value, .. } = declaration.kind else {
            panic!()
        };
        let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize] else {
            panic!()
        };
        assert_eq!(
            &c.name_table[c.declarations[global as usize].name as usize],
            constructor
        );
        for seed in [0, 1, 17] {
            let payload = d.argument_types.first().map(|ty| {
                relation_tests::sample(
                    ty,
                    seed,
                    &types,
                    &response["facts"],
                    emitted.closure().closed(),
                )
            });
            let args = payload
                .as_ref()
                .map(|v| {
                    let raw = relation_tests::storage(v, &types);
                    vec![if raw.len() == 1 {
                        V::Bit(raw[0])
                    } else {
                        V::Cube(raw)
                    }]
                })
                .unwrap_or_default();
            let expected = MonomorphicValue::TaggedSum {
                type_id: d.result_type.clone(),
                arm: arm.into(),
                payload: payload.into_iter().collect(),
            };
            let raw = relation_tests::storage(&expected, &types);
            let out = run(&c, &core_name(&d.name), args);
            let indices: BTreeSet<usize> = if raw.len() <= 1024 {
                (0..raw.len()).collect()
            } else {
                raw.iter()
                    .enumerate()
                    .filter_map(|(i, &b)| b.then_some(i))
                    .chain(
                        (0..types[&d.result_type].depth)
                            .flat_map(|i| [1usize << i, (1usize << i) - 1]),
                    )
                    .chain([raw.len() - 1])
                    .collect()
            };
            for index in indices {
                let mut value = out.clone();
                for k in 0..types[&d.result_type].depth {
                    value = apply(&c, value, V::Bit(index & (1 << k) != 0));
                }
                assert_eq!(
                    bit(value),
                    raw[index],
                    "{} {arm} seed{seed} bit{index}",
                    d.result_type
                );
                observations += 1;
            }
        }
    }
    structural_equivalence_tests::same_definition_closure(&storage_cert, &c, &dependencies)
        .unwrap();
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
    changed["definitions"][0]["expression_sha256"] = json!("00".repeat(32));
    assert!(import_csharp_practical_ordinary_contract_expressions(
        &serde_json::to_vec(&changed).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    output("tagged-make.json", &p.canonical_bytes());
    output(
        "tagged-make.hex",
        p.certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            .as_bytes(),
    );
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    eprintln!("tagged constructor storage observations: {observations}");
}
