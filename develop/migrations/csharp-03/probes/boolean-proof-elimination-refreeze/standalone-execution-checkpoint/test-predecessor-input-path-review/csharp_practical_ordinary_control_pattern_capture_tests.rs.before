//! Original governing producers, exact merged environments and hostile import.
use super::*;
use mpk_vc::csharp_practical_vir_model::ControlVcProgram;

#[derive(Default)]
pub(super) struct Coverage {
    captures: usize,
    paths: usize,
    linked_observations: usize,
    newly_linked: usize,
    transport_checks: usize,
    scope_checks: usize,
    true_scopes: usize,
}
impl Coverage {
    pub(super) fn finish(&self) {
        assert_eq!(
            (
                self.captures,
                self.paths,
                self.linked_observations,
                self.newly_linked
            ),
            (7, 113, 194, 86)
        );
        assert_eq!(self.transport_checks, 388);
        assert!(self.scope_checks >= 226 && self.true_scopes > 0);
        eprintln!("governing captures: {} exact producers, {} consuming paths, {} physical observations ({} newly linked), {} transport checks, {} full scope checks, {} true scopes",
                  self.captures, self.paths, self.linked_observations, self.newly_linked,
                  self.transport_checks, self.scope_checks, self.true_scopes);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    id: &str,
    program: &OrdinaryControlPredicateProgram,
    vir: &ValidatedPracticalVir,
    control: &ControlVcProgram,
    native: &OrdinaryControlEdgeProgram,
    certificate: &mpk_cert::encode::Certificate,
    depths: &BTreeMap<&str, u32>,
    coverage: &mut Coverage,
) {
    let prior =
        generate_csharp_practical_ordinary_control_predicates_with_pattern_scopes(vir).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-scopes");
    assert!(fs::read(root.join(format!("{id}.json"))).unwrap() == prior.canonical_bytes());
    assert_eq!(
        fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
        prior
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n"
    );
    assert_eq!(program.pattern_scopes(), prior.pattern_scopes());
    let before = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
    let current = declaration_bodies(certificate);
    for (name, body) in declaration_bodies(&before) {
        assert!(
            current.get(&name) == Some(&body),
            "{id}: changed prior scope {name}"
        );
    }
    assert_eq!(program.pattern_captures().len(), control.patterns().len());
    for capture in program.pattern_captures() {
        coverage.captures += 1;
        assert!(capture.execution_establishment_pending);
        assert!(capture.pending_reasons.is_empty() && capture.definition.is_some());
        let original = control
            .patterns()
            .iter()
            .find(|p| p.id == capture.pattern_id)
            .unwrap();
        assert_eq!(capture.function_id, original.function_id);
        assert_eq!(capture.source_ordinal, original.source_ordinal);
        assert_eq!(capture.governing_value, original.governing_value);
        let flow = native
            .functions()
            .iter()
            .find(|f| f.source.function_id == capture.function_id)
            .unwrap();
        let graph = flow.source.source_graph.as_ref().unwrap();
        let decision = graph
            .nodes
            .iter()
            .find(|n| {
                n.kind == "pattern_decision" && n.source_ordinal == Some(original.source_ordinal)
            })
            .unwrap();
        assert_eq!(decision.inputs[0], capture.governing_source_value_id);
        let producer = graph
            .nodes
            .iter()
            .find(|n| n.result == decision.inputs[0])
            .unwrap();
        assert_eq!(producer.id, capture.producer_source_node_id);
        let function = vir
            .functions()
            .iter()
            .find(|f| f.id == original.function_id)
            .unwrap();
        let anchor = function
            .control_protocol
            .as_ref()
            .unwrap()
            .anchors
            .iter()
            .find(|a| a.source_node_id == producer.id)
            .unwrap();
        assert_eq!(capture.producer_entry_node_id, anchor.entry_node_id);
        assert_eq!(capture.producer_exit_node_id, anchor.exit_node_id);
        assert_eq!(capture.producer_artifact_node_ids, anchor.artifact_node_ids);
        assert_eq!(anchor.result.as_ref(), Some(&capture.governing_value));
        let mut definition_points = vec![];
        for parameter in &function.parameter_values {
            definition_points.push((parameter, &function.blocks[0].node.id));
        }
        for block in &function.blocks {
            for phi in &block.phi_values {
                definition_points.push((&phi.value, &block.node.id));
            }
            for literal in &block.literal_values {
                definition_points.push((&literal.result, &block.node.id));
            }
            if let Some(invocation) = &block.invocation {
                definition_points.push((&invocation.result, &invocation.normal_successor_id));
            }
            if let Some(exception) = &block.handler_exception_value {
                definition_points.push((exception, &block.node.id));
            }
        }
        let points = definition_points
            .iter()
            .filter(|(v, _)| **v == capture.governing_value)
            .collect::<Vec<_>>();
        assert_eq!(points.len(), 1);
        assert_eq!(&capture.native_definition_point.node_id, points[0].1);
        assert_eq!(
            capture.native_definition_point.value_id,
            capture.governing_value.id
        );
        assert_eq!(
            capture.native_definition_point.type_id,
            capture.governing_value.type_id
        );
        let governed = capture.governing_argument_index.unwrap();
        assert_eq!(capture.arguments[governed], capture.native_definition_point);
        let mut normal = vec![];
        let mut exceptional = vec![];
        for execution in flow
            .source_executions
            .iter()
            .filter(|e| e.source_node_id == producer.id)
        {
            let edge = flow
                .edges
                .iter()
                .find(|e| e.source.id == execution.edge_id)
                .unwrap();
            if edge.source.kind == "normal" {
                normal.push(execution);
            } else {
                exceptional.push(execution.edge_id.clone());
            }
        }
        assert_eq!(capture.excluded_exceptional_edge_ids, exceptional);
        assert_eq!(capture.alternatives.len(), normal.len());
        for (alternative, execution) in capture.alternatives.iter().zip(normal) {
            assert_eq!(&alternative.source_execution, execution);
            assert_eq!(
                alternative
                    .argument_indices
                    .iter()
                    .map(|&i| capture.arguments[i].clone())
                    .collect::<Vec<_>>(),
                execution.arguments
            );
            let gov = alternative.governing_native_argument_index.unwrap();
            assert_eq!(execution.arguments[gov], capture.native_definition_point);
            assert_eq!(alternative.argument_indices[gov], governed);
        }
    }
    assert_eq!(
        program.pattern_capture_scopes().len(),
        prior.pattern_scopes().len()
    );
    for (scope, old) in program
        .pattern_capture_scopes()
        .iter()
        .zip(prior.pattern_scopes())
    {
        assert_eq!(scope.source_step, old.source_step);
        assert_eq!(scope.source_sequent_id, old.source_sequent_id);
        assert_eq!(scope.function_id, old.function_id);
        assert_eq!(scope.pattern_id, old.pattern_id);
        assert!(scope.original_pattern_predicate_pending);
        assert_eq!(scope.executions.len(), old.executions.len());
        let capture = program
            .pattern_captures()
            .iter()
            .find(|c| c.pattern_id == scope.pattern_id)
            .unwrap();
        for (path, prior_path) in scope.executions.iter().zip(&old.executions) {
            coverage.paths += 1;
            assert_eq!(path.source_execution, prior_path.source_execution);
            assert_eq!(
                path.native_argument_indices,
                prior_path.native_argument_indices
            );
            let native_count = path.source_execution.argument_count;
            assert_eq!(
                &path.arguments[..native_count],
                &prior_path.arguments[..native_count]
            );
            assert!(path.pending_observation_binding_indices.is_empty());
            let dependency = path.capture_dependency.as_ref().unwrap();
            assert_eq!(
                dependency.producer_source_node_id,
                capture.producer_source_node_id
            );
            assert_eq!(Some(&dependency.definition), capture.definition.as_ref());
            assert_eq!(
                dependency
                    .argument_indices
                    .iter()
                    .map(|&i| path.arguments[i].clone())
                    .collect::<Vec<_>>(),
                capture.arguments
            );
            assert_eq!(
                path.arguments[dependency.governing_argument_index],
                capture.native_definition_point
            );
            assert_eq!(path.observations.len(), prior_path.observations.len());
            for (i, (observation, previous)) in path
                .observations
                .iter()
                .zip(&prior_path.observations)
                .enumerate()
            {
                coverage.linked_observations += 1;
                assert_eq!(observation.source, previous.source);
                assert_eq!(
                    observation.native_definition_point,
                    previous.native_definition_point
                );
                assert_eq!(
                    observation.native_argument_index,
                    previous.native_argument_index
                );
                if i == 0 {
                    assert_eq!(
                        observation.captured_argument_index,
                        Some(dependency.governing_argument_index)
                    );
                } else {
                    assert!(observation.captured_argument_index.is_none());
                }
                let target = observation
                    .native_argument_index
                    .or(observation.captured_argument_index)
                    .unwrap();
                assert_eq!(path.arguments[target], observation.native_definition_point);
                coverage.newly_linked += usize::from(previous.native_argument_index.is_none());
                let transport = path
                    .components
                    .iter()
                    .find(|c| c.role == format!("observation_transport:{i}"))
                    .unwrap();
                assert_eq!(
                    transport.argument_indices,
                    [observation.observation_argument_index, target]
                );
                let depth = depths[observation.source.type_id.as_str()];
                let empty = || {
                    if depth == 0 {
                        V::Bit(false)
                    } else {
                        sparse_cube(depth, BTreeSet::new())
                    }
                };
                let changed = if depth == 0 {
                    V::Bit(true)
                } else {
                    sparse_cube(depth, BTreeSet::from([(1usize << depth) - 1]))
                };
                assert!(bit(run(
                    certificate,
                    &transport.definition,
                    vec![empty(), empty()]
                )));
                assert!(!bit(run(
                    certificate,
                    &transport.definition,
                    vec![empty(), changed]
                )));
                coverage.transport_checks += 2;
            }
            for flags in [false, true] {
                let mut arguments = path
                    .arguments
                    .iter()
                    .map(|a| {
                        let depth = depths[a.type_id.as_str()];
                        if depth == 0 {
                            V::Bit(flags)
                        } else {
                            sparse_cube(depth, BTreeSet::new())
                        }
                    })
                    .collect::<Vec<_>>();
                for observation in &path.observations {
                    let target = observation
                        .native_argument_index
                        .or(observation.captured_argument_index)
                        .unwrap();
                    arguments[observation.observation_argument_index] = arguments[target].clone();
                }
                let native_args = prior_path
                    .arguments
                    .iter()
                    .map(|a| arguments[path.arguments.iter().position(|p| p == a).unwrap()].clone())
                    .collect::<Vec<_>>();
                let native = bit(run(
                    &before,
                    prior_path.definition.as_ref().unwrap(),
                    native_args,
                ));
                let captured_args = dependency
                    .argument_indices
                    .iter()
                    .map(|&i| arguments[i].clone())
                    .collect::<Vec<_>>();
                let captured = capture.alternatives.iter().any(|alternative| {
                    let source = &alternative.source_execution;
                    if let Some(definition) = &source.definition {
                        bit(run(
                            &before,
                            definition,
                            alternative
                                .argument_indices
                                .iter()
                                .map(|&i| captured_args[i].clone())
                                .collect(),
                        ))
                    } else {
                        source.components.iter().all(|component| {
                            bit(run(
                                &before,
                                &component.definition,
                                component
                                    .argument_indices
                                    .iter()
                                    .map(|&i| {
                                        captured_args[alternative.argument_indices[i]].clone()
                                    })
                                    .collect(),
                            ))
                        })
                    }
                });
                assert_eq!(
                    bit(run(
                        certificate,
                        capture.definition.as_ref().unwrap(),
                        captured_args
                    )),
                    captured
                );
                let expected = native && captured;
                let eval_scope = |arguments: Vec<V>| {
                    if let Some(definition) = &path.definition {
                        bit(run(certificate, definition, arguments))
                    } else {
                        assert_eq!(path.pending_definition_reasons, ["combined_binder_limit"]);
                        path.components.iter().all(|component| {
                            bit(run(
                                certificate,
                                &component.definition,
                                component
                                    .argument_indices
                                    .iter()
                                    .map(|&i| arguments[i].clone())
                                    .collect(),
                            ))
                        })
                    }
                };
                assert_eq!(
                    eval_scope(arguments.clone()),
                    expected,
                    "{id} {}",
                    scope.source_sequent_id
                );
                coverage.scope_checks += 1;
                coverage.true_scopes += usize::from(expected);
                let governing = &path.observations[0];
                if governing.native_argument_index.is_none() {
                    assert!(!dependency
                        .argument_indices
                        .contains(&governing.observation_argument_index));
                    let depth = depths[governing.source.type_id.as_str()];
                    arguments[governing.observation_argument_index] = if depth == 0 {
                        V::Bit(!flags)
                    } else {
                        sparse_cube(depth, BTreeSet::from([(1usize << depth) - 1]))
                    };
                    assert!(!eval_scope(arguments));
                    coverage.scope_checks += 1;
                }
            }
        }
    }
    if !program.pattern_captures().is_empty() {
        let mut altered: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        altered["pattern_captures"][0]["producer_source_node_id"] =
            json!("different-governing-producer");
        assert!(
            import_csharp_practical_ordinary_control_predicates_with_pattern_captures(
                &serde_json::to_vec(&altered).unwrap(),
                program.certificate_bytes(),
                vir
            )
            .is_err()
        );
    }
}
