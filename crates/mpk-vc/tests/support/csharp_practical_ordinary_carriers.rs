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
