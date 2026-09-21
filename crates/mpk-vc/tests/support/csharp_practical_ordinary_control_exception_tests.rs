//! Independent ordered-failure oracles and exact exception storage/edge linkage.
use super::*;

#[test]
fn csharp_03_t06_w09_control_native_exceptions_original_sources() {
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
        NativeRuntime::Exceptions,
    );
}

#[test]
fn csharp_03_t06_w09_control_native_exceptions_preserve_existing_declarations() {
    preserves_existing_declarations("before-native-exceptions", false);
}

pub(super) fn run_exceptions(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    cert: &mpk_cert::encode::Certificate,
    depths: &BTreeMap<&str, u32>,
    exception_vcs: &ExceptionVcProgram,
) -> usize {
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let types = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect();
    let mut observations = 0;
    let mut enabled = 0;
    let mut disabled = 0;
    for f in p.functions() {
        let expected = f
            .native_operations
            .iter()
            .flat_map(|o| {
                o.source
                    .checks
                    .iter()
                    .filter(|c| c.check.tag == RequiredCheckTag::Exception)
                    .map(move |c| (o, c))
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), f.native_exceptions.len());
        let native = vir
            .functions()
            .iter()
            .find(|n| n.id == f.source.function_id)
            .unwrap();
        for ((operation, check), e) in expected.into_iter().zip(&f.native_exceptions) {
            assert_eq!(e.operation_id, operation.source.id);
            assert_eq!(&e.source, check);
            assert_eq!(e.ownership, operation.ownership);
            assert!(e.pending_constant_names.is_empty());
            let edge = f.edges.iter().find(|x| x.source.id == e.edge_id).unwrap();
            assert_eq!(edge.source.check_id.as_ref(), Some(&check.check.id));
            assert_eq!(edge.source.source_node_id, operation.source.node_id);
            let successor = check.exceptional_successor.as_ref().unwrap();
            assert_eq!(
                edge.source.target_node_id.as_ref(),
                Some(&successor.target_id)
            );
            let count = operation.invocation.operands.len();
            assert_eq!(e.arguments.len(), count + 1);
            assert_eq!(e.arguments[..count], operation.arguments[..count]);
            let actual = &e.arguments[count];
            assert_eq!(actual.kind, "exception_value");
            assert_eq!(actual.node_id, successor.target_id);
            assert_eq!(actual.edge_id.as_ref(), Some(&edge.source.id));
            let original_edge = exception_vcs
                .functions()
                .iter()
                .find(|v| v.function_id == f.source.function_id)
                .unwrap()
                .edges
                .iter()
                .find(|v| v.edge.id == e.edge_id)
                .unwrap();
            let original_exception = original_edge.exception_value.as_ref().unwrap();
            assert_eq!(actual.value_id, original_exception.id);
            assert_eq!(actual.type_id, original_exception.type_id);
            assert_eq!(Some(&actual.node_id), original_edge.post_node_id.as_ref());
            assert_eq!(original_edge.frozen_exception_value, check.exception_value);
            let original_value = &native
                .blocks
                .iter()
                .find(|b| b.node.id == operation.source.node_id)
                .unwrap()
                .exception_values
                .iter()
                .find(|v| v.check_id == check.check.id)
                .unwrap()
                .value;
            assert_eq!(check.exception_value.as_ref(), Some(original_value));
            assert_eq!(actual.type_id, original_value.type_id());
            let storage = relation_tests::storage(original_value, &types);
            let ones = storage
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| v.then_some(i))
                .collect::<BTreeSet<_>>();
            let depth = depths[actual.type_id.as_str()];
            let literal = run(cert, &e.literal_definition, vec![]);
            for (index, expected) in storage.iter().enumerate() {
                let mut leaf = literal.clone();
                for selector in 0..depth {
                    leaf = apply(cert, leaf, V::Bit(index & (1 << selector) != 0));
                }
                assert_eq!(bit(leaf), *expected);
                observations += 1;
            }
            // Each invocation is checked with independent operands. None + an
            // invalid index exercises competing null/range failures; integer
            // boundaries exercise division-by-zero and overflow independently.
            let mut check_enabled = 0;
            let mut check_disabled = 0;
            for sample in [
                [0, 0, 0],
                [3, 1, 13],
                [3, -1, 13],
                [3, 3, 13],
                [i32::MIN, -1, 0],
                [i32::MIN, 1, 0],
                [i32::MAX, -1, 0],
                [i32::MIN, 0, 0],
                [i32::MAX, 1, 0],
                [-1, 1, 0],
            ] {
                let args = e.arguments[..count]
                    .iter()
                    .enumerate()
                    .map(|(i, a)| guard_input(p, &a.type_id, depths[a.type_id.as_str()], sample[i]))
                    .collect::<Vec<_>>();
                let expected = reference(&check.failure_guard, &sample, p, edge);
                assert_eq!(
                    bit(run(
                        cert,
                        e.guard_definition.as_ref().unwrap(),
                        args.clone()
                    )),
                    expected
                );
                observations += 1;
                enabled += usize::from(expected);
                disabled += usize::from(!expected);
                check_enabled += usize::from(expected);
                check_disabled += usize::from(!expected);
                // Exact exception, wrong tag, and a changed last physical bit
                // (including inactive payload/padding) are distinguished.
                for corrupt in [None, Some(0), Some(storage.len() - 1)] {
                    let mut bits = ones.clone();
                    if let Some(index) = corrupt {
                        if !bits.remove(&index) {
                            bits.insert(index);
                        }
                    }
                    let mut values = args.clone();
                    values.push(sparse_cube(depth, bits));
                    assert_eq!(
                        bit(run(cert, e.definition.as_ref().unwrap(), values)),
                        expected && corrupt.is_none(),
                        "{} {} {sample:?} {corrupt:?}",
                        operation.invocation.operation_id,
                        check.check.id
                    );
                    observations += 1;
                }
            }
            assert!(
                check_enabled > 0 && check_disabled > 0,
                "unexercised outcome: {} {}",
                operation.invocation.operation_id,
                check.check.id
            );
        }
    }
    assert!(enabled > 0 && disabled > 0);
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    let function = original["functions"]
        .as_array()
        .unwrap()
        .iter()
        .position(|f| {
            f["native_exceptions"]
                .as_array()
                .is_some_and(|v| !v.is_empty())
        })
        .unwrap();
    let fields = original["functions"][function]["native_exceptions"][0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for field in fields {
        let mut changed = original.clone();
        changed["functions"][function]["native_exceptions"][0][&field] = json!("wrong-exception");
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
        .remove("native_exceptions");
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&omitted).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let mut extra = original;
    let exception = extra["functions"][function]["native_exceptions"][0].clone();
    extra["functions"][function]["native_exceptions"]
        .as_array_mut()
        .unwrap()
        .push(exception);
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&extra).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    eprintln!("native exception enabled={enabled} disabled={disabled}");
    observations
}
