use super::*;
use core_eval::{bit, run, sparse_cube};

#[test]
fn csharp_03_t06_w09_binding_relations_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_BINDING_RELATIONS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/binding-relations");
    let mut rows = vec![];
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
        let p = generate_csharp_practical_ordinary_binding_relations(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(
            p.projections(),
            generate_csharp_practical_ordinary_binding_projections(vir)
                .unwrap()
                .definitions()
        );
        assert_eq!(p.agreements().len(), p.projections().len());
        let symbols = p
            .predicates()
            .iter()
            .map(|d| d.symbol.as_str())
            .chain(
                p.projections()
                    .iter()
                    .map(|d| d.projection.project.id.as_str()),
            )
            .chain(
                p.projections()
                    .iter()
                    .filter(|p| p.reconstruct_definition.is_some())
                    .map(|p| p.projection.reconstruct.id.as_str()),
            )
            .collect::<BTreeSet<_>>();
        let full_vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full_vc.binding_vcs();
        for symbol in vc.definition_names().iter().filter(|s| {
            s.starts_with("Mpk.CSharp.Binding.ObserveEqual.")
                || s.starts_with("Mpk.CSharp.Binding.Equal.")
        }) {
            let type_id = symbol
                .strip_prefix("Mpk.CSharp.Binding.ObserveEqual.")
                .or_else(|| symbol.strip_prefix("Mpk.CSharp.Binding.Equal."))
                .unwrap();
            let internal = emitted.closure().closed().entries().iter().any(|e| {
                e["instance_id"] == type_id
                    && e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1"
            });
            if internal {
                assert!(p.unresolved_vc_symbols().contains(symbol));
            } else {
                assert!(
                    symbols.contains(symbol.as_str()),
                    "{id}: public comparison left unresolved: {symbol}"
                );
            }
        }
        for agreement in p.agreements() {
            assert_eq!(
                agreement.comparison_symbol,
                format!(
                    "Mpk.CSharp.Binding.ObserveEqual.{}",
                    agreement.semantic_type_id
                )
            );
            assert!(symbols.contains(agreement.comparison_symbol.as_str()));
        }
        if id == "float-make-commutation" {
            let normal = vc
                .sequents()
                .iter()
                .filter(|s| s.kind == "operation_normal_commutation")
                .collect::<Vec<_>>();
            assert_eq!(normal.len(), 1);
            let comparison = head(&normal[0].goals[1]);
            let semantic = csharp_practical_closed_instance_id(
                &bundle,
                &instance("ordered_entry", vec![primitive("bool"), primitive("f32")]),
            )
            .unwrap();
            assert_eq!(
                comparison,
                format!("Mpk.CSharp.Binding.ObserveEqual.{semantic}")
            );
            let agreement = p
                .agreements()
                .iter()
                .find(|d| d.semantic_type_id == semantic)
                .unwrap();
            assert_eq!(comparison, agreement.comparison_symbol);
            assert_eq!(p.projections().len(), 3);
            assert_eq!(
                p.projections()
                    .iter()
                    .filter(|p| p.reconstruct_definition.is_some())
                    .count(),
                2
            );
            assert!(comparison.starts_with("Mpk.CSharp.Binding.ObserveEqual."));
            assert_eq!(
                vc.representations()
                    .iter()
                    .map(|r| r.commutations.len())
                    .sum::<usize>(),
                1
            );
        }
        assert_eq!(
            p.unresolved_vc_symbols(),
            vc.definition_names()
                .iter()
                .filter(|s| !symbols.contains(s.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        );
        for rep in vc.representations() {
            assert!(symbols.contains(
                format!(
                    "Mpk.CSharp.Binding.ObserveEqual.{}",
                    rep.projection.source_type_id
                )
                .as_str()
            ));
            assert!(symbols.contains(
                format!(
                    "Mpk.CSharp.Binding.Equal.{}",
                    rep.projection.semantic_type_id
                )
                .as_str()
            ));
            for member in &rep.reconstruction_member_ids {
                assert!(
                    symbols.contains(format!("Mpk.CSharp.Binding.MemberEqual.{member}").as_str())
                );
            }
            assert!(p
                .unresolved_vc_symbols()
                .contains(&rep.projection.reconstruct.id));
        }
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let standalone = generate_csharp_practical_ordinary_binding_projections(vir).unwrap();
        let standalone =
            mpk_cert::decode_canonical_certificate(standalone.certificate_bytes()).unwrap();
        let roots = p
            .projections()
            .iter()
            .map(|d| d.project_definition.clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&standalone, &cert, &roots).unwrap();
        if !p.projections().is_empty() {
            let observations = generate_csharp_practical_ordinary_observations(vir).unwrap();
            let observations =
                mpk_cert::decode_canonical_certificate(observations.certificate_bytes()).unwrap();
            let roots = p
                .predicates()
                .iter()
                .filter(|p| p.symbol.starts_with("Mpk.CSharp.Binding.ObserveEqual."))
                .map(|p| p.definition.clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&observations, &cert, &roots)
                .unwrap();
        }

        assert_eq!(
            import_csharp_practical_ordinary_binding_relations(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        // Symbol signatures and the unresolved inventory are acceptance inputs,
        // not optional annotations that a caller may remove or retarget.
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "binding_vc_sha256",
            "projections",
            "predicates",
            "agreements",
            "unresolved_vc_symbols",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut changed = metadata.clone();
            changed[field] = json!("forged");
            assert!(
                import_csharp_practical_ordinary_binding_relations(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err(),
                "{id}: {field}"
            );
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_binding_relations(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_binding_relations(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        eprintln!(
            "binding relations {id}: {} predicates, {} agreements, {} terms",
            p.predicates().len(),
            p.agreements().len(),
            cert.term_table.len()
        );
        rows.push(json!({"id":id,"metadata":metadata,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(rows.len(), 45);
    let data = json!({"sources":rows});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/binding-relations/certificates.json"),
            data
        );
    }
}

#[test]
fn csharp_03_t06_w09_binding_relations_source_observation_and_commutation() {
    let bundle = b();
    let mut count = 0;
    let mut saw_nan = false;
    let mut saw_extra = false;
    for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
        matches!(
            id.as_str(),
            "bool-float-entry"
                | "binding-vc-result"
                | "remapped-boundary-sequence"
                | "float-make-commutation"
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
        let p = generate_csharp_practical_ordinary_binding_relations(emitted.vir()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
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
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let check = |name: &str, a: &MonomorphicValue, b: &MonomorphicValue, expected: bool| {
            let (ad, av) = sparse_storage(a, &types);
            let (bd, bv) = sparse_storage(b, &types);
            assert_eq!(
                bit(run(
                    &cert,
                    name,
                    vec![sparse_cube(ad, av), sparse_cube(bd, bv)]
                )),
                expected,
                "{id} {name}"
            );
        };
        for d in p.agreements() {
            // The small nested wrapper is exercised in full here. Sequence
            // result/domain cases retain their existing full-capacity suites.
            if types[&d.source_type_id].depth > 10 {
                continue;
            }
            let eq = generate_structural_program(
                &bundle,
                emitted.closure().roots(),
                oracle.closed,
                &d.semantic_type_id,
            )
            .unwrap();
            let observe = p
                .predicates()
                .iter()
                .find(|p| {
                    p.symbol == format!("Mpk.CSharp.Binding.ObserveEqual.{}", d.source_type_id)
                })
                .unwrap();
            if id == "bool-float-entry" {
                let value = |bits: &str| {
                    let mut v =
                        relation_tests::sample(&d.source_type_id, 0, &types, &facts, oracle.closed);
                    let MonomorphicValue::Product { fields, .. } = &mut v else {
                        panic!()
                    };
                    *fields.iter_mut().find(|f| f.name == "Value").unwrap().value =
                        MonomorphicValue::F32Bits {
                            type_id: ty("f32"),
                            bits: bits.into(),
                        };
                    v
                };
                let semeq = p
                    .predicates()
                    .iter()
                    .find(|p| {
                        p.symbol == format!("Mpk.CSharp.Binding.Equal.{}", d.semantic_type_id)
                    })
                    .unwrap();
                for (left, right, observation, equality) in [
                    ("80000000", "00000000", false, true),
                    ("7fc00000", "7fc00000", true, false),
                    ("7fc00000", "7fc00001", false, false),
                    ("7f800001", "7fc00000", false, false),
                    ("3f800000", "3f800000", true, true),
                ] {
                    let a = value(left);
                    let b = value(right);
                    let pa = oracle.project(&a, &d.semantic_type_id);
                    let pb = oracle.project(&b, &d.semantic_type_id);
                    check(&d.result_agreement_definition, &a, &pb, observation);
                    check(&semeq.definition, &pa, &pb, observation);
                    assert_eq!(eq.structural_equal(&pa, &pb).unwrap(), equality);
                    count += 2;
                }
            }
            for seed in 0..4 {
                let v =
                    relation_tests::sample(&d.source_type_id, seed, &types, &facts, oracle.closed);
                validate_monomorphic_value(&bundle, emitted.closure().roots(), oracle.closed, &v)
                    .unwrap();
                let projected = oracle.project(&v, &d.semantic_type_id);
                let matches = eq.structural_equal(&projected, &projected).unwrap();
                check(&d.result_agreement_definition, &v, &projected, true);
                let semantic_equal = p
                    .predicates()
                    .iter()
                    .find(|p| {
                        p.symbol == format!("Mpk.CSharp.Binding.Equal.{}", d.semantic_type_id)
                    })
                    .unwrap();
                check(&semantic_equal.definition, &projected, &projected, true);
                count += 1;
                count += 1;
                check(&observe.definition, &v, &v, true);
                count += 1;
                if id == "bool-float-entry" && !matches {
                    saw_nan = true;
                }
                let different = relation_tests::sample(
                    &d.source_type_id,
                    (seed + 1) % 4,
                    &types,
                    &facts,
                    oracle.closed,
                );
                let different = oracle.project(&different, &d.semantic_type_id);
                check(
                    &d.result_agreement_definition,
                    &v,
                    &different,
                    projected == different,
                );
                count += 1;
                if id == "remapped-boundary-sequence" {
                    let mut changed = v.clone();
                    let MonomorphicValue::Product { fields, .. } = &mut changed else {
                        panic!()
                    };
                    *fields.iter_mut().find(|f| f.name == "Extra").unwrap().value =
                        MonomorphicValue::Signed {
                            type_id: ty("i32"),
                            value: "73".into(),
                        };
                    check(&observe.definition, &v, &changed, false);
                    count += 1;
                    check(&d.result_agreement_definition, &changed, &projected, true);
                    count += 1;
                    let original = facts["types"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|t| t["id"] == d.source_type_id)
                        .unwrap();
                    for member in original["members"].as_array().unwrap() {
                        let member_id = csharp_practical_stored_member_id(
                            &d.source_type_id,
                            member["name"].as_str().unwrap(),
                            &member["type"],
                            member["storage"].as_str().unwrap(),
                        )
                        .unwrap();
                        let pred = p
                            .predicates()
                            .iter()
                            .find(|p| {
                                p.symbol == format!("Mpk.CSharp.Binding.MemberEqual.{member_id}")
                            })
                            .unwrap();
                        check(&pred.definition, &v, &changed, member["name"] != "Extra");
                        count += 1;
                    }
                    saw_extra = true;
                }
            }
        }
    }
    assert!(saw_nan && saw_extra);
    assert_eq!(count, 126);
    eprintln!("binding relation ordinary observations: {count}");
}

#[test]
fn csharp_03_t06_w09_binding_result_source_requests() {
    let bundle = b();
    let own=csharp_practical_declaration_id(&json!({"kind":"type","namespace":"BindingOperationCases","owner":"","name":"Rep","parameter_type_ids":[],"result_type_id":""})).unwrap();
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"BindingOperationCases","owner":own,"name":"Make","parameter_type_ids":[ty("bool"),ty("f32")],"result_type_id":own})).unwrap();
    let code="namespace BindingOperationCases;public readonly struct Rep{public readonly bool Key;public readonly float Value;public Rep(bool key,float value){Key=key;Value=value;}public static Rep Make(bool key,float value){return new Rep(key,value);}}\n";
    let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
    let binding = SemanticBindingInput {
        source_type_id: own.clone(),
        source_content_sha256: captures.entries()[0].raw_sha256().into(),
        role: "ordered_entry".into(),
        member_map: vec![
            SemanticBindingMember {
                role: "key".into(),
                member_id: csharp_practical_stored_member_id(
                    &own,
                    "Key",
                    &primitive("bool"),
                    "readonly_field",
                )
                .unwrap(),
            },
            SemanticBindingMember {
                role: "value".into(),
                member_id: csharp_practical_stored_member_id(
                    &own,
                    "Value",
                    &primitive("f32"),
                    "readonly_field",
                )
                .unwrap(),
            },
        ],
        inferred_argument_ids: vec![ty("bool"), ty("f32")],
        tag_arms: vec![],
        default_arm: "ineligible".into(),
        bounds: vec![],
        operation_map: vec![SemanticOperationMapping {
            operation: "make".into(),
            member_id: root.clone(),
        }],
        enum_arms: BTreeMap::new(),
    };
    let sidecar = build_semantic_bindings(&plain, &captures, vec![binding])
        .unwrap()
        .canonical_bytes()
        .to_vec();
    let (context, captures) =
        support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
    let requests = json!([{"id":"float-make-commutation","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_BINDING_RESULT_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/binding-result-sources/requests.json")).unwrap(),bytes);
    }
}

fn head(t: &ContractTerm) -> &str {
    match t {
        ContractTerm::App { function, .. } => head(function),
        ContractTerm::Const { name, .. } => name,
        _ => panic!("expected applied predicate"),
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let mut rows = projection_sources();
    let requests = read("ordinary-foundation/binding-result-sources/requests.json");
    let responses = read("ordinary-foundation/binding-result-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    rows.push((
        "float-make-commutation".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    rows
}

#[path = "csharp_practical_ordinary_binding_guard_tests.rs"]
mod guard_tests;

#[path = "csharp_practical_ordinary_binding_rebuild_tests.rs"]
mod rebuild_tests;
