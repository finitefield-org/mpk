use std::collections::BTreeSet;

use mpk_cli::frontend_protocol::{
    validate_frontend_process, validate_frontend_protocol_limit, AcceptedFrontendEnvelope,
    FrontendProcessFacts, FrontendProtocolCode, FrontendProtocolError, FrontendProtocolRequest,
    FRONTEND_STDERR_BYTES_MAX,
};
use mpk_vc::{validate_release_limit, ReleaseRegistryErrorCode, ReleaseValidationPhase};
use serde_json::{json, Value};

struct ReleaseLimitCase {
    id: &'static str,
    maximum: u64,
    phase: ReleaseValidationPhase,
}

const RELEASE_LIMITS: [ReleaseLimitCase; 14] = [
    ReleaseLimitCase {
        id: "registry_canonical_bytes",
        maximum: 67_108_864,
        phase: ReleaseValidationPhase::RegistryHash,
    },
    ReleaseLimitCase {
        id: "registry_transport_bytes",
        maximum: 67_108_865,
        phase: ReleaseValidationPhase::Transport,
    },
    ReleaseLimitCase {
        id: "json_nesting",
        maximum: 256,
        phase: ReleaseValidationPhase::Transport,
    },
    ReleaseLimitCase {
        id: "string_bytes",
        maximum: 1_048_576,
        phase: ReleaseValidationPhase::Transport,
    },
    ReleaseLimitCase {
        id: "bundle_descriptors",
        maximum: 1_024,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "tuples",
        maximum: 4_096,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "execution_host_profiles",
        maximum: 256,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "native_runtime_layout_profiles",
        maximum: 256,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "components",
        maximum: 8_192,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "serialized_inventory_entries",
        maximum: 262_144,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "unique_bundle_files",
        maximum: 262_144,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "portable_path_bytes",
        maximum: 1_024,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "bundle_file_bytes",
        maximum: 4_294_967_296,
        phase: ReleaseValidationPhase::Scalar,
    },
    ReleaseLimitCase {
        id: "bundle_declared_bytes",
        maximum: 34_359_738_368,
        phase: ReleaseValidationPhase::Scalar,
    },
];

struct ProtocolLimitCase {
    family: &'static str,
    maximum: u64,
    above_code: FrontendProtocolCode,
}

const PROTOCOL_LIMITS: [ProtocolLimitCase; 7] = [
    ProtocolLimitCase {
        family: "stdout_transport_bytes",
        maximum: 268_435_456,
        above_code: FrontendProtocolCode::ProtocolLimit,
    },
    ProtocolLimitCase {
        family: "stderr_observed_bytes",
        maximum: 2_097_152,
        above_code: FrontendProtocolCode::ProtocolLimit,
    },
    ProtocolLimitCase {
        family: "json_nesting",
        maximum: 256,
        above_code: FrontendProtocolCode::ProtocolMalformed,
    },
    ProtocolLimitCase {
        family: "string_bytes",
        maximum: 1_048_576,
        above_code: FrontendProtocolCode::ProtocolMalformed,
    },
    ProtocolLimitCase {
        family: "issues",
        maximum: 1_024,
        above_code: FrontendProtocolCode::ProtocolLimit,
    },
    ProtocolLimitCase {
        family: "issue_message_bytes",
        maximum: 4_096,
        above_code: FrontendProtocolCode::ProtocolShape,
    },
    ProtocolLimitCase {
        family: "combined_issue_message_bytes",
        maximum: 2_097_152,
        above_code: FrontendProtocolCode::ProtocolLimit,
    },
];

#[test]
fn release_registry_limits_have_exact_closed_boundaries() {
    let mut ids = BTreeSet::new();

    assert_eq!(RELEASE_LIMITS.len(), 14);
    for case in &RELEASE_LIMITS {
        assert!(
            ids.insert(case.id),
            "duplicate release limit ID {}",
            case.id
        );
        let below = case.maximum.checked_sub(1).expect("maximum is positive");
        let above = case.maximum.checked_add(1).expect("above value fits");

        validate_release_limit(case.id, below).expect("below maximum accepts");
        validate_release_limit(case.id, case.maximum).expect("inclusive maximum accepts");

        let error = validate_release_limit(case.id, above).expect_err("above maximum rejects");
        assert_eq!(error.phase(), case.phase, "{} phase", case.id);
        assert_eq!(error.code(), ReleaseRegistryErrorCode::Limit, "{}", case.id);
        assert_eq!(error.code().as_str(), "FRONTEND_REGISTRY_LIMIT");
    }

    let unknown = validate_release_limit("registry_entries", 0)
        .expect_err("unregistered release limit rejects");
    assert_eq!(unknown.phase(), ReleaseValidationPhase::Scalar);
    assert_eq!(unknown.code(), ReleaseRegistryErrorCode::Invalid);
}

#[test]
fn frontend_protocol_limit_families_have_derived_boundaries() {
    let mut families = BTreeSet::new();

    assert_eq!(PROTOCOL_LIMITS.len(), 7);
    for case in &PROTOCOL_LIMITS {
        assert!(
            families.insert(case.family),
            "duplicate frontend limit family {}",
            case.family
        );
        let below = case.maximum.checked_sub(1).expect("maximum is positive");
        let above = case.maximum.checked_add(1).expect("above value fits");

        validate_frontend_protocol_limit(case.family, below).expect("below maximum accepts");
        validate_frontend_protocol_limit(case.family, case.maximum)
            .expect("inclusive maximum accepts");

        let error = validate_frontend_protocol_limit(case.family, above)
            .expect_err("above maximum rejects");
        assert_eq!(error.code(), case.above_code, "{} code", case.family);
    }

    let unknown = validate_frontend_protocol_limit("json_nodes", 0)
        .expect_err("unregistered protocol limit rejects");
    assert_eq!(unknown.code(), FrontendProtocolCode::ProtocolShape);
}

#[test]
fn stream_limit_precedes_kill_and_non_success_has_no_artifacts() {
    let accepted = validate_rejected(
        vec![issue("captured input is unsupported")],
        Vec::new(),
        Some(3),
        false,
        FRONTEND_STDERR_BYTES_MAX,
    )
    .expect("stderr at its inclusive maximum accepts");
    assert_non_success_without_artifacts(&accepted);

    let limited = validate_rejected(
        vec![issue("captured input is unsupported")],
        Vec::new(),
        None,
        true,
        FRONTEND_STDERR_BYTES_MAX + 1,
    )
    .expect_err("stream limit owns precedence over the resulting kill");
    assert_eq!(limited.code(), FrontendProtocolCode::ProtocolLimit);

    for (exit_code, signaled) in [(Some(3), true), (None, false)] {
        let killed = validate_rejected(
            vec![issue("captured input is unsupported")],
            Vec::new(),
            exit_code,
            signaled,
            FRONTEND_STDERR_BYTES_MAX,
        )
        .expect_err("an operational kill never returns an accepted envelope");
        assert_eq!(killed.code(), FrontendProtocolCode::ProcessKilled);
    }
}

#[test]
fn normalized_issue_count_and_message_boundaries_preserve_status() {
    let at_count = validate_rejected(repeated_issues(1_024, "x"), Vec::new(), Some(3), false, 0)
        .expect("1,024 normalized issues accept");
    assert_non_success_without_artifacts(&at_count);

    let raw_count_error =
        validate_rejected(repeated_issues(1_025, "x"), Vec::new(), Some(3), false, 0)
            .expect_err("an untruncated issue list rejects");
    assert_eq!(raw_count_error.code(), FrontendProtocolCode::ProtocolLimit);

    let normalized_count = validate_rejected(
        repeated_issues(1_023, "x"),
        vec![truncation_marker(2)],
        Some(3),
        false,
        0,
    )
    .expect("the longest fitting prefix plus marker accepts");
    assert_non_success_without_artifacts(&normalized_count);

    let raw_message_error = validate_rejected(
        vec![issue(&"x".repeat(4_097))],
        Vec::new(),
        Some(3),
        false,
        0,
    )
    .expect_err("an incoming untruncated message is shape-invalid");
    assert_eq!(
        raw_message_error.code(),
        FrontendProtocolCode::ProtocolShape
    );

    let suffix = " [truncated]";
    let normalized_message = format!("{}{}", "x".repeat(4_096 - suffix.len()), suffix);
    assert_eq!(normalized_message.len(), 4_096);
    let normalized_message = validate_rejected(
        vec![issue(&normalized_message)],
        Vec::new(),
        Some(3),
        false,
        0,
    )
    .expect("a producer-truncated message at the maximum accepts");
    assert_non_success_without_artifacts(&normalized_message);
}

#[test]
fn normalized_combined_message_boundary_preserves_status() {
    let message = "x".repeat(4_096);
    let at = validate_rejected(
        repeated_issues(512, &message),
        Vec::new(),
        Some(3),
        false,
        0,
    )
    .expect("combined message bytes at 2 MiB accept");
    assert_non_success_without_artifacts(&at);

    let above = validate_rejected(
        repeated_issues(513, &message),
        Vec::new(),
        Some(3),
        false,
        0,
    )
    .expect_err("an untruncated combined message list rejects");
    assert_eq!(above.code(), FrontendProtocolCode::ProtocolLimit);

    let normalized = validate_rejected(
        repeated_issues(511, &message),
        vec![truncation_marker(2)],
        Some(3),
        false,
        0,
    )
    .expect("the combined-message prefix plus marker accepts");
    assert_non_success_without_artifacts(&normalized);
}

#[test]
fn strict_json_limit_failures_remain_malformed() {
    let depth = 257;
    let mut nested = Vec::with_capacity(depth * 2 + 2);
    nested.extend(vec![b'['; depth]);
    nested.push(b'0');
    nested.extend(vec![b']'; depth]);
    nested.push(b'\n');
    let nesting = validate_raw_transport(&nested, Some(1), false, 0)
        .expect_err("JSON nesting above the limit rejects");
    assert_eq!(nesting.code(), FrontendProtocolCode::ProtocolMalformed);

    let mut string = Vec::with_capacity(1_048_580);
    string.push(b'"');
    string.extend(vec![b'x'; 1_048_577]);
    string.extend_from_slice(b"\"\n");
    let oversized_string = validate_raw_transport(&string, Some(1), false, 0)
        .expect_err("decoded string above the limit rejects");
    assert_eq!(
        oversized_string.code(),
        FrontendProtocolCode::ProtocolMalformed
    );
}

#[test]
fn shape_precedes_noncanonical_transport_and_lexical_errors_precede_issue_limits() {
    let parameters = json!({"pointer_width":64,"target_id":"linux/amd64"});
    let selection =
        json!({"function":"example.com/mpk/vector.Identity","package":"example.com/mpk/vector"});
    let malformed_shape = json!({
        "diagnostics": [],
        "phase": "capture",
        "rejected_features": repeated_issues(1_025, "captured input is unsupported"),
        "schema": "mpk.frontend.cli.v0",
        "selection": selection,
        "semantic_parameters": parameters,
        "semantic_profile": "mpk.go.fixed.v0",
        "source_language": "go",
        "status": "rejected",
        "unexpected": true
    });
    let mut pretty =
        serde_json::to_vec_pretty(&malformed_shape).expect("mixed shape transport serializes");
    pretty.push(b'\n');
    let shape =
        validate_raw_transport_with_identity(&pretty, Some(3), false, 0, &parameters, &selection)
            .expect_err("shape owns precedence over later JCS equality");
    assert_eq!(shape.code(), FrontendProtocolCode::ProtocolShape);

    let oversized = json!({
        "diagnostics": [],
        "phase": "capture",
        "rejected_features": repeated_issues(1_025, "x"),
        "schema": "mpk.frontend.cli.v0",
        "selection": selection,
        "semantic_parameters": parameters,
        "semantic_profile": "mpk.go.fixed.v0",
        "source_language": "go",
        "status": "rejected"
    });
    let mut malformed = serde_json::to_vec(&oversized).expect("oversized envelope serializes");
    *malformed.last_mut().expect("envelope has closing brace") = b']';
    malformed.push(b'\n');
    let lexical = validate_raw_transport_with_identity(
        &malformed,
        Some(3),
        false,
        0,
        &parameters,
        &selection,
    )
    .expect_err("malformed JSON owns precedence over the recorded issue limit");
    assert_eq!(lexical.code(), FrontendProtocolCode::ProtocolMalformed);
}

fn validate_rejected(
    rejected_features: Vec<Value>,
    diagnostics: Vec<Value>,
    exit_code: Option<i32>,
    signaled: bool,
    stderr_observed_bytes: usize,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    let parameters = json!({"pointer_width":64,"target_id":"linux/amd64"});
    let selection =
        json!({"function":"example.com/mpk/vector.Identity","package":"example.com/mpk/vector"});
    let envelope = json!({
        "diagnostics": diagnostics,
        "phase": "capture",
        "rejected_features": rejected_features,
        "schema": "mpk.frontend.cli.v0",
        "selection": selection,
        "semantic_parameters": parameters,
        "semantic_profile": "mpk.go.fixed.v0",
        "source_language": "go",
        "status": "rejected"
    });
    let mut transport = serde_json::to_vec(&envelope).expect("serialize frontend envelope");
    transport.push(b'\n');
    validate_raw_transport_with_identity(
        &transport,
        exit_code,
        signaled,
        stderr_observed_bytes,
        &parameters,
        &selection,
    )
}

fn validate_raw_transport(
    transport: &[u8],
    exit_code: Option<i32>,
    signaled: bool,
    stderr_observed_bytes: usize,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    let parameters = json!({"pointer_width":64,"target_id":"linux/amd64"});
    let selection =
        json!({"function":"example.com/mpk/vector.Identity","package":"example.com/mpk/vector"});
    validate_raw_transport_with_identity(
        transport,
        exit_code,
        signaled,
        stderr_observed_bytes,
        &parameters,
        &selection,
    )
}

fn validate_raw_transport_with_identity(
    transport: &[u8],
    exit_code: Option<i32>,
    signaled: bool,
    stderr_observed_bytes: usize,
    parameters: &Value,
    selection: &Value,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    validate_frontend_process(
        FrontendProtocolRequest {
            source_language: "go",
            semantic_profile: "mpk.go.fixed.v0",
            semantic_parameters: parameters,
            selection,
            release_registry: None,
            captured_inputs: &[],
        },
        FrontendProcessFacts {
            exit_code,
            signaled,
            stdout: transport,
            stderr_observed_bytes,
        },
    )
}

fn issue(message: &str) -> Value {
    json!({"code":"GO_SUBSET_LIMIT","message":message})
}

fn repeated_issues(count: usize, message: &str) -> Vec<Value> {
    (0..count).map(|_| issue(message)).collect()
}

fn truncation_marker(omitted: usize) -> Value {
    json!({
        "code": "GO_LIMIT_DIAGNOSTICS_TRUNCATED",
        "message": format!("{omitted} normalized issues omitted")
    })
}

fn assert_non_success_without_artifacts(envelope: &AcceptedFrontendEnvelope) {
    assert_eq!(envelope.status, "rejected");
    assert_eq!(envelope.phase, "capture");
    assert!(envelope.artifacts.is_none());
}
