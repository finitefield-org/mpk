use super::*;
use core_eval::{apply, bit as observed_bit, run, sparse_cube, V};
use mpk_cert::Certificate;
use sequence_tests::{number, observe, word};

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = relation_tests::sources();
    let requests = read("ordinary-foundation/sequence-sources/requests.json");
    let responses = read("ordinary-foundation/sequence-sources/responses.json");
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    sources.push((
        "float-sequence".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    let requests = read("ordinary-foundation/construction-sources/requests.json");
    let responses = read("ordinary-foundation/construction-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 3);
    assert_eq!(responses.as_array().unwrap().len(), 3);
    for (request, response) in requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
    {
        assert_eq!(request["id"], response["id"]);
        sources.push((
            request["id"].as_str().unwrap().into(),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    sources
}

fn op<'a>(
    d: &'a OrdinaryConstructionDefinition,
    suffix: &str,
) -> &'a OrdinaryConstructionOperation {
    d.operations
        .iter()
        .find(|o| o.operation_id == format!("{}.{suffix}", d.carrier.type_id))
        .unwrap()
}
fn failure(
    c: &Certificate,
    operation: &OrdinaryConstructionOperation,
    label: &str,
    arguments: &[V],
) -> bool {
    let f = operation
        .failures
        .iter()
        .find(|f| f.label == label)
        .unwrap();
    observed_bit(run(
        c,
        f.definition
            .as_ref()
            .expect("storage predicate, not external ownership"),
        f.argument_indices
            .iter()
            .map(|&i| arguments[i].clone())
            .collect(),
    ))
}
// Independent little-endian sparse layout: two role bits, with leading
// padding in the length/bitmap fields. Only initialized cells contain values.
fn encoded(
    length: u32,
    child_depth: u32,
    elements: &BTreeMap<u32, Vec<bool>>,
) -> (u32, BTreeSet<usize>) {
    let depth = 16 + child_depth;
    let mut ones = BTreeSet::new();
    for bit in 0..32 {
        if length & (1 << bit) != 0 {
            ones.insert(bit << (depth - 5));
        }
    }
    for (&index, value) in elements {
        ones.insert(2 | ((index as usize) << (2 + child_depth)));
        for (leaf, &set) in value.iter().enumerate() {
            if set {
                ones.insert(1 | ((index as usize) << 2) | (leaf << 16));
            }
        }
    }
    (depth, ones)
}
fn check_sparse(c: &Certificate, value: V, depth: u32, expected: &BTreeSet<usize>) {
    // Sample padding; all nonzero expected leaves and neighboring addresses
    // are checked, plus every selector bit under every role.
    let limit = 1usize << depth;
    let mut addresses = BTreeSet::from([0, 1, 2, 3, limit - 1]);
    for bit in 2..depth {
        for role in 0..4 {
            addresses.insert((1 << bit) | role);
        }
    }
    for &address in expected {
        addresses.insert(address);
        addresses.insert(address.saturating_sub(1));
        addresses.insert((address + 1).min(limit - 1));
    }
    for address in addresses {
        let mut leaf = value.clone();
        for selector in 0..depth {
            leaf = apply(c, leaf, V::Bit(address & (1 << selector) != 0));
        }
        assert_eq!(
            observed_bit(leaf),
            expected.contains(&address),
            "construction leaf {address}"
        );
    }
}

#[test]
fn csharp_03_t06_w09_constructions_original_sources_and_mutations() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_CONSTRUCTIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/construction-operations");
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
        let p = generate_csharp_practical_ordinary_constructions(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let expected = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
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
        assert_eq!(
            import_csharp_practical_ordinary_constructions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "static_transformers",
            "definitions",
            "certificate_sha256",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_constructions(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        for field in [
            "normal_definition",
            "failures",
            "argument_type_ids",
            "result_type_id",
        ] {
            let mut changed = data.clone();
            changed["definitions"][0]["operations"][0][field] = json!([]);
            assert!(import_csharp_practical_ordinary_constructions(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_constructions(
            &p.canonical_bytes(),
            &changed,
            emitted.vir()
        )
        .is_err());
        for d in p.definitions() {
            assert_eq!(d.operations.len(), 5);
            for suffix in ["read", "fill", "rewrite", "freeze"] {
                let f = &op(d, suffix).failures[0];
                assert_eq!(f.label, "ownership");
                assert!(f.definition.is_none());
                assert_eq!(f.argument_indices, [0]);
            }
        }
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
    assert_eq!(metrics.len(), 5);
    eprintln!(
        "construction certificates: {} actual-source contexts",
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

#[test]
fn csharp_03_t06_w09_constructions_storage_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(check_semantics)
        .unwrap()
        .join()
        .unwrap();
}
fn check_semantics() {
    let bundle = b();
    let mut types_checked = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_constructions(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(emitted.vir())
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            // A declaration identity can recur with different source content.
            // Never skip its semantics based only on the type ID.
            if !types_checked.insert((emitted.vir().hash().to_owned(), d.element_type_id.clone())) {
                continue;
            }
            eprintln!("construction semantics {id}: {}", d.element_type_id);
            let child = types[&d.element_type_id].depth;
            let sample = |seed| {
                let v = relation_tests::sample(
                    &d.element_type_id,
                    seed,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &v,
                )
                .unwrap();
                relation_tests::storage(&v, &types)
            };
            let read = op(d, "read");
            let allocate = op(d, "allocate");
            let fill = op(d, "fill");
            let rewrite = op(d, "rewrite");
            let freeze = op(d, "freeze");
            for length in [
                0u32,
                1,
                3,
                4096,
                4097,
                16384,
                16385,
                u32::MAX,
                i32::MIN as u32,
                i32::MAX as u32,
            ] {
                let args = [word(length), V::Bit(false)];
                assert_eq!(
                    failure(&c, allocate, "negative_length", &args),
                    (length as i32) < 0
                );
                assert_eq!(
                    failure(&c, allocate, "construction_bound", &args),
                    length > 16384
                );
            }
            for length in [0u32, 1, 3] {
                // False always means explicitly uninitialized storage. True is
                // tested as a storage equation only; it is not a default proof.
                for default in [false, true] {
                    let state = run(
                        &c,
                        &allocate.normal_definition,
                        vec![word(length), V::Bit(default)],
                    );
                    let mut expected = if default {
                        (0..length).map(|i| (i, vec![false; 1 << child])).collect()
                    } else {
                        BTreeMap::new()
                    };
                    let (depth, ones) = encoded(length, child, &expected);
                    assert_eq!(depth, d.carrier.depth);
                    check_sparse(&c, state.clone(), depth, &ones);
                    assert_eq!(
                        number(&c, run(&c, &d.length_definition, vec![state.clone()])),
                        length
                    );
                    assert_eq!(
                        observed_bit(run(&c, &d.complete_definition, vec![state.clone()])),
                        default || length == 0
                    );
                    for index in [0u32, 1, 2, 3, 4095, 4096, 16383, 16384, u32::MAX] {
                        let args = [state.clone(), word(index)];
                        assert_eq!(failure(&c, read, "index_range", &args), index >= length);
                        if index < length {
                            assert_eq!(failure(&c, read, "uninitialized", &args), !default);
                            assert_eq!(failure(&c, fill, "already_initialized", &args), default);
                        }
                    }
                    let mut state = state;
                    for index in (0..length).rev() {
                        let value = sample(index as usize + 1);
                        let operation = if default { rewrite } else { fill };
                        assert_eq!(
                            failure(&c, rewrite, "incomplete", std::slice::from_ref(&state)),
                            expected.len() != length as usize
                        );
                        let args = vec![
                            state.clone(),
                            word(index),
                            if child == 0 {
                                V::Bit(value[0])
                            } else {
                                V::Cube(value.clone())
                            },
                        ];
                        let last_gate = if default {
                            "incomplete"
                        } else {
                            "already_initialized"
                        };
                        assert!(!failure(&c, operation, last_gate, &args));
                        state = run(&c, &operation.normal_definition, args);
                        expected.insert(index, value.clone());
                        assert!(failure(
                            &c,
                            fill,
                            "already_initialized",
                            &[state.clone(), word(index)]
                        ));
                        let (_, ones) = encoded(length, child, &expected);
                        check_sparse(&c, state.clone(), depth, &ones);
                        observe(
                            &c,
                            run(
                                &c,
                                &read.normal_definition,
                                vec![state.clone(), word(index)],
                            ),
                            &value,
                        );
                        assert_eq!(
                            observed_bit(run(&c, &d.complete_definition, vec![state.clone()])),
                            expected.len() == length as usize
                        );
                    }
                    assert!(!failure(
                        &c,
                        freeze,
                        "incomplete",
                        std::slice::from_ref(&state)
                    ));
                    assert!(!failure(
                        &c,
                        freeze,
                        "publication_bound",
                        std::slice::from_ref(&state)
                    ));
                    let published = run(&c, &freeze.normal_definition, vec![state]);
                    let mut ones = BTreeSet::new();
                    let output_depth = types[&d.published_type_id].depth;
                    for bit in 0..32 {
                        if length & (1 << bit) != 0 {
                            ones.insert(bit << (output_depth - 5));
                        }
                    }
                    for (&index, value) in &expected {
                        for (leaf, &set) in value.iter().enumerate() {
                            if set {
                                ones.insert(1 | ((index as usize) << 1) | (leaf << 13));
                            }
                        }
                    }
                    check_sparse(&c, published, output_depth, &ones);
                }
            }
            for length in [4096, 4097, 16384, 16385] {
                let (depth, ones) = encoded(length, child, &BTreeMap::new());
                let state = sparse_cube(depth, ones);
                assert_eq!(
                    failure(
                        &c,
                        freeze,
                        "publication_bound",
                        std::slice::from_ref(&state)
                    ),
                    length > 4096
                );
                assert!(failure(&c, freeze, "incomplete", &[state]));
            }
            let value = sample(1);
            // Make an aliased raw write observable even for one-bit elements.
            let replacement = value.iter().map(|v| !v).collect::<Vec<_>>();
            let expected = BTreeMap::from([(0, value)]);
            let (depth, ones) = encoded(1, child, &expected);
            for bit in 14..32 {
                let state = sparse_cube(depth, ones.clone());
                let args = vec![
                    state.clone(),
                    word(1 << bit),
                    if child == 0 {
                        V::Bit(replacement[0])
                    } else {
                        V::Cube(replacement.clone())
                    },
                ];
                assert!(failure(&c, fill, "index_range", &args));
                // Even the raw update uses full-word equality and cannot alias.
                check_sparse(&c, run(&c, &fill.normal_definition, args), depth, &ones);
            }
        }
    }
    assert_eq!(types_checked.len(), 5);
    eprintln!(
        "construction storage semantics: {} source/type contexts",
        types_checked.len()
    );
}

#[test]
fn csharp_03_t06_w09_construction_full_bitmap_boundary() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let (_, row, facts) = sources()
                .into_iter()
                .find(|(id, _, _)| id == "bool-construction")
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
            let p = generate_csharp_practical_ordinary_constructions(emitted.vir()).unwrap();
            let d = p
                .definitions()
                .iter()
                .find(|d| d.element_type_id == ty("bool"))
                .unwrap();
            let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            // Same full bitmap with an invalid length must not become complete by
            // the common fold's capacity clamp. The last missing bit is also read.
            for (length, initialized, complete) in [
                (16385, 16384, false),
                (16384, 16384, true),
                (16384, 16383, false),
                (16383, 16383, true),
            ] {
                eprintln!("construction bitmap: length {length}, initialized {initialized}");
                let elements = (0..initialized).map(|i| (i, vec![false])).collect();
                let (depth, ones) = encoded(length, 0, &elements);
                let value = sparse_cube(depth, ones);
                assert_eq!(
                    observed_bit(run(&c, &d.complete_definition, vec![value])),
                    complete
                );
            }
            // Publication retains exactly the first 4096 initialized cells,
            // including the final address; the normal body does not certify its
            // separate ownership, public-element or aggregate-cell obligations.
            let elements = (0..4096).map(|i| (i, vec![i == 4095])).collect();
            let (depth, ones) = encoded(4096, 0, &elements);
            let value = sparse_cube(depth, ones);
            let freeze = op(d, "freeze");
            assert!(!failure(
                &c,
                freeze,
                "publication_bound",
                std::slice::from_ref(&value)
            ));
            let output = run(&c, &freeze.normal_definition, vec![value]);
            let expected = BTreeSet::from([12 << (13 - 5), 1 | (4095 << 1)]);
            check_sparse(&c, output, 13, &expected);
        })
        .unwrap()
        .join()
        .unwrap();
}
