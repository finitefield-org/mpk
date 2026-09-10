//! Complete attachment scopes,including method/loop subjects and source owners.
#[path = "csharp_practical_ordinary_binding_clause_tests.rs"]
mod binding_clause_tests;
#[path = "csharp_practical_ordinary_calendar_clause_tests.rs"]
mod calendar_clause_tests;
#[path = "csharp_practical_ordinary_collection_clause_tests.rs"]
mod collection_clause_tests;
#[path = "csharp_practical_ordinary_decimal_clause_tests.rs"]
mod decimal_clause_tests;
#[path = "csharp_practical_ordinary_decimal_codec_clause_tests.rs"]
mod decimal_codec_clause_tests;
#[path = "csharp_practical_ordinary_exception_clause_tests.rs"]
mod exception_clause_tests;
#[path = "csharp_practical_ordinary_fixed_codec_clause_tests.rs"]
mod fixed_codec_clause_tests;
#[path = "csharp_practical_ordinary_floating_clause_tests.rs"]
mod floating_clause_tests;
#[path = "csharp_practical_ordinary_integer_codec_clause_tests.rs"]
mod integer_codec_clause_tests;
#[path = "csharp_practical_ordinary_integer_data_tests.rs"]
mod integer_data_tests;
#[path = "csharp_practical_ordinary_quantifier_tests.rs"]
mod quantifier_tests;
#[path = "csharp_practical_ordinary_string_clause_tests.rs"]
mod string_clause_tests;
#[path = "csharp_practical_ordinary_structural_data_tests.rs"]
mod structural_data_tests;
#[path = "csharp_practical_ordinary_tagged_make_tests.rs"]
mod tagged_make_tests;
#[path = "csharp_practical_ordinary_transition_clause_tests.rs"]
mod transition_clause_tests;
use super::*;
use mpk_vc::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
fn truth() -> J {
    J::Object(vec![
        ("tag".into(), J::string("literal")),
        ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
        ("value".into(), J::Bool(true)),
    ])
}
fn overflowing() -> J {
    let number = |n: &str| {
        J::Object(vec![
            ("tag".into(), J::string("literal")),
            ("type_id".into(), J::string("mpk.csharp.value.i32.v1")),
            ("value".into(), J::string(n)),
        ])
    };
    let binary = |op: &str, ty: &str, left: J, right: J| {
        J::Object(vec![
            ("tag".into(), J::string("binary")),
            ("type_id".into(), J::string(ty)),
            (
                "operation_id".into(),
                J::string(format!("integer.i32.{op}.checked")),
            ),
            ("left".into(), left),
            ("right".into(), right),
        ])
    };
    binary(
        "equal",
        "mpk.csharp.value.bool.v1",
        binary(
            "add",
            "mpk.csharp.value.i32.v1",
            number("2147483647"),
            number("1"),
        ),
        number("0"),
    )
}
fn multiowner_requests() -> Value {
    let original = requests();
    let raw = original[0]["inputs"]
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
    let source=b"namespace Data;public readonly struct First{public readonly int Value;}public readonly struct Second{public readonly long Value;}public static class Entry{public static int Run(First left,Second right){return 0;}}\n";
    let ty = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Data","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let first = ty("First");
    let second = ty("Second");
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"Data","owner":ty("Entry"),"name":"Run","parameter_type_ids":[first,second],"result_type_id":"mpk.csharp.value.i32.v1"})).unwrap();
    let mut requests = vec![];
    for partial in [false, true] {
        let (context, captures) = support::context_with_sidecars(
            &b(),
            &root,
            source,
            vec![
                "contracts/first.json".into(),
                "contracts/second.json".into(),
            ],
            |context| {
                [("First", "i32"), ("Second", "i64")]
                    .into_iter()
                    .map(|(name, primitive)| {
                        let source_type = ty(name);
                        let member = csharp_practical_stored_member_id(
                            &source_type,
                            "Value",
                            &json!({"kind":"primitive","id":primitive}),
                            "readonly_field",
                        )
                        .unwrap();
                        let J::Object(mut fields) = template.clone() else {
                            panic!()
                        };
                        fields.retain(|(key, _)| key != "contract_sha256");
                        for (key, value) in &mut fields {
                            match key.as_str() {
                                "semantic_context" => *value = context.semantic_context().clone(),
                                "source_type_id" => *value = J::string(&source_type),
                                "source_content_sha256" => {
                                    *value = J::string(format!("{:x}", Sha256::digest(source)))
                                }
                                "ordered_member_ids" => *value = J::Array(vec![J::string(&member)]),
                                "recursive_default" => {
                                    *value = J::Object(vec![("Value".into(), J::string("0"))])
                                }
                                "construction_invariant" => *value = J::Null,
                                "invariants" => {
                                    *value = J::Array(vec![if partial {
                                        overflowing()
                                    } else {
                                        truth()
                                    }])
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
                    })
                    .collect()
            },
        );
        requests.push(json!({"id":if partial {"multiowner-partial"} else {"multiowner-total"},"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    json!(requests)
}
#[test]
fn csharp_03_t06_w09_multiowner_requests() {
    let requests = multiowner_requests();
    if let Some(out) = std::env::var_os("MPK_W09_MULTIOWNER_REQUESTS_OUT") {
        fs::write(out, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/contract-expressions/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    let root = std::env::var_os("MPK_W09_CONTRACT_EXPRESSIONS_OUT").map(std::path::PathBuf::from);
    if let Some(root) = root {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/contract-expressions");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn pin(name: &str, program: &OrdinaryContractExpressionProgram) {
    output(
        &format!("{name}.hex"),
        program
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            .as_bytes(),
    );
    output(&format!("{name}.json"), &program.canonical_bytes());
}
fn metadata(program: &OrdinaryContractExpressionProgram, vc: &ValidatedPracticalVc) {
    let data = vc.data_vcs();
    let json: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
    assert_eq!(json["data_vc_sha256"], data.hash());
    assert_eq!(program.definitions().len(), data.contracts().len());
    for ((d, obligation), expression) in program
        .definitions()
        .iter()
        .zip(data.contracts())
        .zip(data.contract_expressions())
    {
        assert_eq!(d.data_contract_id, obligation.id);
        assert_eq!(d.attachment_sha256, obligation.attachment_sha256);
        assert_eq!(d.expression_sha256, expression.expression_sha256());
        assert_eq!(d.subjects, obligation.subjects);
        assert_eq!(d.result_type, expression.term().type_id());
    }
}
#[test]
fn csharp_03_t06_w09_contract_expression_method_and_loop_scopes() {
    let bundle = b();
    let requests = read("control-vc/measure-requests.json");
    let responses = read("control-vc/measure-responses.json");
    let mut scopes = 0;
    let mut samples = 0;
    let mut first_context_program: Option<(Vec<u8>, Vec<u8>)> = None;
    for id in [
        "total_variable",
        "partial_without_measure",
        "total_descending",
    ] {
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
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let p = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
        metadata(&p, &vc);
        if let Some((metadata, certificate)) = &first_context_program {
            assert!(import_csharp_practical_ordinary_contract_expressions(
                metadata,
                certificate,
                vir
            )
            .is_err());
        } else {
            first_context_program = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        }
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        assert!(p.definitions().iter().any(|d| d.allows_old));
        assert!(p
            .definitions()
            .iter()
            .any(|d| d.subjects.iter().any(|(name, _)| name == "result")));
        let mut by_expression: BTreeMap<&str, BTreeSet<Vec<(String, String)>>> = BTreeMap::new();
        let contract: Value = serde_json::from_str(
            request["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| !i["path"].as_str().unwrap().ends_with(".cs"))
                .unwrap()["utf8"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let declared_owner = contract["contract_sha256"].as_str().unwrap();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.owner.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["", declared_owner])
        );
        for (d, original) in p.definitions().iter().zip(vir.contract_expressions()) {
            by_expression
                .entry(&d.expression_sha256)
                .or_default()
                .insert(d.subjects.clone());
            assert!(d.exception_scope.is_none());
            // Preserve the frozen empty owner of loop-local captures;do not
            // invent ownership that belongs to control use-point instantiation.
            assert!(d.owner.is_empty() || d.owner == declared_owner);
            for seed in [1u32, 17, 0xffff_ff80] {
                let arguments = d
                    .subjects
                    .iter()
                    .enumerate()
                    .map(|(i, (_, ty))| {
                        if ty == "mpk.csharp.value.i32.v1" {
                            let n = seed.wrapping_add(i as u32 * 7);
                            sparse_cube(5, (0..32).filter(|bit| n & (1 << bit) != 0).collect())
                        } else {
                            sparse_cube(depths[ty.as_str()], BTreeSet::new())
                        }
                    })
                    .collect::<Vec<_>>();
                assert!(bit(run(&c, &d.definedness_definition, arguments.clone())));
                match original.term() {
                    ContractTerm::Var { index, type_id } => {
                        assert_eq!(type_id, "mpk.csharp.value.i32.v1");
                        let position = d.subjects.len() - 1 - index;
                        assert_eq!(d.subjects[position].0, "current:local:0");
                        assert_eq!(
                            count(&c, run(&c, &d.value_definition, arguments)),
                            seed.wrapping_add(position as u32 * 7)
                        );
                    }
                    ContractTerm::Const { .. } => {
                        assert!(bit(run(&c, &d.value_definition, arguments)))
                    }
                    other => panic!("unexpected fixture term {other:?}"),
                }
                samples += 1;
            }
        }
        assert!(by_expression.values().any(|s| s.len() > 1));
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.value_definition)
                .collect::<BTreeSet<_>>()
                .len(),
            p.definitions().len()
        );
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for field in [
            "owner",
            "subjects",
            "allows_old",
            "value_definition",
            "definedness_definition",
        ] {
            let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            changed["definitions"][0][field] = if field == "allows_old" {
                json!(!p.definitions()[0].allows_old)
            } else if field == "subjects" {
                json!([])
            } else {
                json!("changed")
            };
            assert!(
                import_csharp_practical_ordinary_contract_expressions(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err(),
                "{id} {field}"
            );
        }
        scopes += p.definitions().len();
        pin(id, &p);
    }
    eprintln!("all contract expressions: {scopes} method/loop attachments,{samples} value/definedness sample pairs");
}
#[test]
fn csharp_03_t06_w09_source_clause_multiowner_carriers() {
    let bundle = b();
    let requests = multiowner_requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_MULTIOWNER_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/contract-expressions/responses.json")
    };
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let partial = id == "multiowner-partial";
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
        let vir = emitted.vir();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let p = generate_csharp_practical_ordinary_source_clauses(vir).unwrap();
        let all = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
        metadata(&all, &vc);
        assert_eq!(p.definitions().len(), 2);
        assert_eq!(all.definitions().len(), 2);
        assert_eq!(
            p.definitions()[0].expression_sha256,
            p.definitions()[1].expression_sha256
        );
        assert_ne!(p.definitions()[0].definition, p.definitions()[1].definition);
        if partial {
            assert_ne!(
                p.definitions()[0].definedness_definition,
                p.definitions()[1].definedness_definition
            );
        }
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| depths[d.source_type_id.as_str()])
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([5, 6])
        );
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let ac = mpk_cert::decode_canonical_certificate(all.certificate_bytes()).unwrap();
        for d in p.definitions() {
            let a = all
                .definitions()
                .iter()
                .find(|a| a.attachment_sha256 == d.attachment_sha256)
                .unwrap();
            let input = sparse_cube(depths[d.source_type_id.as_str()], BTreeSet::new());
            assert_eq!(
                bit(run(&ac, &a.definedness_definition, vec![input.clone()])),
                !partial
            );
            if partial {
                assert!(!bit(run(
                    &c,
                    d.definedness_definition.as_ref().unwrap(),
                    vec![input]
                )));
            } else {
                assert!(bit(run(&c, &d.definition, vec![input.clone()])));
                assert!(bit(run(&ac, &a.value_definition, vec![input])));
            }
        }
        if partial {
            assert!(generate_csharp_practical_ordinary_structural_public(vir).is_err());
        } else {
            let integrated = generate_csharp_practical_ordinary_structural_public(vir).unwrap();
            let ic =
                mpk_cert::decode_canonical_certificate(integrated.certificate_bytes()).unwrap();
            let names = c
                .declarations
                .iter()
                .map(|d| c.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&c, &ic, &names).unwrap();
            assert_eq!(integrated.source_clauses().unwrap(), p.definitions());
            output(
                &format!("{id}-integrated.hex"),
                integrated
                    .certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
                    .as_bytes(),
            );
            output(
                &format!("{id}-integrated.json"),
                &integrated.canonical_bytes(),
            );
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
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &all.canonical_bytes(),
                all.certificate_bytes(),
                vir
            )
            .unwrap(),
            all
        );
        output(
            &format!("{id}-source.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        output(&format!("{id}-source.json"), &p.canonical_bytes());
        pin(id, &all);
    }
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
}

fn old_result_requests() -> Value {
    let original = read("control-vc/measure-requests.json")[0].clone();
    assert_eq!(original["id"], "total_variable");
    let inputs = original["inputs"].as_array().unwrap();
    // Keep the same signature but capture through the data frontend without a
    // loop; loop scopes are independently replayed from the control fixtures.
    let source = "namespace Business;public static class Entry{public static int Run(int n,int[] a,string s){return n;}}\n";
    let raw = inputs
        .iter()
        .find(|i| !i["path"].as_str().unwrap().ends_with(".cs"))
        .unwrap()["utf8"]
        .as_str()
        .unwrap();
    let template =
        a::parse_canonical_practical_json(a::PracticalArtifactKind::MethodContract, raw.as_bytes())
            .unwrap();
    let (context, captures) = support::context_with_sidecar(
        &b(),
        original["roots"][0].as_str().unwrap(),
        source.as_bytes(),
        |context| {
            let J::Object(mut fields) = template.clone() else {
                panic!()
            };
            fields.retain(|(k, _)| k != "contract_sha256");
            let integer = "mpk.csharp.value.i32.v1";
            let variable = J::Object(vec![
                ("tag".into(), J::string("variable")),
                ("type_id".into(), J::string(integer)),
                ("binding_id".into(), J::string("parameter:0")),
            ]);
            let old = J::Object(vec![
                ("tag".into(), J::string("old")),
                ("type_id".into(), J::string(integer)),
                ("expression".into(), variable),
            ]);
            let result = J::Object(vec![
                ("tag".into(), J::string("result")),
                ("type_id".into(), J::string(integer)),
            ]);
            let equal = J::Object(vec![
                ("tag".into(), J::string("binary")),
                ("type_id".into(), J::string("mpk.csharp.value.bool.v1")),
                (
                    "operation_id".into(),
                    J::string("integer.i32.equal.checked"),
                ),
                ("left".into(), result),
                ("right".into(), old),
            ]);
            for (key, value) in &mut fields {
                match key.as_str() {
                    "semantic_context" => *value = context.semantic_context().clone(),
                    "source_content_sha256" => {
                        *value = J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                    }
                    "loops" => *value = J::Array(vec![]),
                    "ensures" => *value = J::Array(vec![equal.clone()]),
                    _ => {}
                }
            }
            let hash = mpk_vc::hash_domain_separated_raw(
                a::METHOD_CONTRACT_HASH_DOMAIN,
                &a::canonical_practical_json_bytes(&J::Object(fields.clone())).unwrap(),
            )
            .unwrap()
            .to_hex();
            fields.push(("contract_sha256".into(), J::string(hash)));
            a::canonical_practical_json_bytes(&J::Object(fields)).unwrap()
        },
    );
    json!([{"id":"old-result","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.path().ends_with(".cs"){"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}])
}
#[test]
fn csharp_03_t06_w09_old_result_requests() {
    let request = old_result_requests();
    if let Some(path) = std::env::var_os("MPK_W09_OLD_RESULT_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/contract-expressions/old-result-requests.json"),
            request
        );
    }
}
#[test]
fn csharp_03_t06_w09_contract_expression_old_and_result_values() {
    let bundle = b();
    let requests = old_result_requests();
    let request = &requests[0];
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_OLD_RESULT_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/contract-expressions/old-result-responses.json")
    };
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let vc = generate_csharp_practical_vc(PracticalVcSource {
        artifact_context: &context,
        captured_inputs: &captures,
        vir,
    })
    .unwrap();
    let p = generate_csharp_practical_ordinary_contract_expressions(vir).unwrap();
    metadata(&p, &vc);
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    let depths = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c.depth))
        .collect::<BTreeMap<_, _>>();
    let ensures = p
        .definitions()
        .iter()
        .filter(|d| d.allows_old)
        .collect::<Vec<_>>();
    assert_eq!(ensures.len(), 1);
    let ensures = ensures[0];
    assert_eq!(ensures.subjects.len(), 7);
    for (current, entry, result) in [
        (1u32, 17u32, 17u32),
        (17, 1, 17),
        (42, 42, 42),
        (0, 0xffff_ffff, 0xffff_ffff),
        (0xffff_ffff, 0, 0),
    ] {
        let arguments = ensures
            .subjects
            .iter()
            .map(|(name, ty)| {
                let value = match name.as_str() {
                    "current:parameter:0" => Some(current),
                    "entry:parameter:0" => Some(entry),
                    "result" => Some(result),
                    _ => None,
                };
                value.map_or_else(
                    || sparse_cube(depths[ty.as_str()], BTreeSet::new()),
                    |n| sparse_cube(5, (0..32).filter(|i| n & (1 << i) != 0).collect()),
                )
            })
            .collect::<Vec<_>>();
        // Distinct current/entry values expose aliasing. These binding probes
        // do not claim the assignment is reachable in the original CFG.
        assert!(bit(run(
            &c,
            &ensures.definedness_definition,
            arguments.clone()
        )));
        assert_eq!(
            bit(run(&c, &ensures.value_definition, arguments)),
            entry == result
        );
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
    let mut mutation: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    let row = mutation["definitions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|d| d["attachment_sha256"] == ensures.attachment_sha256)
        .unwrap();
    row["subjects"].as_array_mut().unwrap().swap(0, 3);
    assert!(import_csharp_practical_ordinary_contract_expressions(
        &serde_json::to_vec(&mutation).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    pin("old-result", &p);
    output(
        "old-result-requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
}

#[path = "csharp_practical_ordinary_floating_data_tests.rs"]
mod floating_data_tests;

#[path = "csharp_practical_ordinary_decimal_data_tests.rs"]
mod decimal_data_tests;
