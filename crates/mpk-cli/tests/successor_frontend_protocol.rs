use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolCode,
    SuccessorFrontendProtocolRequest, SUCCESSOR_FRONTEND_SCHEMA,
};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_registry_semantic_context,
    validate_semantic_profile_registry, RegistryRevision, SelectionEnvelope, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::successor_source_manifest_hash_value;
use mpk_vc::{
    canonical_json_bytes, parse_strict_json, CapturedInput, InputKind, ReleaseRegistryIdentity,
    StrictJsonLimits,
};
use serde_json::{json, Value};

const SEMANTIC_PROFILE_REGISTRY: &[u8] =
    include_bytes!("../../../release/bundles/semantic-profile-registry.json");
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
    let registry =
        validate_semantic_profile_registry(SEMANTIC_PROFILE_REGISTRY, RegistryRevision::Revision2)
            .expect("installed revision-2 registry validates");
    let envelope = load(ACTIVE_ENVELOPE);
    let context_value = envelope["semantic_context"].clone();
    let selection_value = envelope["selection"].clone();

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
            json!([{"code":"CSHARP_SUBSET_DECLARATION","message":"C# source is outside the frozen profile"}]),
            json!([]),
        ),
        (
            "source-error",
            "metadata",
            4,
            json!([]),
            json!([{"code":"CSHARP_SOURCE_DIAGNOSTIC","message":"C# compiler diagnostic CS0103"}]),
        ),
        (
            "frontend-error",
            "metadata",
            1,
            json!([]),
            json!([{"code":"CSHARP_TOOLCHAIN_OPTIONS","message":"C# frontend failed closed"}]),
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
fn csharp_diagnostic_codes_messages_and_phases_are_closed() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();
    let base = json!({
        "schema": SUCCESSOR_FRONTEND_SCHEMA,
        "status": "rejected",
        "phase": "subset",
        "semantic_context": fixture.envelope["semantic_context"].clone(),
        "selection": fixture.envelope["selection"].clone(),
        "rejected_features": [{
            "code": "CSHARP_SUBSET_DECLARATION",
            "message": "C# source is outside the frozen profile"
        }],
        "diagnostics": [],
    });

    for (label, mutation) in [
        ("unknown code", ("code", "CSHARP_UNKNOWN")),
        ("unknown limit", ("code", "CSHARP_LIMIT_UNKNOWN")),
        (
            "host message",
            ("message", "unsupported token from /tmp/input.cs"),
        ),
    ] {
        let mut envelope = base.clone();
        envelope["rejected_features"][0][mutation.0] = Value::String(mutation.1.into());
        let stdout = canonical_line(&envelope);
        assert_eq!(
            validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 3))
                .expect_err(label)
                .code(),
            SuccessorFrontendProtocolCode::ProtocolShape,
            "{label}"
        );
    }

    let mut wrong_phase = base.clone();
    wrong_phase["phase"] = Value::String("typecheck".into());
    let stdout = canonical_line(&wrong_phase);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 3))
            .expect_err("fixed C# diagnostic phase rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolShape
    );

    let mut wrong_compiler_id = base;
    wrong_compiler_id["status"] = Value::String("source-error".into());
    wrong_compiler_id["phase"] = Value::String("metadata".into());
    wrong_compiler_id["diagnostics"] = json!([{
        "code": "CSHARP_SOURCE_DIAGNOSTIC",
        "message": "C# compiler diagnostic BAD001"
    }]);
    wrong_compiler_id["rejected_features"] = json!([]);
    let stdout = canonical_line(&wrong_compiler_id);
    assert_eq!(
        validate_successor_frontend_process(fixture.request(&captured), process(&stdout, 4))
            .expect_err("non-Roslyn compiler ID rejects")
            .code(),
        SuccessorFrontendProtocolCode::ProtocolShape
    );
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
fn successor_frontend_parser_rejects_dual_identity() {
    let fixture = protocol_fixture();
    let captured = captured_inputs();
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
