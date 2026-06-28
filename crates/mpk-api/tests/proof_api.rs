use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, ApiTermId, ApplyProofRequest, BinderTermRequest,
    ConstTermRequest, ConvProofRequest, ExactProofRequest, IntroProofRequest, ProofResponse,
    ReflProofRequest, SortTermRequest, StartSessionRequest, VarTermRequest,
};
use mpk_cert::encode::ProofNode;

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Proofs"))
        .expect("session starts")
        .session_id
}

fn register_simple_axiom(api: &mut ApiService, session_id: &mpk_api::SessionId) -> ApiTermId {
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let sort_core = api
        .session(session_id)
        .and_then(|session| session.core_term_id(sort.term_id))
        .expect("sort core term is addressable");
    api.session_mut(session_id)
        .expect("session exists")
        .environment_mut()
        .register_axiom("Example.Api.Proofs.trivial", sort_core)
        .expect("test axiom registers");
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: "Example.Api.Proofs.trivial".to_owned(),
        levels: Vec::new(),
    })
    .expect("const term constructs")
    .term_id
}

#[test]
fn proof_api_builds_simple_theorem_root() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let proof_term = register_simple_axiom(&mut api, &session_id);

    let proof = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort.term_id,
        })
        .expect("exact proof constructs");

    assert_eq!(proof.proof_id, ApiProofId(0));
    assert_eq!(
        api.session(&session_id)
            .and_then(|session| session.proof_node(proof.proof_id)),
        Some(&ProofNode::Exact {
            term: proof_term.as_u32(),
            expected_type: sort.term_id.as_u32(),
        })
    );
}

#[test]
fn constructs_core_bootstrap_proofs_over_ids() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let proof_term = register_simple_axiom(&mut api, &session_id);
    let var0 = api
        .term_var(VarTermRequest {
            session_id: session_id.clone(),
            index: 0,
        })
        .expect("var term constructs");
    let pi = api
        .term_pi(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort.term_id,
            body: sort.term_id,
        })
        .expect("pi term constructs");
    let lam = api
        .term_lam(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort.term_id,
            body: var0.term_id,
        })
        .expect("lambda term constructs");

    let exact_const = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort.term_id,
        })
        .expect("exact proof constructs");
    let exact_lam = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: lam.term_id,
            expected_type: pi.term_id,
        })
        .expect("lambda exact proof constructs");
    let apply = api
        .proof_apply(ApplyProofRequest {
            session_id: session_id.clone(),
            function_proof: exact_lam.proof_id,
            argument_proofs: vec![exact_const.proof_id],
            expected_type: sort.term_id,
        })
        .expect("apply proof constructs");
    let exact_var = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: var0.term_id,
            expected_type: sort.term_id,
        })
        .expect("local exact proof constructs");
    let intro = api
        .proof_intro(IntroProofRequest {
            session_id: session_id.clone(),
            domain_type: sort.term_id,
            body_proof: exact_var.proof_id,
            expected_type: pi.term_id,
        })
        .expect("intro proof constructs");
    let refl = api
        .proof_refl(ReflProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort.term_id,
        })
        .expect("refl proof constructs");
    let conv = api
        .proof_conv(ConvProofRequest {
            session_id: session_id.clone(),
            proof: exact_const.proof_id,
            expected_type: sort.term_id,
            defeq_witness: Some(sort.term_id),
        })
        .expect("conv proof constructs");

    assert_eq!(exact_const.proof_id, ApiProofId(0));
    assert_eq!(exact_lam.proof_id, ApiProofId(1));
    assert_eq!(apply.proof_id, ApiProofId(2));
    assert_eq!(exact_var.proof_id, ApiProofId(3));
    assert_eq!(intro.proof_id, ApiProofId(4));
    assert_eq!(refl.proof_id, ApiProofId(5));
    assert_eq!(conv.proof_id, ApiProofId(6));
    let session = api.session(&session_id).expect("session exists");
    assert_eq!(session.proof_node_count(), 7);
    assert_eq!(
        session.proof_node(apply.proof_id),
        Some(&ProofNode::Apply {
            function_proof: exact_lam.proof_id.as_u32(),
            argument_proofs: vec![exact_const.proof_id.as_u32()],
            expected_type: sort.term_id.as_u32(),
        })
    );
    assert_eq!(
        session.proof_node(intro.proof_id),
        Some(&ProofNode::Intro {
            domain_type: sort.term_id.as_u32(),
            body_proof: exact_var.proof_id.as_u32(),
            expected_type: pi.term_id.as_u32(),
        })
    );
    assert_eq!(
        session.proof_node(refl.proof_id),
        Some(&ProofNode::Refl {
            term: proof_term.as_u32(),
            expected_type: sort.term_id.as_u32(),
        })
    );
    assert_eq!(
        session.proof_node(conv.proof_id),
        Some(&ProofNode::Conv {
            proof: exact_const.proof_id.as_u32(),
            expected_type: sort.term_id.as_u32(),
            defeq_witness: Some(sort.term_id.as_u32()),
        })
    );
}

#[test]
fn proof_api_returns_structured_errors_without_mutating_table() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let initial_proof_count = api
        .session(&session_id)
        .expect("session exists")
        .proof_node_count();

    let unknown_term = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: ApiTermId(404),
            expected_type: sort.term_id,
        })
        .expect_err("unknown term rejects");
    assert_eq!(unknown_term.code, ApiErrorCode::UnknownTerm);
    assert_eq!(unknown_term.field.as_deref(), Some("term"));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        initial_proof_count
    );

    let unknown_proof = api
        .proof_apply(ApplyProofRequest {
            session_id: session_id.clone(),
            function_proof: ApiProofId(404),
            argument_proofs: Vec::new(),
            expected_type: sort.term_id,
        })
        .expect_err("unknown proof rejects");
    assert_eq!(unknown_proof.code, ApiErrorCode::UnknownProof);
    assert_eq!(unknown_proof.field.as_deref(), Some("function_proof"));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        initial_proof_count
    );

    let unknown_session = api
        .proof_refl(ReflProofRequest {
            session_id: mpk_api::SessionId("s404".to_owned()),
            term: sort.term_id,
            expected_type: sort.term_id,
        })
        .expect_err("unknown session rejects");
    assert_eq!(unknown_session.code, ApiErrorCode::UnknownSession);
}

#[test]
fn proof_response_serializes_stably() {
    let response = ProofResponse {
        session_id: mpk_api::SessionId("s1".to_owned()),
        proof_id: ApiProofId(2),
    };

    let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "proof_id": 2
}"#
    );
}

#[test]
fn proof_error_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let error = api
        .proof_apply(ApplyProofRequest {
            session_id,
            function_proof: ApiProofId(404),
            argument_proofs: Vec::new(),
            expected_type: sort.term_id,
        })
        .expect_err("unknown proof rejects");

    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");

    assert_eq!(
        encoded,
        r#"{
  "code": "UNKNOWN_PROOF",
  "message": "proof id 404 is not registered in this API session",
  "field": "function_proof"
}"#
    );
}
