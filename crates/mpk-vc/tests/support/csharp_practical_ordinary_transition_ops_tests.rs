//! Frozen transition product operations at original captured concrete instances.
use super::*;
use core_eval::{apply, bit, run, sparse_cube, V};

fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = structural_foundation_tests::sources();
    let requests = read("ordinary-foundation/transition-clauses/requests.json");
    let responses = read("ordinary-foundation/transition-clauses/responses.json");
    for request in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == request["id"])
            .unwrap();
        assert!(response.get("reject").is_none());
        sources.push((
            format!("clauses-{}", request["id"].as_str().unwrap()),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    sources
}
#[test]
fn csharp_03_t06_w09_transition_operations_original_source_certificates() {
    let bundle = b();
    let mut metrics = vec![];

    let output =
        std::env::var_os("MPK_W09_TRANSITION_OPERATIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/transition-operations");
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
        let p = generate_csharp_practical_ordinary_transition_operations(emitted.vir()).unwrap();
        let expected = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.transition.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected,
            p.definitions()
                .iter()
                .map(|d| d.carrier.type_id.as_str())
                .collect()
        );
        if p.definitions().is_empty() {
            continue;
        }

        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_transition_operations(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut m = data.clone();
            m[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_transition_operations(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        for field in [
            "state_type_id",
            "event_type_id",
            "events_type_id",
            "response_type_id",
            "make_definition",
            "state_definition",
            "events_definition",
            "response_definition",
            "equality_definition",
            "compare_definition",
            "event_bound_definition",
            "event_bound_argument_index",
        ] {
            let mut m = data.clone();
            m["definitions"][0][field] = json!("forged");
            assert!(import_csharp_practical_ordinary_transition_operations(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_transition_operations(
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
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert!(metrics.len() >= 3);
    eprintln!(
        "transition operation certificates: {} actual-source contexts",
        metrics.len()
    );
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

fn input(value: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    let raw = relation_tests::storage(value, types);
    sparse_cube(
        types[value.type_id()].depth,
        raw.iter()
            .enumerate()
            .filter_map(|(i, &v)| v.then_some(i))
            .collect(),
    )
}
fn observe(
    c: &mpk_cert::Certificate,
    out: V,
    expected: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> usize {
    let raw = relation_tests::storage(expected, types);
    let depth = types[expected.type_id()].depth;
    let probes: BTreeSet<_> = if raw.len() <= 1024 {
        (0..raw.len()).collect()
    } else {
        raw.iter()
            .enumerate()
            .filter_map(|(i, &v)| v.then_some(i))
            .chain((0..depth).flat_map(|i| [1usize << i, (1usize << i) - 1]))
            .chain([raw.len() - 1])
            .collect()
    };
    for &index in &probes {
        let mut value = out.clone();
        for shift in 0..depth {
            value = apply(c, value, V::Bit(index & (1 << shift) != 0));
        }
        assert_eq!(bit(value), raw[index], "{} bit {index}", expected.type_id());
    }
    probes.len()
}
#[test]
fn csharp_03_t06_w09_transition_operations_original_source_semantics() {
    let bundle = b();
    let mut contexts = 0;
    let mut observations = 0;
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
        let p = generate_csharp_practical_ordinary_transition_operations(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types: BTreeMap<_, _> = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect();
        for d in p.definitions() {
            contexts += 1;
            let closed = emitted.closure().closed();
            let oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                closed,
                &d.carrier.type_id,
            )
            .unwrap();
            assert_eq!(oracle.is_total(), d.compare_definition.is_some());
            let sample = |ty: &str, seed| relation_tests::sample(ty, seed, &types, &facts, closed);
            let state = sample(&d.state_type_id, 0);
            let response = sample(&d.response_type_id, 0);
            let event_a = sample(&d.event_type_id, 1);
            let event_b = sample(&d.event_type_id, 2);
            // Isolated state/event/response changes, ordered events and duplicates.
            let mut values = vec![];
            for (state, events, response) in [
                (state.clone(), vec![], response.clone()),
                (sample(&d.state_type_id, 1), vec![], response.clone()),
                (state.clone(), vec![], sample(&d.response_type_id, 1)),
                (
                    state.clone(),
                    vec![event_a.clone(), event_b.clone()],
                    response.clone(),
                ),
                (
                    state.clone(),
                    vec![event_b, event_a.clone()],
                    response.clone(),
                ),
                (state, vec![event_a.clone(), event_a], response),
            ] {
                let value = MonomorphicValue::Transition {
                    type_id: d.carrier.type_id.clone(),
                    state: Box::new(state.clone()),
                    events: events.clone(),
                    response: Box::new(response.clone()),
                };
                validate_monomorphic_value(&bundle, emitted.closure().roots(), closed, &value)
                    .unwrap();
                let events = MonomorphicValue::Sequence {
                    type_id: d.events_type_id.clone(),
                    elements: events,
                };
                let arguments = [&state, &events, &response].map(|v| input(v, &types));
                assert_eq!(d.event_bound_argument_index, 1);
                assert!(!bit(run(
                    &c,
                    &d.event_bound_definition,
                    vec![arguments[1].clone()]
                )));
                observations += observe(
                    &c,
                    run(&c, &d.make_definition, arguments.to_vec()),
                    &value,
                    &types,
                ) + 1;
                for (getter, expected) in [
                    (&d.state_definition, &state),
                    (&d.events_definition, &events),
                    (&d.response_definition, &response),
                ] {
                    observations += observe(
                        &c,
                        run(&c, getter, vec![input(&value, &types)]),
                        expected,
                        &types,
                    );
                }
                values.push(value);
            }
            for left in &values {
                for right in &values {
                    let args = vec![input(left, &types), input(right, &types)];
                    assert_eq!(
                        bit(run(&c, &d.equality_definition, args.clone())),
                        oracle.structural_equal(left, right).unwrap()
                    );
                    observations += 1;
                    if let Some(compare) = &d.compare_definition {
                        let expected = match oracle.canonical_compare(left, right).unwrap() {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        };
                        assert_eq!(
                            sequence_tests::number(&c, run(&c, compare, args)) as i32,
                            expected
                        );
                        observations += 1;
                    }
                }
            }
            // Raw lengths exercise inclusive 4096 and every upper u32 length bit.
            let depth = types[&d.events_type_id].depth;
            for length in [0u32, 1, 4095, 4096, 4097, 65536, u32::MAX]
                .into_iter()
                .chain((13..32).map(|i| 1u32 << i))
            {
                let encoded = sparse_cube(
                    depth,
                    (0..32)
                        .filter(|i| length & (1 << i) != 0)
                        .map(|i| (i as usize) << (depth - 5))
                        .collect(),
                );
                assert_eq!(
                    bit(run(&c, &d.event_bound_definition, vec![encoded])),
                    length > 4096,
                    "{id} length {length}"
                );
                observations += 1;
            }
            eprintln!("transition operations {id}: exact product/getters, ordered equality/compare and event bound passed");
        }
    }
    assert!(contexts >= 3 && observations > 0);
    eprintln!("transition operations: {contexts} instances, {observations} observations");
}

#[test]
fn csharp_03_t06_w09_transition_operations_integrated_definition_closure() {
    let bundle = b();
    let mut contexts = 0;
    let mut declarations = 0;
    let output =
        std::env::var_os("MPK_W09_TRANSITION_INTEGRATED_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/transition-integrated");
    let mut metrics = vec![];
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
        let standalone =
            generate_csharp_practical_ordinary_transition_operations(emitted.vir()).unwrap();
        if standalone.definitions().is_empty() {
            continue;
        }
        let combined =
            generate_csharp_practical_ordinary_structural_foundations(emitted.vir()).unwrap();
        assert_eq!(combined.transitions(), standalone.definitions());
        assert!(combined.deferred_instances().is_empty());
        let left = mpk_cert::decode_canonical_certificate(standalone.certificate_bytes()).unwrap();
        let right = mpk_cert::decode_canonical_certificate(combined.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&right).unwrap();
        let mut names = BTreeSet::new();
        for d in standalone.definitions() {
            names.extend(
                [
                    &d.make_definition,
                    &d.state_definition,
                    &d.events_definition,
                    &d.response_definition,
                    &d.event_bound_definition,
                    &d.equality_definition,
                ]
                .into_iter()
                .cloned(),
            );
            names.extend(d.compare_definition.iter().cloned());
        }
        declarations +=
            structural_equivalence_tests::same_definition_closure(&left, &right, &names)
                .unwrap()
                .len();
        // Integration only appends terms/declarations to the existing published context.
        if id == "binding-vc-transition" {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/verification-logs/transition-operations/before-integration/binding-vc-transition.hex");
            let hex = fs::read_to_string(path).unwrap();
            let previous = hex
                .trim()
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect::<Vec<_>>();
            let previous = mpk_cert::decode_canonical_certificate(&previous).unwrap();
            assert_eq!(
                &right.term_table[..previous.term_table.len()],
                previous.term_table
            );
            // Canonical encoding sorts the name table, so inserting a new name
            // may renumber name indices while preserving each declaration.
            for (old, new) in previous.declarations.iter().zip(&right.declarations) {
                assert_eq!(
                    previous.name_table[old.name as usize],
                    right.name_table[new.name as usize]
                );
                assert_eq!(old.kind, new.kind);
            }
        }
        let hex = combined
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        let file = format!("{id}.hex");
        if let Some(output) = &output {
            fs::create_dir_all(output).unwrap();
            fs::write(output.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"metadata":serde_json::from_slice::<Value>(&combined.canonical_bytes()).unwrap()}));
        contexts += 1;
    }
    assert_eq!(contexts, 3);
    if let Some(output) = &output {
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
    eprintln!("transition integration: {contexts} contexts, {declarations} exact transitive definitions; prior terms/declarations preserved");
}
