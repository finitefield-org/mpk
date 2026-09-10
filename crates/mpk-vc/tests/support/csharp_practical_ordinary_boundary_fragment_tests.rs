use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

fn word(value: u32) -> V {
    sparse_cube(5, (0..32).filter(|i| value & (1 << i) != 0).collect())
}
fn document(length: u32, bytes: &[(usize, u8)]) -> V {
    let mut bits: BTreeSet<_> = (0..32)
        .filter(|i| length & (1 << i) != 0)
        .map(|i| i << 19)
        .collect();
    for &(index, byte) in bytes {
        assert!(index < 1_048_576);
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                bits.insert(1 | (index << 1) | (k << 21));
            }
        }
    }
    sparse_cube(24, bits)
}
fn leaf(cert: &mpk_cert::encode::Certificate, mut value: V, depth: usize, index: usize) -> bool {
    for i in 0..depth {
        value = apply(cert, value, V::Bit(index & (1 << i) != 0));
    }
    bit(value)
}
fn value_word(cert: &mpk_cert::encode::Certificate, value: V, depth: usize) -> u32 {
    (0..(1 << depth)).fold(0, |v, i| {
        v | ((leaf(cert, value.clone(), depth, i) as u32) << i)
    })
}
fn check_document(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryBoundaryFragmentDefinition,
    output: V,
    length: u32,
    samples: &[(u32, u8)],
) {
    assert_eq!(
        value_word(
            cert,
            run(cert, &d.document.length_definition, vec![output.clone()]),
            5
        ),
        length
    );
    for &(index, expected) in samples {
        let actual = run(
            cert,
            &d.document.read_byte_definition,
            vec![output.clone(), word(index)],
        );
        assert_eq!(
            value_word(cert, actual, 3),
            expected as u32,
            "index {index}, length {length}"
        );
    }
    // Header padding never becomes data, including after normalizing input.
    for i in 1..19 {
        assert!(!leaf(cert, output.clone(), 24, 1 << i));
    }
}
fn with_program(
    f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryBoundaryFragmentDefinition),
) {
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
    let p = generate_csharp_practical_ordinary_boundary_fragments(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}
#[test]
fn csharp_03_t06_w09_boundary_fragments_semantics() {
    with_program(|cert, d| {
        let cells = (0..256).map(|i| (i, i as u8)).collect::<Vec<_>>();
        let input = document(256, &cells);
        for start in 0..256 {
            let args = vec![input.clone(), word(start), word(1)];
            assert!(bit(run(cert, &d.slice_valid_definition, args.clone())));
            let output = run(cert, &d.slice_definition, args);
            check_document(cert, d, output, 1, &[(0, start as u8), (1, 0)]);
        }
        let mut range_cases = 0;
        for start in [0, 1, 63, 64, 127, 128, 255, 256] {
            for count in [0, 1, 2, 63, 128, 256] {
                let valid = count <= 256 - start;
                let args = vec![input.clone(), word(start), word(count)];
                assert_eq!(
                    bit(run(cert, &d.slice_valid_definition, args.clone())),
                    valid
                );
                let output = run(cert, &d.slice_definition, args);
                let length = if valid { count } else { 0 };
                let samples = [0, 1, length / 2, length.saturating_sub(1), length]
                    .map(|i| (i, if i < length { (start + i) as u8 } else { 0 }));
                check_document(cert, d, output, length, &samples);
                range_cases += 1;
            }
        }
        for left_len in [0, 1, 2, 7, 8, 16] {
            for right_len in [0, 1, 2, 7, 8, 16] {
                // Extra source cells are deliberately nonzero inactive storage.
                let left_cells = (0..17).map(|i| (i, 0x40 + i as u8)).collect::<Vec<_>>();
                let right_cells = (0..17).map(|i| (i, 0xa0 + i as u8)).collect::<Vec<_>>();
                let args = vec![
                    document(left_len, &left_cells),
                    document(right_len, &right_cells),
                ];
                assert!(bit(run(cert, &d.concat_valid_definition, args.clone())));
                let output = run(cert, &d.concat_definition, args);
                let length = left_len + right_len;
                let samples = (0..=length)
                    .map(|i| {
                        (
                            i,
                            if i < left_len {
                                0x40 + i as u8
                            } else if i < length {
                                0xa0 + (i - left_len) as u8
                            } else {
                                0
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                check_document(cert, d, output, length, &samples);
            }
        }
        eprintln!("boundary fragments: 256 single-byte values, {range_cases} slice ranges, 36 complete short concatenations");
    });
}
#[test]
fn csharp_03_t06_w09_boundary_fragments_full_bounds() {
    with_program(|cert, d| {
        const MAX: u32 = 1_048_576;
        // Every byte-address bit, plus carry across the half/full-capacity edge.
        for k in 0..20 {
            let at = 1u32 << k;
            let input = document(MAX, &[(at as usize, 0xa5), (MAX as usize - 1, 0x5a)]);
            let args = vec![input, word(at), word(1)];
            assert!(bit(run(cert, &d.slice_valid_definition, args.clone())));
            check_document(
                cert,
                d,
                run(cert, &d.slice_definition, args),
                1,
                &[(0, 0xa5), (1, 0)],
            );
            let args = vec![
                document(at, &[(at as usize - 1, 0x6d)]),
                document(1, &[(0, 0xd6)]),
            ];
            assert!(bit(run(cert, &d.concat_valid_definition, args.clone())));
            check_document(
                cert,
                d,
                run(cert, &d.concat_definition, args),
                at + 1,
                &[(at - 1, 0x6d), (at, 0xd6), (at + 1, 0)],
            );
        }
        for (left, right) in [(0, MAX), (MAX, 0), (MAX / 2, MAX / 2), (MAX - 1, 1)] {
            let left_cells = if left == 0 {
                vec![]
            } else {
                vec![(left as usize - 1, 0x6d)]
            };
            let right_cells = if right == 0 {
                vec![]
            } else {
                vec![(right as usize - 1, 0xd6)]
            };
            let args = vec![document(left, &left_cells), document(right, &right_cells)];
            assert!(bit(run(cert, &d.concat_valid_definition, args.clone())));
            let mut samples = vec![
                (MAX - 1, if right == 0 { 0x6d } else { 0xd6 }),
                (MAX, 0),
                (u32::MAX, 0),
            ];
            if left > 0 {
                samples.push((left - 1, 0x6d));
            }
            check_document(
                cert,
                d,
                run(cert, &d.concat_definition, args),
                MAX,
                &samples,
            );
        }
        for (start, count, valid) in [
            (MAX, 0, true),
            (MAX - 1, 1, true),
            (MAX - 1, 2, false),
            (MAX, 1, false),
            (u32::MAX, 1, false),
            (1, u32::MAX, false),
        ] {
            let args = vec![
                document(MAX, &[(MAX as usize - 1, 0x5a)]),
                word(start),
                word(count),
            ];
            assert_eq!(
                bit(run(cert, &d.slice_valid_definition, args.clone())),
                valid
            );
            let length = if valid { count } else { 0 };
            check_document(
                cert,
                d,
                run(cert, &d.slice_definition, args),
                length,
                &[(0, if length == 1 { 0x5a } else { 0 }), (1, 0)],
            );
        }
        for invalid in [MAX + 1, 1 << 31, u32::MAX] {
            for args in [
                vec![document(invalid, &[(0, 0xff)]), word(0), word(0)],
                vec![document(MAX, &[]), word(invalid), word(0)],
                vec![document(MAX, &[]), word(0), word(invalid)],
            ] {
                assert!(!bit(run(cert, &d.slice_valid_definition, args.clone())));
                check_document(
                    cert,
                    d,
                    run(cert, &d.slice_definition, args),
                    0,
                    &[(0, 0), (MAX - 1, 0)],
                );
            }
        }
        for (left, right) in [
            (MAX, 1),
            (1, MAX),
            (MAX, MAX),
            (u32::MAX, 1),
            (1, u32::MAX),
            (1 << 31, 1 << 31),
            (MAX + 1, 0),
            (0, MAX + 1),
        ] {
            let args = vec![document(left, &[(0, 0xff)]), document(right, &[(0, 0xff)])];
            assert!(!bit(run(cert, &d.concat_valid_definition, args.clone())));
            check_document(
                cert,
                d,
                run(cert, &d.concat_definition, args),
                0,
                &[(0, 0), (MAX - 1, 0)],
            );
        }
        eprintln!("boundary fragments: all 20 address bits, four exact 1 MiB concatenations, full-u32 overflow and empty end slices");
    });
}

#[test]
fn csharp_03_t06_w09_boundary_fragments_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BOUNDARY_FRAGMENTS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/boundary-fragments");
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
        let p = generate_csharp_practical_ordinary_boundary_fragments(vir)
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
            import_csharp_practical_ordinary_boundary_fragments(
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
            assert!(import_csharp_practical_ordinary_boundary_fragments(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_boundary_fragments(
            &p.canonical_bytes(),
            &bad,
            vir
        )
        .is_err());
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_boundary_fragments(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.document.depth, 24);
        assert!(d.static_transformers > 0 && d.static_transformers <= 16_384);
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
                &d.slice_valid_definition,
                vec![document(0, &[]), word(0), word(0)]
            )));
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "boundary fragments contexts {}, nonempty {}",
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
            read("ordinary-foundation/boundary-fragments/certificates.json"),
            manifest
        );
    }
}
