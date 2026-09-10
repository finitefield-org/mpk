//! Integer codec recipes, including Duration and Instant nominal identities.
use super::*;
use crate::ordinary_carriers::literal_tests::{integer_format_tests, integer_parse_tests};
fn configurations() -> Vec<(String, String, u32, bool)> {
    [
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "duration", "instant",
    ]
    .into_iter()
    .map(|token| {
        let id = match token {
            "duration" => "duration_ticks".into(),
            "instant" => "unix_milliseconds".into(),
            _ => format!("integer.{token}"),
        };
        let width = if matches!(token, "duration" | "instant") {
            64
        } else {
            token[1..].parse().unwrap()
        };
        (id, token.into(), width, !token.starts_with('u'))
    })
    .collect()
}
fn literal(token: &str, text: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", J::string(text)),
    ])
}
fn clauses(id: &str, token: &str, width: u32, signed: bool) -> Vec<J> {
    let min = if signed { -(1i128 << (width - 1)) } else { 0 };
    let max = (1i128 << (width - u32::from(signed))) - 1;
    let result = csharp_practical_closed_instance_id(
        &b(),
        &instance("result", vec![primitive(token), primitive("parse_error")]),
    )
    .unwrap();
    let parameters = || J::object(vec![("scale", J::Null), ("rounding", J::Null)]);
    let mut clauses = vec![];
    for n in BTreeSet::from([min, 0, max]) {
        let formatted = J::object(vec![
            ("tag", J::string("codec_format")),
            ("type_id", J::string(ty("string"))),
            ("codec_id", J::string(id)),
            ("codec_parameters", parameters()),
            ("value", literal(token, &n.to_string())),
            ("mode", J::string("canonical")),
        ]);
        let parsed = J::object(vec![
            ("tag", J::string("codec_parse")),
            ("type_id", J::string(&result)),
            ("codec_id", J::string(id)),
            ("codec_parameters", parameters()),
            ("text", formatted),
        ]);
        let payload = J::object(vec![
            ("tag", J::string("tagged_payload")),
            ("type_id", J::string(ty(token))),
            ("value", parsed),
            ("arm", J::string("ok")),
        ]);
        clauses.push(J::object(vec![
            ("tag", J::string("structural_equal")),
            ("type_id", J::string(ty("bool"))),
            ("left", payload),
            ("right", literal(token, &n.to_string())),
        ]));
    }
    for text in [
        min.to_string(),
        max.to_string(),
        "01".into(),
        "x".into(),
        (max + 1).to_string(),
    ] {
        let ok = BoundaryCodec::new(id, &ty(token), None, None)
            .unwrap()
            .parse(&text.encode_utf16().collect::<Vec<_>>())
            .is_ok();
        let parsed = J::object(vec![
            ("tag", J::string("codec_parse")),
            ("type_id", J::string(&result)),
            ("codec_id", J::string(id)),
            ("codec_parameters", parameters()),
            ("text", literal("string", &text)),
        ]);
        clauses.push(J::object(vec![
            ("tag", J::string("tagged_is")),
            ("type_id", J::string(ty("bool"))),
            ("value", parsed),
            ("arm", J::string(if ok { "ok" } else { "error" })),
        ]));
    }
    clauses
}
fn requests() -> Value {
    requests_for(
        configurations()
            .into_iter()
            .map(|(id, token, width, signed)| {
                let expressions = clauses(&id, &token, width, signed);
                (id, expressions)
            })
            .collect(),
    )
}
pub(super) fn requests_for(cases: Vec<(String, Vec<J>)>) -> Value {
    requests_for_source(cases, None)
}
pub(super) fn requests_for_source(
    cases: Vec<(String, Vec<J>)>,
    source_override: Option<&str>,
) -> Value {
    let row = read("ordinary-foundation/quantifiers/requests.json")[0].clone();
    let inputs = row["inputs"].as_array().unwrap();
    let source = inputs
        .iter()
        .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let source = source_override.unwrap_or(source);
    let raw = inputs
        .iter()
        .find(|i| i["path"] == "contracts/data.json")
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let template =
        a::parse_canonical_practical_json(a::PracticalArtifactKind::MethodContract, raw.as_bytes())
            .unwrap();
    let root = row["roots"][0].as_str().unwrap();
    json!(cases.into_iter().map(|(id, expressions)| {
        let (context, captures) = support::context_with_sidecars(&b(), root, source.as_bytes(),
            vec!["contracts/data.json".into()], |context| {
                let J::Object(mut fields) = template.clone() else { panic!() };
                fields.retain(|(k, _)| k != "contract_sha256");
                for (k, v) in &mut fields {
                    match k.as_str() {
                        "semantic_context" => *v = context.semantic_context().clone(),
                        "compilation_id" => *v = J::string(context.compilation_id()),
                        "source_content_sha256" => *v = J::string(format!("{:x}", Sha256::digest(source.as_bytes()))),
                        "ensures" => *v = J::Array(expressions.clone()),
                        _ => {}
                    }
                }
                let hash = mpk_vc::hash_domain_separated_raw(a::METHOD_CONTRACT_HASH_DOMAIN,
                    &a::canonical_practical_json_bytes(&J::Object(fields.clone())).unwrap())
                    .unwrap().to_hex();
                fields.push(("contract_sha256".into(), J::string(hash)));
                vec![a::canonical_practical_json_bytes(&J::Object(fields)).unwrap()]
            });
        json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),
            "inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){
                "source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()})
    }).collect::<Vec<_>>())
}
#[test]
fn csharp_03_t06_w09_integer_codec_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_INTEGER_CODEC_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/integer-codec-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_INTEGER_CODEC_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-codec-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn alias(name: &str) -> String {
    format!(
        "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
#[test]
fn csharp_03_t06_w09_integer_codec_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_INTEGER_CODEC_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/integer-codec-clauses/responses.json")
    };
    let mut checked = 0;
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
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
        let formats = generate_csharp_practical_ordinary_integer_formats(vir).unwrap();
        let parsers = generate_csharp_practical_ordinary_integer_parsers(vir).unwrap();
        for recipe in recipes.values() {
            let (sc, target) = if recipe.tag == "codec_format" {
                let mut d = formats
                    .definitions()
                    .iter()
                    .find(|d| d.codec_id == id)
                    .unwrap()
                    .clone();
                let target = d.format_definition.clone();
                d.format_definition = alias(&recipe.name);
                let width = 1u32 << d.value_depth;
                for n in BTreeSet::from([
                    0,
                    1,
                    1u64 << (width - 1),
                    if width == 64 {
                        u64::MAX
                    } else {
                        (1u64 << width) - 1
                    },
                ]) {
                    integer_format_tests::format_case(
                        &c,
                        &d,
                        n,
                        &bundle,
                        emitted.closure().roots(),
                        emitted.closure().closed(),
                    );
                }
                (
                    mpk_cert::decode_canonical_certificate(formats.certificate_bytes()).unwrap(),
                    target,
                )
            } else {
                let mut d = parsers
                    .definitions()
                    .iter()
                    .find(|d| d.codec_id == id)
                    .unwrap()
                    .clone();
                let target = d.parse_definition.clone();
                d.parse_definition = alias(&recipe.name);
                for text in [
                    "0",
                    "-0",
                    "+1",
                    "01",
                    "-1",
                    "x",
                    "",
                    "18446744073709551615",
                    "18446744073709551616",
                    "-9223372036854775808",
                    "-9223372036854775809",
                    "１２",
                ] {
                    integer_parse_tests::parsed_case(
                        &c,
                        &d,
                        &text.encode_utf16().collect::<Vec<_>>(),
                        None,
                    );
                }
                for length in [16385, u32::MAX] {
                    integer_parse_tests::parsed_case(&c, &d, &[], Some(length));
                }
                (
                    mpk_cert::decode_canonical_certificate(parsers.certificate_bytes()).unwrap(),
                    target,
                )
            };
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
                c.name_table[c.declarations[global as usize].name as usize],
                target
            );
            structural_equivalence_tests::same_definition_closure(
                &sc,
                &c,
                &BTreeSet::from([target]),
            )
            .unwrap();
        }
        let types = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        for d in p.definitions() {
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| {
                    let depth = types
                        .carriers()
                        .iter()
                        .find(|c| &c.type_id == ty)
                        .unwrap()
                        .depth;
                    if depth == 0 {
                        V::Bit(false)
                    } else {
                        sparse_cube(depth, BTreeSet::new())
                    }
                })
                .collect::<Vec<_>>();
            assert!(
                bit(run(&c, &d.definedness_definition, args.clone())),
                "{id}: definedness"
            );
            assert!(bit(run(&c, &d.value_definition, args)), "{id}: value");
            checked += 1;
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
        eprintln!("integer codec clause {id} passed");
    }
    assert_eq!(checked, 76);
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
fn csharp_03_t06_w09_integer_codec_clauses_pinned_bytes() {
    let bundle = b();
    let requests = requests();
    assert_eq!(
        read("ordinary-foundation/integer-codec-clauses/requests.json"),
        requests
    );
    let responses = read("ordinary-foundation/integer-codec-clauses/responses.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-codec-clauses");
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
    assert_eq!(requests.as_array().unwrap().len(), 10);
}
