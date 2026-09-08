//! T06-W04: replay real source captures, check every native edge and mutation.
use crate::{b, read, support};
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
pub(super) fn nodes(p: &ControlVcProgram) -> usize {
    p.sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .map(|p| p.term.nodes())
        .sum::<usize>()
        + p.functions()
            .iter()
            .flat_map(|f| f.edges.iter().map(|e| &e.guard).chain(&f.entry_conditions))
            .map(|p| p.term.nodes())
            .sum::<usize>()
}
fn guard_truth(t: &ContractTerm, value: bool) -> bool {
    match t {
        ContractTerm::Var { .. } => value,
        ContractTerm::Const { name, .. } if name == "Mpk.CSharp.Bool.true" => true,
        ContractTerm::Const { name, .. } if name == "Mpk.CSharp.Bool.false" => false,
        ContractTerm::App {
            function, argument, ..
        } => match function.as_ref() {
            ContractTerm::Const { name, .. } if name == "Mpk.CSharp.Bool.Not" => {
                !guard_truth(argument, value)
            }
            _ => panic!("expected native branch guard"),
        },
        _ => panic!("expected branch guard"),
    }
}
fn check(vc: &ValidatedPracticalVc, source: PracticalVcSource<'_>, id: &str) -> Value {
    let p = vc.control_vcs();
    for pred in p
        .sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .chain(
            p.functions()
                .iter()
                .flat_map(|f| f.edges.iter().map(|e| &e.guard).chain(&f.entry_conditions)),
        )
    {
        super::construction::assert_typed(
            &pred.term,
            &pred
                .bindings
                .iter()
                .map(|b| b.type_id.clone())
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(
        p.sequents()
            .iter()
            .map(|s| &s.id)
            .collect::<BTreeSet<_>>()
            .len(),
        p.sequents().len(),
        "{id}"
    );
    for f in p.functions() {
        for block in source
            .vir
            .functions()
            .iter()
            .find(|v| v.id == f.function_id)
            .unwrap()
            .blocks
            .iter()
            .filter(|b| b.condition_value_id.is_some())
        {
            let edges = f
                .edges
                .iter()
                .filter(|e| e.source_node_id == block.node.id && e.kind == "normal")
                .collect::<Vec<_>>();
            assert_eq!(edges.len(), 2);
            for value in [false, true] {
                let active = edges
                    .iter()
                    .filter(|e| guard_truth(&e.guard.term, value))
                    .collect::<Vec<_>>();
                assert_eq!(active.len(), 1);
                assert_eq!(
                    active[0].target_node_id.as_ref(),
                    Some(&block.node.normal_successor_ids[usize::from(!value)])
                );
            }
        }
    }
    for l in p.loops() {
        let f = p
            .functions()
            .iter()
            .find(|f| f.function_id == l.function_id)
            .unwrap();
        let inside = l.member_node_ids.iter().collect::<BTreeSet<_>>();
        let expected = f
            .edges
            .iter()
            .filter(|e| {
                inside.contains(&e.source_node_id)
                    || e.target_node_id
                        .as_ref()
                        .is_some_and(|t| inside.contains(t))
            })
            .map(|e| &e.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected,
            l.edges.iter().map(|e| &e.edge_id).collect(),
            "{id}"
        );
        assert_eq!(expected.len(), l.edges.len());
        for backedge in &l.region.backedge_source_ids {
            assert!(l.edges.iter().any(|e| e.role == "backedge"
                && f.edges
                    .iter()
                    .any(|flow| flow.id == e.edge_id && flow.source_node_id == *backedge)));
        }
        for edge in &l.edges {
            let ss = p
                .sequents()
                .iter()
                .filter(|s| s.region_id == l.region.id && s.id.ends_with(&edge.edge_id))
                .collect::<Vec<_>>();
            assert_eq!(
                ss.len(),
                usize::from(edge.role != "internal"),
                "{id} {}",
                edge.role
            );
            if edge.role == "backedge" {
                assert!(ss[0]
                    .goals
                    .iter()
                    .flat_map(|g| &g.bindings)
                    .filter(|b| b.kind == "current_slot" && b.edge_id.is_some())
                    .all(|b| b.edge_id.as_ref() == Some(&edge.edge_id)));
            }
        }
        if l.origin == "source_contract" {
            assert!(!l.invariants.is_empty());
        }
    }
    if id == "property_claim" {
        assert!(p.patterns().iter().any(|p| !p.total_getter_ids.is_empty()));
    }
    let mut owned_steps = BTreeSet::new();
    for pattern in p.patterns() {
        assert!(!pattern.steps.is_empty());
        if id == "nonexhaustive" {
            assert_eq!(pattern.no_match_rule, "modeled_exception");
            assert!(!pattern.no_match_node_ids.is_empty());
        }
        for step in &pattern.steps {
            assert!(owned_steps.insert((&pattern.function_id, &step.source_node_id)));
            let raw = p
                .functions()
                .iter()
                .find(|f| f.function_id == pattern.function_id)
                .unwrap()
                .source_graph
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|n| n.id == step.source_node_id)
                .unwrap();
            assert_eq!(raw.successors, step.successor_source_ids);
            assert_eq!(raw.inputs, step.source_inputs);
            assert_eq!(
                p.sequents()
                    .iter()
                    .filter(|s| s.id == format!("{}.{}", pattern.id, step.source_node_id))
                    .count(),
                1
            );
        }
    }
    assert_eq!(
        import_csharp_practical_control_vcs(&p.canonical_bytes(), source).unwrap(),
        *p
    );
    if !p.functions().is_empty() {
        let v: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let mut mutations = vec![];
        if let Some((fi, ei)) = p.functions().iter().enumerate().find_map(|(i, f)| {
            f.edges
                .iter()
                .position(|e| matches!(e.guard.term, ContractTerm::Var { .. }))
                .map(|j| (i, j))
        }) {
            let mut m = v.clone();
            m["functions"][fi]["edges"][ei]["guard"]["term"] = json!({"form":"const","name":"Mpk.CSharp.Bool.true","type_id":"mpk.csharp.value.bool.v1"});
            mutations.push(m);
        }
        let mut m = v.clone();
        m["functions"][0]["slot_rule"] = json!("all_slots_assigned");
        mutations.push(m);
        let mut m = v.clone();
        m["source_ir_sha256"] = json!("0".repeat(64));
        mutations.push(m);
        if !p.sequents().is_empty() {
            let mut m = v.clone();
            m["sequents"].as_array_mut().unwrap().remove(0);
            mutations.push(m);
        }
        if !p.loops().is_empty() {
            let mut m = v.clone();
            m["loops"][0]["edges"].as_array_mut().unwrap().remove(0);
            mutations.push(m);
        }
        if !p.patterns().is_empty() {
            let mut m = v.clone();
            m["patterns"][0]["steps"][0]["successor_source_ids"] = json!([]);
            mutations.push(m);
        }
        for m in mutations {
            let bytes = super::construction::edited_bytes(&p.canonical_bytes(), &m);
            assert!(
                import_csharp_practical_control_vcs(&bytes, source).is_err(),
                "{id}"
            );
        }
        assert!(vc.obligation_groups().iter().any(|g| g
            .subject_ids()
            .contains(&format!("control_program:{}", p.hash()))));
    }
    json!({"id":id,"control_sha256":p.hash(),"loops":p.loops().len(),"patterns":p.patterns().len(),"sequents":p.sequents().len(),"edge_roles":p.loops().iter().flat_map(|l|l.edges.iter().map(|e|e.role.clone())).collect::<BTreeSet<_>>()})
}
#[test]
fn csharp_03_t06_w04_original_control_paths_and_stable_ids() {
    let b = b();
    let requests = read("control-vc/loop-requests.json");
    let responses = read("control-emission/loop-responses.json");
    let mut summary = vec![];
    for (r, response) in requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
    {
        assert_eq!(r["id"], response["id"]);
        let (context, captures) = support::replay_context(&b, r);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(src).unwrap_or_else(|e| panic!("{} {e:?}", r["id"]));
        summary.push(check(&vc, src, r["id"].as_str().unwrap()));
    }
    assert_eq!(summary.len(), 27);
    for c in read("control-emission/source-cases.json")
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["stage"] == "patterns" && !c["data"].is_null())
    {
        if c["data"]["control_lowering"]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| !f["loops"].as_array().unwrap().is_empty())
        {
            continue;
        }
        let r = &c["source_case"];
        let (context, captures) = support::context(
            &b,
            r["root"].as_str().unwrap(),
            r["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&c["data"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(src).unwrap_or_else(|e| panic!("{} {e:?}", r["id"]));
        summary.push(check(&vc, src, r["id"].as_str().unwrap()));
    }
    assert_eq!(summary.len(), 61);
    assert!(summary
        .iter()
        .any(|r| r["id"] == "nonexhaustive" && r["patterns"].as_u64().unwrap() > 0));
    if let Ok(path) = std::env::var("MPK_T06_W04_GOLDEN_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    } else {
        assert_eq!(json!(summary), read("control-vc/goldens.json"));
    }
}

// Evaluate only the ordinary arithmetic comparison over explicitly supplied
// snapshots. This is a test oracle, not a kernel proof or a host-library axiom.
fn measure_truth(p: &ControlPredicate, before: i64, after: i64) -> bool {
    fn int(t: &ContractTerm, p: &ControlPredicate, before: i64, after: i64) -> i64 {
        let ContractTerm::Var { index, .. } = t else {
            panic!("measure variable")
        };
        let b = &p.bindings[*index];
        assert_eq!(b.kind, "current_slot");
        assert_eq!(b.value_id, "local:0");
        if b.edge_id.is_some() {
            after
        } else {
            before
        }
    }
    let ContractTerm::App {
        function, argument, ..
    } = &p.term
    else {
        panic!("comparison")
    };
    let ContractTerm::App {
        function: head,
        argument: left,
        ..
    } = function.as_ref()
    else {
        panic!("comparison")
    };
    let ContractTerm::Const { name, .. } = head.as_ref() else {
        panic!()
    };
    assert!(name.starts_with("Mpk.CSharp.Integer.MathLess."));
    int(left, p, before, after) < int(argument, p, before, after)
}
#[test]
fn csharp_03_t06_w04_total_partial_and_variable_snapshots() {
    let b = b();
    let requests = read("control-vc/measure-requests.json");
    let responses = read("control-vc/measure-responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 4);
    assert_eq!(responses.as_array().unwrap().len(), 4);
    for (r, response) in requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
    {
        assert_eq!(r["id"], response["id"]);
        let (context, captures) = support::replay_context(&b, r);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source);
        if r["id"] == "total_without_measure" {
            assert!(emitted.is_err());
            continue;
        }
        let emitted = emitted.unwrap();
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(src).unwrap();
        let p = vc.control_vcs();
        let l = &p.loops()[0];
        if r["id"] == "total_variable" {
            assert_eq!(p.functions()[0].entry_conditions.len(), 1);
            assert!(p.functions()[0].entry_conditions[0]
                .bindings
                .iter()
                .all(|b| b.kind == "entry_slot"));
            let post = p
                .sequents()
                .iter()
                .find(|s| s.kind == "method_return")
                .unwrap();
            assert!(post
                .goals
                .iter()
                .flat_map(|g| &g.bindings)
                .any(|b| b.kind == "result"));
            assert!(post
                .goals
                .iter()
                .flat_map(|g| &g.bindings)
                .any(|b| b.kind == "entry_slot"));
        }
        if r["id"] == "partial_without_measure" {
            assert_eq!(l.termination, "partial");
            assert!(l.decreases.is_empty());
            assert!(!p
                .definition_names()
                .iter()
                .any(|n| n.contains("MathLess") || n.contains("NonNegative")));
        } else {
            assert_eq!(l.termination, "total");
            assert_eq!(l.decreases.len(), 1);
            let s = p.sequents().iter().find(|s| s.kind == "backedge").unwrap();
            let goal = s.goals.last().unwrap();
            assert!(measure_truth(goal, 3, 2));
            assert!(!measure_truth(goal, 0, 1));
            assert!(!measure_truth(goal, 0, 0));
            if r["id"] == "total_descending" {
                for before in 1..=3 {
                    assert!(measure_truth(goal, before, before - 1));
                }
            }
            let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            let q = changed["sequents"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|s| s["kind"] == "backedge")
                .unwrap();
            for binding in q["goals"].as_array_mut().unwrap().last_mut().unwrap()["bindings"]
                .as_array_mut()
                .unwrap()
            {
                binding["edge_id"] = Value::Null;
            }
            assert!(import_csharp_practical_control_vcs(
                &super::construction::edited_bytes(&p.canonical_bytes(), &changed),
                src
            )
            .is_err());
        }
        assert_eq!(
            p.sequents()
                .iter()
                .filter(|s| s.kind == "method_return")
                .count(),
            emitted
                .vir()
                .functions()
                .iter()
                .flat_map(|f| &f.blocks)
                .filter(|b| matches!(b.node.abrupt, Some(AbruptCompletion::Return { .. })))
                .count()
        );
    }
}
