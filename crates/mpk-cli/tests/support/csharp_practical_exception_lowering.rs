//! W04: original-source throws, closed values, exceptional contracts and mutations.
use super::lowering::{evaluate, R};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn fixtures() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/exception-lowering/source-cases.json",
    ))
    .unwrap()
}
pub(super) fn exception_roots(
    b: &ValidatedFoundationBundle,
    case: &Value,
) -> (ValidatedClosedRootSet, ClosedInstanceSet) {
    let primitive = json!({"kind":"primitive","id":"i32"});
    let mut sources = serde_json::Map::new();
    let mut types = vec![primitive.clone()];
    for definition in case["lowering"]["exception_definitions"]
        .as_array()
        .unwrap()
    {
        let id = definition["type_id"].as_str().unwrap();
        let storage = if case["id"] == "get_only" {
            "get_auto"
        } else if case["id"] == "init_only" {
            "init_auto"
        } else {
            "readonly_field"
        };
        let member = csharp_practical_stored_member_id(id, "Code", &primitive, storage).unwrap();
        let empty = case["id"] == "empty";
        let identity = json!({"kind":"type","namespace":"Business","owner":"","name":"Fault","parameter_type_ids":[],"result_type_id":""});
        sources.insert(id.into(),json!({"id":id,"identity":identity,"kind":"sealed_class",
            "members":if empty{vec![]}else{vec![json!({"id":member,"name":"Code","type":primitive,"storage":storage,"ordinal":0,"required":false})]},
            "enum_values":[],"enum_underlying":null,"actual_default":if empty{json!({})}else{json!({member:0})},"public_default":false,"identity_sensitive":false,
            "source_sha256":case["lowering"]["facts"]["sources"][0]["raw_sha256"]}));
        types.push(json!({"kind":"source","id":id}));
    }
    let roots=Value::Array(types.into_iter().enumerate().map(|(i,t)|json!({"origin":"semantic_binding","provenance_id":format!("exception.type.{i}"),"type":t})).collect());
    let bytes = canonical_closed_root_set_transport(b, &roots, &json!(sources)).unwrap();
    let r = validate_closed_root_set(b, &bytes).unwrap();
    let c = derive_closed_instances(b, &r).unwrap();
    (r, c)
}
fn prepare(
    case: &Value,
    wire_mutation: impl FnOnce(&mut Value),
    contract_mutation: impl FnOnce(&mut J),
) -> Result<LoweredLoopControl, LoopLoweringError> {
    let b = bundle();
    let (r, c) = exception_roots(&b, case);
    let mut source = case.clone();
    source["facts"] = case["lowering"]["facts"].clone();
    let (ctx, captures) = context_support::context_with_sidecar(
        &b,
        case["root"].as_str().unwrap(),
        case["source"].as_str().unwrap().as_bytes(),
        |ctx| {
            let mut doc = document(ctx, &source, "total", rows(&source));
            contract_mutation(&mut doc);
            hashed(doc)
        },
    );
    let mut wire = case["lowering"].clone();
    wire_mutation(&mut wire);
    prepare_exception_lowering(
        &b,
        &r,
        &c,
        &ctx,
        &captures,
        &serde_json::to_vec(&source["facts"]).unwrap(),
        &serde_json::to_vec(&wire).unwrap(),
        &DataContractEnvironment::default(),
        &BTreeSet::new(),
    )
}
fn interpret(
    f: &LoopControlFunction,
    run: &Value,
    case_id: &str,
) -> (Option<i64>, String, Option<i64>) {
    let mut slots = BTreeMap::from([("parameter:0".into(), R::Number(run["n"].as_i64().unwrap()))]);
    slots.insert("this".into(), R::Number(0));
    let mut values = BTreeMap::<String, R>::new();
    let nodes = f
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let mut current = &f.nodes[0];
    let mut error = String::new();
    let mut code = None;
    for _ in 0..10000 {
        let inputs = current
            .inputs
            .iter()
            .map(|id| values[id].clone())
            .collect::<Vec<_>>();
        match current.kind.as_str() {
            "return" => {
                return (
                    Some(inputs.first().unwrap_or(&slots["this"]).number().unwrap()),
                    String::new(),
                    None,
                )
            }
            "throw" => return (None, error, None),
            "exception_exit" => return (None, error, code),
            "explicit_throw" => {
                error = if current.slot.starts_with("System.") {
                    current.slot.clone()
                } else {
                    "Business.Fault".into()
                };
                if error == "Business.Fault" && case_id != "empty" {
                    code = Some(inputs[0].number().unwrap());
                }
                current = nodes[current.exceptional_successors[0].as_str()];
                continue;
            }
            "branch" | "loop_header" => {
                current =
                    nodes[current.successors[usize::from(!inputs[0].boolean().unwrap())].as_str()];
                continue;
            }
            "evaluate" => {
                let value = match current.operation.as_str() {
                    "construction_assign" => {
                        slots.insert("this".into(), inputs[1].clone());
                        Ok(inputs[1].clone())
                    }
                    "closed_exception" => Ok(inputs.first().cloned().unwrap_or(R::Null)),
                    // Fixture payload construction is a pure immutable Code
                    // field. T03 separately validates constructor execution.
                    "construct"
                        if case_id == "constructor_failure" && inputs[0].number().unwrap() < 0 =>
                    {
                        Err("ArgumentException".into())
                    }
                    "construct" => Ok(if inputs.is_empty() {
                        R::Null
                    } else if case_id == "argument_order" {
                        R::Number(inputs[0].number().unwrap() * 10 + inputs[1].number().unwrap())
                    } else {
                        inputs[0].clone()
                    }),
                    _ => evaluate(
                        current,
                        current.source_ordinal.map(|i| &f.operations[i]),
                        &inputs,
                        &mut slots,
                    ),
                };
                match value {
                    Ok(v) => {
                        values.insert(current.result.clone(), v);
                    }
                    Err(e) => {
                        error = format!("System.{e}");
                        assert_eq!(current.exceptional_successors.len(), 1);
                        current = nodes[current.exceptional_successors[0].as_str()];
                        continue;
                    }
                }
            }
            "entry" | "jump" | "pattern_decision" => (),
            kind => panic!("{kind}"),
        }
        current = nodes[current.successors[0].as_str()];
    }
    panic!("execution budget")
}
#[test]
fn csharp_03_t04_w04_closed_throws_match_original_clr() {
    let mut count = 0;
    for case in fixtures().iter().filter(|c| c["accepted"] == true) {
        let prepared =
            prepare(case, |_| {}, |_| {}).unwrap_or_else(|e| panic!("{}:{e:?}", case["id"]));
        assert_eq!(prepared.artifact_count(), 0);
        assert!(!prepared.exceptions().is_empty());
        assert!(prepared
            .exceptions()
            .iter()
            .all(|e| !e.declared && e.catch_or_unreachable));
        let f = prepared
            .functions()
            .iter()
            .find(|f| f.callable_id == case["root"].as_str().unwrap())
            .unwrap();
        for run in case["runs"].as_array().unwrap() {
            assert_eq!(
                interpret(f, run, case["id"].as_str().unwrap()),
                (
                    run["value"].as_i64(),
                    run["error"].as_str().unwrap().into(),
                    run["code"].as_i64()
                ),
                "{}:{run}",
                case["id"]
            );
            if case["id"] == "constructor_failure" {
                let ctor = prepared
                    .functions()
                    .iter()
                    .find(|f| f.nodes.iter().any(|n| n.operation == "construction_assign"))
                    .unwrap();
                let n = run["n"].as_i64().unwrap();
                assert_eq!(
                    interpret(ctor, run, "constructor_failure"),
                    if n < 0 {
                        (None, "System.ArgumentException".into(), None)
                    } else {
                        (Some(n), String::new(), None)
                    }
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 90);
}
fn exception_case(id: &str, ensures: Vec<J>) -> J {
    J::object(vec![
        ("exception_type_id", J::string(id)),
        ("path_condition", boolean()),
        ("ensures", J::Array(ensures)),
    ])
}
#[test]
fn csharp_03_t04_w04_exceptional_contracts_and_closed_set() {
    let cases = fixtures();
    let case = cases.iter().find(|c| c["id"] == "payload").unwrap();
    let id = case["lowering"]["exception_definitions"][0]["type_id"]
        .as_str()
        .unwrap();
    let declared = prepare(
        case,
        |_| {},
        |doc| {
            set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case(id, vec![boolean()])]),
            )
        },
    )
    .unwrap();
    assert!(declared.exceptions()[0].declared);
    assert!(!declared.exceptions()[0].catch_or_unreachable);
    for invalid in [
        "System.OutOfMemoryException",
        "System.StackOverflowException",
        "System.Exception",
        "System.NotSupportedException",
    ] {
        assert!(prepare(
            case,
            |_| {},
            |doc| set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case(invalid, vec![])])
            )
        )
        .is_err());
    }
    assert!(prepare(
        case,
        |_| {},
        |doc| set(
            doc,
            "exceptional_cases",
            J::Array(vec![exception_case(id, vec![integer()])])
        )
    )
    .is_err());
    assert!(prepare(
        case,
        |_| {},
        |doc| set(
            doc,
            "exceptional_cases",
            J::Array(vec![exception_case(id, vec![]), exception_case(id, vec![])])
        )
    )
    .is_ok());
}
#[test]
fn csharp_03_t04_w04_mutated_values_edges_and_declarations_reject() {
    let cases = fixtures();
    let case = cases.iter().find(|c| c["id"] == "payload").unwrap();
    for mutation in 0..7 {
        assert!(
            prepare(
                case,
                |wire| {
                    if mutation == 0 {
                        wire["exception_definitions"][0]["sealed_type"] = json!(false);
                    } else if mutation == 1 {
                        wire["exception_definitions"][0]["direct_base_type_id"] =
                            json!("System.ArgumentException");
                    } else if mutation == 2 {
                        wire["exception_definitions"][0]["payload_member_names"] = json!([]);
                    } else {
                        let nodes = wire["functions"][0]["nodes"].as_array_mut().unwrap();
                        let n = nodes
                            .iter_mut()
                            .find(|n| n["operation"] == "closed_exception")
                            .unwrap();
                        match mutation {
                            3 => n["slot"] = json!("System.ArgumentException"),
                            4 => n["inputs"] = json!([]),
                            5 => n["source_ordinal"] = json!(0),
                            _ => {
                                let n = nodes
                                    .iter_mut()
                                    .find(|n| n["kind"] == "explicit_throw")
                                    .unwrap();
                                n["exceptional_successors"] = json!([]);
                            }
                        }
                    }
                },
                |_| {}
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn csharp_03_t04_w04_constructor_storage_and_receiver_mutations_reject() {
    let cases = fixtures();
    let case = cases
        .iter()
        .find(|c| c["id"] == "constructor_failure")
        .unwrap();
    for mutation in 0..3 {
        assert_eq!(
            prepare(
                case,
                |wire| {
                    let function = wire["functions"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|f| {
                            f["nodes"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|n| n["operation"] == "construction_assign")
                        })
                        .unwrap();
                    let nodes = function["nodes"].as_array_mut().unwrap();
                    let other = nodes.iter().find(|n| n["operation"] == "constant").unwrap()
                        ["result"]
                        .clone();
                    let thrown = nodes.iter().find(|n| n["kind"] == "throw").unwrap()["id"].clone();
                    let store = nodes
                        .iter_mut()
                        .find(|n| n["operation"] == "construction_assign")
                        .unwrap();
                    if mutation == 0 {
                        store["slot"] = json!("Message");
                    } else if mutation == 1 {
                        store["inputs"][0] = other;
                    } else {
                        store["exceptional_successors"] = json!([thrown]);
                    }
                },
                |_| {}
            )
            .unwrap_err(),
            LoopLoweringError::Operand
        );
    }
}

#[test]
fn csharp_03_t04_w04_payload_contract_scope_and_type() {
    let cases = fixtures();
    let case = cases.iter().find(|c| c["id"] == "payload").unwrap();
    let id = case["lowering"]["exception_definitions"][0]["type_id"]
        .as_str()
        .unwrap();
    let member = csharp_practical_stored_member_id(
        id,
        "Code",
        &json!({"kind":"primitive","id":"i32"}),
        "readonly_field",
    )
    .unwrap();
    let is = J::object(vec![
        ("tag", J::string("exception_is")),
        ("type_id", J::string(ty("bool"))),
        ("value", variable("exception", "exception")),
        ("exception_type_id", J::string(id)),
    ]);
    let payload = J::object(vec![
        ("tag", J::string("exception_payload")),
        ("type_id", J::string(ty("i32"))),
        ("value", variable("exception", "exception")),
        ("member_id", J::string(member)),
    ]);
    let clause = |value: J| {
        J::object(vec![
            ("tag", J::string("let")),
            ("type_id", J::string(ty("bool"))),
            ("binding_id", J::string("payload")),
            ("value", value),
            ("body", is.clone()),
        ])
    };
    prepare(
        case,
        |_| {},
        |doc| {
            set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case(id, vec![clause(payload.clone())])]),
            )
        },
    )
    .unwrap();
    let mut aliased = payload.clone();
    set(&mut aliased, "value", variable("exception", "caught"));
    let alias = J::object(vec![
        ("tag", J::string("let")),
        ("type_id", J::string(ty("bool"))),
        ("binding_id", J::string("caught")),
        ("value", variable("exception", "exception")),
        ("body", clause(aliased)),
    ]);
    prepare(
        case,
        |_| {},
        |doc| {
            set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case(id, vec![alias])]),
            )
        },
    )
    .unwrap();
    for mutation in 0..5 {
        let mut changed = payload.clone();
        match mutation {
            0 => set(&mut changed, "type_id", J::string(ty("string"))),
            1 => set(&mut changed, "member_id", J::string("Message")),
            2 => set(&mut changed, "value", variable("i32", "parameter:0")),
            4 => set(
                &mut changed,
                "value",
                literal(
                    "exception",
                    J::object(vec![
                        ("kind", J::string("closed_exception")),
                        ("type_id", J::string(ty("exception"))),
                        ("tag", J::U64(0)),
                        ("source_type_id", J::Null),
                        ("payload", J::Null),
                    ]),
                ),
            ),
            _ => set(
                &mut changed,
                "value",
                J::object(vec![
                    ("tag", J::string("old")),
                    ("type_id", J::string(ty("exception"))),
                    ("expression", variable("exception", "exception")),
                ]),
            ),
        };
        assert!(prepare(
            case,
            |_| {},
            |doc| set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case(id, vec![clause(changed)])])
            )
        )
        .is_err());
    }
    assert!(prepare(
        case,
        |_| {},
        |doc| set(doc, "ensures", J::Array(vec![is.clone()]))
    )
    .is_err());
}
#[test]
fn csharp_03_t04_w04_consumes_data_owner_successors_without_reconversion() {
    let cases = fixtures();
    let case = cases
        .iter()
        .find(|c| c["id"] == "argument_failure")
        .unwrap();
    let b = bundle();
    let (r, c) = exception_roots(&b, case);
    let prepared = prepare(
        case,
        |_| {},
        |doc| {
            set(
                doc,
                "exceptional_cases",
                J::Array(vec![exception_case("System.DivideByZeroException", vec![])]),
            )
        },
    )
    .unwrap();
    let signature = scalar_operation_signature("integer.i32.divide.checked").unwrap();
    let invocation = OperationInvocation {
        operation_id: signature.id.clone(),
        operands: signature
            .argument_type_ids
            .iter()
            .enumerate()
            .map(|(i, t)| TypedValueRef {
                id: format!("value.{i}"),
                type_id: t.clone(),
            })
            .collect(),
        result: TypedValueRef {
            id: "value.result".into(),
            type_id: signature.normal_result_type_id.clone(),
        },
        ordered_check_ids: signature
            .ordered_checks
            .iter()
            .map(|c| c.id.clone())
            .collect(),
        normal_successor_id: "normal".into(),
        exceptional_successors: signature
            .ordered_checks
            .iter()
            .filter(|c| c.tag == RequiredCheckTag::Exception)
            .enumerate()
            .map(|(i, c)| ExceptionalSuccessor {
                check_id: c.id.clone(),
                exception_type_id: c.failure_type_id.clone().unwrap(),
                target_id: format!("exception.{i}"),
            })
            .collect(),
    };
    let results = prepared
        .operation_exception_results(
            &r,
            &c,
            case["root"].as_str().unwrap(),
            &signature,
            &invocation,
        )
        .unwrap();
    assert_eq!(
        results
            .iter()
            .map(|r| r.successor.clone())
            .collect::<Vec<_>>(),
        invocation.exceptional_successors
    );
    assert!(results[0].declared && !results[0].catch_or_unreachable);
    assert!(!results[1].declared && results[1].catch_or_unreachable);
    let mut changed = invocation.clone();
    changed.exceptional_successors[0].exception_type_id = "System.OutOfMemoryException".into();
    assert!(prepared
        .operation_exception_results(&r, &c, case["root"].as_str().unwrap(), &signature, &changed)
        .is_err());
    changed = invocation.clone();
    changed.exceptional_successors.reverse();
    assert!(prepared
        .operation_exception_results(&r, &c, case["root"].as_str().unwrap(), &signature, &changed)
        .is_err());
}
#[test]
fn csharp_03_t04_w04_rejections_reach_the_intended_gate() {
    let cases = fixtures();
    assert_eq!(cases.len(), 39);
    for (id, code) in [
        ("message", "exception_throw_shape"),
        ("expression", "exception_throw_expression"),
        ("oom", "exception_throw_shape"),
        ("stack", "exception_throw_shape"),
        ("unsealed", "exception_base"),
        ("wrong_base", "exception_base"),
        ("mutable", "data_field"),
        ("observe", "exception_value_api"),
        ("catch", "exception_handler_later_owner"),
    ] {
        let case = cases.iter().find(|c| c["id"] == id).unwrap();
        assert_eq!(case["accepted"], false);
        assert_eq!(case["code"], code);
    }
}

#[test]
fn csharp_03_t04_w04_pinned_conformance() {
    let manifest: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/exception-lowering/exception-lowering-inputs.json",
    ))
    .unwrap();
    for file in manifest["files"].as_array().unwrap() {
        let bytes = read(file["path"].as_str().unwrap());
        assert_eq!(bytes.len() as u64, file["size_bytes"].as_u64().unwrap());
        assert_eq!(
            mpk_vc::hash::sha256_raw_file_bytes(&bytes).to_hex(),
            file["sha256"]
        );
    }
    let conformance: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/exception-lowering/conformance.json",
    ))
    .unwrap();
    for key in ["frozen_probe", "source_cases"] {
        let bytes = read(conformance[key]["path"].as_str().unwrap());
        assert_eq!(
            mpk_vc::hash::sha256_raw_file_bytes(&bytes).to_hex(),
            conformance[key]["sha256"]
        );
    }
    let cases = fixtures();
    assert_eq!(
        conformance["original_clr_executions"].as_u64().unwrap(),
        cases
            .iter()
            .map(|c| c["runs"].as_array().unwrap().len() as u64)
            .sum::<u64>()
    );
    assert_eq!(conformance["cases"].as_array().unwrap().len(), cases.len());
    for (case, record) in cases.iter().zip(conformance["cases"].as_array().unwrap()) {
        assert_eq!(case["id"], record["id"]);
        assert_eq!(case["accepted"], record["accepted"]);
        assert_eq!(
            mpk_vc::hash::sha256_raw_file_bytes(case["source"].as_str().unwrap().as_bytes())
                .to_hex(),
            record["source_sha256"]
        );
        if case["accepted"] == true {
            assert_eq!(
                mpk_vc::hash::sha256_raw_file_bytes(
                    &serde_json::to_vec(&case["lowering"]["functions"]).unwrap()
                )
                .to_hex(),
                record["graph_sha256"]
            );
        }
    }
}
#[test]
fn csharp_03_t04_w04_does_not_extend_legacy_wire_by_null_field() {
    let cases: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/pattern-lowering/source-cases.json",
    ))
    .unwrap();
    let case = cases.iter().find(|c| c["id"] == "constant").unwrap();
    let b = bundle();
    let (r, c) = roots(&b);
    let mut source = case.clone();
    source["facts"] = case["lowering"]["facts"].clone();
    let (ctx, captures) = context_support::context_with_sidecar(
        &b,
        case["root"].as_str().unwrap(),
        case["source"].as_str().unwrap().as_bytes(),
        |ctx| hashed(document(ctx, &source, "total", rows(&source))),
    );
    let mut wire = case["lowering"].clone();
    let facts = serde_json::to_vec(&source["facts"]).unwrap();
    let check = |wire: &Value| {
        prepare_pattern_lowering(
            &b,
            &r,
            &c,
            &ctx,
            &captures,
            &facts,
            &serde_json::to_vec(wire).unwrap(),
            &DataContractEnvironment::default(),
            &BTreeSet::new(),
        )
    };
    check(&wire).unwrap();
    wire["exception_definitions"] = Value::Null;
    assert_eq!(check(&wire).unwrap_err(), LoopLoweringError::Source);
}
#[test]
fn csharp_03_t04_w04_hostile_source_tree_counts_reject_without_overflow() {
    let cases = fixtures();
    let case = cases.iter().find(|c| c["id"] == "payload").unwrap();
    assert_eq!(
        prepare(
            case,
            |wire| {
                let function = &mut wire["functions"][0];
                let ops = function["operations"].as_array_mut().unwrap();
                let throw = ops.iter().position(|o| o["kind"] == "Throw").unwrap();
                ops[throw]["child_count"] = json!(u64::MAX);
                ops[throw + 1]["child_count"] = json!(u64::MAX);
                let body = serde_json::to_string(&function["operations"]).unwrap();
                let id = function["callable_id"].clone();
                let mut syntax: Value =
                    serde_json::from_str(wire["normalized_syntax_utf8"].as_str().unwrap()).unwrap();
                let callable = syntax["callables"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| c["id"] == id)
                    .unwrap();
                callable["body_sha256"] =
                    json!(mpk_vc::hash::sha256_raw_file_bytes(body.as_bytes()).to_hex());
                callable["body"] = json!(body);
                let text = serde_json::to_string(&syntax).unwrap();
                let hash = mpk_vc::hash::sha256_raw_file_bytes(text.as_bytes()).to_hex();
                wire["normalized_syntax_utf8"] = json!(text);
                wire["normalized_syntax_sha256"] = json!(hash);
                wire["sequence_handoff"]["source"] = json!(hash);
            },
            |_| {}
        )
        .unwrap_err(),
        LoopLoweringError::Operand
    );
}
