//! Total contract observations reuse frozen sequence/sum layouts.
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
pub(super) fn literal(label: &str) -> J {
    let (_, raw, value) = literal_clause_tests::cases()
        .into_iter()
        .find(|(name, _, _)| name == label)
        .unwrap();
    J::Object(vec![
        ("tag".into(), J::string("literal")),
        ("type_id".into(), J::string(value.type_id())),
        ("value".into(), raw),
    ])
}
fn observation(tag: &str, ty: &str, field: &str, value: J) -> J {
    J::Object(vec![
        ("tag".into(), J::string(tag)),
        ("type_id".into(), J::string(ty)),
        (field.into(), value),
    ])
}
fn equal_word(ty: &str, value: J, number: &str) -> J {
    let token = ty
        .strip_prefix("mpk.csharp.value.")
        .unwrap()
        .strip_suffix(".v1")
        .unwrap();
    J::Object(vec![
        ("tag".into(), J::string("binary")),
        ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
        (
            "operation_id".into(),
            J::string(format!("integer.{token}.equal.checked")),
        ),
        ("left".into(), value),
        (
            "right".into(),
            J::Object(vec![
                ("tag".into(), J::string("literal")),
                ("type_id".into(), J::string(ty)),
                ("value".into(), J::string(number)),
            ]),
        ),
    ])
}
fn clauses() -> Vec<(String, J, bool)> {
    let i32_id = "mpk.csharp.value.i32.v1";
    let u32_id = "mpk.csharp.value.u32.v1";
    let sequence = literal("sequence");
    let J::Object(mut empty) = sequence.clone() else {
        panic!()
    };
    empty
        .iter_mut()
        .find(|(name, _)| name == "value")
        .unwrap()
        .1 = J::Array(vec![]);
    let mut clauses = vec![
        (
            "empty length".into(),
            equal_word(
                i32_id,
                observation("sequence_length", i32_id, "sequence", J::Object(empty)),
                "0",
            ),
            true,
        ),
        (
            "two item length".into(),
            equal_word(
                i32_id,
                observation("sequence_length", i32_id, "sequence", sequence),
                "2",
            ),
            true,
        ),
    ];
    for value in ["none", "some"] {
        for arm in ["none", "some"] {
            let J::Object(mut expression) = observation(
                "tagged_is",
                "mpk.csharp.value.bool.v1",
                "value",
                literal(value),
            ) else {
                panic!()
            };
            expression.push(("arm".into(), J::string(arm)));
            clauses.push((
                format!("{value} is {arm}"),
                J::Object(expression),
                value == arm,
            ));
        }
    }
    for (arm, code) in [("input_bound", "0"), ("range", "4")] {
        let J::Object(mut value) = literal("parse_error") else {
            panic!()
        };
        value.iter_mut().find(|(k, _)| k == "value").unwrap().1 = J::string(arm);
        clauses.push((
            format!("parse error {arm}"),
            equal_word(
                u32_id,
                observation("parse_error_kind", u32_id, "value", J::Object(value)),
                code,
            ),
            true,
        ));
    }
    clauses
}
fn total_requests() -> Value {
    requests_with_expressions(
        clauses().into_iter().map(|(_, expr, _)| expr).collect(),
        "total-observation-clauses",
    )
}
pub(super) fn requests_with_expressions(expressions: Vec<J>, id: &str) -> Value {
    let original = requests();
    let original = &original[0];
    let inputs = original["inputs"].as_array().unwrap();
    let code = inputs
        .iter()
        .find(|i| i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let contract = inputs
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
    let (context, captures) = support::context_with_sidecar(
        &b(),
        original["roots"][0].as_str().unwrap(),
        code.as_bytes(),
        |context| {
            let J::Object(mut fields) = template.clone() else {
                panic!()
            };
            fields.retain(|(key, _)| key != "contract_sha256");
            for (key, value) in &mut fields {
                match key.as_str() {
                    "semantic_context" => *value = context.semantic_context().clone(),
                    "invariants" => *value = J::Array(expressions.clone()),
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
        },
    );
    json!([{"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}
#[test]
fn csharp_03_t06_w09_total_clause_requests() {
    let requests = total_requests();
    if let Some(path) = std::env::var_os("MPK_W09_TOTAL_CLAUSE_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/total-clauses/requests.json"),
            requests
        );
    }
}
fn core_name(name: &str) -> String {
    format!(
        "Mpk.CSharp.Ordinary.ContractDefinition.N{}",
        name.as_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    )
}
fn header(depth: u32, number: u32, payload: bool) -> V {
    let mut ones = (0..32)
        .filter(|i| number & (1 << i) != 0)
        .map(|i| i << (depth - 5))
        .collect::<BTreeSet<_>>();
    if payload {
        ones.insert(1);
    }
    sparse_cube(depth, ones)
}
#[test]
fn csharp_03_t06_w09_total_clauses_original_source() {
    let bundle = b();
    let request = total_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_TOTAL_CLAUSE_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/total-clauses/responses.json")
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
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let depths: BTreeMap<_, _> = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c.depth))
        .collect();
    assert_eq!(p.definitions().len(), clauses().len());
    for ((label, expression, expected), definition) in clauses().iter().zip(p.definitions()) {
        let mut digest = Sha256::new();
        digest.update(b"MPK-CSHARP-CONTRACT-EXPRESSION-1.0\0");
        digest.update(a::canonical_practical_json_bytes(expression).unwrap());
        let hash = format!("{:x}", digest.finalize());
        assert_eq!(definition.expression_sha256, hash);
        assert_eq!(
            bit(run(
                &c,
                &definition.definition,
                vec![sparse_cube(
                    depths[definition.source_type_id.as_str()],
                    BTreeSet::new()
                )]
            )),
            *expected,
            "{label}"
        );
    }
    let mut operations = BTreeMap::new();
    for expression in vir.contract_expressions() {
        for d in expression.definitions() {
            if matches!(
                d.tag.as_str(),
                "sequence_length" | "tagged_is" | "parse_error_kind"
            ) {
                operations.insert(d.name.clone(), d.clone());
            }
        }
    }
    assert_eq!(operations.len(), 4);
    let words = [
        0,
        1,
        2,
        3,
        4,
        4095,
        4096,
        4097,
        16384,
        65536,
        0x8000_0000,
        u32::MAX,
    ];
    let mut observations = 0;
    for (name, d) in &operations {
        let name = core_name(name);
        let depth = depths[d.argument_types[0].as_str()];
        match d.tag.as_str() {
            "sequence_length" => {
                for word in words.into_iter().chain((0..32).map(|i| 1 << i)) {
                    // Includes invalid high-bit headers to detect truncation;
                    // these raw probes do not claim public-domain membership.
                    assert_eq!(
                        count(&c, run(&c, &name, vec![header(depth, word, true)])),
                        word
                    );
                    observations += 32;
                }
            }
            "tagged_is" => {
                let params: Value = serde_json::from_str(&d.parameters).unwrap();
                let expected_tag = if params["arm"] == "none" { 0 } else { 1 };
                for word in words {
                    for payload in [false, true] {
                        assert_eq!(
                            bit(run(&c, &name, vec![header(depth, word, payload)])),
                            word == expected_tag,
                            "tag {word} payload {payload}"
                        );
                        observations += 1;
                    }
                }
            }
            "parse_error_kind" => {
                assert_eq!(depth, 5);
                for word in words.into_iter().chain((0..32).map(|i| 1 << i)) {
                    assert_eq!(
                        count(&c, run(&c, &name, vec![header(depth, word, false)])),
                        word
                    );
                    observations += 32;
                }
            }
            _ => unreachable!(),
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
    validate_csharp_practical_certificate_structure(&integrated_cert).unwrap();
    let public = generate_csharp_practical_ordinary_public_defaults(vir).unwrap();
    let public_cert = mpk_cert::decode_canonical_certificate(public.certificate_bytes()).unwrap();
    let base = generate_csharp_practical_ordinary_structural_foundations(vir).unwrap();
    let base_cert = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
    for prior in [&c, &public_cert, &base_cert] {
        let names = prior
            .declarations
            .iter()
            .map(|d| prior.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(prior, &integrated_cert, &names)
            .unwrap();
    }
    assert_eq!(integrated.relations(), base.relations());
    assert_eq!(integrated.collections(), base.collections());
    assert_eq!(integrated.domains(), base.domains());
    assert_eq!(integrated.public_domains().unwrap(), public.definitions());
    assert_eq!(integrated.public_defaults(), public.public_defaults());
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
    let outputs = [
        (
            "total-source.hex",
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into_bytes(),
        ),
        (
            "total-integrated.hex",
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
    let out = std::env::var_os("MPK_W09_TOTAL_CLAUSES_OUT").map(std::path::PathBuf::from);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/total-clauses");
    for (name, bytes) in outputs {
        if let Some(out) = &out {
            fs::create_dir_all(out).unwrap();
            fs::write(out.join(name), bytes).unwrap();
        } else {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
        }
    }
    eprintln!("total clauses: {} source expressions, {} direct functions, {observations} bit/tag observations; source {} terms/{} declarations, integrated {} terms/{} declarations", clauses().len(), operations.len(), c.term_table.len(), c.declarations.len(), integrated_cert.term_table.len(), integrated_cert.declarations.len());
}
