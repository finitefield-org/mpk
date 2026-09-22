//! Source normal steps compose existing value/effect relations with one shared
//! SSA environment. Entry selection, exceptional steps and whole-body proofs
//! remain separate obligations; an incomplete step has no execution predicate.
use super::*;
use crate::csharp_practical_vir_validation::{PracticalControlAnchor, PracticalVirFunction};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlStepComponent {
    pub role: String,
    pub definition: String,
    /// Indices in the step's shared arguments, in the original relation order.
    pub argument_indices: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlSourceStep {
    pub source_node_id: String,
    pub native_node_ids: Vec<String>,
    /// SSA arguments name their actual definition points. Source entry and exit
    /// slots remain different snapshots even when their native node IDs coincide.
    pub arguments: Vec<ControlBinding>,
    pub components: Vec<OrdinaryControlStepComponent>,
    pub entry_phi_arguments: Vec<usize>,
    pub source_result_argument: Option<usize>,
    pub condition_argument: Option<usize>,
    pub return_arguments: Vec<usize>,
    pub pending_reasons: Vec<String>,
    /// Conjunction of every component, never guarded implication. If the shared
    /// composed binder/type depth exceeds 256, all components must hold at their
    /// indexed inputs.
    /// Components are executable only when pending_reasons is empty.
    pub definition: Option<String>,
}

type Values = BTreeMap<String, ControlBinding>;
fn values(native: &PracticalVirFunction) -> R<Values> {
    let entry = &native
        .blocks
        .first()
        .ok_or(OrdinaryCarrierError::Linkage)?
        .node
        .id;
    let mut values = BTreeMap::new();
    let mut insert = |node: &str, value: &TypedValueRef| -> R<()> {
        let binding = ControlBinding {
            kind: "ssa".into(),
            edge_id: None,
            node_id: node.into(),
            value_id: value.id.clone(),
            type_id: value.type_id.clone(),
        };
        if values.insert(value.id.clone(), binding).is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        Ok(())
    };
    for value in &native.parameter_values {
        insert(entry, value)?;
    }
    for block in &native.blocks {
        for phi in &block.phi_values {
            insert(&block.node.id, &phi.value)?;
        }
        for literal in &block.literal_values {
            insert(&block.node.id, &literal.result)?;
        }
        if let Some(invocation) = &block.invocation {
            insert(&invocation.normal_successor_id, &invocation.result)?;
        }
        if let Some(exception) = &block.handler_exception_value {
            insert(&block.node.id, exception)?;
        }
    }
    Ok(values)
}
impl OrdinaryControlSourceStep {
    fn argument(&mut self, binding: ControlBinding) -> usize {
        if let Some(i) = self.arguments.iter().position(|a| a == &binding) {
            return i;
        }
        self.arguments.push(binding);
        self.arguments.len() - 1
    }
    fn ssa(&mut self, values: &Values, id: &str, ty: Option<&str>) -> R<usize> {
        let binding = values.get(id).ok_or(OrdinaryCarrierError::Linkage)?;
        if ty.is_some_and(|ty| ty != binding.type_id) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        Ok(self.argument(binding.clone()))
    }
    fn component(&mut self, role: &str, definition: &str, indices: Vec<usize>) {
        self.components.push(OrdinaryControlStepComponent {
            role: role.into(),
            definition: definition.into(),
            argument_indices: indices,
        });
    }
}

fn normal_path(
    native: &PracticalVirFunction,
    anchor: &PracticalControlAnchor,
) -> R<Option<Vec<String>>> {
    let owned = anchor.artifact_node_ids.iter().collect::<BTreeSet<_>>();
    if owned.len() != anchor.artifact_node_ids.len()
        || !owned.contains(&anchor.entry_node_id)
        || !owned.contains(&anchor.exit_node_id)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut path = vec![];
    let mut node = &anchor.entry_node_id;
    loop {
        if !owned.contains(node) || path.contains(node) {
            return Ok(None);
        }
        path.push(node.clone());
        let block = native
            .blocks
            .iter()
            .find(|b| &b.node.id == node)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if node == &anchor.exit_node_id {
            break;
        }
        let [next] = block.node.normal_successor_ids.as_slice() else {
            return Ok(None);
        };
        node = next;
    }
    Ok((path.len() == owned.len()).then_some(path))
}

pub(super) fn append(
    c: &mut Clauses<'_>,
    p: &mut OrdinaryControlEdgeProgram,
    vir: &ValidatedPracticalVir,
) -> R<()> {
    for flow in &mut p.functions {
        let Some(graph) = &flow.source.source_graph else {
            continue;
        };
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let protocol = native
            .control_protocol
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let values = values(native)?;
        for node in &graph.nodes {
            let mut step = OrdinaryControlSourceStep {
                source_node_id: node.id.clone(),
                native_node_ids: vec![],
                arguments: vec![],
                components: vec![],
                entry_phi_arguments: vec![],
                source_result_argument: None,
                condition_argument: None,
                return_arguments: vec![],
                pending_reasons: vec![],
                definition: None,
            };
            let frame = flow
                .source_frames
                .iter()
                .find(|f| f.source_node_id == node.id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if let Some(reason) = &frame.pending_reason {
                step.pending_reasons.push(reason.clone());
                flow.source_steps.push(step);
                continue;
            }
            let anchor = protocol
                .anchors
                .iter()
                .find(|a| a.source_node_id == node.id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let Some(path) = normal_path(native, anchor)? else {
                step.pending_reasons
                    .push("native_internal_path_selection_pending".into());
                flow.source_steps.push(step);
                continue;
            };
            step.native_node_ids = path;
            let indices = frame
                .arguments
                .iter()
                .map(|a| step.argument(a.clone()))
                .collect();
            step.component(
                "source_frame",
                frame
                    .definition
                    .as_ref()
                    .ok_or(OrdinaryCarrierError::Linkage)?,
                indices,
            );
            if node.operation == "update" {
                let mut effects = flow
                    .memory_effects
                    .iter()
                    .filter(|effect| effect.source_node_id == node.id);
                let effect = effects.next().ok_or(OrdinaryCarrierError::Linkage)?;
                if effects.next().is_some() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let indices = effect
                    .arguments
                    .iter()
                    .map(|argument| match argument.kind.as_str() {
                        "ssa" => step.ssa(&values, &argument.value_id, Some(&argument.type_id)),
                        "source_before_assigned" => Ok(step.argument(ControlBinding {
                            kind: "source_entry_assigned".into(),
                            edge_id: None,
                            node_id: argument.node_id.clone(),
                            value_id: argument.value_id.clone(),
                            type_id: argument.type_id.clone(),
                        })),
                        "source_before_slot" => Ok(step.argument(ControlBinding {
                            kind: "source_entry_slot".into(),
                            edge_id: None,
                            node_id: argument.node_id.clone(),
                            value_id: argument.value_id.clone(),
                            type_id: argument.type_id.clone(),
                        })),
                        "source_after_assigned" => Ok(step.argument(ControlBinding {
                            kind: "source_exit_assigned".into(),
                            edge_id: None,
                            node_id: argument.node_id.clone(),
                            value_id: argument.value_id.clone(),
                            type_id: argument.type_id.clone(),
                        })),
                        "source_after_slot" => Ok(step.argument(ControlBinding {
                            kind: "source_exit_slot".into(),
                            edge_id: None,
                            node_id: argument.node_id.clone(),
                            value_id: argument.value_id.clone(),
                            type_id: argument.type_id.clone(),
                        })),
                        _ => Err(OrdinaryCarrierError::Linkage),
                    })
                    .collect::<R<Vec<_>>>()?;
                step.component("memory_effect", &effect.definition, indices);
            }
            if let Some(transfer) = &frame.transfer {
                let relation = flow
                    .slot_relations
                    .iter()
                    .find(|r| r.transfer.as_ref() == Some(transfer))
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let mut indices = vec![];
                for a in &relation.arguments {
                    let i = if a.role == "ssa" {
                        step.ssa(
                            &values,
                            a.value_id.as_deref().ok_or(OrdinaryCarrierError::Linkage)?,
                            Some(&a.type_id),
                        )?
                    } else {
                        let kind = match a.role.as_str() {
                            "before_assigned" => "source_entry_assigned",
                            "before_value" => "source_entry_slot",
                            "after_assigned" => "source_exit_assigned",
                            "after_value" => "source_exit_slot",
                            _ => return Err(OrdinaryCarrierError::Linkage),
                        };
                        step.argument(ControlBinding {
                            kind: kind.into(),
                            edge_id: None,
                            node_id: a.node_id.clone(),
                            value_id: a.slot_id.clone(),
                            type_id: a.type_id.clone(),
                        })
                    };
                    indices.push(i);
                }
                step.component("slot_transfer", &relation.definition, indices);
            }
            for (position, id) in step.native_node_ids.clone().iter().enumerate() {
                let block = native
                    .blocks
                    .iter()
                    .find(|b| &b.node.id == id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if position != 0 && !block.phi_values.is_empty() {
                    step.pending_reasons
                        .push("internal_phi_selection_pending".into());
                }
                if position == 0 {
                    for phi in &block.phi_values {
                        let i = step.ssa(&values, &phi.value.id, Some(&phi.value.type_id))?;
                        step.entry_phi_arguments.push(i);
                    }
                }
                for literal in &block.literal_values {
                    let relation = flow
                        .native_literals
                        .iter()
                        .find(|l| l.result.node_id == *id && l.source == *literal)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let i = step.ssa(&values, &literal.result.id, Some(&literal.result.type_id))?;
                    step.component("literal_result", &relation.definition, vec![i]);
                }
                if let Some(invocation) = &block.invocation {
                    if !step
                        .native_node_ids
                        .contains(&invocation.normal_successor_id)
                    {
                        step.pending_reasons
                            .push("normal_result_outside_source_anchor".into());
                    }
                    if let Some(operation) = flow
                        .native_operations
                        .iter()
                        .find(|o| o.source.node_id == *id)
                    {
                        if operation.invocation != *invocation {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        if let Some(definition) = operation.predicates.get("normal_execution") {
                            let indices = operation
                                .arguments
                                .iter()
                                .map(|a| step.ssa(&values, &a.value_id, Some(&a.type_id)))
                                .collect::<R<Vec<_>>>()?;
                            step.component("native_normal", definition, indices);
                        } else {
                            step.pending_reasons.push("native_semantics_pending".into());
                        }
                    } else {
                        if node.operation != "construct" {
                            step.pending_reasons
                                .push("native_call_semantics_pending".into());
                            continue;
                        }
                        let signature =
                            crate::csharp_practical_vir_model::source_value_constructor_operation(
                                vir.construction_context().1,
                                vir.data_closed(),
                                &invocation.result.type_id,
                            )
                            .map_err(|_| OrdinaryCarrierError::Linkage)?;
                        let constructor = vir
                            .functions()
                            .iter()
                            .find(|function| function.id == invocation.operation_id)
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        if invocation.operands.len() != signature.argument_type_ids.len()
                            || node.inputs.len() != signature.argument_type_ids.len()
                            || node.result.is_empty()
                            || invocation
                                .operands
                                .iter()
                                .map(|operand| operand.type_id.as_str())
                                .ne(signature.argument_type_ids.iter().map(String::as_str))
                            || anchor.result.as_ref() != Some(&invocation.result)
                            || constructor
                                .parameter_values
                                .iter()
                                .map(|parameter| parameter.type_id.as_str())
                                .ne(signature.argument_type_ids.iter().map(String::as_str))
                            || constructor.result_type_ids
                                != [signature.normal_result_type_id.clone()]
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        let definition = crate::csharp_practical_vir_model::data_vc::DataSemanticDefinition {
                            id: format!("source.constructor.{}", node.id),
                            family: crate::csharp_practical_vir_model::data_vc::DataDefinitionFamily::SourceValue,
                            signature,
                            foundation_equation: None,
                            structural_recipes: serde_json::Value::Null,
                            carrier_definitions: vec![],
                            relation_name: format!("{PREFIX}.SourceConstructor.{}", node.id),
                            failure_names: vec![],
                            failure_result_names: vec![],
                        };
                        let relation = source_value_data::emit(c, &definition)?;
                        let mut indices = invocation
                            .operands
                            .iter()
                            .map(|operand| step.ssa(&values, &operand.id, Some(&operand.type_id)))
                            .collect::<R<Vec<_>>>()?;
                        indices.push(step.ssa(
                            &values,
                            &invocation.result.id,
                            Some(&invocation.result.type_id),
                        )?);
                        step.component(
                            "source_constructor_normal",
                            &relation.relation_definition,
                            indices,
                        );
                        let pending_position = flow
                            .pending_native_invocation_node_ids
                            .iter()
                            .position(|id| id == &block.node.id)
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        flow.pending_native_invocation_node_ids
                            .remove(pending_position);
                        if flow
                            .source_constructor_invocation_node_ids
                            .iter()
                            .any(|id| id == &block.node.id)
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        flow.source_constructor_invocation_node_ids
                            .push(block.node.id.clone());
                    }
                }
                if matches!(
                    block.node.tag,
                    ControlNodeTag::Throw
                        | ControlNodeTag::Rethrow
                        | ControlNodeTag::HandlerEntry
                        | ControlNodeTag::FinallyEntry
                        | ControlNodeTag::FinallyExit
                ) {
                    step.pending_reasons
                        .push("exception_execution_pending".into());
                }
            }
            if let Some(result) = &anchor.result {
                step.source_result_argument =
                    Some(step.ssa(&values, &result.id, Some(&result.type_id))?);
            }
            let exit = native
                .blocks
                .iter()
                .find(|b| b.node.id == anchor.exit_node_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if let Some(id) = &exit.condition_value_id {
                step.condition_argument = Some(step.ssa(&values, id, Some(SOURCE_BOOL))?);
            }
            for id in &exit.return_value_ids {
                let i = step.ssa(&values, id, None)?;
                step.return_arguments.push(i);
            }
            if step.pending_reasons.is_empty() {
                let depths = step
                    .arguments
                    .iter()
                    .map(|a| {
                        c.carriers
                            .get(a.type_id.as_str())
                            .map(|c| c.depth)
                            .ok_or(OrdinaryCarrierError::Linkage)
                    })
                    .collect::<R<Vec<_>>>()?;
                let definition = name("SourceNormalStep", &(&flow.source.function_id, &step));
                if emit_compact(&mut c.b, &definition, &depths, &step.components)? {
                    step.definition = Some(definition);
                }
            }
            flow.source_steps.push(step);
        }
    }
    Ok(())
}

/// A late cube type is nested beneath every preceding formal binder. Counting
/// only the number of formals would reject otherwise valid factored steps when
/// define() constructs an overdeep Lam/Pi. The component calls themselves have
/// depth zero: they apply global constants to shared variables.
fn emit_compact(
    b: &mut Builder,
    definition: &str,
    depths: &[u32],
    components: &[OrdinaryControlStepComponent],
) -> R<bool> {
    if depths.len() > 256
        || depths
            .iter()
            .enumerate()
            .any(|(i, &depth)| depth as usize > 256 - i)
    {
        return Ok(false);
    }
    let terms = (0..depths.len())
        .map(|i| b.var((depths.len() - 1 - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let mut body = bit(b, true)?;
    for component in components {
        let part = call(
            b,
            &component.definition,
            component
                .argument_indices
                .iter()
                .map(|&i| terms[i])
                .collect(),
        )?;
        body = call(b, "Std.Bool.and", vec![body, part])?;
    }
    define(b, definition, depths, 0, body)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_control_source_step_composed_binder_boundary() {
        let mut b = Builder::new().unwrap();
        let yes = bit(&mut b, true).unwrap();
        define(&mut b, "Test.DeepComponent", &[253], 0, yes).unwrap();
        let component = |index| OrdinaryControlStepComponent {
            role: "test".into(),
            definition: "Test.DeepComponent".into(),
            argument_indices: vec![index],
        };
        // Three earlier formals + the deep input type exactly reach the limit.
        assert!(emit_compact(&mut b, "Test.AtLimit", &[0, 0, 0, 253], &[component(3)],).unwrap());
        let declaration = b.c.declarations.last().unwrap();
        let DeclarationKind::Def { ty, value, .. } = declaration.kind else {
            panic!("expected the compact step definition");
        };
        assert_eq!(b.binders[ty as usize], 256);
        assert_eq!(b.binders[value as usize], 256);
        let term_count = b.c.term_table.len();
        let declaration_count = b.c.declarations.len();
        // One more earlier formal must keep the component representation. It
        // must not attempt construction or leave partially emitted terms behind.
        assert!(
            !emit_compact(&mut b, "Test.Factored", &[0, 0, 0, 0, 253], &[component(4)],).unwrap()
        );
        assert!(!emit_compact(&mut b, "Test.ManyFormals", &[0; 257], &[]).unwrap());
        assert_eq!(b.c.term_table.len(), term_count);
        assert_eq!(b.c.declarations.len(), declaration_count);
        // Independently exercise the actual Lam/Pi builder at the same boundary.
        assert!(matches!(
            define(&mut b, "Test.TooDeep", &[0, 0, 0, 0, 253], 0, yes),
            Err(OrdinaryCarrierError::Limit)
        ));
    }
}
