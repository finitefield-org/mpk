//! Actual original-source array carriers and their recursive JSON composition.
use super::*;

const SOURCE: &str = "namespace Boundary;public readonly struct Inner{public readonly int[] Values;}public readonly struct Payload{public readonly bool[] Flags;public readonly int[] Numbers;public readonly Inner[] Nested;}public static class Entry{public static Payload Run(Payload p){return p;}}\n";
fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-sequence-sources")
}
fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_SEQUENCES_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-sequences")
        })
}
#[test]
fn csharp_03_t06_w09_json_sequence_requests() {
    let requests = source_tests::requests_for(&[("source-arrays", SOURCE)]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_SEQUENCE_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}
#[test]
fn csharp_03_t06_w09_json_sequence_source() {
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    let bundle = b();
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
    assert_eq!(p.sequences().len(), 3);
    assert_eq!(p.products().len(), 2);
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    for seq in p.sequences() {
        let layout = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == seq.carrier.type_id)
            .unwrap();
        assert_eq!(&seq.carrier, layout);
        let entry = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == seq.carrier.type_id)
            .unwrap();
        assert_eq!(
            entry["template_id"],
            "mpk.csharp.semantic.bounded_sequence.v1"
        );
        assert_eq!(seq.capacity, 4096);
        assert_eq!(
            layout.shape,
            OrdinaryShape::Sequence {
                capacity: 4096,
                element: Box::new(OrdinaryShape::Reference {
                    type_id: seq.element_type_id.clone()
                })
            }
        );
        assert!(!p.deferred_type_ids().contains(&seq.carrier.type_id));
    }
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_json_products(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
    let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
    assert_eq!(p.primitives(), values.definitions());
    for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
        let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let names = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        super::super::super::super::structural_equivalence_tests::same_definition_closure(
            &old, &cert, &names,
        )
        .unwrap();
    }
    if let Some(previous) = std::env::var_os("MPK_W09_JSON_SEQUENCES_PREVIOUS") {
        let previous = std::path::PathBuf::from(previous);
        let previous_row: Value =
            serde_json::from_slice(&fs::read(previous.join("certificate.json")).unwrap()).unwrap();
        let hex = fs::read_to_string(previous.join("source-arrays.hex")).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let old = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let mut names = BTreeSet::new();
        let mut count = 0;
        for d in p.sequences().iter().filter(|d| d.packet_depth < 32) {
            let before = previous_row["program"]["sequences"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["carrier"]["type_id"] == d.carrier.type_id)
                .unwrap();
            assert_eq!(&serde_json::to_value(d).unwrap(), before);
            names.extend([
                d.parse_definition.clone(),
                d.header_definition.clone(),
                d.value_definition.clone(),
                d.child_parse_definition.clone(),
                d.pipeline_definition.clone(),
            ]);
            count += 1;
        }
        assert_eq!(count, 2);
        super::super::super::super::structural_equivalence_tests::same_definition_closure(
            &old, &cert, &names,
        )
        .unwrap();
        eprintln!("Both scalar-array parser dependency closures and metadata are unchanged by the wide-argument fix");
    }
    let defined = p
        .primitives()
        .iter()
        .map(|d| d.carrier.type_id.as_str())
        .chain(p.products().iter().map(|d| d.carrier.type_id.as_str()))
        .chain(p.sequences().iter().map(|d| d.carrier.type_id.as_str()))
        .chain(p.enums().iter().map(|d| d.carrier.type_id.as_str()))
        .chain(p.vocabulary().iter().map(|d| d.carrier.type_id.as_str()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        p.deferred_type_ids()
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        layouts
            .carriers()
            .iter()
            .map(|c| c.type_id.as_str())
            .filter(|id| !defined.contains(id))
            .collect()
    );
    for (key, value) in [
        ("element_type_id", json!("forged")),
        ("capacity", json!(4095)),
        ("pipeline_definition", json!("forged")),
        ("child_parse_definition", json!("forged")),
    ] {
        let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        bad["sequences"][0][key] = value;
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&bad).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let row = json!({"id":"source-arrays","program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
    let bytes = serde_json::to_vec_pretty(&row).unwrap();
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    let out = pins();
    if std::env::var_os("MPK_W09_JSON_SEQUENCES_OUT").is_some() {
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("certificate.json"), bytes).unwrap();
        fs::write(out.join("source-arrays.hex"), hex).unwrap();
    } else {
        assert_eq!(fs::read(out.join("certificate.json")).unwrap(), bytes);
        assert_eq!(
            fs::read_to_string(out.join("source-arrays.hex")).unwrap(),
            hex
        );
    }
    eprintln!(
        "JSON arrays: three sequences and two source products; {} terms, {} declarations",
        cert.term_table.len(),
        cert.declarations.len()
    );
}

fn seq_ones(element_depth: usize, elements: &[BTreeSet<usize>]) -> BTreeSet<usize> {
    let padded = (12 + element_depth).max(5);
    let mut ones = (0..32)
        .filter(|i| elements.len() & (1 << i) != 0)
        .map(|i| i << (1 + padded - 5))
        .collect::<BTreeSet<_>>();
    for (index, element) in elements.iter().enumerate() {
        ones.extend(element.iter().map(|bit| 1 | (index << 1) | (bit << 13)));
    }
    ones
}
#[test]
fn csharp_03_t06_w09_json_sequence_actual_core() {
    use core_eval::{apply, bit};
    let root = pins();
    let row: Value =
        serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("source-arrays.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let seqs = row["program"]["sequences"].as_array().unwrap();
    let bools = seqs
        .iter()
        .find(|d| d["element_type_id"] == "mpk.csharp.value.bool.v1")
        .unwrap();
    let ints = seqs
        .iter()
        .find(|d| d["element_type_id"] == "mpk.csharp.value.i32.v1")
        .unwrap();
    let nested = seqs
        .iter()
        .find(|d| {
            d["element_type_id"]
                .as_str()
                .unwrap()
                .starts_with("mpk.csharp.source.")
        })
        .unwrap();
    let negative = (1..32).collect::<BTreeSet<_>>(); // -2
    let three = BTreeSet::from([0, 1]);
    let int_values = seq_ones(5, &[negative.clone(), three.clone()]);
    let nested_values = seq_ones(18, &[int_values.clone(), seq_ones(5, &[])]);
    let mut cases = 0;
    let mut checked = 0;
    let minimum_case = std::env::var("MPK_W09_JSON_SEQUENCE_MIN_CASE")
        .ok()
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(1);
    let mut check = |d: &Value,
                     text: &str,
                     start: u32,
                     ending: u8,
                     depth: u32,
                     cells: Option<u32>,
                     ones: &BTreeSet<usize>,
                     full: bool| {
        cases += 1;
        if cases < minimum_case {
            return;
        }
        checked += 1;
        let pd = d["packet_depth"].as_u64().unwrap() as usize;
        let vd = d["carrier"]["depth"].as_u64().unwrap() as usize;
        let result = run(
            &cert,
            d["parse_definition"].as_str().unwrap(),
            vec![
                document(
                    text.len() as u32,
                    &text.bytes().enumerate().collect::<Vec<_>>(),
                ),
                word(start),
                word_tag(ending),
                word(depth),
            ],
        );
        let end = text.len() - usize::from(ending != 0);
        let mut expected = BTreeSet::new();
        if let Some(cells) = cells {
            expected.insert(0);
            if ending == 0 {
                expected.insert(2);
            }
            for i in 0..32 {
                if end & (1 << i) != 0 {
                    expected.insert((2 + i) << 1);
                }
                if cells & (1 << i) != 0 {
                    expected.insert((34 + i) << 1);
                }
            }
            expected.extend(ones.iter().map(|i| 1 | (i << 1)));
        }
        let mut probes = if full {
            (0..1 << pd).collect::<BTreeSet<_>>()
        } else {
            (0..128).map(|i| i << 1).collect::<BTreeSet<_>>()
        };
        if !full {
            probes.extend((0..vd).map(|i| 1 << i));
            probes.extend((0..2048).map(|i| 1 | (i << 1)));
            probes.extend(expected.iter().copied());
            for i in ones {
                for bit in 0..vd {
                    probes.insert(1 | ((i ^ (1 << bit)) << 1));
                }
            }
            // All length word bits plus header high padding; last physical slots.
            for i in 0..32 {
                probes.insert(1 | ((i << (vd - 5)) << 1));
            }
            probes.extend((7..vd).map(|i| (1 << i) << 1));
            probes.extend((0..64).map(|i| 1 | (((1 << vd) - 1 - i) << 1)));
            if vd == 18 {
                for index in [0, 1, 2, 4095] {
                    for bit in 0..32 {
                        probes.insert(1 | ((1 | (index << 1) | (bit << 13)) << 1));
                    }
                }
            }
        }
        for i in &probes {
            let mut v = result.clone();
            for bit_index in 0..pd {
                v = apply(&cert, v, V::Bit(i & (1 << bit_index) != 0));
            }
            assert_eq!(
                bit(v),
                expected.contains(i),
                "sequence case {} packet bit {i}",
                cases
            );
        }
        eprintln!(
            "JSON sequence core {cases}: {} packet bits, valid {}",
            probes.len(),
            cells.is_some()
        );
    };
    check(
        bools,
        "[true,false]",
        0,
        0,
        0,
        Some(3),
        &seq_ones(0, &[BTreeSet::from([0]), BTreeSet::new()]),
        true,
    );
    check(bools, "[]", 0, 0, 32, Some(1), &BTreeSet::new(), true);
    for text in [
        "[true,]",
        "[,true]",
        "[true false]",
        "[1]",
        "[false]x",
        "[false,",
        "[false,,true]",
    ] {
        check(bools, text, 0, 0, 0, None, &BTreeSet::new(), false);
    }
    check(bools, "[true]", 0, 0, 32, None, &BTreeSet::new(), false);
    check(ints, "[-2,3]", 0, 0, 31, Some(3), &int_values, false);
    check(
        ints,
        "[-2,2147483648]",
        0,
        0,
        0,
        None,
        &BTreeSet::new(),
        false,
    );
    check(ints, "xx[-2,3],", 2, 1, 0, Some(3), &int_values, false);
    check(
        nested,
        r#"[{"Values":[-2,3]},{"Values":[]}]"#,
        0,
        0,
        29,
        Some(7),
        &nested_values,
        false,
    );
    check(
        nested,
        r#"[{"Values":[-2,3]},{"Values":[]}]"#,
        0,
        0,
        30,
        None,
        &BTreeSet::new(),
        false,
    );
    check(
        nested,
        r#"[{"Values":[]}]"#,
        0,
        0,
        30,
        Some(3),
        &seq_ones(18, &[BTreeSet::new()]),
        false,
    );
    let payload = row["program"]["products"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["member_names"] == json!(["Flags", "Numbers", "Nested"]))
        .unwrap();
    let mut payload_values = seq_ones(0, &[BTreeSet::from([0]), BTreeSet::new()])
        .into_iter()
        .map(|i| i << 20)
        .collect::<BTreeSet<_>>();
    payload_values.extend(int_values.iter().map(|i| 1 | (i << 15)));
    payload_values.extend(nested_values.iter().map(|i| 2 | (i << 2)));
    check(
        payload,
        r#"{"Flags":[true,false],"Numbers":[-2,3],"Nested":[{"Values":[-2,3]},{"Values":[]}]}"#,
        0,
        0,
        0,
        Some(14),
        &payload_values,
        false,
    );
    check(
        payload,
        r#"{"Flags":[true,false],"Numbers":[-2,3],"Nested":[{"Values":[-2,3]},]}"#,
        0,
        0,
        0,
        None,
        &BTreeSet::new(),
        false,
    );
    assert!(checked > 0, "selected cases must exist");
    eprintln!(
        "JSON sequence runtime: {checked} of {cases} cases; complete C14 packets when selected and explicit wide probes"
    );
}

#[test]
fn csharp_03_t06_w09_json_sequence_capacity_core() {
    use core_eval::{apply, bit};
    let root = pins();
    let row: Value =
        serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("source-arrays.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let d = row["program"]["sequences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["element_type_id"] == "mpk.csharp.value.bool.v1")
        .unwrap();
    assert_eq!(d["capacity"], 4096);
    assert_eq!(d["packet_depth"], 14);
    for count in [4095_usize, 4096, 4097] {
        let text = format!("[{}true]", "false,".repeat(count - 1));
        eprintln!("JSON sequence capacity {count}: evaluating original complete document");
        let result = run(
            &cert,
            d["parse_definition"].as_str().unwrap(),
            vec![
                document(
                    text.len() as u32,
                    &text.bytes().enumerate().collect::<Vec<_>>(),
                ),
                word(0),
                word_tag(0),
                word(0),
            ],
        );
        let valid = count <= 4096;
        let mut expected = BTreeSet::new();
        if valid {
            expected.extend([0, 2]);
            for i in 0..32 {
                if text.len() & (1 << i) != 0 {
                    expected.insert((2 + i) << 1);
                }
                if (count + 1) & (1 << i) != 0 {
                    expected.insert((34 + i) << 1);
                }
                if count & (1 << i) != 0 {
                    expected.insert(1 | ((i << 8) << 1));
                }
            }
            expected.insert(1 | ((1 | ((count - 1) << 1)) << 1));
        }
        let mut probes = (0..128).map(|i| i << 1).collect::<BTreeSet<_>>();
        probes.extend((0..32).map(|i| 1 | ((i << 8) << 1)));
        probes.extend((0..14).map(|i| 1 << i));
        for index in [0, 1, 4093, 4094, 4095] {
            probes.insert(1 | ((1 | (index << 1)) << 1));
        }
        probes.extend(expected.iter().copied());
        for index in probes {
            let mut v = result.clone();
            for i in 0..14 {
                v = apply(&cert, v, V::Bit(index & (1 << i) != 0));
            }
            assert_eq!(
                bit(v),
                expected.contains(&index),
                "capacity {count}, packet bit {index}"
            );
        }
        eprintln!(
            "JSON sequence capacity {count}: full header,length and selected edge slots passed"
        );
    }
}
