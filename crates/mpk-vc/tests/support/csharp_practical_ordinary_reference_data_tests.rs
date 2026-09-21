//! Nullable references preserve source SSA and exact NullReferenceException edges.
use super::super::relation_tests::{sample, storage};
use super::*;

fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_REFERENCE_DATA_OUT") {
        fs::create_dir_all(&root).unwrap();
        fs::write(Path::new(&root).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/reference-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let replay = read("data-phase/data-stage-replay.json");
    let ids = [
        (
            "guarded",
            "a7451d5f65ff8bf6a31838c8de9846d5f3cbff7c153fbe9c0369f89cab1b4b7f",
        ),
        (
            "conditional",
            "c72927bb0e68b6a99eca939f5ecb42d603cd238af5cd76da7602a7674398091c",
        ),
        (
            "suppressed",
            "c7de411b37114b07c8bdd184ab54cee24112de5d8949d5435896f62372e400db",
        ),
        (
            "property",
            "e8809d9ef15a0877162dfd6f6491ffb83dafae536e5a6030b5b11d765daae511",
        ),
    ];
    ids.into_iter()
        .map(|(label, id)| {
            let row = replay
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap();
            assert!(row["outcome"].get("reject").is_none());
            (label.into(), row.clone(), row["outcome"]["facts"].clone())
        })
        .collect()
}
#[test]
fn csharp_03_t06_w09_reference_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_reference_data_original_source() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(true))
        .unwrap()
        .join()
        .unwrap();
}
fn verify(runtime: bool) {
    let bundle = b();
    let filter = std::env::var("MPK_W09_REFERENCE_DATA_CONTEXT").ok();
    let case_filter = std::env::var("MPK_W09_REFERENCE_DATA_CASE").ok();
    let mut observations = 0usize;
    let mut manifest = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;

    for (id, request, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_reference_data(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let foundations = generate_csharp_practical_ordinary_outcomes(vir).unwrap();
        let available = foundations
            .definitions()
            .iter()
            .filter(|d| {
                OutcomeModel::new(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &d.carrier.type_id,
                )
                .unwrap()
                .role()
                    == "option"
            })
            .flat_map(|d| &d.operations)
            .filter(|o| o.operation_id.ends_with(".value"))
            .map(|o| (o.argument_type_ids[0].as_str(), o))
            .collect::<BTreeMap<_, _>>();
        assert!(!p.definitions().is_empty(), "{id}");
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| d.family == DataDefinitionFamily::NullableOutcome
                    && available.contains_key(
                        d.signature
                            .id
                            .strip_prefix("reference.value.")
                            .unwrap_or("")
                    ))
                .collect::<Vec<_>>()
        );
        for d in p.definitions() {
            assert_eq!(
                &d.operation,
                available[d.source.signature.argument_type_ids[0].as_str()]
            );
        }
        let included = p
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_definition_ids(),
            data.definitions()
                .iter()
                .filter(|d| !included.contains(d.id.as_str()))
                .map(|d| d.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.operations().iter().map(|o| &o.source).collect::<Vec<_>>(),
            data.operations()
                .iter()
                .filter(|o| included.contains(o.definition_id.as_str()))
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_reference_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_reference_data(meta, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        if runtime && filter.as_ref().is_none_or(|s| s == &id) {
            for d in p.definitions() {
                let sig = &d.source.signature;
                let suffix = "value";
                assert_eq!(
                    sig.id,
                    format!("reference.value.{}", sig.argument_type_ids[0])
                );
                eprintln!("reference data start: {id} {}", sig.id);
                let started = std::time::Instant::now();
                let carrier = &sig.argument_type_ids[0];
                let model = OutcomeModel::new(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    carrier,
                )
                .unwrap();
                let mut cases = vec![(vec![model.construct("none", None).unwrap()], None)];
                for seed in 0..3 {
                    let payload = sample(
                        &sig.normal_result_type_id,
                        seed,
                        &types,
                        &facts,
                        emitted.closure().closed(),
                    );
                    cases.push((
                        vec![model.construct("some", Some(payload.clone())).unwrap()],
                        Some(payload),
                    ));
                }
                for (values, expected) in cases {
                    let failed = expected.is_none();
                    let case = if failed { "none" } else { "some" };
                    if case_filter
                        .as_ref()
                        .is_some_and(|selected| selected != case)
                    {
                        continue;
                    }
                    eprintln!("reference case: {id} {case}");
                    let raw = expected.as_ref().map_or_else(
                        || vec![false; 1 << types[&sig.normal_result_type_id].depth],
                        |v| storage(v, &types),
                    );
                    let inputs = values
                        .iter()
                        .map(|v| input(storage(v, &types)))
                        .collect::<Vec<_>>();
                    for failure in &d.failure_definitions {
                        assert_eq!(bit(run(&cert, failure, inputs.clone())), failed);
                    }
                    for wrong in [None, Some(0), Some(raw.len() - 1)] {
                        let mut bits = raw.clone();
                        if let Some(i) = wrong {
                            bits[i] = !bits[i];
                        }
                        let mut args = inputs.clone();
                        args.push(input(bits));
                        if !failed {
                            assert_eq!(
                                bit(run(&cert, &d.relation_definition, args.clone())),
                                wrong.is_none(),
                                "{id} {suffix} definition"
                            );
                        }
                        for o in p
                            .operations()
                            .iter()
                            .filter(|o| o.source.definition_id == d.source.id)
                        {
                            assert_eq!(
                                o.source
                                    .subjects
                                    .iter()
                                    .map(|s| &s.type_id)
                                    .collect::<Vec<_>>(),
                                sig.argument_type_ids
                                    .iter()
                                    .chain(std::iter::once(&sig.normal_result_type_id))
                                    .collect::<Vec<_>>()
                            );
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_guard"], args.clone())),
                                !failed
                            );
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                                failed || wrong.is_none()
                            );
                            if !failed {
                                assert_eq!(
                                    bit(run(
                                        &cert,
                                        &o.predicates["success_relation"],
                                        args.clone()
                                    )),
                                    wrong.is_none()
                                );
                            }
                            for (index, check) in o.source.checks.iter().enumerate() {
                                assert_eq!(check.check.id, "exception.null_receiver");
                                assert_eq!(
                                    check.check.failure_type_id.as_deref(),
                                    Some("System.NullReferenceException")
                                );
                                assert_eq!(
                                    check
                                        .exceptional_successor
                                        .as_ref()
                                        .unwrap()
                                        .exception_type_id,
                                    "System.NullReferenceException"
                                );
                                assert_eq!(index, 0);
                                assert!(bit(run(
                                    &cert,
                                    &o.predicates["check.0.prefix"],
                                    args.clone()
                                )));
                                assert_eq!(
                                    bit(run(&cert, &o.predicates["check.0.failed"], args.clone())),
                                    failed
                                );
                                assert_eq!(
                                    bit(run(&cert, &o.predicates["check.0.guard"], args.clone())),
                                    failed
                                );
                            }
                        }
                        observations += 1;
                    }
                }
                eprintln!(
                    "reference data end: {id} {} {:.3}s",
                    sig.id,
                    started.elapsed().as_secs_f64()
                );
            }
        }
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "data_vc_sha256",
            "certificate_sha256",
        ] {
            let mut changed = meta.clone();
            changed[key] = json!("changed");
            assert!(import_csharp_practical_ordinary_reference_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = meta.clone();
        changed["definitions"][0]["relation_definition"] = json!("changed");
        assert!(import_csharp_practical_ordinary_reference_data(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        if let Some(index) = p
            .definitions()
            .iter()
            .position(|d| !d.failure_definitions.is_empty())
        {
            let mut changed = meta.clone();
            changed["definitions"][index]["failure_definitions"][0] = json!("changed");
            assert!(import_csharp_practical_ordinary_reference_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        if !p.operations().is_empty() {
            let mut changed = meta.clone();
            changed["operations"][0]["source"]["checks"][0]["check"]["failure_type_id"] =
                json!("System.InvalidOperationException");
            assert!(import_csharp_practical_ordinary_reference_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for field in ["function_id", "node_id", "normal_successor_id"] {
            if !p.operations().is_empty() {
                let mut changed = meta.clone();
                changed["operations"][0]["source"][field] = json!("changed");
                assert!(import_csharp_practical_ordinary_reference_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_reference_data(
            &p.canonical_bytes(),
            &damaged,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            format!(
                "{}\n",
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
            .as_bytes(),
        );
        manifest.push(json!({"id":id,"definitions":p.definitions().len(),"use_points":p.operations().len(),"pending_definition_ids":p.pending_definition_ids(),"certificate_sha256":meta["certificate_sha256"],"bytes":p.certificate_bytes().len()}));
        eprintln!(
            "reference data context {id}: {} definitions, {} SSA points",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(manifest.len(), 4);
    if runtime {
        assert!(observations > 0, "selector matched no reference operation");
    }
    output(
        "certificates.json",
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    eprintln!(
        "reference data: {} contexts, {observations} result observations",
        manifest.len()
    );
}
