use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::PathBuf;
fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/error-dispatch")
}
fn corpus() -> Value {
    serde_json::from_slice(&std::fs::read(directory().join("corpus.json")).unwrap()).unwrap()
}
fn unhex(v: &Value) -> Vec<u8> {
    v.as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn keys(c: &Value) -> GoTypeKeyComponents {
    let p = &c["input"];
    let sources = p["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(&s["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(&unhex(&p["components_hex"]), &sources).unwrap();
    assert_eq!(keys.types().len(), 1);
    keys
}
fn retain(name: &str, value: &Value) {
    if let Some(output) = std::env::var_os("MPK_GO_ERROR_DISPATCH_RUST_OUTPUT") {
        use std::io::Write;
        let p = PathBuf::from(output);
        std::fs::create_dir_all(&p).unwrap();
        let mut raw = serde_json::to_vec(value).unwrap();
        raw.push(b'\n');
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.join(name))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
}
fn locate(k: &GoTypeKeyComponents, v: &Value, kind: &str) -> GoSourceSelectionRef {
    let source = v["source"].as_str().unwrap();
    let parsed = k.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == kind
                && n.start() == v["start"].as_u64().unwrap() as usize
                && n.end() == v["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn rows(
    c: &Value,
    k: &GoTypeKeyComponents,
    test: &Value,
) -> (Vec<GoSourceSelectionRef>, Vec<GoErrorEntryConstraint>) {
    let input = &test["input"];
    let cases = input["entry_cases"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|name| {
                    locate(
                        k,
                        &c["declarations"]
                            [format!("example.com/policy.{}", name.as_str().unwrap())],
                        "TypeSpec",
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut roots = vec![];
    let mut entries = vec![];
    for row in input["roots"].as_array().unwrap() {
        let r = locate(k, &row["ref"], "FuncDecl");
        let name = row["name"].as_str().unwrap();
        roots.push(r.clone());
        if matches!(
            name,
            "Entry" | "Expr" | "Paren" | "Guarded" | "Branch" | "Imported" | "OtherRoot"
        ) {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![],
                if name == "OtherRoot" {
                    vec![]
                } else {
                    cases.clone()
                },
            ));
        }
        if name == "Nested" {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![GoErrorEntryStep::Field(0)],
                cases.clone(),
            ));
            entries.push(GoErrorEntryConstraint::new(
                r,
                0,
                vec![GoErrorEntryStep::Field(1), GoErrorEntryStep::Elements],
                vec![],
            ));
        }
    }
    (roots, entries)
}
fn reference(s: &GoSourceSelection, r: &GoSourceSelectionRef) -> Value {
    let p = &s.source(r.source_object_id()).unwrap().nodes()[r.node()];
    json!({"source":r.source_object_id(),"start":p.start(),"end":p.end()})
}
fn normalized(mut values: Vec<Value>) -> Value {
    values.sort_by_key(|v| serde_json::to_string(v).unwrap());
    json!(values)
}
fn actual(s: &GoSourceErrorSelection) -> Value {
    let selected = s.selection();
    let bodies = normalized(
        selected
            .declarations()
            .iter()
            .filter(|d| {
                d.kind() == "function"
                    && d.roles().any(|role| {
                        matches!(
                            role,
                            GoSourceSelectionRole::SelectedRoot
                                | GoSourceSelectionRole::BodyDependency
                        )
                    })
            })
            .map(|d| reference(selected, d.locator()))
            .collect(),
    );
    let calls=normalized(selected.calls().iter().map(|call|{let source=call.caller().source_object_id();let n=&selected.source(source).unwrap().nodes()[call.call_node()];json!({"caller":reference(selected,call.caller()),"callee":reference(selected,call.callee()),"source":source,"start":n.start(),"end":n.end()})}).collect());
    let dispatches=normalized(s.dispatches().iter().map(|d|{let call=d.call();let source=call.caller().source_object_id();let p=selected.source(source).unwrap();let n=&p.nodes()[call.call_node()];let receiver=&p.nodes()[call.receiver_node()];let targets=d.targets().iter().map(|t|{let m=t.method();let r=GoSourceSelectionRef::new(m.declaration_source_object_id().unwrap(),m.declaration_node().unwrap());json!({"type_id":t.value_type_id(),"callee":reference(selected,&r)})}).collect::<Vec<_>>();json!({"caller":reference(selected,call.caller()),"source":source,"start":n.start(),"end":n.end(),"receiver_start":receiver.start(),"receiver_end":receiver.end(),"method_expression":call.method_expression(),"targets":targets})}).collect());
    json!({"bodies":bodies,"calls":calls,"dispatches":dispatches,"global_cases":s.domain().error_cases().iter().map(|c|c.value_type_id()).collect::<Vec<_>>(),"global_reads":[],"nil_receiver_or_runtime_proof":false})
}
#[test]
fn matches_original_finite_targets_and_reaches_new_cases_until_fixed_point() {
    let c = corpus();
    retain("dispatch-input.json", &c["input"]);
    let k = keys(&c);
    let mut accepted = 0;
    let mut rejected = 0;
    let mut summaries = vec![];
    for (i, test) in c["cases"].as_array().unwrap().iter().enumerate() {
        let name = test["input"]["name"].as_str().unwrap();
        let (roots, entries) = rows(&c, &k, test);
        let result = derive_go_source_error_selection(&k, &roots, &entries);
        let reject = test["input"]["expected_rejection"].as_str().unwrap();
        if reject.is_empty() {
            let s = result.unwrap_or_else(|e| panic!("{name}: {e}"));
            let out = actual(&s);
            retain(&format!("case-{i:02}-actual.json"), &out);
            assert_eq!(out, test["native"], "{name}");
            if name == "seventy-growth-rounds" || name == "seventy-cases" {
                assert_eq!(s.domain().error_cases().len(), 70);
                assert_eq!(s.dispatches()[0].targets().len(), 70);
            }
            if name == "boxing-only" || name == "global-boxing-only" {
                assert!(s.dispatches().is_empty());
                assert_eq!(
                    s.selection()
                        .declarations()
                        .iter()
                        .filter(|d| d.kind() == "function"
                            && d.roles()
                                .any(|r| r == GoSourceSelectionRole::BodyDependency))
                        .count(),
                    0
                );
            }
            accepted += 1;
            summaries.push(json!({"name":name,"accepted":true,"global_cases":s.domain().error_cases().len(),"dispatches":s.dispatches().len()}));
        } else {
            let e = result.unwrap_err();
            retain(
                &format!("case-{i:02}-rejection.json"),
                &json!({"error":format!("{e:?}")}),
            );
            let GoSourceErrorSelectionError::Selection(error) = e else {
                panic!("{name}: expected original target-body rejection")
            };
            let kind = match error.kind() {
                GoSourceSelectionErrorKind::MutablePackageState => "MutablePackageState",
                GoSourceSelectionErrorKind::Body(_) => "Body",
                GoSourceSelectionErrorKind::CallCycle => "CallCycle",
                other => panic!("{name}: {other:?}"),
            };
            assert_eq!(kind, reject, "{name}");
            assert!(error.path().len() >= 2, "{name}: {:?}", error.path());
            if reject == "MutablePackageState" {
                let parsed = k
                    .source_declarations()
                    .source(error.source_object_id())
                    .unwrap();
                let n = &parsed.nodes()[error.node()];
                assert!(test["native"]["global_reads"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["source"] == error.source_object_id()
                        && r["start"] == n.start()
                        && r["end"] == n.end()));
            }
            rejected += 1;
            summaries.push(json!({"name":name,"accepted":false,"kind":kind}));
        }
    }
    assert_eq!((accepted, rejected), (19, 4));
    retain(
        "dispatch-summary.json",
        &json!({"accepted":accepted,"rejected":rejected,"cases":summaries,"seventy_growth_rounds":true,"uncertified_branches_do_not_prune_targets":true}),
    );
}
#[test]
fn entry_coverage_and_nil_safety_are_preserved_when_dispatch_has_no_targets() {
    let c = corpus();
    let k = keys(&c);
    let test = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["input"]["name"] == "nil-only")
        .unwrap();
    let (roots, entries) = rows(&c, &k, test);
    assert!(derive_go_source_selection(&k, &roots).is_err());
    let e = derive_go_source_error_selection(&k, &roots, &[]).unwrap_err();
    assert!(
        matches!(e,GoSourceErrorSelectionError::Domain(e)if *e.kind()==GoSourceDomainErrorKind::MissingEntry)
    );
    let s = derive_go_source_error_selection(&k, &roots, &entries).unwrap();
    assert_eq!(s.dispatches().len(), 1);
    assert!(s.dispatches()[0].targets().is_empty());
    let call = s.dispatches()[0].call();
    assert_eq!(
        s.selection()
            .source(call.caller().source_object_id())
            .unwrap()
            .nodes()[call.receiver_node()]
        .kind(),
        "Ident"
    );
    assert_eq!(s.selection().error_calls().len(), 1);
    retain(
        "dispatch-nil.json",
        &json!({"explicit_entry_required":true,"nil_only_call_retained":true,"non_nil_proof_discharged":false,"runtime_or_program_authority":false}),
    );
}
#[test]
fn global_dispatch_tables_and_source_ownership_are_canonical() {
    let c = corpus();
    let (a, b) = {
        let k = keys(&c);
        let test = c["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["input"]["name"] == "multiple-roots")
            .unwrap();
        let (mut roots, mut entries) = rows(&c, &k, test);
        let a = derive_go_source_error_selection(&k, &roots, &entries).unwrap();
        roots.reverse();
        entries.reverse();
        let b = derive_go_source_error_selection(&k, &roots, &entries).unwrap();
        (a, b)
    };
    assert_eq!(a, b);
    assert_eq!(a.dispatches().len(), 2);
    assert_eq!(a.dispatches()[0].targets(), a.dispatches()[1].targets());
    assert_eq!(a.domain().error_cases().len(), 3);
    for dispatch in a.dispatches() {
        assert!(a
            .selection()
            .source(dispatch.call().caller().source_object_id())
            .is_some());
    }
    retain(
        "dispatch-ownership.json",
        &json!({"owned_original_sources":true,"canonical_root_entry_order":true,"locally_nil_only_entry_uses_global_targets":true}),
    );
}
