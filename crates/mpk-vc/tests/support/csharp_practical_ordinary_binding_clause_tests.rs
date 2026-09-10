//! Forward source-binding contracts retain the complete existing conversion.
use super::*;
fn requests() -> Value {
    let row = read("ordinary-foundation/transition-clauses/requests.json")[0].clone();
    let text = row["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["path"] == "contracts/method.json")
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::MethodContract,
        text.as_bytes(),
    )
    .unwrap();
    let bundle = b();
    let mut requests = vec![];
    for (id, row, facts) in projection_tests::projection_sources() {
        if !(id.starts_with("binding-vc-")
            || [
                "remapped-boundary-sequence",
                "extra-bool-float-map",
                "extra-bool-nullable-map",
            ]
            .contains(&id.as_str()))
        {
            continue;
        }
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let root = row["roots"][0].as_str().unwrap();
        let callable = facts["callables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == root)
            .unwrap();
        let source_id = callable["identity"]["parameter_type_ids"][0]
            .as_str()
            .unwrap();
        let projection = emitted
            .vir()
            .binding_projections()
            .iter()
            .find(|p| p.source_type_id == source_id && p.binding_id != "binding.identity")
            .unwrap_or_else(|| panic!("{id}: missing root binding"));
        let variable = J::object(vec![
            ("tag", J::string("variable")),
            ("type_id", J::string(source_id)),
            ("binding_id", J::string("parameter:0")),
        ]);
        let project = J::object(vec![
            ("tag", J::string("source_project")),
            ("type_id", J::string(&projection.semantic_type_id)),
            ("binding_id", J::string(&projection.binding_id)),
            ("source_value", variable),
        ]);
        // Equality is deliberately observable: a projected NaN is not reflexive.
        let clause = J::object(vec![
            ("tag", J::string("structural_equal")),
            ("type_id", J::string(ty("bool"))),
            ("left", project.clone()),
            ("right", project),
        ]);
        let source = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
            .unwrap()["utf8"]
            .as_str()
            .unwrap();
        let binding = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| {
                serde_json::from_str::<Value>(i["utf8"].as_str().unwrap())
                    .is_ok_and(|v| v["schema"] == "mpk.csharp.semantic_bindings.v1")
            })
            .unwrap();
        let (context, captures) = support::context_with_sidecars(
            &bundle,
            root,
            source.as_bytes(),
            vec![
                binding["path"].as_str().unwrap().into(),
                "contracts/method.json".into(),
            ],
            |context| {
                let J::Object(mut fields) = template.clone() else {
                    panic!()
                };
                fields.retain(|(k, _)| k != "contract_sha256");
                for (k, v) in &mut fields {
                    match k.as_str() {
                        "semantic_context" => *v = context.semantic_context().clone(),
                        "compilation_id" => *v = J::string(context.compilation_id()),
                        "callable_id" => *v = J::string(root),
                        "source_content_sha256" => {
                            *v = J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                        }
                        "ensures" => *v = J::Array(vec![clause.clone()]),
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
                    binding["utf8"].as_str().unwrap().as_bytes().to_vec(),
                    a::canonical_practical_json_bytes(&J::Object(fields)).unwrap(),
                ]
            },
        );
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    assert_eq!(requests.len(), 15);
    json!(requests)
}
#[test]
fn csharp_03_t06_w09_binding_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_BINDING_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_BINDING_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-clauses");
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
fn csharp_03_t06_w09_binding_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_BINDING_CLAUSE_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/binding-clauses/responses.json")
    };
    let mut observations = 0;
    let mut non_reflexive = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
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
        let p = generate_csharp_practical_ordinary_contract_expressions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        metadata(&p, &vc);
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| d.tag == "source_project")
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(recipes.len(), 1);
        let recipe = recipes.values().next().unwrap();
        let full = generate_csharp_practical_ordinary_binding_projections(vir).unwrap();
        let sc = mpk_cert::decode_canonical_certificate(full.certificate_bytes()).unwrap();
        let binding_id: Value = serde_json::from_str(&recipe.parameters).unwrap();
        let projection = full
            .definitions()
            .iter()
            .find(|d| {
                d.projection.binding_id == binding_id["binding_id"].as_str().unwrap()
                    && d.source_carrier.type_id == recipe.argument_types[0]
                    && d.semantic_carrier.type_id == recipe.result_type
            })
            .unwrap();
        let declaration = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == core_name(&recipe.name))
            .unwrap();
        let mpk_cert::encode::DeclarationKind::Def { value, .. } = declaration.kind else {
            panic!()
        };
        let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize] else {
            panic!()
        };
        assert_eq!(
            c.name_table[c.declarations[global as usize].name as usize],
            projection.project_definition
        );
        structural_equivalence_tests::same_definition_closure(
            &sc,
            &c,
            &BTreeSet::from([projection.project_definition.clone()]),
        )
        .unwrap();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = projection_tests::ProjectionOracle {
            facts: &response["facts"],
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        for seed in 0..4 {
            let source = relation_tests::sample(
                &recipe.argument_types[0],
                seed,
                &types,
                &response["facts"],
                oracle.closed,
            );
            let expected = oracle.project(&source, &recipe.result_type);
            let (depth, ones) = projection_tests::sparse_storage(&source, &types);
            let input = sparse_cube(depth, ones);
            let (depth, wanted) = projection_tests::sparse_storage(&expected, &types);
            let out = run(&c, &core_name(&recipe.name), vec![input.clone()]);
            let mut probes = if depth <= 10 {
                (0..1usize << depth).collect::<BTreeSet<_>>()
            } else {
                BTreeSet::from([0, (1usize << depth) - 1])
            };
            probes.extend((0..depth).map(|i| 1usize << i));
            for &at in &wanted {
                probes.extend([
                    at,
                    at.saturating_sub(1),
                    (at + 1).min((1usize << depth) - 1),
                ]);
            }
            for at in probes {
                let mut bit_value = out.clone();
                for i in 0..depth {
                    bit_value = apply(&c, bit_value, V::Bit(at & (1 << i) != 0));
                }
                assert_eq!(
                    bit(bit_value),
                    wanted.contains(&at),
                    "{id} seed{seed} bit{at}"
                );
                observations += 1;
            }
            if seed == 1 {
                let model = generate_structural_program(
                    &bundle,
                    emitted.closure().roots(),
                    oracle.closed,
                    &recipe.result_type,
                )
                .unwrap();
                let expected = model.structural_equal(&expected, &expected).unwrap();
                non_reflexive += usize::from(!expected);
                assert_eq!(p.definitions().len(), 1);
                let clause = &p.definitions()[0];
                let args = clause
                    .subjects
                    .iter()
                    .map(|(name, ty)| {
                        if name.ends_with("parameter:0") {
                            assert_eq!(ty, &recipe.argument_types[0]);
                            input.clone()
                        } else {
                            sparse_cube(types[ty].depth, BTreeSet::new())
                        }
                    })
                    .collect::<Vec<_>>();
                assert!(bit(run(&c, &clause.definedness_definition, args.clone())));
                assert_eq!(
                    bit(run(&c, &clause.value_definition, args)),
                    expected,
                    "{id} source equality"
                );
                observations += 2;
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
        changed["definitions"][0]["result_type"] = json!("changed");
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
        eprintln!("binding clause {id} passed");
    }
    assert_eq!(non_reflexive, 1);
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    eprintln!("binding contract observations: {observations}; NaN source-contract false cases: {non_reflexive}");
}
