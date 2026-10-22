use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::PathBuf;
fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-domains")
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
    if let Some(output) = std::env::var_os("MPK_GO_SOURCE_DOMAINS_RUST_OUTPUT") {
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
fn locate(keys: &GoTypeKeyComponents, v: &Value, kind: &str) -> GoSourceSelectionRef {
    let source = v["source"].as_str().unwrap();
    let p = keys.source_declarations().source(source).unwrap();
    let node = p
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
fn root(c: &Value, k: &GoTypeKeyComponents, name: &str) -> GoSourceSelectionRef {
    locate(k, &c["roots"][name], "FuncDecl")
}
fn case(c: &Value, k: &GoTypeKeyComponents, name: &str) -> GoSourceSelectionRef {
    locate(
        k,
        &c["declarations"][format!("example.com/policy.{name}")],
        "TypeSpec",
    )
}
fn paths(name: &str) -> Vec<(usize, Vec<GoErrorEntryStep>)> {
    use GoErrorEntryStep::{Elements as E, Field as F};
    match name {
        "Plain" | "Box" => vec![(0, vec![])],
        "Nested" => vec![
            (0, vec![F(0)]),
            (0, vec![F(1), E, F(0)]),
            (0, vec![F(2), E]),
        ],
        "Receiver" => vec![
            (0, vec![F(0)]),
            (0, vec![F(1), E, F(0)]),
            (0, vec![F(2), E]),
            (1, vec![]),
        ],
        "Slice" => vec![(0, vec![]), (1, vec![E])],
        "DepthError" => vec![(0, vec![E; 16])],
        _ => vec![],
    }
}
fn entries(
    c: &Value,
    k: &GoTypeKeyComponents,
    name: &str,
    cases: &[&str],
) -> Vec<GoErrorEntryConstraint> {
    paths(name)
        .into_iter()
        .map(|(input, path)| {
            GoErrorEntryConstraint::new(
                root(c, k, name),
                input,
                path,
                cases.iter().map(|n| case(c, k, n)).collect(),
            )
        })
        .collect()
}
fn properties(p: &GoSourceValueProperties) -> Value {
    json!({"plain":p.is_plain(),"depth":p.aggregate_depth(),"weight":p.value_weight_units(),"contains_error":p.contains_source_error()})
}
fn kind(e: &GoSourceDomainError) -> String {
    format!("{:?}", e.kind())
}
#[test]
fn original_compiler_graphs_agree_on_global_plain_depth_and_weights() {
    let c = corpus();
    retain("domain-input.json", &c["input"]);
    let k = keys(&c);
    let scopes = build_go_source_scopes(&k).unwrap();
    let mut results = vec![];
    for observation in c["observations"].as_array().unwrap() {
        let name = observation["root"].as_str().unwrap();
        let cases = observation["cases"]
            .as_array()
            .map(|a| a.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>())
            .unwrap_or_default();
        let r = root(&c, &k, name);
        let selection = derive_go_source_selection(&k, std::slice::from_ref(&r)).unwrap();
        let rows = entries(&c, &k, name, &cases);
        let actual = derive_go_source_value_domain(&selection, &rows);
        let expected_error = observation["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["error"].as_str().unwrap())
            .find(|e| !e.is_empty());
        let result = match expected_error {
            Some(expected) => {
                let error = actual.unwrap_err();
                assert_eq!(kind(&error), expected, "{name} {cases:?}");
                json!({"error":kind(&error)})
            }
            None => {
                let domain = actual.unwrap_or_else(|e| panic!("{name} {cases:?}: {e}"));
                let sig =
                    derive_go_source_signature(&scopes, r.source_object_id(), r.node()).unwrap();
                let inputs = sig
                    .slots()
                    .iter()
                    .filter(|s| s.role() != GoSourceSignatureRole::Result)
                    .map(|s| properties(domain.properties(s.type_id()).unwrap()))
                    .collect::<Vec<_>>();
                let expected = observation["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p["properties"].clone())
                    .collect::<Vec<_>>();
                assert_eq!(inputs, expected, "{name} {cases:?}");
                assert_eq!(domain.entries().len(), paths(name).len());
                for row in domain.entries() {
                    for step in row.path() {
                        let key = domain
                            .types()
                            .find(|t| t.id() == step.owner_type_id())
                            .unwrap();
                        let shape = domain.underlying(key.id()).unwrap_or(key.id());
                        let shape = domain.types().find(|t| t.id() == shape).unwrap();
                        assert_eq!(
                            step.uses_capacity(),
                            matches!(shape, GoSourceTypeKey::Slice { .. })
                        );
                    }
                }
                json!({"inputs":inputs,"entry_paths":domain.entries().len(),"global_cases":domain.error_cases().len()})
            }
        };
        results.push(json!({"root":name,"cases":cases,"result":result}));
    }
    retain("domain-observations.json", &json!(results));
    assert_eq!(results.len(), 15);
}
#[test]
fn entry_predicates_are_complete_unique_and_bound_to_original_paths() {
    let c = corpus();
    let k = keys(&c);
    let r = root(&c, &k, "Plain");
    let s = derive_go_source_selection(&k, std::slice::from_ref(&r)).unwrap();
    let reject = |rows: &[GoErrorEntryConstraint], expected: GoSourceDomainErrorKind| {
        let e = derive_go_source_value_domain(&s, rows).unwrap_err();
        assert_eq!(*e.kind(), expected);
    };
    reject(&[], GoSourceDomainErrorKind::MissingEntry);
    let row = entries(&c, &k, "Plain", &["Code"])[0].clone();
    reject(&[row.clone(), row], GoSourceDomainErrorKind::DuplicateEntry);
    reject(
        &entries(&c, &k, "Plain", &["Code", "Alias"]),
        GoSourceDomainErrorKind::DuplicateCase,
    );
    reject(
        &[GoErrorEntryConstraint::new(r.clone(), 1, vec![], vec![])],
        GoSourceDomainErrorKind::EntryTarget,
    );
    reject(
        &[GoErrorEntryConstraint::new(
            root(&c, &k, "Scalar"),
            0,
            vec![],
            vec![],
        )],
        GoSourceDomainErrorKind::EntryTarget,
    );
    let mut extra = entries(&c, &k, "Plain", &[]);
    extra.push(GoErrorEntryConstraint::new(
        r.clone(),
        0,
        vec![GoErrorEntryStep::Field(0)],
        vec![],
    ));
    reject(&extra, GoSourceDomainErrorKind::EntryTarget);
    for name in ["Text", "Arr", "Pointer", "Wrong"] {
        reject(
            &entries(&c, &k, "Plain", &[name]),
            GoSourceDomainErrorKind::Payload,
        );
    }
    let e =
        derive_go_source_value_domain(&s, &entries(&c, &k, "Plain", &["Directed"])).unwrap_err();
    assert!(matches!(
        e.kind(),
        GoSourceDomainErrorKind::Directive { .. }
    ));
    assert!(e.source().is_some());
    // The exact path owner and capacity/length distinction are source-derived.
    let n = root(&c, &k, "Receiver");
    let selection = derive_go_source_selection(&k, &[n]).unwrap();
    let rows = entries(&c, &k, "Receiver", &["Code"]);
    let d = derive_go_source_value_domain(&selection, &rows).unwrap();
    assert_eq!(
        d.entries()
            .iter()
            .map(|r| r.input_position())
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1]
    );
    assert!(d.entries()[1].path()[1].uses_capacity());
    assert!(!d.entries()[2].path()[1].uses_capacity());
    let e = derive_go_source_value_domain(&selection, &rows[..3]).unwrap_err();
    assert_eq!(*e.kind(), GoSourceDomainErrorKind::MissingEntry);
    assert_eq!(e.input_position(), Some(1));
    retain(
        "domain-predicates.json",
        &json!({"missing":true,"duplicate_rows":true,"alias_duplicate":true,"wrong_root_or_slot":true,"extra_path":true,"excluded_payloads":4,"entry_directive":true,"receiver_first":true,"slice_capacity":true,"array_length":true}),
    );
}
#[test]
fn one_global_union_covers_body_cases_and_all_entry_rows_without_a_case_cap() {
    let c = corpus();
    let k = keys(&c);
    let r = root(&c, &k, "Box");
    let s = derive_go_source_selection(&k, &[r.clone()]).unwrap();
    let d = derive_go_source_value_domain(&s, &entries(&c, &k, "Box", &["Code"])).unwrap();
    assert_eq!(s.body_boxed_error_type_ids().len(), 1);
    assert_eq!(d.error_cases().len(), 2);
    assert_eq!(d.signature_dependencies().len(), 2);
    assert!(d.error_cases().iter().any(|i| i.value_type_id()
        == c["declarations"]["example.com/policy/helper.Other"]["type_id"]
            .as_str()
            .unwrap()));
    let scope = build_go_source_scopes(&k).unwrap();
    let sig = derive_go_source_signature(&scope, r.source_object_id(), r.node()).unwrap();
    assert_eq!(
        d.properties(sig.slots()[0].type_id())
            .unwrap()
            .value_weight_units(),
        3
    );
    // A locally nil-only []error cannot hide another input's descriptor payload.
    let slice = root(&c, &k, "Slice");
    let selected = derive_go_source_selection(&k, &[slice.clone()]).unwrap();
    let rows = [
        GoErrorEntryConstraint::new(slice.clone(), 0, vec![], vec![case(&c, &k, "Ref")]),
        GoErrorEntryConstraint::new(slice, 1, vec![GoErrorEntryStep::Elements], vec![]),
    ];
    assert_eq!(
        *derive_go_source_value_domain(&selected, &rows)
            .unwrap_err()
            .kind(),
        GoSourceDomainErrorKind::NonPlainBacking
    );
    let r = root(&c, &k, "Plain");
    let selected = derive_go_source_selection(&k, &[r.clone()]).unwrap();
    let cases = (0..70).map(|i| case(&c, &k, &format!("Case{i}"))).collect();
    let row = GoErrorEntryConstraint::new(r, 0, vec![], cases);
    let wide = derive_go_source_value_domain(&selected, &[row]).unwrap();
    assert_eq!(wide.error_cases().len(), 70);
    let scopes = build_go_source_scopes(&k).unwrap();
    let rr = root(&c, &k, "Plain");
    let sig = derive_go_source_signature(&scopes, rr.source_object_id(), rr.node()).unwrap();
    assert_eq!(
        wide.properties(sig.slots()[0].type_id())
            .unwrap()
            .value_weight_units(),
        71
    );
    assert_eq!(
        *derive_go_source_value_domain(&selected, &entries(&c, &k, "Plain", &["Huge"]))
            .unwrap_err()
            .kind(),
        GoSourceDomainErrorKind::Weight
    );
    retain(
        "domain-union.json",
        &json!({"body_and_entry_union":2,"signature_only_dependencies":2,"global_slice_rejection":true,"explicit_entry_cases":70,"oversized_payload_rejected":true}),
    );
}
#[test]
fn domains_own_their_sources_and_are_canonical_across_entry_and_root_order() {
    let c = corpus();
    let (d, other) = {
        let k = keys(&c);
        let roots = vec![root(&c, &k, "Nested"), root(&c, &k, "Plain")];
        let mut rows = entries(&c, &k, "Nested", &["Code", "Flag"]);
        rows.extend(entries(&c, &k, "Plain", &[]));
        let s = derive_go_source_selection(&k, &roots).unwrap();
        let a = derive_go_source_value_domain(&s, &rows).unwrap();
        rows.reverse();
        let mut roots = roots;
        roots.reverse();
        let b =
            derive_go_source_value_domain(&derive_go_source_selection(&k, &roots).unwrap(), &rows)
                .unwrap();
        (a, b)
    };
    assert_eq!(d, other);
    for source in d.selection().roots() {
        assert!(d.selection().source(source.source_object_id()).is_some());
    }
    retain(
        "domain-ownership.json",
        &json!({"source_ownership":true,"canonical_root_entry_order":true,"capture_or_program_proof_claimed":false}),
    );
}
