use mpk_api::{
    theory_strategy_certificate, theory_strategy_certificate_evidence, ApiErrorCode, ApiProofId,
    ApiService, ApiTermId, ConstTermRequest, PolicyAxiomProfile, PolicyObligationDescriptor,
    PolicyObligationPattern, PolicyReadiness, PolicyStrategyErrorCode, PolicyStrategyMetadata,
    PolicyStrategyProfile, ProofProfile, SortTermRequest, StartSessionRequest, StrategyKind,
    StrategyProveRequest, TheoryStrategyKind, PAYMENT_POLICY_ALPHA_PROFILE,
    PAYMENT_POLICY_RUST_ALPHA_PROFILE, POLICY_STRATEGY_REGISTRY,
};
use mpk_cert::encode::ProofNode;
use mpk_vc::{SemanticProfile, SourceLanguage};

const RESERVE_FIRST_OBLIGATION: &str =
    "example.com/payment/reserve.ApprovedReserveCents.then.post0";

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.PolicyStrategy"))
        .expect("session starts")
        .session_id
}

fn start_session_with_profile(
    api: &mut ApiService,
    proof_profile: ProofProfile,
) -> mpk_api::SessionId {
    api.start_session(
        StartSessionRequest::new("Example.Api.PolicyStrategy").with_proof_profile(proof_profile),
    )
    .expect("session starts")
    .session_id
}

fn sort_zero(api: &mut ApiService, session_id: &mpk_api::SessionId) -> ApiTermId {
    api.term_sort(SortTermRequest {
        session_id: session_id.clone(),
        universe: 0,
    })
    .expect("sort term constructs")
    .term_id
}

fn register_simple_axiom(
    api: &mut ApiService,
    session_id: &mpk_api::SessionId,
    sort: ApiTermId,
    name: &str,
) -> ApiTermId {
    let sort_core = api
        .session(session_id)
        .and_then(|session| session.core_term_id(sort))
        .expect("sort core term is addressable");
    api.session_mut(session_id)
        .expect("session exists")
        .environment_mut()
        .register_axiom(name, sort_core)
        .expect("test axiom registers");
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: name.to_owned(),
        levels: Vec::new(),
    })
    .expect("const term constructs")
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
fn strategy_registry_binds_exact_go_and_rust_tuples_in_stable_order() {
    assert_eq!(POLICY_STRATEGY_REGISTRY.len(), 2);
    let go = POLICY_STRATEGY_REGISTRY[0];
    assert_eq!(
        go.strategy_profile,
        PolicyStrategyProfile::PaymentPolicyAlpha
    );
    assert_eq!(go.source_language, SourceLanguage::Go);
    assert_eq!(go.semantic_profile, SemanticProfile::GoFixedV0);
    assert_eq!(go.axiom_profile, PolicyAxiomProfile::ZeroAxiom);

    let rust = POLICY_STRATEGY_REGISTRY[1];
    assert_eq!(
        rust.strategy_profile,
        PolicyStrategyProfile::PaymentPolicyRustAlpha
    );
    assert_eq!(rust.source_language, SourceLanguage::Rust);
    assert_eq!(rust.semantic_profile, SemanticProfile::RustCheckedV0);
    assert_eq!(rust.axiom_profile, PolicyAxiomProfile::MvpTheory);
    assert_eq!(rust.source_language_name(), "rust");
    assert_eq!(rust.semantic_profile_name(), "mpk.rust.checked.v0");
    assert_eq!(
        serde_json::to_string(&POLICY_STRATEGY_REGISTRY).unwrap(),
        r#"[{"strategy_profile":"payment-policy-alpha","source_language":"go","semantic_profile":"mpk.go.fixed.v0","axiom_profile":"zero-axiom"},{"strategy_profile":"payment-policy-rust-alpha","source_language":"rust","semantic_profile":"mpk.rust.checked.v0","axiom_profile":"mvp-theory"}]"#
    );
}

#[test]
fn rust_strategy_reuses_neutral_patterns_with_rust_specific_readiness() {
    let rust_profile = PAYMENT_POLICY_RUST_ALPHA_PROFILE
        .parse::<PolicyStrategyProfile>()
        .expect("Rust policy strategy parses");
    assert_eq!(rust_profile, PolicyStrategyProfile::PaymentPolicyRustAlpha);
    assert_eq!(
        rust_profile.canonical_name(),
        PAYMENT_POLICY_RUST_ALPHA_PROFILE
    );

    let go = PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).unwrap();
    let rust = PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_RUST_ALPHA_PROFILE).unwrap();
    assert_eq!(
        rust.allowed_obligation_patterns, go.allowed_obligation_patterns,
        "the supported business and safety patterns are language-neutral"
    );
    assert_eq!(
        rust.candidate_theory_strategies,
        go.candidate_theory_strategies
    );
    assert!(rust_profile
        .readiness_description(PolicyReadiness::Ready)
        .contains("Rust function"));
    assert!(rust_profile
        .readiness_description(PolicyReadiness::Ready)
        .contains("checked arithmetic"));
    for readiness in [
        PolicyReadiness::Ready,
        PolicyReadiness::Unsupported,
        PolicyReadiness::SourceError,
        PolicyReadiness::FrontendError,
    ] {
        let description = rust_profile.readiness_description(readiness);
        assert!(description.contains("Rust"));
        assert!(!description.contains("Go"));
    }
    assert_eq!(
        serde_json::to_string(&[
            PolicyReadiness::Ready,
            PolicyReadiness::Unsupported,
            PolicyReadiness::SourceError,
            PolicyReadiness::FrontendError,
        ])
        .unwrap(),
        r#"["ready","unsupported","source_error","frontend_error"]"#
    );
}

#[test]
fn payment_policy_alpha_closes_first_reserve_nonnegative_with_checked_linarith() {
    let metadata =
        PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).expect("metadata");
    metadata
        .validate_obligation(&PolicyObligationDescriptor::new(
            RESERVE_FIRST_OBLIGATION,
            PolicyObligationPattern::NonNegativeResult,
        ))
        .expect("first reserve obligation is inside payment-policy-alpha");
    let theory_candidate = metadata
        .theory_candidates()
        .into_iter()
        .find(|candidate| candidate.theory == TheoryStrategyKind::Linarith)
        .expect("linarith candidate exists");
    let evidence = theory_strategy_certificate_evidence(theory_candidate.theory);
    assert_eq!(evidence.format, "mpk.linarith.v0");
    assert_eq!(
        evidence.theory_certificate_hash,
        "a85d54f8d5c32dba5f414490120847013b7c727a3ce8b6ae2c3a44aae4edd7e1"
    );

    let mut api = ApiService::new();
    let session_id = start_session_with_profile(&mut api, ProofProfile::MvpStrict);
    let sort = sort_zero(&mut api, &session_id);
    let _witness = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.PolicyStrategy.reserveNonnegativeWitness",
    );

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

    assert!(response.ok);
    assert_eq!(response.proof_id, Some(ApiProofId(0)));
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Theory);
    assert!(response.attempts[0].ok);

    let session = api.session(&session_id).expect("session exists");
    assert_eq!(session.theory_certificate_count(), 1);
    assert_eq!(
        session
            .theory_certificate(0)
            .expect("registered theory certificate"),
        &theory_strategy_certificate(TheoryStrategyKind::Linarith)
    );
    assert!(matches!(
        session.proof_node(ApiProofId(0)),
        Some(ProofNode::Theory {
            theory_certificate: 0,
            expected_type
        }) if *expected_type == sort.as_u32()
    ));
}

#[test]
fn payment_policy_alpha_reserve_theory_fails_under_non_theory_profile() {
    let metadata =
        PolicyStrategyMetadata::parse_profile(PAYMENT_POLICY_ALPHA_PROFILE).expect("metadata");
    metadata
        .validate_obligation(&PolicyObligationDescriptor::new(
            RESERVE_FIRST_OBLIGATION,
            PolicyObligationPattern::NonNegativeResult,
        ))
        .expect("first reserve obligation is inside payment-policy-alpha");
    let theory_candidate = metadata
        .theory_candidates()
        .into_iter()
        .find(|candidate| candidate.theory == TheoryStrategyKind::Linarith)
        .expect("linarith candidate exists");

    let mut api = ApiService::new();
    let session_id = start_session_with_profile(&mut api, ProofProfile::MvpStructural);
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

#[test]
fn unknown_strategy_profile_rejects_deterministically() {
    let error = PolicyStrategyMetadata::parse_profile("payment-policy-basic")
        .expect_err("unknown strategy profile rejects");

    assert_eq!(error.code, PolicyStrategyErrorCode::UnknownStrategyProfile);
    assert_eq!(
        error.to_string(),
        "UNKNOWN_STRATEGY_PROFILE: unknown policy strategy profile \"payment-policy-basic\"; expected one of: payment-policy-alpha, payment-policy-rust-alpha"
    );
    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");
    assert_eq!(
        encoded,
        r#"{
  "code": "UNKNOWN_STRATEGY_PROFILE",
  "message": "unknown policy strategy profile \"payment-policy-basic\"; expected one of: payment-policy-alpha, payment-policy-rust-alpha",
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
