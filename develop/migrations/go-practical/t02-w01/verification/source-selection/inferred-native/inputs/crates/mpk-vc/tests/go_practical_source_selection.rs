use mpk_vc::go_practical_type_keys::{
    derive_go_source_selection, import_go_type_key_components, GoSourceSelection,
    GoSourceSelectionRef, GoTypeKeyComponents, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-selection")
}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_SELECTION_RUST_OUTPUT") {
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
fn keys(input: &Value) -> std::result::Result<GoTypeKeyComponents, String> {
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
    Ok(keys)
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

fn control(label: &str, text: &str) -> (Value, GoTypeKeyComponents) {
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
    let graph = keys(&input).unwrap();
    (input, graph)
}

fn locator(
    keys: &GoTypeKeyComponents,
    source: &str,
    start: usize,
    end: usize,
) -> GoSourceSelectionRef {
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| n.kind() == "FuncDecl" && n.start() == start && n.end() == end)
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn reference(s: &GoSourceSelection, r: &GoSourceSelectionRef) -> Value {
    let node = &s.source(r.source_object_id()).unwrap().nodes()[r.node()];
    let pkg = s
        .packages()
        .iter()
        .find(|p| p.source_objects.iter().any(|id| id == r.source_object_id()))
        .unwrap();
    json!({"package":pkg.import_path,"source":r.source_object_id(),"start":node.start(),"end":node.end()})
}
fn normalized(mut rows: Vec<Value>) -> Value {
    rows.sort_by_key(|v| serde_json::to_string(v).unwrap());
    json!(rows)
}
fn output(s: &GoSourceSelection) -> Value {
    let declarations = s
        .declarations()
        .iter()
        .map(|d| {
            let mut roles = d.roles().map(|r| r.as_str()).collect::<Vec<_>>();
            roles.sort();
            json!({"ref":reference(s,d.locator()),"kind":d.kind(),"names":d.names(),"roles":roles})
        })
        .collect();
    let calls=s.calls().iter().map(|c|{let n=&s.source(c.caller().source_object_id()).unwrap().nodes()[c.call_node()];json!({"caller":reference(s,c.caller()),"callee":reference(s,c.callee()),"source":c.caller().source_object_id(),"start":n.start(),"end":n.end()})}).collect();
    let dependencies=s.dependencies().iter().map(|d|json!({"from":reference(s,d.from()),"to":reference(s,d.to()),"role":d.role().as_str()})).collect();
    json!({"declarations":normalized(declarations),"calls":normalized(calls),"dependencies":normalized(dependencies)})
}
#[test]
fn reconstructs_original_selected_functions_without_native_type_facts() {
    let cases: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases = cases.as_array().unwrap();
    assert_eq!(cases.len(), 31);
    let (mut positive, mut negative) = (0, 0);
    let mut failures = vec![];
    for (n, row) in cases.iter().enumerate() {
        let input = &row["input"];
        retain(&format!("case-{n:02}-input.json"), input);
        let keys = keys(input).unwrap();
        let r = &input["root"];
        let r = locator(
            &keys,
            r["source"].as_str().unwrap(),
            r["start"].as_u64().unwrap() as usize,
            r["end"].as_u64().unwrap() as usize,
        );
        let result = derive_go_source_selection(&keys, &[r]);
        let actual = match &result {
            Ok(s) => output(s),
            Err(e) => json!({"error":format!("{e:?}")}),
        };
        retain(&format!("case-{n:02}-actual.json"), &actual);
        if input["expected_issue_codes"].is_null() {
            positive += 1;
            if let Ok(s) = &result {
                let native = &row["native"];
                let calls = normalized(native["calls"].as_array().unwrap().clone());
                if actual["calls"] != calls {
                    failures.push(format!(
                        "case {n} calls expected {calls} actual {}",
                        actual["calls"]
                    ))
                }
                let deps = normalized(native["dependencies"].as_array().unwrap().clone());
                if actual["dependencies"] != deps {
                    failures.push(format!(
                        "case {n} dependencies expected {deps} actual {}",
                        actual["dependencies"]
                    ))
                }
                let expected=normalized(native["declarations"].as_array().unwrap().iter().map(|d|json!({"ref":d["ref"],"kind":d["kind"],"names":d["names"],"roles":d["roles"]})).collect());
                if actual["declarations"] != expected {
                    failures.push(format!("case {n} declaration mismatch"))
                }
                assert!(!s
                    .source(s.roots()[0].source_object_id())
                    .unwrap()
                    .source()
                    .bytes()
                    .is_empty());
            } else {
                failures.push(format!("case {n} positive failed: {actual}"))
            }
        } else {
            negative += 1;
            if result.is_ok() {
                failures.push(format!("case {n} negative accepted"))
            }
        }
    }
    retain(
        "summary.json",
        &json!({"positive":positive,"negative":negative,"failures":failures}),
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn selection_owns_sources_and_rejects_nonoriginal_roots() {
    let(input,keys)=control("selection-owned","package policy\nfunc Root(x uint16)uint16{return Sub(x)}\nfunc Sub(x uint16)uint16{return x+1}\n");
    let source = input["sources"][0]["object"]["id"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, n)| {
            n.kind() == "FuncDecl"
                && parsed.spelling(
                    n.edges()
                        .iter()
                        .find(|e| e.role() == "Name")
                        .unwrap()
                        .targets()[0],
                ) == Some(b"Root")
        })
        .unwrap()
        .0;
    let r = GoSourceSelectionRef::new(source, node);
    let selected = derive_go_source_selection(&keys, std::slice::from_ref(&r)).unwrap();
    assert_eq!(selected.calls().len(), 1);
    assert!(derive_go_source_selection(&keys, &[r.clone(), r]).is_err());
    assert!(derive_go_source_selection(&keys, &[]).is_err());
    assert!(
        derive_go_source_selection(&keys, &[GoSourceSelectionRef::new(source, usize::MAX)])
            .is_err()
    );
    retain("owned-actual.json", &output(&selected));
    drop(keys);
    drop(input);
    assert!(!selected
        .source(selected.roots()[0].source_object_id())
        .unwrap()
        .source()
        .bytes()
        .is_empty());
}

#[test]
fn many_original_roots_are_canonical_and_have_no_small_helper_or_parameter_cap() {
    let cases: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let input = &cases[0]["input"];
    retain("many-roots-input.json", input);
    let keys = keys(input).unwrap();
    let source = input["root"]["source"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let roots = (0..70)
        .map(|i| {
            let name = format!("Root{i}");
            let node = parsed
                .nodes()
                .iter()
                .enumerate()
                .find(|(_, n)| {
                    n.kind() == "FuncDecl"
                        && n.edges()
                            .iter()
                            .find(|e| e.role() == "Name")
                            .is_some_and(|e| {
                                parsed.spelling(e.targets()[0]) == Some(name.as_bytes())
                            })
                })
                .unwrap()
                .0;
            GoSourceSelectionRef::new(source, node)
        })
        .collect::<Vec<_>>();
    let selected = derive_go_source_selection(&keys, &roots).unwrap();
    assert_eq!(selected.roots().len(), 70);
    assert_eq!(selected.calls().len(), 70);
    let mut reversed = roots;
    reversed.reverse();
    let again = derive_go_source_selection(&keys, &reversed).unwrap();
    assert_eq!(selected, again);
    retain("many-roots-actual.json", &output(&selected));
}
