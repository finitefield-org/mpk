use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, import_go_type_key_components, GoNameLookup, GoSourceScopeError,
    GoSourceScopes, GoTypeKeyComponents, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-scopes")
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
    let bytes = canonical(v);
    let value = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
    )
    .unwrap();
    v[field] = json!(hash_canonical_json(HashDomain::new(domain), &value)
        .unwrap()
        .to_hex())
}
fn retain(name: &str, raw: &[u8]) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_SCOPES_RUST_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn fixture(path: &Path) -> GoTypeKeyComponents {
    let mut files: Vec<_> = std::fs::read_dir(path.join("sources"))
        .unwrap()
        .map(|f| f.unwrap().path())
        .collect();
    files.sort();
    let sources: Vec<_> = files
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .map(|p| {
            GoTypeSourceBuffer::import(
                &std::fs::read(&p).unwrap(),
                &std::fs::read(p.with_extension("raw")).unwrap(),
            )
            .unwrap()
        })
        .collect();
    import_go_type_key_components(
        &std::fs::read(path.join("components.json")).unwrap(),
        &sources,
    )
    .unwrap()
}
fn response(graph: &GoSourceScopes, source: &str, node: usize) -> Value {
    match graph.lookup_identifier(source, node).unwrap() {
        Some(GoNameLookup::Original(b)) => {
            json!({"name":b.name(),"kind":b.kind().as_str(),"definition_source_object_id":b.source_object_id(),"definition_start_byte":graph.source(b.source_object_id()).unwrap().nodes()[b.name_node()].start()})
        }
        Some(GoNameLookup::Predeclared { name, kind }) => {
            json!({"name":name,"kind":kind.as_str(),"definition_source_object_id":null,"definition_start_byte":null})
        }
        None => Value::Null,
    }
}
fn ident(graph: &GoSourceScopes, source: &str, start: usize, end: usize, name: &str) -> usize {
    let mut nodes = graph
        .source(source)
        .unwrap()
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.kind() == "Ident"
                && n.start() == start
                && n.end() == end
                && n.attributes().any(|(k, v)| k == "Name" && v == name)
        });
    let node = nodes.next().unwrap().0;
    assert!(nodes.next().is_none());
    node
}
#[test]
fn matches_original_go_lexical_binding_observations() {
    let keys = fixture(&root().join("native-components"));
    let graph = build_go_source_scopes(&keys).unwrap();
    let original: Value =
        serde_json::from_slice(&std::fs::read(root().join("native-components/uses.json")).unwrap())
            .unwrap();
    let uses = original["uses"].as_array().unwrap();
    assert!(uses.len() > 100);
    for (n, row) in uses.iter().enumerate() {
        retain(&format!("query-{n:04}.json"), &canonical(row));
        let source = row["source_object_id"].as_str().unwrap();
        let id = ident(
            &graph,
            source,
            row["start_byte"].as_u64().unwrap() as usize,
            row["end_byte"].as_u64().unwrap() as usize,
            row["name"].as_str().unwrap(),
        );
        let actual = response(&graph, source, id);
        retain(&format!("actual-{n:04}.json"), &canonical(&actual));
        let mut expected = row.clone();
        expected.as_object_mut().unwrap().remove("source_object_id");
        expected.as_object_mut().unwrap().remove("start_byte");
        expected.as_object_mut().unwrap().remove("end_byte");
        assert_eq!(actual, expected, "original lexical use {n}: {row}");
    }
    println!("{} exact original Go lexical definitions", uses.len());
    // Independent owned syntax and declarations survive caller key destruction.
    drop(keys);
    assert_eq!(
        graph.bindings().len(),
        build_go_source_scopes(&fixture(&root().join("native-components")))
            .unwrap()
            .bindings()
            .len()
    );
    assert_eq!(
        graph.lookup_identifier("missing", 0),
        Err(GoSourceScopeError::Locator)
    );
}
// Candidate package syntax only. No compiler/type/capture authority is asserted
// for these intentionally incomplete or invalid original package examples.
fn candidate(label: &str, text: &str) -> GoTypeKeyComponents {
    let file = GoTypeSourceBuffer::new("root", "policy.go", "go_file", text.as_bytes()).unwrap();
    let module = GoTypeSourceBuffer::new(
        "root",
        "go.mod",
        "go_mod",
        b"module example.com/policy\ngo 1.25.0\n",
    )
    .unwrap();
    let mut package = json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/policy","package_name":"policy","source_objects":[file.object().id]});
    seal(&mut package, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut scalar = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"uint16","width":16,"signed":false});
    seal(&mut scalar, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut sources = vec![json!(file.object()), json!(module.object())];
    sources.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let mut wire = json!({"schema":"mpk.go.type_components.v2","source_objects":sources,"packages":[package.clone()],"declarations":[],"types":[scalar.clone()],"underlying":[],"aliases":[],"package_roots":[package["id"]],"type_roots":[scalar["id"]]});
    seal(&mut wire, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    retain(
        &format!("{label}-source.json"),
        &canonical(&json!({"object":file.object(),"whole_original_bytes":file.bytes()})),
    );
    retain(
        &format!("{label}-module.json"),
        &canonical(&json!({"object":module.object(),"whole_original_bytes":module.bytes()})),
    );
    let bytes = canonical(&wire);
    retain(&format!("{label}-candidate.json"), &bytes);
    import_go_type_key_components(&bytes, &[file, module]).unwrap()
}
fn query(
    graph: &GoSourceScopes,
    needle: &str,
    name: &str,
) -> std::result::Result<Option<GoNameLookup>, GoSourceScopeError> {
    let source = graph.bindings().first().unwrap().source_object_id();
    let parsed = graph.source(source).unwrap();
    let raw = std::str::from_utf8(parsed.source().bytes()).unwrap();
    let at = raw.find(needle).unwrap() + needle.rfind(name).unwrap();
    let id = ident(graph, source, at, at + name.len(), name);
    graph.lookup_identifier(source, id)
}
#[test]
fn original_blockers_visibility_and_missing_imports_fail_closed() {
    let text="package policy\nimport \"example.com/unknown-path\"\nfunc f(){ var int = 1; _ = int; _ = uint16 }\n";
    let graph = build_go_source_scopes(&candidate("missing-default-import", text)).unwrap();
    let Some(GoNameLookup::Original(b)) = query(&graph, "_ = int", "int").unwrap() else {
        panic!("shadow lost")
    };
    assert_eq!(b.kind().as_str(), "variable");
    assert_eq!(
        query(&graph, "_ = uint16", "uint16"),
        Err(GoSourceScopeError::Import)
    );
    let text="package policy\nimport alias \"example.com/unknown-path\"\nfunc f(){ _ = alias; _ = uint16 }\n";
    let graph = build_go_source_scopes(&candidate("missing-aliased-import", text)).unwrap();
    let Some(GoNameLookup::Original(b)) = query(&graph, "_ = alias", "alias").unwrap() else {
        panic!("import shadow lost")
    };
    assert_eq!(b.import_path(), Some("example.com/unknown-path"));
    assert_eq!(b.kind().as_str(), "import");
    assert!(matches!(
        query(&graph, "_ = uint16", "uint16").unwrap(),
        Some(GoNameLookup::Predeclared { .. })
    ));
    let text="package policy\nimport . \"example.com/unknown-path\"\nfunc f(){ _ = uint16; _ = Exported }\n";
    let graph = build_go_source_scopes(&candidate("missing-dot-import", text)).unwrap();
    assert!(matches!(
        query(&graph, "_ = uint16", "uint16").unwrap(),
        Some(GoNameLookup::Predeclared { .. })
    ));
    assert_eq!(
        query(&graph, "_ = Exported", "Exported"),
        Err(GoSourceScopeError::Import)
    );
}
#[test]
fn rejects_duplicate_and_invalid_short_bindings() {
    for (label, text) in [
        (
            "duplicate-package-name",
            "package policy\ntype X uint16\nvar X uint16\n",
        ),
        (
            "duplicate-local-name",
            "package policy\nfunc f(){var x uint16; const x=1}\n",
        ),
        (
            "no-new-short-name",
            "package policy\nfunc f(x uint16){x:=1; _=x}\n",
        ),
        (
            "duplicate-short-destination",
            "package policy\nfunc f(){x,x:=1,2; _=x}\n",
        ),
        (
            "import-package-conflict",
            "package policy\nimport alias \"example.com/unknown\"\nvar alias uint16\n",
        ),
    ] {
        let keys = candidate(label, text);
        let error = build_go_source_scopes(&keys).unwrap_err();
        retain(
            &format!("{label}-actual.json"),
            &canonical(
                &json!({"error":format!("{error:?}"),"source_typing_or_context_or_program_authority":false}),
            ),
        );
        assert_eq!(error, GoSourceScopeError::Duplicate, "{label}");
    }
}
