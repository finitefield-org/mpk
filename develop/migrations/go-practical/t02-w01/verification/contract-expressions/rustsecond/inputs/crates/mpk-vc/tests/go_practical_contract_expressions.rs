use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/contract-expressions")
}
fn corpus() -> Value {
    serde_json::from_slice(&std::fs::read(directory().join("corpus.json")).unwrap()).unwrap()
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
    let inputs = &c["input"];
    let sources = inputs["sources"]
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
    let keys = import_go_type_key_components(&unhex(&inputs["components_hex"]), &sources).unwrap();
    assert_eq!(keys.types().len(), 1);
    assert!(keys.declarations().is_empty());
    keys
}
fn raw(c: &Value, n: usize) -> Value {
    serde_json::from_slice(&unhex(&c["cases"][n]["raw_hex"])).unwrap()
}
fn overlay(v: &Value) -> GoSourceOverlay {
    let buffer = GoTypeSourceBuffer::new(
        "overlays",
        "candidate.json",
        "overlay",
        &serde_json::to_vec(v).unwrap(),
    )
    .unwrap();
    GoSourceOverlay::import(&buffer).unwrap()
}
fn typed(
    k: &GoTypeKeyComponents,
    v: &Value,
) -> Result<GoTypedSourceOverlaySet, GoContractTypingError> {
    derive_go_typed_source_overlays(k, &[overlay(v)])
}
fn retain(name: &str, value: &Value) {
    if let Some(path) = std::env::var_os("MPK_GO_CONTRACT_EXPRESSIONS_RUST_OUTPUT") {
        use std::io::Write;
        let path = PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        let mut bytes = serde_json::to_vec(value).unwrap();
        bytes.push(b'\n');
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .unwrap()
            .write_all(&bytes)
            .unwrap();
    }
}
fn boolean(v: bool) -> Value {
    json!({"op":"bool.literal","value":v})
}
fn nat(v: &str) -> Value {
    json!({"op":"nat.literal","value":v})
}
fn integer(v: &str) -> Value {
    json!({"op":"int.literal","value":v})
}
fn zero(t: &str) -> Value {
    json!({"op":"source.zero","type":t})
}
fn word(t: &str, n: &str) -> Value {
    json!({"op":"word.literal","type":t,"value":n})
}
fn input(n: usize) -> Value {
    json!({"op":"input","position":n})
}
fn local(n: &str) -> Value {
    json!({"op":"local","name":n})
}
fn bound(n: &str) -> Value {
    json!({"op":"bound","name":n})
}
fn unary(op: &str, a: Value) -> Value {
    json!({"op":op,"args":[a]})
}
fn binary(op: &str, a: Value, b: Value) -> Value {
    json!({"op":op,"args":[a,b]})
}
fn equality(a: Value) -> Value {
    binary("eq", a.clone(), a)
}
fn old(a: Value) -> Value {
    json!({"op":"old","value":a})
}
fn quantifier(op: &str, name: &str, lower: Value, upper: Value, body: Value) -> Value {
    json!({"op":op,"name":name,"lower":lower,"upper":upper,"body":body})
}
fn annotate(k: &GoTypeKeyComponents, v: &Value, ordinal: usize, invariant: Value) -> Value {
    // Derive selectors from the owned original source in Rust. Native expected
    // spans/types/bindings are used only for comparisons, never as inputs here.
    let source = bind_go_source_overlay(k, &overlay(v)).unwrap();
    let l = &source.loops()[ordinal];
    let node = &source
        .source(source.root().source_object_id())
        .unwrap()
        .nodes()[l.node()];
    let mut result = v.clone();
    result["loops"] = json!([{"selector":{"kind":if node.kind()=="RangeStmt"{"range"}else{"for"},"start_byte":node.start(),"end_byte":node.end(),"ordinal":ordinal},"invariants":[invariant],"variant":{"kind":"absent"}}]);
    result
}
fn ensures(v: &Value, expression: Value) -> Value {
    let mut v = v.clone();
    v["ensures"] = json!([expression]);
    v
}
fn all_nodes(e: &GoContractExpression) -> Vec<&GoContractExpression> {
    let mut result = vec![];
    let mut todo = vec![e];
    while let Some(n) = todo.pop() {
        result.push(n);
        todo.extend(n.children());
    }
    result
}
fn type_view(t: &GoContractExpressionType) -> Value {
    match t {
        GoContractExpressionType::Bool => json!({"logical":"Bool"}),
        GoContractExpressionType::Nat => json!({"logical":"Nat"}),
        GoContractExpressionType::Int => json!({"logical":"Int"}),
        GoContractExpressionType::Source(id) => json!({"source":id}),
    }
}

#[test]
fn compares_complete_actual_header_scopes_with_native_go() {
    let c = corpus();
    let k = keys(&c);
    retain("input.json", &c["input"]);
    let mut actual = vec![];
    for i in 0..4 {
        let checked = typed(&k, &raw(&c, i)).unwrap();
        let source = &checked.source_selection().overlays()[0];
        let parsed = source.source(source.root().source_object_id()).unwrap();
        let rows = checked.overlays()[0].loops().iter().map(|l| {
            let n = &parsed.nodes()[l.node()];
            let bindings = l.bindings().iter().map(|b| {
                json!({"name":b.binding().name(),"name_start_byte":parsed.nodes()[b.binding().name_node()].start(),"type_id":b.type_id()})
            }).collect::<Vec<_>>();
            json!({"selector":{"kind":if n.kind()=="RangeStmt"{"range"}else{"for"},"start_byte":n.start(),"end_byte":n.end(),"ordinal":l.ordinal()},"bindings":bindings})
        }).collect::<Vec<_>>();
        assert_eq!(json!(rows), c["cases"][i]["native"]["loops"], "root {i}");
        assert!(checked.overlays()[0]
            .loops()
            .iter()
            .all(|l| l.annotation_row().is_none()
                && l.invariants().is_empty()
                && l.variant().is_none()));
        actual.push(json!({"function":c["cases"][i]["native"]["function"],"loops":rows}));
    }
    retain("native-comparison.json", &json!(actual));
}

#[test]
fn types_all_seventy_frozen_source_heads_without_importing_historical_scope() {
    let c = corpus();
    let k = keys(&c);
    let v = raw(&c, 0);
    let source: Value =
        serde_json::from_slice(&std::fs::read(directory().join("source-witnesses.json")).unwrap())
            .unwrap();
    let mut heads = BTreeSet::new();
    let mut actual = vec![];
    for w in source["witnesses"].as_array().unwrap() {
        let op = w["op"].as_str().unwrap();
        let e = w["source"].clone();
        let logical = w["expected_logical_type"].as_str();
        let predicate = if logical == Some("Bool") {
            e.clone()
        } else if matches!(op, "input" | "old" | "slice.view2" | "slice.view3") {
            unary("is_nil", e.clone())
        } else {
            equality(e.clone())
        };
        let candidate = if w["phase"] == "ensures" {
            ensures(&v, predicate)
        } else {
            annotate(&k, &v, w["ordinal"].as_u64().unwrap() as usize, predicate)
        };
        let checked = typed(&k, &candidate).unwrap_or_else(|e| panic!("{op}: {e}"));
        let expression = if w["phase"] == "ensures" {
            &checked.overlays()[0].ensures()[0]
        } else {
            &checked.overlays()[0].loops()[w["ordinal"].as_u64().unwrap() as usize].invariants()[0]
        };
        let original = if logical == Some("Bool") {
            expression
        } else {
            &expression.children()[0]
        };
        let expected = match logical {
            Some("Bool") => json!({"logical":"Bool"}),
            Some("Nat") => json!({"logical":"Nat"}),
            Some("Int") => json!({"logical":"Int"}),
            _ => type_view(original.expression_type()),
        };
        assert_eq!(type_view(original.expression_type()), expected, "{op}");
        if w["expected_type_kind"] == "source" {
            assert!(
                matches!(
                    original.expression_type(),
                    GoContractExpressionType::Source(_)
                ),
                "{op}"
            );
        }
        for n in all_nodes(original) {
            heads.insert(n.operator().to_string());
        }
        actual.push(json!({"op":op,"type":type_view(original.expression_type()),"original_source_expression":e}));
    }
    let expected = source["witnesses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["op"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(heads, expected);
    assert_eq!(heads.len(), 70);
    retain(
        "seventy-heads.json",
        &json!({"heads":heads,"results":actual}),
    );
}

#[test]
fn rejects_type_nominal_phase_and_lexical_counterexamples() {
    let c = corpus();
    let k = keys(&c);
    let v = raw(&c, 0);
    let text = |t: &str| json!({"op":"bytes.literal","type":t,"value":"00ff"});
    let field = |value: Value, name: &str| json!({"op":"field","value":value,"field":name});
    let case = |value: Value, name: &str| json!({"op":"error.is","value":value,"case":name});
    let mut tests = vec![
        (
            "source Bool is not logical Bool",
            ensures(&v, zero("bool")),
            GoContractTypingErrorKind::Type,
        ),
        (
            "logical condition requires Bool",
            ensures(
                &v,
                json!({"op":"if","args":[input(2),boolean(true),boolean(false)]}),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "nominal words",
            ensures(
                &v,
                equality(binary(
                    "word.add",
                    zero("example.com/policy.Word"),
                    zero("example.com/policy.OtherWord"),
                )),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "logical/source words",
            ensures(
                &v,
                equality(binary("word.add", nat("1"), word("uint8", "1"))),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "nominal strings",
            ensures(
                &v,
                binary(
                    "bytes.prefix",
                    text("example.com/policy.Text"),
                    text("string"),
                ),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "logical Bool ordering",
            ensures(&v, binary("lt", boolean(false), boolean(true))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "slice equality",
            ensures(&v, equality(input(0))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "error equality",
            ensures(&v, equality(input(3))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "struct with slice equality",
            ensures(&v, equality(zero("example.com/policy.NotComparable"))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "source index",
            ensures(
                &v,
                equality(binary("element", input(0), word("uint8", "0"))),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "string capacity",
            ensures(&v, equality(unary("capacity", text("string")))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "source bool bridge",
            ensures(&v, unary("bool.value", boolean(true))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "signed bridge domain",
            ensures(&v, equality(unary("nat.from_signed", word("uint8", "1")))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "unsigned bridge domain",
            ensures(&v, equality(unary("int.from_unsigned", word("int8", "1")))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "missing direct field",
            ensures(
                &v,
                equality(field(zero("example.com/policy.Line"), "Other")),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "field source domain",
            ensures(&v, equality(field(word("uint8", "1"), "Amount"))),
            GoContractTypingErrorKind::Type,
        ),
        (
            "case source domain",
            ensures(
                &v,
                case(word("uint8", "1"), "example.com/policy.RejectCode"),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "case outside global U",
            ensures(&v, case(input(3), "example.com/policy.OtherCode")),
            GoContractTypingErrorKind::Case,
        ),
        (
            "input position",
            ensures(&v, equality(input(4))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "result position",
            ensures(&v, equality(json!({"op":"result","position":1}))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "free binder",
            ensures(&v, equality(bound("j"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "local at return",
            ensures(&v, equality(local("w"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "range at return",
            ensures(&v, equality(json!({"op":"range.index"}))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "old result",
            ensures(&v, equality(old(json!({"op":"result","position":0})))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "nested old",
            ensures(&v, equality(old(old(nat("0"))))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "binder in lower bound",
            ensures(
                &v,
                quantifier("forall", "j", bound("j"), nat("1"), boolean(true)),
            ),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "binder result type",
            ensures(
                &v,
                equality(quantifier("nat.sum", "j", nat("0"), nat("1"), integer("1"))),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "binder interval type",
            ensures(
                &v,
                quantifier("forall", "j", integer("0"), nat("1"), boolean(true)),
            ),
            GoContractTypingErrorKind::Type,
        ),
        (
            "fresh binder",
            ensures(
                &v,
                quantifier(
                    "forall",
                    "j",
                    nat("0"),
                    nat("1"),
                    quantifier("exists", "j", nat("0"), nat("1"), boolean(true)),
                ),
            ),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "hidden body local",
            annotate(&k, &v, 0, equality(local("hidden"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "sibling range local",
            annotate(&k, &v, 0, equality(local("v"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "future body local",
            annotate(&k, &v, 6, equality(local("later"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "constant hides variable",
            annotate(&k, &v, 4, equality(local("i"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "type hides variable",
            annotate(&k, &v, 5, equality(local("i"))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "old header local",
            annotate(&k, &v, 0, equality(old(local("i")))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "range on for header",
            annotate(&k, &v, 0, equality(json!({"op":"range.length"}))),
            GoContractTypingErrorKind::Scope,
        ),
        (
            "old range",
            annotate(&k, &v, 2, equality(old(json!({"op":"range.index"})))),
            GoContractTypingErrorKind::Scope,
        ),
    ];
    let mut requires = v.clone();
    requires["requires"] = json!([equality(json!({"op":"result","position":0}))]);
    tests.push((
        "result at entry",
        requires,
        GoContractTypingErrorKind::Scope,
    ));
    let mut bad_variant = annotate(&k, &v, 0, boolean(true));
    bad_variant["loops"][0]["variant"] = json!({"kind":"candidate","expression":integer("0")});
    tests.push((
        "variant requires Nat",
        bad_variant,
        GoContractTypingErrorKind::Type,
    ));
    let mut failures = vec![];
    for (name, value, kind) in tests {
        let err = typed(&k, &value).unwrap_err();
        assert_eq!(err.kind(), &kind, "{name}: {err}");
        assert!(err.overlay_object_id().is_some(), "{name}");
        failures.push(json!({"case":name,"kind":format!("{:?}",err.kind()),"phase":format!("{:?}",err.phase()),"loop":err.loop_ordinal(),"clause":err.clause(),"path":err.expression_path()}));
    }
    let nested = ensures(
        &v,
        binary("bool.and", boolean(true), unary("bool.value", nat("0"))),
    );
    let err = typed(&k, &nested).unwrap_err();
    assert_eq!(err.expression_path(), [1]);
    assert_eq!(err.phase(), Some(GoContractPhase::Ensures));
    assert_eq!(err.clause(), Some(0));
    retain("counterexamples.json", &json!(failures));
}

#[test]
fn preserves_whole_old_heap_namespaces_and_lexical_de_bruijn_depths() {
    let c = corpus();
    let k = keys(&c);
    let v = raw(&c, 0);
    let element = |descriptor: Value, index: Value| binary("element", descriptor, index);
    let a = old(element(input(0), nat("0")));
    let b = element(old(input(0)), nat("0"));
    let checked = typed(&k, &ensures(&v, binary("eq", a, b))).unwrap();
    let terms = checked.overlays()[0].ensures()[0].children();
    assert_eq!(terms[0].observation(), Some(GoContractObservation::Entry));
    assert_eq!(
        terms[0].children()[0].observation(),
        Some(GoContractObservation::Entry)
    );
    assert_eq!(terms[1].observation(), Some(GoContractObservation::Return));
    assert_eq!(
        terms[1].children()[0].observation(),
        Some(GoContractObservation::Entry)
    );
    assert_eq!(
        terms[0].children()[0].children()[0].observation(),
        Some(GoContractObservation::EntryArguments)
    );
    let formula = |outer: &str, inner: &str| {
        quantifier(
            "forall",
            outer,
            nat("0"),
            unary("length", input(0)),
            quantifier(
                "exists",
                inner,
                bound(outer),
                unary("length", input(0)),
                binary(
                    "eq",
                    old(element(input(0), bound(outer))),
                    element(input(0), bound(inner)),
                ),
            ),
        )
    };
    let first = typed(&k, &ensures(&v, formula("i", "j"))).unwrap();
    let second = typed(&k, &ensures(&v, formula("x", "y"))).unwrap();
    assert_eq!(
        first.overlays()[0].ensures(),
        second.overlays()[0].ensures()
    );
    let nodes = all_nodes(&first.overlays()[0].ensures()[0]);
    let depths = nodes
        .iter()
        .filter_map(|e| match e.reference() {
            Some(GoContractExpressionReference::Bound { depth }) => Some(*depth),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(depths, BTreeSet::from([0, 1]));
    let entry = typed(&k, &{
        let mut e = v.clone();
        e["requires"] = json!([equality(element(input(0), nat("0")))]);
        e
    })
    .unwrap();
    assert_eq!(
        entry.overlays()[0].requires()[0].children()[0].observation(),
        Some(GoContractObservation::Entry)
    );
    let header = typed(
        &k,
        &annotate(&k, &v, 2, binary("sequence.equal", input(0), input(0))),
    )
    .unwrap();
    let loop_node = header.overlays()[0].loops()[2].node();
    assert_eq!(
        header.overlays()[0].loops()[2].invariants()[0].observation(),
        Some(GoContractObservation::Header { loop_node })
    );
    // The derived object retains complete originals after the caller drops keys.
    let original_source = header.source_selection().overlays()[0]
        .root()
        .source_object_id()
        .to_string();
    drop(k);
    assert!(header
        .source_selection()
        .selection()
        .selection()
        .source(&original_source)
        .is_some());
    retain(
        "namespaces.json",
        &json!({"old_whole_element":"entry","old_descriptor_element":"return","de_bruijn_depths":depths,"alpha_equivalent":true,"original_sources_owned":true}),
    );
}

#[test]
fn checks_literal_ranges_alias_identity_and_contract_only_root_types() {
    let c = corpus();
    let k = keys(&c);
    let v = raw(&c, 0);
    let mut results = vec![];
    for (name, width, signed) in [
        ("int8", 8, true),
        ("int16", 16, true),
        ("int32", 32, true),
        ("int64", 64, true),
        ("int", 64, true),
        ("uint8", 8, false),
        ("uint16", 16, false),
        ("uint32", 32, false),
        ("uint64", 64, false),
        ("uint", 64, false),
        ("byte", 8, false),
        ("rune", 32, true),
        ("example.com/policy.Word", 8, true),
    ] {
        let (lo, hi) = if signed {
            (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
        } else {
            (0, (1i128 << width) - 1)
        };
        for n in [lo, hi] {
            typed(&k, &ensures(&v, equality(word(name, &n.to_string())))).unwrap();
        }
        for n in [lo - 1, hi + 1] {
            let err = typed(&k, &ensures(&v, equality(word(name, &n.to_string())))).unwrap_err();
            assert_eq!(
                err.kind(),
                &GoContractTypingErrorKind::Literal,
                "{name}:{n}"
            );
        }
        results.push(json!({"type":name,"min":lo.to_string(),"max":hi.to_string()}));
    }
    let huge = "9".repeat(5000);
    typed(&k, &ensures(&v, equality(nat(&huge)))).unwrap();
    typed(&k, &ensures(&v, equality(integer(&format!("-{huge}"))))).unwrap();
    assert_eq!(
        typed(&k, &ensures(&v, equality(word("uint64", &huge))))
            .unwrap_err()
            .kind(),
        &GoContractTypingErrorKind::Literal
    );
    let alias = typed(
        &k,
        &ensures(&v, equality(word("example.com/policy.WordAlias", "-128"))),
    )
    .unwrap();
    let named = typed(
        &k,
        &ensures(&v, equality(word("example.com/policy.Word", "-128"))),
    )
    .unwrap();
    assert_eq!(alias.overlays()[0].ensures(), named.overlays()[0].ensures());
    assert_ne!(alias.type_dependencies(), named.type_dependencies());
    typed(&k,&ensures(&v,binary("eq",json!({"op":"error.payload","value":input(3),"case":"example.com/policy.RejectAlias"}),zero("example.com/policy.RejectCode")))).unwrap();
    let only = typed(
        &k,
        &ensures(&v, equality(zero("example.com/policy.ContractOnly"))),
    )
    .unwrap();
    assert_eq!(only.value_domain().error_cases().len(), 1);
    assert_eq!(
        only.source_selection()
            .selection()
            .selection()
            .declarations()
            .iter()
            .filter(|d| d.kind() == "function")
            .count(),
        1
    );
    for (name, kind) in [
        ("Huge", GoSourceDomainErrorKind::Weight),
        ("BadBacking", GoSourceDomainErrorKind::NonPlainBacking),
    ] {
        let err = typed(
            &k,
            &ensures(
                &v,
                unary("is_nil", zero(&format!("example.com/policy.{name}"))),
            ),
        )
        .unwrap_err();
        let GoContractTypingErrorKind::Domain(e) = err.kind() else {
            panic!("{err}")
        };
        assert_eq!(e.kind(), &kind);
    }
    for e in [
        zero("uintptr"),
        zero("example.com/policy.Absent"),
        zero("Word"),
        zero("example.com/policy.ErrorCells"),
    ] {
        assert!(typed(&k, &ensures(&v, equality(e))).is_err());
    }
    // Exact source nominal field owner and ordered ordinal remain explicit.
    let field = typed(
        &k,
        &ensures(
            &v,
            equality(
                json!({"op":"field","value":zero("example.com/policy.Line"),"field":"Amount"}),
            ),
        ),
    )
    .unwrap();
    assert!(matches!(
        field.overlays()[0].ensures()[0].children()[0].reference(),
        Some(GoContractExpressionReference::Field { ordinal: 0, .. })
    ));
    retain(
        "literal-domains.json",
        &json!({"word_boundaries":results,"unbounded_logical_digits":5000,"aliases_canonical":true,"contract_only_body_selected":false,"closed_error_cases":1}),
    );
}

#[test]
fn retains_partial_operations_as_typed_candidates_without_claiming_safety() {
    let c = corpus();
    let k = keys(&c);
    let v = raw(&c, 0);
    let expressions = vec![
        binary("word.div", word("int8", "1"), word("int8", "0")),
        binary("word.shl", word("int8", "1"), word("int64", "-1")),
        unary("nat.from_signed", word("int64", "-1")),
        binary(
            "byte_at",
            json!({"op":"bytes.literal","type":"string","value":""}),
            nat("999999999999999999999999999"),
        ),
        binary("element", input(0), nat("999999999999999999999999999")),
        json!({"op":"error.payload","value":zero("error"),"case":"example.com/policy.RejectCode"}),
    ];
    let mut operators = vec![];
    for e in expressions {
        operators.push(e["op"].clone());
        typed(&k, &ensures(&v, equality(e))).unwrap();
    }
    let reslice = json!({"op":"slice.view3","args":[input(0),nat("99"),nat("98"),nat("97")]});
    typed(&k, &ensures(&v, unary("is_nil", reslice))).unwrap();
    let concat = json!({"op":"bytes.concat","args":[{"op":"bytes.literal","type":"string","value":"ff".repeat(256)},{"op":"bytes.literal","type":"string","value":"ff".repeat(256)}]});
    typed(&k, &ensures(&v, equality(concat))).unwrap();
    retain(
        "partial-operations.json",
        &json!({"typed_operators":operators,"reslice_and_concat_typed":true,"definedness_or_runtime_proof":false}),
    );
}
