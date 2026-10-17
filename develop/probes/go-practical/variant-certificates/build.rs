use std::{env, fs, path::Path};
fn once(text: String, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "variant predecessor anchor");
    text.replacen(old, new, 1)
}
fn mirror(source: &Path, output: &Path, relative: &Path, root: &Path) {
    fs::create_dir_all(output.join(relative)).unwrap();
    let mut paths = fs::read_dir(source.join(relative))
        .unwrap()
        .map(|p| p.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    for input in paths {
        let rel = relative.join(input.file_name().unwrap());
        if input.is_dir() {
            mirror(source, output, &rel, root);
            continue;
        }
        assert_eq!(input.extension().and_then(|s| s.to_str()), Some("rs"));
        println!("cargo:rerun-if-changed={}", input.display());
        let mut text = fs::read_to_string(input).unwrap();
        if rel == Path::new("main.rs") {
            text = text
                .lines()
                .map(|s| {
                    format!(
                        "{}\n",
                        s.strip_prefix("//!")
                            .map(|s| format!("//{s}"))
                            .unwrap_or(s.into())
                    )
                })
                .collect();
            let builder=root.join("develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs");
            text=once(text,"\"../../../../migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs\"",&format!("{:?}",builder));
            text=once(text,"Expr::Constant(name) => b.constant(name).unwrap(),","Expr::Constant(name) => if name==\"__Sort0\" {b.sort} else {b.constant(name).unwrap()},");
            text=once(text,"let args = std::env::args().collect::<Vec<_>>();","let args = std::env::args().collect::<Vec<_>>();\n    if args.get(1).map(String::as_str)==Some(\"variant-generate\") { assert_eq!(args.len(),4); word::generate_variant(Path::new(&args[2]),Path::new(&args[3])); return; }");
        } else if rel == Path::new("word.rs") {
            text.push_str("\n#[allow(dead_code)]\nmod termination;\npub(super) fn generate_variant(root:&Path,output:&Path) { termination::generate_variant(root,output); }\n");
        }
        fs::write(output.join(rel), text).unwrap();
    }
}
fn main() {
    let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let root = manifest.ancestors().nth(4).unwrap();
    let source = root.join("develop/probes/go-practical/certificates/src");
    let output = env::var_os("OUT_DIR").unwrap();
    let output = Path::new(&output).join("predecessor");
    if output.exists() {
        fs::remove_dir_all(&output).unwrap();
    }
    mirror(&source, &output, Path::new(""), root);
    let input =
        root.join("develop/probes/go-practical/termination-certificates/src/termination.rs");
    println!("cargo:rerun-if-changed={}", input.display());
    let text = fs::read_to_string(input).unwrap();
    let mut text = once(
        text,
        "Probe.GoPractical.NatTermination",
        "Probe.GoPractical.VariantTermination",
    );
    text.push_str("\nmod variant;\npub(super) fn generate_variant(root:&Path,output:&Path) { variant::generate(root,output); }\n");
    fs::write(output.join("word/termination.rs"), text).unwrap();
    let input = manifest.join("src/variant.rs");
    println!("cargo:rerun-if-changed={}", input.display());
    fs::create_dir_all(output.join("word/termination")).unwrap();
    fs::copy(input, output.join("word/termination/variant.rs")).unwrap();
}
