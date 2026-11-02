//! Source-step composition, exact shared SSA origins and independent mutations.
use super::*;

#[test]
fn csharp_03_t06_w09_control_source_steps_original_sources() {
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
        NativeRuntime::Steps,
    );
}
#[test]
fn csharp_03_t06_w09_control_source_steps_preserve_existing_declarations() {
    preserves_existing_declarations("before-source-steps", false);
}

fn sparse(value: &BTreeSet<usize>, depth: u32) -> V {
    sparse_cube(depth, value.clone())
}
fn word(bits: &BTreeSet<usize>) -> i32 {
    bits.iter()
        .filter(|&&i| i < 32)
        .fold(0u32, |v, &i| v | (1u32 << i)) as i32
}
fn bits(value: i32) -> BTreeSet<usize> {
    (0..32).filter(|i| value as u32 & (1 << i) != 0).collect()
}
pub(super) fn run_steps(
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
    let mut positive = 0;
    let mut negative = 0;
    let mut shared = 0;
    let mut complete = 0;
    let mut pending = BTreeMap::<String, usize>::new();
    for f in p.functions() {
        let native = vir
            .functions()
            .iter()
            .find(|n| n.id == f.source.function_id)
            .unwrap();
        let graph = f.source.source_graph.as_ref().unwrap();
        let protocol = native.control_protocol.as_ref().unwrap();
        assert_eq!(f.source_steps.len(), graph.nodes.len());
        for (source, step) in graph.nodes.iter().zip(&f.source_steps) {
            assert_eq!(source.id, step.source_node_id);
            if !step.pending_reasons.is_empty() {
                assert!(step.definition.is_none());
                for reason in &step.pending_reasons {
                    *pending.entry(reason.clone()).or_default() += 1;
                }
                continue;
            }
            complete += 1;
            let anchor = protocol
                .anchors
                .iter()
                .find(|a| a.source_node_id == source.id)
                .unwrap();
            assert_eq!(step.native_node_ids.first(), Some(&anchor.entry_node_id));
            assert_eq!(step.native_node_ids.last(), Some(&anchor.exit_node_id));
            assert_eq!(
                step.native_node_ids.iter().collect::<BTreeSet<_>>(),
                anchor.artifact_node_ids.iter().collect()
            );
            let blocks = step
                .native_node_ids
                .iter()
                .map(|id| native.blocks.iter().find(|b| &b.node.id == id).unwrap())
                .collect::<Vec<_>>();
            for pair in blocks.windows(2) {
                assert_eq!(
                    pair[0].node.normal_successor_ids,
                    vec![pair[1].node.id.clone()]
                );
            }
            let frame = f
                .source_frames
                .iter()
                .find(|x| x.source_node_id == source.id)
                .unwrap();
            let transfer = f.slot_relations.iter().find(|r| {
                r.transfer
                    .as_ref()
                    .is_some_and(|t| t.source_node_id == source.id)
            });
            let ops = f
                .native_operations
                .iter()
                .filter(|o| step.native_node_ids.contains(&o.source.node_id))
                .collect::<Vec<_>>();
            let literals = f
                .native_literals
                .iter()
                .filter(|l| step.native_node_ids.contains(&l.result.node_id))
                .collect::<Vec<_>>();
            assert_eq!(
                step.components.len(),
                1 + usize::from(transfer.is_some()) + ops.len() + literals.len()
            );
            let mut expected_components =
                vec![("source_frame", frame.definition.as_ref().unwrap().as_str())];
            if let Some(t) = transfer {
                expected_components.push(("slot_transfer", t.definition.as_str()));
            }
            expected_components.extend(
                ops.iter()
                    .map(|o| ("native_normal", o.predicates["normal_execution"].as_str())),
            );
            expected_components.extend(
                literals
                    .iter()
                    .map(|l| ("literal_result", l.definition.as_str())),
            );
            expected_components.sort();
            let mut actual_components = step
                .components
                .iter()
                .map(|c| (c.role.as_str(), c.definition.as_str()))
                .collect::<Vec<_>>();
            actual_components.sort();
            assert_eq!(actual_components, expected_components);
            let ssa = |id: &str| {
                step.arguments
                    .iter()
                    .position(|a| a.kind == "ssa" && a.value_id == id)
                    .unwrap()
            };
            // Independently reconstruct each SSA definition point from original VIR.
            for a in &step.arguments {
                assert!(a.edge_id.is_none());
                if a.kind == "ssa" {
                    let mut origins = vec![];
                    if native
                        .parameter_values
                        .iter()
                        .any(|v| v.id == a.value_id && v.type_id == a.type_id)
                    {
                        origins.push(&native.blocks[0].node.id);
                    }
                    for b in &native.blocks {
                        if b.phi_values
                            .iter()
                            .any(|v| v.value.id == a.value_id && v.value.type_id == a.type_id)
                            || b.literal_values
                                .iter()
                                .any(|v| v.result.id == a.value_id && v.result.type_id == a.type_id)
                            || b.handler_exception_value
                                .as_ref()
                                .is_some_and(|v| v.id == a.value_id && v.type_id == a.type_id)
                        {
                            origins.push(&b.node.id);
                        }
                        if let Some(i) = &b.invocation {
                            if i.result.id == a.value_id && i.result.type_id == a.type_id {
                                origins.push(&i.normal_successor_id);
                            }
                        }
                    }
                    assert_eq!(origins, vec![&a.node_id]);
                } else {
                    assert_eq!(
                        a.node_id,
                        if a.kind.starts_with("source_entry_") {
                            anchor.entry_node_id.clone()
                        } else {
                            anchor.exit_node_id.clone()
                        }
                    );
                }
            }
            assert_eq!(
                step.source_result_argument,
                anchor.result.as_ref().map(|v| ssa(&v.id))
            );
            assert_eq!(
                step.condition_argument,
                blocks
                    .last()
                    .unwrap()
                    .condition_value_id
                    .as_ref()
                    .map(|id| ssa(id))
            );
            assert_eq!(
                step.return_arguments,
                blocks
                    .last()
                    .unwrap()
                    .return_value_ids
                    .iter()
                    .map(|id| ssa(id))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                step.entry_phi_arguments,
                blocks[0]
                    .phi_values
                    .iter()
                    .map(|v| ssa(&v.value.id))
                    .collect::<Vec<_>>()
            );
            for component in &step.components {
                let actual = component
                    .argument_indices
                    .iter()
                    .map(|&i| &step.arguments[i])
                    .collect::<Vec<_>>();
                match component.role.as_str() {
                    "source_frame" => {
                        assert_eq!(Some(&component.definition), frame.definition.as_ref());
                        assert_eq!(actual, frame.arguments.iter().collect::<Vec<_>>());
                    }
                    "slot_transfer" => {
                        let t = transfer.unwrap();
                        assert_eq!(component.definition, t.definition);
                        for (a, old) in actual.iter().zip(&t.arguments) {
                            if old.role == "ssa" {
                                assert_eq!(a.value_id, *old.value_id.as_ref().unwrap());
                                assert_eq!(a.kind, "ssa");
                            } else {
                                assert_eq!(a.value_id, old.slot_id);
                                assert_eq!(a.node_id, old.node_id);
                                let side = if old.role.starts_with("before_") {
                                    "source_entry_"
                                } else {
                                    "source_exit_"
                                };
                                assert_eq!(
                                    a.kind,
                                    format!(
                                        "{side}{}",
                                        if old.role.ends_with("assigned") {
                                            "assigned"
                                        } else {
                                            "slot"
                                        }
                                    )
                                );
                            }
                            assert_eq!(a.type_id, old.type_id);
                        }
                        assert_eq!(actual.len(), t.arguments.len());
                    }
                    "literal_result" => {
                        let l = literals
                            .iter()
                            .find(|l| l.definition == component.definition)
                            .unwrap();
                        assert_eq!(component.argument_indices, vec![ssa(&l.source.result.id)]);
                    }
                    "native_normal" => {
                        let o = ops
                            .iter()
                            .find(|o| o.predicates["normal_execution"] == component.definition)
                            .unwrap();
                        assert_eq!(
                            component.argument_indices,
                            o.invocation
                                .operands
                                .iter()
                                .chain(std::iter::once(&o.invocation.result))
                                .map(|v| ssa(&v.id))
                                .collect::<Vec<_>>()
                        );
                    }
                    other => panic!("unexpected step component {other}"),
                }
            }
            let depths = step
                .arguments
                .iter()
                .map(|a| types[&a.type_id].depth)
                .collect::<Vec<_>>();
            let mut values = vec![BTreeSet::new(); step.arguments.len()];
            for literal in &literals {
                values[ssa(&literal.source.result.id)] =
                    relation_tests::storage(&literal.source.value, &types)
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &v)| v.then_some(i))
                        .collect();
            }
            // Independent scalar normal examples include a literal operand sharing
            // the same binder as its invocation use (e.g. the increment's one).
            for o in &ops {
                let args = o
                    .invocation
                    .operands
                    .iter()
                    .map(|v| word(&values[ssa(&v.id)]))
                    .collect::<Vec<_>>();
                let value = match o.invocation.operation_id.as_str() {
                    "integer.i32.add.checked" => args[0].checked_add(args[1]),
                    "integer.i32.subtract.checked" => args[0].checked_sub(args[1]),
                    "integer.i32.multiply.checked" => args[0].checked_mul(args[1]),
                    "integer.i32.negate.checked" => args[0].checked_neg(),
                    "integer.i32.divide.checked" => args[0].checked_div(args[1]),
                    "integer.i32.less.checked" | "integer.i32.less.unchecked" => {
                        Some(i32::from(args[0] < args[1]))
                    }
                    "integer.i32.greater.unchecked" => Some(i32::from(args[0] > args[1])),
                    "structural.equal.mpk.csharp.value.i32.v1" => {
                        Some(i32::from(args[0] == args[1]))
                    }
                    "integer.convert.char.i32.checked" | "integer.convert.u32.i32.unchecked" => {
                        Some(args[0])
                    }
                    _ => None,
                };
                if let Some(value) = value {
                    values[ssa(&o.invocation.result.id)] = bits(value);
                }
            }
            if let Some(t) = transfer {
                let writing = t.transfer.as_ref().unwrap().kind != "load";
                let target = &t.transfer.as_ref().unwrap().slot;
                let native_arg = t.arguments.iter().find(|a| a.role == "ssa").unwrap();
                let native_value = values[ssa(native_arg.value_id.as_ref().unwrap())].clone();
                for (i, a) in step
                    .arguments
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| &a.value_id == target && a.kind != "ssa")
                {
                    let active = !writing || a.kind.starts_with("source_exit_");
                    if a.kind.ends_with("assigned") {
                        if active {
                            values[i].insert(0);
                        }
                    } else if active {
                        values[i] = native_value.clone();
                        if a.type_id != native_arg.type_id && t.memory_binding.is_none() {
                            let child = types[&native_arg.type_id].depth;
                            let depth = depths[i];
                            values[i] = native_value
                                .iter()
                                .map(|&i| 1 | (i << (depth - child)))
                                .collect();
                            values[i].insert(0);
                        }
                    }
                }
            }
            shared += step
                .components
                .iter()
                .map(|c| c.argument_indices.len())
                .sum::<usize>()
                .saturating_sub(step.arguments.len());
            let check = |input: &[BTreeSet<usize>]| {
                let args = input
                    .iter()
                    .zip(&depths)
                    .map(|(v, &d)| sparse(v, d))
                    .collect::<Vec<_>>();
                let expected = step
                    .components
                    .iter()
                    .map(|c| {
                        bit(run(
                            cert,
                            &c.definition,
                            c.argument_indices
                                .iter()
                                .map(|&i| args[i].clone())
                                .collect(),
                        ))
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .all(|v| v);
                let actual = bit(run(cert, step.definition.as_ref().unwrap(), args));
                assert_eq!(actual, expected, "{}", step.source_node_id);
                actual
            };
            let allowed = check(&values);
            positive += usize::from(allowed);
            negative += usize::from(!allowed);
            observations += 1;
            for o in ops
                .iter()
                .filter(|o| o.invocation.operation_id == "integer.i32.add.checked")
            {
                if allowed {
                    for literal in &literals {
                        if o.invocation
                            .operands
                            .iter()
                            .any(|v| v.id == literal.source.result.id)
                        {
                            let mut changed = values.clone();
                            let literal_arg = ssa(&literal.source.result.id);
                            changed[literal_arg] = bits(word(&values[literal_arg]).wrapping_add(1));
                            let inputs = o
                                .invocation
                                .operands
                                .iter()
                                .map(|v| word(&changed[ssa(&v.id)]))
                                .collect::<Vec<_>>();
                            if let Some(result) = inputs[0].checked_add(inputs[1]) {
                                changed[ssa(&o.invocation.result.id)] = bits(result);
                                let native_args = o
                                    .invocation
                                    .operands
                                    .iter()
                                    .chain(std::iter::once(&o.invocation.result))
                                    .map(|v| {
                                        let i = ssa(&v.id);
                                        sparse(&changed[i], depths[i])
                                    })
                                    .collect();
                                assert!(bit(run(
                                    cert,
                                    &o.predicates["normal_execution"],
                                    native_args
                                )));
                                assert!(
                                    !check(&changed),
                                    "a valid addition must still reject a forged source literal"
                                );
                                observations += 2;
                                negative += 1;
                            }
                        }
                    }
                }
            }
            for i in 0..values.len() {
                let mut changed = values.clone();
                let at = if step.arguments[i].kind.ends_with("assigned") {
                    0
                } else {
                    (1usize << depths[i]) - 1
                };
                if !changed[i].remove(&at) {
                    changed[i].insert(at);
                }
                let result = check(&changed);
                positive += usize::from(result);
                negative += usize::from(!result);
                observations += 1;
            }
        }
    }
    assert!(complete > 0 && positive > 0 && negative > 0 && shared > 0);
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    let index = original["functions"][0]["source_steps"]
        .as_array()
        .unwrap()
        .iter()
        .position(|s| s["definition"].is_string())
        .unwrap();
    for field in original["functions"][0]["source_steps"][index]
        .as_object()
        .unwrap()
        .keys()
    {
        let mut changed = original.clone();
        changed["functions"][0]["source_steps"][index][field] = json!("forged-step");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let mut changed = original;
    changed["functions"][0]
        .as_object_mut()
        .unwrap()
        .remove("source_steps");
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&changed).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    eprintln!("source steps: {complete} complete local relations, {positive} true/{negative} false, {shared} shared arguments; pending {pending:?}");
    observations
}

#[test]
fn csharp_03_t06_w09_control_source_steps_completeness_metadata() {
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
        NativeRuntime::StepMetadata,
    );
}

#[test]
fn csharp_03_t06_w09_control_source_constructor_step_metadata() {
    run_selected_cases_mode(
        &["type"],
        false,
        false,
        false,
        false,
        false,
        NativeRuntime::StepMetadata,
    );
}

#[test]
fn csharp_03_t06_w09_control_source_execution_completeness_metadata() {
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
        NativeRuntime::ExecutionMetadata,
    );
}

pub(super) fn audit_executions(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    cert: &mpk_cert::encode::Certificate,
    depths: &BTreeMap<&str, u32>,
) -> usize {
    let definitions = declaration_bodies(cert);
    let mut bridge_samples = BTreeMap::<String, (String, Vec<u32>)>::new();
    let mut total = 0;
    for f in p.functions() {
        let mut expected_pending = BTreeSet::new();
        let graph = f.source.source_graph.as_ref().unwrap();
        let nodes = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let mut independently_reachable = BTreeSet::new();
        let mut todo = vec![graph.nodes.first().unwrap().id.as_str()];
        while let Some(id) = todo.pop() {
            if independently_reachable.insert(id) {
                let node = nodes.get(id).unwrap();
                todo.extend(
                    node.successors
                        .iter()
                        .chain(&node.exceptional_successors)
                        .map(String::as_str),
                );
            }
        }
        let expected_excluded_unreachable = graph
            .nodes
            .iter()
            .filter(|node| !independently_reachable.contains(node.id.as_str()))
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>();
        let mut expected = BTreeSet::new();
        for step in &f.source_steps {
            if !step.pending_reasons.is_empty() {
                if step.pending_reasons == ["unreachable_source_node"] {
                    assert!(expected_excluded_unreachable.contains(&step.source_node_id));
                    continue;
                }
                assert!(independently_reachable.contains(step.source_node_id.as_str()));
                if step.pending_reasons == ["function_entry_relation_separate"] {
                    let frame = f
                        .source_frames
                        .iter()
                        .find(|frame| frame.source_node_id == step.source_node_id)
                        .unwrap();
                    let exit = frame.exit_node_id.as_ref().unwrap();
                    let outgoing = f
                        .edges
                        .iter()
                        .filter(|edge| {
                            &edge.source.source_node_id == exit
                                && edge.source.kind != "exception"
                                && edge.source.kind != "function_entry"
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(outgoing.len(), 1);
                    expected.insert((step.source_node_id.clone(), outgoing[0].source.id.clone()));
                    continue;
                }
                let source = f
                    .source
                    .source_graph
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|source| source.id == step.source_node_id)
                    .unwrap();
                if source.kind == "handler_search"
                    && step.pending_reasons == ["exception_execution_separate"]
                {
                    let outgoing = f
                        .edges
                        .iter()
                        .filter(|edge| {
                            edge.source.source_node_id == source.id
                                && edge.source.kind != "function_entry"
                        })
                        .collect::<Vec<_>>();
                    if outgoing.is_empty() {
                        let entry = f
                            .node_entries
                            .iter()
                            .find(|entry| entry.node_id == source.id)
                            .unwrap();
                        assert!(!entry.incoming.is_empty());
                        for incoming in &entry.incoming {
                            expected
                                .insert((step.source_node_id.clone(), incoming.edge_id.clone()));
                        }
                    }
                    for edge in outgoing {
                        expected.insert((step.source_node_id.clone(), edge.source.id.clone()));
                    }
                    continue;
                }
                if source.kind == "builtin_throw"
                    && step.pending_reasons == ["exception_execution_separate"]
                {
                    let outgoing = f
                        .edges
                        .iter()
                        .filter(|edge| {
                            edge.builtin_throw_source_node_id.as_deref() == Some(source.id.as_str())
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(outgoing.len(), 1);
                    expected.insert((step.source_node_id.clone(), outgoing[0].source.id.clone()));
                    continue;
                }
                expected_pending.insert(step.source_node_id.clone());
                continue;
            }
            assert!(independently_reachable.contains(step.source_node_id.as_str()));
            let exit = step.native_node_ids.last().unwrap();
            let outgoing = f
                .edges
                .iter()
                .filter(|e| {
                    &e.source.source_node_id == exit
                        && e.source.kind != "exception"
                        && e.source.kind != "function_entry"
                })
                .collect::<Vec<_>>();
            if outgoing.is_empty() {
                expected_pending.insert(step.source_node_id.clone());
            }
            for edge in outgoing {
                expected.insert((step.source_node_id.clone(), edge.source.id.clone()));
            }
        }
        assert_eq!(
            f.pending_source_execution_node_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected_pending
        );
        assert_eq!(
            f.excluded_unreachable_source_node_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected_excluded_unreachable
        );
        assert_eq!(
            f.source_executions
                .iter()
                .map(|e| (e.source_node_id.clone(), e.edge_id.clone()))
                .collect::<BTreeSet<_>>(),
            expected
        );
        for execution in &f.source_executions {
            total += 1;
            let step = f
                .source_steps
                .iter()
                .find(|s| s.source_node_id == execution.source_node_id)
                .unwrap();
            let edge = f
                .edges
                .iter()
                .find(|e| e.source.id == execution.edge_id)
                .unwrap();
            let source = f
                .source
                .source_graph
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|source| source.id == execution.source_node_id)
                .unwrap();
            let function_entry = source.kind == "entry";
            let handler_search = source.kind == "handler_search";
            let builtin_throw = source.kind == "builtin_throw";
            let unhandled_exception = handler_search
                && execution.target_node_id.is_none()
                && edge.source.kind == "exception"
                && edge.source.target_node_id.as_deref() == Some(source.id.as_str());
            if function_entry {
                assert_eq!(step.pending_reasons, ["function_entry_relation_separate"]);
                let frame = f
                    .source_frames
                    .iter()
                    .find(|frame| frame.source_node_id == step.source_node_id)
                    .unwrap();
                assert_eq!(execution.entry_node_id, step.source_node_id);
                assert_eq!(Some(&execution.exit_node_id), frame.exit_node_id.as_ref());
            } else if handler_search {
                assert_eq!(step.pending_reasons, ["exception_execution_separate"]);
                assert!(step.native_node_ids.is_empty());
                assert_eq!(execution.entry_node_id, step.source_node_id);
                assert_eq!(execution.exit_node_id, step.source_node_id);
            } else if builtin_throw {
                assert_eq!(step.pending_reasons, ["exception_execution_separate"]);
                assert!(step.native_node_ids.is_empty());
                let frame = f
                    .source_frames
                    .iter()
                    .find(|frame| frame.source_node_id == step.source_node_id)
                    .unwrap();
                assert_eq!(Some(&execution.entry_node_id), frame.entry_node_id.as_ref());
                assert_eq!(Some(&execution.exit_node_id), frame.exit_node_id.as_ref());
                assert_eq!(
                    edge.builtin_throw_source_node_id.as_ref(),
                    Some(&step.source_node_id)
                );
            } else {
                assert!(step.pending_reasons.is_empty());
                assert_eq!(step.native_node_ids.first(), Some(&execution.entry_node_id));
                assert_eq!(step.native_node_ids.last(), Some(&execution.exit_node_id));
            }
            if unhandled_exception {
                assert_eq!(execution.target_node_id, None);
            } else {
                assert_eq!(execution.target_node_id, edge.source.target_node_id);
            }
            assert_eq!(execution.argument_count, execution.arguments.len());
            assert_eq!(execution.component_count, execution.components.len());
            assert_eq!(
                execution.component_map_sha256,
                format!(
                    "{:x}",
                    Sha256::digest(
                        serde_json::to_vec(&(&execution.arguments, &execution.components)).unwrap()
                    )
                )
            );
            assert!(execution.components.iter().all(|c| {
                definitions.contains_key(&c.definition)
                    && c.argument_indices
                        .iter()
                        .all(|&i| i < execution.arguments.len())
            }));
            for component in &execution.components {
                if component.role.ends_with(":assigned")
                    || component.role.ends_with(":value")
                    || component.role == "selected_target_edge"
                    || component.role == "unselected_target_edge"
                {
                    bridge_samples
                        .entry(component.definition.clone())
                        .or_insert_with(|| {
                            (
                                component.role.clone(),
                                component
                                    .argument_indices
                                    .iter()
                                    .map(|&i| {
                                        let ty = &execution.arguments[i].type_id;
                                        depths[ty.as_str()]
                                    })
                                    .collect(),
                            )
                        });
                }
            }
            if function_entry {
                for role in [
                    "function_entry_guard",
                    "function_entry_node",
                    "function_entry_slots",
                    "bootstrap_guard",
                    "bootstrap_join",
                    "source_entry_node",
                ] {
                    assert!(execution
                        .components
                        .iter()
                        .any(|component| component.role.starts_with(role)));
                }
            } else if unhandled_exception {
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("unhandled_exception_entry")));
                assert_eq!(
                    execution
                        .components
                        .iter()
                        .filter(|c| c.role == "selected_target_edge")
                        .count(),
                    1
                );
            } else if handler_search {
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("handler_search_entry")));
                assert!(!execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("source_step")));
            } else if builtin_throw {
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("source_node_entry")));
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("builtin_throw_entry")));
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role == "builtin_throw_guard"));
            } else {
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("source_node_entry")));
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("source_step")));
            }
            assert!(
                unhandled_exception
                    || execution
                        .components
                        .iter()
                        .any(|c| c.role == "outgoing_guard" || c.role == "builtin_throw_guard")
            );
            let slot_bridges = execution
                .components
                .iter()
                .filter(|c| {
                    c.role.starts_with("source_entry_slot:")
                        || c.role.starts_with("source_exit_slot:")
                })
                .count();
            assert_eq!(
                slot_bridges,
                if function_entry {
                    2 * f.source.slots.len()
                } else if unhandled_exception {
                    0
                } else if handler_search {
                    2 * usize::from(edge.join.is_some()) * f.source.slots.len()
                } else {
                    (2 + 2 * usize::from(edge.join.is_some())) * f.source.slots.len()
                }
            );
            if function_entry {
                assert_eq!(
                    execution
                        .components
                        .iter()
                        .filter(|component| {
                            component.role.starts_with("function_entry_slot:")
                                || component.role.starts_with("bootstrap_source_slot:")
                        })
                        .count(),
                    4 * f.source.slots.len()
                );
            }
            if edge.source.target_node_id.is_some() && !unhandled_exception {
                assert!(execution.components.iter().any(|c| c.role == "edge_join"));
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role.starts_with("target_node_entry")));
                assert_eq!(
                    execution
                        .components
                        .iter()
                        .filter(|c| c.role == "selected_target_edge")
                        .count(),
                    if function_entry { 3 } else { 1 }
                );
            }
            if edge.phi_join.is_some() {
                assert!(execution
                    .components
                    .iter()
                    .any(|c| c.role == "edge_phi_join"));
            }
            if let Some(definition) = &execution.definition {
                assert!(definitions.contains_key(definition));
            }
        }
    }
    for (definition, (role, depths)) in bridge_samples {
        if role.ends_with(":assigned") {
            assert!(bit(run(
                cert,
                &definition,
                vec![V::Bit(false), V::Bit(false)]
            )));
            assert!(bit(run(
                cert,
                &definition,
                vec![V::Bit(true), V::Bit(true)]
            )));
            assert!(!bit(run(
                cert,
                &definition,
                vec![V::Bit(true), V::Bit(false)]
            )));
        } else if role.ends_with(":value") {
            let zero = encoded(depths[0], 0);
            let changed = encoded(depths[0], -1);
            assert!(bit(run(
                cert,
                &definition,
                vec![zero.clone(), changed.clone(), V::Bit(false)]
            )));
            assert!(bit(run(
                cert,
                &definition,
                vec![zero.clone(), zero, V::Bit(true)]
            )));
            assert!(!bit(run(
                cert,
                &definition,
                vec![changed.clone(), encoded(depths[0], 0), V::Bit(true)]
            )));
        } else {
            let selected = role == "selected_target_edge";
            assert!(bit(run(cert, &definition, vec![V::Bit(selected)])));
            assert!(!bit(run(cert, &definition, vec![V::Bit(!selected)])));
        }
    }
    // The smallest fixture keeps this exhaustive canonical-metadata mutation
    // check cheap while covering every serialized execution field and the
    // owning execution collection. The skipped binding maps are reconstructed from the
    // retained records and committed by their exact counts and digest above.
    if total == 19 {
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let function_index = original["functions"]
            .as_array()
            .unwrap()
            .iter()
            .position(|f| !f["source_executions"].as_array().unwrap().is_empty())
            .unwrap();
        let execution_index = 0;
        let fields = original["functions"][function_index]["source_executions"][execution_index]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for field in fields {
            let mut changed = original.clone();
            changed["functions"][function_index]["source_executions"][execution_index][&field] =
                json!("forged-source-execution");
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = original.clone();
        changed["functions"][function_index]
            .as_object_mut()
            .unwrap()
            .remove("source_executions");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        for collection in [
            "pending_source_execution_node_ids",
            "excluded_unreachable_source_node_ids",
        ] {
            let mut changed = original.clone();
            changed["functions"][function_index][collection] = json!(["forged-source-node"]);
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
    }
    if let Some(function_index) = p
        .functions()
        .iter()
        .position(|f| !f.excluded_unreachable_source_node_ids.is_empty())
    {
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let mut changed = original.clone();
        assert!(changed["functions"][function_index]
            .as_object_mut()
            .unwrap()
            .remove("excluded_unreachable_source_node_ids")
            .is_some());
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    total
}

pub(super) fn audit_steps(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
) -> usize {
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let depths: BTreeMap<_, _> = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c.depth as usize))
        .collect();
    let mut complete = 0;
    for f in p.functions() {
        let native = vir
            .functions()
            .iter()
            .find(|n| n.id == f.source.function_id)
            .unwrap();
        let protocol = native.control_protocol.as_ref().unwrap();
        assert_eq!(f.source_steps.len(), f.source_frames.len());
        let source_constructor_ids = f
            .source_steps
            .iter()
            .filter(|step| {
                step.components
                    .iter()
                    .any(|component| component.role == "source_constructor_normal")
            })
            .flat_map(|step| {
                let anchor = protocol
                    .anchors
                    .iter()
                    .find(|anchor| anchor.source_node_id == step.source_node_id)
                    .unwrap();
                native
                    .blocks
                    .iter()
                    .filter(|block| {
                        anchor.artifact_node_ids.contains(&block.node.id)
                            && block.invocation.is_some()
                            && !f
                                .native_operations
                                .iter()
                                .any(|operation| operation.source.node_id == block.node.id)
                    })
                    .map(|block| block.node.id.clone())
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            f.source_constructor_invocation_node_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            source_constructor_ids
        );
        assert!(f
            .source_constructor_invocation_node_ids
            .iter()
            .all(|id| !f.pending_native_invocation_node_ids.contains(id)));
        for (frame, step) in f.source_frames.iter().zip(&f.source_steps) {
            assert_eq!(frame.source_node_id, step.source_node_id);
            if let Some(reason) = &frame.pending_reason {
                assert_eq!(step.pending_reasons, vec![reason.clone()]);
                assert!(
                    step.arguments.is_empty()
                        && step.components.is_empty()
                        && step.definition.is_none()
                );
                continue;
            }
            let anchor = protocol
                .anchors
                .iter()
                .find(|a| a.source_node_id == frame.source_node_id)
                .unwrap();
            let missing = native
                .blocks
                .iter()
                .filter(|b| anchor.artifact_node_ids.contains(&b.node.id))
                .filter(|b| {
                    b.invocation.is_some()
                        && !f
                            .native_operations
                            .iter()
                            .any(|o| o.source.node_id == b.node.id)
                })
                .count();
            let source_constructor = missing == 1
                && step
                    .components
                    .iter()
                    .any(|component| component.role == "source_constructor_normal");
            assert_eq!(
                source_constructor,
                anchor
                    .artifact_node_ids
                    .iter()
                    .any(|id| { f.source_constructor_invocation_node_ids.contains(id) })
            );
            if missing > 0 && !source_constructor {
                assert_eq!(missing, 1);
                assert_eq!(step.pending_reasons, vec!["native_call_semantics_pending"]);
                assert!(step.definition.is_none());
            } else {
                // Every supported frame with original defined native operations
                // must yield a complete normal step, not silently stay pending.
                assert!(
                    step.pending_reasons.is_empty(),
                    "{} {:?}",
                    frame.source_node_id,
                    step.pending_reasons
                );
                let composed_depth = step
                    .arguments
                    .iter()
                    .enumerate()
                    .map(|(i, a)| i + depths[a.type_id.as_str()])
                    .fold(step.arguments.len(), usize::max);
                assert_eq!(step.definition.is_some(), composed_depth <= 256);
                assert!(!step.components.is_empty());
                complete += 1;
            }
        }
    }
    if let Some(function_index) = p
        .functions()
        .iter()
        .position(|f| !f.source_constructor_invocation_node_ids.is_empty())
    {
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for collection in [
            "pending_native_invocation_node_ids",
            "source_constructor_invocation_node_ids",
        ] {
            let mut changed = original.clone();
            changed["functions"][function_index][collection] =
                json!(["forged-native-invocation-node"]);
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = original;
        assert!(changed["functions"][function_index]
            .as_object_mut()
            .unwrap()
            .remove("source_constructor_invocation_node_ids")
            .is_some());
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    assert!(complete > 0);
    complete
}
