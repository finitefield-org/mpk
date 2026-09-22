use super::*;
use core_eval::V;

fn selected(kind: &str) -> bool {
    matches!(
        kind,
        "member_agreement"
            | "exactly_one_arm"
            | "tag_payload_agreement"
            | "bound"
            | "canonical_order_and_uniqueness"
            | "nonempty_invalid"
    )
}

#[test]
fn csharp_03_t06_w09_binding_conditions_original_source_certificates() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BINDING_CONDITIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-conditions");
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut kinds = BTreeMap::<String, usize>::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources() {
        eprintln!("binding conditions start {id}");
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
        let p = generate_csharp_practical_ordinary_binding_conditions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| selected(&s.kind))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| !selected(&s.kind))
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
            assert_eq!(c.assumption_definitions.len(), c.sequent.assumptions.len());
            assert_eq!(c.goal_definitions.len(), c.sequent.goals.len());
            *kinds.entry(c.sequent.kind.clone()).or_default() += 1;
        }
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        // The composition must preserve every transitive guard/order definition
        // and public clause, not merely predicate names or sampled truth values.
        let orders = generate_csharp_practical_ordinary_binding_orders(vir).unwrap();
        assert_eq!(p.projections(), orders.projections());
        assert_eq!(p.predicates(), orders.predicates());
        let before = mpk_cert::decode_canonical_certificate(orders.certificate_bytes()).unwrap();
        let roots = orders
            .predicates()
            .iter()
            .map(|d| d.definition.clone())
            .chain(
                orders
                    .projections()
                    .iter()
                    .map(|d| d.project_definition.clone()),
            )
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        let old_dir = if vc
            .definition_names()
            .iter()
            .any(|s| s.starts_with("Mpk.CSharp.Binding.CanonicalOrder."))
        {
            "binding-orders"
        } else {
            "binding-guards"
        };
        let hex =
            |bytes: &[u8]| bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n";
        assert_eq!(
            fs::read_to_string(
                fixture
                    .parent()
                    .unwrap()
                    .join(old_dir)
                    .join(format!("{id}.hex"))
            )
            .unwrap(),
            hex(orders.certificate_bytes())
        );
        let domains = generate_csharp_practical_ordinary_public_domains(vir).unwrap();
        assert_eq!(p.public_domains(), domains.definitions());
        let before = mpk_cert::decode_canonical_certificate(domains.certificate_bytes()).unwrap();
        let roots = domains
            .definitions()
            .iter()
            .map(|d| d.valid_definition.clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_binding_conditions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_conditions(meta, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "binding-vc-boundary_field" {
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for field in metadata.as_object().unwrap().keys() {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_binding_conditions(
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
            assert!(import_csharp_practical_ordinary_binding_conditions(
                &p.canonical_bytes(),
                &corrupt,
                vir
            )
            .is_err());
            assert!(import_csharp_practical_ordinary_binding_conditions(
                &vec![0; 16 * 1024 * 1024 + 1],
                &[],
                vir
            )
            .is_err());
            assert!(import_csharp_practical_ordinary_binding_conditions(
                &[],
                &vec![0; 16 * 1024 * 1024 + 1],
                vir
            )
            .is_err());
        }
        let actual_hex = hex(p.certificate_bytes());
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), actual_hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                actual_hex
            );
        }
        eprintln!(
            "binding conditions complete {id}: {} conditions, {} proofs pending",
            p.conditions().len(),
            p.pending_proof_ids().len()
        );
        rows.push(json!({"id":id,"metadata":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(rows.len(), 45);
    assert_eq!(kinds.len(), 6);
    let data = json!({"sources":rows,"condition_kinds":kinds});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-conditions/certificates.json"),
            data
        );
    }
}

// Interpret the original VC's Boolean connectives in Rust. Atomic predicates
// and projections are evaluated through their separately preserved definitions;
// no emitted assumption/goal/condition definition is used as the oracle.
fn original(
    term: &ContractTerm,
    cert: &mpk_cert::encode::Certificate,
    value: &V,
    symbols: &BTreeMap<String, String>,
) -> V {
    if let ContractTerm::Var { index, .. } = term {
        assert_eq!(*index, 0);
        return value.clone();
    }
    let mut head = term;
    let mut args = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = head
    {
        args.push(argument.as_ref());
        head = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = head else {
        panic!("unsupported original VC term")
    };
    let values = args
        .iter()
        .map(|arg| original(arg, cert, value, symbols))
        .collect::<Vec<_>>();
    let bools = || values.iter().cloned().map(bit).collect::<Vec<_>>();
    match name.as_str() {
        "Mpk.CSharp.Bool.true" => {
            assert!(values.is_empty());
            V::Bit(true)
        }
        "Mpk.CSharp.Bool.false" => {
            assert!(values.is_empty());
            V::Bit(false)
        }
        "Mpk.CSharp.Bool.Not" => {
            let [a]: [bool; 1] = bools().try_into().unwrap();
            V::Bit(!a)
        }
        "Mpk.CSharp.Bool.And" => {
            let [a, b]: [bool; 2] = bools().try_into().unwrap();
            V::Bit(a && b)
        }
        "Mpk.CSharp.Bool.Or" => {
            let [a, b]: [bool; 2] = bools().try_into().unwrap();
            V::Bit(a || b)
        }
        _ => run(cert, &symbols[name], values),
    }
}

#[test]
fn csharp_03_t06_w09_binding_conditions_original_sequents() {
    let bundle = b();
    let mut kinds = BTreeSet::new();
    let mut observations = 0;
    let mut false_goals = 0;
    let mut vacuous = 0;
    // Four original sources cover all six composed kinds. Existing guard/order
    // suites own exhaustive scalar, collection-capacity and ordering semantics.
    for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
        matches!(
            id.as_str(),
            "binding-vc-boundary_field"
                | "binding-vc-bounded_sequence"
                | "binding-vc-ordered_map"
                | "binding-vc-validation"
        )
    }) {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_binding_conditions(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let symbols = p
            .predicates()
            .iter()
            .map(|p| (p.symbol.clone(), p.definition.clone()))
            .chain(
                p.public_domains()
                    .iter()
                    .map(|d| (d.symbol.clone(), d.valid_definition.clone())),
            )
            .chain(p.projections().iter().map(|p| {
                (
                    p.projection.project.id.clone(),
                    p.project_definition.clone(),
                )
            }))
            .collect();
        for condition in p.conditions() {
            kinds.insert(condition.sequent.kind.clone());
            let [subject] = condition.sequent.subjects.as_slice() else {
                panic!()
            };
            for seed in 0..4 {
                let sample = relation_tests::sample(
                    &subject.type_id,
                    seed,
                    &types,
                    &facts,
                    emitted.closure().closed(),
                );
                let (depth, bits) = sparse_storage(&sample, &types);
                let mut samples = vec![sparse_cube(depth, bits)];
                if seed == 0 {
                    // Deliberately invalid storage includes unknown tags,
                    // padding and over-capacity lengths. This exercises false
                    // goals and implication's false-assumption branch.
                    samples.push(V::UniformCube(true, depth));
                }
                for value in samples {
                    let mut assumed = true;
                    let mut goals = true;
                    for (terms, definitions, all) in [
                        (
                            &condition.sequent.assumptions,
                            &condition.assumption_definitions,
                            &mut assumed,
                        ),
                        (
                            &condition.sequent.goals,
                            &condition.goal_definitions,
                            &mut goals,
                        ),
                    ] {
                        for (term, definition) in terms.iter().zip(definitions) {
                            let expected = bit(original(term, &cert, &value, &symbols));
                            assert_eq!(
                                bit(run(&cert, definition, vec![value.clone()])),
                                expected,
                                "{id}: {definition}"
                            );
                            *all &= expected;
                            observations += 1;
                        }
                    }
                    false_goals += usize::from(!goals);
                    vacuous += usize::from(!assumed && !goals);
                    assert_eq!(
                        bit(run(&cert, &condition.condition_definition, vec![value])),
                        !assumed || goals,
                        "{id}: {} seed {seed}",
                        condition.sequent.id
                    );
                    observations += 1;
                }
            }
        }
        eprintln!("binding conditions original sequents {id}: {observations} observations");
    }
    assert_eq!(kinds.len(), 6);
    assert!(false_goals > 0);
    assert!(vacuous > 0);
    eprintln!("binding condition observations: {observations}, false goals: {false_goals}, false-assumption conditions: {vacuous}");
}
