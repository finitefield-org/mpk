//! Complete reachable source executions connected to exact incoming and
//! outgoing native CFG snapshots. Function entry, exception-producing
//! operations, alias updates, constructor calls and terminal paths use their
//! dedicated compositions. Structurally unreachable source nodes are retained
//! as explicit exclusions.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSourceExecution {
    pub source_node_id: String,
    pub entry_node_id: String,
    pub exit_node_id: String,
    pub edge_id: String,
    pub target_node_id: Option<String>,
    /// Reconstructed from the retained entry, step and edge records. The
    /// canonical metadata carries the exact map hash without duplicating those
    /// large binding objects for every source edge.
    #[serde(skip)]
    pub arguments: Vec<ControlBinding>,
    #[serde(skip)]
    pub components: Vec<OrdinaryControlStepComponent>,
    pub argument_count: usize,
    pub component_count: usize,
    pub component_map_sha256: String,
    pub state_rule: String,
    pub definition: Option<String>,
}

impl OrdinaryControlSourceExecution {
    fn argument(&mut self, binding: ControlBinding) -> usize {
        if let Some(i) = self.arguments.iter().position(|a| a == &binding) {
            return i;
        }
        self.arguments.push(binding);
        self.arguments.len() - 1
    }

    fn relation(&mut self, role: impl Into<String>, definition: &str, args: &[ControlBinding]) {
        let argument_indices = args.iter().map(|a| self.argument(a.clone())).collect();
        self.components.push(OrdinaryControlStepComponent {
            role: role.into(),
            definition: definition.into(),
            argument_indices,
        });
    }
}

fn add_state_bridge(
    c: &mut Clauses<'_>,
    execution: &mut OrdinaryControlSourceExecution,
    role: &str,
    left_assigned: ControlBinding,
    left_value: ControlBinding,
    right_assigned: ControlBinding,
    right_value: ControlBinding,
) -> R<()> {
    if left_assigned.type_id != SOURCE_BOOL
        || right_assigned.type_id != SOURCE_BOOL
        || left_value.type_id != right_value.type_id
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let value_depth = c
        .carriers
        .get(left_value.type_id.as_str())
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    let assigned_definition = name("SourceExecutionAssignedEqual", &"v1");
    if !c.b.globals.contains_key(&assigned_definition) {
        let bool_equal = construction_data::physical_equal(&mut c.b, 0)?;
        let left_flag = c.b.var(1)?;
        let right_flag = c.b.var(0)?;
        let same_assigned = call(&mut c.b, &bool_equal, vec![left_flag, right_flag])?;
        define(&mut c.b, &assigned_definition, &[0, 0], 0, same_assigned)?;
    }
    execution.relation(
        format!("{role}:assigned"),
        &assigned_definition,
        &[left_assigned.clone(), right_assigned],
    );

    // Keep the two potentially C253 values first. Adding both assignedness
    // flags to this component would exceed the nested 256-binder limit.
    let value_definition = name("SourceExecutionValueWhenAssignedEqual", &left_value.type_id);
    if !c.b.globals.contains_key(&value_definition) {
        let left = c.b.var(2)?;
        let right = c.b.var(1)?;
        let assigned = c.b.var(0)?;
        let value_equal = construction_data::physical_equal(&mut c.b, value_depth)?;
        let same_value = call(&mut c.b, &value_equal, vec![left, right])?;
        let yes = bit(&mut c.b, true)?;
        let relevant_value = mux(&mut c.b, assigned, same_value, yes)?;
        define(
            &mut c.b,
            &value_definition,
            &[value_depth, value_depth, 0],
            0,
            relevant_value,
        )?;
    }
    execution.relation(
        format!("{role}:value"),
        &value_definition,
        &[left_value, right_value, left_assigned],
    );
    Ok(())
}

fn add_entry(
    execution: &mut OrdinaryControlSourceExecution,
    entry: &OrdinaryControlNodeEntry,
    role: &str,
) -> R<()> {
    if let Some(definition) = &entry.definition {
        execution.relation(role, definition, &entry.arguments);
    } else if !entry.components.is_empty() {
        for component in &entry.components {
            let arguments = component
                .argument_indices
                .iter()
                .map(|&i| {
                    entry
                        .arguments
                        .get(i)
                        .cloned()
                        .ok_or(OrdinaryCarrierError::Linkage)
                })
                .collect::<R<Vec<_>>>()?;
            execution.relation(
                format!("{role}:{}", component.role),
                &component.definition,
                &arguments,
            );
        }
    } else {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(())
}

fn add_step(
    execution: &mut OrdinaryControlSourceExecution,
    step: &OrdinaryControlSourceStep,
) -> R<()> {
    if let Some(definition) = &step.definition {
        execution.relation("source_step", definition, &step.arguments);
    } else if !step.components.is_empty() && step.pending_reasons.is_empty() {
        for component in &step.components {
            let arguments = component
                .argument_indices
                .iter()
                .map(|&i| {
                    step.arguments
                        .get(i)
                        .cloned()
                        .ok_or(OrdinaryCarrierError::Linkage)
                })
                .collect::<R<Vec<_>>>()?;
            execution.relation(
                format!("source_step:{}", component.role),
                &component.definition,
                &arguments,
            );
        }
    } else {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(())
}

fn add_selected_target(
    c: &mut Clauses<'_>,
    execution: &mut OrdinaryControlSourceExecution,
    entry: &OrdinaryControlNodeEntry,
    edge_id: &str,
) -> R<()> {
    let mut arguments = vec![];
    let mut selected_position = None;
    for incoming in &entry.incoming {
        let argument = entry
            .arguments
            .get(incoming.selected_argument)
            .cloned()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if incoming.edge_id == edge_id {
            selected_position = Some(arguments.len());
        }
        arguments.push(argument);
    }
    let selected_position = selected_position.ok_or(OrdinaryCarrierError::Linkage)?;
    for (i, argument) in arguments.into_iter().enumerate() {
        let selected = i == selected_position;
        let definition = name("SourceExecutionSelected", &selected);
        if !c.b.globals.contains_key(&definition) {
            let value = c.b.var(0)?;
            let body = if selected {
                value
            } else {
                call(&mut c.b, "Std.Bool.not", vec![value])?
            };
            define(&mut c.b, &definition, &[0], 0, body)?;
        }
        execution.relation(
            if selected {
                "selected_target_edge"
            } else {
                "unselected_target_edge"
            },
            &definition,
            &[argument],
        );
    }
    Ok(())
}

fn state_pair(
    bindings: &[ControlBinding],
    kind_assigned: &str,
    kind_value: &str,
    node_id: &str,
    edge_id: Option<&str>,
    slot: &str,
) -> R<(ControlBinding, ControlBinding)> {
    let find = |kind: &str| {
        bindings
            .iter()
            .find(|a| {
                a.kind == kind
                    && a.node_id == node_id
                    && a.edge_id.as_deref() == edge_id
                    && a.value_id == slot
            })
            .cloned()
            .ok_or(OrdinaryCarrierError::Linkage)
    };
    Ok((find(kind_assigned)?, find(kind_value)?))
}

fn slot_argument(argument: &OrdinaryControlSlotArgument) -> ControlBinding {
    ControlBinding {
        kind: argument.role.clone(),
        edge_id: None,
        node_id: argument.node_id.clone(),
        value_id: argument
            .value_id
            .clone()
            .unwrap_or_else(|| argument.slot_id.clone()),
        type_id: argument.type_id.clone(),
    }
}

fn emit_compact(
    c: &mut Clauses<'_>,
    execution: &OrdinaryControlSourceExecution,
) -> R<Option<String>> {
    let depths = execution
        .arguments
        .iter()
        .map(|a| {
            c.carriers
                .get(a.type_id.as_str())
                .map(|c| c.depth)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    if depths.len() > 256
        || depths
            .iter()
            .enumerate()
            .any(|(i, &depth)| depth as usize > 256 - i)
    {
        return Ok(None);
    }
    let terms = (0..depths.len())
        .map(|i| c.b.var((depths.len() - 1 - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let mut body = bit(&mut c.b, true)?;
    for component in &execution.components {
        let part = call(
            &mut c.b,
            &component.definition,
            component
                .argument_indices
                .iter()
                .map(|&i| terms[i])
                .collect(),
        )?;
        body = call(&mut c.b, "Std.Bool.and", vec![body, part])?;
    }
    let definition = name(
        "SourceNormalExecution",
        &(
            &execution.source_node_id,
            &execution.edge_id,
            &execution.arguments,
            &execution.components,
        ),
    );
    define(&mut c.b, &definition, &depths, 0, body)?;
    Ok(Some(definition))
}

fn finish(c: &mut Clauses<'_>, execution: &mut OrdinaryControlSourceExecution) -> R<()> {
    execution.argument_count = execution.arguments.len();
    execution.component_count = execution.components.len();
    execution.component_map_sha256 = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(&execution.arguments, &execution.components))
                .map_err(|_| OrdinaryCarrierError::Shape)?,
        ),
    );
    execution.definition = emit_compact(c, execution)?;
    Ok(())
}

fn append_function_entry(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
) -> R<OrdinaryControlSourceExecution> {
    let source = flow
        .source
        .source_graph
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?
        .nodes
        .iter()
        .find(|node| node.kind == "entry")
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let frame = flow
        .source_frames
        .iter()
        .find(|frame| frame.source_node_id == source.id)
        .filter(|frame| frame.pending_reason.as_deref() == Some("function_entry_relation_separate"))
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut incoming_edges = flow.edges.iter().filter(|edge| {
        edge.source.kind == "function_entry"
            && edge.source.target_node_id.as_deref() == Some(source.id.as_str())
    });
    let incoming = incoming_edges.next().ok_or(OrdinaryCarrierError::Linkage)?;
    if incoming_edges.next().is_some() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let frame_entry = frame
        .entry_node_id
        .as_deref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let frame_exit = frame
        .exit_node_id
        .as_deref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let initial_entry = flow
        .node_entries
        .iter()
        .find(|entry| entry.node_id == source.id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut bootstrap_edges = flow.edges.iter().filter(|edge| {
        edge.source.source_node_id == source.id
            && edge.source.kind != "exception"
            && edge.source.kind != "function_entry"
    });
    let bootstrap = bootstrap_edges
        .next()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if bootstrap_edges.next().is_some()
        || bootstrap.source.target_node_id.as_deref() != Some(frame_entry)
        || frame_entry != frame_exit
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let source_entry = flow
        .node_entries
        .iter()
        .find(|entry| entry.node_id == frame_entry)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut outgoing_edges = flow.edges.iter().filter(|edge| {
        edge.source.source_node_id == frame_exit
            && edge.source.kind != "exception"
            && edge.source.kind != "function_entry"
    });
    let outgoing = outgoing_edges.next().ok_or(OrdinaryCarrierError::Linkage)?;
    if outgoing_edges.next().is_some() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let slot_relation = flow
        .slot_relations
        .iter()
        .find(|relation| {
            relation.transfer.is_none() && relation.execution_scope == "function_entry"
        })
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let slot_arguments = slot_relation
        .arguments
        .iter()
        .map(slot_argument)
        .collect::<Vec<_>>();
    let mut execution = OrdinaryControlSourceExecution {
        source_node_id: source.id.clone(),
        entry_node_id: source.id.clone(),
        exit_node_id: frame_exit.to_owned(),
        edge_id: outgoing.source.id.clone(),
        target_node_id: outgoing.source.target_node_id.clone(),
        arguments: vec![],
        components: vec![],
        argument_count: 0,
        component_count: 0,
        component_map_sha256: String::new(),
        state_rule: concat!(
            "function_entry_slots_to_selected_initial_entry; ",
            "initial_entry_to_source_entry_anchor; ",
            "source_entry_anchor_to_selected_target_entry"
        )
        .into(),
        definition: None,
    };

    execution.relation(
        "function_entry_guard",
        incoming
            .guard_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?,
        &incoming.source.guard.bindings,
    );
    add_entry(&mut execution, initial_entry, "function_entry_node")?;
    add_selected_target(c, &mut execution, initial_entry, &incoming.source.id)?;
    execution.relation(
        "function_entry_slots",
        &slot_relation.definition,
        &slot_arguments,
    );
    for (slot, _) in &flow.source.slots {
        let (slot_assigned, slot_value) = state_pair(
            &slot_arguments,
            "entry_assigned",
            "entry_value",
            &source.id,
            None,
            slot,
        )?;
        let (entry_assigned, entry_value) = state_pair(
            &initial_entry.arguments,
            "slot_assigned",
            "current_slot",
            &source.id,
            None,
            slot,
        )?;
        add_state_bridge(
            c,
            &mut execution,
            &format!("function_entry_slot:{slot}"),
            slot_assigned,
            slot_value,
            entry_assigned,
            entry_value,
        )?;
    }

    execution.relation(
        "bootstrap_guard",
        bootstrap
            .guard_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?,
        &bootstrap.source.guard.bindings,
    );
    let bootstrap_join = bootstrap
        .join
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    execution.relation(
        "bootstrap_join",
        &bootstrap_join.definition,
        &bootstrap_join.arguments,
    );
    for (slot, _) in &flow.source.slots {
        let (entry_assigned, entry_value) = state_pair(
            &initial_entry.arguments,
            "slot_assigned",
            "current_slot",
            &source.id,
            None,
            slot,
        )?;
        let (edge_assigned, edge_value) = state_pair(
            &bootstrap_join.arguments,
            "slot_assigned",
            "current_slot",
            &source.id,
            Some(&bootstrap.source.id),
            slot,
        )?;
        add_state_bridge(
            c,
            &mut execution,
            &format!("bootstrap_source_slot:{slot}"),
            entry_assigned,
            entry_value,
            edge_assigned,
            edge_value,
        )?;
    }
    if let Some(phi) = &bootstrap.phi_join {
        execution.relation("bootstrap_phi_join", &phi.definition, &phi.arguments);
    }
    add_entry(&mut execution, source_entry, "source_entry_node")?;
    add_selected_target(c, &mut execution, source_entry, &bootstrap.source.id)?;

    execution.relation(
        "outgoing_guard",
        outgoing
            .guard_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?,
        &outgoing.source.guard.bindings,
    );
    let outgoing_join = outgoing
        .join
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    execution.relation(
        "edge_join",
        &outgoing_join.definition,
        &outgoing_join.arguments,
    );
    for (slot, _) in &flow.source.slots {
        let (entry_assigned, entry_value) = state_pair(
            &source_entry.arguments,
            "slot_assigned",
            "current_slot",
            frame_exit,
            None,
            slot,
        )?;
        let (edge_assigned, edge_value) = state_pair(
            &outgoing_join.arguments,
            "slot_assigned",
            "current_slot",
            frame_exit,
            Some(&outgoing.source.id),
            slot,
        )?;
        add_state_bridge(
            c,
            &mut execution,
            &format!("source_exit_slot:{slot}"),
            entry_assigned,
            entry_value,
            edge_assigned,
            edge_value,
        )?;
    }
    if let Some(phi) = &outgoing.phi_join {
        execution.relation("edge_phi_join", &phi.definition, &phi.arguments);
    }
    let target_id = outgoing
        .source
        .target_node_id
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let target_entry = flow
        .node_entries
        .iter()
        .find(|entry| &entry.node_id == target_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    add_entry(&mut execution, target_entry, "target_node_entry")?;
    add_selected_target(c, &mut execution, target_entry, &outgoing.source.id)?;
    finish(c, &mut execution)?;
    Ok(execution)
}

fn append_handler_search(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
) -> R<(Vec<OrdinaryControlSourceExecution>, BTreeSet<String>)> {
    let source_graph = flow
        .source
        .source_graph
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut executions = vec![];
    let mut completed = BTreeSet::new();
    for source in source_graph
        .nodes
        .iter()
        .filter(|source| source.kind == "handler_search")
    {
        let step = flow
            .source_steps
            .iter()
            .find(|step| step.source_node_id == source.id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if step.pending_reasons != ["exception_execution_separate"] {
            continue;
        }
        if !step.native_node_ids.is_empty()
            || !step.arguments.is_empty()
            || !step.components.is_empty()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let source_entry = flow
            .node_entries
            .iter()
            .find(|entry| entry.node_id == source.id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let outgoing = flow
            .edges
            .iter()
            .filter(|edge| {
                edge.source.source_node_id == source.id && edge.source.kind != "function_entry"
            })
            .collect::<Vec<_>>();
        if outgoing.is_empty() {
            if source_entry.incoming.is_empty() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            completed.insert(source.id.clone());
            for incoming in &source_entry.incoming {
                let edge = flow
                    .edges
                    .iter()
                    .find(|edge| edge.source.id == incoming.edge_id)
                    .filter(|edge| {
                        edge.source.kind == "exception"
                            && edge.source.target_node_id.as_deref() == Some(source.id.as_str())
                    })
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let mut execution = OrdinaryControlSourceExecution {
                    source_node_id: source.id.clone(),
                    entry_node_id: source.id.clone(),
                    exit_node_id: source.id.clone(),
                    edge_id: edge.source.id.clone(),
                    target_node_id: None,
                    arguments: vec![],
                    components: vec![],
                    argument_count: 0,
                    component_count: 0,
                    component_map_sha256: String::new(),
                    state_rule: "selected_exception_edge_to_unhandled_termination".into(),
                    definition: None,
                };
                add_entry(&mut execution, source_entry, "unhandled_exception_entry")?;
                add_selected_target(c, &mut execution, source_entry, &edge.source.id)?;
                finish(c, &mut execution)?;
                executions.push(execution);
            }
            continue;
        }
        completed.insert(source.id.clone());
        for edge in outgoing {
            let mut execution = OrdinaryControlSourceExecution {
                source_node_id: source.id.clone(),
                entry_node_id: source.id.clone(),
                exit_node_id: source.id.clone(),
                edge_id: edge.source.id.clone(),
                target_node_id: edge.source.target_node_id.clone(),
                arguments: vec![],
                components: vec![],
                argument_count: 0,
                component_count: 0,
                component_map_sha256: String::new(),
                state_rule: concat!(
                    "selected_incoming_to_handler_search_entry; ",
                    "handler_search_entry_to_selected_edge_and_target_entry"
                )
                .into(),
                definition: None,
            };
            add_entry(&mut execution, source_entry, "handler_search_entry")?;
            execution.relation(
                "outgoing_guard",
                edge.guard_definition
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?,
                &edge.source.guard.bindings,
            );
            if let Some(join) = &edge.join {
                execution.relation("edge_join", &join.definition, &join.arguments);
                for (slot, _) in &flow.source.slots {
                    let (entry_assigned, entry_value) = state_pair(
                        &source_entry.arguments,
                        "slot_assigned",
                        "current_slot",
                        &source.id,
                        None,
                        slot,
                    )?;
                    let (edge_assigned, edge_value) = state_pair(
                        &join.arguments,
                        "slot_assigned",
                        "current_slot",
                        &source.id,
                        Some(&edge.source.id),
                        slot,
                    )?;
                    add_state_bridge(
                        c,
                        &mut execution,
                        &format!("source_exit_slot:{slot}"),
                        entry_assigned,
                        entry_value,
                        edge_assigned,
                        edge_value,
                    )?;
                }
            } else if edge.source.target_node_id.is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            if let Some(phi) = &edge.phi_join {
                execution.relation("edge_phi_join", &phi.definition, &phi.arguments);
            }
            if let Some(target) = &edge.source.target_node_id {
                let target_entry = flow
                    .node_entries
                    .iter()
                    .find(|entry| &entry.node_id == target)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                add_entry(&mut execution, target_entry, "target_node_entry")?;
                add_selected_target(c, &mut execution, target_entry, &edge.source.id)?;
            }
            finish(c, &mut execution)?;
            executions.push(execution);
        }
    }
    Ok((executions, completed))
}

fn append_builtin_throw(
    c: &mut Clauses<'_>,
    flow: &OrdinaryControlEdgeFunction,
) -> R<Option<OrdinaryControlSourceExecution>> {
    let source_graph = flow
        .source
        .source_graph
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let Some(source) = source_graph
        .nodes
        .iter()
        .find(|source| source.kind == "builtin_throw")
    else {
        return Ok(None);
    };
    let step = flow
        .source_steps
        .iter()
        .find(|step| step.source_node_id == source.id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if step.pending_reasons != ["exception_execution_separate"] {
        return Ok(None);
    }
    if !step.native_node_ids.is_empty() || !step.arguments.is_empty() || !step.components.is_empty()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let frame = flow
        .source_frames
        .iter()
        .find(|frame| frame.source_node_id == source.id)
        .filter(|frame| frame.pending_reason.as_deref() == Some("exception_execution_separate"))
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let entry_node_id = frame
        .entry_node_id
        .as_deref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if frame.exit_node_id.as_deref() != Some(entry_node_id) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let source_entry = flow
        .node_entries
        .iter()
        .find(|entry| entry.node_id == source.id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let native_entry = flow
        .node_entries
        .iter()
        .find(|entry| entry.node_id == entry_node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut edges = flow
        .edges
        .iter()
        .filter(|edge| edge.builtin_throw_source_node_id.as_deref() == Some(source.id.as_str()));
    let edge = edges.next().ok_or(OrdinaryCarrierError::Linkage)?;
    if edges.next().is_some()
        || edge.source.source_node_id != entry_node_id
        || edge.source.kind != "exception"
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut execution = OrdinaryControlSourceExecution {
        source_node_id: source.id.clone(),
        entry_node_id: entry_node_id.into(),
        exit_node_id: entry_node_id.into(),
        edge_id: edge.source.id.clone(),
        target_node_id: edge.source.target_node_id.clone(),
        arguments: vec![],
        components: vec![],
        argument_count: 0,
        component_count: 0,
        component_map_sha256: String::new(),
        state_rule: concat!(
            "selected_source_entry_to_native_throw_entry; ",
            "frozen_builtin_exception_to_selected_handler_entry"
        )
        .into(),
        definition: None,
    };
    add_entry(&mut execution, source_entry, "source_node_entry")?;
    add_entry(&mut execution, native_entry, "builtin_throw_entry")?;
    for (slot, _) in &flow.source.slots {
        let (source_assigned, source_value) = state_pair(
            &source_entry.arguments,
            "slot_assigned",
            "current_slot",
            &source.id,
            None,
            slot,
        )?;
        let (native_assigned, native_value) = state_pair(
            &native_entry.arguments,
            "slot_assigned",
            "current_slot",
            entry_node_id,
            None,
            slot,
        )?;
        add_state_bridge(
            c,
            &mut execution,
            &format!("source_entry_slot:{slot}"),
            source_assigned,
            source_value,
            native_assigned,
            native_value,
        )?;
    }
    execution.relation(
        "builtin_throw_guard",
        edge.guard_definition
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?,
        &edge.source.guard.bindings,
    );
    let join = edge.join.as_ref().ok_or(OrdinaryCarrierError::Linkage)?;
    execution.relation("edge_join", &join.definition, &join.arguments);
    for (slot, _) in &flow.source.slots {
        let (native_assigned, native_value) = state_pair(
            &native_entry.arguments,
            "slot_assigned",
            "current_slot",
            entry_node_id,
            None,
            slot,
        )?;
        let (edge_assigned, edge_value) = state_pair(
            &join.arguments,
            "slot_assigned",
            "current_slot",
            entry_node_id,
            Some(&edge.source.id),
            slot,
        )?;
        add_state_bridge(
            c,
            &mut execution,
            &format!("source_exit_slot:{slot}"),
            native_assigned,
            native_value,
            edge_assigned,
            edge_value,
        )?;
    }
    if let Some(phi) = &edge.phi_join {
        execution.relation("edge_phi_join", &phi.definition, &phi.arguments);
    }
    let target = edge
        .source
        .target_node_id
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let target_entry = flow
        .node_entries
        .iter()
        .find(|entry| &entry.node_id == target)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    add_entry(&mut execution, target_entry, "target_node_entry")?;
    add_selected_target(c, &mut execution, target_entry, &edge.source.id)?;
    finish(c, &mut execution)?;
    Ok(Some(execution))
}

pub(super) fn append(c: &mut Clauses<'_>, p: &mut OrdinaryControlEdgeProgram) -> R<()> {
    for flow in &mut p.functions {
        let mut pending = BTreeSet::new();
        let mut excluded_unreachable = BTreeSet::new();
        flow.source_executions.push(append_function_entry(c, flow)?);
        let (handler_executions, completed_handler_searches) = append_handler_search(c, flow)?;
        flow.source_executions.extend(handler_executions);
        let builtin_throw = append_builtin_throw(c, flow)?;
        let completed_builtin_throw = builtin_throw
            .as_ref()
            .map(|execution| execution.source_node_id.clone());
        flow.source_executions.extend(builtin_throw);
        for step in &flow.source_steps {
            if !step.pending_reasons.is_empty() {
                if step.pending_reasons == ["unreachable_source_node"] {
                    excluded_unreachable.insert(step.source_node_id.clone());
                    continue;
                }
                if step.pending_reasons == ["function_entry_relation_separate"] {
                    continue;
                }
                if completed_handler_searches.contains(&step.source_node_id) {
                    continue;
                }
                if completed_builtin_throw.as_ref() == Some(&step.source_node_id) {
                    continue;
                }
                pending.insert(step.source_node_id.clone());
                continue;
            }
            let entry_node_id = step
                .native_node_ids
                .first()
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let exit_node_id = step
                .native_node_ids
                .last()
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let source_entry = flow
                .node_entries
                .iter()
                .find(|e| &e.node_id == entry_node_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let outgoing = flow
                .edges
                .iter()
                .filter(|e| {
                    &e.source.source_node_id == exit_node_id
                        && e.source.kind != "exception"
                        && e.source.kind != "function_entry"
                })
                .collect::<Vec<_>>();
            if outgoing.is_empty() {
                pending.insert(step.source_node_id.clone());
                continue;
            }
            for edge in outgoing {
                let guard = edge
                    .guard_definition
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let mut execution = OrdinaryControlSourceExecution {
                    source_node_id: step.source_node_id.clone(),
                    entry_node_id: entry_node_id.clone(),
                    exit_node_id: exit_node_id.clone(),
                    edge_id: edge.source.id.clone(),
                    target_node_id: edge.source.target_node_id.clone(),
                    arguments: vec![],
                    components: vec![],
                    argument_count: 0,
                    component_count: 0,
                    component_map_sha256: String::new(),
                    state_rule: concat!(
                        "selected_incoming_to_source_entry; complete_local_normal_step; ",
                        "source_exit_to_selected_edge_and_target_entry"
                    )
                    .into(),
                    definition: None,
                };
                add_entry(&mut execution, source_entry, "source_node_entry")?;
                add_step(&mut execution, step)?;
                for (slot, _) in &flow.source.slots {
                    let (entry_assigned, entry_value) = state_pair(
                        &source_entry.arguments,
                        "slot_assigned",
                        "current_slot",
                        entry_node_id,
                        None,
                        slot,
                    )?;
                    let (step_assigned, step_value) = state_pair(
                        &step.arguments,
                        "source_entry_assigned",
                        "source_entry_slot",
                        entry_node_id,
                        None,
                        slot,
                    )?;
                    add_state_bridge(
                        c,
                        &mut execution,
                        &format!("source_entry_slot:{slot}"),
                        entry_assigned,
                        entry_value,
                        step_assigned,
                        step_value,
                    )?;
                }
                execution.relation("outgoing_guard", guard, &edge.source.guard.bindings);
                if let Some(join) = &edge.join {
                    execution.relation("edge_join", &join.definition, &join.arguments);
                    for (slot, _) in &flow.source.slots {
                        let (step_assigned, step_value) = state_pair(
                            &step.arguments,
                            "source_exit_assigned",
                            "source_exit_slot",
                            exit_node_id,
                            None,
                            slot,
                        )?;
                        let (edge_assigned, edge_value) = state_pair(
                            &join.arguments,
                            "slot_assigned",
                            "current_slot",
                            exit_node_id,
                            Some(&edge.source.id),
                            slot,
                        )?;
                        add_state_bridge(
                            c,
                            &mut execution,
                            &format!("source_exit_slot:{slot}"),
                            step_assigned,
                            step_value,
                            edge_assigned,
                            edge_value,
                        )?;
                    }
                } else if edge.source.target_node_id.is_some() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                if let Some(phi) = &edge.phi_join {
                    execution.relation("edge_phi_join", &phi.definition, &phi.arguments);
                }
                if let Some(target) = &edge.source.target_node_id {
                    let target_entry = flow
                        .node_entries
                        .iter()
                        .find(|e| &e.node_id == target)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    add_entry(&mut execution, target_entry, "target_node_entry")?;
                    add_selected_target(c, &mut execution, target_entry, &edge.source.id)?;
                }
                finish(c, &mut execution)?;
                flow.source_executions.push(execution);
            }
        }
        flow.pending_source_execution_node_ids = pending.into_iter().collect();
        flow.excluded_unreachable_source_node_ids = excluded_unreachable.into_iter().collect();
    }
    Ok(())
}
