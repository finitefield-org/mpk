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
fn with_program(f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryJsonKeywordDefinition)) {
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
    let p = generate_csharp_practical_ordinary_json_keywords(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}

// Independent byte-level oracle. Prefix recognition intentionally leaves the
// suffix to the enclosing parser; whole success is exact byte equality.
fn oracle(bytes: &[u8]) -> Option<(bool, bool, u32)> {
    [
        (b"null".as_slice(), true, false),
        (b"false".as_slice(), false, false),
        (b"true".as_slice(), false, true),
    ]
    .into_iter()
    .find(|(token, _, _)| bytes.starts_with(token))
    .map(|(token, null, boolean)| (null, boolean, token.len() as u32))
}
fn check_packet(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryJsonKeywordDefinition,
    input: V,
    expected: Option<(bool, bool, u32)>,
    whole: bool,
) {
    let output = run(cert, &d.parse_definition, vec![input]);
    let mut bits = [false; 64];
    if let Some((null, boolean, consumed)) = expected {
        bits[0] = true;
        bits[1] = whole;
        bits[2] = null;
        bits[3] = boolean;
        for i in 0..32 {
            bits[i + 4] = consumed & (1 << i) != 0;
        }
    }
    // Check every result and padding bit, including all high u32 count bits.
    for (i, bit) in bits.into_iter().enumerate() {
        assert_eq!(leaf(cert, output.clone(), 6, i), bit, "packet bit {i}");
    }
}
#[test]
fn csharp_03_t06_w09_json_keywords_semantics() {
    with_program(|cert, d| {
        let mut cases = BTreeSet::new();
        for token in [b"null".as_slice(), b"false".as_slice(), b"true".as_slice()] {
            cases.insert(token.to_vec());
            for end in 0..token.len() {
                cases.insert(token[..end].to_vec());
            }
            for at in 0..token.len() {
                for bit in 0..8 {
                    let mut bad = token.to_vec();
                    bad[at] ^= 1 << bit;
                    cases.insert(bad);
                }
            }
            for suffix in [
                b",".as_slice(),
                b"]",
                b"}",
                b":",
                b" ",
                b"true",
                &[0],
                &[255],
            ] {
                let mut bytes = token.to_vec();
                bytes.extend(suffix);
                cases.insert(bytes);
            }
            for prefix in [b" ".as_slice(), b"\n", b"\xef\xbb\xbf", &[34]] {
                let mut bytes = prefix.to_vec();
                bytes.extend(token);
                cases.insert(bytes);
            }
        }
        // Exhaust the first byte, including non-ASCII and NUL, without a host
        // parser in the generated certificate or the observation machine.
        for first in 0..=255 {
            cases.insert(vec![first, b'r', b'u', b'e']);
        }
        let count = cases.len();
        for (i, bytes) in cases.into_iter().enumerate() {
            let expected = oracle(&bytes);
            let whole = expected.is_some_and(|(_, _, n)| n as usize == bytes.len());
            let cells = bytes.iter().copied().enumerate().collect::<Vec<_>>();
            check_packet(
                cert,
                d,
                document(bytes.len() as u32, &cells),
                expected,
                whole,
            );
            if i % 64 == 0 {
                eprintln!("JSON keywords {i}/{count}");
            }
        }
        eprintln!("JSON keywords {count} lexical cases, all 64 result bits");
    });
}
#[test]
fn csharp_03_t06_w09_json_keywords_bounds_and_composition() {
    with_program(|cert, d| {
        const MAX: u32 = 1_048_576;
        for token in [b"null".as_slice(), b"false".as_slice(), b"true".as_slice()] {
            let cells = token.iter().copied().enumerate().collect::<Vec<_>>();
            let expected = oracle(token);
            for length in [0, 1, 3, 4, 5, 255, 65_536, MAX, MAX + 1, 1 << 31, u32::MAX] {
                let valid = length >= token.len() as u32 && length <= MAX;
                check_packet(
                    cert,
                    d,
                    document(length, &cells),
                    if valid { expected } else { None },
                    length == token.len() as u32,
                );
            }
            // Pass actual ordinary Slice/Concat closures into the parser.
            for at in [1, 64, 65_536, MAX - token.len() as u32] {
                let shifted = cells
                    .iter()
                    .map(|&(i, byte)| (at as usize + i, byte))
                    .collect::<Vec<_>>();
                let slice = run(
                    cert,
                    &d.fragments.slice_definition,
                    vec![document(MAX, &shifted), word(at), word(token.len() as u32)],
                );
                check_packet(cert, d, slice, expected, true);
            }
            for split in 0..=token.len() {
                let left = token[..split]
                    .iter()
                    .copied()
                    .enumerate()
                    .collect::<Vec<_>>();
                let right = token[split..]
                    .iter()
                    .copied()
                    .enumerate()
                    .collect::<Vec<_>>();
                let joined = run(
                    cert,
                    &d.fragments.concat_definition,
                    vec![
                        document(split as u32, &left),
                        document((token.len() - split) as u32, &right),
                    ],
                );
                check_packet(cert, d, joined, expected, true);
            }
        }
        eprintln!("JSON keywords: 33 full-u32 length cases, 12 high-offset slices, 16 token-split concatenations");
    });
}

#[test]
fn csharp_03_t06_w09_json_keywords_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_JSON_KEYWORDS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-keywords");
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
        let p = generate_csharp_practical_ordinary_json_keywords(vir)
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
            import_csharp_practical_ordinary_json_keywords(
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
            assert!(import_csharp_practical_ordinary_json_keywords(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_json_keywords(&p.canonical_bytes(), &bad, vir)
                .is_err()
        );
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_json_keywords(m, c, vir).is_err());
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
            check_packet(&cert, d, document(0, &[]), None, false);
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "JSON keywords contexts {}, nonempty {}",
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
            read("ordinary-foundation/json-keywords/certificates.json"),
            manifest
        );
    }
}
