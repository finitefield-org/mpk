use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{bit, sparse_cube};
fn document(bytes: &[u8], length: u32, inactive: &[(usize, u8)]) -> V {
    let mut cells = BTreeSet::new();
    for k in 0..32 {
        if length & (1 << k) != 0 {
            cells.insert(k << 19);
        }
    }
    for (at, byte) in bytes
        .iter()
        .copied()
        .enumerate()
        .chain(inactive.iter().copied())
    {
        assert!(at < 1_048_576);
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                cells.insert(1 | (at << 1) | (k << 21));
            }
        }
    }
    sparse_cube(24, cells)
}
#[test]
fn csharp_03_t06_w09_json_raw_limits_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_JSON_RAW_LIMITS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-raw-limits");
    let mut rows = vec![];
    let mut observed = false;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut examined = 0;
    for (id, row, facts) in document_sources() {
        examined += 1;
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
        let p = generate_csharp_practical_ordinary_json_raw_limits(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let boundary = vc.boundary_vcs();
        assert_eq!(p.definition().is_none(), boundary.contracts().is_empty());
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_json_raw_limits(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "boundary_program_sha256",
            "definition",
            "certificate_sha256",
        ] {
            let mut forged = meta.clone();
            forged[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_json_raw_limits(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_raw_limits(&p.canonical_bytes(), &bad, vir)
                .is_err()
        );
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_json_raw_limits(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.document.depth, 24);
        assert_eq!(d.block_bytes, 32);
        assert_eq!(d.blocks_per_step, 8);
        assert_eq!(d.scan_steps, 4_096);
        assert!(d.static_transformers > 4_104 && d.static_transformers <= 16_384);
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        if !observed {
            assert!(bit(run(
                &cert,
                &d.valid_definition,
                vec![document(&[], 0, &[])]
            )));
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "raw JSON limit contexts {}, nonempty {}",
        examined,
        rows.len()
    );
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/json-raw-limits/certificates.json"),
            manifest
        );
    }
}

#[test]
fn csharp_03_t06_w09_json_raw_limits_complete_documents() {
    let bundle = b();
    let (_, request, facts) = document_sources()
        .into_iter()
        .find(|(id, _, _)| id.starts_with("document-"))
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_json_raw_limits(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    let d = p.definition().unwrap();
    let mut cases = vec![
        ("{}".into(), true),
        ("[]".into(), true),
        (r#"{"a":"[{}]","b":[1,true,null]}"#.into(), true),
        (r#""quoted\"\\text""#.into(), true),
    ];
    for n in [31, 32, 33, 34] {
        cases.push((format!("{}0{}", "[".repeat(n), "]".repeat(n)), n <= 32));
        cases.push((format!("{}{}", "[".repeat(n), "]".repeat(n)), n <= 33));
    }
    for n in [60, 61, 62, 63, 64, 65, 126, 127, 128] {
        cases.push((format!(r#"{{"{}":"\"[{{}}]\\"}}"#, "a".repeat(n)), true));
    }
    for (text, expected) in &cases {
        // These are valid JSON trees. Grammar rejection is not credited to the
        // limit scanner; expected values depend only on the frozen raw bound.
        serde_json::from_str::<Value>(text).unwrap();
        assert_eq!(
            bit(run(
                &cert,
                &d.valid_definition,
                vec![document(text.as_bytes(), text.len() as u32, &[(900, 255)])]
            )),
            *expected,
            "{text}"
        );
    }
    for length in [1_048_577, u32::MAX] {
        assert!(!bit(run(
            &cert,
            &d.valid_definition,
            vec![document(&[], length, &[])]
        )));
    }
    eprintln!("Raw JSON complete documents:{} trees plus2 overlength cases;depth32/33,empty containers,escaped strings and block boundaries",cases.len());
}
