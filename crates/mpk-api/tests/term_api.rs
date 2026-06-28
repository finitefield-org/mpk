use mpk_api::{
    ApiErrorCode, ApiService, ApiTermId, AppTermRequest, BinderTermRequest, ConstTermRequest,
    LetTermRequest, SortTermRequest, StartSessionRequest, VarTermRequest,
};

fn start_session(api: &mut ApiService) -> mpk_api::SessionId {
    api.start_session(StartSessionRequest::new("Example.Api.Terms"))
        .expect("session starts")
        .session_id
}

#[test]
fn constructs_terms_over_interned_ids() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);

    let sort = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term constructs");
    let sort_again = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 0,
        })
        .expect("sort term reuses interned core term");
    assert_eq!(sort_again.term_id, sort.term_id);

    let var0 = api
        .term_var(VarTermRequest {
            session_id: session_id.clone(),
            index: 0,
        })
        .expect("var term constructs");

    let sort_core = api
        .session(&session_id)
        .and_then(|session| session.core_term_id(sort.term_id))
        .expect("sort core term is addressable");
    api.session_mut(&session_id)
        .expect("session exists")
        .environment_mut()
        .register_axiom("Example.Api.Const", sort_core)
        .expect("test constant registers");

    let constant = api
        .term_const(ConstTermRequest {
            session_id: session_id.clone(),
            name: "Example.Api.Const".to_owned(),
            levels: Vec::new(),
        })
        .expect("const term constructs");

    let app = api
        .term_app(AppTermRequest {
            session_id: session_id.clone(),
            function: var0.term_id,
            arguments: vec![constant.term_id],
        })
        .expect("app term constructs");
    let app_again = api
        .term_app(AppTermRequest {
            session_id: session_id.clone(),
            function: var0.term_id,
            arguments: vec![constant.term_id],
        })
        .expect("app term reuses interned core term");
    assert_eq!(app_again.term_id, app.term_id);

    let lam = api
        .term_lam(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort.term_id,
            body: var0.term_id,
        })
        .expect("lambda term constructs");
    let pi = api
        .term_pi(BinderTermRequest {
            session_id: session_id.clone(),
            ty: sort.term_id,
            body: sort.term_id,
        })
        .expect("pi term constructs");
    let let_term = api
        .term_let(LetTermRequest {
            session_id: session_id.clone(),
            ty: sort.term_id,
            value: constant.term_id,
            body: var0.term_id,
        })
        .expect("let term constructs");

    assert!(lam.term_id.as_u32() > app.term_id.as_u32());
    assert!(pi.term_id.as_u32() > lam.term_id.as_u32());
    assert!(let_term.term_id.as_u32() > pi.term_id.as_u32());
}

#[test]
fn term_app_with_no_arguments_returns_function_id() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let var0 = api
        .term_var(VarTermRequest {
            session_id: session_id.clone(),
            index: 0,
        })
        .expect("var term constructs");

    let app = api
        .term_app(AppTermRequest {
            session_id,
            function: var0.term_id,
            arguments: Vec::new(),
        })
        .expect("empty app constructs");

    assert_eq!(app.term_id, var0.term_id);
}

#[test]
fn term_api_returns_structured_errors() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let initial_term_count = api
        .session(&session_id)
        .expect("session exists")
        .terms()
        .len();

    let unsupported_level = api
        .term_sort(SortTermRequest {
            session_id: session_id.clone(),
            universe: 1,
        })
        .expect_err("nonzero universe rejects");
    assert_eq!(
        unsupported_level.code,
        ApiErrorCode::UnsupportedUniverseLevel
    );
    assert_eq!(unsupported_level.field.as_deref(), Some("universe"));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .terms()
            .len(),
        initial_term_count
    );

    let unknown_term = api
        .term_app(AppTermRequest {
            session_id: session_id.clone(),
            function: ApiTermId(404),
            arguments: Vec::new(),
        })
        .expect_err("unknown function term rejects");
    assert_eq!(unknown_term.code, ApiErrorCode::UnknownTerm);
    assert_eq!(unknown_term.field.as_deref(), Some("function"));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .terms()
            .len(),
        initial_term_count
    );

    let unknown_global = api
        .term_const(ConstTermRequest {
            session_id: session_id.clone(),
            name: "Example.Api.Missing".to_owned(),
            levels: Vec::new(),
        })
        .expect_err("unknown global rejects");
    assert_eq!(unknown_global.code, ApiErrorCode::UnknownGlobal);
    assert_eq!(unknown_global.field.as_deref(), Some("name"));
    assert_eq!(
        api.session(&session_id)
            .expect("session exists")
            .terms()
            .len(),
        initial_term_count
    );

    let unknown_session = api
        .term_var(VarTermRequest {
            session_id: mpk_api::SessionId("s404".to_owned()),
            index: 0,
        })
        .expect_err("unknown session rejects");
    assert_eq!(unknown_session.code, ApiErrorCode::UnknownSession);
}

#[test]
fn term_response_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let response = api
        .term_var(VarTermRequest {
            session_id,
            index: 2,
        })
        .expect("var term constructs");

    let encoded = serde_json::to_string_pretty(&response).expect("response serializes");

    assert_eq!(
        encoded,
        r#"{
  "session_id": "s1",
  "term_id": 0
}"#
    );
}

#[test]
fn term_error_serializes_stably() {
    let mut api = ApiService::new();
    let session_id = start_session(&mut api);
    let error = api
        .term_sort(SortTermRequest {
            session_id,
            universe: 1,
        })
        .expect_err("nonzero universe rejects");

    let encoded = serde_json::to_string_pretty(&error).expect("error serializes");

    assert_eq!(
        encoded,
        r#"{
  "code": "UNSUPPORTED_UNIVERSE_LEVEL",
  "message": "API term construction currently supports only universe level 0",
  "field": "universe",
  "detail": "1"
}"#
    );
}
