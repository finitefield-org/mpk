use mpk_api::{
    ApiErrorCode, ApiProofId, ApiService, BatchCandidate, BatchCheckMode, BatchCheckSummary,
    ConstTermRequest, ExactProofRequest, JsonlExportRequest, JsonlImportRequest, SortTermRequest,
    StartSessionRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Jsonl"))
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
        .register_axiom("Example.Api.Jsonl.trivial", sort_core)
        .expect("test axiom registers");
    api.term_const(ConstTermRequest {
        session_id: session_id.clone(),
        name: "Example.Api.Jsonl.trivial".to_owned(),
        levels: Vec::new(),
    })
    .expect("const term constructs")
    .term_id
}

#[test]
fn jsonl_round_trip_feeds_batch_candidate_checking() {
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
    let candidates = vec![
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
    ];

    let exported = api
        .vc_export_candidates_jsonl(JsonlExportRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates: candidates.clone(),
        })
        .expect("JSONL export succeeds");

    assert_eq!(exported.records, 3);
    assert_eq!(
        exported.jsonl,
        "{\"candidate_id\":\"ok\",\"proof_id\":0}\n\
         {\"candidate_id\":\"bad\",\"proof_id\":1}\n\
         {\"candidate_id\":\"missing\",\"proof_id\":404}\n"
    );

    let imported = api
        .vc_import_candidates_jsonl(JsonlImportRequest {
            session_id: session_id.clone(),
            mode: BatchCheckMode::FailFastPerCandidate,
            jsonl: exported.jsonl,
        })
        .expect("JSONL import succeeds");

    assert_eq!(imported.records, 3);
    assert_eq!(imported.batch_request.session_id, session_id);
    assert_eq!(
        imported.batch_request.mode,
        BatchCheckMode::FailFastPerCandidate
    );
    assert_eq!(imported.batch_request.candidates, candidates);

    let checked = api
        .vc_check_candidates(imported.batch_request)
        .expect("imported candidates check");
    assert_eq!(
        checked.summary,
        BatchCheckSummary {
            total: 3,
            accepted: 1,
            rejected: 2,
        }
    );
}

#[test]
fn jsonl_import_skips_blank_lines() {
    let api = {
        let mut api = ApiService::new();
        start_session(&mut api);
        api
    };

    let imported = api
        .vc_import_candidates_jsonl(JsonlImportRequest {
            session_id: mpk_api::SessionId("s1".to_owned()),
            mode: BatchCheckMode::FailFastPerCandidate,
            jsonl: "\n{\"candidate_id\":\"only\",\"proof_id\":7}\n  \n".to_owned(),
        })
        .expect("JSONL import succeeds");

    assert_eq!(imported.records, 1);
    assert_eq!(
        imported.batch_request.candidates,
        vec![BatchCandidate {
            candidate_id: "only".to_owned(),
            proof_id: ApiProofId(7),
        }]
    );
}

#[test]
fn jsonl_import_rejects_invalid_records_structurally() {
    let api = {
        let mut api = ApiService::new();
        start_session(&mut api);
        api
    };

    let error = api
        .vc_import_candidates_jsonl(JsonlImportRequest {
            session_id: mpk_api::SessionId("s1".to_owned()),
            mode: BatchCheckMode::FailFastPerCandidate,
            jsonl: "{\"candidate_id\":\"ok\",\"proof_id\":0}\n{\"candidate_id\":\"bad\",\"proof_id\":\"oops\"}\n"
                .to_owned(),
        })
        .expect_err("invalid JSONL rejects");

    assert_eq!(error.code, ApiErrorCode::InvalidJsonl);
    assert_eq!(error.field.as_deref(), Some("jsonl[2]"));
    assert_eq!(
        error.detail.as_deref(),
        Some("line=1; column=39; category=data")
    );
}

#[test]
fn jsonl_export_response_serializes_stably() {
    let api = {
        let mut api = ApiService::new();
        start_session(&mut api);
        api
    };

    let response = api
        .vc_export_candidates_jsonl(JsonlExportRequest {
            session_id: mpk_api::SessionId("s1".to_owned()),
            mode: BatchCheckMode::FailFastPerCandidate,
            candidates: vec![BatchCandidate {
                candidate_id: "missing".to_owned(),
                proof_id: ApiProofId(404),
            }],
        })
        .expect("JSONL export succeeds");

    let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "mode": "fail_fast_per_candidate",
  "records": 1,
  "jsonl": "{\"candidate_id\":\"missing\",\"proof_id\":404}\n"
}"#
    );
}
