//! Fixed-width hex/calendar contract adapters and actual recipe observations.
use super::*;
use crate::ordinary_carriers::literal_tests::{calendar_codec_tests, hex_codec_tests};

type CodecCases = (
    &'static str,
    &'static str,
    Vec<&'static str>,
    Vec<&'static str>,
);
fn cases() -> Vec<CodecCases> {
    vec![
        (
            "binary32",
            "f32",
            vec!["00000000", "80000000", "7fc01234", "ff800000"],
            vec!["7FC01234", "0", "gg000000", "１２３４５６７８"],
        ),
        (
            "binary64",
            "f64",
            vec![
                "0000000000000000",
                "8000000000000000",
                "7ff8123456789abc",
                "fff0000000000000",
            ],
            vec![
                "7FF8123456789ABC",
                "0",
                "gg00000000000000",
                "１２３４５６７８９０１２３４５６",
            ],
        ),
        (
            "guid.n",
            "guid",
            vec![
                "00000000000000000000000000000000",
                "0123456789abcdef0123456789abcdef",
                "ffffffffffffffffffffffffffffffff",
            ],
            vec![
                "0123456789ABCDEF0123456789ABCDEF",
                "00000000-0000-0000-0000-000000000000",
                "g0000000000000000000000000000000",
                "",
            ],
        ),
        (
            "guid.d",
            "guid",
            vec![
                "00000000-0000-0000-0000-000000000000",
                "01234567-89ab-cdef-0123-456789abcdef",
                "ffffffff-ffff-ffff-ffff-ffffffffffff",
            ],
            vec![
                "01234567-89AB-CDEF-0123-456789ABCDEF",
                "00000000000000000000000000000000",
                "g0000000-0000-0000-0000-000000000000",
                "",
            ],
        ),
        (
            "date",
            "date",
            vec!["0001-01-01", "2000-02-29", "9999-12-31"],
            vec!["0000-01-01", "1900-02-29", "2000/01/01", "２０００-02-29"],
        ),
        (
            "time",
            "time",
            vec!["00:00:00.0000000", "12:34:56.1234567", "23:59:59.9999999"],
            vec![
                "24:00:00.0000000",
                "00:00:60.0000000",
                "00:00:00.000000",
                "００:00:00.0000000",
            ],
        ),
    ]
}
fn literal(token: &str, text: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", J::string(text)),
    ])
}
fn result_id(token: &str) -> String {
    csharp_practical_closed_instance_id(
        &b(),
        &instance("result", vec![primitive(token), primitive("parse_error")]),
    )
    .unwrap()
}
fn parsed(id: &str, token: &str, text: J) -> J {
    J::object(vec![
        ("tag", J::string("codec_parse")),
        ("type_id", J::string(result_id(token))),
        ("codec_id", J::string(id)),
        (
            "codec_parameters",
            J::object(vec![("scale", J::Null), ("rounding", J::Null)]),
        ),
        ("text", text),
    ])
}
fn is_arm(value: J, arm: &str) -> J {
    J::object(vec![
        ("tag", J::string("tagged_is")),
        ("type_id", J::string(ty("bool"))),
        ("value", value),
        ("arm", J::string(arm)),
    ])
}
fn requests() -> Value {
    let mut rows: Vec<(String, Vec<J>)> = cases()
        .into_iter()
        .map(|(id, token, valid, invalid)| {
            let mut clauses = vec![];
            for text in valid {
                let value = if token == "guid" {
                    text.replace('-', "")
                } else {
                    text.into()
                };
                let format = J::object(vec![
                    ("tag", J::string("codec_format")),
                    ("type_id", J::string(ty("string"))),
                    ("codec_id", J::string(id)),
                    (
                        "codec_parameters",
                        J::object(vec![("scale", J::Null), ("rounding", J::Null)]),
                    ),
                    ("value", literal(token, &value)),
                    ("mode", J::string("canonical")),
                ]);
                // NaN retains its bits but is not structurally reflexive. Observe
                // the Result tag here; direct calls below check every payload bit.
                clauses.push(is_arm(parsed(id, token, format), "ok"));
            }
            for text in invalid {
                assert!(BoundaryCodec::new(id, &ty(token), None, None)
                    .unwrap()
                    .parse(&text.encode_utf16().collect::<Vec<_>>())
                    .is_err());
                clauses.push(is_arm(parsed(id, token, literal("string", text)), "error"));
            }
            (id.into(), clauses)
        })
        .collect();
    let mixed = rows
        .iter()
        .flat_map(|(_, expressions)| expressions.iter().cloned())
        .collect();
    rows.push(("mixed".into(), mixed));
    integer_codec_clause_tests::requests_for(rows)
}
#[test]
fn csharp_03_t06_w09_fixed_codec_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_FIXED_CODEC_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/fixed-codec-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_FIXED_CODEC_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/fixed-codec-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
pub(super) fn alias(name: &str) -> String {
    format!(
        "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn observe(c: &mpk_cert::encode::Certificate, value: &V, depth: u32, at: usize) -> bool {
    let mut value = value.clone();
    for i in 0..depth {
        value = apply(c, value, V::Bit(at & (1 << i) != 0));
    }
    bit(value)
}
#[test]
fn csharp_03_t06_w09_fixed_codec_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_FIXED_CODEC_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/fixed-codec-clauses/responses.json")
    };
    let mut source_clauses = 0;
    let mut observations = 0;
    for (id, token, valid, invalid) in cases() {
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none(), "{id}: {response}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_contract_expressions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        metadata(&p, &vc);
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| matches!(d.tag.as_str(), "codec_parse" | "codec_format"))
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(recipes.len(), 2);
        let parse = recipes.values().find(|d| d.tag == "codec_parse").unwrap();
        let format = recipes.values().find(|d| d.tag == "codec_format").unwrap();
        let (sc, parse_name, format_name) = if matches!(id, "date" | "time") {
            let full = generate_csharp_practical_ordinary_calendar_codecs(vir).unwrap();
            let mut d = full
                .definitions()
                .iter()
                .find(|d| d.codec_id == id)
                .unwrap()
                .clone();
            let original = (d.parse_definition.clone(), d.format_definition.clone());
            d.parse_definition = alias(&parse.name);
            for text in valid.iter().chain(&invalid) {
                calendar_codec_tests::parse_case(
                    &c,
                    &d,
                    &text.encode_utf16().collect::<Vec<_>>(),
                    None,
                );
            }
            for length in [16385, u32::MAX] {
                calendar_codec_tests::parse_case(&c, &d, &[], Some(length));
            }
            (
                mpk_cert::decode_canonical_certificate(full.certificate_bytes()).unwrap(),
                original.0,
                original.1,
            )
        } else {
            let full = generate_csharp_practical_ordinary_hex_codecs(vir).unwrap();
            let mut d = full
                .definitions()
                .iter()
                .find(|d| d.codec_id == id)
                .unwrap()
                .clone();
            let original = (d.parse_definition.clone(), d.format_definition.clone());
            d.parse_definition = alias(&parse.name);
            for text in valid.iter().chain(&invalid) {
                hex_codec_tests::parse_case(&c, &d, &text.encode_utf16().collect::<Vec<_>>(), None);
            }
            for length in [16385, u32::MAX] {
                hex_codec_tests::parse_case(&c, &d, &[], Some(length));
            }
            (
                mpk_cert::decode_canonical_certificate(full.certificate_bytes()).unwrap(),
                original.0,
                original.1,
            )
        };
        for (recipe, target) in [(parse, &parse_name), (format, &format_name)] {
            let decl = c
                .declarations
                .iter()
                .find(|d| c.name_table[d.name as usize] == alias(&recipe.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = decl.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                &c.name_table[c.declarations[global as usize].name as usize],
                target
            );
        }
        structural_equivalence_tests::same_definition_closure(
            &sc,
            &c,
            &BTreeSet::from([parse_name, format_name]),
        )
        .unwrap();
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let codec = BoundaryCodec::new(id, &ty(token), None, None).unwrap();
        for text in valid {
            let value = codec
                .parse(&text.encode_utf16().collect::<Vec<_>>())
                .unwrap();
            let expected = codec
                .format(
                    &bundle,
                    emitted.closure().roots(),
                    emitted.closure().closed(),
                    &value,
                )
                .unwrap();
            let (depth, ones) = projection_tests::sparse_storage(&value, &types);
            let formatted = run(&c, &alias(&format.name), vec![sparse_cube(depth, ones)]);
            let text_value = MonomorphicValue::String {
                type_id: ty("string"),
                utf16: expected.clone(),
            };
            let (depth, wanted) = projection_tests::sparse_storage(&text_value, &types);
            assert_eq!(depth, 19);
            let mut addresses = BTreeSet::new();
            for pad in 0..14 {
                addresses
                    .extend((0..32).map(|i| (if pad == 0 { 0 } else { 1 << pad }) | (i << 14)));
            }
            for index in
                (0..=expected.len()).chain([63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383])
            {
                addresses.extend((0..16).map(|k| 1 | (index << 1) | (k << 15)));
            }
            addresses.extend(&wanted);
            for at in addresses {
                assert_eq!(
                    observe(&c, &formatted, depth, at),
                    wanted.contains(&at),
                    "{id}: format {text} at {at}"
                );
                observations += 1;
            }
            let parsed = run(&c, &alias(&parse.name), vec![formatted]);
            let result = MonomorphicValue::TaggedSum {
                type_id: result_id(token),
                arm: "ok".into(),
                payload: vec![value],
            };
            let (depth, wanted) = projection_tests::sparse_storage(&result, &types);
            for at in 0..1usize << depth {
                assert_eq!(
                    observe(&c, &parsed, depth, at),
                    wanted.contains(&at),
                    "{id}: parse(format({text})) at {at}"
                );
                observations += 1;
            }
        }
        for d in p.definitions() {
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| sparse_cube(types[ty].depth, BTreeSet::new()))
                .collect::<Vec<_>>();
            assert!(bit(run(&c, &d.definedness_definition, args.clone())));
            assert!(bit(run(&c, &d.value_definition, args)));
            source_clauses += 1;
        }
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        changed["definitions"][0]["result_type"] = json!("changed");
        assert!(import_csharp_practical_ordinary_contract_expressions(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        eprintln!("fixed codec clause {id} passed");
    }
    assert_eq!(source_clauses, 44);
    eprintln!("fixed codec clauses: {source_clauses} source clauses; {observations} format/composition bit observations");
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
}

#[test]
fn csharp_03_t06_w09_fixed_codec_clauses_mixed_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_FIXED_CODEC_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/fixed-codec-clauses/responses.json")
    };
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "mixed")
        .unwrap();
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "mixed")
        .unwrap();
    assert!(response.get("reject").is_none(), "{response}");
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
    let vc = generate_csharp_practical_vc(PracticalVcSource {
        artifact_context: &context,
        captured_inputs: &captures,
        vir,
    })
    .unwrap();
    metadata(&p, &vc);
    assert_eq!(p.definitions().len(), 44);
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let hex = generate_csharp_practical_ordinary_hex_codecs(vir).unwrap();
    let calendar = generate_csharp_practical_ordinary_calendar_codecs(vir).unwrap();
    let mut roots = [BTreeSet::new(), BTreeSet::new()];
    let recipes = vir
        .contract_expressions()
        .iter()
        .flat_map(|e| e.definitions())
        .filter(|d| matches!(d.tag.as_str(), "codec_parse" | "codec_format"))
        .map(|d| (d.name.clone(), d.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(recipes.len(), 12);
    for d in recipes.values() {
        let parameters: Value = serde_json::from_str(&d.parameters).unwrap();
        let id = parameters["codec_id"].as_str().unwrap();
        let is_calendar = matches!(id, "date" | "time");
        let target = if is_calendar {
            let codec = calendar
                .definitions()
                .iter()
                .find(|c| c.codec_id == id)
                .unwrap();
            if d.tag == "codec_parse" {
                &codec.parse_definition
            } else {
                &codec.format_definition
            }
        } else {
            let codec = hex.definitions().iter().find(|c| c.codec_id == id).unwrap();
            if d.tag == "codec_parse" {
                &codec.parse_definition
            } else {
                &codec.format_definition
            }
        };
        roots[usize::from(is_calendar)].insert(target.clone());
        let decl = c
            .declarations
            .iter()
            .find(|x| c.name_table[x.name as usize] == alias(&d.name))
            .unwrap();
        let mpk_cert::encode::DeclarationKind::Def { value, .. } = decl.kind else {
            panic!()
        };
        let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize] else {
            panic!()
        };
        assert_eq!(
            &c.name_table[c.declarations[global as usize].name as usize],
            target
        );
    }
    for (bytes, roots) in [
        (hex.certificate_bytes(), &roots[0]),
        (calendar.certificate_bytes(), &roots[1]),
    ] {
        structural_equivalence_tests::same_definition_closure(
            &mpk_cert::decode_canonical_certificate(bytes).unwrap(),
            &c,
            roots,
        )
        .unwrap();
    }
    assert_eq!(
        import_csharp_practical_ordinary_contract_expressions(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    output("mixed.json", &p.canonical_bytes());
    output(
        "mixed.hex",
        p.certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            .as_bytes(),
    );
    eprintln!(
        "fixed codec mixed source: 44 attachments and all 12 nominal aliases/closures passed"
    );
}

#[test]
fn csharp_03_t06_w09_fixed_codec_clauses_pinned_bytes() {
    let bundle = b();
    let requests = requests();
    assert_eq!(
        read("ordinary-foundation/fixed-codec-clauses/requests.json"),
        requests
    );
    let responses = read("ordinary-foundation/fixed-codec-clauses/responses.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/fixed-codec-clauses");
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_contract_expressions(emitted.vir()).unwrap();
        assert_eq!(
            fs::read(fixture.join(format!("{id}.json"))).unwrap(),
            p.canonical_bytes()
        );
        assert_eq!(
            fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(requests.as_array().unwrap().len(), 7);
}
