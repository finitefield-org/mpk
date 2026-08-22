#![allow(internal_features)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

#[path = "../src/rustc_driver.rs"]
mod rustc_driver_adapter;

use rust2vir_internal::driver_protocol::{
    parse_request_transport, DriverInputIdentity, DriverRequest,
};
use rust2vir_internal::file_loader::SnapshotFileLoader;
use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::mir_access::{
    compatibility_fingerprint, validate_compatibility, MirAccessError, MirAccessTracker,
    MIR_DIALECT_SHA256, MIR_DIALECT_SUMMARY, MIR_PROFILE_ID, MIR_QUERY,
};
use rust2vir_internal::sha256::{digest, hex, Sha256};
use rust2vir_internal::EXPECTED_RUSTC_COMMIT;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const VECTOR: &[u8] = include_bytes!("../testdata/rust-driver-v0.json");
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[test]
fn pinned_query_and_dialect_fingerprint_are_golden() {
    assert_eq!(
        compatibility_fingerprint(MIR_DIALECT_SUMMARY),
        MIR_DIALECT_SHA256
    );
    assert_eq!(
        validate_compatibility(
            MIR_PROFILE_ID,
            EXPECTED_RUSTC_COMMIT,
            MIR_QUERY,
            MIR_DIALECT_SUMMARY,
            MIR_DIALECT_SHA256,
        ),
        Ok(())
    );
    for (profile, commit, query, summary, fingerprint, expected) in [
        (
            "mpk.rust.mir.future",
            EXPECTED_RUSTC_COMMIT,
            MIR_QUERY,
            MIR_DIALECT_SUMMARY,
            MIR_DIALECT_SHA256,
            MirAccessError::Profile,
        ),
        (
            MIR_PROFILE_ID,
            "0000000000000000000000000000000000000000",
            MIR_QUERY,
            MIR_DIALECT_SUMMARY,
            MIR_DIALECT_SHA256,
            MirAccessError::Compiler,
        ),
        (
            MIR_PROFILE_ID,
            EXPECTED_RUSTC_COMMIT,
            "optimized_mir",
            MIR_DIALECT_SUMMARY,
            MIR_DIALECT_SHA256,
            MirAccessError::Query,
        ),
        (
            MIR_PROFILE_ID,
            EXPECTED_RUSTC_COMMIT,
            MIR_QUERY,
            "changed dialect",
            MIR_DIALECT_SHA256,
            MirAccessError::Dialect,
        ),
        (
            MIR_PROFILE_ID,
            EXPECTED_RUSTC_COMMIT,
            MIR_QUERY,
            MIR_DIALECT_SUMMARY,
            "0000000000000000000000000000000000000000000000000000000000000000",
            MirAccessError::Dialect,
        ),
    ] {
        assert_eq!(
            validate_compatibility(profile, commit, query, summary, fingerprint),
            Err(expected)
        );
    }
}

#[test]
fn every_body_is_forced_and_borrowed_once_before_completion() {
    let mut tracker = MirAccessTracker::new(["vector::f".to_owned()]).unwrap();
    tracker.force("vector::f", MIR_QUERY).unwrap();
    tracker.mark_borrowed("vector::f").unwrap();
    assert_eq!(tracker.finish().unwrap(), ["vector::f"]);

    let mut optimized = MirAccessTracker::new(["vector::f".to_owned()]).unwrap();
    assert_eq!(
        optimized.force("vector::f", "optimized_mir"),
        Err(MirAccessError::QueryTheft)
    );
    assert_eq!(optimized.finish(), Err(MirAccessError::Incomplete));

    let mut unforced = MirAccessTracker::new(["vector::f".to_owned()]).unwrap();
    assert_eq!(
        unforced.mark_borrowed("vector::f"),
        Err(MirAccessError::QueryTheft)
    );

    let mut duplicate = MirAccessTracker::new(["vector::f".to_owned()]).unwrap();
    duplicate.force("vector::f", MIR_QUERY).unwrap();
    assert_eq!(
        duplicate.force("vector::f", MIR_QUERY),
        Err(MirAccessError::DuplicateRequest)
    );
    assert_eq!(
        duplicate.mark_borrowed("vector::unknown"),
        Err(MirAccessError::UnknownBody)
    );
}

#[test]
fn pinned_callbacks_borrow_preoptimization_mir_from_the_real_compiler() {
    for (target, pointer_width) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let request = request(target, pointer_width);
        let root = std::env::temp_dir().join(format!(
            "rust2vir-mir-access-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let source_path = root.join("src/lib.rs");
        let output_path = root.join("target");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir(&output_path).unwrap();
        let source = b"pub fn identity(x: u64) -> u64 { x }\n";
        fs::write(&source_path, source).unwrap();
        let loader = SnapshotFileLoader::open(
            &root,
            "src/lib.rs",
            &[DriverInputIdentity {
                kind: "source".to_owned(),
                normalized_path: "src/lib.rs".to_owned(),
                size_bytes: source.len() as u64,
                sha256: hex(&digest(source)),
            }],
        )
        .unwrap();
        let arguments = compiler_arguments(&source_path, &output_path, target);
        assert_eq!(
            rustc_driver_adapter::run_primary(&arguments, &request, Arc::new(loader)),
            Ok(())
        );
        fs::remove_dir_all(root).unwrap();
    }
}

fn compiler_arguments(
    source: &std::path::Path,
    output: &std::path::Path,
    target: &str,
) -> Vec<String> {
    [
        "/mpk/toolchain/bin/rustc".to_owned(),
        "--crate-name".to_owned(),
        "vector".to_owned(),
        "--edition=2021".to_owned(),
        source.to_str().unwrap().to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        "--emit=metadata".to_owned(),
        "--out-dir".to_owned(),
        output.to_str().unwrap().to_owned(),
        "--target".to_owned(),
        target.to_owned(),
        "-C".to_owned(),
        "overflow-checks=yes".to_owned(),
        "-C".to_owned(),
        "panic=abort".to_owned(),
        "-C".to_owned(),
        "debug-assertions=no".to_owned(),
        "-C".to_owned(),
        "opt-level=0".to_owned(),
        "-Z".to_owned(),
        "mir-opt-level=0".to_owned(),
        "--remap-path-prefix=/mpk/input=.".to_owned(),
    ]
    .into_iter()
    .collect()
}

fn request(target: &str, pointer_width: u8) -> DriverRequest {
    let vector = json::parse(VECTOR, VECTOR.len()).unwrap();
    let fixture = vector.as_object().unwrap()["valid_request"]
        .as_object()
        .unwrap();
    let mut value = fixture["value"].clone();
    let root = value.as_object_mut().unwrap();
    let parameters = root
        .get_mut("semantic_parameters")
        .unwrap()
        .as_object_mut()
        .unwrap();
    parameters.insert("target_id".to_owned(), JsonValue::String(target.to_owned()));
    parameters.insert(
        "pointer_width".to_owned(),
        JsonValue::Number(pointer_width.to_string()),
    );
    root.get_mut("compiler")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("target".to_owned(), JsonValue::String(target.to_owned()));
    root.remove("request_fingerprint");
    let mut hasher = Sha256::new();
    hasher.update(b"MPK-RUST-DRIVER-REQUEST-0.1");
    hasher.update(&[0]);
    hasher.update(&json::canonical(&value).unwrap());
    value.as_object_mut().unwrap().insert(
        "request_fingerprint".to_owned(),
        JsonValue::String(hex(&hasher.finish())),
    );
    let mut bytes = json::canonical(&value).unwrap();
    bytes.push(b'\n');
    parse_request_transport(&bytes).unwrap()
}
