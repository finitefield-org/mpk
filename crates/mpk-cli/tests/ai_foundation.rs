use std::process::{Command, Output};

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

#[cfg(not(feature = "vertex-ai"))]
#[test]
fn explain_requires_the_opt_in_feature() {
    let output = run_mpk(&["explain"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "mpk explain requires a build with --features vertex-ai\n"
    );
}

#[cfg(feature = "vertex-ai")]
#[test]
fn feature_enabled_explain_route_is_a_non_network_placeholder() {
    let output = run_mpk(&["explain"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        stderr(&output),
        "mpk explain is reserved for the Vertex AI foundation and is not implemented yet\n"
    );
}

#[test]
fn top_level_help_reserves_explain_without_credential_flags() {
    let output = run_mpk(&["--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("mpk explain <evidence.json>"));
    assert!(help.contains("not available until the implementation tasks are complete"));
    assert!(!help.contains("--api-key"));
    assert!(!help.contains("--access-token"));
    assert!(!help.contains("--credentials"));
}

#[cfg(feature = "vertex-ai")]
#[test]
fn feature_help_is_explicitly_not_an_implemented_command() {
    let output = run_mpk(&["explain", "--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("not available until the implementation tasks are complete"));
}
