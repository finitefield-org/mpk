// Deterministic aggregate extension of the frozen private affix producer.
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
            let builder = root.join("develop/probes/go-practical/support/builder.rs");
            text = replace_once(text, "\"../../support/builder.rs\"", &format!("{:?}", builder));
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
fn predecessor_main() {
    let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let root = manifest.ancestors().nth(4).unwrap();
    let source = manifest.parent().unwrap().join("certificates/src");
    let extension = manifest.parent().unwrap().join("affix-certificates/src");
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

fn rewrite(path: &Path, old: &str, new: &str) {
    let text = fs::read_to_string(path).unwrap();
    fs::write(path, replace_once(text, old, new)).unwrap();
}
fn main() {
    predecessor_main();
    let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let out = env::var_os("OUT_DIR").unwrap();
    let output = Path::new(&out).join("predecessor");
    let main = output.join("main.rs");
    let text = fs::read_to_string(&main)
        .unwrap()
        .replace("affix-generate", "aggregate-generate")
        .replace("affix-trace", "aggregate-trace");
    let text = replace_once(text, "assert_eq!(args.len(), 5, \"aggregate-generate REPO_ROOT OUTPUT CAPACITY\");", "assert_eq!(args.len(), 6, \"aggregate-generate REPO_ROOT OUTPUT BYTE_BOUND STORAGE_BOUND\");");
    let text = replace_once(text, "word::generate_stringops(\n            Path::new(&args[2]),\n            Path::new(&args[3]),\n            args[4].parse().unwrap(),", "word::generate_stringops(\n            Path::new(&args[2]),\n            Path::new(&args[3]),\n            args[4].parse::<usize>().unwrap().max(args[5].parse().unwrap()),");
    fs::write(main, text).unwrap();
    rewrite(
        &output.join(ACCESS),
        "assert!([0, 1, 31, 32, 33, 255, 256, 257].contains(&count));",
        "assert!(count <= 257);",
    );
    rewrite(
        &output.join(ACCESS),
        "{PREFIX}.StringAffixCapacity{count}",
        "{PREFIX}.AggregateCapacity{count}",
    );
    let validity = output.join(ACCESS).with_extension("").join("validity.rs");
    let mut text = fs::read_to_string(&validity).unwrap();
    text.push_str("\npub(super) fn aggregate_install(p: &mut Producer, count: usize, trace: Option<&Path>) { install_core(p); embedding::install(p, count, trace); }\n");
    fs::write(validity, text).unwrap();
    let access = output.join(ACCESS);
    let mut text = fs::read_to_string(&access).unwrap();
    text.push_str("\nfn aggregate_validity(p: &mut Producer, count: usize, trace: Option<&Path>) { validity::aggregate_install(p, count, trace); }\n");
    fs::write(access, text).unwrap();
    let affix = output.join(STRINGOPS).with_extension("").join("affix.rs");
    rewrite(
        &affix,
        "mod emit;",
        "#[allow(dead_code)]\nmod emit;\nmod aggregate;",
    );
    rewrite(
        &affix,
        "emit::generate(base, output, count);",
        "aggregate::generate(base, output, count);",
    );
    let emit = affix.with_extension("").join("emit.rs");
    rewrite(
        &emit,
        "fn proof_call(suf: bool, d: usize, f: &str) -> Typed",
        "pub(super) fn proof_call(suf: bool, d: usize, f: &str) -> Typed",
    );
    let input = manifest.join("src/aggregate.rs");
    println!("cargo:rerun-if-changed={}", input.display());
    fs::copy(input, affix.with_extension("").join("aggregate.rs")).unwrap();
}
