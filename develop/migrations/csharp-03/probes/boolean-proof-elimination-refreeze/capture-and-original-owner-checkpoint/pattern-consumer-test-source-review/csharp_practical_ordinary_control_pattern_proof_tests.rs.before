//! Original source goal types and universal premise projections, not execution proofs.
use super::*;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

#[path = "csharp_practical_ordinary_control_pattern_packed_proof_tests.rs"]
mod packed;

fn constant_name(c: &Certificate, id: u32) -> &str {
    let TermNode::Const { global, .. } = c.term_table[id as usize] else {
        panic!("expected a named proposition constituent");
    };
    &c.name_table[c.declarations[global as usize].name as usize]
}

fn definition(c: &Certificate, name: &str) -> u32 {
    let d = c
        .declarations
        .iter()
        .find(|d| c.name_table[d.name as usize] == name)
        .unwrap();
    let DeclarationKind::Def { value, .. } = d.kind else {
        panic!("expected a proposition definition");
    };
    value
}

fn truth(
    c: &Certificate,
    term: u32,
    predicate: &str,
    indices: &[usize],
    count: usize,
    offset: u32,
) {
    let TermNode::App {
        function,
        ref arguments,
    } = c.term_table[term as usize]
    else {
        panic!("expected equality truth proposition");
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
        panic!("expected the exact original predicate application");
    };
    assert_eq!(constant_name(c, *function), predicate);
    assert_eq!(arguments.len(), indices.len());
    for (&term, &index) in arguments.iter().zip(indices) {
        assert_eq!(
            c.term_table[term as usize],
            TermNode::Var((count - 1 - index) as u32 + offset)
        );
    }
}

#[test]
fn csharp_03_t06_w09_pattern_proof_types_preserve_original_goals_and_premises() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(original_proof_types)
        .unwrap()
        .join()
        .unwrap();
}

fn original_proof_types() {
    let mut counts = [0usize; 5];
    each_context(|id, vir| {
        let prior =
            generate_csharp_practical_ordinary_control_predicates_with_pattern_routes(vir).unwrap();
        let p = generate_csharp_practical_ordinary_control_predicates_with_pattern_proof_types(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let bodies = declaration_bodies(&cert);
        for (name, body) in declaration_bodies(&old) {
            assert_eq!(bodies.get(&name), Some(&body), "{id}: unchanged {name}");
        }
        assert_eq!(p.pattern_sources(), prior.pattern_sources());
        assert_eq!(p.pattern_capture_scopes(), prior.pattern_capture_scopes());
        assert_eq!(p.sequents(), prior.sequents());
        let foundation = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation");
        assert_eq!(
            fs::read(foundation.join(format!("control-predicates/with-pattern-routes/{id}.json")))
                .unwrap(),
            prior.canonical_bytes()
        );
        assert!(cert.proof_node_table.is_empty() && cert.theory_certificates.is_empty());
        let report = mpk_kernel::verify_certificate_bytes(p.certificate_bytes())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(report.axiom_count, 0);
        for entry in p.pattern_proof_types() {
            counts[0] += 1;
            counts[1] += entry.components.len();
            assert!(
                entry.source_refinement_proof_pending
                    && entry.execution_establishment_proof_pending
            );
            let scope = p
                .pattern_capture_scopes()
                .iter()
                .find(|s| s.source_sequent_id == entry.source_sequent_id)
                .unwrap();
            let path = scope
                .executions
                .iter()
                .find(|e| e.source_execution.edge_id == entry.native_edge_id)
                .unwrap();
            assert_eq!(entry.arguments, path.arguments);
            assert_eq!(entry.components, path.components);
            let sequence = p
                .sequents()
                .iter()
                .find(|s| s.source.id == entry.source_sequent_id)
                .unwrap();
            assert_eq!(
                entry.goal_definition,
                *sequence.goals[0].definition.as_ref().unwrap()
            );
            for (&index, binding) in entry
                .goal_argument_indices
                .iter()
                .zip(&sequence.goals[0].source.bindings)
            {
                assert_eq!(&entry.arguments[index], binding);
            }
            let Some(name) = &entry.refinement_proposition_definition else {
                counts[4] += 1;
                assert!(
                    entry.scope_proposition_definition.is_none() && entry.premise_proofs.is_empty()
                );
                assert_eq!(entry.pending_definition_reasons, ["combined_binder_limit"]);
                continue;
            };
            counts[2] += 1;
            assert!(entry.pending_definition_reasons.is_empty());
            let mut value = definition(&cert, name);
            for _ in &entry.arguments {
                let TermNode::Pi { body, .. } = cert.term_table[value as usize] else {
                    panic!("missing physical binder")
                };
                value = body;
            }
            let TermNode::Pi {
                ty: mut premise,
                body: goal,
            } = cert.term_table[value as usize]
            else {
                panic!("missing complete path premise")
            };
            truth(
                &cert,
                goal,
                &entry.goal_definition,
                &entry.goal_argument_indices,
                entry.arguments.len(),
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
                        panic!("omitted component")
                    };
                    assert_eq!(constant_name(&cert, function), "Std.Logic.And");
                    assert_eq!(arguments.len(), 2);
                    premise = arguments[1];
                    arguments[0]
                };
                truth(
                    &cert,
                    term,
                    &component.definition,
                    &component.argument_indices,
                    entry.arguments.len(),
                    0,
                );
            }
            assert_eq!(entry.premise_proofs.len(), entry.components.len());
            for (proof, component) in entry.premise_proofs.iter().zip(&entry.components) {
                assert_eq!(&proof.source, component);
                let declaration = cert
                    .declarations
                    .iter()
                    .find(|d| cert.name_table[d.name as usize] == proof.theorem)
                    .unwrap();
                assert!(matches!(declaration.kind, DeclarationKind::Theorem { .. }));
                counts[3] += 1;
            }
        }
        if let Ok(root) = std::env::var("MPK_W09_PATTERN_PROOF_TYPES_OUTPUT") {
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
    });
    assert_eq!(counts, [113, 703, 108, 664, 5]);
    println!("Original pattern proof types: 113 paths, 703 premises, 108 refinement types, 664 checked projections, 5 binder-limited paths; source and execution proofs pending");
}
