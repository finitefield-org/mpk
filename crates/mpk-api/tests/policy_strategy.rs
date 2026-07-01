use mpk_api::{
    ApiErrorCode, ApiService, PolicyObligationDescriptor, PolicyObligationPattern,
    PolicyStrategyErrorCode, PolicyStrategyMetadata, PolicyStrategyProfile, SortTermRequest,
    StartSessionRequest, StrategyKind, StrategyProveRequest, TheoryStrategyKind,
    PAYMENT_POLICY_ALPHA_PROFILE,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.PolicyStrategy"))
        .expect("session starts")
        .session_id
}

fn sort_zero(api: &mut ApiService, session_id: &mpk_api::SessionId) -> mpk_api::ApiTermId {
    api.term_sort(SortTermRequest {
        session_id: session_id.clone(),
        universe: 0,
    })
    .expect("sort term constructs")
    .term_id
}

#[test]
fn payment_policy_alpha_profile_metadata_is_stable() {
    let profile = PAYMENT_POLICY_ALPHA_PROFILE
        .parse::<PolicyStrategyProfile>()
        .expect("payment-policy-alpha parses");
    assert_eq!(profile, PolicyStrategyProfile::PaymentPolicyAlpha);
    assert_eq!(profile.canonical_name(), PAYMENT_POLICY_ALPHA_PROFILE);

    let metadata =
        PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).expect("metadata");
    assert_eq!(metadata.profile, PolicyStrategyProfile::PaymentPolicyAlpha);
    assert_eq!(
        metadata.allowed_obligation_patterns,
        vec![
            PolicyObligationPattern::NonNegativeResult,
            PolicyObligationPattern::ResultBoundedByInputAmount,
            PolicyObligationPattern::RefundBoundedByPaidMinusAlreadyRefunded,
            PolicyObligationPattern::DiscountOrFeeBoundedByConfiguredCaps,
            PolicyObligationPattern::BranchResultEqualsSelectedInput,
            PolicyObligationPattern::IntegerRuntimeSafety,
        ]
    );
    assert_eq!(
        metadata.candidate_theory_strategies,
        vec![
            TheoryStrategyKind::Linarith,
            TheoryStrategyKind::BitVecGround,
            TheoryStrategyKind::BoolTautology,
        ]
    );
    metadata
        .validate_obligation(&PolicyObligationDescriptor::new(
            "vc:reserve.then.post0",
            PolicyObligationPattern::NonNegativeResult,
        ))
        .expect("supported obligation is accepted");
    for pattern in [
        PolicyObligationPattern::ResultBoundedByInputAmount,
        PolicyObligationPattern::RefundBoundedByPaidMinusAlreadyRefunded,
        PolicyObligationPattern::DiscountOrFeeBoundedByConfiguredCaps,
        PolicyObligationPattern::BranchResultEqualsSelectedInput,
    ] {
        metadata
            .validate_obligation(&PolicyObligationDescriptor::new(
                format!("vc:{pattern:?}"),
                pattern,
            ))
            .expect("POE-08 classifier pattern is accepted by payment-policy-alpha");
    }

    let encoded = serde_json::to_string_pretty(&metadata).expect("metadata serializes");
    assert_eq!(
        encoded,
        r#"{
  "profile": "payment-policy-alpha",
  "allowed_obligation_patterns": [
    "non_negative_result",
    "result_bounded_by_input_amount",
    "refund_bounded_by_paid_minus_already_refunded",
    "discount_or_fee_bounded_by_configured_caps",
    "branch_result_equals_selected_input",
    "integer_runtime_safety"
  ],
  "candidate_theory_strategies": [
    "linarith",
    "bit_vec_ground",
    "bool_tautology"
  ]
}"#
    );
}

#[test]
fn unknown_strategy_profile_rejects_deterministically() {
    let error = PolicyStrategyMetadata::parse_profile("payment-policy-basic")
        .expect_err("unknown strategy profile rejects");

    assert_eq!(error.code, PolicyStrategyErrorCode::UnknownStrategyProfile);
    assert_eq!(
        error.to_string(),
        "UNKNOWN_STRATEGY_PROFILE: unknown policy strategy profile \"payment-policy-basic\"; expected one of: payment-policy-alpha"
    );
    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");
    assert_eq!(
        encoded,
        r#"{
  "code": "UNKNOWN_STRATEGY_PROFILE",
  "message": "unknown policy strategy profile \"payment-policy-basic\"; expected one of: payment-policy-alpha",
  "field": "strategy_profile",
  "detail": "payment-policy-basic"
}"#
    );
}

#[test]
fn obligation_outside_strategy_profile_has_deterministic_reason() {
    let metadata =
        PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).expect("metadata");
    let error = metadata
        .validate_obligation(&PolicyObligationDescriptor::new(
            "vc:gateway.db.state",
            PolicyObligationPattern::ExternalStateInvariant,
        ))
        .expect_err("profile rejects external state obligations");

    assert_eq!(
        error.code,
        PolicyStrategyErrorCode::ObligationOutsideProfile
    );
    assert_eq!(
        error.to_string(),
        "OBLIGATION_OUTSIDE_PROFILE: obligation \"vc:gateway.db.state\" with pattern \"external_state_invariant\" is outside policy strategy profile \"payment-policy-alpha\""
    );
    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");
    assert_eq!(
        encoded,
        r#"{
  "code": "OBLIGATION_OUTSIDE_PROFILE",
  "message": "obligation \"vc:gateway.db.state\" with pattern \"external_state_invariant\" is outside policy strategy profile \"payment-policy-alpha\"",
  "field": "obligation.pattern",
  "detail": "profile=payment-policy-alpha; obligation_id=vc:gateway.db.state; pattern=external_state_invariant"
}"#
    );
}

#[test]
fn profile_metadata_does_not_weaken_mvp_strict_theory_requirement() {
    let metadata =
        PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).expect("metadata");
    let mut candidates = metadata.theory_candidates();
    let theory_candidate = candidates.remove(0);

    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: false,
            apply: Vec::new(),
            theory: vec![theory_candidate],
        })
        .expect("strategy runner succeeds");

    assert!(!response.ok);
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Theory);
    assert_eq!(
        response.attempts[0]
            .error
            .as_ref()
            .expect("theory attempt has error")
            .code,
        ApiErrorCode::StrategyNotApplicable
    );
    let session = api.session(&session_id).expect("session exists");
    assert_eq!(session.theory_certificate_count(), 0);
    assert_eq!(session.proof_node_count(), 0);
}
