use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes, check_go_function_body, evaluate_go_constant_binding,
    import_go_type_key_components, GoEvaluatedConstant, GoSourceScopes, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/aggregate-constants")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_AGGREGATE_CONSTANTS_RUST_OUTPUT") {
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
fn original_array_constants_and_contextual_iota() {
    let (input, graph) = control(
        "aggregate-constant-basic",
        r#"package policy
type W uint16
type A=[3]W
var array A
var inferred=[4]W{}
func Probe(a A){const(Result=len(a);_);_=Result}
const(I=len([iota+1]W{});J;K)
const N=cap(inferred)
"#,
    );
    let mut output = vec![];
    for (name, want) in [
        ("Result", "3"),
        ("I", "1"),
        ("J", "2"),
        ("K", "3"),
        ("N", "4"),
    ] {
        let b = graph.bindings().iter().find(|b| b.name() == name).unwrap();
        let value = evaluate_go_constant_binding(&graph, b).unwrap();
        assert_eq!(value.integer_decimal().as_deref(), Some(want));
        output.push(json!({"name":name,"value":value.integer_decimal()}));
    }
    retain("aggregate-basic-actual.json", &json!(output));
    drop(input);
}
#[test]
fn local_array_reads_and_cross_dependency_cycles() {
    let (input, graph) = control(
        "aggregate-local-read",
        "package policy\nfunc Probe(){var a [2]byte;const N=len(a);var _ [N]byte}\n",
    );
    let source = input["sources"][0]["object"]["id"].as_str().unwrap();
    let node = graph
        .source(source)
        .unwrap()
        .nodes()
        .iter()
        .position(|n| n.kind() == "FuncDecl")
        .unwrap();
    let body = check_go_function_body(&graph, source, node).unwrap();
    assert_eq!(body.local_variables().len(), 1);
    retain(
        "aggregate-local-actual.json",
        &json!({"locals":body.local_variables().len()}),
    );
    let (_, graph) = control(
        "aggregate-cycle",
        "package policy\nconst N=len(a)\nvar a [N]byte\n",
    );
    let b = graph.bindings().iter().find(|b| b.name() == "N").unwrap();
    assert!(evaluate_go_constant_binding(&graph, b).is_err());
}

fn output(value: &GoEvaluatedConstant) -> Value {
    json!({"accepted":true,"kind":value.kind().as_str(),"integer_decimal":value.integer_decimal(),"boolean":value.boolean(),"bytes_hex":value.bytes().map(|raw|raw.iter().map(|b|format!("{b:02x}")).collect::<String>()),"type_key":value.scalar_type().map(|t|t.source_type_key()),"underlying_builtin":value.scalar_type().map(|t|t.underlying_builtin()),"source_typing_or_context_or_program_authority":false})
}
#[test]
fn matches_original_go_aggregate_constant_reference() {
    let corpus: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases = corpus.as_array().unwrap();
    assert!(cases.len() > 350);
    let (mut accepted, mut rejected, mut outside) = (0, 0, 0);
    let mut failures = vec![];
    for (n, row) in cases.iter().enumerate() {
        let input = &row["input"];
        retain(&format!("case-{n:05}-input.json"), input);
        let result = scopes(input).and_then(|graph| {
            let b = graph
                .bindings()
                .iter()
                .find(|b| {
                    b.source_object_id() == input["query_source_object_id"].as_str().unwrap()
                        && b.name() == "Result"
                })
                .ok_or("binding".to_string())?;
            evaluate_go_constant_binding(&graph, b).map_err(|e| format!("{e:?}"))
        });
        let actual = match &result {
            Ok(v) => output(v),
            Err(e) => {
                json!({"accepted":false,"error":e,"source_typing_or_context_or_program_authority":false})
            }
        };
        retain(&format!("case-{n:05}-actual.json"), &actual);
        if input["outside_aggregate_reader"] == true && row["native"]["accepted"] == true {
            if result.is_ok() {
                failures.push(format!("outside {n} {} accepted", input["label"]))
            }
            outside += 1;
        } else if row["native"]["accepted"] == true {
            if actual != row["native"] {
                failures.push(format!(
                    "case {n} {} expected {} actual {actual}",
                    input["label"], row["native"]
                ))
            }
            accepted += 1;
        } else {
            if result.is_ok() {
                failures.push(format!(
                    "case {n} {} falsely accepted {actual}",
                    input["label"]
                ))
            }
            rejected += 1;
        }
    }
    retain(
        "summary.json",
        &json!({"accepted":accepted,"rejected":rejected,"outside":outside,"failures":failures}),
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn flat_cross_reader_dependencies_and_thread_sessions() {
    let count = 512;
    let mut text = "package policy\nconst C0=1\n".to_string();
    for n in 1..=count {
        text += &format!("var a{n}[C{}]byte;const C{n}=len(a{n})\n", n - 1);
    }
    text += &format!("const Result=C{count}\n");
    let (input, graph) = control("aggregate-deep-dependencies", &text);
    let b = graph
        .bindings()
        .iter()
        .find(|b| b.name() == "Result")
        .unwrap();
    let result = evaluate_go_constant_binding(&graph, b).unwrap();
    assert_eq!(result.integer_decimal().as_deref(), Some("1"));
    retain(
        "aggregate-deep-actual.json",
        &json!({"cross_reader_links":count,"result":output(&result)}),
    );
    drop(input);
    drop(graph);
    std::thread::scope(|scope| {
        for n in 0..4 {
            scope.spawn(move || {
                let (_, graph) = control(
                    &format!("aggregate-thread-{n}"),
                    &format!("package policy\nvar a[{}]byte;const Result=cap(a)\n", n + 1),
                );
                let b = graph
                    .bindings()
                    .iter()
                    .find(|b| b.name() == "Result")
                    .unwrap();
                let v = evaluate_go_constant_binding(&graph, b).unwrap();
                assert_eq!(v.integer_decimal().unwrap(), (n + 1).to_string());
                let (_, bad) = control(
                    &format!("aggregate-thread-bad-{n}"),
                    "package policy\nvar a[N]byte;const N=len(a)\n",
                );
                let b = bad.bindings().iter().find(|b| b.name() == "N").unwrap();
                assert!(evaluate_go_constant_binding(&bad, b).is_err());
                let again = evaluate_go_constant_binding(
                    &graph,
                    graph
                        .bindings()
                        .iter()
                        .find(|b| b.name() == "Result")
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(again, v);
                retain(&format!("aggregate-thread-{n}-actual.json"), &output(&v));
            });
        }
    });
}
