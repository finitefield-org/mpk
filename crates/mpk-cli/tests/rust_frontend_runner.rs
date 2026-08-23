use mpk_vc::{validate_release_registry, CompilerIdentity, ReleaseSelectionRequest};
use std::fs;
use std::path::PathBuf;

#[test]
fn tracked_registry_resolves_both_closed_rust_release_tuples() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = fs::read(root.join("release/bundles/bundle-registry.json"))
        .expect("read tracked release registry");
    let registry = validate_release_registry(&bytes).expect("validate tracked release registry");
    let rust_frontend = registry
        .registry()
        .frontend_bundles
        .iter()
        .find(|bundle| bundle.source_language == "rust")
        .expect("Rust frontend is registered");
    let rust_toolchain = registry
        .registry()
        .toolchain_bundles
        .iter()
        .find(|bundle| bundle.source_language == "rust")
        .expect("Rust toolchain is registered");
    assert_eq!(rust_frontend.name, "rust2vir");
    assert_eq!(rust_frontend.main.path, "bin/rust2vir");
    assert_eq!(rust_frontend.subordinate_binaries.len(), 1);
    assert_eq!(
        rust_frontend.subordinate_binaries[0].name,
        "rust2vir-driver"
    );
    assert_eq!(
        rust_frontend.subordinate_binaries[0].path,
        "bin/rust2vir-driver"
    );
    assert!(matches!(
        &rust_toolchain.compiler,
        CompilerIdentity::Rust { release, rustc_commit }
            if release == "1.89.0-nightly"
                && rustc_commit == "4d08223c054cf5a56d9761ca925fd46ffebe7115"
    ));

    for (target_id, pointer_width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let selected = registry
            .resolve(&ReleaseSelectionRequest {
                registry_id: registry.registry().id.clone(),
                registry_sha256: registry.registry_digest().to_hex(),
                source_language: "rust".to_owned(),
                semantic_profile: "mpk.rust.checked.v0".to_owned(),
                target_id: target_id.to_owned(),
                frontend_bundle_id: Some(rust_frontend.bundle_id.clone()),
                toolchain_bundle_id: Some(rust_toolchain.bundle_id.clone()),
            })
            .expect("registered Rust tuple resolves");
        assert_eq!(selected.release_tuple.pointer_width, pointer_width);
        assert_eq!(selected.release_tuple.limit_profile_id, "mpk.vir.limits.v0");
        assert_eq!(selected.frontend.bundle_id, rust_frontend.bundle_id);
        assert_eq!(selected.toolchain.bundle_id, rust_toolchain.bundle_id);
    }
}
