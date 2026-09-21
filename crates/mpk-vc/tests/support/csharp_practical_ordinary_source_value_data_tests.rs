//! Source products use their captured member order, not semantic equality.
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
    let root = std::env::var_os("MPK_W09_SOURCE_VALUE_DATA_OUT").map(std::path::PathBuf::from);
    if let Some(root) = root {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/source-value-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_source_value_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_source_value_data_original_source() {
    verify(true);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let goldens = read("construction-vc/goldens.json");
    let mut observations = 0;
    let mut definitions = 0;
    let mut use_points = 0;
    let mut contexts = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut manifest = vec![];
    let filter = std::env::var("MPK_W09_SOURCE_VALUE_DATA_CONTEXT").ok();
    for expected in goldens.as_array().unwrap() {
        let id = expected["id"].as_str().unwrap();
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
        let facts = &response["facts"];
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_source_value_data(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let included = p
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| d.family == DataDefinitionFamily::SourceValue)
                .collect::<Vec<_>>()
        );
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
            import_csharp_practical_ordinary_source_value_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((metadata, bytes)) = &previous {
            assert!(
                import_csharp_practical_ordinary_source_value_data(metadata, bytes, vir).is_err()
            );
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        if runtime && filter.as_ref().is_none_or(|selected| selected == id) {
            for operation in p.operations() {
                let d = p
                    .definitions()
                    .iter()
                    .find(|d| d.source.id == operation.source.definition_id)
                    .unwrap();
                let sig = &d.source.signature;
                eprintln!("source value start: {id} {}", sig.id);
                for seed in 0..3 {
                    let values = sig
                        .argument_type_ids
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            sample(t, seed + i, &types, facts, emitted.closure().closed())
                        })
                        .collect::<Vec<_>>();
                    let expected = if sig.tag == ClosedOperationTag::FieldRead {
                        let MonomorphicValue::Product { fields, .. } = &values[0] else {
                            panic!()
                        };
                        let source = facts["types"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|t| t["id"] == sig.argument_type_ids[0])
                            .unwrap();
                        let member = sig.id.strip_prefix("field.read.").unwrap();
                        let index = source["members"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .position(|m| {
                                csharp_practical_stored_member_id(
                                    &sig.argument_type_ids[0],
                                    m["name"].as_str().unwrap(),
                                    &m["type"],
                                    m["storage"].as_str().unwrap(),
                                )
                                .unwrap()
                                    == member
                            })
                            .unwrap();
                        fields[index].value.as_ref().clone()
                    } else {
                        assert_eq!(sig.tag, ClosedOperationTag::ValueConstruct);
                        let source = facts["types"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|t| t["id"] == sig.normal_result_type_id)
                            .unwrap();
                        MonomorphicValue::Product {
                            type_id: sig.normal_result_type_id.clone(),
                            fields: source["members"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .zip(&values)
                                .map(|(m, v)| NamedMonomorphicValue {
                                    name: m["name"].as_str().unwrap().into(),
                                    value: Box::new(v.clone()),
                                })
                                .collect(),
                        }
                    };
                    let bits = storage(&expected, &types);
                    // Include a last-physical-bit mutation: padding is part of
                    // the result relation even when it is outside stored fields.
                    for wrong in [None, Some(0), Some(bits.len() - 1)] {
                        let mut actual = bits.clone();
                        if let Some(index) = wrong {
                            actual[index] = !actual[index];
                        }
                        let mut args = values
                            .iter()
                            .map(|v| input(storage(v, &types)))
                            .collect::<Vec<_>>();
                        args.push(input(actual));
                        assert!(bit(run(
                            &cert,
                            &operation.predicates["success_guard"],
                            args.clone()
                        )));
                        for role in ["success_relation", "success_goal"] {
                            assert_eq!(
                                bit(run(&cert, &operation.predicates[role], args.clone())),
                                wrong.is_none(),
                                "{id}: {} seed {seed} {role} {wrong:?}",
                                sig.id
                            );
                        }
                        observations += 1;
                    }
                }
                eprintln!("source value completed: {id} {}", sig.id);
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
            assert!(import_csharp_practical_ordinary_source_value_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for path in ["value_definition", "relation_definition"] {
            if !p.definitions().is_empty() {
                let mut changed = meta.clone();
                changed["definitions"][0][path] = json!("changed");
                assert!(import_csharp_practical_ordinary_source_value_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        if !p.operations().is_empty() {
            let mut changed = meta.clone();
            changed["operations"][0]["source"]["normal_successor_id"] = json!("changed");
            assert!(import_csharp_practical_ordinary_source_value_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_source_value_data(
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
        definitions += p.definitions().len();
        use_points += p.operations().len();
        contexts += 1;
        eprintln!(
            "source value context {id}: {} definitions, {} SSA points",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(contexts, 7);
    assert!(definitions > 0 && use_points > 0);
    if runtime {
        assert!(
            observations > 0,
            "selector matched no source value operation"
        );
    }
    output(
        "certificates.json",
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    eprintln!("source value data: {contexts} contexts, {definitions} definitions, {use_points} SSA points, {observations} result observations");
}
