use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, check_go_function_body, derive_go_source_selection,
    import_go_type_key_components, GoErrorBoxing, GoErrorBoxingProjection, GoSourceSelection,
    GoSourceSelectionRef, GoTypeSourceBuffer,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/error-boxings")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap();
    s.as_bytes()
        .chunks_exact(2)
        .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(path) = std::env::var_os("MPK_GO_ERROR_BOXINGS_RUST_OUTPUT") {
        use std::io::Write;
        let path = PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        let mut raw = serde_json::to_vec(value).unwrap();
        raw.push(b'\n');
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path.join(name))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
}
fn corpus() -> Value {
    serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap()
}
fn keys(input: &Value) -> mpk_vc::go_practical_type_keys::GoTypeKeyComponents {
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &bytes(&s["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(&bytes(&input["components_hex"]), &sources).unwrap();
    assert_eq!(keys.types().len(), 1);
    keys
}
fn locator(
    keys: &mpk_vc::go_practical_type_keys::GoTypeKeyComponents,
    v: &Value,
) -> GoSourceSelectionRef {
    let source = v["source"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == "FuncDecl"
                && n.start() == v["start"].as_u64().unwrap() as usize
                && n.end() == v["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn boxing(site: &GoErrorBoxing, parsed: &mpk_vc::go_practical_type_keys::GoParsedSource) -> Value {
    let n = &parsed.nodes()[site.expression_node()];
    let projection = match site.projection() {
        GoErrorBoxingProjection::Result(slot) => json!(slot),
        GoErrorBoxingProjection::RangeElement => json!("range_element"),
    };
    json!({"source":site.source_object_id(),"start":n.start(),"end":n.end(),"projection":projection,"type_id":site.value_type_id()})
}
fn normalized(mut rows: Vec<Value>) -> Value {
    rows.sort_by_key(|v| serde_json::to_string(v).unwrap());
    json!(rows)
}
fn body(s: &GoSourceSelection) -> Value {
    normalized(
        s.body_error_boxings()
            .iter()
            .map(|site| boxing(site, s.source(site.source_object_id()).unwrap()))
            .collect(),
    )
}
#[test]
fn reconstructs_original_error_boxings_and_body_type_union_without_native_facts() {
    let corpus = corpus();
    let input = &corpus["input"];
    retain("input.json", input);
    let keys = keys(input);
    let scopes = build_go_source_scopes(&keys).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 46);
    let (mut typed_count, mut runtime_count) = (0, 0);
    let mut summaries = vec![];
    for (i, case) in cases.iter().enumerate() {
        let root = locator(&keys, &case["root"][0]);
        let function =
            check_go_function_body(&scopes, root.source_object_id(), root.node()).unwrap();
        let parsed = function.typing().source(root.source_object_id()).unwrap();
        let n = &parsed.nodes()[root.node()];
        let typed = normalized(
            function
                .typing()
                .error_boxings()
                .iter()
                .filter(|site| {
                    site.source_object_id() == root.source_object_id() && {
                        let p = &parsed.nodes()[site.expression_node()];
                        n.start() <= p.start() && p.end() <= n.end()
                    }
                })
                .map(|site| boxing(site, parsed))
                .collect(),
        );
        let selected = derive_go_source_selection(&keys, &[root]).unwrap();
        let actual = body(&selected);
        let union = selected.body_boxed_error_type_ids();
        let expected_types = case["native"]["body"]
            .as_array()
            .unwrap()
            .iter()
            .map(|site| site["type_id"].as_str().unwrap().to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        retain(
            &format!("case-{i:02}-actual.json"),
            &json!({"name":case["name"],"typed":typed,"body":actual,"body_type_union":union,"global_error_universe_complete":false}),
        );
        assert_eq!(
            typed,
            normalized(case["native"]["typed"].as_array().unwrap().clone()),
            "{} typed",
            case["name"]
        );
        assert_eq!(
            actual,
            normalized(case["native"]["body"].as_array().unwrap().clone()),
            "{} body",
            case["name"]
        );
        assert_eq!(union, expected_types, "{} union", case["name"]);
        match case["name"].as_str().unwrap() {
            "Nil" | "Rebox" | "BlankDefault" | "DirectLen" | "ConstLen" | "GlobalLen"
            | "GlobalRange" | "RangeZero" | "RangeOne" | "RangeBlank" => assert!(union.is_empty()),
            "ManyCaller" => assert_eq!(actual.as_array().unwrap().len(), 70),
            "Wide" => assert_eq!(actual.as_array().unwrap().len(), 16),
            "Branches" => assert_eq!(union.len(), 2),
            _ => (),
        }
        typed_count += typed.as_array().unwrap().len();
        runtime_count += actual.as_array().unwrap().len();
        summaries.push(json!({"name":case["name"],"typed":typed.as_array().unwrap().len(),"body":actual.as_array().unwrap().len(),"types":union.len()}));
    }
    retain(
        "summary.json",
        &json!({"cases":cases.len(),"typed_sites":typed_count,"body_sites":runtime_count,"cases_summary":summaries}),
    );
}
#[test]
fn multi_root_boxing_union_is_owned_and_does_not_guess_entry_error_cases() {
    let corpus = corpus();
    let keys = keys(&corpus["input"]);
    let cases = corpus["cases"].as_array().unwrap();
    let mut roots = ["Basic", "HelperOnly", "Unreachable", "Rebox"]
        .iter()
        .map(|&name| {
            locator(
                &keys,
                &cases.iter().find(|c| c["name"] == name).unwrap()["root"][0],
            )
        })
        .collect::<Vec<_>>();
    let first = derive_go_source_selection(&keys, &roots).unwrap();
    assert_eq!(first.body_boxed_error_type_ids().len(), 2);
    assert_eq!(first.body_error_boxings().len(), 3);
    roots.reverse();
    assert_eq!(first, derive_go_source_selection(&keys, &roots).unwrap());
    retain(
        "many-roots-actual.json",
        &json!({"body":body(&first),"types":first.body_boxed_error_type_ids(),"entry_error_values_constrained":false}),
    );
    drop(keys);
    drop(corpus);
    assert!(first.source(first.roots()[0].source_object_id()).is_some());
}
