use super::*;
use core_eval::V;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir;
#[path = "csharp_practical_ordinary_construction_type_tests.rs"]
mod construction_storage_tests;

#[test]
fn csharp_03_t06_w09_concrete_type_proofs_original_source() {
    type_proofs_source(false);
}

#[test]
fn csharp_03_t06_w09_construction_type_proofs_original_source() {
    type_proofs_source(true);
}

type TypeGenerator =
    fn(&ValidatedPracticalVir) -> Result<OrdinaryConcreteTypeProgram, OrdinaryCarrierError>;
type TypeProofGenerator =
    fn(&ValidatedPracticalVir) -> Result<OrdinaryConcreteTypeProofProgram, OrdinaryCarrierError>;
type TypeProofImporter = fn(
    &[u8],
    &[u8],
    &ValidatedPracticalVir,
) -> Result<OrdinaryConcreteTypeProofProgram, OrdinaryCarrierError>;

fn type_proofs_source(with_storage: bool) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let generate_types: TypeGenerator = if with_storage { generate_csharp_practical_ordinary_concrete_types_with_construction_storage } else { generate_csharp_practical_ordinary_concrete_types };
            let generate_proofs: TypeProofGenerator = if with_storage { generate_csharp_practical_ordinary_concrete_type_proofs_with_construction_storage } else { generate_csharp_practical_ordinary_concrete_type_proofs };
            let import_proofs: TypeProofImporter = if with_storage { import_csharp_practical_ordinary_concrete_type_proofs_with_construction_storage } else { import_csharp_practical_ordinary_concrete_type_proofs };
            let bundle = b();
            let output =
                std::env::var_os("MPK_W09_CONCRETE_TYPE_PROOFS_OUT").map(std::path::PathBuf::from);
            if let Some(dir) = &output {
                fs::create_dir_all(dir).unwrap();
            }
            let mut count = 0;
            let mut pending_states = 0;
            let mut contexts = 0;
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            for (id, row, facts) in sources() {
                let (context, captures) = support::replay_context(&bundle, &row);
                let source = ValidatedDataSource::import_captured_facts(
                    &bundle,
                    &context,
                    &captures,
                    &serde_json::to_vec(&facts).unwrap(),
                )
                .unwrap();
                let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
                let vir = emitted.vir();
                let original = generate_types(vir).unwrap();
                let p = generate_proofs(vir)
                    .unwrap_or_else(|error| panic!("{id}: {error:?}"));
                assert_eq!(
                    p.proofs().iter().map(|p| &p.sequent).collect::<Vec<_>>(),
                    original
                        .conditions()
                        .iter()
                        .map(|c| &c.sequent)
                        .collect::<Vec<_>>()
                );
                assert_eq!(p.pending_proof_ids(), original.pending_proof_ids());
                if with_storage {
                    let legacy = generate_csharp_practical_ordinary_concrete_types(vir).unwrap();
                    let old = mpk_cert::decode_canonical_certificate(legacy.certificate_bytes()).unwrap();
                    let extended = mpk_cert::decode_canonical_certificate(original.certificate_bytes()).unwrap();
                    assert!(extended.term_table.starts_with(&old.term_table));
                    assert_eq!((&extended.module, &extended.source_manifest, &extended.imports), (&old.module, &old.source_manifest, &old.imports));
                    for (a, z) in old.declarations.iter().zip(&extended.declarations) {
                        assert_eq!(old.name_table[a.name as usize], extended.name_table[z.name as usize]);
                        assert_eq!(a.kind, z.kind);
                    }
                    assert_eq!(original.public_domains(), legacy.public_domains());
                    assert_eq!(original.source_clauses(), legacy.source_clauses());
                    assert_eq!(original.pending_proof_ids(), legacy.pending_proof_ids());
                    assert_eq!(p.construction_storage_domains(), original.construction_storage_domains());
                    assert_eq!(original.construction_storage_domains().len(), legacy.pending_type_instances().len());
                    assert!(original.construction_storage_domains().iter().all(|d| d.private_storage_only && d.ownership_pending));
                    for d in legacy.definitions() { assert_eq!(Some(d), original.definitions().iter().find(|n| n.instance.instance_id == d.instance.instance_id)); }
                    for c in legacy.conditions() { assert_eq!(Some(c), original.conditions().iter().find(|n| n.sequent.id == c.sequent.id)); }
                    let full = generate_csharp_practical_vc(PracticalVcSource { artifact_context: &context, captured_inputs: &captures, vir }).unwrap();
                    assert_eq!(original.definitions().iter().map(|d| &d.instance).collect::<Vec<_>>(), full.binding_vcs().instances().iter().collect::<Vec<_>>());
                    assert_eq!(original.conditions().iter().map(|c| &c.sequent).collect::<Vec<_>>(), full.binding_vcs().sequents().iter().filter(|s| s.kind == "concrete_type_equivalence").collect::<Vec<_>>());
                    assert!(original.pending_type_instances().is_empty());
                    assert_eq!(import_csharp_practical_ordinary_concrete_types_with_construction_storage(&original.canonical_bytes(), original.certificate_bytes(), vir).unwrap(), original);
                    if legacy.pending_type_instances().is_empty() { assert_eq!(original, legacy); }
                }

                assert_eq!(
                    p.pending_type_instances(),
                    original.pending_type_instances()
                );
                let before =
                    mpk_cert::decode_canonical_certificate(original.certificate_bytes()).unwrap();
                let after = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                assert!(after.term_table.starts_with(&before.term_table));
                assert_eq!(after.module, before.module);
                assert_eq!(after.source_manifest, before.source_manifest);
                assert_eq!(after.imports, before.imports);
                for (old, new) in before.declarations.iter().zip(&after.declarations) {
                    assert_eq!(
                        before.name_table[old.name as usize],
                        after.name_table[new.name as usize]
                    );
                    assert_eq!(old.kind, new.kind);
                }
                validate_csharp_practical_certificate_structure(&after).unwrap();
                let checked = mpk_kernel::verify_certificate_bytes(p.certificate_bytes())
                    .unwrap_or_else(|error| panic!("{id}: {error:?}"));
                assert_eq!(checked.axiom_count, 0);
                for proof in p.proofs() {
                    assert!(proof.sequent.assumptions.is_empty());
                    assert_eq!(proof.sequent.subjects.len(), 1);
                    assert_eq!(proof.sequent.goals.len(), 1);
                    let theorem = after
                        .declarations
                        .iter()
                        .find(|d| after.name_table[d.name as usize] == proof.theorem)
                        .unwrap();
                    let DeclarationKind::Theorem { ty, .. } = theorem.kind else {
                        panic!("missing theorem")
                    };
                    let TermNode::Const { global, ref levels } = after.term_table[ty as usize]
                    else {
                        panic!("missing original named proposition")
                    };
                    assert!(levels.is_empty());
                    assert_eq!(
                        after.name_table[after.declarations[global as usize].name as usize],
                        proof.proposition_definition
                    );
                    let DeclarationKind::Def { mut value, .. } =
                        after.declarations[global as usize].kind
                    else {
                        panic!("missing proposition body")
                    };
                    let TermNode::Pi { body, .. } = after.term_table[value as usize] else {
                        panic!("missing universal subject")
                    };
                    value = body;
                    let TermNode::App {
                        function,
                        ref arguments,
                    } = after.term_table[value as usize]
                    else {
                        panic!("missing exact equality")
                    };
                    assert_eq!(arguments.len(), 3);
                    let TermNode::Const { global, .. } = after.term_table[function as usize] else {
                        panic!("missing equality foundation")
                    };
                    assert_eq!(
                        after.name_table[after.declarations[global as usize].name as usize],
                        "Std.Eq"
                    );
                    let d = original.definitions().iter().find(|d| d.instance.instance_id == proof.sequent.owner_id).unwrap();
                    for (operand, expected) in [(arguments[1], &d.public_domain), (arguments[2], &d.definition)] {
                        let TermNode::App { function, arguments } = &after.term_table[operand as usize] else { panic!("missing complete original operand") };
                        assert_eq!(arguments.len(), 1);
                        assert!(matches!(after.term_table[arguments[0] as usize], TermNode::Var(0)));
                        let TermNode::Const { global, ref levels } = after.term_table[*function as usize] else { panic!("missing original operand definition") };
                        assert!(levels.is_empty());
                        assert_eq!(&after.name_table[after.declarations[global as usize].name as usize], expected);
                    }

                }
                assert_eq!(
                    import_proofs(
                        &p.canonical_bytes(),
                        p.certificate_bytes(),
                        vir
                    )
                    .unwrap(),
                    p
                );
                if let Some((metadata, certificate)) = &previous {
                    assert!(import_proofs(
                        metadata,
                        certificate,
                        vir
                    )
                    .is_err());
                }
                if contexts == 0 || (with_storage && id == "bool-construction") {
                    let mut wrong = after.clone();
                    let proof_name = &p.proofs()[0].theorem;
                    let false_global = wrong
                        .declarations
                        .iter()
                        .position(|d| wrong.name_table[d.name as usize] == "Std.Bool.false")
                        .unwrap() as u32;
                    let false_term = wrong
                        .term_table
                        .iter()
                        .position(|t| matches!(t, TermNode::Const {global, levels} if *global == false_global && levels.is_empty()))
                        .unwrap() as u32;
                    if contexts == 0 {
                    let declaration = wrong
                        .declarations
                        .iter_mut()
                        .find(|d| after.name_table[d.name as usize] == *proof_name)
                        .unwrap();
                    let DeclarationKind::Theorem {ty, ..} = declaration.kind else { panic!("missing theorem") };
                    declaration.kind=DeclarationKind::Theorem {ty, proof:false_term};
                    } else {
                        let d = original.definitions().iter().find(|d| original.construction_storage_domains().iter().any(|s| s.carrier.type_id == d.instance.instance_id)).unwrap();
                        let mut definitions_only = before.clone();
                        replace_predicate(&mut definitions_only, &d.definition, true);
                        definitions_only.export_block = mpk_cert::build_export_block(&definitions_only).unwrap();
                        definitions_only.axiom_report = mpk_cert::build_axiom_report(&definitions_only).unwrap();
                        definitions_only.hashes.export_hash = mpk_cert::export_block_hash(&definitions_only.export_block);
                        definitions_only.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&definitions_only.axiom_report);
                        let definitions_only = mpk_cert::encode::encode_certificate(&definitions_only);
                        assert_eq!(mpk_kernel::verify_certificate_bytes(&definitions_only).unwrap().axiom_count, 0);
                        replace_predicate(&mut wrong, &d.definition, true);
                    }
                    wrong.export_block = mpk_cert::build_export_block(&wrong).unwrap();
                    wrong.axiom_report = mpk_cert::build_axiom_report(&wrong).unwrap();
                    wrong.hashes.export_hash = mpk_cert::export_block_hash(&wrong.export_block);
                    wrong.hashes.axiom_report_hash =
                        mpk_cert::axiom_report_hash_for_report(&wrong.axiom_report);
                    let wrong_bytes=mpk_cert::encode::encode_certificate(&wrong);
                    assert_eq!(
                        mpk_kernel::verify_certificate_bytes(&wrong_bytes)
                            .unwrap_err()
                            .kind(),
                        mpk_kernel::VerificationErrorKind::CoreCheck
                    );
                    assert!(import_proofs(&p.canonical_bytes(),&wrong_bytes,vir).is_err());
                    if let Some(dir)=&output {
                        let hex=wrong_bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>()+"\n";
                        fs::write(dir.join(format!("{id}-wrong.hex")),hex).unwrap();
                    }
                    let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                    for field in metadata.as_object().unwrap().keys() {
                        let mut changed = metadata.clone();
                        changed[field] = json!("forged");
                        assert!(
                            import_proofs(
                                &serde_json::to_vec(&changed).unwrap(),
                                p.certificate_bytes(),
                                vir
                            )
                            .is_err(),
                            "{field}"
                        );
                    }
                    let too_large = vec![0; 16 * 1024 * 1024 + 1];
                    assert!(import_proofs(
                        &too_large,
                        &[],
                        vir
                    )
                    .is_err());
                    assert!(import_proofs(
                        &[],
                        &too_large,
                        vir
                    )
                    .is_err());
                }
                previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
                if let Some(dir) = &output {
                    fs::write(dir.join(format!("{id}.json")), p.canonical_bytes()).unwrap();
                    let hex = p
                        .certificate_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                        + "\n";
                    fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
                }
                contexts += 1;
                count += p.proofs().len();
                pending_states += p.pending_type_instances().len();
                println!(
                    "Original concrete type proofs {id}: {} supplied; {} internal states pending",
                    p.proofs().len(),
                    p.pending_type_instances().len()
                );
            }
            assert_eq!((contexts, count, pending_states), if with_storage { (45, 87, 0) } else { (45, 81, 6) });
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn csharp_03_t06_w09_concrete_types_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_CONCRETE_TYPES_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/concrete-types");
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut count = 0;
    let mut pending = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_concrete_types(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        let internal = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_type_instances().iter().collect::<Vec<_>>(),
            vc.instances()
                .iter()
                .filter(|d| internal.contains(d.instance_id.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.instance)
                .collect::<Vec<_>>(),
            vc.instances()
                .iter()
                .filter(|d| !internal.contains(d.instance_id.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind == "concrete_type_equivalence"
                    && !internal.contains(s.owner_id.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind != "concrete_type_equivalence"
                    || internal.contains(s.owner_id.as_str()))
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_proof_ids(),
            vc.sequents()
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let independent = generate_csharp_practical_ordinary_public_domains(vir).unwrap();
        assert_eq!(p.public_domains(), independent.definitions());
        assert_eq!(p.source_clauses(), independent.source_clauses());
        let before =
            mpk_cert::decode_canonical_certificate(independent.certificate_bytes()).unwrap();
        structural_equivalence_tests::same_definition_closure(
            &before,
            &cert,
            &independent
                .definitions()
                .iter()
                .map(|d| d.valid_definition.clone())
                .collect(),
        )
        .unwrap();
        for d in p.definitions() {
            assert!(vc.definition_names().contains(&d.symbol));
            assert_eq!(d.carrier.type_id, d.instance.instance_id);
            assert_ne!(d.definition, d.public_domain);
            let c = p
                .conditions()
                .iter()
                .find(|c| c.sequent.owner_id == d.instance.instance_id)
                .unwrap();
            assert!(c.sequent.assumptions.is_empty());
            assert_eq!(c.sequent.subjects.len(), 1);
            assert_eq!(c.sequent.goals.len(), 1);
            assert_eq!(c.sequent.subjects[0].type_id, d.instance.instance_id);
        }
        assert_eq!(
            import_csharp_practical_ordinary_concrete_types(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((metadata, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_concrete_types(metadata, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "binding-vc-option" {
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for field in metadata.as_object().unwrap().keys() {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_concrete_types(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err(),
                    "{field}"
                );
            }
            let mut changed = p.certificate_bytes().to_vec();
            *changed.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_concrete_types(
                &p.canonical_bytes(),
                &changed,
                vir
            )
            .is_err());
            let oversized = vec![0; 16 * 1024 * 1024 + 1];
            assert!(import_csharp_practical_ordinary_concrete_types(&oversized, &[], vir).is_err());
            assert!(import_csharp_practical_ordinary_concrete_types(&[], &oversized, vir).is_err());
        }
        count += p.conditions().len();
        pending += p.pending_type_instances().len();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!("concrete type definitions complete {id}");
    }
    assert_eq!(rows.len(), 45);
    assert_eq!(count, 81);
    assert_eq!(pending, 6);
    assert_eq!(count + pending, 87);
    let data = json!({"sources":rows,"conditions":count,"pending_type_conditions":pending});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/concrete-types/certificates.json"),
            data
        );
    }
}

// Change only the type predicate in an AST test copy. The original condition
// must compare both actual predicates, even when one side denies membership.
fn replace_predicate(cert: &mut Certificate, name: &str, truth: bool) {
    let d = cert
        .declarations
        .iter()
        .position(|d| cert.name_table[d.name as usize] == name)
        .unwrap();
    let DeclarationKind::Def { value, .. } = cert.declarations[d].kind else {
        panic!()
    };
    let TermNode::Lam { ty, .. } = cert.term_table[value as usize] else {
        panic!()
    };
    let leaf = if truth {
        "Std.Bool.true"
    } else {
        "Std.Bool.false"
    };
    let global = cert
        .declarations
        .iter()
        .position(|d| cert.name_table[d.name as usize] == leaf)
        .unwrap() as u32;
    let body = cert.term_table.len() as u32;
    cert.term_table.push(TermNode::Const {
        global,
        levels: vec![],
    });
    let value = cert.term_table.len() as u32;
    cert.term_table.push(TermNode::Lam { ty, body });
    let DeclarationKind::Def { value: old, .. } = &mut cert.declarations[d].kind else {
        panic!()
    };
    *old = value;
}

#[test]
fn csharp_03_t06_w09_concrete_types_original_predicates() {
    let bundle = b();
    let mut observations = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut mutants = 0;
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_concrete_types(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let carriers = p
            .public_domains()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            let c = p
                .conditions()
                .iter()
                .find(|c| c.sequent.owner_id == d.instance.instance_id)
                .unwrap();
            let mut values = (0..3)
                .map(|seed| {
                    let value = relation_tests::sample(
                        &d.instance.instance_id,
                        seed,
                        &carriers,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let (depth, ones) = projection_tests::sparse_storage(&value, &carriers);
                    sparse_cube(depth, ones)
                })
                .collect::<Vec<_>>();
            values.extend([false, true].map(|truth| {
                if d.carrier.depth == 0 {
                    V::Bit(truth)
                } else {
                    V::UniformCube(truth, d.carrier.depth)
                }
            }));
            let mut no = cert.clone();
            let mut yes = cert.clone();
            replace_predicate(&mut no, &d.definition, false);
            replace_predicate(&mut yes, &d.definition, true);
            for value in values {
                let expected = bit(run(&cert, &d.public_domain, vec![value.clone()]));
                assert_eq!(
                    bit(run(&cert, &d.definition, vec![value.clone()])),
                    expected,
                    "{id}"
                );
                assert!(
                    bit(run(&cert, &c.condition_definition, vec![value.clone()])),
                    "{id}"
                );
                let changed = if expected { &no } else { &yes };
                assert!(
                    !bit(run(changed, &c.condition_definition, vec![value])),
                    "{id}: changed concrete definition was ignored"
                );
                observations += 1;
                mutants += 1;
                accepted += usize::from(expected);
                rejected += usize::from(!expected);
            }
        }
        eprintln!("concrete type predicates complete {id}");
    }
    assert_eq!(observations, 405);
    assert_eq!(mutants, observations);
    assert!(accepted > 0 && rejected > 0);
    eprintln!("concrete type predicates: {observations} observations, {accepted} admitted, {rejected} rejected, {mutants} false mutated conditions");
}
