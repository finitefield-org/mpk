use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::PathBuf;
fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-overlays")
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
    let k = import_go_type_key_components(&unhex(&p["components_hex"]), &sources).unwrap();
    assert_eq!(k.types().len(), 1);
    k
}
fn retain(name: &str, v: &Value) {
    if let Some(p) = std::env::var_os("MPK_GO_SOURCE_OVERLAYS_RUST_OUTPUT") {
        use std::io::Write;
        let p = PathBuf::from(p);
        std::fs::create_dir_all(&p).unwrap();
        let mut raw = serde_json::to_vec(v).unwrap();
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
fn packet(row: &Value) -> GoTypeSourceBuffer {
    GoTypeSourceBuffer::import(
        &serde_json::to_vec(&row["input"]["object"]).unwrap(),
        &unhex(&row["input"]["raw_hex"]),
    )
    .unwrap()
}
fn raw(row: &Value) -> Value {
    serde_json::from_slice(&unhex(&row["input"]["raw_hex"])).unwrap()
}
fn source(v: &Value) -> GoSourceOverlay {
    let b = GoTypeSourceBuffer::new(
        "overlays",
        "candidate.json",
        "overlay",
        &serde_json::to_vec(v).unwrap(),
    )
    .unwrap();
    GoSourceOverlay::import(&b).unwrap()
}
fn boolean(v: bool) -> Value {
    json!({"op":"bool.literal","value":v})
}
fn actual(bound: &GoBoundSourceOverlay, scopes: &GoSourceScopes) -> Value {
    let r = bound.root();
    let parsed = bound.source(r.source_object_id()).unwrap();
    let n = &parsed.nodes()[r.node()];
    let entries = bound
        .entries()
        .iter()
        .map(|e| {
            let path = e
                .path()
                .iter()
                .map(|s| match s {
                    GoErrorEntryStep::Elements => json!("elements"),
                    GoErrorEntryStep::Field(i) => json!(i),
                })
                .collect::<Vec<_>>();
            let mut cases = e
                .cases()
                .iter()
                .map(|r| {
                    derive_go_source_type_declaration(scopes, r.source_object_id(), r.node())
                        .unwrap()
                        .roots()[0]
                        .clone()
                })
                .collect::<Vec<_>>();
            cases.sort();
            json!({"input":e.input_position(),"path":path,"cases":cases})
        })
        .collect::<Vec<_>>();
    let loops=bound.loops().iter().map(|l|{let n=&parsed.nodes()[l.node()];json!({"kind":if n.kind()=="ForStmt"{"for"}else{"range"},"start_byte":n.start(),"end_byte":n.end(),"ordinal":l.ordinal()})}).collect::<Vec<_>>();
    json!({"root":{"source":r.source_object_id(),"start":n.start(),"end":n.end()},"inputs":bound.signature().slots().iter().filter(|s|s.role()!=GoSourceSignatureRole::Result).count(),"results":bound.signature().slots().iter().filter(|s|s.role()==GoSourceSignatureRole::Result).count(),"entries":entries,"loops":loops})
}
#[test]
fn binds_actual_original_functions_entry_fields_aliases_and_physical_loops() {
    let c = corpus();
    retain("input.json", &c["input"]);
    let k = keys(&c);
    let scopes = build_go_source_scopes(&k).unwrap();
    let mut summaries = vec![];
    for (i, row) in c["cases"].as_array().unwrap().iter().enumerate() {
        let p = packet(row);
        let overlay = GoSourceOverlay::import(&p).unwrap();
        assert_eq!(overlay.source_buffer(), &p);
        let b = bind_go_source_overlay(&k, &overlay)
            .unwrap_or_else(|e| panic!("{}: {e}", row["input"]["name"]));
        let observed = actual(&b, &scopes);
        assert_eq!(observed, row["native"]);
        retain(&format!("case-{i:02}-actual.json"), &observed);
        if row["input"]["name"] == "receiver" || row["input"]["name"] == "receiver-alias" {
            assert_eq!(
                b.loops()
                    .iter()
                    .map(|l| l.overlay_row())
                    .collect::<Vec<_>>(),
                vec![None, Some(0), None]
            );
        }
        if row["input"]["name"] == "seventy-inputs" {
            assert_eq!(b.entries().len(), 70);
        }
        let selection = derive_go_source_overlay_selection(&k, &[overlay]).unwrap();
        assert_eq!(selection.overlays()[0].root(), b.root());
        summaries.push(json!({"name":row["input"]["name"],"entry_paths":b.entries().len(),"loops":b.loops().len(),"typed_contract_or_runtime_proof":false}));
    }
    assert_eq!(summaries.len(), 13);
    retain(
        "summary.json",
        &json!({"cases":summaries,"source_only_constructor_input":true}),
    );
}
#[test]
fn rejects_forged_paths_duplicate_aliases_and_nonoriginal_loop_selectors() {
    let c = corpus();
    let k = keys(&c);
    let cases = c["cases"].as_array().unwrap();
    let find = |name: &str| cases.iter().find(|r| r["input"]["name"] == name).unwrap();
    let check = |v: &Value, kind: GoSourceOverlayErrorKind| {
        let e = bind_go_source_overlay(&k, &source(v)).unwrap_err();
        assert_eq!(*e.kind(), kind, "{v}");
    };
    let mut v = raw(find("nested"));
    v["error_inputs"].as_array_mut().unwrap().pop();
    check(&v, GoSourceOverlayErrorKind::MissingEntry);
    let mut v = raw(find("plain"));
    let copy = v["error_inputs"][0].clone();
    v["error_inputs"].as_array_mut().unwrap().push(copy);
    check(&v, GoSourceOverlayErrorKind::DuplicateEntry);
    let mut v = raw(find("plain"));
    v["error_inputs"][0]["cases"] =
        json!(["example.com/policy.Code", "example.com/policy.CodeAlias"]);
    check(&v, GoSourceOverlayErrorKind::DuplicateCase);
    let mut v = raw(find("plain"));
    v["error_inputs"][0]["input"] = json!(1);
    check(&v, GoSourceOverlayErrorKind::EntryPath);
    let mut v = raw(find("no-error"));
    v["error_inputs"] = json!([{"input":0,"path":[],"cases":[]}]);
    check(&v, GoSourceOverlayErrorKind::EntryPath);
    let mut v = raw(find("embedded"));
    v["error_inputs"][0]["path"] = json!([{"kind":"field","name":"E"}]);
    check(&v, GoSourceOverlayErrorKind::EntryPath);
    let mut v = raw(find("nested"));
    v["error_inputs"][0]["path"] = json!([{"kind":"elements"}]);
    check(&v, GoSourceOverlayErrorKind::EntryPath);
    let mut v = raw(find("plain"));
    v["error_inputs"][0]["cases"] = json!(["example.com/policy.Missing"]);
    check(&v, GoSourceOverlayErrorKind::TypeSelector);
    let mut v = raw(find("plain"));
    v["source"] = json!("forged-absent.go");
    check(&v, GoSourceOverlayErrorKind::Source);
    let mut v = raw(find("plain"));
    v["function"]["package"] = json!("policy");
    check(&v, GoSourceOverlayErrorKind::Selector);
    let mut v = raw(find("receiver"));
    v["function"]["function"] = json!("Pointer");
    check(&v, GoSourceOverlayErrorKind::Selector);
    let mut v = raw(find("receiver"));
    v["loops"][0]["selector"]["start_byte"] =
        json!(v["loops"][0]["selector"]["start_byte"].as_u64().unwrap() + 1);
    check(&v, GoSourceOverlayErrorKind::LoopSelector);
    let mut v = raw(find("receiver"));
    let copy = v["loops"][0].clone();
    v["loops"].as_array_mut().unwrap().push(copy);
    check(&v, GoSourceOverlayErrorKind::DuplicateLoop);
    let mut v = raw(find("receiver"));
    v["loops"][0]["selector"]["ordinal"] = json!(0);
    check(&v, GoSourceOverlayErrorKind::LoopSelector);
    let mut v = raw(find("plain"));
    v["error_inputs"][0]["cases"] = json!(["example.com/policy.Err"]);
    assert!(matches!(
        derive_go_source_overlay_selection(&k, &[source(&v)])
            .unwrap_err()
            .kind(),
        GoSourceOverlayErrorKind::Selection(_)
    ));
    retain(
        "rejections.json",
        &json!({"missing_zero_array_path":true,"duplicates_and_aliases":true,"wrong_slot_field_element_package_source_receiver_loop":true,"invalid_concrete_case_fails_selection":true}),
    );
}
#[test]
fn parses_all_seventy_frozen_heads_and_exact_registered_expression_limits() {
    let c = corpus();
    let base = raw(&c["cases"][10]);
    let witnesses: Value = serde_json::from_slice(
        &std::fs::read(
            directory().join("../../../t01-w04/contract-wire/expression-witnesses.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let catalog: Value = serde_json::from_slice(
        &std::fs::read(directory().join("../../../t01-w04/contract-wire/catalog.json")).unwrap(),
    )
    .unwrap();
    let mut covered = std::collections::BTreeSet::new();
    let rows = witnesses["witnesses"].as_array().unwrap();
    for row in rows {
        let expression = &row["source"];
        let mut v = base.clone();
        v["requires"] = json!([expression]);
        source(&v);
        let mut pending = vec![expression];
        while let Some(node) = pending.pop() {
            if let Some(op) = node["op"].as_str() {
                covered.insert(op.to_string());
            }
            if let Some(args) = node["args"].as_array() {
                pending.extend(args);
            }
            for key in ["value", "lower", "upper", "body"] {
                if node[key].is_object() {
                    pending.push(&node[key]);
                }
            }
        }
        let mut wrong = v.clone();
        wrong["requires"][0]["unexpected"] = json!(true);
        let p = GoTypeSourceBuffer::new(
            "overlays",
            "bad.json",
            "overlay",
            &serde_json::to_vec(&wrong).unwrap(),
        )
        .unwrap();
        assert_eq!(
            *GoSourceOverlay::import(&p).unwrap_err().kind(),
            GoSourceOverlayErrorKind::Shape
        );
    }
    let expected = catalog["source_arms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["op"].as_str().unwrap().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(covered, expected);
    assert_eq!(covered.len(), 70);
    let mut v = base.clone();
    let mut expression = boolean(true);
    for _ in 0..32 {
        expression = json!({"op":"bool.not","args":[expression]});
    }
    v["ensures"] = json!([expression.clone()]);
    assert_eq!(source(&v).expression_depth(), 32);
    v["ensures"] = json!([{"op":"bool.not","args":[expression]}]);
    let p = GoTypeSourceBuffer::new(
        "overlays",
        "deep.json",
        "overlay",
        &serde_json::to_vec(&v).unwrap(),
    )
    .unwrap();
    assert_eq!(
        *GoSourceOverlay::import(&p).unwrap_err().kind(),
        GoSourceOverlayErrorKind::Limit
    );
    let mut tree = boolean(true);
    for _ in 0..9 {
        tree = json!({"op":"bool.and","args":[tree.clone(),tree]});
    }
    let mut v = base.clone();
    v["requires"] = json!([tree]);
    assert_eq!(source(&v).expression_nodes(), 1024);
    v["ensures"] = json!([boolean(true), boolean(true)]);
    let p = GoTypeSourceBuffer::new(
        "overlays",
        "large.json",
        "overlay",
        &serde_json::to_vec(&v).unwrap(),
    )
    .unwrap();
    assert_eq!(
        *GoSourceOverlay::import(&p).unwrap_err().kind(),
        GoSourceOverlayErrorKind::Limit
    );
    let mut v = base;
    v["requires"] = json!([{"op":"nat.literal","value":"1".repeat(5000)}]);
    source(&v);
    retain(
        "syntax.json",
        &json!({"heads":covered,"maximum_expression_nodes":1024,"maximum_expression_depth":32,"no_private_4096_digit_cap":true,"expression_types_or_proofs_claimed":false}),
    );
}
#[test]
fn raw_bytes_required_nullable_members_and_namespace_ownership_remain_exact() {
    let c = corpus();
    let k = keys(&c);
    let base = raw(&c["cases"][0]);
    let original = source(&base);
    let mut bytes = serde_json::to_vec_pretty(&base).unwrap();
    bytes.push(b'\n');
    let altered = GoSourceOverlay::import(
        &GoTypeSourceBuffer::new("overlays", "candidate.json", "overlay", &bytes).unwrap(),
    )
    .unwrap();
    assert_ne!(
        original.source_buffer().object().id,
        altered.source_buffer().object().id
    );
    assert_eq!(
        bind_go_source_overlay(&k, &original).unwrap().root(),
        bind_go_source_overlay(&k, &altered).unwrap().root()
    );
    let mut wrong = base.clone();
    wrong["function"]
        .as_object_mut()
        .unwrap()
        .remove("receiver");
    let p = GoTypeSourceBuffer::new(
        "overlays",
        "missing.json",
        "overlay",
        &serde_json::to_vec(&wrong).unwrap(),
    )
    .unwrap();
    assert_eq!(
        *GoSourceOverlay::import(&p).unwrap_err().kind(),
        GoSourceOverlayErrorKind::Shape
    );
    let raw = serde_json::to_vec(&base).unwrap();
    let mut duplicate = b"{\"schema\":\"ignored\",".to_vec();
    duplicate.extend_from_slice(&raw[1..]);
    let p = GoTypeSourceBuffer::new("overlays", "duplicate.json", "overlay", &duplicate).unwrap();
    assert!(matches!(
        GoSourceOverlay::import(&p).unwrap_err().kind(),
        GoSourceOverlayErrorKind::Json(_)
    ));
    let mut exact = raw.clone();
    exact.resize(1_048_576, b' ');
    let p = GoTypeSourceBuffer::new("overlays", "maximum.json", "overlay", &exact).unwrap();
    assert_eq!(
        GoSourceOverlay::import(&p).unwrap().source_buffer().bytes(),
        exact
    );
    exact.push(b' ');
    assert!(GoTypeSourceBuffer::new("overlays", "too-large.json", "overlay", &exact).is_err());
    let go = &c["input"]["sources"][0]["object"];
    let p = GoTypeSourceBuffer::new(
        go["capability"].as_str().unwrap(),
        go["path"].as_str().unwrap(),
        "overlay",
        &raw,
    )
    .unwrap();
    assert_eq!(
        *bind_go_source_overlay(&k, &GoSourceOverlay::import(&p).unwrap())
            .unwrap_err()
            .kind(),
        GoSourceOverlayErrorKind::Source
    );
    let bound = {
        let owned = keys(&c);
        let mut input = raw.clone();
        let p = GoTypeSourceBuffer::new("overlays", "owned.json", "overlay", &input).unwrap();
        let overlay = GoSourceOverlay::import(&p).unwrap();
        input.fill(b' ');
        bind_go_source_overlay(&owned, &overlay).unwrap()
    };
    assert!(bound.source(bound.root().source_object_id()).is_some());
    assert_eq!(bound.overlay().source_buffer().bytes(), raw);
    retain(
        "raw-ownership.json",
        &json!({"raw_whitespace_identity":true,"required_nullable_receiver":true,"duplicate_JSON_names_rejected":true,"overlay_byte_boundary":1048576,"source_namespace_collision_rejected":true,"owned_original_bytes":true}),
    );
}
#[test]
fn entry_overlay_sets_share_global_cases_and_reject_alias_duplicate_roots() {
    let c = corpus();
    let k = keys(&c);
    let a = GoSourceOverlay::import(&packet(&c["cases"][0])).unwrap();
    let b = GoSourceOverlay::import(&packet(&c["cases"][2])).unwrap();
    let s = derive_go_source_overlay_selection(&k, &[a.clone(), b.clone()]).unwrap();
    let reversed = derive_go_source_overlay_selection(&k, &[b, a.clone()]).unwrap();
    assert_eq!(s, reversed);
    assert_eq!(s.selection().domain().error_cases().len(), 2);
    assert_eq!(s.selection().dispatches()[0].targets().len(), 2);
    assert_eq!(
        *derive_go_source_overlay_selection(&k, &[a.clone(), a])
            .unwrap_err()
            .kind(),
        GoSourceOverlayErrorKind::DuplicateOverlay
    );
    let original = GoSourceOverlay::import(&packet(&c["cases"][6])).unwrap();
    let alias = GoSourceOverlay::import(&packet(&c["cases"][7])).unwrap();
    assert_eq!(
        *derive_go_source_overlay_selection(&k, &[original, alias])
            .unwrap_err()
            .kind(),
        GoSourceOverlayErrorKind::DuplicateOverlay
    );
    retain(
        "overlay-set.json",
        &json!({"canonical_root_order":true,"global_case_count":2,"duplicate_paths_and_alias_resolved_functions_reject":true,"whole_module_contract_set_or_context_claimed":false}),
    );
}

#[test]
fn practical_request_roots_resolve_original_receiver_aliases_and_reject_duplicates() {
    use mpk_vc::go_practical_selection::import_go_practical_selection;
    use mpk_vc::{canonical_json_bytes, parse_strict_json, StrictJsonLimits};
    let corpus = corpus();
    let keys = keys(&corpus);
    let original = raw(&corpus["cases"][6])["function"].clone();
    let alias = raw(&corpus["cases"][7])["function"].clone();
    let strict = |value: &Value| {
        parse_strict_json(
            &serde_json::to_vec(value).unwrap(),
            StrictJsonLimits::new(4_194_304, 1_000_000, 64, 1_000_000),
        )
        .unwrap()
    };
    let selection = |roots: Vec<Value>| {
        import_go_practical_selection(&strict(&json!({
            "schema":"mpk.selection.go_functions.v1", "value":{"roots":roots}
        })))
        .unwrap()
    };
    let direct =
        resolve_go_practical_named_roots(&keys, &selection(vec![original.clone()])).unwrap();
    let via_alias =
        resolve_go_practical_named_roots(&keys, &selection(vec![alias.clone()])).unwrap();
    assert_eq!(direct, via_alias);
    let mut roots = vec![original, alias];
    roots.sort_by_cached_key(|root| canonical_json_bytes(&strict(root)).unwrap());
    assert!(matches!(
        resolve_go_practical_named_roots(&keys, &selection(roots)),
        Err(GoPracticalSourceBindingError::Duplicate {
            first: 0,
            second: 1
        })
    ));
}
