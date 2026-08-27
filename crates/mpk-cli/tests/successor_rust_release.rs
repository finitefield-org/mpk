use mpk_cli::successor_release_bundle::{
    validate_successor_bundle_candidate, validate_successor_release_registry,
    SuccessorReleaseSelectionRequest, RUST_STAGING_DRIVER_SHA256, RUST_STAGING_FRONTEND_BUNDLE_ID,
    RUST_STAGING_FRONTEND_SHA256, RUST_STAGING_TOOLCHAIN_BUNDLE_ID,
    RUST_STAGING_TOOLCHAIN_DISTRIBUTION_SHA256,
};
use mpk_vc::semantic_profile_registry::{
    validate_inactive_semantic_profile_registry, InactiveRegistryRevision,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(relative: &str) -> Vec<u8> {
    std::fs::read(repository_root().join(relative)).expect("checked-in staging artifact")
}

fn semantic_registry() -> mpk_vc::semantic_profile_registry::ValidatedSemanticProfileRegistry {
    validate_inactive_semantic_profile_registry(
        &read("develop/migrations/csharp-02-staging/semantic-profile-registry.json"),
        InactiveRegistryRevision::Revision2,
    )
    .expect("revision-2 semantic registry")
}

#[test]
fn staged_rust_candidate_and_registry_resolve_only_the_two_successor_targets() {
    let semantic = semantic_registry();
    let candidate_bytes = read("develop/migrations/csharp-02-staging/rust-bundle-candidate.json");
    let candidate = validate_successor_bundle_candidate(&candidate_bytes, &semantic)
        .expect("successor Rust candidate validates");
    assert_eq!(candidate.candidate().frontend_bundles.len(), 1);
    assert_eq!(candidate.candidate().toolchain_bundles.len(), 1);
    assert_eq!(candidate.candidate().tuples.len(), 2);
    let frontend = &candidate.candidate().frontend_bundles[0];
    assert_eq!(frontend.bundle_id, RUST_STAGING_FRONTEND_BUNDLE_ID);
    assert_eq!(frontend.main.binary_sha256, RUST_STAGING_FRONTEND_SHA256);
    assert_eq!(frontend.subordinate_binaries.len(), 1);
    assert_eq!(
        frontend.subordinate_binaries[0].binary_sha256,
        RUST_STAGING_DRIVER_SHA256
    );
    let toolchain = &candidate.candidate().toolchain_bundles[0];
    assert_eq!(toolchain.bundle_id, RUST_STAGING_TOOLCHAIN_BUNDLE_ID);
    assert_eq!(
        toolchain.distribution_sha256,
        RUST_STAGING_TOOLCHAIN_DISTRIBUTION_SHA256
    );

    let registry_bytes = read("develop/migrations/csharp-02-staging/rust-bundle-registry.json");
    let registry = validate_successor_release_registry(&registry_bytes, &semantic)
        .expect("successor Rust registry validates");
    let document: Value = serde_json::from_slice(&registry_bytes).expect("registry JSON");
    let mut resolved_targets = Vec::new();
    for tuple in document["tuples"].as_array().expect("release tuples") {
        let resolved = registry
            .resolve(
                &semantic,
                SuccessorReleaseSelectionRequest {
                    semantic_context: &tuple["semantic_context"],
                    frontend_bundle_id: RUST_STAGING_FRONTEND_BUNDLE_ID,
                    toolchain_bundle_id: RUST_STAGING_TOOLCHAIN_BUNDLE_ID,
                },
            )
            .expect("exact successor Rust tuple resolves");
        assert_eq!(resolved.semantic_context.source_language(), "rust");
        assert_eq!(
            resolved.semantic_context.semantic_profile(),
            "mpk.rust.checked.v0"
        );
        assert_eq!(
            resolved.frontend.main.binary_sha256,
            RUST_STAGING_FRONTEND_SHA256
        );
        resolved_targets.push(
            tuple["semantic_context"]["semantic_parameters"]["value"]["target_id"]
                .as_str()
                .expect("target ID"),
        );
    }
    assert_eq!(
        resolved_targets,
        ["i686-unknown-linux-gnu", "x86_64-unknown-linux-gnu"]
    );
}

#[test]
fn predecessor_and_crossed_rust_release_shapes_fail_closed() {
    let semantic = semantic_registry();
    let candidate_bytes = read("develop/migrations/csharp-02-staging/rust-bundle-candidate.json");
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
    tuple.insert("source_language".into(), Value::String("rust".into()));
    tuple.insert(
        "semantic_profile".into(),
        Value::String("mpk.rust.checked.v0".into()),
    );
    tuple.insert(
        "target_id".into(),
        Value::String("i686-unknown-linux-gnu".into()),
    );
    tuple.insert("pointer_width".into(), Value::from(32));
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_tuple).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut predecessor_frontend = candidate.clone();
    predecessor_frontend["frontend_bundles"][0]["version"] = Value::String("0.1.0".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_frontend).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut predecessor_driver = candidate.clone();
    predecessor_driver["frontend_bundles"][0]["subordinate_binaries"][0]["binary_sha256"] =
        Value::String("e18ada1ff29d0a9dce87230698cd89d77274633de716559ada1dc34f40e0f3ee".into());
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_driver).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut crossed_distribution = candidate.clone();
    crossed_distribution["toolchain_bundles"][0]["distribution_sha256"] =
        Value::String("0".repeat(64));
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&crossed_distribution).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let mut crossed_target = candidate.clone();
    crossed_target["toolchain_bundles"][0]["profile_contracts"][0]["value"]["target_libraries"]
        [0]["content_sha256"] = Value::String("0".repeat(64));
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&crossed_target).expect("serialize mutation"),
        &semantic,
    )
    .is_err());

    let active: Value = serde_json::from_slice(&read("release/bundles/bundle-registry.json"))
        .expect("active registry");
    let active_rust_frontend = active["frontend_bundles"]
        .as_array()
        .expect("frontends")
        .iter()
        .find(|bundle| bundle["source_language"] == "rust")
        .expect("active Rust frontend");
    assert_eq!(
        active_rust_frontend["schema"],
        "mpk.release.frontend_bundle.v0"
    );
    let mut predecessor_descriptor = candidate;
    predecessor_descriptor["frontend_bundles"][0] = active_rust_frontend.clone();
    assert!(validate_successor_bundle_candidate(
        &serde_json::to_vec(&predecessor_descriptor).expect("serialize mutation"),
        &semantic,
    )
    .is_err());
}
