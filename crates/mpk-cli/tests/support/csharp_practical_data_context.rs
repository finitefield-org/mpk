use mpk_vc::csharp_practical_registry::*;
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vir_model::*;
use mpk_vc::{canonical_json_bytes, StrictJsonValue};
use serde_json::{json, Map, Value};
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";
pub fn context(
    b: &ValidatedFoundationBundle,
    root_id: &str,
    source: &[u8],
) -> (PracticalArtifactContext, CapturedInputSet) {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(
        &canonical_successor_registry_transport(&registry_value).unwrap(),
    )
    .unwrap();
    let request = request_fixture(
        context_fixture(&registry_value),
        practical_selection("data", root_id),
    );
    let request = validate_successor_semantic_request(&registry, &canonical(&request)).unwrap();
    let context = bind_practical_artifact_context(&request, b).unwrap();
    let captures = capture_original_inputs(
        &context,
        vec![OriginalInput {
            kind: OriginalInputKind::Source,
            path: "src/Entry.cs".into(),
            bytes: source.to_vec(),
        }],
    )
    .unwrap();
    (context, captures)
}
pub fn context_with_sidecar(
    b: &ValidatedFoundationBundle,
    root_id: &str,
    source: &[u8],
    build: impl FnOnce(&PracticalArtifactContext) -> Vec<u8>,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(
        &canonical_successor_registry_transport(&registry_value).unwrap(),
    )
    .unwrap();
    let mut selection = practical_selection("data", root_id);
    selection["sidecar_paths"] = json!(["contracts/data.json"]);
    selection["selection_sha256"] = json!(csharp_practical_selection_hash(&selection).unwrap());
    let request = request_fixture(context_fixture(&registry_value), selection);
    let request = validate_successor_semantic_request(&registry, &canonical(&request)).unwrap();
    let context = bind_practical_artifact_context(&request, b).unwrap();
    let bytes = build(&context);
    let captures = capture_original_inputs(
        &context,
        vec![
            OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Entry.cs".into(),
                bytes: source.to_vec(),
            },
            OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: "contracts/data.json".into(),
                bytes,
            },
        ],
    )
    .unwrap();
    // Explicit fixture regeneration only; captured bytes are passed through
    // the pinned compiler, and responses are subsequently imported independently.
    if let Ok(path) = std::env::var("MPK_W14_REQUEST_OUTPUT") {
        use std::io::Write;
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap();
        let request = json!({"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if context.source_paths().iter().any(|p|p==e.path()) {"source"} else {"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()});
        writeln!(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap(),
            "{}",
            request
        )
        .unwrap();
    }
    (context, captures)
}

fn candidate_registry() -> Value {
    let profiles = SUCCESSOR_PROFILE_ORDER
        .into_iter()
        .map(candidate_entry)
        .collect::<Vec<_>>();
    let mut registry = json!({
        "schema": SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
        "id": SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
        "revision": SUCCESSOR_CANDIDATE_REVISION,
        "profiles": profiles,
        "registry_sha256": ZERO_SHA256
    });
    registry["registry_sha256"] =
        Value::String(successor_profile_registry_hash(&registry).expect("registry hash"));
    registry
}

fn candidate_entry(profile: SuccessorCompiledSemanticProfile) -> Value {
    let contracts = SUCCESSOR_CONTRACT_FIELDS
        .into_iter()
        .map(|field| {
            (
                field.as_str().to_owned(),
                Value::String(SuccessorProfileContract::new(profile, field).contract_id()),
            )
        })
        .collect::<Map<_, _>>();
    let mut entry = json!({
        "schema": SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA,
        "source_language": profile.source_language(),
        "semantic_profile": profile.semantic_profile(),
        "semantic_parameters_schema": profile.semantic_parameters_schema(),
        "selection_schema": profile.selection_schema(),
        "foundation_descriptor": foundation_descriptor(),
        "contracts": contracts,
        "entry_sha256": ZERO_SHA256
    });
    entry["entry_sha256"] =
        Value::String(successor_profile_entry_hash(&entry).expect("entry hash"));
    entry
}

fn context_fixture(registry: &Value) -> Value {
    let entry = registry["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_profile"] == "mpk.csharp.practical.v1")
        .unwrap();
    json!({
        "schema": SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
        "profile_registry": {
            "schema": registry["schema"],
            "id": registry["id"],
            "revision": registry["revision"],
            "registry_sha256": registry["registry_sha256"]
        },
        "profile_entry_sha256": entry["entry_sha256"],
        "source_language": "csharp",
        "semantic_profile": "mpk.csharp.practical.v1",
        "semantic_parameters": {
            "schema": CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
            "value": {
                "check_overflow_default": true,
                "documentation_mode": "none",
                "language_version": "14.0",
                "nullable_context": "enable",
                "optimization": "release",
                "platform": "x64",
                "pointer_width": 64,
                "preprocessor_symbols": [],
                "source_kind": "regular",
                "target_framework": "net10.0",
                "target_id": "linux-x64",
                "unsafe": false
            }
        },
        "foundation_descriptor": foundation_descriptor()
    })
}

fn foundation_descriptor() -> Value {
    json!({
        "schema": FOUNDATION_DESCRIPTOR_SCHEMA,
        "id": FOUNDATION_DESCRIPTOR_ID,
        "content_sha256": FOUNDATION_DESCRIPTOR_CONTENT_SHA256
    })
}

fn practical_selection(compilation_id: &str, root_id: &str) -> Value {
    let mut selection = json!({
        "schema": CSHARP_PRACTICAL_SELECTION_SCHEMA,
        "compilation_id": compilation_id,
        "source_paths": ["src/Entry.cs"],
        "selected_root_ids": [root_id],
        "sidecar_paths": [],
        "selection_sha256": ZERO_SHA256
    });
    selection["selection_sha256"] =
        Value::String(csharp_practical_selection_hash(&selection).expect("selection hash"));
    selection
}

fn request_fixture(context: Value, selection: Value) -> Value {
    let mut request = json!({
        "schema": SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
        "semantic_context": context,
        "selection": selection,
        "request_sha256": ZERO_SHA256
    });
    request["request_sha256"] =
        Value::String(successor_validated_request_hash(&request).expect("request hash"));
    request
}

fn canonical(value: &Value) -> Vec<u8> {
    canonical_json_bytes(&to_strict(value)).expect("canonical JSON")
}

fn to_strict(value: &Value) -> StrictJsonValue {
    match value {
        Value::Null => StrictJsonValue::Null,
        Value::Bool(value) => StrictJsonValue::Bool(*value),
        Value::Number(value) => StrictJsonValue::Integer(value.as_i64().unwrap()),
        Value::String(value) => StrictJsonValue::String(value.clone()),
        Value::Array(values) => StrictJsonValue::Array(values.iter().map(to_strict).collect()),
        Value::Object(values) => StrictJsonValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), to_strict(value)))
                .collect(),
        ),
    }
}

/// Rebind the original multi-file selection captured by a stage harness.
pub fn replay_context(
    b: &ValidatedFoundationBundle,
    row: &Value,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let registry_value = candidate_registry();
    let registry = validate_candidate_successor_registry(
        &canonical_successor_registry_transport(&registry_value).unwrap(),
    )
    .unwrap();
    let mut selection = practical_selection(
        row["compilation_id"].as_str().unwrap(),
        row["roots"][0].as_str().unwrap(),
    );
    selection["selected_root_ids"] = row["roots"].clone();
    let inputs = row["inputs"].as_array().unwrap();
    selection["source_paths"] = json!(inputs
        .iter()
        .filter(|i| i["kind"] == "source")
        .map(|i| i["path"].clone())
        .collect::<Vec<_>>());
    selection["sidecar_paths"] = json!(inputs
        .iter()
        .filter(|i| i["kind"] == "sidecar")
        .map(|i| i["path"].clone())
        .collect::<Vec<_>>());
    selection["selection_sha256"] = json!(csharp_practical_selection_hash(&selection).unwrap());
    let request = request_fixture(context_fixture(&registry_value), selection);
    let request = validate_successor_semantic_request(&registry, &canonical(&request)).unwrap();
    let context = bind_practical_artifact_context(&request, b).unwrap();
    let captures = capture_original_inputs(
        &context,
        inputs
            .iter()
            .map(|i| OriginalInput {
                kind: if i["kind"] == "source" {
                    OriginalInputKind::Source
                } else {
                    OriginalInputKind::Sidecar
                },
                path: i["path"].as_str().unwrap().into(),
                bytes: i["utf8"].as_str().unwrap().as_bytes().to_vec(),
            })
            .collect(),
    )
    .unwrap();
    (context, captures)
}
