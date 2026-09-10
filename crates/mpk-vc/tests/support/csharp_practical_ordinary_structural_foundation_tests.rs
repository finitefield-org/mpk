use super::*;

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = BTreeMap::new();
    for (id, row, facts) in entry_tests::sources()
        .into_iter()
        .chain(collection_tests::sources())
        .chain(construction_tests::sources())
        .chain(outcome_tests::extra_sources())
        .chain(observation_tests::sources())
    {
        if let Some(previous) = sources.insert(id, (row.clone(), facts.clone())) {
            assert_eq!(previous, (row, facts));
        }
    }
    let rows = read("data-phase/data-stage-replay.json");
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "0fd1cf08493a395f3053454f01cf17fb0c4cc2687e7c290f1015215ca3e33b89")
        .unwrap();
    sources.insert(
        "unit-void-source".into(),
        (row.clone(), row["outcome"]["facts"].clone()),
    );
    let nullable_string = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "57de098014bbd8059f221b24525f74c6cb70320bfdcac33794de4ae0af3e143e")
        .unwrap();
    sources.insert(
        "nullable-string-default".into(),
        (
            nullable_string.clone(),
            nullable_string["outcome"]["facts"].clone(),
        ),
    );
    let requests = read("ordinary-foundation/finite-sources/requests.json");
    let responses = read("ordinary-foundation/finite-sources/responses.json");
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    sources.insert(
        "parse-error-contract".into(),
        (requests[0].clone(), responses[0]["facts"].clone()),
    );
    sources
        .into_iter()
        .map(|(id, (row, facts))| (id, row, facts))
        .collect()
}
fn referenced_definitions(value: &Value, names: &BTreeSet<&str>) {
    match value {
        Value::Object(fields) => {
            for (name, value) in fields {
                if name == "definition" || name.ends_with("_definition") {
                    if let Some(name) = value.as_str() {
                        assert!(names.contains(name), "missing concrete definition: {name}");
                    }
                } else {
                    referenced_definitions(value, names);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                referenced_definitions(value, names);
            }
        }
        _ => {}
    }
}
fn emitted_operation_ids(p: &OrdinaryStructuralFoundationProgram) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut add = |id: String| assert!(ids.insert(id));
    for d in p.sequences() {
        for suffix in ["length", "read", "equal"] {
            add(format!("{}.{suffix}", d.carrier.type_id));
        }
        if d.compare_definition.is_some() {
            add(format!("{}.compare", d.carrier.type_id));
        }
    }
    for d in p.entries() {
        for suffix in ["make", "key", "value", "equal"] {
            add(format!("{}.{suffix}", d.carrier.type_id));
        }
        if d.compare_definition.is_some() {
            add(format!("{}.compare", d.carrier.type_id));
        }
    }
    for d in p.constructions() {
        for op in &d.operations {
            add(op.operation_id.clone());
        }
    }
    for d in p.outcomes() {
        for op in &d.operations {
            add(op.operation_id.clone());
        }
    }
    for d in p.collections() {
        for op in &d.operations {
            add(op.operation_id.clone());
        }
    }
    for d in p.money() {
        for op in &d.operations {
            add(op.operation_id.clone());
        }
    }
    ids
}

#[test]
fn csharp_03_t06_w09_structural_foundation_original_sources() {
    let bundle = b();
    let mut metrics = vec![];
    let mut templates = BTreeSet::new();
    let output =
        std::env::var_os("MPK_W09_STRUCTURAL_FOUNDATIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-foundations");
    // Keep the shared unit-3 source inventory stable for other components,
    // but include every captured Money specialization in this integrated corpus.
    let mut integrated_sources = sources();
    let requests = read("ordinary-foundation/money-sources/requests.json");
    let responses = read("ordinary-foundation/money-sources/responses.json");
    assert_eq!(
        requests.as_array().unwrap().len(),
        responses.as_array().unwrap().len()
    );
    for (request, response) in requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
    {
        assert_eq!(request["id"], response["id"]);
        assert!(response.get("reject").is_none());
        let id = request["id"].as_str().unwrap();
        assert!(!integrated_sources
            .iter()
            .any(|(existing, _, _)| existing == id));
        integrated_sources.push((id.to_owned(), request.clone(), response["facts"].clone()));
    }
    for (id, row, facts) in integrated_sources {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_structural_foundations(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let names = c
            .declarations
            .iter()
            .map(|d| c.name_table[d.name as usize].as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(names.len(), c.declarations.len());
        referenced_definitions(&metadata, &names);
        assert_eq!(
            p.source_observations()
                .iter()
                .map(|d| &d.carrier)
                .collect::<Vec<_>>(),
            p.relations().iter().map(|d| &d.carrier).collect::<Vec<_>>()
        );
        assert_eq!(
            p.carriers(),
            generate_csharp_practical_ordinary_carriers(emitted.vir())
                .unwrap()
                .carriers()
        );
        let actual = emitted_operation_ids(&p);
        let mut expected = BTreeSet::new();
        let mut deferred = BTreeSet::new();
        for entry in emitted.closure().closed().entries() {
            let template = entry["template_id"].as_str().unwrap();
            templates.insert(template.to_owned());
            let set = if template == "mpk.csharp.semantic.transition.v1" {
                &mut deferred
            } else {
                &mut expected
            };
            for operation in entry["operation_definitions"].as_array().unwrap() {
                assert!(set.insert(operation["id"].as_str().unwrap().to_owned()));
            }
        }
        assert_eq!(
            actual, expected,
            "complete uninvoked-operation coverage: {id}"
        );
        assert_eq!(
            deferred,
            p.deferred_instances()
                .iter()
                .flat_map(|d| d.operation_ids.iter().cloned())
                .collect()
        );
        for d in p.deferred_instances() {
            assert_eq!(d.internal_unit, 6);
            assert_eq!(d.template_id, "mpk.csharp.semantic.transition.v1");
        }
        for d in p.constructions() {
            for op in &d.operations {
                for f in &op.failures {
                    assert_eq!(f.definition.is_none(), f.label == "ownership");
                }
            }
        }
        // For a real construction+collection closure the shared pipeline must
        // fit the frozen limit and cost less than its independent programs.
        // Aggregate regrouping also reduces those standalone programs, so
        // exceeding the limit independently is no longer a required premise.
        if id == "nested-box" {
            let standalone = [
                generate_csharp_practical_ordinary_domains(emitted.vir())
                    .unwrap()
                    .canonical_bytes(),
                generate_csharp_practical_ordinary_sequences(emitted.vir())
                    .unwrap()
                    .canonical_bytes(),
                generate_csharp_practical_ordinary_constructions(emitted.vir())
                    .unwrap()
                    .canonical_bytes(),
            ];
            let independent = standalone
                .iter()
                .map(|bytes| {
                    let value: Value = serde_json::from_slice(bytes).unwrap();
                    assert_eq!(value["source_ir_sha256"], metadata["source_ir_sha256"]);
                    value["static_transformers"].as_u64().unwrap()
                })
                .sum::<u64>();
            assert!(independent > p.static_transformers() as u64);
            assert!(p.static_transformers() <= 16384);
            assert!(!p.constructions().is_empty() && !p.sequences().is_empty());
        }
        assert_eq!(
            import_csharp_practical_ordinary_structural_foundations(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        if id == "binding-vc-validation" {
            for field in [
                "schema",
                "source_ir_sha256",
                "foundation_sha256",
                "carriers",
                "storage",
                "relations",
                "source_observations",
                "domains",
                "defaults",
                "finite_operations",
                "sequences",
                "constructions",
                "entries",
                "outcomes",
                "collections",
                "money",
                "deferred_instances",
                "static_transformers",
                "certificate_sha256",
            ] {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(import_csharp_practical_ordinary_structural_foundations(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
        let mut corrupted = p.certificate_bytes().to_vec();
        *corrupted.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_structural_foundations(
            &p.canonical_bytes(),
            &corrupted,
            emitted.vir()
        )
        .is_err());
        let file = format!("{id}.hex");
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(output) = &output {
            fs::create_dir_all(output).unwrap();
            fs::write(output.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        eprintln!("integrated structural foundation {id}: {} operations, {} terms, {} declarations, {} transformers",actual.len(),c.term_table.len(),c.declarations.len(),p.static_transformers());
        metrics.push(json!({"id":id,"file":file,"operation_count":actual.len(),"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":metadata}));
    }
    assert_eq!(metrics.len(), 44);
    assert_eq!(templates.len(), 12);
    if let Some(output) = output {
        fs::write(
            output.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join("certificates.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_structural_foundation_composed_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(composed_semantics)
        .unwrap()
        .join()
        .unwrap();
}

fn composed_semantics() {
    use core_eval::{bit as observed_bit, run, sparse_cube, V};
    use relation_tests::{sample, storage};
    use sequence_tests::{observe, word};

    let bundle = b();
    let mut contexts = 0;
    let mut observations = 0;
    let selected = [
        "extra-bool-nullable-map",
        "extra-validation-product",
        "bool-construction",
        "binding-vc-ordered_entry",
        "boundary-exception-payload",
    ];
    for (id, row, facts) in sources()
        .into_iter()
        .filter(|(id, _, _)| selected.contains(&id.as_str()))
    {
        contexts += 1;
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_structural_foundations(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = p
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let input = |value: &MonomorphicValue| {
            let bits = storage(value, &types);
            sparse_cube(
                bits.len().trailing_zeros(),
                bits.into_iter()
                    .enumerate()
                    .filter_map(|(i, b)| b.then_some(i))
                    .collect(),
            )
        };
        eprintln!("composed ordinary structural semantics {id}");
        match id.as_str() {
            "extra-bool-nullable-map" => {
                let map = &p.collections()[0];
                let op = |suffix: &str| {
                    map.operations
                        .iter()
                        .find(|op| op.operation_id.ends_with(suffix))
                        .unwrap()
                };
                let empty = MonomorphicValue::OrderedMap {
                    type_id: map.carrier.type_id.clone(),
                    entries: vec![],
                };
                let key = MonomorphicValue::Bool {
                    type_id: map.key_type_id.clone(),
                    value: true,
                };
                let none = MonomorphicValue::Option {
                    type_id: map.value_type_id.clone().unwrap(),
                    arm: OptionArm::None,
                    value: None,
                };
                let added = run(
                    &c,
                    &op(".add").normal_definition,
                    vec![input(&empty), input(&key), input(&none)],
                );
                let found = run(
                    &c,
                    &op(".lookup").normal_definition,
                    vec![added, input(&key)],
                );
                let expected = MonomorphicValue::TaggedSum {
                    type_id: op(".lookup").result_type_id.clone(),
                    arm: "found".into(),
                    payload: vec![none.clone()],
                };
                observe(&c, found.clone(), &storage(&expected, &types));
                observations += 1;
                let lookup = p
                    .outcomes()
                    .iter()
                    .find(|d| d.carrier.type_id == expected.type_id())
                    .unwrap();
                let value = lookup
                    .operations
                    .iter()
                    .find(|op| op.operation_id.ends_with(".value"))
                    .unwrap();
                observe(
                    &c,
                    run(&c, &value.normal_definition, vec![found]),
                    &storage(&none, &types),
                );
                observations += 1;
                let missing = MonomorphicValue::TaggedSum {
                    type_id: expected.type_id().into(),
                    arm: "missing_key".into(),
                    payload: vec![],
                };
                observe(
                    &c,
                    run(
                        &c,
                        &op(".lookup").normal_definition,
                        vec![input(&empty), input(&key)],
                    ),
                    &storage(&missing, &types),
                );
                observations += 1;
            }
            "extra-validation-product" => {
                let d = p
                    .outcomes()
                    .iter()
                    .find(|d| d.template_id == "mpk.csharp.semantic.validation.v1")
                    .unwrap();
                let op = |suffix: &str| {
                    d.operations
                        .iter()
                        .find(|op| op.operation_id.ends_with(suffix))
                        .unwrap()
                };
                let seq = p
                    .sequences()
                    .iter()
                    .find(|s| s.carrier.type_id == op(".append_errors").result_type_id)
                    .unwrap();
                let left = sample(
                    &seq.element_type_id,
                    1,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                let right = sample(
                    &seq.element_type_id,
                    2,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                assert_ne!(left, right);
                let sequence = |elements| MonomorphicValue::Sequence {
                    type_id: seq.carrier.type_id.clone(),
                    elements,
                };
                let appended = run(
                    &c,
                    &op(".append_errors").normal_definition,
                    vec![
                        input(&sequence(vec![left.clone()])),
                        input(&sequence(vec![right.clone()])),
                    ],
                );
                let invalid = run(&c, &op(".invalid").normal_definition, vec![appended]);
                assert!(!observed_bit(run(
                    &c,
                    &op(".is_valid").normal_definition,
                    vec![invalid.clone()]
                )));
                observations += 1;
                let errors = run(&c, &op(".errors").normal_definition, vec![invalid]);
                for (i, expected) in [left, right].iter().enumerate() {
                    observe(
                        &c,
                        run(
                            &c,
                            &seq.read_definition,
                            vec![errors.clone(), word(i as u32)],
                        ),
                        &storage(expected, &types),
                    );
                    observations += 1;
                }
            }
            "bool-construction" => {
                let d = &p.constructions()[0];
                let op = |suffix: &str| {
                    d.operations
                        .iter()
                        .find(|op| op.operation_id.ends_with(suffix))
                        .unwrap()
                };
                let allocated = run(
                    &c,
                    &op(".allocate").normal_definition,
                    vec![word(2), V::Bit(false)],
                );
                assert!(!observed_bit(run(
                    &c,
                    &d.complete_definition,
                    vec![allocated.clone()]
                )));
                observations += 1;
                let first = run(
                    &c,
                    &op(".fill").normal_definition,
                    vec![allocated, word(0), V::Bit(false)],
                );
                let second = run(
                    &c,
                    &op(".fill").normal_definition,
                    vec![first, word(1), V::Bit(true)],
                );
                assert!(observed_bit(run(
                    &c,
                    &d.complete_definition,
                    vec![second.clone()]
                )));
                observations += 1;
                let published = run(&c, &op(".freeze").normal_definition, vec![second]);
                let seq = p
                    .sequences()
                    .iter()
                    .find(|s| s.carrier.type_id == d.published_type_id)
                    .unwrap();
                for (i, expected) in [false, true].into_iter().enumerate() {
                    assert_eq!(
                        observed_bit(run(
                            &c,
                            &seq.read_definition,
                            vec![published.clone(), word(i as u32)]
                        )),
                        expected
                    );
                    observations += 1;
                }
                // This observation never discharges the source ownership gate.
                assert!(op(".freeze")
                    .failures
                    .iter()
                    .any(|f| f.label == "ownership" && f.definition.is_none()));
            }
            "binding-vc-ordered_entry" => {
                let d=p.storage().iter().find(|d|d.carrier.type_id.starts_with("mpk.csharp.source.")&&matches!(&d.operations,OrdinaryStructuralOperations::Product{operations} if operations.fields.len()==2)).unwrap();
                let value = sample(
                    &d.carrier.type_id,
                    1,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                let MonomorphicValue::Product { fields, .. } = &value else {
                    panic!()
                };
                let OrdinaryStructuralOperations::Product { operations } = &d.operations else {
                    panic!()
                };
                observe(
                    &c,
                    run(
                        &c,
                        &operations.make_definition,
                        fields.iter().map(|f| input(&f.value)).collect(),
                    ),
                    &storage(&value, &types),
                );
                observations += 1;
                for (field, projection) in fields.iter().zip(&operations.fields) {
                    observe(
                        &c,
                        run(&c, &projection.definition, vec![input(&value)]),
                        &storage(&field.value, &types),
                    );
                    observations += 1;
                }
                let mut other = value.clone();
                let MonomorphicValue::Product { fields, .. } = &mut other else {
                    panic!()
                };
                let last = fields.last_mut().unwrap();
                *last.value = sample(
                    last.value.type_id(),
                    3,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                let relation = p
                    .relations()
                    .iter()
                    .find(|r| r.carrier.type_id == d.carrier.type_id)
                    .unwrap();
                assert!(!observed_bit(run(
                    &c,
                    &relation.equality_definition,
                    vec![input(&value), input(&other)]
                )));
                observations += 1;
            }
            "boundary-exception-payload" => {
                let constructor = p
                    .finite_operations()
                    .iter()
                    .find(|op| {
                        op.operation_id.ends_with(".construct") && op.argument_type_ids.len() == 1
                    })
                    .unwrap();
                let source_id = &constructor.argument_type_ids[0];
                let payload = sample(source_id, 1, &types, &facts, emitted.closure().closed());
                let exception = run(&c, &constructor.definition, vec![input(&payload)]);
                let predicate = p
                    .finite_operations()
                    .iter()
                    .find(|op| {
                        op.operation_id.ends_with(".is_type")
                            && op.specialization == constructor.specialization
                    })
                    .unwrap();
                assert!(observed_bit(run(
                    &c,
                    &predicate.definition,
                    vec![exception.clone()]
                )));
                observations += 1;
                let OrdinaryShape::Sum { arms } = &types[&constructor.result_type_id].shape else {
                    panic!()
                };
                let arm = arms.iter().find(|a| &a.id == source_id).unwrap();
                let expected = MonomorphicValue::ClosedException {
                    type_id: constructor.result_type_id.clone(),
                    tag: arm.tag,
                    source_type_id: Some(source_id.clone()),
                    payload: Some(Box::new(payload.clone())),
                };
                observe(&c, exception.clone(), &storage(&expected, &types));
                observations += 1;
                let MonomorphicValue::Product { fields, .. } = payload else {
                    panic!()
                };
                for (member, expected) in arm.fields.iter().zip(fields) {
                    let get = p
                        .finite_operations()
                        .iter()
                        .find(|op| {
                            op.operation_id.ends_with(".payload")
                                && op.specialization.as_ref() == Some(&member.id)
                        })
                        .unwrap();
                    assert_eq!(get.active_tag_requirement, Some(arm.tag));
                    observe(
                        &c,
                        run(&c, &get.definition, vec![exception.clone()]),
                        &storage(&expected.value, &types),
                    );
                    observations += 1;
                }
            }
            _ => unreachable!(),
        }
    }
    assert_eq!(contexts, 5);
    eprintln!(
        "composed ordinary structural semantics: {contexts} sources, {observations} observations"
    );
}
