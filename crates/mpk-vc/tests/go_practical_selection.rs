use mpk_vc::go_practical_selection::{import_go_practical_selection, GoPracticalSelectionError};
use mpk_vc::{canonical_json_bytes, parse_strict_json, StrictJsonLimits, StrictJsonValue};
use serde_json::{json, Value};

fn strict(value: &Value) -> StrictJsonValue {
    parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(4_194_304, 1_000_000, 64, 1_000_000),
    )
    .unwrap()
}

fn root(function: &str, receiver: Value) -> Value {
    json!({"package": "example.test/quote", "function": function, "receiver": receiver})
}

fn selection(roots: Vec<Value>) -> Value {
    json!({"schema":"mpk.selection.go_functions.v1", "value":{"roots": roots}})
}

#[test]
fn frozen_go_request_selection_corpus_is_preserved() {
    let rows: Vec<Value> = serde_json::from_slice(include_bytes!(
        "../../../develop/migrations/go-practical/t01-w04/context-protocol/vectors.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 152, "retain the complete frozen corpus");
    let mut accepted = 0;
    let mut rejected = 0;
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        if id != "request-go_practical"
            && !id.starts_with("request-go-")
            && !id.starts_with("selection-reject-")
        {
            continue;
        }
        let value = &row["value"]["selection"];
        let result = import_go_practical_selection(&strict(value));
        let expected = row["expected_accepted"].as_bool().unwrap();
        assert_eq!(
            result.is_ok(),
            expected,
            "frozen selection {id}: {result:?}"
        );
        if expected {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (6, 23));
}

#[test]
fn selection_identity_and_raw_receiver_selectors_follow_the_frozen_preimage() {
    let value = selection(vec![root("Quote", Value::Null)]);
    let parsed = import_go_practical_selection(&strict(&value)).unwrap();
    assert_eq!(
        parsed.canonical_bytes(),
        br#"{"schema":"mpk.selection.go_functions.v1","value":{"roots":[{"function":"Quote","package":"example.test/quote","receiver":null}]}}"#
    );
    // Independently produced by the frozen Python canonical/hash observer.
    assert_eq!(
        parsed.selection_sha256(),
        "198d3169e9678149f2fa72f3c415c041e3f9d6d379eab0a0d53e12d392064059"
    );
    assert_eq!(parsed.roots()[0].package(), "example.test/quote");
    assert_eq!(parsed.roots()[0].function(), "Quote");
    assert!(parsed.roots()[0].receiver().is_none());

    let receiver = json!({"package":"example.test/quote", "type_name":"LedgerAlias"});
    let parsed = import_go_practical_selection(&strict(&selection(vec![root("init", receiver)])))
        .expect("method init is distinct from package init");
    let receiver = parsed.roots()[0].receiver().unwrap();
    assert_eq!(receiver.package(), "example.test/quote");
    assert_eq!(receiver.type_name(), "LedgerAlias");

    // Raw selector syntax is intentionally checked before source/package
    // resolution; this layer must not apply the later captured-path grammar.
    let raw_selector = json!({"package":"example.test//quote", "function":"見積", "receiver":null});
    assert!(import_go_practical_selection(&strict(&selection(vec![raw_selector]))).is_ok());
}

#[test]
fn selection_count_and_canonical_order_have_exact_boundaries() {
    let mut roots = (0..8192)
        .map(|i| root(&format!("F{i}"), Value::Null))
        .collect::<Vec<_>>();
    roots.sort_by_cached_key(|row| canonical_json_bytes(&strict(row)).unwrap());
    let value = selection(roots.clone());
    assert_eq!(
        import_go_practical_selection(&strict(&value))
            .unwrap()
            .roots()
            .len(),
        8192
    );
    roots.push(root("Beyond", Value::Null));
    roots.sort_by_cached_key(|row| canonical_json_bytes(&strict(row)).unwrap());
    assert_eq!(
        import_go_practical_selection(&strict(&selection(roots))),
        Err(GoPracticalSelectionError::Limit)
    );
    let duplicate = selection(vec![root("A", Value::Null), root("A", Value::Null)]);
    assert_eq!(
        import_go_practical_selection(&strict(&duplicate)),
        Err(GoPracticalSelectionError::Order)
    );
}

#[test]
fn constructed_duplicate_json_members_do_not_gain_an_identity() {
    let duplicate = StrictJsonValue::Object(vec![
        (
            "schema".into(),
            StrictJsonValue::String("mpk.selection.go_functions.v1".into()),
        ),
        (
            "schema".into(),
            StrictJsonValue::String("mpk.selection.go_functions.v1".into()),
        ),
    ]);
    assert_eq!(
        import_go_practical_selection(&duplicate),
        Err(GoPracticalSelectionError::Json)
    );
}
