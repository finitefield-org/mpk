//! Original input/state observations, mutation distinction and strict import.
use super::*;
const BOOL: &str = "mpk.csharp.value.bool.v1";

#[test]
fn csharp_03_t06_w09_pattern_observations_original_sources() {
    let bundle = b();
    let mut requests = read("control-vc/loop-requests.json");
    let mut responses = read("control-emission/loop-responses.json");
    requests.as_array_mut().unwrap().extend(
        read("control-vc/measure-requests.json")
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == "total_variable")
            .cloned(),
    );
    responses.as_array_mut().unwrap().extend(
        read("control-vc/measure-responses.json")
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == "total_variable")
            .cloned(),
    );
    let patterns = read("control-emission/source-cases.json");
    let mut goals = 0;
    let mut operands = 0;
    let mut slots = 0;
    let mut nullable_slots = 0;
    let mut distinct_updates = 0;
    let mut previous: Option<Vec<u8>> = None;
    for id in [
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
    ] {
        let (context, captures, facts) =
            if let Some(request) = requests.as_array().unwrap().iter().find(|r| r["id"] == id) {
                let response = responses
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == id)
                    .unwrap();
                let (context, captures) = support::replay_context(&bundle, request);
                (context, captures, response["facts"].clone())
            } else {
                let row = patterns
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["stage"] == "patterns" && r["source_case"]["id"] == id)
                    .unwrap();
                assert_eq!(row["accepted"], true);
                let source = &row["source_case"];
                let (context, captures) = support::context(
                    &bundle,
                    source["root"].as_str().unwrap(),
                    source["source"].as_str().unwrap().as_bytes(),
                );
                (context, captures, row["data"].clone())
            };
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let original = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let old = original.control_vcs();
        let native = read(&format!("ordinary-foundation/control-edges/{id}.json"));
        let observed = generate_csharp_practical_control_vcs_with_pattern_observations(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert!(old.pattern_observations().is_empty());
        assert!(serde_json::from_slice::<Value>(&old.canonical_bytes())
            .unwrap()
            .get("pattern_observations")
            .is_none());
        assert_eq!(observed.functions(), old.functions());
        assert_eq!(observed.patterns(), old.patterns());
        assert_eq!(observed.loops(), old.loops());
        assert_eq!(observed.unresolved_regions(), old.unresolved_regions());
        assert_eq!(observed.definition_names(), old.definition_names());
        assert_eq!(observed.sequents().len(), old.sequents().len());
        assert_eq!(
            observed.pattern_observations().len(),
            old.patterns().iter().map(|p| p.steps.len()).sum::<usize>()
        );
        for (new, before) in observed.sequents().iter().zip(old.sequents()) {
            assert_eq!(new.id, before.id);
            let Some(observation) = observed
                .pattern_observations()
                .iter()
                .find(|o| o.sequent_id == new.id)
            else {
                assert_eq!(new, before);
                continue;
            };
            goals += 1;
            assert!(observation.source_semantics_pending);
            let mut source_shell = new.clone();
            source_shell.goals = before.goals.clone();
            assert_eq!(&source_shell, before);
            let [goal] = new.goals.as_slice() else {
                panic!()
            };
            let prefix = observation.original_binding_count;
            assert_eq!(
                &goal.bindings[..prefix],
                before.goals[0].bindings.as_slice()
            );
            let pattern = old
                .patterns()
                .iter()
                .find(|p| p.id == observation.pattern_id)
                .unwrap();
            let step = pattern
                .steps
                .iter()
                .find(|s| s.source_node_id == observation.source_node_id)
                .unwrap();
            let flow = old
                .functions()
                .iter()
                .find(|f| f.function_id == new.function_id)
                .unwrap();
            let graph = flow.source_graph.as_ref().unwrap();
            let function = vir
                .functions()
                .iter()
                .find(|f| f.id == new.function_id)
                .unwrap();
            assert_eq!(observation.operands.len(), step.source_inputs.len());
            for (i, operand) in observation.operands.iter().enumerate() {
                operands += 1;
                assert_eq!(operand.source_input_index, i);
                assert_eq!(operand.source_value_id, step.source_inputs[i]);
                assert_eq!(operand.binding_index, prefix + i);
                let producer = graph
                    .nodes
                    .iter()
                    .find(|n| n.result == step.source_inputs[i])
                    .unwrap();
                let anchor = function
                    .control_protocol
                    .as_ref()
                    .unwrap()
                    .anchors
                    .iter()
                    .find(|a| a.source_node_id == producer.id)
                    .unwrap();
                assert_eq!(operand.producer_source_node_id, producer.id);
                assert_eq!(anchor.result.as_ref(), Some(&operand.native_value));
                let binding = &goal.bindings[operand.binding_index];
                assert_eq!(binding.kind, "ssa");
                assert_eq!(binding.node_id, step.entry_node_id);
                assert_eq!(binding.value_id, operand.native_value.id);
                assert_eq!(binding.type_id, operand.native_value.type_id);
                assert!(binding.edge_id.is_none());
                if id == "guard_order" && step.operation == "unary_update" {
                    assert_ne!(binding.value_id, goal.bindings[0].value_id);
                    let invocation = function
                        .blocks
                        .iter()
                        .filter(|b| step.artifact_node_ids.contains(&b.node.id))
                        .filter_map(|b| b.invocation.as_ref())
                        .find(|v| v.operands.iter().any(|v| v.id == binding.value_id))
                        .unwrap();
                    assert!(invocation.operation_id.contains("add"));
                    // Instantiate the emitted application in two environments
                    // indistinguishable to the original two-argument formula.
                    let mut good = vec![0i32; goal.bindings.len()];
                    good[0] = 2;
                    good[1] = 5;
                    good[operand.binding_index] = 4;
                    let mut wrong = good.clone();
                    wrong[operand.binding_index] = 7;
                    assert_eq!(&good[..prefix], &wrong[..prefix]);
                    let relation = |environment: &[i32]| {
                        let mut term = &goal.term;
                        let mut arguments = vec![];
                        while let ContractTerm::App {
                            function, argument, ..
                        } = term
                        {
                            let ContractTerm::Var { index, .. } = argument.as_ref() else {
                                panic!()
                            };
                            arguments.push(environment[*index]);
                            term = function;
                        }
                        arguments.reverse();
                        arguments[operand.binding_index].checked_add(1) == Some(arguments[1])
                    };
                    assert!(relation(&good));
                    assert!(!relation(&wrong));
                    distinct_updates += 1;
                }
            }
            if let Some(slot) = &observation.slot {
                slots += 1;
                let transfer = flow
                    .transfers
                    .iter()
                    .find(|t| t.source_node_id == step.source_node_id)
                    .unwrap();
                assert_eq!(&slot.source, transfer);
                let nominal = flow
                    .slots
                    .iter()
                    .find(|(s, _)| s == &transfer.slot)
                    .unwrap()
                    .1
                    .as_str();
                // Retained native frame pins independently fix the represented
                // storage; source payload and closed nullable storage differ.
                let native_flow = native["functions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|f| f["source"]["function_id"] == new.function_id)
                    .unwrap();
                let storage = native_flow["slot_type_overrides"][&transfer.slot]
                    .as_str()
                    .unwrap_or(nominal);
                assert_eq!(slot.nominal_type_id, nominal);
                assert_eq!(slot.storage_type_id, storage);
                if storage != nominal {
                    nullable_slots += 1;
                    assert_eq!(id, "type");
                    assert_eq!(transfer.slot, "local:0");
                    assert_eq!(transfer.value.type_id, storage);
                }
                let indices = [
                    slot.before_assigned_index,
                    slot.before_value_index,
                    slot.after_assigned_index,
                    slot.after_value_index,
                ];
                assert_eq!(
                    indices,
                    [
                        prefix + observation.operands.len(),
                        prefix + observation.operands.len() + 1,
                        prefix + observation.operands.len() + 2,
                        prefix + observation.operands.len() + 3
                    ]
                );
                for (i, expected) in [
                    "source_entry_assigned",
                    "source_entry_slot",
                    "source_exit_assigned",
                    "source_exit_slot",
                ]
                .iter()
                .enumerate()
                {
                    let binding = &goal.bindings[indices[i]];
                    assert_eq!(&binding.kind, expected);
                    assert_eq!(binding.value_id, step.source_slot);
                    assert_eq!(
                        &binding.node_id,
                        if i < 2 {
                            &step.entry_node_id
                        } else {
                            &step.exit_node_id
                        }
                    );
                    assert_eq!(binding.type_id, if i % 2 == 0 { BOOL } else { storage });
                    assert!(binding.edge_id.is_none());
                }
                assert_ne!(goal.bindings[indices[0]], goal.bindings[indices[2]]);
                assert_ne!(goal.bindings[indices[1]], goal.bindings[indices[3]]);
            } else {
                assert!(!["load", "store", "pattern_bind"].contains(&step.operation.as_str()));
            }
            let mut term = &goal.term;
            let mut arguments = vec![];
            while let ContractTerm::App {
                function, argument, ..
            } = term
            {
                arguments.push(argument.as_ref());
                term = function;
            }
            arguments.reverse();
            assert_eq!(arguments.len(), goal.bindings.len());
            for (i, (argument, binding)) in arguments.iter().zip(&goal.bindings).enumerate() {
                assert!(matches!(argument, ContractTerm::Var { index, type_id }
                    if *index == i && type_id == &binding.type_id));
            }
            let ContractTerm::Const { name, type_id } = term else {
                panic!()
            };
            assert_eq!(
                name,
                &format!(
                    "Mpk.CSharp.Control.PatternStep.{}.{}",
                    pattern.id, step.source_node_id
                )
            );
            let expected = goal
                .bindings
                .iter()
                .rev()
                .fold(BOOL.to_string(), |r, b| format!("({}->{r})", b.type_id));
            assert_eq!(type_id, &expected);
        }
        let bytes = observed.canonical_bytes();
        assert_eq!(
            import_csharp_practical_control_vcs_with_pattern_observations(&bytes, vir).unwrap(),
            observed
        );
        if let Some(previous) = previous {
            assert!(
                import_csharp_practical_control_vcs_with_pattern_observations(&previous, vir)
                    .is_err()
            );
        }
        previous = Some(bytes.clone());
        if !observed.pattern_observations().is_empty() {
            let mut metadata: Value = serde_json::from_slice(&bytes).unwrap();
            metadata["pattern_observations"][0]["source_semantics_pending"] = json!(false);
            assert!(
                import_csharp_practical_control_vcs_with_pattern_observations(
                    &serde_json::to_vec(&metadata).unwrap(),
                    vir
                )
                .is_err()
            );
        }
        let root = if let Some(path) = std::env::var_os("MPK_W09_PATTERN_OBSERVATION_OUTPUT") {
            let root = PathBuf::from(path);
            fs::create_dir_all(&root).unwrap();
            root
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../develop/migrations/csharp-03/ordinary-foundation/pattern-observation-vc",
            )
        };
        let path = root.join(format!("{id}.json"));
        if std::env::var_os("MPK_W09_PATTERN_OBSERVATION_OUTPUT").is_some() {
            fs::write(path, bytes).unwrap();
        } else {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }
    assert_eq!(goals, 102);
    assert!(operands > 0 && slots > 0);
    assert!(nullable_slots > 0);
    assert_eq!(distinct_updates, 2);
    eprintln!("pattern source observations: {goals} original goals, {operands} exact operands, {slots} distinct before/after slot snapshots, {nullable_slots} nullable storage observations, {distinct_updates} captured/current update distinctions");
}
