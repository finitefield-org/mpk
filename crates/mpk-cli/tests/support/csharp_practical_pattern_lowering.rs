//! W03 source decision graphs, native attachment and independent execution.
use super::*;
use std::collections::BTreeSet;
fn fixtures() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/pattern-lowering/source-cases.json",
    ))
    .unwrap()
}
fn pattern_roots(
    b: &ValidatedFoundationBundle,
    case: &Value,
) -> (ValidatedClosedRootSet, ClosedInstanceSet) {
    let primitive = |id: &str| json!({"kind":"primitive","id":id});
    let array =
        json!({"kind":"instance","template":"bounded_sequence","arguments":[primitive("i32")]});
    let option = |t: Value| json!({"kind":"instance","template":"option","arguments":[t]});
    let mut types = vec![
        array.clone(),
        option(array),
        option(primitive("i32")),
        option(primitive("string")),
    ];
    let mut sources = serde_json::Map::new();
    for (name, kind) in [("Box", "sealed_class"), ("Kind", "enum")] {
        if !case["source"]
            .as_str()
            .unwrap()
            .contains(&format!("{}{{", name))
        {
            continue;
        }
        let identity = json!({"kind":"type","namespace":"Business","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""});
        let id = csharp_practical_declaration_id(&identity).unwrap();
        let member =
            csharp_practical_stored_member_id(&id, "Stored", &primitive("i32"), "readonly_field")
                .unwrap();
        let descriptor = json!({"id":id,"identity":identity,"kind":kind,
            "members":if kind=="enum" {vec![]} else {vec![json!({"id":member,"name":"Stored","type":primitive("i32"),"storage":"readonly_field","ordinal":0,"required":false})]},
            "enum_values":if kind=="enum" {vec!["0","1"]} else {vec![]},
            "enum_underlying":if kind=="enum" {json!("i32")}else{Value::Null},
            "actual_default":if kind=="enum" {json!({})}else{json!({member:0})},
            "public_default":kind=="enum","identity_sensitive":false,
            "source_sha256":case["lowering"]["facts"]["sources"][0]["raw_sha256"]});
        sources.insert(id.clone(), descriptor);
        types.push(option(json!({"kind":"source","id":id})));
    }
    let roots=Value::Array(types.into_iter().enumerate().map(|(i,t)|json!({"origin":"semantic_binding","provenance_id":format!("pattern.type.{i}"),"type":t})).collect());
    let bytes = canonical_closed_root_set_transport(b, &roots, &json!(sources)).unwrap();
    let r = validate_closed_root_set(b, &bytes).unwrap();
    let c = derive_closed_instances(b, &r).unwrap();
    (r, c)
}
fn prepare(
    case: &Value,
    mutation: impl FnOnce(&mut Value),
) -> Result<LoweredLoopControl, LoopLoweringError> {
    let b = bundle();
    let (r, c) = pattern_roots(&b, case);
    let mut source = case.clone();
    source["facts"] = case["lowering"]["facts"].clone();
    let (context, captures) = context_support::context_with_sidecar(
        &b,
        case["root"].as_str().unwrap(),
        case["source"].as_str().unwrap().as_bytes(),
        |ctx| hashed(document(ctx, &source, "total", rows(&source))),
    );
    let claims =
        serde_json::from_value::<BTreeSet<String>>(case["lowering"]["total_getters"].clone())
            .unwrap();
    let mut wire = case["lowering"].clone();
    mutation(&mut wire);
    prepare_pattern_lowering(
        &b,
        &r,
        &c,
        &context,
        &captures,
        &serde_json::to_vec(&source["facts"]).unwrap(),
        &serde_json::to_vec(&wire).unwrap(),
        &DataContractEnvironment::default(),
        &claims,
    )
}
#[test]
fn csharp_03_t04_w03_decision_graphs_match_original_clr() {
    let mut count = 0;
    assert!(fixtures().iter().any(|c| c["id"] == "array_arm"));
    for case in fixtures().iter().filter(|c| c["accepted"] == true) {
        let prepared = prepare(case, |_| {}).unwrap_or_else(|e| panic!("{}: {e:?}", case["id"]));
        assert_eq!(prepared.artifact_count(), 0);
        let function = prepared
            .functions()
            .iter()
            .find(|f| f.callable_id == case["root"].as_str().unwrap())
            .unwrap();
        if case["id"] == "array_arm" {
            let freezes = prepared.sequence_handoff()["steps"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|s| s["operation"] == "alias_freeze")
                .collect::<Vec<_>>();
            assert_eq!(
                freezes
                    .iter()
                    .flat_map(|s| s["arrays"].as_array().unwrap())
                    .collect::<Vec<_>>()
                    .len(),
                2
            );
            for freeze in freezes {
                assert_eq!(freeze["action"], "freeze");
                assert!(freeze["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|t| t["ConstructionTypeId"] != ""));
            }
        }
        for run in case["runs"].as_array().unwrap() {
            let actual = lowering::interpret(function, run);
            let expected = if let Some(value) = run["value"].as_i64() {
                Ok(value)
            } else {
                Err(run["error"].as_str().unwrap().to_owned())
            };
            assert_eq!(actual, expected, "{}: {run}", case["id"]);
            count += 1;
        }
    }
    assert!(count >= 800, "{count}");
}
#[test]
fn csharp_03_t04_w03_unmatched_path_and_claim_mutations_reject() {
    let cases = fixtures();
    for (id, opcode) in [
        ("relational", "pattern_relational"),
        ("list", "pattern_length"),
    ] {
        let case = cases.iter().find(|c| c["id"] == id).unwrap();
        assert_eq!(
            prepare(case, |w| {
                let n = w["functions"][0]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["operation"] == opcode)
                    .unwrap();
                n["operation"] = json!("pattern_true");
            })
            .unwrap_err(),
            LoopLoweringError::Operand
        );
    }
    let case = cases.iter().find(|c| c["id"] == "nonexhaustive").unwrap();
    assert_eq!(
        prepare(case, |w| {
            let n = w["functions"][0]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["kind"] == "throw" && n["slot"] != "")
                .unwrap();
            n["slot"] = json!("System.InvalidOperationException");
        })
        .unwrap_err(),
        LoopLoweringError::Graph
    );
    assert_eq!(
        prepare(case, |w| {
            let n = w["functions"][0]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["kind"] == "throw" && n["slot"] != "")
                .unwrap();
            n["slot"] = json!("");
        })
        .unwrap_err(),
        LoopLoweringError::Graph
    );
    let list = cases.iter().find(|c| c["id"] == "list").unwrap();
    assert_eq!(
        prepare(list, |w| {
            let n = w["functions"][0]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["operation"] == "pattern_length")
                .unwrap();
            n["slot"] = json!("42");
        })
        .unwrap_err(),
        LoopLoweringError::Operand
    );
    let case = cases.iter().find(|c| c["id"] == "property_claim").unwrap();
    let p = prepare(case, |_| {}).unwrap();
    assert_eq!(p.total_getters().len(), 1);
    assert_eq!(
        prepare(case, |w| w["total_getters"] = json!([])).unwrap_err(),
        LoopLoweringError::Source
    );
}
#[test]
fn csharp_03_t04_w03_private_pins_and_rejections_are_retained() {
    let manifest: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/pattern-lowering/pattern-lowering-inputs.json",
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
        "develop/migrations/csharp-03/pattern-lowering/conformance.json",
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
    for id in [
        "slice",
        "positional",
        "goto_case",
        "goto_default",
        "fallthrough",
        "overlap",
        "binding_scope",
        "identity",
        "unsupported_list",
        "getter_missing",
        "list_guard_write",
        "list_statement_guard_write",
    ] {
        let case = cases.iter().find(|c| c["id"] == id).unwrap();
        assert_eq!(case["accepted"], false, "{id}");
        assert!(!case["diagnostic"].as_str().unwrap().is_empty());
    }
    for id in ["list_guard_write", "list_statement_guard_write"] {
        let case = cases.iter().find(|c| c["id"] == id).unwrap();
        assert_eq!(
            case["code"], "active_foreach_read_borrow",
            "{id}: must reach the read-borrow barrier"
        );
    }
    let project = String::from_utf8(read("csharp-tools/csharp2vir/CSharp2Vir.csproj")).unwrap();
    assert!(!project.contains("PracticalPatternLowering"));
}
