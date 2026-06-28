use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, BatchCandidate, BatchCheckMode, BatchCheckRequest,
    BatchCheckSummary, ConstTermRequest, ExactProofRequest, SortTermRequest, StartSessionRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Batch"))
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

fn register_simple_axiom(
    api: &mut ApiService,
    session_id: &mpk_api::SessionId,
    sort: mpk_api::ApiTermId,
) -> mpk_api::ApiTermId {
    let sort_core = api
        .session(session_id)
        .and_then(|session| session.core_term_id(sort))
        .expect("sort core term is addressable");
    api.session_mut(session_id)
        .expect("session exists")
        .environment_mut()
        .register_axiom("Example.Api.Batch.trivial", sort_core)
        .expect("test axiom registers");
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: "Example.Api.Batch.trivial".to_owned(),
        levels: Vec::new(),
    })
    .expect("const term constructs")
    .term_id
}

#[test]
fn batch_returns_per_candidate_verdicts_in_order() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let proof_term = register_simple_axiom(&mut api, &session_id, sort);
    let good = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: proof_term,
            expected_type: sort,
        })
        .expect("good proof constructs")
        .proof_id;
    let bad = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: sort,
            expected_type: sort,
        })
        .expect("bad proof constructs")
        .proof_id;

    let response = api
        .vc_check_candidates(BatchCheckRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates: vec![
                BatchCandidate {
                    candidate_id: "ok".to_owned(),
                    proof_id: good,
                },
                BatchCandidate {
                    candidate_id: "bad".to_owned(),
                    proof_id: bad,
                },
                BatchCandidate {
                    candidate_id: "missing".to_owned(),
                    proof_id: ApiProofId(404),
                },
            ],
        })
        .expect("batch returns verdicts");

    assert_eq!(response.session_id, session_id);
    assert_eq!(
        response.summary,
        BatchCheckSummary {
            total: 3,
            accepted: 1,
            rejected: 2,
        }
    );
    assert_eq!(response.verdicts[0].candidate_id, "ok");
    assert!(response.verdicts[0].ok);
    assert_eq!(response.verdicts[0].term_id, Some(proof_term));
    assert_eq!(response.verdicts[1].candidate_id, "bad");
    assert!(!response.verdicts[1].ok);
    assert_eq!(
        response.verdicts[1]
            .error
            .as_ref()
            .expect("bad candidate has error")
            .code,
        ApiErrorCode::ProofCheckFailed
    );
    assert_eq!(response.verdicts[2].candidate_id, "missing");
    assert!(!response.verdicts[2].ok);
    assert_eq!(
        response.verdicts[2]
            .error
            .as_ref()
            .expect("missing candidate has error")
            .code,
        ApiErrorCode::UnknownProof
    );
}

#[test]
fn batch_handles_ten_thousand_fake_candidates() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let candidates = (0..10_000)
        .map(|index| BatchCandidate {
            candidate_id: format!("fake-{index:05}"),
            proof_id: ApiProofId(index),
        })
        .collect();

    let response = api
        .vc_check_candidates(BatchCheckRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates,
        })
        .expect("batch handles fake candidates");

    assert_eq!(
        response.summary,
        BatchCheckSummary {
            total: 10_000,
            accepted: 0,
            rejected: 10_000,
        }
    );
    assert_eq!(response.verdicts.len(), 10_000);
    assert_eq!(response.verdicts[0].candidate_id, "fake-00000");
    assert_eq!(response.verdicts[9_999].candidate_id, "fake-09999");
    assert!(response.verdicts.iter().all(|verdict| !verdict.ok));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .proof_node_count(),
        0
    );
}

#[test]
fn batch_response_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let response = api
        .vc_check_candidates(BatchCheckRequest {
            session_id,
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates: vec![BatchCandidate {
                candidate_id: "missing".to_owned(),
                proof_id: ApiProofId(404),
            }],
        })
        .expect("batch returns verdicts");

    let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "mode": "fail_fast_per_candidate",
  "summary": {
    "total": 1,
    "accepted": 0,
    "rejected": 1
  },
  "verdicts": [
    {
      "candidate_id": "missing",
      "proof_id": 404,
      "ok": false,
      "error": {
        "code": "UNKNOWN_PROOF",
        "message": "proof id 404 is not registered in this API session",
        "field": "proof_id"
      }
    }
  ]
}"#
    );
}
