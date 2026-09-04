//! Candidate-only source artifacts for the practical C# profile.
//!
//! The active source-artifact family remains unchanged.  This module is a
//! private, explicitly injected staging boundary: it accepts only a validated
//! practical-profile request, the registered foundation bundle, and artifacts
//! whose complete linkage is recomputed here.  It neither discovers nor
//! installs a registry and it deliberately does not interpret `mpk.vir.v2`;
//! that importer belongs to CSHARP-03-T02-W05.

use crate::csharp_practical_registry::{
    SuccessorCompiledSemanticProfile, SuccessorSemanticContext, ValidatedSuccessorRequest,
    CSHARP_PRACTICAL_PROFILE, FOUNDATION_DESCRIPTOR_ID, FOUNDATION_DESCRIPTOR_SCHEMA,
};
use crate::csharp_practical_vir_model::{
    csharp_practical_declaration_id, csharp_practical_stored_member_id,
    validate_closed_instance_set, validate_closed_operation_signature,
    validate_foundation_context_linkage, ClosedInstanceSet, ClosedOperationSignature,
    RequiredCheck, ValidatedClosedRootSet, ValidatedFoundationBundle, CLOSED_INSTANCES_SCHEMA,
    CSHARP_PRACTICAL_OPERATIONS_SCHEMA, CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
};
use crate::hash::{hash_domain_separated_raw, sha256_raw_file_bytes, HashDomain};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const TYPE_CONTRACT_SCHEMA: &str = "mpk.csharp.type_contract.v1";
pub const METHOD_CONTRACT_SCHEMA: &str = "mpk.csharp.contract.v1";
pub const SEMANTIC_BINDING_SCHEMA: &str = "mpk.csharp.semantic_binding.v1";
pub const SEMANTIC_BINDINGS_SCHEMA: &str = "mpk.csharp.semantic_bindings.v1";
pub const BOUNDARY_CONTRACT_SCHEMA: &str = "mpk.csharp.boundary.v1";
pub const BOUNDARY_INPUT_SCHEMA: &str = "mpk.csharp.boundary_input.v1";
pub const BOUNDARY_OUTPUT_SCHEMA: &str = "mpk.csharp.boundary_output.v1";
pub const TRANSITION_CONTRACT_SCHEMA: &str = "mpk.csharp.transition.v1";
pub const SOURCE_MAP_SCHEMA: &str = "mpk.source_map.v2";
pub const FRONTEND_SOURCE_MANIFEST_SCHEMA: &str = "mpk.source_manifest.frontend.v2";
pub const CERTIFICATE_SOURCE_MANIFEST_SCHEMA: &str = "mpk.source_manifest.certificate.v2";
pub const SOURCE_ARTIFACTS_SCHEMA: &str = "mpk.frontend.source_artifacts.v2";
pub const SUCCESSOR_VIR_SCHEMA: &str = "mpk.vir.v2";
pub const SUCCESSOR_VC_SCHEMA: &str = "mpk.vc.v3";
pub const SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA: &str = "mpk.vc.cert_skeleton.v3";

pub const TYPE_CONTRACT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-TYPE-CONTRACT-1.0");
pub const METHOD_CONTRACT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-METHOD-CONTRACT-1.0");
pub const SEMANTIC_BINDING_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SEMANTIC-BINDING-1.0");
pub const SEMANTIC_BINDING_SET_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SEMANTIC-BINDING-SET-1.0");
pub const BOUNDARY_CONTRACT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-BOUNDARY-CONTRACT-1.0");
pub const BOUNDARY_INPUT_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-BOUNDARY-INPUT-1.0");
pub const BOUNDARY_OUTPUT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-BOUNDARY-OUTPUT-1.0");
pub const CANONICAL_VALUE_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-CANONICAL-VALUE-1.0");
pub const TRANSITION_CONTRACT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-TRANSITION-CONTRACT-1.0");
pub const OPERATIONS_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-OPERATIONS-1.0");
pub const REQUIRED_CHECKS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-REQUIRED-CHECKS-1.0");
pub const DECLARATION_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-CSHARP-DECLARATION-1.0");
pub const DECLARATION_PROVENANCE_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-DECLARATION-PROVENANCE-1.0");
pub const SOURCE_MAP_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MAP-2.0");
pub const SOURCE_MANIFEST_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-SOURCE-MANIFEST-2.0");
pub const SOURCE_ARTIFACTS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-FRONTEND-SOURCE-ARTIFACTS-2.0");
pub const INPUT_SET_HASH_DOMAIN: HashDomain = HashDomain::new("MPK-INPUT-SET-0.1");

pub const PRACTICAL_SOURCE_ARTIFACT_IDENTITIES: &[&str] = &[
    TYPE_CONTRACT_SCHEMA,
    METHOD_CONTRACT_SCHEMA,
    SEMANTIC_BINDING_SCHEMA,
    SEMANTIC_BINDINGS_SCHEMA,
    BOUNDARY_CONTRACT_SCHEMA,
    BOUNDARY_INPUT_SCHEMA,
    BOUNDARY_OUTPUT_SCHEMA,
    TRANSITION_CONTRACT_SCHEMA,
    CLOSED_INSTANCES_SCHEMA,
    CSHARP_PRACTICAL_OPERATIONS_SCHEMA,
    CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
    SOURCE_MAP_SCHEMA,
    FRONTEND_SOURCE_MANIFEST_SCHEMA,
    CERTIFICATE_SOURCE_MANIFEST_SCHEMA,
    SOURCE_ARTIFACTS_SCHEMA,
];

pub const PRACTICAL_SOURCE_ARTIFACT_HASH_DOMAINS: &[&str] = &[
    "MPK-CSHARP-TYPE-CONTRACT-1.0",
    "MPK-CSHARP-METHOD-CONTRACT-1.0",
    "MPK-CSHARP-SEMANTIC-BINDING-1.0",
    "MPK-CSHARP-SEMANTIC-BINDING-SET-1.0",
    "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
    "MPK-CSHARP-BOUNDARY-INPUT-1.0",
    "MPK-CSHARP-BOUNDARY-OUTPUT-1.0",
    "MPK-CSHARP-CANONICAL-VALUE-1.0",
    "MPK-CSHARP-TRANSITION-CONTRACT-1.0",
    "MPK-CSHARP-OPERATIONS-1.0",
    "MPK-CSHARP-REQUIRED-CHECKS-1.0",
    "MPK-CSHARP-DECLARATION-1.0",
    "MPK-CSHARP-DECLARATION-PROVENANCE-1.0",
    "MPK-CSHARP-CLOSED-INSTANCES-1.0",
    "MPK-CSHARP-PRACTICAL-FOUNDATION-1.0",
    "MPK-SOURCE-MAP-2.0",
    "MPK-SOURCE-MANIFEST-2.0",
    "MPK-FRONTEND-SOURCE-ARTIFACTS-2.0",
    "MPK-INPUT-SET-0.1",
];

pub const PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX: usize = 16 * 1024 * 1024;
pub const PRACTICAL_ARTIFACT_NESTING_MAX: usize = 128;
pub const SEMANTIC_BINDING_COUNT_MAX: usize = 128;

const TYPE_CONTRACT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "compilation_id",
    "source_type_id",
    "source_content_sha256",
    "ordered_member_ids",
    "recursive_default",
    "default_eligible",
    "required_member_ids",
    "init_member_ids",
    "construction_invariant",
    "invariants",
    "structural_equality",
    "structural_order",
    "contract_sha256",
];
const METHOD_CONTRACT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "compilation_id",
    "callable_id",
    "source_content_sha256",
    "termination",
    "requires",
    "ensures",
    "exceptional_cases",
    "modifies",
    "loops",
    "contract_sha256",
];
const SEMANTIC_BINDINGS_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "compilation_id",
    "bindings",
    "binding_set_sha256",
];
const BOUNDARY_CONTRACT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "compilation_id",
    "boundary_id",
    "selected_callable_id",
    "input_fields",
    "output_fields",
    "canonical_json_profile",
    "parse_format_profile",
    "evidence_linkage",
    "contract_sha256",
];
const BOUNDARY_INPUT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "boundary_contract_sha256",
    "raw_input",
    "canonical_document_utf8_sha256",
    "canonical_value",
    "canonical_value_sha256",
    "capture_sha256",
];
const BOUNDARY_OUTPUT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "boundary_contract_sha256",
    "source_value",
    "source_value_sha256",
    "canonical_document_utf8_sha256",
    "reparsed_value",
    "reparsed_value_sha256",
    "capture_sha256",
];
const TRANSITION_CONTRACT_FIELDS: &[&str] = &[
    "schema",
    "semantic_context",
    "compilation_id",
    "transition_id",
    "selected_callable_id",
    "state_type_id",
    "command_type_id",
    "context_type_id",
    "apply_result_binding_id",
    "transition_binding_id",
    "domain_error_binding_id",
    "state_invariant",
    "version_rule",
    "idempotency",
    "accepted_commands",
    "event_relation",
    "response_relation",
    "errors",
    "contract_sha256",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalArtifactKind {
    ArtifactRef,
    SourceLocation,
    Selection,
    TypeContract,
    MethodContract,
    SemanticBindings,
    BoundaryContract,
    BoundaryInput,
    BoundaryOutput,
    TransitionContract,
    Operations,
    RequiredChecks,
    SourceMap,
    FrontendManifest,
    CertificateManifest,
    SourceArtifacts,
}

impl PracticalArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArtifactRef => "artifact_ref",
            Self::SourceLocation => "source_location",
            Self::Selection => "selection",
            Self::TypeContract => "type_contract",
            Self::MethodContract => "method_contract",
            Self::SemanticBindings => "semantic_bindings",
            Self::BoundaryContract => "boundary_contract",
            Self::BoundaryInput => "boundary_input",
            Self::BoundaryOutput => "boundary_output",
            Self::TransitionContract => "transition_contract",
            Self::Operations => "operations",
            Self::RequiredChecks => "required_checks",
            Self::SourceMap => "source_map",
            Self::FrontendManifest => "frontend_manifest",
            Self::CertificateManifest => "certificate_manifest",
            Self::SourceArtifacts => "source_artifacts",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalArtifactPhase {
    Transport,
    Shape,
    Order,
    Identity,
    Hash,
    Context,
    Source,
    Foundation,
    Operation,
    Boundary,
    Linkage,
}

impl PracticalArtifactPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Order => "order",
            Self::Identity => "identity",
            Self::Hash => "hash",
            Self::Context => "context",
            Self::Source => "source",
            Self::Foundation => "foundation",
            Self::Operation => "operation",
            Self::Boundary => "boundary",
            Self::Linkage => "linkage",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PracticalArtifactErrorCode {
    Json,
    DuplicateField,
    FieldOrder,
    Shape,
    Schema,
    Hash,
    Context,
    Compilation,
    SourceInventory,
    SourceSpan,
    Foundation,
    Operation,
    BoundaryBytes,
    BoundaryValue,
    MissingMember,
    DuplicateMember,
    Linkage,
}

impl PracticalArtifactErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "CSHARP_PRACTICAL_ARTIFACT_JSON",
            Self::DuplicateField => "CSHARP_PRACTICAL_ARTIFACT_DUPLICATE_FIELD",
            Self::FieldOrder => "CSHARP_PRACTICAL_ARTIFACT_FIELD_ORDER",
            Self::Shape => "CSHARP_PRACTICAL_ARTIFACT_SHAPE",
            Self::Schema => "CSHARP_PRACTICAL_ARTIFACT_SCHEMA",
            Self::Hash => "CSHARP_PRACTICAL_ARTIFACT_HASH",
            Self::Context => "CSHARP_PRACTICAL_ARTIFACT_CONTEXT",
            Self::Compilation => "CSHARP_PRACTICAL_ARTIFACT_COMPILATION",
            Self::SourceInventory => "CSHARP_PRACTICAL_SOURCE_INVENTORY",
            Self::SourceSpan => "CSHARP_PRACTICAL_SOURCE_SPAN",
            Self::Foundation => "CSHARP_PRACTICAL_ARTIFACT_FOUNDATION",
            Self::Operation => "CSHARP_PRACTICAL_ARTIFACT_OPERATION",
            Self::BoundaryBytes => "CSHARP_PRACTICAL_BOUNDARY_BYTES",
            Self::BoundaryValue => "CSHARP_PRACTICAL_BOUNDARY_VALUE",
            Self::MissingMember => "CSHARP_PRACTICAL_ARTIFACT_MISSING_MEMBER",
            Self::DuplicateMember => "CSHARP_PRACTICAL_ARTIFACT_DUPLICATE_MEMBER",
            Self::Linkage => "CSHARP_PRACTICAL_ARTIFACT_LINKAGE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PracticalArtifactError {
    kind: PracticalArtifactKind,
    phase: PracticalArtifactPhase,
    code: PracticalArtifactErrorCode,
}

impl PracticalArtifactError {
    pub const fn kind(&self) -> PracticalArtifactKind {
        self.kind
    }

    pub const fn phase(&self) -> PracticalArtifactPhase {
        self.phase
    }

    pub const fn code(&self) -> PracticalArtifactErrorCode {
        self.code
    }
}

impl fmt::Display for PracticalArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} for {} at {}",
            self.code.as_str(),
            self.kind.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for PracticalArtifactError {}

fn failure(
    kind: PracticalArtifactKind,
    phase: PracticalArtifactPhase,
    code: PracticalArtifactErrorCode,
) -> PracticalArtifactError {
    PracticalArtifactError { kind, phase, code }
}

/// JSON value used by the practical profile's schema-ordered canonical form.
///
/// Unlike RFC 8785/JCS, objects retain the exact field order frozen by the
/// schema.  Positive integers retain the full `u64` metadata range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PracticalJsonValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    String(String),
    /// A JSON string containing at least one lone UTF-16 surrogate.
    ///
    /// Ordinary Unicode-scalar strings use `String`; this form preserves the
    /// exact C# UTF-16 value that Rust's UTF-8 `String` cannot represent.
    Utf16String(Vec<u16>),
    Array(Vec<PracticalJsonValue>),
    Object(Vec<(String, PracticalJsonValue)>),
}

impl PracticalJsonValue {
    pub fn object(entries: Vec<(&str, PracticalJsonValue)>) -> Self {
        Self::Object(
            entries
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
        )
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn utf16_string(value: impl Into<Vec<u16>>) -> Self {
        Self::Utf16String(value.into())
    }

    pub fn get(&self, name: &str) -> Option<&Self> {
        self.as_object()?
            .iter()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    pub fn as_object(&self) -> Option<&[(String, PracticalJsonValue)]> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[PracticalJsonValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::U64(value) => Some(*value),
            Self::I64(value) if *value >= 0 => u64::try_from(*value).ok(),
            _ => None,
        }
    }

    fn without_last_field(&self, field: &str) -> Option<Self> {
        let entries = self.as_object()?;
        let (last_name, _) = entries.last()?;
        if last_name != field {
            return None;
        }
        Some(Self::Object(entries[..entries.len() - 1].to_vec()))
    }
}

struct PracticalJsonVisitor;

impl<'de> Visitor<'de> for PracticalJsonVisitor {
    type Value = PracticalJsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a practical-profile canonical JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::I64(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::U64(value))
    }

    fn visit_f64<E: de::Error>(self, _value: f64) -> Result<Self::Value, E> {
        Err(E::custom("floating-point JSON numbers are forbidden"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(PracticalJsonValue::String(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(PracticalJsonSeed)? {
            values.push(value);
        }
        Ok(PracticalJsonValue::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut names = BTreeSet::new();
        let mut entries = Vec::new();
        while let Some(name) = map.next_key::<String>()? {
            if !names.insert(name.clone()) {
                return Err(de::Error::custom(format!(
                    "duplicate practical JSON field {name:?}"
                )));
            }
            let value = map.next_value_seed(PracticalJsonSeed)?;
            entries.push((name, value));
        }
        Ok(PracticalJsonValue::Object(entries))
    }
}

struct PracticalJsonSeed;

impl<'de> DeserializeSeed<'de> for PracticalJsonSeed {
    type Value = PracticalJsonValue;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(PracticalJsonVisitor)
    }
}

impl<'de> Deserialize<'de> for PracticalJsonValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PracticalJsonVisitor)
    }
}

pub fn canonical_practical_json_bytes(
    value: &PracticalJsonValue,
) -> Result<Vec<u8>, PracticalArtifactError> {
    let mut output = Vec::new();
    write_practical_json(value, &mut output, 0).map_err(|code| {
        failure(
            PracticalArtifactKind::SourceArtifacts,
            PracticalArtifactPhase::Transport,
            code,
        )
    })?;
    Ok(output)
}

pub fn parse_canonical_practical_json(
    kind: PracticalArtifactKind,
    transport: &[u8],
) -> Result<PracticalJsonValue, PracticalArtifactError> {
    if transport.len() > PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX
        || transport.starts_with(&[0xef, 0xbb, 0xbf])
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Transport,
            PracticalArtifactErrorCode::Json,
        ));
    }
    let surrogate_transport = preprocess_surrogate_escapes(transport);
    let parse_transport = surrogate_transport.as_deref().unwrap_or(transport);
    let mut deserializer = serde_json::Deserializer::from_slice(parse_transport);
    let mut parsed = PracticalJsonValue::deserialize(&mut deserializer).map_err(|error| {
        let code = if error.to_string().contains("duplicate practical JSON field") {
            PracticalArtifactErrorCode::DuplicateField
        } else {
            PracticalArtifactErrorCode::Json
        };
        failure(kind, PracticalArtifactPhase::Transport, code)
    })?;
    deserializer.end().map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Transport,
            PracticalArtifactErrorCode::Json,
        )
    })?;
    if surrogate_transport.is_some() {
        decode_surrogate_markers(&mut parsed).map_err(|_| {
            failure(
                kind,
                PracticalArtifactPhase::Transport,
                PracticalArtifactErrorCode::Json,
            )
        })?;
    }
    let mut canonical = Vec::new();
    write_practical_json(&parsed, &mut canonical, 0)
        .map_err(|code| failure(kind, PracticalArtifactPhase::Transport, code))?;
    if canonical != transport {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Transport,
            PracticalArtifactErrorCode::Json,
        ));
    }
    Ok(parsed)
}

fn write_practical_json(
    value: &PracticalJsonValue,
    output: &mut Vec<u8>,
    depth: usize,
) -> Result<(), PracticalArtifactErrorCode> {
    if depth > PRACTICAL_ARTIFACT_NESTING_MAX {
        return Err(PracticalArtifactErrorCode::Json);
    }
    match value {
        PracticalJsonValue::Null => output.extend_from_slice(b"null"),
        PracticalJsonValue::Bool(true) => output.extend_from_slice(b"true"),
        PracticalJsonValue::Bool(false) => output.extend_from_slice(b"false"),
        PracticalJsonValue::I64(value) => output.extend_from_slice(value.to_string().as_bytes()),
        PracticalJsonValue::U64(value) => output.extend_from_slice(value.to_string().as_bytes()),
        PracticalJsonValue::String(value) => write_practical_string(value, output)?,
        PracticalJsonValue::Utf16String(value) => write_practical_utf16_string(value, output)?,
        PracticalJsonValue::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_practical_json(value, output, depth + 1)?;
            }
            output.push(b']');
        }
        PracticalJsonValue::Object(entries) => {
            let mut names = BTreeSet::new();
            output.push(b'{');
            for (index, (name, value)) in entries.iter().enumerate() {
                if !names.insert(name) {
                    return Err(PracticalArtifactErrorCode::DuplicateField);
                }
                if index != 0 {
                    output.push(b',');
                }
                write_practical_string(name, output)?;
                output.push(b':');
                write_practical_json(value, output, depth + 1)?;
            }
            output.push(b'}');
        }
    }
    if output.len() > PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX {
        return Err(PracticalArtifactErrorCode::Json);
    }
    Ok(())
}

fn write_practical_string(
    value: &str,
    output: &mut Vec<u8>,
) -> Result<(), PracticalArtifactErrorCode> {
    output.push(b'"');
    for character in value.chars() {
        write_practical_character(character, output);
        require_practical_output_bound(output)?;
    }
    output.push(b'"');
    require_practical_output_bound(output)
}

fn write_practical_utf16_string(
    value: &[u16],
    output: &mut Vec<u8>,
) -> Result<(), PracticalArtifactErrorCode> {
    output.push(b'"');
    let mut index = 0;
    while index < value.len() {
        let unit = value[index];
        if (0xd800..=0xdbff).contains(&unit)
            && value
                .get(index + 1)
                .is_some_and(|next| (0xdc00..=0xdfff).contains(next))
        {
            let trailing = value[index + 1];
            let scalar =
                0x1_0000 + ((u32::from(unit) - 0xd800) << 10) + (u32::from(trailing) - 0xdc00);
            write_practical_character(
                char::from_u32(scalar).expect("validated surrogate pair is a Unicode scalar"),
                output,
            );
            index += 2;
        } else if (0xd800..=0xdfff).contains(&unit) {
            write_practical_hex_escape(unit, output);
            index += 1;
        } else {
            write_practical_character(
                char::from_u32(u32::from(unit)).expect("nonsurrogate UTF-16 unit is a scalar"),
                output,
            );
            index += 1;
        }
        require_practical_output_bound(output)?;
    }
    output.push(b'"');
    require_practical_output_bound(output)
}

fn write_practical_character(character: char, output: &mut Vec<u8>) {
    match character {
        '"' => output.extend_from_slice(b"\\\""),
        '\\' => output.extend_from_slice(b"\\\\"),
        '\u{0000}'..='\u{001f}' => write_practical_hex_escape(character as u16, output),
        _ => {
            let mut bytes = [0_u8; 4];
            output.extend_from_slice(character.encode_utf8(&mut bytes).as_bytes());
        }
    }
}

fn write_practical_hex_escape(unit: u16, output: &mut Vec<u8>) {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";
    output.extend_from_slice(b"\\u");
    for shift in [12, 8, 4, 0] {
        output.push(LOWER_HEX[usize::from((unit >> shift) & 0x0f)]);
    }
}

fn require_practical_output_bound(output: &[u8]) -> Result<(), PracticalArtifactErrorCode> {
    if output.len() <= PRACTICAL_ARTIFACT_TRANSPORT_BYTES_MAX {
        Ok(())
    } else {
        Err(PracticalArtifactErrorCode::Json)
    }
}

fn preprocess_surrogate_escapes(transport: &[u8]) -> Option<Vec<u8>> {
    const MARKER_ESCAPE: &[u8] = b"\\u0001";
    let mut output = Vec::with_capacity(transport.len());
    let mut in_string = false;
    let mut found_surrogate = false;
    let mut index = 0;
    while index < transport.len() {
        let byte = transport[index];
        if !in_string {
            output.push(byte);
            in_string = byte == b'"';
            index += 1;
            continue;
        }
        if byte == b'"' {
            output.push(byte);
            in_string = false;
            index += 1;
            continue;
        }
        if byte == b'\\' && transport.get(index + 1) == Some(&b'u') && index + 6 <= transport.len()
        {
            let digits = &transport[index + 2..index + 6];
            if let Some(unit) = decode_hex_quad(digits) {
                if unit == 1 {
                    output.extend_from_slice(MARKER_ESCAPE);
                    output.extend_from_slice(MARKER_ESCAPE);
                    index += 6;
                    continue;
                }
                if (0xd800..=0xdfff).contains(&unit) {
                    output.extend_from_slice(MARKER_ESCAPE);
                    output.push(b's');
                    output.extend_from_slice(digits);
                    found_surrogate = true;
                    index += 6;
                    continue;
                }
            }
        }
        if byte == b'\\' && index + 1 < transport.len() {
            output.extend_from_slice(&transport[index..index + 2]);
            index += 2;
        } else {
            output.push(byte);
            index += 1;
        }
    }
    found_surrogate.then_some(output)
}

fn decode_hex_quad(digits: &[u8]) -> Option<u16> {
    if digits.len() != 4 {
        return None;
    }
    digits.iter().try_fold(0_u16, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => u16::from(byte - b'0'),
            b'a'..=b'f' => u16::from(byte - b'a') + 10,
            b'A'..=b'F' => u16::from(byte - b'A') + 10,
            _ => return None,
        };
        Some((value << 4) | digit)
    })
}

fn decode_surrogate_markers(value: &mut PracticalJsonValue) -> Result<(), ()> {
    match value {
        PracticalJsonValue::String(text) if text.contains('\u{0001}') => {
            let characters = text.chars().collect::<Vec<_>>();
            let mut units = Vec::with_capacity(characters.len());
            let mut has_surrogate = false;
            let mut index = 0;
            while index < characters.len() {
                if characters[index] != '\u{0001}' {
                    let mut buffer = [0_u16; 2];
                    units.extend_from_slice(characters[index].encode_utf16(&mut buffer));
                    index += 1;
                    continue;
                }
                if characters.get(index + 1) == Some(&'\u{0001}') {
                    units.push(1);
                    index += 2;
                    continue;
                }
                if characters.get(index + 1) != Some(&'s') || index + 6 > characters.len() {
                    return Err(());
                }
                let mut digits = [0_u8; 4];
                for (target, character) in digits
                    .iter_mut()
                    .zip(characters[index + 2..index + 6].iter())
                {
                    *target = u8::try_from(*character).map_err(|_| ())?;
                }
                let unit = decode_hex_quad(&digits).ok_or(())?;
                if !(0xd800..=0xdfff).contains(&unit) {
                    return Err(());
                }
                units.push(unit);
                has_surrogate = true;
                index += 6;
            }
            if has_surrogate {
                *value = PracticalJsonValue::Utf16String(units);
            } else {
                *text = String::from_utf16(&units).map_err(|_| ())?;
            }
        }
        PracticalJsonValue::Array(values) => {
            for value in values {
                decode_surrogate_markers(value)?;
            }
        }
        PracticalJsonValue::Object(entries) => {
            for (name, value) in entries {
                let mut decoded_name = PracticalJsonValue::String(name.clone());
                decode_surrogate_markers(&mut decoded_name)?;
                *name = match decoded_name {
                    PracticalJsonValue::String(name) => name,
                    _ => return Err(()),
                };
                decode_surrogate_markers(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct PracticalArtifactContext {
    typed_semantic_context: SuccessorSemanticContext,
    semantic_context: PracticalJsonValue,
    compilation_id: String,
    selection_sha256: String,
    source_paths: Vec<String>,
    selected_root_ids: Vec<String>,
    sidecar_paths: Vec<String>,
    linkage_key: Vec<u8>,
}

impl PracticalArtifactContext {
    pub fn semantic_context(&self) -> &PracticalJsonValue {
        &self.semantic_context
    }

    pub fn compilation_id(&self) -> &str {
        &self.compilation_id
    }

    pub fn selection_sha256(&self) -> &str {
        &self.selection_sha256
    }

    pub fn source_paths(&self) -> &[String] {
        &self.source_paths
    }

    pub fn selected_root_ids(&self) -> &[String] {
        &self.selected_root_ids
    }

    pub fn sidecar_paths(&self) -> &[String] {
        &self.sidecar_paths
    }
}

pub fn bind_practical_artifact_context(
    request: &ValidatedSuccessorRequest,
    foundation: &ValidatedFoundationBundle,
) -> Result<PracticalArtifactContext, PracticalArtifactError> {
    if request.compiled_profile() != SuccessorCompiledSemanticProfile::CSharpPracticalV1
        || request.semantic_context().semantic_profile() != CSHARP_PRACTICAL_PROFILE
    {
        return Err(failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Context,
            PracticalArtifactErrorCode::Context,
        ));
    }
    validate_foundation_context_linkage(foundation, request.semantic_context()).map_err(|_| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        )
    })?;
    let selection = request.selection();
    let compilation_id = selection
        .get("compilation_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            failure(
                PracticalArtifactKind::Selection,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            )
        })?
        .to_owned();
    let selection_sha256 = selection
        .get("selection_sha256")
        .and_then(Value::as_str)
        .filter(|value| valid_sha256(value))
        .ok_or_else(|| {
            failure(
                PracticalArtifactKind::Selection,
                PracticalArtifactPhase::Hash,
                PracticalArtifactErrorCode::Hash,
            )
        })?
        .to_owned();
    let source_paths = string_array(selection.get("source_paths")).ok_or_else(|| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let selected_root_ids = string_array(selection.get("selected_root_ids")).ok_or_else(|| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let sidecar_paths = string_array(selection.get("sidecar_paths")).ok_or_else(|| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let semantic_context = semantic_context_value(request.semantic_context())?;
    let linkage_preimage = PracticalJsonValue::object(vec![
        ("semantic_context", semantic_context.clone()),
        (
            "compilation_id",
            PracticalJsonValue::string(&compilation_id),
        ),
        (
            "selection_sha256",
            PracticalJsonValue::string(&selection_sha256),
        ),
        (
            "foundation_sha256",
            PracticalJsonValue::string(foundation.content_sha256()),
        ),
    ]);
    let linkage_key = canonical_practical_json_bytes(&linkage_preimage).map_err(|_| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        )
    })?;
    Ok(PracticalArtifactContext {
        typed_semantic_context: request.semantic_context().clone(),
        semantic_context,
        compilation_id,
        selection_sha256,
        source_paths,
        selected_root_ids,
        sidecar_paths,
        linkage_key,
    })
}

fn semantic_context_value(
    context: &SuccessorSemanticContext,
) -> Result<PracticalJsonValue, PracticalArtifactError> {
    let registry = context.profile_registry();
    let descriptor = context.foundation_descriptor();
    let parameter_value = practical_parameters_value(context.semantic_parameters().value())?;
    Ok(PracticalJsonValue::object(vec![
        ("schema", PracticalJsonValue::string(context.schema())),
        (
            "profile_registry",
            PracticalJsonValue::object(vec![
                ("schema", PracticalJsonValue::string(registry.schema())),
                ("id", PracticalJsonValue::string(registry.id())),
                ("revision", PracticalJsonValue::U64(registry.revision())),
                (
                    "registry_sha256",
                    PracticalJsonValue::string(registry.registry_sha256()),
                ),
            ]),
        ),
        (
            "profile_entry_sha256",
            PracticalJsonValue::string(context.profile_entry_sha256()),
        ),
        (
            "source_language",
            PracticalJsonValue::string(context.source_language()),
        ),
        (
            "semantic_profile",
            PracticalJsonValue::string(context.semantic_profile()),
        ),
        (
            "semantic_parameters",
            PracticalJsonValue::object(vec![
                (
                    "schema",
                    PracticalJsonValue::string(context.semantic_parameters().schema()),
                ),
                ("value", parameter_value),
            ]),
        ),
        (
            "foundation_descriptor",
            PracticalJsonValue::object(vec![
                ("schema", PracticalJsonValue::string(descriptor.schema())),
                ("id", PracticalJsonValue::string(descriptor.id())),
                (
                    "content_sha256",
                    PracticalJsonValue::string(descriptor.content_sha256()),
                ),
            ]),
        ),
    ]))
}

fn practical_parameters_value(value: &Value) -> Result<PracticalJsonValue, PracticalArtifactError> {
    const FIELDS: &[&str] = &[
        "check_overflow_default",
        "documentation_mode",
        "language_version",
        "nullable_context",
        "optimization",
        "platform",
        "pointer_width",
        "preprocessor_symbols",
        "source_kind",
        "target_framework",
        "target_id",
        "unsafe",
    ];
    let object = value.as_object().ok_or_else(|| {
        failure(
            PracticalArtifactKind::Selection,
            PracticalArtifactPhase::Context,
            PracticalArtifactErrorCode::Context,
        )
    })?;
    let mut entries = Vec::with_capacity(FIELDS.len());
    for field in FIELDS {
        let item = object.get(*field).ok_or_else(|| {
            failure(
                PracticalArtifactKind::Selection,
                PracticalArtifactPhase::Context,
                PracticalArtifactErrorCode::Context,
            )
        })?;
        entries.push(((*field).to_owned(), serde_value_to_practical(item)?));
    }
    Ok(PracticalJsonValue::Object(entries))
}

fn serde_value_to_practical(value: &Value) -> Result<PracticalJsonValue, PracticalArtifactError> {
    match value {
        Value::Null => Ok(PracticalJsonValue::Null),
        Value::Bool(value) => Ok(PracticalJsonValue::Bool(*value)),
        Value::Number(value) => value
            .as_u64()
            .map(PracticalJsonValue::U64)
            .or_else(|| value.as_i64().map(PracticalJsonValue::I64))
            .ok_or_else(|| {
                failure(
                    PracticalArtifactKind::SourceArtifacts,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                )
            }),
        Value::String(value) => Ok(PracticalJsonValue::String(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(serde_value_to_practical)
            .collect::<Result<Vec<_>, _>>()
            .map(PracticalJsonValue::Array),
        Value::Object(values) => values
            .iter()
            .map(|(name, value)| serde_value_to_practical(value).map(|value| (name.clone(), value)))
            .collect::<Result<Vec<_>, _>>()
            .map(PracticalJsonValue::Object),
    }
}

fn string_array(value: Option<&Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|value| value.as_str().map(str::to_owned))
        .collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn raw_sha256(bytes: &[u8]) -> String {
    sha256_raw_file_bytes(bytes).to_hex()
}

fn hash_complete(
    domain: HashDomain,
    value: &PracticalJsonValue,
    kind: PracticalArtifactKind,
) -> Result<String, PracticalArtifactError> {
    let bytes = canonical_practical_json_bytes(value).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Hash,
            PracticalArtifactErrorCode::Hash,
        )
    })?;
    hash_domain_separated_raw(domain, &bytes)
        .map(|digest| digest.to_hex())
        .map_err(|_| {
            failure(
                kind,
                PracticalArtifactPhase::Hash,
                PracticalArtifactErrorCode::Hash,
            )
        })
}

fn hash_complete_with_sorted_objects(
    domain: HashDomain,
    value: &PracticalJsonValue,
    kind: PracticalArtifactKind,
) -> Result<String, PracticalArtifactError> {
    hash_complete(domain, &with_sorted_object_keys(value), kind)
}

fn with_sorted_object_keys(value: &PracticalJsonValue) -> PracticalJsonValue {
    match value {
        PracticalJsonValue::Array(values) => {
            PracticalJsonValue::Array(values.iter().map(with_sorted_object_keys).collect())
        }
        PracticalJsonValue::Object(entries) => {
            let mut entries = entries
                .iter()
                .map(|(name, value)| (name.clone(), with_sorted_object_keys(value)))
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            PracticalJsonValue::Object(entries)
        }
        other => other.clone(),
    }
}

fn hash_without_last(
    domain: HashDomain,
    value: &PracticalJsonValue,
    hash_field: &str,
    kind: PracticalArtifactKind,
) -> Result<String, PracticalArtifactError> {
    let preimage = value.without_last_field(hash_field).ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::FieldOrder,
        )
    })?;
    hash_complete(domain, &preimage, kind)
}

fn require_exact_fields(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
    expected: &[&str],
) -> Result<(), PracticalArtifactError> {
    let object = value.as_object().ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let actual = object
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    if actual == expected {
        return Ok(());
    }
    let actual_set = actual.iter().copied().collect::<BTreeSet<_>>();
    let expected_set = expected.iter().copied().collect::<BTreeSet<_>>();
    let code = if actual_set != expected_set {
        PracticalArtifactErrorCode::Shape
    } else {
        PracticalArtifactErrorCode::FieldOrder
    };
    Err(failure(kind, PracticalArtifactPhase::Order, code))
}

fn string_field<'a>(
    kind: PracticalArtifactKind,
    value: &'a PracticalJsonValue,
    field: &str,
) -> Result<&'a str, PracticalArtifactError> {
    value
        .get(field)
        .and_then(PracticalJsonValue::as_str)
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            )
        })
}

fn require_hash_field(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
    field: &str,
    domain: HashDomain,
) -> Result<String, PracticalArtifactError> {
    let actual = string_field(kind, value, field)?;
    if !valid_sha256(actual) || hash_without_last(domain, value, field, kind)? != actual {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Hash,
            PracticalArtifactErrorCode::Hash,
        ));
    }
    Ok(actual.to_owned())
}

fn require_sorted_object_hash_field(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
    field: &str,
    domain: HashDomain,
) -> Result<String, PracticalArtifactError> {
    let actual = string_field(kind, value, field)?;
    let preimage = value.without_last_field(field).ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::FieldOrder,
        )
    })?;
    if !valid_sha256(actual)
        || hash_complete_with_sorted_objects(domain, &preimage, kind)? != actual
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Hash,
            PracticalArtifactErrorCode::Hash,
        ));
    }
    Ok(actual.to_owned())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactRef {
    schema: String,
    sha256: String,
    canonical_bytes: u64,
    linkage_key: Vec<u8>,
    source_value: Option<Box<PracticalJsonValue>>,
    input_set_sha256: Option<String>,
}

impl ArtifactRef {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub const fn canonical_bytes(&self) -> u64 {
        self.canonical_bytes
    }

    /// Creates a context-bound reference to an artifact whose parser belongs
    /// to a later task.  W04 accepts only the frozen VIR/VC/skeleton identities
    /// here and never treats this reference as proof that its body is valid.
    pub fn opaque_successor(
        context: &PracticalArtifactContext,
        schema: &str,
        sha256: &str,
        canonical_bytes: u64,
    ) -> Result<Self, PracticalArtifactError> {
        if !matches!(
            schema,
            SUCCESSOR_VIR_SCHEMA | SUCCESSOR_VC_SCHEMA | SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA
        ) || !valid_sha256(sha256)
            || canonical_bytes == 0
        {
            return Err(failure(
                PracticalArtifactKind::ArtifactRef,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Schema,
            ));
        }
        Ok(Self {
            schema: schema.to_owned(),
            sha256: sha256.to_owned(),
            canonical_bytes,
            linkage_key: context.linkage_key.clone(),
            source_value: None,
            input_set_sha256: None,
        })
    }

    pub fn value(&self) -> PracticalJsonValue {
        PracticalJsonValue::object(vec![
            ("schema", PracticalJsonValue::string(&self.schema)),
            ("sha256", PracticalJsonValue::string(&self.sha256)),
            (
                "canonical_bytes",
                PracticalJsonValue::U64(self.canonical_bytes),
            ),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedPracticalArtifact {
    kind: PracticalArtifactKind,
    schema: String,
    hash: String,
    canonical_bytes: Vec<u8>,
    value: PracticalJsonValue,
    linkage_key: Vec<u8>,
    input_set_sha256: Option<String>,
}

impl ValidatedPracticalArtifact {
    pub const fn kind(&self) -> PracticalArtifactKind {
        self.kind
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn value(&self) -> &PracticalJsonValue {
        &self.value
    }

    pub fn artifact_ref(&self) -> ArtifactRef {
        ArtifactRef {
            schema: self.schema.clone(),
            sha256: self.hash.clone(),
            canonical_bytes: u64::try_from(self.canonical_bytes.len())
                .expect("bounded artifact length fits u64"),
            linkage_key: self.linkage_key.clone(),
            source_value: Some(Box::new(self.value.clone())),
            input_set_sha256: self.input_set_sha256.clone(),
        }
    }

    fn with_input_set(mut self, captures: &CapturedInputSet) -> Self {
        self.input_set_sha256 = Some(captures.snapshot_sha256().to_owned());
        self
    }
}

fn finalized_artifact(
    context: &PracticalArtifactContext,
    kind: PracticalArtifactKind,
    schema: &'static str,
    hash_field: &'static str,
    domain: HashDomain,
    mut fields: Vec<(&'static str, PracticalJsonValue)>,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let preimage = PracticalJsonValue::object(fields.clone());
    let hash = hash_complete(domain, &preimage, kind)?;
    fields.push((hash_field, PracticalJsonValue::string(&hash)));
    let value = PracticalJsonValue::object(fields);
    let canonical_bytes = canonical_practical_json_bytes(&value).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Transport,
            PracticalArtifactErrorCode::Json,
        )
    })?;
    Ok(ValidatedPracticalArtifact {
        kind,
        schema: schema.to_owned(),
        hash,
        canonical_bytes,
        value,
        linkage_key: context.linkage_key.clone(),
        input_set_sha256: None,
    })
}

fn require_linkage(
    context: &PracticalArtifactContext,
    kind: PracticalArtifactKind,
    reference: &ArtifactRef,
) -> Result<(), PracticalArtifactError> {
    if reference.linkage_key == context.linkage_key {
        Ok(())
    } else {
        Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ))
    }
}

fn recognized_artifact_schema(schema: &str) -> bool {
    matches!(
        schema,
        TYPE_CONTRACT_SCHEMA
            | METHOD_CONTRACT_SCHEMA
            | SEMANTIC_BINDINGS_SCHEMA
            | BOUNDARY_CONTRACT_SCHEMA
            | BOUNDARY_INPUT_SCHEMA
            | BOUNDARY_OUTPUT_SCHEMA
            | TRANSITION_CONTRACT_SCHEMA
            | CLOSED_INSTANCES_SCHEMA
            | CSHARP_PRACTICAL_OPERATIONS_SCHEMA
            | CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA
            | SOURCE_MAP_SCHEMA
            | FRONTEND_SOURCE_MANIFEST_SCHEMA
            | CERTIFICATE_SOURCE_MANIFEST_SCHEMA
            | SOURCE_ARTIFACTS_SCHEMA
            | SUCCESSOR_VIR_SCHEMA
            | SUCCESSOR_VC_SCHEMA
            | SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA
    )
}

pub fn validate_artifact_ref_document(transport: &[u8]) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::ArtifactRef;
    let value = parse_canonical_practical_json(kind, transport)?;
    validate_artifact_ref_value(kind, &value).map(|_| ())
}

fn validate_artifact_ref_value(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
) -> Result<(&str, &str, u64), PracticalArtifactError> {
    require_exact_fields(kind, value, &["schema", "sha256", "canonical_bytes"])?;
    let schema = string_field(kind, value, "schema")?;
    let sha256 = string_field(kind, value, "sha256")?;
    let canonical_bytes = value
        .get("canonical_bytes")
        .and_then(PracticalJsonValue::as_u64)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            )
        })?;
    if !recognized_artifact_schema(schema) || !valid_sha256(sha256) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Schema,
        ));
    }
    Ok((schema, sha256, canonical_bytes))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    source_file_ordinal: u16,
    start_byte: u32,
    end_byte: u32,
}

impl SourceLocation {
    pub const fn source_file_ordinal(self) -> u16 {
        self.source_file_ordinal
    }

    pub const fn start_byte(self) -> u32 {
        self.start_byte
    }

    pub const fn end_byte(self) -> u32 {
        self.end_byte
    }

    fn value(self) -> PracticalJsonValue {
        PracticalJsonValue::object(vec![
            (
                "source_file_ordinal",
                PracticalJsonValue::U64(u64::from(self.source_file_ordinal)),
            ),
            (
                "start_byte",
                PracticalJsonValue::U64(u64::from(self.start_byte)),
            ),
            (
                "end_byte",
                PracticalJsonValue::U64(u64::from(self.end_byte)),
            ),
        ])
    }
}

pub fn validate_source_location_document(
    transport: &[u8],
) -> Result<SourceLocation, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceLocation;
    let value = parse_canonical_practical_json(kind, transport)?;
    parse_source_location(kind, &value)
}

fn parse_source_location(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
) -> Result<SourceLocation, PracticalArtifactError> {
    require_exact_fields(
        kind,
        value,
        &["source_file_ordinal", "start_byte", "end_byte"],
    )?;
    let ordinal = u16::try_from(
        value
            .get("source_file_ordinal")
            .and_then(PracticalJsonValue::as_u64)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                )
            })?,
    )
    .map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let start = u32::try_from(
        value
            .get("start_byte")
            .and_then(PracticalJsonValue::as_u64)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                )
            })?,
    )
    .map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    let end = u32::try_from(
        value
            .get("end_byte")
            .and_then(PracticalJsonValue::as_u64)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                )
            })?,
    )
    .map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    if start >= end {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Source,
            PracticalArtifactErrorCode::SourceSpan,
        ));
    }
    Ok(SourceLocation {
        source_file_ordinal: ordinal,
        start_byte: start,
        end_byte: end,
    })
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OriginalInputKind {
    Source,
    Sidecar,
}

impl OriginalInputKind {
    const fn retained_input_kind(self) -> &'static str {
        match self {
            Self::Source => "source",
            // MPK-INPUT-SET-0.1 names verification-overlay inputs
            // "contract". Keep that retained spelling in the emitted row so
            // an importer can recompute the unchanged hash preimage directly.
            Self::Sidecar => "contract",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginalInput {
    pub kind: OriginalInputKind,
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedOriginalInput {
    kind: OriginalInputKind,
    path: String,
    raw_sha256: String,
    bytes: Vec<u8>,
}

impl CapturedOriginalInput {
    pub const fn kind(&self) -> OriginalInputKind {
        self.kind
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn value(&self) -> PracticalJsonValue {
        PracticalJsonValue::object(vec![
            (
                "kind",
                PracticalJsonValue::string(self.kind.retained_input_kind()),
            ),
            ("normalized_path", PracticalJsonValue::string(&self.path)),
            (
                "size_bytes",
                PracticalJsonValue::U64(
                    u64::try_from(self.bytes.len()).expect("captured input length fits u64"),
                ),
            ),
            ("sha256", PracticalJsonValue::string(&self.raw_sha256)),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct CapturedInputSet {
    entries: Vec<CapturedOriginalInput>,
    snapshot_sha256: String,
}

impl CapturedInputSet {
    pub fn entries(&self) -> &[CapturedOriginalInput] {
        &self.entries
    }

    pub fn snapshot_sha256(&self) -> &str {
        &self.snapshot_sha256
    }

    pub fn entry(&self, path: &str) -> Option<&CapturedOriginalInput> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    fn contains_source_raw_sha256(&self, sha256: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.kind == OriginalInputKind::Source && entry.raw_sha256 == sha256)
    }

    fn source_ordinal(&self, path: &str) -> Option<u16> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == OriginalInputKind::Source)
            .position(|entry| entry.path == path)
            .and_then(|index| u16::try_from(index).ok())
    }

    fn values(&self) -> Vec<PracticalJsonValue> {
        self.entries
            .iter()
            .map(CapturedOriginalInput::value)
            .collect()
    }
}

pub fn capture_original_inputs(
    context: &PracticalArtifactContext,
    mut inputs: Vec<OriginalInput>,
) -> Result<CapturedInputSet, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceMap;
    inputs.sort_by(|left, right| {
        (&left.path, left.kind.retained_input_kind())
            .cmp(&(&right.path, right.kind.retained_input_kind()))
    });
    let expected = context
        .source_paths
        .iter()
        .map(|path| (OriginalInputKind::Source, path.as_str()))
        .chain(
            context
                .sidecar_paths
                .iter()
                .map(|path| (OriginalInputKind::Sidecar, path.as_str())),
        )
        .collect::<BTreeSet<_>>();
    let actual = inputs
        .iter()
        .map(|input| (input.kind, input.path.as_str()))
        .collect::<BTreeSet<_>>();
    if actual != expected || actual.len() != inputs.len() {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Source,
            PracticalArtifactErrorCode::SourceInventory,
        ));
    }
    let mut entries = Vec::with_capacity(inputs.len());
    for input in inputs {
        if !crate::source_map::is_portable_normalized_path(&input.path)
            || u32::try_from(input.bytes.len()).is_err()
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Source,
                PracticalArtifactErrorCode::SourceInventory,
            ));
        }
        entries.push(CapturedOriginalInput {
            kind: input.kind,
            path: input.path,
            raw_sha256: raw_sha256(&input.bytes),
            bytes: input.bytes,
        });
    }
    let retained_inputs = entries
        .iter()
        .map(|entry| crate::source_manifest::InputEntry {
            kind: match entry.kind {
                OriginalInputKind::Source => crate::source_map::InputKind::Source,
                OriginalInputKind::Sidecar => crate::source_map::InputKind::Contract,
            },
            normalized_path: entry.path.clone(),
            size_bytes: i64::try_from(entry.bytes.len()).expect("u32 checked length fits i64"),
            sha256: entry.raw_sha256.clone(),
        })
        .collect::<Vec<_>>();
    let snapshot_sha256 = crate::source_manifest::input_set_hash(&retained_inputs)
        .map_err(|_| {
            failure(
                kind,
                PracticalArtifactPhase::Hash,
                PracticalArtifactErrorCode::Hash,
            )
        })?
        .as_str()
        .to_owned();
    Ok(CapturedInputSet {
        entries,
        snapshot_sha256,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticBindingMember {
    pub role: String,
    pub member_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticArmMapping {
    pub source_tag: String,
    pub semantic_arm: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticBound {
    pub id: String,
    pub maximum: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticOperationMapping {
    pub operation: String,
    pub member_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticBindingInput {
    pub source_type_id: String,
    pub source_content_sha256: String,
    pub role: String,
    pub member_map: Vec<SemanticBindingMember>,
    pub tag_arms: Vec<SemanticArmMapping>,
    pub inferred_argument_ids: Vec<String>,
    pub default_arm: String,
    pub bounds: Vec<SemanticBound>,
    pub operation_map: Vec<SemanticOperationMapping>,
}

const SEMANTIC_BINDING_FIELDS: &[&str] = &[
    "schema",
    "source_type_id",
    "source_content_sha256",
    "role",
    "member_map",
    "tag_arms",
    "inferred_argument_ids",
    "default_arm",
    "bounds",
    "operation_map",
    "binding_sha256",
];

const OPTION_MEMBERS: &[&str] = &["tag", "value"];
const RESULT_MEMBERS: &[&str] = &["tag", "value", "error"];
const VALIDATION_MEMBERS: &[&str] = &["tag", "value", "errors"];
const TRANSITION_MEMBERS: &[&str] = &["state", "events", "response"];
const INSTANT_MEMBERS: &[&str] = &["milliseconds"];
const MONEY_MEMBERS: &[&str] = &["amount", "currency"];
const ELEMENT_MEMBERS: &[&str] = &["elements"];
const ENTRY_MEMBERS: &[&str] = &["key", "value"];
const MAP_MEMBERS: &[&str] = &["entries"];

const OPTION_ARMS: &[&str] = &["none", "some"];
const LOOKUP_ARMS: &[&str] = &["missing_key", "found"];
const RESULT_ARMS: &[&str] = &["ok", "error"];
const VALIDATION_ARMS: &[&str] = &["valid", "invalid"];
const BOUNDARY_ARMS: &[&str] = &["missing", "null", "value"];
const NO_NAMES: &[&str] = &[];

const BOUNDED_SEQUENCE_OPERATIONS: &[&str] = &["length", "read", "equal", "compare"];
const ORDERED_ENTRY_OPERATIONS: &[&str] = &["make", "key", "value", "equal", "compare"];
const ORDERED_MAP_OPERATIONS: &[&str] = &[
    "validate", "count", "contains", "lookup", "add", "replace", "equal", "compare",
];
const ORDERED_SET_OPERATIONS: &[&str] =
    &["validate", "count", "contains", "add", "equal", "compare"];
const OPTION_OPERATIONS: &[&str] = &[
    "none",
    "some",
    "has_value",
    "value",
    "value_or",
    "equal",
    "compare",
];
const LOOKUP_OPERATIONS: &[&str] = &["missing", "found", "is_found", "value", "equal", "compare"];
const RESULT_OPERATIONS: &[&str] = &[
    "ok",
    "error",
    "is_ok",
    "value",
    "error_value",
    "equal",
    "compare",
];
const VALIDATION_OPERATIONS: &[&str] = &[
    "valid",
    "invalid",
    "is_valid",
    "value",
    "errors",
    "append_errors",
    "equal",
    "compare",
];
const BOUNDARY_OPERATIONS: &[&str] = &[
    "missing", "null", "value", "tag", "payload", "equal", "compare",
];
const TRANSITION_OPERATIONS: &[&str] = &["make", "state", "events", "response", "equal", "compare"];
const MONEY_OPERATIONS: &[&str] = &[
    "create",
    "amount",
    "currency",
    "add",
    "subtract",
    "multiply",
    "divide",
    "amount_compare",
    "equal",
    "compare",
];
const INSTANT_OPERATIONS: &[&str] = &[
    "milliseconds",
    "compare",
    "add_duration",
    "subtract_duration",
    "difference",
];

#[derive(Clone, Copy)]
struct SemanticBindingRole {
    members: &'static [&'static str],
    arms: &'static [&'static str],
    argument_count: usize,
    default_arm: &'static str,
    bound: Option<(&'static str, u64)>,
    operations: &'static [&'static str],
}

fn semantic_binding_role(role: &str) -> Option<SemanticBindingRole> {
    let ordinary = |members, argument_count, operations| SemanticBindingRole {
        members,
        arms: NO_NAMES,
        argument_count,
        default_arm: "ineligible",
        bound: None,
        operations,
    };
    Some(match role {
        "option" => SemanticBindingRole {
            members: OPTION_MEMBERS,
            arms: OPTION_ARMS,
            argument_count: 1,
            default_arm: "none",
            bound: None,
            operations: OPTION_OPERATIONS,
        },
        "lookup" => SemanticBindingRole {
            members: OPTION_MEMBERS,
            arms: LOOKUP_ARMS,
            argument_count: 1,
            default_arm: "missing_key",
            bound: None,
            operations: LOOKUP_OPERATIONS,
        },
        "result" => SemanticBindingRole {
            members: RESULT_MEMBERS,
            arms: RESULT_ARMS,
            argument_count: 2,
            default_arm: "ineligible",
            bound: None,
            operations: RESULT_OPERATIONS,
        },
        "validation" => SemanticBindingRole {
            members: VALIDATION_MEMBERS,
            arms: VALIDATION_ARMS,
            argument_count: 2,
            default_arm: "ineligible",
            bound: Some(("errors", 256)),
            operations: VALIDATION_OPERATIONS,
        },
        "boundary_field" => SemanticBindingRole {
            members: OPTION_MEMBERS,
            arms: BOUNDARY_ARMS,
            argument_count: 1,
            default_arm: "ineligible",
            bound: None,
            operations: BOUNDARY_OPERATIONS,
        },
        "transition" => SemanticBindingRole {
            members: TRANSITION_MEMBERS,
            arms: NO_NAMES,
            argument_count: 3,
            default_arm: "ineligible",
            bound: Some(("events", 4_096)),
            operations: TRANSITION_OPERATIONS,
        },
        "instant" => ordinary(INSTANT_MEMBERS, 0, INSTANT_OPERATIONS),
        "money" => ordinary(MONEY_MEMBERS, 1, MONEY_OPERATIONS),
        "bounded_sequence" => SemanticBindingRole {
            bound: Some(("length", 4_096)),
            ..ordinary(ELEMENT_MEMBERS, 1, BOUNDED_SEQUENCE_OPERATIONS)
        },
        "ordered_entry" => ordinary(ENTRY_MEMBERS, 2, ORDERED_ENTRY_OPERATIONS),
        "ordered_map" => SemanticBindingRole {
            bound: Some(("length", 4_096)),
            ..ordinary(MAP_MEMBERS, 2, ORDERED_MAP_OPERATIONS)
        },
        "ordered_set" => SemanticBindingRole {
            bound: Some(("length", 4_096)),
            ..ordinary(ELEMENT_MEMBERS, 1, ORDERED_SET_OPERATIONS)
        },
        _ => return None,
    })
}

fn semantic_binding_value(
    binding: &SemanticBindingInput,
) -> Result<PracticalJsonValue, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let role = semantic_binding_role(&binding.role).ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    if !valid_source_declaration_id(&binding.source_type_id)
        || !valid_sha256(&binding.source_content_sha256)
        || binding.inferred_argument_ids.len() != role.argument_count
        || binding
            .inferred_argument_ids
            .iter()
            .any(|argument| !valid_concrete_type_id(argument))
        || binding.default_arm != role.default_arm
        || !valid_semantic_bounds(&binding.bounds, role.bound)
        || !valid_semantic_operations(&binding.operation_map, role.operations)
        || binding
            .member_map
            .iter()
            .any(|member| !valid_source_member_id(&member.member_id))
        || binding
            .tag_arms
            .iter()
            .any(|arm| !valid_canonical_enum_carrier(&arm.source_tag))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    let member_map = ordered_string_map(
        kind,
        binding
            .member_map
            .iter()
            .map(|member| (member.role.clone(), member.member_id.clone())),
        role.members,
    )?;
    let member_ids = binding
        .member_map
        .iter()
        .map(|member| member.member_id.as_str())
        .collect::<BTreeSet<_>>();
    if member_ids.len() != binding.member_map.len() {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::DuplicateMember,
        ));
    }
    let tag_arms = ordered_string_map(
        kind,
        binding
            .tag_arms
            .iter()
            .map(|arm| (arm.semantic_arm.clone(), arm.source_tag.clone())),
        role.arms,
    )?;
    if binding
        .tag_arms
        .iter()
        .map(|arm| arm.source_tag.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != binding.tag_arms.len()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::DuplicateMember,
        ));
    }
    let bounds = role.bound.map_or_else(
        || PracticalJsonValue::Object(Vec::new()),
        |(name, maximum)| {
            PracticalJsonValue::object(vec![(name, PracticalJsonValue::U64(maximum))])
        },
    );
    let operation_map = ordered_operation_map(&binding.operation_map, role.operations);
    let fields = vec![
        (
            "schema",
            PracticalJsonValue::string(SEMANTIC_BINDING_SCHEMA),
        ),
        (
            "source_type_id",
            PracticalJsonValue::string(&binding.source_type_id),
        ),
        (
            "source_content_sha256",
            PracticalJsonValue::string(&binding.source_content_sha256),
        ),
        ("role", PracticalJsonValue::string(&binding.role)),
        ("member_map", member_map),
        ("tag_arms", tag_arms),
        (
            "inferred_argument_ids",
            string_values(&binding.inferred_argument_ids),
        ),
        (
            "default_arm",
            PracticalJsonValue::string(&binding.default_arm),
        ),
        ("bounds", bounds),
        ("operation_map", operation_map),
    ];
    let preimage = PracticalJsonValue::object(fields.clone());
    let hash = hash_complete_with_sorted_objects(SEMANTIC_BINDING_HASH_DOMAIN, &preimage, kind)?;
    let mut complete = fields;
    complete.push(("binding_sha256", PracticalJsonValue::string(hash)));
    Ok(PracticalJsonValue::object(complete))
}

fn ordered_string_map(
    kind: PracticalArtifactKind,
    supplied: impl IntoIterator<Item = (String, String)>,
    expected_names: &[&str],
) -> Result<PracticalJsonValue, PracticalArtifactError> {
    let supplied = supplied.into_iter().collect::<BTreeMap<_, _>>();
    if supplied.len() != expected_names.len()
        || supplied
            .keys()
            .any(|name| !expected_names.contains(&name.as_str()))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    Ok(PracticalJsonValue::Object(
        expected_names
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    PracticalJsonValue::string(
                        supplied.get(*name).expect("complete name set checked"),
                    ),
                )
            })
            .collect(),
    ))
}

fn valid_semantic_bounds(bounds: &[SemanticBound], expected: Option<(&str, u64)>) -> bool {
    match (bounds, expected) {
        ([], None) => true,
        ([bound], Some((id, maximum))) => bound.id == id && bound.maximum == maximum,
        _ => false,
    }
}

fn valid_semantic_operations(operations: &[SemanticOperationMapping], inventory: &[&str]) -> bool {
    operations.iter().all(|operation| {
        inventory.contains(&operation.operation.as_str())
            && valid_source_declaration_id(&operation.member_id)
    }) && operations
        .iter()
        .map(|operation| operation.operation.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        == operations.len()
}

fn ordered_operation_map(
    supplied: &[SemanticOperationMapping],
    inventory: &[&str],
) -> PracticalJsonValue {
    let by_name = supplied
        .iter()
        .map(|mapping| (mapping.operation.as_str(), mapping.member_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    PracticalJsonValue::Object(
        inventory
            .iter()
            .filter_map(|name| {
                by_name
                    .get(name)
                    .map(|member_id| ((*name).to_owned(), PracticalJsonValue::string(*member_id)))
            })
            .collect(),
    )
}

fn valid_canonical_enum_carrier(value: &str) -> bool {
    let canonical = value == "0"
        || value.strip_prefix('-').is_some_and(|digits| {
            !digits.is_empty()
                && !digits.starts_with('0')
                && digits.bytes().all(|byte| byte.is_ascii_digit())
        })
        || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()));
    canonical
        && value
            .parse::<i128>()
            .is_ok_and(|value| value >= i128::from(i64::MIN) && value <= i128::from(u64::MAX))
}

pub fn build_semantic_bindings(
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    mut bindings: Vec<SemanticBindingInput>,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    bindings.sort_by(|left, right| left.source_type_id.cmp(&right.source_type_id));
    if bindings.len() > SEMANTIC_BINDING_COUNT_MAX
        || bindings
            .windows(2)
            .any(|pair| pair[0].source_type_id == pair[1].source_type_id)
        || bindings
            .iter()
            .any(|binding| !captures.contains_source_raw_sha256(&binding.source_content_sha256))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Source,
            PracticalArtifactErrorCode::SourceInventory,
        ));
    }
    let values = bindings
        .iter()
        .map(semantic_binding_value)
        .collect::<Result<Vec<_>, _>>()?;
    finalized_artifact(
        context,
        kind,
        SEMANTIC_BINDINGS_SCHEMA,
        "binding_set_sha256",
        SEMANTIC_BINDING_SET_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SEMANTIC_BINDINGS_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            ("bindings", PracticalJsonValue::Array(values)),
        ],
    )
    .map(|artifact| artifact.with_input_set(captures))
}

pub fn validate_semantic_bindings_document(
    context: Option<&PracticalArtifactContext>,
    captures: Option<&CapturedInputSet>,
    transport: &[u8],
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let value = parse_canonical_practical_json(kind, transport)?;
    validate_semantic_bindings_value(context, captures, &value)?;
    let hash = require_hash_field(
        kind,
        &value,
        "binding_set_sha256",
        SEMANTIC_BINDING_SET_HASH_DOMAIN,
    )?;
    let linkage_key = context.map_or_else(Vec::new, |context| context.linkage_key.clone());
    Ok(ValidatedPracticalArtifact {
        kind,
        schema: SEMANTIC_BINDINGS_SCHEMA.to_owned(),
        hash,
        canonical_bytes: transport.to_vec(),
        value,
        linkage_key,
        input_set_sha256: captures.map(|captures| captures.snapshot_sha256().to_owned()),
    })
}

fn validate_semantic_bindings_value(
    context: Option<&PracticalArtifactContext>,
    captures: Option<&CapturedInputSet>,
    value: &PracticalJsonValue,
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    require_exact_fields(kind, value, SEMANTIC_BINDINGS_FIELDS)?;
    if string_field(kind, value, "schema")? != SEMANTIC_BINDINGS_SCHEMA
        || value
            .get("semantic_context")
            .and_then(PracticalJsonValue::as_object)
            .is_none()
        || !valid_compilation_id(string_field(kind, value, "compilation_id")?)
        || value
            .get("bindings")
            .and_then(PracticalJsonValue::as_array)
            .is_none()
        || !string_field(kind, value, "binding_set_sha256").is_ok_and(valid_sha256)
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    if let Some(context) = context {
        require_context_and_compilation(kind, context, value)?;
    }
    let bindings = value
        .get("bindings")
        .and_then(PracticalJsonValue::as_array)
        .expect("shape checked");
    if bindings.len() > SEMANTIC_BINDING_COUNT_MAX {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    let mut previous_source_type_id: Option<&str> = None;
    for binding in bindings {
        validate_semantic_binding_entry(context, captures, binding)?;
        let source_type_id = binding
            .get("source_type_id")
            .and_then(PracticalJsonValue::as_str)
            .expect("entry checked");
        if previous_source_type_id.is_some_and(|previous| previous >= source_type_id) {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Order,
                PracticalArtifactErrorCode::DuplicateMember,
            ));
        }
        previous_source_type_id = Some(source_type_id);
    }
    Ok(())
}

fn validate_semantic_binding_entry(
    _context: Option<&PracticalArtifactContext>,
    captures: Option<&CapturedInputSet>,
    value: &PracticalJsonValue,
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    require_exact_fields(kind, value, SEMANTIC_BINDING_FIELDS)?;
    let role_name = string_field(kind, value, "role")?;
    let role = semantic_binding_role(role_name).ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    if string_field(kind, value, "schema")? != SEMANTIC_BINDING_SCHEMA
        || !valid_source_declaration_id(string_field(kind, value, "source_type_id")?)
        || !string_field(kind, value, "source_content_sha256").is_ok_and(valid_sha256)
        || string_field(kind, value, "default_arm")? != role.default_arm
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    validate_binding_members(value.get("member_map"), role.members)?;
    validate_binding_arm_mappings(value.get("tag_arms"), role.arms)?;
    validate_binding_bounds(value.get("bounds"), role.bound)?;
    validate_binding_operation_map(value.get("operation_map"), role.operations)?;
    let arguments = practical_string_array_for(kind, value.get("inferred_argument_ids"))?;
    if arguments.len() != role.argument_count
        || arguments
            .iter()
            .any(|argument| !valid_concrete_type_id(argument))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    if let Some(captures) = captures {
        let source_hash = string_field(kind, value, "source_content_sha256")?;
        if !captures.contains_source_raw_sha256(source_hash) {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Source,
                PracticalArtifactErrorCode::SourceInventory,
            ));
        }
    }
    require_sorted_object_hash_field(kind, value, "binding_sha256", SEMANTIC_BINDING_HASH_DOMAIN)?;
    Ok(())
}

fn validate_binding_members(
    value: Option<&PracticalJsonValue>,
    expected: &[&str],
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let value = value.ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    require_exact_fields(kind, value, expected)?;
    let member_ids = expected
        .iter()
        .map(|name| string_field(kind, value, name))
        .collect::<Result<Vec<_>, _>>()?;
    if member_ids
        .iter()
        .any(|member| !valid_source_member_id(member))
        || member_ids.iter().copied().collect::<BTreeSet<_>>().len() != member_ids.len()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    Ok(())
}

fn validate_binding_arm_mappings(
    value: Option<&PracticalJsonValue>,
    expected: &[&str],
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let value = value.ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    require_exact_fields(kind, value, expected)?;
    let values = expected
        .iter()
        .map(|arm| string_field(kind, value, arm))
        .collect::<Result<Vec<_>, _>>()?;
    if values
        .iter()
        .any(|value| !valid_canonical_enum_carrier(value))
        || values.iter().copied().collect::<BTreeSet<_>>().len() != values.len()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    Ok(())
}

fn validate_binding_bounds(
    value: Option<&PracticalJsonValue>,
    expected: Option<(&str, u64)>,
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let value = value.ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        )
    })?;
    match expected {
        Some((name, maximum)) => {
            require_exact_fields(kind, value, &[name])?;
            if value.get(name).and_then(PracticalJsonValue::as_u64) != Some(maximum) {
                return Err(failure(
                    kind,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                ));
            }
        }
        None => require_exact_fields(kind, value, NO_NAMES)?,
    }
    Ok(())
}

fn validate_binding_operation_map(
    value: Option<&PracticalJsonValue>,
    inventory: &[&str],
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SemanticBindings;
    let entries = value
        .and_then(PracticalJsonValue::as_object)
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            )
        })?;
    let mut previous_index = None;
    for (name, value) in entries {
        let index = inventory
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Shape,
                    PracticalArtifactErrorCode::Shape,
                )
            })?;
        if previous_index.is_some_and(|previous| previous >= index) {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Order,
                PracticalArtifactErrorCode::FieldOrder,
            ));
        }
        if value
            .as_str()
            .is_none_or(|member| !valid_source_declaration_id(member))
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            ));
        }
        previous_index = Some(index);
    }
    Ok(())
}

pub fn validate_selection_document(
    context: &PracticalArtifactContext,
    transport: &[u8],
) -> Result<(), PracticalArtifactError> {
    const FIELDS: &[&str] = &[
        "schema",
        "compilation_id",
        "source_paths",
        "selected_root_ids",
        "sidecar_paths",
        "selection_sha256",
    ];
    let kind = PracticalArtifactKind::Selection;
    let value = parse_canonical_practical_json(kind, transport)?;
    require_exact_fields(kind, &value, FIELDS)?;
    if string_field(kind, &value, "schema")? != "mpk.selection.csharp_members.v1"
        || string_field(kind, &value, "compilation_id")? != context.compilation_id
        || string_field(kind, &value, "selection_sha256")? != context.selection_sha256
        || practical_string_array(value.get("source_paths"))? != context.source_paths
        || practical_string_array(value.get("selected_root_ids"))? != context.selected_root_ids
        || practical_string_array(value.get("sidecar_paths"))? != context.sidecar_paths
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ));
    }
    Ok(())
}

pub fn validate_contract_artifact(
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    kind: PracticalArtifactKind,
    transport: &[u8],
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    if matches!(
        kind,
        PracticalArtifactKind::BoundaryInput | PracticalArtifactKind::BoundaryOutput
    ) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryBytes,
        ));
    }
    let (schema, fields, hash_field, domain) = contract_root_spec(kind).ok_or_else(|| {
        failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Schema,
        )
    })?;
    let value = parse_canonical_practical_json(kind, transport)?;
    require_exact_fields(kind, &value, fields)?;
    if string_field(kind, &value, "schema")? != schema {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Schema,
        ));
    }
    validate_contract_root_types(kind, &value)?;
    require_context_and_compilation(kind, context, &value)?;
    if matches!(
        kind,
        PracticalArtifactKind::TypeContract | PracticalArtifactKind::MethodContract
    ) {
        let source_hash = string_field(kind, &value, "source_content_sha256")?;
        if !captures.contains_source_raw_sha256(source_hash) {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Source,
                PracticalArtifactErrorCode::SourceInventory,
            ));
        }
    }
    let hash = require_hash_field(kind, &value, hash_field, domain)?;
    Ok(ValidatedPracticalArtifact {
        kind,
        schema: schema.to_owned(),
        hash,
        canonical_bytes: transport.to_vec(),
        value,
        linkage_key: context.linkage_key.clone(),
        input_set_sha256: Some(captures.snapshot_sha256().to_owned()),
    })
}

fn contract_root_spec(
    kind: PracticalArtifactKind,
) -> Option<(
    &'static str,
    &'static [&'static str],
    &'static str,
    HashDomain,
)> {
    match kind {
        PracticalArtifactKind::TypeContract => Some((
            TYPE_CONTRACT_SCHEMA,
            TYPE_CONTRACT_FIELDS,
            "contract_sha256",
            TYPE_CONTRACT_HASH_DOMAIN,
        )),
        PracticalArtifactKind::MethodContract => Some((
            METHOD_CONTRACT_SCHEMA,
            METHOD_CONTRACT_FIELDS,
            "contract_sha256",
            METHOD_CONTRACT_HASH_DOMAIN,
        )),
        PracticalArtifactKind::BoundaryContract => Some((
            BOUNDARY_CONTRACT_SCHEMA,
            BOUNDARY_CONTRACT_FIELDS,
            "contract_sha256",
            BOUNDARY_CONTRACT_HASH_DOMAIN,
        )),
        PracticalArtifactKind::BoundaryInput => Some((
            BOUNDARY_INPUT_SCHEMA,
            BOUNDARY_INPUT_FIELDS,
            "capture_sha256",
            BOUNDARY_INPUT_HASH_DOMAIN,
        )),
        PracticalArtifactKind::BoundaryOutput => Some((
            BOUNDARY_OUTPUT_SCHEMA,
            BOUNDARY_OUTPUT_FIELDS,
            "capture_sha256",
            BOUNDARY_OUTPUT_HASH_DOMAIN,
        )),
        PracticalArtifactKind::TransitionContract => Some((
            TRANSITION_CONTRACT_SCHEMA,
            TRANSITION_CONTRACT_FIELDS,
            "contract_sha256",
            TRANSITION_CONTRACT_HASH_DOMAIN,
        )),
        _ => None,
    }
}

fn validate_contract_root_types(
    kind: PracticalArtifactKind,
    value: &PracticalJsonValue,
) -> Result<(), PracticalArtifactError> {
    let object = |name| matches!(value.get(name), Some(PracticalJsonValue::Object(_)));
    let boolean = |name| matches!(value.get(name), Some(PracticalJsonValue::Bool(_)));
    let source_member_array = |name| {
        value
            .get(name)
            .and_then(PracticalJsonValue::as_array)
            .is_some_and(|values| {
                let identifiers = values
                    .iter()
                    .map(PracticalJsonValue::as_str)
                    .collect::<Option<Vec<_>>>();
                identifiers.is_some_and(|identifiers| {
                    identifiers
                        .iter()
                        .all(|value| valid_source_member_id(value))
                        && identifiers.iter().copied().collect::<BTreeSet<_>>().len()
                            == identifiers.len()
                })
            })
    };
    let object_array = |name| {
        value
            .get(name)
            .and_then(PracticalJsonValue::as_array)
            .is_some_and(|values| {
                values
                    .iter()
                    .all(|value| matches!(value, PracticalJsonValue::Object(_)))
            })
    };
    let strings_valid = |fields: &[&str]| {
        fields.iter().all(|field| {
            value
                .get(field)
                .and_then(PracticalJsonValue::as_str)
                .is_some_and(|item| !item.is_empty())
        })
    };
    let valid = match kind {
        PracticalArtifactKind::TypeContract => {
            object("semantic_context")
                && strings_valid(&[
                    "compilation_id",
                    "source_type_id",
                    "source_content_sha256",
                    "structural_equality",
                    "structural_order",
                    "contract_sha256",
                ])
                && string_field(kind, value, "compilation_id").is_ok_and(valid_compilation_id)
                && string_field(kind, value, "source_type_id")
                    .is_ok_and(valid_source_declaration_id)
                && source_member_array("ordered_member_ids")
                && object("recursive_default")
                && boolean("default_eligible")
                && source_member_array("required_member_ids")
                && source_member_array("init_member_ids")
                && matches!(
                    value.get("construction_invariant"),
                    Some(PracticalJsonValue::Null | PracticalJsonValue::Object(_))
                )
                && object_array("invariants")
                && matches!(
                    value
                        .get("structural_equality")
                        .and_then(PracticalJsonValue::as_str),
                    Some("ineligible" | "field_complete")
                )
                && matches!(
                    value
                        .get("structural_order")
                        .and_then(PracticalJsonValue::as_str),
                    Some("ineligible" | "canonical_field_order")
                )
                && string_field(kind, value, "source_content_sha256").is_ok_and(valid_sha256)
        }
        PracticalArtifactKind::MethodContract => {
            object("semantic_context")
                && strings_valid(&[
                    "compilation_id",
                    "callable_id",
                    "source_content_sha256",
                    "termination",
                    "contract_sha256",
                ])
                && string_field(kind, value, "compilation_id").is_ok_and(valid_compilation_id)
                && string_field(kind, value, "callable_id").is_ok_and(valid_source_declaration_id)
                && matches!(
                    value
                        .get("termination")
                        .and_then(PracticalJsonValue::as_str),
                    Some("partial" | "total")
                )
                && object_array("requires")
                && object_array("ensures")
                && object_array("exceptional_cases")
                && value
                    .get("modifies")
                    .and_then(PracticalJsonValue::as_array)
                    .is_some_and(|values| values.is_empty())
                && object_array("loops")
                && string_field(kind, value, "source_content_sha256").is_ok_and(valid_sha256)
        }
        PracticalArtifactKind::BoundaryContract => {
            object("semantic_context")
                && strings_valid(&[
                    "compilation_id",
                    "boundary_id",
                    "selected_callable_id",
                    "canonical_json_profile",
                    "parse_format_profile",
                    "contract_sha256",
                ])
                && string_field(kind, value, "compilation_id").is_ok_and(valid_compilation_id)
                && string_field(kind, value, "boundary_id").is_ok_and(valid_canonical_id)
                && string_field(kind, value, "selected_callable_id")
                    .is_ok_and(valid_source_declaration_id)
                && object_array("input_fields")
                && object_array("output_fields")
                && value
                    .get("canonical_json_profile")
                    .and_then(PracticalJsonValue::as_str)
                    == Some("mpk.csharp.canonical_json.v1")
                && value
                    .get("parse_format_profile")
                    .and_then(PracticalJsonValue::as_str)
                    == Some("mpk.csharp.parse_format.v1")
                && valid_boundary_evidence(value.get("evidence_linkage"))
        }
        PracticalArtifactKind::BoundaryInput => {
            object("semantic_context")
                && strings_valid(&[
                    "boundary_contract_sha256",
                    "canonical_document_utf8_sha256",
                    "canonical_value_sha256",
                    "capture_sha256",
                ])
                && object("raw_input")
                && object("canonical_value")
        }
        PracticalArtifactKind::BoundaryOutput => {
            object("semantic_context")
                && strings_valid(&[
                    "boundary_contract_sha256",
                    "source_value_sha256",
                    "canonical_document_utf8_sha256",
                    "reparsed_value_sha256",
                    "capture_sha256",
                ])
                && object("source_value")
                && object("reparsed_value")
        }
        PracticalArtifactKind::TransitionContract => {
            object("semantic_context")
                && strings_valid(&[
                    "compilation_id",
                    "transition_id",
                    "selected_callable_id",
                    "state_type_id",
                    "command_type_id",
                    "context_type_id",
                    "apply_result_binding_id",
                    "transition_binding_id",
                    "domain_error_binding_id",
                    "contract_sha256",
                ])
                && string_field(kind, value, "compilation_id").is_ok_and(valid_compilation_id)
                && [
                    "transition_id",
                    "apply_result_binding_id",
                    "transition_binding_id",
                    "domain_error_binding_id",
                ]
                .iter()
                .all(|field| string_field(kind, value, field).is_ok_and(valid_canonical_id))
                && [
                    "selected_callable_id",
                    "state_type_id",
                    "command_type_id",
                    "context_type_id",
                ]
                .iter()
                .all(|field| {
                    string_field(kind, value, field).is_ok_and(valid_source_declaration_id)
                })
                && object("state_invariant")
                && object("version_rule")
                && object("idempotency")
                && object_array("accepted_commands")
                && object("event_relation")
                && object("response_relation")
                && object_array("errors")
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(failure(
            kind,
            PracticalArtifactPhase::Shape,
            PracticalArtifactErrorCode::Shape,
        ))
    }
}

fn valid_boundary_evidence(value: Option<&PracticalJsonValue>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let Ok(()) = require_exact_fields(
        PracticalArtifactKind::BoundaryContract,
        value,
        &[
            "raw_input_domain",
            "canonical_value_domain",
            "canonical_output_domain",
            "reparse_equality",
        ],
    ) else {
        return false;
    };
    value
        .get("raw_input_domain")
        .and_then(PracticalJsonValue::as_str)
        == Some("MPK-CSHARP-BOUNDARY-INPUT-1.0")
        && value
            .get("canonical_value_domain")
            .and_then(PracticalJsonValue::as_str)
            == Some("MPK-CSHARP-CANONICAL-VALUE-1.0")
        && value
            .get("canonical_output_domain")
            .and_then(PracticalJsonValue::as_str)
            == Some("MPK-CSHARP-BOUNDARY-OUTPUT-1.0")
        && value
            .get("reparse_equality")
            .and_then(PracticalJsonValue::as_str)
            == Some("typed_field_complete")
}

fn require_context_and_compilation(
    kind: PracticalArtifactKind,
    context: &PracticalArtifactContext,
    value: &PracticalJsonValue,
) -> Result<(), PracticalArtifactError> {
    if value.get("semantic_context") != Some(&context.semantic_context) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Context,
            PracticalArtifactErrorCode::Context,
        ));
    }
    if string_field(kind, value, "compilation_id")? != context.compilation_id {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Context,
            PracticalArtifactErrorCode::Compilation,
        ));
    }
    Ok(())
}

fn practical_string_array(
    value: Option<&PracticalJsonValue>,
) -> Result<Vec<String>, PracticalArtifactError> {
    practical_string_array_for(PracticalArtifactKind::Selection, value)
}

fn practical_string_array_for(
    kind: PracticalArtifactKind,
    value: Option<&PracticalJsonValue>,
) -> Result<Vec<String>, PracticalArtifactError> {
    value
        .and_then(PracticalJsonValue::as_array)
        .and_then(|values| {
            values
                .iter()
                .map(|value| value.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
        })
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Shape,
                PracticalArtifactErrorCode::Shape,
            )
        })
}

fn valid_canonical_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 1_024 || !value.is_ascii() {
        return false;
    }
    let mut previous_separator = false;
    for byte in value.bytes() {
        let separator = matches!(byte, b'.' | b'_' | b'-');
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && !separator {
            return false;
        }
        if separator && previous_separator {
            return false;
        }
        previous_separator = separator;
    }
    value
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn valid_compilation_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || !value.is_ascii() {
        return false;
    }
    let bytes = value.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut separator = false;
    for byte in &bytes[1..] {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            separator = false;
        } else if matches!(byte, b'.' | b'_' | b'-') && !separator {
            separator = true;
        } else {
            return false;
        }
    }
    !separator
}

fn valid_provenance_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=256).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes[1..].iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b':' | b'-')
        })
}

fn valid_source_declaration_id(value: &str) -> bool {
    value
        .strip_prefix("mpk.csharp.source.")
        .is_some_and(valid_sha256)
}

fn valid_source_member_id(value: &str) -> bool {
    value
        .strip_prefix("mpk.csharp.member.")
        .is_some_and(valid_sha256)
}

fn valid_concrete_type_id(value: &str) -> bool {
    const PRIMITIVES: &[&str] = &[
        "bool",
        "i8",
        "u8",
        "i16",
        "u16",
        "i32",
        "u32",
        "i64",
        "u64",
        "char",
        "f32",
        "f64",
        "decimal",
        "string",
        "date",
        "time",
        "duration",
        "guid",
        "day_of_week",
        "unit",
        "parse_error",
        "instant",
        "exception",
    ];
    value
        .strip_prefix("mpk.csharp.instance.")
        .is_some_and(valid_sha256)
        || valid_source_declaration_id(value)
        || value
            .strip_prefix("mpk.csharp.value.")
            .and_then(|value| value.strip_suffix(".v1"))
            .is_some_and(|primitive| PRIMITIVES.contains(&primitive))
}

fn valid_source_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value.len() <= 512
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_source_namespace(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(valid_source_identifier)
}

fn string_values(values: &[String]) -> PracticalJsonValue {
    PracticalJsonValue::Array(values.iter().map(PracticalJsonValue::string).collect())
}

pub fn bind_closed_instances(
    context: &PracticalArtifactContext,
    foundation: &ValidatedFoundationBundle,
    captures: &CapturedInputSet,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) -> Result<ArtifactRef, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceArtifacts;
    if closed.value().get("schema").and_then(Value::as_str) != Some(CLOSED_INSTANCES_SCHEMA)
        || closed
            .value()
            .get("semantic_profile")
            .and_then(Value::as_str)
            != Some(CSHARP_PRACTICAL_PROFILE)
        || closed.value().get("foundation_id").and_then(Value::as_str)
            != Some(FOUNDATION_DESCRIPTOR_ID)
        || closed
            .value()
            .get("foundation_sha256")
            .and_then(Value::as_str)
            != Some(foundation.content_sha256())
        || !valid_sha256(closed.closed_set_sha256())
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        ));
    }
    validate_foundation_context_linkage(foundation, &context.typed_semantic_context).map_err(
        |_| {
            failure(
                kind,
                PracticalArtifactPhase::Foundation,
                PracticalArtifactErrorCode::Foundation,
            )
        },
    )?;
    validate_closed_instance_set(foundation, roots, closed.canonical_json()).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        )
    })?;
    validate_closed_source_capture(captures, roots)?;
    validate_closed_instance_provenance(roots, closed)?;
    Ok(ArtifactRef {
        schema: CLOSED_INSTANCES_SCHEMA.to_owned(),
        sha256: closed.closed_set_sha256().to_owned(),
        canonical_bytes: u64::try_from(closed.canonical_json().len())
            .expect("bounded closed-instance transport fits u64"),
        linkage_key: context.linkage_key.clone(),
        source_value: None,
        input_set_sha256: Some(captures.snapshot_sha256().to_owned()),
    })
}

fn validate_closed_source_capture(
    captures: &CapturedInputSet,
    roots: &ValidatedClosedRootSet,
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceArtifacts;
    let value: Value = serde_json::from_slice(roots.canonical_json()).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        )
    })?;
    let source_types = value
        .get("source_types")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Foundation,
                PracticalArtifactErrorCode::Foundation,
            )
        })?;
    if source_types.values().any(|source| {
        source
            .get("source_sha256")
            .and_then(Value::as_str)
            .is_none_or(|sha256| !captures.contains_source_raw_sha256(sha256))
    }) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Source,
            PracticalArtifactErrorCode::SourceInventory,
        ));
    }
    Ok(())
}

fn validate_closed_instance_provenance(
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
) -> Result<(), PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceArtifacts;
    let root_value: Value = serde_json::from_slice(roots.canonical_json()).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        )
    })?;
    let root_ids = root_value
        .get("roots")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            failure(
                kind,
                PracticalArtifactPhase::Foundation,
                PracticalArtifactErrorCode::Foundation,
            )
        })?
        .iter()
        .filter_map(|root| root.get("provenance_id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    for entry in closed.entries() {
        let provenance = entry
            .get("provenance_ids")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Foundation,
                    PracticalArtifactErrorCode::Foundation,
                )
            })?;
        if provenance.is_empty()
            || provenance
                .iter()
                .any(|id| id.as_str().is_none_or(|id| !root_ids.contains(id)))
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Foundation,
                PracticalArtifactErrorCode::Foundation,
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ConcreteOperationTables {
    operations: ValidatedPracticalArtifact,
    required_checks: ValidatedPracticalArtifact,
}

impl ConcreteOperationTables {
    pub fn operations(&self) -> &ValidatedPracticalArtifact {
        &self.operations
    }

    pub fn required_checks(&self) -> &ValidatedPracticalArtifact {
        &self.required_checks
    }
}

pub fn build_concrete_operation_tables(
    context: &PracticalArtifactContext,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    closed_ref: &ArtifactRef,
    mut signatures: Vec<ClosedOperationSignature>,
) -> Result<ConcreteOperationTables, PracticalArtifactError> {
    let kind = PracticalArtifactKind::Operations;
    require_linkage(context, kind, closed_ref)?;
    if closed_ref.schema != CLOSED_INSTANCES_SCHEMA
        || closed_ref.sha256 != closed.closed_set_sha256()
        || signatures.len() > 4_096
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        ));
    }
    signatures.sort_by(|left, right| left.id.cmp(&right.id));
    if signatures.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::DuplicateMember,
        ));
    }
    for signature in &signatures {
        validate_closed_operation_signature(roots, closed, signature).map_err(|_| {
            failure(
                kind,
                PracticalArtifactPhase::Operation,
                PracticalArtifactErrorCode::Operation,
            )
        })?;
    }
    let expected_foundation = closed
        .entries()
        .iter()
        .flat_map(|entry| {
            entry
                .get("operation_definitions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|operation| operation.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let actual_foundation = signatures
        .iter()
        .filter(|signature| signature.tag.as_str() == "foundation")
        .map(|signature| signature.id.as_str())
        .collect::<BTreeSet<_>>();
    if actual_foundation != expected_foundation {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Operation,
            PracticalArtifactErrorCode::MissingMember,
        ));
    }

    let mut checks = BTreeMap::<String, RequiredCheck>::new();
    for signature in &signatures {
        for check in &signature.ordered_checks {
            if let Some(previous) = checks.get(&check.id) {
                if previous != check {
                    return Err(failure(
                        PracticalArtifactKind::RequiredChecks,
                        PracticalArtifactPhase::Operation,
                        PracticalArtifactErrorCode::DuplicateMember,
                    ));
                }
            } else {
                checks.insert(check.id.clone(), check.clone());
            }
        }
    }
    let check_values = checks
        .values()
        .map(|check| {
            PracticalJsonValue::object(vec![
                ("id", PracticalJsonValue::string(&check.id)),
                ("tag", PracticalJsonValue::string(check.tag.as_str())),
                (
                    "failure_type_id",
                    check
                        .failure_type_id
                        .as_ref()
                        .map_or(PracticalJsonValue::Null, PracticalJsonValue::string),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let mut required_checks = finalized_artifact(
        context,
        PracticalArtifactKind::RequiredChecks,
        CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
        "required_checks_sha256",
        REQUIRED_CHECKS_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            ("closed_instances", closed_ref.value()),
            ("checks", PracticalJsonValue::Array(check_values)),
        ],
    )?;
    required_checks.input_set_sha256 = closed_ref.input_set_sha256.clone();
    let operation_values = signatures
        .iter()
        .map(|signature| {
            PracticalJsonValue::object(vec![
                ("id", PracticalJsonValue::string(&signature.id)),
                ("tag", PracticalJsonValue::string(signature.tag.as_str())),
                (
                    "argument_type_ids",
                    string_values(&signature.argument_type_ids),
                ),
                (
                    "normal_result_type_id",
                    PracticalJsonValue::string(&signature.normal_result_type_id),
                ),
                (
                    "ordered_check_ids",
                    PracticalJsonValue::Array(
                        signature
                            .ordered_checks
                            .iter()
                            .map(|check| PracticalJsonValue::string(&check.id))
                            .collect(),
                    ),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let mut operations = finalized_artifact(
        context,
        PracticalArtifactKind::Operations,
        CSHARP_PRACTICAL_OPERATIONS_SCHEMA,
        "operations_sha256",
        OPERATIONS_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(CSHARP_PRACTICAL_OPERATIONS_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            ("closed_instances", closed_ref.value()),
            ("required_checks", required_checks.artifact_ref().value()),
            ("operations", PracticalJsonValue::Array(operation_values)),
        ],
    )?;
    operations.input_set_sha256 = closed_ref.input_set_sha256.clone();
    Ok(ConcreteOperationTables {
        operations,
        required_checks,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceDeclarationKind {
    Type,
    Constructor,
    Method,
}

impl SourceDeclarationKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Constructor => "constructor",
            Self::Method => "method",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDeclarationIdentity {
    pub namespace: String,
    pub kind: SourceDeclarationKind,
    pub containing_source_type_id: Option<String>,
    pub source_name: String,
    pub parameter_type_ids: Vec<String>,
    pub result_type_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceStoredMemberStorage {
    ReadonlyField,
    GetAuto,
    InitAuto,
}

impl SourceStoredMemberStorage {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ReadonlyField => "readonly_field",
            Self::GetAuto => "get_auto",
            Self::InitAuto => "init_auto",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceStoredMemberIdentity {
    pub owner_source_type_id: String,
    pub source_name: String,
    pub closed_type: Value,
    pub storage: SourceStoredMemberStorage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceMapIdentity {
    Declaration(SourceDeclarationIdentity),
    StoredMember(SourceStoredMemberIdentity),
}

pub fn canonical_source_declaration_id(
    identity: &SourceDeclarationIdentity,
) -> Result<String, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceMap;
    let valid_shape = match &identity.kind {
        SourceDeclarationKind::Type => {
            identity.containing_source_type_id.is_none()
                && identity.parameter_type_ids.is_empty()
                && identity.result_type_id.is_none()
        }
        SourceDeclarationKind::Constructor => identity
            .containing_source_type_id
            .as_ref()
            .zip(identity.result_type_id.as_ref())
            .is_some_and(|(owner, result)| owner == result),
        SourceDeclarationKind::Method => {
            identity.containing_source_type_id.is_some() && identity.result_type_id.is_some()
        }
    };
    if !valid_source_namespace(&identity.namespace)
        || !valid_source_identifier(&identity.source_name)
        || identity
            .containing_source_type_id
            .as_deref()
            .is_some_and(|value| !valid_source_declaration_id(value))
        || identity
            .result_type_id
            .as_deref()
            .is_some_and(|value| !valid_concrete_type_id(value))
        || identity
            .parameter_type_ids
            .iter()
            .any(|value| !valid_concrete_type_id(value))
        || !valid_shape
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    let exact_identity = serde_json::json!({
        "kind": identity.kind.as_str(),
        "namespace": identity.namespace,
        "owner": identity.containing_source_type_id.as_deref().unwrap_or(""),
        "name": identity.source_name,
        "parameter_type_ids": identity.parameter_type_ids,
        "result_type_id": identity.result_type_id.as_deref().unwrap_or(""),
    });
    csharp_practical_declaration_id(&exact_identity).map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Shape,
        )
    })
}

pub fn canonical_source_stored_member_id(
    identity: &SourceStoredMemberIdentity,
) -> Result<String, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceMap;
    if !valid_source_declaration_id(&identity.owner_source_type_id)
        || !valid_source_identifier(&identity.source_name)
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Shape,
        ));
    }
    csharp_practical_stored_member_id(
        &identity.owner_source_type_id,
        &identity.source_name,
        &identity.closed_type,
        identity.storage.as_str(),
    )
    .map_err(|_| {
        failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Shape,
        )
    })
}

fn canonical_source_map_identity_id(
    identity: &SourceMapIdentity,
) -> Result<String, PracticalArtifactError> {
    match identity {
        SourceMapIdentity::Declaration(identity) => canonical_source_declaration_id(identity),
        SourceMapIdentity::StoredMember(identity) => canonical_source_stored_member_id(identity),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMapDeclaration {
    pub declaration_id: String,
    pub identity: SourceMapIdentity,
    pub provenance_id: String,
    pub source_path: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub artifact_node_ids: Vec<String>,
}

pub fn build_practical_source_map(
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    vir: &ArtifactRef,
    mut declarations: Vec<SourceMapDeclaration>,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceMap;
    require_linkage(context, kind, vir)?;
    if vir.schema != SUCCESSOR_VIR_SCHEMA {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Identity,
            PracticalArtifactErrorCode::Schema,
        ));
    }
    declarations.sort_by(|left, right| left.declaration_id.cmp(&right.declaration_id));
    if declarations
        .windows(2)
        .any(|pair| pair[0].declaration_id == pair[1].declaration_id)
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::DuplicateMember,
        ));
    }
    let declaration_ids = declarations
        .iter()
        .map(|declaration| declaration.declaration_id.as_str())
        .collect::<BTreeSet<_>>();
    if !context
        .selected_root_ids
        .iter()
        .all(|root| declaration_ids.contains(root.as_str()))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Source,
            PracticalArtifactErrorCode::MissingMember,
        ));
    }
    let mut provenance_ids = BTreeSet::new();
    let mut artifact_nodes = BTreeSet::new();
    let mut entries = Vec::with_capacity(declarations.len());
    for declaration in &mut declarations {
        if canonical_source_map_identity_id(&declaration.identity)? != declaration.declaration_id {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Linkage,
            ));
        }
        declaration.artifact_node_ids.sort();
        if !valid_provenance_id(&declaration.provenance_id)
            || declaration.artifact_node_ids.is_empty()
            || declaration
                .artifact_node_ids
                .iter()
                .any(|node| !valid_canonical_id(node))
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Shape,
            ));
        }
        if !provenance_ids.insert(declaration.provenance_id.as_str())
            || declaration
                .artifact_node_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || declaration
                .artifact_node_ids
                .iter()
                .any(|node| !artifact_nodes.insert(node.as_str()))
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Order,
                PracticalArtifactErrorCode::DuplicateMember,
            ));
        }
        let source = captures
            .entry(&declaration.source_path)
            .filter(|entry| entry.kind == OriginalInputKind::Source)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Source,
                    PracticalArtifactErrorCode::SourceInventory,
                )
            })?;
        let ordinal = captures
            .source_ordinal(&declaration.source_path)
            .ok_or_else(|| {
                failure(
                    kind,
                    PracticalArtifactPhase::Source,
                    PracticalArtifactErrorCode::SourceInventory,
                )
            })?;
        let location = SourceLocation {
            source_file_ordinal: ordinal,
            start_byte: declaration.start_byte,
            end_byte: declaration.end_byte,
        };
        if declaration.start_byte >= declaration.end_byte
            || usize::try_from(declaration.end_byte)
                .ok()
                .is_none_or(|end| end > source.bytes.len())
        {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Source,
                PracticalArtifactErrorCode::SourceSpan,
            ));
        }
        let byte_length = declaration.end_byte - declaration.start_byte;
        let provenance_preimage = PracticalJsonValue::object(vec![
            (
                "declaration_id",
                PracticalJsonValue::string(&declaration.declaration_id),
            ),
            (
                "source_path",
                PracticalJsonValue::string(&declaration.source_path),
            ),
            (
                "source_content_sha256",
                PracticalJsonValue::string(source.raw_sha256()),
            ),
            (
                "start_byte",
                PracticalJsonValue::U64(u64::from(declaration.start_byte)),
            ),
            (
                "byte_length",
                PracticalJsonValue::U64(u64::from(byte_length)),
            ),
        ]);
        let provenance_sha256 = hash_complete_with_sorted_objects(
            DECLARATION_PROVENANCE_HASH_DOMAIN,
            &provenance_preimage,
            kind,
        )?;
        entries.push(PracticalJsonValue::object(vec![
            (
                "declaration_id",
                PracticalJsonValue::string(&declaration.declaration_id),
            ),
            (
                "provenance_id",
                PracticalJsonValue::string(&declaration.provenance_id),
            ),
            ("source_location", location.value()),
            (
                "source_content_sha256",
                PracticalJsonValue::string(source.raw_sha256()),
            ),
            (
                "provenance_sha256",
                PracticalJsonValue::string(provenance_sha256),
            ),
            (
                "artifact_node_ids",
                string_values(&declaration.artifact_node_ids),
            ),
        ]));
    }
    finalized_artifact(
        context,
        kind,
        SOURCE_MAP_SCHEMA,
        "source_map_sha256",
        SOURCE_MAP_HASH_DOMAIN,
        vec![
            ("schema", PracticalJsonValue::string(SOURCE_MAP_SCHEMA)),
            ("semantic_context", context.semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            (
                "selection_sha256",
                PracticalJsonValue::string(context.selection_sha256()),
            ),
            (
                "source_snapshot_sha256",
                PracticalJsonValue::string(captures.snapshot_sha256()),
            ),
            ("vir", vir.value()),
            ("entries", PracticalJsonValue::Array(entries)),
        ],
    )
    .map(|artifact| artifact.with_input_set(captures))
}

pub fn validate_expected_artifact(
    expected: &ValidatedPracticalArtifact,
    transport: &[u8],
) -> Result<(), PracticalArtifactError> {
    let parsed = parse_canonical_practical_json(expected.kind, transport)?;
    if parsed != expected.value {
        return Err(failure(
            expected.kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct FrontendManifestArtifacts {
    pub type_contracts: Vec<ArtifactRef>,
    pub method_contracts: Vec<ArtifactRef>,
    pub semantic_bindings: ArtifactRef,
    pub boundary_contracts: Vec<ArtifactRef>,
    pub boundary_inputs: Vec<ArtifactRef>,
    pub boundary_outputs: Vec<ArtifactRef>,
    pub transition_contracts: Vec<ArtifactRef>,
    pub closed_instances: ArtifactRef,
    pub operations: ArtifactRef,
    pub required_checks: ArtifactRef,
    pub vir: ArtifactRef,
    pub source_map: ArtifactRef,
}

pub fn build_frontend_source_manifest(
    context: &PracticalArtifactContext,
    foundation: &ValidatedFoundationBundle,
    captures: &CapturedInputSet,
    mut artifacts: FrontendManifestArtifacts,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::FrontendManifest;
    normalize_refs(
        context,
        kind,
        &mut artifacts.type_contracts,
        TYPE_CONTRACT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut artifacts.method_contracts,
        METHOD_CONTRACT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut artifacts.boundary_contracts,
        BOUNDARY_CONTRACT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut artifacts.boundary_inputs,
        BOUNDARY_INPUT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut artifacts.boundary_outputs,
        BOUNDARY_OUTPUT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut artifacts.transition_contracts,
        TRANSITION_CONTRACT_SCHEMA,
    )?;
    for (reference, schema) in [
        (&artifacts.semantic_bindings, SEMANTIC_BINDINGS_SCHEMA),
        (&artifacts.closed_instances, CLOSED_INSTANCES_SCHEMA),
        (&artifacts.operations, CSHARP_PRACTICAL_OPERATIONS_SCHEMA),
        (
            &artifacts.required_checks,
            CSHARP_PRACTICAL_REQUIRED_CHECKS_SCHEMA,
        ),
        (&artifacts.vir, SUCCESSOR_VIR_SCHEMA),
        (&artifacts.source_map, SOURCE_MAP_SCHEMA),
    ] {
        require_linkage(context, kind, reference)?;
        if reference.schema != schema {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Schema,
            ));
        }
    }
    for reference in artifacts
        .type_contracts
        .iter()
        .chain(&artifacts.method_contracts)
        .chain(&artifacts.boundary_contracts)
        .chain(&artifacts.transition_contracts)
        .chain([
            &artifacts.semantic_bindings,
            &artifacts.closed_instances,
            &artifacts.operations,
            &artifacts.required_checks,
            &artifacts.source_map,
        ])
    {
        require_input_set_linkage(kind, captures, reference)?;
    }
    require_embedded_artifact_ref(
        kind,
        &artifacts.required_checks,
        "closed_instances",
        &artifacts.closed_instances,
    )?;
    require_embedded_artifact_ref(
        kind,
        &artifacts.operations,
        "closed_instances",
        &artifacts.closed_instances,
    )?;
    require_embedded_artifact_ref(
        kind,
        &artifacts.operations,
        "required_checks",
        &artifacts.required_checks,
    )?;
    require_embedded_artifact_ref(kind, &artifacts.source_map, "vir", &artifacts.vir)?;
    require_embedded_string(
        kind,
        &artifacts.source_map,
        "selection_sha256",
        context.selection_sha256(),
    )?;
    require_embedded_string(
        kind,
        &artifacts.source_map,
        "source_snapshot_sha256",
        captures.snapshot_sha256(),
    )?;
    let boundary_contract_hashes = artifacts
        .boundary_contracts
        .iter()
        .map(|reference| reference.sha256.as_str())
        .collect::<BTreeSet<_>>();
    for reference in artifacts
        .boundary_inputs
        .iter()
        .chain(&artifacts.boundary_outputs)
    {
        let Some(contract_sha256) = embedded_string(reference, "boundary_contract_sha256") else {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Linkage,
                PracticalArtifactErrorCode::Linkage,
            ));
        };
        if !boundary_contract_hashes.contains(contract_sha256) {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Linkage,
                PracticalArtifactErrorCode::Linkage,
            ));
        }
    }
    if context
        .typed_semantic_context
        .foundation_descriptor()
        .content_sha256()
        != foundation.content_sha256()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        ));
    }
    let descriptor = context.typed_semantic_context.foundation_descriptor();
    finalized_artifact(
        context,
        kind,
        FRONTEND_SOURCE_MANIFEST_SCHEMA,
        "manifest_sha256",
        SOURCE_MANIFEST_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(FRONTEND_SOURCE_MANIFEST_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            (
                "compilation_id",
                PracticalJsonValue::string(context.compilation_id()),
            ),
            (
                "selection_sha256",
                PracticalJsonValue::string(context.selection_sha256()),
            ),
            ("inputs", PracticalJsonValue::Array(captures.values())),
            (
                "input_set_sha256",
                PracticalJsonValue::string(captures.snapshot_sha256()),
            ),
            ("type_contracts", ref_values(&artifacts.type_contracts)),
            ("method_contracts", ref_values(&artifacts.method_contracts)),
            ("semantic_bindings", artifacts.semantic_bindings.value()),
            (
                "boundary_contracts",
                ref_values(&artifacts.boundary_contracts),
            ),
            ("boundary_inputs", ref_values(&artifacts.boundary_inputs)),
            ("boundary_outputs", ref_values(&artifacts.boundary_outputs)),
            (
                "transition_contracts",
                ref_values(&artifacts.transition_contracts),
            ),
            (
                "foundation_descriptor",
                PracticalJsonValue::object(vec![
                    ("schema", PracticalJsonValue::string(descriptor.schema())),
                    ("id", PracticalJsonValue::string(descriptor.id())),
                    (
                        "content_sha256",
                        PracticalJsonValue::string(descriptor.content_sha256()),
                    ),
                ]),
            ),
            ("closed_instances", artifacts.closed_instances.value()),
            ("operations", artifacts.operations.value()),
            ("required_checks", artifacts.required_checks.value()),
            ("vir", artifacts.vir.value()),
            ("source_map", artifacts.source_map.value()),
        ],
    )
    .map(|artifact| artifact.with_input_set(captures))
}

pub fn build_certificate_source_manifest(
    context: &PracticalArtifactContext,
    frontend_manifest: &ArtifactRef,
    vc: &ArtifactRef,
    certificate_skeleton: &ArtifactRef,
    certificate_sha256: &str,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::CertificateManifest;
    for (reference, schema) in [
        (frontend_manifest, FRONTEND_SOURCE_MANIFEST_SCHEMA),
        (vc, SUCCESSOR_VC_SCHEMA),
        (certificate_skeleton, SUCCESSOR_CERTIFICATE_SKELETON_SCHEMA),
    ] {
        require_linkage(context, kind, reference)?;
        if reference.schema != schema {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Schema,
            ));
        }
    }
    if !valid_sha256(certificate_sha256) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Hash,
            PracticalArtifactErrorCode::Hash,
        ));
    }
    finalized_artifact(
        context,
        kind,
        CERTIFICATE_SOURCE_MANIFEST_SCHEMA,
        "manifest_sha256",
        SOURCE_MANIFEST_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(CERTIFICATE_SOURCE_MANIFEST_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            ("frontend_manifest", frontend_manifest.value()),
            ("vc", vc.value()),
            ("certificate_skeleton", certificate_skeleton.value()),
            (
                "certificate_sha256",
                PracticalJsonValue::string(certificate_sha256),
            ),
        ],
    )
}

pub struct FrontendSourceArtifactLinks<'a> {
    pub vir: &'a ArtifactRef,
    pub source_map: &'a ArtifactRef,
    pub source_manifest: &'a ValidatedPracticalArtifact,
    pub semantic_bindings: &'a ArtifactRef,
    pub closed_instances: &'a ArtifactRef,
    pub boundary_contracts: Vec<ArtifactRef>,
    pub transition_contracts: Vec<ArtifactRef>,
}

pub fn build_frontend_source_artifacts(
    context: &PracticalArtifactContext,
    foundation: &ValidatedFoundationBundle,
    mut links: FrontendSourceArtifactLinks<'_>,
) -> Result<ValidatedPracticalArtifact, PracticalArtifactError> {
    let kind = PracticalArtifactKind::SourceArtifacts;
    if links.source_manifest.kind != PracticalArtifactKind::FrontendManifest
        || links.source_manifest.linkage_key != context.linkage_key
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ));
    }
    let source_manifest_ref = links.source_manifest.artifact_ref();
    normalize_refs(
        context,
        kind,
        &mut links.boundary_contracts,
        BOUNDARY_CONTRACT_SCHEMA,
    )?;
    normalize_refs(
        context,
        kind,
        &mut links.transition_contracts,
        TRANSITION_CONTRACT_SCHEMA,
    )?;
    for (reference, schema) in [
        (links.vir, SUCCESSOR_VIR_SCHEMA),
        (links.source_map, SOURCE_MAP_SCHEMA),
        (&source_manifest_ref, FRONTEND_SOURCE_MANIFEST_SCHEMA),
        (links.semantic_bindings, SEMANTIC_BINDINGS_SCHEMA),
        (links.closed_instances, CLOSED_INSTANCES_SCHEMA),
    ] {
        require_linkage(context, kind, reference)?;
        if reference.schema != schema {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Schema,
            ));
        }
    }
    let expected_manifest_links = [
        ("vir", links.vir.value()),
        ("source_map", links.source_map.value()),
        ("semantic_bindings", links.semantic_bindings.value()),
        ("closed_instances", links.closed_instances.value()),
    ];
    if links.source_manifest.value.get("semantic_context") != Some(&context.semantic_context)
        || links
            .source_manifest
            .value
            .get("selection_sha256")
            .and_then(PracticalJsonValue::as_str)
            != Some(context.selection_sha256())
        || expected_manifest_links
            .iter()
            .any(|(field, expected)| links.source_manifest.value.get(field) != Some(expected))
        || links.source_manifest.value.get("boundary_contracts")
            != Some(&ref_values(&links.boundary_contracts))
        || links.source_manifest.value.get("transition_contracts")
            != Some(&ref_values(&links.transition_contracts))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ));
    }
    let descriptor = context.typed_semantic_context.foundation_descriptor();
    if descriptor.schema() != FOUNDATION_DESCRIPTOR_SCHEMA
        || descriptor.id() != FOUNDATION_DESCRIPTOR_ID
        || descriptor.content_sha256() != foundation.content_sha256()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Foundation,
            PracticalArtifactErrorCode::Foundation,
        ));
    }
    finalized_artifact(
        context,
        kind,
        SOURCE_ARTIFACTS_SCHEMA,
        "artifacts_sha256",
        SOURCE_ARTIFACTS_HASH_DOMAIN,
        vec![
            (
                "schema",
                PracticalJsonValue::string(SOURCE_ARTIFACTS_SCHEMA),
            ),
            ("semantic_context", context.semantic_context.clone()),
            (
                "selection_sha256",
                PracticalJsonValue::string(context.selection_sha256()),
            ),
            ("vir", links.vir.value()),
            ("source_map", links.source_map.value()),
            ("source_manifest", source_manifest_ref.value()),
            ("semantic_bindings", links.semantic_bindings.value()),
            ("closed_instances", links.closed_instances.value()),
            (
                "foundation_descriptor",
                PracticalJsonValue::object(vec![
                    ("schema", PracticalJsonValue::string(descriptor.schema())),
                    ("id", PracticalJsonValue::string(descriptor.id())),
                    (
                        "content_sha256",
                        PracticalJsonValue::string(descriptor.content_sha256()),
                    ),
                ]),
            ),
            ("boundary_contracts", ref_values(&links.boundary_contracts)),
            (
                "transition_contracts",
                ref_values(&links.transition_contracts),
            ),
        ],
    )
}

fn normalize_refs(
    context: &PracticalArtifactContext,
    kind: PracticalArtifactKind,
    references: &mut [ArtifactRef],
    schema: &str,
) -> Result<(), PracticalArtifactError> {
    for reference in references.iter() {
        require_linkage(context, kind, reference)?;
        if reference.schema != schema {
            return Err(failure(
                kind,
                PracticalArtifactPhase::Identity,
                PracticalArtifactErrorCode::Schema,
            ));
        }
    }
    references.sort_by(|left, right| left.sha256.cmp(&right.sha256));
    if references
        .windows(2)
        .any(|pair| pair[0].sha256 == pair[1].sha256)
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Order,
            PracticalArtifactErrorCode::DuplicateMember,
        ));
    }
    Ok(())
}

fn require_input_set_linkage(
    kind: PracticalArtifactKind,
    captures: &CapturedInputSet,
    reference: &ArtifactRef,
) -> Result<(), PracticalArtifactError> {
    if reference.input_set_sha256.as_deref() == Some(captures.snapshot_sha256()) {
        Ok(())
    } else {
        Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ))
    }
}

fn require_embedded_artifact_ref(
    kind: PracticalArtifactKind,
    container: &ArtifactRef,
    field: &str,
    expected: &ArtifactRef,
) -> Result<(), PracticalArtifactError> {
    let expected = expected.value();
    if container
        .source_value
        .as_deref()
        .and_then(|value| value.get(field))
        == Some(&expected)
    {
        Ok(())
    } else {
        Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ))
    }
}

fn embedded_string<'a>(reference: &'a ArtifactRef, field: &str) -> Option<&'a str> {
    reference.source_value.as_deref()?.get(field)?.as_str()
}

fn require_embedded_string(
    kind: PracticalArtifactKind,
    reference: &ArtifactRef,
    field: &str,
    expected: &str,
) -> Result<(), PracticalArtifactError> {
    if embedded_string(reference, field) == Some(expected) {
        Ok(())
    } else {
        Err(failure(
            kind,
            PracticalArtifactPhase::Linkage,
            PracticalArtifactErrorCode::Linkage,
        ))
    }
}

fn ref_values(references: &[ArtifactRef]) -> PracticalJsonValue {
    PracticalJsonValue::Array(references.iter().map(ArtifactRef::value).collect())
}

#[derive(Clone, Debug)]
pub struct BoundaryInputCapture {
    artifact: ValidatedPracticalArtifact,
    raw_bytes: Vec<u8>,
    canonical_document: Vec<u8>,
}

impl BoundaryInputCapture {
    pub fn artifact(&self) -> &ValidatedPracticalArtifact {
        &self.artifact
    }

    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw_bytes
    }

    pub fn canonical_document(&self) -> &[u8] {
        &self.canonical_document
    }
}

pub fn build_boundary_input_capture(
    context: &PracticalArtifactContext,
    boundary_contract: &ArtifactRef,
    provenance_id: &str,
    raw_bytes: &[u8],
    canonical_document: &[u8],
) -> Result<BoundaryInputCapture, PracticalArtifactError> {
    let kind = PracticalArtifactKind::BoundaryInput;
    require_linkage(context, kind, boundary_contract)?;
    if boundary_contract.schema != BOUNDARY_CONTRACT_SCHEMA
        || !valid_canonical_id(provenance_id)
        || u32::try_from(raw_bytes.len()).is_err()
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryBytes,
        ));
    }
    let canonical_value = parse_canonical_practical_json(kind, canonical_document)?;
    if !matches!(canonical_value, PracticalJsonValue::Object(_)) {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryValue,
        ));
    }
    let raw_digest = raw_sha256(raw_bytes);
    let canonical_value_sha256 =
        hash_complete(CANONICAL_VALUE_HASH_DOMAIN, &canonical_value, kind)?;
    let artifact = finalized_artifact(
        context,
        kind,
        BOUNDARY_INPUT_SCHEMA,
        "capture_sha256",
        BOUNDARY_INPUT_HASH_DOMAIN,
        vec![
            ("schema", PracticalJsonValue::string(BOUNDARY_INPUT_SCHEMA)),
            ("semantic_context", context.semantic_context.clone()),
            (
                "boundary_contract_sha256",
                PracticalJsonValue::string(boundary_contract.sha256()),
            ),
            (
                "raw_input",
                PracticalJsonValue::object(vec![
                    ("provenance_id", PracticalJsonValue::string(provenance_id)),
                    ("raw_sha256", PracticalJsonValue::string(&raw_digest)),
                    (
                        "size_bytes",
                        PracticalJsonValue::U64(
                            u64::try_from(raw_bytes.len()).expect("u32 checked length fits u64"),
                        ),
                    ),
                ]),
            ),
            (
                "canonical_document_utf8_sha256",
                PracticalJsonValue::string(raw_sha256(canonical_document)),
            ),
            ("canonical_value", canonical_value),
            (
                "canonical_value_sha256",
                PracticalJsonValue::string(canonical_value_sha256),
            ),
        ],
    )?;
    Ok(BoundaryInputCapture {
        artifact,
        raw_bytes: raw_bytes.to_vec(),
        canonical_document: canonical_document.to_vec(),
    })
}

pub fn validate_boundary_input_capture(
    context: &PracticalArtifactContext,
    boundary_contract: &ArtifactRef,
    provenance_id: &str,
    raw_bytes: &[u8],
    canonical_document: &[u8],
    transport: &[u8],
) -> Result<BoundaryInputCapture, PracticalArtifactError> {
    let expected = build_boundary_input_capture(
        context,
        boundary_contract,
        provenance_id,
        raw_bytes,
        canonical_document,
    )?;
    validate_expected_artifact(&expected.artifact, transport)?;
    Ok(expected)
}

#[derive(Clone, Debug)]
pub struct BoundaryOutputCapture {
    artifact: ValidatedPracticalArtifact,
    canonical_document: Vec<u8>,
}

impl BoundaryOutputCapture {
    pub fn artifact(&self) -> &ValidatedPracticalArtifact {
        &self.artifact
    }

    pub fn canonical_document(&self) -> &[u8] {
        &self.canonical_document
    }
}

pub fn build_boundary_output_capture(
    context: &PracticalArtifactContext,
    boundary_contract: &ArtifactRef,
    source_value: PracticalJsonValue,
) -> Result<BoundaryOutputCapture, PracticalArtifactError> {
    let kind = PracticalArtifactKind::BoundaryOutput;
    require_linkage(context, kind, boundary_contract)?;
    if boundary_contract.schema != BOUNDARY_CONTRACT_SCHEMA
        || !matches!(source_value, PracticalJsonValue::Object(_))
    {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryValue,
        ));
    }
    let canonical_document = canonical_practical_json_bytes(&source_value)?;
    let reparsed = parse_canonical_practical_json(kind, &canonical_document)?;
    if reparsed != source_value {
        return Err(failure(
            kind,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryValue,
        ));
    }
    let source_value_sha256 = hash_complete(CANONICAL_VALUE_HASH_DOMAIN, &source_value, kind)?;
    let reparsed_value_sha256 = hash_complete(CANONICAL_VALUE_HASH_DOMAIN, &reparsed, kind)?;
    let artifact = finalized_artifact(
        context,
        kind,
        BOUNDARY_OUTPUT_SCHEMA,
        "capture_sha256",
        BOUNDARY_OUTPUT_HASH_DOMAIN,
        vec![
            ("schema", PracticalJsonValue::string(BOUNDARY_OUTPUT_SCHEMA)),
            ("semantic_context", context.semantic_context.clone()),
            (
                "boundary_contract_sha256",
                PracticalJsonValue::string(boundary_contract.sha256()),
            ),
            ("source_value", source_value),
            (
                "source_value_sha256",
                PracticalJsonValue::string(source_value_sha256),
            ),
            (
                "canonical_document_utf8_sha256",
                PracticalJsonValue::string(raw_sha256(&canonical_document)),
            ),
            ("reparsed_value", reparsed),
            (
                "reparsed_value_sha256",
                PracticalJsonValue::string(reparsed_value_sha256),
            ),
        ],
    )?;
    Ok(BoundaryOutputCapture {
        artifact,
        canonical_document,
    })
}

pub fn validate_boundary_output_capture(
    context: &PracticalArtifactContext,
    boundary_contract: &ArtifactRef,
    source_value: PracticalJsonValue,
    canonical_document: &[u8],
    transport: &[u8],
) -> Result<BoundaryOutputCapture, PracticalArtifactError> {
    let expected = build_boundary_output_capture(context, boundary_contract, source_value)?;
    if expected.canonical_document != canonical_document {
        return Err(failure(
            PracticalArtifactKind::BoundaryOutput,
            PracticalArtifactPhase::Boundary,
            PracticalArtifactErrorCode::BoundaryBytes,
        ));
    }
    validate_expected_artifact(&expected.artifact, transport)?;
    Ok(expected)
}
