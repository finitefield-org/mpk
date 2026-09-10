use super::integer_format_tests::format_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

fn fixture() -> (
    mpk_cert::encode::Certificate,
    Vec<OrdinaryDecimalParseDefinition>,
) {
    let bundle = b();
    let (_, row, facts) = format_sources()
        .into_iter()
        .find(|(id, _, _)| id == "literal-decimal")
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
    let p = generate_csharp_practical_ordinary_decimal_parsers(emitted.vir()).unwrap();
    (
        mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap(),
        p.definitions().to_vec(),
    )
}
fn parsed_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryDecimalParseDefinition,
    input: &[u16],
    raw_length: Option<u32>,
) {
    let length = raw_length.unwrap_or(input.len() as u32);
    let oracle = BoundaryCodec::new(
        &d.codec_id,
        &d.value_type_id,
        d.scale,
        d.rounding.as_deref(),
    )
    .unwrap();
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
    // Inactive storage and header padding must never affect parsing.
    cells.insert(2);
    if length < 16384 {
        cells.insert(1 | (16383 << 1) | (15 << 15));
    }
    let value = run(cert, &d.parse_definition, vec![sparse_cube(19, cells)]);
    let mut wanted = BTreeSet::new();
    match expected {
        Ok(MonomorphicValue::DecimalBits {
            negative,
            scale,
            coefficient,
            ..
        }) => {
            let coefficient = coefficient.parse::<u128>().unwrap();
            if negative {
                wanted.insert(1);
            }
            for i in 0..8 {
                if scale & (1 << i) != 0 {
                    wanted.insert(1 | ((1 + (i << 6)) << 1));
                }
            }
            for i in 0..96 {
                if coefficient & (1u128 << i) != 0 {
                    wanted.insert(1 | ((2 + (i << 2)) << 1));
                }
            }
        }
        Ok(other) => panic!("unexpected oracle result: {other:?}"),
        Err(error) => {
            wanted.insert(0);
            let tag = match error {
                ParseErrorArm::InputBound => 0,
                ParseErrorArm::Syntax => 1,
                ParseErrorArm::Noncanonical => 2,
                ParseErrorArm::ScalePrecision => 3,
                ParseErrorArm::Range => 4,
            };
            for i in 0..32 {
                if tag & (1 << i) != 0 {
                    wanted.insert(1 | (i << 5));
                }
            }
        }
    }
    for at in 0..1024 {
        let mut leaf = value.clone();
        for i in 0..10 {
            leaf = apply(cert, leaf, V::Bit(at & (1 << i) != 0));
        }
        assert_eq!(
            bit(leaf),
            wanted.contains(&at),
            "{} {:?}/{:?}, input {input:?}, length {length}, bit {at}",
            d.codec_id,
            d.scale,
            d.rounding
        );
    }
}
#[test]
fn csharp_03_t06_w09_decimal_parsers_semantics() {
    let (cert, definitions) = fixture();
    let mut count = 0;
    for d in &definitions {
        let input = match d.scale {
            None | Some(0) => "1".into(),
            Some(s) => format!("1.{}", "0".repeat(s as usize)),
        };
        eprintln!(
            "decimal parser configuration {:?}/{:?}",
            d.scale, d.rounding
        );
        parsed_case(&cert, d, &input.encode_utf16().collect::<Vec<_>>(), None);
        count += 1;
    }
    let max = ((1u128 << 96) - 1).to_string();
    for d in definitions.iter().filter(|d| {
        d.scale.is_none()
            || d.rounding.as_deref() == Some("ToEven")
                && [Some(0), Some(1), Some(2), Some(28)].contains(&d.scale)
    }) {
        let mut inputs = BTreeSet::<String>::from_iter(
            [
                "", "+", "-", ".", "1.", ".1", "-.1", "1..2", "1.2.3", "1+2", "--1", "0", "-0",
                "+0", "00", "01", "-00.1", "+1", "1", "-1", "1.0", "-0.00", "0.01", "1.01", "1.10",
                " 1", "1 ", "1e2", "1,0", "00x", "+999x", "0.001", "-0.001",
            ]
            .map(Into::into),
        );
        inputs.extend([
            max.clone(),
            format!("-{max}"),
            (1u128 << 96).to_string(),
            format!("{max}.0"),
            format!("{max}.00"),
            format!("{max}.1"),
            format!("{max}.{}", "0".repeat(28)),
            format!("{max}.{}1", "0".repeat(27)),
            format!("0.{}1", "0".repeat(27)),
            format!("0.{}1", "0".repeat(28)),
            format!("0.{}", "0".repeat(29)),
            // Exact 192-bit wrap catches a lost sticky overflow even when
            // every wrapped high coefficient bit is zero.
            "6277101735386680763835789423207666416102355444464034512896".into(),
            "6277101735386680763835789423207666416102355444464034512897".into(),
            "9".repeat(59),
            "9".repeat(60),
            format!("{}.{}", "9".repeat(30), "0".repeat(28)),
            format!("+{}.{}", "9".repeat(30), "0".repeat(29)),
            format!("{}.{}x", "9".repeat(30), "0".repeat(29)),
        ]);
        for input in inputs {
            eprintln!("decimal parser {:?}: {input:?}", d.scale);
            parsed_case(&cert, d, &input.encode_utf16().collect::<Vec<_>>(), None);
            count += 1;
        }
        for bad in [0x80, 0xd800, 0xffff, 0xff11, 0x0131] {
            parsed_case(&cert, d, &[b'0' as u16, b'.' as u16, bad], None);
            count += 1;
        }
        for length in [16385, 65536, 1 << 31, u32::MAX] {
            parsed_case(&cert, d, &[b'+' as u16, b'x' as u16], Some(length));
            count += 1;
        }
    }
    eprintln!("decimal parser semantic observations: {count}");
}
#[test]
fn csharp_03_t06_w09_decimal_parsers_full_input_bound() {
    let (cert, definitions) = fixture();
    let normalized = definitions.iter().find(|d| d.scale.is_none()).unwrap();
    let fixed = definitions
        .iter()
        .find(|d| d.scale == Some(28) && d.rounding.as_deref() == Some("ToEven"))
        .unwrap();
    let mut syntax = vec![b'0' as u16; 16384];
    syntax[16383] = b'x' as u16;
    let noncanonical = vec![b'0' as u16; 16384];
    let mut precision = vec![b'1' as u16; 16384];
    precision[1] = b'.' as u16;
    let range = vec![b'9' as u16; 16384];
    for (name, d, input) in [
        ("late syntax", normalized, syntax),
        ("leading zero", normalized, noncanonical),
        ("precision", fixed, precision),
        ("range", normalized, range),
    ] {
        eprintln!("decimal parser full input: {name}");
        parsed_case(&cert, d, &input, None);
    }
}

#[test]
fn csharp_03_t06_w09_decimal_parsers_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_DECIMAL_PARSERS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-parsers");
    let mut rows = vec![];
    let mut definitions = 0;
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
        let p = generate_csharp_practical_ordinary_decimal_parsers(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_decimal_parsers(
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
            assert!(import_csharp_practical_ordinary_decimal_parsers(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_decimal_parsers(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_decimal_parsers(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if p.definitions().is_empty() {
            continue;
        }
        let old = super::super::structural_equivalence_tests::certificate(
            "decimal-parsers/previous-step-eight",
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
        assert_eq!(p.definitions().len(), 146);
        let configs = p
            .definitions()
            .iter()
            .map(|d| (d.scale, d.rounding.clone()))
            .collect::<BTreeSet<_>>();
        assert_eq!(configs.len(), 146);
        assert!(configs.contains(&(None, None)));
        for scale in 0..=28 {
            for rounding in [
                "ToEven",
                "AwayFromZero",
                "ToZero",
                "ToNegativeInfinity",
                "ToPositiveInfinity",
            ] {
                assert!(configs.contains(&(Some(scale), Some(rounding.into()))));
            }
        }
        definitions += p.definitions().len();
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "decimal parser source {id}: {} codecs",
            p.definitions().len()
        );
    }
    assert_eq!((examined, rows.len(), definitions), (65, 6, 876));
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/decimal-parsers/certificates.json"),
            manifest
        );
    }
}
