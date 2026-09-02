use mpk_vc::semantic_profile_registry::{
    validate_semantic_profile_registry, RegistryRevision, ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, SuccessorSourceManifestStage,
    SuccessorSourceManifestValidationContext, SuccessorSourceMapValidationContext,
    SUCCESSOR_SOURCE_MANIFEST_SCHEMA, SUCCESSOR_SOURCE_MAP_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::{
    import_vir_json, sha256_raw_file_bytes, CapturedInput, InputKind, ReleaseRegistryIdentity,
    SourceManifest, SourceMap, SourceReference, SyntheticPermission,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ACTIVE_ROOT: &str = "fixtures/vir-go";

struct OwnedCapturedInput {
    kind: InputKind,
    normalized_path: String,
    bytes: Vec<u8>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(relative: impl AsRef<Path>) -> Vec<u8> {
    fs::read(repository_root().join(relative.as_ref()))
        .unwrap_or_else(|error| panic!("read {}: {error}", relative.as_ref().display()))
}

fn json(relative: impl AsRef<Path>) -> Value {
    let path = relative.as_ref();
    serde_json::from_slice(&read(path))
        .unwrap_or_else(|error| panic!("decode {}: {error}", path.display()))
}

fn semantic_registry() -> ValidatedSemanticProfileRegistry {
    validate_semantic_profile_registry(
        &read("release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision3,
    )
    .expect("active revision-3 semantic registry")
}

fn cases_by_id(index: &Value) -> BTreeMap<&str, &Value> {
    index["cases"]
        .as_array()
        .expect("positive case array")
        .iter()
        .map(|case| (case["id"].as_str().expect("case ID"), case))
        .collect()
}

fn negatives_by_id(index: &Value) -> BTreeMap<&str, &Value> {
    index["negative_cases"]
        .as_array()
        .expect("negative case array")
        .iter()
        .map(|case| (case["id"].as_str().expect("negative case ID"), case))
        .collect()
}

fn artifact<'a>(case: &'a Value, kind: &str) -> &'a Value {
    case["artifacts"]
        .as_array()
        .expect("artifact array")
        .iter()
        .find(|artifact| artifact["kind"] == kind)
        .unwrap_or_else(|| panic!("missing {kind} artifact for {}", case["id"]))
}

fn checked_artifact(root: &str, descriptor: &Value) -> Vec<u8> {
    let path = descriptor["path"].as_str().expect("artifact path");
    let bytes = read(Path::new(root).join(path));
    assert_eq!(
        bytes.len() as u64,
        descriptor["bytes"].as_u64().expect("artifact byte count"),
        "byte count changed for {path}"
    );
    assert_eq!(
        sha256_raw_file_bytes(&bytes).to_hex(),
        descriptor["sha256"].as_str().expect("artifact digest"),
        "raw digest changed for {path}"
    );
    bytes
}

fn input_kind(value: &Value) -> InputKind {
    match value.as_str().expect("input kind") {
        "source" => InputKind::Source,
        "contract" => InputKind::Contract,
        "build_manifest" => InputKind::BuildManifest,
        "lockfile" => InputKind::Lockfile,
        other => panic!("unknown captured input kind {other}"),
    }
}

fn captured_storage(source_root: &str, manifest: &Value) -> Vec<OwnedCapturedInput> {
    manifest["inputs"]
        .as_array()
        .expect("manifest inputs")
        .iter()
        .map(|entry| {
            let kind = input_kind(&entry["kind"]);
            let normalized_path = entry["normalized_path"]
                .as_str()
                .expect("normalized input path")
                .to_owned();
            let path = repository_root().join(source_root).join(&normalized_path);
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error)
                    if kind == InputKind::Lockfile
                        && error.kind() == std::io::ErrorKind::NotFound =>
                {
                    Vec::new()
                }
                Err(error) => panic!("read captured input {}: {error}", path.display()),
            };
            assert_eq!(
                bytes.len() as u64,
                entry["size_bytes"].as_u64().expect("captured byte count")
            );
            assert_eq!(
                sha256_raw_file_bytes(&bytes).to_hex(),
                entry["sha256"].as_str().expect("captured digest")
            );
            OwnedCapturedInput {
                kind,
                normalized_path,
                bytes,
            }
        })
        .collect()
}

fn captured_refs(storage: &[OwnedCapturedInput]) -> Vec<CapturedInput<'_>> {
    storage
        .iter()
        .map(|input| CapturedInput {
            kind: input.kind,
            normalized_path: &input.normalized_path,
            bytes: &input.bytes,
        })
        .collect()
}

fn synthetic_permissions(source_map: &Value) -> Vec<SyntheticPermission> {
    source_map["entries"]
        .as_array()
        .expect("source-map entries")
        .iter()
        .filter(|entry| entry["origin"]["kind"] == "synthetic")
        .map(|entry| {
            let reason = entry["origin"]["reason"]
                .as_str()
                .expect("synthetic reason");
            assert_eq!(reason, "go.control_flow_join");
            SyntheticPermission {
                reference: serde_json::from_value::<SourceReference>(entry["reference"].clone())
                    .expect("synthetic source reference"),
                reason: reason.to_owned(),
            }
        })
        .collect()
}

#[test]
fn active_go_producer_is_successor_only_and_semantically_equal_to_the_complete_baseline() {
    let registry = semantic_registry();
    let active_index = json(format!("{ACTIVE_ROOT}/frontend-index.json"));
    assert_eq!(active_index["schema"], "mpk.go_vir_frontend_corpus.v1");
    assert_eq!(active_index["positive_source_count"], 13);
    assert_eq!(active_index["negative_source_count"], 8);
    assert_eq!(active_index["deterministic_runs"], 2);

    let active_cases = cases_by_id(&active_index);
    assert_eq!(active_cases.len(), 13);

    for (id, active_case) in &active_cases {
        assert_eq!(active_case["frontend_status"], "ir-lowered");
        assert_eq!(
            active_case["selection"]["schema"],
            "mpk.selection.go_function.v0"
        );

        let active_envelope =
            checked_artifact(ACTIVE_ROOT, artifact(active_case, "frontend_envelope"));
        let active_vir = checked_artifact(ACTIVE_ROOT, artifact(active_case, "vir"));
        let active_map = checked_artifact(ACTIVE_ROOT, artifact(active_case, "source_map"));
        let active_manifest = checked_artifact(
            ACTIVE_ROOT,
            artifact(active_case, "source_manifest_frontend"),
        );

        let envelope_value: Value =
            serde_json::from_slice(&active_envelope).expect("successor frontend envelope");
        let vir_value: Value = serde_json::from_slice(&active_vir).expect("successor VIR JSON");
        let map_value: Value = serde_json::from_slice(&active_map).expect("successor map JSON");
        let manifest_value: Value =
            serde_json::from_slice(&active_manifest).expect("successor manifest JSON");
        assert_eq!(envelope_value["schema"], "mpk.frontend.cli.v1");
        assert_eq!(envelope_value["status"], "ir-lowered");
        assert_eq!(
            envelope_value["semantic_context"],
            active_index["semantic_context"]
        );
        assert_eq!(envelope_value["selection"], active_case["selection"]);
        assert!(envelope_value.get("source_language").is_none());
        assert!(envelope_value.get("semantic_profile").is_none());
        assert!(envelope_value.get("semantic_parameters").is_none());
        assert_eq!(envelope_value["ir"]["schema"], SUCCESSOR_VIR_SCHEMA);
        assert_eq!(envelope_value["ir"]["value"], vir_value);
        assert_eq!(envelope_value["source_map"], map_value);
        assert_eq!(envelope_value["source_manifest"], manifest_value);
        assert_eq!(vir_value["schema"], SUCCESSOR_VIR_SCHEMA);
        assert_eq!(map_value["schema"], SUCCESSOR_SOURCE_MAP_SCHEMA);
        assert_eq!(manifest_value["schema"], SUCCESSOR_SOURCE_MANIFEST_SCHEMA);

        let storage = captured_storage(
            active_case["source_root"].as_str().expect("source root"),
            &manifest_value,
        );
        let captured = captured_refs(&storage);
        let active_permissions = synthetic_permissions(&map_value);
        let successor_vir = import_successor_vir_json(&active_vir, &registry)
            .unwrap_or_else(|error| panic!("{id}: successor VIR rejected: {error}"));
        let successor_map = import_successor_source_map_json(
            &active_map,
            SuccessorSourceMapValidationContext {
                registry: &registry,
                vir: &successor_vir,
                captured_inputs: &captured,
                synthetic_permissions: &active_permissions,
            },
        )
        .unwrap_or_else(|error| panic!("{id}: successor source map rejected: {error}"));
        let release_registry: ReleaseRegistryIdentity =
            serde_json::from_value(manifest_value["release_registry"].clone())
                .expect("release-registry identity");
        let successor_manifest = import_successor_source_manifest_json(
            &active_manifest,
            SuccessorSourceManifestStage::Frontend,
            SuccessorSourceManifestValidationContext {
                registry: &registry,
                vir: &successor_vir,
                source_map: &successor_map,
                captured_inputs: &captured,
                expected_release_registry: &release_registry,
            },
        )
        .unwrap_or_else(|error| panic!("{id}: successor source manifest rejected: {error}"));
        assert_eq!(
            successor_manifest.manifest().selection().value(),
            &active_case["selection"]["value"]
        );

        assert!(import_vir_json(&active_vir).is_err());
        assert!(serde_json::from_slice::<SourceMap>(&active_map).is_err());
        assert!(serde_json::from_slice::<SourceManifest>(&active_manifest).is_err());
    }

    let active_negatives = negatives_by_id(&active_index);
    assert_eq!(active_negatives.len(), 8);
    for active_case in active_negatives.values() {
        assert_eq!(active_case["outcome"], "rejected");
        let envelope = checked_artifact(ACTIVE_ROOT, &active_case["artifact"]);
        let value: Value = serde_json::from_slice(&envelope).expect("negative envelope JSON");
        assert_eq!(value["schema"], "mpk.frontend.cli.v1");
        assert_eq!(value["status"], "rejected");
        assert_eq!(value["phase"], active_case["phase"]);
        assert_eq!(value["semantic_context"], active_index["semantic_context"]);
        assert_eq!(value["selection"]["schema"], "mpk.selection.go_function.v0");
        assert!(value["selection"].get("value").is_some());
        assert!(value.get("source_language").is_none());
        assert!(value.get("semantic_profile").is_none());
        assert!(value.get("semantic_parameters").is_none());
    }

    let report = json("develop/migrations/archive/go-successor-semantic-difference-report.json");
    assert_eq!(report["schema"], "mpk.go_successor_semantic_difference.v1");
    assert_eq!(report["active_artifact_family"], "mpk.vir.v0");
    assert_eq!(report["successor_artifact_family"], "mpk.vir.v1");
    assert_eq!(report["summary"]["positive_cases"], 13);
    assert_eq!(report["summary"]["negative_cases"], 8);
    for field in [
        "source_behavior_changes",
        "required_check_changes",
        "vc_input_intent_changes",
        "diagnostic_changes",
    ] {
        assert_eq!(report["summary"][field], 0, "{field} must remain zero");
    }
    for case in report["positive_cases"]
        .as_array()
        .expect("positive difference cases")
    {
        for field in [
            "source_behavior_equal",
            "required_checks_equal",
            "vc_input_intent_equal",
            "diagnostics_equal",
        ] {
            assert_eq!(case[field], true, "{} changed for {}", field, case["id"]);
        }
    }
    for case in report["negative_cases"]
        .as_array()
        .expect("negative difference cases")
    {
        assert_eq!(
            case["diagnostics_equal"], true,
            "diagnostic changed for {}",
            case["id"]
        );
    }
}
