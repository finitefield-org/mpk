use super::*;
use core_eval::{apply, V};

fn observed(value: &MonomorphicValue) -> Value {
    fn normalize(value: &mut Value) {
        match value {
            Value::Array(values) => values.iter_mut().for_each(normalize),
            Value::Object(fields) if fields.get("kind") == Some(&json!("decimal_bits")) => {
                let mut coefficient: u128 =
                    fields["coefficient"].as_str().unwrap().parse().unwrap();
                let mut scale = fields["scale"].as_u64().unwrap();
                while scale > 0 && coefficient.is_multiple_of(10) {
                    coefficient /= 10;
                    scale -= 1;
                }
                fields.insert("coefficient".into(), json!(coefficient.to_string()));
                fields.insert("scale".into(), json!(scale));
                if coefficient == 0 {
                    fields.insert("negative".into(), json!(false));
                }
            }
            Value::Object(fields) => fields.values_mut().for_each(normalize),
            _ => {}
        }
    }
    let mut result = serde_json::to_value(value).unwrap();
    normalize(&mut result);
    result
}

fn selected(kind: &str) -> bool {
    matches!(
        kind,
        "projection_total"
            | "reconstruction"
            | "source_round_trip"
            | "semantic_round_trip"
            | "member_reconstruction"
            | "identity_projection"
    )
}

#[test]
fn csharp_03_t06_w09_binding_reconstruction_original_sources() {
    reconstruction_sources(sources(), (45, 37), true);
}

#[test]
fn csharp_03_t06_w09_binding_reconstruction_noninjective_candidate() {
    let cases = sources()
        .into_iter()
        .filter(|(id, _, _)| id == "remapped-boundary-sequence")
        .collect();
    reconstruction_sources(cases, (1, 2), false);
}

#[test]
fn csharp_03_t06_w09_binding_reconstruction_condition_compiler_preserves_pins() {
    let bundle = b();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-reconstruction");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("certificates.json")).unwrap()).unwrap();
    let mut count = 0;
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
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let completions = vir
            .binding_projections()
            .iter()
            .filter(|p| p.binding_id != "binding.identity")
            .map(|p| p.source_type_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|ty| relation_tests::sample(&ty, 2, &types, &facts, emitted.closure().closed()))
            .collect::<Vec<_>>();
        let p =
            generate_csharp_practical_ordinary_binding_reconstruction(vir, &completions).unwrap();
        let expected = manifest["sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),
            expected["metadata"],
            "{id}"
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        assert_eq!(
            fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
            hex,
            "{id}"
        );
        count += 1;
    }
    assert_eq!(count, 45);
}

fn reconstruction_sources(
    cases: Vec<(String, Value, Value)>,
    expected: (usize, usize),
    pins: bool,
) {
    let bundle = b();
    let output =
        std::env::var_os("MPK_W09_BINDING_RECONSTRUCTION_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-reconstruction");
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut total_candidates = 0;
    let mut total_obligations = 0;
    let mut observations = 0;
    let mut negative_members = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in cases {
        eprintln!("binding reconstruction start {id}");
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
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let bindings: Value =
            serde_json::from_slice(emitted.closure().bindings().canonical_bytes()).unwrap();
        let oracle = ProjectionOracle {
            facts: &facts,
            bindings: bindings["bindings"].as_array().unwrap().clone(),
            closed: emitted.closure().closed(),
        };
        let completions = vir
            .binding_projections()
            .iter()
            .filter(|p| p.binding_id != "binding.identity")
            .map(|p| p.source_type_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            // The independent rebuild oracle needs stored completion elements
            // for every observed source index. Seed 2 contains two elements,
            // covering every source sample below, including recursive wrappers.
            .map(|ty| relation_tests::sample(&ty, 2, &types, &facts, oracle.closed))
            .collect::<Vec<_>>();
        let p = generate_csharp_practical_ordinary_binding_reconstruction(vir, &completions)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = vc.binding_vcs();
        assert_eq!(
            p.pending_proof_ids(),
            vc.sequents()
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.obligations()
                .iter()
                .map(|o| &o.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| selected(&s.kind))
                .collect::<Vec<_>>()
        );
        assert_eq!(p.candidates().len(), vir.binding_projections().len());
        let base = generate_csharp_practical_ordinary_binding_rebuilds(vir).unwrap();
        let before = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
        let roots = base
            .definitions()
            .iter()
            .flat_map(|d| {
                [
                    d.rebuild_definition.clone(),
                    d.projection.project_definition.clone(),
                ]
            })
            .collect();
        structural_equivalence_tests::same_definition_closure(&before, &cert, &roots).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_binding_reconstruction(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir,
                &completions
            )
            .unwrap(),
            p
        );
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_reconstruction(
                m,
                c,
                vir,
                &completions
            )
            .is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "remapped-boundary-sequence" {
            for field in [
                "schema",
                "source_ir_sha256",
                "foundation_sha256",
                "binding_vc_sha256",
                "construction_sha256",
                "completions",
                "candidates",
                "source_clauses",
                "public_domains",
                "predicates",
                "obligations",
                "unresolved_vc_symbols",
                "pending_proof_ids",
                "certificate_sha256",
            ] {
                let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_binding_reconstruction(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir,
                        &completions
                    )
                    .is_err(),
                    "{field}"
                );
            }
            let mut corrupt = p.certificate_bytes().to_vec();
            *corrupt.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_binding_reconstruction(
                &p.canonical_bytes(),
                &corrupt,
                vir,
                &completions
            )
            .is_err());
            assert!(generate_csharp_practical_ordinary_binding_reconstruction(
                vir,
                &completions[1..]
            )
            .is_err());
            let mut duplicate = completions.clone();
            duplicate.push(completions[0].clone());
            assert!(
                generate_csharp_practical_ordinary_binding_reconstruction(vir, &duplicate).is_err()
            );
            let mut foreign = completions.clone();
            foreign.push(MonomorphicValue::Bool {
                type_id: ty("bool"),
                value: true,
            });
            assert!(
                generate_csharp_practical_ordinary_binding_reconstruction(vir, &foreign).is_err()
            );
            let mut reordered = completions.clone();
            reordered.reverse();
            assert_eq!(
                generate_csharp_practical_ordinary_binding_reconstruction(vir, &reordered).unwrap(),
                p
            );
        }
        for candidate in p.candidates() {
            let projection = &candidate.rebuild.projection;
            assert!(!p
                .unresolved_vc_symbols()
                .contains(&projection.projection.reconstruct.id));
            let seed = completions
                .iter()
                .find(|v| v.type_id() == projection.source_carrier.type_id);
            for n in 0..3 {
                let source = relation_tests::sample(
                    &projection.source_carrier.type_id,
                    n,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let semantic = oracle.project(&source, &projection.semantic_carrier.type_id);
                let expected = seed
                    .map(|seed| rebuild_tests::rebuild(&oracle, &semantic, seed))
                    .unwrap_or_else(|| semantic.clone());
                let (depth, wanted) = sparse_storage(&expected, &types);
                let (sd, semantic_bits) = sparse_storage(&semantic, &types);
                let actual = run(
                    &cert,
                    &candidate.reconstruct_definition,
                    vec![sparse_cube(sd, semantic_bits)],
                );
                let mut addresses = if depth <= 10 {
                    (0..1 << depth).collect::<BTreeSet<_>>()
                } else {
                    BTreeSet::from([0, (1 << depth) - 1])
                };
                addresses.extend((0..depth).map(|i| 1 << i));
                for &at in &wanted {
                    addresses.extend([at, at.saturating_sub(1), (at + 1).min((1 << depth) - 1)]);
                }
                for at in addresses {
                    let mut leaf = actual.clone();
                    for i in 0..depth {
                        leaf = apply(&cert, leaf, V::Bit(at & (1 << i) != 0));
                    }
                    assert_eq!(bit(leaf), wanted.contains(&at), "{id} {n} address {at}");
                    observations += 1;
                }
                for obligation in p.obligations().iter().filter(|o| {
                    o.sequent.owner_id == projection.projection.id
                        && o.sequent.kind == "source_round_trip"
                }) {
                    let (depth, bits) = sparse_storage(&source, &types);
                    let value = sparse_cube(depth, bits);
                    let admitted = bit(run(
                        &cert,
                        &obligation.assumption_definitions[0],
                        vec![value.clone()],
                    ));
                    let goal = bit(run(
                        &cert,
                        &obligation.goal_definitions[0],
                        vec![value.clone()],
                    ));
                    assert_eq!(
                        goal,
                        observed(&expected) == observed(&source),
                        "{id} {n} round trip"
                    );
                    assert_eq!(
                        bit(run(&cert, &obligation.condition_definition, vec![value])),
                        !admitted || goal
                    );
                }
            }
            if id == "remapped-boundary-sequence" {
                let original = seed.unwrap();
                let mut changed = original.clone();
                let MonomorphicValue::Product { fields, .. } = &mut changed else {
                    panic!()
                };
                *fields.iter_mut().find(|f| f.name == "Extra").unwrap().value =
                    MonomorphicValue::Signed {
                        type_id: ty("i32"),
                        value: "73".into(),
                    };
                assert_eq!(
                    oracle.project(original, &projection.semantic_carrier.type_id),
                    oracle.project(&changed, &projection.semantic_carrier.type_id)
                );
                let member = facts["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == projection.source_carrier.type_id)
                    .unwrap()["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|m| m["name"] == "Extra")
                    .unwrap();
                let member = csharp_practical_stored_member_id(
                    &projection.source_carrier.type_id,
                    member["name"].as_str().unwrap(),
                    &member["type"],
                    member["storage"].as_str().unwrap(),
                )
                .unwrap();
                let failed = p
                    .obligations()
                    .iter()
                    .filter(|o| {
                        (o.sequent.kind == "source_round_trip"
                            && o.sequent.owner_id == projection.projection.id)
                            || (o.sequent.kind == "member_reconstruction"
                                && o.sequent.owner_id.ends_with(&member))
                    })
                    .collect::<Vec<_>>();
                assert_eq!(failed.len(), 2);
                for o in &failed {
                    for (source, expected) in [(original, true), (&changed, false)] {
                        let (depth, bits) = sparse_storage(source, &types);
                        let value = sparse_cube(depth, bits);
                        assert!(bit(run(
                            &cert,
                            &o.assumption_definitions[0],
                            vec![value.clone()]
                        )));
                        assert_eq!(
                            bit(run(&cert, &o.goal_definitions[0], vec![value.clone()])),
                            expected
                        );
                        assert_eq!(
                            bit(run(&cert, &o.condition_definition, vec![value])),
                            expected
                        );
                    }
                    negative_members += 1;
                }
                let mut alternate = completions.clone();
                *alternate
                    .iter_mut()
                    .find(|v| v.type_id() == changed.type_id())
                    .unwrap() = changed.clone();
                assert!(import_csharp_practical_ordinary_binding_reconstruction(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    vir,
                    &alternate
                )
                .is_err());
                let alternative =
                    generate_csharp_practical_ordinary_binding_reconstruction(vir, &alternate)
                        .unwrap();
                let alternative_cert =
                    mpk_cert::decode_canonical_certificate(alternative.certificate_bytes())
                        .unwrap();
                for o in failed {
                    for (source, expected) in [(original, false), (&changed, true)] {
                        let (depth, bits) = sparse_storage(source, &types);
                        assert_eq!(
                            bit(run(
                                &alternative_cert,
                                &o.condition_definition,
                                vec![sparse_cube(depth, bits)]
                            )),
                            expected,
                            "changing the completion must change the rejected public source value"
                        );
                    }
                }
            }
        }
        total_candidates += p.candidates().len();
        total_obligations += p.obligations().len();
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = output.as_ref().filter(|_| pins) {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else if pins {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id": id, "metadata": meta, "terms": cert.term_table.len(), "declarations": cert.declarations.len()}));
        eprintln!(
            "binding reconstruction complete {id}: {} candidates, {} obligations",
            p.candidates().len(),
            p.obligations().len()
        );
    }
    assert_eq!(
        (rows.len(), total_candidates, negative_members),
        (expected.0, expected.1, 4)
    );
    let manifest = json!({"sources":rows,"candidates":total_candidates,"obligations":total_obligations,"observed_bits":observations,"negative_conditions":negative_members});
    if let Some(dir) = output.filter(|_| pins) {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else if pins {
        assert_eq!(
            read("ordinary-foundation/binding-reconstruction/certificates.json"),
            manifest
        );
    }
    eprintln!("binding reconstruction: {total_candidates} candidates, {total_obligations} obligations, {observations} bit observations, {negative_members} negative conditions");
}
