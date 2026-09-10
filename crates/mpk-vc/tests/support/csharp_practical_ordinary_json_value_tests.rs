//! Source-derived typed primitive packets; compound grammar remains outstanding.
use super::*;

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut rows = document_sources();
    let requests = read("ordinary-foundation/json-calendar-sources/requests.json");
    let responses = read("ordinary-foundation/json-calendar-sources/responses.json");
    for request in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == request["id"])
            .unwrap();
        assert!(response.get("reject").is_none());
        rows.push((
            format!("calendar-{}", request["id"].as_str().unwrap()),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    rows
}

#[test]
fn csharp_03_t06_w09_json_values_original_sources_and_token_preservation() {
    let bundle = b();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-values");
    let out = std::env::var_os("MPK_W09_JSON_VALUES_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut contexts = 0;
    let mut decimal_programs = 0;
    for (id, row, facts) in sources() {
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
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let tokens = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        let program = generate_csharp_practical_ordinary_json_values(vir)
            .unwrap_or_else(|error| panic!("{id}: typed JSON generation failed: {error:?}"));
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_json_values(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let old = mpk_cert::decode_canonical_certificate(tokens.certificate_bytes()).unwrap();
        let roots = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        super::super::super::structural_equivalence_tests::same_definition_closure(
            &old, &cert, &roots,
        )
        .unwrap();
        let reachable = layouts
            .carriers()
            .iter()
            .map(|c| c.type_id.as_str())
            .collect::<BTreeSet<_>>();
        let defined = program
            .definitions()
            .iter()
            .map(|d| d.carrier.type_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            program
                .deferred_type_ids()
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            reachable.difference(&defined).copied().collect()
        );
        let Some(tokens) = tokens.definition() else {
            assert!(program.definitions().is_empty());
            continue;
        };
        let mut expected = BTreeMap::new();
        for d in tokens.scalars.iter().chain(&tokens.quoted_scalars) {
            let kind = if d.kind == "null" { "unit" } else { &d.kind };
            let ty = format!("mpk.csharp.value.{kind}.v1");
            if reachable.contains(ty.as_str()) {
                expected.insert(d.parse_definition.clone(), (ty, None, None));
            }
        }
        if reachable.contains("mpk.csharp.value.string.v1") {
            expected.insert(
                tokens.string_frame_definition.clone(),
                ("mpk.csharp.value.string.v1".into(), None, None),
            );
        }
        for d in &tokens.quoted_codecs {
            if reachable.contains(d.codec.value_type_id.as_str()) {
                expected.insert(
                    d.parse_definition.clone(),
                    (d.codec.value_type_id.clone(), None, None),
                );
            }
        }
        for d in &tokens.quoted_calendars {
            if reachable.contains(d.codec.value_type_id.as_str()) {
                expected.insert(
                    d.parse_definition.clone(),
                    (d.codec.value_type_id.clone(), None, None),
                );
            }
        }
        for d in &tokens.quoted_decimals {
            if reachable.contains(d.codec.value_type_id.as_str()) {
                expected.insert(
                    d.parse_definition.clone(),
                    (
                        d.codec.value_type_id.clone(),
                        d.codec.scale,
                        d.codec.rounding.clone(),
                    ),
                );
            }
        }
        let actual = program
            .definitions()
            .iter()
            .map(|d| {
                (
                    d.token_definition.clone(),
                    (d.carrier.type_id.clone(), d.scale, d.rounding.clone()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            actual.len(),
            program.definitions().len(),
            "duplicate typed codec"
        );
        assert_eq!(actual, expected);
        for d in program.definitions() {
            assert_eq!(
                &d.carrier,
                layouts
                    .carriers()
                    .iter()
                    .find(|c| c.type_id == d.carrier.type_id)
                    .unwrap()
            );
            assert_eq!(d.packet_depth, d.carrier.depth.max(7) + 1);
            for name in [
                &d.parse_definition,
                &d.token_definition,
                &d.header_definition,
                &d.value_definition,
            ] {
                assert!(cert
                    .declarations
                    .iter()
                    .any(|decl| &cert.name_table[decl.name as usize] == name));
            }
        }
        if reachable.contains("mpk.csharp.value.decimal.v1") {
            assert_eq!(
                program
                    .definitions()
                    .iter()
                    .filter(|d| d.carrier.type_id == "mpk.csharp.value.decimal.v1")
                    .count(),
                146
            );
            decimal_programs += 1;
        }
        let mut forged: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        forged["definitions"][0]["carrier"]["type_id"] = json!("forged");
        assert!(import_csharp_practical_ordinary_json_values(
            &serde_json::to_vec(&forged).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bad = program.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_json_values(
            &program.canonical_bytes(),
            &bad,
            vir
        )
        .is_err());
        let hex = program
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
        eprintln!(
            "Typed JSON {id}: {} adapters, {} terms, {} declarations",
            program.definitions().len(),
            cert.term_table.len(),
            cert.declarations.len()
        );
        rows.push(json!({"id":id,"program":serde_json::from_slice::<Value>(&program.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(contexts, 72);
    assert_eq!(rows.len(), 7);
    assert_eq!(decimal_programs, 2);
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if let Some(out) = &out {
        fs::write(out.join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(root.join("certificates.json")).unwrap(), bytes);
    }
    eprintln!("Typed JSON: {contexts} source contexts, seven complete programs; all prior token definitions/dependencies unchanged");
}
