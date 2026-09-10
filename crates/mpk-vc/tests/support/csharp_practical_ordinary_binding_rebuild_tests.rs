use super::*;
use core_eval::{apply, V};

#[test]
fn csharp_03_t06_w09_binding_rebuilds_original_source_certificates() {
    let bundle = b();
    let mut rows = vec![];
    let mut total = 0;
    let mut pending = 0;
    let out = std::env::var_os("MPK_W09_BINDING_REBUILDS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-rebuilds");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_binding_rebuilds(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let base = generate_csharp_practical_ordinary_binding_projections(vir).unwrap();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.projection.clone())
                .collect::<Vec<_>>(),
            base.definitions()
        );
        let wanted = p
            .definitions()
            .iter()
            .filter(|d| d.completion_witness_required)
            .map(|d| d.projection.projection.reconstruct.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_reconstruct_symbols(),
            wanted.into_iter().collect::<Vec<_>>()
        );
        for d in p.definitions() {
            assert_eq!(
                d.completion_witness_required,
                d.projection.reconstruct_definition.is_none()
            );
            assert_ne!(
                Some(&d.rebuild_definition),
                d.projection.reconstruct_definition.as_ref()
            );
        }
        total += p.definitions().len();
        pending += p.pending_reconstruct_symbols().len();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let before = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
        let roots = base
            .definitions()
            .iter()
            .map(|d| d.project_definition.clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_binding_rebuilds(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "binding_vc_sha256",
            "definitions",
            "pending_reconstruct_symbols",
            "certificate_sha256",
        ] {
            let mut changed = meta.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_binding_rebuilds(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_binding_rebuilds(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_rebuilds(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "binding rebuild {id}: {} definitions, {} pending unary witnesses",
            p.definitions().len(),
            p.pending_reconstruct_symbols().len()
        );
    }
    assert_eq!((rows.len(), total, pending), (45, 37, 35));
    let data =
        json!({"sources":rows,"rebuild_definitions":total,"pending_unary_witnesses":pending});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-rebuilds/certificates.json"),
            data
        );
    }
}

fn arm(v: &MonomorphicValue) -> (&str, Vec<&MonomorphicValue>) {
    match v {
        MonomorphicValue::Option { arm, value, .. } => (
            if *arm == OptionArm::None {
                "none"
            } else {
                "some"
            },
            value.iter().map(|v| v.as_ref()).collect(),
        ),
        MonomorphicValue::BoundaryPresence { arm, value, .. } => (
            match arm {
                BoundaryArm::Missing => "missing",
                BoundaryArm::Null => "null",
                BoundaryArm::Value => "value",
            },
            value.iter().map(|v| v.as_ref()).collect(),
        ),
        MonomorphicValue::TaggedSum { arm, payload, .. } => (arm, payload.iter().collect()),
        _ => panic!(),
    }
}
fn arguments(oracle: &ProjectionOracle<'_>, id: &str) -> Vec<String> {
    oracle
        .closed
        .entries()
        .iter()
        .find(|e| e["instance_id"] == id)
        .unwrap()["argument_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_owned())
        .collect()
}
fn rebuild(
    oracle: &ProjectionOracle<'_>,
    semantic: &MonomorphicValue,
    completion: &MonomorphicValue,
) -> MonomorphicValue {
    if semantic.type_id() == completion.type_id() {
        return semantic.clone();
    }
    if let Some(binding) = oracle
        .bindings
        .iter()
        .find(|b| b["source_type_id"] == completion.type_id())
    {
        let source = oracle.facts["types"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == completion.type_id())
            .unwrap();
        let mut result = completion.clone();
        let MonomorphicValue::Product { fields, .. } = &mut result else {
            panic!()
        };
        for field in fields {
            let declaration = source["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["name"] == field.name)
                .unwrap();
            let member = csharp_practical_stored_member_id(
                completion.type_id(),
                &field.name,
                &declaration["type"],
                declaration["storage"].as_str().unwrap(),
            )
            .unwrap();
            let Some((role, _)) = binding["member_map"]
                .as_object()
                .unwrap()
                .iter()
                .find(|(_, m)| m.as_str() == Some(member.as_str()))
            else {
                continue;
            };
            let selected = match semantic {
                MonomorphicValue::Instant { milliseconds, .. } => {
                    assert_eq!(role, "milliseconds");
                    Some(MonomorphicValue::Signed {
                        type_id: ty("i64"),
                        value: milliseconds.clone(),
                    })
                }
                MonomorphicValue::Money {
                    amount, currency, ..
                } => Some(
                    if role == "amount" {
                        amount.as_ref()
                    } else {
                        assert_eq!(role, "currency");
                        currency.as_ref()
                    }
                    .clone(),
                ),
                MonomorphicValue::OrderedEntry { key, value, .. } => Some(
                    if role == "key" {
                        key.as_ref()
                    } else {
                        assert_eq!(role, "value");
                        value.as_ref()
                    }
                    .clone(),
                ),
                MonomorphicValue::Transition {
                    state,
                    events,
                    response,
                    ..
                } => Some(match role.as_str() {
                    "state" => state.as_ref().clone(),
                    "response" => response.as_ref().clone(),
                    "events" => MonomorphicValue::Sequence {
                        type_id: oracle.instance(
                            "bounded_sequence",
                            &arguments(oracle, semantic.type_id())[1..2],
                        ),
                        elements: events.clone(),
                    },
                    _ => panic!(),
                }),
                MonomorphicValue::Sequence { .. }
                | MonomorphicValue::OrderedSet { .. }
                | MonomorphicValue::OrderedMap { .. } => Some(semantic.clone()),
                _ => {
                    let (arm, payload) = arm(semantic);
                    if role == "tag" {
                        let MonomorphicValue::Enum { carrier, .. } = field.value.as_mut() else {
                            panic!()
                        };
                        *carrier = binding["tag_arms"][arm].as_str().unwrap().into();
                        continue;
                    }
                    let active = match arm {
                        "invalid" => "errors",
                        "error" => "error",
                        _ => "value",
                    };
                    if role == active {
                        payload.first().map(|p| (*p).clone())
                    } else {
                        None
                    }
                }
            };
            if let Some(selected) = selected {
                *field.value = rebuild(oracle, &selected, &field.value);
            }
        }
        return result;
    }
    match completion {
        MonomorphicValue::Array {
            type_id,
            elements: seed,
        }
        | MonomorphicValue::Sequence {
            type_id,
            elements: seed,
        } => {
            let elements = match semantic {
                MonomorphicValue::Sequence { elements, .. }
                | MonomorphicValue::OrderedSet { elements, .. } => elements.clone(),
                MonomorphicValue::OrderedMap { entries, .. } => {
                    let entry =
                        oracle.instance("ordered_entry", &arguments(oracle, semantic.type_id()));
                    entries
                        .iter()
                        .map(|e| MonomorphicValue::OrderedEntry {
                            type_id: entry.clone(),
                            key: e.key.clone(),
                            value: e.value.clone(),
                        })
                        .collect()
                }
                _ => panic!(),
            };
            assert!(
                seed.len() >= elements.len(),
                "explicit completions must cover this oracle case's active elements"
            );
            let elements = elements
                .iter()
                .zip(seed)
                .map(|(value, seed)| rebuild(oracle, value, seed))
                .collect();
            if matches!(completion, MonomorphicValue::Array { .. }) {
                MonomorphicValue::Array {
                    type_id: type_id.clone(),
                    elements,
                }
            } else {
                MonomorphicValue::Sequence {
                    type_id: type_id.clone(),
                    elements,
                }
            }
        }
        MonomorphicValue::Option {
            type_id,
            value: seed,
            ..
        } => {
            let (active, payload) = arm(semantic);
            MonomorphicValue::Option {
                type_id: type_id.clone(),
                arm: if active == "none" {
                    OptionArm::None
                } else {
                    OptionArm::Some
                },
                value: payload.first().map(|p| {
                    Box::new(rebuild(
                        oracle,
                        p,
                        seed.as_ref().expect("completion supplies active payload"),
                    ))
                }),
            }
        }
        _ => panic!("missing reconstruction oracle: {semantic:?} into {completion:?}"),
    }
}
fn check_bits(
    cert: &mpk_cert::encode::Certificate,
    definition: &str,
    semantic: &MonomorphicValue,
    completion: &MonomorphicValue,
    expected: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> usize {
    let (td, tbits) = sparse_storage(semantic, types);
    let (sd, sbits) = sparse_storage(completion, types);
    let (depth, expected) = sparse_storage(expected, types);
    check_sparse_bits(
        cert,
        definition,
        (td, tbits),
        (sd, sbits),
        (depth, expected),
    )
}
fn check_sparse_bits(
    cert: &mpk_cert::encode::Certificate,
    definition: &str,
    (td, tbits): (u32, BTreeSet<usize>),
    (sd, sbits): (u32, BTreeSet<usize>),
    (depth, expected): (u32, BTreeSet<usize>),
) -> usize {
    assert_eq!(sd, depth);
    let prior = sbits.clone();
    let result = run(
        cert,
        definition,
        vec![sparse_cube(td, tbits), sparse_cube(sd, sbits)],
    );
    let mut addresses = if depth <= 10 {
        (0..1usize << depth).collect::<BTreeSet<_>>()
    } else {
        BTreeSet::from([0, (1usize << depth) - 1])
    };
    addresses.extend((0..depth).map(|i| 1usize << i));
    addresses.extend(prior);
    for &at in &expected {
        addresses.extend([
            at,
            at.saturating_sub(1),
            (at + 1).min((1usize << depth) - 1),
        ]);
    }
    for at in &addresses {
        let mut v = result.clone();
        for i in 0..depth {
            v = apply(cert, v, V::Bit(at & (1usize << i) != 0));
        }
        assert_eq!(
            bit(v),
            expected.contains(at),
            "{definition}: source storage bit {at}"
        );
    }
    addresses.len()
}

// Supply explicit completion cells for newly active semantic payloads. These
// are test inputs, not defaults or a generated unary reconstruction witness.
fn cover_source_payloads(completion: &mut MonomorphicValue, source: &MonomorphicValue) {
    match (completion, source) {
        (
            MonomorphicValue::Product { fields: target, .. },
            MonomorphicValue::Product { fields: source, .. },
        ) => {
            for (target, source) in target.iter_mut().zip(source) {
                assert_eq!(target.name, source.name);
                cover_source_payloads(&mut target.value, &source.value);
            }
        }
        (
            MonomorphicValue::Array {
                elements: target, ..
            }
            | MonomorphicValue::Sequence {
                elements: target, ..
            },
            MonomorphicValue::Array {
                elements: source, ..
            }
            | MonomorphicValue::Sequence {
                elements: source, ..
            },
        ) => {
            for (i, source) in source.iter().enumerate() {
                if i == target.len() {
                    target.push(source.clone());
                } else {
                    cover_source_payloads(&mut target[i], source);
                }
            }
        }
        (
            MonomorphicValue::Option {
                arm, value: target, ..
            },
            MonomorphicValue::Option {
                value: Some(source),
                ..
            },
        ) => {
            if let Some(target) = target {
                cover_source_payloads(target, source);
            } else {
                *arm = OptionArm::Some;
                *target = Some(source.clone());
            }
        }
        _ => {}
    }
}
#[test]
fn csharp_03_t06_w09_binding_rebuilds_source_observations() {
    let bundle = b();
    let mut count = 0;
    let mut bits = 0;
    let mut roles = BTreeSet::new();
    let mut saw_extra = false;
    let mut changed_inputs = 0;
    let mut raw_completion_lengths = 0;
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_binding_rebuilds(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let binding: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = ProjectionOracle {
            facts: &facts,
            bindings: binding["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        roles.extend(
            oracle
                .bindings
                .iter()
                .map(|b| b["role"].as_str().unwrap().to_owned()),
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for d in p.definitions() {
            for seed in 0..4 {
                let source = relation_tests::sample(
                    &d.projection.source_carrier.type_id,
                    seed,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let semantic = oracle.project(&source, &d.projection.semantic_carrier.type_id);
                let expected = rebuild(&oracle, &semantic, &source);
                assert_eq!(expected, source);
                bits += check_bits(
                    &cert,
                    &d.rebuild_definition,
                    &semantic,
                    &source,
                    &expected,
                    &types,
                );
                count += 1;
                if id == "remapped-boundary-sequence" {
                    let mut completion = source.clone();
                    let MonomorphicValue::Product { fields, .. } = &mut completion else {
                        panic!()
                    };
                    *fields.iter_mut().find(|f| f.name == "Extra").unwrap().value =
                        MonomorphicValue::Signed {
                            type_id: ty("i32"),
                            value: "73".into(),
                        };
                    let expected = rebuild(&oracle, &semantic, &completion);
                    assert_eq!(
                        oracle.project(&expected, &d.projection.semantic_carrier.type_id),
                        semantic
                    );
                    assert_ne!(expected, source);
                    bits += check_bits(
                        &cert,
                        &d.rebuild_definition,
                        &semantic,
                        &completion,
                        &expected,
                        &types,
                    );
                    count += 1;
                    saw_extra = true;
                    if seed == 2 {
                        let MonomorphicValue::Product { fields, .. } = &completion else {
                            panic!()
                        };
                        if let Some((index, items)) =
                            fields.iter().enumerate().find(|(_, f)| f.name == "Items")
                        {
                            let (depth, original) = sparse_storage(&completion, &types);
                            let item_depth = types[items.value.type_id()].depth;
                            let role_bits = usize::BITS - (fields.len() - 1).leading_zeros();
                            let max_depth = fields
                                .iter()
                                .map(|f| types[f.value.type_id()].depth)
                                .max()
                                .unwrap();
                            let shift = role_bits + max_depth - item_depth;
                            for length in [0u32, u32::MAX] {
                                let mut cells = original.clone();
                                for bit in 0..32 {
                                    let at = index | (bit << (item_depth - 5 + shift));
                                    if length & (1 << bit) == 0 {
                                        cells.remove(&at);
                                    } else {
                                        cells.insert(at);
                                    }
                                }
                                // Completion length never selects semantic output length;
                                // stored completion cells remain explicit inputs even when
                                // its header marks them inactive or is outside the domain.
                                bits += check_sparse_bits(
                                    &cert,
                                    &d.rebuild_definition,
                                    sparse_storage(&semantic, &types),
                                    (depth, cells),
                                    sparse_storage(&expected, &types),
                                );
                                raw_completion_lengths += 1;
                            }
                        }
                    }
                }
            }
            let completion = relation_tests::sample(
                &d.projection.source_carrier.type_id,
                3,
                &types,
                &facts,
                oracle.closed,
            );
            for seed in 0..3 {
                let source = relation_tests::sample(
                    &d.projection.source_carrier.type_id,
                    seed,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let semantic = oracle.project(&source, &d.projection.semantic_carrier.type_id);
                let mut completion = completion.clone();
                cover_source_payloads(&mut completion, &source);
                let expected = rebuild(&oracle, &semantic, &completion);
                assert_eq!(
                    oracle.project(&expected, &d.projection.semantic_carrier.type_id),
                    semantic
                );
                changed_inputs += usize::from(expected != completion);
                bits += check_bits(
                    &cert,
                    &d.rebuild_definition,
                    &semantic,
                    &completion,
                    &expected,
                    &types,
                );
                count += 1;
            }
            eprintln!(
                "binding rebuild source {id}: {}",
                d.projection.source_carrier.type_id
            );
        }
    }
    assert!(saw_extra);
    assert_eq!(roles.len(), 12);
    assert_eq!(count, 267);
    assert!(changed_inputs > 50);
    assert_eq!(raw_completion_lengths, 2);
    eprintln!("binding rebuild source observations: {count}; changed inputs: {changed_inputs}; storage bits: {bits}");
    eprintln!("binding rebuild raw completion length observations: {raw_completion_lengths}");
}
