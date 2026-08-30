use crate::cli::{LowerRequest, NonSuccessStatus};
use crate::driver_protocol::{DriverOutput, DriverRequest, DriverStatus};
use crate::json::{self, JsonValue};
use crate::limits::RustLimitId;
use crate::sha256::{hex, Sha256};
use std::collections::BTreeMap;

const VIR_DOMAIN: &[u8] = b"MPK-VIR-1.0";
const INPUT_SET_DOMAIN: &[u8] = b"MPK-INPUT-SET-0.1";
const SOURCE_MAP_DOMAIN: &[u8] = b"MPK-SOURCE-MAP-1.0";
const SOURCE_MANIFEST_DOMAIN: &[u8] = b"MPK-SOURCE-MANIFEST-1.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmissionError {
    Integrity,
    IrLimit,
}

pub fn success_envelope(
    request: &LowerRequest,
    private_request: &DriverRequest,
    output: &DriverOutput,
    core_prelude: bool,
) -> Result<Vec<u8>, EmissionError> {
    if output.status() != DriverStatus::Lowered {
        return Err(EmissionError::Integrity);
    }
    let output_root = object(output.value())?;
    let lowering = object(required(output_root, "raw_lowering")?)?;
    let vir_source = required(lowering, "vir")?;
    json::canonical_size(vir_source, RustLimitId::VirJcs.maximum() as usize)
        .map_err(|_| EmissionError::IrLimit)?;
    let vir = vir_source.clone();
    let vir_root = object(&vir)?;
    let vir_hash = string(vir_root, "vir_hash")?.to_owned();
    let mut vir_preimage = vir.clone();
    object_mut(&mut vir_preimage)?.remove("vir_hash");
    let vir_preimage =
        json::canonical_bounded(&vir_preimage, RustLimitId::VirJcs.maximum() as usize)
            .map_err(|_| EmissionError::IrLimit)?;
    if domain_hash(VIR_DOMAIN, &vir_preimage) != vir_hash {
        return Err(EmissionError::Integrity);
    }
    json::canonical_size(&vir, RustLimitId::VirJcs.maximum() as usize)
        .map_err(|_| EmissionError::IrLimit)?;

    let source_map_source = required(output_root, "raw_source_map")?;
    json::canonical_size(
        source_map_source,
        RustLimitId::SourceMapJcs.maximum() as usize,
    )
    .map_err(|_| EmissionError::IrLimit)?;
    let mut source_map = source_map_source.clone();
    let source_map_root = object_mut(&mut source_map)?;
    source_map_root.insert("schema".to_owned(), string_value("mpk.source_map.v1"));
    let source_map_hash = domain_hash(
        SOURCE_MAP_DOMAIN,
        &json::canonical_bounded(&source_map, RustLimitId::SourceMapJcs.maximum() as usize)
            .map_err(|_| EmissionError::IrLimit)?,
    );
    object_mut(&mut source_map)?
        .insert("source_map_hash".to_owned(), string_value(&source_map_hash));
    json::canonical_size(&source_map, RustLimitId::SourceMapJcs.maximum() as usize)
        .map_err(|_| EmissionError::IrLimit)?;

    let request_root = object(private_request.value())?;
    let manifest_max = RustLimitId::SourceManifestJcs.maximum() as usize;
    let inputs_source = required(request_root, "inputs")?;
    let input_set_hash = domain_hash(
        INPUT_SET_DOMAIN,
        &json::canonical_bounded(inputs_source, manifest_max)
            .map_err(|_| EmissionError::IrLimit)?,
    );
    if string(request_root, "input_set_hash")? != input_set_hash {
        return Err(EmissionError::Integrity);
    }
    let inputs = inputs_source.clone();
    let units = public_units(vir_root, request)?;
    let _ = core_prelude;
    let mut manifest = JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string_value("mpk.source_manifest.v1")),
        (
            "semantic_context".to_owned(),
            required(request_root, "semantic_context")?.clone(),
        ),
        (
            "selection".to_owned(),
            required(request_root, "selection")?.clone(),
        ),
        (
            "limit_profile".to_owned(),
            string_value("mpk.vir.limits.v0"),
        ),
        (
            "release_registry".to_owned(),
            required(request_root, "release_registry")?.clone(),
        ),
        (
            "toolchain".to_owned(),
            required(request_root, "toolchain")?.clone(),
        ),
        (
            "frontend".to_owned(),
            required(request_root, "frontend")?.clone(),
        ),
        ("units".to_owned(), units),
        (
            "target".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("id".to_owned(), string_value(request.target.id())),
                (
                    "pointer_width".to_owned(),
                    number_value(i64::from(request.target.pointer_width())),
                ),
            ])),
        ),
        ("inputs".to_owned(), inputs),
        ("input_set_hash".to_owned(), string_value(&input_set_hash)),
        ("vir_hash".to_owned(), string_value(&vir_hash)),
        ("source_map_hash".to_owned(), string_value(&source_map_hash)),
    ]));
    let source_manifest_hash = domain_hash(
        SOURCE_MANIFEST_DOMAIN,
        &json::canonical_bounded(&manifest, manifest_max).map_err(|_| EmissionError::IrLimit)?,
    );
    object_mut(&mut manifest)?.insert(
        "source_manifest_hash".to_owned(),
        string_value(&source_manifest_hash),
    );
    json::canonical_size(&manifest, manifest_max).map_err(|_| EmissionError::IrLimit)?;

    let envelope = JsonValue::Object(BTreeMap::from([
        ("diagnostics".to_owned(), JsonValue::Array(Vec::new())),
        (
            "ir".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("schema".to_owned(), string_value("mpk.vir.v1")),
                ("sha256".to_owned(), string_value(&vir_hash)),
                ("value".to_owned(), vir),
            ])),
        ),
        ("phase".to_owned(), string_value("emission")),
        ("rejected_features".to_owned(), JsonValue::Array(Vec::new())),
        ("schema".to_owned(), string_value("mpk.frontend.cli.v1")),
        (
            "selection".to_owned(),
            required(request_root, "selection")?.clone(),
        ),
        (
            "semantic_context".to_owned(),
            required(request_root, "semantic_context")?.clone(),
        ),
        ("source_manifest".to_owned(), manifest),
        ("source_map".to_owned(), source_map),
        ("status".to_owned(), string_value("ir-lowered")),
    ]));
    transport(&envelope)
}

pub fn driver_non_success_envelope(
    request: &LowerRequest,
    output: &DriverOutput,
) -> Result<Vec<u8>, EmissionError> {
    if output.status() == DriverStatus::Lowered {
        return Err(EmissionError::Integrity);
    }
    let root = object(output.value())?;
    let mut issues = required(root, "diagnostics")?
        .as_array()
        .ok_or(EmissionError::Integrity)?
        .to_vec();
    let has_marker = issues
        .last()
        .and_then(JsonValue::as_object)
        .and_then(|issue| issue.get("code"))
        .and_then(JsonValue::as_str)
        == Some("RUST_LIMIT_DIAGNOSTICS_TRUNCATED");
    let marker = has_marker.then(|| issues.pop().expect("observed final diagnostic"));
    let (rejected, diagnostics) = if output.status() == DriverStatus::Rejected {
        (
            JsonValue::Array(issues),
            JsonValue::Array(marker.into_iter().collect()),
        )
    } else {
        if let Some(marker) = marker {
            issues.push(marker);
        }
        (JsonValue::Array(Vec::new()), JsonValue::Array(issues))
    };
    let status = match output.status() {
        DriverStatus::Rejected => "rejected",
        DriverStatus::SourceError => "source-error",
        DriverStatus::FrontendError => "frontend-error",
        DriverStatus::Lowered => return Err(EmissionError::Integrity),
    };
    public_non_success(request, status, output.phase(), rejected, diagnostics)
}

pub fn local_non_success_envelope(
    request: &LowerRequest,
    status: NonSuccessStatus,
    phase: &str,
    code: &str,
    message: &str,
) -> Result<Vec<u8>, EmissionError> {
    if !stable_code(code)
        || message.is_empty()
        || message.len() > 4_096
        || message.chars().any(char::is_control)
    {
        return Err(EmissionError::Integrity);
    }
    let mut issue = BTreeMap::from([
        ("code".to_owned(), string_value(code)),
        ("message".to_owned(), string_value(message)),
    ]);
    if matches!(phase, "subset" | "lowering" | "emission") {
        issue.insert(
            "function_id".to_owned(),
            string_value(&request.selection.function),
        );
    }
    let issue = JsonValue::Array(vec![JsonValue::Object(issue)]);
    let (status, rejected, diagnostics) = match status {
        NonSuccessStatus::Rejected => ("rejected", issue, JsonValue::Array(Vec::new())),
        NonSuccessStatus::SourceError => ("source-error", JsonValue::Array(Vec::new()), issue),
        NonSuccessStatus::FrontendError => ("frontend-error", JsonValue::Array(Vec::new()), issue),
    };
    public_non_success(request, status, phase, rejected, diagnostics)
}

fn public_non_success(
    request: &LowerRequest,
    status: &str,
    phase: &str,
    rejected_features: JsonValue,
    diagnostics: JsonValue,
) -> Result<Vec<u8>, EmissionError> {
    transport(&JsonValue::Object(BTreeMap::from([
        ("diagnostics".to_owned(), diagnostics),
        ("phase".to_owned(), string_value(phase)),
        ("rejected_features".to_owned(), rejected_features),
        ("schema".to_owned(), string_value("mpk.frontend.cli.v1")),
        ("selection".to_owned(), selection(request)),
        ("semantic_context".to_owned(), semantic_context(request)),
        ("status".to_owned(), string_value(status)),
    ])))
}

fn selection(request: &LowerRequest) -> JsonValue {
    crate::successor::selection_envelope(
        &request.selection.package,
        &request.selection.crate_name,
        &request.selection.function,
    )
}

fn semantic_context(request: &LowerRequest) -> JsonValue {
    crate::successor::semantic_context(request.target.id(), request.target.pointer_width())
}

fn public_units(
    vir: &BTreeMap<String, JsonValue>,
    request: &LowerRequest,
) -> Result<JsonValue, EmissionError> {
    let units = required(vir, "units")?
        .as_array()
        .ok_or(EmissionError::Integrity)?;
    if units.len() != 1 {
        return Err(EmissionError::Integrity);
    }
    let selected_unit = object(&units[0])?;
    if string(selected_unit, "id")? != request.selection.crate_name
        || string(selected_unit, "name")? != request.selection.package
        || required(selected_unit, "functions")?
            .as_array()
            .ok_or(EmissionError::Integrity)?
            .iter()
            .filter(|function| {
                function
                    .as_object()
                    .and_then(|function| function.get("id"))
                    .and_then(JsonValue::as_str)
                    == Some(request.selection.function.as_str())
            })
            .count()
            != 1
    {
        return Err(EmissionError::Integrity);
    }
    Ok(JsonValue::Array(
        units
            .iter()
            .map(|unit| {
                let unit = object(unit)?;
                Ok(JsonValue::Object(BTreeMap::from([
                    ("identity".to_owned(), string_value(string(unit, "id")?)),
                    ("kind".to_owned(), string_value("lib")),
                    ("name".to_owned(), string_value(string(unit, "name")?)),
                ])))
            })
            .collect::<Result<Vec<_>, EmissionError>>()?,
    ))
}

fn transport(value: &JsonValue) -> Result<Vec<u8>, EmissionError> {
    // The outer frontend runner owns this stream boundary and classifies an
    // excess as FRONTEND_PROTOCOL_LIMIT. The component limits above keep a
    // successful Rust envelope below that ceiling without reclassifying it in
    // the producer.
    let mut bytes = json::canonical(value).map_err(|_| EmissionError::Integrity)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn domain_hash(domain: &[u8], payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(payload);
    hex(&hasher.finish())
}

fn stable_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 128
        && (code.starts_with("RUST_") || code.starts_with("FRONTEND_"))
        && code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, EmissionError> {
    value.as_object().ok_or(EmissionError::Integrity)
}

fn object_mut(value: &mut JsonValue) -> Result<&mut BTreeMap<String, JsonValue>, EmissionError> {
    value.as_object_mut().ok_or(EmissionError::Integrity)
}

fn required<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a JsonValue, EmissionError> {
    object.get(name).ok_or(EmissionError::Integrity)
}

fn string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a str, EmissionError> {
    required(object, name)?
        .as_str()
        .ok_or(EmissionError::Integrity)
}

fn string_value(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}

fn number_value(value: i64) -> JsonValue {
    JsonValue::Number(value.to_string())
}
