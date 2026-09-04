//! Candidate-only frontend protocol for the practical C# profile.
//!
//! This module has no command-line route, registry discovery, process launch,
//! or installed-bundle fallback.  Callers inject the validated successor
//! registry, immutable captured inputs, and (for success) the already
//! validated complete source-artifact root.

use crate::frontend_protocol::FrontendProcessFacts;
use mpk_vc::csharp_practical_registry::{
    validate_successor_semantic_request, SuccessorCompiledSemanticProfile,
    ValidatedSuccessorRegistry, ValidatedSuccessorRequest, SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
};
use mpk_vc::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, parse_canonical_practical_json,
    practical_semantic_context_value, CapturedInputSet, OriginalInputKind,
    PracticalArtifactContext, PracticalArtifactErrorCode, PracticalArtifactKind,
    PracticalJsonValue, ValidatedPracticalArtifact, BOUNDARY_CONTRACT_SCHEMA,
    METHOD_CONTRACT_SCHEMA, PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX, SEMANTIC_BINDING_SCHEMA,
    SOURCE_ARTIFACTS_SCHEMA, TRANSITION_CONTRACT_SCHEMA, TYPE_CONTRACT_SCHEMA,
};
use mpk_vc::source_manifest::{input_set_hash, InputEntry};
use mpk_vc::{
    canonical_json_bytes, hash_domain_separated_raw, sha256_raw_file_bytes, HashDomain, InputKind,
    StrictJsonValue,
};
use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const CSHARP_PRACTICAL_FRONTEND_CLI_SCHEMA: &str = "mpk.frontend.cli.v2";
pub const CSHARP_PRACTICAL_FRONTEND_REQUEST_SCHEMA: &str = "mpk.frontend.request.v2";
pub const CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA: &str = "mpk.frontend.success.v2";
pub const CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA: &str = "mpk.frontend.diagnostic.v2";
pub const CSHARP_PRACTICAL_FRONTEND_REJECTED_STATUS: &str = "rejected";
pub const CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE: &str =
    "The selected construct is outside the frozen practical profile.";

pub const CSHARP_PRACTICAL_FRONTEND_REQUEST_BYTES_MAX: usize = 131_072;
pub const CSHARP_PRACTICAL_FRONTEND_STDOUT_BYTES_MAX: usize = 268_435_456;
pub const CSHARP_PRACTICAL_FRONTEND_STDERR_BYTES_MAX: usize = 2_097_152;
pub const CSHARP_PRACTICAL_NORMALIZED_ISSUES_MAX: usize = 1_024;

const SOURCE_FILES_MAX: usize = 256;
const SOURCE_FILE_BYTES_MAX: usize = 1_048_576;
const SOURCE_TOTAL_BYTES_MAX: usize = 16_777_216;
const SIDECAR_FILES_MAX: usize = 128;
const SIDECAR_FILE_BYTES_MAX: usize = 1_048_576;
const SIDECAR_TOTAL_BYTES_MAX: usize = 8_388_608;
const SNAPSHOT_ENTRIES_MAX: usize = 512;
const SNAPSHOT_TOTAL_BYTES_MAX: usize = 33_554_432;
const NORMALIZED_PATH_BYTES_MAX: usize = 1_024;

pub const CSHARP_PRACTICAL_FRONTEND_REQUEST_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-REQUEST-2.0");
pub const CSHARP_PRACTICAL_FRONTEND_SUCCESS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-SUCCESS-2.0");
pub const CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-DIAGNOSTIC-2.0");

const FRONTEND_REQUEST_FIELDS: &[&str] = &[
    "schema",
    "semantic_request",
    "source_snapshot",
    "sidecars",
    "request_sha256",
];
const SEMANTIC_REQUEST_FIELDS: &[&str] =
    &["schema", "semantic_context", "selection", "request_sha256"];
const SOURCE_SNAPSHOT_FIELDS: &[&str] = &["entries", "snapshot_sha256"];
const SOURCE_SNAPSHOT_ENTRY_FIELDS: &[&str] = &["path", "raw_sha256", "size_bytes"];
const SIDECAR_SET_FIELDS: &[&str] = &["entries", "set_sha256"];
const SIDECAR_REF_FIELDS: &[&str] = &["schema", "path", "raw_sha256"];
const SUCCESS_FIELDS: &[&str] = &[
    "schema",
    "request_sha256",
    "semantic_context",
    "artifacts",
    "success_sha256",
];
const DIAGNOSTIC_FIELDS: &[&str] = &[
    "schema",
    "raw_request_sha256",
    "raw_request_size_bytes",
    "request_linkage",
    "status",
    "phase",
    "diagnostics",
    "diagnostic_sha256",
];
const DIAGNOSTIC_ENTRY_FIELDS: &[&str] = &["code", "message", "location"];
const SOURCE_LOCATION_FIELDS: &[&str] = &["source_file_ordinal", "start_byte", "end_byte"];

const REGISTERED_SIDECAR_SCHEMAS: &[&str] = &[
    TYPE_CONTRACT_SCHEMA,
    METHOD_CONTRACT_SCHEMA,
    SEMANTIC_BINDING_SCHEMA,
    BOUNDARY_CONTRACT_SCHEMA,
    TRANSITION_CONTRACT_SCHEMA,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PracticalSidecarDescriptor {
    pub schema: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct ValidatedPracticalFrontendRequest {
    semantic_request: ValidatedSuccessorRequest,
    semantic_context: PracticalJsonValue,
    value: PracticalJsonValue,
    canonical_bytes: Vec<u8>,
    source_sizes: Vec<u32>,
    input_set_sha256: String,
}

impl ValidatedPracticalFrontendRequest {
    pub fn semantic_request(&self) -> &ValidatedSuccessorRequest {
        &self.semantic_request
    }

    pub fn request_sha256(&self) -> &str {
        self.value
            .get("request_sha256")
            .and_then(PracticalJsonValue::as_str)
            .expect("validated frontend request has its hash")
    }

    pub fn semantic_context(&self) -> &PracticalJsonValue {
        &self.semantic_context
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn value(&self) -> &PracticalJsonValue {
        &self.value
    }

    pub fn input_set_sha256(&self) -> &str {
        &self.input_set_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalDiagnosticFamily {
    Protocol,
    Limit,
    Declaration,
    Type,
    Dependency,
    Generic,
    SourceBinding,
    Object,
    Initializer,
    Ownership,
    Array,
    Collection,
    Order,
    String,
    ParseFormat,
    Float,
    Decimal,
    Nullable,
    Result,
    BusinessValue,
    LoopContract,
    Switch,
    Pattern,
    Exception,
    Boundary,
    Transition,
    Effect,
    Foundation,
    Lowering,
}

impl PracticalDiagnosticFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protocol => "CSHARP_PRACTICAL_PROTOCOL",
            Self::Limit => "CSHARP_PRACTICAL_LIMIT",
            Self::Declaration => "CSHARP_PRACTICAL_DECLARATION",
            Self::Type => "CSHARP_PRACTICAL_TYPE",
            Self::Dependency => "CSHARP_PRACTICAL_DEPENDENCY",
            Self::Generic => "CSHARP_PRACTICAL_GENERIC",
            Self::SourceBinding => "CSHARP_PRACTICAL_SOURCE_BINDING",
            Self::Object => "CSHARP_PRACTICAL_OBJECT",
            Self::Initializer => "CSHARP_PRACTICAL_INITIALIZER",
            Self::Ownership => "CSHARP_PRACTICAL_OWNERSHIP",
            Self::Array => "CSHARP_PRACTICAL_ARRAY",
            Self::Collection => "CSHARP_PRACTICAL_COLLECTION",
            Self::Order => "CSHARP_PRACTICAL_ORDER",
            Self::String => "CSHARP_PRACTICAL_STRING",
            Self::ParseFormat => "CSHARP_PRACTICAL_PARSE_FORMAT",
            Self::Float => "CSHARP_PRACTICAL_FLOAT",
            Self::Decimal => "CSHARP_PRACTICAL_DECIMAL",
            Self::Nullable => "CSHARP_PRACTICAL_NULLABLE",
            Self::Result => "CSHARP_PRACTICAL_RESULT",
            Self::BusinessValue => "CSHARP_PRACTICAL_BUSINESS_VALUE",
            Self::LoopContract => "CSHARP_PRACTICAL_LOOP_CONTRACT",
            Self::Switch => "CSHARP_PRACTICAL_SWITCH",
            Self::Pattern => "CSHARP_PRACTICAL_PATTERN",
            Self::Exception => "CSHARP_PRACTICAL_EXCEPTION",
            Self::Boundary => "CSHARP_PRACTICAL_BOUNDARY",
            Self::Transition => "CSHARP_PRACTICAL_TRANSITION",
            Self::Effect => "CSHARP_PRACTICAL_EFFECT",
            Self::Foundation => "CSHARP_PRACTICAL_FOUNDATION",
            Self::Lowering => "CSHARP_PRACTICAL_LOWERING",
        }
    }

    pub const fn phase(self) -> u8 {
        match self {
            Self::Protocol | Self::Limit => 0,
            Self::Dependency => 1,
            Self::Declaration | Self::Type => 2,
            Self::Generic => 3,
            Self::SourceBinding | Self::Foundation => 4,
            Self::Boundary | Self::Transition => 5,
            Self::Object
            | Self::Initializer
            | Self::Ownership
            | Self::Array
            | Self::Collection
            | Self::Order
            | Self::String
            | Self::ParseFormat
            | Self::Float
            | Self::Decimal
            | Self::Nullable
            | Self::Result
            | Self::BusinessValue => 6,
            Self::LoopContract | Self::Switch | Self::Pattern | Self::Exception | Self::Effect => 7,
            Self::Lowering => 8,
        }
    }

    const fn precedence(self) -> u8 {
        match self {
            Self::Protocol => 0,
            Self::Limit => 1,
            Self::Dependency => 2,
            Self::Declaration => 3,
            Self::Type => 4,
            Self::Generic => 5,
            Self::SourceBinding => 6,
            Self::Foundation => 7,
            Self::Boundary => 8,
            Self::Transition => 9,
            Self::Object => 10,
            Self::Initializer => 11,
            Self::Ownership => 12,
            Self::Array => 13,
            Self::Collection => 14,
            Self::Order => 15,
            Self::String => 16,
            Self::ParseFormat => 17,
            Self::Float => 18,
            Self::Decimal => 19,
            Self::Nullable => 20,
            Self::Result => 21,
            Self::BusinessValue => 22,
            Self::LoopContract => 23,
            Self::Switch => 24,
            Self::Pattern => 25,
            Self::Exception => 26,
            Self::Effect => 27,
            Self::Lowering => 28,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "CSHARP_PRACTICAL_PROTOCOL" => Self::Protocol,
            "CSHARP_PRACTICAL_LIMIT" => Self::Limit,
            "CSHARP_PRACTICAL_DECLARATION" => Self::Declaration,
            "CSHARP_PRACTICAL_TYPE" => Self::Type,
            "CSHARP_PRACTICAL_DEPENDENCY" => Self::Dependency,
            "CSHARP_PRACTICAL_GENERIC" => Self::Generic,
            "CSHARP_PRACTICAL_SOURCE_BINDING" => Self::SourceBinding,
            "CSHARP_PRACTICAL_OBJECT" => Self::Object,
            "CSHARP_PRACTICAL_INITIALIZER" => Self::Initializer,
            "CSHARP_PRACTICAL_OWNERSHIP" => Self::Ownership,
            "CSHARP_PRACTICAL_ARRAY" => Self::Array,
            "CSHARP_PRACTICAL_COLLECTION" => Self::Collection,
            "CSHARP_PRACTICAL_ORDER" => Self::Order,
            "CSHARP_PRACTICAL_STRING" => Self::String,
            "CSHARP_PRACTICAL_PARSE_FORMAT" => Self::ParseFormat,
            "CSHARP_PRACTICAL_FLOAT" => Self::Float,
            "CSHARP_PRACTICAL_DECIMAL" => Self::Decimal,
            "CSHARP_PRACTICAL_NULLABLE" => Self::Nullable,
            "CSHARP_PRACTICAL_RESULT" => Self::Result,
            "CSHARP_PRACTICAL_BUSINESS_VALUE" => Self::BusinessValue,
            "CSHARP_PRACTICAL_LOOP_CONTRACT" => Self::LoopContract,
            "CSHARP_PRACTICAL_SWITCH" => Self::Switch,
            "CSHARP_PRACTICAL_PATTERN" => Self::Pattern,
            "CSHARP_PRACTICAL_EXCEPTION" => Self::Exception,
            "CSHARP_PRACTICAL_BOUNDARY" => Self::Boundary,
            "CSHARP_PRACTICAL_TRANSITION" => Self::Transition,
            "CSHARP_PRACTICAL_EFFECT" => Self::Effect,
            "CSHARP_PRACTICAL_FOUNDATION" => Self::Foundation,
            "CSHARP_PRACTICAL_LOWERING" => Self::Lowering,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PracticalDiagnosticLocation {
    pub source_file_ordinal: u16,
    pub start_byte: u32,
    pub end_byte: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PracticalDiagnosticFinding {
    pub family: PracticalDiagnosticFamily,
    pub location: Option<PracticalDiagnosticLocation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedPracticalDiagnostic {
    family: PracticalDiagnosticFamily,
    location: Option<PracticalDiagnosticLocation>,
}

impl AcceptedPracticalDiagnostic {
    pub const fn family(&self) -> PracticalDiagnosticFamily {
        self.family
    }

    pub const fn location(&self) -> Option<PracticalDiagnosticLocation> {
        self.location
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalFrontendOutcome {
    Success,
    Rejected,
}

#[derive(Clone, Debug)]
pub struct AcceptedPracticalFrontendEnvelope {
    outcome: PracticalFrontendOutcome,
    phase: Option<u8>,
    diagnostics: Vec<AcceptedPracticalDiagnostic>,
    artifacts_sha256: Option<String>,
    canonical_transport: Vec<u8>,
}

impl AcceptedPracticalFrontendEnvelope {
    pub const fn outcome(&self) -> PracticalFrontendOutcome {
        self.outcome
    }

    pub const fn phase(&self) -> Option<u8> {
        self.phase
    }

    pub fn diagnostics(&self) -> &[AcceptedPracticalDiagnostic] {
        &self.diagnostics
    }

    pub fn artifacts_sha256(&self) -> Option<&str> {
        self.artifacts_sha256.as_deref()
    }

    pub fn canonical_transport(&self) -> &[u8] {
        &self.canonical_transport
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PracticalFrontendValidationContext<'a> {
    pub raw_request: &'a [u8],
    pub validated_request: Option<&'a ValidatedPracticalFrontendRequest>,
    pub artifact_context: Option<&'a PracticalArtifactContext>,
    pub captured_inputs: Option<&'a CapturedInputSet>,
    pub expected_artifacts: Option<&'a ValidatedPracticalArtifact>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalFrontendProtocolPhase {
    Transport,
    Shape,
    Schema,
    Request,
    Inventory,
    Hash,
    Linkage,
    Diagnostic,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalFrontendProtocolCode {
    ProcessKilled,
    Missing,
    Truncated,
    Json,
    DuplicateField,
    FieldOrder,
    Shape,
    Schema,
    Limit,
    SemanticRequest,
    Inventory,
    Hash,
    Linkage,
    StatusExit,
    UnexpectedUsage,
    DiagnosticPhase,
    DiagnosticOrder,
    DiagnosticLocation,
    PublicData,
    PartialArtifacts,
    Artifact,
}

impl PracticalFrontendProtocolCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessKilled => "CSHARP_PRACTICAL_FRONTEND_PROCESS_KILLED",
            Self::Missing => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_MISSING",
            Self::Truncated => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_TRUNCATED",
            Self::Json => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_JSON",
            Self::DuplicateField => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_DUPLICATE_FIELD",
            Self::FieldOrder => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_FIELD_ORDER",
            Self::Shape => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_SHAPE",
            Self::Schema => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_SCHEMA",
            Self::Limit => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_LIMIT",
            Self::SemanticRequest => "CSHARP_PRACTICAL_FRONTEND_SEMANTIC_REQUEST",
            Self::Inventory => "CSHARP_PRACTICAL_FRONTEND_INVENTORY",
            Self::Hash => "CSHARP_PRACTICAL_FRONTEND_PROTOCOL_HASH",
            Self::Linkage => "CSHARP_PRACTICAL_FRONTEND_LINKAGE",
            Self::StatusExit => "CSHARP_PRACTICAL_FRONTEND_STATUS_EXIT",
            Self::UnexpectedUsage => "CSHARP_PRACTICAL_FRONTEND_UNEXPECTED_USAGE",
            Self::DiagnosticPhase => "CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_PHASE",
            Self::DiagnosticOrder => "CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_ORDER",
            Self::DiagnosticLocation => "CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_LOCATION",
            Self::PublicData => "CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_PUBLIC_DATA",
            Self::PartialArtifacts => "CSHARP_PRACTICAL_FRONTEND_PARTIAL_ARTIFACTS",
            Self::Artifact => "CSHARP_PRACTICAL_FRONTEND_ARTIFACT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PracticalFrontendProtocolError {
    phase: PracticalFrontendProtocolPhase,
    code: PracticalFrontendProtocolCode,
}

impl PracticalFrontendProtocolError {
    pub const fn phase(&self) -> PracticalFrontendProtocolPhase {
        self.phase
    }

    pub const fn code(&self) -> PracticalFrontendProtocolCode {
        self.code
    }
}

impl fmt::Display for PracticalFrontendProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for PracticalFrontendProtocolError {}

pub fn build_csharp_practical_frontend_request(
    semantic_request: &ValidatedSuccessorRequest,
    captured_inputs: &CapturedInputSet,
    sidecars: &[PracticalSidecarDescriptor],
) -> Result<ValidatedPracticalFrontendRequest, PracticalFrontendProtocolError> {
    finalize_request(semantic_request.clone(), captured_inputs, sidecars)
}

pub fn import_csharp_practical_frontend_request(
    registry: &ValidatedSuccessorRegistry,
    captured_inputs: &CapturedInputSet,
    transport: &[u8],
) -> Result<ValidatedPracticalFrontendRequest, PracticalFrontendProtocolError> {
    if transport.len() > CSHARP_PRACTICAL_FRONTEND_REQUEST_BYTES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    let value = parse_protocol_json(transport)?;
    require_exact_fields(&value, FRONTEND_REQUEST_FIELDS)?;
    if string_field(&value, "schema")? != CSHARP_PRACTICAL_FRONTEND_REQUEST_SCHEMA {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Schema,
            PracticalFrontendProtocolCode::Schema,
        ));
    }
    let semantic_value = field(&value, "semantic_request")?;
    require_exact_fields(semantic_value, SEMANTIC_REQUEST_FIELDS)?;
    if string_field(semantic_value, "schema")? != SUCCESSOR_VALIDATED_REQUEST_SCHEMA {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Request,
            PracticalFrontendProtocolCode::SemanticRequest,
        ));
    }
    let semantic_transport = successor_canonical_transport(semantic_value)?;
    let semantic_request = validate_successor_semantic_request(registry, &semantic_transport)
        .map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            )
        })?;
    validate_source_snapshot(field(&value, "source_snapshot")?)?;
    let sidecars = parse_sidecar_descriptors(field(&value, "sidecars")?)?;
    let expected = finalize_request(semantic_request, captured_inputs, &sidecars)?;
    if expected.value != value || expected.canonical_bytes != transport {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    Ok(expected)
}

fn validate_source_snapshot(
    value: &PracticalJsonValue,
) -> Result<(), PracticalFrontendProtocolError> {
    require_exact_fields(value, SOURCE_SNAPSHOT_FIELDS)?;
    let entries = array_field(value, "entries")?;
    let mut previous: Option<Vec<u8>> = None;
    for entry in entries {
        require_exact_fields(entry, SOURCE_SNAPSHOT_ENTRY_FIELDS)?;
        let path = string_field(entry, "path")?;
        let raw_sha256 = string_field(entry, "raw_sha256")?;
        let _ = u32_field(entry, "size_bytes")?;
        if path.len() > NORMALIZED_PATH_BYTES_MAX || !valid_sha256(raw_sha256) {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            ));
        }
        let key = canonical_practical_json_bytes(entry).map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            )
        })?;
        if previous.as_ref().is_some_and(|prior| prior >= &key) {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            ));
        }
        previous = Some(key);
    }
    if !valid_sha256(string_field(value, "snapshot_sha256")?) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Hash,
            PracticalFrontendProtocolCode::Hash,
        ));
    }
    Ok(())
}

pub fn emit_csharp_practical_frontend_success(
    request: &ValidatedPracticalFrontendRequest,
    context: &PracticalArtifactContext,
    captured_inputs: &CapturedInputSet,
    artifacts: &ValidatedPracticalArtifact,
) -> Result<Vec<u8>, PracticalFrontendProtocolError> {
    validate_success_lineage(request, context, captured_inputs, artifacts)?;
    let preimage = PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA),
        ),
        (
            "request_sha256",
            PracticalJsonValue::string(request.request_sha256()),
        ),
        ("semantic_context", request.semantic_context.clone()),
        ("artifacts", artifacts.value().clone()),
    ]);
    let success_sha256 = protocol_hash(CSHARP_PRACTICAL_FRONTEND_SUCCESS_HASH_DOMAIN, &preimage)?;
    let value = append_field(
        preimage,
        "success_sha256",
        PracticalJsonValue::string(success_sha256),
    )?;
    canonical_output(&value)
}

pub fn emit_csharp_practical_frontend_diagnostic(
    raw_request: &[u8],
    validated_request: Option<&ValidatedPracticalFrontendRequest>,
    findings: &[PracticalDiagnosticFinding],
) -> Result<Vec<u8>, PracticalFrontendProtocolError> {
    if findings.is_empty() || findings.len() > CSHARP_PRACTICAL_NORMALIZED_ISSUES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    let raw_request_size = raw_request_size(raw_request)?;
    if validated_request.is_some_and(|request| request.canonical_bytes() != raw_request) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    let phase = findings
        .iter()
        .map(|finding| finding.family.phase())
        .min()
        .expect("nonempty findings");
    if validated_request.is_none()
        && (phase != 0
            || findings
                .iter()
                .filter(|finding| finding.family.phase() == phase)
                .any(|finding| finding.location.is_some()))
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::DiagnosticPhase,
        ));
    }
    let mut selected = findings
        .iter()
        .copied()
        .filter(|finding| finding.family.phase() == phase)
        .collect::<Vec<_>>();
    for finding in &selected {
        validate_location(validated_request, finding.location)?;
    }
    selected.sort_by_key(diagnostic_key);
    let diagnostics = selected.iter().map(diagnostic_value).collect::<Vec<_>>();
    let linkage = diagnostic_linkage(validated_request);
    let preimage = PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA),
        ),
        (
            "raw_request_sha256",
            PracticalJsonValue::string(raw_sha256(raw_request)),
        ),
        (
            "raw_request_size_bytes",
            PracticalJsonValue::U64(u64::from(raw_request_size)),
        ),
        ("request_linkage", linkage),
        (
            "status",
            PracticalJsonValue::string(CSHARP_PRACTICAL_FRONTEND_REJECTED_STATUS),
        ),
        ("phase", PracticalJsonValue::U64(u64::from(phase))),
        ("diagnostics", PracticalJsonValue::Array(diagnostics)),
    ]);
    let diagnostic_sha256 =
        protocol_hash(CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_HASH_DOMAIN, &preimage)?;
    let value = append_field(
        preimage,
        "diagnostic_sha256",
        PracticalJsonValue::string(diagnostic_sha256),
    )?;
    canonical_output(&value)
}

pub fn validate_csharp_practical_frontend_process(
    expected: PracticalFrontendValidationContext<'_>,
    process: FrontendProcessFacts<'_>,
) -> Result<AcceptedPracticalFrontendEnvelope, PracticalFrontendProtocolError> {
    validate_expected_context(expected)?;
    if process.stdout.len() > CSHARP_PRACTICAL_FRONTEND_STDOUT_BYTES_MAX
        || process.stderr_observed_bytes > CSHARP_PRACTICAL_FRONTEND_STDERR_BYTES_MAX
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Output,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    if process.signaled || process.exit_code.is_none() {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Output,
            PracticalFrontendProtocolCode::ProcessKilled,
        ));
    }
    let exit_code = process.exit_code.expect("checked exit code");
    if exit_code == 2 {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Output,
            PracticalFrontendProtocolCode::UnexpectedUsage,
        ));
    }
    if process.stdout.is_empty() {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Missing,
        ));
    }
    if !process.stdout.ends_with(b"\n") {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Truncated,
        ));
    }
    let document = &process.stdout[..process.stdout.len() - 1];
    if document.is_empty() {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Json,
        ));
    }
    let value = parse_protocol_json(document)?;
    match string_field(&value, "schema")? {
        CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA => {
            if exit_code != 0 {
                return Err(protocol(
                    PracticalFrontendProtocolPhase::Output,
                    PracticalFrontendProtocolCode::StatusExit,
                ));
            }
            validate_success_value(expected, &value, process.stdout)
        }
        CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA => {
            if exit_code != 3 {
                return Err(protocol(
                    PracticalFrontendProtocolPhase::Output,
                    PracticalFrontendProtocolCode::StatusExit,
                ));
            }
            validate_diagnostic_value(expected, &value, process.stdout)
        }
        _ => Err(protocol(
            PracticalFrontendProtocolPhase::Schema,
            PracticalFrontendProtocolCode::Schema,
        )),
    }
}

fn finalize_request(
    semantic_request: ValidatedSuccessorRequest,
    captured_inputs: &CapturedInputSet,
    sidecars: &[PracticalSidecarDescriptor],
) -> Result<ValidatedPracticalFrontendRequest, PracticalFrontendProtocolError> {
    if semantic_request.compiled_profile() != SuccessorCompiledSemanticProfile::CSharpPracticalV1 {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Request,
            PracticalFrontendProtocolCode::SemanticRequest,
        ));
    }
    let semantic_context = practical_semantic_context_value(semantic_request.semantic_context())
        .map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            )
        })?;
    let semantic_value = normalized_semantic_request(&semantic_request, &semantic_context)?;
    let inventory = build_inventory(&semantic_request, captured_inputs, sidecars)?;
    let preimage = PracticalJsonValue::object(vec![
        (
            "schema",
            PracticalJsonValue::string(CSHARP_PRACTICAL_FRONTEND_REQUEST_SCHEMA),
        ),
        ("semantic_request", semantic_value),
        ("source_snapshot", inventory.source_snapshot),
        ("sidecars", inventory.sidecar_set),
    ]);
    let request_sha256 = protocol_hash(CSHARP_PRACTICAL_FRONTEND_REQUEST_HASH_DOMAIN, &preimage)?;
    let value = append_field(
        preimage,
        "request_sha256",
        PracticalJsonValue::string(request_sha256),
    )?;
    let canonical_bytes = canonical_practical_json_bytes(&value).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Json,
        )
    })?;
    if canonical_bytes.len() > CSHARP_PRACTICAL_FRONTEND_REQUEST_BYTES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    Ok(ValidatedPracticalFrontendRequest {
        semantic_request,
        semantic_context,
        value,
        canonical_bytes,
        source_sizes: inventory.source_sizes,
        input_set_sha256: captured_inputs.snapshot_sha256().to_owned(),
    })
}

struct PracticalFrontendInventory {
    source_snapshot: PracticalJsonValue,
    sidecar_set: PracticalJsonValue,
    source_sizes: Vec<u32>,
}

fn build_inventory(
    request: &ValidatedSuccessorRequest,
    captured_inputs: &CapturedInputSet,
    sidecars: &[PracticalSidecarDescriptor],
) -> Result<PracticalFrontendInventory, PracticalFrontendProtocolError> {
    let source_paths = selection_paths(request, "source_paths")?;
    let sidecar_paths = selection_paths(request, "sidecar_paths")?;
    let actual_sources = captured_inputs
        .entries()
        .iter()
        .filter(|entry| entry.kind() == OriginalInputKind::Source)
        .collect::<Vec<_>>();
    let actual_sidecars = captured_inputs
        .entries()
        .iter()
        .filter(|entry| entry.kind() == OriginalInputKind::Sidecar)
        .collect::<Vec<_>>();
    if actual_sources
        .iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>()
        != source_paths.iter().map(String::as_str).collect::<Vec<_>>()
        || actual_sidecars
            .iter()
            .map(|entry| entry.path())
            .collect::<Vec<_>>()
            != sidecar_paths.iter().map(String::as_str).collect::<Vec<_>>()
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Inventory,
            PracticalFrontendProtocolCode::Inventory,
        ));
    }
    validate_inventory_limits(&actual_sources, &actual_sidecars)?;

    let mut sidecar_schemas = BTreeMap::new();
    for sidecar in sidecars {
        if !REGISTERED_SIDECAR_SCHEMAS.contains(&sidecar.schema.as_str())
            || sidecar.path.len() > NORMALIZED_PATH_BYTES_MAX
            || sidecar_schemas
                .insert(sidecar.path.as_str(), sidecar.schema.as_str())
                .is_some()
        {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            ));
        }
    }
    if sidecar_schemas.keys().copied().collect::<Vec<_>>()
        != sidecar_paths.iter().map(String::as_str).collect::<Vec<_>>()
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Inventory,
            PracticalFrontendProtocolCode::Inventory,
        ));
    }

    let source_sizes = actual_sources
        .iter()
        .map(|entry| u32::try_from(entry.bytes().len()).expect("source limit fits u32"))
        .collect::<Vec<_>>();
    let source_entries = actual_sources
        .iter()
        .zip(&source_sizes)
        .map(|(entry, size)| {
            PracticalJsonValue::object(vec![
                ("path", PracticalJsonValue::string(entry.path())),
                ("raw_sha256", PracticalJsonValue::string(entry.raw_sha256())),
                ("size_bytes", PracticalJsonValue::U64(u64::from(*size))),
            ])
        })
        .collect::<Vec<_>>();
    let source_snapshot_sha256 = retained_subset_hash(&actual_sources, InputKind::Source)?;
    let source_snapshot = PracticalJsonValue::object(vec![
        ("entries", PracticalJsonValue::Array(source_entries)),
        (
            "snapshot_sha256",
            PracticalJsonValue::string(source_snapshot_sha256),
        ),
    ]);

    let mut sidecar_entries = actual_sidecars
        .iter()
        .map(|entry| {
            let schema = sidecar_schemas[entry.path()];
            validate_captured_sidecar_schema(entry, schema)?;
            let value = PracticalJsonValue::object(vec![
                ("schema", PracticalJsonValue::string(schema)),
                ("path", PracticalJsonValue::string(entry.path())),
                ("raw_sha256", PracticalJsonValue::string(entry.raw_sha256())),
            ]);
            let key = canonical_practical_json_bytes(&value).map_err(|_| {
                protocol(
                    PracticalFrontendProtocolPhase::Inventory,
                    PracticalFrontendProtocolCode::Inventory,
                )
            })?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, PracticalFrontendProtocolError>>()?;
    sidecar_entries.sort_by(|left, right| left.0.cmp(&right.0));
    if sidecar_entries
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Inventory,
            PracticalFrontendProtocolCode::Inventory,
        ));
    }
    let sidecar_set_sha256 = retained_subset_hash(&actual_sidecars, InputKind::Contract)?;
    let sidecar_set = PracticalJsonValue::object(vec![
        (
            "entries",
            PracticalJsonValue::Array(
                sidecar_entries
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect(),
            ),
        ),
        ("set_sha256", PracticalJsonValue::string(sidecar_set_sha256)),
    ]);
    Ok(PracticalFrontendInventory {
        source_snapshot,
        sidecar_set,
        source_sizes,
    })
}

fn validate_inventory_limits(
    sources: &[&mpk_vc::csharp_practical_source_artifacts::CapturedOriginalInput],
    sidecars: &[&mpk_vc::csharp_practical_source_artifacts::CapturedOriginalInput],
) -> Result<(), PracticalFrontendProtocolError> {
    let source_total = checked_total(sources)?;
    let sidecar_total = checked_total(sidecars)?;
    let snapshot_total = source_total
        .checked_add(sidecar_total)
        .ok_or_else(limit_error)?;
    if sources.is_empty()
        || sources.len() > SOURCE_FILES_MAX
        || sidecars.len() > SIDECAR_FILES_MAX
        || sources.len() + sidecars.len() > SNAPSHOT_ENTRIES_MAX
        || source_total > SOURCE_TOTAL_BYTES_MAX
        || sidecar_total > SIDECAR_TOTAL_BYTES_MAX
        || snapshot_total > SNAPSHOT_TOTAL_BYTES_MAX
        || sources
            .iter()
            .any(|entry| entry.bytes().len() > SOURCE_FILE_BYTES_MAX)
        || sidecars
            .iter()
            .any(|entry| entry.bytes().len() > SIDECAR_FILE_BYTES_MAX)
        || sources
            .iter()
            .chain(sidecars)
            .any(|entry| entry.path().len() > NORMALIZED_PATH_BYTES_MAX)
    {
        return Err(limit_error());
    }
    Ok(())
}

fn checked_total(
    entries: &[&mpk_vc::csharp_practical_source_artifacts::CapturedOriginalInput],
) -> Result<usize, PracticalFrontendProtocolError> {
    entries.iter().try_fold(0_usize, |total, entry| {
        total
            .checked_add(entry.bytes().len())
            .ok_or_else(limit_error)
    })
}

fn retained_subset_hash(
    entries: &[&mpk_vc::csharp_practical_source_artifacts::CapturedOriginalInput],
    kind: InputKind,
) -> Result<String, PracticalFrontendProtocolError> {
    let values = entries
        .iter()
        .map(|entry| InputEntry {
            kind,
            normalized_path: entry.path().to_owned(),
            size_bytes: i64::try_from(entry.bytes().len()).expect("u32 input size fits i64"),
            sha256: entry.raw_sha256().to_owned(),
        })
        .collect::<Vec<_>>();
    input_set_hash(&values)
        .map(|digest| digest.as_str().to_owned())
        .map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Hash,
                PracticalFrontendProtocolCode::Hash,
            )
        })
}

fn selection_paths(
    request: &ValidatedSuccessorRequest,
    name: &str,
) -> Result<Vec<String>, PracticalFrontendProtocolError> {
    request
        .selection()
        .get(name)
        .and_then(Value::as_array)
        .and_then(|values| {
            values
                .iter()
                .map(|value| value.as_str().map(str::to_owned))
                .collect()
        })
        .ok_or_else(|| {
            protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            )
        })
}

fn normalized_semantic_request(
    request: &ValidatedSuccessorRequest,
    semantic_context: &PracticalJsonValue,
) -> Result<PracticalJsonValue, PracticalFrontendProtocolError> {
    const SELECTION_FIELDS: &[&str] = &[
        "schema",
        "compilation_id",
        "source_paths",
        "selected_root_ids",
        "sidecar_paths",
        "selection_sha256",
    ];
    let selection = request.selection().as_object().ok_or_else(|| {
        protocol(
            PracticalFrontendProtocolPhase::Request,
            PracticalFrontendProtocolCode::SemanticRequest,
        )
    })?;
    let mut normalized_selection = Vec::with_capacity(SELECTION_FIELDS.len());
    for name in SELECTION_FIELDS {
        let value = selection.get(*name).ok_or_else(|| {
            protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            )
        })?;
        normalized_selection.push(((*name).to_owned(), serde_to_practical(value)?));
    }
    Ok(PracticalJsonValue::object(vec![
        ("schema", PracticalJsonValue::string(request.schema())),
        ("semantic_context", semantic_context.clone()),
        (
            "selection",
            PracticalJsonValue::Object(normalized_selection),
        ),
        (
            "request_sha256",
            PracticalJsonValue::string(request.request_sha256()),
        ),
    ]))
}

fn parse_sidecar_descriptors(
    value: &PracticalJsonValue,
) -> Result<Vec<PracticalSidecarDescriptor>, PracticalFrontendProtocolError> {
    require_exact_fields(value, SIDECAR_SET_FIELDS)?;
    let entries = array_field(value, "entries")?;
    let mut previous: Option<Vec<u8>> = None;
    let mut descriptors = Vec::with_capacity(entries.len());
    for entry in entries {
        require_exact_fields(entry, SIDECAR_REF_FIELDS)?;
        let schema = string_field(entry, "schema")?;
        let path = string_field(entry, "path")?;
        let raw_sha256 = string_field(entry, "raw_sha256")?;
        if !REGISTERED_SIDECAR_SCHEMAS.contains(&schema)
            || !valid_sha256(raw_sha256)
            || path.len() > NORMALIZED_PATH_BYTES_MAX
        {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            ));
        }
        let key = canonical_practical_json_bytes(entry).map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            )
        })?;
        if previous.as_ref().is_some_and(|prior| prior >= &key) {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Inventory,
                PracticalFrontendProtocolCode::Inventory,
            ));
        }
        previous = Some(key);
        descriptors.push(PracticalSidecarDescriptor {
            schema: schema.to_owned(),
            path: path.to_owned(),
        });
    }
    if !valid_sha256(string_field(value, "set_sha256")?) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Hash,
            PracticalFrontendProtocolCode::Hash,
        ));
    }
    Ok(descriptors)
}

fn validate_success_lineage(
    request: &ValidatedPracticalFrontendRequest,
    context: &PracticalArtifactContext,
    captured_inputs: &CapturedInputSet,
    artifacts: &ValidatedPracticalArtifact,
) -> Result<(), PracticalFrontendProtocolError> {
    if !context.matches_request(request.semantic_request())
        || request.input_set_sha256() != captured_inputs.snapshot_sha256()
        || artifacts.kind() != PracticalArtifactKind::SourceArtifacts
        || artifacts.schema() != SOURCE_ARTIFACTS_SCHEMA
        || !artifacts.matches_validated_lineage(context, captured_inputs)
        || artifacts.value().get("semantic_context") != Some(request.semantic_context())
        || artifacts
            .value()
            .get("selection_sha256")
            .and_then(PracticalJsonValue::as_str)
            != request
                .semantic_request()
                .selection()
                .get("selection_sha256")
                .and_then(Value::as_str)
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Artifact,
        ));
    }
    Ok(())
}

fn validate_expected_context(
    expected: PracticalFrontendValidationContext<'_>,
) -> Result<(), PracticalFrontendProtocolError> {
    raw_request_size(expected.raw_request)?;
    if let Some(request) = expected.validated_request {
        if request.canonical_bytes() != expected.raw_request {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Linkage,
                PracticalFrontendProtocolCode::Linkage,
            ));
        }
    } else if expected.artifact_context.is_some()
        || expected.captured_inputs.is_some()
        || expected.expected_artifacts.is_some()
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    Ok(())
}

fn validate_success_value(
    expected: PracticalFrontendValidationContext<'_>,
    value: &PracticalJsonValue,
    transport: &[u8],
) -> Result<AcceptedPracticalFrontendEnvelope, PracticalFrontendProtocolError> {
    require_exact_fields(value, SUCCESS_FIELDS)?;
    let request = expected.validated_request.ok_or_else(|| {
        protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        )
    })?;
    let context = expected.artifact_context.ok_or_else(|| {
        protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        )
    })?;
    let captured_inputs = expected.captured_inputs.ok_or_else(|| {
        protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        )
    })?;
    let artifacts = expected.expected_artifacts.ok_or_else(|| {
        protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Artifact,
        )
    })?;
    validate_success_lineage(request, context, captured_inputs, artifacts)?;
    if string_field(value, "schema")? != CSHARP_PRACTICAL_FRONTEND_SUCCESS_SCHEMA
        || string_field(value, "request_sha256")? != request.request_sha256()
        || field(value, "semantic_context")? != request.semantic_context()
        || field(value, "artifacts")? != artifacts.value()
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    validate_last_hash(
        value,
        "success_sha256",
        CSHARP_PRACTICAL_FRONTEND_SUCCESS_HASH_DOMAIN,
    )?;
    Ok(AcceptedPracticalFrontendEnvelope {
        outcome: PracticalFrontendOutcome::Success,
        phase: None,
        diagnostics: Vec::new(),
        artifacts_sha256: Some(artifacts.hash().to_owned()),
        canonical_transport: transport.to_vec(),
    })
}

fn validate_diagnostic_value(
    expected: PracticalFrontendValidationContext<'_>,
    value: &PracticalJsonValue,
    transport: &[u8],
) -> Result<AcceptedPracticalFrontendEnvelope, PracticalFrontendProtocolError> {
    if value.get("artifacts").is_some() {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::PartialArtifacts,
        ));
    }
    require_exact_fields(value, DIAGNOSTIC_FIELDS)?;
    if string_field(value, "schema")? != CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_SCHEMA
        || string_field(value, "status")? != CSHARP_PRACTICAL_FRONTEND_REJECTED_STATUS
        || string_field(value, "raw_request_sha256")? != raw_sha256(expected.raw_request)
        || u32_field(value, "raw_request_size_bytes")? != raw_request_size(expected.raw_request)?
    {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    if field(value, "request_linkage")? != &diagnostic_linkage(expected.validated_request) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Linkage,
            PracticalFrontendProtocolCode::Linkage,
        ));
    }
    let phase = u8::try_from(u64_field(value, "phase")?).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::DiagnosticPhase,
        )
    })?;
    if phase > 8 {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::DiagnosticPhase,
        ));
    }
    let entries = array_field(value, "diagnostics")?;
    if entries.is_empty() || entries.len() > CSHARP_PRACTICAL_NORMALIZED_ISSUES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Diagnostic,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    let mut diagnostics = Vec::with_capacity(entries.len());
    let mut previous = None;
    for entry in entries {
        require_exact_fields(entry, DIAGNOSTIC_ENTRY_FIELDS)?;
        let family =
            PracticalDiagnosticFamily::parse(string_field(entry, "code")?).ok_or_else(|| {
                protocol(
                    PracticalFrontendProtocolPhase::Diagnostic,
                    PracticalFrontendProtocolCode::PublicData,
                )
            })?;
        if family.phase() != phase
            || string_field(entry, "message")? != CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE
        {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Diagnostic,
                if family.phase() != phase {
                    PracticalFrontendProtocolCode::DiagnosticPhase
                } else {
                    PracticalFrontendProtocolCode::PublicData
                },
            ));
        }
        let location = parse_location(field(entry, "location")?)?;
        if expected.validated_request.is_none() && (phase != 0 || location.is_some()) {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Diagnostic,
                PracticalFrontendProtocolCode::DiagnosticPhase,
            ));
        }
        validate_location(expected.validated_request, location)?;
        let finding = PracticalDiagnosticFinding { family, location };
        let key = diagnostic_key(&finding);
        if previous.as_ref().is_some_and(|prior| prior > &key) {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Diagnostic,
                PracticalFrontendProtocolCode::DiagnosticOrder,
            ));
        }
        previous = Some(key);
        diagnostics.push(AcceptedPracticalDiagnostic { family, location });
    }
    validate_last_hash(
        value,
        "diagnostic_sha256",
        CSHARP_PRACTICAL_FRONTEND_DIAGNOSTIC_HASH_DOMAIN,
    )?;
    Ok(AcceptedPracticalFrontendEnvelope {
        outcome: PracticalFrontendOutcome::Rejected,
        phase: Some(phase),
        diagnostics,
        artifacts_sha256: None,
        canonical_transport: transport.to_vec(),
    })
}

fn validate_location(
    request: Option<&ValidatedPracticalFrontendRequest>,
    location: Option<PracticalDiagnosticLocation>,
) -> Result<(), PracticalFrontendProtocolError> {
    let Some(location) = location else {
        return Ok(());
    };
    let size = request
        .and_then(|request| {
            request
                .source_sizes
                .get(usize::from(location.source_file_ordinal))
        })
        .copied()
        .ok_or_else(location_error)?;
    if location.start_byte >= location.end_byte || location.end_byte > size {
        return Err(location_error());
    }
    Ok(())
}

fn diagnostic_linkage(request: Option<&ValidatedPracticalFrontendRequest>) -> PracticalJsonValue {
    match request {
        None => {
            PracticalJsonValue::object(vec![("state", PracticalJsonValue::string("unvalidated"))])
        }
        Some(request) => PracticalJsonValue::object(vec![
            ("state", PracticalJsonValue::string("validated")),
            (
                "request_sha256",
                PracticalJsonValue::string(request.request_sha256()),
            ),
            ("semantic_context", request.semantic_context.clone()),
        ]),
    }
}

fn diagnostic_value(finding: &PracticalDiagnosticFinding) -> PracticalJsonValue {
    PracticalJsonValue::object(vec![
        ("code", PracticalJsonValue::string(finding.family.as_str())),
        (
            "message",
            PracticalJsonValue::string(CSHARP_PRACTICAL_PUBLIC_DIAGNOSTIC_MESSAGE),
        ),
        (
            "location",
            finding
                .location
                .map_or(PracticalJsonValue::Null, |location| {
                    PracticalJsonValue::object(vec![
                        (
                            "source_file_ordinal",
                            PracticalJsonValue::U64(u64::from(location.source_file_ordinal)),
                        ),
                        (
                            "start_byte",
                            PracticalJsonValue::U64(u64::from(location.start_byte)),
                        ),
                        (
                            "end_byte",
                            PracticalJsonValue::U64(u64::from(location.end_byte)),
                        ),
                    ])
                }),
        ),
    ])
}

fn parse_location(
    value: &PracticalJsonValue,
) -> Result<Option<PracticalDiagnosticLocation>, PracticalFrontendProtocolError> {
    if value == &PracticalJsonValue::Null {
        return Ok(None);
    }
    require_exact_fields(value, SOURCE_LOCATION_FIELDS)?;
    let source_file_ordinal =
        u16::try_from(u64_field(value, "source_file_ordinal")?).map_err(|_| location_error())?;
    let start_byte = u32_field(value, "start_byte")?;
    let end_byte = u32_field(value, "end_byte")?;
    Ok(Some(PracticalDiagnosticLocation {
        source_file_ordinal,
        start_byte,
        end_byte,
    }))
}

fn diagnostic_key(
    finding: &PracticalDiagnosticFinding,
) -> (u8, Option<PracticalDiagnosticLocation>, &'static str) {
    (
        finding.family.precedence(),
        finding.location,
        finding.family.as_str(),
    )
}

fn canonical_output(value: &PracticalJsonValue) -> Result<Vec<u8>, PracticalFrontendProtocolError> {
    let mut bytes = canonical_practical_json_bytes(value).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Output,
            PracticalFrontendProtocolCode::Json,
        )
    })?;
    bytes.push(b'\n');
    if bytes.len() > CSHARP_PRACTICAL_FRONTEND_STDOUT_BYTES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Output,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    Ok(bytes)
}

fn validate_last_hash(
    value: &PracticalJsonValue,
    hash_field: &str,
    domain: HashDomain,
) -> Result<(), PracticalFrontendProtocolError> {
    let entries = value.as_object().ok_or_else(shape_error)?;
    let (last_name, last_value) = entries.last().ok_or_else(shape_error)?;
    let actual = last_value.as_str().filter(|value| valid_sha256(value));
    let preimage = PracticalJsonValue::Object(entries[..entries.len() - 1].to_vec());
    if last_name != hash_field || actual != Some(protocol_hash(domain, &preimage)?.as_str()) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Hash,
            PracticalFrontendProtocolCode::Hash,
        ));
    }
    Ok(())
}

fn protocol_hash(
    domain: HashDomain,
    value: &PracticalJsonValue,
) -> Result<String, PracticalFrontendProtocolError> {
    let bytes = canonical_practical_json_bytes(value).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Hash,
            PracticalFrontendProtocolCode::Hash,
        )
    })?;
    hash_domain_separated_raw(domain, &bytes)
        .map(|digest| digest.to_hex())
        .map_err(|_| {
            protocol(
                PracticalFrontendProtocolPhase::Hash,
                PracticalFrontendProtocolCode::Hash,
            )
        })
}

fn raw_sha256(bytes: &[u8]) -> String {
    sha256_raw_file_bytes(bytes).to_hex()
}

fn raw_request_size(bytes: &[u8]) -> Result<u32, PracticalFrontendProtocolError> {
    u32::try_from(bytes.len()).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Limit,
        )
    })
}

fn validate_captured_sidecar_schema(
    entry: &mpk_vc::csharp_practical_source_artifacts::CapturedOriginalInput,
    expected_schema: &str,
) -> Result<(), PracticalFrontendProtocolError> {
    let value =
        parse_canonical_practical_json(PracticalArtifactKind::SourceArtifacts, entry.bytes())
            .map_err(|_| {
                protocol(
                    PracticalFrontendProtocolPhase::Inventory,
                    PracticalFrontendProtocolCode::Inventory,
                )
            })?;
    if value.get("schema").and_then(PracticalJsonValue::as_str) != Some(expected_schema) {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Inventory,
            PracticalFrontendProtocolCode::Inventory,
        ));
    }
    Ok(())
}

fn append_field(
    value: PracticalJsonValue,
    name: &str,
    field_value: PracticalJsonValue,
) -> Result<PracticalJsonValue, PracticalFrontendProtocolError> {
    let PracticalJsonValue::Object(mut entries) = value else {
        return Err(shape_error());
    };
    entries.push((name.to_owned(), field_value));
    Ok(PracticalJsonValue::Object(entries))
}

fn parse_protocol_json(
    transport: &[u8],
) -> Result<PracticalJsonValue, PracticalFrontendProtocolError> {
    if transport.len() > PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX {
        return Err(protocol(
            PracticalFrontendProtocolPhase::Transport,
            PracticalFrontendProtocolCode::Limit,
        ));
    }
    parse_canonical_practical_json(PracticalArtifactKind::SourceArtifacts, transport).map_err(
        |error| {
            protocol(
                PracticalFrontendProtocolPhase::Transport,
                if error.code() == PracticalArtifactErrorCode::DuplicateField {
                    PracticalFrontendProtocolCode::DuplicateField
                } else {
                    PracticalFrontendProtocolCode::Json
                },
            )
        },
    )
}

fn successor_canonical_transport(
    value: &PracticalJsonValue,
) -> Result<Vec<u8>, PracticalFrontendProtocolError> {
    let serde = practical_to_serde(value)?;
    let strict = serde_to_strict(&serde)?;
    canonical_json_bytes(&strict).map_err(|_| {
        protocol(
            PracticalFrontendProtocolPhase::Request,
            PracticalFrontendProtocolCode::SemanticRequest,
        )
    })
}

fn practical_to_serde(value: &PracticalJsonValue) -> Result<Value, PracticalFrontendProtocolError> {
    Ok(match value {
        PracticalJsonValue::Null => Value::Null,
        PracticalJsonValue::Bool(value) => Value::Bool(*value),
        PracticalJsonValue::I64(value) => Value::Number(Number::from(*value)),
        PracticalJsonValue::U64(value) => Value::Number(Number::from(*value)),
        PracticalJsonValue::String(value) => Value::String(value.clone()),
        PracticalJsonValue::Utf16String(_) => {
            return Err(protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            ));
        }
        PracticalJsonValue::Array(values) => Value::Array(
            values
                .iter()
                .map(practical_to_serde)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        PracticalJsonValue::Object(entries) => {
            let mut object = Map::new();
            for (name, value) in entries {
                object.insert(name.clone(), practical_to_serde(value)?);
            }
            Value::Object(object)
        }
    })
}

fn serde_to_practical(value: &Value) -> Result<PracticalJsonValue, PracticalFrontendProtocolError> {
    Ok(match value {
        Value::Null => PracticalJsonValue::Null,
        Value::Bool(value) => PracticalJsonValue::Bool(*value),
        Value::Number(value) => value
            .as_u64()
            .map(PracticalJsonValue::U64)
            .or_else(|| value.as_i64().map(PracticalJsonValue::I64))
            .ok_or_else(shape_error)?,
        Value::String(value) => PracticalJsonValue::String(value.clone()),
        Value::Array(values) => PracticalJsonValue::Array(
            values
                .iter()
                .map(serde_to_practical)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(values) => PracticalJsonValue::Object(
            values
                .iter()
                .map(|(name, value)| Ok((name.clone(), serde_to_practical(value)?)))
                .collect::<Result<Vec<_>, PracticalFrontendProtocolError>>()?,
        ),
    })
}

fn serde_to_strict(value: &Value) -> Result<StrictJsonValue, PracticalFrontendProtocolError> {
    Ok(match value {
        Value::Null => StrictJsonValue::Null,
        Value::Bool(value) => StrictJsonValue::Bool(*value),
        Value::Number(value) => StrictJsonValue::Integer(value.as_i64().ok_or_else(|| {
            protocol(
                PracticalFrontendProtocolPhase::Request,
                PracticalFrontendProtocolCode::SemanticRequest,
            )
        })?),
        Value::String(value) => StrictJsonValue::String(value.clone()),
        Value::Array(values) => StrictJsonValue::Array(
            values
                .iter()
                .map(serde_to_strict)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(values) => StrictJsonValue::Object(
            values
                .iter()
                .map(|(name, value)| Ok((name.clone(), serde_to_strict(value)?)))
                .collect::<Result<Vec<_>, PracticalFrontendProtocolError>>()?,
        ),
    })
}

fn require_exact_fields(
    value: &PracticalJsonValue,
    expected: &[&str],
) -> Result<(), PracticalFrontendProtocolError> {
    let object = value.as_object().ok_or_else(shape_error)?;
    let actual = object
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    if actual == expected {
        return Ok(());
    }
    let actual_set = actual.iter().copied().collect::<BTreeSet<_>>();
    let expected_set = expected.iter().copied().collect::<BTreeSet<_>>();
    Err(protocol(
        PracticalFrontendProtocolPhase::Shape,
        if actual_set == expected_set {
            PracticalFrontendProtocolCode::FieldOrder
        } else {
            PracticalFrontendProtocolCode::Shape
        },
    ))
}

fn field<'a>(
    value: &'a PracticalJsonValue,
    name: &str,
) -> Result<&'a PracticalJsonValue, PracticalFrontendProtocolError> {
    value.get(name).ok_or_else(shape_error)
}

fn string_field<'a>(
    value: &'a PracticalJsonValue,
    name: &str,
) -> Result<&'a str, PracticalFrontendProtocolError> {
    field(value, name)?.as_str().ok_or_else(shape_error)
}

fn array_field<'a>(
    value: &'a PracticalJsonValue,
    name: &str,
) -> Result<&'a [PracticalJsonValue], PracticalFrontendProtocolError> {
    field(value, name)?.as_array().ok_or_else(shape_error)
}

fn u64_field(
    value: &PracticalJsonValue,
    name: &str,
) -> Result<u64, PracticalFrontendProtocolError> {
    field(value, name)?.as_u64().ok_or_else(shape_error)
}

fn u32_field(
    value: &PracticalJsonValue,
    name: &str,
) -> Result<u32, PracticalFrontendProtocolError> {
    u32::try_from(u64_field(value, name)?).map_err(|_| shape_error())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn protocol(
    phase: PracticalFrontendProtocolPhase,
    code: PracticalFrontendProtocolCode,
) -> PracticalFrontendProtocolError {
    PracticalFrontendProtocolError { phase, code }
}

fn shape_error() -> PracticalFrontendProtocolError {
    protocol(
        PracticalFrontendProtocolPhase::Shape,
        PracticalFrontendProtocolCode::Shape,
    )
}

fn limit_error() -> PracticalFrontendProtocolError {
    protocol(
        PracticalFrontendProtocolPhase::Inventory,
        PracticalFrontendProtocolCode::Limit,
    )
}

fn location_error() -> PracticalFrontendProtocolError {
    protocol(
        PracticalFrontendProtocolPhase::Diagnostic,
        PracticalFrontendProtocolCode::DiagnosticLocation,
    )
}
