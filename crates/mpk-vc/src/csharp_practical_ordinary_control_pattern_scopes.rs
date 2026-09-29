//! Exact native premises and observation transport for original W04 patterns.
//! These scopes do not define the still-pending PatternStep predicates or prove
//! that an application execution establishes a scope.
use super::*;
use crate::csharp_practical_vir_model::{ControlVcProgram, PatternStepVc};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternObservation {
    pub source: ControlBinding,
    /// A single SSA definition in this exact native function. The original
    /// observation point is retained separately, including before/after points.
    pub native_definition_point: ControlBinding,
    pub observation_argument_index: usize,
    pub native_argument_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternExecutionScope {
    pub source_execution: OrdinaryControlSourceExecution,
    pub edge_kind: String,
    pub arguments: Vec<ControlBinding>,
    pub native_argument_indices: Vec<usize>,
    pub observations: Vec<OrdinaryControlPatternObservation>,
    pub components: Vec<OrdinaryControlStepComponent>,
    /// No available native argument is invented for an observation. A caller
    /// must separately establish missing observation/producers before proof use.
    pub pending_observation_binding_indices: Vec<usize>,
    pub pending_definition_reasons: Vec<String>,
    pub definition: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternScope {
    pub source_sequent_id: String,
    pub function_id: String,
    pub pattern_id: String,
    pub source_step: PatternStepVc,
    /// Ordered source-edge alternatives; normal and exceptional paths retain
    /// different premises, arguments and outgoing snapshots.
    pub executions: Vec<OrdinaryControlPatternExecutionScope>,
    pub excluded_unreachable: bool,
    pub original_pattern_predicate_pending: bool,
}

fn argument(arguments: &mut Vec<ControlBinding>, binding: ControlBinding) -> usize {
    if let Some(i) = arguments.iter().position(|a| a == &binding) {
        i
    } else {
        arguments.push(binding);
        arguments.len() - 1
    }
}

fn finish(
    b: &mut Builder,
    p: &mut OrdinaryControlPatternExecutionScope,
    depths: &[u32],
    id: &str,
) -> R<()> {
    if depths.len() != p.arguments.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    // The actual bound includes a carrier's selector lambdas beneath the
    // shared argument binders. Retain components if composition exceeds it.
    if p.arguments.len() + depths.iter().copied().max().unwrap_or(0) as usize > 256 {
        p.pending_definition_reasons
            .push("combined_binder_limit".into());
        return Ok(());
    }
    let mut body = bit(b, true)?;
    for component in &p.components {
        let args = component
            .argument_indices
            .iter()
            .map(|&i| {
                if i >= p.arguments.len() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                b.var((p.arguments.len() - 1 - i) as u32)
            })
            .collect::<R<Vec<_>>>()?;
        let value = call(b, &component.definition, args)?;
        body = call(b, "Std.Bool.and", vec![body, value])?;
    }
    define(b, id, depths, 0, body)?;
    p.definition = Some(id.into());
    Ok(())
}

pub(super) fn emit(
    c: &mut Clauses<'_>,
    vir: &ValidatedPracticalVir,
    control: &ControlVcProgram,
    native: &OrdinaryControlEdgeProgram,
) -> R<Vec<OrdinaryControlPatternScope>> {
    let mut scopes = vec![];
    for pattern in control.patterns() {
        let function = vir
            .functions()
            .iter()
            .find(|f| f.id == pattern.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let values = control_edges::value_definition_points(function)?;
        let flow = native
            .functions()
            .iter()
            .find(|f| f.source.function_id == pattern.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for step in &pattern.steps {
            let source_sequent_id = format!("{}.{}", pattern.id, step.source_node_id);
            let sequent = control
                .sequents()
                .iter()
                .find(|s| s.id == source_sequent_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let [goal] = sequent.goals.as_slice() else {
                return Err(OrdinaryCarrierError::Linkage);
            };
            if sequent.function_id != pattern.function_id
                || sequent.source_node_id != step.entry_node_id
                || sequent.target_node_id.as_ref() != Some(&step.exit_node_id)
                || !sequent.assumptions.is_empty()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let mut scope = OrdinaryControlPatternScope {
                source_sequent_id,
                function_id: pattern.function_id.clone(),
                pattern_id: pattern.id.clone(),
                source_step: step.clone(),
                executions: vec![],
                excluded_unreachable: flow
                    .excluded_unreachable_source_node_ids
                    .contains(&step.source_node_id),
                original_pattern_predicate_pending: true,
            };
            for execution in flow
                .source_executions
                .iter()
                .filter(|e| e.source_node_id == step.source_node_id)
            {
                let edge = flow
                    .edges
                    .iter()
                    .find(|e| e.source.id == execution.edge_id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let mut p = OrdinaryControlPatternExecutionScope {
                    source_execution: execution.clone(),
                    edge_kind: edge.source.kind.clone(),
                    arguments: execution.arguments.clone(),
                    native_argument_indices: (0..execution.arguments.len()).collect(),
                    observations: vec![],
                    components: vec![],
                    pending_observation_binding_indices: vec![],
                    pending_definition_reasons: vec![],
                    definition: None,
                };
                if let Some(definition) = &execution.definition {
                    p.components.push(OrdinaryControlStepComponent {
                        role: "native_source_execution".into(),
                        definition: definition.clone(),
                        argument_indices: p.native_argument_indices.clone(),
                    });
                } else if !execution.components.is_empty() {
                    p.components = execution.components.clone();
                } else {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                for (i, source) in goal.bindings.iter().enumerate() {
                    let point = values
                        .get(&source.value_id)
                        .filter(|p| source.kind == "ssa" && p.type_id == source.type_id)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let native_index = execution.arguments.iter().position(|a| a == point);
                    let observed = argument(&mut p.arguments, source.clone());
                    p.observations.push(OrdinaryControlPatternObservation {
                        source: source.clone(),
                        native_definition_point: point.clone(),
                        observation_argument_index: observed,
                        native_argument_index: native_index,
                    });
                    if let Some(native_index) = native_index {
                        let depth = c
                            .carriers
                            .get(source.type_id.as_str())
                            .ok_or(OrdinaryCarrierError::Linkage)?
                            .depth;
                        // Transport identical physical observations, including
                        // NaN payloads and inactive/padding bits. Semantic value
                        // equality is not SSA observation transport.
                        let definition = construction_data::physical_equal(&mut c.b, depth)?;
                        p.components.push(OrdinaryControlStepComponent {
                            role: format!("observation_transport:{i}"),
                            definition,
                            argument_indices: vec![observed, native_index],
                        });
                    } else {
                        p.pending_observation_binding_indices.push(i);
                    }
                }
                let id = name(
                    "PatternExecutionScope",
                    &(
                        vir.hash(),
                        &sequent.id,
                        &execution.edge_id,
                        &execution.component_map_sha256,
                    ),
                );
                let depths = p
                    .arguments
                    .iter()
                    .map(|a| {
                        c.carriers
                            .get(a.type_id.as_str())
                            .map(|layout| layout.depth)
                            .ok_or(OrdinaryCarrierError::Linkage)
                    })
                    .collect::<R<Vec<_>>>()?;
                finish(&mut c.b, &mut p, &depths, &id)?;
                scope.executions.push(p);
            }
            if scope.executions.is_empty() && !scope.excluded_unreachable {
                return Err(OrdinaryCarrierError::Linkage);
            }
            scopes.push(scope);
        }
    }
    Ok(scopes)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;

    #[test]
    fn native_premise_and_observation_transport_both_required() {
        let mut b = Builder::new().unwrap();
        let equality = construction_data::physical_equal(&mut b, 0).unwrap();
        let bindings = (0..4)
            .map(|i| ControlBinding {
                kind: "ssa".into(),
                edge_id: None,
                node_id: format!("point.{i}"),
                value_id: format!("value.{}", i % 2),
                type_id: SOURCE_BOOL.into(),
            })
            .collect::<Vec<_>>();
        let source_execution = OrdinaryControlSourceExecution {
            source_node_id: "source".into(),
            entry_node_id: "entry".into(),
            exit_node_id: "exit".into(),
            edge_id: "edge".into(),
            target_node_id: None,
            arguments: bindings[..2].to_vec(),
            components: vec![],
            argument_count: 2,
            component_count: 1,
            component_map_sha256: "test".into(),
            state_rule: "identity_result".into(),
            definition: Some(equality.clone()),
        };
        let mut scope = OrdinaryControlPatternExecutionScope {
            source_execution,
            edge_kind: "normal".into(),
            arguments: bindings,
            native_argument_indices: vec![0, 1],
            observations: vec![],
            components: vec![
                OrdinaryControlStepComponent {
                    role: "native_source_execution".into(),
                    definition: equality.clone(),
                    argument_indices: vec![0, 1],
                },
                OrdinaryControlStepComponent {
                    role: "observation_transport:0".into(),
                    definition: equality.clone(),
                    argument_indices: vec![2, 0],
                },
                OrdinaryControlStepComponent {
                    role: "observation_transport:1".into(),
                    definition: equality,
                    argument_indices: vec![3, 1],
                },
            ],
            pending_observation_binding_indices: vec![],
            pending_definition_reasons: vec![],
            definition: None,
        };
        finish(&mut b, &mut scope, &[0; 4], "Mpk.PatternScope.Test").unwrap();
        let certificate = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for bits in 0..16 {
            let values = (0..4).map(|i| bits & (1 << i) != 0).collect::<Vec<_>>();
            let expected =
                values[0] == values[1] && values[2] == values[0] && values[3] == values[1];
            assert_eq!(
                observed(run(
                    &certificate,
                    scope.definition.as_ref().unwrap(),
                    values.into_iter().map(V::Bit).collect()
                )),
                expected
            );
        }
        let mut b = Builder::new().unwrap();
        scope.definition = None;
        finish(
            &mut b,
            &mut scope,
            &[253, 0, 0, 0],
            "Mpk.PatternScope.TooDeep",
        )
        .unwrap();
        assert!(scope.definition.is_none());
        assert_eq!(scope.pending_definition_reasons, ["combined_binder_limit"]);
        assert_eq!(scope.components.len(), 3);
    }
}
