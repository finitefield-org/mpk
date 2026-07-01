use std::process::{Command, Output};

const POLICY_SCAN_USAGE: &str = "mpk policy scan <target> --function <function-id> --contract <contract.json> --json-out <scan.json> [--go2gir <go2gir>]";
const POLICY_VERIFY_USAGE: &str = "mpk policy verify <target> --function <function-id> --contract <contract.json> --strategy-profile <profile> --checker-profile <checker-profile> --evidence-json <evidence.json> --evidence-md <evidence.md>";

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
fn policy_scan_help_returns_success() {
    let output = run_mpk(&["policy", "scan", "--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), format!("{POLICY_SCAN_USAGE}\n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn policy_verify_help_returns_success() {
    let output = run_mpk(&["policy", "verify", "--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), format!("{POLICY_VERIFY_USAGE}\n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn policy_scan_missing_required_flags_returns_usage_error() {
    let output = run_mpk(&["policy", "scan", "examples/order_policy"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan missing required flags: --function, --contract, --json-out\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_verify_missing_required_flags_returns_usage_error() {
    let output = run_mpk(&["policy", "verify", "examples/order_policy"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy verify missing required flags: --function, --contract, --strategy-profile, --checker-profile, --evidence-json, --evidence-md\n{POLICY_VERIFY_USAGE}\n")
    );
}

#[test]
fn policy_scan_rejects_strategy_profile_flag() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan.json",
        "--strategy-profile",
        "payment-policy-alpha",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan does not accept --strategy-profile; use mpk policy verify\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_verify_rejects_unknown_checker_profile() {
    let output = run_mpk(&[
        "policy",
        "verify",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "unchecked",
        "--evidence-json",
        "/tmp/mpk-evidence.json",
        "--evidence-md",
        "/tmp/mpk-evidence.md",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy verify has unknown checker profile: \"unchecked\"; expected one of: core-bootstrap, mvp-structural, mvp-strict\n{POLICY_VERIFY_USAGE}\n")
    );
}

#[test]
fn policy_verify_rejects_unknown_strategy_profile() {
    let output = run_mpk(&[
        "policy",
        "verify",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--strategy-profile",
        "payment-policy-basic",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        "/tmp/mpk-evidence.json",
        "--evidence-md",
        "/tmp/mpk-evidence.md",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy verify has unknown strategy profile: \"payment-policy-basic\"; expected one of: payment-policy-alpha\n{POLICY_VERIFY_USAGE}\n")
    );
}

#[test]
fn policy_scan_rejects_duplicate_flag() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--function",
        "example.com/orderpolicy.Other",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan.json",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan has duplicate flag: --function\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_scan_rejects_unknown_flag() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan.json",
        "--evidence-json",
        "/tmp/mpk-evidence.json",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan has unknown flag: --evidence-json\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_scan_rejects_empty_flag_value() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan.json",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan flag --function must not be empty\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_scan_rejects_missing_flag_value() {
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        format!("policy scan flag --json-out requires a value\n{POLICY_SCAN_USAGE}\n")
    );
}

#[test]
fn policy_verify_route_reports_not_implemented_after_valid_routing() {
    let output = run_mpk(&[
        "policy",
        "verify",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--strategy-profile",
        "payment-policy-alpha",
        "--checker-profile",
        "mvp-strict",
        "--evidence-json",
        "/tmp/mpk-evidence.json",
        "--evidence-md",
        "/tmp/mpk-evidence.md",
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "policy verify not implemented until POE-10\n"
    );
}

#[test]
fn policy_scan_reports_missing_go2gir_after_valid_routing() {
    let missing_go2gir = std::env::temp_dir()
        .join(format!("mpk-missing-go2gir-{}", std::process::id()))
        .display()
        .to_string();
    let output = run_mpk(&[
        "policy",
        "scan",
        "examples/order_policy",
        "--function",
        "example.com/orderpolicy.ApprovedReserveCents",
        "--contract",
        "examples/order_policy/policy_contract.json",
        "--json-out",
        "/tmp/mpk-policy-scan.json",
        "--go2gir",
        &missing_go2gir,
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("policy scan failed: go2gir binary not found"));
}
