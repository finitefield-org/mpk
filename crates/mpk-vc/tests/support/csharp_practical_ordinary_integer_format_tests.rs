use super::*;
use core_eval::{apply, bit, sparse_cube};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn format_sources() -> Vec<(String, Value, Value)> {
    let mut cases = sources();
    let replay = read("data-phase/data-stage-replay.json");
    let row = replay
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "14e30f193aa9e5aac03d640da54ad6ad22365f42b40d55c0d138ce8020de95f3")
        .unwrap();
    cases.push((
        "duration-add-source".into(),
        row.clone(),
        row["outcome"]["facts"].clone(),
    ));
    cases
}

#[test]
fn csharp_03_t06_w09_integer_formats_pinned_sources() {
    let bundle = b();
    let manifest = read("ordinary-foundation/integer-formats/certificates.json");
    let rows = manifest["sources"].as_array().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-formats");
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
        let p = generate_csharp_practical_ordinary_integer_formats(emitted.vir()).unwrap();
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

fn observe(cert: &mpk_cert::encode::Certificate, v: &V, at: usize) -> bool {
    let mut v = v.clone();
    for i in 0..19 {
        v = apply(cert, v, V::Bit(at & (1 << i) != 0));
    }
    bit(v)
}
pub(in crate::ordinary_carriers) fn format_case(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryIntegerFormatDefinition,
    n: u64,
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) {
    let width = 1u32 << d.value_depth;
    let signed = if width == 64 {
        n as i64
    } else {
        ((n << (64 - width)) as i64) >> (64 - width)
    };
    let value = if d.codec_id == "duration_ticks" {
        MonomorphicValue::Duration {
            type_id: d.value_type_id.clone(),
            ticks: signed.to_string(),
        }
    } else if d.codec_id == "unix_milliseconds" {
        MonomorphicValue::Instant {
            type_id: d.value_type_id.clone(),
            milliseconds: signed.to_string(),
        }
    } else if d.signed {
        MonomorphicValue::Signed {
            type_id: d.value_type_id.clone(),
            value: signed.to_string(),
        }
    } else {
        MonomorphicValue::Unsigned {
            type_id: d.value_type_id.clone(),
            value: n.to_string(),
        }
    };
    let oracle = BoundaryCodec::new(&d.codec_id, &d.value_type_id, None, None).unwrap();
    let expected = oracle.format(bundle, roots, closed, &value).unwrap();
    let input = (0..width as usize).filter(|i| n & (1 << i) != 0).collect();
    let result = run(
        cert,
        &d.format_definition,
        vec![sparse_cube(d.value_depth, input)],
    );
    let length = (0..32).fold(0u32, |n, i| {
        n | (u32::from(observe(cert, &result, i << 14)) << i)
    });
    assert_eq!(
        length,
        expected.len() as u32,
        "{} value {value:?}: length",
        d.codec_id
    );
    let mut actual = vec![];
    for i in 0..length as usize {
        let ch = (0..16).fold(0u16, |n, k| {
            n | (u16::from(observe(cert, &result, 1 | (i << 1) | (k << 15))) << k)
        });
        actual.push(ch);
    }
    assert_eq!(actual, expected, "{} value {value:?}", d.codec_id);
    assert_eq!(oracle.parse(&actual).unwrap(), value);
    for padding in 1..14 {
        for k in 0..32 {
            assert!(!observe(cert, &result, (1 << padding) | (k << 14)));
        }
    }
    let mut inactive = BTreeSet::from([
        length as usize,
        21,
        31,
        32,
        255,
        256,
        4095,
        4096,
        8191,
        8192,
        16383,
    ]);
    inactive.extend((0..14).map(|i| 1 << i).filter(|i| *i >= length as usize));
    for i in inactive {
        for k in 0..16 {
            assert!(
                !observe(cert, &result, 1 | (i << 1) | (k << 15)),
                "{} inactive {i} bit {k}",
                d.codec_id
            );
        }
    }
}
#[test]
fn csharp_03_t06_w09_integer_formats_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_INTEGER_FORMATS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &out {
        fs::create_dir_all(p).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-formats");
    let mut rows = vec![];
    let mut unique = BTreeMap::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut source_count = 0;
    for (id, row, facts) in format_sources() {
        source_count += 1;
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
        let p = generate_csharp_practical_ordinary_integer_formats(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_integer_formats(
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
            let mut changed = meta.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_integer_formats(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_integer_formats(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_integer_formats(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
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
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
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
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!(
            "integer format source {id}: {} codecs",
            p.definitions().len()
        );
    }
    assert_eq!(source_count, 65);
    assert_eq!(unique.len(), 10);
    let manifest = json!({"contexts_examined":source_count,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/integer-formats/certificates.json"),
            manifest
        );
    }
    let mut count = 0;
    for (id, (d, cert, roots, closed)) in unique {
        let width = 1u32 << d.value_depth;
        let mask = if width == 64 {
            u64::MAX
        } else {
            (1u64 << width) - 1
        };
        let sign = 1u64 << (width - 1);
        let mut values = BTreeSet::from([0, 1, 9, 10, 11, 99, 100, 101, mask, sign, sign - 1]);
        let mut power = 1u64;
        while let Some(next) = power.checked_mul(10).filter(|n| *n <= mask) {
            values.extend([next - 1, next, next.saturating_add(1) & mask]);
            power = next;
        }
        if d.signed {
            for n in [1u64, 9, 10, 99, 100] {
                values.insert(n.wrapping_neg() & mask);
            }
        }
        values.retain(|n| *n <= mask);
        for n in values {
            eprintln!("integer format {id}: input bits {n:x}");
            format_case(&cert, &d, n, &bundle, &roots, &closed);
            count += 1;
        }
    }
    eprintln!("integer formatting observations: {count}");
}
