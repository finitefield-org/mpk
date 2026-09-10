//! Original-source enum JSON: all eight underlying widths and product composition.
use super::*;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-enum-sources")
}
fn requests() -> Value {
    let mut source = String::from("namespace Boundary;");
    for (i, (ty, low, high)) in [
        ("sbyte", "-128", "127"),
        ("byte", "0", "255"),
        ("short", "-32768", "32767"),
        ("ushort", "0", "65535"),
        ("int", "-2147483648", "2147483647"),
        ("uint", "0", "4294967295U"),
        ("long", "-9223372036854775808L", "9223372036854775807L"),
        ("ulong", "0", "18446744073709551615UL"),
    ]
    .into_iter()
    .enumerate()
    {
        // Aliases are disallowed; unsigned enums have only Zero and High.
        let low = if i % 2 == 0 {
            format!("Low={low},")
        } else {
            String::new()
        };
        source.push_str(&format!("public enum E{i}:{ty}{{{low}Zero=0,High={high}}}"));
    }
    source.push_str("public readonly struct Payload{");
    for i in 0..8 {
        source.push_str(&format!("public readonly E{i} V{i};"));
    }
    let parameters = (0..8)
        .map(|i| format!("E{i} p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    source.push_str(&format!("public Payload({parameters}){{"));
    for i in 0..8 {
        source.push_str(&format!("V{i}=p{i};"));
    }
    source.push_str(
        "}}public static class Entry{public static Payload Run(Payload p){return new Payload(",
    );
    source.push_str(
        &(0..8)
            .map(|i| format!("p.V{i}"))
            .collect::<Vec<_>>()
            .join(","),
    );
    source.push_str(");}}\n");
    super::source_tests::requests_for(&[("all-underlying", source.as_str())])
}

#[test]
fn csharp_03_t06_w09_json_enum_source_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_ENUM_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}

fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_ENUMS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-enums"))
}

#[test]
fn csharp_03_t06_w09_json_enums_original_source() {
    let bundle = b();
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    let facts = &responses[0]["facts"];
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
    assert_eq!(p.enums().len(), 8);
    assert_eq!(p.products().len(), 1);
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let underlying = p
        .enums()
        .iter()
        .map(|d| d.underlying.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        underlying,
        BTreeSet::from(["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64"])
    );
    for d in p.enums() {
        let ty = facts["types"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == d.carrier.type_id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&d.declared_values).unwrap(),
            ty["enum_values"]
        );
        assert_eq!(
            &d.carrier,
            layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == d.carrier.type_id)
                .unwrap()
        );
        assert_eq!(d.packet_depth, 8);
        assert!(!p.deferred_type_ids().contains(&d.carrier.type_id));
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
    for (key, value) in [
        ("underlying", json!("u64")),
        ("declared_values", json!(["1"])),
        ("token_definition", json!("forged")),
        ("value_definition", json!("forged")),
    ] {
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        if key == "underlying" && forged["enums"][0][key] == value {
            forged["enums"][0][key] = json!("i8");
        } else {
            forged["enums"][0][key] = value;
        }
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    let row = json!({"id":"all-underlying","program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),
        "terms":cert.term_table.len(),"declarations":cert.declarations.len()});
    let bytes = serde_json::to_vec_pretty(&row).unwrap();
    let root = pins();
    if std::env::var_os("MPK_W09_JSON_ENUMS_OUT").is_some() {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("all-underlying.hex"), hex).unwrap();
        fs::write(root.join("certificate.json"), bytes).unwrap();
    } else {
        assert_eq!(
            fs::read_to_string(root.join("all-underlying.hex")).unwrap(),
            hex
        );
        assert_eq!(fs::read(root.join("certificate.json")).unwrap(), bytes);
    }
    eprintln!(
        "JSON enums: eight underlying types and source product, {} terms, {} declarations",
        cert.term_table.len(),
        cert.declarations.len()
    );
}

#[test]
fn csharp_03_t06_w09_json_enums_actual_core() {
    let root = pins();
    let row: Value =
        serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("all-underlying.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let mut cases = 0;
    let mut check = |d: &Value,
                     text: &str,
                     start: u32,
                     ending: u8,
                     depth: Option<u32>,
                     good: bool,
                     cells: u32,
                     ones: &BTreeSet<usize>| {
        let doc = document(
            text.len() as u32,
            &text.bytes().enumerate().collect::<Vec<_>>(),
        );
        let mut args = vec![doc, word(start), word_tag(ending)];
        if let Some(depth) = depth {
            args.push(word(depth));
        }
        let result = run(&cert, d["parse_definition"].as_str().unwrap(), args);
        let packet_depth = d["packet_depth"].as_u64().unwrap() as usize;
        let mut expected = vec![false; 1 << packet_depth];
        if good {
            let end = text.len() as u32 - u32::from(ending != 0);
            expected[0] = true;
            expected[2] = ending == 0;
            for i in 0..32 {
                expected[(2 + i) << 1] = end & (1 << i) != 0;
                expected[(34 + i) << 1] = cells & (1 << i) != 0;
            }
            for i in ones {
                expected[1 | (i << 1)] = true;
            }
        }
        for (i, want) in expected.into_iter().enumerate() {
            assert_eq!(
                leaf(&cert, result.clone(), packet_depth, i),
                want,
                "{} {text} bit{i}",
                d["underlying"]
            );
        }
        cases += 1;
        eprintln!(
            "JSON enum core {cases}: {} {text}, accepted {good}",
            d["underlying"]
        );
    };
    for d in row["program"]["enums"].as_array().unwrap() {
        let ty = d["underlying"].as_str().unwrap();
        let signed = ty.starts_with('i');
        let width = ty[1..].parse::<u32>().unwrap();
        let minimum = if signed { -(1_i128 << (width - 1)) } else { 0 };
        let maximum = (1_i128 << (width - u32::from(signed))) - 1;
        for value in [minimum, maximum, 0] {
            let ones = (0..width as usize)
                .filter(|i| (value as u128) & (1 << i) != 0)
                .collect();
            check(d, &format!("\"{value}\""), 0, 0, None, true, 1, &ones);
        }
        for text in [
            "\"1\"",
            "0",
            "\"-0\"",
            "\"+0\"",
            "\"00\"",
            "\"High\"",
            "\"\\u0030\"",
        ] {
            check(d, text, 0, 0, None, false, 0, &BTreeSet::new());
        }
        for value in [minimum - 1, maximum + 1] {
            check(
                d,
                &format!("\"{value}\""),
                0,
                0,
                None,
                false,
                0,
                &BTreeSet::new(),
            );
        }
        check(d, "x\"0\",", 1, 1, None, true, 1, &BTreeSet::new());
        check(d, "\"0\":", 0, 4, None, false, 0, &BTreeSet::new());
    }
    let product = &row["program"]["products"][0];
    assert_eq!(
        product["member_names"],
        json!(["V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7"])
    );
    assert_eq!(product["carrier"]["depth"], 9);
    let vals = [
        i8::MIN as i128,
        u8::MAX as i128,
        i16::MIN as i128,
        u16::MAX as i128,
        i32::MIN as i128,
        u32::MAX as i128,
        i64::MIN as i128,
        u64::MAX as i128,
    ];
    let text = format!(
        "{{{}}}",
        vals.iter()
            .enumerate()
            .map(|(i, v)| format!("\"V{i}\":\"{v}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut ones = BTreeSet::new();
    for (field, (&value, width)) in vals
        .iter()
        .zip([8_u32, 8, 16, 16, 32, 32, 64, 64])
        .enumerate()
    {
        for bit in 0..width {
            if (value as u128) & (1 << bit) != 0 {
                ones.insert(field | ((bit as usize) << (3 + 6 - width.ilog2())));
            }
        }
    }
    check(product, &text, 0, 0, Some(31), true, 9, &ones);
    check(product, &text, 0, 0, Some(32), false, 0, &BTreeSet::new());
    check(
        product,
        &text.replacen("\"-128\"", "\"1\"", 1),
        0,
        0,
        Some(0),
        false,
        0,
        &BTreeSet::new(),
    );
    assert_eq!(cases, 115);
}
