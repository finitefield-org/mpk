use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};
fn input(units: &[u16], length: u32, inactive: &[(usize, u16)]) -> V {
    let mut cells = BTreeSet::new();
    for k in 0..32 {
        if length & (1 << k) != 0 {
            cells.insert(k << 14);
        }
    }
    for (at, unit) in units
        .iter()
        .copied()
        .enumerate()
        .chain(inactive.iter().copied())
    {
        assert!(at < 16384);
        for k in 0..16 {
            if unit & (1 << k) != 0 {
                cells.insert(1 | (at << 1) | (k << 15));
            }
        }
    }
    sparse_cube(19, cells)
}
fn word(n: u32, depth: u32) -> V {
    sparse_cube(depth, (0..32).filter(|i| n & (1 << i) != 0).collect())
}
fn read(c: &mpk_cert::encode::Certificate, mut v: V, depth: u32) -> u64 {
    let root = v.clone();
    let mut result = 0;
    for k in 0..1usize << depth {
        v = root.clone();
        for s in 0..depth {
            v = apply(c, v, V::Bit(k & (1 << s) != 0));
        }
        if bit(v) {
            result |= 1u64 << k;
        }
    }
    result
}
fn oracle(units: &[u16]) -> Vec<u8> {
    canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(units.to_vec())).unwrap()
}
fn with_program(f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryJsonStringDefinition)) {
    let bundle = b();
    let (_, row, facts) = document_sources()
        .into_iter()
        .find(|(id, _, _)| id.starts_with("document-"))
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_json_strings(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}
fn check(c: &mpk_cert::encode::Certificate, d: &OrdinaryJsonStringDefinition, units: &[u16]) {
    let expected = oracle(units);
    let text = input(units, units.len() as u32, &[]);
    assert!(bit(run(c, &d.bounded_definition, vec![text.clone()])));
    let doc = run(c, &d.format_definition, vec![text]);
    assert_eq!(
        read(
            c,
            run(c, &d.document.length_definition, vec![doc.clone()]),
            5
        ),
        expected.len() as u64
    );
    for (i, byte) in expected.iter().enumerate() {
        let got = run(
            c,
            &d.document.read_byte_definition,
            vec![doc.clone(), word(i as u32, 5)],
        );
        assert_eq!(read(c, got, 3), *byte as u64, "units {units:?}, byte {i}");
    }
    for i in [expected.len(), 16384, 98305, 1048575] {
        if i < expected.len() {
            continue;
        }
        let got = run(
            c,
            &d.document.read_byte_definition,
            vec![doc.clone(), word(i as u32, 5)],
        );
        assert_eq!(read(c, got, 3), 0);
    }
}
#[test]
fn csharp_03_t06_w09_json_strings_semantics() {
    with_program(|c, d| {
        let mut cases = vec![
            vec![],
            vec![34, 92, 47, 8, 9, 10, 12, 13, 0],
            vec![0xd800, 0xdc00],
            vec![0xdbff, 0xdfff],
            vec![0xd800, 0xd800, 0xdc00],
            vec![0xdc00, 0xd800],
            vec![0xd800, 97, 0xdc00],
            vec![0xd800, 34, 0xdc00],
        ];
        for u in 0..=255 {
            cases.push(vec![u]);
        }
        for u in [
            0x100, 0x7ff, 0x800, 0xd7ff, 0xd800, 0xdbff, 0xdc00, 0xdfff, 0xe000, 0xffff,
        ] {
            cases.push(vec![u]);
            for v in [0, 0xd7ff, 0xd800, 0xdbff, 0xdc00, 0xdfff, 0xe000] {
                cases.push(vec![u, v]);
            }
        }
        for (i, units) in cases.iter().enumerate() {
            if i % 16 == 0 {
                eprintln!("JSON string case {i}/{}", cases.len());
            }
            check(c, d, units);
        }
        for length in [16385, 1 << 31, u32::MAX] {
            let text = input(&[], length, &[]);
            assert!(!bit(run(c, &d.bounded_definition, vec![text.clone()])));
            let doc = run(c, &d.format_definition, vec![text]);
            assert_eq!(
                read(c, run(c, &d.document.length_definition, vec![doc]), 5),
                0
            );
        }
        for units in [vec![], vec![0xd800], vec![97]] {
            let text = input(
                &units,
                units.len() as u32,
                &[(units.len(), 0xdc00), (16383, 0xdfff)],
            );
            let doc = run(c, &d.format_definition, vec![text]);
            let expected = oracle(&units);
            assert_eq!(
                read(
                    c,
                    run(c, &d.document.length_definition, vec![doc.clone()]),
                    5
                ),
                expected.len() as u64
            );
            for (i, byte) in expected.iter().enumerate() {
                assert_eq!(
                    read(
                        c,
                        run(
                            c,
                            &d.document.read_byte_definition,
                            vec![doc.clone(), word(i as u32, 5)]
                        ),
                        3
                    ),
                    *byte as u64
                );
            }
        }
        eprintln!(
            "JSON string {} oracle cases plus full-u32 bounds and inactive storage",
            cases.len()
        );
    });
}
#[test]
fn csharp_03_t06_w09_json_strings_full_bound() {
    with_program(|c, d| {
        for paired in [false, true] {
            let units = if paired {
                [0xd800, 0xdc00].repeat(8192)
            } else {
                vec![0; 16384]
            };
            eprintln!("JSON string full UTF-16 capacity, paired={paired}");
            let expected = oracle(&units);
            let doc = run(c, &d.format_definition, vec![input(&units, 16384, &[])]);
            assert_eq!(
                read(
                    c,
                    run(c, &d.document.length_definition, vec![doc.clone()]),
                    5
                ),
                expected.len() as u64
            );
            // Full prefix-sum construction is demanded by the output length;
            // observe both ends and every storage address-bit crossing.
            let mut positions = BTreeSet::new();
            for edge in [
                0,
                1,
                2,
                16,
                32,
                64,
                128,
                256,
                512,
                1024,
                2048,
                4096,
                8192,
                16384,
                32768,
                65536,
                expected.len() - 1,
            ] {
                for i in edge.saturating_sub(3)..=(edge + 3).min(expected.len()) {
                    positions.insert(i);
                }
            }
            for i in positions {
                let actual = read(
                    c,
                    run(
                        c,
                        &d.document.read_byte_definition,
                        vec![doc.clone(), word(i as u32, 5)],
                    ),
                    3,
                );
                assert_eq!(
                    actual,
                    expected.get(i).copied().unwrap_or(0) as u64,
                    "paired {paired}, byte {i}"
                );
            }
        }
    });
}
#[test]
fn csharp_03_t06_w09_json_strings_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_JSON_STRINGS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-strings");
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
        let p = generate_csharp_practical_ordinary_json_strings(vir)
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
            import_csharp_practical_ordinary_json_strings(
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
            assert!(import_csharp_practical_ordinary_json_strings(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_strings(&p.canonical_bytes(), &bad, vir).is_err()
        );
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_json_strings(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.document.depth, 24);
        assert_eq!(d.input_depth, 19);
        assert_eq!(d.state_depth, 24);
        assert_eq!(d.scan_steps, 8_192);
        assert!(d.static_transformers > 8_194 && d.static_transformers <= 16_384);
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
            check(&cert, d, &[]);
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!("JSON string contexts {}, nonempty {}", examined, rows.len());
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            super::read("ordinary-foundation/json-strings/certificates.json"),
            manifest
        );
    }
}
