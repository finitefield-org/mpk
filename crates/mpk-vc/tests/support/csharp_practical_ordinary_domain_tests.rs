use super::*;
use core_eval::{apply, bit as observed_bit, run, V};
use relation_tests::{sample, storage};
fn sources() -> Vec<(String, Value, Value)> {
    relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .collect()
}

fn observed_count(c: &mpk_cert::encode::Certificate, value: V) -> u32 {
    (0..32).fold(0, |n, i| {
        let mut v = value.clone();
        for j in 0..5 {
            v = apply(c, v, V::Bit(i & (1 << j) != 0));
        }
        n | (u32::from(observed_bit(v)) << i)
    })
}
pub(super) fn cells(v: &MonomorphicValue) -> u32 {
    use MonomorphicValue::*;
    1 + match v {
        String { utf16, .. } => utf16.len() as u32,
        Product { fields, .. } => fields.iter().map(|f| cells(&f.value)).sum(),
        Array { elements, .. } | Sequence { elements, .. } | OrderedSet { elements, .. } => {
            elements.iter().map(cells).sum()
        }
        OrderedEntry { key, value, .. } => cells(key) + cells(value),
        OrderedMap { entries, .. } => entries
            .iter()
            .map(|e| cells(&e.key) + cells(&e.value))
            .sum(),
        Option { value, .. } | BoundaryPresence { value, .. } => {
            value.as_deref().map(cells).unwrap_or(0)
        }
        TaggedSum { payload, .. } => payload.iter().map(cells).sum(),
        Money {
            amount, currency, ..
        } => cells(amount) + cells(currency),
        Transition {
            state,
            events,
            response,
            ..
        } => cells(state) + events.iter().map(cells).sum::<u32>() + cells(response),
        ClosedException { payload, .. } => payload.as_deref().map(cells).unwrap_or(0),
        _ => 0,
    }
}
fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
// Pack a map's length and active entry bits at the same concrete addresses as
// the dense storage encoder. No omitted bit is valid by assumption: the core
// domain still checks every region it demands, reading omitted bits as false.
pub(super) fn sparse_map_storage(
    value: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> (u32, BTreeSet<usize>) {
    let MonomorphicValue::OrderedMap { entries, .. } = value else {
        panic!()
    };
    let carrier = &types[value.type_id()];
    let OrdinaryShape::Sequence { capacity, element } = &carrier.shape else {
        panic!()
    };
    let OrdinaryShape::Product { fields } = element.as_ref() else {
        panic!()
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].id, "key");
    assert_eq!(fields[1].id, "value");
    let indices = capacity.next_power_of_two().trailing_zeros();
    let mut bits = BTreeSet::new();
    for i in 0..32 {
        if entries.len() & (1 << i) != 0 {
            bits.insert(i << (carrier.depth - 5));
        }
    }
    for (index, entry) in entries.iter().enumerate() {
        let children = [storage(&entry.key, types), storage(&entry.value, types)];
        let depths = children.each_ref().map(|v| v.len().trailing_zeros());
        let max = depths[0].max(depths[1]);
        let entry_depth = 1 + max;
        let array_padding = carrier.depth - 1 - indices - entry_depth;
        for (role, child) in children.iter().enumerate() {
            let base = 1 | (index << (1 + array_padding)) | (role << (1 + array_padding + indices));
            let shift = 1 + array_padding + indices + 1 + max - depths[role];
            for (leaf, &set) in child.iter().enumerate() {
                if set {
                    bits.insert(base | (leaf << shift));
                }
            }
        }
    }
    (carrier.depth, bits)
}

#[test]
fn csharp_03_t06_w09_sparse_map_input_matches_dense_storage() {
    let bundle = b();
    let mut count = 0;
    for (id, row, facts) in sources() {
        if !matches!(
            id.as_str(),
            "binding-vc-ordered_map" | "boundary-map-decimal"
        ) {
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
        let carriers = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = carriers
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let entry = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .find(|e| e["template_id"] == "mpk.csharp.semantic.ordered_map.v1")
            .unwrap();
        for seed in 0..=2 {
            let value = sample(
                entry["instance_id"].as_str().unwrap(),
                seed,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            let (depth, sparse) = sparse_map_storage(&value, &types);
            let dense = storage(&value, &types);
            assert_eq!(dense.len(), 1usize << depth);
            assert_eq!(
                sparse,
                dense
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &b)| b.then_some(i))
                    .collect()
            );
            count += 1;
        }
    }
    assert_eq!(count, 6);
}

#[test]
fn csharp_03_t06_w09_domains_source_string_compound_map_order() {
    string_compound_map_order(None);
}
#[test]
fn csharp_03_t06_w09_domains_source_string_duplicate_probe() {
    string_compound_map_order(Some(("boundary-map-string", "duplicate")));
}
fn string_compound_map_order(filter: Option<(&'static str, &'static str)>) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let bundle = b();
            let mut count = 0;
            for (id, row, facts) in domain_sources::sources() {
                if !matches!(id.as_str(), "boundary-map-string" | "boundary-map-compound") {
                    continue;
                }
                if filter.is_some_and(|(context, _)| context != id) {
                    continue;
                }
                let compound = id == "boundary-map-compound";
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let program = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
                let types = program
                    .definitions()
                    .iter()
                    .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
                    .collect::<BTreeMap<_, _>>();
                let entry = emitted
                    .closure()
                    .closed()
                    .entries()
                    .iter()
                    .find(|e| e["template_id"] == "mpk.csharp.semantic.ordered_map.v1")
                    .unwrap();
                let domain = program
                    .definitions()
                    .iter()
                    .find(|d| d.carrier.type_id == entry["instance_id"].as_str().unwrap())
                    .unwrap();
                let cert =
                    mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
                for (case, valid) in [
                    ("duplicate", false),
                    ("descending", false),
                    ("increasing", true),
                ] {
                    if filter.is_some_and(|(_, selected)| selected != case) {
                        continue;
                    }
                    let mut value = sample(
                        &domain.carrier.type_id,
                        2,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let MonomorphicValue::OrderedMap { entries, .. } = &mut value else {
                        panic!()
                    };
                    for (i, entry) in entries.iter_mut().enumerate() {
                        let order = if case == "duplicate" {
                            0
                        } else if case == "descending" {
                            1 - i
                        } else {
                            i
                        };
                        if compound {
                            // Equal first fields force ordering to reach the source
                            // product's second field, rather than stop at its string.
                            let MonomorphicValue::Product { fields, .. } = entry.key.as_mut()
                            else {
                                panic!()
                            };
                            let MonomorphicValue::String { utf16, .. } = fields[0].value.as_mut()
                            else {
                                panic!()
                            };
                            *utf16 = vec![0xd800];
                            let MonomorphicValue::Signed { value, .. } = fields[1].value.as_mut()
                            else {
                                panic!()
                            };
                            *value = order.to_string();
                        } else {
                            let MonomorphicValue::String { utf16, .. } = entry.key.as_mut() else {
                                panic!()
                            };
                            *utf16 = if order == 0 {
                                vec![0xd800]
                            } else {
                                vec![0xd800, 0]
                            };
                        }
                    }
                    assert_eq!(
                        validate_monomorphic_value(
                            &bundle,
                            emitted.closure().roots(),
                            emitted.closure().closed(),
                            &value
                        )
                        .is_ok(),
                        valid,
                        "{id} {case}"
                    );
                    let (depth, bits) = sparse_map_storage(&value, &types);
                    eprintln!(
                        "string/compound map domain {id}: {case}, C{depth}, {} true input leaves",
                        bits.len()
                    );
                    let input = core_eval::sparse_cube(depth, bits);
                    assert_eq!(
                        observed_count(&cert, run(&cert, &domain.count_definition, vec![input])),
                        if valid { cells(&value) } else { 65537 },
                        "{id} {case}"
                    );
                    count += 1;
                }
            }
            assert_eq!(count, if filter.is_some() { 1 } else { 6 });
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_aggregate_source_reconstruction() {
    let bundle = b();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut contexts = 0;
    for (id, row, facts) in sources() {
        let expected = match id.as_str() {
            "string" => vec![14],
            "binding-vc-bounded_sequence" => vec![12],
            "construction-vc-positive_constructor" => vec![],
            _ => continue,
        };
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
        let p = generate_csharp_practical_ordinary_aggregate_folds(vir).unwrap();
        let metadata = p.canonical_bytes();
        let certificate = p.certificate_bytes();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.index_bits)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            import_csharp_practical_ordinary_aggregate_folds(&metadata, certificate, vir).unwrap(),
            p
        );
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        assert_eq!(
            data["static_transformers"],
            if expected.is_empty() { 0 } else { 8217 }
        );
        for key in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut changed = data.clone();
            changed[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_aggregate_folds(
                &serde_json::to_vec(&changed).unwrap(),
                certificate,
                vir
            )
            .is_err());
        }
        let mut corrupt = certificate.to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_aggregate_folds(&metadata, &corrupt, vir).is_err()
        );
        if let Some((old_metadata, old_certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_aggregate_folds(
                old_metadata,
                old_certificate,
                vir
            )
            .is_err());
        }
        previous = Some((metadata, certificate.to_vec()));
        contexts += 1;
    }
    assert_eq!(contexts, 3);
}
#[test]
fn csharp_03_t06_w09_domains_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_DOMAINS_OUT").map(std::path::PathBuf::from);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/domains");
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut metrics = vec![];
    for (id, row, facts) in sources() {
        eprintln!("domain source {id}");
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_domains(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let metadata = p.canonical_bytes();
        let bytes = p.certificate_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_domains(&metadata, bytes, emitted.vir()).unwrap(),
            p
        );
        let cert = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        for key in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "static_transformers",
            "certificate_sha256",
            "definitions",
        ] {
            let mut changed = data.clone();
            changed[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_domains(
                &serde_json::to_vec(&changed).unwrap(),
                bytes,
                emitted.vir()
            )
            .is_err());
        }
        let mut corrupt = bytes.to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_domains(&metadata, &corrupt, emitted.vir()).is_err()
        );
        let file = format!("{id}.hex");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(p) = &output {
            fs::write(p.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixtures.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"metadata":data}));
    }
    if let Some(p) = output {
        fs::write(
            p.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixtures.join("certificates.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}
#[test]
fn csharp_03_t06_w09_domains_source_small_values() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
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
                let p = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
                let types = p
                    .definitions()
                    .iter()
                    .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
                    .collect::<BTreeMap<_, _>>();
                let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                for d in p.definitions() {
                    // This focused pass covers scalars and small product/sum storage.
                    // Full sequences, deep padding, and role boundaries have separate tests.
                    if d.carrier.depth > 10 {
                        continue;
                    }
                    eprintln!("domain observation {id} {}", d.carrier.type_id);
                    for seed in [0, 1, 2] {
                        let value = sample(
                            &d.carrier.type_id,
                            seed,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        );
                        validate_monomorphic_value(
                            &bundle,
                            emitted.closure().roots(),
                            emitted.closure().closed(),
                            &value,
                        )
                        .unwrap();
                        let encoded = storage(&value, &types);
                        let actual = observed_count(
                            &cert,
                            run(&cert, &d.count_definition, vec![input(encoded)]),
                        );
                        assert_eq!(
                            actual,
                            cells(&value),
                            "{id} {} seed {seed}",
                            d.carrier.type_id
                        );
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_domains_source_collections_and_rejections() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let selected = [
                "bounded_sequence",
                "ordered_map",
                "ordered_set",
                "validation",
                "transition",
                "money",
            ];
            for (id, row, facts) in sources() {
                let Some(template) = id
                    .strip_prefix("binding-vc-")
                    .filter(|s| selected.contains(s))
                else {
                    continue;
                };
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let p = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
                let types = p
                    .definitions()
                    .iter()
                    .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
                    .collect::<BTreeMap<_, _>>();
                let entry = emitted
                    .closure()
                    .closed()
                    .entries()
                    .iter()
                    .find(|e| e["template_id"] == format!("mpk.csharp.semantic.{template}.v1"))
                    .unwrap();
                let d = p
                    .definitions()
                    .iter()
                    .find(|d| d.carrier.type_id == entry["instance_id"].as_str().unwrap())
                    .unwrap();
                let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                let observe = |value: &MonomorphicValue| {
                    observed_count(
                        &cert,
                        run(
                            &cert,
                            &d.count_definition,
                            vec![input(storage(value, &types))],
                        ),
                    )
                };
                // Nonempty collections make the logical wrappers observable.
                let value = sample(
                    &d.carrier.type_id,
                    if template == "validation" { 1 } else { 2 },
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &value,
                )
                .unwrap();
                eprintln!(
                    "collection domain {template}: valid value, {} logical cells",
                    cells(&value)
                );
                assert_eq!(observe(&value), cells(&value), "{template}");
                if let OrdinaryShape::Sequence { capacity, .. } = d.carrier.shape {
                    let mut excessive = storage(&value, &types);
                    // Length occupies the all-zero prefix followed by five selectors.
                    let shift = d.carrier.depth - 5;
                    for i in 0..32 {
                        excessive[i << shift] = (capacity + 1) & (1 << i) != 0;
                    }
                    assert_eq!(
                        observed_count(
                            &cert,
                            run(&cert, &d.count_definition, vec![input(excessive)])
                        ),
                        65537,
                        "{template} excessive length"
                    );
                    let mut tail = storage(&value, &types);
                    // Highest physical leaf is outside the two active elements.
                    *tail.last_mut().unwrap() = true;
                    eprintln!("collection domain {template}: dirty inactive tail");
                    assert_eq!(
                        observed_count(&cert, run(&cert, &d.count_definition, vec![input(tail)])),
                        65537,
                        "{template} tail"
                    );
                }
                let mut invalid = value.clone();
                match &mut invalid {
                    MonomorphicValue::OrderedMap { entries, .. } => {
                        entries[1].key = entries[0].key.clone()
                    }
                    MonomorphicValue::OrderedSet { elements, .. } => {
                        elements[1] = elements[0].clone()
                    }
                    MonomorphicValue::TaggedSum { payload, .. } if template == "validation" => {
                        let MonomorphicValue::Sequence { elements, .. } = &mut payload[0] else {
                            panic!()
                        };
                        elements.clear();
                    }
                    _ => continue,
                }
                assert!(validate_monomorphic_value(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &invalid
                )
                .is_err());
                eprintln!("collection domain {template}: reject duplicate or empty-invalid arm");
                assert_eq!(observe(&invalid), 65537, "{template} invalid value");
                if template == "validation" {
                    let mut boundary = value.clone();
                    let MonomorphicValue::TaggedSum { payload, .. } = &mut boundary else {
                        panic!()
                    };
                    let MonomorphicValue::Sequence { elements, .. } = &mut payload[0] else {
                        panic!()
                    };
                    elements.resize(256, elements[0].clone());
                    validate_monomorphic_value(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &boundary,
                    )
                    .unwrap();
                    eprintln!("collection domain validation: inclusive 256 errors");
                    assert_eq!(observe(&boundary), cells(&boundary));
                    let MonomorphicValue::TaggedSum { payload, .. } = &mut boundary else {
                        panic!()
                    };
                    let MonomorphicValue::Sequence { elements, .. } = &mut payload[0] else {
                        panic!()
                    };
                    elements.push(elements[0].clone());
                    assert!(validate_monomorphic_value(
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                        &boundary
                    )
                    .is_err());
                    assert_eq!(observe(&boundary), 65537);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_domains_source_decimal_collection_order() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let mut cases = 0;
            for (id, row, facts) in domain_sources::sources() {
                let template = match id.as_str() {
                    "boundary-map-decimal" => "ordered_map",
                    "boundary-set-decimal" => "ordered_set",
                    _ => continue,
                };
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let program = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
                let types = program
                    .definitions()
                    .iter()
                    .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
                    .collect::<BTreeMap<_, _>>();
                let instance = emitted
                    .closure()
                    .closed()
                    .entries()
                    .iter()
                    .find(|e| e["template_id"] == format!("mpk.csharp.semantic.{template}.v1"))
                    .unwrap();
                let domain = program
                    .definitions()
                    .iter()
                    .find(|d| d.carrier.type_id == instance["instance_id"].as_str().unwrap())
                    .unwrap();
                let cert =
                    mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
                // Every pair has valid decimal representations. Ordering must
                // compare numeric values, including scale and signed zero.
                for (label, left, right, valid) in [
                    (
                        "equivalent scales",
                        (false, 2, "100"),
                        (false, 0, "1"),
                        false,
                    ),
                    (
                        "signed scaled zero",
                        (true, 28, "0"),
                        (false, 0, "0"),
                        false,
                    ),
                    ("descending", (false, 0, "2"), (false, 0, "1"), false),
                    ("increasing", (false, 2, "100"), (false, 0, "2"), true),
                ] {
                    let mut value = sample(
                        &domain.carrier.type_id,
                        2,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let keys: Vec<&mut MonomorphicValue> = match &mut value {
                        MonomorphicValue::OrderedMap { entries, .. } => {
                            entries.iter_mut().map(|e| e.key.as_mut()).collect()
                        }
                        MonomorphicValue::OrderedSet { elements, .. } => {
                            elements.iter_mut().collect()
                        }
                        _ => panic!(),
                    };
                    assert_eq!(keys.len(), 2);
                    for (key, (sign, exponent, digits)) in keys.into_iter().zip([left, right]) {
                        let MonomorphicValue::DecimalBits {
                            negative,
                            scale,
                            coefficient,
                            ..
                        } = key
                        else {
                            panic!()
                        };
                        *negative = sign;
                        *scale = exponent;
                        *coefficient = digits.into();
                    }
                    assert_eq!(
                        validate_monomorphic_value(
                            &bundle,
                            emitted.closure().roots(),
                            emitted.closure().closed(),
                            &value
                        )
                        .is_ok(),
                        valid,
                        "{id}: {label}"
                    );
                    eprintln!("decimal collection domain {id}: {label}");
                    assert_eq!(
                        observed_count(
                            &cert,
                            run(
                                &cert,
                                &domain.count_definition,
                                vec![input(storage(&value, &types))]
                            )
                        ),
                        if valid { cells(&value) } else { 65537 },
                        "{id}: {label}"
                    );
                    cases += 1;
                }
            }
            assert_eq!(cases, 8);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_domains_source_total_cell_boundary() {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(|| {
        let bundle=b();
        let (_,row,facts)=domain_sources::sources().into_iter().find(|(id,_,_)|id=="boundary-total-cells").unwrap();
        let (context,captures)=support::replay_context(&bundle,&row);
        let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();
        let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();
        let p=generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
        let types=p.definitions().iter().map(|d|(d.carrier.type_id.clone(),d.carrier.clone())).collect::<BTreeMap<_,_>>();
        let item_id=domain_sources::source_id("Item");
        let d=p.definitions().iter().find(|d|matches!(&d.carrier.shape,OrdinaryShape::Sequence { element,.. } if matches!(element.as_ref(),OrdinaryShape::Reference { type_id } if type_id==&item_id))).unwrap();
        let mut item=sample(&item_id,0,&types,&facts,emitted.closure().closed());
        let MonomorphicValue::Product { fields,.. }=&mut item else {panic!()};
        for field in fields {
            if let MonomorphicValue::Signed { value,.. }=field.value.as_mut() {*value="0".into();}
        }
        assert_eq!(cells(&item),17);
        let mut value=MonomorphicValue::Sequence {type_id:d.carrier.type_id.clone(),elements:vec![item;3855]};
        assert_eq!(cells(&value),65536);
        validate_monomorphic_value(&bundle,emitted.closure().roots(),emitted.closure().closed(),&value).unwrap();
        let cert=mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        eprintln!("source total-cell domain: 3855 items, exactly 65536 cells");
        assert_eq!(observed_count(&cert,run(&cert,&d.count_definition,vec![input(storage(&value,&types))])),65536);
        let MonomorphicValue::Sequence { elements,.. }=&mut value else {panic!()};
        let MonomorphicValue::Product {fields,..}=&mut elements[0] else {panic!()};
        let flag=fields.iter_mut().find(|f|f.name=="Flag").unwrap();
        let MonomorphicValue::Option {arm,value,..}=flag.value.as_mut() else {panic!()};
        *arm=OptionArm::Some;
        *value=Some(Box::new(MonomorphicValue::Bool {type_id:ty("bool"),value:false}));
        // The same physical type/length adds exactly one logical payload cell.
        let value=MonomorphicValue::Sequence {type_id:d.carrier.type_id.clone(),elements:elements.clone()};
        assert_eq!(cells(&value),65537);
        assert!(validate_monomorphic_value(&bundle,emitted.closure().roots(),emitted.closure().closed(),&value).is_err());
        eprintln!("source total-cell domain: same layout, exactly 65537 cells");
        assert_eq!(observed_count(&cert,run(&cert,&d.count_definition,vec![input(storage(&value,&types))])),65537);
    }).unwrap().join().unwrap();
}

#[test]
fn csharp_03_t06_w09_domains_source_sum_and_unused_storage_rejections() {
    let bundle = b();
    let mut high_tags = 0;
    let mut empty_arms = 0;
    let mut unused_roles = 0;
    for (id, row, facts) in relation_tests::sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
        let types = p
            .definitions()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for d in p.definitions().iter().filter(|d| d.carrier.depth <= 10) {
            let value = sample(
                &d.carrier.type_id,
                0,
                &types,
                &facts,
                emitted.closure().closed(),
            );
            let bits = storage(&value, &types);
            let reject = |bits| {
                assert_eq!(
                    observed_count(&cert, run(&cert, &d.count_definition, vec![input(bits)])),
                    65537,
                    "{id} {}",
                    d.carrier.type_id
                )
            };
            match &d.carrier.shape {
                OrdinaryShape::Sum { arms } => {
                    for tag in [0x8000_0000u32, u32::MAX] {
                        assert!(arms.iter().all(|a| a.tag != tag));
                        let mut changed = bits.clone();
                        for i in 0..32 {
                            changed[i << (d.carrier.depth - 5)] = tag & (1 << i) != 0;
                        }
                        reject(changed);
                        high_tags += 1;
                    }
                    if let Some(empty) = arms.iter().find(|a| a.fields.is_empty()) {
                        let mut changed = vec![false; bits.len()];
                        for i in 0..32 {
                            changed[i << (d.carrier.depth - 5)] = empty.tag & (1 << i) != 0;
                        }
                        *changed.last_mut().unwrap() = true;
                        reject(changed);
                        empty_arms += 1;
                    }
                }
                OrdinaryShape::Product { fields }
                    if fields.len() < fields.len().max(1).next_power_of_two() =>
                {
                    let mut changed = bits.clone();
                    changed[fields.len()] = true;
                    reject(changed);
                    unused_roles += 1;
                }
                _ => (),
            }
        }
    }
    assert!(
        high_tags >= 10 && empty_arms >= 5 && unused_roles >= 1,
        "missing rejection family: {high_tags}/{empty_arms}/{unused_roles}"
    );
    eprintln!("source-domain rejections: {high_tags} high tags, {empty_arms} empty-arm payloads, {unused_roles} unused product roles");
}

#[test]
fn csharp_03_t06_w09_domains_user_exception_payload_storage() {
    let bundle = b();
    let mut positives = 0;
    let mut rejections = 0;
    for (id, row, facts) in domain_sources::exception_sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
        let types = program
            .definitions()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        let domain = program
            .definitions()
            .iter()
            .find(|d| d.carrier.type_id == ty("exception"))
            .unwrap();
        let value = sample(
            &domain.carrier.type_id,
            3,
            &types,
            &facts,
            emitted.closure().closed(),
        );
        let MonomorphicValue::ClosedException {
            tag,
            payload: Some(payload),
            ..
        } = &value
        else {
            panic!("missing source payload: {id}")
        };
        assert!(*tag >= 9);
        let MonomorphicValue::Product { fields, .. } = payload.as_ref() else {
            panic!()
        };
        let empty = id == "boundary-exception-empty";
        assert_eq!(fields.len(), if empty { 0 } else { 3 });
        validate_monomorphic_value(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &value,
        )
        .unwrap();
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        let bits = storage(&value, &types);
        let observe = |bits| {
            observed_count(
                &cert,
                run(&cert, &domain.count_definition, vec![input(bits)]),
            )
        };
        let expected = if empty { 2 } else { 5 };
        assert_eq!(cells(&value), expected);
        assert_eq!(observe(bits.clone()), expected, "{id}");
        positives += 1;
        let mutations = if empty {
            // The source payload is an empty product: both its storage Bool
            // and its surrounding sum padding must remain zero.
            vec![1, bits.len() - 1]
        } else {
            // C8: role bit, two product selectors, then five field selectors.
            // Role 3 is unused; Bool and char have leading field padding.
            assert_eq!(domain.carrier.depth, 8);
            vec![
                2,
                1 | (3 << 1),
                1 | (1 << 1) | (1 << 3),
                1 | (2 << 1) | (1 << 3),
            ]
        };
        for address in mutations {
            let mut changed = bits.clone();
            assert!(!changed[address], "{id} padding {address}");
            changed[address] = true;
            assert_eq!(observe(changed), 65537, "{id} padding {address}");
            rejections += 1;
        }
        let mut builtin = bits.clone();
        for i in 0..32 {
            builtin[i << (domain.carrier.depth - 5)] = false;
        }
        // A built-in arm has no payload, even in a universe with source fields.
        builtin[1] = true;
        assert_eq!(observe(builtin), 65537, "{id} builtin with user payload");
        rejections += 1;
    }
    assert_eq!(positives, 2);
    assert_eq!(rejections, 8);
    eprintln!("source-exception domains: {positives} source payload counts and {rejections} storage rejections");
}

#[test]
fn csharp_03_t06_w09_domains_source_full_utf16_capacity() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let (_, row, facts) = relation_tests::sources()
                .into_iter()
                .find(|(id, _, _)| id == "string")
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
            let p = generate_csharp_practical_ordinary_domains(emitted.vir()).unwrap();
            let types = p
                .definitions()
                .iter()
                .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
                .collect::<BTreeMap<_, _>>();
            let d = p
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == ty("string"))
                .unwrap();
            let mut utf16 = vec![0; 16384];
            utf16[0] = 0xd800;
            utf16[16383] = 0xffff;
            let value = MonomorphicValue::String {
                type_id: ty("string"),
                utf16,
            };
            validate_monomorphic_value(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &value,
            )
            .unwrap();
            let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            let encoded = storage(&value, &types);
            eprintln!("source UTF-16 domain: all 16384 code units, including isolated surrogate");
            assert_eq!(
                observed_count(
                    &cert,
                    run(&cert, &d.count_definition, vec![input(encoded.clone())])
                ),
                16385
            );
            let mut excessive = encoded.clone();
            for i in 0..32 {
                excessive[i << (d.carrier.depth - 5)] = 16385u32 & (1 << i) != 0;
            }
            assert_eq!(
                observed_count(
                    &cert,
                    run(&cert, &d.count_definition, vec![input(excessive)])
                ),
                65537
            );
            let mut padding = encoded;
            padding[2] = true;
            assert_eq!(
                observed_count(&cert, run(&cert, &d.count_definition, vec![input(padding)])),
                65537
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
