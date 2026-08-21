use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use sha2::{Digest, Sha256};

const BASELINE: &str = "develop/migrations/go-gir-semantic-baseline.json";
const PROFILE_VECTOR: &str = "develop/specs/vectors/go-vir-profile-v0.json";
const REPORT_JSON: &str = "develop/migrations/go-gir-to-vir-report.json";
const REPORT_MARKDOWN: &str = "develop/migrations/go-gir-to-vir-report.md";
const SCRIPT: &str = "scripts/compare-go-gir-vir.py";

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[test]
fn checked_reports_are_exactly_derived_and_development_only() {
    let root = repo_root();
    let json = run_comparator(&root, &["--format", "json"]);
    assert_success(&json);
    assert_eq!(
        json.stdout,
        fs::read(root.join(REPORT_JSON)).expect("checked JSON migration report")
    );

    let report: Value =
        serde_json::from_slice(&json.stdout).expect("generated migration report JSON");
    assert_eq!(report["schema"], "mpk.go_gir_to_vir_report.v0");
    assert_eq!(report["status"], "equivalent_with_reviewed_changes");
    assert_eq!(report["lifecycle"]["development_only"], true);
    assert_eq!(report["lifecycle"]["production_input"], false);
    assert_eq!(report["lifecycle"]["release_artifact"], false);
    assert_eq!(
        report["lifecycle"]["archive_or_delete_owner"],
        "GO-VIR-02-T12"
    );
    assert_eq!(report["summary"]["unexplained_difference_count"], 0);
    assert_eq!(report["findings"], Value::Array(Vec::new()));

    let markdown = run_comparator(&root, &["--format", "markdown"]);
    assert_success(&markdown);
    assert_eq!(
        markdown.stdout,
        fs::read(root.join(REPORT_MARKDOWN)).expect("checked Markdown migration report")
    );

    let checked = run_comparator(&root, &["--check"]);
    assert_success(&checked);
}

#[test]
fn removed_runtime_check_fails_the_harness() {
    let root = repo_root();
    let temporary = TestDirectory::new(&root, "removed-runtime-check");
    let mut baseline = read_json(&root.join(BASELINE));
    baseline["behavioral_anchors"]["runtime_checks"]["checks"]
        .as_array_mut()
        .expect("runtime check array")
        .remove(0);

    let baseline_path = temporary.path.join("baseline.json");
    let baseline_digest = write_json(&baseline_path, &baseline);
    let mut vector = read_json(&root.join(PROFILE_VECTOR));
    vector["migration_baseline"]["integrity"]["sha256"] = Value::String(baseline_digest);
    let vector_path = temporary.path.join("vector.json");
    write_json(&vector_path, &vector);

    let output = run_comparator_paths(&root, &baseline_path, &vector_path);
    assert!(
        !output.status.success(),
        "removed runtime check was accepted"
    );
    assert_stderr_contains(&output, "MIGRATION_RUNTIME_CHECK_SET");
}

#[test]
fn changed_negative_rejection_class_fails_the_harness() {
    let root = repo_root();
    let temporary = TestDirectory::new(&root, "changed-negative-rejection");
    let mut vector = read_json(&root.join(PROFILE_VECTOR));
    vector["migration_baseline"]["corpora"]["go_basic"]["negative"][0]["code"] =
        Value::String("GO_SUBSET_POINTER".into());
    let vector_path = temporary.path.join("vector.json");
    write_json(&vector_path, &vector);

    let output = run_comparator_paths(&root, &root.join(BASELINE), &vector_path);
    assert!(
        !output.status.success(),
        "changed negative rejection class was accepted"
    );
    assert_stderr_contains(&output, "MIGRATION_NEGATIVE_REJECTION");
}

#[test]
fn extra_obligation_kind_fails_the_harness() {
    let root = repo_root();
    let temporary = TestDirectory::new(&root, "extra-obligation-kind");
    let mut vector = read_json(&root.join(PROFILE_VECTOR));
    vector["migration_baseline"]["obligation_kind_map"]
        .as_array_mut()
        .expect("obligation kind mapping")
        .push(serde_json::json!({
            "old": "unreviewed_old_kind",
            "new": "unreviewed_new_kind",
            "intent": "unreviewed"
        }));
    let vector_path = temporary.path.join("vector.json");
    write_json(&vector_path, &vector);

    let output = run_comparator_paths(&root, &root.join(BASELINE), &vector_path);
    assert!(
        !output.status.success(),
        "extra obligation kind was accepted"
    );
    assert_stderr_contains(&output, "MIGRATION_OBLIGATION_KIND");
}

#[test]
fn checker_disagreement_fails_the_harness() {
    let root = repo_root();
    let temporary = TestDirectory::new(&root, "checker-disagreement");
    let mut baseline = read_json(&root.join(BASELINE));
    baseline["checker_baseline"]["release_report"]["certificate"]["reference_verdict"] =
        Value::String("rejected".into());

    let baseline_path = temporary.path.join("baseline.json");
    let baseline_digest = write_json(&baseline_path, &baseline);
    let mut vector = read_json(&root.join(PROFILE_VECTOR));
    vector["migration_baseline"]["integrity"]["sha256"] = Value::String(baseline_digest);
    let vector_path = temporary.path.join("vector.json");
    write_json(&vector_path, &vector);

    let output = run_comparator_paths(&root, &baseline_path, &vector_path);
    assert!(
        !output.status.success(),
        "checker disagreement was accepted"
    );
    assert_stderr_contains(&output, "MIGRATION_CHECKER_DISAGREEMENT");
}

#[test]
fn reviewed_legacy_hash_change_does_not_change_semantic_outcome() {
    let root = repo_root();
    let temporary = TestDirectory::new(&root, "reviewed-hash-change");
    let replacement = "0000000000000000000000000000000000000000000000000000000000000000";

    let mut baseline = read_json(&root.join(BASELINE));
    baseline["corpora"]["go_alpha"]["vc_fixture"]["source_gir_hash"] =
        Value::String(replacement.into());
    let baseline_path = temporary.path.join("baseline.json");
    let baseline_digest = write_json(&baseline_path, &baseline);

    let mut vector = read_json(&root.join(PROFILE_VECTOR));
    vector["migration_baseline"]["integrity"]["sha256"] = Value::String(baseline_digest);
    vector["migration_baseline"]["corpora"]["go_alpha"]["vc_fixture"]["old_source_gir_hash"] =
        Value::String(replacement.into());
    let vector_path = temporary.path.join("vector.json");
    write_json(&vector_path, &vector);

    let output = run_comparator_paths(&root, &baseline_path, &vector_path);
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).expect("migration report");
    assert_eq!(report["status"], "equivalent_with_reviewed_changes");
    assert_eq!(report["summary"]["unexplained_difference_count"], 0);
}

#[test]
fn production_crates_do_not_reference_the_migration_comparator() {
    let root = repo_root();
    let mut candidates = Vec::new();
    collect_production_files(&root.join("crates"), &mut candidates);
    assert!(!candidates.is_empty());

    for path in candidates {
        let text = fs::read_to_string(&path).expect("production source is UTF-8");
        assert!(
            !text.contains("compare-go-gir-vir") && !text.contains("go-gir-to-vir-report"),
            "production file {} depends on the development-only migration harness",
            path.display()
        );
    }
}

fn run_comparator_paths(root: &Path, baseline: &Path, vector: &Path) -> Output {
    Command::new("python3")
        .current_dir(root)
        .arg(SCRIPT)
        .arg("--baseline")
        .arg(baseline)
        .arg("--profile-vector")
        .arg(vector)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run migration comparator")
}

fn run_comparator(root: &Path, args: &[&str]) -> Output {
    Command::new("python3")
        .current_dir(root)
        .arg(SCRIPT)
        .args(args)
        .output()
        .expect("run migration comparator")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "comparator failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stderr_contains(output: &Output, expected: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "stderr did not contain {expected:?}: {stderr}"
    );
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON fixture")).expect("parse JSON fixture")
}

fn write_json(path: &Path, value: &Value) -> String {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize JSON fixture");
    bytes.push(b'\n');
    fs::write(path, &bytes).expect("write JSON fixture");
    format!("{:x}", Sha256::digest(&bytes))
}

fn collect_production_files(directory: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read crates directory") {
        let entry = entry.expect("read crates entry");
        let path = entry.path();
        if path.is_dir() {
            collect_production_files(&path, output);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml")
            || path.extension().and_then(|extension| extension.to_str()) == Some("rs")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "src")
        {
            output.push(path);
        }
    }
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(root: &Path, label: &str) -> Self {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = root
            .join("target/go-migration-tests")
            .join(format!("{label}-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&path).expect("create migration test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
