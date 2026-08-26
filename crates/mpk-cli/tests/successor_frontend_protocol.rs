use mpk_cli::frontend_protocol::{
    validate_frontend_process, FrontendProcessFacts, FrontendProtocolCode, FrontendProtocolRequest,
};
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolCode,
    SuccessorFrontendProtocolRequest, SUCCESSOR_FRONTEND_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_inactive_semantic_profile_registry,
    validate_registry_selection_envelope, validate_registry_semantic_context,
    InactiveRegistryRevision, SelectionEnvelope, SemanticContext, ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    successor_contract_hash_value, successor_source_manifest_hash_value,
    successor_source_map_hash_value, successor_vir_hash_value, SUCCESSOR_RELEASE_REGISTRY_ID,
    SUCCESSOR_RELEASE_REGISTRY_SCHEMA, SUCCESSOR_SOURCE_MANIFEST_SCHEMA,
    SUCCESSOR_SOURCE_MAP_SCHEMA, SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::{
    canonical_json_bytes, input_set_hash, parse_strict_json, CapturedInput, InputEntry, InputKind,
    ReleaseRegistryIdentity, StrictJsonLimits,
};
use serde_json::{json, Value};

const REGISTRY_VECTORS: &[u8] =
    include_bytes!("../../../develop/specs/vectors/semantic-profile-registry-v1.json");
const ACTIVE_ENVELOPE: &[u8] =
    include_bytes!("../../../fixtures/vir-go/frontend/basic-arith/frontend-envelope.json");
const GO_MOD: &[u8] = include_bytes!("../../../fixtures/go-basic/go.mod");
const GO_SOURCE: &[u8] = include_bytes!("../../../fixtures/go-basic/positive/arith/arith.go");
const ZERO_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

struct ProtocolFixture {
    registry: ValidatedSemanticProfileRegistry,
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    release_registry: ReleaseRegistryIdentity,
    envelope: Value,
}

impl ProtocolFixture {
    fn request<'a>(
        &'a self,
        captured_inputs: &'a [CapturedInput<'a>],
    ) -> SuccessorFrontendProtocolRequest<'a> {
        SuccessorFrontendProtocolRequest {
            registry: &self.registry,
            semantic_context: &self.semantic_context,
            selection: &self.selection,
            release_registry: &self.release_registry,
            captured_inputs,
            synthetic_permissions: &[],
        }
    }
}

fn load(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("checked-in JSON fixture")
}

fn canonical(value: &Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).expect("JSON value serializes");
    let strict = parse_strict_json(
        &encoded,
        StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576),
    )
    .expect("test value is strict JSON");
    canonical_json_bytes(&strict).expect("test value canonicalizes")
}

fn canonical_line(value: &Value) -> Vec<u8> {
    let mut bytes = canonical(value);
    bytes.push(b'\n');
    bytes
}

fn captured_inputs() -> [CapturedInput<'static>; 3] {
    [
        CapturedInput {
            kind: InputKind::BuildManifest,
            normalized_path: "go.mod",
            bytes: GO_MOD,
        },
        CapturedInput {
            kind: InputKind::Lockfile,
            normalized_path: "go.sum",
            bytes: b"",
        },
        CapturedInput {
            kind: InputKind::Source,
            normalized_path: "positive/arith/arith.go",
            bytes: GO_SOURCE,
        },
    ]
}

fn protocol_fixture() -> ProtocolFixture {
    let vectors = load(REGISTRY_VECTORS);
    let registry_value = vectors["fixtures"]["base_registry"].clone();
    let registry = validate_inactive_semantic_profile_registry(
        &canonical_registry_transport(&registry_value).expect("canonical registry transport"),
        InactiveRegistryRevision::Revision1,
    )
    .expect("frozen revision-1 registry validates");
    let context_value = vectors["fixtures"]["go_request"]["semantic_context"].clone();

    let mut envelope = load(ACTIVE_ENVELOPE);
    let raw_selection = envelope["selection"].clone();
    let selection_value = json!({
        "schema": "mpk.selection.go_function.v0",
        "value": raw_selection,
    });
    envelope["schema"] = Value::String(SUCCESSOR_FRONTEND_SCHEMA.into());
    envelope
        .as_object_mut()
        .expect("frontend root")
        .remove("source_language");
    envelope
        .as_object_mut()
        .expect("frontend root")
        .remove("semantic_profile");
    envelope
        .as_object_mut()
        .expect("frontend root")
        .remove("semantic_parameters");
    envelope["semantic_context"] = context_value.clone();
    envelope["selection"] = selection_value.clone();

    let vir = &mut envelope["ir"]["value"];
    vir["schema"] = Value::String(SUCCESSOR_VIR_SCHEMA.into());
    vir.as_object_mut()
        .expect("VIR root")
        .remove("source_language");
    vir.as_object_mut()
        .expect("VIR root")
        .remove("semantic_profile");
    vir.as_object_mut()
        .expect("VIR root")
        .remove("semantic_parameters");
    vir["semantic_context"] = context_value.clone();
    for unit in vir["units"].as_array_mut().expect("VIR units") {
        for function in unit["functions"].as_array_mut().expect("VIR functions") {
            let contract = &mut function["contracts"];
            contract
                .as_object_mut()
                .expect("VIR contract")
                .remove("semantic_profile");
            contract
                .as_object_mut()
                .expect("VIR contract")
                .remove("semantic_parameters");
            contract["semantic_context"] = context_value.clone();
            contract["contract_hash"] = Value::String(ZERO_SHA256.into());
            contract["contract_hash"] = Value::String(
                successor_contract_hash_value(contract)
                    .expect("successor contract hash")
                    .as_str()
                    .into(),
            );
        }
    }
    vir["vir_hash"] = Value::String(ZERO_SHA256.into());
    vir["vir_hash"] = Value::String(
        successor_vir_hash_value(vir)
            .expect("successor VIR hash")
            .as_str()
            .into(),
    );
    let vir_hash = vir["vir_hash"].clone();
    envelope["ir"]["schema"] = Value::String(SUCCESSOR_VIR_SCHEMA.into());
    envelope["ir"]["sha256"] = vir_hash.clone();

    let source_map = &mut envelope["source_map"];
    source_map["schema"] = Value::String(SUCCESSOR_SOURCE_MAP_SCHEMA.into());
    source_map["semantic_context"] = context_value.clone();
    source_map["source_ir_schema"] = Value::String(SUCCESSOR_VIR_SCHEMA.into());
    source_map["source_ir_hash"] = vir_hash.clone();
    source_map["source_map_hash"] = Value::String(ZERO_SHA256.into());
    source_map["source_map_hash"] = Value::String(
        successor_source_map_hash_value(source_map)
            .expect("successor source-map hash")
            .as_str()
            .into(),
    );
    let source_map_hash = source_map["source_map_hash"].clone();

    let source_manifest = &mut envelope["source_manifest"];
    source_manifest["schema"] = Value::String(SUCCESSOR_SOURCE_MANIFEST_SCHEMA.into());
    source_manifest
        .as_object_mut()
        .expect("source-manifest root")
        .remove("source_language");
    source_manifest
        .as_object_mut()
        .expect("source-manifest root")
        .remove("semantic_profile");
    source_manifest
        .as_object_mut()
        .expect("source-manifest root")
        .remove("semantic_parameters");
    source_manifest["semantic_context"] = context_value.clone();
    source_manifest["selection"] = selection_value.clone();
    source_manifest["release_registry"] = json!({
        "schema": SUCCESSOR_RELEASE_REGISTRY_SCHEMA,
        "id": SUCCESSOR_RELEASE_REGISTRY_ID,
        "registry_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    });
    source_manifest["target"]
        .as_object_mut()
        .expect("target")
        .remove("language_configuration");
    source_manifest["vir_hash"] = vir_hash;
    source_manifest["source_map_hash"] = source_map_hash;
    let inputs: Vec<InputEntry> =
        serde_json::from_value(source_manifest["inputs"].clone()).expect("common manifest inputs");
    source_manifest["input_set_hash"] = Value::String(
        input_set_hash(&inputs)
            .expect("input-set hash")
            .as_str()
            .into(),
    );
    source_manifest["source_manifest_hash"] = Value::String(ZERO_SHA256.into());
    source_manifest["source_manifest_hash"] = Value::String(
        successor_source_manifest_hash_value(source_manifest)
            .expect("successor source-manifest hash")
            .as_str()
            .into(),
    );

    let semantic_context = validate_registry_semantic_context(&registry, &context_value)
        .expect("successor semantic context");
    let selection =
        validate_registry_selection_envelope(&registry, &semantic_context, &selection_value)
            .expect("successor selection envelope");
    let release_registry =
        serde_json::from_value(envelope["source_manifest"]["release_registry"].clone())
            .expect("release-registry identity");
    ProtocolFixture {
        registry,
        semantic_context,
        selection,
        release_registry,
        envelope,
    }
}

fn process<'a>(stdout: &'a [u8], exit_code: i32) -> FrontendProcessFacts<'a> {
    FrontendProcessFacts {
        exit_code: Some(exit_code),
        signaled: false,
        stdout,
        stderr_observed_bytes: 0,
    }
}

#[test]
fn successor_success_envelope_is_canonical_and_artifact_linked() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();
    let stdout = canonical_line(&fixture.envelope);
    let accepted =
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0))
            .expect("successor success envelope validates");
    assert_eq!(accepted.status(), "ir-lowered");
    assert_eq!(accepted.phase(), "emission");
    assert_eq!(accepted.semantic_context(), &fixture.semantic_context);
    assert_eq!(accepted.selection(), &fixture.selection);
    assert_eq!(accepted.canonical_bytes(), stdout);
    let artifacts = accepted.artifacts().expect("success artifacts");
    assert_eq!(
        artifacts.vir().hash().as_str(),
        fixture.envelope["ir"]["sha256"]
    );
    assert_eq!(
        artifacts.source_map().hash().as_str(),
        fixture.envelope["source_manifest"]["source_map_hash"]
    );
    assert_eq!(
        artifacts.source_manifest().manifest().selection(),
        &fixture.selection
    );
}

#[test]
fn successor_error_envelopes_keep_the_typed_request_identity() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();
    for (status, phase, exit, rejected, diagnostics) in [
        (
            "rejected",
            "subset",
            3,
            json!([{"code":"CSHARP_SUBSET","message":"unsupported construct"}]),
            json!([]),
        ),
        (
            "source-error",
            "source",
            4,
            json!([]),
            json!([{"code":"CSHARP_SOURCE","message":"invalid source"}]),
        ),
        (
            "frontend-error",
            "metadata",
            1,
            json!([]),
            json!([{"code":"CSHARP_FRONTEND","message":"frontend failed"}]),
        ),
    ] {
        let envelope = json!({
            "schema": SUCCESSOR_FRONTEND_SCHEMA,
            "status": status,
            "phase": phase,
            "semantic_context": fixture.envelope["semantic_context"].clone(),
            "selection": fixture.envelope["selection"].clone(),
            "rejected_features": rejected,
            "diagnostics": diagnostics,
        });
        let stdout = canonical_line(&envelope);
        let accepted =
            validate_successor_frontend_process(fixture.request(&captured), process(&stdout, exit))
                .expect("successor error envelope validates");
        assert_eq!(accepted.status(), status);
        assert!(accepted.artifacts().is_none());
    }
}

#[test]
fn protocol_and_artifact_identity_mismatches_fail_at_the_staged_boundary() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();

    let mut wrong_context = fixture.envelope.clone();
    wrong_context["semantic_context"]["semantic_parameters"]["value"]["pointer_width"] = json!(32);
    let stdout = canonical_line(&wrong_context);
    let error =
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0))
            .expect_err("root semantic-context mismatch rejects");
    assert_eq!(
        error.code(),
        SuccessorFrontendProtocolCode::ProtocolIdentityMismatch
    );

    let mut wrong_selection = fixture.envelope.clone();
    wrong_selection["selection"]["value"]["function"] = Value::String(
        "github.com/finitefield-org/mpk/fixtures/go-basic/positive/arith.BoolAnd".into(),
    );
    let stdout = canonical_line(&wrong_selection);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0),)
            .expect_err("root selection mismatch rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolIdentityMismatch
    );

    let mut wrong_wrapper = fixture.envelope.clone();
    wrong_wrapper["ir"]["sha256"] = Value::String(ZERO_SHA256.into());
    let stdout = canonical_line(&wrong_wrapper);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0),)
            .expect_err("IR wrapper mismatch rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolArtifactMismatch
    );

    let mut wrong_manifest_selection = fixture.envelope.clone();
    wrong_manifest_selection["source_manifest"]["selection"]["value"]["function"] = Value::String(
        "github.com/finitefield-org/mpk/fixtures/go-basic/positive/arith.BoolAnd".into(),
    );
    wrong_manifest_selection["source_manifest"]["source_manifest_hash"] = Value::String(
        successor_source_manifest_hash_value(&wrong_manifest_selection["source_manifest"])
            .expect("mutated manifest rehashes")
            .as_str()
            .into(),
    );
    let stdout = canonical_line(&wrong_manifest_selection);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0),)
            .expect_err("manifest selection mismatch rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolArtifactMismatch
    );

    let mut noncanonical = serde_json::to_vec_pretty(&fixture.envelope).expect("pretty JSON");
    noncanonical.push(b'\n');
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&noncanonical, 0),)
            .expect_err("noncanonical transport rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolNoncanonical
    );
    let stdout = canonical_line(&fixture.envelope);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 1),)
            .expect_err("status/exit mismatch rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolStatusExit
    );

    let mut leaked_path = fixture.envelope.clone();
    leaked_path["source_manifest"]["frontend"]["version"] =
        Value::String("failed while reading /tmp/private-source.cs".into());
    let stdout = canonical_line(&leaked_path);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0),)
            .expect_err("public successor transport rejects an absolute path")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolShape
    );
}

#[test]
fn current_and_successor_frontend_parsers_reject_each_other() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();
    let mut active_stdout = ACTIVE_ENVELOPE.to_vec();
    active_stdout.push(b'\n');
    assert_eq!(
        validate_successor_frontend_process(
            fixture.request(&captured),
            process(&active_stdout, 0),
        )
        .expect_err("v1 parser rejects the active v0 envelope")
        .code(),
        SuccessorFrontendProtocolCode::ProtocolShape
    );

    let successor_stdout = canonical_line(&fixture.envelope);
    let active_parameters = json!({"pointer_width":64,"target_id":"linux/amd64"});
    let active_selection = fixture.envelope["selection"]["value"].clone();
    let error = validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "go",
            semantic_profile: "mpk.go.fixed.v0",
            semantic_parameters: &active_parameters,
            selection: &active_selection,
            release_registry: None,
            captured_inputs: &captured,
        },
        process(&successor_stdout, 0),
    )
    .expect_err("active v0 parser rejects the v1 envelope");
    assert_eq!(error.code(), FrontendProtocolCode::ProtocolShape);

    let mut dual_identity = fixture.envelope.clone();
    dual_identity["source_language"] = Value::String("go".into());
    let stdout = canonical_line(&dual_identity);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 0),)
            .expect_err("a flat compatibility identity is an unknown field")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolShape
    );
}
