//! W03 definedness compiled separately from source predicate values.
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
const I32: &str = "mpk.csharp.value.i32.v1";
const BOOL: &str = "mpk.csharp.value.bool.v1";
fn expr(value: Value) -> J {
    // Contract expressions require the frozen field order. serde_json's map
    // order is insufficient even when all fields and types are correct.
    let Some(tag) = value.get("tag").and_then(Value::as_str) else {
        return a::parse_canonical_practical_json(
            a::PracticalArtifactKind::TypeContract,
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
    };
    let rest: &[&str] = match tag {
        "literal" => &["value"],
        "variable" => &["binding_id"],
        "field" => &["receiver", "member_id"],
        "binary" => &["operation_id", "left", "right"],
        "unary" => &["operation_id", "operand"],
        "conditional" => &["condition", "when_true", "when_false"],
        "let" => &["binding_id", "value", "body"],
        _ => panic!("unexpected test expression {tag}"),
    };
    assert_eq!(value.as_object().unwrap().len(), 2 + rest.len());
    J::Object(
        ["tag", "type_id"]
            .into_iter()
            .chain(rest.iter().copied())
            .map(|key| (key.into(), expr(value.get(key).unwrap().clone())))
            .collect(),
    )
}
fn clauses(template: &J) -> Vec<(String, J)> {
    let template: Value =
        serde_json::from_slice(&a::canonical_practical_json_bytes(template).unwrap()).unwrap();
    let field = template["invariants"][0]["left"].clone();
    assert_eq!(field["tag"], "field");
    let literal = |n: i32| json!({"tag":"literal","type_id":I32,"value":n.to_string()});
    let binary = |op: &str, left: Value, right: Value| json!({"tag":"binary","type_id":if op == "equal" {BOOL} else {I32},"operation_id":format!("integer.i32.{op}.checked"),"left":left,"right":right});
    let equal_zero = |value| binary("equal", value, literal(0));
    let truth = json!({"tag":"literal","type_id":BOOL,"value":true});
    let divide = binary("divide", literal(12), field.clone());
    let add = binary("add", field.clone(), literal(1));
    let choose = |yes, no| json!({"tag":"conditional","type_id":BOOL,"condition":equal_zero(field.clone()),"when_true":yes,"when_false":no});
    let variable = json!({"tag":"variable","type_id":I32,"binding_id":"divisor"});
    let local_divide = equal_zero(binary("divide", literal(12), variable));
    let bind = |value, body| json!({"tag":"let","type_id":BOOL,"binding_id":"divisor","value":value,"body":body});
    vec![
        ("checked add", equal_zero(add.clone())),
        ("checked negate", equal_zero(json!({"tag":"unary","type_id":I32,"operation_id":"integer.i32.negate.checked","operand":field.clone()}))),
        ("divide", equal_zero(divide.clone())),
        ("remainder", equal_zero(binary("remainder", literal(i32::MIN), field.clone()))),
        ("guarded division", choose(truth.clone(), equal_zero(divide.clone()))),
        ("selected failing branch", choose(equal_zero(divide), truth.clone())),
        ("unused checked let", bind(add, truth)),
        ("local divisor", bind(field.clone(), local_divide.clone())),
        ("outer divisor under nested let", bind(field, json!({"tag":"let","type_id":BOOL,"binding_id":"unused","value":literal(0),"body":local_divide}))),
    ].into_iter().map(|(label, value)|(label.into(),expr(value))).collect()
}
fn expected(index: usize, n: i32) -> Option<bool> {
    match index {
        0 => n.checked_add(1).map(|v| v == 0),
        1 => n.checked_neg().map(|v| v == 0),
        2 | 7 | 8 => 12i32.checked_div(n).map(|v| v == 0),
        3 => i32::MIN.checked_rem(n).map(|v| v == 0),
        4 => {
            if n == 0 {
                Some(true)
            } else {
                expected(2, n)
            }
        }
        5 => {
            if n == 0 {
                expected(2, n)
            } else {
                Some(true)
            }
        }
        6 => n.checked_add(1).map(|_| true),
        _ => unreachable!(),
    }
}
fn definedness_requests() -> Value {
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
                    "invariants" => {
                        *value = J::Array(
                            clauses(&template)
                                .into_iter()
                                .map(|(_, expr)| expr)
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
        },
    );
    json!([{"id":"checked-definedness-clauses","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}

#[test]
fn csharp_03_t06_w09_definedness_requests() {
    let request = definedness_requests();
    if let Some(path) = std::env::var_os("MPK_W09_DEFINEDNESS_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/definedness-clauses/requests.json"),
            request
        );
    }
}
#[test]
fn csharp_03_t06_w09_definedness_original_source() {
    let bundle = b();
    let request = definedness_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_DEFINEDNESS_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/definedness-clauses/responses.json")
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
    let contract = request[0]["inputs"]
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
    // This template already contains our clauses, so recover their exact order
    // directly, rather than regenerate from the original field expression.
    let canonical: Value =
        serde_json::from_slice(&a::canonical_practical_json_bytes(&template).unwrap()).unwrap();
    assert_eq!(p.definitions().len(), 9);
    let numbers = [
        i32::MIN,
        i32::MIN + 1,
        -12,
        -1,
        0,
        1,
        12,
        i32::MAX - 1,
        i32::MAX,
    ];
    let mut cases = 0;
    for (index, definition) in p.definitions().iter().enumerate() {
        let raw = expr(canonical["invariants"][index].clone());
        let mut hash = Sha256::new();
        hash.update(b"MPK-CSHARP-CONTRACT-EXPRESSION-1.0\0");
        hash.update(a::canonical_practical_json_bytes(&raw).unwrap());
        assert_eq!(
            definition.expression_sha256,
            format!("{:x}", hash.finalize())
        );
        let carrier = layouts
            .carriers()
            .iter()
            .find(|t| t.type_id == definition.source_type_id)
            .unwrap();
        // Value.Amount is the only stored field; verify layout before using the
        // independent source i32 values as a Boolean-cube oracle.
        let OrdinaryShape::Product { fields } = &carrier.shape else {
            panic!()
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(
            fields[0].shape,
            OrdinaryShape::Reference {
                type_id: I32.into()
            }
        );
        assert_eq!(carrier.depth, 5);
        for n in numbers {
            let value = sparse_cube(5, (0..32).filter(|i| (n as u32) & (1 << i) != 0).collect());
            let expected = expected(index, n);
            assert_eq!(
                bit(run(
                    &c,
                    definition.definedness_definition.as_ref().unwrap(),
                    vec![value.clone()]
                )),
                expected.is_some(),
                "defined clause {index}, {n}"
            );
            // Never use the failure-side result as a normal contract value.
            if let Some(expected) = expected {
                assert_eq!(
                    bit(run(&c, &definition.definition, vec![value])),
                    expected,
                    "value clause {index}, {n}"
                );
            }
            cases += 1;
        }
    }
    // Public-domain/control owners cannot yet discharge these separate W03
    // conditions. All current consumers must fail closed instead of assuming it.
    assert!(generate_csharp_practical_ordinary_public_domains(vir).is_err());
    assert!(generate_csharp_practical_ordinary_public_defaults(vir).is_err());
    assert!(generate_csharp_practical_ordinary_structural_public(vir).is_err());
    assert_eq!(
        import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let mut mutated: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    mutated["definitions"][0]
        .as_object_mut()
        .unwrap()
        .remove("definedness_definition");
    assert!(import_csharp_practical_ordinary_source_clauses(
        &serde_json::to_vec(&mutated).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let outputs = [
        (
            "checked-source.hex",
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .into_bytes(),
        ),
        ("program.json", p.canonical_bytes()),
        (
            "requests.json",
            serde_json::to_vec_pretty(&request).unwrap(),
        ),
    ];
    let out = std::env::var_os("MPK_W09_DEFINEDNESS_OUT").map(std::path::PathBuf::from);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/definedness-clauses");
    for (name, bytes) in outputs {
        if let Some(out) = &out {
            fs::create_dir_all(out).unwrap();
            fs::write(out.join(name), bytes).unwrap();
        } else {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
        }
    }
    eprintln!(
        "definedness: {cases} original-source cases, {} terms, {} declarations",
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn csharp_03_t06_w09_definedness_input_types() {
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
    let raw = request[0]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| !i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let template =
        a::parse_canonical_practical_json(a::PracticalArtifactKind::TypeContract, raw.as_bytes())
            .unwrap();
    let json: Value = serde_json::from_str(raw).unwrap();
    let mut env = DataContractEnvironment::default();
    env.variables.insert(
        "this".into(),
        json["invariants"][0]["left"]["receiver"]["type_id"]
            .as_str()
            .unwrap()
            .into(),
    );
    let mut failures = vec![];
    for (label, expression) in clauses(&template) {
        if let Err(error) = import_verification_contract_expression(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &env,
            &a::canonical_practical_json_bytes(&expression).unwrap(),
        ) {
            failures.push(format!("{label}: {error:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

/// Fast byte-compatibility gate for later recipe changes. The separate source
/// test retains the independent arithmetic/definedness semantic observations.
#[test]
fn csharp_03_t06_w09_definedness_certificate_compatibility() {
    let bundle = b();
    let request = definedness_requests();
    let responses = read("ordinary-foundation/definedness-clauses/responses.json");
    let (context, captures) = support::replay_context(&bundle, &request[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/definedness-clauses");
    let program = generate_csharp_practical_ordinary_source_clauses(emitted.vir()).unwrap();
    assert_eq!(
        fs::read(root.join("program.json")).unwrap(),
        program.canonical_bytes()
    );
    assert_eq!(
        fs::read_to_string(root.join("checked-source.hex"))
            .unwrap()
            .trim(),
        program
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
}
