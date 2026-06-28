use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, ApiTermId, ApplyStrategyCandidate, BinderTermRequest,
    ConstTermRequest, ExactProofRequest, SortTermRequest, StartSessionRequest, StrategyKind,
    StrategyProveRequest, VarTermRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Strategies"))
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
