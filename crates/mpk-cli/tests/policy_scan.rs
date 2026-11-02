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

#[test]
fn removed_and_unknown_languages_reject_before_source_capture() {
    let temporary = tempfile::tempdir().unwrap();
    let candidate = load("release/bundles/candidates/go.json");
    let selection = serde_json::json!({
        "schema":"mpk.selection.go_function.v0",
        "value":{"package":"example.invalid/policy","function":"example.invalid/policy.Identity"}
    });
    let selection_path = temporary.path().join("selection.json");
    fs::write(&selection_path, serde_json::to_vec(&selection).unwrap()).unwrap();
    for language in ["java", "csharp", "python", "typescript"] {
        let mut context = candidate["tuples"][0]["semantic_context"].clone();
        context["source_language"] = serde_json::json!(language);
        context["semantic_profile"] = serde_json::json!(format!("mpk.{language}.scalar.v0"));
        let context_path = temporary.path().join("context.json");
        let output_path = temporary.path().join("scan.json");
        fs::write(&context_path, serde_json::to_vec(&context).unwrap()).unwrap();
        let output = run_mpk(&[
            "policy",
            "scan",
            "missing-source-root",
            "--semantic-context",
            context_path.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
            "--contract",
            "contract.json",
            "--json-out",
            output_path.to_str().unwrap(),
        ]);
        assert_eq!(output.status.code(), Some(1), "{language}");
        assert!(
            stderr(&output).contains("semantic request is invalid"),
            "{language}"
        );
        assert!(output.stdout.is_empty());
        assert!(!output_path.exists());
    }
}
