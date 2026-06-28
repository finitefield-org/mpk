use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, ApiTermId, ApplyProofRequest, BinderTermRequest,
    CheckNodeRequest, CheckNodeResponse, ConstTermRequest, ConvProofRequest, ExactProofRequest,
    IntroProofRequest, ReflProofRequest, SortTermRequest, StartSessionRequest, VarTermRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Check"))
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
) -> ApiTermId {
    let sort_core = api
        .session(session_id)
        .and_then(|session| session.core_term_id(sort))
        .expect("sort core term is addressable");
    api.session_mut(session_id)
        .expect("session exists")
        .environment_mut()
        .register_axiom("Example.Api.Check.trivial", sort_core)
        .expect("test axiom registers");
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: "Example.Api.Check.trivial".to_owned(),
        levels: Vec::new(),
    })
    .expect("const term constructs")
    .term_id
}

#[test]
fn check_node_accepts_simple_exact_proof() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let proof_term = register_simple_axiom(&mut api, &session_id, sort);
    let proof = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort,
        })
        .expect("exact proof constructs");

    let checked = api
        .proof_check_node(CheckNodeRequest {
            session_id: session_id.clone(),
            proof_id: proof.proof_id,
        })
        .expect("proof checks");

    assert_eq!(
        checked,
        CheckNodeResponse {
            session_id,
            proof_id: proof.proof_id,
            ok: true,
            term_id: proof_term,
        }
    );
}

#[test]
fn check_node_validates_core_bootstrap_proofs_recursively() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let proof_term = register_simple_axiom(&mut api, &session_id, sort);
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
    let exact_const = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort,
        })
        .expect("exact proof constructs")
        .proof_id;
    let exact_lam = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: lam,
            expected_type: pi,
        })
        .expect("lambda exact proof constructs")
        .proof_id;
    let apply = api
        .proof_apply(ApplyProofRequest {
            session_id: session_id.clone(),
            function_proof: exact_lam,
            argument_proofs: vec![exact_const],
            expected_type: sort,
        })
        .expect("apply proof constructs")
        .proof_id;
    let exact_var = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: var0,
            expected_type: sort,
        })
        .expect("local exact proof constructs")
        .proof_id;
    let intro = api
        .proof_intro(IntroProofRequest {
            session_id: session_id.clone(),
            domain_type: sort,
            body_proof: exact_var,
            expected_type: pi,
        })
        .expect("intro proof constructs")
        .proof_id;
    let refl = api
        .proof_refl(ReflProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort,
        })
        .expect("refl proof constructs")
        .proof_id;
    let conv = api
        .proof_conv(ConvProofRequest {
            session_id: session_id.clone(),
            proof: exact_const,
            expected_type: sort,
            defeq_witness: Some(sort),
        })
        .expect("conv proof constructs")
        .proof_id;

    for proof_id in [apply, intro, refl, conv] {
        let checked = api
            .proof_check_node(CheckNodeRequest {
                session_id: session_id.clone(),
                proof_id,
            })
            .expect("proof checks");
        assert!(checked.ok);
        assert_eq!(checked.proof_id, proof_id);
    }
}

#[test]
fn bad_nodes_return_structured_errors_without_mutating_proof_table() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let bad = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: sort,
            expected_type: sort,
        })
        .expect("bad exact proof can be constructed");
    let initial_proof_count = api
        .session(&session_id)
        .expect("session exists")
        .proof_node_count();

    let error = api
        .proof_check_node(CheckNodeRequest {
            session_id: session_id.clone(),
            proof_id: bad.proof_id,
        })
        .expect_err("bad proof rejects");

    assert_eq!(error.code, ApiErrorCode::ProofCheckFailed);
    assert_eq!(error.field.as_deref(), Some("proof_id"));
    assert_eq!(
        error.message,
        "proof id 0 failed: core checking failed while checking proof node"
    );
    assert_eq!(
        error.detail.as_deref(),
        Some(
            r#"{"code":"CORE_TYPE_MISMATCH","location":[{"field":"check"}],"details":{"expected_kind":"sort","expected_term_index":"0","inferred_kind":"sort","inferred_term_index":"1","kind":"check_type_mismatch","term_index":"0"}}"#
        )
    );
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        initial_proof_count
    );

    let unknown = api
        .proof_check_node(CheckNodeRequest {
            session_id,
            proof_id: ApiProofId(404),
        })
        .expect_err("unknown proof rejects");
    assert_eq!(unknown.code, ApiErrorCode::UnknownProof);
    assert_eq!(unknown.field.as_deref(), Some("proof_id"));
}

#[test]
fn check_node_response_serializes_stably() {
    let response = CheckNodeResponse {
        session_id: mpk_api::SessionId("s1".to_owned()),
        proof_id: ApiProofId(2),
        ok: true,
        term_id: ApiTermId(9),
    };

    let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "proof_id": 2,
  "ok": true,
  "term_id": 9
}"#
    );
}

#[test]
fn check_node_error_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let error = api
        .proof_check_node(CheckNodeRequest {
            session_id,
            proof_id: ApiProofId(404),
        })
        .expect_err("unknown proof rejects");

    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");

    assert_eq!(
        encoded,
        r#"{
  "code": "UNKNOWN_PROOF",
  "message": "proof id 404 is not registered in this API session",
  "field": "proof_id"
}"#
    );
}
