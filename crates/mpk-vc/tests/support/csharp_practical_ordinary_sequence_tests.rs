use super::*;
use core_eval::{apply, bit as observed_bit, run, sparse_cube, V};
use mpk_cert::Certificate;

pub(super) fn word(value: u32) -> V {
    V::Cube((0..32).map(|i| value & (1 << i) != 0).collect())
}
pub(super) fn number(c: &Certificate, value: V) -> u32 {
    (0..32).fold(0, |result, i| {
        let mut leaf = value.clone();
        for selector in 0..5 {
            leaf = apply(c, leaf, V::Bit(i & (1 << selector) != 0));
        }
        result | (u32::from(observed_bit(leaf)) << i)
    })
}
fn storage(
    value: &MonomorphicValue,
    d: &OrdinarySequenceOperations,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> V {
    let MonomorphicValue::Sequence { elements, .. } = value else {
        panic!()
    };
    let child_depth = types[&d.element_type_id].depth;
    let array_padding = d.carrier.depth - 1 - 12 - child_depth;
    let mut ones = BTreeSet::new();
    for bit in 0..32 {
        if elements.len() & (1 << bit) != 0 {
            ones.insert(bit << (d.carrier.depth - 5));
        }
    }
    for (index, child) in elements.iter().enumerate() {
        let base = 1 | (index << (1 + array_padding));
        for (leaf, set) in relation_tests::storage(child, types)
            .into_iter()
            .enumerate()
        {
            if set {
                ones.insert(base | (leaf << (1 + array_padding + 12)));
            }
        }
    }
    sparse_cube(d.carrier.depth, ones)
}
pub(super) fn observe(c: &Certificate, value: V, expected: &[bool]) {
    let depth = expected.len().trailing_zeros();
    let addresses = if depth <= 10 {
        (0..expected.len()).collect::<BTreeSet<_>>()
    } else {
        // Observe all nonzero leaves, their neighbors, high selectors and both
        // endpoints without traversing millions of inactive padding leaves.
        let mut addresses = BTreeSet::from([0, expected.len() - 1]);
        addresses.extend((0..depth).map(|bit| 1 << bit));
        for (i, &set) in expected.iter().enumerate() {
            if set {
                addresses.insert(i);
                addresses.insert(i.saturating_sub(1));
                addresses.insert((i + 1).min(expected.len() - 1));
            }
        }
        addresses
    };
    for address in addresses {
        let mut leaf = value.clone();
        for selector in 0..depth {
            leaf = apply(c, leaf, V::Bit(address & (1 << selector) != 0));
        }
        assert_eq!(
            observed_bit(leaf),
            expected[address],
            "read address {address}"
        );
    }
}

#[test]
fn csharp_03_t06_w09_sequences_original_sources_and_boundaries() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| check_sources(true))
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_sequences_original_source_certificates() {
    check_sources(false);
}

#[test]
fn csharp_03_t06_w09_sequences_equal_length_element_order() {
    let bundle = b();
    let mut contexts = 0;
    for (id, row, facts) in relation_tests::sources()
        .into_iter()
        .filter(|(id, _, _)| matches!(id.as_str(), "binding-vc-bounded_sequence" | "nested-box"))
    {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_sequences(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(p.definitions().len(), 1);
        let d = &p.definitions()[0];
        let make = |seeds: &[usize]| MonomorphicValue::Sequence {
            type_id: d.carrier.type_id.clone(),
            elements: seeds
                .iter()
                .map(|&seed| {
                    relation_tests::sample(
                        &d.element_type_id,
                        seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    )
                })
                .collect(),
        };
        let relation = generate_structural_program(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &d.carrier.type_id,
        )
        .unwrap();
        // Same lengths cannot decide either result. Check both a late differing
        // element and first-difference precedence when later elements disagree.
        for (left, right) in [
            (make(&[1, 2]), make(&[1, 1])),
            (make(&[1, 2]), make(&[2, 1])),
        ] {
            for v in [&left, &right] {
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    v,
                )
                .unwrap();
            }
            assert!(
                !relation.structural_equal(&left, &right).unwrap(),
                "{id}: distinct fixture required"
            );
            for (a, z) in [(&left, &right), (&right, &left)] {
                let args = vec![storage(a, d, &types), storage(z, d, &types)];
                assert!(!observed_bit(run(&c, &d.equality_definition, args.clone())));
                let expected = match relation.canonical_compare(a, z).unwrap() {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                assert_ne!(expected, 0);
                assert_eq!(
                    number(&c, run(&c, d.compare_definition.as_ref().unwrap(), args)) as i32,
                    expected
                );
            }
        }
        contexts += 1;
    }
    assert_eq!(contexts, 2);
}

#[test]
fn csharp_03_t06_w09_sequence_index_high_bits_never_alias() {
    let bundle = b();
    let (_, row, facts) = relation_tests::sources()
        .into_iter()
        .find(|(id, _, _)| id == "binding-vc-bounded_sequence")
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let program = generate_csharp_practical_ordinary_sequences(emitted.vir()).unwrap();
    let d = program
        .definitions()
        .iter()
        .find(|d| d.element_type_id == ty("i32"))
        .unwrap();
    let certificate = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
    let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
        .unwrap()
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect::<BTreeMap<_, _>>();
    let value = MonomorphicValue::Sequence {
        type_id: d.carrier.type_id.clone(),
        elements: [123, 456]
            .map(|value| MonomorphicValue::Signed {
                type_id: ty("i32"),
                value: value.to_string(),
            })
            .to_vec(),
    };
    validate_monomorphic_value(
        &bundle,
        emitted.closure().roots(),
        emitted.closure().closed(),
        &value,
    )
    .unwrap();
    let encoded = storage(&value, d, &types);
    for high_bit in 12..32 {
        for low_bits in [0, 1] {
            // Every high index bit, including the i32 sign bit, would select
            // a nonzero stored element if only the 12 address bits were used.
            let index = (1u32 << high_bit) | low_bits;
            assert!(observed_bit(run(
                &certificate,
                &d.index_range_definition,
                vec![encoded.clone(), word(index)]
            )));
            assert_eq!(
                number(
                    &certificate,
                    run(
                        &certificate,
                        &d.read_definition,
                        vec![encoded.clone(), word(index)]
                    )
                ),
                0
            );
        }
    }
}

fn check_sources(observe_values: bool) {
    let bundle = b();
    let mut sources = relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .collect::<Vec<_>>();
    let requests = read("ordinary-foundation/sequence-sources/requests.json");
    let responses = read("ordinary-foundation/sequence-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    sources.push((
        "float-sequence".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    let output = std::env::var_os("MPK_W09_SEQUENCES_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/sequence-operations");
    let mut metrics = vec![];
    let mut full_capacity = false;
    let mut float_contexts = 0;
    let mut reads = 0;
    for (id, row, facts) in sources {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_ordinary_sequences(emitted.vir()).unwrap();
        let sequences = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.bounded_sequence.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            sequences,
            program
                .definitions()
                .iter()
                .map(|d| d.carrier.type_id.as_str())
                .collect()
        );
        if program.definitions().is_empty() {
            continue;
        }
        eprintln!(
            "sequence source {id}: {} instances",
            program.definitions().len()
        );
        let certificate =
            mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&certificate).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in program.definitions() {
            if !observe_values {
                continue;
            }
            let make = |seeds: &[usize]| MonomorphicValue::Sequence {
                type_id: d.carrier.type_id.clone(),
                elements: seeds
                    .iter()
                    .map(|&seed| {
                        relation_tests::sample(
                            &d.element_type_id,
                            seed,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        )
                    })
                    .collect(),
            };
            let mut values = vec![make(&[]), make(&[0]), make(&[1, 2])];
            if !full_capacity && d.element_type_id == ty("i32") {
                let mut value = make(&[1]);
                let MonomorphicValue::Sequence { elements, .. } = &mut value else {
                    panic!()
                };
                elements.resize(4096, elements[0].clone());
                elements[4095] = MonomorphicValue::Signed {
                    type_id: ty("i32"),
                    value: "12345".into(),
                };
                values.push(value);
                full_capacity = true;
            }
            let relation = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &d.carrier.type_id,
            )
            .unwrap();
            assert_eq!(d.compare_definition.is_some(), relation.is_total());
            if d.element_type_id == ty("f32") {
                float_contexts += 1;
                assert!(d.compare_definition.is_none());
            }
            for value in &values {
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    value,
                )
                .unwrap();
                let MonomorphicValue::Sequence { elements, .. } = value else {
                    panic!()
                };
                let encoded = storage(value, d, &types);
                // Cross-check the independent sparse packer on manageable cubes.
                if d.carrier.depth <= 20 {
                    let dense = relation_tests::storage(value, &types);
                    observe(&certificate, encoded.clone(), &dense);
                }
                assert_eq!(
                    number(
                        &certificate,
                        run(&certificate, &d.length_definition, vec![encoded.clone()])
                    ),
                    elements.len() as u32
                );
                let mut indices = BTreeSet::from([
                    -1i32,
                    i32::MIN,
                    i32::MAX,
                    0,
                    1,
                    4095,
                    4096,
                    8192,
                    1 << 20,
                    elements.len() as i32,
                ]);
                if !elements.is_empty() {
                    indices.insert(elements.len() as i32 - 1);
                }
                for index in indices {
                    let fail = index < 0 || index as usize >= elements.len() || index >= 4096;
                    assert_eq!(
                        observed_bit(run(
                            &certificate,
                            &d.index_range_definition,
                            vec![encoded.clone(), word(index as u32)]
                        )),
                        fail,
                        "{id} index {index}"
                    );
                    let result = run(
                        &certificate,
                        &d.read_definition,
                        vec![encoded.clone(), word(index as u32)],
                    );
                    let expected = if fail {
                        vec![false; 1 << types[&d.element_type_id].depth]
                    } else {
                        relation_tests::storage(&elements[index as usize], &types)
                    };
                    observe(&certificate, result, &expected);
                    reads += 1;
                }
            }
            // Full-capacity iteration is already covered by the shared fold
            // tests. Here compare small values, prefixes and non-reflexive NaN.
            for left in &values[..3] {
                for right in &values[..3] {
                    let arguments = vec![storage(left, d, &types), storage(right, d, &types)];
                    assert_eq!(
                        observed_bit(run(&certificate, &d.equality_definition, arguments.clone())),
                        relation.structural_equal(left, right).unwrap()
                    );
                    if let Some(compare) = &d.compare_definition {
                        let expected = match relation.canonical_compare(left, right).unwrap() {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        };
                        assert_eq!(
                            number(&certificate, run(&certificate, compare, arguments)) as i32,
                            expected
                        );
                    }
                }
            }
        }
        let metadata = program.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_sequences(
                &metadata,
                program.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            program
        );
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "certificate_sha256",
            "static_transformers",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_sequences(
                &serde_json::to_vec(&changed).unwrap(),
                program.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = program.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_sequences(&metadata, &changed, emitted.vir()).is_err()
        );
        let file = format!("{id}.hex");
        let hex = program
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
        metrics.push(json!({"id":id,"file":file,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"metadata":data}));
    }
    assert!(metrics.len() >= 10);
    if observe_values {
        assert!(full_capacity && float_contexts == 1 && reads >= 300);
        eprintln!("sequence operations: {} source contexts; {reads} indexed reads; full 4096 capacity and NaN covered", metrics.len());
    } else {
        eprintln!(
            "sequence certificates: {} source contexts; no value observations in this pass",
            metrics.len()
        );
    }
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
