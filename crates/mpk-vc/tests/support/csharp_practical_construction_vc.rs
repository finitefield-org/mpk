//! T06-W02 actual-source construction/publication VCs and hostile handoffs.
use crate::support;
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::{fs, path::Path};
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn read(path: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03")
                .join(path),
        )
        .unwrap(),
    )
    .unwrap()
}
// Preserve the original schema ordering while editing values. Assert an
// unchanged round trip first so rejection cannot be due to reordered JSON.
pub(super) fn edited_bytes(original: &[u8], changed: &Value) -> Vec<u8> {
    use mpk_vc::csharp_practical_source_artifacts::PracticalJsonValue as J;
    fn order(shape: &J, v: &Value) -> J {
        if let Some(form) = v.get("form").and_then(Value::as_str) {
            let keys: &[&str] = match form {
                "var" => &["form", "index", "type_id"],
                "const" => &["form", "name", "type_id"],
                "app" => &["form", "function", "argument", "type_id"],
                "lam" => &["form", "parameter_type", "body", "type_id"],
                "let" => &["form", "value", "body", "type_id"],
                _ => panic!("unknown test term"),
            };
            return J::Object(
                keys.iter()
                    .map(|k| ((*k).into(), order(shape.get(k).unwrap_or(&J::Null), &v[k])))
                    .collect(),
            );
        }
        match (shape, v) {
            (J::Object(fields), Value::Object(values)) => {
                let mut result = fields
                    .iter()
                    .filter_map(|(k, s)| values.get(k).map(|v| (k.clone(), order(s, v))))
                    .collect::<Vec<_>>();
                result.extend(
                    values
                        .iter()
                        .filter(|(k, _)| !fields.iter().any(|(f, _)| f == *k))
                        .map(|(k, v)| (k.clone(), serde_json::from_value(v.clone()).unwrap())),
                );
                J::Object(result)
            }
            (J::Array(shapes), Value::Array(values)) => J::Array(
                values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.get("id")
                            .and_then(Value::as_str)
                            .and_then(|id| {
                                shapes
                                    .iter()
                                    .find(|s| s.get("id").and_then(J::as_str) == Some(id))
                            })
                            .or_else(|| shapes.get(i))
                            .map(|s| order(s, v))
                            .unwrap_or_else(|| serde_json::from_value(v.clone()).unwrap())
                    })
                    .collect(),
            ),
            _ => serde_json::from_value(v.clone()).unwrap(),
        }
    }
    mpk_vc::csharp_practical_source_artifacts::canonical_practical_json_bytes(&order(
        &serde_json::from_slice::<J>(original).unwrap(),
        changed,
    ))
    .unwrap()
}
pub(super) fn assert_typed(term: &ContractTerm, binders: &[String]) {
    match term {
        ContractTerm::Var { index, type_id } => assert_eq!(type_id, &binders[*index]),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => {
            assert_typed(function, binders);
            assert_typed(argument, binders);
            assert_eq!(
                function.type_id(),
                format!("({}->{type_id})", argument.type_id())
            );
        }
        ContractTerm::Lam {
            parameter_type,
            body,
            type_id,
        } => {
            let mut nested = vec![parameter_type.clone()];
            nested.extend_from_slice(binders);
            assert_typed(body, &nested);
            assert_eq!(type_id, &format!("({parameter_type}->{})", body.type_id()));
        }
        ContractTerm::Let {
            value,
            body,
            type_id,
        } => {
            assert_typed(value, binders);
            let mut nested = vec![value.type_id().into()];
            nested.extend_from_slice(binders);
            assert_typed(body, &nested);
            assert_eq!(type_id, body.type_id());
        }
        ContractTerm::Const { .. } => {}
    }
}
// Small test oracle for these field/integer/boolean clauses. It evaluates the
// generated ordinary term and definition table, not the source/parser AST.
fn eval(term: &ContractTerm, definitions: &[ContractDefinition], value: &Value) -> Value {
    if let ContractTerm::Var { index: 0, .. } = term {
        return value.clone();
    }
    let mut args = vec![];
    let mut head = term;
    while let ContractTerm::App {
        function, argument, ..
    } = head
    {
        args.push(eval(argument, definitions, value));
        head = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = head else {
        panic!("unexpected term {head:?}");
    };
    if name.starts_with("Mpk.CSharp.FinalizedSnapshot.")
        || name.starts_with("Mpk.CSharp.AssignedSnapshot.")
    {
        return args[0].clone();
    }
    if let Some(member) = name.strip_prefix("Mpk.CSharp.SlotRead.") {
        return args[0][member].clone();
    }
    let d = definitions.iter().find(|d| d.name == *name).unwrap();
    let p: Value = serde_json::from_str(&d.parameters).unwrap();
    match d.tag.as_str() {
        "literal" => p["value"].clone(),
        "field" => args[0][p["member_id"].as_str().unwrap()].clone(),
        "binary" => {
            let a = args[0].as_str().unwrap().parse::<i32>().unwrap();
            let b = args[1].as_str().unwrap().parse::<i32>().unwrap();
            json!(match p["operation_id"].as_str().unwrap() {
                "integer.i32.greater.checked" => a > b,
                "integer.i32.greater_equal.checked" => a >= b,
                x => panic!("unexpected operation {x}"),
            })
        }
        x => panic!("unexpected definition {x}"),
    }
}
#[test]
fn csharp_03_t06_w02_actual_source_goldens_and_failing_conditions() {
    let b = bundle();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let mut golden = vec![];
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        if id == "missing_required" || id == "enum_no_zero" {
            assert!(response.get("reject").is_some(), "{response}");
            continue;
        }
        assert!(response.get("facts").is_some(), "{id}: {response}");
        let (ctx, captures) = support::replay_context(&b, request);
        let imported = ValidatedDataSource::import_captured_facts(
            &b,
            &ctx,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        );
        if id == "enum_no_zero" {
            assert!(imported
                .and_then(|s| emit_data_phase(&b, &ctx, &captures, &s))
                .is_err());
            continue;
        }
        let source = imported.unwrap();
        let emitted =
            emit_data_phase(&b, &ctx, &captures, &source).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let source = PracticalVcSource {
            artifact_context: &ctx,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(source).unwrap();
        let p = vc.construction_vcs();
        assert_eq!(
            &import_csharp_practical_construction_vcs(&p.canonical_bytes(), source).unwrap(),
            p
        );
        assert_eq!(
            import_csharp_practical_vc_json(vc.canonical_bytes(), source)
                .unwrap()
                .construction_vcs(),
            p
        );
        for t in p.types() {
            assert_typed(&t.public_body, std::slice::from_ref(&t.type_id));
        }
        for goal in p
            .sequents()
            .iter()
            .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        {
            assert_typed(&goal.term, std::slice::from_ref(&goal.subject.type_id));
        }
        let expected_nodes = super::data::nodes(vc.data_vcs())
            + vc.type_encodings().len()
            + vc.operation_encodings().len()
            + vc.control_encodings().len()
            + vc.obligation_groups().len()
            + vc.obligation_groups()
                .iter()
                .map(|g| g.subject_ids().len())
                .sum::<usize>()
            + vc.contract_expressions()
                .iter()
                .map(|e| e.term().nodes())
                .sum::<usize>()
            + p.types()
                .iter()
                .map(|t| t.public_body.nodes())
                .sum::<usize>()
            + p.sequents()
                .iter()
                .flat_map(|s| s.assumptions.iter().chain(&s.goals))
                .map(|g| g.term.nodes())
                .sum::<usize>();
        assert_eq!(
            vc.resource_reservation().ordinary_term_nodes_minimum(),
            expected_nodes as u64
        );
        let names = vc
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions().iter().map(|d| &d.name))
            .chain(p.definition_names())
            .chain(vc.data_vcs().definition_names())
            .collect::<std::collections::BTreeSet<_>>();
        let declarations = vc.type_encodings().len()
            + vc.operation_encodings().len()
            + vc.obligation_groups().len()
            + names.len()
            + p.sequents().len()
            + vc.data_vcs()
                .operations()
                .iter()
                .map(|o| 1 + o.checks.len())
                .sum::<usize>()
            + vc.data_vcs().ownership().len()
            + vc.data_vcs().contracts().len();
        assert_eq!(
            vc.resource_reservation().generated_declarations_minimum(),
            declarations as u64
        );
        let t = &p.types()[0];
        if id == "enum_zero" {
            assert_eq!(t.enum_values.as_ref().unwrap(), &["0", "1"]);
            assert!(p.sequents().iter().any(|s| s.kind == "literal_domain"));
        } else {
            let amount = t
                .members
                .iter()
                .find(|m| m.type_id == "mpk.csharp.value.i32.v1")
                .unwrap();
            let expected = if id == "broken_constructor" {
                -1
            } else if id.ends_with("default") || id == "broken_initializer" {
                0
            } else {
                1
            };
            let value = json!({amount.id.clone():expected.to_string()});
            let clause = &t.public_clauses[0];
            let expected_truth = !id.starts_with("broken");
            assert_eq!(
                eval(clause.term(), clause.definitions(), &value),
                json!(expected_truth),
                "{id}"
            );
            let kind = if id.contains("initializer") {
                "publication"
            } else if id.ends_with("default") {
                "literal_domain"
            } else {
                "public_return"
            };
            let goals = p
                .sequents()
                .iter()
                .filter(|s| s.kind == kind)
                .flat_map(|s| &s.goals)
                .filter(|g| g.attachment_sha256.as_deref() == Some(clause.attachment_sha256()))
                .collect::<Vec<_>>();
            assert!(!goals.is_empty(), "{id}: missing {kind}");
            for g in goals {
                assert_eq!(
                    eval(&g.term, clause.definitions(), &value),
                    json!(expected_truth)
                );
            }
            let entry = p
                .sequents()
                .iter()
                .flat_map(|s| &s.assumptions)
                .find(|g| g.attachment_sha256.as_deref() == Some(clause.attachment_sha256()))
                .unwrap();
            let preserved = p
                .sequents()
                .iter()
                .filter(|s| s.kind == "public_operation_preservation")
                .flat_map(|s| &s.goals)
                .find(|g| {
                    g.subject == entry.subject && g.attachment_sha256 == entry.attachment_sha256
                })
                .unwrap();
            assert_eq!(preserved.term, entry.term);
            assert_eq!(
                eval(
                    &preserved.term,
                    clause.definitions(),
                    &json!({amount.id.clone():"-1"})
                ),
                json!(false)
            );
            if id.contains("initializer") {
                let ctor = p
                    .sequents()
                    .iter()
                    .find(|s| s.kind == "construction_invariant")
                    .unwrap();
                assert!(!ctor
                    .goals
                    .iter()
                    .any(|g| g.attachment_sha256.as_deref() == Some(clause.attachment_sha256())));
                let c = t.construction_clause.as_ref().unwrap();
                assert!(ctor
                    .goals
                    .iter()
                    .any(|g| g.attachment_sha256.as_deref() == Some(c.attachment_sha256())));
                let publication = p
                    .sequents()
                    .iter()
                    .find(|s| s.kind == "publication")
                    .unwrap();
                assert!(publication.assumptions.is_empty());
                assert_eq!(publication.point, "before_operation");
                assert_eq!(
                    publication.structural_evidence[0].definitely_assigned,
                    if id == "positive_initializer" { 3 } else { 1 }
                );
            }
        }
        assert!(
            vc.resource_reservation().ordinary_term_nodes_minimum()
                >= p.sequents()
                    .iter()
                    .flat_map(|s| s.assumptions.iter().chain(&s.goals))
                    .map(|g| g.term.nodes() as u64)
                    .sum()
        );
        for row in p.sequents() {
            let f = emitted
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == row.function_id)
                .unwrap();
            let block = f.blocks.iter().find(|b| b.node.id == row.node_id).unwrap();
            if !row.assumptions.is_empty() {
                assert_eq!(row.point, "entry");
            }
            for g in &row.goals {
                if row.kind == "publication" {
                    assert_eq!(
                        g.subject.value_id,
                        block.invocation.as_ref().unwrap().operands[0].id
                    );
                    assert_eq!(g.subject.view, "private_members");
                    assert_eq!(
                        g.subject.type_id,
                        block.invocation.as_ref().unwrap().operands[0].type_id
                    );
                    assert!(g.subject.source_type_id.is_some());
                }
                if row.kind == "public_return" || row.kind == "construction_invariant" {
                    assert!(block.return_value_ids.contains(&g.subject.value_id));
                }
                if row.kind == "literal_domain" {
                    assert!(block
                        .literal_values
                        .iter()
                        .any(|v| v.result.id == g.subject.value_id
                            && v.result.type_id == g.subject.type_id));
                }
            }
            assert!(vc.obligation_groups().iter().any(|g| g.proof_owner()
                == LaterProofOwner::ConstructionAndTypeInvariants
                && g.subject_ids().contains(&row.id)));
        }
        let skeleton = emit_csharp_practical_vc_skeleton(source, &vc).unwrap();
        let sk: Value = serde_json::from_slice(skeleton.canonical_bytes()).unwrap();
        for group in vc.obligation_groups().iter().filter(|g| {
            g.proof_owner() == LaterProofOwner::ConstructionAndTypeInvariants
                && g.function_id().is_some()
        }) {
            assert!(group
                .dependencies()
                .iter()
                .any(|d| d == "vc.group.0200.global"));
            assert!(sk["theorem_declarations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["group_id"] == group.id()
                    && d["subject_ids"] == json!(group.subject_ids())));
        }
        golden.push(json!({"id":id,"vir_sha256":emitted.vir().hash(),"vc_sha256":vc.hash(),"construction":p}));
    }
    if let Ok(path) = std::env::var("MPK_T06_W02_GOLDEN_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&golden).unwrap()).unwrap();
    } else {
        assert_eq!(json!(golden), read("construction-vc/goldens.json"));
    }
}
#[test]
fn csharp_03_t06_w02_hostile_construction_and_preservation_handoffs() {
    let b = bundle();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "positive_initializer")
        .unwrap();
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == request["id"])
        .unwrap();
    let (ctx, captures) = support::replay_context(&b, request);
    let captured = ValidatedDataSource::import_captured_facts(
        &b,
        &ctx,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&b, &ctx, &captures, &captured).unwrap();
    let source = PracticalVcSource {
        artifact_context: &ctx,
        captured_inputs: &captures,
        vir: emitted.vir(),
    };
    let vc = generate_csharp_practical_vc(source).unwrap();
    let original_bytes = vc.construction_vcs().canonical_bytes();
    let original: Value = serde_json::from_slice(&vc.construction_vcs().canonical_bytes()).unwrap();
    assert_eq!(edited_bytes(&original_bytes, &original), original_bytes);
    assert!(import_csharp_practical_construction_vcs(
        &edited_bytes(&original_bytes, &original),
        source
    )
    .is_ok());
    for kind in [
        "publication",
        "construction_invariant",
        "public_operation_preservation",
        "member_assignment",
    ] {
        let mut bad = original.clone();
        let rows = bad["sequents"].as_array_mut().unwrap();
        let pos = rows.iter().position(|r| r["kind"] == kind).unwrap();
        rows.remove(pos);
        assert!(import_csharp_practical_construction_vcs(
            &edited_bytes(&original_bytes, &bad),
            source
        )
        .is_err());
    }
    for mutation in 0..7 {
        let mut bad = original.clone();
        let t = &mut bad["types"][0];
        match mutation {
            0 => t["structural_default"] = json!(true),
            1 => t["members"][0]["required"] = json!(false),
            2 => t["public_clauses"] = json!([]),
            3 => {
                bad["sequents"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|s| !s["assumptions"].as_array().unwrap().is_empty())
                    .unwrap()["assumptions"] = json!([])
            }
            4 => bad["sequents"][0]["node_id"] = json!("forged"),
            5 => {
                bad["sequents"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|s| s["kind"] == "public_operation_preservation")
                    .unwrap()["goals"][0]["term"] = json!({"form":"const","name":"Mpk.CSharp.Bool.true","type_id":"mpk.csharp.value.bool.v1"})
            }
            _ => bad["closed_roots_sha256"] = json!("0".repeat(64)),
        }
        assert_ne!(bad, original);
        assert!(import_csharp_practical_construction_vcs(
            &edited_bytes(&original_bytes, &bad),
            source
        )
        .is_err());
    }
    let id = vc
        .construction_vcs()
        .sequents()
        .iter()
        .find(|s| s.kind == "publication")
        .unwrap()
        .id
        .clone();
    let transport = String::from_utf8(vc.canonical_bytes().to_vec()).unwrap();
    let bad = transport.replace(&id, &format!("{id}forged"));
    assert_eq!(
        import_csharp_practical_vc_json(bad.as_bytes(), source)
            .err()
            .unwrap()
            .phase(),
        PracticalVcValidationPhase::Obligations
    );
}
#[test]
fn csharp_03_t06_w02_existing_initialization_protocols() {
    let b = bundle();
    let requests = read("data-phase/object-construction-requests.json");
    let responses = read("data-phase/object-construction-responses.json");
    for row in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == row["id"])
            .unwrap();
        let (ctx, captures) = support::replay_context(&b, row);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &ctx,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &ctx, &captures, &source).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &ctx,
            captured_inputs: &captures,
            vir: emitted.vir(),
        })
        .unwrap();
        assert!(
            vc.construction_vcs()
                .sequents()
                .iter()
                .any(|s| s.kind == "publication"),
            "{}",
            row["id"]
        );
        for f in emitted.vir().functions() {
            if let Some(protocol) = &f.object_protocol {
                for d in &protocol.exceptional_discards {
                    assert!(vc
                        .construction_vcs()
                        .sequents()
                        .iter()
                        .any(|s| s.function_id == f.id
                            && s.node_id == d.exit_node_id
                            && s.kind == "discard_without_publication"
                            && s.goals.is_empty()));
                }
            }
        }
    }
}
