//! T06-W03 original frontend cross-checks and ordered ordinary sequents.
use crate::{b, read, support};
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn nodes(p: &DataVcProgram) -> usize {
    p.contracts()
        .iter()
        .map(|c| c.definedness.nodes())
        .sum::<usize>()
        + p.operations()
            .iter()
            .map(|o| {
                o.success_guard.nodes()
                    + o.success_relation.nodes()
                    + o.success_goal.nodes()
                    + o.checks
                        .iter()
                        .map(|c| {
                            c.prefix_guard.nodes()
                                + c.failure_predicate.nodes()
                                + c.failure_guard.nodes()
                                + c.static_goal.as_ref().map_or(0, ContractTerm::nodes)
                                + c.tagged_result_goal.as_ref().map_or(0, ContractTerm::nodes)
                        })
                        .sum::<usize>()
            })
            .sum::<usize>()
}
fn typed(t: &ContractTerm, subjects: &[TypedValueRef]) {
    match t {
        ContractTerm::Var { index, type_id } => assert_eq!(&subjects[*index].type_id, type_id),
        ContractTerm::Const { .. } => {}
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => {
            typed(function, subjects);
            typed(argument, subjects);
            assert_eq!(
                function.type_id(),
                format!("({}->{type_id})", argument.type_id())
            );
        }
        _ => panic!("unexpected sequent binder"),
    }
}
// A truth-table proof of the ordinary guard composition. Semantic predicates
// are independently chosen atoms here, not claims about a host/BCL evaluation.
fn boolean(t: &ContractTerm, atoms: &BTreeMap<String, bool>) -> bool {
    let mut args = vec![];
    let mut f = t;
    while let ContractTerm::App {
        function, argument, ..
    } = f
    {
        args.push(argument.as_ref());
        f = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = f else {
        panic!("predicate head")
    };
    match name.as_str() {
        "Mpk.CSharp.Bool.true" => true,
        "Mpk.CSharp.Bool.false" => false,
        "Mpk.CSharp.Bool.Not" => !boolean(args[0], atoms),
        "Mpk.CSharp.Bool.And" => boolean(args[0], atoms) && boolean(args[1], atoms),
        "Mpk.CSharp.Bool.Or" => boolean(args[0], atoms) || boolean(args[1], atoms),
        _ => atoms[name],
    }
}
fn check_rows(p: &DataVcProgram) {
    for o in p.operations() {
        let d = p
            .definitions()
            .iter()
            .find(|d| d.id == o.definition_id)
            .unwrap();
        assert_eq!(
            o.checks.iter().map(|c| &c.check).collect::<Vec<_>>(),
            d.signature.ordered_checks.iter().collect::<Vec<_>>()
        );
        let mut atoms = d
            .failure_names
            .iter()
            .map(|n| (n.clone(), false))
            .collect::<BTreeMap<_, _>>();
        atoms.insert(d.relation_name.clone(), true);
        for n in d.failure_result_names.iter().flatten() {
            atoms.insert(n.clone(), true);
        }
        for t in [&o.success_guard, &o.success_relation, &o.success_goal] {
            typed(t, &o.subjects);
            assert!(boolean(t, &atoms));
        }
        atoms.insert(d.relation_name.clone(), false);
        assert!(
            !boolean(&o.success_goal, &atoms),
            "missing normal relation {}",
            d.signature.id
        );
        atoms.insert(d.relation_name.clone(), true);
        for (i, c) in o.checks.iter().enumerate() {
            typed(&c.failure_predicate, &o.subjects[..o.subjects.len() - 1]);
            typed(&c.prefix_guard, &o.subjects[..o.subjects.len() - 1]);
            for t in [&c.prefix_guard, &c.failure_predicate, &c.failure_guard]
                .into_iter()
                .chain(c.static_goal.iter())
                .chain(c.tagged_result_goal.iter())
            {
                typed(t, &o.subjects);
            }
            // Both i and every later failure hold: exactly i must win.
            for (j, n) in d.failure_names.iter().enumerate() {
                atoms.insert(n.clone(), j >= i);
            }
            assert!(boolean(&c.prefix_guard, &atoms));
            assert!(boolean(&c.failure_guard, &atoms));
            assert!(!boolean(&o.success_guard, &atoms));
            for (j, later) in o.checks.iter().enumerate() {
                assert_eq!(boolean(&later.failure_guard, &atoms), j == i);
            }
            if let Some(goal) = &c.static_goal {
                assert!(!boolean(goal, &atoms));
            }
            if let Some(goal) = &c.tagged_result_goal {
                assert!(boolean(goal, &atoms));
                let n = d.failure_result_names[i].as_ref().unwrap();
                atoms.insert(n.clone(), false);
                assert!(!boolean(goal, &atoms));
                atoms.insert(n.clone(), true);
            }
            assert_eq!(
                c.exceptional_successor.is_some(),
                c.check.tag == RequiredCheckTag::Exception
            );
        }
    }
}
#[test]
fn csharp_03_t06_w03_replay_semantic_rows_and_ordering() {
    let b = b();
    let rows = read("data-phase/data-stage-replay.json");
    let mut seen = BTreeSet::new();
    let mut count = 0;
    let mut summary = vec![];
    for row in rows.as_array().unwrap() {
        let Some(facts) = row["outcome"].get("facts") else {
            continue;
        };
        let keys = facts["callables"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["data_steps"].as_array().unwrap().iter())
            .map(|d| format!("{}:{}", d["family"], d["operation"]))
            .collect::<BTreeSet<_>>();
        if keys.is_empty() || keys.is_subset(&seen) {
            continue;
        }
        let (context, captures) = support::replay_context(&b, row);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap();
        let Ok(emitted) = emit_data_phase(&b, &context, &captures, &source) else {
            continue;
        };
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(src)
            .unwrap_or_else(|e| panic!("{} {keys:?}: {e:?}", row["id"]));
        let p = vc.data_vcs();
        check_rows(p);
        for c in p.contracts() {
            super::construction::assert_typed(
                &c.definedness,
                &c.subjects
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect::<Vec<_>>(),
            );
        }
        assert_eq!(
            import_csharp_practical_data_vcs(&p.canonical_bytes(), src).unwrap(),
            *p
        );
        for o in p.operations() {
            let inv = emitted
                .vir()
                .functions()
                .iter()
                .find(|f| f.id == o.function_id)
                .unwrap()
                .blocks
                .iter()
                .find(|b| b.node.id == o.node_id)
                .unwrap()
                .invocation
                .as_ref()
                .unwrap();
            assert_eq!(&o.subjects[..o.subjects.len() - 1], inv.operands);
            assert_eq!(o.subjects.last().unwrap(), &inv.result);
            assert_eq!(o.normal_successor_id, inv.normal_successor_id);
            let group = vc
                .obligation_groups()
                .iter()
                .find(|g| g.subject_ids().contains(&o.id))
                .unwrap();
            assert!(group
                .dependencies()
                .iter()
                .any(|s| s == "vc.group.0300.global"));
        }
        summary.push(json!({"capture_id":row["id"],"rows":keys,"data_sha256":p.hash(),"operations":p.operations().len(),"checks":p.operations().iter().map(|o|o.checks.len()).sum::<usize>()}));
        seen.extend(keys);
        count += 1;
    }
    assert_eq!(count, 208);
    assert_eq!(seen.len(), 218);
    if let Ok(path) = std::env::var("MPK_T06_W03_GOLDEN_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    } else {
        assert_eq!(json!(summary), read("data-vc/goldens.json"));
    }
    eprintln!(
        "data VC coverage: {} rows / {count} original captures",
        seen.len()
    );
}

#[test]
fn csharp_03_t06_w03_handoff_rejects_semantic_and_edge_mutations() {
    let b = b();
    let rows = read("data-phase/data-stage-replay.json");
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["outcome"]["facts"]["callables"]
                .as_array()
                .is_some_and(|cs| {
                    cs.iter().any(|c| {
                        c["data_steps"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|d| d["operation"] == "decimal.divide")
                    })
                })
        })
        .unwrap();
    let (context, captures) = support::replay_context(&b, row);
    let source = ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&row["outcome"]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
    let src = PracticalVcSource {
        artifact_context: &context,
        captured_inputs: &captures,
        vir: emitted.vir(),
    };
    let vc = generate_csharp_practical_vc(src).unwrap();
    let bytes = vc.data_vcs().canonical_bytes();
    let original: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(super::construction::edited_bytes(&bytes, &original), bytes);
    assert!(import_csharp_practical_data_vcs(&bytes, src).is_ok());
    let index = vc
        .data_vcs()
        .operations()
        .iter()
        .position(|o| o.checks.len() == 2)
        .unwrap();
    let mut variants = vec![];
    let mut v = original.clone();
    v["operations"][index]["checks"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    variants.push(v);
    let mut v = original.clone();
    v["operations"][index]["checks"][1]["prefix_guard"] =
        json!({"form":"const","name":"Mpk.CSharp.Bool.true","type_id":"mpk.csharp.value.bool.v1"});
    variants.push(v);
    let mut v = original.clone();
    v["operations"][index]["checks"][0]["exceptional_successor"]["target_id"] =
        json!("other.target");
    variants.push(v);
    let mut v = original.clone();
    v["operations"][index]["subjects"][0]["id"] = json!("other.operand");
    variants.push(v);
    let mut v = original.clone();
    v["operations"][index]["success_goal"] =
        json!({"form":"const","name":"Mpk.CSharp.Bool.true","type_id":"mpk.csharp.value.bool.v1"});
    variants.push(v);
    let mut v = original.clone();
    v["operations"].as_array_mut().unwrap().remove(index);
    variants.push(v);
    let mut v = original.clone();
    v["closed_set_sha256"] = json!("0".repeat(64));
    variants.push(v);
    let mut v = original.clone();
    v["definitions"][0]["carrier_definitions"][0] = json!({"axiom":true});
    variants.push(v);
    let mut v = original.clone();
    v["definition_names"] = json!([]);
    variants.push(v);
    for v in variants {
        assert!(import_csharp_practical_data_vcs(
            &super::construction::edited_bytes(&bytes, &v),
            src
        )
        .is_err());
    }
}
