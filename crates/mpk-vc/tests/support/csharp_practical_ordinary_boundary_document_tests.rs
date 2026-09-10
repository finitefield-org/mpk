use super::integer_format_tests::format_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

pub(super) fn document_sources() -> Vec<(String, Value, Value)> {
    let mut sources = format_sources();
    let requests = read("boundary-output/source-requests.json");
    let responses = read("boundary-output/source-responses.json");
    for request in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == request["id"])
            .unwrap();
        assert!(response.get("reject").is_none());
        sources.push((
            format!("document-{}", request["id"].as_str().unwrap()),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    sources
}

fn observe(cert: &mpk_cert::encode::Certificate, v: &V, depth: u32, index: usize) -> bool {
    let mut v = v.clone();
    for i in 0..depth {
        v = apply(cert, v, V::Bit(index & (1 << i) != 0));
    }
    bit(v)
}
fn word(n: u32) -> V {
    sparse_cube(5, (0..32).filter(|i| n & (1 << i) != 0).collect())
}
fn buffer(bytes: &[(usize, u8)]) -> V {
    let mut cells = BTreeSet::new();
    for &(at, byte) in bytes {
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                cells.insert(at | (k << 20));
            }
        }
    }
    sparse_cube(23, cells)
}
fn checked_storage(cert: &mpk_cert::encode::Certificate, d: &OrdinaryBoundaryDocumentDefinition) {
    assert_eq!(d.type_name, "Mpk.CSharp.Ordinary.BoundaryDocument");
    assert_eq!(d.depth, 24);
    assert_eq!(
        d.shape,
        OrdinaryShape::Sequence {
            capacity: 1_048_576,
            element: Box::new(OrdinaryShape::Bits { width: 8 })
        }
    );
    let positions = [
        0usize, 1, 16_383, 16_384, 16_385, 65_535, 65_536, 524_287, 524_288, 1_048_575,
    ];
    let bytes = positions
        .iter()
        .enumerate()
        .map(|(i, &at)| (at, (i * 23 + 1) as u8))
        .collect::<Vec<_>>();
    for length in [
        0,
        1,
        16_384,
        16_385,
        65_536,
        1_048_575,
        1_048_576,
        1_048_577,
        1 << 31,
        u32::MAX,
    ] {
        let document = run(cert, &d.make_definition, vec![word(length), buffer(&bytes)]);
        let observed_length = run(cert, &d.length_definition, vec![document.clone()]);
        for k in 0..32 {
            assert_eq!(
                observe(cert, &observed_length, 5, k),
                length & (1 << k) != 0
            );
        }
        assert_eq!(
            bit(run(cert, &d.bounded_definition, vec![document.clone()])),
            length <= 1_048_576
        );
        for pad in 1..19 {
            assert!(!observe(cert, &document, 24, 1 << pad));
        }
        for &(at, byte) in &bytes {
            let expected = if at < length as usize { byte } else { 0 };
            let read = run(
                cert,
                &d.read_byte_definition,
                vec![document.clone(), word(at as u32)],
            );
            for k in 0..8 {
                assert_eq!(
                    observe(cert, &read, 3, k),
                    expected & (1 << k) != 0,
                    "length {length}, index {at}, bit {k}"
                );
                assert_eq!(
                    observe(cert, &document, 24, 1 | (at << 1) | (k << 21)),
                    expected & (1 << k) != 0
                );
            }
        }
        for index in [1_048_576, 1_048_577, 1 << 31, u32::MAX] {
            let read = run(
                cert,
                &d.read_byte_definition,
                vec![document.clone(), word(index)],
            );
            for k in 0..8 {
                assert!(!observe(cert, &read, 3, k));
            }
        }
    }
    // A complete canonical JSON document can exceed the application string
    // capacity while each decoded string remains within its 16,384-unit bound.
    let first = format!("{{\"text\":\"{}\",\"tail\":true}}", "a".repeat(16_384));
    let second = format!("{{\"text\":\"{}\",\"tail\":null}}", "a".repeat(16_384));
    assert_eq!(first.len(), second.len());
    assert_eq!(&first.as_bytes()[..16_384], &second.as_bytes()[..16_384]);
    for text in [&first, &second] {
        parse_canonical_practical_json(PracticalArtifactKind::BoundaryInput, text.as_bytes())
            .unwrap();
    }
    let documents = [&first, &second].map(|text| {
        let bytes = text.bytes().enumerate().collect::<Vec<_>>();
        run(
            cert,
            &d.make_definition,
            vec![word(text.len() as u32), buffer(&bytes)],
        )
    });
    let differences = first
        .bytes()
        .zip(second.bytes())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert!(!differences.is_empty());
    assert!(differences.iter().all(|i| *i > 16_384));
    for (document, text) in documents.iter().zip([&first, &second]) {
        for index in (0..text.len()).filter(|i| *i < 12 || *i >= 16_380) {
            let byte = run(
                cert,
                &d.read_byte_definition,
                vec![document.clone(), word(index as u32)],
            );
            for k in 0..8 {
                assert_eq!(
                    observe(cert, &byte, 3, k),
                    text.as_bytes()[index] & (1 << k) != 0
                );
            }
        }
    }
}
#[test]
fn csharp_03_t06_w09_boundary_documents_original_sources_and_storage() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BOUNDARY_DOCUMENTS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/boundary-documents");
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
        let p = generate_csharp_practical_ordinary_boundary_documents(vir)
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
        assert_eq!(
            meta["contract_ids"].as_array().unwrap().len(),
            boundary.contracts().len()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_boundary_documents(
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
            "contract_ids",
            "definition",
            "certificate_sha256",
        ] {
            let mut forged = meta.clone();
            forged[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_boundary_documents(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_boundary_documents(
            &p.canonical_bytes(),
            &bad,
            vir
        )
        .is_err());
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_boundary_documents(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        assert!(layouts.carriers().iter().all(|c| c.type_id != d.type_name));
        if let Some(string) = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == "mpk.csharp.value.string.v1")
        {
            assert_eq!(string.depth, 19);
        }
        for sequent in boundary
            .sequents()
            .iter()
            .filter(|s| s.kind == "input_acceptance" || s.kind == "input_field")
        {
            assert_eq!(sequent.subjects[0].type_id, d.type_name);
        }
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
            checked_storage(&cert, d);
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "boundary document contexts {}, nonempty {}",
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
            read("ordinary-foundation/boundary-documents/certificates.json"),
            manifest
        );
    }
}
