//! Complete bounded environments and exact original source/premise linkage.
use super::*;
use std::rc::Rc;

fn packed_truth(
    c: &Certificate,
    term: u32,
    predicate: &str,
    indices: &[usize],
    env: &OrdinaryControlPatternEnvironment,
    offset: u32,
) {
    let TermNode::App {
        function,
        ref arguments,
    } = c.term_table[term as usize]
    else {
        panic!("truth equality")
    };
    assert_eq!(constant_name(c, function), "Std.Eq");
    assert_eq!(arguments.len(), 3);
    assert_eq!(constant_name(c, arguments[0]), "Std.Bool");
    assert_eq!(constant_name(c, arguments[2]), "Std.Bool.true");
    let TermNode::App {
        function,
        arguments,
    } = &c.term_table[arguments[1] as usize]
    else {
        panic!("original predicate")
    };
    assert_eq!(constant_name(c, *function), predicate);
    assert_eq!(arguments.len(), indices.len());
    for (&term, &index) in arguments.iter().zip(indices) {
        let TermNode::App {
            function,
            ref arguments,
        } = c.term_table[term as usize]
        else {
            panic!("complete original field projection")
        };
        assert_eq!(
            constant_name(c, function),
            env.fields[index].read_definition
        );
        assert_eq!(arguments.len(), 1);
        assert_eq!(c.term_table[arguments[0] as usize], TermNode::Var(offset));
    }
}

#[test]
fn csharp_03_t06_w09_packed_pattern_proof_types_cover_complete_original_environments() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(original_packed_types)
        .unwrap()
        .join()
        .unwrap();
}

fn original_packed_types() {
    let mut counts = [0usize; 4];
    each_context(|id, vir| {
        println!("Packed proof context: {id}");
        let prior =
            generate_csharp_practical_ordinary_control_predicates_with_pattern_routes(vir).unwrap();
        let p =
            generate_csharp_practical_ordinary_control_predicates_with_packed_pattern_proof_types(
                vir,
            )
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        // A helper-only certificate can never supply the required refinements.
        assert!(link_csharp_practical_ordinary_pattern_refinement_candidate(
            &p,
            p.certificate_bytes()
        )
        .is_err());
        if !p.pattern_proof_types().is_empty() {
            let expected =
                csharp_practical_ordinary_pattern_refinement_candidate_theorems(&p).unwrap();
            assert_eq!(expected.len(), p.pattern_proof_types().len());
            for (theorem, path) in expected.iter().zip(p.pattern_proof_types()) {
                assert_eq!(theorem.source_sequent_id, path.source_sequent_id);
                assert_eq!(theorem.native_edge_id, path.native_edge_id);
                assert_eq!(
                    Some(&theorem.proposition_definition),
                    path.refinement_proposition_definition.as_ref()
                );
            }
        }
        let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let bodies = declaration_bodies(&cert);
        for (name, body) in declaration_bodies(&old) {
            assert_eq!(bodies.get(&name), Some(&body), "{id}: preserved {name}");
        }
        assert_eq!(p.pattern_sources(), prior.pattern_sources());
        assert_eq!(p.pattern_capture_scopes(), prior.pattern_capture_scopes());
        assert_eq!(p.sequents(), prior.sequents());
        assert!(cert.proof_node_table.is_empty() && cert.theory_certificates.is_empty());
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(p.certificate_bytes())
                .unwrap_or_else(|e| panic!("{id}: {e:?}"))
                .axiom_count,
            0
        );
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for entry in p.pattern_proof_types() {
            counts[0] += 1;
            counts[1] += entry.components.len();
            counts[2] += entry.premise_proofs.len();
            assert!(entry.pending_definition_reasons.is_empty());
            assert!(
                entry.source_refinement_proof_pending
                    && entry.execution_establishment_proof_pending
            );
            let env = entry.packed_environment.as_ref().unwrap();
            assert_eq!(env.fields.len(), entry.arguments.len());
            assert_eq!(
                env.address_depth,
                entry.arguments.len().next_power_of_two().trailing_zeros()
            );
            assert_eq!(
                env.payload_depth,
                entry
                    .arguments
                    .iter()
                    .map(|a| depths[a.type_id.as_str()])
                    .max()
                    .unwrap()
            );
            assert_eq!(env.depth, env.address_depth + env.payload_depth);
            let mut points = BTreeSet::new();
            for (index, field) in env.fields.iter().enumerate() {
                assert_eq!(field.argument_index, index);
                assert_eq!(field.type_id, entry.arguments[index].type_id);
                assert_eq!(field.depth, depths[field.type_id.as_str()]);
                let size = 1usize << field.depth;
                if index % 2 == 0 {
                    points.insert(index << env.payload_depth);
                }
                if size > 1 && index % 3 == 0 {
                    points.insert((index << env.payload_depth) | (size - 1));
                }
                let declaration = cert
                    .declarations
                    .iter()
                    .find(|d| cert.name_table[d.name as usize] == field.write_read_theorem)
                    .unwrap();
                assert!(matches!(declaration.kind, DeclarationKind::Theorem { .. }));
            }
            let points = Rc::new(points);
            let state = V::SparseCube(points.clone(), 0, 1, 1usize << env.depth);
            for field in &env.fields {
                for index in [0, (1usize << field.depth) - 1] {
                    let mut value = run(&cert, &field.read_definition, vec![state.clone()]);
                    for bit in 0..field.depth {
                        value = apply(&cert, value, V::Bit(index & (1 << bit) != 0));
                    }
                    assert_eq!(
                        observed(value),
                        points.contains(&((field.argument_index << env.payload_depth) | index)),
                        "{id}: source field {} bit {index}",
                        field.argument_index
                    );
                    counts[3] += 1;
                }
            }
            let value = definition(
                &cert,
                entry.refinement_proposition_definition.as_ref().unwrap(),
            );
            let TermNode::Pi { body, .. } = cert.term_table[value as usize] else {
                panic!("packed environment binder")
            };
            let TermNode::Pi {
                ty: mut premise,
                body: goal,
            } = cert.term_table[body as usize]
            else {
                panic!("complete scope proof binder")
            };
            packed_truth(
                &cert,
                goal,
                &entry.goal_definition,
                &entry.goal_argument_indices,
                env,
                1,
            );
            for (index, component) in entry.components.iter().enumerate() {
                let term = if index + 1 == entry.components.len() {
                    premise
                } else {
                    let TermNode::App {
                        function,
                        ref arguments,
                    } = cert.term_table[premise as usize]
                    else {
                        panic!("complete conjunction")
                    };
                    assert_eq!(constant_name(&cert, function), "Std.Logic.And");
                    premise = arguments[1];
                    arguments[0]
                };
                packed_truth(
                    &cert,
                    term,
                    &component.definition,
                    &component.argument_indices,
                    env,
                    0,
                );
                assert_eq!(entry.premise_proofs[index].source, *component);
            }
        }
        if let Ok(root) = std::env::var("MPK_W09_PACKED_PATTERN_OUTPUT") {
            let root = PathBuf::from(root);
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join(format!("{id}.json")), p.canonical_bytes()).unwrap();
            let hex = p
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            fs::write(root.join(format!("{id}.hex")), format!("{hex}\n")).unwrap();
        }
        println!("Packed proof context passed: {id}");
    });
    assert_eq!(&counts[..3], &[113, 703, 703]);
    println!("Packed original proof types: 113 paths, 703 complete premises/projections, {} original field-bit observations; source/execution/application proofs pending",counts[3]);
}
