//! Exact native premises and observation transport for original W04 patterns.
//! These scopes do not define the still-pending PatternStep predicates or prove
//! that an application execution establishes a scope.
use super::*;
use crate::csharp_practical_vir_model::{ControlVcProgram, PatternStepVc};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternObservation {
    pub source: ControlBinding,
    /// The exact SSA definition or source slot phase in this native function.
    /// The original observation point is retained separately.
    pub native_definition_point: ControlBinding,
    pub observation_argument_index: usize,
    pub native_argument_index: Option<usize>,
    /// Exact matched producer argument, under the explicit capture premise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_argument_index: Option<usize>,
    /// Independently represented pure source literal when native lowering erases it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_literal_definition: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternRouteObservation {
    pub source: ControlBinding,
    pub observation_argument_index: usize,
    pub successor_source_node_id: String,
    pub native_edge_id: String,
    pub native_target_node_id: Option<String>,
    /// Conditional on the explicit native source-execution premise below.
    pub selected: bool,
    pub definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternExecutionScope {
    pub source_execution: OrdinaryControlSourceExecution,
    pub edge_kind: String,
    pub arguments: Vec<ControlBinding>,
    pub native_argument_indices: Vec<usize>,
    pub observations: Vec<OrdinaryControlPatternObservation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub route_observations: Vec<OrdinaryControlPatternRouteObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_dependency: Option<OrdinaryControlPatternCaptureDependency>,
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

fn route_selection(
    flow: &OrdinaryControlEdgeFunction,
    edge: &control_edges::OrdinaryControlEdgeDefinition,
    observation: &crate::csharp_practical_vir_model::PatternStepObservation,
) -> R<usize> {
    let route = observation
        .route
        .as_ref()
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if route.source_kind == "builtin_throw" {
        let exception = route
            .closed_exception_type
            .as_deref()
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if route.binding_indices.len() != 1
            || edge.builtin_throw_source_node_id.as_deref()
                != Some(observation.source_node_id.as_str())
            || edge.source.kind != "exception"
            || edge.source.check_id.as_deref()
                != Some(format!("exception.closed.{exception}").as_str())
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        return Ok(0);
    }
    if edge.source.kind != "normal" {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let branch = route.source_kind == "branch";
    let guard_index = if branch {
        let [input] = observation.operands.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let [binding] = edge.source.guard.bindings.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if binding.kind != "ssa"
            || binding.value_id != input.native_value.id
            || binding.type_id != SOURCE_BOOL
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let variable = ContractTerm::Var {
            index: 0,
            type_id: SOURCE_BOOL.into(),
        };
        if edge.source.guard.term == variable {
            Some(0)
        } else if edge.source.guard.term
            == (ContractTerm::App {
                function: Box::new(ContractTerm::Const {
                    name: "Mpk.CSharp.Bool.Not".into(),
                    type_id: format!("({SOURCE_BOOL}->{SOURCE_BOOL})"),
                }),
                argument: Box::new(variable),
                type_id: SOURCE_BOOL.into(),
            })
        {
            Some(1)
        } else {
            return Err(OrdinaryCarrierError::Linkage);
        }
    } else {
        None
    };
    let mut matches =
        route
            .successor_source_node_ids
            .iter()
            .enumerate()
            .filter(|(index, source)| {
                guard_index.is_none_or(|selected| selected == *index)
                    && flow.source_frames.iter().any(|frame| {
                        frame.source_node_id == **source
                            && frame.entry_node_id.is_some()
                            && frame.entry_node_id == edge.source.target_node_id
                    })
            });
    let (index, _) = matches.next().ok_or(OrdinaryCarrierError::Linkage)?;
    if matches.next().is_some() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(index)
}

fn fixed_observation(b: &mut Builder, selected: bool) -> R<String> {
    let name = name("PatternPathObservedSelection", &selected);
    if !b.globals.contains_key(&name) {
        let equality = construction_data::physical_equal(b, 0)?;
        let value = b.var(0)?;
        let expected = bit(b, selected)?;
        let body = call(b, &equality, vec![value, expected])?;
        define(b, &name, &[0], 0, body)?;
    }
    Ok(name)
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
    if !p.pending_definition_reasons.is_empty() {
        return Ok(());
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
    captures: Option<&[OrdinaryControlPatternCapture]>,
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
        let capture = captures
            .map(|captures| {
                captures
                    .iter()
                    .find(|capture| capture.pattern_id == pattern.id)
                    .ok_or(OrdinaryCarrierError::Linkage)
            })
            .transpose()?;
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
            let source_observation = control
                .pattern_observations()
                .iter()
                .find(|o| o.sequent_id == source_sequent_id);
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
                original_pattern_predicate_pending: !c.constants.contains_key(&format!(
                    "Mpk.CSharp.Control.PatternStep.{}.{}",
                    pattern.id, step.source_node_id
                )),
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
                    route_observations: vec![],
                    capture_dependency: None,
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
                if let Some(capture) = capture {
                    if capture.function_id != pattern.function_id
                        || capture.governing_value != pattern.governing_value
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    if let (Some(definition), Some(governing)) =
                        (&capture.definition, capture.governing_argument_index)
                    {
                        let indices = capture
                            .arguments
                            .iter()
                            .map(|a| argument(&mut p.arguments, a.clone()))
                            .collect::<Vec<_>>();
                        let governing_argument_index = *indices
                            .get(governing)
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        if p.arguments[governing_argument_index] != capture.native_definition_point
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        p.components.push(OrdinaryControlStepComponent {
                            role: "governing_capture".into(),
                            definition: definition.clone(),
                            argument_indices: indices.clone(),
                        });
                        p.capture_dependency = Some(OrdinaryControlPatternCaptureDependency {
                            pattern_id: pattern.id.clone(),
                            producer_source_node_id: capture.producer_source_node_id.clone(),
                            definition: definition.clone(),
                            argument_indices: indices,
                            governing_argument_index,
                        });
                    } else {
                        p.pending_definition_reasons
                            .push("governing_capture_unresolved".into());
                    }
                }
                for (i, source) in goal.bindings.iter().enumerate() {
                    if source.kind == "source_route_selected" {
                        let observation =
                            source_observation.ok_or(OrdinaryCarrierError::Linkage)?;
                        let route = observation
                            .route
                            .as_ref()
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        let route_index = route
                            .binding_indices
                            .iter()
                            .position(|&index| index == i)
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        if source.type_id != SOURCE_BOOL
                            || source.edge_id.is_some()
                            || source.node_id != step.exit_node_id
                            || source.value_id != route.successor_source_node_ids[route_index]
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        let selected = route_selection(flow, edge, observation)? == route_index;
                        let observed = argument(&mut p.arguments, source.clone());
                        let definition = fixed_observation(&mut c.b, selected)?;
                        p.components.push(OrdinaryControlStepComponent {
                            role: format!("source_route_observation:{i}"),
                            definition: definition.clone(),
                            argument_indices: vec![observed],
                        });
                        p.route_observations
                            .push(OrdinaryControlPatternRouteObservation {
                                source: source.clone(),
                                observation_argument_index: observed,
                                successor_source_node_id: source.value_id.clone(),
                                native_edge_id: edge.source.id.clone(),
                                native_target_node_id: edge.source.target_node_id.clone(),
                                selected,
                                definition,
                            });
                        continue;
                    }
                    let point = if source.kind == "ssa" {
                        values
                            .get(&source.value_id)
                            .filter(|p| p.type_id == source.type_id)
                            .cloned()
                            .ok_or(OrdinaryCarrierError::Linkage)?
                    } else {
                        let slot = source_observation
                            .and_then(|o| o.slot.as_ref())
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        let phases = [
                            (
                                slot.before_assigned_index,
                                "source_entry_assigned",
                                &step.entry_node_id,
                                SOURCE_BOOL,
                            ),
                            (
                                slot.before_value_index,
                                "source_entry_slot",
                                &step.entry_node_id,
                                slot.storage_type_id.as_str(),
                            ),
                            (
                                slot.after_assigned_index,
                                "source_exit_assigned",
                                &step.exit_node_id,
                                SOURCE_BOOL,
                            ),
                            (
                                slot.after_value_index,
                                "source_exit_slot",
                                &step.exit_node_id,
                                slot.storage_type_id.as_str(),
                            ),
                        ];
                        if !phases.iter().any(|(index, kind, node, ty)| {
                            *index == i
                                && source.kind == *kind
                                && &source.node_id == *node
                                && source.type_id == *ty
                                && source.value_id == slot.source.slot
                                && source.edge_id.is_none()
                        }) {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        source.clone()
                    };
                    let native_index = execution.arguments.iter().position(|a| a == &point);
                    let captured_index = if source_observation.is_some() {
                        p.capture_dependency.as_ref().and_then(|dependency| {
                            dependency
                                .argument_indices
                                .iter()
                                .copied()
                                .find(|&index| p.arguments[index] == point)
                        })
                    } else if i == 0 {
                        p.capture_dependency
                            .as_ref()
                            .map(|dependency| dependency.governing_argument_index)
                    } else {
                        None
                    };
                    if captured_index.is_some_and(|i| p.arguments[i] != point) {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    let observed = argument(&mut p.arguments, source.clone());
                    p.observations.push(OrdinaryControlPatternObservation {
                        source: source.clone(),
                        native_definition_point: point.clone(),
                        observation_argument_index: observed,
                        native_argument_index: native_index,
                        captured_argument_index: captured_index,
                        source_literal_definition: None,
                    });
                    if let Some(native_index) = native_index.or(captured_index) {
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
                    } else if source_observation.is_some()
                        && control
                            .pattern_observations()
                            .iter()
                            .any(|o| o.route.is_some())
                        && source.type_id == "mpk.csharp.value.unit.v1"
                    {
                        let operand = source_observation
                            .and_then(|o| o.operands.iter().find(|o| o.binding_index == i))
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        let graph = flow
                            .source
                            .source_graph
                            .as_ref()
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        let producer = graph
                            .nodes
                            .iter()
                            .find(|n| n.id == operand.producer_source_node_id)
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        let literal = producer
                            .source_ordinal
                            .and_then(|ordinal| graph.operations.get(ordinal))
                            .ok_or(OrdinaryCarrierError::Linkage)?;
                        if producer.operation != "constant"
                            || literal.constant.as_deref() != Some("null")
                            || producer.result != operand.source_value_id
                            || !producer.exceptional_successors.is_empty()
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        let definition = fixed_observation(&mut c.b, false)?;
                        p.components.push(OrdinaryControlStepComponent {
                            role: format!("pure_source_null_unit_observation:{i}"),
                            definition: definition.clone(),
                            argument_indices: vec![observed],
                        });
                        p.observations
                            .last_mut()
                            .ok_or(OrdinaryCarrierError::Linkage)?
                            .source_literal_definition = Some(definition);
                    } else {
                        p.pending_observation_binding_indices.push(i);
                    }
                }
                let role = if source_observation.is_some() {
                    if captures.is_some() {
                        "PatternExecutionScopeWithCaptureAndSourceObservations"
                    } else {
                        "PatternExecutionScopeWithSourceObservations"
                    }
                } else if captures.is_some() {
                    "PatternExecutionScopeWithCapture"
                } else {
                    "PatternExecutionScope"
                };
                let id = if source_observation.is_some() {
                    name(
                        role,
                        &(
                            vir.hash(),
                            control.hash(),
                            &sequent.id,
                            &execution.edge_id,
                            &execution.component_map_sha256,
                        ),
                    )
                } else {
                    name(
                        role,
                        &(
                            vir.hash(),
                            &sequent.id,
                            &execution.edge_id,
                            &execution.component_map_sha256,
                        ),
                    )
                };
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
    fn governing_capture_is_separate_from_mutated_current_value() {
        let mut b = Builder::new().unwrap();
        let equality = construction_data::physical_equal(&mut b, 0).unwrap();
        let before = b.var(1).unwrap();
        let after = b.var(0).unwrap();
        let updated = call(&mut b, "Std.Bool.not", vec![before]).unwrap();
        let mutation = call(&mut b, &equality, vec![updated, after]).unwrap();
        define(&mut b, "Mpk.Scope.Mutation", &[0, 0], 0, mutation).unwrap();
        let bindings = (0..6)
            .map(|i| ControlBinding {
                kind: "ssa".into(),
                edge_id: None,
                node_id: format!("point.{i}"),
                value_id: format!("value.{i}"),
                type_id: SOURCE_BOOL.into(),
            })
            .collect::<Vec<_>>();
        let source_execution = OrdinaryControlSourceExecution {
            source_node_id: "consumer".into(),
            entry_node_id: "entry".into(),
            exit_node_id: "exit".into(),
            edge_id: "normal".into(),
            target_node_id: None,
            arguments: bindings[2..4].to_vec(),
            components: vec![],
            argument_count: 2,
            component_count: 1,
            component_map_sha256: "test".into(),
            state_rule: "current_value_mutation".into(),
            definition: Some("Mpk.Scope.Mutation".into()),
        };
        let mut scope = OrdinaryControlPatternExecutionScope {
            source_execution,
            edge_kind: "normal".into(),
            arguments: bindings,
            native_argument_indices: vec![2, 3],
            observations: vec![],
            route_observations: vec![],
            capture_dependency: None,
            components: vec![
                OrdinaryControlStepComponent {
                    role: "native_source_execution".into(),
                    definition: "Mpk.Scope.Mutation".into(),
                    argument_indices: vec![2, 3],
                },
                OrdinaryControlStepComponent {
                    role: "governing_capture".into(),
                    definition: equality.clone(),
                    argument_indices: vec![0, 1],
                },
                OrdinaryControlStepComponent {
                    role: "observation_transport:0".into(),
                    definition: equality.clone(),
                    argument_indices: vec![4, 1],
                },
                OrdinaryControlStepComponent {
                    role: "observation_transport:1".into(),
                    definition: equality,
                    argument_indices: vec![5, 3],
                },
            ],
            pending_observation_binding_indices: vec![],
            pending_definition_reasons: vec![],
            definition: None,
        };
        finish(&mut b, &mut scope, &[0; 6], "Mpk.Scope.Captured").unwrap();
        let certificate = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let mut captured_differs_from_current = 0;
        for bits in 0..64 {
            let values = (0..6).map(|i| bits & (1 << i) != 0).collect::<Vec<_>>();
            let expected = values[0] == values[1]
                && values[2] != values[3]
                && values[4] == values[1]
                && values[5] == values[3];
            assert_eq!(
                observed(run(
                    &certificate,
                    scope.definition.as_ref().unwrap(),
                    values.iter().copied().map(V::Bit).collect()
                )),
                expected
            );
            captured_differs_from_current += usize::from(expected && values[1] != values[3]);
        }
        assert!(captured_differs_from_current > 0);
    }

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
            route_observations: vec![],
            capture_dependency: None,
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
