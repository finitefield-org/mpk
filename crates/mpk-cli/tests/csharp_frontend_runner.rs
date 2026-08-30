use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
    ACTIVE_RELEASE_REGISTRY_SHA256, CSHARP_FRONTEND_BUNDLE_ID, CSHARP_TOOLCHAIN_BUNDLE_ID,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::validate_release_registry;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DOTNET_PROGRAM: &str = "/mpk/toolchain/dotnet/dotnet";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> Vec<u8> {
    fs::read(repository_root().join(relative)).expect("read repository input")
}

fn load(relative: &str) -> Value {
    serde_json::from_slice(&read(relative)).expect("parse repository JSON")
}

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("serialize canonical JSON");
    bytes.push(b'\n');
    bytes
}

#[test]
fn csharp_candidate_is_an_exact_member_of_the_active_successor_release() {
    let semantic_bytes = read("release/bundles/semantic-profile-registry.json");
    let semantic = validate_semantic_profile_registry(&semantic_bytes, RegistryRevision::Revision2)
        .expect("active semantic registry");
    let registry_bytes = read("release/bundles/bundle-registry.json");
    let registry = validate_successor_release_registry(&registry_bytes, &semantic)
        .expect("active successor release registry");
    let candidate_bytes = read("release/bundles/candidates/csharp.json");
    let candidate = validate_successor_bundle_candidate(&candidate_bytes, &semantic)
        .expect("active C# candidate");

    assert_eq!(registry.registry_sha256(), ACTIVE_RELEASE_REGISTRY_SHA256);
    assert_eq!(registry.registry().frontend_bundles.len(), 3);
    assert_eq!(registry.registry().toolchain_bundles.len(), 3);
    assert_eq!(registry.registry().tuples.len(), 4);
    assert_eq!(candidate.candidate().frontend_bundles.len(), 1);
    assert_eq!(candidate.candidate().toolchain_bundles.len(), 1);
    assert_eq!(candidate.candidate().tuples.len(), 1);
    assert_eq!(
        candidate.candidate().frontend_bundles[0].bundle_id,
        CSHARP_FRONTEND_BUNDLE_ID
    );
    assert_eq!(
        candidate.candidate().toolchain_bundles[0].bundle_id,
        CSHARP_TOOLCHAIN_BUNDLE_ID
    );
    assert!(registry
        .registry()
        .frontend_bundles
        .contains(&candidate.candidate().frontend_bundles[0]));
    assert!(registry
        .registry()
        .toolchain_bundles
        .contains(&candidate.candidate().toolchain_bundles[0]));
    assert!(registry
        .registry()
        .tuples
        .contains(&candidate.candidate().tuples[0]));

    assert!(
        validate_release_registry(&registry_bytes).is_err(),
        "the predecessor release parser accepted successor registry bytes"
    );
    let mut predecessor_candidate: Value =
        serde_json::from_slice(&candidate_bytes).expect("candidate JSON");
    predecessor_candidate["schema"] = json!("mpk.release.bundle_candidate.v0");
    assert!(validate_successor_bundle_candidate(
        &canonical_line(&predecessor_candidate),
        &semantic
    )
    .is_err());
}

#[test]
fn csharp_launcher_and_isolation_contract_remain_closed_after_activation() {
    let profile = load("develop/specs/vectors/csharp-profile-v0.json");
    let launcher = &profile["launcher_contract"];
    assert_eq!(launcher["program"], DOTNET_PROGRAM);
    assert_eq!(launcher["working_directory"], "/mpk/source");
    assert_eq!(launcher["stdin"], "null");
    assert_eq!(launcher["stdout"], "bounded_frontend_protocol");
    assert_eq!(launcher["stderr"], "bounded_diagnostic_only");
    assert_eq!(launcher["inherited_environment"], json!([]));
    assert_eq!(
        launcher["runtime_config"],
        json!({
            "framework_name": "Microsoft.NETCore.App",
            "framework_version": "10.0.11",
            "roll_forward": "Disable",
            "tfm": "net10.0"
        })
    );
    assert_eq!(
        launcher["argv_prefix"],
        json!([
            DOTNET_PROGRAM,
            "exec",
            "--depsfile",
            "/mpk/frontend/csharp2vir.deps.json",
            "--runtimeconfig",
            "/mpk/frontend/csharp2vir.runtimeconfig.json",
            "--fx-version",
            "10.0.11",
            "/mpk/frontend/csharp2vir.dll"
        ])
    );
    assert_eq!(
        launcher["environment"]
            .as_object()
            .expect("closed environment")
            .len(),
        18
    );

    let isolation = profile["isolation_cases"]
        .as_array()
        .expect("isolation cases")
        .iter()
        .map(|case| case["id"].as_str().expect("isolation ID"))
        .collect::<Vec<_>>();
    assert_eq!(isolation.len(), 12);
    for required in [
        "isolation.no_network",
        "isolation.no_restore",
        "isolation.no_environment_inheritance",
        "isolation.no_dynamic_native_search",
        "isolation.no_candidate_execution",
        "isolation.no_plugins",
    ] {
        assert!(isolation.contains(&required), "missing {required}");
    }
}

#[test]
fn cutover_owner_is_registered_on_both_semantic_registry_vectors() {
    let manifest = load("develop/specs/vectors/manifest.json");
    for path in [
        "develop/specs/vectors/semantic-profile-registry-v1.json",
        "develop/specs/vectors/semantic-profile-registry-v2.json",
    ] {
        let record = manifest["vectors"]
            .as_array()
            .expect("vector records")
            .iter()
            .find(|record| record["path"] == path)
            .unwrap_or_else(|| panic!("missing vector record {path}"));
        assert!(record["implementation_test_owners"]
            .as_array()
            .expect("implementation owners")
            .contains(&json!("crates/mpk-cli/tests/successor_atomic_cutover.rs")));
    }
}
