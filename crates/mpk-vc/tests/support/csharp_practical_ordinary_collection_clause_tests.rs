//! Contract collection reads reuse complete ordinary collection definitions.
use super::*;
fn raw(v: &MonomorphicValue) -> J {
    match v {
        MonomorphicValue::Bool { value, .. } => J::Bool(*value),
        MonomorphicValue::Signed { value, .. } | MonomorphicValue::Unsigned { value, .. } => {
            J::string(value)
        }
        MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
            J::string(bits)
        }
        MonomorphicValue::String { utf16, .. } => J::Utf16String(utf16.clone()),
        MonomorphicValue::DecimalBits {
            negative,
            scale,
            coefficient,
            ..
        } => {
            if coefficient == "0" {
                return J::string("0");
            }
            let mut digits = coefficient.clone();
            // Contract literals use decimal.normalized. Raw ordinary-call
            // observations below still retain the original scale and sign.
            let mut scale = *scale as usize;
            while scale > 0 && digits.ends_with('0') {
                digits.pop();
                scale -= 1;
            }
            while digits.len() <= scale {
                digits.insert(0, '0');
            }
            if scale > 0 {
                digits.insert(digits.len() - scale, '.');
            }
            if *negative {
                digits.insert(0, '-');
            }
            J::string(digits)
        }
        MonomorphicValue::Product { fields, .. } => J::Object(
            fields
                .iter()
                .map(|f| (f.name.clone(), raw(&f.value)))
                .collect(),
        ),
        MonomorphicValue::OrderedMap { entries, .. } => J::Array(
            entries
                .iter()
                .map(|e| J::object(vec![("key", raw(&e.key)), ("value", raw(&e.value))]))
                .collect(),
        ),
        MonomorphicValue::OrderedSet { elements, .. } => {
            J::Array(elements.iter().map(raw).collect())
        }
        MonomorphicValue::Option { arm, value, .. } => {
            let mut fields = vec![(
                "tag",
                J::string(if *arm == OptionArm::None {
                    "none"
                } else {
                    "some"
                }),
            )];
            if let Some(value) = value {
                fields.push(("payload", raw(value)));
            }
            J::object(fields)
        }
        _ => panic!("unexpected collection fixture value: {v:?}"),
    }
}
fn literal(v: &MonomorphicValue) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(v.type_id())),
        ("value", raw(v)),
    ])
}
fn cases(
    d: &OrdinaryCollectionDefinition,
    types: &BTreeMap<String, OrdinaryCarrier>,
    facts: &Value,
    closed: &ClosedInstanceSet,
) -> Vec<(MonomorphicValue, MonomorphicValue)> {
    let sample = |ty: &str, seed| relation_tests::sample(ty, seed, types, facts, closed);
    let mut cases = vec![];
    for size in 0..=2 {
        let mut value = sample(&d.carrier.type_id, size);
        // The shared sample's decimal keys 1.00/1 are numeric duplicates.
        if d.key_type_id == ty("decimal") {
            match &mut value {
                MonomorphicValue::OrderedMap { entries, .. } => entries.truncate(1),
                MonomorphicValue::OrderedSet { elements, .. } => elements.truncate(1),
                _ => unreachable!(),
            }
        }
        for query in 0..=2 {
            cases.push((value.clone(), sample(&d.key_type_id, query)));
        }
    }
    cases
}
fn clauses(
    p: &OrdinaryCollectionProgram,
    facts: &Value,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) -> Vec<J> {
    let types = generate_csharp_practical_ordinary_carriers(vir)
        .unwrap()
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect();
    let mut clauses = vec![];
    let bundle = b();
    for d in p.definitions() {
        let model =
            OrderedCollectionModel::new(&bundle, roots, closed, &d.carrier.type_id).unwrap();
        let map = d.value_type_id.is_some();
        for (value, query) in cases(d, &types, facts, closed) {
            let present = model.contains(&value, &query).unwrap();
            let contains = J::object(vec![
                (
                    "tag",
                    J::string(if map { "map_contains" } else { "set_contains" }),
                ),
                ("type_id", J::string(ty("bool"))),
                (if map { "map" } else { "set" }, literal(&value)),
                (if map { "key" } else { "element" }, literal(&query)),
            ]);
            clauses.push(J::object(vec![
                ("tag", J::string("structural_equal")),
                ("type_id", J::string(ty("bool"))),
                ("left", contains),
                (
                    "right",
                    J::object(vec![
                        ("tag", J::string("literal")),
                        ("type_id", J::string(ty("bool"))),
                        ("value", J::Bool(present)),
                    ]),
                ),
            ]));
            if map {
                let lookup = d
                    .operations
                    .iter()
                    .find(|o| o.operation_id.ends_with(".lookup"))
                    .unwrap();
                let value = J::object(vec![
                    ("tag", J::string("map_lookup")),
                    ("type_id", J::string(&lookup.result_type_id)),
                    ("map", literal(&value)),
                    ("key", literal(&query)),
                ]);
                clauses.push(J::object(vec![
                    ("tag", J::string("tagged_is")),
                    ("type_id", J::string(ty("bool"))),
                    ("value", value),
                    (
                        "arm",
                        J::string(if present { "found" } else { "missing_key" }),
                    ),
                ]));
            }
        }
    }
    clauses
}
fn requests() -> Value {
    let template_row = read("ordinary-foundation/transition-clauses/requests.json")[0].clone();
    let text = template_row["inputs"]
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
    let selected = [
        "binding-vc-ordered_map",
        "binding-vc-ordered_set",
        "boundary-map-string",
        "boundary-map-decimal",
        "boundary-set-decimal",
        "boundary-map-compound",
        "extra-bool-float-map",
        "extra-bool-nullable-map",
        "extra-shared-lookup-maps",
    ];
    for (id, row, facts) in collection_tests::sources() {
        if !selected.contains(&id.as_str()) {
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
        let p = generate_csharp_practical_ordinary_collections(emitted.vir()).unwrap();
        let clauses = clauses(
            &p,
            &facts,
            emitted.vir(),
            emitted.closure().roots(),
            emitted.closure().closed(),
        );
        let source = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
            .unwrap()["utf8"]
            .as_str()
            .unwrap();
        let binding_entry = row["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| {
                serde_json::from_str::<Value>(i["utf8"].as_str().unwrap())
                    .is_ok_and(|v| v["schema"] == "mpk.csharp.semantic_bindings.v1")
            })
            .unwrap();
        let binding = binding_entry["utf8"].as_str().unwrap();
        let root = row["roots"][0].as_str().unwrap();
        let (context, captures) = support::context_with_sidecars(
            &bundle,
            root,
            source.as_bytes(),
            vec![
                binding_entry["path"].as_str().unwrap().into(),
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
                        "ensures" => *v = J::Array(clauses.clone()),
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
    assert_eq!(requests.len(), 9);
    json!(requests)
}
#[test]
fn csharp_03_t06_w09_collection_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_COLLECTION_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/collection-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/collection-clauses");
    if let Some(path) = std::env::var_os("MPK_W09_COLLECTION_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(path);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
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
fn input(v: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    if let MonomorphicValue::OrderedMap { .. } = v {
        let (depth, ones) = domain_tests::sparse_map_storage(v, types);
        sparse_cube(depth, ones)
    } else {
        let bits = relation_tests::storage(v, types);
        if bits.len() == 1 {
            V::Bit(bits[0])
        } else {
            V::Cube(bits)
        }
    }
}
#[test]
fn csharp_03_t06_w09_collection_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value =
        if let Some(path) = std::env::var_os("MPK_W09_COLLECTION_CLAUSE_RESPONSES") {
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
        } else {
            read("ordinary-foundation/collection-clauses/responses.json")
        };
    let mut observations = 0;
    let mut recipe_count = 0;
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
        for d in p.definitions() {
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| sparse_cube(types[ty].depth, BTreeSet::new()))
                .collect::<Vec<_>>();
            assert!(
                bit(run(&c, &d.definedness_definition, args.clone())),
                "{id}"
            );
            assert!(bit(run(&c, &d.value_definition, args)), "{id}");
            observations += 2;
        }
        let full = generate_csharp_practical_ordinary_collections(vir).unwrap();
        let sc = mpk_cert::decode_canonical_certificate(full.certificate_bytes()).unwrap();
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| {
                matches!(
                    d.tag.as_str(),
                    "map_contains" | "map_lookup" | "set_contains"
                )
            })
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut dependencies = BTreeSet::new();
        for recipe in recipes.values() {
            let d = full
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == recipe.argument_types[0])
                .unwrap();
            let lookup = recipe.tag == "map_lookup";
            let op = d
                .operations
                .iter()
                .find(|o| {
                    o.operation_id
                        .ends_with(if lookup { ".lookup" } else { ".contains" })
                })
                .unwrap();
            let declaration = c
                .declarations
                .iter()
                .find(|row| c.name_table[row.name as usize] == core_name(&recipe.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = declaration.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                op.normal_definition
            );
            dependencies.insert(op.normal_definition.clone());
            let model = OrderedCollectionModel::new(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.carrier.type_id,
            )
            .unwrap();
            for (value, query) in cases(d, &types, &response["facts"], emitted.closure().closed()) {
                let result = run(
                    &c,
                    &core_name(&recipe.name),
                    vec![input(&value, &types), input(&query, &types)],
                );
                if lookup {
                    let expected = model.lookup(&value, &query).unwrap();
                    let raw = relation_tests::storage(&expected, &types);
                    for (index, expected) in raw.iter().enumerate() {
                        let mut out = result.clone();
                        for i in 0..types[&op.result_type_id].depth {
                            out = apply(&c, out, V::Bit(index & (1 << i) != 0));
                        }
                        assert_eq!(bit(out), *expected, "{id} lookup bit{index}");
                        observations += 1;
                    }
                } else {
                    assert_eq!(bit(result), model.contains(&value, &query).unwrap(), "{id}");
                    observations += 1;
                }
            }
        }
        recipe_count += recipes.len();
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
        eprintln!("collection clauses {id}: {} read recipes", recipes.len());
    }
    assert_eq!(recipe_count, 18);
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    eprintln!("collection contract observations: {observations}; {recipe_count} recipes");
}

#[test]
fn csharp_03_t06_w09_collection_clauses_pinned_bytes() {
    let bundle = b();
    let requests = requests();
    assert_eq!(
        read("ordinary-foundation/collection-clauses/requests.json"),
        requests
    );
    let responses = read("ordinary-foundation/collection-clauses/responses.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/collection-clauses");
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
        let p = generate_csharp_practical_ordinary_contract_expressions(emitted.vir()).unwrap();
        assert_eq!(
            fs::read(fixture.join(format!("{id}.json"))).unwrap(),
            p.canonical_bytes()
        );
        assert_eq!(
            fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(requests.as_array().unwrap().len(), 9);
}
