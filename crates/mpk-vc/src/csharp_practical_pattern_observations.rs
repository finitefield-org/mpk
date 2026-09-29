//! Complete source operands and slot snapshots for original pattern obligations.
//! The historical two-observation route remains reconstructible. This route
//! supplies the additional arguments needed by source semantics; it supplies no
//! predicate definition, execution premise or discharged proof.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatternObservationError {
    Contract,
    Limit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PatternOperandObservation {
    pub source_input_index: usize,
    pub source_value_id: String,
    pub producer_source_node_id: String,
    pub native_value: TypedValueRef,
    /// Index in the original sequent's extended binding/argument order.
    pub binding_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PatternSlotObservation {
    pub source: ControlSlotTransfer,
    pub nominal_type_id: String,
    pub storage_type_id: String,
    pub before_assigned_index: usize,
    pub before_value_index: usize,
    pub after_assigned_index: usize,
    pub after_value_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PatternStepObservation {
    pub sequent_id: String,
    pub function_id: String,
    pub pattern_id: String,
    pub source_node_id: String,
    pub original_binding_count: usize,
    pub operands: Vec<PatternOperandObservation>,
    pub slot: Option<PatternSlotObservation>,
    pub source_semantics_pending: bool,
}

fn contract(error: ControlVcError) -> PatternObservationError {
    match error {
        ControlVcError::Contract => PatternObservationError::Contract,
        ControlVcError::Limit => PatternObservationError::Limit,
    }
}

fn append_slot(
    bindings: &mut Vec<ControlBinding>,
    transfer: &ControlSlotTransfer,
    kind: &str,
    node_id: &str,
    type_id: &str,
) -> usize {
    let index = bindings.len();
    bindings.push(ControlBinding {
        kind: kind.into(),
        edge_id: None,
        node_id: node_id.into(),
        value_id: transfer.slot.clone(),
        type_id: type_id.into(),
    });
    index
}

pub fn generate_csharp_practical_control_vcs_with_pattern_observations(
    vir: &ValidatedPracticalVir,
) -> Result<ControlVcProgram, PatternObservationError> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| PatternObservationError::Contract)?;
    let mut program = generate_control_vcs(vir, &data).map_err(contract)?;
    let payloads = option_payload_types(vir).map_err(contract)?;
    for pattern in &program.patterns {
        let function = vir
            .functions()
            .iter()
            .find(|f| f.id == pattern.function_id)
            .ok_or(PatternObservationError::Contract)?;
        let protocol = function
            .control_protocol
            .as_ref()
            .ok_or(PatternObservationError::Contract)?;
        let flow = program
            .functions
            .iter()
            .find(|f| f.function_id == pattern.function_id)
            .ok_or(PatternObservationError::Contract)?;
        let graph = flow
            .source_graph
            .as_ref()
            .ok_or(PatternObservationError::Contract)?;
        let storage_types = represented_slot_types(vir, flow).map_err(contract)?;
        for step in &pattern.steps {
            let sequent_id = format!("{}.{}", pattern.id, step.source_node_id);
            let sequent = program
                .sequents
                .iter_mut()
                .find(|s| s.id == sequent_id)
                .ok_or(PatternObservationError::Contract)?;
            let [goal] = sequent.goals.as_mut_slice() else {
                return Err(PatternObservationError::Contract);
            };
            let original_binding_count = goal.bindings.len();
            let original_nodes = goal.term.nodes();
            let symbol = format!(
                "Mpk.CSharp.Control.PatternStep.{}.{}",
                pattern.id, step.source_node_id
            );
            let arguments = |bindings: &[ControlBinding]| {
                bindings
                    .iter()
                    .enumerate()
                    .map(|(index, binding)| ContractTerm::Var {
                        index,
                        type_id: binding.type_id.clone(),
                    })
                    .collect()
            };
            if goal.term != apply(&symbol, arguments(&goal.bindings), BOOL)
                || !sequent.assumptions.is_empty()
                || sequent.function_id != pattern.function_id
                || sequent.source_node_id != step.entry_node_id
                || sequent.target_node_id.as_ref() != Some(&step.exit_node_id)
            {
                return Err(PatternObservationError::Contract);
            }
            let mut observation = PatternStepObservation {
                sequent_id: sequent_id.clone(),
                function_id: pattern.function_id.clone(),
                pattern_id: pattern.id.clone(),
                source_node_id: step.source_node_id.clone(),
                original_binding_count,
                operands: vec![],
                slot: None,
                source_semantics_pending: true,
            };
            for (source_input_index, source_value_id) in step.source_inputs.iter().enumerate() {
                let mut producers = graph.nodes.iter().filter(|n| &n.result == source_value_id);
                let producer = producers.next().ok_or(PatternObservationError::Contract)?;
                if producers.next().is_some() {
                    return Err(PatternObservationError::Contract);
                }
                let mut anchors = protocol
                    .anchors
                    .iter()
                    .filter(|a| a.source_node_id == producer.id);
                let anchor = anchors.next().ok_or(PatternObservationError::Contract)?;
                if anchors.next().is_some() {
                    return Err(PatternObservationError::Contract);
                }
                let value = anchor
                    .result
                    .as_ref()
                    .ok_or(PatternObservationError::Contract)?;
                observation.operands.push(PatternOperandObservation {
                    source_input_index,
                    source_value_id: source_value_id.clone(),
                    producer_source_node_id: producer.id.clone(),
                    native_value: value.clone(),
                    binding_index: goal.bindings.len(),
                });
                goal.bindings.push(bound(value, &step.entry_node_id));
            }
            if matches!(step.operation.as_str(), "load" | "store" | "pattern_bind") {
                let mut transfers = flow
                    .transfers
                    .iter()
                    .filter(|t| t.source_node_id == step.source_node_id);
                let transfer = transfers.next().ok_or(PatternObservationError::Contract)?;
                if transfers.next().is_some()
                    || transfer.kind != step.operation
                    || transfer.slot != step.source_slot
                    || transfer.entry_node_id != step.entry_node_id
                    || transfer.exit_node_id != step.exit_node_id
                    || step.result.as_ref() != Some(&transfer.value)
                {
                    return Err(PatternObservationError::Contract);
                }
                let nominal = flow
                    .slots
                    .iter()
                    .find(|(slot, _)| slot == &transfer.slot)
                    .map(|(_, ty)| ty)
                    .ok_or(PatternObservationError::Contract)?;
                let ty = storage_types.get(&transfer.slot).unwrap_or(nominal);
                if ty != &transfer.value.type_id
                    && payloads.get(ty) != Some(&transfer.value.type_id)
                {
                    return Err(PatternObservationError::Contract);
                }
                let before_assigned_index = append_slot(
                    &mut goal.bindings,
                    transfer,
                    "source_entry_assigned",
                    &transfer.entry_node_id,
                    BOOL,
                );
                let before_value_index = append_slot(
                    &mut goal.bindings,
                    transfer,
                    "source_entry_slot",
                    &transfer.entry_node_id,
                    ty,
                );
                let after_assigned_index = append_slot(
                    &mut goal.bindings,
                    transfer,
                    "source_exit_assigned",
                    &transfer.exit_node_id,
                    BOOL,
                );
                let after_value_index = append_slot(
                    &mut goal.bindings,
                    transfer,
                    "source_exit_slot",
                    &transfer.exit_node_id,
                    ty,
                );
                observation.slot = Some(PatternSlotObservation {
                    source: transfer.clone(),
                    nominal_type_id: nominal.clone(),
                    storage_type_id: ty.clone(),
                    before_assigned_index,
                    before_value_index,
                    after_assigned_index,
                    after_value_index,
                });
            }
            goal.term = apply(&symbol, arguments(&goal.bindings), BOOL);
            if goal.bindings.len() > 256 {
                return Err(PatternObservationError::Limit);
            }
            program.node_count = program
                .node_count
                .checked_add(goal.term.nodes() - original_nodes)
                .ok_or(PatternObservationError::Limit)?;
            if program.node_count > 262_144 {
                return Err(PatternObservationError::Limit);
            }
            program.pattern_observations.push(observation);
        }
    }
    // New free arguments add application/variable nodes, not new symbols or
    // lexical binders. Preserve the complete original flow/formula cost and
    // symbol inventory, including guards and entry conditions.
    Ok(program)
}

pub fn import_csharp_practical_control_vcs_with_pattern_observations(
    input: &[u8],
    vir: &ValidatedPracticalVir,
) -> Result<ControlVcProgram, PatternObservationError> {
    if input.len() > 16 * 1024 * 1024 {
        return Err(PatternObservationError::Limit);
    }
    let program = generate_csharp_practical_control_vcs_with_pattern_observations(vir)?;
    if input != program.canonical_bytes() {
        return Err(PatternObservationError::Contract);
    }
    Ok(program)
}
