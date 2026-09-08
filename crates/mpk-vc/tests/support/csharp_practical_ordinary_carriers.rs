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

#[test]
fn csharp_03_t06_w09_floating_source_linkage_rejects_substitution() {
    floating_source_linkage(&[
        (
            "56d52302cbf3bc6a7c0fe7c798f4960eedb8f823995ca2c01ccf5ce6b07b1c9f",
            "floating.single.add",
        ),
        (
            "c82e3b158e3982bea08ddfe71a06694592de5168b44380230ff10a3b8458e147",
            "floating.double.add",
        ),
        (
            "5d7655018362332f047f20374264f504b2892846562721bb1f8934e65380b79a",
            "floating.double.multiply",
        ),
        (
            "cb6d43ad84c0eec06f922b7b4d3849f227b45722ea88c76c2dde2965257c9009",
            "floating.double.divide",
        ),
        (
            "64d4ee463fcfbe531f822913c1d498d9cab513f964110c22017f7c3d6a664338",
            "floating.single.remainder",
        ),
        (
            "bfa98c70faaf4387583af786ee1aaf47e6b0c231485e2a881a7d336e626d3278",
            "floating.double.remainder",
        ),
        (
            "b89ed5e739f259d8204156cfd7c2940dbc3389d8aeecc7af20319c5c9253289a",
            "floating.double.min",
        ),
        (
            "1cc09aad4452c4d54ad182b6f8bc97dea22a35c74e32262819e6ddf42f622776",
            "floating.single.is_nan",
        ),
    ]);
}

#[test]
fn csharp_03_t06_w09_floating_conversion_source_linkage_rejects_substitution() {
    floating_source_linkage(&[
        (
            "0f374d42b0b7af270138710ba14df3757ee3c519d7dc69b252630177e90d5caf",
            "numeric.conversion.single_to_double",
        ),
        (
            "1dcb0a84b0cc293550e207354eda7487ae16c11893e0791820a258eae96d9907",
            "numeric.conversion.int32_to_single",
        ),
        (
            "33463313175964d2809ec3d900bca3369925857eb1aaa146a86fac094824243f",
            "numeric.conversion.double_to_single",
        ),
        (
            "3f7473c4146111708d02d884348caaab33f8f538e493b6aa0a23ea2a6b2819a1",
            "numeric.conversion.int64_to_double",
        ),
        (
            "6873354712737a52c876eaad9734d1be9227e9eb319fb097a65fe3263e25c50f",
            "numeric.conversion.double_to_int64.checked",
        ),
        (
            "b470b0c18a66cc7d324d1b65064827d48ccf2003af4180c4ee11d4955498ec92",
            "numeric.conversion.single_to_int32.checked",
        ),
    ]);
}

fn floating_source_linkage(cases: &[(&str, &str)]) {
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for &(id, required) in cases {
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
        let p = generate_csharp_practical_ordinary_floating(vir).unwrap();
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
                [
                    "floating.single.",
                    "floating.double.",
                    "numeric.conversion.",
                ]
                .iter()
                .any(|prefix| id.starts_with(prefix))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            import_csharp_practical_ordinary_floating(&bytes, p.certificate_bytes(), vir).unwrap(),
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
            assert!(import_csharp_practical_ordinary_floating(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for mutation in 0..5 {
            let mut changed = original.clone();
            match mutation {
                0 => changed["definitions"] = json!([]),
                1 => {
                    changed["definitions"][0]["operation"]["ordered_checks"] = json!([{"id":"forged","tag":"exception","failure_type_id":"System.OverflowException"}])
                }
                2 => changed["definitions"][0]["result_definition"] = json!("forged"),
                3 => {
                    changed["definitions"][0]["operation"]["normal_result_type_id"] =
                        json!("mpk.csharp.value.string.v1")
                }
                _ => changed["definitions"][0]["operation"]["ordered_checks"] = json!([]),
            }
            if changed != original {
                assert!(import_csharp_practical_ordinary_floating(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_floating(&bytes, &bad, vir).is_err());
        if let Some((metadata, certificate)) = &previous {
            assert!(import_csharp_practical_ordinary_floating(metadata, certificate, vir).is_err());
        }
        previous = Some((bytes, p.certificate_bytes().to_vec()));
    }
}

#[test]
fn csharp_03_t06_w09_decimal_source_linkage_rejects_substitution() {
    decimal_source_linkage(&[
        (
            "37665fd6c9e9c3eee21b0fc6700d87bef8045cd225000647f47e5f6b996c1f2e",
            "decimal.conversion.int32_to_decimal",
        ),
        (
            "5cd2156d6da7d76ac732315a572313f25d66ec695aed6200fe51417952e99a9f",
            "decimal.conversion.int64_to_decimal",
        ),
        (
            "376c32018fa43b298d056ab4198e20059f75f51e8cbfad61d19d74f7d49b57ec",
            "decimal.conversion.decimal_to_int32",
        ),
        (
            "9163f49d1ce6de8951bda3216e8225768075d0460823b44c4ba16a86cd0f6894",
            "decimal.negate",
        ),
        (
            "c10b65a25ffc662d82649dea01d4b8c047a70cab24589ff879b6ae68616ee7c0",
            "decimal.plus",
        ),
        (
            "9eaef94b95284094b781e90d04b5f29cd9c0652480e3e0f97189c30342c8fda2",
            "decimal.floor",
        ),
        (
            "c06e389076390e84a5231a1e8af2d207d6ca2c410f753bc56f352309e34d9c9d",
            "decimal.truncate",
        ),
        (
            "c390fed93b3738e099050079d9e192384d1bfc0a60dc40e2673032459176bb07",
            "decimal.ceiling",
        ),
        (
            "37b1e1ef34ab5cc62f2950586c2383091eec483b1f59d07a47ecc24523c1ca11",
            "decimal.round.ToEven.2",
        ),
        (
            "8fc5631c055a1bfd6e5ee1a6b6c9eb652ffaab35922bdf21882f10150beb8994",
            "decimal.round.ToEven.1",
        ),
        (
            "39d435c3683125074f80a3d49c19663bdf8ae9900283a424830c393ed821c4e1",
            "decimal.round.ToPositiveInfinity.2",
        ),
        (
            "81f0473d7fb5133dd1967f6107881471330ffe62a308efa6824e556b9a99b980",
            "decimal.round.ToZero.2",
        ),
        (
            "8e9060e789c3a2b948cf75af9d62f2ec394d0d073c301baa74d0bdbf8556e7f2",
            "decimal.round.ToNegativeInfinity.2",
        ),
        (
            "a995ddf1cc51e65cd9c7fbe6c2512b0b813663a52a69f97bff15d4691a27e1e5",
            "decimal.round.AwayFromZero.2",
        ),
    ]);
}

fn decimal_source_linkage(cases: &[(&str, &str)]) {
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for &(id, required) in cases {
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
        let carriers = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let decimal = carriers
            .carriers()
            .iter()
            .find(|c| c.type_id == "mpk.csharp.value.decimal.v1")
            .unwrap();
        assert_eq!(decimal.depth, 9);
        let p = generate_csharp_practical_ordinary_decimal(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let metadata = p.canonical_bytes();
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
            .filter(|s| s.starts_with("decimal."))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            import_csharp_practical_ordinary_decimal(&metadata, p.certificate_bytes(), vir)
                .unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&metadata).unwrap();
        for mutation in 0..8 {
            let mut changed = original.clone();
            match mutation {
                0 => changed["source_ir_sha256"] = json!("0".repeat(64)),
                1 => changed["foundation_sha256"] = json!("0".repeat(64)),
                2 => changed["certificate_sha256"] = json!("0".repeat(64)),
                3 => changed["definitions"] = json!([]),
                4 => changed["definitions"][0]["operation"]["ordered_checks"] = json!([]),
                5 => {
                    changed["definitions"][0]["operation"]["normal_result_type_id"] =
                        json!("mpk.csharp.value.i64.v1")
                }
                6 => changed["definitions"][0]["result_definition"] = json!("forged"),
                _ => changed["definitions"][0]["operation"]["id"] = json!("decimal.round.ToEven.3"),
            }
            if changed != original {
                assert!(import_csharp_practical_ordinary_decimal(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_decimal(&metadata, &bad, vir).is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_decimal(m, c, vir).is_err());
        }
        previous = Some((metadata, p.certificate_bytes().to_vec()));
    }
}

#[test]
fn csharp_03_t06_w09_decimal_arithmetic_source_linkage_rejects_substitution() {
    decimal_source_linkage(&[
        (
            "072499d479575581ac35f2f8f4890b04df24427f5043591a586129eb1a188238",
            "decimal.remainder",
        ),
        (
            "12fd6ef71100b3141a5f195dc9fed18b7fad26052f32d394536b4afa37dd9353",
            "decimal.less",
        ),
        (
            "300c49bdb94b3da7c6b766b8b981dc9c1d3e1314ffba1ab9e35a1dba4eab5365",
            "decimal.greater",
        ),
        (
            "550ba0310e316b190e013bad80b230c4d727b19d86fee7745690b2b0ac6cbe4b",
            "decimal.multiply",
        ),
        (
            "6040e7b6fe806e2b5a7aa78f43e37f9a3ebceb67dbe5be8b922dc6e80a9a1b22",
            "decimal.add",
        ),
        (
            "8c85e8172f2e3be2bef8622eb6fcbe1cfd3cf83bc9dc30cb324bfe9ab106bf55",
            "decimal.equal",
        ),
        (
            "a662f2d29c2e22873d15bee55e4bf204f94039a02dca5d702eaef8972551563c",
            "decimal.greater_equal",
        ),
        (
            "cfd713dba7b564e51e8a7412681721b6d6c7ee52bf103b493640fc28581ec241",
            "decimal.not_equal",
        ),
        (
            "d3bfc038e46a90e05f2b4d64b37b2d42d4d66b7aac729db7c668a38f4baf1140",
            "decimal.less_equal",
        ),
        (
            "d7c2d9c04df90b2e3d6d2b9e90dffc8ecfdaffc407a88086a1347d4f35f0aca6",
            "decimal.subtract",
        ),
        (
            "e3a7f6683c7f700fa22c8998fb907fd03433b4cfc53a65018eb76d5ec13de5ee",
            "decimal.divide",
        ),
        (
            "259be2f4eb7992db7f06a6c7d4385b0563852afda806de33709b627be83e6c82",
            "decimal.equal",
        ),
    ]);
}

#[test]
fn csharp_03_t06_w09_string_basic_source_linkage_rejects_substitution() {
    let cases = [
        (
            "15a93564d8819ba33b2c3c752501c69a1dc5aa3dda240c595db8eff7abd05f7f",
            "string.is_null_or_empty",
        ),
        (
            "422102785bc05b7b701aaa79b90ad51dfb452955b538a13450244d01eb549fd4",
            "string.length",
        ),
        (
            "4b3aea39a57210c852642e12b9b206dab57a6811628f5080b520934e7cae77f8",
            "string.length",
        ),
        (
            "58192b52bd4bf5c30ff78ffab9fd8a6d02293bcfd723822d7701ba338f0f9673",
            "string.length",
        ),
        (
            "5c9185c954edd922bee4892b0ad4a612ee6efe729b9d0a907ea0b083dc32effc",
            "string.index",
        ),
        (
            "784052557e7e5768a3617cb28ecdbf9e7b31bf51df84ed6d95b0878b3626bff4",
            "string.length",
        ),
        (
            "85983195c27c7992e8a87d99c134fcd0efef4d57bbc6dc2ebc868a0651b205bf",
            "string.length",
        ),
        (
            "87b745463754ae3dbf72720245889117da7547adcb795bc61c74d3d7b39ce021",
            "string.index",
        ),
        (
            "b863eb406b3aa79c8ad88cc0bb04c8e6b39c983f77270bf3e6c206588161f5d8",
            "string.length",
        ),
    ];
    string_source_linkage(&cases, "string-basic-circuits", "MPK_W09_STRING_OUT", &[20]);
}

fn string_source_linkage(
    cases: &[(&str, &str)],
    directory: &str,
    output_env: &str,
    expected_depths: &[u32],
) {
    let mut depths = BTreeSet::new();
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../develop/migrations/csharp-03/ordinary-foundation/{directory}"
    ));
    let output = std::env::var_os(output_env).map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut metrics = vec![];
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for &(id, required) in cases {
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
        let carriers = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let text = carriers
            .carriers()
            .iter()
            .find(|c| c.type_id == "mpk.csharp.value.string.v1")
            .unwrap();
        assert_eq!(text.depth, 19);
        let p = generate_csharp_practical_ordinary_strings(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        for d in p.definitions() {
            for text in &d.operation.argument_type_ids {
                let carrier = carriers
                    .carriers()
                    .iter()
                    .find(|c| &c.type_id == text)
                    .unwrap();
                if carrier.depth >= 19 {
                    depths.insert(carrier.depth);
                }
            }
        }
        if directory == "string-ordinal-circuits" {
            // Count all StepTwo occurrences before DAG sharing, plus helpers.
            assert_eq!(p.static_transformers(), 8234);
        }
        let metadata = p.canonical_bytes();
        let actual = p
            .definitions()
            .iter()
            .map(|d| d.operation.id.as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            actual
                .iter()
                .any(|op| *op == required || op.starts_with(&format!("{required}."))),
            "{id}: {actual:?}"
        );
        let wire: Value =
            serde_json::from_slice(emitted.operations().operations().canonical_bytes()).unwrap();
        let expected = wire["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .filter(|s| s.starts_with("string."))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            import_csharp_practical_ordinary_strings(&metadata, p.certificate_bytes(), vir)
                .unwrap(),
            p
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let file = format!("{required}.{}.hex", &id[..8]);
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(&file), &hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture_root.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"source":id,"file":file,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"metadata":serde_json::from_slice::<Value>(&metadata).unwrap()}));
        let original: Value = serde_json::from_slice(&metadata).unwrap();
        for mutation in 0..8 {
            let mut changed = original.clone();
            match mutation {
                0 => changed["source_ir_sha256"] = json!("0".repeat(64)),
                1 => changed["foundation_sha256"] = json!("0".repeat(64)),
                2 => changed["certificate_sha256"] = json!("0".repeat(64)),
                3 => changed["definitions"] = json!([]),
                4 => changed["definitions"][0]["operation"]["ordered_checks"] = json!([]),
                5 => {
                    changed["definitions"][0]["operation"]["normal_result_type_id"] =
                        json!("mpk.csharp.value.i64.v1")
                }
                6 => changed["definitions"][0]["result_definition"] = json!("forged"),
                _ => changed["definitions"][0]["operation"]["id"] = json!("string.unimplemented"),
            }
            if changed != original {
                assert!(import_csharp_practical_ordinary_strings(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_strings(&metadata, &bad, vir).is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_strings(m, c, vir).is_err());
        }
        previous = Some((metadata, p.certificate_bytes().to_vec()));
    }
    // Pin the actual carrier forms of each original source cohort.
    assert_eq!(depths, expected_depths.iter().copied().collect());
    if let Some(dir) = output {
        fs::write(
            dir.join("metrics.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture_root.join("metrics.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_string_ordinal_source_linkage_rejects_substitution() {
    string_source_linkage(
        &[
            (
                "04089ad467c60b8d1e43ffd42d2cc38da74dad54aedff050fde19dd409c8e33e",
                "string.equality.operator",
            ),
            (
                "ce54b60f5c0c18c31094869566c99c43188c101a3d5e8006c211e34f7029dfed",
                "string.inequality.operator",
            ),
            (
                "02182a7a664c19a27525493f29cc5201bf0be1c5c91e7b2577609d4294c23e1a",
                "string.equals.ordinal",
            ),
            (
                "0bc1c0c07287e9cbf86bd6b3fbdbb1e726156d19596811c4f31e3e6184d35015",
                "string.equals.instance.ordinal",
            ),
            (
                "76de5d4d6d2c6a3bdae3af0d89b9bf62dfcaafc0c0b4ecc2d1d330f887bad009",
                "string.compare.ordinal",
            ),
            (
                "9aeebaac704d50a85728a423d1d6411c702677b31514bee6108394d4fcaf9142",
                "string.contains.ordinal",
            ),
            (
                "edde2f36630ba55e891f35d901c2dbf178dbd721b4477cc1c45fe2b73da6e364",
                "string.starts_with.ordinal",
            ),
            (
                "abda15b63719a81e5eec2f45bbc198e01a167ef4224f99434e1833a8f1a600d7",
                "string.ends_with.ordinal",
            ),
        ],
        "string-ordinal-circuits",
        "MPK_W09_STRING_ORDINAL_OUT",
        &[19, 20],
    );
}

#[test]
fn csharp_03_t06_w09_string_construction_source_linkage_rejects_substitution() {
    string_source_linkage(
        &[
            (
                "304b28cd651bf51eea71d91bd718875fd27af3310db3d2fee97db2cf14b3f92f",
                "string.concat.operator.char_string",
            ),
            (
                "63bb510bd024f2abd2fb68a5569710929d69aaaacc2fa3043d1cf571901609a4",
                "string.interpolation.restricted",
            ),
            (
                "90ec9e7b48c233a20391174dd3ebe4e231edacf05e279d5d58908262757ae878",
                "string.interpolation.restricted",
            ),
            (
                "c0c19a7b58bba12700f3de83a3d155fd9cdfca541a9c9c146f372dd0519a98fe",
                "string.concat.operator.string_char",
            ),
            (
                "eb35be66582a7e6f92940b23f56212ef0941b4b4f50405fc977facea99fd02fa",
                "string.concat.string4",
            ),
            (
                "f09a5ff3dbc39178dca2c649cfa1b5dccecd5d5a642955c5c930a699536b64b2",
                "string.concat.operator.string_string",
            ),
            (
                "f57dee68d8fc8e5603fb3757146330a3fc4de9f68fd684297fa4b1b835a10bd0",
                "string.substring.start_length",
            ),
        ],
        "string-construction-circuits",
        "MPK_W09_STRING_CONSTRUCT_OUT",
        &[20],
    );
}

#[test]
fn csharp_03_t06_w09_structural_source_linkage_rejects_substitution() {
    let bundle = b();
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-storage");
    let output = std::env::var_os("MPK_W09_STRUCTURAL_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut metrics = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut products = 0;
    let mut sums = 0;
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
            let (context, captures) = support::replay_context(&bundle, req);
            let source = ValidatedDataSource::import_captured_facts(
                &bundle,
                &context,
                &captures,
                &serde_json::to_vec(&response["facts"]).unwrap(),
            )
            .unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let vir = emitted.vir();
            let carriers = generate_csharp_practical_ordinary_carriers(vir).unwrap();
            let p = generate_csharp_practical_ordinary_structural(vir)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let expected = carriers
                .carriers()
                .iter()
                .filter(|c| {
                    matches!(
                        c.shape,
                        OrdinaryShape::Product { .. } | OrdinaryShape::Sum { .. }
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                p.definitions()
                    .iter()
                    .map(|d| &d.carrier)
                    .collect::<Vec<_>>(),
                expected
            );
            for d in p.definitions() {
                match (&d.carrier.shape, &d.operations) {
                    (
                        OrdinaryShape::Product { fields },
                        OrdinaryStructuralOperations::Product { operations },
                    ) => {
                        products += 1;
                        assert_eq!(
                            fields.iter().map(|f| &f.id).collect::<Vec<_>>(),
                            operations
                                .fields
                                .iter()
                                .map(|f| &f.field_id)
                                .collect::<Vec<_>>()
                        );
                    }
                    (
                        OrdinaryShape::Sum { arms },
                        OrdinaryStructuralOperations::Sum { arms: ops, .. },
                    ) => {
                        sums += 1;
                        assert_eq!(
                            arms.iter().map(|a| (a.tag, &a.id)).collect::<Vec<_>>(),
                            ops.iter().map(|a| (a.tag, &a.arm_id)).collect::<Vec<_>>()
                        );
                        for (a, op) in arms.iter().zip(ops) {
                            assert_eq!(
                                a.fields.iter().map(|f| &f.id).collect::<Vec<_>>(),
                                op.fields.iter().map(|f| &f.field_id).collect::<Vec<_>>()
                            );
                        }
                    }
                    _ => panic!("shape/operation mismatch"),
                }
            }
            let metadata = p.canonical_bytes();
            let bytes = p.certificate_bytes();
            assert_eq!(
                import_csharp_practical_ordinary_structural(&metadata, bytes, vir).unwrap(),
                p
            );
            let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            validate_csharp_practical_certificate_structure(&c).unwrap();
            let original: Value = serde_json::from_slice(&metadata).unwrap();
            for mutation in 0..6 {
                let mut m = original.clone();
                match mutation {
                    0 => m["source_ir_sha256"] = json!("0".repeat(64)),
                    1 => m["foundation_sha256"] = json!("0".repeat(64)),
                    2 => m["certificate_sha256"] = json!("0".repeat(64)),
                    3 => m["definitions"] = json!([]),
                    4 if !p.definitions().is_empty() => {
                        m["definitions"][0]["carrier"]["depth"] = json!(254)
                    }
                    5 if !p.definitions().is_empty() => {
                        m["definitions"][0]["carrier"]["type_id"] = json!("Forged")
                    }
                    _ => m["definitions"] = json!([{"kind": "forged"}]),
                }
                if m != original {
                    assert!(import_csharp_practical_ordinary_structural(
                        &serde_json::to_vec(&m).unwrap(),
                        bytes,
                        vir
                    )
                    .is_err());
                }
            }
            let mut bad = bytes.to_vec();
            *bad.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_structural(&metadata, &bad, vir).is_err());
            if let Some((m, c)) = &previous {
                if m != &metadata {
                    assert!(import_csharp_practical_ordinary_structural(m, c, vir).is_err());
                }
            }
            previous = Some((metadata.clone(), bytes.to_vec()));
            let file = format!("{id}.hex");
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if let Some(dir) = &output {
                fs::write(dir.join(&file), hex).unwrap();
            } else {
                assert_eq!(fs::read_to_string(fixture_root.join(&file)).unwrap(), hex);
            }
            metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":original}));
        }
    }
    assert_eq!(metrics.len(), 24);
    assert!(products > 0 && sums > 0);
    if let Some(dir) = output {
        fs::write(
            dir.join("metrics.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture_root.join("metrics.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_scalar_domains_original_sources_and_mutations() {
    let bundle = b();
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/scalar-domains");
    let output = std::env::var_os("MPK_W09_SCALAR_DOMAINS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut metrics = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut saw_enum = false;
    let mut saw_decimal = false;
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
            let (context, captures) = support::replay_context(&bundle, req);
            let source = ValidatedDataSource::import_captured_facts(
                &bundle,
                &context,
                &captures,
                &serde_json::to_vec(&response["facts"]).unwrap(),
            )
            .unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let vir = emitted.vir();
            let carriers = generate_csharp_practical_ordinary_carriers(vir).unwrap();
            let p = generate_csharp_practical_ordinary_scalar_domains(vir).unwrap();
            let expected = carriers
                .carriers()
                .iter()
                .filter(|c| {
                    matches!(c.shape, OrdinaryShape::Bits { .. })
                        || c.type_id == "mpk.csharp.value.decimal.v1"
                })
                .collect::<Vec<_>>();
            assert_eq!(
                p.definitions()
                    .iter()
                    .map(|d| &d.carrier)
                    .collect::<Vec<_>>(),
                expected
            );
            for d in p.definitions() {
                if let OrdinaryScalarDomainRule::DeclaredEnum { values } = &d.rule {
                    saw_enum = true;
                    let ty = response["facts"]["types"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|t| t["id"] == d.carrier.type_id)
                        .unwrap();
                    assert_eq!(ty["kind"], "enum");
                    assert_eq!(ty["enum_values"], json!(values));
                }
                saw_decimal |= d.rule == OrdinaryScalarDomainRule::Decimal;
            }
            let metadata = p.canonical_bytes();
            let bytes = p.certificate_bytes();
            assert_eq!(
                import_csharp_practical_ordinary_scalar_domains(&metadata, bytes, vir).unwrap(),
                p
            );
            let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            validate_csharp_practical_certificate_structure(&c).unwrap();
            let original: Value = serde_json::from_slice(&metadata).unwrap();
            for key in [
                "source_ir_sha256",
                "foundation_sha256",
                "certificate_sha256",
                "schema",
            ] {
                let mut m = original.clone();
                m[key] = json!("forged");
                assert!(import_csharp_practical_ordinary_scalar_domains(
                    &serde_json::to_vec(&m).unwrap(),
                    bytes,
                    vir
                )
                .is_err());
            }
            // Mutating the rule must reject even while retaining the original valid bytes/hash.
            for index in 0..p.definitions().len() {
                let mut m = original.clone();
                m["definitions"][index]["rule"] = json!({"kind":"forged"});
                assert!(import_csharp_practical_ordinary_scalar_domains(
                    &serde_json::to_vec(&m).unwrap(),
                    bytes,
                    vir
                )
                .is_err());
            }
            let mut bad = bytes.to_vec();
            *bad.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_scalar_domains(&metadata, &bad, vir).is_err());
            if let Some((m, c)) = &previous {
                if m != &metadata {
                    assert!(import_csharp_practical_ordinary_scalar_domains(m, c, vir).is_err());
                }
            }
            previous = Some((metadata, bytes.to_vec()));
            let file = format!("{id}.hex");
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if let Some(dir) = &output {
                fs::write(dir.join(&file), hex).unwrap();
            } else {
                assert_eq!(fs::read_to_string(fixture_root.join(&file)).unwrap(), hex);
            }
            metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":original}));
        }
    }
    assert_eq!(metrics.len(), 24);
    assert!(saw_enum && saw_decimal);
    if let Some(dir) = output {
        fs::write(
            dir.join("metrics.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture_root.join("metrics.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_scalar_domains_ranges_from_original_sources() {
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/scalar-domain-ranges");
    let output = std::env::var_os("MPK_W09_SCALAR_DOMAIN_RANGES_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut metrics = vec![];
    for (id, required) in [
        (
            "dec3e56d4ee60ff5a7696151300fdf3cb592d5dee385e0da40e9b2a8ce46ebc9",
            vec![("time", 863_999_999_999u64)],
        ),
        (
            "05e148b20eb9017bdab9a113ab247301fa596ef171961443f1c68727025fe98f",
            vec![("date", 3_652_058u64), ("day_of_week", 6u64)],
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
        let p = generate_csharp_practical_ordinary_scalar_domains(vir).unwrap();
        for (token, maximum) in required {
            let definition = p
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == format!("mpk.csharp.value.{token}.v1"))
                .unwrap();
            assert_eq!(definition.rule, OrdinaryScalarDomainRule::Range { maximum });
        }
        let metadata = p.canonical_bytes();
        let bytes = p.certificate_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_scalar_domains(&metadata, bytes, vir).unwrap(),
            p
        );
        let mut forged: Value = serde_json::from_slice(&metadata).unwrap();
        for definition in forged["definitions"].as_array_mut().unwrap() {
            if definition["rule"]["kind"] == "range" {
                definition["rule"]["maximum"] = json!(0);
            }
        }
        assert!(import_csharp_practical_ordinary_scalar_domains(
            &serde_json::to_vec(&forged).unwrap(),
            bytes,
            vir
        )
        .is_err());
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        let file = format!("{id}.hex");
        if let Some(dir) = &output {
            fs::write(dir.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":serde_json::from_slice::<Value>(&metadata).unwrap()}));
    }
    if let Some(dir) = output {
        fs::write(
            dir.join("metrics.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join("metrics.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_ordered_folds_source_replay_and_mutations() {
    let bundle = b();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/ordered-folds");
    let output = std::env::var_os("MPK_W09_ORDERED_FOLD_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut cases = vec![];
    for (family, id, expected) in [
        ("binding-vc", "bounded_sequence", vec![12u32]),
        ("binding-vc", "validation", vec![12]),
        ("binding-vc", "ordered_map", vec![12]),
        ("construction-vc", "positive_constructor", vec![]),
    ] {
        let requests = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        let row = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        cases.push((
            format!("{family}-{id}"),
            row,
            response["facts"].clone(),
            expected,
        ));
    }
    let replay = read("data-phase/data-stage-replay.json");
    let row = replay
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "5c9185c954edd922bee4892b0ad4a612ee6efe729b9d0a907ea0b083dc32effc")
        .unwrap();
    cases.push((
        "string-index".to_owned(),
        row.clone(),
        row["outcome"]["facts"].clone(),
        vec![14],
    ));
    let mut metrics = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts, expected) in cases {
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
        let p = generate_csharp_practical_ordinary_ordered_folds(vir).unwrap();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.index_bits)
                .collect::<Vec<_>>(),
            expected,
            "{id}"
        );
        let metadata = p.canonical_bytes();
        let bytes = p.certificate_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_ordered_folds(&metadata, bytes, vir).unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&metadata).unwrap();
        assert_eq!(
            original["static_transformers"],
            if expected.is_empty() { 0 } else { 8192 }
        );
        for key in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
            "static_transformers",
        ] {
            let mut changed = original.clone();
            changed[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_ordered_folds(
                &serde_json::to_vec(&changed).unwrap(),
                bytes,
                vir
            )
            .is_err());
        }
        let mut changed = original.clone();
        changed["definitions"] = json!([{"index_bits":15,"capacity":32768}]);
        assert!(import_csharp_practical_ordinary_ordered_folds(
            &serde_json::to_vec(&changed).unwrap(),
            bytes,
            vir
        )
        .is_err());
        let mut bad = bytes.to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_ordered_folds(&metadata, &bad, vir).is_err());
        if let Some((m, c)) = &previous {
            if m != &metadata {
                assert!(import_csharp_practical_ordinary_ordered_folds(m, c, vir).is_err());
            }
        }
        previous = Some((metadata, bytes.to_vec()));
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let file = format!("{id}.hex");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(p) = &output {
            fs::write(p.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":original}));
    }
    assert_eq!(metrics.len(), 5);
    if let Some(p) = output {
        fs::write(
            p.join("metrics.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join("metrics.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[allow(dead_code)]
#[path = "../../src/csharp_practical_ordinary_test_eval.rs"]
mod core_eval;

#[path = "csharp_practical_ordinary_relation_tests.rs"]
mod relation_tests;
