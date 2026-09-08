//! Carrier certificates are type/helper checks, not application proof receipts.
use super::*;
use sha2::{Digest, Sha256};

#[test]
fn csharp_03_t06_w09_carriers_original_inputs_and_mutations() {
    let b = b();
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/carriers");
    let output = std::env::var_os("MPK_W09_CARRIERS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut goldens = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut saw_validation_bound = false;
    for family in [
        "binding-vc",
        "exception-vc",
        "transition-vc",
        "construction-vc",
    ] {
        let requests = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        for req in requests.as_array().unwrap() {
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == req["id"])
                .unwrap();
            if response.get("reject").is_some() {
                continue;
            }
            let id = format!("{family}-{}", req["id"].as_str().unwrap());
            let (context, captures) = support::replay_context(&b, req);
            let source = ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(&response["facts"]).unwrap(),
            )
            .unwrap();
            let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
            let vir = emitted.vir();
            let p = generate_csharp_practical_ordinary_carriers(vir)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let bytes = p.certificate_bytes();
            let metadata = p.canonical_bytes();
            assert_eq!(
                import_csharp_practical_ordinary_carriers(&metadata, bytes, vir).unwrap(),
                p
            );
            let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            validate_csharp_practical_certificate_structure(&c).unwrap();
            let ids = p
                .carriers()
                .iter()
                .map(|c| c.type_id.as_str())
                .collect::<BTreeSet<_>>();
            assert_eq!(ids.len(), p.carriers().len());
            let wire: Value = serde_json::from_slice(vir.canonical_bytes()).unwrap();
            for entry in wire["expanded_foundation"].as_array().unwrap() {
                assert!(ids.contains(entry["instance_id"].as_str().unwrap()));
            }
            for carrier in p.carriers() {
                if let OrdinaryShape::Sum { arms } = &carrier.shape {
                    for arm in arms {
                        for field in &arm.fields {
                            if let OrdinaryShape::RoleBound { maximum, .. } = field.shape {
                                assert_eq!(maximum, 256);
                                saw_validation_bound = true;
                            }
                        }
                    }
                }
            }
            for source_type in response["facts"]["types"].as_array().unwrap() {
                let id = source_type["id"].as_str().unwrap();
                let Some(carrier) = p.carriers().iter().find(|c| c.type_id == id) else {
                    continue;
                };
                if source_type["kind"] == "enum" {
                    assert_eq!(carrier.shape, OrdinaryShape::Bits { width: 32 });
                } else {
                    let OrdinaryShape::Product { fields } = &carrier.shape else {
                        panic!()
                    };
                    let expected = source_type["members"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|m| {
                            csharp_practical_stored_member_id(
                                id,
                                m["name"].as_str().unwrap(),
                                &m["type"],
                                m["storage"].as_str().unwrap(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        fields.iter().map(|f| &f.id).collect::<Vec<_>>(),
                        expected.iter().collect::<Vec<_>>()
                    );
                }
            }
            // Every concrete field/tag is bound to the original validated input,
            // not to a caller-provided carrier inventory or accepted byte hash.
            let v: Value = serde_json::from_slice(&metadata).unwrap();
            let mut missing = v.clone();
            missing["carriers"].as_array_mut().unwrap().pop();
            let mut shape = v.clone();
            shape["carriers"][0]["depth"] = json!(254);
            let mut foundation = v.clone();
            foundation["foundation_sha256"] = json!("0".repeat(64));
            for m in [missing, shape, foundation] {
                assert!(import_csharp_practical_ordinary_carriers(
                    &serde_json::to_vec(&m).unwrap(),
                    bytes,
                    vir
                )
                .is_err());
            }
            let mut altered = bytes.to_vec();
            *altered.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_carriers(&metadata, &altered, vir).is_err());
            if let Some((m, c)) = &previous {
                assert!(import_csharp_practical_ordinary_carriers(m, c, vir).is_err());
            }
            previous = Some((metadata.clone(), bytes.to_vec()));
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if let Some(dir) = &output {
                fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
            } else {
                assert_eq!(
                    fs::read_to_string(fixture_root.join(format!("{id}.hex"))).unwrap(),
                    hex,
                    "{id}"
                );
            }
            goldens.push(json!({"id":id,"source_ir_sha256":vir.hash(),
                "metadata_sha256":format!("{:x}", Sha256::digest(&metadata)),
                "certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(bytes)),
                "types":p.carriers(),"terms":c.term_table.len(),"declarations":c.declarations.len()}));
        }
    }
    assert!(saw_validation_bound);
    if let Some(dir) = &output {
        fs::write(
            dir.join("goldens.json"),
            serde_json::to_vec_pretty(&goldens).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            json!(goldens),
            read("ordinary-foundation/carriers/goldens.json")
        );
    }
}

#[test]
fn csharp_03_t06_w09_integer_source_linkage_rejects_substitution() {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut definitions = 0;
    for id in [
        "positive_constructor",
        "broken_constructor",
        "positive_initializer",
    ] {
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
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_integers(vir).unwrap();
        let bytes = p.canonical_bytes();
        definitions += p.definitions().len();
        let operation_wire: Value =
            serde_json::from_slice(emitted.operations().operations().canonical_bytes()).unwrap();
        let expected = operation_wire["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .filter(|id| id.starts_with("integer.") || id.starts_with("boolean."))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.operation.id.as_str())
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            import_csharp_practical_ordinary_integers(&bytes, p.certificate_bytes(), vir).unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
        ] {
            let mut changed = original.clone();
            changed[key] = json!("0".repeat(64));
            assert!(import_csharp_practical_ordinary_integers(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = original;
        changed["definitions"] = json!([]);
        if !p.definitions().is_empty() {
            assert!(import_csharp_practical_ordinary_integers(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_integers(&bytes, &bad, vir).is_err());
        if let Some((metadata, certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_integers(metadata, certificate, vir).is_err());
        }
        previous = Some((bytes, p.certificate_bytes().to_vec()));
    }
    assert!(definitions > 0);
}

#[test]
fn csharp_03_t06_w09_temporal_source_linkage_rejects_substitution() {
    let bundle = b();
    let mut rows = read("data-phase/data-stage-replay.json");
    let requests = read("data-phase/data-sidecar-requests.json");
    let responses = read("data-phase/data-sidecar-responses.json");
    for id in [
        "1d420cdfab591490568d24323e3474d89ce7d8ada62f3693a657647c6235afe5",
        "e321919bbcc2dc31606907fd91a8d1f373ab1480dd506a38ed8ffd0c698138f7",
    ] {
        let mut row = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone();
        row["outcome"] = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone();
        rows.as_array_mut().unwrap().push(row);
    }
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let cases = [
        (
            "14e30f193aa9e5aac03d640da54ad6ad22365f42b40d55c0d138ce8020de95f3",
            "duration.add",
        ),
        (
            "dec3e56d4ee60ff5a7696151300fdf3cb592d5dee385e0da40e9b2a8ce46ebc9",
            "time.add_duration",
        ),
        (
            "1d420cdfab591490568d24323e3474d89ce7d8ada62f3693a657647c6235afe5",
            "instant.add_duration",
        ),
        (
            "e321919bbcc2dc31606907fd91a8d1f373ab1480dd506a38ed8ffd0c698138f7",
            "instant.add_duration",
        ),
    ];
    for (id, required) in cases {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&row["outcome"]["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_temporal(vir).unwrap();
        let bytes = p.canonical_bytes();
        let actual = p
            .definitions()
            .iter()
            .map(|d| d.operation.id.as_str())
            .collect::<BTreeSet<_>>();
        assert!(actual.contains(required), "{id}: {actual:?}");
        let wire: Value =
            serde_json::from_slice(emitted.operations().operations().canonical_bytes()).unwrap();
        let expected = wire["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .filter(|id| {
                ["time.", "duration.", "instant."]
                    .iter()
                    .any(|prefix| id.starts_with(prefix))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            import_csharp_practical_ordinary_temporal(&bytes, p.certificate_bytes(), vir).unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
        ] {
            let mut changed = original.clone();
            changed[key] = json!("0".repeat(64));
            assert!(import_csharp_practical_ordinary_temporal(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for mutation in 0..4 {
            let mut changed = original.clone();
            match mutation {
                0 => changed["definitions"] = json!([]),
                1 => changed["definitions"][0]["operation"]["ordered_checks"] = json!([]),
                2 => changed["definitions"][0]["result_definition"] = json!("forged"),
                _ => {
                    changed["definitions"][0]["operation"]["argument_type_ids"][0] =
                        json!("mpk.csharp.value.u64.v1")
                }
            }
            if changed != original {
                assert!(import_csharp_practical_ordinary_temporal(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_temporal(&bytes, &bad, vir).is_err());
        if let Some((metadata, certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_temporal(metadata, certificate, vir).is_err());
        }
        previous = Some((bytes, p.certificate_bytes().to_vec()));
    }
}

#[test]
fn csharp_03_t06_w09_calendar_source_linkage_rejects_substitution() {
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, required) in [
        (
            "be730f79173090110933bb19af1646bba7965bd2c6c713c7fde66473dfc748e1",
            "date.construct",
        ),
        (
            "ce80aad8a9692d5285f1e510eeaa2f6c3b210d31167db7dcf30c7ef299959d8a",
            "date.add_months",
        ),
        (
            "0769b3fa2547d9ee8cd3f9ac8a9fc05dbe289aca239cab39ffef5cf392b9b3dc",
            "date.add_years",
        ),
        (
            "cbb74344e9ab18afe5cc41c88bc1b311b2bfb4714ecbc2d6896ff7d152af6efd",
            "guid.compare",
        ),
        (
            "d79dcdedffe1bfa594f950fa5f0516b89479b895e44ab97336539521c3868d2f",
            "guid.empty",
        ),
        (
            "7dbdb7767caf25ae31e51bfb43a841dc5fb0d502605d12080ba61b0777f04389",
            "guid.equal",
        ),
        (
            "05e148b20eb9017bdab9a113ab247301fa596ef171961443f1c68727025fe98f",
            "date.day_of_week",
        ),
    ] {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&row["outcome"]["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_calendar(vir).unwrap();
        let bytes = p.canonical_bytes();
        let actual = p
            .definitions()
            .iter()
            .map(|d| d.operation.id.as_str())
            .collect::<BTreeSet<_>>();
        assert!(actual.contains(required), "{id}: {actual:?}");
        let wire: Value =
            serde_json::from_slice(emitted.operations().operations().canonical_bytes()).unwrap();
        let expected = wire["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .filter(|id| {
                ["date.", "guid.", "day_of_week."]
                    .iter()
                    .any(|prefix| id.starts_with(prefix))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            import_csharp_practical_ordinary_calendar(&bytes, p.certificate_bytes(), vir).unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
        ] {
            let mut changed = original.clone();
            changed[key] = json!("0".repeat(64));
            assert!(import_csharp_practical_ordinary_calendar(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for mutation in 0..4 {
            let mut changed = original.clone();
            match mutation {
                0 => changed["definitions"] = json!([]),
                1 => changed["definitions"][0]["operation"]["ordered_checks"] = json!([]),
                2 => changed["definitions"][0]["result_definition"] = json!("forged"),
                _ => {
                    changed["definitions"][0]["operation"]["normal_result_type_id"] =
                        json!("mpk.csharp.value.string.v1")
                }
            }
            if changed != original {
                assert!(import_csharp_practical_ordinary_calendar(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_calendar(&bytes, &bad, vir).is_err());
        if let Some((metadata, certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_calendar(metadata, certificate, vir).is_err());
        }
        previous = Some((bytes, p.certificate_bytes().to_vec()));
    }
}
