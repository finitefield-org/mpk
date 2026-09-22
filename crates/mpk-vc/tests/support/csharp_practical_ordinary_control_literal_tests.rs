//! Original VIR linkage and independently encoded full literal storage.
use super::*;

#[test]
fn csharp_03_t06_w09_control_native_literals_original_sources() {
    run_selected_cases_mode(
        &[
            "count_fill",
            "while",
            "for",
            "short_circuit",
            "switch",
            "is_binding",
            "guard_order",
            "guard_throw",
            "total_variable",
            "index_update",
            "foreach_string",
            "foreach_string_var",
            "foreach_array",
            "foreach_array_var",
            "lookup",
            "governing_throw",
            "type",
            "string_property",
        ],
        false,
        false,
        false,
        false,
        false,
        NativeRuntime::Literals,
    );
}

#[test]
fn csharp_03_t06_w09_control_native_literals_preserve_existing_declarations() {
    preserves_existing_declarations("before-native-literals", false);
}

pub(super) fn run_literals(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    cert: &mpk_cert::encode::Certificate,
) -> usize {
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let types: BTreeMap<_, _> = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect();
    let mut observations = 0;
    let mut literals = 0;
    for flow in p.functions() {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .unwrap();
        let expected = native
            .blocks
            .iter()
            .flat_map(|b| b.literal_values.iter().map(move |v| (&b.node.id, v)))
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), flow.native_literals.len());
        for ((node, source), literal) in expected.into_iter().zip(&flow.native_literals) {
            assert_eq!(&literal.source, source);
            assert_eq!(&literal.result.node_id, node);
            assert_eq!(literal.result.kind, "literal_result");
            assert!(literal.result.edge_id.is_none());
            assert_eq!(literal.result.value_id, source.result.id);
            assert_eq!(literal.result.type_id, source.result.type_id);
            let depth = types[&source.result.type_id].depth;
            let storage = relation_tests::storage(&source.value, &types);
            assert_eq!(storage.len(), 1 << depth);
            let ones = storage
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.then_some(i))
                .collect::<BTreeSet<_>>();
            let value = run(cert, &literal.literal_definition, vec![]);
            let mut indices = ones.clone();
            if storage.len() <= 256 {
                indices.extend(0..storage.len());
            }
            indices.extend([0, storage.len() / 2, storage.len() - 1]);
            for index in indices {
                let mut leaf = value.clone();
                for selector in 0..depth {
                    leaf = apply(cert, leaf, V::Bit(index & (1 << selector) != 0));
                }
                assert_eq!(bit(leaf), storage[index]);
                observations += 1;
            }
            // Equality executes over the complete cube, not just the sampled
            // observations above. Include high padding and inactive payloads.
            for corrupt in [
                None,
                Some(0),
                Some(storage.len() / 2),
                Some(storage.len() - 1),
            ] {
                let mut bits = ones.clone();
                if let Some(index) = corrupt {
                    if !bits.remove(&index) {
                        bits.insert(index);
                    }
                }
                assert_eq!(
                    bit(run(
                        cert,
                        &literal.definition,
                        vec![sparse_cube(depth, bits)]
                    )),
                    corrupt.is_none(),
                    "{} {corrupt:?}",
                    source.result.id
                );
                observations += 1;
            }
            literals += 1;
        }
    }
    assert!(literals > 0);
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    let function = original["functions"]
        .as_array()
        .unwrap()
        .iter()
        .position(|f| {
            f["native_literals"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        })
        .unwrap();
    for field in ["source", "result", "literal_definition", "definition"] {
        let mut changed = original.clone();
        changed["functions"][function]["native_literals"][0][field] = json!("wrong-literal");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    for field in ["kind", "edge_id", "node_id", "value_id", "type_id"] {
        let mut changed = original.clone();
        changed["functions"][function]["native_literals"][0]["result"][field] =
            json!("wrong-binding");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let mut omitted = original.clone();
    omitted["functions"][function]
        .as_object_mut()
        .unwrap()
        .remove("native_literals");
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&omitted).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let mut extra = original;
    let record = extra["functions"][function]["native_literals"][0].clone();
    extra["functions"][function]["native_literals"]
        .as_array_mut()
        .unwrap()
        .push(record);
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&extra).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    eprintln!("checked {literals} exact source literal bindings");
    observations
}
