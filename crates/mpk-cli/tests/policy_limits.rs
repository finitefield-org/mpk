use std::collections::BTreeSet;

use mpk_cli::policy_report::render_policy_evidence_v1_markdown;
use mpk_cli::policy_schema::{
    import_policy_evidence_v1_for_consumer, validate_policy_limit, PolicyValidationPhase,
    POLICY_MARKDOWN_BYTES_MAX,
};

struct PolicyLimitCase {
    counter: &'static str,
    maximum: u64,
    phase: PolicyValidationPhase,
    code: &'static str,
}

const POLICY_LIMITS: [PolicyLimitCase; 14] = [
    PolicyLimitCase {
        counter: "json_transport_bytes",
        maximum: 268_435_456,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_JSON_BYTES",
    },
    PolicyLimitCase {
        counter: "markdown_bytes",
        maximum: 268_435_456,
        phase: PolicyValidationPhase::Report,
        code: "POLICY_LIMIT_MARKDOWN_BYTES",
    },
    PolicyLimitCase {
        counter: "json_nesting",
        maximum: 256,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_JSON_INVALID",
    },
    PolicyLimitCase {
        counter: "string_bytes",
        maximum: 1_048_576,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_JSON_INVALID",
    },
    PolicyLimitCase {
        counter: "array_elements_default",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "object_members_default",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "helper_artifacts",
        maximum: 65_536,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "certificates",
        maximum: 1,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "checked_declarations",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "theory_certificates",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "properties",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "member_rows",
        maximum: 262_144,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "references_per_member",
        maximum: 4_096,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
    PolicyLimitCase {
        counter: "recipe_argv_elements",
        maximum: 65_536,
        phase: PolicyValidationPhase::Transport,
        code: "POLICY_LIMIT_COLLECTION",
    },
];

#[test]
fn closed_policy_limit_registry_enforces_derived_boundaries() {
    let mut counters = BTreeSet::new();

    assert_eq!(POLICY_LIMITS.len(), 14);
    for case in &POLICY_LIMITS {
        assert!(
            counters.insert(case.counter),
            "duplicate policy counter ID {}",
            case.counter
        );

        let below = case
            .maximum
            .checked_sub(1)
            .expect("registered maximum is positive");
        let above = case
            .maximum
            .checked_add(1)
            .expect("registered maximum leaves room for an above case");

        validate_policy_limit(case.counter, below).expect("below maximum accepts");
        validate_policy_limit(case.counter, case.maximum).expect("inclusive maximum accepts");

        let error = validate_policy_limit(case.counter, above).expect_err("above maximum rejects");
        assert_eq!(error.phase(), case.phase, "{} phase", case.counter);
        assert_eq!(error.code(), case.code, "{} code", case.counter);
    }
}

#[test]
fn unknown_policy_counter_uses_closed_registry_error() {
    let error = validate_policy_limit("policy_report_rows", 0)
        .expect_err("unregistered policy counter rejects");

    assert_eq!(error.phase(), PolicyValidationPhase::Transport);
    assert_eq!(error.code(), "POLICY_LIMIT_COLLECTION");
    assert!(error.to_string().contains("unknown policy counter"));
}

#[test]
fn policy_transport_preserves_nesting_limit_error() {
    let depth = 257;
    let mut transport = Vec::with_capacity(depth * 2 + 2);
    transport.extend(vec![b'['; depth]);
    transport.push(b'0');
    transport.extend(vec![b']'; depth]);
    transport.push(b'\n');

    let error = import_policy_evidence_v1_for_consumer(&transport)
        .expect_err("transport nesting above the registered maximum rejects");
    assert_eq!(error.phase(), PolicyValidationPhase::Transport);
    assert_eq!(error.code(), "POLICY_JSON_INVALID");
}

#[test]
fn helper_limit_is_enforced_by_the_production_transport_scanner() {
    let count = 65_537;
    let mut document: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/vir-go/policy/evidence.json"
    ))
    .expect("committed evidence fixture parses");
    let helpers = document["helper_artifacts"]
        .as_array_mut()
        .expect("fixture helper_artifacts is an array");
    let exemplar = helpers.first().cloned().expect("fixture has a helper");
    *helpers = vec![exemplar; count];
    let mut transport = serde_json::to_vec(&document).expect("oversized evidence serializes");
    transport.push(b'\n');

    let error = import_policy_evidence_v1_for_consumer(&transport)
        .expect_err("field-specific limit rejects before missing schema fields");
    assert_eq!(error.phase(), PolicyValidationPhase::Transport);
    assert_eq!(error.code(), "POLICY_LIMIT_COLLECTION");

    document["properties"] = serde_json::Value::Array(Vec::new());
    let mut mixed_shape =
        serde_json::to_vec(&document).expect("mixed shape/limit evidence serializes");
    mixed_shape.push(b'\n');
    let shape = import_policy_evidence_v1_for_consumer(&mixed_shape)
        .expect_err("shape validation owns precedence over the recorded helper limit");
    assert_eq!(shape.phase(), PolicyValidationPhase::Shape);
    assert_eq!(shape.code(), "POLICY_SHAPE");

    document["properties"] = serde_json::from_slice::<serde_json::Value>(include_bytes!(
        "../../../fixtures/vir-go/policy/evidence.json"
    ))
    .expect("committed evidence fixture parses")["properties"]
        .clone();
    document["vc_hash"] = serde_json::Value::String("not-a-sha256".to_owned());
    let mut mixed_scalar =
        serde_json::to_vec(&document).expect("mixed scalar/limit evidence serializes");
    mixed_scalar.push(b'\n');
    let scalar = import_policy_evidence_v1_for_consumer(&mixed_scalar)
        .expect_err("scalar validation owns precedence over the recorded helper limit");
    assert_eq!(scalar.phase(), PolicyValidationPhase::Scalar);
    assert_eq!(scalar.code(), "POLICY_SCALAR");

    document["vc_hash"] = serde_json::from_slice::<serde_json::Value>(include_bytes!(
        "../../../fixtures/vir-go/policy/evidence.json"
    ))
    .expect("committed evidence fixture parses")["vc_hash"]
        .clone();
    document["properties"][0]["members"][0]
        .as_object_mut()
        .expect("fixture member is an object")
        .remove("status");
    let mut mixed_nested_shape =
        serde_json::to_vec(&document).expect("mixed nested-shape evidence serializes");
    mixed_nested_shape.push(b'\n');
    let shape = import_policy_evidence_v1_for_consumer(&mixed_nested_shape)
        .expect_err("complete nested shape validation owns precedence over helper limits");
    assert_eq!(shape.phase(), PolicyValidationPhase::Shape);
    assert_eq!(shape.code(), "POLICY_SHAPE");

    document["properties"][0]["members"][0]["status"] =
        serde_json::Value::String("not-a-status".to_owned());
    let mut mixed_nested_scalar =
        serde_json::to_vec(&document).expect("mixed nested-scalar evidence serializes");
    mixed_nested_scalar.push(b'\n');
    let scalar = import_policy_evidence_v1_for_consumer(&mixed_nested_scalar)
        .expect_err("complete nested scalar validation owns precedence over helper limits");
    assert_eq!(scalar.phase(), PolicyValidationPhase::Scalar);
    assert_eq!(scalar.code(), "POLICY_SCALAR");
}

#[test]
fn malformed_transport_precedes_a_recorded_helper_limit() {
    let maximum = 65_536;
    let mut transport = Vec::with_capacity(maximum * 5 + 24);
    transport.extend_from_slice(b"{\"helper_artifacts\":[");
    for index in 0..maximum {
        if index != 0 {
            transport.push(b',');
        }
        transport.extend_from_slice(b"null");
    }
    transport.extend_from_slice(b",]}\n");

    let error = import_policy_evidence_v1_for_consumer(&transport)
        .expect_err("malformed JSON owns precedence over the recorded next-slot limit");
    assert_eq!(error.phase(), PolicyValidationPhase::Transport);
    assert_eq!(error.code(), "POLICY_JSON_INVALID");
}

#[test]
fn production_markdown_renderer_stays_within_registered_bound() {
    let evidence = import_policy_evidence_v1_for_consumer(include_bytes!(
        "../../../fixtures/vir-go/policy/evidence.json"
    ))
    .expect("committed policy evidence imports");
    let markdown = render_policy_evidence_v1_markdown(&evidence).expect("bounded Markdown renders");

    assert!(markdown.starts_with("# MPK Policy Evidence Report\n"));
    assert!((markdown.len() as u64) < POLICY_MARKDOWN_BYTES_MAX);
    validate_policy_limit("markdown_bytes", markdown.len() as u64)
        .expect("rendered Markdown satisfies its registered bound");
}
