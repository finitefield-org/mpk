//! Credential and transport boundaries for the optional Vertex AI explainer.
//!
//! The request body is owned by [`crate::ai_explain::ExplainPreparedRequest`]
//! and is deliberately treated as opaque here.  This module is responsible
//! for authentication, the fixed Vertex endpoint, HTTP policy, retries, and
//! provider-envelope checks.  It does not call the provider from tests and it
//! does not validate the model-generated explanation itself.

use std::error::Error as StdError;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_LENGTH, RETRY_AFTER};
use reqwest::redirect::Policy;
use wait_timeout::ChildExt;

use crate::ai_explain::{
    validate_model_id, AiExplainError, AiExplainErrorCode, ExplainPreparedRequest,
    VertexGenerateResponse,
};

/// The stable error type used by the Vertex authentication and transport
/// boundaries.  Keeping the existing error type avoids a second error-code
/// serialization contract.
pub type VertexAiError = AiExplainError;

const GCLOUD_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const TOTAL_REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_TOKEN_OUTPUT_BYTES: usize = 16 * 1024;
const MAX_SUCCESS_BODY_BYTES: usize = 1024 * 1024;
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;
const MAX_ATTEMPTS: u8 = 3;
const MAX_RETRY_AFTER: u64 = 10;
const RETRY_DELAY_ATTEMPT_TWO: Duration = Duration::from_millis(250);
const RETRY_DELAY_ATTEMPT_THREE: Duration = Duration::from_secs(1);

const AUTH_UNAVAILABLE_DETAIL: &str = "gcloud could not be started";
const AUTH_FAILED_DETAIL: &str = "application default access token was unavailable";
const CONFIG_INVALID_DETAIL: &str = "Vertex configuration is not allowed";
const PROTOCOL_ERROR_DETAIL: &str = "provider response did not match the Vertex envelope";

fn error(code: AiExplainErrorCode, detail: &'static str) -> VertexAiError {
    AiExplainError::new(code, detail)
}

/// An ADC bearer token whose formatting is checked at construction and whose
/// debug representation is always redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretAccessToken(String);

impl SecretAccessToken {
    /// Construct a token for a fake provider or another reviewed token source.
    /// The value is never included in a returned error.
    pub fn new(value: impl Into<String>) -> Result<Self, VertexAiError> {
        let value = value.into();
        if validate_token68(value.as_bytes(), MAX_TOKEN_OUTPUT_BYTES) {
            Ok(Self(value))
        } else {
            Err(error(
                AiExplainErrorCode::VertexAuthFailed,
                AUTH_FAILED_DETAIL,
            ))
        }
    }

    fn parse_output(output: &[u8]) -> Result<Self, VertexAiError> {
        if output.len() > MAX_TOKEN_OUTPUT_BYTES {
            return Err(error(
                AiExplainErrorCode::VertexAuthFailed,
                AUTH_FAILED_DETAIL,
            ));
        }

        let value = if output.ends_with(b"\r\n") {
            &output[..output.len() - 2]
        } else if output.ends_with(b"\n") {
            &output[..output.len() - 1]
        } else {
            output
        };
        if std::str::from_utf8(value).is_err() || !validate_token68(value, MAX_TOKEN_OUTPUT_BYTES) {
            return Err(error(
                AiExplainErrorCode::VertexAuthFailed,
                AUTH_FAILED_DETAIL,
            ));
        }
        // The UTF-8 check above also means this conversion cannot expose an
        // arbitrary byte sequence in a diagnostic.
        Ok(Self(String::from_utf8(value.to_vec()).map_err(|_| {
            error(AiExplainErrorCode::VertexAuthFailed, AUTH_FAILED_DETAIL)
        })?))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretAccessToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretAccessToken(<redacted>)")
    }
}

fn validate_token68(value: &[u8], max_len: usize) -> bool {
    if value.is_empty() || value.len() > max_len {
        return false;
    }
    let mut has_base_character = false;
    let mut padding_started = false;
    for &byte in value {
        if byte == b'=' {
            padding_started = true;
        } else if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
        {
            if padding_started {
                return false;
            }
            has_base_character = true;
        } else {
            return false;
        }
    }
    has_base_character
}

/// Provides a short-lived ADC bearer token without exposing the credential
/// source or token value to callers.
pub trait AccessTokenProvider: Send + Sync {
    fn access_token(&self) -> Result<SecretAccessToken, VertexAiError>;
}

/// Local-development ADC provider.  The executable is selected as a path;
/// its argument vector is fixed and is never interpreted by a shell.
pub struct GcloudAccessTokenProvider {
    executable: PathBuf,
    timeout: Duration,
}

impl GcloudAccessTokenProvider {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self::with_timeout(executable, GCLOUD_TIMEOUT)
    }

    pub fn with_timeout(executable: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            executable: executable.into(),
            timeout,
        }
    }
}

impl AccessTokenProvider for GcloudAccessTokenProvider {
    fn access_token(&self) -> Result<SecretAccessToken, VertexAiError> {
        let mut child = Command::new(&self.executable)
            .args([
                "auth",
                "application-default",
                "print-access-token",
                "--quiet",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| {
                error(
                    AiExplainErrorCode::VertexAuthUnavailable,
                    AUTH_UNAVAILABLE_DETAIL,
                )
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            terminate_and_reap(&mut child);
            error(AiExplainErrorCode::VertexAuthFailed, AUTH_FAILED_DETAIL)
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            terminate_and_reap(&mut child);
            error(AiExplainErrorCode::VertexAuthFailed, AUTH_FAILED_DETAIL)
        })?;

        let stdout_reader = thread::spawn(|| drain_bounded(stdout));
        let stderr_reader = thread::spawn(|| drain_bounded(stderr));

        let status = match child.wait_timeout(self.timeout) {
            Ok(Some(status)) => status,
            Ok(None) | Err(_) => {
                terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(error(
                    AiExplainErrorCode::VertexAuthFailed,
                    AUTH_FAILED_DETAIL,
                ));
            }
        };

        let stdout = match stdout_reader.join() {
            Ok(Ok(bytes)) => bytes,
            _ => {
                let _ = stderr_reader.join();
                return Err(error(
                    AiExplainErrorCode::VertexAuthFailed,
                    AUTH_FAILED_DETAIL,
                ));
            }
        };
        let stderr_ok = matches!(stderr_reader.join(), Ok(Ok(_)));
        if !status.success() || !stderr_ok {
            return Err(error(
                AiExplainErrorCode::VertexAuthFailed,
                AUTH_FAILED_DETAIL,
            ));
        }
        SecretAccessToken::parse_output(&stdout)
    }
}

fn drain_bounded<R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut retained = Vec::with_capacity(MAX_TOKEN_OUTPUT_BYTES.min(4096));
    let mut buffer = [0_u8; 4096];
    let mut oversized = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = MAX_TOKEN_OUTPUT_BYTES.saturating_sub(retained.len());
        if read <= remaining {
            retained.extend_from_slice(&buffer[..read]);
        } else {
            retained.extend_from_slice(&buffer[..remaining]);
            oversized = true;
        }
    }
    if oversized {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bounded child output exceeded its limit",
        ))
    } else {
        Ok(retained)
    }
}

fn terminate_and_reap(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn valid_project_id(project: &str) -> bool {
    (6..=30).contains(&project.len())
        && project
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
        && project
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && project
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_location(location: &str) -> bool {
    !location.is_empty()
        && location.len() + "-aiplatform".len() <= 63
        && location
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && location
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && location
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Build the only endpoint shape accepted by this transport.
pub fn build_vertex_endpoint(
    project: &str,
    location: &str,
    model: &str,
) -> Result<String, VertexAiError> {
    if !valid_project_id(project) || !valid_location(location) || validate_model_id(model).is_err()
    {
        return Err(error(
            AiExplainErrorCode::VertexConfigInvalid,
            CONFIG_INVALID_DETAIL,
        ));
    }

    if location == "global" {
        Ok(format!(
            "https://aiplatform.googleapis.com/v1/projects/{project}/locations/global/publishers/google/models/{model}:generateContent"
        ))
    } else {
        Ok(format!(
            "https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent"
        ))
    }
}

struct HttpRequest {
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

struct HttpResponse {
    status: u16,
    retry_after: Option<String>,
    body: Vec<u8>,
    body_too_large: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpExecutorError {
    ConnectBeforeSendTimeout,
    ResponseTimeout,
    Transport,
}

trait HttpExecutor: Send + Sync {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpExecutorError>;
}

struct ReqwestHttpExecutor {
    client: Client,
}

impl ReqwestHttpExecutor {
    fn new() -> Result<Self, VertexAiError> {
        let client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .retry(reqwest::retry::never())
            .http1_only()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TOTAL_REQUEST_TIMEOUT)
            .build()
            .map_err(|_| {
                error(
                    AiExplainErrorCode::VertexTransportFailed,
                    "Vertex HTTP client could not be initialized",
                )
            })?;
        Ok(Self { client })
    }
}

impl HttpExecutor for ReqwestHttpExecutor {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpExecutorError> {
        let mut builder = self.client.post(&request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name, value);
        }

        let mut response = builder
            .body(request.body.clone())
            .send()
            .map_err(|request_error| {
                if request_error.is_timeout() {
                    if request_error.is_connect() {
                        HttpExecutorError::ConnectBeforeSendTimeout
                    } else {
                        HttpExecutorError::ResponseTimeout
                    }
                } else {
                    HttpExecutorError::Transport
                }
            })?;
        read_http_response(&mut response)
    }
}

fn read_http_response(response: &mut Response) -> Result<HttpResponse, HttpExecutorError> {
    let status = response.status().as_u16();
    let declared_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let retry_after = retry_after_header(response);
    read_bounded_http_response(
        status,
        declared_length,
        retry_after,
        response,
        |read_error| {
            if is_timeout_io_error(&read_error) {
                HttpExecutorError::ResponseTimeout
            } else {
                HttpExecutorError::Transport
            }
        },
    )
}

fn read_bounded_http_response<R, F>(
    status: u16,
    declared_length: Option<u64>,
    retry_after: Option<String>,
    reader: &mut R,
    map_read_error: F,
) -> Result<HttpResponse, HttpExecutorError>
where
    R: Read,
    F: Fn(io::Error) -> HttpExecutorError,
{
    let limit = if status == 200 {
        MAX_SUCCESS_BODY_BYTES
    } else {
        MAX_ERROR_BODY_BYTES
    };
    if declared_length.is_some_and(|length| length > limit as u64) {
        return Ok(HttpResponse {
            status,
            retry_after,
            body: Vec::new(),
            body_too_large: true,
        });
    }

    let mut body = Vec::with_capacity(declared_length.unwrap_or(0).min(limit as u64) as usize);
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(&map_read_error)?;
        if read == 0 {
            break;
        }
        if body.len().saturating_add(read) > limit {
            return Ok(HttpResponse {
                status,
                retry_after,
                body: Vec::new(),
                body_too_large: true,
            });
        }
        body.extend_from_slice(&buffer[..read]);
    }

    Ok(HttpResponse {
        status,
        retry_after,
        body,
        body_too_large: false,
    })
}

fn is_timeout_io_error(error: &io::Error) -> bool {
    if error.kind() == io::ErrorKind::TimedOut {
        return true;
    }
    let mut source = error.source();
    while let Some(source_error) = source {
        if source_error
            .downcast_ref::<reqwest::Error>()
            .is_some_and(|request_error| request_error.is_timeout())
        {
            return true;
        }
        if source_error
            .downcast_ref::<io::Error>()
            .is_some_and(is_timeout_io_error)
        {
            return true;
        }
        source = source_error.source();
    }
    false
}

fn retry_after_header(response: &Response) -> Option<String> {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

/// The transport sends the exact bytes held by the prepared AUX-02 request.
pub trait VertexTransport: Send + Sync {
    fn generate(
        &self,
        request: &ExplainPreparedRequest,
        token: &SecretAccessToken,
    ) -> Result<VertexGenerateResponse, VertexAiError>;
}

/// Fixed-endpoint Vertex transport using reqwest's blocking client.
pub struct ReqwestVertexTransport {
    project: String,
    endpoint: String,
    executor: Arc<dyn HttpExecutor>,
    sleeper: Arc<dyn Fn(Duration) + Send + Sync>,
}

impl ReqwestVertexTransport {
    pub fn new(
        project: impl Into<String>,
        location: impl AsRef<str>,
        model: impl AsRef<str>,
    ) -> Result<Self, VertexAiError> {
        let project = project.into();
        let endpoint = build_vertex_endpoint(&project, location.as_ref(), model.as_ref())?;
        let executor = Arc::new(ReqwestHttpExecutor::new()?);
        Ok(Self {
            project,
            endpoint,
            executor,
            sleeper: Arc::new(thread::sleep),
        })
    }

    #[cfg(test)]
    fn new_for_test(
        project: &str,
        location: &str,
        model: &str,
        executor: Arc<dyn HttpExecutor>,
        sleeper: Arc<dyn Fn(Duration) + Send + Sync>,
    ) -> Result<Self, VertexAiError> {
        Ok(Self {
            project: project.to_owned(),
            endpoint: build_vertex_endpoint(project, location, model)?,
            executor,
            sleeper,
        })
    }

    fn request(&self, body: &[u8], token: &SecretAccessToken) -> HttpRequest {
        HttpRequest {
            url: self.endpoint.clone(),
            headers: vec![
                (
                    "Authorization".to_owned(),
                    format!("Bearer {}", token.as_str()),
                ),
                ("Content-Type".to_owned(), "application/json".to_owned()),
                ("X-Goog-User-Project".to_owned(), self.project.clone()),
            ],
            body: body.to_vec(),
        }
    }
}

impl VertexTransport for ReqwestVertexTransport {
    fn generate(
        &self,
        request: &ExplainPreparedRequest,
        token: &SecretAccessToken,
    ) -> Result<VertexGenerateResponse, VertexAiError> {
        if request.request_body.len() > crate::ai_explain::MAX_VERTEX_REQUEST_BYTES {
            return Err(error(
                AiExplainErrorCode::AiExplainPayloadTooLarge,
                "Vertex request exceeds the 96 KiB limit",
            ));
        }

        let mut attempt = 0_u8;
        loop {
            attempt += 1;
            let http_request = self.request(&request.request_body, token);
            let response = match self.executor.execute(&http_request) {
                Ok(response) => response,
                Err(HttpExecutorError::ConnectBeforeSendTimeout) => {
                    if attempt < MAX_ATTEMPTS {
                        (self.sleeper)(retry_delay(attempt, None));
                        continue;
                    }
                    return Err(error(
                        AiExplainErrorCode::VertexTimeout,
                        "Vertex connection timed out",
                    ));
                }
                Err(HttpExecutorError::ResponseTimeout) => {
                    return Err(error(
                        AiExplainErrorCode::VertexTimeout,
                        "Vertex request timed out",
                    ));
                }
                Err(HttpExecutorError::Transport) => {
                    return Err(error(
                        AiExplainErrorCode::VertexTransportFailed,
                        "Vertex transport failed",
                    ));
                }
            };

            if response.body_too_large {
                return Err(error(
                    AiExplainErrorCode::VertexProtocolError,
                    "provider response exceeded the size limit",
                ));
            }

            if response.status == 200 {
                return parse_provider_response(&response.body);
            }

            let retryable_status = matches!(response.status, 429 | 500 | 502 | 503 | 504);
            if retryable_status && attempt < MAX_ATTEMPTS {
                (self.sleeper)(retry_delay(attempt, response.retry_after.as_deref()));
                continue;
            }

            return Err(status_error(response.status));
        }
    }
}

fn retry_delay(attempt: u8, retry_after: Option<&str>) -> Duration {
    let base = match attempt {
        1 => RETRY_DELAY_ATTEMPT_TWO,
        _ => RETRY_DELAY_ATTEMPT_THREE,
    };
    let retry_after = retry_after
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds <= MAX_RETRY_AFTER)
        .map(Duration::from_secs);
    retry_after.filter(|delay| *delay > base).unwrap_or(base)
}

fn status_error(status: u16) -> VertexAiError {
    match status {
        401 | 403 => error(
            AiExplainErrorCode::VertexPermissionDenied,
            "Vertex permission was denied",
        ),
        404 => error(
            AiExplainErrorCode::VertexNotFound,
            "Vertex model or endpoint was not found",
        ),
        429 => error(
            AiExplainErrorCode::VertexRateLimited,
            "Vertex rate limit was exhausted",
        ),
        500 | 502 | 503 | 504 => error(
            AiExplainErrorCode::VertexUnavailable,
            "Vertex service remained unavailable",
        ),
        _ => error(
            AiExplainErrorCode::VertexRequestFailed,
            "Vertex request failed",
        ),
    }
}

fn parse_provider_response(body: &[u8]) -> Result<VertexGenerateResponse, VertexAiError> {
    let response: VertexGenerateResponse =
        serde_json::from_slice(body).map_err(|_| protocol_error())?;
    validate_provider_response(&response)?;
    Ok(response)
}

fn validate_provider_response(response: &VertexGenerateResponse) -> Result<(), VertexAiError> {
    if response
        .prompt_feedback
        .as_ref()
        .and_then(|feedback| feedback.block_reason.as_ref())
        .is_some()
        || response.candidates.iter().any(|candidate| {
            candidate
                .safety_ratings
                .as_ref()
                .is_some_and(|ratings| ratings.iter().any(|rating| rating.blocked == Some(true)))
        })
    {
        return Err(error(
            AiExplainErrorCode::VertexResponseBlocked,
            "Vertex response was blocked",
        ));
    }

    if response.candidates.len() != 1 {
        return Err(protocol_error());
    }
    let candidate = &response.candidates[0];
    if candidate.index.is_some_and(|index| index != 0)
        || candidate.finish_reason.as_deref() != Some("STOP")
        || candidate.grounding_metadata.is_some()
        || candidate.citation_metadata.is_some()
        || candidate.url_context_metadata.is_some()
    {
        return Err(protocol_error());
    }
    let content = candidate.content.as_ref().ok_or_else(protocol_error)?;
    if content.role.as_deref().is_some_and(|role| role != "model") || content.parts.len() != 1 {
        return Err(protocol_error());
    }
    let part = &content.parts[0];
    if part.text.is_none()
        || part.thought == Some(true)
        || part.inline_data.is_some()
        || part.function_call.is_some()
        || part.function_response.is_some()
        || part.file_data.is_some()
        || part.executable_code.is_some()
        || part.code_execution_result.is_some()
    {
        return Err(protocol_error());
    }

    let response_id = response.response_id.as_deref().ok_or_else(protocol_error)?;
    if !validate_token68(response_id.as_bytes(), 256) {
        return Err(protocol_error());
    }
    let model_version = response
        .model_version
        .as_deref()
        .ok_or_else(protocol_error)?;
    if !validate_model_version(model_version) {
        return Err(protocol_error());
    }
    let create_time = response.create_time.as_deref().ok_or_else(protocol_error)?;
    if !validate_create_time(create_time) {
        return Err(protocol_error());
    }
    if let Some(usage) = response.usage_metadata.as_ref() {
        if [
            usage.prompt_token_count,
            usage.thoughts_token_count,
            usage.candidates_token_count,
            usage.total_token_count,
        ]
        .into_iter()
        .flatten()
        .any(|count| count > 10_000_000)
        {
            return Err(protocol_error());
        }
    }
    Ok(())
}

fn protocol_error() -> VertexAiError {
    error(
        AiExplainErrorCode::VertexProtocolError,
        PROTOCOL_ERROR_DETAIL,
    )
}

fn validate_model_version(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
}

fn validate_create_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() > 35 || bytes.len() < 20 || !bytes.is_ascii() {
        return false;
    }
    let fixed_positions = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if fixed_positions
        .into_iter()
        .any(|(index, expected)| bytes.get(index).copied() != Some(expected))
        || ![0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
            .into_iter()
            .all(|range| bytes[range].iter().all(u8::is_ascii_digit))
    {
        return false;
    }

    let mut timezone_start = 19;
    if bytes.get(timezone_start) == Some(&b'.') {
        let fraction_start = timezone_start + 1;
        timezone_start = fraction_start;
        while bytes.get(timezone_start).is_some_and(u8::is_ascii_digit) {
            timezone_start += 1;
        }
        if ![3, 6, 9].contains(&(timezone_start - fraction_start)) {
            return false;
        }
    }

    match bytes.get(timezone_start..) {
        Some(timezone) if timezone == b"Z" => true,
        Some(timezone)
            if timezone.len() == 6
                && matches!(timezone[0], b'+' | b'-')
                && timezone[3] == b':'
                && timezone[1..3].iter().all(u8::is_ascii_digit)
                && timezone[4..6].iter().all(u8::is_ascii_digit) =>
        {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::fs;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    use crate::ai_explain::{build_vertex_request, ExplainLanguage};

    const EVIDENCE_FIXTURE: &[u8] =
        include_bytes!("../../../examples/payment_policies/reserve/evidence_alpha.json");
    const TEST_TOKEN: &str = "TEST_TOKEN_PLACEHOLDER";
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct FakeExecutor {
        scripted: Mutex<VecDeque<Result<HttpResponse, HttpExecutorError>>>,
        requests: Mutex<Vec<CapturedRequest>>,
    }

    struct CapturedRequest {
        url: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    }

    impl FakeExecutor {
        fn new(scripted: Vec<Result<HttpResponse, HttpExecutorError>>) -> Self {
            Self {
                scripted: Mutex::new(scripted.into()),
                requests: Mutex::new(Vec::new()),
            }
        }

        fn requests(&self) -> Vec<CapturedRequest> {
            let mut requests = self.requests.lock().expect("request mutex is healthy");
            std::mem::take(&mut *requests)
        }
    }

    impl HttpExecutor for FakeExecutor {
        fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpExecutorError> {
            self.requests
                .lock()
                .expect("request mutex is healthy")
                .push(CapturedRequest {
                    url: request.url.clone(),
                    headers: request.headers.clone(),
                    body: request.body.clone(),
                });
            self.scripted
                .lock()
                .expect("script mutex is healthy")
                .pop_front()
                .unwrap_or(Err(HttpExecutorError::Transport))
        }
    }

    fn token() -> SecretAccessToken {
        SecretAccessToken::new(TEST_TOKEN).expect("test token is valid")
    }

    fn prepared_request() -> ExplainPreparedRequest {
        build_vertex_request(EVIDENCE_FIXTURE, ExplainLanguage::English)
            .expect("fixture builds a request")
    }

    fn response() -> HttpResponse {
        HttpResponse {
            status: 200,
            retry_after: None,
            body: br#"{
                "candidates": [{
                    "content": {"role": "model", "parts": [{"text": "{\"overview\":\"ok\"}"}]},
                    "finishReason": "STOP",
                    "index": 0
                }],
                "responseId": "response-1",
                "modelVersion": "gemini-3.5-flash-001",
                "createTime": "2026-08-14T12:34:56Z",
                "usageMetadata": {"promptTokenCount": 1, "totalTokenCount": 2}
            }"#
            .to_vec(),
            body_too_large: false,
        }
    }

    fn status_response(status: u16) -> HttpResponse {
        HttpResponse {
            status,
            retry_after: None,
            body: b"PROVIDER_BODY_MUST_NOT_BE_EXPOSED".to_vec(),
            body_too_large: false,
        }
    }

    fn blocked_response() -> HttpResponse {
        HttpResponse {
            status: 200,
            retry_after: None,
            body: br#"{
                "candidates": [],
                "promptFeedback": {"blockReason": "BLOCK_REASON_MUST_NOT_BE_EXPOSED"}
            }"#
            .to_vec(),
            body_too_large: false,
        }
    }

    fn transport_with(
        scripted: Vec<Result<HttpResponse, HttpExecutorError>>,
        sleeps: &Arc<Mutex<Vec<Duration>>>,
    ) -> (ReqwestVertexTransport, Arc<FakeExecutor>) {
        let executor = Arc::new(FakeExecutor::new(scripted));
        let sleep_log = Arc::clone(sleeps);
        let sleeper: Arc<dyn Fn(Duration) + Send + Sync> = Arc::new(move |delay| {
            sleep_log
                .lock()
                .expect("sleep mutex is healthy")
                .push(delay);
        });
        let transport = ReqwestVertexTransport::new_for_test(
            "sample-project",
            "global",
            "gemini-3.5-flash",
            Arc::clone(&executor) as Arc<dyn HttpExecutor>,
            sleeper,
        )
        .expect("test endpoint is valid");
        (transport, executor)
    }

    fn temp_directory(label: &str) -> std::path::PathBuf {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!("mpk-vertex-{label}-{counter}"));
        fs::create_dir_all(&directory).expect("temporary directory is created");
        directory
    }

    #[test]
    fn token_parser_enforces_one_token68_line_and_redacts_debug() {
        for output in [
            b"TEST_TOKEN_PLACEHOLDER\n".as_slice(),
            b"TEST_TOKEN_PLACEHOLDER\r\n".as_slice(),
            b"abcXYZ-._~+/==".as_slice(),
        ] {
            assert!(SecretAccessToken::parse_output(output).is_ok());
        }
        for output in [
            b"".as_slice(),
            b" \n".as_slice(),
            b"abc def\n".as_slice(),
            b"abc\ndef\n".as_slice(),
            b"abc=def\n".as_slice(),
            b"abc\r\nextra\n".as_slice(),
            &[0xff, b'\n'],
        ] {
            let error = SecretAccessToken::parse_output(output).expect_err("token is rejected");
            assert_eq!(error.code(), AiExplainErrorCode::VertexAuthFailed);
        }
        let oversized = vec![b'a'; MAX_TOKEN_OUTPUT_BYTES + 1];
        assert_eq!(
            SecretAccessToken::parse_output(&oversized)
                .expect_err("oversized token is rejected")
                .code(),
            AiExplainErrorCode::VertexAuthFailed
        );
        let secret = SecretAccessToken::new(TEST_TOKEN).expect("test token is valid");
        assert!(!format!("{secret:?}").contains(TEST_TOKEN));
        assert!(!format!("{secret:?}").contains("TEST_TOKEN"));
    }

    #[cfg(unix)]
    fn write_executable(contents: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        use std::os::unix::fs::PermissionsExt;

        let directory = temp_directory("gcloud");
        let executable = directory.join("gcloud-fake");
        fs::write(&executable, contents).expect("fake executable is written");
        let mut permissions = fs::metadata(&executable)
            .expect("fake executable metadata is readable")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("fake executable is executable");
        (executable.clone(), executable.with_extension("args"))
    }

    #[cfg(unix)]
    #[test]
    fn gcloud_provider_uses_only_fixed_arguments() {
        let (executable, args_file) = write_executable(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$0.args\"\nprintf '%s\\n' 'TEST_TOKEN_PLACEHOLDER'\n",
        );
        let provider = GcloudAccessTokenProvider::new(&executable);
        let token = provider.access_token().expect("fake ADC succeeds");
        assert_eq!(token, SecretAccessToken::new(TEST_TOKEN).unwrap());
        assert_eq!(
            fs::read_to_string(args_file).expect("arguments are recorded"),
            "auth\napplication-default\nprint-access-token\n--quiet\n"
        );
        fs::remove_dir_all(executable.parent().unwrap()).expect("temporary directory is removed");
    }

    #[cfg(unix)]
    #[test]
    fn gcloud_provider_bounds_output_and_reaps_timeouts_without_leaking_errors() {
        let (executable, _) = write_executable(
            "#!/bin/sh\ni=0\nwhile [ \"$i\" -lt 17000 ]; do printf x; i=$((i + 1)); done\nprintf '\\n'\ni=0\nwhile [ \"$i\" -lt 17000 ]; do printf x >&2; i=$((i + 1)); done\nprintf '\\n' >&2\n",
        );
        let error = GcloudAccessTokenProvider::new(&executable)
            .access_token()
            .expect_err("noisy child is rejected");
        assert_eq!(error.code(), AiExplainErrorCode::VertexAuthFailed);
        assert!(!error.to_string().contains(TEST_TOKEN));
        fs::remove_dir_all(executable.parent().unwrap()).expect("temporary directory is removed");

        let (executable, _) = write_executable("#!/bin/sh\nsleep 2\n");
        let error = GcloudAccessTokenProvider::with_timeout(&executable, Duration::from_millis(20))
            .access_token()
            .expect_err("timed out child is rejected");
        assert_eq!(error.code(), AiExplainErrorCode::VertexAuthFailed);
        fs::remove_dir_all(executable.parent().unwrap()).expect("temporary directory is removed");

        let error = GcloudAccessTokenProvider::new("/definitely/missing/gcloud")
            .access_token()
            .expect_err("missing executable is rejected");
        assert_eq!(error.code(), AiExplainErrorCode::VertexAuthUnavailable);
        assert!(!error.to_string().contains("/definitely/missing"));
    }

    #[test]
    fn endpoint_construction_is_fixed_to_global_or_regional_vertex_v1() {
        assert_eq!(
            build_vertex_endpoint("sample-project", "global", "gemini-3.5-flash").unwrap(),
            "https://aiplatform.googleapis.com/v1/projects/sample-project/locations/global/publishers/google/models/gemini-3.5-flash:generateContent"
        );
        assert_eq!(
            build_vertex_endpoint("sample-project", "us-central1", "gemini-3.5-flash").unwrap(),
            "https://us-central1-aiplatform.googleapis.com/v1/projects/sample-project/locations/us-central1/publishers/google/models/gemini-3.5-flash:generateContent"
        );
        for (project, location, model) in [
            ("bad", "global", "gemini-3.5-flash"),
            ("sample-project", "us-central1/evil", "gemini-3.5-flash"),
            ("sample-project", "GLOBAL", "gemini-3.5-flash"),
            ("sample-project", "global", "unreviewed-model"),
        ] {
            assert_eq!(
                build_vertex_endpoint(project, location, model)
                    .expect_err("invalid endpoint is rejected")
                    .code(),
                AiExplainErrorCode::VertexConfigInvalid
            );
        }
    }

    #[test]
    fn transport_sends_exact_prepared_body_and_fixed_headers() {
        assert_eq!("2026-08-14T12:34:56Z".as_bytes().get(19), Some(&b'Z'));
        assert_eq!(
            "2026-08-14T12:34:56Z".as_bytes().get(19..),
            Some(b"Z".as_slice())
        );
        assert!(validate_create_time("2026-08-14T12:34:56Z"));
        assert!(validate_create_time("2026-08-14T12:34:56.123+09:00"));
        assert!(validate_create_time("2026-08-14T12:34:56.123456-09:00"));
        assert!(!validate_create_time("2026-08-14T12:34:56.12Z"));
        assert!(validate_model_version("gemini-3.5-flash-001"));
        assert!(validate_token68(b"response-1", 256));
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(vec![Ok(response())], &sleeps);
        let prepared = prepared_request();
        let expected_body = prepared.request_body.clone();
        let result = transport
            .generate(&prepared, &token())
            .expect("fake response is valid");
        assert_eq!(result.candidates.len(), 1);
        let requests = executor.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://aiplatform.googleapis.com/v1/projects/sample-project/locations/global/publishers/google/models/gemini-3.5-flash:generateContent");
        assert_eq!(requests[0].body, expected_body);
        assert_eq!(
            requests[0].headers,
            vec![
                (
                    "Authorization".to_owned(),
                    "Bearer ".to_owned() + TEST_TOKEN
                ),
                ("Content-Type".to_owned(), "application/json".to_owned()),
                (
                    "X-Goog-User-Project".to_owned(),
                    "sample-project".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn retry_policy_covers_retryable_statuses_and_retry_after() {
        for status in [429, 500, 502, 503, 504] {
            let sleeps = Arc::new(Mutex::new(Vec::new()));
            let (transport, executor) =
                transport_with(vec![Ok(status_response(status)), Ok(response())], &sleeps);
            transport
                .generate(&prepared_request(), &token())
                .expect("retryable response succeeds on second attempt");
            assert_eq!(executor.requests().len(), 2);
            assert_eq!(sleeps.lock().unwrap().as_slice(), [RETRY_DELAY_ATTEMPT_TWO]);
        }

        for status in [400, 401, 403, 404] {
            let sleeps = Arc::new(Mutex::new(Vec::new()));
            let (transport, executor) =
                transport_with(vec![Ok(status_response(status)), Ok(response())], &sleeps);
            let error = transport
                .generate(&prepared_request(), &token())
                .expect_err("non-retryable status fails");
            let expected = match status {
                401 | 403 => AiExplainErrorCode::VertexPermissionDenied,
                404 => AiExplainErrorCode::VertexNotFound,
                _ => AiExplainErrorCode::VertexRequestFailed,
            };
            assert_eq!(error.code(), expected);
            assert_eq!(executor.requests().len(), 1);
            assert!(sleeps.lock().unwrap().is_empty());
            assert!(!error
                .to_string()
                .contains("PROVIDER_BODY_MUST_NOT_BE_EXPOSED"));
        }

        let mut first = status_response(429);
        first.retry_after = Some("10".to_owned());
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, _) = transport_with(vec![Ok(first), Ok(response())], &sleeps);
        transport
            .generate(&prepared_request(), &token())
            .expect("Retry-After succeeds");
        assert_eq!(sleeps.lock().unwrap().as_slice(), [Duration::from_secs(10)]);

        for retry_after in ["11", "Wed, 21 Oct 2015 07:28:00 GMT", "10x"] {
            let mut first = status_response(503);
            first.retry_after = Some(retry_after.to_owned());
            let sleeps = Arc::new(Mutex::new(Vec::new()));
            let (transport, _) = transport_with(vec![Ok(first), Ok(response())], &sleeps);
            transport
                .generate(&prepared_request(), &token())
                .expect("invalid Retry-After falls back to base delay");
            assert_eq!(sleeps.lock().unwrap().as_slice(), [RETRY_DELAY_ATTEMPT_TWO]);
        }

        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(
            vec![
                Ok(status_response(429)),
                Ok(status_response(429)),
                Ok(status_response(429)),
            ],
            &sleeps,
        );
        let error = transport
            .generate(&prepared_request(), &token())
            .expect_err("rate limit is exhausted");
        assert_eq!(error.code(), AiExplainErrorCode::VertexRateLimited);
        assert_eq!(executor.requests().len(), 3);
        assert_eq!(
            sleeps.lock().unwrap().as_slice(),
            [RETRY_DELAY_ATTEMPT_TWO, RETRY_DELAY_ATTEMPT_THREE]
        );

        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(
            vec![
                Ok(status_response(503)),
                Ok(status_response(503)),
                Ok(status_response(503)),
            ],
            &sleeps,
        );
        let error = transport
            .generate(&prepared_request(), &token())
            .expect_err("unavailability is exhausted");
        assert_eq!(error.code(), AiExplainErrorCode::VertexUnavailable);
        assert_eq!(executor.requests().len(), 3);
    }

    #[test]
    fn timeout_transport_and_oversize_paths_do_not_retry() {
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(
            vec![
                Err(HttpExecutorError::ConnectBeforeSendTimeout),
                Ok(response()),
            ],
            &sleeps,
        );
        transport
            .generate(&prepared_request(), &token())
            .expect("connect timeout retries");
        assert_eq!(executor.requests().len(), 2);
        assert_eq!(sleeps.lock().unwrap().as_slice(), [RETRY_DELAY_ATTEMPT_TWO]);

        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(
            vec![
                Err(HttpExecutorError::ConnectBeforeSendTimeout),
                Err(HttpExecutorError::ConnectBeforeSendTimeout),
                Err(HttpExecutorError::ConnectBeforeSendTimeout),
            ],
            &sleeps,
        );
        let error = transport
            .generate(&prepared_request(), &token())
            .expect_err("three connect timeouts exhaust the retry budget");
        assert_eq!(error.code(), AiExplainErrorCode::VertexTimeout);
        assert_eq!(executor.requests().len(), 3);
        assert_eq!(
            sleeps.lock().unwrap().as_slice(),
            [RETRY_DELAY_ATTEMPT_TWO, RETRY_DELAY_ATTEMPT_THREE]
        );

        for transport_error in [
            HttpExecutorError::ResponseTimeout,
            HttpExecutorError::Transport,
        ] {
            let sleeps = Arc::new(Mutex::new(Vec::new()));
            let (transport, executor) =
                transport_with(vec![Err(transport_error), Ok(response())], &sleeps);
            let error = transport
                .generate(&prepared_request(), &token())
                .expect_err("ambiguous or terminal transport error fails");
            assert_eq!(
                error.code(),
                if transport_error == HttpExecutorError::ResponseTimeout {
                    AiExplainErrorCode::VertexTimeout
                } else {
                    AiExplainErrorCode::VertexTransportFailed
                }
            );
            assert_eq!(executor.requests().len(), 1);
            assert!(sleeps.lock().unwrap().is_empty());
        }

        let oversized = HttpResponse {
            status: 200,
            retry_after: None,
            body: Vec::new(),
            body_too_large: true,
        };
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, executor) = transport_with(vec![Ok(oversized), Ok(response())], &sleeps);
        let error = transport
            .generate(&prepared_request(), &token())
            .expect_err("oversized body is terminal");
        assert_eq!(error.code(), AiExplainErrorCode::VertexProtocolError);
        assert_eq!(executor.requests().len(), 1);
        assert!(sleeps.lock().unwrap().is_empty());
    }

    #[test]
    fn response_limits_enforce_declared_and_incremental_bytes() {
        for (status, limit) in [(200, MAX_SUCCESS_BODY_BYTES), (503, MAX_ERROR_BODY_BYTES)] {
            let declared = read_bounded_http_response(
                status,
                Some((limit + 1) as u64),
                None,
                &mut Cursor::new(Vec::<u8>::new()),
                |_| HttpExecutorError::Transport,
            )
            .expect("declared response size is handled");
            assert!(declared.body_too_large);

            let mut streamed_body = vec![b'x'; limit + 1];
            let streamed = read_bounded_http_response(
                status,
                None,
                None,
                &mut Cursor::new(&mut streamed_body),
                |_| HttpExecutorError::Transport,
            )
            .expect("streamed response size is handled");
            assert!(streamed.body_too_large);
        }
    }

    #[test]
    fn blocked_and_malformed_provider_envelopes_are_stable_and_secret_free() {
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let (transport, _) = transport_with(vec![Ok(blocked_response())], &sleeps);
        let error = transport
            .generate(&prepared_request(), &token())
            .expect_err("blocked prompt is rejected");
        assert_eq!(error.code(), AiExplainErrorCode::VertexResponseBlocked);
        assert!(!error
            .to_string()
            .contains("BLOCK_REASON_MUST_NOT_BE_EXPOSED"));

        let malformed_bodies = [
            br#"{"candidates":[]}"#.as_slice(),
            br#"{"candidates":[{"finishReason":"MAX_TOKENS","content":{"parts":[{"text":"x"}]}}],"responseId":"r","modelVersion":"m","createTime":"2026-08-14T12:34:56Z"}"#.as_slice(),
            br#"{"candidates":[{"finishReason":"STOP","content":{"parts":[{}]}}],"responseId":"r","modelVersion":"m","createTime":"2026-08-14T12:34:56Z"}"#.as_slice(),
            br#"{"candidates":[{"finishReason":"STOP","content":{"parts":[{"text":"x"}]}}],"modelVersion":"m","createTime":"2026-08-14T12:34:56Z"}"#.as_slice(),
            br#"{"candidates":[{"finishReason":"STOP","content":{"parts":[{"text":"x"}]}}],"responseId":"r","createTime":"2026-08-14T12:34:56Z"}"#.as_slice(),
            br#"{"candidates":[{"finishReason":"STOP","content":{"parts":[{"text":"x"}]}}],"responseId":"r","modelVersion":"m","createTime":"not-a-time"}"#.as_slice(),
            br#"PROVIDER_BODY_MUST_NOT_BE_EXPOSED"#.as_slice(),
        ];
        for body in malformed_bodies {
            let executor = Arc::new(FakeExecutor::new(vec![Ok(HttpResponse {
                status: 200,
                retry_after: None,
                body: body.to_vec(),
                body_too_large: false,
            })]));
            let sleeper: Arc<dyn Fn(Duration) + Send + Sync> = Arc::new(|_| {});
            let transport = ReqwestVertexTransport::new_for_test(
                "sample-project",
                "global",
                "gemini-3.5-flash",
                Arc::clone(&executor) as Arc<dyn HttpExecutor>,
                sleeper,
            )
            .unwrap();
            let error = transport
                .generate(&prepared_request(), &token())
                .expect_err("malformed provider envelope is rejected");
            assert_eq!(error.code(), AiExplainErrorCode::VertexProtocolError);
            assert!(!error
                .to_string()
                .contains("PROVIDER_BODY_MUST_NOT_BE_EXPOSED"));
        }
    }

    #[test]
    fn candidate_safety_rating_and_metadata_limits_are_enforced() {
        let safety_body = br#"{
            "candidates": [{
                "content": {"parts": [{"text":"x"}]},
                "finishReason": "STOP",
                "safetyRatings": [{"blocked": true}]
            }],
            "responseId": "r",
            "modelVersion": "m",
            "createTime": "2026-08-14T12:34:56Z"
        }"#;
        let mut oversized_usage = response();
        oversized_usage.body = br#"{
            "candidates": [{"content":{"parts":[{"text":"x"}]},"finishReason":"STOP"}],
            "responseId":"r","modelVersion":"m","createTime":"2026-08-14T12:34:56Z",
            "usageMetadata":{"totalTokenCount":10000001}
        }"#
        .to_vec();
        for (body, expected_code) in [
            (
                safety_body.to_vec(),
                AiExplainErrorCode::VertexResponseBlocked,
            ),
            (
                oversized_usage.body,
                AiExplainErrorCode::VertexProtocolError,
            ),
        ] {
            let sleeps = Arc::new(Mutex::new(Vec::new()));
            let (transport, _) = transport_with(
                vec![Ok(HttpResponse {
                    status: 200,
                    retry_after: None,
                    body,
                    body_too_large: false,
                })],
                &sleeps,
            );
            let error = transport
                .generate(&prepared_request(), &token())
                .expect_err("unsafe provider envelope is rejected");
            assert_eq!(error.code(), expected_code);
        }
    }
}
