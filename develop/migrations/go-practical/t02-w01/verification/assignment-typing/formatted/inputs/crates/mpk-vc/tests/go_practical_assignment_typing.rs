use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, check_go_assignment_statement, import_go_type_key_components,
    GoAssignmentTyping, GoSourceScopes, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/assignment-typing")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_ASSIGNMENT_TYPING_RUST_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let mut raw = serde_json::to_vec(value).unwrap();
        raw.push(b'\n');
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
}
fn scopes(input: &Value) -> std::result::Result<GoSourceScopes, String> {
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&v["object"]).unwrap(),
                &bytes(&v["raw_hex"]),
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| format!("{e:?}"))?;
    let keys = import_go_type_key_components(&bytes(&input["components_hex"]), &sources)
        .map_err(|e| format!("{e:?}"))?;
    build_go_source_scopes(&keys).map_err(|e| format!("{e:?}"))
}

fn output(value: &GoAssignmentTyping) -> Value {
    let source = value.source_object_id();
    let typing = value.typing();
    let parsed = typing.source(source).unwrap();
    let stmt = &parsed.nodes()[value.statement_node()];
    let key = |id: &str| typing.types().find(|k| k.id() == id).unwrap().clone();
    let mut calls = vec![];
    for h in typing.call_headers() {
        if h.call_source_object_id() != source {
            continue;
        };
        let call = &parsed.nodes()[h.call_node()];
        if call.start() < stmt.start() || call.end() > stmt.end() {
            continue;
        }
        let original = typing.source(h.declaration_source_object_id()).unwrap();
        let decl = &original.nodes()[h.declaration_node()];
        let body = h.body_node().map(|n| &original.nodes()[n]);
        calls.push(json!({"start_byte":call.start(),"end_byte":call.end(),"declaration_source":h.declaration_source_object_id(),"declaration_start":decl.start(),"declaration_end":decl.end(),"body_start":body.map(|b|b.start()),"body_end":body.map(|b|b.end())}));
    }
    calls.sort_by_key(|r| (r["start_byte"].as_u64(), r["end_byte"].as_u64()));
    json!({"accepted":true,"body_state_or_program_proof":false,"operator":stmt.attributes().find(|(k,_)|*k=="Tok").unwrap().1,"targets":value.targets().iter().map(|t|{let n=&parsed.nodes()[t.node()];json!({"start_byte":n.start(),"end_byte":n.end(),"type_key":t.type_id().map(key),"declares_variable":t.declares_variable()})}).collect::<Vec<_>>(),"slots":typing.values().iter().map(|v|json!({"type_key":v.source_type_key(),"addressable":v.addressable(),"constant":v.constant().is_some(),"untyped_nil":v.is_untyped_nil()})).collect::<Vec<_>>(),"calls":calls})
}
#[test]
fn matches_original_go_assignment_statements() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases = corpus.as_array().unwrap();
    assert!(cases.len() > 1000);
    let (mut accepted, mut rejected, mut outside) = (0, 0, 0);
    let mut failures = vec![];
    for (n, row) in cases.iter().enumerate() {
        let input = &row["input"];
        retain(&format!("case-{n:05}-input.json"), input);
        let actual = scopes(input).and_then(|graph| {
            let source = input["query_source_object_id"].as_str().unwrap();
            let parsed = graph.source(source).ok_or("source".to_string())?;
            let node = parsed
                .nodes()
                .iter()
                .position(|v| {
                    matches!(v.kind(), "AssignStmt" | "IncDecStmt")
                        && v.start() as u64 == input["statement_start_byte"].as_u64().unwrap()
                        && v.end() as u64 == input["statement_end_byte"].as_u64().unwrap()
                })
                .ok_or("statement".to_string())?;
            check_go_assignment_statement(&graph, source, node).map_err(|e| format!("{e:?}"))
        });
        let outcome = match &actual {
            Ok(v) => output(v),
            Err(e) => json!({"accepted":false,"error":e,"body_state_or_program_proof":false}),
        };
        retain(&format!("case-{n:05}-actual.json"), &outcome);
        if input["outside_assignment_typing"] == true && row["native"]["accepted"] == true {
            if actual.is_ok() {
                failures.push(format!(
                    "outside case {n} {} accepted: {outcome}",
                    input["label"]
                ))
            };
            outside += 1
        } else if row["native"]["accepted"] == true {
            if outcome != row["native"] {
                failures.push(format!(
                    "case {n} {} expected {} actual {outcome}",
                    input["label"], row["native"]
                ))
            };
            accepted += 1
        } else {
            if actual.is_ok() {
                failures.push(format!(
                    "case {n} {} falsely accepted: {outcome}",
                    input["label"]
                ))
            };
            rejected += 1
        }
    }
    retain(
        "summary.json",
        &json!({"accepted":accepted,"rejected":rejected,"outside":outside,"failures":failures}),
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
fn canonical(v: &Value) -> Vec<u8> {
    canonical_json_bytes(
        &parse_strict_json(
            &serde_json::to_vec(v).unwrap(),
            StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
        )
        .unwrap(),
    )
    .unwrap()
}
fn seal(v: &mut Value, domain: &'static str, field: &str) {
    v.as_object_mut().unwrap().remove(field);
    let b = canonical(v);
    let x = parse_strict_json(
        &b,
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
    )
    .unwrap();
    v[field] = json!(hash_canonical_json(HashDomain::new(domain), &x)
        .unwrap()
        .to_hex())
}
fn sort(v: &mut Value, key: &str) {
    v.as_array_mut()
        .unwrap()
        .sort_by(|a, b| a[key].as_str().cmp(&b[key].as_str()))
}

fn control(label: &str, text: &str) -> (Value, GoSourceScopes) {
    let cap = format!("scalar-controls/{label}");
    let file = GoTypeSourceBuffer::new(&cap, "policy.go", "go_file", text.as_bytes()).unwrap();
    let module = GoTypeSourceBuffer::new(
        &cap,
        "go.mod",
        "go_mod",
        b"module example.com/scalar\ngo 1.25.0\n",
    )
    .unwrap();
    let mut p = json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/scalar","package_name":"policy","source_objects":[file.object().id]});
    seal(&mut p, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut t = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});
    seal(&mut t, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut v = json!({"schema":"mpk.go.type_components.v2","source_objects":[file.object(),module.object()],"packages":[p.clone()],"declarations":[],"types":[t.clone()],"underlying":[],"aliases":[],"package_roots":[p["id"]],"type_roots":[t["id"]]});
    sort(&mut v["source_objects"], "id");
    seal(&mut v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    let input = json!({"components_hex":canonical(&v).iter().map(|b|format!("{b:02x}")).collect::<String>(),"sources":[{"object":file.object(),"raw_hex":file.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()},{"object":module.object(),"raw_hex":module.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()}]});
    retain(&format!("{label}-input.json"), &input);
    let graph = scopes(&input).unwrap();
    (input, graph)
}

#[test]
fn assignment_headers_keep_original_targets_and_joint_call_identity_without_body_proof() {
    let(input,graph)=control("assignment-owned","package policy\ntype W uint16\nfunc Pair()(W,uint64)\nfunc Slice()[]W\nfunc Index()int\nfunc Probe(x W,u uint64){x,u=Pair();Slice()[Index()]+=1;x=\"unchecked elsewhere\"}\n");
    let source = input["sources"][0]["object"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let parsed = graph.source(&source).unwrap();
    let stmt = |text: &[u8]| {
        parsed
            .nodes()
            .iter()
            .enumerate()
            .find(|(i, n)| n.kind() == "AssignStmt" && parsed.spelling(*i) == Some(text))
            .unwrap()
            .0
    };
    let pair_node = stmt(b"x,u=Pair()");
    let pair = check_go_assignment_statement(&graph, &source, pair_node).unwrap();
    assert_eq!(pair.typing().call_headers().len(), 1);
    assert_eq!(pair.typing().values().len(), 2);
    assert!(pair.typing().call_headers()[0].body_node().is_none());
    assert_eq!(pair.targets().len(), 2);
    let compound =
        check_go_assignment_statement(&graph, &source, stmt(b"Slice()[Index()]+=1")).unwrap();
    assert_eq!(compound.typing().call_headers().len(), 2);
    assert!(compound.targets()[0].type_id().is_some());
    assert!(
        check_go_assignment_statement(&graph, &source, stmt(b"x=\"unchecked elsewhere\"")).is_err()
    );
    assert!(check_go_assignment_statement(&graph, "missing", pair_node).is_err());
    assert!(check_go_assignment_statement(&graph, &source, usize::MAX).is_err());
    let not_statement = parsed
        .nodes()
        .iter()
        .position(|n| n.kind() == "CallExpr")
        .unwrap();
    assert!(check_go_assignment_statement(&graph, &source, not_statement).is_err());
    retain(
        "owned-assignment-actual.json",
        &json!({"pair":output(&pair),"compound":output(&compound),"other_body_statement_independently_rejected":true,"body_state_or_program_proof":false}),
    );
    drop(graph);
    drop(input);
    assert_eq!(
        pair.typing().source(&source).unwrap().spelling(pair_node),
        Some(b"x,u=Pair()".as_slice())
    );
}
#[test]
fn actual_assignment_context_work_is_flat() {
    let text = format!(
        "package policy\nfunc Probe(x uint16,n uint64){{x={}1<<n}}\n",
        "-".repeat(20000)
    );
    let (input, graph) = control("assignment-deep", &text);
    let source = input["sources"][0]["object"]["id"].as_str().unwrap();
    let parsed = graph.source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| n.kind() == "AssignStmt")
        .unwrap();
    let typed = check_go_assignment_statement(&graph, source, node).unwrap();
    assert_eq!(typed.typing().values().len(), 1);
    retain("assignment-deep-actual.json", &output(&typed));
}
