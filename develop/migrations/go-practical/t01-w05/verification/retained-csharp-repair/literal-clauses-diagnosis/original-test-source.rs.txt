//! Canonical contract literals, including raw UTF-16 and shared structural parts.
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};

pub(super) fn cases() -> Vec<(String, J, MonomorphicValue)> {
    let id = |token: &str| format!("mpk.csharp.value.{token}.v1");
    let source_id = csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Data","owner":"","name":"Value","parameter_type_ids":[],"result_type_id":""})).unwrap();
    let product = |n: &str| MonomorphicValue::Product {
        type_id: source_id.clone(),
        fields: vec![NamedMonomorphicValue {
            name: "Amount".into(),
            value: Box::new(MonomorphicValue::Signed {
                type_id: id("i32"),
                value: n.into(),
            }),
        }],
    };
    let source_ty = json!({"kind":"source","id":source_id});
    let sequence_id = csharp_practical_closed_instance_id(
        &b(),
        &instance("bounded_sequence", vec![source_ty.clone()]),
    )
    .unwrap();
    let option_id =
        csharp_practical_closed_instance_id(&b(), &instance("option", vec![source_ty])).unwrap();
    let raw_product = |n: &str| J::Object(vec![("Amount".into(), J::string(n))]);
    let utf16 = vec![97, 0, 0xd800, 0x62, 0xffff];
    vec![
        (
            "unit".into(),
            J::Null,
            MonomorphicValue::Unit {
                type_id: id("unit"),
            },
        ),
        (
            "char".into(),
            J::Utf16String(vec![0xd800]),
            MonomorphicValue::Char {
                type_id: id("char"),
                utf16: 0xd800,
            },
        ),
        (
            "string".into(),
            J::Utf16String(utf16.clone()),
            MonomorphicValue::String {
                type_id: id("string"),
                utf16,
            },
        ),
        (
            "f32".into(),
            J::string("3fc00000"),
            MonomorphicValue::F32Bits {
                type_id: id("f32"),
                bits: "3fc00000".into(),
            },
        ),
        (
            "f64".into(),
            J::string("c004000000000000"),
            MonomorphicValue::F64Bits {
                type_id: id("f64"),
                bits: "c004000000000000".into(),
            },
        ),
        (
            "decimal".into(),
            J::string("1.23"),
            MonomorphicValue::DecimalBits {
                type_id: id("decimal"),
                negative: false,
                scale: 2,
                coefficient: "123".into(),
            },
        ),
        (
            "guid".into(),
            J::string("00112233445566778899aabbccddeeff"),
            MonomorphicValue::Guid {
                type_id: id("guid"),
                n: "00112233445566778899aabbccddeeff".into(),
            },
        ),
        (
            "date".into(),
            J::string("0001-01-02"),
            MonomorphicValue::Date {
                type_id: id("date"),
                day_number: 1,
            },
        ),
        (
            "time".into(),
            J::string("00:00:00.0000001"),
            MonomorphicValue::Time {
                type_id: id("time"),
                ticks: "1".into(),
            },
        ),
        (
            "duration".into(),
            J::string("-1"),
            MonomorphicValue::Duration {
                type_id: id("duration"),
                ticks: "-1".into(),
            },
        ),
        (
            "instant".into(),
            J::string("-1"),
            MonomorphicValue::Instant {
                type_id: id("instant"),
                milliseconds: "-1".into(),
            },
        ),
        (
            "day_of_week".into(),
            J::string("2"),
            MonomorphicValue::Enum {
                type_id: id("day_of_week"),
                underlying: "i32".into(),
                carrier: "2".into(),
            },
        ),
        (
            "parse_error".into(),
            J::string("range"),
            MonomorphicValue::ParseError {
                type_id: id("parse_error"),
                arm: ParseErrorArm::Range,
            },
        ),
        ("product-one".into(), raw_product("1"), product("1")),
        ("product-negative".into(), raw_product("-2"), product("-2")),
        (
            "sequence".into(),
            J::Array(vec![raw_product("1"), raw_product("-2")]),
            MonomorphicValue::Sequence {
                type_id: sequence_id,
                elements: vec![product("1"), product("-2")],
            },
        ),
        (
            "none".into(),
            J::Object(vec![("tag".into(), J::string("none"))]),
            MonomorphicValue::Option {
                type_id: option_id.clone(),
                arm: OptionArm::None,
                value: None,
            },
        ),
        (
            "some".into(),
            J::Object(vec![
                ("tag".into(), J::string("some")),
                ("payload".into(), raw_product("1")),
            ]),
            MonomorphicValue::Option {
                type_id: option_id,
                arm: OptionArm::Some,
                value: Some(Box::new(product("1"))),
            },
        ),
    ]
}
fn literal_requests() -> Value {
    let original = requests();
    let original = &original[0];
    let code = original["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let contract = original["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| !i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let template = a::parse_canonical_practical_json(
        a::PracticalArtifactKind::TypeContract,
        contract.as_bytes(),
    )
    .unwrap();
    let root = original["roots"][0].as_str().unwrap();
    let (context, captures) =
        support::context_with_sidecar(&b(), root, code.as_bytes(), |context| {
            let J::Object(mut fields) = template.clone() else {
                panic!()
            };
            fields.retain(|(key, _)| key != "contract_sha256");
            for (key, value) in &mut fields {
                match key.as_str() {
                    "semantic_context" => *value = context.semantic_context().clone(),
                    "invariants" => {
                        *value = J::Array(
                            cases()
                                .iter()
                                .map(|(_, raw, expected)| {
                                    let literal = J::Object(vec![
                                        ("tag".into(), J::string("literal")),
                                        ("type_id".into(), J::string(expected.type_id())),
                                        ("value".into(), raw.clone()),
                                    ]);
                                    J::Object(vec![
                                        ("tag".into(), J::string("structural_equal")),
                                        ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
                                        ("left".into(), literal.clone()),
                                        ("right".into(), literal),
                                    ])
                                })
                                .collect(),
                        )
                    }
                    _ => {}
                }
            }
            let hash = mpk_vc::hash_domain_separated_raw(
                a::TYPE_CONTRACT_HASH_DOMAIN,
                &a::canonical_practical_json_bytes(&J::Object(fields.clone())).unwrap(),
            )
            .unwrap()
            .to_hex();
            fields.push(("contract_sha256".into(), J::string(hash)));
            a::canonical_practical_json_bytes(&J::Object(fields)).unwrap()
        });
    json!([{"id":"rich-literal-clauses","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}
#[test]
fn csharp_03_t06_w09_literal_clause_requests() {
    let requests = literal_requests();
    if let Some(path) = std::env::var_os("MPK_W09_LITERAL_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/literal-clauses/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_literal_clauses_original_source() {
    let bundle = b();
    let request = literal_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_LITERAL_CLAUSE_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/literal-clauses/responses.json")
    };
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_source_clauses(vir).unwrap();
    assert_eq!(p.definitions().len(), cases().len());
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let types: BTreeMap<_, _> = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect();
    let mut literals = BTreeMap::new();
    for expr in vir.contract_expressions() {
        for d in expr.definitions().iter().filter(|d| {
            d.tag == "literal"
                && d.result_type != "mpk.csharp.value.i32.v1"
                && d.result_type != "mpk.csharp.value.bool.v1"
        }) {
            let params = a::parse_canonical_practical_json(
                a::PracticalArtifactKind::MethodContract,
                d.parameters.as_bytes(),
            )
            .unwrap();
            literals.insert(
                (
                    d.result_type.clone(),
                    a::canonical_practical_json_bytes(params.get("value").unwrap()).unwrap(),
                ),
                format!(
                    "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
                    d.name
                        .as_bytes()
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
                ),
            );
        }
    }
    assert_eq!(literals.len(), cases().len());
    let mut observations = 0;
    for (label, raw, expected) in cases() {
        let name = &literals[&(
            expected.type_id().into(),
            a::canonical_practical_json_bytes(&raw).unwrap(),
        )];
        let (depth, ones) = projection_tests::sparse_storage(&expected, &types);
        let size = 1usize << depth;
        let probes = if depth <= 10 {
            (0..size).collect::<BTreeSet<_>>()
        } else {
            ones.iter()
                .copied()
                .chain(ones.iter().flat_map(|i| [i.saturating_sub(1), i + 1]))
                .chain([0, 1, size / 2, size - 1])
                .filter(|i| *i < size)
                .collect()
        };
        let value = run(&c, name, vec![]);
        for index in probes {
            let mut v = value.clone();
            for k in 0..depth {
                v = apply(&c, v, V::Bit(index & (1 << k) != 0));
            }
            assert_eq!(bit(v), ones.contains(&index), "{label}: {index}");
            observations += 1;
        }
    }
    assert_eq!(
        import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let integrated = generate_csharp_practical_ordinary_structural_public(vir).unwrap();
    let integrated_cert =
        mpk_cert::decode_canonical_certificate(integrated.certificate_bytes()).unwrap();
    let names = c
        .declarations
        .iter()
        .map(|d| c.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(&c, &integrated_cert, &names).unwrap();
    validate_csharp_practical_certificate_structure(&integrated_cert).unwrap();
    assert_eq!(integrated.source_clauses().unwrap(), p.definitions());
    assert_eq!(
        import_csharp_practical_ordinary_structural_public(
            &integrated.canonical_bytes(),
            integrated.certificate_bytes(),
            vir
        )
        .unwrap(),
        integrated
    );
    let public = generate_csharp_practical_ordinary_public_defaults(vir).unwrap();
    let public_cert = mpk_cert::decode_canonical_certificate(public.certificate_bytes()).unwrap();
    let names = public_cert
        .declarations
        .iter()
        .map(|d| public_cert.name_table[d.name as usize].clone())
        .collect();
    structural_equivalence_tests::same_definition_closure(&public_cert, &integrated_cert, &names)
        .unwrap();
    assert_eq!(integrated.public_domains().unwrap(), public.definitions());
    assert_eq!(integrated.public_defaults(), public.public_defaults());
    eprintln!(
        "literal source costs: {} terms, {} declarations; integrated: {} terms, {} declarations",
        c.term_table.len(),
        c.declarations.len(),
        integrated_cert.term_table.len(),
        integrated_cert.declarations.len()
    );
    let out = std::env::var_os("MPK_W09_LITERAL_CLAUSES_OUT").map(std::path::PathBuf::from);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/literal-clauses");
    let outputs = [
        (
            "literal-source.hex",
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into_bytes(),
        ),
        (
            "literal-integrated.hex",
            integrated
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into_bytes(),
        ),
        ("program.json", p.canonical_bytes()),
        ("integrated-program.json", integrated.canonical_bytes()),
        (
            "requests.json",
            serde_json::to_vec_pretty(&request).unwrap(),
        ),
    ];
    for (name, bytes) in outputs {
        if let Some(out) = &out {
            fs::create_dir_all(out).unwrap();
            fs::write(out.join(name), bytes).unwrap();
        } else {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
        }
    }
    eprintln!(
        "rich contract literal observations: {observations}; {} constants",
        literals.len()
    );
}

#[test]
fn csharp_03_t06_w09_literal_clause_input_types() {
    let bundle = b();
    let request = requests();
    let responses = read("ordinary-foundation/public-domain-sources/responses.json");
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let mut failures = vec![];
    for (label, raw, expected) in cases() {
        let literal = J::Object(vec![
            ("tag".into(), J::string("literal")),
            ("type_id".into(), J::string(expected.type_id())),
            ("value".into(), raw),
        ]);
        let bytes = a::canonical_practical_json_bytes(&literal).unwrap();
        let result = import_verification_contract_expression(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &DataContractEnvironment::default(),
            &bytes,
        );
        if let Err(e) = result {
            failures.push(format!("{label}: {e:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}
