//! Transition contract projections preserve complete nominal field carriers.
use super::*;
fn source_type(name: &str) -> Value {
    let id=csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap();
    json!({"kind":"source","id":id})
}
fn parts(compound: bool) -> (String, Vec<(String, String, J)>) {
    let b = b();
    let state = if compound {
        source_type("State")
    } else {
        primitive("i32")
    };
    let event = if compound {
        source_type("Event")
    } else {
        primitive("i32")
    };
    let response = if compound {
        instance("option", vec![primitive("bool")])
    } else {
        primitive("bool")
    };
    let id = |t: &Value| {
        if t["kind"] == "primitive" {
            ty(t["id"].as_str().unwrap())
        } else if t["kind"] == "source" {
            t["id"].as_str().unwrap().to_string()
        } else {
            csharp_practical_closed_instance_id(&b, t).unwrap()
        }
    };
    let transition = id(&instance(
        "transition",
        vec![state.clone(), event.clone(), response.clone()],
    ));
    let raw_state = if compound {
        J::object(vec![
            ("Name", J::string("A😀")),
            ("Version", J::string("9")),
        ])
    } else {
        J::string("-2")
    };
    let raw_events = J::Array(
        [3, -1, 3]
            .into_iter()
            .map(|n| {
                if compound {
                    J::object(vec![("Code", J::string(n.to_string()))])
                } else {
                    J::string(n.to_string())
                }
            })
            .collect(),
    );
    let raw_response = if compound {
        J::object(vec![("tag", J::string("some")), ("payload", J::Bool(true))])
    } else {
        J::Bool(true)
    };
    (
        transition,
        vec![
            ("state".into(), id(&state), raw_state),
            (
                "events".into(),
                id(&instance("bounded_sequence", vec![event])),
                raw_events,
            ),
            ("response".into(), id(&response), raw_response),
        ],
    )
}
fn literal(ty: &str, value: J) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty)),
        ("value", value),
    ])
}
fn clauses(compound: bool) -> Vec<J> {
    let (instance, parts) = parts(compound);
    let raw = J::Object(
        parts
            .iter()
            .map(|(role, _, raw)| (role.clone(), raw.clone()))
            .collect(),
    );
    let value = literal(&instance, raw);
    parts
        .into_iter()
        .map(|(role, ty, raw)| {
            let projection = J::object(vec![
                ("tag", J::string(format!("transition_{role}"))),
                ("type_id", J::string(&ty)),
                ("value", value.clone()),
            ]);
            J::object(vec![
                ("tag", J::string("structural_equal")),
                ("type_id", J::string(super::ty("bool"))),
                ("left", projection),
                ("right", literal(&ty, raw)),
            ])
        })
        .collect()
}
fn requests() -> Value {
    let originals = read("ordinary-foundation/json-transition-sources/requests.json");
    let mut requests = vec![];
    for original in originals.as_array().unwrap() {
        let id = original["id"].as_str().unwrap();
        let compound = id == "compound";
        let input = |path: &str| {
            original["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["path"] == path)
                .unwrap()["utf8"]
                .as_str()
                .unwrap()
        };
        let source = input("src/Entry.cs");
        let binding = input("contracts/data.json");
        let template = a::parse_canonical_practical_json(
            a::PracticalArtifactKind::MethodContract,
            input("contracts/method.json").as_bytes(),
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
                fields.retain(|(k, _)| k != "contract_sha256");
                for (key, value) in &mut fields {
                    match key.as_str() {
                        "semantic_context" => *value = context.semantic_context().clone(),
                        "ensures" => *value = J::Array(clauses(compound)),
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
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    json!(requests)
}
#[test]
fn csharp_03_t06_w09_transition_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_TRANSITION_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/transition-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_TRANSITION_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/transition-clauses");
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
fn observe(c: &mpk_cert::Certificate, value: V, depth: u32, index: usize) -> bool {
    let mut value = value;
    for i in 0..depth {
        value = apply(c, value, V::Bit(index & (1 << i) != 0));
    }
    bit(value)
}
#[test]
fn csharp_03_t06_w09_transition_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value =
        if let Some(path) = std::env::var_os("MPK_W09_TRANSITION_CLAUSE_RESPONSES") {
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
        } else {
            read("ordinary-foundation/transition-clauses/responses.json")
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
        assert_eq!(p.definitions().len(), 3);
        for d in p.definitions() {
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| sparse_cube(types[ty].depth, BTreeSet::new()))
                .collect::<Vec<_>>();
            assert!(bit(run(&c, &d.definedness_definition, args.clone())));
            assert!(bit(run(&c, &d.value_definition, args)));
        }
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| d.tag.starts_with("transition_"))
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(recipes.len(), 3);
        let structural = generate_csharp_practical_ordinary_structural(vir).unwrap();
        let sc = mpk_cert::decode_canonical_certificate(structural.certificate_bytes()).unwrap();
        let mut dependencies = BTreeSet::new();
        for d in recipes.values() {
            let role = d.tag.strip_prefix("transition_").unwrap();
            let instance = &d.argument_types[0];
            let structure = structural
                .definitions()
                .iter()
                .find(|s| &s.carrier.type_id == instance)
                .unwrap();
            let OrdinaryStructuralOperations::Product { operations } = &structure.operations else {
                panic!()
            };
            let getter = &operations
                .fields
                .iter()
                .find(|f| f.field_id == role)
                .unwrap()
                .definition;
            dependencies.insert(getter.clone());
            let declaration = c
                .declarations
                .iter()
                .find(|row| c.name_table[row.name as usize] == core_name(&d.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = declaration.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                &c.name_table[c.declarations[global as usize].name as usize],
                getter
            );
            for seed in [0, 1, 17] {
                let input = relation_tests::sample(
                    instance,
                    seed,
                    &types,
                    &response["facts"],
                    emitted.closure().closed(),
                );
                let MonomorphicValue::Transition {
                    state,
                    events,
                    response,
                    ..
                } = &input
                else {
                    panic!()
                };
                let expected = match role {
                    "state" => state.as_ref().clone(),
                    "events" => MonomorphicValue::Sequence {
                        type_id: d.result_type.clone(),
                        elements: events.clone(),
                    },
                    _ => response.as_ref().clone(),
                };
                let raw = relation_tests::storage(&expected, &types);
                let out = run(
                    &c,
                    &core_name(&d.name),
                    vec![V::Cube(relation_tests::storage(&input, &types))],
                );
                let probes: BTreeSet<_> = if raw.len() <= 1024 {
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
                for index in probes {
                    assert_eq!(
                        observe(&c, out.clone(), types[&d.result_type].depth, index),
                        raw[index],
                        "{id} {role} seed{seed} bit{index}"
                    );
                    observations += 1;
                }
            }
            // Independent raw product addressing reaches every selector bit,
            // including the tail of the full event carrier, without traversing
            // an unrelated whole collection algorithm.
            let OrdinaryShape::Product { fields } = &types[instance].shape else {
                panic!()
            };
            for phase in 0..2 {
                let mut input = BTreeSet::new();
                let mut expected = BTreeSet::new();
                for (slot, field) in fields.iter().enumerate() {
                    let shape = match &field.shape {
                        OrdinaryShape::RoleBound { value, .. } => value.as_ref(),
                        shape => shape,
                    };
                    let OrdinaryShape::Reference { type_id } = shape else {
                        panic!()
                    };
                    let depth = types[type_id].depth;
                    let ones: BTreeSet<_> = (0..depth)
                        .map(|i| 1usize << i)
                        .chain([0, (1usize << depth) - 1])
                        .filter(|index| {
                            (index.count_ones() as usize + slot + phase).is_multiple_of(2)
                        })
                        .collect();
                    input.extend(
                        ones.iter()
                            .map(|index| slot | (index << (types[instance].depth - depth))),
                    );
                    if field.id == role {
                        expected = ones;
                    }
                }
                let out = run(
                    &c,
                    &core_name(&d.name),
                    vec![sparse_cube(types[instance].depth, input)],
                );
                let depth = types[&d.result_type].depth;
                let probes = (0..depth)
                    .flat_map(|i| [1usize << i, (1usize << i) - 1])
                    .chain([0, (1usize << depth) - 1])
                    .collect::<BTreeSet<_>>();
                for index in probes {
                    assert_eq!(
                        observe(&c, out.clone(), depth, index),
                        expected.contains(&index),
                        "raw{id} {role} phase{phase} bit{index}"
                    );
                    observations += 1;
                }
            }
        }
        structural_equivalence_tests::same_definition_closure(&sc, &c, &dependencies).unwrap();
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
    }
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    eprintln!("transition clause projection observations: {observations}");
}
