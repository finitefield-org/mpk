use super::*;
use core_eval::{apply, bit, sparse_cube};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn csharp_03_t06_w09_hex_codecs_pinned_sources() {
    let bundle = b();
    let manifest = read("ordinary-foundation/hex-codecs/certificates.json");
    let rows = manifest["sources"].as_array().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/hex-codecs");
    let mut matched = BTreeSet::new();
    let mut examined = 0;
    for (id, row, facts) in sources() {
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
        let p = generate_csharp_practical_ordinary_hex_codecs(emitted.vir()).unwrap();
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
    assert_eq!((examined, matched.len(), rows.len()), (64, 11, 11));
}

fn text_cells(text: &[u16], length: u32) -> BTreeSet<usize> {
    let mut ones = BTreeSet::new();
    for i in 0..32 {
        if length & (1 << i) != 0 {
            ones.insert(i << 14);
        }
    }
    for (i, c) in text.iter().enumerate() {
        for bit in 0..16 {
            if c & (1 << bit) != 0 {
                ones.insert(1 | (i << 1) | (bit << 15));
            }
        }
    }
    ones
}
fn scalar_cells(value: u128, width: usize) -> BTreeSet<usize> {
    (0..width).filter(|i| value & (1u128 << i) != 0).collect()
}
fn observed(cert: &mpk_cert::encode::Certificate, result: &V, depth: u32, at: usize) -> bool {
    let mut v = result.clone();
    for i in 0..depth {
        v = apply(cert, v, V::Bit(at & (1 << i) != 0));
    }
    bit(v)
}
pub(in crate::ordinary_carriers) fn parse_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryHexCodecDefinition,
    input: &[u16],
    override_length: Option<u32>,
) {
    let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
    let length = override_length.unwrap_or(input.len() as u32);
    let expected = if length > 16384 {
        Err(ParseErrorArm::InputBound)
    } else {
        oracle.parse(input)
    };
    let mut cells = text_cells(input, length);
    // Inactive non-ASCII storage must not alter accepted text or its error.
    cells.insert(1 | (16383 << 1) | (15 << 15));
    let result = run(cert, &d.parse_definition, vec![sparse_cube(19, cells)]);
    let depth = d.value_depth.max(5) + 1;
    let mut wanted = BTreeSet::new();
    match expected {
        Ok(value) => {
            let n = match value {
                MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
                    u128::from_str_radix(&bits, 16).unwrap()
                }
                MonomorphicValue::Guid { n, .. } => u128::from_str_radix(&n, 16).unwrap(),
                _ => panic!(),
            };
            wanted.extend(
                scalar_cells(n, 1 << d.value_depth)
                    .into_iter()
                    .map(|at| 1 | (at << 1)),
            );
        }
        Err(error) => {
            wanted.insert(0);
            let tag = match error {
                ParseErrorArm::InputBound => 0,
                ParseErrorArm::Syntax => 1,
                ParseErrorArm::Noncanonical => 2,
                _ => panic!(),
            };
            for i in 0..32 {
                if tag & (1 << i) != 0 {
                    wanted.insert(1 | (i << (depth - 5)));
                }
            }
        }
    }
    for at in 0..1usize << depth {
        assert_eq!(
            observed(cert, &result, depth, at),
            wanted.contains(&at),
            "{} parse {input:?} length {length}: bit {at}",
            d.codec_id
        );
    }
}
fn format_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryHexCodecDefinition,
    n: u128,
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) {
    let width = 1usize << d.value_depth;
    let value = match d.codec_id.as_str() {
        "binary32" => MonomorphicValue::F32Bits {
            type_id: d.value_type_id.clone(),
            bits: format!("{n:08x}"),
        },
        "binary64" => MonomorphicValue::F64Bits {
            type_id: d.value_type_id.clone(),
            bits: format!("{n:016x}"),
        },
        _ => MonomorphicValue::Guid {
            type_id: d.value_type_id.clone(),
            n: format!("{n:032x}"),
        },
    };
    let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
    let text = oracle.format(bundle, roots, closed, &value).unwrap();
    let wanted = text_cells(&text, text.len() as u32);
    let result = run(
        cert,
        &d.format_definition,
        vec![sparse_cube(d.value_depth, scalar_cells(n, width))],
    );
    let mut addresses = wanted.clone();
    addresses.extend((0..32).map(|i| i << 14));
    for padding in 1..14 {
        addresses.extend((0..32).map(|i| (1 << padding) | (i << 14)));
    }
    // Every active unit and first inactive unit; every capacity index's low
    // character bit; high character bits in the final inactive unit.
    for i in 0..=text.len() {
        for k in 0..16 {
            addresses.insert(1 | (i << 1) | (k << 15));
        }
    }
    addresses.extend((0..16384).map(|i| 1 | (i << 1)));
    addresses.extend((0..16).map(|k| 1 | (16383 << 1) | (k << 15)));
    for at in addresses {
        assert_eq!(
            observed(cert, &result, 19, at),
            wanted.contains(&at),
            "{} format {n:x}: bit {at}",
            d.codec_id
        );
    }
    parse_case(cert, d, &text, None);
}
#[test]
fn csharp_03_t06_w09_hex_codecs_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_HEX_CODECS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/hex-codecs");
    let mut rows = vec![];
    let mut unique = BTreeMap::new();
    let mut sources_count = 0;
    for (id, row, facts) in sources() {
        sources_count += 1;
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
        let p = generate_csharp_practical_ordinary_hex_codecs(vir).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_hex_codecs(
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
            assert!(import_csharp_practical_ordinary_hex_codecs(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_hex_codecs(&p.canonical_bytes(), &corrupt, vir)
                .is_err()
        );
        if p.definitions().is_empty() {
            continue;
        }
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        for d in p.definitions() {
            unique.entry(d.codec_id.clone()).or_insert((
                d.clone(),
                cert.clone(),
                emitted.closure().roots().clone(),
                emitted.closure().closed().clone(),
            ));
        }
        eprintln!(
            "hex codec source {id}: {} definitions",
            p.definitions().len()
        );
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
    }
    assert_eq!(sources_count, 64);
    assert_eq!(unique.len(), 4);
    let manifest = json!({"contexts_examined":sources_count,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/hex-codecs/certificates.json"),
            manifest
        );
    }
    let mut parse_count = 0;
    let mut format_count = 0;
    for (id, (d, cert, roots, closed)) in unique {
        let width = 1usize << d.value_depth;
        let max = if width == 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };
        let mut values = BTreeSet::from([
            0,
            1,
            max,
            1u128 << (width - 1),
            0x0123456789abcdef0123456789abcdefu128 & max,
            0x0123456789abcdeffedcba9876543210u128 & max,
            0x7fc00001u128 & max,
            0x7ff0000000000001u128 & max,
        ]);
        match id.as_str() {
            "binary32" => values.extend([0x7f800000, 0xff800000, 0x7f7fffff, 0x00800000]),
            "binary64" => values.extend([
                0x7ff0000000000000,
                0xfff0000000000000,
                0x7fefffffffffffff,
                0x0010000000000000,
            ]),
            _ => {}
        }
        for n in values {
            format_case(&cert, &d, n, &bundle, &roots, &closed);
            format_count += 1;
        }
        let base = match id.as_str() {
            "binary32" => "0123abcd",
            "binary64" => "0123456789abcdef",
            "guid.n" => "0123456789abcdef0123456789abcdef",
            _ => "01234567-89ab-cdef-0123-456789abcdef",
        };
        let good: Vec<u16> = base.encode_utf16().collect();
        let mut inputs = vec![
            good.clone(),
            vec![],
            base.to_uppercase().encode_utf16().collect(),
            format!(" {base}").encode_utf16().collect(),
            format!("{base} ").encode_utf16().collect(),
            good[..good.len() - 1].to_vec(),
            vec![b'0' as u16; 16384],
            vec![b'0' as u16; 16385],
        ];
        for i in 0..good.len() {
            for ch in [b'g' as u16, b'-' as u16, 0x130, 0xd800, 0xffff] {
                let mut input = good.clone();
                input[i] = ch;
                inputs.push(input);
            }
            if good[i] != b'-' as u16 {
                let mut input = good.clone();
                input[i] = b'A' as u16;
                inputs.push(input);
                let mut input = good.clone();
                input[0] = b'A' as u16;
                input[i] = 0x130;
                inputs.push(input);
            }
        }
        for input in inputs {
            parse_case(&cert, &d, &input, None);
            parse_count += 1;
        }
        for length in [16385, u32::MAX, 1 << 31] {
            parse_case(&cert, &d, &good, Some(length));
            parse_count += 1;
        }
    }
    eprintln!(
        "hex codecs: {format_count} format/parse round trips; {parse_count} parse/error cases"
    );
}
