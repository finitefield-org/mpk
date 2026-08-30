use std::process::{Command, Output};

use mpk_cli::successor_cli::EXPLAIN_USAGE;

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
fn explain_is_the_offline_successor_route_in_every_build() {
    let output = run_mpk(&["explain", "--help"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), format!("{EXPLAIN_USAGE}\n"));
    assert!(output.stderr.is_empty());
    assert!(EXPLAIN_USAGE.contains("--semantic-context"));
    assert!(EXPLAIN_USAGE.contains("--selection"));
    assert!(EXPLAIN_USAGE.contains("--request-json-out"));
    for removed in ["--provider", "--project", "--location", "--credentials"] {
        assert!(!EXPLAIN_USAGE.contains(removed));
    }
}

#[test]
fn explain_requires_a_successor_source_request() {
    let output = run_mpk(&["explain"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("source-root positional is missing"));
}

#[test]
fn top_level_help_exposes_no_provider_or_predecessor_route() {
    let output = run_mpk(&["--help"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("mpk explain <source-root>"));
    assert!(help.contains("--semantic-context <context.json>"));
    for removed in [
        "--api-key",
        "--access-token",
        "--credentials",
        "--provider",
        "mpk explain <evidence.json>",
    ] {
        assert!(!help.contains(removed), "help exposed {removed}");
    }
}
