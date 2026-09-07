//! W06: actual-source data/control closure and ordinary VIR integration.
use super::*;
use mpk_vc::hash::sha256_raw_file_bytes;
#[path = "csharp_practical_control_execution.rs"]
mod execution;

fn fixtures() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/source-cases.json",
    ))
    .unwrap()
}

#[test]
fn csharp_03_t04_w06_actual_source_data_control_capture() {
    let b = bundle();
    let mut accepted = 0;
    let mut failures = Vec::new();
    for case in fixtures() {
        let source = &case["source_case"];
        if case["accepted"] != true {
            assert_eq!(case["artifact_count"], 0);
            assert!(case["diagnostic"].as_str().is_some_and(|d| !d.is_empty()));
            assert!(case["data"].is_null());
            continue;
        }
        accepted += 1;
        if case["diagnostic"] != "" {
            failures.push(format!(
                "{}:{}:{}",
                case["stage"], source["id"], case["diagnostic"]
            ));
            continue;
        }
        let (context, captures) = context_support::context(
            &b,
            source["root"].as_str().unwrap(),
            source["source"].as_str().unwrap().as_bytes(),
        );
        let bytes = serde_json::to_vec(&case["data"]).unwrap();
        match ValidatedDataSource::import_captured_facts(&b, &context, &captures, &bytes) {
            Ok(data) => {
                assert!(data.control_lowering().is_some());
                if let Err(e) = validate_control_source(&b, &data) {
                    failures.push(format!("control:{}:{}:{e:?}", case["stage"], source["id"]));
                }
            }
            Err(e) => failures.push(format!("{}:{}:{e:?}", case["stage"], source["id"])),
        }
    }
    assert_eq!(accepted, 111);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_pruned_ssa_for_actual_control_paths() {
    use std::collections::BTreeMap;
    let b = bundle();
    let mut failures = Vec::new();
    for case in fixtures() {
        if case["data"].is_null() {
            continue;
        }
        let source = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            source["root"].as_str().unwrap(),
            source["source"].as_str().unwrap().as_bytes(),
        );
        let bytes = serde_json::to_vec(&case["data"]).unwrap();
        let data = match ValidatedDataSource::import_captured_facts(&b, &context, &captures, &bytes)
        {
            Ok(s) => s,
            Err(_) => continue,
        };
        let control = match validate_control_source(&b, &data) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (function, handler) in control.functions().iter().zip(control.handlers()) {
            let method = control.facts()["methods"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["callable_id"] == function.callable_id)
                .unwrap();
            let parameters = method["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    let slot = p["id"].as_str().unwrap();
                    (
                        slot.into(),
                        format!("{}.parameter.{slot}", function.callable_id),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let caught = handler
                .graph()
                .regions
                .iter()
                .flat_map(|r| &r.catches)
                .filter(|c| !c.local.is_empty())
                .flat_map(|c| {
                    std::iter::once(&c.entry)
                        .chain(c.filter.as_ref())
                        .map(|id| (id.clone(), (c.local.clone(), format!("{id}.exception"))))
                })
                .collect();
            let graph = handler_control_graph(function, handler, control.universe()).unwrap();
            if let Err(e) = derive_control_ssa(&graph, &parameters, &caught) {
                failures.push(format!(
                    "{}:{}:{}:{e:?}",
                    case["stage"], source["id"], function.callable_id
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_importer_checks_construction_loop_fixed_point() {
    use mpk_vc::csharp_practical_vir_validation as v;
    let b = bundle();
    let cases: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/data-phase/data-source-cases.json",
    ))
    .unwrap();
    let case = cases
        .iter()
        .find(|c| c["name"] == "array_dynamic_read")
        .unwrap();
    let facts = &case["facts"];
    let root = facts["selected_root_ids"][0].as_str().unwrap();
    let (context, captures) =
        context_support::context(&b, root, case["source_utf8"].as_str().unwrap().as_bytes());
    let source = ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: &context,
        captured_inputs: &captures,
        foundation_descriptor_transport: registered_foundation_descriptor_transport(),
        foundation_definitions_transport: registered_foundation_definitions_transport(),
        closed_roots_transport: emitted.closure().roots().canonical_json(),
        closed_instances_transport: emitted.closure().closed().canonical_json(),
        semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
        required_checks_transport: emitted.operations().required_checks().canonical_bytes(),
        operations_transport: emitted.operations().operations().canonical_bytes(),
    };
    let mut functions = emitted.vir().functions().to_vec();
    let f = functions.iter_mut().find(|f| f.id == root).unwrap();
    let read = f
        .blocks
        .iter()
        .position(|b| {
            b.invocation
                .as_ref()
                .is_some_and(|i| i.operation_id.ends_with(".read"))
        })
        .unwrap();
    let allocation = f.blocks[read].invocation.as_ref().unwrap().operands[0].clone();
    let target = f.blocks[read].node.id.clone();
    let before = f
        .blocks
        .iter()
        .position(|b| b.node.normal_successor_ids.contains(&target))
        .unwrap();
    let ordinal = f.blocks.len();
    let predecessor = format!("{}.node.{:06}", f.id, ordinal + 2);
    let header_id = format!("{}.node.{ordinal:06}", f.id);
    let body_id = format!("{}.node.{:06}", f.id, ordinal + 1);
    let loop_id = format!("{}#loop#0000", f.id);
    let phi = TypedValueRef {
        id: format!("{}.loop_owner", f.id),
        type_id: allocation.type_id.clone(),
    };
    let condition = TypedValueRef {
        id: format!("{}.loop_condition", f.id),
        type_id: "mpk.csharp.value.bool.v1".into(),
    };
    // This synthetic control insertion isolates the independent importer's
    // ownership fixed point. Actual-source control emission has separate tests.
    for block in &mut f.blocks {
        if let Some(call) = &mut block.invocation {
            for operand in &mut call.operands {
                if operand.id == allocation.id {
                    *operand = phi.clone();
                }
            }
        }
        if block.node.tag != ControlNodeTag::Exit {
            for action in &mut block.construction_actions {
                if let v::PracticalConstructionAction::Discard {
                    construction_id, ..
                } = action
                {
                    if *construction_id == allocation.id {
                        *construction_id = phi.id.clone();
                    }
                }
            }
        }
    }
    f.blocks[before].node.normal_successor_ids = vec![predecessor.clone()];
    if let Some(call) = &mut f.blocks[before].invocation {
        call.normal_successor_id = predecessor.clone();
    }
    let mut header = f
        .blocks
        .iter()
        .find(|b| b.node.tag == ControlNodeTag::Entry)
        .unwrap()
        .clone();
    header.node = ControlNode {
        id: header_id.clone(),
        ordinal: ordinal as u32,
        tag: ControlNodeTag::LoopHeader,
        condition_type_id: Some(condition.type_id.clone()),
        normal_successor_ids: vec![body_id.clone(), target.clone()],
        exceptional_successors: vec![],
        abrupt: None,
        loop_id: Some(loop_id.clone()),
        region_stack: vec![],
    };
    let mut incoming = vec![
        v::PracticalVirPhiIncoming {
            predecessor_node_id: predecessor.clone(),
            value_id: allocation.id.clone(),
        },
        v::PracticalVirPhiIncoming {
            predecessor_node_id: body_id.clone(),
            value_id: phi.id.clone(),
        },
    ];
    incoming.sort_by(|a, b| a.predecessor_node_id.cmp(&b.predecessor_node_id));
    header.phi_values = vec![v::PracticalVirPhiValue {
        value: phi.clone(),
        incoming,
    }];
    header.literal_values = vec![v::PracticalVirLiteral {
        result: condition.clone(),
        value: MonomorphicValue::Bool {
            type_id: condition.type_id.clone(),
            value: false,
        },
    }];
    header.condition_value_id = Some(condition.id);
    let mut back = f
        .blocks
        .iter()
        .find(|b| b.node.tag == ControlNodeTag::Entry)
        .unwrap()
        .clone();
    back.node = ControlNode {
        id: body_id.clone(),
        ordinal: (ordinal + 1) as u32,
        tag: ControlNodeTag::Jump,
        condition_type_id: None,
        normal_successor_ids: vec![header_id.clone()],
        exceptional_successors: vec![],
        abrupt: None,
        loop_id: None,
        region_stack: vec![],
    };
    let mut preheader = back.clone();
    preheader.node.id = predecessor;
    preheader.node.ordinal = (ordinal + 2) as u32;
    f.blocks.extend([header, back, preheader]);
    f.loops.push(LoopRegion {
        id: loop_id,
        parent_loop_id: None,
        header_node_id: header_id,
        body_entry_node_id: body_id.clone(),
        continue_target_node_id: body_id.clone(),
        break_target_node_id: target,
        backedge_source_ids: vec![body_id.clone()],
    });
    for mutation in 0..3 {
        let mut changed = functions.clone();
        let f = changed.iter_mut().find(|f| f.id == root).unwrap();
        match mutation {
            0 => {}
            1 => {
                f.blocks[ordinal].phi_values[0]
                    .incoming
                    .iter_mut()
                    .find(|i| i.predecessor_node_id == body_id)
                    .unwrap()
                    .value_id = allocation.id.clone()
            }
            2 => f.blocks[read].invocation.as_mut().unwrap().operands[0] = allocation.clone(),
            _ => unreachable!(),
        }
        let bytes = v::canonical_csharp_practical_vir_transport(
            input,
            v::PracticalVirContents {
                functions: changed,
                source_obligations: emitted.vir().source_obligations().to_vec(),
                ..Default::default()
            },
        )
        .unwrap();
        let result = v::import_csharp_practical_vir_json(&bytes, input);
        assert_eq!(
            result.is_ok(),
            mutation == 0,
            "loop ownership mutation {mutation}: {:?}",
            result.err()
        );
    }
}

#[test]
fn csharp_03_t04_w06_actual_source_loops_emit_ordinary_vir() {
    let b = bundle();
    let mut requests = Vec::new();
    let mut selections = Vec::new();
    for case in fixtures()
        .into_iter()
        .filter(|r| r["stage"] == "loops" && !r["data"].is_null())
    {
        let source = &case["source_case"];
        let adapted = json!({"root":source["root"],"source":source["source"],"facts":case["data"]["control_lowering"]["facts"]});
        let (context, captures) = context_support::context_with_sidecar(
            &b,
            source["root"].as_str().unwrap(),
            source["source"].as_str().unwrap().as_bytes(),
            |ctx| hashed(document(ctx, &adapted, "total", rows(&adapted))),
        );
        let runs = if source["runs"].as_array().unwrap().is_empty() {
            [-1, 0, 1, 2, 4]
                .into_iter()
                .map(|n| json!({"n":n,"a":[],"s":""}))
                .collect::<Vec<_>>()
        } else {
            source["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| json!({"n":r["n"],"a":r["a"],"s":r["s"]}))
                .collect()
        };
        requests.push(json!({"id":source["id"],"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"runs":runs,
            "inputs":captures.entries().iter().map(|e| json!({"path":e.path(),"kind":if e.kind()==a::OriginalInputKind::Source{"source"}else{"sidecar"},"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
        if !case["data"]["callables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == source["root"] && c["identity"]["name"] == "Run")
        {
            requests
                .last_mut()
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("runs");
        }
        selections.push((source["id"].as_str().unwrap().to_owned(), context, captures));
    }
    assert_eq!(requests.len(), 27);
    if let Some(path) = std::env::var_os("MPK_CSHARP_CONTROL_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let responses: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/loop-responses.json",
    ))
    .unwrap();
    assert_eq!(responses.len(), selections.len());
    let mut failures = Vec::new();
    for ((id, context, captures), response) in selections.into_iter().zip(responses) {
        assert_eq!(response["id"], id);
        assert!(response.get("reject").is_none(), "{response}");
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                let function = emitted
                    .vir()
                    .functions()
                    .iter()
                    .find(|f| f.id == context.selected_root_ids()[0])
                    .unwrap();
                for run in response["runs"].as_array().into_iter().flatten() {
                    let expected = if run["error"] == "" {
                        Ok(run["value"].clone())
                    } else {
                        Err(run["error"].as_str().unwrap().to_owned())
                    };
                    assert_eq!(
                        execution::execute(&b, &emitted, &function.id, run),
                        expected,
                        "{id}: {run}"
                    );
                }
                assert!(!emitted
                    .vir()
                    .functions()
                    .iter()
                    .flat_map(|f| &f.loops)
                    .collect::<Vec<_>>()
                    .is_empty());
                assert!(!emitted.vir().data_contracts().is_empty());
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
                assert_eq!(
                    emitted.artifacts().canonical_bytes(),
                    again.artifacts().canonical_bytes()
                );
            }
            Err(error) => failures.push(format!("{id}: {error:?}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_partial_callee_cannot_support_total_caller() {
    use std::collections::BTreeMap;
    let b = bundle();
    let case = fixtures()
        .into_iter()
        .find(|r| r["stage"] == "handlers" && r["source_case"]["id"] == "get_only")
        .unwrap();
    let row = &case["source_case"];
    let root = row["root"].as_str().unwrap();
    let (context, captures) =
        context_support::context(&b, root, row["source"].as_str().unwrap().as_bytes());
    let source = ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&case["data"]).unwrap(),
    )
    .unwrap();
    assert!(derive_control_termination(&source, &BTreeMap::new())
        .unwrap()
        .values()
        .all(|m| m == "total"));
    for callee in source.callables().iter().filter(|c| c.id() != root) {
        let mut claims = BTreeMap::from([(callee.id().to_owned(), "partial".to_owned())]);
        let derived = derive_control_termination(&source, &claims).unwrap();
        assert_eq!(derived[root], "partial", "callee {}", callee.id());
        claims.insert(root.to_owned(), "total".to_owned());
        assert!(derive_control_termination(&source, &claims).is_err());
        claims.insert(root.to_owned(), "partial".to_owned());
        assert_eq!(
            derive_control_termination(&source, &claims).unwrap()[root],
            "partial"
        );
    }
    assert!(derive_control_termination(
        &source,
        &BTreeMap::from([("missing".into(), "total".into())])
    )
    .is_err());
}

#[test]
fn csharp_03_t04_w06_source_control_linkage_and_cfg_mutations_reject() {
    let b = bundle();
    let case = fixtures()
        .into_iter()
        .find(|r| r["stage"] == "loops" && r["source_case"]["id"] == "while")
        .unwrap();
    let row = &case["source_case"];
    let (context, captures) = context_support::context(
        &b,
        row["root"].as_str().unwrap(),
        row["source"].as_str().unwrap().as_bytes(),
    );
    for mutation in 0..8 {
        let mut data = case["data"].clone();
        let control = &mut data["control_lowering"];
        match mutation {
            0 => control["normalized_syntax_sha256"] = json!("0".repeat(64)),
            1 => control["functions"] = json!([]),
            2 => control["facts"]["methods"][0]["parameters"][0]["type_id"] = json!(ty("bool")),
            3 => control["functions"][0]["nodes"][0]["successors"] = json!(["missing"]),
            4 => {
                let nodes = control["functions"][0]["nodes"].as_array_mut().unwrap();
                let n = nodes
                    .iter_mut()
                    .find(|n| n["operation"] == "binary")
                    .unwrap();
                n["inputs"][0] = n["result"].clone();
            }
            5 => {
                let nodes = control["functions"][0]["nodes"].as_array_mut().unwrap();
                nodes.iter_mut().find(|n| n["operation"] == "load").unwrap()["slot"] =
                    json!("local:999");
            }
            6 => control["functions"][0]["loops"][0]["backedges"] = json!([]),
            7 => control["functions"][0]["loops"][0]["parent"] = json!("missing"),
            _ => unreachable!(),
        }
        assert!(
            ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(&data).unwrap()
            )
            .is_err(),
            "control mutation {mutation}"
        );
    }
    let seeds: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/fuzz-seeds.json",
    ))
    .unwrap();
    assert_eq!(seeds.len(), 37);
    let corpus = fixtures();
    for (index, seed) in seeds.iter().enumerate() {
        assert_eq!(seed["seed"], index);
        let case = corpus
            .iter()
            .find(|r| r["stage"] == seed["stage"] && r["source_case"]["id"] == seed["case_id"])
            .unwrap();
        let row = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let mut data = case["data"].clone();
        let f = data["control_lowering"]["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["callable_id"] == seed["function_id"])
            .unwrap();
        let node = f["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["id"] == seed["node_id"])
            .unwrap();
        match seed["mutation"].as_str().unwrap() {
            "source_ordinal_bounds" => node["source_ordinal"] = json!(usize::MAX),
            "normal_target" => node["successors"][0] = json!("missing.node"),
            "value_definition" => node["inputs"][0] = json!("missing.value"),
            "exception_target" => node["exceptional_successors"][0] = json!("missing.handler"),
            _ => panic!("unknown fuzz family"),
        }
        assert!(
            ValidatedDataSource::import_captured_facts(
                &b,
                &context,
                &captures,
                &serde_json::to_vec(&data).unwrap()
            )
            .is_err(),
            "fuzz seed {seed}"
        );
    }
}

#[test]
fn csharp_03_t04_w06_actual_source_patterns_emit_ordinary_vir() {
    let b = bundle();
    let mut failures = Vec::new();
    let mut count = 0;
    for case in fixtures()
        .into_iter()
        .filter(|r| r["stage"] == "patterns" && !r["data"].is_null())
    {
        if case["data"]["control_lowering"]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| !f["loops"].as_array().unwrap().is_empty())
        {
            continue;
        }
        count += 1;
        let row = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&case["data"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                for run in row["runs"].as_array().unwrap() {
                    let expected = if run["error"] == "" {
                        Ok(run["value"].clone())
                    } else {
                        Err(run["error"].as_str().unwrap().to_owned())
                    };
                    assert_eq!(
                        execution::execute(&b, &emitted, row["root"].as_str().unwrap(), run),
                        expected,
                        "{}: {run}",
                        row["id"]
                    );
                }
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
                assert_eq!(
                    emitted.artifacts().canonical_bytes(),
                    again.artifacts().canonical_bytes()
                );
            }
            Err(e) => failures.push(format!("{}: {e:?}", row["id"])),
        }
    }
    assert_eq!(count, 34);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_actual_source_exceptions_emit_ordinary_vir() {
    let b = bundle();
    let mut failures = Vec::new();
    let mut count = 0;
    for case in fixtures()
        .into_iter()
        .filter(|r| r["stage"] == "exceptions" && !r["data"].is_null())
    {
        if case["data"]["control_lowering"]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| !f["loops"].as_array().unwrap().is_empty())
        {
            continue;
        }
        count += 1;
        let row = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&case["data"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                for run in row["runs"].as_array().unwrap() {
                    let error = run["error"].as_str().unwrap();
                    let source_type = case["data"]["types"].as_array().unwrap().iter().find(|t| {
                        format!(
                            "{}.{}",
                            t["namespace"].as_str().unwrap(),
                            t["name"].as_str().unwrap()
                        ) == error
                    });
                    let exception = source_type
                        .map(|t| t["id"].as_str().unwrap())
                        .unwrap_or(error)
                        .rsplit('.')
                        .next()
                        .unwrap();
                    let expected = if error.is_empty() {
                        Ok(run["value"].clone())
                    } else {
                        Err(exception.into())
                    };
                    let (actual, payload) = execution::execute_exception(
                        &b,
                        &emitted,
                        row["root"].as_str().unwrap(),
                        run,
                    );
                    assert_eq!(actual, expected, "{}: {run}", row["id"]);
                    if !run["code"].is_null() {
                        let code = payload
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .values()
                            .find(|v| v.is_number())
                            .cloned()
                            .unwrap();
                        assert_eq!(code, run["code"], "{}: payload {run}", row["id"]);
                    }
                }
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
                assert_eq!(
                    emitted.artifacts().canonical_bytes(),
                    again.artifacts().canonical_bytes()
                );
            }
            Err(e) => failures.push(format!("{}: {e:?}", row["id"])),
        }
    }
    assert_eq!(count, 19);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_real_collection_algorithms_emit_ordinary_vir() {
    let b = bundle();
    let sources: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/collection-sources.json",
    ))
    .unwrap();
    let captures_only: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/collection-capture.json",
    ))
    .unwrap();
    assert_eq!(sources.len(), 6);
    assert_eq!(captures_only.len(), sources.len());
    let mut selections = Vec::new();
    let mut requests = Vec::new();
    for (case, first) in sources.iter().zip(&captures_only) {
        assert_eq!(case["id"], first["id"]);
        assert!(first.get("reject").is_none());
        let root = first["facts"]["selected_root_ids"][0].as_str().unwrap();
        let adapted = json!({"root":root,"source":case["source"],"facts":first["facts"]["control_lowering"]["facts"]});
        let (context, captures) = context_support::context_with_sidecar(
            &b,
            root,
            case["source"].as_str().unwrap().as_bytes(),
            |ctx| hashed(document(ctx, &adapted, "total", rows(&adapted))),
        );
        requests.push(json!({"id":case["id"],"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==a::OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>(),"runs":case["runs"]}));
        selections.push((context, captures));
    }
    if let Some(path) = std::env::var_os("MPK_CSHARP_COLLECTION_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let responses: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/collection-responses.json",
    ))
    .unwrap();
    assert_eq!(responses.len(), selections.len());
    let mut failures = Vec::new();
    for (((case, first), response), (context, captures)) in sources
        .iter()
        .zip(&captures_only)
        .zip(&responses)
        .zip(selections)
    {
        assert_eq!(case["id"], response["id"]);
        assert_eq!(first["runs"], response["runs"]);
        assert!(response.get("reject").is_none());
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                for run in response["runs"].as_array().into_iter().flatten() {
                    let expected = if run["error"] == "" {
                        Ok(run["value"].clone())
                    } else {
                        Err(run["error"].as_str().unwrap().to_owned())
                    };
                    assert_eq!(
                        execution::execute(&b, &emitted, &context.selected_root_ids()[0], run),
                        expected,
                        "{}: {run}",
                        case["id"]
                    );
                }
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
            }
            Err(e) => failures.push(format!("{}: {e:?}", case["id"])),
        }
    }
    let package: Value = serde_json::from_slice(&read(
        "develop/specs/vectors/csharp-practical-foundation-v1.json",
    ))
    .unwrap();
    let vector = package["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == "loops.runtime_source.array_two_pass.0000")
        .unwrap();
    let runs = responses[0]["runs"].as_array().unwrap();
    let mut observed = runs[..3]
        .iter()
        .map(|r| r["value"].as_i64().unwrap().to_string())
        .collect::<Vec<_>>();
    observed.push(
        runs[3..]
            .iter()
            .map(|r| r["value"].as_i64().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    assert_eq!(json!(observed), vector["expected"]["value"]);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_ordinary_control_mutations_reject_without_roslyn() {
    use mpk_vc::csharp_practical_vir_validation as v;
    let b = bundle();
    let responses: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/loop-responses.json",
    ))
    .unwrap();
    let response = responses.iter().find(|r| r["id"] == "while").unwrap();
    let case = fixtures()
        .into_iter()
        .find(|r| r["stage"] == "loops" && r["source_case"]["id"] == "while")
        .unwrap();
    let row = &case["source_case"];
    let adapted = json!({"root":row["root"],"source":row["source"],"facts":case["data"]["control_lowering"]["facts"]});
    let (context, captures) = context_support::context_with_sidecar(
        &b,
        row["root"].as_str().unwrap(),
        row["source"].as_str().unwrap().as_bytes(),
        |ctx| hashed(document(ctx, &adapted, "total", rows(&adapted))),
    );
    let source = ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: &context,
        captured_inputs: &captures,
        foundation_descriptor_transport: registered_foundation_descriptor_transport(),
        foundation_definitions_transport: registered_foundation_definitions_transport(),
        closed_roots_transport: emitted.closure().roots().canonical_json(),
        closed_instances_transport: emitted.closure().closed().canonical_json(),
        semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
        required_checks_transport: emitted.operations().required_checks().canonical_bytes(),
        operations_transport: emitted.operations().operations().canonical_bytes(),
    };
    for mutation in 0..10 {
        let mut functions = emitted.vir().functions().to_vec();
        let f = &mut functions[0];
        match mutation {
            0 => f.control_protocol = None,
            1 => f.control_protocol.as_mut().unwrap().anchors[0].source_node_id = "missing".into(),
            2 => {
                let anchor = f
                    .control_protocol
                    .as_mut()
                    .unwrap()
                    .anchors
                    .iter_mut()
                    .find(|a| a.source_span.is_some())
                    .unwrap();
                anchor.source_span.as_mut().unwrap().start_byte += 1;
            }
            3 => f.loops[0].break_target_node_id = f.loops[0].body_entry_node_id.clone(),
            4 => f.loops[0].backedge_source_ids.clear(),
            5 => {
                let branch = f
                    .blocks
                    .iter_mut()
                    .find(|b| b.node.tag == ControlNodeTag::LoopHeader)
                    .unwrap();
                branch.node.normal_successor_ids.swap(0, 1);
            }
            6 => {
                let call = f
                    .blocks
                    .iter_mut()
                    .find_map(|b| b.invocation.as_mut())
                    .unwrap();
                call.operands[0].id = call.result.id.clone();
            }
            7 => {
                let phi = f
                    .blocks
                    .iter_mut()
                    .find_map(|b| b.phi_values.first_mut())
                    .unwrap();
                phi.incoming[0].predecessor_node_id = "missing".into();
            }
            8 => {
                let anchor = f
                    .control_protocol
                    .as_mut()
                    .unwrap()
                    .anchors
                    .iter_mut()
                    .find(|a| a.artifact_node_ids.len() > 1)
                    .unwrap();
                anchor.artifact_node_ids.pop();
            }
            9 => {
                let anchor = &mut f.control_protocol.as_mut().unwrap().anchors[0];
                anchor.artifact_node_ids = vec![anchor.entry_node_id.clone(); 4097];
            }
            _ => unreachable!(),
        }
        let bytes = v::canonical_csharp_practical_vir_transport(
            input,
            v::PracticalVirContents {
                functions,
                source_exceptions: emitted.vir().source_exceptions().to_vec(),
                binding_projections: emitted.vir().binding_projections().to_vec(),
                binding_commutations: emitted.vir().binding_commutations().to_vec(),
                data_contracts: emitted.vir().data_contracts().to_vec(),
                source_obligations: emitted.vir().source_obligations().to_vec(),
            },
        )
        .unwrap();
        let error = v::import_csharp_practical_vir_json(&bytes, input)
            .err()
            .unwrap_or_else(|| panic!("control mutation {mutation} accepted"));
        if mutation == 9 {
            assert_eq!(error.phase(), v::PracticalVirImportPhase::Resource);
        }
    }
}

#[test]
fn csharp_03_t04_w06_actual_source_catches_emit_ordinary_vir() {
    let b = bundle();
    let mut failures = Vec::new();
    let mut count = 0;
    for case in fixtures().into_iter().filter(|r| {
        r["stage"] == "handlers"
            && r["accepted"] == true
            && !r["data"]["control_lowering"]["functions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| !f["loops"].as_array().unwrap().is_empty())
    }) {
        count += 1;
        let row = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&case["data"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
                for run in row["runs"].as_array().unwrap() {
                    let error = run["error"].as_str().unwrap();
                    let expected = if error.is_empty() {
                        Ok(run["value"].clone())
                    } else {
                        Err(error.rsplit('.').next().unwrap().to_owned())
                    };
                    let (actual, trace) = execution::execute_handler(
                        &b,
                        &emitted,
                        row["root"].as_str().unwrap(),
                        run,
                    );
                    assert_eq!(actual, expected, "{}: {run}", row["id"]);
                    assert_eq!(json!(trace), run["trace"], "trace {}: {run}", row["id"]);
                }
            }
            Err(e) => failures.push(format!("{}: {e:?}", row["id"])),
        }
    }
    assert_eq!(count, 27);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_control_cross_feature_loops_emit_ordinary_vir() {
    let b = bundle();
    let mut requests = Vec::new();
    let mut selected = Vec::new();
    for case in fixtures().into_iter().filter(|r| {
        r["stage"] != "loops"
            && r["accepted"] == true
            && r["data"]["control_lowering"]["functions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| !f["loops"].as_array().unwrap().is_empty())
    }) {
        let row = &case["source_case"];
        let adapted = json!({"root":row["root"],"source":row["source"],"facts":case["data"]["control_lowering"]["facts"]});
        let (context, captures) = context_support::context_with_sidecar(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
            |ctx| hashed(document(ctx, &adapted, "total", rows(&adapted))),
        );
        let id = format!(
            "{}:{}",
            case["stage"].as_str().unwrap(),
            row["id"].as_str().unwrap()
        );
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"path":e.path(),"kind":if e.kind()==a::OriginalInputKind::Source{"source"}else{"sidecar"},"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
        selected.push((id, row.clone(), context, captures));
    }
    assert_eq!(selected.len(), 4);
    if let Some(path) = std::env::var_os("MPK_CSHARP_CROSS_CONTROL_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let responses: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/cross-loop-responses.json",
    ))
    .unwrap();
    assert_eq!(responses.len(), selected.len());
    let mut failures = Vec::new();
    for ((id, row, context, captures), response) in selected.into_iter().zip(responses) {
        assert_eq!(response["id"], id);
        assert!(
            response.get("reject").is_none(),
            "{id}: {}",
            response["reject"]
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
                for run in row["runs"].as_array().unwrap() {
                    let error = run["error"].as_str().unwrap();
                    let expected = if error.is_empty() {
                        Ok(run["value"].clone())
                    } else {
                        let source_type = response["facts"]["types"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|t| {
                                format!(
                                    "{}.{}",
                                    t["namespace"].as_str().unwrap(),
                                    t["name"].as_str().unwrap()
                                ) == error
                            });
                        Err(source_type
                            .map(|t| t["id"].as_str().unwrap())
                            .unwrap_or(error)
                            .rsplit('.')
                            .next()
                            .unwrap()
                            .to_owned())
                    };
                    if row["trace"].is_array() || run["trace"].is_array() {
                        let (actual, trace) = execution::execute_handler(
                            &b,
                            &emitted,
                            row["root"].as_str().unwrap(),
                            run,
                        );
                        assert_eq!(actual, expected, "{id}: {run}");
                        assert_eq!(json!(trace), run["trace"], "trace {id}: {run}");
                    }
                    assert_eq!(
                        execution::execute(&b, &emitted, row["root"].as_str().unwrap(), run),
                        expected,
                        "{id}: {run}"
                    );
                }
            }
            Err(e) => failures.push(format!("{id}: {e:?}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_filter_and_finally_mutations_reject_without_roslyn() {
    use mpk_vc::csharp_practical_vir_validation as v;
    let b = bundle();
    for name in ["filter_divide", "finally_normal"] {
        let case = fixtures()
            .into_iter()
            .find(|r| r["stage"] == "handlers" && r["source_case"]["id"] == name)
            .unwrap();
        let row = &case["source_case"];
        let (context, captures) = context_support::context(
            &b,
            row["root"].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&case["data"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
        let input = v::PracticalVirImportContext {
            data_source_facts: Some(source.captured_facts()),
            artifact_context: &context,
            captured_inputs: &captures,
            foundation_descriptor_transport: registered_foundation_descriptor_transport(),
            foundation_definitions_transport: registered_foundation_definitions_transport(),
            closed_roots_transport: emitted.closure().roots().canonical_json(),
            closed_instances_transport: emitted.closure().closed().canonical_json(),
            semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
            required_checks_transport: emitted.operations().required_checks().canonical_bytes(),
            operations_transport: emitted.operations().operations().canonical_bytes(),
        };
        if name == "finally_normal" {
            let roots: Value =
                serde_json::from_slice(emitted.closure().roots().canonical_json()).unwrap();
            assert!(roots["roots"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["provenance_id"]
                    .as_str()
                    .is_some_and(|id| id.ends_with(".control.pending_return"))));
            assert_eq!(
                derive_closed_instances(&b, emitted.closure().roots())
                    .unwrap()
                    .canonical_json(),
                emitted.closure().closed().canonical_json()
            );
            for mutation in 0..3 {
                let mut rows = roots["roots"].as_array().unwrap().clone();
                let index = rows
                    .iter()
                    .position(|r| {
                        r["provenance_id"]
                            .as_str()
                            .is_some_and(|id| id.ends_with(".control.pending_return"))
                    })
                    .unwrap();
                match mutation {
                    0 => {
                        rows.remove(index);
                    }
                    1 => {
                        rows[index]["provenance_id"] = json!("control.pending_return.wrong_source")
                    }
                    2 => {
                        rows[index]["type"] = json!({"kind":"instance","template":"option","arguments":[{"kind":"primitive","id":"i64"}]})
                    }
                    _ => unreachable!(),
                }
                let transport =
                    canonical_closed_root_set_transport(&b, &json!(rows), &roots["source_types"])
                        .unwrap();
                let changed = v::PracticalVirImportContext {
                    closed_roots_transport: &transport,
                    ..input
                };
                assert!(
                    v::import_csharp_practical_vir_json(emitted.vir().canonical_bytes(), changed)
                        .is_err(),
                    "control root/provenance mutation {mutation}"
                );
            }
        }
        for mutation in 0..8 {
            let mut functions = emitted.vir().functions().to_vec();
            let f = functions.iter_mut().find(|f| f.id == row["root"]).unwrap();
            if name == "filter_divide" {
                let region = &mut f.exception_regions[0];
                match mutation {
                    0 => region.catches.swap(0, 1),
                    1 => region.catches[0].filter.as_mut().unwrap().throw_means_false = false,
                    2 => {
                        region.catches[0]
                            .filter
                            .as_mut()
                            .unwrap()
                            .preserves_original_exception = false
                    }
                    3 => {
                        region.catches[0]
                            .filter
                            .as_mut()
                            .unwrap()
                            .execution
                            .as_mut()
                            .unwrap()
                            .next_search_node_id = region.try_entry_node_id.clone()
                    }
                    4 => {
                        region.catches[0]
                            .filter
                            .as_mut()
                            .unwrap()
                            .execution
                            .as_mut()
                            .unwrap()
                            .node_ids
                            .pop();
                    }
                    5 => {
                        let filter = region.catches[0].filter.as_mut().unwrap();
                        filter.execution.as_mut().unwrap().result_node_id =
                            filter.execution.as_ref().unwrap().entry_node_id.clone();
                    }
                    6 => {
                        let f = region.catches[0]
                            .filter
                            .as_mut()
                            .unwrap()
                            .execution
                            .as_mut()
                            .unwrap();
                        f.selected_node_id = f.next_search_node_id.clone();
                    }
                    7 => {
                        let f = region.catches[0]
                            .filter
                            .as_mut()
                            .unwrap()
                            .execution
                            .as_mut()
                            .unwrap();
                        f.node_ids = vec![f.entry_node_id.clone(); 4097];
                    }
                    _ => unreachable!(),
                }
            } else {
                match mutation {
                    0 => f.exception_regions[0].finally_entry_node_id = None,
                    1 => f.exception_regions[0].search_entry_node_ids.clear(),
                    2 => {
                        f.unwind_plans
                            .iter_mut()
                            .find(|p| p.search_entry_node_id.is_some())
                            .unwrap()
                            .search_entry_node_id = None
                    }
                    3 => f
                        .unwind_plans
                        .iter_mut()
                        .find(|p| p.search_entry_node_id.is_some())
                        .unwrap()
                        .finally_region_ids
                        .clear(),
                    4 => {
                        f.blocks
                            .iter_mut()
                            .find(|b| b.node.tag == ControlNodeTag::FinallyExit)
                            .unwrap()
                            .node
                            .tag = ControlNodeTag::Jump
                    }
                    5 => {
                        let value = f
                            .blocks
                            .iter_mut()
                            .flat_map(|b| &mut b.literal_values)
                            .find(|v| matches!(v.value, MonomorphicValue::Signed { .. }))
                            .unwrap();
                        if let MonomorphicValue::Signed { value, .. } = &mut value.value {
                            *value = "17".into();
                        }
                    }
                    6 => f
                        .control_protocol
                        .as_mut()
                        .unwrap()
                        .escape_search_node_ids
                        .clear(),
                    7 => {
                        let b = f
                            .blocks
                            .iter_mut()
                            .find(|b| b.handler_exception_source_id.is_some())
                            .unwrap();
                        b.handler_exception_source_id =
                            Some(b.handler_exception_value.as_ref().unwrap().id.clone());
                    }
                    _ => unreachable!(),
                }
            }
            let bytes = v::canonical_csharp_practical_vir_transport(
                input,
                v::PracticalVirContents {
                    functions,
                    source_exceptions: emitted.vir().source_exceptions().to_vec(),
                    binding_projections: emitted.vir().binding_projections().to_vec(),
                    binding_commutations: emitted.vir().binding_commutations().to_vec(),
                    data_contracts: emitted.vir().data_contracts().to_vec(),
                    source_obligations: emitted.vir().source_obligations().to_vec(),
                },
            )
            .unwrap();
            assert!(
                v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                "{name} mutation {mutation}"
            );
        }
    }
}

#[test]
fn csharp_03_t04_w06_construction_exception_paths_emit_and_discard() {
    use mpk_vc::csharp_practical_vir_validation as v;
    let b = bundle();
    let mut sources: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/construction-sources.json",
    ))
    .unwrap();
    let mut initial: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/construction-capture.json",
    ))
    .unwrap();
    let selected = std::env::var("MPK_CSHARP_CONSTRUCTION_CASE").ok();
    if let Some(id) = &selected {
        sources.retain(|r| r["id"] == *id);
        initial.retain(|r| r["id"] == *id);
        assert_eq!(sources.len(), 1);
    }
    assert_eq!(sources.len(), initial.len());
    let mut requests = Vec::new();
    let mut selections = Vec::new();
    for (case, first) in sources.iter().zip(&initial) {
        assert_eq!(case["id"], first["id"]);
        if case["accepted"] == false {
            assert_eq!(first["reject"], case["diagnostic"], "{}", case["id"]);
            assert_eq!(first["artifact_count"], 0);
            continue;
        }
        assert!(first.get("reject").is_none(), "{}: {first}", case["id"]);
        let root = first["facts"]["selected_root_ids"][0].as_str().unwrap();
        let adapted = json!({"root":root,"source":case["source"],"facts":first["facts"]["control_lowering"]["facts"]});
        let mut targets = first["facts"]["control_lowering"]["facts"]["methods"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["callable_id"] == root || !m["loops"].as_array().unwrap().is_empty())
            .map(|m| m["callable_id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        targets.sort();
        let paths = if targets.len() == 1 {
            vec!["contracts/data.json".into()]
        } else {
            (0..targets.len())
                .map(|i| format!("contracts/control-{i:03}.json"))
                .collect()
        };
        let (context, captures) = context_support::context_with_sidecars(
            &b,
            root,
            case["source"].as_str().unwrap().as_bytes(),
            paths,
            |ctx| {
                targets
                    .iter()
                    .map(|id| {
                        let mut method = adapted.clone();
                        method["root"] = json!(id);
                        hashed(document(ctx, &method, "total", rows(&method)))
                    })
                    .collect()
            },
        );
        requests.push(json!({"id":case["id"],"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==a::OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>(),"runs":case["runs"]}));
        selections.push((case, context, captures));
    }
    if let Some(path) = std::env::var_os("MPK_CSHARP_CONSTRUCTION_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let mut responses: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/construction-responses.json",
    ))
    .unwrap();
    if let Some(id) = &selected {
        responses.retain(|r| r["id"] == *id);
    }
    assert_eq!(responses.len(), selections.len());
    let mut failures = Vec::new();
    for ((case, context, captures), response) in selections.into_iter().zip(responses) {
        assert_eq!(case["id"], response["id"]);
        assert!(response.get("reject").is_none(), "{response}");
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]));
        match emit_data_phase(&b, &context, &captures, &source) {
            Ok(emitted) => {
                for run in response["runs"].as_array().unwrap() {
                    let expected = if run["error"] == "" {
                        Ok(run["value"].clone())
                    } else {
                        Err(run["error"].as_str().unwrap().into())
                    };
                    assert_eq!(
                        execution::execute(&b, &emitted, &context.selected_root_ids()[0], run),
                        expected,
                        "{}: {run}",
                        case["id"]
                    );
                }
                let input = v::PracticalVirImportContext {
                    data_source_facts: Some(source.captured_facts()),
                    artifact_context: &context,
                    captured_inputs: &captures,
                    foundation_descriptor_transport: registered_foundation_descriptor_transport(),
                    foundation_definitions_transport: registered_foundation_definitions_transport(),
                    closed_roots_transport: emitted.closure().roots().canonical_json(),
                    closed_instances_transport: emitted.closure().closed().canonical_json(),
                    semantic_bindings_transport: emitted.closure().bindings().canonical_bytes(),
                    required_checks_transport: emitted
                        .operations()
                        .required_checks()
                        .canonical_bytes(),
                    operations_transport: emitted.operations().operations().canonical_bytes(),
                };
                let has_sequences = emitted
                    .vir()
                    .functions()
                    .iter()
                    .flat_map(|f| &f.blocks)
                    .any(|b| {
                        matches!(
                            b.node.tag,
                            ControlNodeTag::HandlerEntry | ControlNodeTag::FinallyEntry
                        ) && !b.construction_actions.is_empty()
                    });
                let has_objects = emitted
                    .vir()
                    .functions()
                    .iter()
                    .filter_map(|f| f.object_protocol.as_ref())
                    .any(|p| !p.exceptional_discards.is_empty());
                for mutation in 0..4 {
                    if mutation < 2 && !has_sequences || mutation >= 2 && !has_objects {
                        continue;
                    }
                    let mut functions = emitted.vir().functions().to_vec();
                    if mutation < 2 {
                        let block = functions
                            .iter_mut()
                            .flat_map(|f| &mut f.blocks)
                            .find(|b| {
                                matches!(
                                    b.node.tag,
                                    ControlNodeTag::HandlerEntry | ControlNodeTag::FinallyEntry
                                ) && !b.construction_actions.is_empty()
                            })
                            .unwrap();
                        if mutation == 0 {
                            block.construction_actions.clear();
                        } else {
                            let v::PracticalConstructionAction::Discard {
                                construction_id, ..
                            } = &mut block.construction_actions[0]
                            else {
                                unreachable!()
                            };
                            construction_id.push_str(".stale");
                        }
                    } else {
                        let protocol = functions
                            .iter_mut()
                            .filter_map(|f| f.object_protocol.as_mut())
                            .find(|p| !p.exceptional_discards.is_empty())
                            .unwrap();
                        if mutation == 2 {
                            protocol.exceptional_discards.clear();
                        } else {
                            protocol.exceptional_discards[0].origin_value_ids[0].push_str(".stale");
                        }
                    }
                    let bytes = v::canonical_csharp_practical_vir_transport(
                        input,
                        v::PracticalVirContents {
                            functions,
                            source_exceptions: emitted.vir().source_exceptions().to_vec(),
                            binding_projections: emitted.vir().binding_projections().to_vec(),
                            binding_commutations: emitted.vir().binding_commutations().to_vec(),
                            data_contracts: emitted.vir().data_contracts().to_vec(),
                            source_obligations: emitted.vir().source_obligations().to_vec(),
                        },
                    )
                    .unwrap();
                    assert!(
                        v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                        "{} cleanup mutation {mutation}",
                        case["id"]
                    );
                }
                let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
                assert_eq!(
                    emitted.vir().canonical_bytes(),
                    again.vir().canonical_bytes()
                );
            }
            Err(e) => failures.push(format!("{}: {e:?}", case["id"])),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn csharp_03_t04_w06_conformance_binds_current_capture_and_rejection_matrix() {
    let conformance: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/conformance.json",
    ))
    .unwrap();
    assert_eq!(conformance["work_item"], "CSHARP-03-T04-W06");
    for file in conformance["files"].as_array().unwrap() {
        let bytes = read(file["path"].as_str().unwrap());
        assert_eq!(file["sha256"], sha256_raw_file_bytes(&bytes).to_hex());
        assert_eq!(file["size_bytes"], bytes.len());
    }
    let cases = fixtures();
    assert_eq!(conformance["source_cases"], cases.len());
    assert_eq!(
        conformance["accepted_source_cases"],
        cases.iter().filter(|c| c["accepted"] == true).count()
    );
    assert_eq!(
        conformance["rejected_source_cases"],
        cases.iter().filter(|c| c["accepted"] == false).count()
    );
    for (name, count_key, runs_key) in [
        ("loop", None, "loop_clr_runs"),
        (
            "collection",
            Some("collection_cases"),
            "collection_clr_runs",
        ),
        (
            "construction",
            Some("accepted_construction_cases"),
            "construction_clr_runs",
        ),
    ] {
        let rows: Vec<Value> = serde_json::from_slice(&read(&format!(
            "develop/migrations/csharp-03/control-emission/{name}-responses.json"
        )))
        .unwrap();
        if let Some(key) = count_key {
            assert_eq!(conformance[key], rows.len());
        }
        assert_eq!(
            conformance[runs_key],
            rows.iter()
                .map(|r| r["runs"].as_array().map_or(0, Vec::len))
                .sum::<usize>()
        );
    }
    let construction: Vec<Value> = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/control-emission/construction-capture.json",
    ))
    .unwrap();
    assert_eq!(conformance["construction_cases"], construction.len());
    assert_eq!(
        conformance["rejected_construction_cases"],
        construction
            .iter()
            .filter(|r| r.get("reject").is_some())
            .count()
    );
}
