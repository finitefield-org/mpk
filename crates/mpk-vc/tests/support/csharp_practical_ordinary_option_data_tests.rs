//! Native nullable operations retain the original SSA subjects and exception edges.
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
    if let Some(root) = std::env::var_os("MPK_W09_OPTION_DATA_OUT") {
        fs::create_dir_all(&root).unwrap();
        fs::write(Path::new(&root).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/option-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let replay = read("data-phase/data-stage-replay.json");
    let ids = [
        (
            "struct-none",
            "37b22975404961d93087124ce15175005ab6d7bb75c800f55cc7b69fd34f7d92",
        ),
        (
            "i32-value",
            "c11bc0f337639ee2a289645651ebfa4dde975bc8e85344d1c5aea4a79a927fb8",
        ),
        (
            "i32-fallback",
            "a5b91a7a4ecf505b8793a0f8d6962a5c84f9dcae0da15656711b9638323c40e4",
        ),
        (
            "i32-default",
            "59805b93313c7b578312eede63575ebc0d157e177104c8de4a58bd97fce4022e",
        ),
        (
            "nested-string",
            "c33a68ad7b9913539d964367517f235adadf5d7056bf3ee1f1cf604ee1662519",
        ),
        (
            "struct-decimal",
            "dc68f7032018233e8c69aefb0fcdb369af34040715e908b46d05dc25e1bbae7a",
        ),
    ];
    let mut rows = ids
        .into_iter()
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
        .collect::<Vec<_>>();
    let requests = read("ordinary-foundation/lifted-data/requests.json");
    let responses = read("ordinary-foundation/lifted-data/responses.json");
    for id in ["bool", "i32-checked", "i64", "f32", "f64", "decimal"] {
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none());
        rows.push((
            format!("lifted-{id}"),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    rows
}
#[test]
fn csharp_03_t06_w09_option_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_option_data_original_source() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(true))
        .unwrap()
        .join()
        .unwrap();
}
fn payloads(
    id: &str,
    types: &BTreeMap<String, OrdinaryCarrier>,
    facts: &Value,
    closed: &ClosedInstanceSet,
) -> Vec<MonomorphicValue> {
    // Equality must preserve the computed bits, including signed zero and NaN payloads.
    let floating = if id.ends_with(".f32.v1") {
        Some((
            "f32_bits",
            vec!["00000000", "80000000", "3f800000", "7fc00001", "7fc00002"],
        ))
    } else if id.ends_with(".f64.v1") {
        Some((
            "f64_bits",
            vec![
                "0000000000000000",
                "8000000000000000",
                "3ff0000000000000",
                "7ff8000000000001",
                "7ff8000000000002",
            ],
        ))
    } else {
        None
    };
    if let Some((kind, bits)) = floating {
        bits.into_iter()
            .map(|bits| {
                serde_json::from_value(json!({"kind":kind,"type_id":id,"bits":bits})).unwrap()
            })
            .collect()
    } else {
        (0..3)
            .map(|seed| sample(id, seed, types, facts, closed))
            .collect()
    }
}
fn verify(runtime: bool) {
    let bundle = b();
    let filter = std::env::var("MPK_W09_OPTION_DATA_CONTEXT").ok();
    let op_filter = std::env::var("MPK_W09_OPTION_DATA_OPERATION").ok();
    let mut observations = 0usize;
    let mut manifest = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut kinds = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_option_data(vir)
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
            .map(|o| (o.operation_id.as_str(), o))
            .collect::<BTreeMap<_, _>>();
        assert!(!p.definitions().is_empty(), "{id}");
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| d.family == DataDefinitionFamily::Foundation
                    && available.contains_key(d.signature.id.as_str()))
                .collect::<Vec<_>>()
        );
        for d in p.definitions() {
            assert_eq!(&d.operation, available[d.source.signature.id.as_str()]);
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
            import_csharp_practical_ordinary_option_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_option_data(meta, bytes, vir).is_err());
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
                let suffix = sig.id.rsplit('.').next().unwrap();
                if op_filter
                    .as_ref()
                    .is_some_and(|s| s != suffix && s != &sig.id)
                {
                    continue;
                }
                eprintln!("option data start: {id} {}", sig.id);
                let started = std::time::Instant::now();
                kinds.insert(suffix.to_owned());
                let carrier = if matches!(suffix, "none" | "some") {
                    &sig.normal_result_type_id
                } else {
                    &sig.argument_type_ids[0]
                };
                let model = OutcomeModel::new(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    carrier,
                )
                .unwrap();
                let some = available
                    .values()
                    .find(|o| o.operation_id.ends_with(".some") && &o.result_type_id == carrier)
                    .unwrap();
                let payload = &some.argument_type_ids[0];
                let payloads = payloads(payload, &types, &facts, emitted.closure().closed());
                let none = model.construct("none", None).unwrap();
                let mut options = vec![none.clone()];
                options.extend(
                    payloads
                        .iter()
                        .map(|v| model.construct("some", Some(v.clone())).unwrap()),
                );
                let mut cases = vec![];
                match suffix {
                    "none" => cases.push((vec![], Some(none))),
                    "some" => {
                        for v in &payloads {
                            cases.push((
                                vec![v.clone()],
                                Some(model.construct("some", Some(v.clone())).unwrap()),
                            ));
                        }
                    }
                    "has_value" | "value" | "value_or" => {
                        for (index, v) in options.iter().enumerate() {
                            let mut args = vec![v.clone()];
                            let expected = match suffix {
                                "has_value" => Some(MonomorphicValue::Bool {
                                    type_id: sig.normal_result_type_id.clone(),
                                    value: index != 0,
                                }),
                                "value" => model.read(v, "some").ok().cloned(),
                                "value_or" => {
                                    let fallback = payloads.last().unwrap().clone();
                                    args.push(fallback.clone());
                                    Some(model.value_or(v, fallback).unwrap())
                                }
                                _ => unreachable!(),
                            };
                            cases.push((args, expected));
                        }
                    }
                    other => panic!("unexpected option operation {other}"),
                }
                for (values, expected) in cases {
                    let failed = expected.is_none();
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
                                assert_eq!(check.check.id, "invalid_operation");
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
                    "option data end: {id} {} {:.3}s",
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
            assert!(import_csharp_practical_ordinary_option_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = meta.clone();
        changed["definitions"][0]["relation_definition"] = json!("changed");
        assert!(import_csharp_practical_ordinary_option_data(
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
            assert!(import_csharp_practical_ordinary_option_data(
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
                assert!(import_csharp_practical_ordinary_option_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_option_data(&p.canonical_bytes(), &damaged, vir)
                .is_err()
        );
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
            "option data context {id}: {} definitions, {} SSA points",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(manifest.len(), 12);
    if runtime {
        assert!(observations > 0, "selector matched no option operation");
        if filter.is_none() && op_filter.is_none() {
            assert_eq!(
                kinds,
                ["none", "some", "has_value", "value", "value_or"]
                    .map(str::to_owned)
                    .into_iter()
                    .collect()
            );
        }
    }
    output(
        "certificates.json",
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    eprintln!(
        "option data: {} contexts, {observations} result observations",
        manifest.len()
    );
}
