use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpk_cli::successor_cli::POLICY_SCAN_USAGE;
use serde_json::Value;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_mpk(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repository_root())
        .args(arguments)
        .output()
        .expect("mpk command runs")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(repository_root().join(relative)).expect("fixture"))
        .expect("fixture JSON")
}

#[test]
fn successor_scan_help_has_no_caller_selected_identity_or_locator() {
    let output = run_mpk(&["policy", "scan", "--help"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{POLICY_SCAN_USAGE}\n")
    );
    for removed in [
        "--language",
        "--semantic-profile",
        "--frontend-bundle",
        "--toolchain-bundle",
        "--registry-path",
        "--driver",
    ] {
        assert!(!POLICY_SCAN_USAGE.contains(removed));
    }
}

#[test]
fn successor_scan_rejects_raw_locators_as_unknown_arguments() {
    for locator in [
        "--frontend",
        "--frontend-helper",
        "--driver",
        "--toolchain-root",
        "--registry",
        "--registry-path",
        "--release-registry-path",
    ] {
        let output = run_mpk(&[
            "policy",
            "scan",
            "source",
            locator,
            "/tmp/untrusted-locator",
        ]);
        assert_eq!(output.status.code(), Some(2), "{locator}");
        assert!(output.stdout.is_empty());
        assert!(stderr(&output).contains("unknown flag"), "{locator}");
    }
}

#[test]
fn csharp_contracts_come_only_from_the_validated_selection() {
    let temporary = tempfile::tempdir().unwrap();
    let candidate = load("release/bundles/candidates/csharp.json");
    let vector = load("develop/specs/vectors/csharp-profile-v0.json");
    let context = &candidate["tuples"][0]["semantic_context"];
    let selection = &vector["case_harness"]["baseline_selection"];
    let context_path = temporary.path().join("context.json");
    let selection_path = temporary.path().join("selection.json");
    let output_path = temporary.path().join("scan.json");
    fs::write(&context_path, serde_json::to_vec(context).unwrap()).unwrap();
    fs::write(&selection_path, serde_json::to_vec(selection).unwrap()).unwrap();

    let output = run_mpk(&[
        "policy",
        "scan",
        "missing-source-root",
        "--semantic-context",
        context_path.to_str().unwrap(),
        "--selection",
        selection_path.to_str().unwrap(),
        "--contract",
        "contracts/approved.json",
        "--json-out",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("selected only by the validated selection"));
    assert!(!output_path.exists());
}

#[test]
fn duplicate_json_members_reject_before_source_capture() {
    let temporary = tempfile::tempdir().unwrap();
    let context_path = temporary.path().join("context.json");
    let selection_path = temporary.path().join("selection.json");
    let output_path = temporary.path().join("scan.json");
    fs::write(
        &context_path,
        br#"{"source_language":"go","source_language":"go"}"#,
    )
    .unwrap();
    fs::write(&selection_path, br#"{"schema":"x","value":{}}"#).unwrap();

    let output = run_mpk(&[
        "policy",
        "scan",
        "missing-source-root",
        "--semantic-context",
        context_path.to_str().unwrap(),
        "--selection",
        selection_path.to_str().unwrap(),
        "--contract",
        "policy_contract.json",
        "--json-out",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("duplicate JSON object name"));
    assert!(!output_path.exists());
}
