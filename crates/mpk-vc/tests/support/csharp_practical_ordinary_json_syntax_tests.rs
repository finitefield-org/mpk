//! Actual source and sidecar linkage for closed JSON lexical names.
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{
    parse_canonical_practical_json, PracticalArtifactKind,
};

#[test]
fn csharp_03_t06_w09_json_syntax_original_sources() {
    let bundle = b();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-syntax");
    let out = std::env::var_os("MPK_W09_JSON_SYNTAX_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    let mut contexts = 0;
    let mut surrogate = false;
    for (id, row, facts) in document_sources() {
        contexts += 1;
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
        let p = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        assert_eq!(
            p.literals().is_empty(),
            vc.boundary_vcs().contracts().is_empty()
        );
        import_csharp_practical_ordinary_json_syntax(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir,
        )
        .unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert!(p.literals().windows(2).all(|p| p[0].utf8 < p[1].utf8));
        // Derive exact sidecar names/order independently, so omitted or reordered
        // fields cannot pass just by agreeing with the generator's own metadata.
        let mut expected_fields = vec![];
        for contract in vc.boundary_vcs().contracts() {
            let contract = parse_canonical_practical_json(
                PracticalArtifactKind::BoundaryContract,
                contract.as_bytes(),
            )
            .unwrap();
            let owner = contract.get("contract_sha256").unwrap().as_str().unwrap();
            for group in ["input_fields", "output_fields"] {
                for field in contract.get(group).unwrap().as_array().unwrap() {
                    let name = match field.get("json_name").unwrap() {
                        PracticalJsonValue::String(s) => s.encode_utf16().collect::<Vec<_>>(),
                        PracticalJsonValue::Utf16String(s) => s.clone(),
                        _ => panic!("validated field name must be a string"),
                    };
                    expected_fields.push((
                        owner.to_owned(),
                        group.to_owned(),
                        field.get("field_id").unwrap().as_str().unwrap().to_owned(),
                        name,
                    ));
                }
            }
        }
        let actual_fields = p
            .fields()
            .iter()
            .filter(|f| matches!(f.group.as_str(), "input_fields" | "output_fields"))
            .map(|f| {
                (
                    f.owner.clone(),
                    f.group.clone(),
                    f.field_id.clone(),
                    f.name_utf16.clone(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_fields, expected_fields);
        if !p.literals().is_empty() {
            let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
            let mut expected = vec![];
            for carrier in layouts.carriers() {
                if let Some(source) = facts["types"].as_array().unwrap().iter().find(|t| {
                    t["id"].as_str() == Some(carrier.type_id.as_str()) && t["kind"] != "enum"
                }) {
                    for member in source["members"].as_array().unwrap() {
                        let name = member["name"].as_str().unwrap();
                        expected.push((
                            carrier.type_id.clone(),
                            name.to_owned(),
                            name.encode_utf16().collect::<Vec<_>>(),
                        ));
                    }
                }
            }
            let actual = p
                .fields()
                .iter()
                .filter(|f| f.group == "source_members")
                .map(|f| (f.owner.clone(), f.field_id.clone(), f.name_utf16.clone()))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
        }
        for field in p.fields() {
            let literal = p
                .literals()
                .iter()
                .find(|d| d.match_definition == field.match_definition)
                .unwrap();
            let mut expected = canonical_practical_json_bytes(&PracticalJsonValue::Utf16String(
                field.name_utf16.clone(),
            ))
            .unwrap();
            expected.push(b':');
            assert_eq!(literal.utf8, expected);
            surrogate |= field.name_utf16 == [0xd800];
        }
        for bytes in [b"{".as_slice(), b"}", b"[", b"]", b",", b":"] {
            if !p.literals().is_empty() {
                assert!(p.literals().iter().any(|d| d.utf8 == bytes));
            }
        }
        if p.literals().is_empty() {
            continue;
        }
        let mut metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        metadata["literals"][0]["utf8"] = json!([120]);
        assert!(import_csharp_practical_ordinary_json_syntax(
            &serde_json::to_vec(&metadata).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        if !p.fields().is_empty() {
            let mut metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            metadata["fields"][0]["name_utf16"] = json!([120]);
            assert!(import_csharp_practical_ordinary_json_syntax(
                &serde_json::to_vec(&metadata).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_syntax(&p.canonical_bytes(), &bad, vir).is_err()
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &out {
            fs::write(out.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(contexts, 68);
    assert_eq!(rows.len(), 3);
    assert!(surrogate);
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if let Some(out) = &out {
        fs::write(out.join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(root.join("certificates.json")).unwrap(), bytes);
    }
    eprintln!("JSON syntax source contexts {contexts}; three nonempty programs retain the lone-surrogate sidecar name");
}
