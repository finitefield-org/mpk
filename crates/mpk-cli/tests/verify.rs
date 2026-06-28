use std::path::Path;
use std::process::Command;

#[test]
fn cli_verifies_minimal_certificate_fixture() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cert-basic/one-theorem.hex");
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("verify")
        .arg(fixture)
        .output()
        .expect("mpk command runs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("ok module=Example.Basic.OneTheorem"));
}
