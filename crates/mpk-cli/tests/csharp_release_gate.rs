use mpk_cert::encode::AxiomCategory;
use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
    ACTIVE_RELEASE_REGISTRY_SHA256,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ARCHIVED_REVIEW_PATH: &str = "develop/migrations/archive/csharp-02-final-review.json";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative)).expect("read active release input")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("JSON object")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("JSON string")
}

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("serialize canonical JSON");
    bytes.push(b'\n');
    bytes
}

#[test]
fn archived_rehearsal_ledger_remains_canonical_complete_and_empty() {
    let bytes = read(ARCHIVED_REVIEW_PATH);
    let review: Value = serde_json::from_slice(&bytes).expect("review ledger JSON");
    assert_eq!(canonical_line(&review), bytes);
    assert_eq!(review["status"], "reviewed_zero_findings");
    assert_eq!(review["task"], "CSHARP-02-T19");
    assert_eq!(review["trust"], "untrusted_review_record");
    assert!(array(&review["findings"]).is_empty());

    let categories = [
        AxiomCategory::CoreAxiom,
        AxiomCategory::BuiltinTheoryAxiom,
        AxiomCategory::GoSemanticsAxiom,
        AxiomCategory::ExternalAxiom,
    ]
    .map(AxiomCategory::canonical_name);
    assert_eq!(
        array(&review["axiom_review"]["categories"])
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        categories
    );
    assert!(array(&review["axiom_review"]["new_categories"]).is_empty());
    assert_eq!(review["rehearsal"]["gate_passes"], 2);
    assert_eq!(review["rehearsal"]["network_access"], false);
    assert_eq!(review["rehearsal"]["provisioning"], false);
}

#[test]
fn every_active_candidate_is_an_exact_projection_of_the_successor_registry() {
    let semantic_bytes = read("release/bundles/semantic-profile-registry.json");
    let semantic = validate_semantic_profile_registry(&semantic_bytes, RegistryRevision::Revision3)
        .expect("active revision-3 semantic registry");
    let semantic_value: Value =
        serde_json::from_slice(&semantic_bytes).expect("semantic registry JSON");
    assert_eq!(canonical_line(&semantic_value), semantic_bytes);

    let registry_bytes = read("release/bundles/bundle-registry.json");
    let registry = validate_successor_release_registry(&registry_bytes, &semantic)
        .expect("active successor release registry");
    let registry_value: Value =
        serde_json::from_slice(&registry_bytes).expect("release registry JSON");
    assert_eq!(canonical_line(&registry_value), registry_bytes);
    assert_eq!(registry.registry_sha256(), ACTIVE_RELEASE_REGISTRY_SHA256);
    assert_eq!(array(&registry_value["frontend_bundles"]).len(), 4);
    assert_eq!(array(&registry_value["toolchain_bundles"]).len(), 4);
    assert_eq!(array(&registry_value["tuples"]).len(), 5);

    for (profile, tuple_count) in [("go", 1), ("rust", 2), ("csharp", 1), ("java", 1)] {
        let path = format!("release/bundles/candidates/{profile}.json");
        let candidate_bytes = read(&path);
        let candidate_value: Value =
            serde_json::from_slice(&candidate_bytes).expect("candidate JSON");
        assert_eq!(
            canonical_line(&candidate_value),
            candidate_bytes,
            "{profile}"
        );
        let candidate = validate_successor_bundle_candidate(&candidate_bytes, &semantic)
            .unwrap_or_else(|error| panic!("{profile} candidate: {error}"));
        assert_eq!(candidate.candidate().tuples.len(), tuple_count, "{profile}");
        for field in [
            "execution_host_profiles",
            "native_runtime_layout_profiles",
            "frontend_bundles",
            "toolchain_bundles",
            "tuples",
        ] {
            for item in array(&candidate_value[field]) {
                assert!(
                    array(&registry_value[field]).contains(item),
                    "{profile} {field} item is absent from the active registry"
                );
            }
        }
    }

    let csharp = array(&semantic_value["profiles"])
        .iter()
        .find(|entry| entry["source_language"] == "csharp")
        .expect("active C# semantic entry");
    assert_eq!(object(&csharp["contracts"]).len(), 9);
    let java = array(&semantic_value["profiles"])
        .iter()
        .find(|entry| entry["source_language"] == "java")
        .expect("active Java semantic entry");
    assert_eq!(object(&java["contracts"]).len(), 9);
}

#[test]
fn installed_successor_gate_owns_replay_hostile_ambient_and_tamper_rejection() {
    let runner = String::from_utf8(read("crates/mpk-cli/tests/successor_atomic_cutover.rs"))
        .expect("cutover owner source UTF-8");
    for marker in [
        "run_once(false)",
        "run_once(true)",
        "second.stdout == first.stdout",
        "DOTNET_ROOT",
        "LD_LIBRARY_PATH",
        "JAVA_TOOL_OPTIONS",
        "MPK_PLUGIN",
        "run-installed-java-native-gate",
        "--inside-successor-java-trace-probe",
        "native_report[\"undelegated_cgroup_rejected\"] == true",
        "mutations.len() == 10",
        "tampered installed successor image did not fail closed",
        "validate_active_models",
    ] {
        assert!(
            runner.contains(marker),
            "missing installed gate marker {marker}"
        );
    }

    let assembler = String::from_utf8(read("scripts/successor_release_bundles.py"))
        .expect("successor assembler source UTF-8");
    for marker in [
        "def check()",
        "first = build_roots",
        "second = build_roots",
        "def install(",
        "def check_java_trace_parser()",
        "materialize-fixture",
        "run-installed",
        "BUNDLE_JAVA_NATIVE_CGROUP_REQUIRED",
        "--inside-successor-java-trace-probe",
        "source_precedence_mutations",
    ] {
        assert!(
            assembler.contains(marker),
            "missing release assembler marker {marker}"
        );
    }

    let gate =
        String::from_utf8(read("scripts/check-java-frontend.sh")).expect("Java gate source UTF-8");
    for marker in [
        "for pass in 1 2",
        "--check successor",
        "--fixture successor",
        "successor_atomic_cutover",
        "--ignored",
        "/usr/bin/strace",
        "offline_java_candidate_builds_twice_and_refuses_ambient_options",
        "pinned_source_admission_executes_every_owned_case_and_preserves_full_closure",
        "pinned_contract_executor_matches_independent_normalized_hashes_and_all_refusals",
        "pinned_java_reaches_same_byte_certificate_and_private_consumers",
        "pinned_t09_release_rehearsal_builds_and_runs_twice",
        "csharp_profile_vectors",
        "csharp_release_gate",
    ] {
        assert!(
            gate.contains(marker),
            "missing active successor gate marker {marker}"
        );
    }
    for forbidden in [
        "--provision",
        "curl ",
        "wget ",
        "git clone",
        "cargo install",
    ] {
        assert!(
            !gate.contains(forbidden),
            "successor gate can provision or fetch: {forbidden}"
        );
    }

    let retired = String::from_utf8(read("scripts/check-csharp-frontend.sh"))
        .expect("retired C# gate source UTF-8");
    assert!(retired.contains("retired by JAVA-03-T10"));
    assert!(retired.contains("check-java-frontend.sh"));
    assert!(!retired.contains("for pass in 1 2"));
}

#[test]
fn predecessor_publication_actions_are_not_script_routes() {
    let root = repository_root();
    for (script, action) in [
        ("scripts/release_bundles.py", "update-go"),
        ("scripts/release_bundles.py", "check-all"),
        ("scripts/release_bundles.py", "fixture-go"),
        ("scripts/rust_build_inputs.py", "update-candidate"),
        ("scripts/rust_build_inputs.py", "check-candidate"),
    ] {
        let output = Command::new("/usr/bin/python3")
            .args(["-B", script, action])
            .current_dir(&root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("TMPDIR", "/tmp")
            .output()
            .expect("run retired publication action");
        assert_eq!(output.status.code(), Some(64), "{script} {action}");
        assert!(output.stdout.is_empty(), "{script} {action}");
        assert_eq!(
            output.stderr, b"BUNDLE_ASSEMBLER_USAGE\n",
            "{script} {action}"
        );
    }
}

#[test]
fn active_release_artifacts_have_no_host_path_credential_or_network_leakage() {
    let mut artifacts = Vec::new();
    for root in ["release/bundles", "fixtures/csharp", "fixtures/vir-go"] {
        collect_json(&repository_root().join(root), &mut artifacts);
    }
    assert!(artifacts.len() > 20, "complete active JSON inventory");
    for path in artifacts {
        let transport = fs::read_to_string(&path).expect("active artifact UTF-8");
        // The frozen JDK inventory legitimately contains the upstream
        // jmxremote.password.template filename. Remove only that exact public
        // distribution path before scanning the artifact transport for secret
        // material; any other password occurrence must still fail closed.
        let lower = transport
            .to_ascii_lowercase()
            .replace("jdk/conf/management/jmxremote.password.template", "");
        for forbidden in [
            "/root/",
            "/home/",
            "/tmp/",
            "http://",
            "https://",
            "authorization",
            "bearer ",
            "api_key",
            "access_token",
            "password",
            "private_key",
        ] {
            assert!(
                !lower.contains(forbidden),
                "{} leaked {forbidden}",
                path.display()
            );
        }
    }
}

fn collect_json(directory: &Path, output: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .expect("read active artifact directory")
        .map(|entry| entry.expect("active artifact entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        let metadata = fs::symlink_metadata(&path).expect("active artifact metadata");
        assert!(!metadata.file_type().is_symlink(), "{}", path.display());
        if metadata.is_dir() {
            collect_json(&path, output);
        } else if metadata.is_file() && path.extension().is_some_and(|value| value == "json") {
            output.push(path);
        }
    }
}

#[test]
fn production_activation_is_complete_and_the_executable_staging_tree_is_absent() {
    let active = String::from_utf8(read("release/bundles/bundle-registry.json"))
        .expect("active registry UTF-8");
    assert!(active.contains("csharp"));
    assert!(active.contains("mpk.release.bundle_registry.v1"));
    assert!(!repository_root()
        .join("develop/migrations/csharp-02-staging")
        .exists());

    let todo = String::from_utf8(read(
        "develop/docs/06_multilanguage_frontend_design-todo.md",
    ))
    .expect("todo UTF-8");
    assert!(todo.contains("CSHARP-02-T20"));
    assert!(todo.contains("Status: Complete (2026-08-30)."));
}
