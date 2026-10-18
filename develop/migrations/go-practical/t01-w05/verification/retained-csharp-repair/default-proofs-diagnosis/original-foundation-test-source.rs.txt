use super::*;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

fn encode(mut c: Certificate) -> Vec<u8> {
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode::encode_certificate(&c)
}

pub(super) fn false_type(mut c: Certificate, name: &str) -> Vec<u8> {
    let declaration = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap();
    let DeclarationKind::Def {
        ty,
        mut value,
        reducibility,
    } = c.declarations[declaration].kind
    else {
        panic!("missing concrete type predicate")
    };
    let mut binders = vec![];
    while let TermNode::Lam { ty, body } = c.term_table[value as usize] {
        binders.push(ty);
        value = body;
    }
    let no = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == "Std.Bool.false")
        .unwrap() as u32;
    value = c.term_table.len() as u32;
    c.term_table.push(TermNode::Const {
        global: no,
        levels: vec![],
    });
    for ty in binders.into_iter().rev() {
        let body = value;
        value = c.term_table.len() as u32;
        c.term_table.push(TermNode::Lam { ty, body });
    }
    c.declarations[declaration].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    encode(c)
}

pub(super) fn unchanged_prefix(before: &Certificate, after: &Certificate) {
    assert!(after.term_table.starts_with(&before.term_table));
    assert_eq!(
        (&after.module, &after.source_manifest, &after.imports),
        (&before.module, &before.source_manifest, &before.imports)
    );
    for (a, z) in before.declarations.iter().zip(&after.declarations) {
        assert_eq!(
            before.name_table[a.name as usize],
            after.name_table[z.name as usize]
        );
        assert_eq!(a.kind, z.kind);
    }
}

#[test]
fn csharp_03_t06_w09_foundation_proof_assembly_original_source() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let output = std::env::var_os("MPK_W09_FOUNDATION_PROOFS_OUT")
                .map(std::path::PathBuf::from);
            if let Some(dir) = &output {
                fs::create_dir_all(dir).unwrap();
            }
            let filter = std::env::var("MPK_W09_FOUNDATION_SOURCE_FILTER").ok();
            let (mut contexts, mut types, mut operations, mut supplied, mut remaining, mut original) =
                (0, 0, 0, 0, 0, 0);
            let mut mutated = false;
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            for (id, row, facts) in sources() {
                if filter.as_ref().is_some_and(|f| f != &id) {
                    continue;
                }
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle, &context, &captures, &serde_json::to_vec(&facts).unwrap(),
                ).unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let vir = emitted.vir();
                eprintln!("{id}: integrate exact original type and operation proofs");
                let p = generate_csharp_practical_ordinary_foundation_proofs(vir)
                    .unwrap_or_else(|error| panic!("{id}: {error:?}"));
                let full = generate_csharp_practical_vc(PracticalVcSource {
                    artifact_context: &context, captured_inputs: &captures, vir,
                }).unwrap();
                let vc = full.binding_vcs();
                let expected_types = vc.sequents().iter()
                    .filter(|s| s.kind == "concrete_type_equivalence").collect::<Vec<_>>();
                assert_eq!(p.type_proofs().iter().map(|p| &p.sequent).collect::<Vec<_>>(), expected_types);
                let pending = p.pending_operations().iter().map(|o| o.component.operation_id.as_str()).collect::<BTreeSet<_>>();
                let expected_operations = vc.sequents().iter()
                    .filter(|s| s.kind == "concrete_definition_equivalence" && !pending.contains(s.owner_id.as_str())).collect::<Vec<_>>();
                assert_eq!(p.operation_proofs().iter().map(|p| &p.sequent).collect::<Vec<_>>(), expected_operations);
                let supplied_ids = expected_types.iter().chain(&expected_operations).map(|s| s.id.as_str()).collect::<BTreeSet<_>>();
                assert_eq!(supplied_ids.len(), expected_types.len() + expected_operations.len());
                assert_eq!(p.pending_proof_ids(), vc.sequents().iter().map(|s| s.id.clone()).collect::<Vec<_>>());
                assert_eq!(p.supplied_binding_sequent_ids(), vc.sequents().iter().filter(|s| supplied_ids.contains(s.id.as_str())).map(|s| s.id.clone()).collect::<Vec<_>>());
                assert_eq!(p.remaining_binding_sequent_ids(), vc.sequents().iter().filter(|s| !supplied_ids.contains(s.id.as_str())).map(|s| s.id.clone()).collect::<Vec<_>>());
                let legacy_operations = generate_csharp_practical_ordinary_concrete_operation_proofs_with_allocations(vir).unwrap();
                assert_eq!(p.operation_proofs(), legacy_operations.proofs());
                assert_eq!(p.pending_operations(), legacy_operations.pending_operations());
                let old = mpk_cert::decode_canonical_certificate(legacy_operations.certificate_bytes()).unwrap();
                let before = mpk_cert::decode_canonical_certificate(p.definition_program().certificate_bytes()).unwrap();
                let after = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                unchanged_prefix(&old, &before);
                unchanged_prefix(&before, &after);
                let legacy_types = generate_csharp_practical_ordinary_concrete_types_with_construction_storage(vir).unwrap();
                let definition_metadata: Value = serde_json::from_slice(&p.definition_program().canonical_bytes()).unwrap();
                assert!(definition_metadata["static_transformers"].as_u64().unwrap() <= mpk_vc::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX);
                assert_eq!(p.definition_program().definitions(), legacy_types.definitions());
                assert_eq!(p.definition_program().public_domains(), legacy_types.public_domains());
                assert_eq!(p.definition_program().source_clauses(), legacy_types.source_clauses());
                assert_eq!(p.construction_storage_domains(), legacy_types.construction_storage_domains());
                assert_eq!(p.definition_program().conditions().iter().map(|c| &c.sequent).collect::<Vec<_>>(), expected_types);
                assert!(p.construction_storage_domains().iter().all(|d| d.private_storage_only && d.ownership_pending));
                validate_csharp_practical_certificate_structure(&after).unwrap();
                assert!(after.proof_node_table.is_empty() && after.theory_certificates.is_empty());
                assert_eq!(mpk_kernel::verify_certificate_bytes(p.certificate_bytes()).unwrap().axiom_count, 0);
                let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                assert_eq!(metadata["proof_check_pending"], true);
                assert_eq!(metadata["application_scope_pending"], true);
                for proof in p.type_proofs().iter().map(|p| (&p.theorem, &p.proposition_definition))
                    .chain(p.operation_proofs().iter().map(|p| (&p.theorem, &p.proposition_definition))) {
                    let theorem = after.declarations.iter().find(|d| after.name_table[d.name as usize] == *proof.0).unwrap();
                    let DeclarationKind::Theorem { ty, .. } = theorem.kind else { panic!("missing theorem") };
                    let TermNode::Const { global, ref levels } = after.term_table[ty as usize] else { panic!("missing original named proposition") };
                    assert!(levels.is_empty());
                    assert_eq!(&after.name_table[after.declarations[global as usize].name as usize], proof.1);
                }
                for proof in p.type_proofs() {
                    let proposition = after.declarations.iter().find(|d| after.name_table[d.name as usize] == proof.proposition_definition).unwrap();
                    let DeclarationKind::Def { value, .. } = proposition.kind else { panic!("missing type proposition body") };
                    let TermNode::Pi { body, .. } = after.term_table[value as usize] else { panic!("missing original universal value") };
                    let TermNode::App { function, ref arguments } = after.term_table[body as usize] else { panic!("missing original equality") };
                    assert_eq!(arguments.len(), 3);
                    let TermNode::Const { global, .. } = after.term_table[function as usize] else { panic!("missing checked equality") };
                    assert_eq!(after.name_table[after.declarations[global as usize].name as usize], "Std.Eq");
                    let definition = p.definition_program().definitions().iter().find(|d| d.instance.instance_id == proof.sequent.owner_id).unwrap();
                    for (operand, expected) in [(arguments[1], &definition.public_domain), (arguments[2], &definition.definition)] {
                        let TermNode::App { function, ref arguments } = after.term_table[operand as usize] else { panic!("missing complete type operand") };
                        assert_eq!(arguments.len(), 1);
                        assert!(matches!(after.term_table[arguments[0] as usize], TermNode::Var(0)));
                        let TermNode::Const { global, ref levels } = after.term_table[function as usize] else { panic!("missing original type definition") };
                        assert!(levels.is_empty());
                        assert_eq!(&after.name_table[after.declarations[global as usize].name as usize], expected);
                    }
                }
                assert_eq!(import_csharp_practical_ordinary_foundation_proofs(&p.canonical_bytes(), p.certificate_bytes(), vir).unwrap(), p);
                if let Some((metadata, certificate)) = &previous {
                    assert!(import_csharp_practical_ordinary_foundation_proofs(metadata, certificate, vir).is_err());
                }
                if !mutated && !p.type_proofs().is_empty() {
                    let definition = &p.definition_program().definitions()[0].definition;
                    assert_eq!(mpk_kernel::verify_certificate_bytes(&false_type(before, definition)).unwrap().axiom_count, 0);
                    let wrong = false_type(after.clone(), definition);
                    assert_eq!(mpk_kernel::verify_certificate_bytes(&wrong).unwrap_err().kind(), mpk_kernel::VerificationErrorKind::CoreCheck);
                    assert!(import_csharp_practical_ordinary_foundation_proofs(&p.canonical_bytes(), legacy_operations.certificate_bytes(), vir).is_err());
                    for field in ["schema", "binding_vc_sha256", "source_ir_sha256", "supplied_binding_sequent_ids", "remaining_binding_sequent_ids", "pending_proof_ids", "types", "operations"] {
                        let mut changed = metadata.clone(); changed[field] = json!(null);
                        assert!(import_csharp_practical_ordinary_foundation_proofs(&serde_json::to_vec(&changed).unwrap(), p.certificate_bytes(), vir).is_err(), "{field}");
                    }
                    if let Some(dir) = &output { fs::write(dir.join(format!("{id}-wrong.hex")), wrong.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n").unwrap(); }
                    mutated = true;
                }
                if let Some(dir) = &output {
                    fs::write(dir.join(format!("{id}.json")), p.canonical_bytes()).unwrap();
                    fs::write(dir.join(format!("{id}.hex")), p.certificate_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n").unwrap();
                }
                previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
                contexts += 1;
                types += p.type_proofs().len(); operations += p.operation_proofs().len();
                supplied += p.supplied_binding_sequent_ids().len(); remaining += p.remaining_binding_sequent_ids().len(); original += p.pending_proof_ids().len();
            }
            eprintln!("foundation proof assembly: {contexts} contexts, {types} type proofs, {operations} operation proofs, {supplied} supplied sequents, {remaining} remaining sequents, {original} original application proof IDs");
            if filter.is_none() {
                assert_eq!((contexts, types, operations, supplied, remaining, original), (45, 87, 442, 529, 458, 987));
                assert!(mutated);
            }
        })
        .unwrap().join().unwrap();
}
