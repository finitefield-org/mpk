use mpk_api::{
    ApiProofId, ApiService, BinderTermRequest, ExactProofRequest, RepairDiagnosticRequest,
    SortTermRequest, StartSessionRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Diagnostics"))
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
fn rejected_exact_diagnostic_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let bad = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: sort,
            expected_type: sort,
        })
        .expect("bad exact proof constructs");

    let response = api
        .proof_repair_diagnostics(RepairDiagnosticRequest {
            session_id,
            proof_id: bad.proof_id,
        })
        .expect("diagnostic returns");

    let encoded = serde_json::to_string_pretty(&response).expect("diagnostic serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "diagnostic": {
    "ok": false,
    "error_code": "CORE_TYPE_MISMATCH",
    "node_id": 0,
    "expected_type_id": 0,
    "actual_type_id": 1,
    "expected_head": "sort",
    "actual_head": "sort",
    "context_summary": [],
    "repair_hints": [
      "exact",
      "conv"
    ]
  }
}"#
    );
}

#[test]
fn rejected_intro_body_diagnostic_includes_local_context() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let sort = sort_zero(&mut api, &session_id);
    let pi = api
        .term_pi(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort,
            body: sort,
        })
        .expect("pi term constructs")
        .term_id;
    let bad_body = api
        .proof_exact(ExactProofRequest {
            session_id: session_id.clone(),
            term: sort,
            expected_type: sort,
        })
        .expect("bad body proof constructs")
        .proof_id;
    let intro = api
        .proof_intro(mpk_api::IntroProofRequest {
            session_id: session_id.clone(),
            domain_type: sort,
            body_proof: bad_body,
            expected_type: pi,
        })
        .expect("intro proof constructs")
        .proof_id;

    let response = api
        .proof_repair_diagnostics(RepairDiagnosticRequest {
            session_id,
            proof_id: intro,
        })
        .expect("diagnostic returns");

    let encoded = serde_json::to_string_pretty(&response).expect("diagnostic serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "diagnostic": {
    "ok": false,
    "error_code": "CORE_TYPE_MISMATCH",
    "node_id": 0,
    "expected_type_id": 0,
    "actual_type_id": 2,
    "expected_head": "sort",
    "actual_head": "sort",
    "context_summary": [
      0
    ],
    "repair_hints": [
      "exact",
      "conv"
    ]
  }
}"#
    );
}

#[test]
fn unknown_proof_diagnostic_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);

    let response = api
        .proof_repair_diagnostics(RepairDiagnosticRequest {
            session_id,
            proof_id: ApiProofId(404),
        })
        .expect("diagnostic returns");

    let encoded = serde_json::to_string_pretty(&response).expect("diagnostic serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "diagnostic": {
    "ok": false,
    "error_code": "UNKNOWN_PROOF",
    "node_id": 404,
    "context_summary": [],
    "repair_hints": [
      "select-existing-proof"
    ]
  }
}"#
    );
}
