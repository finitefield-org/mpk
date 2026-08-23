use crate::cli::{LowerRequest, NonSuccessStatus};
use crate::driver_protocol::{DriverOutput, DriverRequest, DriverStatus, OUTPUT_TRANSPORT_MAX};
use crate::json::{self, JsonValue};
use crate::session;
use crate::sha256::{hex, Sha256};
use std::collections::BTreeMap;

const VIR_DOMAIN: &[u8] = b"MPK-VIR-0.1";
const INPUT_SET_DOMAIN: &[u8] = b"MPK-INPUT-SET-0.1";
const SOURCE_MAP_DOMAIN: &[u8] = b"MPK-SOURCE-MAP-0.1";
const SOURCE_MANIFEST_DOMAIN: &[u8] = b"MPK-SOURCE-MANIFEST-0.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmissionError;

pub fn success_envelope(
    request: &LowerRequest,
    private_request: &DriverRequest,
    output: &DriverOutput,
    core_prelude: bool,
) -> Result<Vec<u8>, EmissionError> {
    if output.status() != DriverStatus::Lowered {
        return Err(EmissionError);
    }
    let output_root = object(output.value())?;
    let lowering = object(required(output_root, "raw_lowering")?)?;
    let vir = required(lowering, "vir")?.clone();
    let vir_root = object(&vir)?;
    let vir_hash = string(vir_root, "vir_hash")?.to_owned();
    let mut vir_preimage = vir.clone();
    object_mut(&mut vir_preimage)?.remove("vir_hash");
    if domain_hash(VIR_DOMAIN, &canonical(&vir_preimage)?) != vir_hash {
        return Err(EmissionError);
    }

    let mut source_map = required(output_root, "raw_source_map")?.clone();
    let source_map_root = object_mut(&mut source_map)?;
    source_map_root.insert("schema".to_owned(), string_value("mpk.source_map.v0"));
    let source_map_hash = domain_hash(SOURCE_MAP_DOMAIN, &canonical(&source_map)?);
    object_mut(&mut source_map)?
        .insert("source_map_hash".to_owned(), string_value(&source_map_hash));

    let request_root = object(private_request.value())?;
    let inputs = required(request_root, "inputs")?.clone();
    let input_set_hash = domain_hash(INPUT_SET_DOMAIN, &canonical(&inputs)?);
    if string(request_root, "input_set_hash")? != input_set_hash {
        return Err(EmissionError);
    }
    let units = public_units(vir_root, request)?;
    let language_configuration = rust_language_configuration(request, core_prelude)?;
    let mut manifest = JsonValue::Object(BTreeMap::from([
        ("schema".to_owned(), string_value("mpk.source_manifest.v0")),
        ("source_language".to_owned(), string_value("rust")),
        (
            "semantic_profile".to_owned(),
            required(request_root, "semantic_profile")?.clone(),
        ),
        (
            "semantic_parameters".to_owned(),
            required(request_root, "semantic_parameters")?.clone(),
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
                ("language_configuration".to_owned(), language_configuration),
            ])),
        ),
        ("inputs".to_owned(), inputs),
        ("input_set_hash".to_owned(), string_value(&input_set_hash)),
        ("vir_hash".to_owned(), string_value(&vir_hash)),
        ("source_map_hash".to_owned(), string_value(&source_map_hash)),
    ]));
    let source_manifest_hash = domain_hash(SOURCE_MANIFEST_DOMAIN, &canonical(&manifest)?);
    object_mut(&mut manifest)?.insert(
        "source_manifest_hash".to_owned(),
        string_value(&source_manifest_hash),
    );

    let envelope = JsonValue::Object(BTreeMap::from([
        ("diagnostics".to_owned(), JsonValue::Array(Vec::new())),
        (
            "ir".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("schema".to_owned(), string_value("mpk.vir.v0")),
                ("sha256".to_owned(), string_value(&vir_hash)),
                ("value".to_owned(), vir),
            ])),
        ),
        ("phase".to_owned(), string_value("emission")),
        ("rejected_features".to_owned(), JsonValue::Array(Vec::new())),
        ("schema".to_owned(), string_value("mpk.frontend.cli.v0")),
        (
            "selection".to_owned(),
            required(request_root, "selection")?.clone(),
        ),
        (
            "semantic_parameters".to_owned(),
            required(request_root, "semantic_parameters")?.clone(),
        ),
        (
            "semantic_profile".to_owned(),
            required(request_root, "semantic_profile")?.clone(),
        ),
        ("source_language".to_owned(), string_value("rust")),
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
        return Err(EmissionError);
    }
    let root = object(output.value())?;
    let mut issues = required(root, "diagnostics")?
        .as_array()
        .ok_or(EmissionError)?
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
        DriverStatus::Lowered => return Err(EmissionError),
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
        return Err(EmissionError);
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
        ("schema".to_owned(), string_value("mpk.frontend.cli.v0")),
        ("selection".to_owned(), selection(request)),
        (
            "semantic_parameters".to_owned(),
            semantic_parameters(request),
        ),
        (
            "semantic_profile".to_owned(),
            string_value("mpk.rust.checked.v0"),
        ),
        ("source_language".to_owned(), string_value("rust")),
        ("status".to_owned(), string_value(status)),
    ])))
}

fn selection(request: &LowerRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "crate".to_owned(),
            string_value(&request.selection.crate_name),
        ),
        (
            "function".to_owned(),
            string_value(&request.selection.function),
        ),
        ("kind".to_owned(), string_value("lib")),
        (
            "package".to_owned(),
            string_value(&request.selection.package),
        ),
    ]))
}

fn semantic_parameters(request: &LowerRequest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("overflow_mode".to_owned(), string_value("checked")),
        ("panic_mode".to_owned(), string_value("abort")),
        (
            "pointer_width".to_owned(),
            number_value(i64::from(request.target.pointer_width())),
        ),
        ("target_id".to_owned(), string_value(request.target.id())),
    ]))
}

fn rust_language_configuration(
    request: &LowerRequest,
    core_prelude: bool,
) -> Result<JsonValue, EmissionError> {
    let cfg = session::target_cfg(request.target.id()).ok_or(EmissionError)?;
    Ok(JsonValue::Object(BTreeMap::from([
        ("kind".to_owned(), string_value("rust")),
        ("edition".to_owned(), string_value("2021")),
        ("crate_type".to_owned(), string_value("lib")),
        ("enabled_features".to_owned(), JsonValue::Array(Vec::new())),
        (
            "prelude".to_owned(),
            string_value(if core_prelude { "core" } else { "std" }),
        ),
        ("locked".to_owned(), JsonValue::Bool(true)),
        ("offline".to_owned(), JsonValue::Bool(true)),
        ("default_features".to_owned(), JsonValue::Bool(false)),
        ("overflow_checks".to_owned(), JsonValue::Bool(true)),
        ("panic".to_owned(), string_value("abort")),
        ("debug_assertions".to_owned(), JsonValue::Bool(false)),
        ("rustc_opt_level".to_owned(), number_value(0)),
        ("mir_opt_level".to_owned(), number_value(0)),
        ("jobs".to_owned(), number_value(1)),
        ("message_format".to_owned(), string_value("json")),
        (
            "target_allowlist_id".to_owned(),
            string_value("mpk.rust.targets.v0"),
        ),
        (
            "environment_profile_id".to_owned(),
            string_value("mpk.rust.frontend_environment.v0"),
        ),
        (
            "argument_profile_id".to_owned(),
            string_value("mpk.rust.frontend_arguments.v0"),
        ),
        (
            "cfg".to_owned(),
            JsonValue::Array(cfg.iter().map(|value| string_value(value)).collect()),
        ),
    ])))
}

fn public_units(
    vir: &BTreeMap<String, JsonValue>,
    request: &LowerRequest,
) -> Result<JsonValue, EmissionError> {
    let units = required(vir, "units")?.as_array().ok_or(EmissionError)?;
    if units.len() != 1 {
        return Err(EmissionError);
    }
    let selected_unit = object(&units[0])?;
    if string(selected_unit, "id")? != request.selection.crate_name
        || string(selected_unit, "name")? != request.selection.package
        || required(selected_unit, "functions")?
            .as_array()
            .ok_or(EmissionError)?
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
        return Err(EmissionError);
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
    let mut bytes = canonical(value)?;
    if bytes
        .len()
        .checked_add(1)
        .is_none_or(|size| size > OUTPUT_TRANSPORT_MAX)
    {
        return Err(EmissionError);
    }
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

fn canonical(value: &JsonValue) -> Result<Vec<u8>, EmissionError> {
    json::canonical(value).map_err(|_| EmissionError)
}

fn object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, EmissionError> {
    value.as_object().ok_or(EmissionError)
}

fn object_mut(value: &mut JsonValue) -> Result<&mut BTreeMap<String, JsonValue>, EmissionError> {
    value.as_object_mut().ok_or(EmissionError)
}

fn required<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a JsonValue, EmissionError> {
    object.get(name).ok_or(EmissionError)
}

fn string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<&'a str, EmissionError> {
    required(object, name)?.as_str().ok_or(EmissionError)
}

fn string_value(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}

fn number_value(value: i64) -> JsonValue {
    JsonValue::Number(value.to_string())
}
