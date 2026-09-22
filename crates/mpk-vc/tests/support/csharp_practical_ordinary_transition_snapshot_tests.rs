//! W08 complete source snapshots and their paired canonical-field relation,
//! using independently encoded original values.
use super::*;
use core_eval::{bit, run, sparse_cube, V};

fn sources() -> Vec<(String, Value, Value)> {
    let mut result = vec![];
    for family in ["transition", "idempotency", "transition-vc"] {
        let requests = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        for request in requests
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["expected_attachment"] == true)
        {
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == request["id"])
                .unwrap();
            result.push((
                format!("{family}-{}", request["case"].as_str().unwrap()),
                request.clone(),
                response["facts"].clone(),
            ));
        }
    }
    assert_eq!(result.len(), 11);
    result
}
fn input(value: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    sparse_cube(
        types[value.type_id()].depth,
        relation_tests::storage(value, types)
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| b.then_some(i))
            .collect(),
    )
}
fn sequence_with_length(depth: u32, length: u32) -> V {
    sparse_cube(
        depth,
        (0..32)
            .filter(|bit| length & (1 << bit) != 0)
            .map(|bit| bit << (depth - 5))
            .collect(),
    )
}
fn product_field(value: &MonomorphicValue, ordinal: usize) -> MonomorphicValue {
    let MonomorphicValue::Product { fields, .. } = value else {
        panic!()
    };
    fields[ordinal].value.as_ref().clone()
}
fn replace_product_field(
    value: &MonomorphicValue,
    ordinal: usize,
    replacement: MonomorphicValue,
) -> MonomorphicValue {
    let mut value = value.clone();
    let MonomorphicValue::Product { fields, .. } = &mut value else {
        panic!()
    };
    *fields[ordinal].value = replacement;
    value
}

#[test]
fn csharp_03_t06_w09_transition_snapshots_original_sources() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_TRANSITION_SNAPSHOTS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/transition-snapshots");
    let mut metrics = vec![];
    let mut observations = 0;
    let mut nodes = 0;
    let mut member_changes = 0;
    for (id, request, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_transition_snapshots(emitted.vir()).unwrap();
        let vc = generate_csharp_practical_transition_vcs(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        })
        .unwrap();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            vc.snapshots().iter().collect::<Vec<_>>()
        );
        let resolved = p
            .definitions()
            .iter()
            .map(|d| d.symbol.as_str())
            .chain(p.encoding_equalities().iter().map(|d| d.symbol.as_str()))
            .chain(p.history_capacities().iter().map(|d| d.symbol.as_str()))
            .chain(p.retained_histories().iter().flat_map(|d| {
                [
                    d.retained_record_symbol.as_str(),
                    d.retained_key_present_symbol.as_str(),
                    d.retained_keys_unique_symbol.as_str(),
                    d.append_complete_snapshot_symbol.as_str(),
                    d.preserve_retained_history_order_symbol.as_str(),
                ]
            }))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_definition_names(),
            vc.definition_names()
                .iter()
                .filter(|n| !resolved.contains(n.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        );
        assert!(!p.pending_definition_names().is_empty());
        assert!(!p
            .pending_definition_names()
            .iter()
            .any(|n| n.starts_with("Mpk.CSharp.Transition.SourceEqual.")));
        if !vc.snapshots().is_empty() {
            assert!(!p
                .pending_definition_names()
                .iter()
                .any(|n| n.contains("CanonicalFieldEncodingsEqual")));
            assert_eq!(p.encoding_equalities().len(), 1);
        } else {
            assert!(p.encoding_equalities().is_empty());
        }
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let types: BTreeMap<_, _> = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect();
        for d in p.definitions() {
            nodes += 1;
            assert_eq!(
                d.symbol,
                format!("Mpk.CSharp.Transition.SourceEqual.{}", d.source.type_id)
            );
            assert_eq!(d.carrier, types[&d.source.type_id]);
            let oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.source.type_id,
            )
            .unwrap();
            assert!(oracle.is_total());
            let sample = |seed| {
                relation_tests::sample(
                    &d.source.type_id,
                    seed,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                )
            };
            let values = (0..3).map(sample).collect::<Vec<_>>();
            let check = |left: &MonomorphicValue, right: &MonomorphicValue| {
                let expected = oracle.structural_equal(left, right).unwrap();
                assert_eq!(
                    bit(run(
                        &c,
                        &d.equality_definition,
                        vec![input(left, &types), input(right, &types)]
                    )),
                    expected,
                    "{id}: {}",
                    d.source.type_id
                );
                expected
            };
            for left in &values {
                for right in &values {
                    check(left, right);
                    observations += 1;
                }
            }
            if let MonomorphicValue::Product { fields, .. } = &values[0] {
                let OrdinaryShape::Product { fields: layout } = &d.carrier.shape else {
                    panic!()
                };
                assert_eq!(fields.len(), layout.len());
                assert_eq!(
                    layout.iter().map(|f| &f.id).collect::<Vec<_>>(),
                    d.source.member_ids.iter().collect::<Vec<_>>()
                );
                let MonomorphicValue::Product { fields: other, .. } = &values[1] else {
                    panic!()
                };
                for index in 0..fields.len() {
                    let mut changed = values[0].clone();
                    let MonomorphicValue::Product { fields, .. } = &mut changed else {
                        panic!()
                    };
                    fields[index] = other[index].clone();
                    if !check(&values[0], &changed) {
                        member_changes += 1;
                    }
                    observations += 1;
                }
            }
        }
        for helper in p.encoding_equalities() {
            assert_eq!(helper.argument_type_ids.len(), 4);
            assert_eq!(helper.argument_type_ids[0], helper.argument_type_ids[2]);
            assert_eq!(helper.argument_type_ids[1], helper.argument_type_ids[3]);
            let command_oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &helper.argument_type_ids[0],
            )
            .unwrap();
            let context_oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &helper.argument_type_ids[1],
            )
            .unwrap();
            for command_seed in 0..3 {
                for context_seed in 0..3 {
                    let command = relation_tests::sample(
                        &helper.argument_type_ids[0],
                        0,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let context = relation_tests::sample(
                        &helper.argument_type_ids[1],
                        0,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let retained_command = relation_tests::sample(
                        &helper.argument_type_ids[2],
                        command_seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let retained_context = relation_tests::sample(
                        &helper.argument_type_ids[3],
                        context_seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let expected = command_oracle
                        .structural_equal(&command, &retained_command)
                        .unwrap()
                        && context_oracle
                            .structural_equal(&context, &retained_context)
                            .unwrap();
                    let args = [&command, &context, &retained_command, &retained_context]
                        .map(|value| input(value, &types))
                        .to_vec();
                    assert_eq!(bit(run(&c, &helper.definition, args)), expected);
                    observations += 1;
                }
            }
        }
        for capacity in p.history_capacities() {
            assert_eq!(capacity.history_carrier, types[&capacity.history_type_id]);
            for length in [0, 1, 4095, 4096] {
                assert_eq!(
                    bit(run(
                        &c,
                        &capacity.definition,
                        vec![sequence_with_length(capacity.history_carrier.depth, length)]
                    )),
                    length == 4096,
                    "{id}: history length {length}"
                );
                observations += 1;
            }
        }
        for retained in p.retained_histories() {
            let records = (0..3)
                .map(|seed| {
                    relation_tests::sample(
                        &retained.record_type_id,
                        seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    )
                })
                .collect::<Vec<_>>();
            let keys = records
                .iter()
                .map(|record| product_field(record, retained.record_key_member_ordinal))
                .collect::<Vec<_>>();
            let key_oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &retained.key_type_id,
            )
            .unwrap();
            assert!(!key_oracle.structural_equal(&keys[0], &keys[1]).unwrap());
            let history = |elements: Vec<MonomorphicValue>| MonomorphicValue::Sequence {
                type_id: retained.history_type_id.clone(),
                elements,
            };
            for (elements, expected) in [
                (vec![], true),
                (vec![records[0].clone()], true),
                (vec![records[0].clone(), records[1].clone()], true),
                (vec![records[0].clone(), records[0].clone()], false),
            ] {
                assert_eq!(
                    bit(run(
                        &c,
                        &retained.retained_keys_unique_definition,
                        vec![input(&history(elements), &types)]
                    )),
                    expected,
                    "{id}: retained key uniqueness"
                );
                observations += 1;
            }
            let two = history(vec![records[0].clone(), records[1].clone()]);
            for index in 0..2 {
                let actual = run(
                    &c,
                    &retained.retained_record_definition,
                    vec![input(&two, &types), input(&keys[index], &types)],
                );
                sequence_tests::observe(
                    &c,
                    actual,
                    &relation_tests::storage(&records[index], &types),
                );
                observations += 1;
            }
            let state = relation_tests::sample(
                &retained.state_type_id,
                0,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            let command = relation_tests::sample(
                &retained.command_type_id,
                0,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            for (elements, key, expected) in [
                (vec![], keys[0].clone(), false),
                (vec![records[0].clone()], keys[0].clone(), true),
                (vec![records[0].clone()], keys[1].clone(), false),
            ] {
                let state = replace_product_field(
                    &state,
                    retained.state_history_member_ordinal,
                    history(elements),
                );
                let command =
                    replace_product_field(&command, retained.command_key_member_ordinal, key);
                assert_eq!(
                    bit(run(
                        &c,
                        &retained.retained_key_present_definition,
                        vec![input(&state, &types), input(&command, &types)]
                    )),
                    expected,
                    "{id}: retained key presence"
                );
                observations += 1;
            }
            let context = relation_tests::sample(
                &retained.context_type_id,
                0,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            let response = relation_tests::sample(
                &retained.response_type_id,
                0,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            let append_command = replace_product_field(
                &command,
                retained.command_key_member_ordinal,
                keys[1].clone(),
            );
            let mut appended_record = records[1].clone();
            for (ordinal, value) in [
                (
                    retained.record_command_member_ordinal,
                    append_command.clone(),
                ),
                (retained.record_context_member_ordinal, context.clone()),
                (retained.record_response_member_ordinal, response.clone()),
            ] {
                appended_record = replace_product_field(&appended_record, ordinal, value);
            }
            let old_state = replace_product_field(
                &state,
                retained.state_history_member_ordinal,
                history(vec![records[0].clone()]),
            );
            let next_state = replace_product_field(
                &state,
                retained.state_history_member_ordinal,
                history(vec![records[0].clone(), appended_record.clone()]),
            );
            let append = |next: &MonomorphicValue,
                          command: &MonomorphicValue,
                          context: &MonomorphicValue,
                          response: &MonomorphicValue| {
                bit(run(
                    &c,
                    &retained.append_complete_snapshot_definition,
                    vec![
                        input(&old_state, &types),
                        input(next, &types),
                        input(command, &types),
                        input(context, &types),
                        input(response, &types),
                    ],
                ))
            };
            assert!(append(&next_state, &append_command, &context, &response));
            observations += 1;
            let different = |type_id: &str, value: &MonomorphicValue| {
                let oracle = generate_structural_program(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    type_id,
                )
                .unwrap();
                (0..8)
                    .map(|seed| {
                        relation_tests::sample(
                            type_id,
                            seed,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        )
                    })
                    .find(|candidate| !oracle.structural_equal(value, candidate).unwrap())
                    .unwrap()
            };
            let other_command = different(&retained.command_type_id, &append_command);
            let other_context = different(&retained.context_type_id, &context);
            let other_response = different(&retained.response_type_id, &response);
            for (ordinal, replacement) in [
                (retained.record_key_member_ordinal, keys[0].clone()),
                (retained.record_command_member_ordinal, other_command),
                (retained.record_context_member_ordinal, other_context),
                (retained.record_response_member_ordinal, other_response),
            ] {
                let changed_record = replace_product_field(&appended_record, ordinal, replacement);
                let changed_next = replace_product_field(
                    &state,
                    retained.state_history_member_ordinal,
                    history(vec![records[0].clone(), changed_record]),
                );
                assert!(!append(&changed_next, &append_command, &context, &response));
                observations += 1;
            }
            let missing_append = replace_product_field(
                &state,
                retained.state_history_member_ordinal,
                history(vec![records[0].clone()]),
            );
            assert!(!append(
                &missing_append,
                &append_command,
                &context,
                &response
            ));
            observations += 1;
            let preserve = |next: &MonomorphicValue| {
                bit(run(
                    &c,
                    &retained.preserve_retained_history_order_definition,
                    vec![input(&old_state, &types), input(next, &types)],
                ))
            };
            assert!(preserve(&next_state));
            observations += 1;
            let changed_prefix = replace_product_field(
                &state,
                retained.state_history_member_ordinal,
                history(vec![records[1].clone(), appended_record]),
            );
            assert!(!preserve(&changed_prefix));
            observations += 1;
        }
        assert_eq!(
            import_csharp_practical_ordinary_transition_snapshots(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in data.as_object().unwrap().keys() {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_transition_snapshots(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        if !p.definitions().is_empty() {
            for field in ["source", "carrier", "symbol", "equality_definition"] {
                let mut changed = data.clone();
                changed["definitions"][0][field] = json!("forged");
                assert!(import_csharp_practical_ordinary_transition_snapshots(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
        if !p.encoding_equalities().is_empty() {
            for field in [
                "symbol",
                "argument_type_ids",
                "command_equality_definition",
                "context_equality_definition",
                "definition",
            ] {
                let mut changed = data.clone();
                changed["encoding_equalities"][0][field] = json!("forged");
                assert!(import_csharp_practical_ordinary_transition_snapshots(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
        if !p.history_capacities().is_empty() {
            for field in [
                "symbol",
                "history_type_id",
                "history_carrier",
                "length_definition",
                "definition",
            ] {
                let mut changed = data.clone();
                changed["history_capacities"][0][field] = json!("forged");
                assert!(import_csharp_practical_ordinary_transition_snapshots(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
        if !p.retained_histories().is_empty() {
            for field in [
                "contract_sha256",
                "state_type_id",
                "command_type_id",
                "context_type_id",
                "history_type_id",
                "record_type_id",
                "key_type_id",
                "state_history_member_id",
                "state_history_member_ordinal",
                "command_key_member_id",
                "command_key_member_ordinal",
                "record_key_member_id",
                "record_key_member_ordinal",
                "record_command_member_id",
                "record_command_member_ordinal",
                "record_context_member_id",
                "record_context_member_ordinal",
                "record_response_member_id",
                "record_response_member_ordinal",
                "response_type_id",
                "retained_record_symbol",
                "retained_record_definition",
                "retained_key_present_symbol",
                "retained_key_present_definition",
                "retained_keys_unique_symbol",
                "retained_keys_unique_definition",
                "append_complete_snapshot_symbol",
                "append_complete_snapshot_definition",
                "preserve_retained_history_order_symbol",
                "preserve_retained_history_order_definition",
            ] {
                let mut changed = data.clone();
                changed["retained_histories"][0][field] = json!("forged");
                assert!(import_csharp_practical_ordinary_transition_snapshots(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_transition_snapshots(
            &p.canonical_bytes(),
            &changed,
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
        metrics.push(json!({"id":id,"file":file,"metadata":data,"terms":c.term_table.len(),"declarations":c.declarations.len()}));
        eprintln!(
            "transition snapshots {id}: {} exact snapshot nodes; remaining W08 constants explicit",
            p.definitions().len()
        );
    }
    assert_eq!(nodes, 14);
    assert!(member_changes > 0 && observations > 0);
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
    eprintln!("transition snapshots: 11 original contexts, {nodes} nodes, {observations} observations, {member_changes} isolated stored-member changes detected");
}
