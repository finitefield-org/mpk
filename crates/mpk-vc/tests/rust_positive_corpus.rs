use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use mpk_vc::{canonical_json_bytes, parse_strict_json, sha256_raw_file_bytes, StrictJsonLimits};
use serde_json::Value;

const PRODUCER_ROOT: &str = "rust-tools/rust2vir/testdata/positive";
const PUBLIC_ROOT: &str = "fixtures/rust-basic/positive";
const ARTIFACT_KINDS: [(&str, &str); 8] = [
    ("frontend_envelope", "mpk.frontend.cli.v1"),
    ("vir", "mpk.vir.v1"),
    ("source_map", "mpk.source_map.v1"),
    ("source_manifest_frontend", "mpk.source_manifest.v1"),
    ("private_request", "mpk.rust.driver.request.v1"),
    ("private_result", "mpk.rust.driver.v1"),
    ("raw_lowering", "mpk.rust.driver.lowering.v1"),
    ("raw_source_map", "mpk.rust.driver.raw_source_map.v1"),
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: impl AsRef<Path>) -> Vec<u8> {
    fs::read(repository_root().join(relative.as_ref()))
        .unwrap_or_else(|error| panic!("read {}: {error}", relative.as_ref().display()))
}

fn canonical(value: &Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).expect("JSON serializes");
    let strict = parse_strict_json(
        &encoded,
        StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576),
    )
    .expect("strict JSON");
    canonical_json_bytes(&strict).expect("canonical JSON")
}

fn artifact<'a>(case: &'a Value, kind: &str) -> &'a Value {
    case["artifacts"]
        .as_array()
        .expect("artifact array")
        .iter()
        .find(|item| item["kind"] == kind)
        .unwrap_or_else(|| panic!("{} is missing {kind}", case["id"]))
}

#[test]
fn active_rust_positive_corpus_is_successor_only_and_byte_mirrored() {
    let producer_index = read(format!("{PRODUCER_ROOT}/frontend-index.json"));
    let public_index = read(format!("{PUBLIC_ROOT}/frontend-index.json"));
    assert_eq!(producer_index, public_index, "frontend-index mirror");
    let index: Value = serde_json::from_slice(&producer_index).expect("frontend index JSON");
    assert_eq!(producer_index, canonical(&index));
    assert_eq!(index["schema"], "mpk.rust.positive_frontend_corpus.v1");
    assert_eq!(index["deterministic_runs"], 2);
    assert_eq!(
        index["release_registry"]["schema"],
        "mpk.release.bundle_registry.v1"
    );
    assert_eq!(index["release_registry"]["id"], "mpk.release.registry.v1");
    assert_eq!(
        index["private_protocol"],
        serde_json::json!({
            "payload_hash_domain":"MPK-RUST-DRIVER-PAYLOAD-1.0",
            "raw_lowering_schema":"mpk.rust.driver.lowering.v1",
            "raw_source_map_schema":"mpk.rust.driver.raw_source_map.v1",
            "request_hash_domain":"MPK-RUST-DRIVER-REQUEST-1.0",
            "request_schema":"mpk.rust.driver.request.v1",
            "result_schema":"mpk.rust.driver.v1"
        })
    );

    let cases = index["cases"].as_array().expect("positive cases");
    assert_eq!(cases.len(), 13);
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().expect("case ID");
        assert!(ids.insert(id), "duplicate case {id}");
        assert_eq!(case["frontend_status"], "ir-lowered");
        assert_eq!(case["semantic_context"]["source_language"], "rust");
        assert_eq!(case["semantic_context"]["profile_registry"]["revision"], 3);
        assert_eq!(
            case["selection"]["schema"],
            "mpk.selection.rust_function.v0"
        );
        assert_eq!(
            case["artifacts"].as_array().unwrap().len(),
            ARTIFACT_KINDS.len()
        );

        for (kind, schema) in ARTIFACT_KINDS {
            let descriptor = artifact(case, kind);
            let relative = descriptor["path"].as_str().expect("artifact path");
            let producer = read(Path::new(PRODUCER_ROOT).join(relative));
            let public = read(Path::new(PUBLIC_ROOT).join(relative));
            assert_eq!(producer, public, "{id}: {kind} mirror");
            assert_eq!(producer.len() as u64, descriptor["bytes"].as_u64().unwrap());
            assert_eq!(
                sha256_raw_file_bytes(&producer).to_hex(),
                descriptor["sha256"].as_str().unwrap()
            );
            let value: Value = serde_json::from_slice(&producer).expect("artifact JSON");
            let canonical_bytes = canonical(&value);
            if matches!(
                kind,
                "frontend_envelope" | "private_request" | "private_result"
            ) {
                assert_eq!(
                    producer.strip_suffix(b"\n"),
                    Some(canonical_bytes.as_slice()),
                    "{id}: {kind} canonical process line"
                );
            } else {
                assert_eq!(producer, canonical_bytes, "{id}: {kind} canonical");
            }
            assert_eq!(value["schema"], schema, "{id}: {kind} schema");
            if matches!(kind, "frontend_envelope" | "vir" | "source_map") {
                assert_eq!(
                    value["semantic_context"], case["semantic_context"],
                    "{id}: {kind} context"
                );
            }
            for predecessor in [
                "mpk.frontend.cli.v0",
                "mpk.vir.v0",
                "mpk.source_map.v0",
                "mpk.source_manifest.v0",
                "mpk.rust.driver.request.v0",
                "mpk.rust.driver.v0",
            ] {
                assert!(
                    !String::from_utf8_lossy(&producer).contains(predecessor),
                    "{id}: {kind} retained {predecessor}"
                );
            }
        }
    }
}
