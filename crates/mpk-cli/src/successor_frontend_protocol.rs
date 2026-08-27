//! Inactive `mpk.frontend.cli.v1` validator.
//!
//! This module has no runner, registry discovery, command-line route, or
//! compatibility adapter. Callers must inject an already validated inactive
//! semantic registry and the complete expected request identity.

use crate::frontend_protocol::{contains_absolute_path, FrontendProcessFacts};
use mpk_vc::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_registry_semantic_context,
    validate_semantic_context_linkage, SelectionEnvelope, SemanticContext,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, SuccessorSourceManifestStage,
    SuccessorSourceManifestValidationContext, SuccessorSourceMapValidationContext,
    ValidatedSuccessorSourceManifest, ValidatedSuccessorSourceMap, ValidatedSuccessorVir,
    SUCCESSOR_VIR_SCHEMA,
};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, CapturedInput, ReleaseRegistryIdentity,
    StrictJsonLimits, SyntheticPermission,
};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

pub const SUCCESSOR_FRONTEND_SCHEMA: &str = "mpk.frontend.cli.v1";
pub const SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX: usize = 268_435_456;
pub const SUCCESSOR_FRONTEND_STDERR_BYTES_MAX: usize = 2_097_152;

const JSON_NODES_MAX: u64 = 16_777_216;
const JSON_NESTING_MAX: u64 = 256;
const STRING_BYTES_MAX: u64 = 1_048_576;
const ISSUES_MAX: usize = 1_024;
const ISSUE_MESSAGE_BYTES_MAX: usize = 4_096;
const ISSUE_MESSAGE_TOTAL_MAX: usize = 2_097_152;

#[derive(Clone, Copy, Debug)]
pub struct SuccessorFrontendProtocolRequest<'a> {
    pub registry: &'a ValidatedSemanticProfileRegistry,
    pub semantic_context: &'a SemanticContext,
    pub selection: &'a SelectionEnvelope,
    pub release_registry: &'a ReleaseRegistryIdentity,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub synthetic_permissions: &'a [SyntheticPermission],
}

#[derive(Clone, Debug)]
pub struct AcceptedSuccessorFrontendEnvelope {
    status: String,
    phase: String,
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    canonical_bytes: Vec<u8>,
    artifacts: Option<AcceptedSuccessorFrontendArtifacts>,
}

impl AcceptedSuccessorFrontendEnvelope {
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn phase(&self) -> &str {
        &self.phase
    }

    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn artifacts(&self) -> Option<&AcceptedSuccessorFrontendArtifacts> {
        self.artifacts.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct AcceptedSuccessorFrontendArtifacts {
    vir: ValidatedSuccessorVir,
    source_map: ValidatedSuccessorSourceMap,
    source_manifest: ValidatedSuccessorSourceManifest,
}

impl AcceptedSuccessorFrontendArtifacts {
    pub fn vir(&self) -> &ValidatedSuccessorVir {
        &self.vir
    }

    pub fn source_map(&self) -> &ValidatedSuccessorSourceMap {
        &self.source_map
    }

    pub fn source_manifest(&self) -> &ValidatedSuccessorSourceManifest {
        &self.source_manifest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorFrontendProtocolCode {
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

impl SuccessorFrontendProtocolCode {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorFrontendProtocolError {
    code: SuccessorFrontendProtocolCode,
}

impl SuccessorFrontendProtocolError {
    pub const fn code(&self) -> SuccessorFrontendProtocolCode {
        self.code
    }
}

impl fmt::Display for SuccessorFrontendProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for SuccessorFrontendProtocolError {}

pub fn validate_successor_frontend_process(
    request: SuccessorFrontendProtocolRequest<'_>,
    process: FrontendProcessFacts<'_>,
) -> Result<AcceptedSuccessorFrontendEnvelope, SuccessorFrontendProtocolError> {
    if process.stdout.len() > SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX
        || process.stderr_observed_bytes > SUCCESSOR_FRONTEND_STDERR_BYTES_MAX
    {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolLimit));
    }
    if process.signaled || process.exit_code.is_none() {
        return Err(protocol(SuccessorFrontendProtocolCode::ProcessKilled));
    }
    let exit = process.exit_code.unwrap_or(1);
    if exit == 2 {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolUnexpectedUsage,
        ));
    }
    if process.stdout.is_empty() {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolMissing));
    }
    if !process.stdout.ends_with(b"\n") {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolTruncated));
    }

    let json = &process.stdout[..process.stdout.len() - 1];
    if json.is_empty() {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolMalformed));
    }
    if json.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolNoncanonical,
        ));
    }
    let strict = parse_strict_json(
        json,
        StrictJsonLimits::new(
            SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX as u64,
            JSON_NODES_MAX,
            JSON_NESTING_MAX,
            STRING_BYTES_MAX,
        ),
    )
    .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolMalformed))?;
    let mut canonical = canonical_json_bytes_bounded(&strict, SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX)
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolLimit))?;
    if canonical != json {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolNoncanonical,
        ));
    }
    let value: Value = serde_json::from_slice(&canonical)
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolMalformed))?;
    let (status, phase) = validate_shape(&value, exit)?;
    validate_public_paths(&value)?;
    let (semantic_context, selection) = validate_identity(&value, request)?;
    let artifacts = if status == "ir-lowered" {
        Some(validate_success_artifacts(&value, request)?)
    } else {
        None
    };
    canonical.push(b'\n');
    Ok(AcceptedSuccessorFrontendEnvelope {
        status: status.to_owned(),
        phase: phase.to_owned(),
        semantic_context,
        selection,
        canonical_bytes: canonical,
        artifacts,
    })
}

fn validate_shape(
    value: &Value,
    exit: i32,
) -> Result<(&str, &str), SuccessorFrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))?;
    let status = string_field(object, "status")?;
    let phase = string_field(object, "phase")?;
    let common = [
        "schema",
        "status",
        "phase",
        "semantic_context",
        "selection",
        "rejected_features",
        "diagnostics",
    ];
    let expected_exit = match status {
        "ir-lowered" => {
            exact_fields(
                object,
                &common
                    .into_iter()
                    .chain(["ir", "source_map", "source_manifest"])
                    .collect::<Vec<_>>(),
            )?;
            if phase != "emission" {
                return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
            }
            validate_ir_wrapper(field(object, "ir")?)?;
            0
        }
        "frontend-error" | "rejected" | "source-error" => {
            exact_fields(object, &common)?;
            match status {
                "frontend-error" => 1,
                "rejected" => 3,
                "source-error" => 4,
                _ => return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape)),
            }
        }
        _ => return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape)),
    };
    if string_field(object, "schema")? != SUCCESSOR_FRONTEND_SCHEMA {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    let rejected = array_field(object, "rejected_features")?;
    let diagnostics = array_field(object, "diagnostics")?;
    validate_issues(rejected, diagnostics, status, phase)?;
    let phase_valid = match status {
        "ir-lowered" => phase == "emission" && rejected.is_empty() && diagnostics.is_empty(),
        "rejected" => {
            matches!(
                phase,
                "capture"
                    | "source"
                    | "metadata"
                    | "typecheck"
                    | "subset"
                    | "lowering"
                    | "emission"
            ) && !rejected.is_empty()
                && diagnostics.is_empty()
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
                    | "release"
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
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    if exit != expected_exit {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolStatusExit));
    }
    Ok((status, phase))
}

fn validate_ir_wrapper(value: &Value) -> Result<(), SuccessorFrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))?;
    exact_fields(object, &["schema", "sha256", "value"])?;
    if string_field(object, "schema")? != SUCCESSOR_VIR_SCHEMA
        || !is_lower_sha256(string_field(object, "sha256")?)
        || !field(object, "value")?.is_object()
    {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn validate_identity(
    value: &Value,
    request: SuccessorFrontendProtocolRequest<'_>,
) -> Result<(SemanticContext, SelectionEnvelope), SuccessorFrontendProtocolError> {
    let context_value = value
        .get("semantic_context")
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolIdentityMismatch))?;
    let semantic_context = validate_registry_semantic_context(request.registry, context_value)
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolIdentityMismatch))?;
    validate_semantic_context_linkage(request.semantic_context, &semantic_context)
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolIdentityMismatch))?;
    let selection_value = value
        .get("selection")
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolIdentityMismatch))?;
    let selection =
        validate_registry_selection_envelope(request.registry, &semantic_context, selection_value)
            .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolIdentityMismatch))?;
    if &selection != request.selection {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolIdentityMismatch,
        ));
    }
    Ok((semantic_context, selection))
}

fn validate_success_artifacts(
    envelope: &Value,
    request: SuccessorFrontendProtocolRequest<'_>,
) -> Result<AcceptedSuccessorFrontendArtifacts, SuccessorFrontendProtocolError> {
    let ir = &envelope["ir"];
    let vir_bytes = serde_json::to_vec(&ir["value"])
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
    let vir = import_successor_vir_json(&vir_bytes, request.registry)
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
    if ir["sha256"].as_str() != Some(vir.hash().as_str())
        || vir.module().semantic_context() != request.semantic_context
    {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolArtifactMismatch,
        ));
    }

    let source_map_bytes = serde_json::to_vec(&envelope["source_map"])
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
    let source_map = import_successor_source_map_json(
        &source_map_bytes,
        SuccessorSourceMapValidationContext {
            registry: request.registry,
            vir: &vir,
            captured_inputs: request.captured_inputs,
            synthetic_permissions: request.synthetic_permissions,
        },
    )
    .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;

    let source_manifest_bytes = serde_json::to_vec(&envelope["source_manifest"])
        .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
    let source_manifest = import_successor_source_manifest_json(
        &source_manifest_bytes,
        SuccessorSourceManifestStage::Frontend,
        SuccessorSourceManifestValidationContext {
            registry: request.registry,
            vir: &vir,
            source_map: &source_map,
            captured_inputs: request.captured_inputs,
            expected_release_registry: request.release_registry,
        },
    )
    .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
    if source_manifest.manifest().semantic_context() != request.semantic_context
        || source_manifest.manifest().selection() != request.selection
    {
        return Err(protocol(
            SuccessorFrontendProtocolCode::ProtocolArtifactMismatch,
        ));
    }
    validate_success_issue_links(envelope, &vir, &source_manifest, request.captured_inputs)?;
    Ok(AcceptedSuccessorFrontendArtifacts {
        vir,
        source_map,
        source_manifest,
    })
}

fn validate_issues(
    rejected: &[Value],
    diagnostics: &[Value],
    status: &str,
    phase: &str,
) -> Result<(), SuccessorFrontendProtocolError> {
    let issue_count = rejected
        .len()
        .checked_add(diagnostics.len())
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolLimit))?;
    if issue_count > ISSUES_MAX {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolLimit));
    }
    let mut total_message_bytes = 0_usize;
    for issues in [rejected, diagnostics] {
        let mut previous: Option<IssueOrderKey<'_>> = None;
        for issue in issues {
            let object = issue
                .as_object()
                .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))?;
            let has_function = object.contains_key("function_id");
            let has_span = object.contains_key("span");
            let mut expected = vec!["code", "message"];
            if has_function {
                expected.push("function_id");
            }
            if has_span {
                expected.push("span");
            }
            exact_fields(object, &expected)?;
            let code = string_field(object, "code")?;
            let message = string_field(object, "message")?;
            if !valid_issue_code(code)
                || message.is_empty()
                || message.chars().any(char::is_control)
                || message.len() > ISSUE_MESSAGE_BYTES_MAX
            {
                return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
            }
            validate_csharp_issue(code, message, status, phase)?;
            total_message_bytes = total_message_bytes
                .checked_add(message.len())
                .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolLimit))?;
            let function = object
                .get("function_id")
                .map(|value| {
                    value
                        .as_str()
                        .filter(|text| !text.is_empty() && text.len() <= 1_024 && text.is_ascii())
                        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))
                })
                .transpose()?
                .unwrap_or("");
            let (path, start, end) = object
                .get("span")
                .map(validate_span)
                .transpose()?
                .unwrap_or(("", -1, -1));
            let key = IssueOrderKey {
                path,
                start,
                code,
                message,
                function,
                end,
            };
            if previous.as_ref().is_some_and(|prior| prior > &key) {
                return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
            }
            previous = Some(key);
        }
    }
    if total_message_bytes > ISSUE_MESSAGE_TOTAL_MAX {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolLimit));
    }
    Ok(())
}

fn validate_csharp_issue(
    code: &str,
    message: &str,
    status: &str,
    phase: &str,
) -> Result<(), SuccessorFrontendProtocolError> {
    if !code.starts_with("CSHARP_") {
        return Ok(());
    }
    let (expected_status, expected_phase) = match code {
        "CSHARP_CAPTURE_FILE_TYPE" | "CSHARP_CAPTURE_PATH" | "CSHARP_CAPTURE_INVENTORY" => {
            ("rejected", Some("capture"))
        }
        "CSHARP_SOURCE_ENCODING" | "CSHARP_SOURCE_PARSE" => ("source-error", Some("source")),
        "CSHARP_SOURCE_DIAGNOSTIC" => ("source-error", Some("metadata")),
        "CSHARP_TOOLCHAIN_ARCHIVE"
        | "CSHARP_TOOLCHAIN_RUNTIME"
        | "CSHARP_TOOLCHAIN_ROSLYN"
        | "CSHARP_TOOLCHAIN_REFERENCE" => ("frontend-error", Some("release")),
        "CSHARP_TOOLCHAIN_OPTIONS" | "CSHARP_TOOLCHAIN_ADAPTER" => ("frontend-error", None),
        "CSHARP_SUBSET_TYPE" | "CSHARP_SUBSET_LITERAL" => ("rejected", Some("typecheck")),
        "CSHARP_SUBSET_DECLARATION"
        | "CSHARP_SUBSET_CONTROL_FLOW"
        | "CSHARP_SUBSET_OPERATION"
        | "CSHARP_SUBSET_OVERFLOW_CONTEXT"
        | "CSHARP_SUBSET_CHECKED_CONVERSION"
        | "CSHARP_SUBSET_CONVERSION"
        | "CSHARP_SUBSET_CALL"
        | "CSHARP_SUBSET_INITIALIZATION"
        | "CSHARP_SUBSET_PURITY"
        | "CSHARP_SUBSET_ABRUPT"
        | "CSHARP_CONTRACT_JSON"
        | "CSHARP_CONTRACT_SHAPE"
        | "CSHARP_CONTRACT_IDENTITY"
        | "CSHARP_CONTRACT_DUPLICATE"
        | "CSHARP_CONTRACT_MISSING"
        | "CSHARP_CONTRACT_UNUSED"
        | "CSHARP_CONTRACT_TYPE"
        | "CSHARP_CONTRACT_OPERATOR"
        | "CSHARP_CONTRACT_HASH" => ("rejected", Some("subset")),
        "CSHARP_LOWERING_OPERATION"
        | "CSHARP_LOWERING_CFG"
        | "CSHARP_LOWERING_CHECK_MISSING"
        | "CSHARP_LOWERING_CHECK_EXTRA"
        | "CSHARP_LOWERING_CHECK_ORDER" => ("rejected", Some("lowering")),
        "CSHARP_SOURCE_MAP_EXTERNAL" | "CSHARP_SOURCE_MAP_RANGE" | "CSHARP_SOURCE_MAP_UTF16" => {
            ("frontend-error", Some("emission"))
        }
        "CSHARP_FRONTEND_OUTPUT_LIMIT"
        | "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET"
        | "CSHARP_FRONTEND_INTERNAL" => ("frontend-error", None),
        "CSHARP_LIMIT_SOURCE_FILES"
        | "CSHARP_LIMIT_SOURCE_FILE_BYTES"
        | "CSHARP_LIMIT_SOURCE_TOTAL_BYTES"
        | "CSHARP_LIMIT_CONTRACT_FILES"
        | "CSHARP_LIMIT_CONTRACT_FILE_BYTES"
        | "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES"
        | "CSHARP_LIMIT_SNAPSHOT_ENTRIES"
        | "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES"
        | "CSHARP_LIMIT_NORMALIZED_PATH_BYTES"
        | "CSHARP_LIMIT_CANONICAL_METHOD_ID_BYTES"
        | "CSHARP_LIMIT_SELECTED_METHODS"
        | "CSHARP_LIMIT_METHOD_CLOSURE"
        | "CSHARP_LIMIT_SYNTAX_NODES"
        | "CSHARP_LIMIT_OPERATIONS_PER_METHOD"
        | "CSHARP_LIMIT_OPERATIONS_PER_CLOSURE"
        | "CSHARP_LIMIT_CFG_BLOCKS_PER_METHOD"
        | "CSHARP_LIMIT_CFG_BLOCKS_PER_CLOSURE"
        | "CSHARP_LIMIT_CONTRACT_CLAUSES"
        | "CSHARP_LIMIT_CONTRACT_NODES_PER_METHOD"
        | "CSHARP_LIMIT_CONTRACT_NODES_PER_CLOSURE"
        | "CSHARP_LIMIT_CONTRACT_DEPTH"
        | "CSHARP_LIMIT_FRONTEND_ARGUMENT_BYTES"
        | "CSHARP_LIMIT_VIR_CANONICAL_BYTES"
        | "CSHARP_LIMIT_SOURCE_MAP_CANONICAL_BYTES"
        | "CSHARP_LIMIT_SOURCE_MANIFEST_CANONICAL_BYTES" => ("rejected", None),
        _ => return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape)),
    };
    if status != expected_status || expected_phase.is_some_and(|expected| phase != expected) {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    let expected_message = if code == "CSHARP_SOURCE_DIAGNOSTIC" {
        let Some(id) = message.strip_prefix("C# compiler diagnostic ") else {
            return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
        };
        if id.len() != 6
            || !id.starts_with("CS")
            || !id.as_bytes()[2..].iter().all(u8::is_ascii_digit)
        {
            return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
        }
        return Ok(());
    } else if code.starts_with("CSHARP_LIMIT_") {
        "C# profile limit exceeded"
    } else {
        match status {
            "source-error" => "C# source is invalid",
            "rejected" => "C# source is outside the frozen profile",
            "frontend-error" => "C# frontend failed closed",
            _ => return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape)),
        }
    };
    if message != expected_message {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn validate_success_issue_links(
    envelope: &Value,
    vir: &ValidatedSuccessorVir,
    manifest: &ValidatedSuccessorSourceManifest,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), SuccessorFrontendProtocolError> {
    for issue in envelope["rejected_features"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(envelope["diagnostics"].as_array().into_iter().flatten())
    {
        let object = issue
            .as_object()
            .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
        if let Some(function_id) = object.get("function_id").and_then(Value::as_str) {
            if vir.function(function_id).is_none() {
                return Err(protocol(
                    SuccessorFrontendProtocolCode::ProtocolArtifactMismatch,
                ));
            }
        }
        let Some(span) = object.get("span").and_then(Value::as_object) else {
            continue;
        };
        let path = string_field(span, "normalized_path")?;
        let start = integer_field(span, "start")?;
        let end = integer_field(span, "end")?;
        let mut matching = captured_inputs
            .iter()
            .filter(|input| input.normalized_path == path);
        let captured = matching
            .next()
            .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
        let manifest_count = manifest
            .manifest()
            .inputs()
            .iter()
            .filter(|input| input.normalized_path == path)
            .count();
        let start = usize::try_from(start)
            .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
        let end = usize::try_from(end)
            .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
        let text = std::str::from_utf8(captured.bytes)
            .map_err(|_| protocol(SuccessorFrontendProtocolCode::ProtocolArtifactMismatch))?;
        if matching.next().is_some()
            || manifest_count != 1
            || start >= end
            || end > captured.bytes.len()
            || !text.is_char_boundary(start)
            || !text.is_char_boundary(end)
        {
            return Err(protocol(
                SuccessorFrontendProtocolCode::ProtocolArtifactMismatch,
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Eq, PartialEq)]
struct IssueOrderKey<'a> {
    path: &'a str,
    start: i64,
    code: &'a str,
    message: &'a str,
    function: &'a str,
    end: i64,
}

impl Ord for IssueOrderKey<'_> {
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

impl PartialOrd for IssueOrderKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn validate_span(value: &Value) -> Result<(&str, i64, i64), SuccessorFrontendProtocolError> {
    let object = value
        .as_object()
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))?;
    exact_fields(object, &["normalized_path", "start", "end"])?;
    let path = string_field(object, "normalized_path")?;
    let start = integer_field(object, "start")?;
    let end = integer_field(object, "end")?;
    if mpk_vc::validate_manifest_normalized_path(path).is_err() || start < 0 || start >= end {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    Ok((path, start, end))
}

fn valid_issue_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 128
        && code.is_ascii()
        && code.as_bytes()[0].is_ascii_uppercase()
        && code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_public_paths(value: &Value) -> Result<(), SuccessorFrontendProtocolError> {
    match value {
        Value::String(text) => {
            if contains_absolute_path(text) {
                return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
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

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn exact_fields(
    object: &Map<String, Value>,
    fields: &[&str],
) -> Result<(), SuccessorFrontendProtocolError> {
    if object.len() != fields.len() || !fields.iter().all(|name| object.contains_key(*name)) {
        return Err(protocol(SuccessorFrontendProtocolCode::ProtocolShape));
    }
    Ok(())
}

fn field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Value, SuccessorFrontendProtocolError> {
    object
        .get(name)
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, SuccessorFrontendProtocolError> {
    field(object, name)?
        .as_str()
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a [Value], SuccessorFrontendProtocolError> {
    field(object, name)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))
}

fn integer_field(
    object: &Map<String, Value>,
    name: &str,
) -> Result<i64, SuccessorFrontendProtocolError> {
    field(object, name)?
        .as_i64()
        .ok_or_else(|| protocol(SuccessorFrontendProtocolCode::ProtocolShape))
}

const fn protocol(code: SuccessorFrontendProtocolCode) -> SuccessorFrontendProtocolError {
    SuccessorFrontendProtocolError { code }
}
