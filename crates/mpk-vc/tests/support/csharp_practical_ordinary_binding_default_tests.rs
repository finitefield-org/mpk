use super::*;
use core_eval::{apply, V};
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

// Change only the actual default in a test copy. A full Boolean cube makes both
// the declared arm and the representation/padding conjunct independently false.
// These are semantic mutation observations, never accepted source certificates.
fn replace_default_bit(cert: &mut Certificate, definition: &str, depth: u32, address: usize) {
    fn push(cert: &mut Certificate, term: TermNode) -> u32 {
        let id = cert.term_table.len() as u32;
        cert.term_table.push(term);
        id
    }
    fn constant(cert: &mut Certificate, name: &str) -> u32 {
        let global = cert
            .declarations
            .iter()
            .position(|d| cert.name_table[d.name as usize] == name)
            .unwrap() as u32;
        push(
            cert,
            TermNode::Const {
                global,
                levels: vec![],
            },
        )
    }
    fn body(cert: &mut Certificate, depth: u32, level: u32, address: usize) -> u32 {
        if level == depth {
            return constant(cert, "Std.Bool.true");
        }
        let zero = constant(cert, "Std.Bool.false");
        let next = body(cert, depth, level + 1, address);
        let selector = push(cert, TermNode::Var(depth - 1 - level));
        let rec = constant(cert, "Std.Bool.rec");
        let (no, yes) = if address & (1 << level) == 0 {
            (next, zero)
        } else {
            (zero, next)
        };
        push(
            cert,
            TermNode::App {
                function: rec,
                arguments: vec![no, yes, selector],
            },
        )
    }
    assert!(address < 1usize << depth);
    let mut value = body(cert, depth, 0, address);
    let ty = constant(cert, "Std.Bool");
    for _ in 0..depth {
        value = push(cert, TermNode::Lam { ty, body: value });
    }
    let d = cert
        .declarations
        .iter_mut()
        .find(|d| cert.name_table[d.name as usize] == definition)
        .unwrap();
    let DeclarationKind::Def {
        value: original, ..
    } = &mut d.kind
    else {
        panic!()
    };
    *original = value;
}

#[test]
fn csharp_03_t06_w09_binding_defaults_original_closed_conditions() {
    let bundle = b();
    let mut defaults = 0;
    let mut bits = 0;
    let mut conditions = 0;
    let mut declaration_only = 0;
    let mut source_false_arm_true = 0;
    let mut source_true_arm_false = 0;
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
        let p = generate_csharp_practical_ordinary_binding_defaults(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let independent = generate_csharp_practical_ordinary_public_domains(emitted.vir()).unwrap();
        let domain_cert =
            mpk_cert::decode_canonical_certificate(independent.certificate_bytes()).unwrap();
        for actual in p
            .actual_defaults()
            .iter()
            .filter(|d| d.definition.is_some())
        {
            let d = p
                .defaults()
                .iter()
                .find(|d| d.carrier.type_id == actual.source_type_id)
                .unwrap();
            let source = facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == actual.source_type_id)
                .unwrap();
            assert!(
                !source["recursive_default"].is_null(),
                "{id}: exact captured CLR default"
            );
            let definition = actual.definition.as_ref().unwrap();
            let value = run(&cert, definition, vec![]);
            let depth = d.carrier.depth;
            let addresses: Vec<Vec<bool>> = if depth <= 10 {
                (0..1usize << depth)
                    .map(|n| (0..depth).map(|i| n & (1 << i) != 0).collect())
                    .collect()
            } else {
                [vec![false; depth as usize], vec![true; depth as usize]]
                    .into_iter()
                    .chain((0..depth).map(|i| (0..depth).map(|j| i == j).collect()))
                    .collect()
            };
            for address in addresses {
                let mut leaf = value.clone();
                for selector in address {
                    leaf = apply(&cert, leaf, V::Bit(selector));
                }
                assert!(!bit(leaf), "{id}: actual default leaf");
                bits += 1;
            }
            defaults += 1;
            let Some(c) = p
                .conditions()
                .iter()
                .find(|c| c.sequent.owner_id == actual.projection_id)
            else {
                continue;
            };
            let domain = independent
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == actual.source_type_id)
                .unwrap();
            let zero = if depth == 0 {
                V::Bit(false)
            } else {
                V::UniformCube(false, depth)
            };
            let expected_domain = bit(run(&domain_cert, &domain.valid_definition, vec![zero]));
            assert_eq!(
                bit(run(&cert, &c.goal_definitions[0], vec![])),
                expected_domain,
                "{id}: source default membership"
            );
            // The captured actual tag and frozen declared arm are independently
            // reconciled by the validated source importer. Preserve that result.
            assert!(
                bit(run(&cert, &c.goal_definitions[1], vec![])),
                "{id}: actual default arm"
            );
            assert_eq!(
                bit(run(&cert, &c.condition_definition, vec![])),
                expected_domain,
                "{id}: original closed sequent"
            );
            conditions += 1;
            declaration_only += usize::from(!d.public_candidate_admitted);
            if matches!(
                id.as_str(),
                "binding-vc-option" | "extra-lookup-nullable" | "binding-vc-boundary_field"
            ) {
                assert!(depth <= 8);
                for address in 0..1usize << depth {
                    let mut changed = cert.clone();
                    replace_default_bit(&mut changed, definition, depth, address);
                    let source_valid = bit(run(&changed, &c.goal_definitions[0], vec![]));
                    let arm_valid = bit(run(&changed, &c.goal_definitions[1], vec![]));
                    assert_eq!(
                        bit(run(&changed, &c.condition_definition, vec![])),
                        source_valid && arm_valid,
                        "{id}:{address}"
                    );
                    source_false_arm_true += usize::from(!source_valid && arm_valid);
                    source_true_arm_false += usize::from(source_valid && !arm_valid);
                }
            }
        }
        eprintln!("binding default observations {id}");
    }
    assert!(defaults > 0 && bits > 0 && conditions > 0 && declaration_only > 0);
    assert!(source_false_arm_true > 0 && source_true_arm_false > 0);
    eprintln!("binding default observations: {defaults} actual defaults, {bits} bits, {conditions} closed conditions, {declaration_only} unproven declaration flags, {source_false_arm_true} invalid source with matching arm, {source_true_arm_false} valid source with wrong arm");
}

#[test]
fn csharp_03_t06_w09_binding_defaults_original_source_certificates() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BINDING_DEFAULTS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-defaults");
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut compiled = 0;
    let mut pending = BTreeMap::<String, usize>::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources() {
        eprintln!("binding defaults start {id}");
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
        let p = generate_csharp_practical_ordinary_binding_defaults(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        assert_eq!(p.actual_defaults().len(), vc.representations().len());
        let mut expected = vec![];
        let mut expected_pending = vec![];
        for sequent in vc.sequents().iter().filter(|s| s.kind == "actual_default") {
            assert!(sequent.subjects.is_empty());
            assert!(sequent.assumptions.is_empty());
            let rep = vc
                .representations()
                .iter()
                .find(|r| r.projection.id == sequent.owner_id)
                .unwrap();
            let default = p
                .defaults()
                .iter()
                .find(|d| d.carrier.type_id == rep.projection.source_type_id)
                .unwrap();
            let actual = p
                .actual_defaults()
                .iter()
                .find(|d| d.projection_id == sequent.owner_id)
                .unwrap();
            assert_eq!(actual.source_type_id, rep.projection.source_type_id);
            assert_eq!(actual.declared_arm, rep.binding["default_arm"]);
            assert_eq!(
                actual.symbol,
                format!("Mpk.CSharp.Binding.ActualDefault.{}", actual.source_type_id)
            );
            assert_eq!(
                actual.definition,
                default
                    .structural_candidate
                    .as_ref()
                    .map(|c| c.definition.clone())
            );
            let reason = if rep.binding["default_arm"] == "ineligible" {
                assert_eq!(sequent.goals.len(), 1);
                let forbidden = format!(
                    "Mpk.CSharp.Binding.DefaultUseForbidden.{}",
                    sequent.owner_id
                );
                assert_eq!(head(&sequent.goals[0]), forbidden);
                assert!(p.unresolved_vc_symbols().contains(&forbidden));
                Some(OrdinaryBindingDefaultPendingReason::SourceUseProofRequired)
            } else if default.structural_candidate.is_none() {
                Some(OrdinaryBindingDefaultPendingReason::UnrepresentableClrDefault)
            } else {
                assert_eq!(sequent.goals.len(), 2);
                expected.push(sequent);
                None
            };
            if let Some(reason) = reason {
                *pending
                    .entry(
                        serde_json::to_value(&reason)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .into(),
                    )
                    .or_default() += 1;
                expected_pending.push(OrdinaryBindingDefaultPending {
                    sequent: sequent.clone(),
                    reason,
                });
            }
        }
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(p.pending_defaults(), expected_pending);
        let resolved = expected
            .iter()
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| !resolved.contains(s.id.as_str()))
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
        for c in p.conditions() {
            assert!(c.assumption_definitions.is_empty());
            assert_eq!(c.goal_definitions.len(), 2);
        }
        compiled += p.conditions().len();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        // Preserve the complete independent implementations, including recursive
        // public clauses and actual defaults. Names alone cannot establish this.
        let defaults = generate_csharp_practical_ordinary_defaults(vir).unwrap();
        assert_eq!(p.defaults(), defaults.definitions());
        let before = mpk_cert::decode_canonical_certificate(defaults.certificate_bytes()).unwrap();
        let roots = defaults
            .definitions()
            .iter()
            .flat_map(|d| {
                d.structural_candidate
                    .iter()
                    .map(|c| c.definition.clone())
                    .chain(std::iter::once(d.public_admission_definition.clone()))
            })
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        let guards = generate_csharp_practical_ordinary_binding_guards(vir).unwrap();
        assert_eq!(p.projections(), guards.projections());
        assert_eq!(p.predicates(), guards.predicates());
        let before = mpk_cert::decode_canonical_certificate(guards.certificate_bytes()).unwrap();
        let roots = guards
            .predicates()
            .iter()
            .map(|d| d.definition.clone())
            .chain(
                guards
                    .projections()
                    .iter()
                    .map(|d| d.project_definition.clone()),
            )
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        let domains = generate_csharp_practical_ordinary_public_domains(vir).unwrap();
        assert_eq!(p.public_domains(), domains.definitions());
        let before = mpk_cert::decode_canonical_certificate(domains.certificate_bytes()).unwrap();
        let roots = domains
            .definitions()
            .iter()
            .map(|d| d.valid_definition.clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        let roots = p
            .conditions()
            .iter()
            .map(|c| c.condition_definition.clone())
            .collect();
        let dependencies =
            structural_equivalence_tests::same_definition_closure(&cert, &cert, &roots).unwrap();
        for d in p.defaults() {
            assert!(!dependencies.contains(&d.public_admission_definition));
        }
        assert_eq!(
            import_csharp_practical_ordinary_binding_defaults(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_defaults(meta, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "binding-vc-boundary_field" {
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for field in metadata.as_object().unwrap().keys() {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_binding_defaults(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err(),
                    "{field}"
                );
            }
            let mut corrupt = p.certificate_bytes().to_vec();
            *corrupt.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_binding_defaults(
                &p.canonical_bytes(),
                &corrupt,
                vir
            )
            .is_err());
            assert!(import_csharp_practical_ordinary_binding_defaults(
                &vec![0; 16 * 1024 * 1024 + 1],
                &[],
                vir
            )
            .is_err());
            assert!(import_csharp_practical_ordinary_binding_defaults(
                &[],
                &vec![0; 16 * 1024 * 1024 + 1],
                vir
            )
            .is_err());
        }
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!("binding defaults complete {id}: {} conditions, {} defaults unresolved, {} proofs pending", p.conditions().len(), p.pending_defaults().len(), p.pending_proof_ids().len());
        rows.push(json!({"id":id,"metadata":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(rows.len(), 45);
    assert!(compiled > 0);
    assert!(pending["source_use_proof_required"] > 0);
    let data = json!({"sources":rows,"conditions":compiled,"pending_defaults":pending});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-defaults/certificates.json"),
            data
        );
    }
}
