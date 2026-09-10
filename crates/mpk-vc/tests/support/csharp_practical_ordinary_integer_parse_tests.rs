use super::integer_format_tests::format_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::ordinary_carriers) fn parsed_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryIntegerParseDefinition,
    input: &[u16],
    length: Option<u32>,
) {
    let length = length.unwrap_or(input.len() as u32);
    let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
    let expected = if length > 16384 {
        Err(ParseErrorArm::InputBound)
    } else {
        oracle.parse(input)
    };
    let mut cells = BTreeSet::new();
    for i in 0..32 {
        if length & (1 << i) != 0 {
            cells.insert(i << 14);
        }
    }
    for (i, ch) in input.iter().take(16384).enumerate() {
        for k in 0..16 {
            if ch & (1 << k) != 0 {
                cells.insert(1 | (i << 1) | (k << 15));
            }
        }
    }
    if length < 16384 {
        cells.insert(1 | (16383 << 1) | (15 << 15));
    }
    let value = run(cert, &d.parse_definition, vec![sparse_cube(19, cells)]);
    let depth = d.value_depth.max(5) + 1;
    let mut wanted = BTreeSet::new();
    match expected {
        Ok(value) => {
            let n = match value {
                MonomorphicValue::Signed { value, .. }
                | MonomorphicValue::Unsigned { value, .. } => {
                    value.parse::<i128>().unwrap() as u128
                }
                MonomorphicValue::Duration { ticks, .. } => ticks.parse::<i128>().unwrap() as u128,
                MonomorphicValue::Instant { milliseconds, .. } => {
                    milliseconds.parse::<i128>().unwrap() as u128
                }
                _ => panic!(),
            };
            for k in 0..1usize << d.value_depth {
                if n & (1u128 << k) != 0 {
                    wanted.insert(1 | (k << (depth - d.value_depth)));
                }
            }
        }
        Err(error) => {
            wanted.insert(0);
            let tag = match error {
                ParseErrorArm::InputBound => 0,
                ParseErrorArm::Syntax => 1,
                ParseErrorArm::Noncanonical => 2,
                ParseErrorArm::Range => 4,
                _ => panic!(),
            };
            for k in 0..32 {
                if tag & (1 << k) != 0 {
                    wanted.insert(1 | (k << (depth - 5)));
                }
            }
        }
    }
    for at in 0..1usize << depth {
        let mut v = value.clone();
        for i in 0..depth {
            v = apply(cert, v, V::Bit(at & (1 << i) != 0));
        }
        assert_eq!(
            bit(v),
            wanted.contains(&at),
            "{} input {input:?}, length {length}, result bit {at}",
            d.codec_id
        );
    }
}
#[test]
fn csharp_03_t06_w09_integer_parsers_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_INTEGER_PARSERS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-parsers");
    let mut rows = vec![];
    let mut unique = BTreeMap::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut examined = 0;
    for (id, row, facts) in format_sources() {
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
        let p = generate_csharp_practical_ordinary_integer_parsers(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_integer_parsers(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "certificate_sha256",
        ] {
            let mut m = meta.clone();
            m[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_integer_parsers(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_integer_parsers(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_integer_parsers(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if p.definitions().is_empty() {
            continue;
        }
        let old = super::super::structural_equivalence_tests::certificate(
            "integer-parsers/previous-step-eight",
            &format!("{id}.hex"),
        );
        super::super::structural_equivalence_tests::same_aggregate_scan(&old, &cert)
            .unwrap_or_else(|e| panic!("{id} aggregate preservation: {e}"));
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
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        for d in p.definitions() {
            unique
                .entry(d.codec_id.clone())
                .or_insert((d.clone(), cert.clone()));
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "integer parser source {id}: {} codecs",
            p.definitions().len()
        );
    }
    assert_eq!((examined, rows.len(), unique.len()), (65, 54, 10));
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/integer-parsers/certificates.json"),
            manifest
        );
    }
    let mut count = 0;
    for (id, (d, cert)) in unique {
        let width = 1u32 << d.value_depth;
        let max = if d.signed {
            (1u128 << (width - 1)) - 1
        } else {
            (1u128 << width) - 1
        };
        let mut inputs = BTreeSet::from([
            "".into(),
            "0".into(),
            "1".into(),
            "9".into(),
            "10".into(),
            "99".into(),
            "100".into(),
            "-1".into(),
            "-0".into(),
            "+0".into(),
            "+1".into(),
            "00".into(),
            "01".into(),
            "-00".into(),
            "+00".into(),
            "-".into(),
            "+".into(),
            "--1".into(),
            "+-1".into(),
            "1x".into(),
            "00x".into(),
            "+999x".into(),
            " 1".into(),
            "1 ".into(),
            "1.0".into(),
            "1e2".into(),
            "1,000".into(),
            max.to_string(),
            (max - 1).to_string(),
            (max + 1).to_string(),
            u64::MAX.to_string(),
            (u64::MAX as u128 + 1).to_string(),
            "9".repeat(21),
            "+".to_owned() + &"9".repeat(21),
            "0".repeat(24),
            "9".repeat(20) + "x",
        ]);
        if d.signed {
            inputs.extend([
                format!("-{}", max),
                format!("-{}", max + 1),
                format!("-{}", max + 2),
            ]);
        }
        for text in inputs {
            eprintln!("integer parser {id}: {text:?}");
            parsed_case(&cert, &d, &text.encode_utf16().collect::<Vec<_>>(), None);
            count += 1;
        }
        for text in [
            vec![0x130],
            vec![b'0' as u16, 0x130],
            vec![b'+' as u16, 0xd800],
            vec![0xff11],
            vec![b'1' as u16, 0],
        ] {
            parsed_case(&cert, &d, &text, None);
            count += 1;
        }
        for length in [16385, u32::MAX, 1 << 31] {
            parsed_case(&cert, &d, &[b'x' as u16], Some(length));
            count += 1;
        }
    }
    eprintln!("integer parser short observations: {count}");
}
#[test]
fn csharp_03_t06_w09_integer_parsers_full_input_bound() {
    let bundle = b();
    let (_, row, facts) = format_sources()
        .into_iter()
        .find(|(id, _, _)| id == "literal-i64")
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
    let p = generate_csharp_practical_ordinary_integer_parsers(emitted.vir()).unwrap();
    let d = p
        .definitions()
        .iter()
        .find(|d| d.codec_id == "integer.i64")
        .unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    for (name, text) in [
        ("maximum valid digits", vec![b'9' as u16; 16384]),
        ("maximum redundant zeros", vec![b'0' as u16; 16384]),
        ("last invalid after zeros", {
            let mut s = vec![b'0' as u16; 16384];
            s[16383] = 0x130;
            s
        }),
        ("input bound precedes all errors", vec![0xd800; 16385]),
    ] {
        eprintln!("integer parser full input: {name}");
        parsed_case(&cert, d, &text, None);
    }
}

#[test]
fn csharp_03_t06_w09_integer_parsers_pinned_sources() {
    let bundle = b();
    let manifest = read("ordinary-foundation/integer-parsers/certificates.json");
    let rows = manifest["sources"].as_array().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-parsers");
    let mut matched = BTreeSet::new();
    let mut examined = 0;
    for (id, row, facts) in format_sources() {
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
        let p = generate_csharp_practical_ordinary_integer_parsers(emitted.vir()).unwrap();
        if p.definitions().is_empty() {
            assert!(!rows.iter().any(|r| r["id"] == id));
            continue;
        }
        let row = rows.iter().find(|r| r["id"] == id).unwrap();
        assert_eq!(
            row["metadata"],
            serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap()
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        assert_eq!(
            fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
            hex
        );
        assert!(matched.insert(id));
    }
    assert_eq!((examined, matched.len(), rows.len()), (65, 54, 54));
}
