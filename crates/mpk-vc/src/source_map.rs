//! Strict source-map v0 model, importer, and VIR/input linkage validation.

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, StrictJsonError, StrictJsonLimits, StrictJsonValue,
};
use crate::hash::{hash_canonical_json, sha256_raw_file_bytes, HashDomain};
use crate::semantic_profile::SourceLanguage;
use crate::vir::{LowercaseSha256, VirInstruction, VirModule, VIR_SCHEMA_VERSION};
use crate::vir_canonical::vir_hash;
use crate::vir_validate::validate_vir;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const SOURCE_MAP_SCHEMA_VERSION: &str = "mpk.source_map.v0";
pub const SOURCE_MAP_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MAP-0.1");
pub const SOURCE_MAP_CANONICAL_BYTES_MAX: u64 = 33_554_432;
pub const SOURCE_MAP_ENTRIES_MAX: u64 = 323_728;
pub const SOURCE_MAP_JSON_NESTING_MAX: u64 = 256;
pub const SOURCE_MAP_STRING_BYTES_MAX: u64 = 1_048_576;
pub const NORMALIZED_PATH_BYTES_MAX: usize = 1_024;

const SOURCE_MAP_JSON_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    268_435_456,
    SOURCE_MAP_CANONICAL_BYTES_MAX,
    SOURCE_MAP_JSON_NESTING_MAX,
    SOURCE_MAP_STRING_BYTES_MAX,
);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Source,
    Contract,
    BuildManifest,
    Lockfile,
}

/// One immutable input captured by a frontend before public artifact emission.
#[derive(Clone, Copy, Debug)]
pub struct CapturedInput<'a> {
    pub kind: InputKind,
    pub normalized_path: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMap {
    pub schema: String,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub entries: Vec<SourceMapEntry>,
    pub source_map_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMapEntry {
    pub reference: SourceReference,
    pub origin: SourceOrigin,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceReference {
    Function {
        unit_id: String,
        function_id: String,
    },
    Instruction {
        unit_id: String,
        function_id: String,
        block: String,
        instruction: String,
    },
    Terminator {
        unit_id: String,
        function_id: String,
        block: String,
    },
}

impl SourceReference {
    pub fn unit_id(&self) -> &str {
        match self {
            Self::Function { unit_id, .. }
            | Self::Instruction { unit_id, .. }
            | Self::Terminator { unit_id, .. } => unit_id,
        }
    }

    pub fn function_id(&self) -> &str {
        match self {
            Self::Function { function_id, .. }
            | Self::Instruction { function_id, .. }
            | Self::Terminator { function_id, .. } => function_id,
        }
    }

    fn order_key(&self) -> Result<(&str, &str, u8, i64, i64), SourceMapError> {
        match self {
            Self::Function {
                unit_id,
                function_id,
            } => Ok((unit_id, function_id, 0, -1, -1)),
            Self::Instruction {
                unit_id,
                function_id,
                block,
                instruction,
            } => Ok((
                unit_id,
                function_id,
                1,
                i64::from(parse_dense_id(block, "bb")?),
                i64::from(parse_dense_id(instruction, "t")?),
            )),
            Self::Terminator {
                unit_id,
                function_id,
                block,
            } => Ok((
                unit_id,
                function_id,
                2,
                i64::from(parse_dense_id(block, "bb")?),
                -1,
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceOrigin {
    Source {
        input_kind: SourceInputKind,
        normalized_path: String,
        start: i64,
        end: i64,
    },
    Synthetic {
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceInputKind {
    Source,
}

/// Exact profile-owned permission for one synthetic VIR node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntheticPermission {
    pub reference: SourceReference,
    pub reason: String,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceMapValidationContext<'a> {
    pub vir: &'a VirModule,
    pub captured_inputs: &'a [CapturedInput<'a>],
    pub synthetic_permissions: &'a [SyntheticPermission],
}

/// A source map that passed every v0 phase. Fields cannot be constructed by callers.
#[derive(Clone, Debug)]
pub struct ValidatedSourceMap {
    map: SourceMap,
    canonical_bytes: Vec<u8>,
    hash: LowercaseSha256,
    captured_sources: Vec<CapturedSourceIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapturedSourceIdentity {
    pub normalized_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

impl ValidatedSourceMap {
    pub fn map(&self) -> &SourceMap {
        &self.map
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn hash(&self) -> &LowercaseSha256 {
        &self.hash
    }

    pub(crate) fn captured_source_identity(
        &self,
        normalized_path: &str,
    ) -> Option<&CapturedSourceIdentity> {
        let mut matches = self
            .captured_sources
            .iter()
            .filter(|source| source.normalized_path == normalized_path);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMapValidationPhase {
    Transport,
    Shape,
    Scalar,
    Order,
    Linkage,
    Coverage,
    Utf8,
    CanonicalSize,
    Hash,
}

impl SourceMapValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Order => "order",
            Self::Linkage => "linkage",
            Self::Coverage => "coverage",
            Self::Utf8 => "utf8",
            Self::CanonicalSize => "canonical_size",
            Self::Hash => "hash",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMapErrorCode {
    JsonDuplicateKey,
    JsonInvalid,
    Schema,
    Shape,
    Path,
    Range,
    Order,
    Reference,
    InputKind,
    Total,
    Synthetic,
    Utf8Boundary,
    IrIdentity,
    LimitEntries,
    LimitCanonicalBytes,
    Hash,
}

impl SourceMapErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JsonDuplicateKey => "SOURCE_MAP_JSON_DUPLICATE_KEY",
            Self::JsonInvalid => "SOURCE_MAP_JSON_INVALID",
            Self::Schema => "SOURCE_MAP_SCHEMA",
            Self::Shape => "SOURCE_MAP_SHAPE",
            Self::Path => "SOURCE_MAP_PATH",
            Self::Range => "SOURCE_MAP_RANGE",
            Self::Order => "SOURCE_MAP_ORDER",
            Self::Reference => "SOURCE_MAP_REFERENCE",
            Self::InputKind => "SOURCE_MAP_INPUT_KIND",
            Self::Total => "SOURCE_MAP_TOTAL",
            Self::Synthetic => "SOURCE_MAP_SYNTHETIC",
            Self::Utf8Boundary => "SOURCE_MAP_UTF8_BOUNDARY",
            Self::IrIdentity => "SOURCE_MAP_IR_IDENTITY",
            Self::LimitEntries => "SOURCE_MAP_LIMIT_ENTRIES",
            Self::LimitCanonicalBytes => "SOURCE_MAP_LIMIT_CANONICAL_BYTES",
            Self::Hash => "SOURCE_MAP_HASH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMapError {
    pub phase: SourceMapValidationPhase,
    pub code: SourceMapErrorCode,
    pub detail: String,
}

impl SourceMapError {
    fn new(
        phase: SourceMapValidationPhase,
        code: SourceMapErrorCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SourceMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code.as_str(),
            self.phase.as_str(),
            self.detail
        )
    }
}

impl Error for SourceMapError {}

pub fn import_source_map_json(
    input: &[u8],
    context: SourceMapValidationContext<'_>,
) -> Result<ValidatedSourceMap, SourceMapError> {
    let strict = parse_strict_json(input, SOURCE_MAP_JSON_LIMITS).map_err(map_transport_error)?;
    validate_source_map_entry_count(entry_count(&strict)?)?;

    let canonical = canonical_json_bytes(&strict).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Transport,
            SourceMapErrorCode::JsonInvalid,
            error.to_string(),
        )
    })?;
    let map: SourceMap = serde_json::from_slice(&canonical).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Shape,
            SourceMapErrorCode::Shape,
            error.to_string(),
        )
    })?;

    validate_shape(&map)?;
    validate_scalars(&map, context.vir.source_language)?;
    validate_order(&map)?;
    validate_linkage(&map, context)?;
    validate_coverage(&map, context)?;
    validate_utf8_boundaries(&map, context.captured_inputs)?;
    validate_source_map_canonical_size(canonical.len() as u64)?;
    let hash = recompute_source_map_hash_from_value(&strict)?;
    if map.source_map_hash != hash.as_str() {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Hash,
            SourceMapErrorCode::Hash,
            "source_map_hash does not match the recomputed map hash",
        ));
    }

    Ok(ValidatedSourceMap {
        map,
        canonical_bytes: canonical,
        hash,
        captured_sources: context
            .captured_inputs
            .iter()
            .filter(|input| input.kind == InputKind::Source)
            .map(|input| CapturedSourceIdentity {
                normalized_path: input.normalized_path.to_owned(),
                size_bytes: u64::try_from(input.bytes.len()).unwrap_or(u64::MAX),
                sha256: sha256_raw_file_bytes(input.bytes).to_hex(),
            })
            .collect(),
    })
}

pub fn source_map_hash(map: &SourceMap) -> Result<LowercaseSha256, SourceMapError> {
    let bytes = serde_json::to_vec(map).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Hash,
            SourceMapErrorCode::Hash,
            error.to_string(),
        )
    })?;
    let strict = parse_strict_json(&bytes, SOURCE_MAP_JSON_LIMITS).map_err(map_transport_error)?;
    recompute_source_map_hash_from_value(&strict)
}

pub fn validate_source_map_entry_count(count: u64) -> Result<(), SourceMapError> {
    if count > SOURCE_MAP_ENTRIES_MAX {
        Err(SourceMapError::new(
            SourceMapValidationPhase::Transport,
            SourceMapErrorCode::LimitEntries,
            "source-map entry limit exceeded",
        ))
    } else {
        Ok(())
    }
}

pub fn validate_source_map_canonical_size(count: u64) -> Result<(), SourceMapError> {
    if count > SOURCE_MAP_CANONICAL_BYTES_MAX {
        Err(SourceMapError::new(
            SourceMapValidationPhase::CanonicalSize,
            SourceMapErrorCode::LimitCanonicalBytes,
            "source-map canonical byte limit exceeded",
        ))
    } else {
        Ok(())
    }
}

pub fn validate_normalized_path(path: &str) -> Result<(), SourceMapError> {
    if !is_portable_normalized_path(path) {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Path,
            format!("nonportable normalized path {path:?}"),
        ));
    }
    Ok(())
}

pub(crate) fn is_portable_normalized_path(path: &str) -> bool {
    if path.is_empty()
        || path.len() > NORMALIZED_PATH_BYTES_MAX
        || !path.is_ascii()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains(['\\', ':', '\0'])
        || path.to_ascii_lowercase().starts_with("file:")
        || path.starts_with("/mpk/")
    {
        return false;
    }

    path.split('/').all(|component| {
        !component.is_empty()
            && component.len() <= 255
            && component != "."
            && component != ".."
            && !component.ends_with('.')
            && component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            && !is_windows_device_name(component)
    })
}

fn is_windows_device_name(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

fn map_transport_error(error: StrictJsonError) -> SourceMapError {
    let code = if matches!(error, StrictJsonError::DuplicateObjectName { .. }) {
        SourceMapErrorCode::JsonDuplicateKey
    } else {
        SourceMapErrorCode::JsonInvalid
    };
    SourceMapError::new(SourceMapValidationPhase::Transport, code, error.to_string())
}

fn entry_count(value: &StrictJsonValue) -> Result<u64, SourceMapError> {
    let Some(entries) = value.get("entries") else {
        return Ok(0);
    };
    let Some(entries) = entries.as_array() else {
        return Ok(0);
    };
    Ok(u64::try_from(entries.len()).unwrap_or(u64::MAX))
}

fn validate_shape(map: &SourceMap) -> Result<(), SourceMapError> {
    if map.schema != SOURCE_MAP_SCHEMA_VERSION {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Shape,
            SourceMapErrorCode::Schema,
            "unsupported source-map schema",
        ));
    }
    Ok(())
}

fn validate_scalars(
    map: &SourceMap,
    source_language: SourceLanguage,
) -> Result<(), SourceMapError> {
    validate_hash(&map.source_ir_hash)?;
    validate_hash(&map.source_map_hash)?;
    for entry in &map.entries {
        validate_reference_identifiers(&entry.reference, source_language)?;
        match &entry.reference {
            SourceReference::Instruction {
                block, instruction, ..
            } => {
                parse_dense_id(block, "bb")?;
                parse_dense_id(instruction, "t")?;
            }
            SourceReference::Terminator { block, .. } => {
                parse_dense_id(block, "bb")?;
            }
            SourceReference::Function { .. } => {}
        }
        match &entry.origin {
            SourceOrigin::Source {
                normalized_path,
                start,
                end,
                ..
            } => {
                validate_normalized_path(normalized_path)?;
                if *start < 0 || *end < 0 || start >= end {
                    return Err(SourceMapError::new(
                        SourceMapValidationPhase::Scalar,
                        SourceMapErrorCode::Range,
                        "source range must be nonempty and nonnegative",
                    ));
                }
            }
            SourceOrigin::Synthetic { reason } => {
                if !valid_profile_id(reason) {
                    return Err(SourceMapError::new(
                        SourceMapValidationPhase::Scalar,
                        SourceMapErrorCode::Synthetic,
                        "invalid synthetic reason",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_reference_identifiers(
    reference: &SourceReference,
    source_language: SourceLanguage,
) -> Result<(), SourceMapError> {
    let valid = match source_language {
        SourceLanguage::Go => {
            valid_go_unit_id(reference.unit_id())
                && valid_go_function_id(reference.unit_id(), reference.function_id())
        }
        SourceLanguage::Rust => {
            valid_ascii_ident(reference.unit_id())
                && valid_rust_function_id(reference.unit_id(), reference.function_id())
        }
    };
    if valid {
        Ok(())
    } else {
        Err(SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Reference,
            "source-map reference contains a noncanonical VIR identifier",
        ))
    }
}

fn valid_ascii_ident(value: &str) -> bool {
    !value.is_empty()
        && value != "_"
        && value.len() <= 255
        && value.is_ascii()
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_go_unit_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1_024
        && value.is_ascii()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains(['\\', ':'])
        && value.split('/').all(|segment| {
            !matches!(segment, "" | "." | "..")
                && segment
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && segment.bytes().skip(1).all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-')
                })
        })
}

fn valid_go_function_id(unit_id: &str, function_id: &str) -> bool {
    if function_id.len() > 1_024 {
        return false;
    }
    let Some(suffix) = function_id
        .strip_prefix(unit_id)
        .and_then(|suffix| suffix.strip_prefix('.'))
    else {
        return false;
    };
    let segments: Vec<_> = suffix.split('.').collect();
    matches!(segments.len(), 1 | 2) && segments.into_iter().all(valid_ascii_ident)
}

fn valid_rust_function_id(unit_id: &str, function_id: &str) -> bool {
    if function_id.len() > 1_024 {
        return false;
    }
    let Some(suffix) = function_id
        .strip_prefix(unit_id)
        .and_then(|suffix| suffix.strip_prefix("::"))
    else {
        return false;
    };
    !suffix.is_empty() && suffix.split("::").all(valid_ascii_ident)
}

fn validate_hash(value: &str) -> Result<(), SourceMapError> {
    LowercaseSha256::new(value.to_owned()).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Shape,
            error.to_string(),
        )
    })?;
    Ok(())
}

fn parse_dense_id(value: &str, prefix: &str) -> Result<u32, SourceMapError> {
    let Some(decimal) = value.strip_prefix(prefix) else {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Reference,
            format!("invalid dense ID {value:?}"),
        ));
    };
    if decimal.is_empty()
        || !decimal.bytes().all(|byte| byte.is_ascii_digit())
        || (decimal.len() > 1 && decimal.starts_with('0'))
    {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Reference,
            format!("noncanonical dense ID {value:?}"),
        ));
    }
    decimal.parse().map_err(|_| {
        SourceMapError::new(
            SourceMapValidationPhase::Scalar,
            SourceMapErrorCode::Reference,
            format!("dense ID is out of range {value:?}"),
        )
    })
}

fn valid_profile_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let mut need_alphanumeric = true;
    for byte in value.bytes() {
        if need_alphanumeric {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
                return false;
            }
            need_alphanumeric = false;
        } else if matches!(byte, b'.' | b'_' | b'-') {
            need_alphanumeric = true;
        } else if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
            return false;
        }
    }
    !need_alphanumeric
}

fn validate_order(map: &SourceMap) -> Result<(), SourceMapError> {
    for pair in map.entries.windows(2) {
        let left = pair[0].reference.order_key()?;
        let right = pair[1].reference.order_key()?;
        if compare_reference_keys(left, right) != Ordering::Less {
            return Err(SourceMapError::new(
                SourceMapValidationPhase::Order,
                SourceMapErrorCode::Order,
                "source-map references are duplicated or not canonical",
            ));
        }
    }
    Ok(())
}

fn compare_reference_keys(
    left: (&str, &str, u8, i64, i64),
    right: (&str, &str, u8, i64, i64),
) -> Ordering {
    left.0
        .as_bytes()
        .cmp(right.0.as_bytes())
        .then_with(|| left.1.as_bytes().cmp(right.1.as_bytes()))
        .then_with(|| left.2.cmp(&right.2))
        .then_with(|| left.3.cmp(&right.3))
        .then_with(|| left.4.cmp(&right.4))
}

fn validate_linkage(
    map: &SourceMap,
    context: SourceMapValidationContext<'_>,
) -> Result<(), SourceMapError> {
    validate_vir(context.vir).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Linkage,
            SourceMapErrorCode::IrIdentity,
            format!("source-map context contains invalid VIR: {error}"),
        )
    })?;
    let expected_vir_hash = vir_hash(context.vir).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Linkage,
            SourceMapErrorCode::IrIdentity,
            error.to_string(),
        )
    })?;
    if map.source_ir_schema != VIR_SCHEMA_VERSION
        || map.source_ir_schema != context.vir.schema
        || map.source_ir_hash != expected_vir_hash.as_str()
    {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Linkage,
            SourceMapErrorCode::IrIdentity,
            "source-map VIR identity does not match the validated VIR",
        ));
    }

    let expected = expected_references(context.vir);
    for entry in &map.entries {
        if !expected.contains(&entry.reference) {
            return Err(SourceMapError::new(
                SourceMapValidationPhase::Linkage,
                SourceMapErrorCode::Reference,
                "source-map reference does not resolve in VIR",
            ));
        }
        if let SourceOrigin::Source {
            normalized_path,
            start,
            end,
            ..
        } = &entry.origin
        {
            let Some(input) = unique_input(context.captured_inputs, normalized_path) else {
                return Err(SourceMapError::new(
                    SourceMapValidationPhase::Linkage,
                    SourceMapErrorCode::InputKind,
                    "source origin does not resolve one captured source input",
                ));
            };
            if input.kind != InputKind::Source {
                return Err(SourceMapError::new(
                    SourceMapValidationPhase::Linkage,
                    SourceMapErrorCode::InputKind,
                    "source origin resolves a non-source input",
                ));
            }
            let size = i64::try_from(input.bytes.len()).unwrap_or(i64::MAX);
            if *start >= size || *end > size {
                return Err(SourceMapError::new(
                    SourceMapValidationPhase::Linkage,
                    SourceMapErrorCode::Range,
                    "source range exceeds captured input bytes",
                ));
            }
        }
    }
    Ok(())
}

fn unique_input<'a>(inputs: &'a [CapturedInput<'a>], path: &str) -> Option<&'a CapturedInput<'a>> {
    let mut matches = inputs.iter().filter(|input| input.normalized_path == path);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn validate_coverage(
    map: &SourceMap,
    context: SourceMapValidationContext<'_>,
) -> Result<(), SourceMapError> {
    let expected = expected_references(context.vir);
    let actual: BTreeSet<_> = map
        .entries
        .iter()
        .map(|entry| entry.reference.clone())
        .collect();
    if actual != expected {
        return Err(SourceMapError::new(
            SourceMapValidationPhase::Coverage,
            SourceMapErrorCode::Total,
            "source map is not a unique total mapping of VIR nodes",
        ));
    }

    for entry in &map.entries {
        match (&entry.reference, &entry.origin) {
            (SourceReference::Function { .. }, SourceOrigin::Synthetic { .. }) => {
                return Err(SourceMapError::new(
                    SourceMapValidationPhase::Coverage,
                    SourceMapErrorCode::Synthetic,
                    "function declarations cannot have synthetic origins",
                ));
            }
            (reference, SourceOrigin::Synthetic { reason }) => {
                let permitted = context.synthetic_permissions.iter().any(|permission| {
                    permission.reference == *reference && permission.reason == *reason
                });
                if !permitted {
                    return Err(SourceMapError::new(
                        SourceMapValidationPhase::Coverage,
                        SourceMapErrorCode::Synthetic,
                        "synthetic origin is not allowed for this exact VIR node",
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn expected_references(vir: &VirModule) -> BTreeSet<SourceReference> {
    let mut references = BTreeSet::new();
    for unit in &vir.units {
        for function in &unit.functions {
            references.insert(SourceReference::Function {
                unit_id: unit.id.clone(),
                function_id: function.id.clone(),
            });
            for block in &function.blocks {
                for instruction in &block.instructions {
                    references.insert(SourceReference::Instruction {
                        unit_id: unit.id.clone(),
                        function_id: function.id.clone(),
                        block: block.label.clone(),
                        instruction: instruction_id(instruction).to_owned(),
                    });
                }
                references.insert(SourceReference::Terminator {
                    unit_id: unit.id.clone(),
                    function_id: function.id.clone(),
                    block: block.label.clone(),
                });
            }
        }
    }
    references
}

fn instruction_id(instruction: &VirInstruction) -> &str {
    match instruction {
        VirInstruction::Const { id, .. }
        | VirInstruction::Copy { id, .. }
        | VirInstruction::BinOp { id, .. }
        | VirInstruction::UnaryOp { id, .. }
        | VirInstruction::Convert { id, .. }
        | VirInstruction::Field { id, .. }
        | VirInstruction::Index { id, .. }
        | VirInstruction::MakeStruct { id, .. }
        | VirInstruction::MakeArray { id, .. }
        | VirInstruction::CallStatic { id, .. } => id,
    }
}

fn validate_utf8_boundaries(
    map: &SourceMap,
    captured_inputs: &[CapturedInput<'_>],
) -> Result<(), SourceMapError> {
    for entry in &map.entries {
        let SourceOrigin::Source {
            normalized_path,
            start,
            end,
            ..
        } = &entry.origin
        else {
            continue;
        };
        let input = unique_input(captured_inputs, normalized_path).ok_or_else(|| {
            SourceMapError::new(
                SourceMapValidationPhase::Utf8,
                SourceMapErrorCode::Utf8Boundary,
                "captured source disappeared during validation",
            )
        })?;
        let source = std::str::from_utf8(input.bytes).map_err(|_| {
            SourceMapError::new(
                SourceMapValidationPhase::Utf8,
                SourceMapErrorCode::Utf8Boundary,
                "captured source is not UTF-8",
            )
        })?;
        let start = usize::try_from(*start).unwrap_or(usize::MAX);
        let end = usize::try_from(*end).unwrap_or(usize::MAX);
        if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return Err(SourceMapError::new(
                SourceMapValidationPhase::Utf8,
                SourceMapErrorCode::Utf8Boundary,
                "source range splits a Unicode scalar",
            ));
        }
    }
    Ok(())
}

fn recompute_source_map_hash_from_value(
    value: &StrictJsonValue,
) -> Result<LowercaseSha256, SourceMapError> {
    let preimage = value
        .clone_without_fields(&["source_map_hash"])
        .map_err(|error| {
            SourceMapError::new(
                SourceMapValidationPhase::Shape,
                SourceMapErrorCode::Shape,
                error.to_string(),
            )
        })?;
    let digest = hash_canonical_json(SOURCE_MAP_HASH_DOMAIN, &preimage).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Hash,
            SourceMapErrorCode::Hash,
            error.to_string(),
        )
    })?;
    LowercaseSha256::new(digest.to_hex()).map_err(|error| {
        SourceMapError::new(
            SourceMapValidationPhase::Hash,
            SourceMapErrorCode::Hash,
            error.to_string(),
        )
    })
}

/// Rechecks that one captured input has the manifest size and raw digest.
pub(crate) fn captured_input_matches(
    captured: &CapturedInput<'_>,
    size_bytes: u64,
    sha256: &str,
) -> bool {
    u64::try_from(captured.bytes.len()).ok() == Some(size_bytes)
        && sha256_raw_file_bytes(captured.bytes).to_hex() == sha256
}
