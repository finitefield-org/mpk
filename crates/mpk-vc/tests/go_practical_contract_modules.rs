use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/contract-modules")
}
fn corpus() -> Value {
    serde_json::from_slice(&std::fs::read(directory().join("corpus-receivers.json")).unwrap())
        .unwrap()
}
fn unhex(s: &Value) -> Vec<u8> {
    s.as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn keys(c: &Value) -> GoTypeKeyComponents {
    let v = &c["input"];
    let sources = v["sources"]
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
    let k = import_go_type_key_components(&unhex(&v["components_hex"]), &sources).unwrap();
    assert_eq!(k.types().len(), 1);
    assert!(k.declarations().is_empty());
    k
}
fn selector(name: &str) -> GoSourceFunctionSelector {
    serde_json::from_value(json!({"package":"example.com/policy","function":name,"receiver":null}))
        .unwrap()
}
fn roots(row: &Value) -> Vec<GoSourceFunctionSelector> {
    serde_json::from_value(row["roots"].clone()).unwrap()
}
fn overlays(row: &Value) -> Vec<GoSourceOverlay> {
    row["overlays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            let source = GoTypeSourceBuffer::import(
                &serde_json::to_vec(&o["object"]).unwrap(),
                &unhex(&o["raw_hex"]),
            )
            .unwrap();
            GoSourceOverlay::import(&source).unwrap()
        })
        .collect()
}
fn case<'a>(c: &'a Value, name: &str) -> &'a Value {
    c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
}
fn module(
    k: &GoTypeKeyComponents,
    row: &Value,
) -> Result<GoSourceContractModule, GoContractTypingError> {
    derive_go_source_contract_module(k, &roots(row), &overlays(row))
}
fn boolean(v: bool) -> Value {
    json!({"op":"bool.literal","value":v})
}
fn nat(v: &str) -> Value {
    json!({"op":"nat.literal","value":v})
}
fn raw(name: &str) -> Value {
    json!({"schema":"mpk.go.practical.source_overlay.v1","source":"policy.go","function":{"package":"example.com/policy","function":name,"receiver":null},"termination":"partial","requires":[],"ensures":[boolean(true)],"error_inputs":[],"loops":[],"modifies":[]})
}
fn overlay(path: &str, v: &Value) -> GoSourceOverlay {
    let source = GoTypeSourceBuffer::new(
        "candidates",
        path,
        "overlay",
        &serde_json::to_vec(v).unwrap(),
    )
    .unwrap();
    GoSourceOverlay::import(&source).unwrap()
}
fn retain(name: &str, v: &Value) {
    if let Some(p) = std::env::var_os("MPK_GO_CONTRACT_MODULES_RUST_OUTPUT") {
        use std::io::Write;
        let p = PathBuf::from(p);
        std::fs::create_dir_all(&p).unwrap();
        let mut bytes = serde_json::to_vec(v).unwrap();
        bytes.push(b'\n');
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p.join(name))
            .unwrap()
            .write_all(&bytes)
            .unwrap();
    }
}
fn reference(s: &GoSourceSelection, r: &GoSourceSelectionRef) -> Value {
    let n = &s.source(r.source_object_id()).unwrap().nodes()[r.node()];
    json!({"source":r.source_object_id(),"start":n.start(),"end":n.end()})
}
fn normalized(mut v: Vec<Value>) -> Value {
    v.sort_by_key(|v| serde_json::to_string(v).unwrap());
    json!(v)
}
fn actual(m: &GoSourceContractModule) -> Value {
    let s = m.selection();
    let bodies = normalized(
        m.functions()
            .iter()
            .map(|f| reference(s, f.function()))
            .collect(),
    );
    let calls=normalized(s.calls().iter().map(|c|{let source=c.caller().source_object_id();let n=&s.source(source).unwrap().nodes()[c.call_node()];json!({"caller":reference(s,c.caller()),"callee":reference(s,c.callee()),"source":source,"start":n.start(),"end":n.end()})}).collect());
    let dispatches=normalized(m.finite_selection().dispatches().iter().map(|d|{
        let c=d.call();let source=c.caller().source_object_id();let p=s.source(source).unwrap();let n=&p.nodes()[c.call_node()];let receiver=&p.nodes()[c.receiver_node()];
        let targets=d.targets().iter().map(|t|{let method=t.method();let r=GoSourceSelectionRef::new(method.declaration_source_object_id().unwrap(),method.declaration_node().unwrap());json!({"type_id":t.value_type_id(),"callee":reference(s,&r)})}).collect::<Vec<_>>();
        json!({"caller":reference(s,c.caller()),"source":source,"start":n.start(),"end":n.end(),"receiver_start":receiver.start(),"receiver_end":receiver.end(),"method_expression":c.method_expression(),"targets":targets})
    }).collect());
    json!({"bodies":bodies,"calls":calls,"dispatches":dispatches,"global_cases":m.value_domain().error_cases().iter().map(|e|e.value_type_id()).collect::<Vec<_>>(),"global_reads":[],"nil_receiver_or_runtime_proof":false})
}
fn named<'a>(m: &'a GoSourceContractModule, name: &str) -> &'a GoSourceFunctionContract {
    m.functions()
        .iter()
        .find(|f| f.signature().name() == Some(name))
        .unwrap()
}
fn assert_default(f: &GoSourceFunctionContract, universe: &[String]) {
    assert_eq!(f.origin(), &GoSourceContractOrigin::Default);
    assert_eq!(f.overlay_object_id(), None);
    assert_eq!(f.termination(), "partial");
    assert!(f.requires().is_empty());
    assert_eq!(f.ensures().len(), 1);
    assert_eq!(f.ensures()[0].operator(), "bool.literal");
    assert_eq!(f.ensures()[0].literal(), Some(&Value::Bool(true)));
    assert_eq!(
        f.ensures()[0].expression_type(),
        &GoContractExpressionType::Bool
    );
    for e in f.error_inputs() {
        assert_eq!(e.case_type_ids(), universe);
    }
    for l in f.loops() {
        assert_eq!(
            l.invariant_origin(),
            GoSourceLoopCandidateOrigin::GeneratedInvariant
        );
        assert_eq!(
            l.typing().invariants()[0].literal(),
            Some(&Value::Bool(true))
        );
        assert!(l.typing().variant().is_none());
        assert!(!l.pending_total_variant());
    }
}

#[test]
fn compares_all_original_body_closures_and_complete_default_entry_candidates() {
    let c = corpus();
    let k = keys(&c);
    retain("input.json", &c["input"]);
    let mut rows = vec![];
    for (i, row) in c["cases"].as_array().unwrap().iter().enumerate() {
        let name = row["name"].as_str().unwrap();
        let result = module(&k, row);
        let rejection = row["expected_profile_rejection"].as_str().unwrap();
        if rejection.is_empty() {
            let m = result.unwrap_or_else(|e| panic!("{name}: {e}"));
            let view = actual(&m);
            assert_eq!(view, row["native"], "{name}");
            assert_eq!(m.selected_roots().len(), roots(row).len());
            let u = m
                .value_domain()
                .error_cases()
                .iter()
                .map(|e| e.value_type_id().to_string())
                .collect::<Vec<_>>();
            for f in m.functions() {
                if f.overlay_object_id().is_none() {
                    assert_default(f, &u);
                }
            }
            for raw in m.captured_overlays() {
                let original = row["overlays"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|o| o["object"]["id"] == raw.overlay().source_buffer().object().id)
                    .unwrap();
                assert_eq!(
                    raw.overlay().source_buffer().bytes(),
                    unhex(&original["raw_hex"])
                );
            }
            if name == "growth-seventy" {
                assert_eq!(u.len(), 70);
                assert_eq!(m.functions().len(), 141);
                assert_eq!(m.annotation_typing().overlays().len(), 69);
            }
            if name == "many-helpers" {
                assert_eq!(m.functions().len(), 513);
                assert_eq!(m.expression_nodes(), 513);
            }
            if name == "defaults" {
                assert!(u.is_empty());
                assert_eq!(named(&m, "Root").error_inputs().len(), 2);
                assert!(m.finite_selection().dispatches()[0].targets().is_empty());
            }
            if name == "receiver-alias" {
                assert_eq!(named(&m, "Entry").error_inputs()[0].input_position(), 1);
                assert_eq!(
                    named(&m, "Entry").signature().slots()[0].role(),
                    GoSourceSignatureRole::Receiver
                );
            }
            if name == "root-annotation" {
                let root = named(&m, "Root");
                assert_eq!(root.error_inputs()[0].case_type_ids().len(), 1);
                assert_eq!(root.error_inputs()[1].case_type_ids().len(), 1);
                assert_eq!(
                    named(&m, "Worker").error_inputs()[0].case_type_ids().len(),
                    2
                );
            }
            if name == "callee-annotation" {
                assert!(named(&m, "Root").is_selected_root());
                assert!(!named(&m, "Worker").is_selected_root());
                assert_eq!(
                    named(&m, "Root").error_inputs()[0].path()[0].step(),
                    &GoErrorEntryStep::Elements
                );
                assert!(named(&m, "Root").error_inputs()[0].path()[0].uses_capacity());
            }
            retain(&format!("case-{i:02}-actual.json"), &view);
            rows.push(json!({"name":name,"functions":m.functions().len(),"roots":m.selected_roots().len(),"cases":u.len(),"captured_overlays":m.captured_overlays().len(),"expression_nodes":m.expression_nodes()}));
        } else {
            let err = result.unwrap_err();
            let GoContractTypingErrorKind::Overlay(e) = err.kind() else {
                panic!("{name}: {err}")
            };
            let GoSourceOverlayErrorKind::Selection(e) = e.kind() else {
                panic!("{name}: {err}")
            };
            match (rejection, e.as_ref()) {
                ("selection", GoSourceErrorSelectionError::Selection(e)) => {
                    if name == "cycle" {
                        assert_eq!(e.kind(), &GoSourceSelectionErrorKind::CallCycle);
                    }
                    if name == "target-effect" {
                        assert_eq!(e.kind(), &GoSourceSelectionErrorKind::MutablePackageState);
                        assert!(e.path().len() >= 3);
                        let n = &k
                            .source_declarations()
                            .source(e.source_object_id())
                            .unwrap()
                            .nodes()[e.node()];
                        assert!(row["native"]["global_reads"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|r| r["source"] == e.source_object_id()
                                && r["start"] == n.start()
                                && r["end"] == n.end()));
                    }
                }
                ("domain", GoSourceErrorSelectionError::Domain(e)) => {
                    assert_eq!(e.kind(), &GoSourceDomainErrorKind::NonPlainBacking)
                }
                _ => panic!("{name}: {err}"),
            }
            retain(
                &format!("case-{i:02}-rejection.json"),
                &json!({"error":format!("{err:?}")}),
            );
            rows.push(json!({"name":name,"rejected":rejection}));
        }
    }
    retain("module-cases.json", &json!(rows));
}

#[test]
fn preserves_actual_raw_objects_and_excludes_unselected_and_signature_only_annotations() {
    let c = corpus();
    let k = keys(&c);
    for name in ["captured-unselected", "signature-only"] {
        let m = module(&k, case(&c, name)).unwrap();
        assert_eq!(
            m.captured_overlays().len(),
            if name == "captured-unselected" { 2 } else { 1 }
        );
        for captured in m.captured_overlays() {
            assert!(!captured.in_body_closure());
            assert!(m.annotation_typing().overlays().is_empty());
            assert!(m.annotation_typing().type_dependencies().is_empty());
            assert!(!m
                .functions()
                .iter()
                .any(|f| f.function() == captured.function()));
            if name == "signature-only" {
                assert_eq!(m.functions().len(), 1);
                assert_eq!(m.value_domain().error_cases().len(), 1);
                assert!(m.finite_selection().dispatches().is_empty());
                assert!(m
                    .selection()
                    .declarations()
                    .iter()
                    .find(|d| d.locator() == captured.function())
                    .unwrap()
                    .roles()
                    .any(|r| r == GoSourceSelectionRole::SignatureDependency));
            }
        }
    }
    let row = case(&c, "multiple-roots");
    let mut roots = roots(row);
    let mut source = overlays(row);
    let a = derive_go_source_contract_module(&k, &roots, &source).unwrap();
    roots.reverse();
    source.reverse();
    let b = derive_go_source_contract_module(&k, &roots, &source).unwrap();
    assert_eq!(actual(&a), actual(&b));
    assert_eq!(a.selected_roots(), b.selected_roots());
    let original = a.functions()[0].function().source_object_id().to_string();
    drop(k);
    assert!(a.functions()[0]
        .signature()
        .types()
        .source(&original)
        .is_some());
    retain(
        "ownership.json",
        &json!({"nullable_default_overlay_id":true,"unselected_raw_retained":true,"signature_only_body_unselected":true,"root_and_overlay_order_canonical":true,"original_sources_owned":true}),
    );
}

fn loop_overlay(k: &GoTypeKeyComponents, v: &Value, ordinal: usize, variant: Value) -> Value {
    let b = bind_go_source_overlay(k, &overlay("loops.json", v)).unwrap();
    let l = &b.loops()[ordinal];
    let n = &b.source(b.root().source_object_id()).unwrap().nodes()[l.node()];
    let mut v = v.clone();
    v["loops"] = json!([{"selector":{"kind":if n.kind()=="RangeStmt"{"range"}else{"for"},"start_byte":n.start(),"end_byte":n.end(),"ordinal":ordinal},"invariants":[boolean(true)],"variant":variant}]);
    v
}
#[test]
fn distinguishes_supplied_and_generated_loop_candidates_without_assuming_totality() {
    let c = corpus();
    let k = keys(&c);
    let row = case(&c, "total-loops");
    let m = module(&k, row).unwrap();
    let f = named(&m, "Loops");
    assert_eq!(f.termination(), "total");
    assert!(f.loops()[0].pending_total_variant());
    assert!(f.loops()[0].typing().variant().is_none());
    let range = &f.loops()[1];
    assert!(!range.pending_total_variant());
    assert_eq!(
        range.variant_origin(),
        Some(GoSourceLoopCandidateOrigin::GeneratedRange)
    );
    let rank = range.typing().variant().unwrap();
    assert_eq!(rank.operator(), "nat.sub");
    assert_eq!(rank.expression_type(), &GoContractExpressionType::Nat);
    assert_eq!(rank.children()[0].operator(), "range.length");
    assert_eq!(rank.children()[1].operator(), "range.index");
    for e in rank.children() {
        assert_eq!(
            e.observation(),
            Some(GoContractObservation::Header {
                loop_node: range.typing().node()
            })
        );
    }
    let raw_overlay: Value =
        serde_json::from_slice(overlays(row)[0].source_buffer().bytes()).unwrap();
    let absent = loop_overlay(&k, &raw_overlay, 1, json!({"kind":"absent"}));
    let absent = derive_go_source_contract_module(
        &k,
        &[selector("Loops")],
        &[overlay("absent.json", &absent)],
    )
    .unwrap();
    assert_eq!(
        named(&absent, "Loops").loops()[1].invariant_origin(),
        GoSourceLoopCandidateOrigin::Supplied
    );
    assert_eq!(
        named(&absent, "Loops").loops()[1].variant_origin(),
        Some(GoSourceLoopCandidateOrigin::GeneratedRange)
    );
    let supplied = loop_overlay(
        &k,
        &raw_overlay,
        1,
        json!({"kind":"candidate","expression":nat("17")}),
    );
    let supplied = derive_go_source_contract_module(
        &k,
        &[selector("Loops")],
        &[overlay("supplied.json", &supplied)],
    )
    .unwrap();
    assert_eq!(
        named(&supplied, "Loops").loops()[1].variant_origin(),
        Some(GoSourceLoopCandidateOrigin::Supplied)
    );
    assert_eq!(
        named(&supplied, "Loops").loops()[1]
            .typing()
            .variant()
            .unwrap()
            .literal(),
        Some(&json!("17"))
    );
    // A total caller does not relabel default callees as total.
    let mut root = raw("Root");
    root["termination"] = json!("total");
    root["error_inputs"] = json!([{"input":0,"path":[{"kind":"elements"}],"cases":[]},{"input":1,"path":[],"cases":[]}]);
    let m = derive_go_source_contract_module(
        &k,
        &[selector("Root")],
        &[overlay("total-root.json", &root)],
    )
    .unwrap();
    assert_eq!(named(&m, "Root").termination(), "total");
    assert_eq!(named(&m, "Worker").termination(), "partial");
    retain(
        "loop-candidates.json",
        &json!({"for_missing_rank_pending":true,"range_rank_generated":true,"supplied_rank_preserved":true,"callee_totality_promoted":false,"proofs_discharged":false}),
    );
}

#[test]
fn rejects_crossed_selectors_duplicate_alias_roots_and_invalid_selected_annotations() {
    let c = corpus();
    let k = keys(&c);
    let row = case(&c, "receiver-alias");
    let alias = roots(row)[0].clone();
    let named:GoSourceFunctionSelector=serde_json::from_value(json!({"package":"example.com/policy","function":"Entry","receiver":{"package":"example.com/policy","type_name":"Recv"}})).unwrap();
    let err = derive_go_source_contract_module(&k, &[alias, named], &overlays(row)).unwrap_err();
    let GoContractTypingErrorKind::Overlay(e) = err.kind() else {
        panic!("{err}")
    };
    let GoSourceOverlayErrorKind::Selection(e) = e.kind() else {
        panic!("{err}")
    };
    assert!(
        matches!(e.as_ref(),GoSourceErrorSelectionError::Selection(e)if e.kind()==&GoSourceSelectionErrorKind::DuplicateRoot)
    );
    assert!(derive_go_source_contract_module(&k, &[], &[]).is_err());
    assert!(derive_go_source_contract_module(&k, &[selector("Absent")], &[]).is_err());
    let mut duplicate = raw("Loops");
    let a = overlay("a.json", &duplicate);
    duplicate["termination"] = json!("total");
    let b = overlay("b.json", &duplicate);
    let err = derive_go_source_contract_module(&k, &[selector("Loops")], &[a, b]).unwrap_err();
    assert!(
        matches!(err.kind(),GoContractTypingErrorKind::Overlay(e)if e.kind()==&GoSourceOverlayErrorKind::DuplicateOverlay)
    );
    let a = overlay("a.json", &raw("Loops"));
    let mut another = raw("BoundLoop");
    let b = overlay("a.json", &another);
    assert!(derive_go_source_contract_module(&k, &[selector("Loops")], &[a, b]).is_err());
    another["source"] = json!("absent.go");
    assert!(derive_go_source_contract_module(
        &k,
        &[selector("Loops")],
        &[overlay("crossed.json", &another)]
    )
    .is_err());
    let mut worker = raw("Worker");
    let err = derive_go_source_contract_module(
        &k,
        &[selector("Root")],
        &[overlay("missing.json", &worker)],
    )
    .unwrap_err();
    assert!(
        matches!(err.kind(),GoContractTypingErrorKind::Overlay(e)if e.kind()==&GoSourceOverlayErrorKind::MissingEntry)
    );
    worker["error_inputs"] = json!([{"input":0,"path":[],"cases":["example.com/policy.First"]}]);
    worker["requires"] = json!([{"op":"local","name":"ghost"}]);
    let ghost = overlay("ghost.json", &worker);
    let id = ghost.source_buffer().object().id.clone();
    let err = derive_go_source_contract_module(&k, &[selector("Root")], &[ghost]).unwrap_err();
    assert_eq!(err.kind(), &GoContractTypingErrorKind::Scope);
    assert_eq!(err.overlay_object_id(), Some(id.as_str()));
    assert_eq!(err.phase(), Some(GoContractPhase::Requires));
    worker["requires"] = json!([{"op":"error.is","value":{"op":"input","position":0},"case":"example.com/policy.Second"}]);
    let err = derive_go_source_contract_module(
        &k,
        &[selector("Root")],
        &[overlay("universe.json", &worker)],
    )
    .unwrap_err();
    assert_eq!(err.kind(), &GoContractTypingErrorKind::Case);
    retain(
        "invalid-annotations.json",
        &json!({"alias_duplicate_roots_rejected":true,"captured_namespace_and_selectors_exact":true,"missing_selected_entry_rejected":true,"selected_callee_expression_scope_checked":true,"clause_case_does_not_seed_U":true}),
    );
}

fn balanced(leaves: usize) -> Value {
    if leaves == 1 {
        boolean(true)
    } else {
        json!({"op":"bool.and","args":[balanced(leaves/2),balanced(leaves-leaves/2)]})
    }
}
#[test]
fn checks_complete_generated_function_and_module_expression_budgets() {
    let c = corpus();
    let k = keys(&c);
    let tree = balanced(512);
    let source = (0..8)
        .map(|i| {
            let mut v = raw(&format!("Bound{i}"));
            v["requires"] = json!([tree]);
            overlay(&format!("bound-{i}.json"), &v)
        })
        .collect::<Vec<_>>();
    let m = derive_go_source_contract_module(&k, &[selector("Bound0")], &source).unwrap();
    assert_eq!(m.functions().len(), 8);
    assert_eq!(m.expression_nodes(), 8192);
    let err =
        derive_go_source_contract_module(&k, &[selector("Bound0"), selector("BoxOnly")], &source)
            .unwrap_err();
    assert_eq!(err.kind(), &GoContractTypingErrorKind::Limit);
    let mut v = raw("BoundLoop");
    v["requires"] = json!([tree]);
    let v = overlay("generated.json", &v);
    assert_eq!(v.expression_nodes(), 1024);
    let err = derive_go_source_contract_module(&k, &[selector("BoundLoop")], &[v]).unwrap_err();
    assert_eq!(err.kind(), &GoContractTypingErrorKind::Limit);
    let unannotated = module(&k, case(&c, "generated-budget")).unwrap();
    assert_eq!(unannotated.expression_nodes(), 2);
    retain(
        "expression-budgets.json",
        &json!({"exact_module_nodes":8192,"per_function_including_generated":1024,"one_extra_default_or_generated_node_rejected":true,"limits_unchanged":true}),
    );
}

#[test]
fn enforces_total_raw_bytes_without_manufacturing_default_files() {
    let c = corpus();
    let k = keys(&c);
    let mut source = vec![];
    for i in 0..8 {
        let mut bytes = serde_json::to_vec(&raw(&format!("Bound{i}"))).unwrap();
        bytes.resize(1_048_576, b' ');
        let owned =
            GoTypeSourceBuffer::new("padded", &format!("{i}.json"), "overlay", &bytes).unwrap();
        source.push(GoSourceOverlay::import(&owned).unwrap());
    }
    let m = derive_go_source_contract_module(&k, &[selector("Loops")], &source).unwrap();
    assert_eq!(m.captured_overlays().len(), 8);
    assert!(m.captured_overlays().iter().all(|o| !o.in_body_closure()));
    assert_eq!(
        m.captured_overlays()
            .iter()
            .map(|o| o.overlay().source_buffer().bytes().len())
            .sum::<usize>(),
        8_388_608
    );
    assert!(m
        .functions()
        .iter()
        .all(|f| f.overlay_object_id().is_none()));
    source.push(overlay("extra.json", &raw("Worker")));
    assert_eq!(
        derive_go_source_contract_module(&k, &[selector("Loops")], &source)
            .unwrap_err()
            .kind(),
        &GoContractTypingErrorKind::Limit
    );
    let empty = derive_go_source_contract_module(&k, &[selector("Loops")], &[]).unwrap();
    assert!(empty.captured_overlays().is_empty());
    assert!(empty.annotation_typing().overlays().is_empty());
    let ids = empty
        .functions()
        .iter()
        .filter_map(|f| f.overlay_object_id())
        .collect::<BTreeSet<_>>();
    assert!(ids.is_empty());
    retain(
        "raw-byte-budgets.json",
        &json!({"exact_actual_raw_bytes":8_388_608,"one_extra_candidate_rejected":true,"default_raw_overlay_ids":[],"default_raw_source_files":0}),
    );
}
