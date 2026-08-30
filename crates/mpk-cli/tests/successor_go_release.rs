use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
    SuccessorReleaseSelectionRequest, GO_FRONTEND_BUNDLE_ID, GO_FRONTEND_SHA256,
    GO_TOOLCHAIN_BUNDLE_ID,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(relative: &str) -> Vec<u8> {
    std::fs::read(repository_root().join(relative)).expect("checked-in release artifact")
}

fn semantic_registry() -> mpk_vc::semantic_profile_registry::ValidatedSemanticProfileRegistry {
    validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision2,
    )
    .expect("revision-2 semantic registry")
}

#[test]
fn active_go_candidate_and_registry_resolve_the_successor_tuple() {
    let semantic = semantic_registry();
    let candidate_bytes = read("release/bundles/candidates/go.json");
    let candidate = validate_successor_bundle_candidate(&candidate_bytes, &semantic)
        .expect("successor Go candidate validates");
    assert_eq!(candidate.candidate().frontend_bundles.len(), 1);
    assert_eq!(candidate.candidate().toolchain_bundles.len(), 1);
    assert_eq!(candidate.candidate().tuples.len(), 1);
    assert_eq!(
        candidate.candidate().frontend_bundles[0].bundle_id,
        GO_FRONTEND_BUNDLE_ID
    );
    assert_eq!(
        candidate.candidate().frontend_bundles[0].main.binary_sha256,
        GO_FRONTEND_SHA256
    );
    assert_eq!(
        candidate.candidate().toolchain_bundles[0].bundle_id,
        GO_TOOLCHAIN_BUNDLE_ID
    );

    let registry_bytes = read("release/bundles/bundle-registry.json");
    let registry = validate_successor_release_registry(&registry_bytes, &semantic)
        .expect("successor Go registry validates");
    let document: Value = serde_json::from_slice(&registry_bytes).expect("registry JSON");
    let tuple = document["tuples"]
        .as_array()
        .expect("release tuples")
        .iter()
        .find(|tuple| tuple["frontend_bundle_id"] == GO_FRONTEND_BUNDLE_ID)
        .expect("active Go tuple");
    let resolved = registry
        .resolve(
            &semantic,
            SuccessorReleaseSelectionRequest {
                semantic_context: &tuple["semantic_context"],
                frontend_bundle_id: GO_FRONTEND_BUNDLE_ID,
                toolchain_bundle_id: GO_TOOLCHAIN_BUNDLE_ID,
            },
        )
        .expect("exact successor Go tuple resolves");
    assert_eq!(resolved.semantic_context.source_language(), "go");
    assert_eq!(
        resolved.semantic_context.semantic_profile(),
        "mpk.go.fixed.v0"
    );
    assert_eq!(resolved.frontend.main.binary_sha256, GO_FRONTEND_SHA256);
}

#[test]
fn predecessor_and_crossed_go_release_shapes_fail_closed() {
    let semantic = semantic_registry();
    let candidate_bytes = read("release/bundles/candidates/go.json");
    let candidate: Value = serde_json::from_slice(&candidate_bytes).expect("candidate JSON");

    let mut predecessor_schema = candidate.clone();
    predecessor_schema["schema"] = Value::String("mpk.release.bundle_candidate.v0".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_schema).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut predecessor_tuple = candidate.clone();
    let tuple = predecessor_tuple["tuples"][0]
        .as_object_mut()
        .expect("tuple object");
    tuple.remove("semantic_context");
    tuple.insert("source_language".into(), Value::String("go".into()));
    tuple.insert(
        "semantic_profile".into(),
        Value::String("mpk.go.fixed.v0".into()),
    );
    tuple.insert("target_id".into(), Value::String("linux/amd64".into()));
    tuple.insert("pointer_width".into(), Value::from(64));
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_tuple).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut crossed_entry = candidate.clone();
    crossed_entry["frontend_bundles"][0]["profile_contracts"][0]["profile_entry_sha256"] =
        Value::String("d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&crossed_entry).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut predecessor_frontend_version = candidate.clone();
    predecessor_frontend_version["frontend_bundles"][0]["main"]["version"] =
        Value::String("go1.25.0-profile-v0".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_frontend_version).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut crossed_component_release = candidate.clone();
    crossed_component_release["toolchain_bundles"][0]["components"][0]["release"] =
        Value::String("go1.24.0".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&crossed_component_release).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut crossed_target = candidate;
    crossed_target["toolchain_bundles"][0]["profile_contracts"][0]["value"]["target_libraries"]
        [0]["content_sha256"] = Value::String("0".repeat(64));
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&crossed_target).expect("serialize mutation"),
        &semantic,
    )
    .is_err());
}
