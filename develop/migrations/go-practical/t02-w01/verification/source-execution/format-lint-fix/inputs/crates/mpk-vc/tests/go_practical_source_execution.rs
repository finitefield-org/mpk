use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use GoSourceRuntimeData as D;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-execution")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(name)).unwrap()).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn selection() -> (GoSourceErrorSelection, GoSourceScopes) {
    let c = read("corpus.json");
    let input = &c["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys =
        import_go_type_key_components(&unhex(input["components_hex"].as_str().unwrap()), &sources)
            .unwrap();
    let scopes = build_go_source_scopes(&keys).unwrap();
    let locate = |r: &Value, kind: &str| {
        let source = r["source"].as_str().unwrap();
        let node = scopes
            .source(source)
            .unwrap()
            .nodes()
            .iter()
            .position(|n| {
                n.kind() == kind
                    && n.start() == r["start"].as_u64().unwrap() as usize
                    && n.end() == r["end"].as_u64().unwrap() as usize
            })
            .unwrap();
        GoSourceSelectionRef::new(source, node)
    };
    let roots = c["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "TypeSpec"))
        .collect();
    let selected = derive_go_source_error_selection(
        &keys,
        &roots,
        &[GoErrorEntryConstraint::new(
            locate(&c["entry"], "FuncDecl"),
            0,
            vec![],
            cases,
        )],
    )
    .unwrap_or_else(|error| {
        if let GoSourceErrorSelectionError::Selection(e) = &error {
            let parsed = scopes.source(e.source_object_id()).unwrap();
            let node = &parsed.nodes()[e.node()];
            eprintln!(
                "actual failing source {} {:?} bytes {}..{}: {}",
                e.source_object_id(),
                node.kind(),
                node.start(),
                node.end(),
                String::from_utf8_lossy(&parsed.source().bytes()[node.start()..node.end()])
            );
        }
        panic!("{error:?}")
    });
    (selected, scopes)
}
fn shape<'a>(d: &'a GoSourceValueDomain, typ: &str) -> &'a GoSourceTypeKey {
    let base = d.underlying(typ).unwrap_or(typ);
    d.types().find(|t| t.id() == base).unwrap()
}
fn entry(s: &GoSourceErrorSelection, name: &str) -> GoSourceSelectionRef {
    s.selection()
        .roots()
        .iter()
        .find(|r| {
            let p = s.selection().source(r.source_object_id()).unwrap();
            let n = &p.nodes()[r.node()];
            let name_node = n
                .edges()
                .iter()
                .find(|e| e.role() == "Name")
                .unwrap()
                .targets()[0];
            p.nodes()[name_node]
                .attributes()
                .any(|(k, v)| k == "Name" && v == name)
        })
        .unwrap()
        .clone()
}
fn signature(scopes: &GoSourceScopes, r: &GoSourceSelectionRef) -> GoSourceSignature {
    derive_go_source_signature(scopes, r.source_object_id(), r.node()).unwrap()
}

fn input(
    d: &GoSourceValueDomain,
    typ: &str,
    row: &Value,
    previous: &[GoSourceRuntimeValue],
) -> GoSourceRuntimeValue {
    if let Some(i) = row["alias"].as_u64() {
        let v = previous[i as usize].clone();
        assert_eq!(v.type_id(), typ);
        return v;
    }
    let data = match shape(d, typ) {
        GoSourceTypeKey::Builtin { name, .. } if name == "bool" => {
            D::Bool(row["bool"].as_bool().unwrap())
        }
        GoSourceTypeKey::Builtin { name, .. } if name == "string" => {
            D::Bytes(unhex(row["bytes"].as_str().unwrap()))
        }
        GoSourceTypeKey::Builtin { name, .. } if name == "error" => {
            if row["nil"] == true {
                D::Error(None)
            } else {
                let case = row["error"]["case"].as_str().unwrap();
                let id = case_type(d, case);
                D::Error(Some(Box::new(input(
                    d,
                    &id,
                    &json!({"word":row["error"]["word"]}),
                    &[],
                ))))
            }
        }
        GoSourceTypeKey::Builtin { width: Some(w), .. } => {
            let n = row["word"].as_str().unwrap().parse::<i128>().unwrap();
            D::Word((n as u64) & if *w == 64 { u64::MAX } else { (1u64 << w) - 1 })
        }
        GoSourceTypeKey::Array { element, .. } => D::Aggregate(
            row["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| input(d, element, v, &[]))
                .collect(),
        ),
        GoSourceTypeKey::Struct { fields, .. } => D::Aggregate(
            row["items"]
                .as_array()
                .unwrap()
                .iter()
                .zip(fields)
                .map(|(v, f)| input(d, &f.type_id, v, &[]))
                .collect(),
        ),
        GoSourceTypeKey::Slice { element, .. } => D::Slice(if row["nil"] == true {
            GoSourceRuntimeSlice::nil()
        } else {
            let s = &row["slice"];
            GoSourceRuntimeSlice::from_backing(
                s["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| input(d, element, v, &[]))
                    .collect(),
                s["length"].as_u64().unwrap() as usize,
            )
            .unwrap()
        }),
        _ => panic!("input type {typ}"),
    };
    GoSourceRuntimeValue::new(typ, data)
}
fn case_type(d: &GoSourceValueDomain, name: &str) -> String {
    d.error_cases()
        .iter()
        .find(|e| {
            let method = e.method();
            let parsed = d
                .selection()
                .source(method.declaration_source_object_id().unwrap())
                .unwrap();
            let f = &parsed.nodes()[method.declaration_node().unwrap()];
            let recv = f
                .edges()
                .iter()
                .find(|e| e.role() == "Recv")
                .unwrap()
                .targets()[0];
            let fields = &parsed.nodes()[recv];
            let field = fields
                .edges()
                .iter()
                .find(|e| e.role() == "List")
                .unwrap()
                .targets()[0];
            let ty = parsed.nodes()[field]
                .edges()
                .iter()
                .find(|e| e.role() == "Type")
                .unwrap()
                .targets()[0];
            parsed.nodes()[ty]
                .attributes()
                .any(|(k, v)| k == "Name" && v == name)
        })
        .unwrap()
        .value_type_id()
        .into()
}
fn describe(d: &GoSourceValueDomain, v: &GoSourceRuntimeValue) -> Value {
    match v.data() {
        D::Nil => panic!("unmaterialized nil"),
        D::Bool(b) => json!({"kind":"bool","value":b}),
        D::Word(n) => {
            let GoSourceTypeKey::Builtin { width: Some(w), .. } = shape(d, v.type_id()) else {
                panic!()
            };
            json!({"kind":"word","bits":n.to_string(),"width":w})
        }
        D::Bytes(b) => json!({"kind":"bytes","hex":hex(b)}),
        D::Aggregate(xs) => {
            json!({"kind":"aggregate","items":xs.iter().map(|v|describe(d,v)).collect::<Vec<_>>()})
        }
        D::Slice(s) => {
            json!({"kind":"slice","nil":s.is_nil(),"length":s.length(),"capacity":s.capacity(),"items":s.elements().iter().map(|v|describe(d,v)).collect::<Vec<_>>()})
        }
        D::Error(e) => e
            .as_ref()
            .map_or_else(|| json!({"kind":"error","nil":true}), |v| describe(d, v)),
    }
}
fn write(name: &str, data: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_SOURCE_EXECUTION_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(data).unwrap())
            .unwrap();
    }
}
#[test]
fn original_source_execution_matches_complete_native_matrix() {
    let (s, scopes) = selection();
    let d = s.domain();
    assert_eq!(d.error_cases().len(), 70);
    let v = read("vectors.json");
    let native = read("native-values.json");
    let mut failures = vec![];
    let mut rows = vec![];
    for row in v["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let root = entry(&s, row["entry"].as_str().unwrap());
        let sig = signature(&scopes, &root);
        let types = sig
            .slots()
            .iter()
            .filter(|s| s.role() != GoSourceSignatureRole::Result)
            .map(|s| s.type_id().to_string())
            .collect::<Vec<_>>();
        let mut arguments = vec![];
        for (t, a) in types.iter().zip(row["args"].as_array().unwrap()) {
            arguments.push(input(d, t, a, &arguments));
        }
        let actual = execute_go_source(d, &root, arguments.clone(), 100000);
        let expected = native
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .unwrap();
        let observation = match &actual {
            Ok(run) => {
                let (result, fault) = match &run.status {
                    GoSourceExecutionStatus::Returned(v) => (
                        v.iter().map(|v| describe(d, v)).collect::<Vec<_>>(),
                        Value::Null,
                    ),
                    GoSourceExecutionStatus::Fault(f) => (vec![], json!(format!("{f:?}"))),
                    GoSourceExecutionStatus::ObservationLimit => {
                        (vec![], json!("ObservationLimit"))
                    }
                };
                json!({"name":name,"entry":row["entry"],"result":result,"fault":fault,"after":arguments.iter().map(|v|describe(d,v)).collect::<Vec<_>>(),"steps":run.steps,"events":run.events.iter().map(|e|json!({"source":e.origin.source_object_id(),"node":e.origin.node(),"invocation":e.invocation,"kind":format!("{:?}",e.kind)})).collect::<Vec<_>>()})
            }
            Err(e) => {
                json!({"name":name,"unavailable":format!("{:?}",e.kind),"node":e.stopped_at.node(),"source":e.stopped_at.source_object_id(),"steps":e.steps})
            }
        };
        if observation["result"] != expected["result"]
            || observation["fault"] != expected["fault"]
            || observation["after"] != expected["after"]
        {
            failures.push(json!({"name":name,"actual":observation,"expected":expected}));
        }
        if name == "joint-once" {
            if let Ok(run) = &actual {
                let pair = entry(&s, "PairCall");
                assert_eq!(
                    run.events
                        .iter()
                        .filter(|e| e.kind == GoSourceExecutionEventKind::Enter && e.origin == pair)
                        .count(),
                    1
                );
            }
        }
        rows.push(observation);
    }
    write(
        "matrix.json",
        &json!({"rows":rows,"failures":failures,"complete_cases":127,"full_error_cases":70,"certificate_or_context_authority":false}),
    );
    assert!(
        failures.is_empty(),
        "{}",
        serde_json::to_string_pretty(&failures).unwrap()
    );
}
#[test]
fn source_execution_observation_limit_and_input_rejections_are_distinct() {
    let (s, _scopes) = selection();
    let d = s.domain();
    let spin = entry(&s, "Spin");
    let mut seen = vec![];
    for budget in [0, 1, 7, 19] {
        let run = execute_go_source(d, &spin, vec![], budget).unwrap();
        assert_eq!(run.status, GoSourceExecutionStatus::ObservationLimit);
        assert_eq!(run.steps, budget);
        assert_eq!(run.events.len(), budget as usize);
        seen.push(json!({"budget":budget,"steps":run.steps,"status":"ObservationLimit"}));
    }
    let root = entry(&s, "Length");
    let result = execute_go_source(
        d,
        &root,
        vec![GoSourceRuntimeValue::new("forged", D::Bytes(vec![]))],
        100,
    );
    assert_eq!(result.unwrap_err().kind, GoSourceExecutionErrorKind::Input);
    let not_root = GoSourceSelectionRef::new("missing", 0);
    assert_eq!(
        execute_go_source(d, &not_root, vec![], 100)
            .unwrap_err()
            .kind,
        GoSourceExecutionErrorKind::Entry
    );
    write(
        "boundaries.json",
        &json!({"observation_limits":seen,"unknown_entry":"Entry","forged_source_type":"Input","no_kernel_or_source_limit_changed":true}),
    );
}
