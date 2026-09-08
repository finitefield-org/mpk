//! W06: original source bindings, closed definitions and hostile handoffs.
use crate::{b, read, support};
use mpk_vc::csharp_practical_vc_model::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn nodes(p: &BindingVcProgram) -> usize {
    p.sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .map(ContractTerm::nodes)
        .sum()
}
// Finite equation oracle, not a claim about a CLR body or a kernel receipt.
// Distinct countermodels break the projection and reconstruction independently.
fn equation(t: &ContractTerm, input: i64, broken_project: bool, broken_reconstruct: bool) -> i64 {
    if let ContractTerm::Var { index, .. } = t {
        assert_eq!(*index, 0);
        return input;
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
        panic!("unexpected binder")
    };
    let values = args
        .iter()
        .map(|t| equation(t, input, broken_project, broken_reconstruct))
        .collect::<Vec<_>>();
    if name.starts_with("binding.project.") {
        return values[0] + i64::from(broken_project);
    }
    if name.starts_with("binding.reconstruct.") {
        return values[0] + i64::from(broken_reconstruct);
    }
    if name.starts_with("Mpk.CSharp.Binding.ObserveEqual.")
        || name.starts_with("Mpk.CSharp.Binding.Equal.")
    {
        return i64::from(values[0] == values[1]);
    }
    if name.starts_with("Mpk.CSharp.PublicDomain.") {
        return 1;
    }
    panic!("unexpected equation {name}")
}
fn check(p: &BindingVcProgram) {
    let mut definitions = BTreeMap::new();
    fn concrete(t: &ContractTerm, defs: &mut BTreeMap<String, String>) {
        assert!(!t.type_id().contains(['<', '`']), "{}", t.type_id());
        match t {
            ContractTerm::Const { name, type_id } => {
                if let Some(old) = defs.insert(name.clone(), type_id.clone()) {
                    assert_eq!(old, *type_id, "overloaded {name}");
                }
            }
            ContractTerm::App {
                function, argument, ..
            } => {
                concrete(function, defs);
                concrete(argument, defs);
            }
            ContractTerm::Lam { body, .. } => concrete(body, defs),
            ContractTerm::Let { value, body, .. } => {
                concrete(value, defs);
                concrete(body, defs);
            }
            _ => {}
        }
    }
    for s in p.sequents() {
        for term in s.assumptions.iter().chain(&s.goals) {
            super::construction::assert_typed(
                term,
                &s.subjects
                    .iter()
                    .map(|v| v.type_id.clone())
                    .collect::<Vec<_>>(),
            );
            concrete(term, &mut definitions);
        }
    }
    assert_eq!(
        p.sequents()
            .iter()
            .map(|s| &s.id)
            .collect::<BTreeSet<_>>()
            .len(),
        p.sequents().len()
    );
    assert_eq!(
        definitions.keys().collect::<Vec<_>>(),
        p.definition_names().iter().collect::<Vec<_>>()
    );
    for r in p.representations() {
        for kind in [
            "projection_total",
            "reconstruction",
            "source_round_trip",
            "semantic_round_trip",
            "source_invariant",
            "identity_unobservable",
            "actual_default",
        ] {
            assert!(p
                .sequents()
                .iter()
                .any(|s| s.owner_id == r.projection.id && s.kind == kind));
        }
        assert_eq!(
            r.reconstruction_member_ids.len(),
            r.source["members"].as_array().unwrap().len()
        );
        for s in p.sequents().iter().filter(|s| {
            s.owner_id == r.projection.id
                && matches!(s.kind.as_str(), "source_round_trip" | "semantic_round_trip")
        }) {
            for value in [0, 1, 2] {
                assert_eq!(equation(&s.goals[0], value, false, false), 1);
                assert_eq!(equation(&s.goals[0], value, true, false), 0);
                assert_eq!(equation(&s.goals[0], value, false, true), 0);
            }
        }
    }
    for i in p.instances() {
        assert!(p
            .sequents()
            .iter()
            .any(|s| s.owner_id == i.instance_id && s.kind == "concrete_type_equivalence"));
        for op in &i.operation_definitions {
            assert!(p
                .sequents()
                .iter()
                .any(|s| s.owner_id == op["id"].as_str().unwrap()
                    && s.kind == "concrete_definition_equivalence"));
        }
    }
}
#[test]
fn csharp_03_t06_w06_all_binding_roles_and_closed_handoff_mutations() {
    let b = b();
    let requests = read("binding-vc/requests.json");
    let responses = read("binding-vc/responses.json");
    let mut goldens = vec![];
    let mut roles = BTreeSet::new();
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
        .unwrap_or_else(|e| panic!("{} import {e:?}", r["id"]));
        let emitted = emit_data_phase(&b, &context, &captures, &source)
            .unwrap_or_else(|e| panic!("{} emit {e:?}", r["id"]));
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc =
            generate_csharp_practical_vc(src).unwrap_or_else(|e| panic!("{} VC {e:?}", r["id"]));
        let p = vc.binding_vcs();
        check(p);
        for rep in p.representations() {
            roles.insert(rep.binding["role"].as_str().unwrap().to_owned());
        }
        assert!(!p.representations().is_empty());
        assert_eq!(
            import_csharp_practical_binding_vcs(&p.canonical_bytes(), src).unwrap(),
            *p
        );
        let wire: Value = serde_json::from_slice(emitted.vir().canonical_bytes()).unwrap();
        assert_eq!(p.instances().iter().map(|i|json!({"instance_id":i.instance_id,"type_definition":i.type_definition,"operation_definitions":i.operation_definitions})).collect::<Vec<_>>(),*wire["expanded_foundation"].as_array().unwrap());
        let v: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let mut mutations = vec![];
        let mut m = v.clone();
        m["representations"][0]["reconstruction_member_ids"]
            .as_array_mut()
            .unwrap()
            .clear();
        mutations.push(m);
        let mut m = v.clone();
        m["sequents"][0]["goals"].as_array_mut().unwrap().clear();
        mutations.push(m);
        let mut m = v.clone();
        m["representations"][0]["source"]["source_sha256"] = json!("0".repeat(64));
        mutations.push(m);
        if !p.instances().is_empty() {
            let mut m = v.clone();
            m["instances"][0]["operation_definitions"][0]["normal_result_type_id"] = json!("T");
            mutations.push(m);
            let mut m = v.clone();
            m["instances"].as_array_mut().unwrap().pop();
            mutations.push(m);
            let mut m = v.clone();
            m["counters"]["recipe_nodes"] = json!(0);
            mutations.push(m);
        }
        let mut m = v.clone();
        m["foundation_descriptor"]["content_sha256"] = json!("0".repeat(64));
        mutations.push(m);
        for m in mutations {
            assert!(
                import_csharp_practical_binding_vcs(
                    &super::construction::edited_bytes(&p.canonical_bytes(), &m),
                    src
                )
                .is_err(),
                "{}",
                r["id"]
            );
        }
        goldens.push(json!({"id":r["id"],"binding_sha256":p.hash(),"representations":p.representations().len(),"instances":p.instances().len(),"sequents":p.sequents().len()}));
    }
    assert_eq!(roles.len(), 12);
    if let Ok(path) = std::env::var("MPK_T06_W06_GOLDEN_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&goldens).unwrap()).unwrap();
    } else {
        assert_eq!(json!(goldens), read("binding-vc/goldens.json"));
    }
}
#[test]
fn csharp_03_t06_w06_original_returned_errors_and_rounding_commutations() {
    let b = b();
    let requests = read("data-phase/data-sidecar-requests.json");
    let responses = read("data-phase/data-sidecar-responses.json");
    let mut count = 0;
    for r in requests.as_array().unwrap().iter().filter(|r| {
        matches!(
            r["id"].as_str(),
            Some(
                "1d420cdfab591490568d24323e3474d89ce7d8ada62f3693a657647c6235afe5"
                    | "b6462c82b9eb05a4cfc242d1d4dbff193b39ec4f5c0a83f5e959c6457c73a2ec"
            )
        )
    }) {
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
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        };
        let vc = generate_csharp_practical_vc(src).unwrap();
        let p = vc.binding_vcs();
        check(p);
        let returned = p
            .representations()
            .iter()
            .flat_map(|r| &r.commutations)
            .filter_map(|c| c.returned_result.as_ref())
            .count();
        assert!(returned > 0);
        assert!(p
            .sequents()
            .iter()
            .any(|s| s.kind == "returned_error_commutation"));
        let value: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let index = p
            .sequents()
            .iter()
            .position(|s| s.kind == "operation_normal_commutation")
            .unwrap();
        let mut mutated = value;
        mutated["sequents"][index]["goals"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(import_csharp_practical_binding_vcs(
            &super::construction::edited_bytes(&p.canonical_bytes(), &mutated),
            src
        )
        .is_err());
        count += 1;
    }
    assert_eq!(count, 2);
}
