use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use mpk_vc::{emit_theorem_obligations, generate_branch_vcs, import_gir_json};

#[test]
fn max64_example_gir_generates_documented_vc_outputs() {
    let example_dir = max64_example_dir();
    let gir_json = fs::read_to_string(example_dir.join("gir.json")).expect("read Max64 GIR");
    let gir = import_gir_json(&gir_json).expect("import Max64 GIR");

    let vc_module = generate_branch_vcs(&gir).expect("generate Max64 branch VCs");
    let skeleton =
        emit_theorem_obligations(&vc_module).expect("emit Max64 theorem-obligation skeletons");

    assert_eq!(vc_module.source_gir_hash, gir.gir_hash);
    assert_eq!(vc_module.obligations.len(), 6);
    assert_eq!(skeleton.theorem_declarations.len(), 6);
    assert_eq!(
        skeleton.theorem_declarations[0].name,
        "VC.Obligation.example.Max64.then.post0"
    );
    assert_eq!(
        skeleton.theorem_declarations[5].name,
        "VC.Obligation.example.Max64.else.post2"
    );

    assert_fixture(
        &example_dir.join("vc.json"),
        &pretty_json(&vc_module),
        "MPK_UPDATE_MAX64_EXAMPLE",
    );
    assert_fixture(
        &example_dir.join("vc_skeleton.json"),
        &pretty_json(&skeleton),
        "MPK_UPDATE_MAX64_EXAMPLE",
    );
}

fn max64_example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/max64")
        .components()
        .collect()
}

fn pretty_json(value: &impl Serialize) -> String {
    let mut output = serde_json::to_string_pretty(value).expect("serialize fixture JSON");
    output.push('\n');
    output
}

fn assert_fixture(path: &PathBuf, actual: &str, update_env: &str) {
    if env::var_os(update_env).is_some() {
        fs::write(path, actual).expect("write updated Max64 fixture");
        return;
    }

    let expected = fs::read_to_string(path).expect("read Max64 fixture");
    assert_eq!(actual, expected, "fixture mismatch for {}", path.display());
}
