use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};
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
        assert!(at < 1048576);
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                cells.insert(1 | (at << 1) | (k << 21));
            }
        }
    }
    sparse_cube(24, cells)
}
fn observe(c: &mpk_cert::encode::Certificate, root: &V, depth: u32, index: usize) -> bool {
    let mut v = root.clone();
    for k in 0..depth {
        v = apply(c, v, V::Bit(index & (1 << k) != 0));
    }
    bit(v)
}
fn text_length(c: &mpk_cert::encode::Certificate, text: &V) -> u32 {
    let mut length = 0;
    for k in 0..32 {
        if observe(c, text, 19, k << 14) {
            length |= 1 << k;
        }
    }
    length
}
fn text_unit(c: &mpk_cert::encode::Certificate, text: &V, index: usize) -> u16 {
    let mut unit = 0;
    for k in 0..16 {
        if observe(c, text, 19, 1 | (index << 1) | (k << 15)) {
            unit |= 1 << k;
        }
    }
    unit
}
fn oracle(bytes: &[u8]) -> Option<Vec<u16>> {
    if bytes.len() > 1048576 {
        return None;
    }
    let v = parse_canonical_practical_json(PracticalArtifactKind::BoundaryInput, bytes).ok()?;
    let units = match v {
        PracticalJsonValue::String(s) => s.encode_utf16().collect::<Vec<_>>(),
        PracticalJsonValue::Utf16String(u) => u,
        _ => return None,
    };
    (units.len() <= 16384).then_some(units)
}
fn with_program(
    f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryJsonStringParseDefinition),
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
    let p = generate_csharp_practical_ordinary_json_string_parsers(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}
fn check(c: &mpk_cert::encode::Certificate, d: &OrdinaryJsonStringParseDefinition, bytes: &[u8]) {
    let expected = oracle(bytes);
    let input = document(bytes, bytes.len() as u32, &[]);
    assert_eq!(
        bit(run(c, &d.valid_definition, vec![input.clone()])),
        expected.is_some(),
        "JSON bytes {bytes:?}"
    );
    let value = run(c, &d.value_definition, vec![input]);
    let expected = expected.unwrap_or_default();
    assert_eq!(
        text_length(c, &value),
        expected.len() as u32,
        "JSON bytes {bytes:?}"
    );
    for (i, &unit) in expected.iter().enumerate() {
        assert_eq!(
            text_unit(c, &value, i),
            unit,
            "JSON bytes {bytes:?}, unit {i}"
        );
    }
    for i in [expected.len(), 8191, 8192, 16383] {
        if i < 16384 && i >= expected.len() {
            assert_eq!(text_unit(c, &value, i), 0);
        }
    }
    for pad in 1..14 {
        assert!(!observe(c, &value, 19, 1 << pad));
    }
}
#[test]
fn csharp_03_t06_w09_json_scan_regroup_boundaries() {
    with_program(|c, d| {
        // Exercise termination/error on either side of each four-packet group,
        // including packets with multi-byte input and two UTF-16 output units.
        for length in 0..=9 {
            let bytes = format!("\"{}\"", "a".repeat(length)).into_bytes();
            check(c, d, &bytes);
            let mut prefix = bytes.clone();
            prefix.extend(b",false");
            let input = document(&prefix, prefix.len() as u32, &[]);
            assert!(bit(run(c, &d.prefix_valid_definition, vec![input.clone()])));
            assert_eq!(
                consumed(c, &run(c, &d.consumed_definition, vec![input.clone()])),
                bytes.len() as u32
            );
            assert!(!bit(run(c, &d.valid_definition, vec![input])));
        }
        for bytes in [
            b"\"aaa\\u0000\"".as_slice(),
            b"\"aaaaaaa\\u001f\"",
            "\"aaa😀x\"".as_bytes(),
            "\"aaaaaaa😀x\"".as_bytes(),
            b"\"aaa\0\"",
            b"\"aaaaaaa\\u0041\"",
            b"\"aaaaaaa",
            b"\"aaa\xff\"",
        ] {
            check(c, d, bytes);
        }
        eprintln!("JSON regroup: 10 whole/prefix group-edge pairs and eight mixed/error cases");
    });
}

#[test]
fn csharp_03_t06_w09_json_string_parsers_semantics() {
    with_program(|c, d| {
        let mut cases = vec![
            b"\"\"".to_vec(),
            b"null".to_vec(),
            b"[]".to_vec(),
            b"0".to_vec(),
            vec![],
            vec![34],
            b" \"\"".to_vec(),
            b"\"\" ".to_vec(),
            b"\"a\"x".to_vec(),
        ];
        for byte in 0..=255 {
            cases.push(vec![34, byte, 34]);
        }
        for units in [
            vec![0, 8, 9, 10, 12, 13, 31, 34, 92, 47],
            vec![0x7f, 0x80, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xffff],
            vec![0xd800],
            vec![0xdc00],
            vec![0xd800, 0xdc00],
            vec![0xdbff, 0xdfff],
            vec![0xd800, 0xd800, 0xdc00],
            vec![0xd800, 97, 0xdc00],
        ] {
            cases.push(
                canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(units)).unwrap(),
            );
        }
        for text in [
            r#""\n""#,
            r#""\/""#,
            r#""\u0041""#,
            r#""\u0022""#,
            r#""\u005c""#,
            r#""\uD800""#,
            r#""\u00AF""#,
            r#""\ud800\udc00""#,
            r#""\udbff\udfff""#,
            r#""\ud800\ud800""#,
            r#""\udc00\ud800""#,
            r#""\ud800\"\udc00""#,
            r#""\u000""#,
            r#""\u000g""#,
            r#""a""b""#,
        ] {
            cases.push(text.as_bytes().to_vec());
        }
        for raw in [
            vec![0xc0, 0x80],
            vec![0xe0, 0x9f, 0xbf],
            vec![0xed, 0xa0, 0x80],
            vec![0xf0, 0x8f, 0xbf, 0xbf],
            vec![0xf4, 0x90, 0x80, 0x80],
            vec![0xf5, 0x80, 0x80, 0x80],
            vec![0xf0, 0x90],
            vec![0xef, 0xbb, 0xbf],
        ] {
            let mut bytes = vec![34];
            bytes.extend(raw);
            bytes.push(34);
            cases.push(bytes);
        }
        for (i, bytes) in cases.iter().enumerate() {
            if i % 16 == 0 {
                eprintln!("JSON string parser case {i}/{}", cases.len());
            }
            check(c, d, bytes);
        }
        for length in [1048577, 1 << 31, u32::MAX] {
            let input = document(b"\"\"", length, &[]);
            assert!(!bit(run(c, &d.valid_definition, vec![input.clone()])));
            let v = run(c, &d.value_definition, vec![input]);
            assert_eq!(text_length(c, &v), 0);
            assert_eq!(text_unit(c, &v, 0), 0);
        }
        let input = document(b"\"\"", 2, &[(2, 0xff), (1048575, 0xff)]);
        assert!(bit(run(c, &d.valid_definition, vec![input])));
        eprintln!(
            "JSON string parser {} oracle cases and full-u32/inactive storage cases",
            cases.len()
        );
    });
}
#[test]
fn csharp_03_t06_w09_json_string_parsers_full_bound() {
    with_program(|c, d| {
        for units in [
            vec![0; 16384],
            vec![97; 16384],
            [0xd800, 0xdc00].repeat(8192),
            vec![97; 16385],
        ] {
            let bytes =
                canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(units.clone()))
                    .unwrap();
            let good = units.len() <= 16384;
            eprintln!(
                "JSON string parse full bounds: {} bytes, {} units, valid {good}",
                bytes.len(),
                units.len()
            );
            let input = document(&bytes, bytes.len() as u32, &[]);
            assert_eq!(bit(run(c, &d.valid_definition, vec![input.clone()])), good);
            let value = run(c, &d.value_definition, vec![input]);
            assert_eq!(
                text_length(c, &value),
                if good { units.len() as u32 } else { 0 }
            );
            for i in [
                0, 1, 2, 31, 32, 63, 64, 127, 128, 255, 256, 511, 512, 1023, 1024, 2047, 2048,
                4095, 4096, 8191, 8192, 16382, 16383,
            ] {
                assert_eq!(text_unit(c, &value, i), if good { units[i] } else { 0 });
            }
        }
    });
}
#[test]
fn csharp_03_t06_w09_json_string_parsers_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_JSON_STRING_PARSERS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-string-parsers");
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
        let p = generate_csharp_practical_ordinary_json_string_parsers(vir)
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
            import_csharp_practical_ordinary_json_string_parsers(
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
            assert!(import_csharp_practical_ordinary_json_string_parsers(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_json_string_parsers(
            &p.canonical_bytes(),
            &bad,
            vir
        )
        .is_err());
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_json_string_parsers(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.document.depth, 24);
        assert_eq!(d.fragments.document, d.document);
        assert_ne!(d.valid_definition, d.prefix_valid_definition);
        assert_ne!(d.value_definition, d.prefix_value_definition);
        assert_eq!(d.state_depth, 19);
        assert_eq!(d.scan_steps, 8_193);
        assert!(d.static_transformers > 4_096 && d.static_transformers <= 8_192);
        let previous = fs::read_to_string(
            root.join("previous-pair-pipeline")
                .join(format!("{id}.hex")),
        )
        .unwrap();
        let previous = (0..previous.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&previous[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let previous = mpk_cert::decode_canonical_certificate(&previous).unwrap();
        super::json_token_tests::same_json_scan(&previous, &cert).unwrap();
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
            check(&cert, d, b"\"\"");
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "JSON string parser contexts {}, nonempty {}",
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
            super::read("ordinary-foundation/json-string-parsers/certificates.json"),
            manifest
        );
    }
}

fn prefix_oracle(bytes: &[u8]) -> Option<(usize, Vec<u16>)> {
    // Independent whole-document oracle, trying exact short prefixes. No
    // ordinary decoder result chooses the oracle's token endpoint or value.
    if bytes.len() > 1048576 {
        return None;
    }
    (2..=bytes.len()).find_map(|end| oracle(&bytes[..end]).map(|units| (end, units)))
}
fn consumed(c: &mpk_cert::encode::Certificate, value: &V) -> u32 {
    (0..32).fold(0, |n, i| n | ((observe(c, value, 5, i) as u32) << i))
}
fn index_word(value: u32) -> V {
    sparse_cube(5, (0..32).filter(|i| value & (1 << i) != 0).collect())
}
#[test]
fn csharp_03_t06_w09_json_string_prefix_semantics() {
    with_program(|c, d| {
        let mut cases = vec![];
        for units in [
            vec![],
            vec![97, 34, 92],
            vec![0, 10, 31],
            vec![0xd800],
            vec![0xdc00],
            vec![0xdbff, 0xdfff],
            vec![0xd800, 97, 0xdc00],
            vec![0x7f, 0x80, 0x7ff, 0x800, 0xffff],
        ] {
            let token =
                canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(units)).unwrap();
            for suffix in [
                b"".as_slice(),
                b",null]",
                b":true}",
                b" ",
                &[0xff, 0x00],
                b"\"next\"",
            ] {
                let mut bytes = token.clone();
                bytes.extend(suffix);
                cases.push(bytes);
            }
        }
        for raw in [
            b"".as_slice(),
            b"\"",
            b"null",
            b" \"\"",
            b"\"unterminated",
            b"\"\\n\"",
            b"\"\\u000g\"",
            b"\"\\ud800\\udc00\"",
            b"\"\\u0041\"",
            b"\"a\"b\"",
            b"\"\\\"\"tail",
            &[34, 0xf4, 0x90, 0x80, 0x80, 34],
            &[34, 0xf0, 0x90],
            &[34, 0, 34],
        ] {
            cases.push(raw.to_vec());
        }
        for (ordinal, bytes) in cases.iter().enumerate() {
            let expected = prefix_oracle(bytes);
            let input = document(bytes, bytes.len() as u32, &[]);
            assert_eq!(
                bit(run(c, &d.prefix_valid_definition, vec![input.clone()])),
                expected.is_some(),
                "prefix {bytes:?}"
            );
            let end = run(c, &d.consumed_definition, vec![input.clone()]);
            assert_eq!(
                consumed(c, &end) as usize,
                expected.as_ref().map(|e| e.0).unwrap_or(0),
                "consumed {bytes:?}"
            );
            let whole = expected.as_ref().is_some_and(|e| e.0 == bytes.len());
            assert_eq!(
                bit(run(c, &d.valid_definition, vec![input.clone()])),
                whole,
                "whole {bytes:?}"
            );
            let value = run(c, &d.prefix_value_definition, vec![input.clone()]);
            let units = expected.as_ref().map(|e| e.1.as_slice()).unwrap_or(&[]);
            assert_eq!(text_length(c, &value), units.len() as u32);
            for (i, &unit) in units.iter().enumerate() {
                assert_eq!(text_unit(c, &value, i), unit, "prefix {bytes:?}, unit {i}");
            }
            for i in [units.len(), 16383] {
                if i >= units.len() && i < 16384 {
                    assert_eq!(text_unit(c, &value, i), 0);
                }
            }
            if expected.is_some() && !whole {
                let value = run(c, &d.value_definition, vec![input]);
                assert_eq!(text_length(c, &value), 0, "whole value leaked a prefix");
                assert_eq!(text_unit(c, &value, 0), 0);
            }
            if ordinal % 8 == 0 {
                eprintln!("JSON string prefix {ordinal}/{}", cases.len());
            }
        }
        eprintln!(
            "JSON string prefix: {} exact endpoint/value/whole-document oracle cases",
            cases.len()
        );
    });
}
#[test]
fn csharp_03_t06_w09_json_string_prefix_document_bounds_and_slices() {
    with_program(|c, d| {
        assert_eq!(d.fragments.document, d.document);
        for length in [2, 3, 255, 256, 65536, 1048576] {
            let input = document(b"\"\"", length, &[(2, 0xff), (1048575, 0xff)]);
            assert!(bit(run(c, &d.prefix_valid_definition, vec![input.clone()])));
            assert_eq!(
                consumed(c, &run(c, &d.consumed_definition, vec![input.clone()])),
                2
            );
            assert_eq!(bit(run(c, &d.valid_definition, vec![input])), length == 2);
        }
        for length in [0, 1, 1048577, 1 << 31, u32::MAX] {
            let input = document(b"\"\"", length, &[]);
            assert!(!bit(run(
                c,
                &d.prefix_valid_definition,
                vec![input.clone()]
            )));
            assert_eq!(
                consumed(c, &run(c, &d.consumed_definition, vec![input.clone()])),
                0
            );
            let value = run(c, &d.prefix_value_definition, vec![input]);
            assert_eq!(text_length(c, &value), 0);
            assert_eq!(text_unit(c, &value, 0), 0);
        }
        let tokens = [vec![97, 34, 92], vec![0xd800], vec![0xd800, 0xdc00]];
        for units in tokens {
            let token =
                canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(units.clone()))
                    .unwrap();
            for start in [1, 64, 65536, 1048576 - token.len() as u32 - 1] {
                let mut cells = token
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (start as usize + i, v))
                    .collect::<Vec<_>>();
                cells.push((start as usize + token.len(), b','));
                let length = start + token.len() as u32 + 1;
                let doc = document(&[], length, &cells);
                // Pass the ordinary slice closure straight into the parser in
                // the same certificate; no host copy or re-encoded token.
                let args = vec![doc, index_word(start), index_word(length - start)];
                assert!(bit(run(
                    c,
                    &d.fragments.slice_valid_definition,
                    args.clone()
                )));
                let suffix = run(c, &d.fragments.slice_definition, args);
                assert!(bit(run(
                    c,
                    &d.prefix_valid_definition,
                    vec![suffix.clone()]
                )));
                assert_eq!(
                    consumed(c, &run(c, &d.consumed_definition, vec![suffix.clone()])),
                    token.len() as u32
                );
                assert!(!bit(run(c, &d.valid_definition, vec![suffix.clone()])));
                let value = run(c, &d.prefix_value_definition, vec![suffix]);
                assert_eq!(text_length(c, &value), units.len() as u32);
                for (i, &unit) in units.iter().enumerate() {
                    assert_eq!(text_unit(c, &value, i), unit);
                }
            }
        }
        eprintln!("JSON prefix: full document/u32 bounds and 12 ordinary Slice-to-parser compositions across high byte offsets");
    });
}
#[test]
fn csharp_03_t06_w09_json_string_prefix_full_decoded_bound() {
    with_program(|c, d| {
        for length in [16384, 16385] {
            let mut bytes = vec![34];
            bytes.extend(vec![97; length]);
            bytes.extend([34, b',']);
            let input = document(&bytes, bytes.len() as u32, &[]);
            let good = length == 16384;
            eprintln!("JSON prefix full decoded bound {length}, suffix comma");
            assert_eq!(
                bit(run(c, &d.prefix_valid_definition, vec![input.clone()])),
                good
            );
            assert_eq!(
                consumed(c, &run(c, &d.consumed_definition, vec![input.clone()])),
                if good { length as u32 + 2 } else { 0 }
            );
            let value = run(c, &d.prefix_value_definition, vec![input]);
            assert_eq!(text_length(c, &value), if good { length as u32 } else { 0 });
            for i in [0, 1, 63, 64, 8191, 8192, 16383] {
                assert_eq!(text_unit(c, &value, i), if good { 97 } else { 0 });
            }
        }
    });
}

#[test]
fn csharp_03_t06_w09_json_string_prefix_full_control_consumption() {
    with_program(|c, d| {
        // Decoded ASCII capacity alone does not reach consumed-byte bit 16.
        // Canonical controls require six bytes each, so this exact full string
        // catches truncation of the new consumed-byte result above 65,535.
        let mut bytes =
            canonical_practical_json_bytes(&PracticalJsonValue::utf16_string(vec![0; 16384]))
                .unwrap();
        assert_eq!(bytes.len(), 98306);
        bytes.push(b',');
        let input = document(&bytes, bytes.len() as u32, &[]);
        eprintln!("JSON prefix full controls: 98306 consumed bytes, 16384 units, suffix comma");
        assert!(bit(run(c, &d.prefix_valid_definition, vec![input.clone()])));
        assert_eq!(
            consumed(c, &run(c, &d.consumed_definition, vec![input.clone()])),
            98306
        );
        let value = run(c, &d.prefix_value_definition, vec![input]);
        assert_eq!(text_length(c, &value), 16384);
        for i in [0, 63, 64, 8191, 8192, 16383] {
            assert_eq!(text_unit(c, &value, i), 0);
        }
    });
}
