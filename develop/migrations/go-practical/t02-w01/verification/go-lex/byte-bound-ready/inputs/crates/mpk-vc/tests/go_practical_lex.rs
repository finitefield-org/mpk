use mpk_vc::go_practical_type_keys::{lex_go_source, GoLexErrorKind, GoTypeSourceBuffer};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
fn unhex(value: &str) -> Vec<u8> {
    assert!(value.len().is_multiple_of(2));
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn hex(value: &[u8]) -> String {
    value.iter().map(|b| format!("{b:02x}")).collect()
}
fn save(root: &Path, name: &str, value: &Value) {
    use std::io::Write;
    std::fs::create_dir_all(root).unwrap();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(format!("{name}.json")))
        .unwrap()
        .write_all(&serde_json::to_vec(value).unwrap())
        .unwrap();
}
fn compare(
    name: &str,
    source: GoTypeSourceBuffer,
    native: &Value,
    output: Option<&Path>,
) -> (bool, usize) {
    let result = lex_go_source(&source);
    let actual = match &result {
        Ok(lexed) => {
            let mut previous = 0;
            for (n, token) in lexed.tokens().iter().enumerate() {
                assert!(
                    previous <= token.start()
                        && token.start() <= token.end()
                        && token.end() <= source.bytes().len(),
                    "{name}"
                );
                previous = token.start();
                assert_eq!(
                    lexed.spelling(n),
                    Some(&source.bytes()[token.start()..token.end()])
                );
                if token.inserted() {
                    assert_eq!(token.start(), token.end());
                    assert_eq!(token.kind(), ";");
                    assert_eq!(token.literal(), b"\n")
                }
            }
            assert_eq!(lexed.source(), &source);
            let tokens: Vec<_> = lexed
                .tokens()
                .iter()
                .map(|t| json!([t.start(), t.kind(), hex(t.literal())]))
                .collect();
            json!({"accepted":true,"tokens":tokens,"source_id":source.object().id,"source_typing_or_program_authority":false})
        }
        Err(error) => {
            json!({"accepted":false,"error_offset":error.offset,"error_kind":format!("{:?}",error.kind),"source_id":source.object().id,"source_typing_or_program_authority":false})
        }
    };
    if let Some(root) = output {
        save(root, name, &actual)
    }
    assert_eq!(
        actual["accepted"],
        native["accepted"],
        "{name}: {actual}; native first error {:?}",
        native["errors"].as_array().and_then(|e| e.first())
    );
    if actual["accepted"] == true {
        if actual["tokens"] != native["tokens"] {
            let a = actual["tokens"].as_array().unwrap();
            let n = native["tokens"].as_array().unwrap();
            let at = a
                .iter()
                .zip(n)
                .position(|(a, b)| a != b)
                .unwrap_or(a.len().min(n.len()));
            panic!(
                "{name}: first token mismatch {at}: Rust {:?}; Go {:?}; token counts {} vs {}",
                a.get(at),
                n.get(at),
                a.len(),
                n.len()
            );
        }
        let count = actual["tokens"].as_array().unwrap().len();
        (true, count)
    } else {
        assert!(result.unwrap_err().offset <= source.bytes().len());
        (false, 0)
    }
}
#[test]
fn exact_native_go_token_streams_and_rejections() {
    let output = std::env::var_os("MPK_GO_LEX_RUST_OUTPUT").map(PathBuf::from);
    let mut passed = 0;
    let mut rejected = 0;
    let mut tokens = 0;
    if let Some(root) = std::env::var_os("MPK_GO_LEX_CORPUS") {
        let mut cases: Vec<_> = std::fs::read_dir(root)
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
        cases.sort();
        assert!(cases.len() > 5000);
        for case in cases {
            let raw = std::fs::read(case.join("source.raw")).unwrap();
            let object = std::fs::read(case.join("source.json")).unwrap();
            let source = GoTypeSourceBuffer::import(&object, &raw).unwrap();
            let native: Value =
                serde_json::from_slice(&std::fs::read(case.join("native.json")).unwrap()).unwrap();
            let (accepted, n) = compare(
                case.file_name().unwrap().to_str().unwrap(),
                source,
                &native,
                output.as_deref(),
            );
            if accepted {
                passed += 1
            } else {
                rejected += 1
            };
            tokens += n;
        }
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/go-practical/t02-w01/verification/go-lex/smoke.json");
        let corpus: Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
        let cases = corpus.as_array().unwrap();
        assert!(cases.len() > 500);
        for (n, case) in cases.iter().enumerate() {
            let raw = unhex(case["raw_hex"].as_str().unwrap());
            let source =
                GoTypeSourceBuffer::import(&serde_json::to_vec(&case["object"]).unwrap(), &raw)
                    .unwrap();
            let (accepted, count) = compare(
                &format!("smoke-{n:05}"),
                source,
                &case["native"],
                output.as_deref(),
            );
            if accepted {
                passed += 1
            } else {
                rejected += 1
            };
            tokens += count;
        }
    }
    println!("actual original lexical cases: accepted={passed}, rejected={rejected}, exact positive tokens={tokens}");
    if let Some(root) = output {
        save(
            &root,
            "summary",
            &json!({"accepted":passed,"rejected":rejected,"exact_positive_tokens":tokens,"applications_executed":false,"source_typing_or_program_authority":false}),
        )
    }
    assert!(passed > 0 && rejected > 0 && tokens > 0);
}
#[test]
fn physical_source_and_inserted_semicolons_are_owned() {
    let mut raw = b"package x\r\nvar a = `one\r\ntwo`/*first\nsecond*/\na++//end".to_vec();
    let source = GoTypeSourceBuffer::new("root", "x.go", "go_file", &raw).unwrap();
    let lexed = lex_go_source(&source).unwrap();
    raw.fill(0);
    assert_eq!(lexed.source(), &source);
    assert_ne!(lexed.source().bytes(), raw);
    let string = lexed
        .tokens()
        .iter()
        .position(|t| t.kind() == "STRING")
        .unwrap();
    assert_eq!(lexed.tokens()[string].literal(), b"`one\ntwo`");
    assert_eq!(lexed.spelling(string), Some(b"`one\r\ntwo`".as_slice()));
    let after = &lexed.tokens()[string + 1];
    assert!(after.inserted());
    assert_eq!(
        after.start(),
        source
            .bytes()
            .windows(6)
            .position(|b| b == b"first\n")
            .unwrap()
            + 5
    );
    assert_eq!(lexed.tokens().last().unwrap().kind(), "EOF");
    assert_eq!(
        lex_go_source(&GoTypeSourceBuffer::new("root", "go.mod", "go_mod", b"module x").unwrap())
            .unwrap_err()
            .kind,
        GoLexErrorKind::Source
    );
    for (bytes, kind) in [
        (b"\xff".as_slice(), GoLexErrorKind::Utf8),
        (b"x\0".as_slice(), GoLexErrorKind::Nul),
        ("//\u{feff}".as_bytes(), GoLexErrorKind::ByteOrderMark),
    ] {
        let source = GoTypeSourceBuffer::new("root", "x.go", "go_file", bytes).unwrap();
        assert_eq!(lex_go_source(&source).unwrap_err().kind, kind);
    }
}

#[test]
fn lexical_token_count_is_not_the_ast_budget() {
    let raw = vec![b'}'; 4_194_305];
    let source =
        GoTypeSourceBuffer::new("lexical-cases", "byte-bound.go", "go_file", &raw).unwrap();
    if let Some(root) = std::env::var_os("MPK_GO_LEX_RUST_OUTPUT") {
        save(
            Path::new(&root),
            "byte-bound-input",
            &json!({"object":source.object(),"raw_hex":hex(&raw)}),
        );
    }
    let lexed = lex_go_source(&source).unwrap();
    let mut hash = Sha256::new();
    for t in lexed.tokens() {
        hash.update(format!("{}/{}/{}\n", t.start(), t.kind(), hex(t.literal())).as_bytes());
    }
    let digest = hex(&hash.finalize());
    assert_eq!(lexed.tokens().len(), raw.len() + 2);
    assert_eq!(lexed.tokens()[raw.len()].literal(), b"\n");
    assert_eq!(lexed.tokens().last().unwrap().kind(), "EOF");
    let result = json!({"accepted":true,"tokens":lexed.tokens().len(),"source_object":source.object(),"complete_token_stream_sha256":digest,"source_parsing_or_typing":false});
    if let Some(root) = std::env::var_os("MPK_GO_LEX_RUST_OUTPUT") {
        save(Path::new(&root), "byte-bound-result", &result)
    }
    if let Some(root) = std::env::var_os("MPK_GO_LEX_BYTE_BOUND") {
        let native: Value = serde_json::from_slice(
            &std::fs::read(Path::new(&root).join("byte-bound-native.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(result, native);
    } else {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/go-lex/byte-bound-native.json");
        let native: Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
        assert_eq!(result, native);
    }
}
