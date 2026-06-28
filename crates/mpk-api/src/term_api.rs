//! Term construction endpoints for the local AI proof API.

use mpk_core::LevelId;
use serde::{Deserialize, Serialize};

use crate::session::{ApiError, ApiErrorCode, ApiService, ApiSession, SessionId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ApiTermId(pub u32);

impl ApiTermId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SortTermRequest {
    pub session_id: SessionId,
    pub universe: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VarTermRequest {
    pub session_id: SessionId,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConstTermRequest {
    pub session_id: SessionId,
    pub name: String,
    #[serde(default)]
    pub levels: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppTermRequest {
    pub session_id: SessionId,
    pub function: ApiTermId,
    #[serde(default)]
    pub arguments: Vec<ApiTermId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BinderTermRequest {
    pub session_id: SessionId,
    pub ty: ApiTermId,
    pub body: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LetTermRequest {
    pub session_id: SessionId,
    pub ty: ApiTermId,
    pub value: ApiTermId,
    pub body: ApiTermId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TermResponse {
    pub session_id: SessionId,
    pub term_id: ApiTermId,
}

impl ApiService {
    pub fn term_sort(&mut self, request: SortTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let universe = zero_universe(session, request.universe, "universe")?;
        let term = session.terms_mut().sort(universe);
        response(session, session_id, term)
    }

    pub fn term_var(&mut self, request: VarTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let term = session.terms_mut().var(request.index);
        response(session, session_id, term)
    }

    pub fn term_const(&mut self, request: ConstTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let global = session
            .environment()
            .resolve(&request.name)
            .map_err(|error| {
                ApiError::new(
                    ApiErrorCode::InvalidGlobalName,
                    format!("{:?} is not a valid global name", request.name),
                    Some("name".to_owned()),
                    Some(error.to_deterministic_json()),
                )
            })?
            .ok_or_else(|| {
                ApiError::new(
                    ApiErrorCode::UnknownGlobal,
                    format!(
                        "global name {:?} is not registered in this API session",
                        request.name
                    ),
                    Some("name".to_owned()),
                    None,
                )
            })?;
        let levels = request
            .levels
            .iter()
            .enumerate()
            .map(|(index, universe)| zero_universe(session, *universe, format!("levels[{index}]")))
            .collect::<Result<Vec<_>, _>>()?;
        let term = session.terms_mut().constant(global, levels);
        response(session, session_id, term)
    }

    pub fn term_app(&mut self, request: AppTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let function = session.require_term_id(request.function, "function")?;
        let arguments = request
            .arguments
            .iter()
            .enumerate()
            .map(|(index, term_id)| {
                session.require_term_id(*term_id, format!("arguments[{index}]"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let term = session.terms_mut().app(function, arguments);
        response(session, session_id, term)
    }

    pub fn term_lam(&mut self, request: BinderTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let ty = session.require_term_id(request.ty, "ty")?;
        let body = session.require_term_id(request.body, "body")?;
        let term = session.terms_mut().lam(ty, body);
        response(session, session_id, term)
    }

    pub fn term_pi(&mut self, request: BinderTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let ty = session.require_term_id(request.ty, "ty")?;
        let body = session.require_term_id(request.body, "body")?;
        let term = session.terms_mut().pi(ty, body);
        response(session, session_id, term)
    }

    pub fn term_let(&mut self, request: LetTermRequest) -> Result<TermResponse, ApiError> {
        let session_id = request.session_id;
        let session = self.require_session_mut(&session_id)?;
        let ty = session.require_term_id(request.ty, "ty")?;
        let value = session.require_term_id(request.value, "value")?;
        let body = session.require_term_id(request.body, "body")?;
        let term = session.terms_mut().let_term(ty, value, body);
        response(session, session_id, term)
    }

    fn require_session_mut(&mut self, session_id: &SessionId) -> Result<&mut ApiSession, ApiError> {
        self.session_mut(session_id)
            .ok_or_else(|| ApiError::unknown_session(session_id))
    }
}

fn zero_universe(
    session: &ApiSession,
    universe: u32,
    field: impl Into<String>,
) -> Result<LevelId, ApiError> {
    if universe == 0 {
        return Ok(session.levels().zero());
    }

    Err(ApiError::new(
        ApiErrorCode::UnsupportedUniverseLevel,
        "API term construction currently supports only universe level 0",
        Some(field.into()),
        Some(universe.to_string()),
    ))
}

fn response(
    session: &mut ApiSession,
    session_id: SessionId,
    term: mpk_core::TermId,
) -> Result<TermResponse, ApiError> {
    Ok(TermResponse {
        session_id,
        term_id: session.register_term_id(term)?,
    })
}
