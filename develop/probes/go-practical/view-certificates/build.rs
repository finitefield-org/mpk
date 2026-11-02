use std::{env, fs, path::Path};
fn once(text: String, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "view predecessor anchor");
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
            let builder=root.join("develop/probes/go-practical/support/builder.rs");
            text=once(text,"\"../../support/builder.rs\"",&format!("{:?}",builder));
            text=once(text,"Expr::Constant(name) => b.constant(name).unwrap(),","Expr::Constant(name) => if name==\"__Sort0\" {b.sort} else {b.constant(name).unwrap()},");
            text=once(text,"let args = std::env::args().collect::<Vec<_>>();","let args = std::env::args().collect::<Vec<_>>();\n    if args.get(1).map(String::as_str)==Some(\"view-generate\") { assert_eq!(args.len(),4); word::generate_views(Path::new(&args[2]),Path::new(&args[3])); return; }");
        } else if rel == Path::new("word.rs") {
            text.push_str("\nmod views;\npub(super) fn generate_views(root:&Path,output:&Path) { views::generate(root,output); }\n");
        }
        fs::write(output.join(rel), text).unwrap();
    }
}
fn main() {
    let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let root = manifest.ancestors().nth(4).unwrap();
    let source = manifest.parent().unwrap().join("certificates/src");
    let output = env::var_os("OUT_DIR").unwrap();
    let output = Path::new(&output).join("predecessor");
    if output.exists() {
        fs::remove_dir_all(&output).unwrap();
    }
    mirror(&source, &output, Path::new(""), root);
    let input = manifest.join("src/views.rs");
    println!("cargo:rerun-if-changed={}", input.display());
    fs::copy(input, output.join("word/views.rs")).unwrap();
}
