use super::*;
use core_eval::V;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

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
        assert!(p.pending_type_instances().is_empty(), "{id}");
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.instance)
                .collect::<Vec<_>>(),
            vc.instances().iter().collect::<Vec<_>>()
        );
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind == "concrete_type_equivalence")
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind != "concrete_type_equivalence")
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
    assert_eq!(count, 87);
    let data = json!({"sources":rows,"conditions":count});
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
    assert_eq!(observations, 435);
    assert_eq!(mutants, observations);
    assert!(accepted > 0 && rejected > 0);
    eprintln!("concrete type predicates: {observations} observations, {accepted} admitted, {rejected} rejected, {mutants} false mutated conditions");
}
