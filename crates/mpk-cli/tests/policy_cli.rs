use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpk_cli::successor_cli::{POLICY_SCAN_USAGE, POLICY_VERIFY_USAGE};
use serde_json::Value;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_mpk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("mpk command runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

#[test]
fn successor_policy_help_returns_success() {
    for (subcommand, usage) in [("scan", POLICY_SCAN_USAGE), ("verify", POLICY_VERIFY_USAGE)] {
        let output = run_mpk(&["policy", subcommand, "--help"]);
        assert!(output.status.success(), "stderr: {}", stderr(&output));
        assert_eq!(stdout(&output), format!("{usage}\n"));
        assert!(output.stderr.is_empty());
        for removed in [
            "--language",
            "--semantic-profile",
            "--require-release-registry-id",
            "--frontend-bundle",
            "--toolchain-bundle",
        ] {
            assert!(!usage.contains(removed), "successor help exposed {removed}");
        }
    }
}

#[test]
fn successor_policy_requires_context_and_selection() {
    for subcommand in ["scan", "verify"] {
        let output = run_mpk(&["policy", subcommand, "examples/order_policy"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error = stderr(&output);
        assert!(error.contains("--semantic-context is required"), "{error}");
    }
}

#[test]
fn predecessor_identity_and_bundle_flags_are_not_routes() {
    for option in [
        "--language",
        "--semantic-profile",
        "--require-release-registry-id",
        "--require-release-registry-sha256",
        "--frontend-bundle",
        "--toolchain-bundle",
        "--registry-path",
    ] {
        let output = run_mpk(&[
            "policy",
            "scan",
            "examples/order_policy",
            option,
            "predecessor-value",
        ]);
        assert_eq!(output.status.code(), Some(2), "{option} was accepted");
        assert!(output.stdout.is_empty());
        assert!(stderr(&output).contains("unknown flag"), "{option}");
    }
}

#[test]
fn predecessor_semantic_context_rejects_before_source_capture_or_output() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let candidate: Value = serde_json::from_slice(
        &fs::read(repository_root().join("release/bundles/candidates/go.json"))
            .expect("Go candidate"),
    )
    .expect("candidate JSON");
    let mut context = candidate["tuples"][0]["semantic_context"].clone();
    context["profile_registry"]["revision"] = Value::from(1);
    context["profile_registry"]["registry_sha256"] = Value::String(
        "7c9163571cda32aa47984e3e6d949c8857bf62f00110dd1b2c3958eed5e537cc".to_owned(),
    );
    let selection = candidate["tuples"][0]["selection"].clone();
    let context_path = temporary.path().join("context.json");
    let selection_path = temporary.path().join("selection.json");
    let output_path = temporary.path().join("scan.json");
    fs::write(&context_path, serde_json::to_vec(&context).unwrap()).unwrap();
    fs::write(&selection_path, serde_json::to_vec(&selection).unwrap()).unwrap();

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
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("semantic request is invalid"));
    assert!(!output_path.exists());
}
