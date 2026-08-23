use mpk_cli::policy_scan::v1::USAGE;
use std::path::PathBuf;
use std::process::{Command, Output};

const REGISTRY_SHA256: &str = "226baa5e744f2966615a5fe03d6bfa0395db4b191e92bc099e63436fa9936aba";

#[test]
fn released_scan_help_is_generic_and_locator_free() {
    let output = run_mpk(&["policy", "scan", "--help"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{USAGE}\n")
    );
    assert!(USAGE.contains("--language <go|rust>"));
    for locator in [
        "--frontend ",
        "--frontend-helper",
        "--driver",
        "--toolchain-root",
        "--toolchain-path",
        "--registry-path",
    ] {
        assert!(!USAGE.contains(locator));
    }
}

#[test]
fn released_scan_reports_prelaunch_configuration_errors_as_exit_two() {
    let mut crossed = valid_rust_args();
    replace_option(&mut crossed, "--target", "aarch64-unknown-linux-gnu");
    let output = run_mpk_owned(&crossed);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("POLICY_PROFILE_TUPLE"));

    for (option, value) in [
        (
            "--require-release-registry-id",
            "mpk.release.registry.future",
        ),
        (
            "--require-release-registry-sha256",
            "0000000000000000000000000000000000000000000000000000000000000000",
        ),
    ] {
        let mut assertion = valid_rust_args();
        replace_option(&mut assertion, option, value);
        let output = run_mpk_owned(&assertion);
        assert_eq!(output.status.code(), Some(2), "{option}");
        assert!(output.stdout.is_empty(), "{option}");
        assert!(
            stderr(&output).contains("FRONTEND_REGISTRY_ASSERTION"),
            "{option}: {}",
            stderr(&output)
        );
        assert!(!stderr(&output).contains("POLICY_CLI_INPUT"), "{option}");
    }
}

#[test]
fn released_scan_forbids_every_raw_locator_before_required_options() {
    for locator in [
        "--frontend",
        "--frontend-helper",
        "--driver",
        "--removed-frontend",
        "--toolchain-root",
        "--toolchain-path",
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
        assert!(
            stderr(&output).contains("POLICY_CLI_FORBIDDEN_LOCATOR"),
            "{locator}: {}",
            stderr(&output)
        );
    }
}

fn valid_rust_args() -> Vec<String> {
    vec![
        "policy",
        "scan",
        "does-not-exist",
        "--language",
        "rust",
        "--semantic-profile",
        "mpk.rust.checked.v0",
        "--require-release-registry-id",
        "mpk.release.registry.v0",
        "--require-release-registry-sha256",
        REGISTRY_SHA256,
        "--frontend-bundle",
        "frontend.rust.rust2vir.candidate.v0",
        "--toolchain-bundle",
        "toolchain.rust.nightly-2025-06-01.candidate.v0",
        "--target",
        "x86_64-unknown-linux-gnu",
        "--package",
        "vector",
        "--function",
        "vector::identity",
        "--contract",
        "contracts/vector.json",
        "--json-out",
        "out/scan.json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn replace_option(argv: &mut [String], option: &str, value: &str) {
    let position = argv.iter().position(|argument| argument == option).unwrap();
    argv[position + 1] = value.to_owned();
}

fn run_mpk(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args(arguments)
        .output()
        .unwrap()
}

fn run_mpk_owned(arguments: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args(arguments)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
