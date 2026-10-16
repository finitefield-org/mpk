//! Deterministic private producer extension; no source admission or kernel rule.
use std::{env, fs, path::Path};

const STRINGOPS: &str =
    "word/balanced/signed/bytes/bounds/slices/access/validity/reslice/stringops.rs";
const ACCESS: &str = "word/balanced/signed/bytes/bounds/slices/access.rs";
const COVERAGE: &str = "word/balanced/signed/bytes/bounds/coverage.rs";

fn replace_once(source: String, old: &str, new: &str) -> String {
    assert_eq!(
        source.matches(old).count(),
        1,
        "predecessor extension anchor"
    );
    source.replacen(old, new, 1)
}
fn mirror(source: &Path, output: &Path, relative: &Path, extension: &Path, root: &Path) {
    fs::create_dir_all(output.join(relative)).unwrap();
    let mut entries = fs::read_dir(source.join(relative))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for input in entries {
        let rel = relative.join(input.file_name().unwrap());
        if input.is_dir() {
            mirror(source, output, &rel, extension, root);
            continue;
        }
        assert_eq!(input.extension().and_then(|s| s.to_str()), Some("rs"));
        println!("cargo:rerun-if-changed={}", input.display());
        let mut text = fs::read_to_string(&input).unwrap();
        if rel == Path::new("main.rs") {
            // Included crate roots cannot contain inner documentation comments.
            text = text
                .lines()
                .map(|line| {
                    if let Some(s) = line.strip_prefix("//!") {
                        format!("//{s}\n")
                    } else {
                        format!("{line}\n")
                    }
                })
                .collect();
            let builder = root.join("develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs");
            text = replace_once(text, "\"../../../../migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs\"", &format!("{:?}", builder));
            text = text
                .replace("stringop-generate", "affix-generate")
                .replace("stringop-trace", "affix-trace");
        } else if rel == Path::new(ACCESS) {
            text = replace_once(
                text,
                "{PREFIX}.StringOperationCapacity{count}",
                "{PREFIX}.StringAffixCapacity{count}",
            );
            text.push_str("\nfn affix_embedding(p: &mut Producer, count: usize) { super::super::coverage::install_affix_embedding(p, 1usize << max_depth(count)); }\n");
        } else if rel == Path::new(COVERAGE) {
            text.push_str("\npub(super) fn install_affix_embedding(p: &mut Producer, capacity: usize) { let n = Node { d: WORD, p: capacity.ilog2() as usize }; let x = v(\"word\"); let proof = embedding_bits(p, n, x.clone()); let (ty, proof) = fold_binders(&[(\"word\".into(), word_ty())], observations(WORD, prefix(n, x.clone()), index_word(&low_bits(x, n.p))), proof); p.declare(&symbol(\"PrefixEmbeddingBits\", n), ty, proof, true); }\n");
        } else if rel == Path::new(STRINGOPS) {
            text = replace_once(text, "mod emit;", "mod affix;");
            text = replace_once(
                text,
                "pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>)",
                "fn install_predecessor(p: &mut Producer, count: usize, trace: Option<&Path>)",
            );
            text = replace_once(
                text,
                "emit::generate(base, output, count);",
                "affix::emit(base, output, count);",
            );
            text.push_str("\npub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) { install_predecessor(p, count, trace); affix::install(p, count, trace); }\n");
        }
        fs::write(output.join(rel), text).unwrap();
    }
    if relative.as_os_str().is_empty() {
        let dir = output.join(Path::new(STRINGOPS).with_extension(""));
        fs::create_dir_all(&dir).unwrap();
        fs::copy(extension.join("affix.rs"), dir.join("affix.rs")).unwrap();
        let child = dir.join("affix");
        fs::create_dir_all(&child).unwrap();
        fs::copy(extension.join("emit.rs"), child.join("emit.rs")).unwrap();
    }
}
fn main() {
    let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let root = manifest.ancestors().nth(4).unwrap();
    let source = manifest.parent().unwrap().join("certificates/src");
    let extension = manifest.join("src");
    let output = env::var_os("OUT_DIR").unwrap();
    let output = Path::new(&output).join("predecessor");
    println!("cargo:rerun-if-changed={}", source.display());
    for name in ["affix.rs", "emit.rs"] {
        println!("cargo:rerun-if-changed={}", extension.join(name).display());
    }
    if output.exists() {
        fs::remove_dir_all(&output).unwrap();
    }
    mirror(&source, &output, Path::new(""), &extension, root);
}
