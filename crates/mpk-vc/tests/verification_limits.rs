use std::collections::BTreeSet;

use mpk_vc::{
    canonical_json_bytes, canonical_json_bytes_bounded, import_vc_skeleton_v1_json,
    import_vc_v1_json, scan_strict_json, serialize_json_bounded, validate_verification_limit,
    BoundedJsonSerializeError, CanonicalJsonError, GoFixedParameters, PointerWidth,
    SemanticParameters, SemanticProfile, StrictJsonError, StrictJsonEvent, StrictJsonLimits,
    StrictJsonObserver, StrictJsonPathSegment, StrictJsonValue, VcSkeletonValidationPhase,
    VcSourceContext, VcValidationPhase, VerificationLimitError, VerificationLimitId,
    MAX_SUPPORTED_JSON_DEPTH,
};

const REGISTERED_LIMITS: [(VerificationLimitId, &str, u64, &str); 11] = [
    (
        VerificationLimitId::MembersPerFunction,
        "members_per_function",
        100_000,
        "VC_LIMIT_MEMBERS_PER_FUNCTION",
    ),
    (
        VerificationLimitId::MembersPerDocument,
        "members_per_document",
        262_144,
        "VC_LIMIT_MEMBERS_PER_DOCUMENT",
    ),
    (
        VerificationLimitId::AssumptionsPerMember,
        "assumptions_per_member",
        4_096,
        "VC_LIMIT_ASSUMPTIONS_PER_MEMBER",
    ),
    (
        VerificationLimitId::ExpressionNodesPerMember,
        "expression_nodes_per_member",
        8_192,
        "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
    ),
    (
        VerificationLimitId::ExpressionNodesPerDocument,
        "expression_nodes_per_document",
        4_194_304,
        "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
    ),
    (
        VerificationLimitId::MemberExpressionDepth,
        "member_expression_depth",
        256,
        "VC_LIMIT_MEMBER_EXPRESSION_DEPTH",
    ),
    (
        VerificationLimitId::GroupedTheoremDepth,
        "grouped_theorem_depth",
        512,
        "VC_LIMIT_GROUPED_THEOREM_DEPTH",
    ),
    (
        VerificationLimitId::GeneratedProofDepth,
        "generated_proof_depth",
        512,
        "VC_LIMIT_GENERATED_PROOF_DEPTH",
    ),
    (
        VerificationLimitId::CanonicalVcJsonBytes,
        "canonical_vc_json_bytes",
        268_435_456,
        "VC_LIMIT_CANONICAL_JSON_BYTES",
    ),
    (
        VerificationLimitId::CanonicalSkeletonJsonBytes,
        "canonical_skeleton_json_bytes",
        268_435_456,
        "VC_LIMIT_CANONICAL_SKELETON_JSON_BYTES",
    ),
    (
        VerificationLimitId::CanonicalCertificateBytes,
        "canonical_certificate_bytes",
        536_870_912,
        "VC_LIMIT_CANONICAL_CERTIFICATE_BYTES",
    ),
];

fn assert_variant_is_registered(limit: VerificationLimitId) {
    match limit {
        VerificationLimitId::MembersPerFunction
        | VerificationLimitId::MembersPerDocument
        | VerificationLimitId::AssumptionsPerMember
        | VerificationLimitId::ExpressionNodesPerMember
        | VerificationLimitId::ExpressionNodesPerDocument
        | VerificationLimitId::MemberExpressionDepth
        | VerificationLimitId::GroupedTheoremDepth
        | VerificationLimitId::GeneratedProofDepth
        | VerificationLimitId::CanonicalVcJsonBytes
        | VerificationLimitId::CanonicalSkeletonJsonBytes
        | VerificationLimitId::CanonicalCertificateBytes => {}
    }
}

#[test]
fn closed_registry_matches_profile_and_derived_boundaries() {
    let mut ids = BTreeSet::new();
    let mut codes = BTreeSet::new();

    assert_eq!(REGISTERED_LIMITS.len(), 11);
    for &(limit, id, maximum, code) in &REGISTERED_LIMITS {
        assert_variant_is_registered(limit);
        assert!(ids.insert(id), "duplicate verification limit ID {id}");
        assert!(
            codes.insert(code),
            "duplicate verification limit code {code}"
        );
        assert_eq!(limit.as_str(), id);
        assert_eq!(limit.maximum(), maximum);
        assert_eq!(limit.code(), code);
        assert_eq!(VerificationLimitId::try_from(id), Ok(limit));

        let below = maximum
            .checked_sub(1)
            .expect("registered maximum is positive");
        let above = maximum
            .checked_add(1)
            .expect("registered maximum leaves room for an above case");

        validate_verification_limit(id, below).expect("below maximum accepts");
        validate_verification_limit(id, maximum).expect("inclusive maximum accepts");

        let error = validate_verification_limit(id, above).expect_err("above maximum rejects");
        assert_eq!(
            error,
            VerificationLimitError::Exceeded {
                limit,
                count: above,
            }
        );
        assert_eq!(error.code(), code);
    }
}

#[test]
fn every_registered_counter_overflow_uses_its_owner_code() {
    for &(limit, _, _, code) in &REGISTERED_LIMITS {
        let error = limit
            .checked_add_count(u64::MAX, 1)
            .expect_err("checked counter addition rejects overflow");
        assert_eq!(error, VerificationLimitError::CounterOverflow { limit });
        assert_eq!(error.code(), code);
    }
}

#[test]
fn unknown_limit_id_is_rejected_by_the_closed_registry() {
    let id = "canonical_policy_json_bytes";
    let expected = VerificationLimitError::UnknownLimit(id.to_owned());

    assert_eq!(VerificationLimitId::try_from(id), Err(expected.clone()));
    assert_eq!(validate_verification_limit(id, 0), Err(expected));
}

#[test]
fn bounded_json_writers_enforce_encoded_byte_ceiling() {
    let value = StrictJsonValue::Object(vec![(
        "escaped".to_owned(),
        StrictJsonValue::String("\n\"\\\u{0001}".to_owned()),
    )]);
    let expected = br#"{"escaped":"\n\"\\\u0001"}"#;
    let canonical = canonical_json_bytes(&value).expect("canonical JSON encodes");
    assert_eq!(canonical, expected);

    assert_eq!(
        canonical_json_bytes_bounded(&value, expected.len()).expect("exact ceiling accepts"),
        expected
    );
    assert_eq!(
        canonical_json_bytes_bounded(&value, expected.len() - 1),
        Err(CanonicalJsonError::OutputBytesExceeded {
            maximum: expected.len() - 1,
        })
    );

    let serde_value = serde_json::json!({"escaped": "\n\"\\\u{0001}"});
    assert_eq!(
        serialize_json_bounded(&serde_value, expected.len()).expect("exact ceiling accepts"),
        expected
    );
    assert_eq!(
        serialize_json_bounded(&serde_value, expected.len() - 1),
        Err(BoundedJsonSerializeError::OutputBytesExceeded {
            maximum: expected.len() - 1,
        })
    );
}

#[test]
fn bounded_canonical_writer_accepts_maximum_supported_depth() {
    let mut value = StrictJsonValue::Integer(0);
    for _ in 0..MAX_SUPPORTED_JSON_DEPTH {
        value = StrictJsonValue::Array(vec![value]);
    }
    let expected_len = usize::try_from(MAX_SUPPORTED_JSON_DEPTH)
        .expect("depth fits usize")
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(1))
        .expect("canonical length fits usize");
    let canonical = canonical_json_bytes_bounded(&value, expected_len)
        .expect("bounded writer handles the maximum supported parse depth");
    assert_eq!(canonical.len(), expected_len);
}

struct MessageCeiling {
    maximum: u64,
    saw_later: bool,
}

impl StrictJsonObserver for MessageCeiling {
    fn observe(&mut self, event: StrictJsonEvent<'_>) -> Result<(), StrictJsonError> {
        if matches!(
            event,
            StrictJsonEvent::String {
                path: [StrictJsonPathSegment::Key(name)],
                ..
            } if name == "later"
        ) {
            self.saw_later = true;
        }
        Ok(())
    }

    fn string_byte_limit(&mut self, path: &[StrictJsonPathSegment]) -> Option<(&'static str, u64)> {
        matches!(path, [StrictJsonPathSegment::Key(name)] if name == "message")
            .then_some(("message_bytes", self.maximum))
    }
}

#[test]
fn strict_json_scanner_counts_escaped_bytes_and_stops_at_tighter_limit() {
    let limits = StrictJsonLimits::new(1_024, 64, 8, 64);
    let mut at = MessageCeiling {
        maximum: 4,
        saw_later: false,
    };
    scan_strict_json(br#"{"message":"\u20acx","later":"seen"}"#, limits, &mut at)
        .expect("three-byte escaped scalar plus one byte accepts at four");
    assert!(at.saw_later);

    let mut above = MessageCeiling {
        maximum: 3,
        saw_later: false,
    };
    let error = scan_strict_json(br#"{"message":"\u20acx","later":]}"#, limits, &mut above)
        .expect_err("schema string ceiling stops before later malformed syntax");
    assert_eq!(
        error,
        StrictJsonError::ObservedLimitExceeded {
            limit: "message_bytes",
            maximum: 3,
            actual: 4,
        }
    );
    assert!(!above.saw_later);
}

#[test]
fn vc_import_stream_rejects_assumption_limit_before_typed_allocation() {
    let maximum = VerificationLimitId::AssumptionsPerMember.maximum() as usize;
    let mut document: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/vir-go/derived/payment-points/vc.json"
    ))
    .expect("committed VC fixture parses");
    document["functions"][0]["members"][0]["assumptions"] = serde_json::Value::Array(vec![
        serde_json::json!({"kind":"constant","name":"Std.Bool.true"});
        maximum + 1
    ]);
    let input = serde_json::to_vec(&document).expect("oversized VC serializes");

    let error = import_vc_v1_json(&input, &empty_source_context())
        .expect_err("assumption count above the profile rejects during stream observation");
    assert_eq!(error.phase(), VcValidationPhase::StreamLimits);
    assert_eq!(
        error.code(),
        VerificationLimitId::AssumptionsPerMember.code()
    );

    document["functions"][0]["members"][0]["assumptions"]
        .as_array_mut()
        .expect("fixture member assumptions are an array")
        .last_mut()
        .expect("above-limit assumption exists")
        .as_object_mut()
        .expect("assumption is an object")
        .remove("name");
    let mixed_nested_shape =
        serde_json::to_vec(&document).expect("mixed nested-shape VC serializes");
    let shape = import_vc_v1_json(&mixed_nested_shape, &empty_source_context())
        .expect_err("complete nested shape validation owns precedence over stream limits");
    assert_eq!(shape.phase(), VcValidationPhase::Shape);
    assert_eq!(shape.code(), "VC_SHAPE");

    document["functions"][0]["members"][0]["assumptions"]
        .as_array_mut()
        .expect("fixture member assumptions are an array")
        .last_mut()
        .expect("above-limit assumption exists")["name"] =
        serde_json::Value::String("not-an-mpk-name".to_owned());
    let mixed_nested_scalar =
        serde_json::to_vec(&document).expect("mixed nested-scalar VC serializes");
    let scalar = import_vc_v1_json(&mixed_nested_scalar, &empty_source_context())
        .expect_err("complete nested scalar validation owns precedence over stream limits");
    assert_eq!(scalar.phase(), VcValidationPhase::Scalar);
    assert_eq!(scalar.code(), "VC_SCALAR");

    document["vc_hash"] = serde_json::Value::String("not-a-sha256".to_owned());
    let mixed = serde_json::to_vec(&document).expect("mixed-failure VC serializes");
    let scalar = import_vc_v1_json(&mixed, &empty_source_context())
        .expect_err("root scalar validation owns precedence over the recorded stream limit");
    assert_eq!(scalar.phase(), VcValidationPhase::Scalar);
    assert_eq!(scalar.code(), "VC_SCALAR");
}

fn empty_source_context() -> VcSourceContext {
    VcSourceContext {
        id: "limits.test".to_owned(),
        source_ir_schema: "mpk.vir.v0".to_owned(),
        source_ir_hash: "0".repeat(64),
        input_set_hash: "0".repeat(64),
        semantic_profile: SemanticProfile::GoFixedV0,
        semantic_parameters: SemanticParameters::GoFixed(GoFixedParameters {
            target_id: "linux/amd64".to_owned(),
            pointer_width: PointerWidth::Bits64,
        }),
        verification_limit_profile: "mpk.verify.limits.v0".to_owned(),
        functions: Vec::new(),
    }
}

#[test]
fn skeleton_import_stream_limit_preserves_shape_and_scalar_precedence() {
    let maximum = VerificationLimitId::MembersPerDocument.maximum() as usize;
    let mut skeleton: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/vir-go/derived/payment-points/vc-skeleton.json"
    ))
    .expect("committed skeleton fixture parses");
    skeleton["theorem_declarations"][0]["member_ids"] = serde_json::Value::Array(vec![
        serde_json::Value::String("member".to_owned());
        maximum + 1
    ]);
    let input = serde_json::to_vec(&skeleton).expect("oversized skeleton serializes");
    let error = import_vc_skeleton_v1_json(&input, b"", &empty_source_context())
        .expect_err("skeleton aggregate member limit rejects before typed allocation");
    assert_eq!(error.phase(), VcSkeletonValidationPhase::StreamLimits);
    assert_eq!(error.code(), "VC_SKELETON_SHAPE");

    skeleton["source_vc_hash"] = serde_json::Value::String("not-a-sha256".to_owned());
    let mixed = serde_json::to_vec(&skeleton).expect("mixed-failure skeleton serializes");
    let scalar = import_vc_skeleton_v1_json(&mixed, b"", &empty_source_context())
        .expect_err("skeleton scalar validation owns precedence over its stream limit");
    assert_eq!(scalar.phase(), VcSkeletonValidationPhase::Scalar);
    assert_eq!(scalar.code(), "VC_SKELETON_SHAPE");
}
