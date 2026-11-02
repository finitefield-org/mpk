use mpk_vc::go_practical_type_keys::{parse_go_source, GoParsedSource, GoTypeSourceBuffer};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2));
    text.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn record(parsed: &GoParsedSource) -> Value {
    let nodes: Vec<_> = parsed
        .nodes()
        .iter()
        .map(|n| {
            let attrs: serde_json::Map<String, Value> =
                n.attributes().map(|(k, v)| (k.into(), json!(v))).collect();
            let edges: Vec<_> = n
                .edges()
                .iter()
                .map(|e| json!({"role":e.role(),"targets":e.targets()}))
                .collect();
            json!({"kind":n.kind(),"start":n.start(),"end":n.end(),"attrs":attrs,"edges":edges})
        })
        .collect();
    json!({"accepted":true,"nodes":nodes,"root":parsed.root(),"source_typing_or_program_authority":false})
}
fn save(name: &str, value: &Value) {
    if let Some(root) = std::env::var_os("MPK_GO_PARSE_RUST_OUTPUT") {
        use std::io::Write;
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(format!("{name}.json")))
            .unwrap()
            .write_all(&serde_json::to_vec(value).unwrap())
            .unwrap();
    }
}
fn compare(name: &str, source: &GoTypeSourceBuffer, native: &Value) -> (bool, usize) {
    let parsed = parse_go_source(source);
    let actual = match &parsed {
        Ok(parsed) => record(parsed),
        Err(e) => {
            json!({"accepted":false,"offset":e.offset,"error_kind":format!("{:?}",e.kind),"source_typing_or_program_authority":false})
        }
    };
    save(name, &actual);
    assert_eq!(
        actual["accepted"],
        native["accepted"],
        "{name}: actual {actual}; native {:?}",
        native.get("error")
    );
    if let Ok(parsed) = parsed {
        assert_eq!(parsed.source(), source);
        assert_eq!(parsed.root(), 0);
        if actual != *native {
            // Native label is reference provenance rather than syntax metadata.
            let nodes = actual["nodes"].as_array().unwrap();
            let expected = native["nodes"].as_array().unwrap();
            let index = nodes
                .iter()
                .zip(expected)
                .position(|(a, b)| a != b)
                .unwrap_or(nodes.len().min(expected.len()));
            if nodes != expected {
                panic!("{name}: first AST difference {index}: Rust {:?}; Go {:?}; total nodes {} vs {}",nodes.get(index),expected.get(index),nodes.len(),expected.len());
            }
        }
        for (id, node) in parsed.nodes().iter().enumerate() {
            assert!(node.start() <= node.end() && node.end() <= source.bytes().len());
            assert_eq!(
                parsed.spelling(id),
                Some(&source.bytes()[node.start()..node.end()])
            );
            for edge in node.edges() {
                assert!(edge.targets().iter().all(|t| *t < parsed.nodes().len()))
            }
        }
        (true, parsed.nodes().len())
    } else {
        (false, 0)
    }
}
#[test]
fn independent_original_go_syntax() {
    let mut accepted = 0;
    let mut rejected = 0;
    let mut nodes = 0;
    if let Some(root) = std::env::var_os("MPK_GO_PARSE_CORPUS") {
        let mut entries: Vec<_> = std::fs::read_dir(root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("case-")
            })
            .collect();
        entries.sort();
        assert!(entries.len() >= 2934);
        for entry in entries {
            let raw = std::fs::read(entry.join("source.raw")).unwrap();
            let object = std::fs::read(entry.join("source.json")).unwrap();
            let source = GoTypeSourceBuffer::import(&object, &raw).unwrap();
            let native: Value =
                serde_json::from_slice(&std::fs::read(entry.join("native.json")).unwrap()).unwrap();
            let (ok, n) = compare(
                entry.file_name().unwrap().to_str().unwrap(),
                &source,
                &native,
            );
            if ok {
                accepted += 1
            } else {
                rejected += 1
            };
            nodes += n;
        }
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/go-practical/t02-w01/verification/go-parse/smoke.json");
        let cases: Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
        assert!(cases.as_array().unwrap().len() > 30);
        for (n, case) in cases.as_array().unwrap().iter().enumerate() {
            let source = GoTypeSourceBuffer::import(
                &serde_json::to_vec(&case["object"]).unwrap(),
                &unhex(case["raw_hex"].as_str().unwrap()),
            )
            .unwrap();
            let (ok, count) = compare(&format!("smoke-{n:04}"), &source, &case["native"]);
            if ok {
                accepted += 1
            } else {
                rejected += 1
            };
            nodes += count;
        }
    }
    println!("actual independent syntax cases: accepted={accepted}, rejected={rejected}, exact syntax nodes={nodes}");
    save(
        "summary",
        &json!({"accepted":accepted,"rejected":rejected,"nodes":nodes,"source_typing_or_program_authority":false}),
    );
    assert!(accepted > 0 && rejected > 0);
}
#[test]
fn source_spans_and_ast_children_remain_owned() {
    let mut raw=b"package p\r\n//line elsewhere.go:100:2\r\nconst X = `a\r\nb`\r\ntype T = struct{ A,B int `raw\r\ntag` }\r\nfunc F(x int)int { return x + X }\r\n".to_vec();
    let source = GoTypeSourceBuffer::new("root", "p.go", "go_file", &raw).unwrap();
    save(
        "owned-input",
        &json!({"object":source.object(),"raw_hex":hex(&raw)}),
    );
    let parsed = parse_go_source(&source).unwrap();
    save("owned-tree", &record(&parsed));
    raw.fill(0);
    assert_eq!(parsed.source(), &source);
    let mut tags = 0;
    for (id, node) in parsed.nodes().iter().enumerate() {
        if node.kind() == "BasicLit" && parsed.spelling(id).unwrap().starts_with(b"`") {
            assert!(parsed.spelling(id).unwrap().contains(&b'\r'));
            assert!(parsed.spelling(id).unwrap().ends_with(b"`"));
            tags += 1
        }
    }
    assert_eq!(tags, 2);
    assert!(parsed.nodes().iter().any(
        |n| n.kind() == "TypeSpec" && n.attributes().any(|(k, v)| k == "Alias" && v == "true")
    ));
    // A typed-looking AST is only syntax. Undefined names and duplicate value
    // bindings still need independent name/type checking and no profile is active.
    let raw = b"package p; var x Missing; var x Other; func F(){ unknown() }";
    let source = GoTypeSourceBuffer::new("root", "untyped.go", "go_file", raw).unwrap();
    save(
        "untyped-input",
        &json!({"object":source.object(),"raw_hex":hex(raw)}),
    );
    let tree = parse_go_source(&source).unwrap();
    save("untyped-tree", &record(&tree));
}

#[test]
fn deep_syntax_uses_the_pinned_parser_domain(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/go-parse/depth-native.json");let expected:Value=serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
 for depth in [0usize,1000,20000,100000]{
  let raw=format!("package p;var X={}true","!".repeat(depth)).into_bytes();let stem=format!("depth-{depth:06}");let source=GoTypeSourceBuffer::new("parse-depth",&format!("{stem}.go"),"go_file",&raw).unwrap();save(&format!("{stem}-input"),&json!({"object":source.object(),"raw_hex":hex(&raw)}));
  let parsed=parse_go_source(&source);let actual=json!({"accepted":parsed.is_ok(),"depth":depth,"source_id":source.object().id,"source_typing_or_program_authority":false});save(&format!("{stem}-result"),&actual);assert_eq!(actual["accepted"],expected[depth.to_string()]["accepted"],"{stem}");assert_eq!(actual["source_id"],expected[depth.to_string()]["source_id"]);
  if let Ok(tree)=parsed{assert!(tree.nodes().iter().filter(|n|n.kind()=="UnaryExpr").count()==depth)}
 }
}
