//! Exact source producers of the once-evaluated governing SSA values.
//! A capture is an explicit normal-execution premise, never an axiom that the
//! producer ran. Source execution establishment and pattern proofs remain open.
use super::*;
use crate::csharp_practical_vir_model::ControlVcProgram;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternCaptureAlternative {
    pub source_execution: OrdinaryControlSourceExecution,
    /// Exact native execution arguments projected from the capture environment.
    pub argument_indices: Vec<usize>,
    pub components: Vec<OrdinaryControlStepComponent>,
    pub governing_native_argument_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternCapture {
    pub pattern_id: String,
    pub function_id: String,
    pub source_ordinal: usize,
    pub governing_source_value_id: String,
    pub producer_source_node_id: String,
    pub producer_entry_node_id: String,
    pub producer_exit_node_id: String,
    pub producer_artifact_node_ids: Vec<String>,
    pub governing_value: TypedValueRef,
    pub native_definition_point: ControlBinding,
    pub arguments: Vec<ControlBinding>,
    pub governing_argument_index: Option<usize>,
    /// Ordered normal alternatives; failed evaluation cannot start a pattern.
    pub alternatives: Vec<OrdinaryControlPatternCaptureAlternative>,
    pub excluded_exceptional_edge_ids: Vec<String>,
    pub pending_reasons: Vec<String>,
    pub definition: Option<String>,
    pub execution_establishment_pending: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternCaptureDependency {
    pub pattern_id: String,
    pub producer_source_node_id: String,
    pub definition: String,
    /// Indices in the consuming scope's arguments, in capture call order.
    pub argument_indices: Vec<usize>,
    pub governing_argument_index: usize,
}

fn argument(arguments: &mut Vec<ControlBinding>, binding: &ControlBinding) -> usize {
    if let Some(i) = arguments.iter().position(|a| a == binding) {
        i
    } else {
        arguments.push(binding.clone());
        arguments.len() - 1
    }
}

fn compile(
    b: &mut Builder,
    depths: &[u32],
    alternatives: &[OrdinaryControlPatternCaptureAlternative],
    id: &str,
) -> R<Option<String>> {
    if depths.len() + depths.iter().copied().max().unwrap_or(0) as usize > 256 {
        return Ok(None);
    }
    let mut body = bit(b, false)?;
    for alternative in alternatives {
        if alternative.components.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let mut normal = bit(b, true)?;
        for component in &alternative.components {
            let arguments = component
                .argument_indices
                .iter()
                .map(|&i| {
                    if i >= depths.len() {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    b.var((depths.len() - 1 - i) as u32)
                })
                .collect::<R<Vec<_>>>()?;
            let value = call(b, &component.definition, arguments)?;
            normal = call(b, "Std.Bool.and", vec![normal, value])?;
        }
        body = call(b, "Std.Bool.or", vec![body, normal])?;
    }
    define(b, id, depths, 0, body)?;
    Ok(Some(id.into()))
}

pub(super) fn emit(
    c: &mut Clauses<'_>,
    vir: &ValidatedPracticalVir,
    control: &ControlVcProgram,
    native: &OrdinaryControlEdgeProgram,
) -> R<Vec<OrdinaryControlPatternCapture>> {
    let mut captures = vec![];
    for pattern in control.patterns() {
        let function = vir
            .functions()
            .iter()
            .find(|f| f.id == pattern.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let protocol = function
            .control_protocol
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let flow = native
            .functions()
            .iter()
            .find(|f| f.source.function_id == pattern.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let graph = flow
            .source
            .source_graph
            .as_ref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let mut decisions = graph.nodes.iter().filter(|n| {
            n.kind == "pattern_decision" && n.source_ordinal == Some(pattern.source_ordinal)
        });
        let decision = decisions.next().ok_or(OrdinaryCarrierError::Linkage)?;
        if decisions.next().is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let governing_source_value_id = decision
            .inputs
            .first()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let mut producers = graph
            .nodes
            .iter()
            .filter(|n| &n.result == governing_source_value_id);
        let producer = producers.next().ok_or(OrdinaryCarrierError::Linkage)?;
        if producers.next().is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let anchor = protocol
            .anchors
            .iter()
            .find(|a| a.source_node_id == producer.id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if anchor.result.as_ref() != Some(&pattern.governing_value) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let values = control_edges::value_definition_points(function)?;
        let point = values
            .get(&pattern.governing_value.id)
            .filter(|a| a.type_id == pattern.governing_value.type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let mut capture = OrdinaryControlPatternCapture {
            pattern_id: pattern.id.clone(),
            function_id: pattern.function_id.clone(),
            source_ordinal: pattern.source_ordinal,
            governing_source_value_id: governing_source_value_id.clone(),
            producer_source_node_id: producer.id.clone(),
            producer_entry_node_id: anchor.entry_node_id.clone(),
            producer_exit_node_id: anchor.exit_node_id.clone(),
            producer_artifact_node_ids: anchor.artifact_node_ids.clone(),
            governing_value: pattern.governing_value.clone(),
            native_definition_point: point.clone(),
            arguments: vec![],
            governing_argument_index: None,
            alternatives: vec![],
            excluded_exceptional_edge_ids: vec![],
            pending_reasons: vec![],
            definition: None,
            execution_establishment_pending: true,
        };
        for execution in flow
            .source_executions
            .iter()
            .filter(|e| e.source_node_id == producer.id)
        {
            let edge = flow
                .edges
                .iter()
                .find(|e| e.source.id == execution.edge_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if edge.source.kind != "normal" {
                capture
                    .excluded_exceptional_edge_ids
                    .push(execution.edge_id.clone());
                continue;
            }
            let indices = execution
                .arguments
                .iter()
                .map(|a| argument(&mut capture.arguments, a))
                .collect::<Vec<_>>();
            let governing = execution.arguments.iter().position(|a| a == point);
            if let Some(i) = governing {
                if capture
                    .governing_argument_index
                    .replace(indices[i])
                    .is_some_and(|previous| previous != indices[i])
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
            } else {
                capture
                    .pending_reasons
                    .push("governing_value_absent_from_exact_producer_execution".into());
            }
            let components = if let Some(definition) = &execution.definition {
                vec![OrdinaryControlStepComponent {
                    role: "native_governing_producer_execution".into(),
                    definition: definition.clone(),
                    argument_indices: indices.clone(),
                }]
            } else {
                execution
                    .components
                    .iter()
                    .map(|component| {
                        Ok(OrdinaryControlStepComponent {
                            role: component.role.clone(),
                            definition: component.definition.clone(),
                            argument_indices: component
                                .argument_indices
                                .iter()
                                .map(|&i| {
                                    indices.get(i).copied().ok_or(OrdinaryCarrierError::Linkage)
                                })
                                .collect::<R<Vec<_>>>()?,
                        })
                    })
                    .collect::<R<Vec<_>>>()?
            };
            capture
                .alternatives
                .push(OrdinaryControlPatternCaptureAlternative {
                    source_execution: execution.clone(),
                    argument_indices: indices,
                    components,
                    governing_native_argument_index: governing,
                });
        }
        if capture.alternatives.is_empty() {
            capture
                .pending_reasons
                .push("governing_producer_has_no_normal_execution".into());
        }
        if capture.pending_reasons.is_empty() {
            let depths = capture
                .arguments
                .iter()
                .map(|a| {
                    c.carriers
                        .get(a.type_id.as_str())
                        .map(|layout| layout.depth)
                        .ok_or(OrdinaryCarrierError::Linkage)
                })
                .collect::<R<Vec<_>>>()?;
            let id = name("PatternGoverningCapture", &(vir.hash(), &capture));
            capture.definition = compile(&mut c.b, &depths, &capture.alternatives, &id)?;
            if capture.definition.is_none() {
                capture.pending_reasons.push("combined_binder_limit".into());
            }
        }
        captures.push(capture);
    }
    Ok(captures)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;

    #[test]
    fn normal_alternatives_are_disjunctive_with_complete_premises() {
        let mut b = Builder::new().unwrap();
        let equality = construction_data::physical_equal(&mut b, 0).unwrap();
        let source = OrdinaryControlSourceExecution {
            source_node_id: "producer".into(),
            entry_node_id: "entry".into(),
            exit_node_id: "exit".into(),
            edge_id: "normal".into(),
            target_node_id: None,
            arguments: vec![],
            components: vec![],
            argument_count: 4,
            component_count: 2,
            component_map_sha256: "test".into(),
            state_rule: "governing_normal_evaluation".into(),
            definition: None,
        };
        let alternatives = [(vec![0, 1], vec![2, 3]), (vec![0, 2], vec![1, 3])]
            .into_iter()
            .map(|(first, second)| OrdinaryControlPatternCaptureAlternative {
                source_execution: source.clone(),
                argument_indices: (0..4).collect(),
                components: vec![
                    OrdinaryControlStepComponent {
                        role: "first_premise".into(),
                        definition: equality.clone(),
                        argument_indices: first,
                    },
                    OrdinaryControlStepComponent {
                        role: "second_premise".into(),
                        definition: equality.clone(),
                        argument_indices: second,
                    },
                ],
                governing_native_argument_index: Some(0),
            })
            .collect::<Vec<_>>();
        let definition = compile(&mut b, &[0; 4], &alternatives, "Mpk.Capture.Test")
            .unwrap()
            .unwrap();
        let certificate = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for bits in 0..16 {
            let values = (0..4).map(|i| bits & (1 << i) != 0).collect::<Vec<_>>();
            let expected = (values[0] == values[1] && values[2] == values[3])
                || (values[0] == values[2] && values[1] == values[3]);
            assert_eq!(
                observed(run(
                    &certificate,
                    &definition,
                    values.into_iter().map(V::Bit).collect()
                )),
                expected,
            );
        }
        let mut b = Builder::new().unwrap();
        assert!(compile(
            &mut b,
            &[253, 0, 0, 0],
            &alternatives,
            "Mpk.Capture.TooDeep"
        )
        .unwrap()
        .is_none());
        let mut invalid = alternatives;
        invalid[0].components[0].argument_indices[0] = 4;
        assert!(matches!(
            compile(&mut b, &[0; 4], &invalid, "Mpk.Capture.Invalid"),
            Err(OrdinaryCarrierError::Linkage)
        ));
    }
}
