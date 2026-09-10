use super::integer_format_tests::format_sources;
use super::*;
use core_eval::{apply, bit, sparse_cube};

fn observe(cert: &mpk_cert::encode::Certificate, value: &V, at: usize) -> bool {
    let mut value = value.clone();
    for i in 0..19 {
        value = apply(cert, value, V::Bit(at & (1 << i) != 0));
    }
    bit(value)
}
#[test]
fn csharp_03_t06_w09_decimal_formats_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_DECIMAL_FORMATS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-formats");
    let mut rows = vec![];
    let mut tested = None;
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
        let p = generate_csharp_practical_ordinary_decimal_formats(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_decimal_formats(
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
            assert!(import_csharp_practical_ordinary_decimal_formats(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_decimal_formats(&p.canonical_bytes(), &bad, vir)
                .is_err()
        );
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_decimal_formats(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if p.definitions().is_empty() {
            continue;
        }
        assert_eq!(p.definitions().len(), 1);
        assert_eq!(p.definitions()[0].codec_id, "decimal.normalized");
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
        if id == "literal-decimal" {
            tested = Some((
                p.definitions()[0].clone(),
                cert.clone(),
                emitted.closure().roots().clone(),
                emitted.closure().closed().clone(),
            ));
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "decimal format source {id}: {} terms",
            cert.term_table.len()
        );
    }
    assert_eq!(examined, 65);
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/decimal-formats/certificates.json"),
            manifest
        );
    }
    let (d, cert, roots, closed) = tested.expect("actual captured decimal literal source");
    let max = (1u128 << 96) - 1;
    let mut inputs = BTreeSet::from([
        (false, 0, 0u128),
        (true, 0, 0),
        (true, 28, 0),
        (false, 0, max),
        (true, 0, max),
        (false, 28, max),
        (true, 28, max),
        (false, 4, 12500),
        (true, 4, 12500),
        (false, 4, 123),
        (true, 4, 123),
        (false, 1, 10),
        (false, 0, 1u128 << 64),
        (true, 0, 1u128 << 95),
    ]);
    for scale in 0..=28 {
        inputs.insert((false, scale, 1));
        inputs.insert((true, scale, 10u128.pow(scale as u32)));
    }
    let oracle = BoundaryCodec::new("decimal.normalized", &d.value_type_id, None, None).unwrap();
    let mut count = 0;
    for (negative, scale, coefficient) in inputs {
        eprintln!("decimal normalized: sign={negative}, scale={scale}, coefficient={coefficient}");
        let value = MonomorphicValue::DecimalBits {
            type_id: d.value_type_id.clone(),
            negative,
            scale,
            coefficient: coefficient.to_string(),
        };
        let expected = oracle.format(&bundle, &roots, &closed, &value).unwrap();
        // Independent field layout; unused source padding is poisoned.
        let mut ones = BTreeSet::from([511]);
        if negative {
            ones.insert(0);
        }
        for i in 0..8 {
            if scale & (1 << i) != 0 {
                ones.insert(1 + i * 64);
            }
        }
        for i in 0..96 {
            if coefficient & (1u128 << i) != 0 {
                ones.insert(2 + i * 4);
            }
        }
        let result = run(&cert, &d.format_definition, vec![sparse_cube(9, ones)]);
        let len = (0..32).fold(0u32, |n, i| {
            n | (u32::from(observe(&cert, &result, i << 14)) << i)
        });
        assert_eq!(len, expected.len() as u32, "length: {value:?}");
        let actual = (0..len as usize)
            .map(|i| {
                (0..16).fold(0u16, |n, k| {
                    n | (u16::from(observe(&cert, &result, 1 | (i << 1) | (k << 15))) << k)
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "text: {value:?}");
        for pad in 1..14 {
            for k in 0..32 {
                assert!(!observe(&cert, &result, (1 << pad) | (k << 14)));
            }
        }
        let mut inactive = BTreeSet::from([
            len as usize,
            31,
            32,
            63,
            255,
            256,
            4095,
            4096,
            8191,
            8192,
            16383,
        ]);
        inactive.extend((0..14).map(|i| 1 << i).filter(|i| *i >= len as usize));
        for i in inactive {
            if i >= len as usize {
                for k in 0..16 {
                    assert!(
                        !observe(&cert, &result, 1 | (i << 1) | (k << 15)),
                        "inactive {i}:{k}"
                    );
                }
            }
        }
        count += 1;
    }
    eprintln!("decimal normalized observations: {count}");
}

#[test]
fn csharp_03_t06_w09_decimal_formats_pinned_sources() {
    let bundle = b();
    let manifest = read("ordinary-foundation/decimal-formats/certificates.json");
    let rows = manifest["sources"].as_array().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-formats");
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
        let p = generate_csharp_practical_ordinary_decimal_formats(emitted.vir()).unwrap();
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
    assert_eq!((examined, matched.len(), rows.len()), (65, 6, 6));
}
