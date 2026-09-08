//! T06-W05 exact source-handler, outcome-state and contract reconstruction.
use crate::{b, read, support};
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
pub(super) fn predicates(p: &ExceptionVcProgram) -> Vec<&ControlPredicate> {
    p.sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .chain(p.functions().iter().flat_map(|f| {
            f.edges
                .iter()
                .map(|e| &e.edge.guard)
                .chain(&f.entry_conditions)
        }))
        .chain(p.searches().iter().flat_map(|s| {
            std::iter::once(&s.exhausted_guard).chain(s.candidates.iter().flat_map(|c| {
                std::iter::once(&c.selected_guard)
                    .chain(std::iter::once(&c.filter_evaluation_guard))
                    .chain(c.filter_failure_guard.iter())
            }))
        }))
        .collect()
}
pub(super) fn nodes(p: &ExceptionVcProgram) -> usize {
    predicates(p).iter().map(|p| p.term.nodes()).sum()
}
// The Boolean oracle varies actual filter observations independently. Closed
// type matching comes from the admitted finite ancestry table, never CLR objects.
fn evaluate(
    p: &ControlPredicate,
    ty: &str,
    outcomes: &std::collections::BTreeMap<String, FilterOutcome>,
    universe: &[ClosedExceptionArm],
) -> bool {
    fn go(
        t: &ContractTerm,
        p: &ControlPredicate,
        ty: &str,
        outcomes: &std::collections::BTreeMap<String, FilterOutcome>,
        universe: &[ClosedExceptionArm],
    ) -> bool {
        if let ContractTerm::Var { index, .. } = t {
            let b = &p.bindings[*index];
            return match b.kind.as_str() {
                "filter_result" => matches!(outcomes[&b.value_id], FilterOutcome::Boolean(true)),
                "filter_threw" => matches!(outcomes[&b.value_id], FilterOutcome::Threw(_)),
                _ => panic!("{}", b.kind),
            };
        }
        let mut args = vec![];
        let mut head = t;
        while let ContractTerm::App {
            function, argument, ..
        } = head
        {
            args.push(argument.as_ref());
            head = function;
        }
        args.reverse();
        let ContractTerm::Const { name, .. } = head else {
            panic!()
        };
        match name.as_str() {
            "Mpk.CSharp.Bool.true" => true,
            "Mpk.CSharp.Bool.false" => false,
            "Mpk.CSharp.Bool.Not" => !go(args[0], p, ty, outcomes, universe),
            "Mpk.CSharp.Bool.And" => {
                go(args[0], p, ty, outcomes, universe) && go(args[1], p, ty, outcomes, universe)
            }
            "Mpk.CSharp.Bool.Or" => {
                go(args[0], p, ty, outcomes, universe) || go(args[1], p, ty, outcomes, universe)
            }
            n if n.starts_with("Mpk.CSharp.Exception.IsType.") => universe
                .iter()
                .find(|a| a.type_id == ty)
                .unwrap()
                .ancestry
                .iter()
                .any(|a| Some(a.as_str()) == n.strip_prefix("Mpk.CSharp.Exception.IsType.")),
            n if n.starts_with("Mpk.CSharp.Exception.ExactType.") => {
                n.strip_prefix("Mpk.CSharp.Exception.ExactType.") == Some(ty)
            }
            _ => panic!("{name}"),
        }
    }
    go(&p.term, p, ty, outcomes, universe)
}
fn finally_checks(p: &ExceptionVcProgram) {
    use ExceptionCompletionKind::*;
    let incoming = [
        HandlerCompletion::Normal,
        HandlerCompletion::Return(Some(17)),
        HandlerCompletion::Break("loop.1".into()),
        HandlerCompletion::Continue("loop.2".into()),
        HandlerCompletion::Throw {
            exception_type: "System.ArgumentException".into(),
            value: 23,
        },
    ];
    assert_eq!(p.finally_rules().len(), 10);
    for (kind, value) in [Normal, Return, Break, Continue, Throw]
        .into_iter()
        .zip(incoming)
    {
        for produced in [Normal, Throw] {
            let rule = p
                .finally_rules()
                .iter()
                .find(|r| r.incoming == kind && r.produced == produced)
                .unwrap();
            let thrown = HandlerCompletion::Throw {
                exception_type: "System.InvalidOperationException".into(),
                value: 91,
            };
            let actual = complete_handler_finally(
                value.clone(),
                if produced == Normal {
                    HandlerCompletion::Normal
                } else {
                    thrown.clone()
                },
            )
            .unwrap();
            assert_eq!(
                actual,
                if rule.uses_incoming_value_and_target {
                    value.clone()
                } else {
                    thrown
                }
            );
            assert_eq!(rule.result, if produced == Normal { kind } else { Throw });
            assert_eq!(rule.restarts_search, produced == Throw);
        }
        for forbidden in [
            HandlerCompletion::Return(Some(18)),
            HandlerCompletion::Break("out".into()),
            HandlerCompletion::Continue("out".into()),
        ] {
            assert!(complete_handler_finally(value.clone(), forbidden).is_err());
        }
    }
}
fn search_checks(p: &ExceptionVcProgram) {
    for search in p.searches() {
        for arm in p.universe() {
            for mode in 0..(3 + search.candidates.len()) {
                let outcomes = search
                    .candidates
                    .iter()
                    .filter(|c| c.candidate.filter.is_some())
                    .map(|c| {
                        (
                            c.candidate.catch_id.clone(),
                            match mode {
                                0 => FilterOutcome::Boolean(false),
                                1 => FilterOutcome::Boolean(true),
                                2 => FilterOutcome::Threw("System.OverflowException".into()),
                                _ => FilterOutcome::Boolean(
                                    search.candidates[mode - 3].candidate.catch_id
                                        == c.candidate.catch_id,
                                ),
                            },
                        )
                    })
                    .collect();
                let selected = search
                    .candidates
                    .iter()
                    .filter(|c| evaluate(&c.selected_guard, &arm.type_id, &outcomes, p.universe()))
                    .collect::<Vec<_>>();
                let expected = search.candidates.iter().find(|c| {
                    arm.ancestry.contains(&c.candidate.type_id)
                        && (c.candidate.filter.is_none()
                            || matches!(
                                outcomes[&c.candidate.catch_id],
                                FilterOutcome::Boolean(true)
                            ))
                });
                assert_eq!(selected.len(), usize::from(expected.is_some()));
                if let Some(expected) = expected {
                    assert_eq!(selected[0].candidate.catch_id, expected.candidate.catch_id);
                }
                assert_eq!(
                    evaluate(
                        &search.exhausted_guard,
                        &arm.type_id,
                        &outcomes,
                        p.universe()
                    ),
                    selected.is_empty()
                );
            }
        }
    }
}
#[test]
fn csharp_03_t06_w05_original_exception_handlers_and_mutations() {
    let b = b();
    let cases = read("control-emission/source-cases.json");
    let mut goldens = vec![];
    for c in cases.as_array().unwrap().iter().filter(|c| {
        matches!(c["stage"].as_str(), Some("exceptions" | "handlers"))
            && !c["data"].is_null()
            && !c["data"]["control_lowering"]["functions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| !f["loops"].as_array().unwrap().is_empty())
    }) {
        let r = &c["source_case"];
        let id = r["id"].as_str().unwrap();
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
        let vc = generate_csharp_practical_vc(src).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let p = vc.exception_vcs();
        assert!(!p.functions().is_empty(), "{id}");
        for f in emitted.vir().functions() {
            for inv in f.blocks.iter().filter_map(|b| b.invocation.as_ref()) {
                let sig = vc
                    .operation_encodings()
                    .iter()
                    .find(|s| s.operation_id() == inv.operation_id)
                    .unwrap();
                if matches!(
                    sig.operation_tag(),
                    ClosedOperationTag::SourceCall | ClosedOperationTag::ConstructorExecute
                ) {
                    assert!(p
                        .functions()
                        .iter()
                        .find(|x| x.function_id == f.id)
                        .unwrap()
                        .calls
                        .contains(inv));
                }
            }
        }
        for predicate in predicates(p) {
            super::construction::assert_typed(
                &predicate.term,
                &predicate
                    .bindings
                    .iter()
                    .map(|b| b.type_id.clone())
                    .collect::<Vec<_>>(),
            );
        }
        for f in p.functions() {
            assert_eq!(
                f.edges
                    .iter()
                    .map(|e| &e.edge.id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                f.edges.len()
            );
            for e in f.edges.iter().filter(|e| e.edge.kind != "function_entry") {
                assert_eq!(
                    p.sequents()
                        .iter()
                        .filter(|s| s.function_id == f.function_id && s.id.ends_with(&e.edge.id))
                        .count(),
                    1
                );
                let native = f
                    .native
                    .blocks
                    .iter()
                    .find(|b| b.node.id == e.pre_node_id)
                    .unwrap();
                assert_eq!(e.ownership_before, native.ownership_in);
                assert_eq!(
                    e.ownership_after,
                    e.post_node_id
                        .as_ref()
                        .map(|id| &f
                            .native
                            .blocks
                            .iter()
                            .find(|b| &b.node.id == id)
                            .unwrap()
                            .ownership_in)
                        .unwrap_or(&native.ownership_out)
                        .clone()
                );
            }
        }
        search_checks(p);
        finally_checks(p);
        assert_eq!(
            import_csharp_practical_exception_vcs(&p.canonical_bytes(), src).unwrap(),
            *p
        );
        let value: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let mut mutations = vec![];
        let mut m = value.clone();
        m["functions"][0]["composition_rule"] = json!("unwind_before_filter");
        mutations.push(m);
        let mut m = value.clone();
        m["functions"][0]["edges"].as_array_mut().unwrap().remove(0);
        mutations.push(m);
        let mut m = value.clone();
        m["runtime_identity"] = json!("trusted");
        mutations.push(m);
        if let Some(i) = p.searches().iter().position(|s| !s.candidates.is_empty()) {
            let mut m = value.clone();
            m["searches"][i]["candidates"][0]["candidate"]["type_id"] = json!("System.Exception");
            mutations.push(m);
        }
        if id == "uncaught" {
            let mut m = value.clone();
            m["functions"][0]["native"]["exception_regions"]
                .as_array_mut()
                .unwrap()
                .clear();
            mutations.push(m);
            let mut m = value.clone();
            m["finally_rules"][0]["uses_incoming_value_and_target"] = json!(false);
            mutations.push(m);
            let index = p
                .sequents()
                .iter()
                .position(|s| s.kind == "uncaught_result")
                .expect("uncaught exit goal");
            let mut m = value.clone();
            m["sequents"][index]["goals"]
                .as_array_mut()
                .unwrap()
                .clear();
            mutations.push(m);
        }
        for m in mutations {
            assert!(
                import_csharp_practical_exception_vcs(
                    &super::construction::edited_bytes(&p.canonical_bytes(), &m),
                    src
                )
                .is_err(),
                "{id}"
            );
        }
        goldens.push(json!({"id":id,"stage":c["stage"],"exception_sha256":p.hash(),"functions":p.functions().len(),"searches":p.searches().len(),"sequents":p.sequents().len(),"kinds":p.sequents().iter().map(|s|s.kind.clone()).collect::<BTreeSet<_>>()}));
    }
    assert_eq!(goldens.len(), 46);
    if let Ok(path) = std::env::var("MPK_T06_W05_GOLDEN_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&goldens).unwrap()).unwrap();
    } else {
        assert_eq!(json!(goldens), read("exception-vc/goldens.json"));
    }
}

#[test]
fn csharp_03_t06_w05_exceptional_postconditions_are_goals() {
    let b = b();
    let requests = read("exception-vc/requests.json");
    let responses = read("exception-vc/responses.json");
    for r in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == r["id"])
            .unwrap();
        let (context, captures) = support::replay_context(&b, r);
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        })
        .unwrap();
        let p = vc.exception_vcs();
        let literals = emitted
            .vir()
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| d.tag == "literal")
            .map(|d| {
                let parameters: Value = serde_json::from_str(&d.parameters).unwrap();
                (d.name.clone(), parameters["value"].as_bool().unwrap())
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        fn expand(t: &mut ContractTerm, literals: &std::collections::BTreeMap<String, bool>) {
            match t {
                ContractTerm::Const { name, .. } => {
                    if let Some(v) = literals.get(name) {
                        *name = format!("Mpk.CSharp.Bool.{v}");
                    }
                }
                ContractTerm::App {
                    function, argument, ..
                } => {
                    expand(function, literals);
                    expand(argument, literals);
                }
                _ => {}
            }
        }
        let outcomes = p
            .sequents()
            .iter()
            .filter(|s| s.kind == "uncaught_result")
            .collect::<Vec<_>>();
        assert!(!outcomes.is_empty());
        for s in outcomes {
            let passes = s.goals.iter().all(|g| {
                let mut g = g.clone();
                expand(&mut g.term, &literals);
                evaluate(
                    &g,
                    "System.ArgumentException",
                    &Default::default(),
                    p.universe(),
                )
            });
            assert_eq!(passes, r["id"] == "allowed");
        }
    }
}
