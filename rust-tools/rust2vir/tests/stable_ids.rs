use rust2vir_internal::json::JsonValue;
use rust2vir_internal::source_map::{raw_source_map, SourceMapEntry, SourceOrigin, VirReference};
use rust2vir_internal::stable_id::{block_names, breadth_first_order, DenseIds, StableIdError};

#[test]
fn breadth_first_ids_follow_false_before_true_and_skip_unreachable_blocks() {
    let graph = [vec![2, 1], vec![3], vec![3], Vec::new(), vec![0]];
    let order = breadth_first_order(0, graph.len(), |block| graph[block].clone()).unwrap();
    assert_eq!(order, vec![0, 2, 1, 3]);
    assert_eq!(
        block_names(&order).into_iter().collect::<Vec<_>>(),
        vec![
            (0, "bb0".to_owned()),
            (1, "bb2".to_owned()),
            (2, "bb1".to_owned()),
            (3, "bb3".to_owned()),
        ]
    );
}

#[test]
fn dense_value_and_block_parameter_names_have_separate_sequences() {
    let mut ids = DenseIds::default();
    assert_eq!(ids.temporary(), "t0");
    assert_eq!(ids.block_parameter(), "p0");
    assert_eq!(ids.block_parameter(), "p1");
    assert_eq!(ids.temporary(), "t1");
}

#[test]
fn traversal_rejects_missing_entry_and_unknown_successors() {
    assert_eq!(
        breadth_first_order(0, 0, |_| Vec::new()),
        Err(StableIdError::EmptyGraph)
    );
    assert_eq!(
        breadth_first_order(0, 1, |_| vec![1]),
        Err(StableIdError::UnknownSuccessor)
    );
}

#[test]
fn raw_source_map_orders_function_then_all_instructions_then_terminators() {
    let origin = SourceOrigin {
        normalized_path: "src/lib.rs".to_owned(),
        start: 1,
        end: 2,
    };
    let entries = vec![
        SourceMapEntry {
            reference: VirReference::Terminator {
                unit_id: "vector".to_owned(),
                function_id: "vector::f".to_owned(),
                block_index: 0,
            },
            origin: origin.clone(),
        },
        SourceMapEntry {
            reference: VirReference::Instruction {
                unit_id: "vector".to_owned(),
                function_id: "vector::f".to_owned(),
                block_index: 1,
                instruction_index: 1,
            },
            origin: origin.clone(),
        },
        SourceMapEntry {
            reference: VirReference::Function {
                unit_id: "vector".to_owned(),
                function_id: "vector::f".to_owned(),
            },
            origin: origin.clone(),
        },
        SourceMapEntry {
            reference: VirReference::Instruction {
                unit_id: "vector".to_owned(),
                function_id: "vector::f".to_owned(),
                block_index: 0,
                instruction_index: 0,
            },
            origin,
        },
    ];
    let map = raw_source_map(&"a".repeat(64), entries);
    let references = map.as_object().expect("map")["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|entry| {
            let reference = &entry.as_object().expect("entry")["reference"];
            let reference = reference.as_object().expect("reference");
            (
                reference["kind"].as_str().expect("kind"),
                reference
                    .get("block")
                    .and_then(JsonValue::as_str)
                    .unwrap_or(""),
                reference
                    .get("instruction")
                    .and_then(JsonValue::as_str)
                    .unwrap_or(""),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![
            ("function", "", ""),
            ("instruction", "bb0", "t0"),
            ("instruction", "bb1", "t1"),
            ("terminator", "bb0", ""),
        ]
    );
}
