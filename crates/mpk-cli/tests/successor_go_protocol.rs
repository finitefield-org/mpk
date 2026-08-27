use mpk_cli::frontend_protocol::{
    validate_frontend_process, FrontendProcessFacts, FrontendProtocolRequest,
};
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_vc::semantic_profile_registry::{
    validate_inactive_semantic_profile_registry, validate_registry_selection_envelope,
    validate_registry_semantic_context, InactiveRegistryRevision, ValidatedSemanticProfileRegistry,
};
use mpk_vc::{
    CapturedInput, InputKind, ReleaseRegistryIdentity, SourceReference, SyntheticPermission,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const STAGING_ROOT: &str = "develop/migrations/csharp-02-staging/go";
const ACTIVE_ROOT: &str = "fixtures/vir-go";

struct OwnedCapturedInput {
    kind: InputKind,
    path: String,
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
    serde_json::from_slice(&read(relative)).expect("checked-in JSON")
}

fn semantic_registry() -> ValidatedSemanticProfileRegistry {
    validate_inactive_semantic_profile_registry(
        &read("develop/migrations/csharp-02-staging/semantic-profile-registry.json"),
        InactiveRegistryRevision::Revision2,
    )
    .expect("revision-2 semantic registry")
}

fn kind(value: &Value) -> InputKind {
    match value.as_str().expect("input kind") {
        "source" => InputKind::Source,
        "contract" => InputKind::Contract,
        "build_manifest" => InputKind::BuildManifest,
        "lockfile" => InputKind::Lockfile,
        other => panic!("unknown input kind {other}"),
    }
}

fn captured_storage(source_root: &str, manifest: &Value) -> Vec<OwnedCapturedInput> {
    manifest["inputs"]
        .as_array()
        .expect("manifest inputs")
        .iter()
        .map(|entry| {
            let kind = kind(&entry["kind"]);
            let path = entry["normalized_path"]
                .as_str()
                .expect("input path")
                .to_owned();
            let disk_path = repository_root().join(source_root).join(&path);
            let bytes = match fs::read(&disk_path) {
                Ok(bytes) => bytes,
                Err(error)
                    if kind == InputKind::Lockfile
                        && error.kind() == std::io::ErrorKind::NotFound =>
                {
                    Vec::new()
                }
                Err(error) => panic!("read {}: {error}", disk_path.display()),
            };
            OwnedCapturedInput { kind, path, bytes }
        })
        .collect()
}

fn captured_refs(storage: &[OwnedCapturedInput]) -> Vec<CapturedInput<'_>> {
    storage
        .iter()
        .map(|input| CapturedInput {
            kind: input.kind,
            normalized_path: &input.path,
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
        .map(|entry| SyntheticPermission {
            reference: serde_json::from_value::<SourceReference>(entry["reference"].clone())
                .expect("synthetic reference"),
            reason: entry["origin"]["reason"]
                .as_str()
                .expect("synthetic reason")
                .to_owned(),
        })
        .collect()
}

fn process_line(mut bytes: Vec<u8>, exit_code: i32) -> (Vec<u8>, i32) {
    assert!(!bytes.ends_with(b"\n"), "fixtures omit process framing LF");
    bytes.push(b'\n');
    (bytes, exit_code)
}

#[test]
fn staged_go_envelopes_use_only_the_successor_protocol_family() {
    let registry = semantic_registry();
    let index = json(format!("{STAGING_ROOT}/frontend-index.json"));

    for case in index["cases"].as_array().expect("positive cases") {
        let id = case["id"].as_str().expect("case ID");
        let envelope_path = case["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .find(|artifact| artifact["kind"] == "frontend_envelope")
            .expect("frontend envelope")["path"]
            .as_str()
            .expect("envelope path");
        let envelope_bytes = read(Path::new(STAGING_ROOT).join(envelope_path));
        let envelope: Value =
            serde_json::from_slice(&envelope_bytes).expect("successor envelope JSON");
        let semantic_context =
            validate_registry_semantic_context(&registry, &envelope["semantic_context"])
                .unwrap_or_else(|error| panic!("{id}: semantic context rejected: {error}"));
        let selection = validate_registry_selection_envelope(
            &registry,
            &semantic_context,
            &envelope["selection"],
        )
        .unwrap_or_else(|error| panic!("{id}: selection rejected: {error}"));
        let release_registry: ReleaseRegistryIdentity =
            serde_json::from_value(envelope["source_manifest"]["release_registry"].clone())
                .expect("release-registry identity");
        let storage = captured_storage(
            case["source_root"].as_str().expect("source root"),
            &envelope["source_manifest"],
        );
        let captured = captured_refs(&storage);
        let permissions = synthetic_permissions(&envelope["source_map"]);
        let (stdout, exit_code) = process_line(envelope_bytes, 0);
        let accepted = validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &registry,
                semantic_context: &semantic_context,
                selection: &selection,
                release_registry: &release_registry,
                captured_inputs: &captured,
                synthetic_permissions: &permissions,
            },
            FrontendProcessFacts {
                exit_code: Some(exit_code),
                signaled: false,
                stdout: &stdout,
                stderr_observed_bytes: 0,
            },
        )
        .unwrap_or_else(|error| panic!("{id}: successor protocol rejected: {error}"));
        assert_eq!(accepted.status(), "ir-lowered");
        assert_eq!(accepted.phase(), "emission");
        assert!(accepted.artifacts().is_some());

        let active_envelope = read(format!(
            "{ACTIVE_ROOT}/frontend/{id}/frontend-envelope.json"
        ));
        let active_value: Value =
            serde_json::from_slice(&active_envelope).expect("active envelope JSON");
        let (active_stdout, active_exit) = process_line(active_envelope, 0);
        assert!(validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &registry,
                semantic_context: &semantic_context,
                selection: &selection,
                release_registry: &release_registry,
                captured_inputs: &captured,
                synthetic_permissions: &permissions,
            },
            FrontendProcessFacts {
                exit_code: Some(active_exit),
                signaled: false,
                stdout: &active_stdout,
                stderr_observed_bytes: 0,
            },
        )
        .is_err());
        assert!(validate_frontend_process(
            FrontendProtocolRequest {
                source_language: active_value["source_language"]
                    .as_str()
                    .expect("active source language"),
                semantic_profile: active_value["semantic_profile"]
                    .as_str()
                    .expect("active semantic profile"),
                semantic_parameters: &active_value["semantic_parameters"],
                selection: &active_value["selection"],
                release_registry: None,
                captured_inputs: &captured,
            },
            FrontendProcessFacts {
                exit_code: Some(exit_code),
                signaled: false,
                stdout: &stdout,
                stderr_observed_bytes: 0,
            },
        )
        .is_err());
    }

    let release_registry: ReleaseRegistryIdentity =
        serde_json::from_value(index["release_registry"].clone()).expect("release identity");
    for case in index["negative_cases"].as_array().expect("negative cases") {
        let id = case["id"].as_str().expect("negative case ID");
        let envelope_bytes = read(
            Path::new(STAGING_ROOT).join(
                case["artifact"]["path"]
                    .as_str()
                    .expect("negative envelope path"),
            ),
        );
        let envelope: Value =
            serde_json::from_slice(&envelope_bytes).expect("negative envelope JSON");
        let semantic_context =
            validate_registry_semantic_context(&registry, &envelope["semantic_context"])
                .expect("negative semantic context");
        let selection = validate_registry_selection_envelope(
            &registry,
            &semantic_context,
            &envelope["selection"],
        )
        .expect("negative selection");
        let (stdout, exit_code) = process_line(envelope_bytes, 3);
        let accepted = validate_successor_frontend_process(
            SuccessorFrontendProtocolRequest {
                registry: &registry,
                semantic_context: &semantic_context,
                selection: &selection,
                release_registry: &release_registry,
                captured_inputs: &[],
                synthetic_permissions: &[],
            },
            FrontendProcessFacts {
                exit_code: Some(exit_code),
                signaled: false,
                stdout: &stdout,
                stderr_observed_bytes: 0,
            },
        )
        .unwrap_or_else(|error| panic!("{id}: successor rejection rejected: {error}"));
        assert_eq!(accepted.status(), "rejected");
        assert_eq!(accepted.phase(), case["phase"]);
        assert!(accepted.artifacts().is_none());
        assert!(validate_frontend_process(
            FrontendProtocolRequest {
                source_language: "go",
                semantic_profile: "mpk.go.fixed.v0",
                semantic_parameters: &envelope["semantic_context"]["semantic_parameters"]["value"],
                selection: &envelope["selection"]["value"],
                release_registry: None,
                captured_inputs: &[],
            },
            FrontendProcessFacts {
                exit_code: Some(exit_code),
                signaled: false,
                stdout: &stdout,
                stderr_observed_bytes: 0,
            },
        )
        .is_err());
    }
}
