//! Captured day-of-week source plus reachable parse-error type-contract literal.
use super::*;
use sha2::{Digest, Sha256};

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-builtin-sources")
}
fn requests() -> Value {
    use PracticalJsonValue as J;
    let code="namespace Boundary;public readonly struct Payload{public readonly System.DayOfWeek Day;public Payload(System.DayOfWeek day){Day=day;}}public static class Entry{public static Payload Run(Payload p){return new Payload(p.Day);}}\n";
    let mut requests = super::source_tests::requests_for(&[("day-error", code)]);
    let request = &mut requests[0];
    let boundary = request["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "contracts/boundary.json")
        .unwrap();
    let boundary = parse_canonical_practical_json(
        PracticalArtifactKind::BoundaryContract,
        boundary["utf8"].as_str().unwrap().as_bytes(),
    )
    .unwrap();
    let template = read("ordinary-foundation/finite-sources/requests.json");
    let template = template[0]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "contracts/data.json")
        .unwrap();
    let mut contract = parse_canonical_practical_json(
        PracticalArtifactKind::TypeContract,
        template["utf8"].as_str().unwrap().as_bytes(),
    )
    .unwrap();
    let source_id=csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":"Payload","parameter_type_ids":[],"result_type_id":""})).unwrap();
    let member = csharp_practical_stored_member_id(
        &source_id,
        "Day",
        &json!({"kind":"primitive","id":"day_of_week"}),
        "readonly_field",
    )
    .unwrap();
    let J::Object(fields) = &mut contract else {
        panic!()
    };
    fields.retain(|(k, _)| k != "contract_sha256");
    for (k, v) in fields.iter_mut() {
        match k.as_str() {
            "semantic_context" => *v = boundary.get("semantic_context").unwrap().clone(),
            "compilation_id" => *v = J::string(request["compilation_id"].as_str().unwrap()),
            "source_type_id" => *v = J::string(&source_id),
            "source_content_sha256" => {
                *v = J::string(format!("{:x}", Sha256::digest(code.as_bytes())))
            }
            "ordered_member_ids" => *v = J::Array(vec![J::string(&member)]),
            "recursive_default" => *v = J::Object(vec![("Day".into(), J::string("0"))]),
            _ => {}
        }
    }
    let bytes = canonical_practical_json_bytes(&contract).unwrap();
    let mut hash = Sha256::new();
    hash.update("MPK-CSHARP-TYPE-CONTRACT-1.0");
    hash.update([0]);
    hash.update(bytes);
    let J::Object(fields) = &mut contract else {
        unreachable!()
    };
    fields.push((
        "contract_sha256".into(),
        J::string(format!("{:x}", hash.finalize())),
    ));
    let text = String::from_utf8(canonical_practical_json_bytes(&contract).unwrap()).unwrap();
    let inputs = request["inputs"].as_array_mut().unwrap();
    inputs.push(json!({"kind":"sidecar","path":"contracts/data.json","utf8":text}));
    inputs.sort_by_key(|i| i["path"].as_str().unwrap().to_owned());
    requests
}

#[test]
fn csharp_03_t06_w09_json_builtin_source_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_BUILTIN_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}

fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_BUILTINS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-builtins"))
}

#[test]
fn csharp_03_t06_w09_json_builtins_original_source() {
    let bundle = b();
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
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
    assert_eq!(p.products().len(), 1);
    assert_eq!(p.enums().len(), 1);
    assert_eq!(p.vocabulary().len(), 1);
    assert_eq!(
        p.enums()[0].carrier.type_id,
        "mpk.csharp.value.day_of_week.v1"
    );
    assert_eq!(p.enums()[0].underlying, "i32");
    assert_eq!(
        p.enums()[0].declared_values,
        (0..7).map(|i| i.to_string()).collect::<Vec<_>>()
    );
    let words = &p.vocabulary()[0];
    assert_eq!(words.carrier.type_id, "mpk.csharp.value.parse_error.v1");
    assert_eq!(
        words.names,
        [
            "input_bound",
            "syntax",
            "noncanonical",
            "scale_precision",
            "range"
        ]
    );
    assert_eq!(
        words
            .literals
            .iter()
            .map(|l| l.utf8.clone())
            .collect::<Vec<_>>(),
        words
            .names
            .iter()
            .map(|n| format!("\"{n}\"").into_bytes())
            .collect::<Vec<_>>()
    );
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    for carrier in [&p.enums()[0].carrier, &words.carrier] {
        assert_eq!(
            carrier,
            layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == carrier.type_id)
                .unwrap()
        );
        assert!(!p.deferred_type_ids().contains(&carrier.type_id));
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
    for key in ["names", "literals", "parse_definition"] {
        let mut forged: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        forged["vocabulary"][0][key] = json!([]);
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&forged).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let row = json!({"id":"day-error","program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
    let bytes = serde_json::to_vec_pretty(&row).unwrap();
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    let root = pins();
    if std::env::var_os("MPK_W09_JSON_BUILTINS_OUT").is_some() {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("certificate.json"), bytes).unwrap();
        fs::write(root.join("day-error.hex"), hex).unwrap();
    } else {
        assert_eq!(fs::read(root.join("certificate.json")).unwrap(), bytes);
        assert_eq!(fs::read_to_string(root.join("day-error.hex")).unwrap(), hex);
    }
    eprintln!(
        "JSON builtins: day, five errors and source product; {} terms, {} declarations",
        cert.term_table.len(),
        cert.declarations.len()
    );
}

#[test]
fn csharp_03_t06_w09_json_builtins_actual_core() {
    let root = pins();
    let row: Value =
        serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("day-error.hex")).unwrap();
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
                     expected: Option<(u32, u32)>| {
        let doc = document(
            text.len() as u32,
            &text.bytes().enumerate().collect::<Vec<_>>(),
        );
        let mut args = vec![doc, word(start), word_tag(ending)];
        if let Some(depth) = depth {
            args.push(word(depth));
        }
        let result = run(&cert, d["parse_definition"].as_str().unwrap(), args);
        let mut want = vec![false; 256];
        if let Some((value, cells)) = expected {
            let end = text.len() as u32 - u32::from(ending != 0);
            want[0] = true;
            want[2] = ending == 0;
            for i in 0..32 {
                want[(2 + i) << 1] = end & (1 << i) != 0;
                want[(34 + i) << 1] = cells & (1 << i) != 0;
                want[1 | (i << 1)] = value & (1 << i) != 0;
            }
        }
        for (i, wanted) in want.into_iter().enumerate() {
            assert_eq!(leaf(&cert, result.clone(), 8, i), wanted, "{text} bit{i}");
        }
        cases += 1;
        eprintln!(
            "JSON builtin core {cases}: {text}, accepted {}",
            expected.is_some()
        );
    };
    let day = &row["program"]["enums"][0];
    for value in 0..7 {
        check(day, &format!("\"{value}\""), 0, 0, None, Some((value, 1)));
    }
    for text in [
        "\"-1\"",
        "\"7\"",
        "\"4294967296\"",
        "\"Sunday\"",
        "0",
        "\"00\"",
        "\"-0\"",
    ] {
        check(day, text, 0, 0, None, None);
    }
    let error = &row["program"]["vocabulary"][0];
    for (ordinal, name) in [
        "input_bound",
        "syntax",
        "noncanonical",
        "scale_precision",
        "range",
    ]
    .into_iter()
    .enumerate()
    {
        check(
            error,
            &format!("\"{name}\""),
            0,
            0,
            None,
            Some((ordinal as u32, 1)),
        );
    }
    for text in [
        "\"Syntax\"",
        "\"0\"",
        "syntax",
        "\"syntax \"",
        " \"syntax\"",
        "\"syntax",
        "\"\\u0073yntax\"",
        "\"unknown\"",
    ] {
        check(error, text, 0, 0, None, None);
    }
    for (ending, suffix) in [(1, ','), (2, ']'), (3, '}')] {
        check(
            error,
            &format!("x\"range\"{suffix}"),
            1,
            ending,
            None,
            Some((4, 1)),
        );
    }
    for ending in [4, 8, 16, 32, 64, 128] {
        check(error, "\"range\":", 0, ending, None, None);
    }
    let product = &row["program"]["products"][0];
    assert_eq!(product["carrier"]["depth"], 5);
    check(product, "{\"Day\":\"6\"}", 0, 0, Some(31), Some((6, 2)));
    check(product, "{\"Day\":\"6\"}", 0, 0, Some(32), None);
    check(product, "{\"Day\":\"7\"}", 0, 0, Some(0), None);
    assert_eq!(cases, 39);
}
