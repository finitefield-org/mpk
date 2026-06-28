//! JSONL import/export helpers for offline AI proof candidate workflows.

use serde::{Deserialize, Serialize};

use crate::{
    batch::{BatchCandidate, BatchCheckMode, BatchCheckRequest},
    session::{ApiError, ApiErrorCode, ApiService, SessionId},
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonlExportRequest {
    pub session_id: SessionId,
    #[serde(default)]
    pub mode: BatchCheckMode,
    pub candidates: Vec<BatchCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonlExportResponse {
    pub session_id: SessionId,
    pub mode: BatchCheckMode,
    pub records: usize,
    pub jsonl: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonlImportRequest {
    pub session_id: SessionId,
    #[serde(default)]
    pub mode: BatchCheckMode,
    pub jsonl: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JsonlImportResponse {
    pub records: usize,
    pub batch_request: BatchCheckRequest,
}

impl ApiService {
    pub fn vc_export_candidates_jsonl(
        &self,
        request: JsonlExportRequest,
    ) -> Result<JsonlExportResponse, ApiError> {
        let session_id = request.session_id;
        self.session(&session_id)
            .ok_or_else(|| ApiError::unknown_session(&session_id))?;
        let jsonl = export_batch_candidates_jsonl(&request.candidates)?;

        Ok(JsonlExportResponse {
            session_id,
            mode: request.mode,
            records: request.candidates.len(),
            jsonl,
        })
    }

    pub fn vc_import_candidates_jsonl(
        &self,
        request: JsonlImportRequest,
    ) -> Result<JsonlImportResponse, ApiError> {
        let session_id = request.session_id;
        self.session(&session_id)
            .ok_or_else(|| ApiError::unknown_session(&session_id))?;
        let candidates = import_batch_candidates_jsonl(&request.jsonl)?;
        let records = candidates.len();

        Ok(JsonlImportResponse {
            records,
            batch_request: BatchCheckRequest {
                session_id,
                mode: request.mode,
                candidates,
            },
        })
    }
}

pub fn export_batch_candidates_jsonl(candidates: &[BatchCandidate]) -> Result<String, ApiError> {
    let mut output = String::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let line = serde_json::to_string(candidate)
            .map_err(|error| jsonl_serialization_error(index + 1, error))?;
        output.push_str(&line);
        output.push('\n');
    }
    Ok(output)
}

pub fn import_batch_candidates_jsonl(jsonl: &str) -> Result<Vec<BatchCandidate>, ApiError> {
    let mut candidates = Vec::new();
    for (line_index, line) in jsonl.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let candidate = serde_json::from_str::<BatchCandidate>(line)
            .map_err(|error| jsonl_parse_error(line_index + 1, error))?;
        candidates.push(candidate);
    }
    Ok(candidates)
}

fn jsonl_serialization_error(line: usize, error: serde_json::Error) -> ApiError {
    ApiError::new(
        ApiErrorCode::InvalidJsonl,
        format!("failed to serialize JSONL candidate record at line {line}"),
        Some(format!("jsonl[{line}]")),
        Some(jsonl_error_detail(&error)),
    )
}

fn jsonl_parse_error(line: usize, error: serde_json::Error) -> ApiError {
    ApiError::new(
        ApiErrorCode::InvalidJsonl,
        format!("invalid JSONL candidate record at line {line}"),
        Some(format!("jsonl[{line}]")),
        Some(jsonl_error_detail(&error)),
    )
}

fn jsonl_error_detail(error: &serde_json::Error) -> String {
    format!(
        "line={}; column={}; category={}",
        error.line(),
        error.column(),
        jsonl_error_category(error.classify())
    )
}

fn jsonl_error_category(category: serde_json::error::Category) -> &'static str {
    match category {
        serde_json::error::Category::Io => "io",
        serde_json::error::Category::Syntax => "syntax",
        serde_json::error::Category::Data => "data",
        serde_json::error::Category::Eof => "eof",
    }
}
