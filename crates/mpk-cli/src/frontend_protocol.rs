use mpk_vc::{
    canonical_json_bytes, import_frontend_source_manifest_json, import_source_map_json,
    import_vir_json, parse_strict_json, scan_strict_json, CapturedInput, LanguageConfiguration,
    SourceManifestValidationContext, SourceMapValidationContext, StrictJsonError, StrictJsonEvent,
    StrictJsonLimits, StrictJsonObserver, StrictJsonPathSegment, ValidatedReleaseRegistry,
    ValidatedSourceManifest, ValidatedSourceMap, VirModule,
};
use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

pub const FRONTEND_STDOUT_BYTES_MAX: usize = 268_435_456;
pub const FRONTEND_STDERR_BYTES_MAX: usize = 2_097_152;
const JSON_BYTES_MAX: u64 = 268_435_455;
const JSON_NODES_MAX: u64 = 16_777_216;
const JSON_NESTING_MAX: u64 = 256;
const STRING_BYTES_MAX: u64 = 1_048_576;
const ISSUES_MAX: usize = 1_024;
const ISSUE_MESSAGE_BYTES_MAX: usize = 4_096;
const ISSUE_MESSAGE_TOTAL_MAX: usize = 2_097_152;

#[derive(Clone, Copy, Debug)]
pub struct FrontendProtocolRequest<'a> {
    pub source_language: &'a str,
    pub semantic_profile: &'a str,
    pub semantic_parameters: &'a Value,
    pub selection: &'a Value,
    pub release_registry: Option<&'a ValidatedReleaseRegistry>,
    pub captured_inputs: &'a [CapturedInput<'a>],
}

/// Runner-internal immutable staging set. A successful frontend manifest
/// selects its exact captured closure before entering the public protocol
/// validator, whose input inventory remains exact.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FrontendStagingRequest<'a> {
    pub(crate) source_language: &'a str,
    pub(crate) semantic_profile: &'a str,
    pub(crate) semantic_parameters: &'a Value,
    pub(crate) selection: &'a Value,
    pub(crate) release_registry: Option<&'a ValidatedReleaseRegistry>,
    pub(crate) available_inputs: &'a [CapturedInput<'a>],
}

#[derive(Clone, Copy, Debug)]
pub struct FrontendProcessFacts<'a> {
    pub exit_code: Option<i32>,
    pub signaled: bool,
    pub stdout: &'a [u8],
    pub stderr_observed_bytes: usize,
}

#[derive(Clone, Debug)]
pub struct AcceptedFrontendEnvelope {
    pub status: String,
    pub phase: String,
    pub value: Value,
    pub canonical_bytes: Vec<u8>,
    pub artifacts: Option<AcceptedFrontendArtifacts>,
}

#[derive(Clone, Debug)]
pub struct AcceptedFrontendArtifacts {
    pub vir: VirModule,
    pub source_map: ValidatedSourceMap,
    pub source_manifest: ValidatedSourceManifest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendProtocolCode {
    ProcessKilled,
    ProtocolMissing,
    ProtocolTruncated,
    ProtocolMalformed,
    ProtocolShape,
    ProtocolStatusExit,
    ProtocolUnexpectedUsage,
    ProtocolNoncanonical,
    ProtocolLimit,
    ProtocolIdentityMismatch,
    ProtocolArtifactMismatch,
}

impl FrontendProtocolCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessKilled => "FRONTEND_PROCESS_KILLED",
            Self::ProtocolMissing => "FRONTEND_PROTOCOL_MISSING",
            Self::ProtocolTruncated => "FRONTEND_PROTOCOL_TRUNCATED",
            Self::ProtocolMalformed => "FRONTEND_PROTOCOL_MALFORMED",
            Self::ProtocolShape => "FRONTEND_PROTOCOL_SHAPE",
            Self::ProtocolStatusExit => "FRONTEND_PROTOCOL_STATUS_EXIT",
            Self::ProtocolUnexpectedUsage => "FRONTEND_PROTOCOL_UNEXPECTED_USAGE",
            Self::ProtocolNoncanonical => "FRONTEND_PROTOCOL_NONCANONICAL",
            Self::ProtocolLimit => "FRONTEND_PROTOCOL_LIMIT",
            Self::ProtocolIdentityMismatch => "FRONTEND_PROTOCOL_IDENTITY_MISMATCH",
            Self::ProtocolArtifactMismatch => "FRONTEND_PROTOCOL_ARTIFACT_MISMATCH",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendProtocolError {
    code: FrontendProtocolCode,
}

impl FrontendProtocolError {
    fn new(code: FrontendProtocolCode) -> Self {
        Self { code }
    }

    pub const fn code(&self) -> FrontendProtocolCode {
        self.code
    }
}

impl fmt::Display for FrontendProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for FrontendProtocolError {}

/// Checks one named public-frontend limit without constructing a potentially
/// huge transport. The returned code preserves the consumer classification:
/// producers normalize issue messages before transport, while stream and
/// combined-list breaches are protocol limits.
#[doc(hidden)]
pub fn validate_frontend_protocol_limit(
    family: &str,
    count: u64,
) -> Result<(), FrontendProtocolError> {
    let (maximum, code) = match family {
        "stdout_transport_bytes" => (
            FRONTEND_STDOUT_BYTES_MAX as u64,
            FrontendProtocolCode::ProtocolLimit,
        ),
        "stderr_observed_bytes" => (
            FRONTEND_STDERR_BYTES_MAX as u64,
            FrontendProtocolCode::ProtocolLimit,
        ),
        "json_nesting" => (JSON_NESTING_MAX, FrontendProtocolCode::ProtocolMalformed),
        "string_bytes" => (STRING_BYTES_MAX, FrontendProtocolCode::ProtocolMalformed),
        "issues" => (ISSUES_MAX as u64, FrontendProtocolCode::ProtocolLimit),
        "issue_message_bytes" => (
            ISSUE_MESSAGE_BYTES_MAX as u64,
            FrontendProtocolCode::ProtocolShape,
        ),
        "combined_issue_message_bytes" => (
            ISSUE_MESSAGE_TOTAL_MAX as u64,
            FrontendProtocolCode::ProtocolLimit,
        ),
        _ => return Err(protocol(FrontendProtocolCode::ProtocolShape)),
    };
    if count > maximum {
        Err(protocol(code))
    } else {
        Ok(())
    }
}

pub fn validate_frontend_process(
    request: FrontendProtocolRequest<'_>,
    process: FrontendProcessFacts<'_>,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    validate_frontend_process_inner(request, process, false)
}

pub(crate) fn validate_frontend_process_from_staging(
    request: FrontendStagingRequest<'_>,
    process: FrontendProcessFacts<'_>,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    validate_frontend_process_inner(
        FrontendProtocolRequest {
            source_language: request.source_language,
            semantic_profile: request.semantic_profile,
            semantic_parameters: request.semantic_parameters,
            selection: request.selection,
            release_registry: request.release_registry,
            captured_inputs: request.available_inputs,
        },
        process,
        true,
    )
}

fn validate_frontend_process_inner(
    request: FrontendProtocolRequest<'_>,
    process: FrontendProcessFacts<'_>,
    project_staging: bool,
) -> Result<AcceptedFrontendEnvelope, FrontendProtocolError> {
    validate_frontend_protocol_limit(
        "stdout_transport_bytes",
        u64::try_from(process.stdout.len()).unwrap_or(u64::MAX),
    )?;
    validate_frontend_protocol_limit(
        "stderr_observed_bytes",
        u64::try_from(process.stderr_observed_bytes).unwrap_or(u64::MAX),
    )?;
    if process.signaled || process.exit_code.is_none() {
        return Err(protocol(FrontendProtocolCode::ProcessKilled));
    }
    let exit = process.exit_code.unwrap_or(1);
    if exit == 2 {
        return Err(protocol(FrontendProtocolCode::ProtocolUnexpectedUsage));
    }
    if process.stdout.is_empty() {
        return Err(protocol(FrontendProtocolCode::ProtocolMissing));
    }

    let first = first_json_value(process.stdout)?;
    if !process.stdout.ends_with(b"\n") {
        return Err(protocol(FrontendProtocolCode::ProtocolTruncated));
    }
    let json = &process.stdout[..process.stdout.len() - 1];
    if first != json.len() || json.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(protocol(FrontendProtocolCode::ProtocolNoncanonical));
    }
    if let Some(error) = scan_frontend_protocol_limits(json)? {
        validate_frontend_root_before_issue_limit(json)?;
        return Err(error);
    }
    let strict = parse_strict_json(
        json,
        StrictJsonLimits::new(
            JSON_BYTES_MAX,
            JSON_NODES_MAX,
            JSON_NESTING_MAX,
            STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| protocol(FrontendProtocolCode::ProtocolMalformed))?;
    let mut canonical = canonical_json_bytes(&strict)
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolMalformed))?;
    drop(strict);
    let mut value_deserializer = serde_json::Deserializer::from_slice(json);
    value_deserializer.disable_recursion_limit();
    let value = Value::deserialize(&mut value_deserializer)
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolMalformed))?;
    let (status, phase) = validate_shape(&value, exit)?;
    validate_public_paths(&value)?;
    if canonical != json {
        return Err(protocol(FrontendProtocolCode::ProtocolNoncanonical));
    }
    validate_identity(&value, request)?;
    let projected_inputs = if status == "ir-lowered" && project_staging {
        Some(project_manifest_inputs(&value, request.captured_inputs)?)
    } else {
        None
    };
    let artifacts = if status == "ir-lowered" {
        Some(validate_success_artifacts(
            &value,
            request,
            projected_inputs
                .as_deref()
                .unwrap_or(request.captured_inputs),
        )?)
    } else {
        None
    };
    canonical.push(b'\n');
    Ok(AcceptedFrontendEnvelope {
        status: status.to_owned(),
        phase: phase.to_owned(),
        value,
        canonical_bytes: canonical,
        artifacts,
    })
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontendLimitRootProbe {
    schema: String,
    status: String,
    phase: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: IgnoredAny,
    selection: IgnoredAny,
    rejected_features: DiscardingSequence<IgnoredAny>,
    diagnostics: DiscardingSequence<IgnoredAny>,
    #[serde(default)]
    ir: FrontendIrPresence,
    #[serde(default)]
    source_manifest: FrontendFieldPresence,
    #[serde(default)]
    source_map: FrontendFieldPresence,
}

struct DiscardingSequence<T>(PhantomData<fn() -> T>);

impl<'de, T> Deserialize<'de> for DiscardingSequence<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<T>(PhantomData<fn() -> T>);

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = DiscardingSequence<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON array")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while sequence.next_element::<T>()?.is_some() {}
                Ok(DiscardingSequence(PhantomData))
            }
        }

        deserializer.deserialize_seq(Visitor(PhantomData))
    }
}

#[derive(Default)]
struct FrontendFieldPresence(bool);

impl<'de> Deserialize<'de> for FrontendFieldPresence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer)?;
        Ok(Self(true))
    }
}

#[derive(Default)]
struct FrontendIrPresence(bool);

impl<'de> Deserialize<'de> for FrontendIrPresence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let probe = FrontendIrLimitProbe::deserialize(deserializer)?;
        if probe.schema != "mpk.vir.v0" || !is_lower_sha256(&probe.sha256) {
            return Err(serde::de::Error::custom("invalid frontend IR wrapper"));
        }
        Ok(Self(true))
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontendIrLimitProbe {
    schema: String,
    sha256: String,
    value: FrontendObjectOnly,
}

struct FrontendObjectOnly;

impl<'de> Deserialize<'de> for FrontendObjectOnly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FrontendObjectOnly;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(FrontendObjectOnly)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

fn validate_frontend_root_before_issue_limit(input: &[u8]) -> Result<(), FrontendProtocolError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let probe = FrontendLimitRootProbe::deserialize(&mut deserializer)
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolShape))?;
    if probe.schema != "mpk.frontend.cli.v0" {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    let success_fields = [probe.ir.0, probe.source_manifest.0, probe.source_map.0];
    let expected = match probe.status.as_str() {
        "ir-lowered" if probe.phase == "emission" => true,
        "ir-lowered" => return Err(protocol(FrontendProtocolCode::ProtocolShape)),
        "frontend-error" | "rejected" | "source-error" => false,
        _ => return Err(protocol(FrontendProtocolCode::ProtocolShape)),
    };
    if success_fields.iter().any(|present| *present != expected) {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn scan_frontend_protocol_limits(
    input: &[u8],
) -> Result<Option<FrontendProtocolError>, FrontendProtocolError> {
    let mut observer = FrontendProtocolLimitObserver {
        issue_count: 0,
        message_bytes: 0,
        aggregate_error: None,
        message_error: None,
    };
    scan_strict_json(
        input,
        StrictJsonLimits::new(
            JSON_BYTES_MAX,
            JSON_NODES_MAX,
            JSON_NESTING_MAX,
            STRING_BYTES_MAX,
        ),
        &mut observer,
    )
    .map_err(|error| match error {
        StrictJsonError::ObservedLimitExceeded { limit, .. }
        | StrictJsonError::ObservedCounterOverflow { limit } => {
            let code = match limit {
                "issue_message_bytes" => FrontendProtocolCode::ProtocolShape,
                "issues" | "combined_issue_message_bytes" => FrontendProtocolCode::ProtocolLimit,
                _ => FrontendProtocolCode::ProtocolMalformed,
            };
            protocol(code)
        }
        _ => protocol(FrontendProtocolCode::ProtocolMalformed),
    })?;
    Ok(observer
        .aggregate_error
        .or(observer.message_error)
        .map(map_frontend_observed_error))
}

struct FrontendProtocolLimitObserver {
    issue_count: u64,
    message_bytes: u64,
    aggregate_error: Option<StrictJsonError>,
    message_error: Option<StrictJsonError>,
}

impl StrictJsonObserver for FrontendProtocolLimitObserver {
    fn observe(&mut self, event: StrictJsonEvent<'_>) -> Result<(), StrictJsonError> {
        match event {
            StrictJsonEvent::ArrayElement { path, .. } if root_issue_array(path) => {
                match self.issue_count.checked_add(1) {
                    Some(count) => {
                        self.issue_count = count;
                        self.record_max("issues", count, ISSUES_MAX as u64);
                    }
                    None => {
                        self.record(StrictJsonError::ObservedCounterOverflow { limit: "issues" })
                    }
                }
            }
            StrictJsonEvent::String {
                path,
                decoded_bytes,
            } if issue_message_path(path) => match self.message_bytes.checked_add(decoded_bytes) {
                Some(count) => {
                    self.message_bytes = count;
                    self.record_max(
                        "combined_issue_message_bytes",
                        count,
                        ISSUE_MESSAGE_TOTAL_MAX as u64,
                    );
                }
                None => self.record(StrictJsonError::ObservedCounterOverflow {
                    limit: "combined_issue_message_bytes",
                }),
            },
            _ => {}
        }
        Ok(())
    }

    fn string_byte_limit(&mut self, path: &[StrictJsonPathSegment]) -> Option<(&'static str, u64)> {
        issue_message_path(path).then_some(("issue_message_bytes", ISSUE_MESSAGE_BYTES_MAX as u64))
    }

    fn string_byte_limit_exceeded(
        &mut self,
        limit: &'static str,
        maximum: u64,
        actual: u64,
    ) -> Result<(), StrictJsonError> {
        self.record(StrictJsonError::ObservedLimitExceeded {
            limit,
            maximum,
            actual,
        });
        Ok(())
    }
}

impl FrontendProtocolLimitObserver {
    fn record(&mut self, error: StrictJsonError) {
        let aggregate = matches!(
            &error,
            StrictJsonError::ObservedLimitExceeded { limit, .. }
                | StrictJsonError::ObservedCounterOverflow { limit }
                if matches!(*limit, "issues" | "combined_issue_message_bytes")
        );
        let slot = if aggregate {
            &mut self.aggregate_error
        } else {
            &mut self.message_error
        };
        if slot.is_none() {
            *slot = Some(error);
        }
    }

    fn record_max(&mut self, limit: &'static str, actual: u64, maximum: u64) {
        if actual > maximum {
            self.record(StrictJsonError::ObservedLimitExceeded {
                limit,
                maximum,
                actual,
            });
        }
    }
}

fn map_frontend_observed_error(error: StrictJsonError) -> FrontendProtocolError {
    let limit = match error {
        StrictJsonError::ObservedLimitExceeded { limit, .. }
        | StrictJsonError::ObservedCounterOverflow { limit } => limit,
        _ => return protocol(FrontendProtocolCode::ProtocolMalformed),
    };
    let code = match limit {
        "issue_message_bytes" => FrontendProtocolCode::ProtocolShape,
        "issues" | "combined_issue_message_bytes" => FrontendProtocolCode::ProtocolLimit,
        _ => FrontendProtocolCode::ProtocolMalformed,
    };
    protocol(code)
}

fn root_issue_array(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [StrictJsonPathSegment::Key(name)]
            if matches!(name.as_str(), "rejected_features" | "diagnostics")
    )
}

fn issue_message_path(path: &[StrictJsonPathSegment]) -> bool {
    matches!(
        path,
        [
            StrictJsonPathSegment::Key(channel),
            StrictJsonPathSegment::Index(_),
            StrictJsonPathSegment::Key(field),
        ] if matches!(channel.as_str(), "rejected_features" | "diagnostics")
            && field == "message"
    )
}

fn first_json_value(bytes: &[u8]) -> Result<usize, FrontendProtocolError> {
    let candidate = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    // serde_json's ignored-value reader is iterative. Once it identifies the
    // exact first-value prefix, run that prefix through MPK's iterative strict
    // scanner so framing never disables or bypasses the normative depth cap.
    let mut stream = serde_json::Deserializer::from_slice(candidate).into_iter::<IgnoredAny>();
    match stream.next() {
        Some(Ok(_)) => {
            let end = stream.byte_offset();
            let mut observer = |_event: StrictJsonEvent<'_>| Ok(());
            scan_strict_json(
                &candidate[..end],
                StrictJsonLimits::new(
                    FRONTEND_STDOUT_BYTES_MAX as u64,
                    JSON_NODES_MAX,
                    JSON_NESTING_MAX,
                    STRING_BYTES_MAX,
                ),
                &mut observer,
            )
            .map_err(|_| protocol(FrontendProtocolCode::ProtocolMalformed))?;
            Ok(end + (bytes.len() - candidate.len()))
        }
        Some(Err(error)) if error.is_eof() => {
            Err(protocol(FrontendProtocolCode::ProtocolTruncated))
        }
        Some(Err(_)) => Err(protocol(FrontendProtocolCode::ProtocolMalformed)),
        None => Err(protocol(FrontendProtocolCode::ProtocolMissing)),
    }
}

fn validate_shape(value: &Value, exit: i32) -> Result<(&str, &str), FrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))?;
    let status = string_field(object, "status")?;
    let phase = string_field(object, "phase")?;
    let common = [
        "schema",
        "status",
        "phase",
        "source_language",
        "semantic_profile",
        "semantic_parameters",
        "selection",
    ];
    let expected_exit = match status {
        "ir-lowered" => {
            exact_fields(
                object,
                &common
                    .into_iter()
                    .chain([
                        "ir",
                        "source_manifest",
                        "source_map",
                        "rejected_features",
                        "diagnostics",
                    ])
                    .collect::<Vec<_>>(),
            )?;
            if phase != "emission" {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            validate_ir_wrapper(field(object, "ir")?)?;
            0
        }
        "frontend-error" | "rejected" | "source-error" => {
            exact_fields(
                object,
                &common
                    .into_iter()
                    .chain(["rejected_features", "diagnostics"])
                    .collect::<Vec<_>>(),
            )?;
            match status {
                "frontend-error" => 1,
                "rejected" => 3,
                "source-error" => 4,
                _ => unreachable!(),
            }
        }
        _ => return Err(protocol(FrontendProtocolCode::ProtocolShape)),
    };
    if string_field(object, "schema")? != "mpk.frontend.cli.v0" {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    let rejected = array_field(object, "rejected_features")?;
    let diagnostics = array_field(object, "diagnostics")?;
    validate_issues(
        rejected,
        diagnostics,
        status,
        phase,
        string_field(object, "source_language")?,
        field(object, "selection")?,
    )?;
    let phase_valid = match status {
        "ir-lowered" => phase == "emission" && rejected.is_empty(),
        "rejected" => {
            matches!(
                phase,
                "capture" | "source" | "metadata" | "subset" | "lowering" | "emission"
            ) && rejected.len() + diagnostics.len() > 0
        }
        "source-error" => {
            matches!(phase, "capture" | "source" | "metadata" | "typecheck")
                && rejected.is_empty()
                && !diagnostics.is_empty()
        }
        "frontend-error" => {
            matches!(
                phase,
                "capture"
                    | "source"
                    | "metadata"
                    | "typecheck"
                    | "subset"
                    | "lowering"
                    | "emission"
            ) && rejected.is_empty()
                && !diagnostics.is_empty()
        }
        _ => false,
    };
    if !phase_valid {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    if exit != expected_exit {
        return Err(protocol(FrontendProtocolCode::ProtocolStatusExit));
    }
    Ok((status, phase))
}

fn validate_ir_wrapper(value: &Value) -> Result<(), FrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))?;
    exact_fields(object, &["schema", "sha256", "value"])?;
    if string_field(object, "schema")? != "mpk.vir.v0"
        || !is_lower_sha256(string_field(object, "sha256")?)
        || !field(object, "value")?.is_object()
    {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn validate_issues(
    rejected: &[Value],
    diagnostics: &[Value],
    status: &str,
    phase: &str,
    source_language: &str,
    selection: &Value,
) -> Result<(), FrontendProtocolError> {
    let issue_count = rejected
        .len()
        .checked_add(diagnostics.len())
        .and_then(|count| u64::try_from(count).ok())
        .unwrap_or(u64::MAX);
    validate_frontend_protocol_limit("issues", issue_count)?;
    let mut message_bytes = 0_u64;
    for (issues, rejected_channel) in [(rejected, true), (diagnostics, false)] {
        let mut previous: Option<IssueKey<'_>> = None;
        for (index, value) in issues.iter().enumerate() {
            let object = value
                .as_object()
                .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))?;
            let allowed = if object.contains_key("function_id") && object.contains_key("span") {
                vec!["code", "message", "function_id", "span"]
            } else if object.contains_key("function_id") {
                vec!["code", "message", "function_id"]
            } else if object.contains_key("span") {
                vec!["code", "message", "span"]
            } else {
                vec!["code", "message"]
            };
            exact_fields(object, &allowed)?;
            let code = string_field(object, "code")?;
            if code.is_empty()
                || code.len() > 128
                || !code.as_bytes()[0].is_ascii_uppercase()
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
            {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            let message = string_field(object, "message")?;
            if message.is_empty() || message.chars().any(char::is_control) {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            let message_len = u64::try_from(message.len()).unwrap_or(u64::MAX);
            validate_frontend_protocol_limit("issue_message_bytes", message_len)?;
            message_bytes = message_bytes
                .checked_add(message_len)
                .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolLimit))?;
            validate_frontend_protocol_limit("combined_issue_message_bytes", message_bytes)?;
            let function = object.get("function_id").map(value_string).transpose()?;
            let marker = matches!(
                code,
                "GO_LIMIT_DIAGNOSTICS_TRUNCATED" | "RUST_LIMIT_DIAGNOSTICS_TRUNCATED"
            );
            let expected_marker = match source_language {
                "go" => "GO_LIMIT_DIAGNOSTICS_TRUNCATED",
                "rust" => "RUST_LIMIT_DIAGNOSTICS_TRUNCATED",
                _ => return Err(protocol(FrontendProtocolCode::ProtocolShape)),
            };
            if marker
                && (code != expected_marker
                    || !std::ptr::eq(issues, diagnostics)
                    || index + 1 != issues.len()
                    || !valid_truncation_message(message))
            {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            if source_language == "rust"
                && !marker
                && (!valid_rust_issue_policy(code, status, phase, rejected_channel)
                    || function
                        .is_some_and(|function| !valid_rust_function_id(function, selection)))
            {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            if matches!(phase, "subset" | "lowering" | "emission") && function.is_none() && !marker
            {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            let (path, start, end) = if let Some(span) = object.get("span") {
                validate_span(span)?
            } else {
                ("", 0, 0)
            };
            if marker && (function.is_some() || object.contains_key("span")) {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            let key = IssueKey {
                path,
                start,
                code,
                message,
                function: function.unwrap_or(""),
                end,
            };
            if !marker && previous.as_ref().is_some_and(|prior| prior > &key) {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
            if !marker {
                previous = Some(key);
            }
        }
    }
    Ok(())
}

fn valid_rust_issue_policy(code: &str, status: &str, phase: &str, rejected_channel: bool) -> bool {
    let (expected_status, expected_channel, valid_phase) = match code {
        "FRONTEND_SANDBOX_UNAVAILABLE" => (
            "frontend-error",
            false,
            matches!(phase, "metadata" | "typecheck"),
        ),
        "RUST_LIMIT_INPUT_BYTES"
        | "RUST_LIMIT_INPUT_COUNT"
        | "RUST_LIMIT_PATH"
        | "RUST_PREFLIGHT_FILE_TYPE"
        | "RUST_PREFLIGHT_PATH"
        | "RUST_PREFLIGHT_MANIFEST_FIELD"
        | "RUST_PREFLIGHT_WORKSPACE"
        | "RUST_PREFLIGHT_CONFIG"
        | "RUST_PREFLIGHT_TOOLCHAIN_FILE"
        | "RUST_PREFLIGHT_DEPENDENCY"
        | "RUST_PREFLIGHT_BUILD_SCRIPT"
        | "RUST_PREFLIGHT_FEATURE"
        | "RUST_PREFLIGHT_TARGET"
        | "RUST_PREFLIGHT_LOCKFILE" => ("rejected", true, phase == "capture"),
        "RUST_LIMIT_CONTRACT" => ("rejected", true, matches!(phase, "capture" | "subset")),
        "RUST_SOURCE_MANIFEST_PARSE"
        | "RUST_SOURCE_MODULE_MISSING"
        | "RUST_SOURCE_MODULE_AMBIGUOUS"
        | "RUST_SOURCE_MODULE_DUPLICATE"
        | "RUST_SOURCE_MODULE_CYCLE" => ("source-error", false, phase == "capture"),
        "RUST_SOURCE_PARSE" => ("source-error", false, phase == "source"),
        "RUST_SUBSET_CFG"
        | "RUST_SUBSET_MACRO"
        | "RUST_SUBSET_ATTRIBUTE"
        | "RUST_SUBSET_IMPORT"
        | "RUST_SUBSET_VISIBILITY"
        | "RUST_SUBSET_PATH"
        | "RUST_SUBSET_EXPANSION" => ("rejected", true, matches!(phase, "source" | "subset")),
        "RUST_FRONTEND_METADATA_PROCESS" | "RUST_FRONTEND_METADATA_PROTOCOL" => {
            ("frontend-error", false, phase == "metadata")
        }
        "RUST_PREFLIGHT_METADATA_MISMATCH" => ("rejected", true, phase == "metadata"),
        "RUST_SOURCE_NAME"
        | "RUST_SOURCE_TYPE"
        | "RUST_SOURCE_BORROW"
        | "RUST_SOURCE_LITERAL_RANGE" => ("source-error", false, phase == "typecheck"),
        "RUST_SUBSET_IDENTIFIER"
        | "RUST_SUBSET_ITEM"
        | "RUST_SUBSET_FUNCTION_KIND"
        | "RUST_SUBSET_GENERIC"
        | "RUST_SUBSET_TRAIT"
        | "RUST_SUBSET_IMPL"
        | "RUST_SUBSET_STATIC"
        | "RUST_SUBSET_TYPE"
        | "RUST_SUBSET_DROP"
        | "RUST_SUBSET_PATTERN"
        | "RUST_SUBSET_BINDING"
        | "RUST_SUBSET_CONTROL_FLOW"
        | "RUST_SUBSET_MUTATION"
        | "RUST_SUBSET_OPERATION"
        | "RUST_SUBSET_CALL"
        | "RUST_SUBSET_PURITY"
        | "RUST_LIMIT_CALL_CLOSURE"
        | "RUST_LIMIT_AGGREGATE"
        | "RUST_CONTRACT_JSON"
        | "RUST_CONTRACT_SCHEMA"
        | "RUST_CONTRACT_SHAPE"
        | "RUST_CONTRACT_IDENTITY"
        | "RUST_CONTRACT_DUPLICATE"
        | "RUST_CONTRACT_UNUSED"
        | "RUST_CONTRACT_MISSING"
        | "RUST_CONTRACT_RESOLUTION"
        | "RUST_CONTRACT_PROFILE"
        | "RUST_CONTRACT_TYPE"
        | "RUST_CONTRACT_OPERATOR"
        | "RUST_CONTRACT_HASH" => ("rejected", true, phase == "subset"),
        "RUST_TOOLCHAIN_COMPONENT"
        | "RUST_TOOLCHAIN_TARGET"
        | "RUST_TOOLCHAIN_ARGUMENT"
        | "RUST_TOOLCHAIN_LOADER_PATH" => (
            "frontend-error",
            false,
            matches!(phase, "metadata" | "typecheck" | "lowering"),
        ),
        "RUST_TOOLCHAIN_COMMIT" => (
            "frontend-error",
            false,
            matches!(phase, "metadata" | "typecheck" | "lowering"),
        ),
        "RUST_TOOLCHAIN_OPTIONS" => ("frontend-error", false, phase == "typecheck"),
        "RUST_TOOLCHAIN_MIR_ADAPTER" => ("frontend-error", false, phase == "lowering"),
        "RUST_MIR_STATEMENT"
        | "RUST_MIR_RVALUE"
        | "RUST_MIR_OPERAND"
        | "RUST_MIR_PLACE"
        | "RUST_MIR_PROJECTION"
        | "RUST_MIR_TERMINATOR"
        | "RUST_MIR_ASSERTION"
        | "RUST_MIR_CHECKED_PATTERN"
        | "RUST_MIR_CALL"
        | "RUST_MIR_MOVE"
        | "RUST_MIR_CLEANUP"
        | "RUST_LIMIT_MIR_BLOCKS"
        | "RUST_LIMIT_MIR_STATEMENTS"
        | "RUST_SEMANTICS_TYPE"
        | "RUST_SEMANTICS_TARGET"
        | "RUST_SEMANTICS_CHECK_MISSING"
        | "RUST_SEMANTICS_CHECK_EXTRA"
        | "RUST_SEMANTICS_PANIC" => ("rejected", true, phase == "lowering"),
        "RUST_LIMIT_IR" => ("rejected", true, matches!(phase, "lowering" | "emission")),
        "RUST_FRONTEND_SOURCE_INVENTORY" => ("frontend-error", false, phase == "source"),
        "RUST_FRONTEND_SOURCE_MAP_EXTERNAL"
        | "RUST_FRONTEND_SOURCE_MAP_RANGE"
        | "RUST_FRONTEND_SOURCE_MAP_REFERENCE" => ("frontend-error", false, phase == "emission"),
        "RUST_FRONTEND_CHILD_OUTPUT_LIMIT"
        | "RUST_FRONTEND_COMPILER_CRASH"
        | "RUST_FRONTEND_DIAGNOSTIC_BUDGET" => (
            "frontend-error",
            false,
            matches!(
                phase,
                "capture"
                    | "source"
                    | "metadata"
                    | "typecheck"
                    | "subset"
                    | "lowering"
                    | "emission"
            ),
        ),
        "RUST_FRONTEND_DRIVER_PROTOCOL_TRANSPORT"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_SHAPE"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_CANONICAL"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_HASH"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_IDENTITY"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_FILESYSTEM"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_PROCESS"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_COUNT"
        | "RUST_FRONTEND_DRIVER_PROTOCOL_OUTPUT_LIMIT" => (
            "frontend-error",
            false,
            matches!(phase, "typecheck" | "lowering"),
        ),
        _ => return false,
    };
    status == expected_status && rejected_channel == expected_channel && valid_phase
}

fn valid_rust_function_id(function: &str, selection: &Value) -> bool {
    let Some(crate_name) = selection.get("crate").and_then(Value::as_str) else {
        return false;
    };
    if function.len() > 1_024 || !rust_identifier(crate_name) {
        return false;
    }
    let mut segments = function.split("::");
    segments.next() == Some(crate_name)
        && segments.next().is_some_and(rust_identifier)
        && segments.all(rust_identifier)
}

fn rust_identifier(value: &str) -> bool {
    if value == "_" || value.len() > 255 || !value.is_ascii() {
        return false;
    }
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_truncation_message(message: &str) -> bool {
    let Some(omitted) = message.strip_suffix(" normalized issues omitted") else {
        return false;
    };
    !omitted.is_empty()
        && omitted != "0"
        && !omitted.starts_with('0')
        && omitted.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Clone, Eq, PartialEq)]
struct IssueKey<'a> {
    path: &'a str,
    start: i64,
    code: &'a str,
    message: &'a str,
    function: &'a str,
    end: i64,
}

impl Ord for IssueKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.path.as_bytes(),
            self.start,
            self.code.as_bytes(),
            self.message.as_bytes(),
            self.function.as_bytes(),
            self.end,
        )
            .cmp(&(
                other.path.as_bytes(),
                other.start,
                other.code.as_bytes(),
                other.message.as_bytes(),
                other.function.as_bytes(),
                other.end,
            ))
    }
}

impl PartialOrd for IssueKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn validate_span(value: &Value) -> Result<(&str, i64, i64), FrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))?;
    exact_fields(object, &["normalized_path", "start", "end"])?;
    let path = string_field(object, "normalized_path")?;
    let start = integer_field(object, "start")?;
    let end = integer_field(object, "end")?;
    if !portable_path(path) || start < 0 || start >= end {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    Ok((path, start, end))
}

fn validate_identity(
    value: &Value,
    request: FrontendProtocolRequest<'_>,
) -> Result<(), FrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolIdentityMismatch))?;
    if object.get("source_language").and_then(Value::as_str) != Some(request.source_language)
        || object.get("semantic_profile").and_then(Value::as_str) != Some(request.semantic_profile)
        || object.get("semantic_parameters") != Some(request.semantic_parameters)
        || object.get("selection") != Some(request.selection)
    {
        return Err(protocol(FrontendProtocolCode::ProtocolIdentityMismatch));
    }
    if object.get("status").and_then(Value::as_str) == Some("ir-lowered") {
        let registry = request
            .release_registry
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolIdentityMismatch))?;
        let manifest_registry = value
            .pointer("/source_manifest/release_registry")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolIdentityMismatch))?;
        let registry_digest = registry.registry_digest().to_hex();
        if manifest_registry.get("id").and_then(Value::as_str)
            != Some(registry.registry().id.as_str())
            || manifest_registry
                .get("registry_sha256")
                .and_then(Value::as_str)
                != Some(registry_digest.as_str())
        {
            return Err(protocol(FrontendProtocolCode::ProtocolIdentityMismatch));
        }
    }
    Ok(())
}

fn validate_success_artifacts(
    envelope: &Value,
    request: FrontendProtocolRequest<'_>,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<AcceptedFrontendArtifacts, FrontendProtocolError> {
    let registry = request
        .release_registry
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let ir = &envelope["ir"];
    let vir_bytes = serde_json::to_vec(&ir["value"])
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let vir = import_vir_json(&vir_bytes)
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    if ir["schema"] != "mpk.vir.v0"
        || ir["sha256"].as_str() != Some(vir.vir_hash.as_str())
        || source_language_text(vir.source_language) != request.source_language
        || semantic_profile_text(vir.semantic_profile) != request.semantic_profile
    {
        return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
    }
    let map_bytes = serde_json::to_vec(&envelope["source_map"])
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let source_map = import_source_map_json(
        &map_bytes,
        SourceMapValidationContext {
            vir: &vir,
            captured_inputs,
            synthetic_permissions: &[],
        },
    )
    .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let manifest_bytes = serde_json::to_vec(&envelope["source_manifest"])
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let expected_language_configuration = if request.source_language == "rust" {
        Some(
            serde_json::from_value::<LanguageConfiguration>(
                envelope["source_manifest"]["target"]["language_configuration"].clone(),
            )
            .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?,
        )
    } else {
        None
    };
    let manifest = import_frontend_source_manifest_json(
        &manifest_bytes,
        SourceManifestValidationContext {
            vir: &vir,
            source_map: &source_map,
            captured_inputs,
            release_registry: registry,
            expected_language_configuration: expected_language_configuration.as_ref(),
        },
    )
    .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    if manifest.manifest().vir_hash != vir.vir_hash.as_str()
        || manifest.manifest().source_map_hash != source_map.hash().as_str()
    {
        return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
    }
    validate_success_issues(envelope, &vir, manifest.manifest(), captured_inputs)?;
    let selected = request
        .selection
        .get("function")
        .and_then(Value::as_str)
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let count = vir
        .units
        .iter()
        .flat_map(|unit| &unit.functions)
        .filter(|function| function.id == selected)
        .count();
    if count != 1 {
        return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
    }
    Ok(AcceptedFrontendArtifacts {
        vir,
        source_map,
        source_manifest: manifest,
    })
}

fn project_manifest_inputs<'a>(
    envelope: &Value,
    available_inputs: &[CapturedInput<'a>],
) -> Result<Vec<CapturedInput<'a>>, FrontendProtocolError> {
    let available_inputs = index_available_inputs(available_inputs)?;
    let manifest_inputs = envelope
        .pointer("/source_manifest/inputs")
        .and_then(Value::as_array)
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let source_language = match envelope.get("source_language").and_then(Value::as_str) {
        Some("go") => mpk_vc::SourceLanguage::Go,
        Some("rust") => mpk_vc::SourceLanguage::Rust,
        _ => return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch)),
    };
    mpk_vc::validate_source_manifest_input_count(source_language, manifest_inputs.len() as u64)
        .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
    let mut captured_inputs = Vec::with_capacity(manifest_inputs.len());
    for manifest_input in manifest_inputs {
        let object = manifest_input
            .as_object()
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        let path = object
            .get("normalized_path")
            .and_then(Value::as_str)
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        let kind = match object.get("kind").and_then(Value::as_str) {
            Some("source") => mpk_vc::InputKind::Source,
            Some("contract") => mpk_vc::InputKind::Contract,
            Some("build_manifest") => mpk_vc::InputKind::BuildManifest,
            Some("lockfile") => mpk_vc::InputKind::Lockfile,
            _ => return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch)),
        };
        let input = available_inputs
            .get(path)
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        if input.kind != kind {
            return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
        }
        captured_inputs.push(*input);
    }
    Ok(captured_inputs)
}

fn index_available_inputs<'a>(
    available_inputs: &[CapturedInput<'a>],
) -> Result<BTreeMap<&'a str, CapturedInput<'a>>, FrontendProtocolError> {
    let mut inputs = BTreeMap::new();
    let mut folded_paths = BTreeSet::new();
    for input in available_inputs {
        if mpk_vc::validate_manifest_normalized_path(input.normalized_path).is_err()
            || inputs.insert(input.normalized_path, *input).is_some()
            || !folded_paths.insert(input.normalized_path.to_ascii_lowercase())
        {
            return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
        }
    }
    Ok(inputs)
}

fn validate_success_issues(
    envelope: &Value,
    vir: &mpk_vc::VirModule,
    manifest: &mpk_vc::SourceManifest,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), FrontendProtocolError> {
    for issue in envelope["rejected_features"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(envelope["diagnostics"].as_array().into_iter().flatten())
    {
        let object = issue
            .as_object()
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        if let Some(function_id) = object.get("function_id").and_then(Value::as_str) {
            let count = vir
                .units
                .iter()
                .flat_map(|unit| &unit.functions)
                .filter(|function| function.id == function_id)
                .count();
            if count != 1 {
                return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
            }
        }
        let Some(span) = object.get("span").and_then(Value::as_object) else {
            continue;
        };
        let path = span
            .get("normalized_path")
            .and_then(Value::as_str)
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        let start = span
            .get("start")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        let end = span
            .get("end")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        let manifest_count = manifest
            .inputs
            .iter()
            .filter(|input| input.normalized_path == path)
            .count();
        let mut matching = captured_inputs
            .iter()
            .filter(|input| input.normalized_path == path);
        let captured = matching
            .next()
            .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        if matching.next().is_some()
            || manifest_count != 1
            || end > captured.bytes.len()
            || start >= end
        {
            return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
        }
        let text = std::str::from_utf8(captured.bytes)
            .map_err(|_| protocol(FrontendProtocolCode::ProtocolArtifactMismatch))?;
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(protocol(FrontendProtocolCode::ProtocolArtifactMismatch));
        }
    }
    Ok(())
}

fn validate_public_paths(value: &Value) -> Result<(), FrontendProtocolError> {
    match value {
        Value::String(text) => {
            if contains_absolute_path(text) {
                return Err(protocol(FrontendProtocolCode::ProtocolShape));
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_public_paths(value)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_public_paths(value)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

pub(crate) fn contains_absolute_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    for (index, _) in text.char_indices() {
        let at_boundary = index == 0
            || text[..index].chars().next_back().is_some_and(|previous| {
                !previous.is_alphanumeric() && !matches!(previous, '.' | '_' | '-' | '/' | '\\')
            });
        if !at_boundary {
            continue;
        }
        let tail = &bytes[index..];
        let file_uri = tail
            .get(..7)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"file://"));
        let drive = tail.len() >= 3
            && tail[0].is_ascii_alphabetic()
            && tail[1] == b':'
            && matches!(tail[2], b'/' | b'\\');
        if tail.starts_with(b"/") || tail.starts_with(b"\\\\") || file_uri || drive {
            return true;
        }
    }
    false
}

fn exact_fields(object: &Map<String, Value>, fields: &[&str]) -> Result<(), FrontendProtocolError> {
    if object.len() != fields.len() || !fields.iter().all(|field| object.contains_key(*field)) {
        return Err(protocol(FrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Value, FrontendProtocolError> {
    object
        .get(name)
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, FrontendProtocolError> {
    field(object, name)?
        .as_str()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))
}

fn value_string(value: &Value) -> Result<&str, FrontendProtocolError> {
    value
        .as_str()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a [Value], FrontendProtocolError> {
    field(object, name)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))
}

fn integer_field(object: &Map<String, Value>, name: &str) -> Result<i64, FrontendProtocolError> {
    field(object, name)?
        .as_i64()
        .ok_or_else(|| protocol(FrontendProtocolCode::ProtocolShape))
}

fn portable_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1_024
        && path.is_ascii()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| {
            !part.is_empty()
                && part.len() <= 255
                && !matches!(part, "." | "..")
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn source_language_text(language: mpk_vc::SourceLanguage) -> &'static str {
    match language {
        mpk_vc::SourceLanguage::Go => "go",
        mpk_vc::SourceLanguage::Rust => "rust",
    }
}

fn semantic_profile_text(profile: mpk_vc::SemanticProfile) -> &'static str {
    match profile {
        mpk_vc::SemanticProfile::GoFixedV0 => "mpk.go.fixed.v0",
        mpk_vc::SemanticProfile::RustCheckedV0 => "mpk.rust.checked.v0",
    }
}

fn protocol(code: FrontendProtocolCode) -> FrontendProtocolError {
    FrontendProtocolError::new(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request<'a>(parameters: &'a Value, selection: &'a Value) -> FrontendProtocolRequest<'a> {
        FrontendProtocolRequest {
            source_language: "go",
            semantic_profile: "mpk.go.fixed.v0",
            semantic_parameters: parameters,
            selection,
            release_registry: None,
            captured_inputs: &[],
        }
    }

    #[test]
    fn non_success_transport_is_exact_and_identity_pinned() {
        let parameters = json!({"target_id":"linux/amd64","pointer_width":64});
        let selection = json!({"package":"example.com/p","function":"example.com/p.F"});
        let envelope = json!({
            "schema":"mpk.frontend.cli.v0","status":"frontend-error","phase":"capture",
            "source_language":"go","semantic_profile":"mpk.go.fixed.v0",
            "semantic_parameters":parameters,"selection":selection,
            "rejected_features":[],"diagnostics":[{"code":"GO_FRONTEND_INTERNAL","message":"failed"}]
        });
        let mut bytes = serde_json::to_vec(&envelope).expect("serialize");
        bytes.push(b'\n');
        let accepted = validate_frontend_process(
            request(&parameters, &selection),
            FrontendProcessFacts {
                exit_code: Some(1),
                signaled: false,
                stdout: &bytes,
                stderr_observed_bytes: 0,
            },
        )
        .expect("canonical envelope accepts");
        assert_eq!(accepted.status, "frontend-error");

        let mut extra = bytes.clone();
        extra.push(b'\n');
        assert_eq!(
            validate_frontend_process(
                request(&parameters, &selection),
                FrontendProcessFacts {
                    exit_code: Some(1),
                    signaled: false,
                    stdout: &extra,
                    stderr_observed_bytes: 0,
                },
            )
            .expect_err("extra LF rejects")
            .code(),
            FrontendProtocolCode::ProtocolNoncanonical
        );
    }

    #[test]
    fn public_path_scan_rejects_embedded_absolute_path_families() {
        for text in [
            "open /tmp/source.go",
            "open=/mpk/source/main.go",
            "read C:\\work\\main.go",
            "read \\\\server\\share\\main.go",
            "read FILE:///tmp/main.go",
        ] {
            assert_eq!(
                validate_public_paths(&Value::String(text.to_owned()))
                    .expect_err("absolute path rejects")
                    .code(),
                FrontendProtocolCode::ProtocolShape
            );
        }
        for text in ["example.com/mpk/vector", "contracts/main.mpk", "a/b.go"] {
            validate_public_paths(&Value::String(text.to_owned()))
                .expect("portable relative text accepts");
        }
    }
}
