use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn check_accepts_valid_certificate_fixture() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cert-basic/one-theorem.hex");
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("check")
        .arg(fixture)
        .output()
        .expect("mpk command runs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("\"verdict\":\"accepted\""));
    assert!(stdout.contains("\"module\":\"Example.Basic.OneTheorem\""));
    assert!(stdout.contains(
        "\"certificate\":\"37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94\""
    ));
}

#[test]
fn check_rejects_invalid_certificate_fixture_with_json() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/cert-canonical/non-canonical/unsorted-name-table.hex");
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("check")
        .arg(fixture)
        .output()
        .expect("mpk command runs");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("\"verdict\":\"rejected\""));
    assert!(stdout.contains("\"error_code\":\"KERNEL_CANONICAL_CERTIFICATE\""));
    assert!(stdout.contains("\"error_detail\":\"name_table: Z before A\""));
}

#[test]
fn axiom_report_prints_recomputed_report_for_valid_certificate_fixture() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cert-basic/one-theorem.hex");
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("axiom-report")
        .arg(fixture)
        .output()
        .expect("mpk command runs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(
        stdout,
        concat!(
            "{\"certificate_hash\":\"37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94\",",
            "\"axiom_report_hash\":\"0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5\",",
            "\"axiom_report\":{\"summary\":{\"core_axiom_count\":0,",
            "\"builtin_theory_axiom_count\":0,\"go_semantics_axiom_count\":0,",
            "\"external_axiom_count\":0,\"total_axiom_count\":0},",
            "\"entries\":[],\"declaration_dependencies\":[]}}\n"
        )
    );
}

#[test]
fn axiom_report_rejects_invalid_certificate_fixture_with_json() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/cert-canonical/non-canonical/unsorted-name-table.hex");
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .arg("axiom-report")
        .arg(fixture)
        .output()
        .expect("mpk command runs");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("\"verdict\":\"rejected\""));
    assert!(stdout.contains("\"error_code\":\"KERNEL_CANONICAL_CERTIFICATE\""));
    assert!(stdout.contains("\"error_detail\":\"name_table: Z before A\""));
}

#[test]
fn legacy_verify_still_accepts_valid_certificate_fixture() {
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
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("ok module=Example.Basic.OneTheorem"));
}

#[test]
fn package_check_accepts_valid_manifest_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args([
            "package",
            "check",
            "fixtures/package-manifest/valid/basic-package.json",
        ])
        .output()
        .expect("mpk command runs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(
        stdout,
        "ok package=Example.Basic.Package imports=1 certificates=1\n"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn package_verify_certs_uses_the_embedded_reference_checker() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .env("PATH", "/nonexistent")
        .args([
            "package",
            "verify-certs",
            "fixtures/package-manifest/valid/basic-package.json",
        ])
        .output()
        .expect("mpk command runs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "ok package=Example.Basic.Package source_free=1 reference=1\n"
    );
}

#[test]
fn package_check_rejects_invalid_manifest_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args([
            "package",
            "check",
            "fixtures/package-manifest/invalid/missing-certificate-hash.json",
        ])
        .output()
        .expect("mpk command runs");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("package check failed"));
    assert!(stderr.contains("expected_certificate_hash"));
}

#[test]
fn package_verify_certs_rejects_invalid_manifest_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args([
            "package",
            "verify-certs",
            "fixtures/package-manifest/invalid/missing-certificate-hash.json",
        ])
        .output()
        .expect("mpk command runs");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("package verify-certs failed"));
    assert!(stderr.contains("expected_certificate_hash"));
}

#[test]
fn package_check_rejects_duplicate_import_manifest_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_mpk"))
        .current_dir(repo_root())
        .args([
            "package",
            "check",
            "fixtures/package-manifest/invalid/duplicate-import.json",
        ])
        .output()
        .expect("mpk command runs");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("package check failed"));
    assert!(stderr.contains("duplicates import"));
}
