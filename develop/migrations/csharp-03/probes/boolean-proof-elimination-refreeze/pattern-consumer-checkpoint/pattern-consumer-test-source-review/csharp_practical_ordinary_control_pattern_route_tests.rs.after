//! Ordered source choice, governing capture and abrupt-path observations.
use super::*;

#[test]
fn csharp_03_t06_w09_pattern_routes_original_conditions() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(original_routes)
        .unwrap()
        .join()
        .unwrap();
}

fn original_routes() {
    let mut route_conditions = 0;
    let mut route_bindings = 0;
    let mut branch_states = 0;
    let mut route_transports = 0;
    let mut literal_transports = 0;
    let mut source_goals = 0;
    each_context(|id, vir| {
        let old_control =
            generate_csharp_practical_control_vcs_with_pattern_observations(vir).unwrap();
        let control = generate_csharp_practical_control_vcs_with_pattern_routes(vir).unwrap();
        let p = generate_csharp_practical_ordinary_control_predicates_with_pattern_routes(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let prior =
            generate_csharp_practical_ordinary_control_predicates_with_pattern_observations(vir)
                .unwrap();
        let native = generate_csharp_practical_ordinary_control_edges(vir).unwrap();
        let foundation = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation");
        let primitive_input = std::env::var_os("MPK_W09_CONTROL_PATTERN_PRIMITIVE_INPUT")
            .map(PathBuf::from)
            .unwrap_or_else(|| foundation.join("control-predicates/with-pattern-primitives"));
        assert_eq!(
            fs::read(primitive_input.join(format!("{id}.json"))).unwrap(),
            prior.canonical_bytes()
        );
        let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert!(cert.proof_node_table.is_empty() && cert.theory_certificates.is_empty());
        assert!(!cert
            .declarations
            .iter()
            .any(|d| matches!(d.kind, mpk_cert::encode::DeclarationKind::Axiom { .. })));
        let replaced = prior
            .pattern_scopes()
            .iter()
            .chain(prior.pattern_capture_scopes())
            .flat_map(|s| &s.executions)
            .filter_map(|e| e.definition.as_ref())
            .collect::<BTreeSet<_>>();
        let bodies = declaration_bodies(&cert);
        for (name, body) in declaration_bodies(&old) {
            if !replaced.contains(&name) {
                assert_eq!(bodies.get(&name), Some(&body), "{id}: preserved {name}");
            }
        }
        assert_eq!(control.sequents().len(), old_control.sequents().len());
        for sequence in control.sequents() {
            let old = old_control
                .sequents()
                .iter()
                .find(|s| s.id == sequence.id)
                .unwrap();
            assert_eq!(sequence.assumptions, old.assumptions);
            assert_eq!(sequence.source_node_id, old.source_node_id);
            assert_eq!(sequence.target_node_id, old.target_node_id);
            for (goal, old) in sequence.goals.iter().zip(&old.goals) {
                assert_eq!(&goal.bindings[..old.bindings.len()], &old.bindings);
                assert!(goal.bindings[old.bindings.len()..]
                    .iter()
                    .all(|b| b.kind == "source_route_selected"));
            }
        }
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for entry in p.pattern_sources() {
            source_goals += 1;
            assert!(entry.definition.is_some() && entry.pending_definition_reasons.is_empty());
            assert!(entry.native_source_equivalence_proof_pending);
            let Some(route) = &entry.source.route else {
                let old = prior
                    .pattern_sources()
                    .iter()
                    .find(|s| s.source.sequent_id == entry.source.sequent_id)
                    .unwrap();
                assert_eq!(entry, old);
                continue;
            };
            route_conditions += 1;
            route_bindings += route.binding_indices.len();
            let sequence = control
                .sequents()
                .iter()
                .find(|s| s.id == entry.source.sequent_id)
                .unwrap();
            let mut args = sequence.goals[0]
                .bindings
                .iter()
                .map(|b| physical(depths[b.type_id.as_str()], false))
                .collect::<Vec<_>>();
            let definition = entry.definition.as_ref().unwrap();
            if route.source_kind == "branch" {
                let input = entry.source.operands[0].binding_index;
                for condition in [false, true] {
                    for yes in [false, true] {
                        for no in [false, true] {
                            args[input] = V::Bit(condition);
                            args[route.binding_indices[0]] = V::Bit(yes);
                            args[route.binding_indices[1]] = V::Bit(no);
                            assert_eq!(
                                observed(run(&cert, definition, args.clone())),
                                yes == condition && no != condition,
                                "{id}: ordered branch {condition}/{yes}/{no}"
                            );
                            branch_states += 1;
                        }
                    }
                }
            } else {
                let flag = route.binding_indices[0];
                args[flag] = V::Bit(true);
                assert!(
                    observed(run(&cert, definition, args.clone())),
                    "{id}: source route"
                );
                args[flag] = V::Bit(false);
                assert!(
                    !observed(run(&cert, definition, args.clone())),
                    "{id}: route absent"
                );
                if route.source_kind == "pattern_decision" {
                    args[flag] = V::Bit(true);
                    flip(&mut args[entry.source.operands[0].binding_index]);
                    assert!(
                        !observed(run(&cert, definition, args)),
                        "{id}: stale governing capture"
                    );
                }
            }
        }
        for scope in p.pattern_capture_scopes() {
            let observation = control
                .pattern_observations()
                .iter()
                .find(|o| o.sequent_id == scope.source_sequent_id)
                .unwrap();
            let sequence = control
                .sequents()
                .iter()
                .find(|s| s.id == scope.source_sequent_id)
                .unwrap();
            for execution in &scope.executions {
                assert!(
                    execution.pending_observation_binding_indices.is_empty(),
                    "{id}: {:?}",
                    execution.pending_observation_binding_indices
                );
                assert_eq!(
                    execution.observations.len() + execution.route_observations.len(),
                    sequence.goals[0].bindings.len()
                );
                assert!(
                    execution
                        .components
                        .iter()
                        .any(|c| c.role == "native_source_execution")
                        || !execution.source_execution.components.is_empty()
                );
                for route in &execution.route_observations {
                    let source = observation.route.as_ref().unwrap();
                    let route_index = source
                        .successor_source_node_ids
                        .iter()
                        .position(|s| s == &route.successor_source_node_id)
                        .unwrap();
                    let flow = native
                        .functions()
                        .iter()
                        .find(|f| f.source.function_id == scope.function_id)
                        .unwrap();
                    let edge = flow
                        .edges
                        .iter()
                        .find(|e| e.source.id == execution.source_execution.edge_id)
                        .unwrap();
                    let expected = if source.source_kind == "branch" {
                        let positive = matches!(edge.source.guard.term, ContractTerm::Var { .. });
                        route_index == usize::from(!positive)
                    } else {
                        true
                    };
                    assert_eq!(route.selected, expected);
                    assert_eq!(route.native_edge_id, edge.source.id);
                    assert_eq!(route.native_target_node_id, edge.source.target_node_id);
                    assert_eq!(
                        execution.arguments[route.observation_argument_index],
                        route.source
                    );
                    assert!(observed(run(
                        &cert,
                        &route.definition,
                        vec![V::Bit(expected)]
                    )));
                    assert!(!observed(run(
                        &cert,
                        &route.definition,
                        vec![V::Bit(!expected)]
                    )));
                    route_transports += 1;
                }
                for literal in &execution.observations {
                    if let Some(definition) = &literal.source_literal_definition {
                        assert_eq!(literal.source.type_id, "mpk.csharp.value.unit.v1");
                        assert!(observed(run(&cert, definition, vec![V::Bit(false)])));
                        assert!(!observed(run(&cert, definition, vec![V::Bit(true)])));
                        literal_transports += 1;
                    }
                }
            }
        }
        let bytes = p.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_control_predicates_with_pattern_routes(
                &bytes,
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        assert_eq!(
            import_csharp_practical_control_vcs_with_pattern_routes(
                &control.canonical_bytes(),
                vir
            )
            .unwrap(),
            control
        );
        if let Some(index) = control
            .pattern_observations()
            .iter()
            .position(|o| o.route.is_some())
        {
            let mut changed: Value = serde_json::from_slice(&control.canonical_bytes()).unwrap();
            changed["pattern_observations"][index]["route"]["successor_source_node_ids"][0] =
                json!("unrelated.successor");
            assert!(import_csharp_practical_control_vcs_with_pattern_routes(
                &serde_json::to_vec(&changed).unwrap(),
                vir
            )
            .is_err());
        }
        let output = std::env::var_os("MPK_W09_CONTROL_PATTERN_ROUTE_OUTPUT");
        let root = output
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| foundation.join("control-predicates/with-pattern-routes"));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if output.is_some() {
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join(format!("{id}.json")), bytes).unwrap();
            fs::write(root.join(format!("{id}.hex")), hex).unwrap();
            fs::write(
                root.join(format!("{id}.control.json")),
                control.canonical_bytes(),
            )
            .unwrap();
        } else {
            assert_eq!(fs::read(root.join(format!("{id}.json"))).unwrap(), bytes);
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
            assert_eq!(
                fs::read(root.join(format!("{id}.control.json"))).unwrap(),
                control.canonical_bytes()
            );
        }
    });
    assert_eq!(
        (
            source_goals,
            route_conditions,
            route_bindings,
            branch_states,
            route_transports,
            literal_transports
        ),
        (102, 21, 32, 88, 54, 1)
    );
    eprintln!("original pattern routes: {source_goals} defined conditions, {route_conditions} control conditions, {route_bindings} route bindings, {branch_states} branch states, {route_transports} route transports, {literal_transports} literal transport; application proofs pending");
}
