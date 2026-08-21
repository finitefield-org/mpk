use std::process::{Command, Output};

use mpk_cli::policy_scan::v1::USAGE as POLICY_SCAN_USAGE;
use mpk_cli::policy_verify::v1::USAGE as POLICY_VERIFY_USAGE;

fn run_mpk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
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
fn policy_v1_help_returns_success() {
    for (subcommand, usage) in [("scan", POLICY_SCAN_USAGE), ("verify", POLICY_VERIFY_USAGE)] {
        let output = run_mpk(&["policy", subcommand, "--help"]);
        assert!(output.status.success(), "stderr: {}", stderr(&output));
        assert_eq!(stdout(&output), format!("{usage}\n"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn policy_v1_requires_the_registered_selection_tuple() {
    for subcommand in ["scan", "verify"] {
        let output = run_mpk(&["policy", subcommand, "examples/order_policy"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error = stderr(&output);
        assert!(error.contains("POLICY_CLI_REQUIRED"), "{error}");
    }
}

#[test]
fn policy_v1_rejects_unknown_and_private_locator_options() {
    for option in ["--removed-option", "--frontend", "--registry-path"] {
        let output = run_mpk(&[
            "policy",
            "scan",
            "examples/order_policy",
            option,
            "untrusted-value",
        ]);
        assert_eq!(output.status.code(), Some(2), "{option} was accepted");
        assert!(output.stdout.is_empty());
        let error = stderr(&output);
        assert!(
            error.contains("POLICY_CLI_UNKNOWN_OPTION")
                || error.contains("POLICY_CLI_FORBIDDEN_LOCATOR"),
            "{error}"
        );
    }
}

#[test]
fn policy_v1_rejects_malformed_registry_identity_before_execution() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--language",
        "go",
        "--semantic-profile",
        "mpk.go.fixed.v0",
        "--require-release-registry-id",
        "mpk.release.registry.v0",
        "--require-release-registry-sha256",
        "not-a-hash",
        "--frontend-bundle",
        "frontend.go.synthetic.v0",
        "--toolchain-bundle",
        "toolchain.go.synthetic.v0",
        "--target",
        "linux/amd64",
        "--package",
        "example.com/orderpolicy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan-v1.json",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("POLICY_CLI_SCALAR"));
}
