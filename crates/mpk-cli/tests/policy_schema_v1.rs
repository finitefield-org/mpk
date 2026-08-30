//! Frozen predecessor-vector owner retained as a cutover rejection sentinel.
//!
//! The v1 documents remain immutable specification history. Production no
//! longer exposes their parser, so this owner now proves that callers cannot
//! select the predecessor schemas or identities through the installed CLI.

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
fn predecessor_policy_schema_and_identity_flags_are_not_public_routes() {
    for flag in [
        "--schema",
        "--language",
        "--semantic-profile",
        "--require-release-registry-id",
        "--require-release-registry-sha256",
        "--frontend-bundle",
        "--toolchain-bundle",
    ] {
        let output = run_mpk(&[
            "policy",
            "scan",
            "examples/order_policy",
            flag,
            "predecessor-v1",
        ]);
        assert_eq!(output.status.code(), Some(2), "{flag} was accepted");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(stderr.contains("unknown flag"), "{flag}: {stderr}");
    }
}

#[test]
fn successor_context_and_selection_are_the_only_public_policy_inputs() {
    let output = run_mpk(&["policy", "scan", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("--semantic-context <context.json>"));
    assert!(stdout.contains("--selection <selection.json>"));
    assert!(!stdout.contains("mpk.policy.scan.v1"));
    assert!(!stdout.contains("mpk.policy.evidence.v1"));
}
