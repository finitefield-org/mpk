use mpk_cert::encode::AxiomCategory;
use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
};
use mpk_vc::semantic_profile_registry::{
    validate_inactive_semantic_profile_registry, InactiveRegistryRevision,
};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const STAGING_ROOT: &str = "develop/migrations/csharp-02-staging";
const REVIEW_PATH: &str = "develop/migrations/csharp-02-staging/final-review.json";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative)).expect("read staged release input")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative)).expect("parse staged release JSON")
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
fn review_ledger_is_canonical_complete_and_empty() {
    let bytes = read(REVIEW_PATH);
    let review: Value = serde_json::from_slice(&bytes).expect("review ledger JSON");
    assert_eq!(canonical_line(&review), bytes);
    assert_eq!(
        object(&review)
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "axiom_review",
            "findings",
            "rehearsal",
            "reviewed_surfaces",
            "schema",
            "status",
            "task",
            "trust",
        ]
    );
    assert_eq!(
        review["schema"],
        "mpk.csharp.staged_release.final_review.v0"
    );
    assert_eq!(review["status"], "reviewed_zero_findings");
    assert_eq!(review["task"], "CSHARP-02-T19");
    assert_eq!(review["trust"], "untrusted_review_record");
    assert!(array(&review["findings"]).is_empty());
    assert_eq!(array(&review["reviewed_surfaces"]).len(), 7);

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
    assert_eq!(
        review["axiom_review"]["status"],
        "unchanged_zero_new_categories"
    );

    let rehearsal = &review["rehearsal"];
    assert_eq!(rehearsal["gate_passes"], 2);
    assert_eq!(rehearsal["network_access"], false);
    assert_eq!(rehearsal["provisioning"], false);
    assert_eq!(rehearsal["release_state"], "inactive_staging");
    assert_eq!(rehearsal["runtime"], "10.0.11");
    assert!(array(&rehearsal["credential_sources"]).is_empty());
    assert_eq!(
        array(&rehearsal["profile_order"])
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        ["go", "rust", "csharp"]
    );
    assert_eq!(
        array(&rehearsal["installed_fixtures"])
            .iter()
            .map(text)
            .collect::<Vec<_>>(),
        ["go-successor", "rust-successor", "csharp"]
    );
    assert_eq!(array(&rehearsal["artifact_equality"]).len(), 3);
}

#[test]
fn every_staged_profile_candidate_and_registry_is_exactly_validated() {
    let semantic_bytes = read(&format!("{STAGING_ROOT}/semantic-profile-registry.json"));
    let semantic = validate_inactive_semantic_profile_registry(
        &semantic_bytes,
        InactiveRegistryRevision::Revision2,
    )
    .expect("staged revision-2 semantic registry");
    let semantic_value: Value =
        serde_json::from_slice(&semantic_bytes).expect("semantic registry JSON");
    assert_eq!(canonical_line(&semantic_value), semantic_bytes);

    for (profile, candidate_path, registry_path, tuple_count) in [
        (
            "go",
            "go-bundle-candidate.json",
            "go-bundle-registry.json",
            1,
        ),
        (
            "rust",
            "rust-bundle-candidate.json",
            "rust-bundle-registry.json",
            2,
        ),
        (
            "csharp",
            "csharp-bundle-candidate.json",
            "bundle-registry.json",
            1,
        ),
    ] {
        let candidate_bytes = read(&format!("{STAGING_ROOT}/{candidate_path}"));
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

        let registry_bytes = read(&format!("{STAGING_ROOT}/{registry_path}"));
        let registry_value: Value =
            serde_json::from_slice(&registry_bytes).expect("release registry JSON");
        assert_eq!(canonical_line(&registry_value), registry_bytes, "{profile}");
        let registry = validate_successor_release_registry(&registry_bytes, &semantic)
            .unwrap_or_else(|error| panic!("{profile} registry: {error}"));
        assert_eq!(registry.registry().tuples.len(), tuple_count, "{profile}");
        assert_eq!(
            candidate.candidate().tuples,
            registry.registry().tuples,
            "{profile} candidate/registry tuple projection"
        );
    }
}

#[test]
fn installed_rehearsal_owns_two_run_hostile_ambient_and_tamper_rejection() {
    let runner = String::from_utf8(read("crates/mpk-cli/tests/csharp_frontend_runner.rs"))
        .expect("runner source UTF-8");
    for marker in [
        "run_once(false)",
        "run_once(true)",
        "replay.stdout == execution.stdout",
        "DOTNET_ROLL_FORWARD",
        "LD_LIBRARY_PATH",
        "MPK_PLUGIN",
        "BUNDLE_REPRODUCIBILITY_MISMATCH",
        "validate_active_registry_boundary",
    ] {
        assert!(
            runner.contains(marker),
            "missing installed rehearsal marker {marker}"
        );
    }

    let assembler = String::from_utf8(read("scripts/csharp_release_bundles.py"))
        .expect("assembler source UTF-8");
    for marker in [
        "compare_trees(first_output, second_output)",
        "first_candidate != second_candidate",
        "first_registry != second_registry",
        "materialize-fixture",
        "run-installed",
    ] {
        assert!(
            assembler.contains(marker),
            "missing release assembler marker {marker}"
        );
    }

    let gate =
        String::from_utf8(read("scripts/check-csharp-frontend.sh")).expect("C# gate source UTF-8");
    for marker in [
        "for pass in 1 2",
        "--check csharp",
        "--fixture csharp",
        "--fixture go-successor",
        "--fixture rust-successor",
        "csharp_profile_vectors",
        "csharp_release_gate",
    ] {
        assert!(gate.contains(marker), "missing C# gate marker {marker}");
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
            "C# gate can provision or fetch: {forbidden}"
        );
    }
}

#[test]
fn staged_evidence_has_no_host_path_credential_or_network_leakage() {
    let mut artifacts = Vec::new();
    collect_staged_json(&repository_root().join(STAGING_ROOT), &mut artifacts);
    assert!(artifacts.len() > 20, "complete staged JSON inventory");
    for path in artifacts {
        let transport = fs::read_to_string(&path).expect("staged artifact UTF-8");
        let lower = transport.to_ascii_lowercase();
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

    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let isolation = array(&profile["isolation_cases"])
        .iter()
        .map(|case| text(&case["id"]))
        .collect::<Vec<_>>();
    assert!(isolation.contains(&"isolation.no_network"));
    assert!(isolation.contains(&"isolation.no_plugins"));
    assert!(isolation.contains(&"isolation.no_environment_inheritance"));
    assert_eq!(
        profile["launcher_contract"]["inherited_environment"],
        Value::Array(vec![])
    );
}

fn collect_staged_json(directory: &Path, output: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .expect("read staged artifact directory")
        .map(|entry| entry.expect("staged artifact entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        let metadata = fs::symlink_metadata(&path).expect("staged artifact metadata");
        assert!(!metadata.file_type().is_symlink(), "{}", path.display());
        if metadata.is_dir() {
            collect_staged_json(&path, output);
        } else if metadata.is_file() && path.extension().is_some_and(|value| value == "json") {
            output.push(path);
        }
    }
}

#[test]
fn production_activation_is_still_absent() {
    let active = String::from_utf8(read("release/bundles/bundle-registry.json"))
        .expect("active registry UTF-8");
    assert!(!active.contains("csharp"));
    assert!(!active.contains("mpk.release.bundle_registry.v1"));

    let todo = String::from_utf8(read(
        "develop/docs/06_multilanguage_frontend_design-todo.md",
    ))
    .expect("todo UTF-8");
    assert!(todo.contains("CSHARP-02-T20"));
    assert!(todo.contains("T20 alone may activate the successor release"));

    let review = load(REVIEW_PATH);
    assert_eq!(review["rehearsal"]["release_state"], "inactive_staging");
}
