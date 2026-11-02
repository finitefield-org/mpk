use super::*;
use foundation_proof_tests::{false_type, unchanged_prefix};
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

pub(super) fn app<'a>(c: &'a Certificate, t: u32, name: &str) -> &'a [u32] {
    let TermNode::App {
        function,
        ref arguments,
    } = c.term_table[t as usize]
    else {
        panic!("missing {name} application")
    };
    let TermNode::Const { global, ref levels } = c.term_table[function as usize] else {
        panic!("missing {name} constant")
    };
    assert!(levels.is_empty());
    assert_eq!(
        c.name_table[c.declarations[global as usize].name as usize],
        name
    );
    arguments
}

#[test]
fn csharp_03_t06_w09_identity_projection_proofs_original_source() {
    std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        let bundle = b();
        let output = std::env::var_os("MPK_W09_IDENTITY_PROOFS_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &output { fs::create_dir_all(dir).unwrap(); }
        let original = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/verification-logs/foundation-proofs/local/certificates");
        let filter = std::env::var("MPK_W09_IDENTITY_SOURCE_FILTER").ok();
        let (mut contexts, mut proof_count, mut supplied, mut remaining, mut pending) = (0, 0, 0, 0, 0);
        let mut unchanged = 0;
        for (id, row, facts) in sources() {
            if filter.as_ref().is_some_and(|f| f != &id) { continue; }
            let (context, captures) = support::replay_context(&bundle, &row);
            let source = ValidatedDataSource::import_captured_facts(&bundle, &context, &captures, &serde_json::to_vec(&facts).unwrap()).unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let vir = emitted.vir();
            eprintln!("{id}: original identity projection proof assembly");
            let p = generate_csharp_practical_ordinary_identity_proofs(vir).unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let hex = fs::read_to_string(original.join(format!("{id}.hex"))).unwrap();
            let hex = hex.split_whitespace().collect::<String>();
            let old_bytes = (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect::<Vec<_>>();
            assert_eq!(p.foundation().certificate_bytes(), old_bytes);
            assert_eq!(p.foundation().canonical_bytes(), fs::read(original.join(format!("{id}.json"))).unwrap());
            let full = generate_csharp_practical_vc(PracticalVcSource { artifact_context: &context, captured_inputs: &captures, vir }).unwrap();
            let sequents = full.binding_vcs().sequents();
            let originals = sequents.iter().filter(|s| s.kind == "identity_projection").collect::<Vec<_>>();
            assert_eq!(p.proofs().iter().map(|p| &p.sequent).collect::<Vec<_>>(), originals);
            let all = p.foundation().supplied_binding_sequent_ids().iter().chain(p.proofs().iter().map(|p| &p.sequent.id)).collect::<BTreeSet<_>>();
            assert_eq!(p.supplied_binding_sequent_ids(), sequents.iter().filter(|s| all.contains(&s.id)).map(|s| s.id.clone()).collect::<Vec<_>>());
            assert_eq!(p.remaining_binding_sequent_ids(), sequents.iter().filter(|s| !all.contains(&s.id)).map(|s| s.id.clone()).collect::<Vec<_>>());
            assert_eq!(p.pending_proof_ids(), sequents.iter().map(|s| s.id.clone()).collect::<Vec<_>>());
            let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            unchanged_prefix(&mpk_cert::decode_canonical_certificate(&old_bytes).unwrap(), &c);
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            assert_eq!(metadata["proof_check_pending"], true);
            assert_eq!(metadata["application_scope_pending"], true);
            assert!(metadata["static_transformers"].as_u64().unwrap() <= mpk_vc::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX);
            if p.proofs().is_empty() {
                assert_eq!(p.certificate_bytes(), old_bytes);
                assert_eq!(p.definition_certificate_bytes(), old_bytes);
                unchanged += 1;
            } else {
                validate_csharp_practical_certificate_structure(&c).unwrap();
                assert!(c.proof_node_table.is_empty() && c.theory_certificates.is_empty());
                assert_eq!(mpk_kernel::verify_certificate_bytes(p.certificate_bytes()).unwrap().axiom_count, 0);
                let projected = generate_csharp_practical_ordinary_binding_projections(vir).unwrap();
                for proof in p.proofs() {
                    assert!(projected.definitions().contains(&proof.projection));
                    let theorem = c.declarations.iter().find(|d| c.name_table[d.name as usize] == proof.theorem).unwrap();
                    let DeclarationKind::Theorem { ty, .. } = theorem.kind else { panic!("missing theorem") };
                    let TermNode::Const { global, .. } = c.term_table[ty as usize] else { panic!("missing original named type") };
                    assert_eq!(c.name_table[c.declarations[global as usize].name as usize], proof.proposition_definition);
                    let DeclarationKind::Def { value, .. } = c.declarations[global as usize].kind else { panic!("missing original proposition") };
                    let TermNode::Pi { body, .. } = c.term_table[value as usize] else { panic!("missing source receiver") };
                    let TermNode::Pi { ty, body } = c.term_table[body as usize] else { panic!("missing original domain premise") };
                    let premise = app(&c, ty, "Std.Eq");
                    assert_eq!(premise.len(), 3);
                    let domain = p.foundation().definition_program().public_domains().iter().find(|d| d.carrier.type_id == proof.sequent.subjects[0].type_id).unwrap();
                    assert!(matches!(c.term_table[app(&c, premise[1], &domain.valid_definition)[0] as usize], TermNode::Var(0)));
                    let goals = app(&c, body, "Std.Logic.And"); assert_eq!(goals.len(), 2);
                    for (&goal, definition) in goals.iter().zip([&proof.projection.project_definition, proof.projection.reconstruct_definition.as_ref().unwrap()]) {
                        let operands = app(&c, goal, "Std.Eq"); assert_eq!(operands.len(), 3);
                        assert!(matches!(c.term_table[operands[2] as usize], TermNode::Var(1)));
                        assert!(matches!(c.term_table[app(&c, operands[1], definition)[0] as usize], TermNode::Var(1)));
                    }
                }
                let bool_proof = p.proofs().iter().find(|p| p.sequent.subjects[0].type_id == "mpk.csharp.value.bool.v1").unwrap();
                let before = mpk_cert::decode_canonical_certificate(p.definition_certificate_bytes()).unwrap();
                let definition = &bool_proof.projection.project_definition;
                assert_eq!(mpk_kernel::verify_certificate_bytes(&false_type(before, definition)).unwrap().axiom_count, 0);
                let wrong = false_type(c, definition);
                assert_eq!(mpk_kernel::verify_certificate_bytes(&wrong).unwrap_err().kind(), mpk_kernel::VerificationErrorKind::CoreCheck);
                assert_eq!(import_csharp_practical_ordinary_identity_proofs(&p.canonical_bytes(), p.certificate_bytes(), vir).unwrap(), p);
                assert!(import_csharp_practical_ordinary_identity_proofs(&p.canonical_bytes(), &old_bytes, vir).is_err());
                for field in ["schema", "foundation", "original_certificate_sha256", "proofs", "supplied_binding_sequent_ids", "remaining_binding_sequent_ids", "pending_proof_ids"] {
                    let mut changed = metadata.clone(); changed[field] = json!(null);
                    assert!(import_csharp_practical_ordinary_identity_proofs(&serde_json::to_vec(&changed).unwrap(), p.certificate_bytes(), vir).is_err(), "{field}");
                }
                assert!(import_csharp_practical_ordinary_identity_proofs(&p.canonical_bytes(), &wrong, vir).is_err());
                if let Some(dir) = &output {
                    for (suffix, bytes) in [("", p.certificate_bytes()), ("-wrong", wrong.as_slice())] {
                        fs::write(dir.join(format!("{id}{suffix}.hex")), bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n").unwrap();
                    }
                }
            }
            if let Some(dir) = &output { fs::write(dir.join(format!("{id}.json")), p.canonical_bytes()).unwrap(); }
            contexts += 1; proof_count += p.proofs().len(); supplied += p.supplied_binding_sequent_ids().len(); remaining += p.remaining_binding_sequent_ids().len(); pending += p.pending_proof_ids().len();
        }
        eprintln!("identity projection proofs: {contexts} contexts, {proof_count} identity proofs, {supplied} supplied, {remaining} remaining, {pending} original application proof IDs, {unchanged} unchanged certificates");
        if filter.is_none() { assert_eq!((contexts, proof_count, supplied, remaining, pending, unchanged), (45, 2, 531, 456, 987, 44)); }
    }).unwrap().join().unwrap();
}
