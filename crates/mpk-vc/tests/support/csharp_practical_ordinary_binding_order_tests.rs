use super::*;
const ORDER: &str = "Mpk.CSharp.Binding.CanonicalOrder.";

#[test]
fn csharp_03_t06_w09_binding_orders_original_source_certificates() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BINDING_ORDERS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-orders");
    let guard_fixture = std::env::var_os("MPK_W09_BINDING_GUARDS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| fixture.parent().unwrap().join("binding-guards"));
    let mut rows = vec![];
    let mut occurrences = 0;
    let mut unchanged = 0;
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
        let p = generate_csharp_practical_ordinary_binding_orders(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let base = generate_csharp_practical_ordinary_binding_guards(vir).unwrap();
        let pinned: Value =
            serde_json::from_slice(&fs::read(guard_fixture.join("certificates.json")).unwrap())
                .unwrap();
        let previous_guard = pinned["sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(
            previous_guard["metadata"],
            serde_json::from_slice::<Value>(&base.canonical_bytes()).unwrap()
        );
        let guard_hex = base
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        assert_eq!(
            fs::read_to_string(guard_fixture.join(format!("{id}.hex"))).unwrap(),
            guard_hex
        );

        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let wanted = vc
            .binding_vcs()
            .definition_names()
            .iter()
            .filter(|s| s.starts_with(ORDER))
            .collect::<BTreeSet<_>>();
        assert_eq!(p.projections(), base.projections());
        assert_eq!(p.agreements(), base.agreements());
        assert!(base.predicates().iter().all(|d| p.predicates().contains(d)));
        assert_eq!(
            p.predicates()
                .iter()
                .filter(|d| !base.predicates().contains(d))
                .map(|d| &d.symbol)
                .collect::<BTreeSet<_>>(),
            wanted
        );
        assert_eq!(
            p.unresolved_vc_symbols(),
            base.unresolved_vc_symbols()
                .iter()
                .filter(|s| !wanted.contains(s))
                .cloned()
                .collect::<Vec<_>>()
        );
        if wanted.is_empty() {
            assert_eq!(p.certificate_bytes(), base.certificate_bytes());
            unchanged += 1;
            continue;
        }
        occurrences += wanted.len();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let old = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
        let roots = base
            .predicates()
            .iter()
            .map(|d| d.definition.clone())
            .chain(
                base.projections()
                    .iter()
                    .map(|d| d.project_definition.clone()),
            )
            .chain(
                base.agreements()
                    .iter()
                    .map(|d| d.result_agreement_definition.clone()),
            )
            .collect();
        structural_equivalence_tests::same_definition_closure(&old, &cert, &roots).unwrap();
        for d in p
            .predicates()
            .iter()
            .filter(|d| d.symbol.starts_with(ORDER))
        {
            assert_eq!(d.argument_type_ids, [d.symbol.strip_prefix(ORDER).unwrap()]);
            assert_eq!(d.result_type_id, ty("bool"));
        }
        assert_eq!(
            import_csharp_practical_ordinary_binding_orders(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        assert!(import_csharp_practical_ordinary_binding_guards(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "binding_vc_sha256",
            "predicates",
            "unresolved_vc_symbols",
        ] {
            let mut changed = meta.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_binding_orders(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_binding_orders(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_orders(m, c, vir).is_err());
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
        eprintln!(
            "binding order {id}: {} predicates, {} terms",
            wanted.len(),
            cert.term_table.len()
        );
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!((rows.len(), occurrences, unchanged), (9, 10, 36));
    let data = json!({"sources":rows,"order_predicate_occurrences":occurrences,"unchanged_nonordering_contexts":unchanged});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-orders/certificates.json"),
            data
        );
    }
}
fn collection(keys: &[MonomorphicValue], sample: &MonomorphicValue) -> MonomorphicValue {
    match sample {
        MonomorphicValue::OrderedMap { type_id, entries } => MonomorphicValue::OrderedMap {
            type_id: type_id.clone(),
            entries: keys
                .iter()
                .map(|k| MonomorphicMapEntry {
                    key: Box::new(k.clone()),
                    value: entries[0].value.clone(),
                })
                .collect(),
        },
        MonomorphicValue::OrderedSet { type_id, .. } => MonomorphicValue::OrderedSet {
            type_id: type_id.clone(),
            elements: keys.to_vec(),
        },
        _ => panic!(),
    }
}
fn first_key(value: &MonomorphicValue) -> &MonomorphicValue {
    match value {
        MonomorphicValue::OrderedMap { entries, .. } => &entries[0].key,
        MonomorphicValue::OrderedSet { elements, .. } => &elements[0],
        _ => panic!(),
    }
}
#[test]
fn csharp_03_t06_w09_binding_orders_source_semantics() {
    let bundle = b();
    let mut count = 0;
    let mut instances = 0;
    let mut key_types = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_binding_orders(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for pred in p
            .predicates()
            .iter()
            .filter(|d| d.symbol.starts_with(ORDER))
        {
            let t = &pred.argument_type_ids[0];
            let value = relation_tests::sample(t, 2, &types, &facts, emitted.closure().closed());
            let key = first_key(&value);
            let key_id = key.type_id();
            key_types.insert(key_id.to_owned());
            let oracle = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                key_id,
            )
            .unwrap();
            let mut keys = (0..4)
                .map(|seed| {
                    relation_tests::sample(key_id, seed, &types, &facts, emitted.closure().closed())
                })
                .collect::<Vec<_>>();
            keys.sort_by(|a, b| oracle.canonical_compare(a, b).unwrap());
            keys.dedup_by(|a, b| {
                oracle.canonical_compare(a, b).unwrap() == std::cmp::Ordering::Equal
            });
            assert!(keys.len() >= 2);
            for (label, keys) in [
                ("empty", vec![]),
                ("singleton", vec![keys[0].clone()]),
                ("ascending", keys.clone()),
                ("descending", keys.iter().rev().cloned().collect()),
                ("duplicate", vec![keys[0].clone(), keys[0].clone()]),
            ] {
                let expected = keys.windows(2).all(|w| {
                    oracle.canonical_compare(&w[0], &w[1]).unwrap() == std::cmp::Ordering::Less
                });
                let value = collection(&keys, &value);
                let (depth, bits) = sparse_storage(&value, &types);
                eprintln!("binding order {id} {label}: expected={expected}");
                assert_eq!(
                    bit(run(
                        &cert,
                        &pred.definition,
                        vec![sparse_cube(depth, bits.clone())]
                    )),
                    expected,
                    "{id}: {label}"
                );
                count += 1;
                // Setting storage in unused index 4095 must not become a key.
                let mut inactive = bits;
                inactive.insert((1usize << depth) - 1);
                assert_eq!(
                    bit(run(
                        &cert,
                        &pred.definition,
                        vec![sparse_cube(depth, inactive)]
                    )),
                    expected,
                    "{id}: inactive storage {label}"
                );
                count += 1;
            }
            if let MonomorphicValue::OrderedMap { entries, .. } = &value {
                let value_id = entries[0].value.type_id();
                for (label, chosen, expected) in [
                    (
                        "duplicate keys with different values",
                        vec![keys[0].clone(), keys[0].clone()],
                        false,
                    ),
                    (
                        "ascending keys with independently changed values",
                        vec![keys[0].clone(), keys[1].clone()],
                        true,
                    ),
                ] {
                    let mut changed = collection(&chosen, &value);
                    let MonomorphicValue::OrderedMap { entries, .. } = &mut changed else {
                        panic!()
                    };
                    for (entry, seed) in entries.iter_mut().zip([1, 0]) {
                        *entry.value = relation_tests::sample(
                            value_id,
                            seed,
                            &types,
                            &facts,
                            emitted.closure().closed(),
                        );
                    }
                    let (depth, bits) = sparse_storage(&changed, &types);
                    eprintln!("binding order {id}: {label}");
                    assert_eq!(
                        bit(run(&cert, &pred.definition, vec![sparse_cube(depth, bits)])),
                        expected,
                        "{id}: {label}"
                    );
                    count += 1;
                }
            }
            for len in [4097u32, u32::MAX] {
                let depth = types[t].depth;
                let bits = (0..32)
                    .filter(|i| len & (1u32 << i) != 0)
                    .map(|i| i << (depth - 5))
                    .collect();
                assert!(
                    !bit(run(&cert, &pred.definition, vec![sparse_cube(depth, bits)])),
                    "{id}: overlength {len}"
                );
                count += 1;
            }
            if key_id == ty("decimal") {
                for (label, left, right) in [
                    ("scale cohorts", (false, 2, "100"), (false, 0, "1")),
                    ("signed zero", (true, 28, "0"), (false, 0, "0")),
                ] {
                    let keys = [left, right].map(|(negative, scale, coefficient)| {
                        MonomorphicValue::DecimalBits {
                            type_id: key_id.into(),
                            negative,
                            scale,
                            coefficient: coefficient.into(),
                        }
                    });
                    assert_eq!(
                        oracle.canonical_compare(&keys[0], &keys[1]).unwrap(),
                        std::cmp::Ordering::Equal
                    );
                    let (depth, bits) = sparse_storage(&collection(&keys, &value), &types);
                    eprintln!("binding order {id}: {label}");
                    assert!(!bit(run(
                        &cert,
                        &pred.definition,
                        vec![sparse_cube(depth, bits)]
                    )));
                    count += 1;
                }
            }
            instances += 1;
        }
    }
    assert_eq!(instances, 10);
    assert!(
        key_types.contains(&ty("decimal"))
            && key_types.contains(&ty("string"))
            && key_types.contains(&ty("bool"))
            && key_types.contains(&ty("i32"))
    );
    assert_eq!(count, 140);
    eprintln!("binding order semantic observations: {count}");
}

#[test]
fn csharp_03_t06_w09_binding_orders_full_capacity() {
    let bundle = b();
    let mut count = 0;
    for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
        matches!(
            id.as_str(),
            "binding-vc-ordered_map" | "binding-vc-ordered_set"
        )
    }) {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_binding_orders(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let pred = p
            .predicates()
            .iter()
            .find(|d| d.symbol.starts_with(ORDER))
            .unwrap();
        let value = relation_tests::sample(
            &pred.argument_type_ids[0],
            2,
            &types,
            &facts,
            emitted.closure().closed(),
        );
        assert_eq!(first_key(&value).type_id(), ty("i32"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let original = (0..4096)
            .map(|i| MonomorphicValue::Signed {
                type_id: ty("i32"),
                value: i.to_string(),
            })
            .collect::<Vec<_>>();
        for (label, change) in [
            ("full ascending", None),
            ("duplicate across high index bit", Some(2048usize)),
            ("duplicate last key", Some(4095usize)),
        ] {
            let mut keys = original.clone();
            if let Some(index) = change {
                keys[index] = keys[index - 1].clone();
            }
            let value = collection(&keys, &value);
            let (depth, bits) = sparse_storage(&value, &types);
            eprintln!("binding order full capacity {id}: {label}");
            assert_eq!(
                bit(run(&cert, &pred.definition, vec![sparse_cube(depth, bits)])),
                change.is_none(),
                "{id}: {label}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 6);
    eprintln!("binding order full-capacity observations: {count}");
}

#[test]
fn csharp_03_t06_w09_binding_orders_string_compound_edges() {
    let bundle = b();
    let mut count = 0;
    for (id, row, facts) in sources()
        .into_iter()
        .filter(|(id, _, _)| matches!(id.as_str(), "boundary-map-string" | "boundary-map-compound"))
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
        let p = generate_csharp_practical_ordinary_binding_orders(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let pred = p
            .predicates()
            .iter()
            .find(|d| d.symbol.starts_with(ORDER))
            .unwrap();
        let sample = relation_tests::sample(
            &pred.argument_type_ids[0],
            2,
            &types,
            &facts,
            emitted.closure().closed(),
        );
        let original = first_key(&sample);
        let compound = id == "boundary-map-compound";
        let key = |units: Vec<u16>, number: i32| {
            let mut k = original.clone();
            if compound {
                let MonomorphicValue::Product { fields, .. } = &mut k else {
                    panic!()
                };
                let MonomorphicValue::String { utf16, .. } = fields[0].value.as_mut() else {
                    panic!()
                };
                *utf16 = units;
                let MonomorphicValue::Signed { value, .. } = fields[1].value.as_mut() else {
                    panic!()
                };
                *value = number.to_string();
            } else {
                let MonomorphicValue::String { utf16, .. } = &mut k else {
                    panic!()
                };
                *utf16 = units;
            }
            k
        };
        let low = key(vec![0xd800], 0);
        let high = if compound {
            key(vec![0xd800], 1)
        } else {
            key(vec![0xd800, 0], 0)
        };
        let priority = if compound {
            // First field wins even though the second fields increase.
            (key(vec![0xd800, 1], 0), key(vec![0xd800], 1), false)
        } else {
            // Ordinal UTF-16 includes the surrogate boundary as raw code units.
            (key(vec![0xd7ff], 0), key(vec![0xd800], 0), true)
        };
        let oracle = generate_structural_program(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            original.type_id(),
        )
        .unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        for (label, left, right, expected) in [
            ("ascending equal prefix", low.clone(), high.clone(), true),
            ("descending equal prefix", high, low.clone(), false),
            ("equal complete key", low.clone(), low, false),
            ("field/UTF16 priority", priority.0, priority.1, priority.2),
        ] {
            assert_eq!(
                oracle.canonical_compare(&left, &right).unwrap() == std::cmp::Ordering::Less,
                expected
            );
            let (depth, bits) = sparse_storage(&collection(&[left, right], &sample), &types);
            eprintln!("binding order edge {id}: {label}");
            assert_eq!(
                bit(run(&cert, &pred.definition, vec![sparse_cube(depth, bits)])),
                expected,
                "{id}: {label}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 8);
    eprintln!("binding order string/compound observations: {count}");
}
