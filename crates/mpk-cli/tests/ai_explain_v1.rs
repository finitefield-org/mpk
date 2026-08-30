//! Frozen predecessor-vector owner retained as a cutover rejection sentinel.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

#[test]
fn predecessor_explanation_provider_and_evidence_routes_are_absent() {
    for flag in [
        "--provider",
        "--dry-run",
        "--output-json",
        "--output-md",
        "--project",
        "--location",
    ] {
        let output = run_mpk(&["explain", "examples/order_policy", flag, "predecessor-v1"]);
        assert_eq!(output.status.code(), Some(2), "{flag} was accepted");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(stderr.contains("unknown flag"), "{flag}: {stderr}");
    }
}

#[test]
fn successor_explanation_only_emits_a_sanitized_request() {
    let output = run_mpk(&["explain", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("--semantic-context <context.json>"));
    assert!(stdout.contains("--selection <selection.json>"));
    assert!(stdout.contains("--request-json-out <sanitized-request.json>"));
    assert!(!stdout.contains("--provider"));
    assert!(!stdout.contains("mpk.ai.explanation.v1"));
}
