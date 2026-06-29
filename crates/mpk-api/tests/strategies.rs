use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, ApiTermId, ApplyStrategyCandidate, BinderTermRequest,
    ConstTermRequest, ExactProofRequest, ProofProfile, SortTermRequest, StartSessionRequest,
    StrategyKind, StrategyProveRequest, TheoryStrategyCandidate, TheoryStrategyKind,
    VarTermRequest,
};
use mpk_cert::encode::ProofNode;

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Strategies"))
        .expect("session starts")
        .session_id
}

fn start_session_with_profile(
    api: &mut ApiService,
    proof_profile: ProofProfile,
) -> mpk_api::SessionId {
    api.start_session(
        StartSessionRequest::new("Example.Api.Strategies").with_proof_profile(proof_profile),
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
fn strategies_try_exact_then_refl_in_safe_order() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let proof_term = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.Strategies.reflFixture",
    );

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: vec![sort],
            refl_terms: vec![proof_term],
            split: true,
            apply: Vec::new(),
            theory: Vec::new(),
        })
        .expect("strategy runner succeeds");

    assert!(response.ok);
    assert_eq!(response.proof_id, Some(ApiProofId(1)));
    assert_eq!(response.term_id, Some(proof_term));
    assert_eq!(response.attempts.len(), 2);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Exact);
    assert!(!response.attempts[0].ok);
    assert_eq!(
        response.attempts[0]
            .error
            .as_ref()
            .expect("failed exact has error")
            .code,
        ApiErrorCode::ProofCheckFailed
    );
    assert_eq!(response.attempts[1].strategy, StrategyKind::Refl);
    assert!(response.attempts[1].ok);
    assert_eq!(response.attempts[1].proof_id, Some(ApiProofId(1)));
}

#[test]
fn split_strategy_proves_simple_propositional_fixture() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let implication = api
        .term_pi(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort,
            body: sort,
        })
        .expect("pi term constructs")
        .term_id;

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: implication,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: true,
            apply: Vec::new(),
            theory: Vec::new(),
        })
        .expect("strategy runner succeeds");

    assert!(response.ok);
    assert_eq!(response.proof_id, Some(ApiProofId(1)));
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Split);
    assert!(response.attempts[0].ok);
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        2
    );
}

#[test]
fn apply_strategy_proves_simple_application_fixture() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let proof_term = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.Strategies.applyArg",
    );
    let var0 = api
        .term_var(VarTermRequest {
            session_id: session_id.clone(),
            index: 0,
        })
        .expect("var term constructs")
        .term_id;
    let pi = api
        .term_pi(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort,
            body: sort,
        })
        .expect("pi term constructs")
        .term_id;
    let lam = api
        .term_lam(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort,
            body: var0,
        })
        .expect("lambda term constructs")
        .term_id;
    let exact_arg = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort,
        })
        .expect("argument proof constructs")
        .proof_id;
    let exact_function = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: lam,
            expected_type: pi,
        })
        .expect("function proof constructs")
        .proof_id;

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: false,
            apply: vec![ApplyStrategyCandidate {
                function_proof: exact_function,
                argument_proofs: vec![exact_arg],
            }],
            theory: Vec::new(),
        })
        .expect("strategy runner succeeds");

    assert!(response.ok);
    assert_eq!(response.proof_id, Some(ApiProofId(2)));
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Apply);
    assert!(response.attempts[0].ok);
}

#[test]
fn split_reports_not_applicable_for_non_pi_goal() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id,
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: true,
            apply: Vec::new(),
            theory: Vec::new(),
        })
        .expect("strategy runner succeeds");

    assert!(!response.ok);
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Split);
    assert_eq!(
        response.attempts[0]
            .error
            .as_ref()
            .expect("split attempt has error")
            .code,
        ApiErrorCode::StrategyNotApplicable
    );
}

#[test]
fn theory_strategy_proves_max64_simple_vc_through_checked_certificate() {
    let mut api = ApiService::new();
    let session_id = start_session_with_profile(&mut api, ProofProfile::MvpStrict);
    let sort = sort_zero(&mut api, &session_id);
    let _witness = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.Strategies.Max64.then.post0.witness",
    );

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: false,
            apply: Vec::new(),
            theory: vec![TheoryStrategyCandidate {
                theory: TheoryStrategyKind::Linarith,
            }],
        })
        .expect("strategy runner succeeds");

    assert!(response.ok);
    assert_eq!(response.proof_id, Some(ApiProofId(0)));
    assert_eq!(response.attempts.len(), 1);
    assert_eq!(response.attempts[0].strategy, StrategyKind::Theory);
    assert!(response.attempts[0].ok);

    let session = api.session(&session_id).expect("session exists");
    assert_eq!(session.theory_certificate_count(), 1);
    assert!(matches!(
        session.proof_node(ApiProofId(0)),
        Some(ProofNode::Theory {
            theory_certificate: 0,
            expected_type
        }) if *expected_type == sort.as_u32()
    ));
}

#[test]
fn theory_strategy_builds_all_checked_certificate_kinds() {
    let mut api = ApiService::new();
    let session_id = start_session_with_profile(&mut api, ProofProfile::MvpStrict);
    let sort = sort_zero(&mut api, &session_id);
    let _witness = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.Strategies.allTheoryKindsWitness",
    );

    for (index, theory) in [
        TheoryStrategyKind::BoolTautology,
        TheoryStrategyKind::BitVecGround,
        TheoryStrategyKind::Linarith,
        TheoryStrategyKind::ArrayReadWrite,
    ]
    .into_iter()
    .enumerate()
    {
        let response = api
            .proof_try_strategies(StrategyProveRequest {
                session_id: session_id.clone(),
                expected_type: sort,
                exact_terms: Vec::new(),
                refl_terms: Vec::new(),
                split: false,
                apply: Vec::new(),
                theory: vec![TheoryStrategyCandidate { theory }],
            })
            .expect("strategy runner succeeds");

        let proof_id = ApiProofId(u32::try_from(index).expect("test index fits in u32"));
        assert!(response.ok, "theory strategy {theory:?} should check");
        assert_eq!(response.proof_id, Some(proof_id));
        assert_eq!(response.attempts.len(), 1);
        assert_eq!(response.attempts[0].strategy, StrategyKind::Theory);
        assert!(response.attempts[0].ok);

        let session = api.session(&session_id).expect("session exists");
        assert_eq!(session.theory_certificate_count(), index + 1);
        assert!(matches!(
            session.proof_node(proof_id),
            Some(ProofNode::Theory {
                theory_certificate,
                expected_type
            }) if usize::try_from(*theory_certificate).expect("certificate id fits in usize") == index
                && *expected_type == sort.as_u32()
        ));
    }
}

#[test]
fn theory_strategy_requires_mvp_strict_profile() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let _witness = register_simple_axiom(
        &mut api,
        &session_id,
        sort,
        "Example.Api.Strategies.nonStrictTheoryWitness",
    );

    let response = api
        .proof_try_strategies(StrategyProveRequest {
            session_id: session_id.clone(),
            expected_type: sort,
            exact_terms: Vec::new(),
            refl_terms: Vec::new(),
            split: false,
            apply: Vec::new(),
            theory: vec![TheoryStrategyCandidate {
                theory: TheoryStrategyKind::Linarith,
            }],
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
