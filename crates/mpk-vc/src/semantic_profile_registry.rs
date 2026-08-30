//! Installed semantic-profile registry validation.
//!
//! This module implements the frozen `mpk.semantic_profile.registry.v1`
//! mechanism for private, explicitly injected staging consumers. It does not
//! locate an installed registry, select an active release, or alter the
//! current Go/Rust [`crate::semantic_profile`] path. Revision selection is a
//! closed test/staging input and never a compatibility negotiation.

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, StrictJsonLimits, StrictJsonValue,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const SEMANTIC_REGISTRY_SCHEMA: &str = "mpk.semantic_profile.registry.v1";
pub const SEMANTIC_REGISTRY_ENTRY_SCHEMA: &str = "mpk.semantic_profile.entry.v1";
pub const SEMANTIC_REGISTRY_LIMIT_PROFILE: &str = "mpk.semantic_profile.registry.limits.v1";

pub const GO_FIXED_PROFILE: &str = "mpk.go.fixed.v0";
pub const RUST_CHECKED_PROFILE: &str = "mpk.rust.checked.v0";
pub const CSHARP_SCALAR_PROFILE: &str = "mpk.csharp.scalar.v0";

pub const GO_FIXED_ENTRY_SHA256: &str =
    "b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96";
pub const RUST_CHECKED_ENTRY_SHA256: &str =
    "1cee9716bb21d07e07b8bc1de59ecaf83437549a4d595039486312260816f057";
pub const CSHARP_SCALAR_ENTRY_SHA256: &str =
    "d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac";
pub const REVISION_1_REGISTRY_SHA256: &str =
    "7c9163571cda32aa47984e3e6d949c8857bf62f00110dd1b2c3958eed5e537cc";
pub const REVISION_2_REGISTRY_SHA256: &str =
    "6928e49ab2d0af03bdc1b92c189f99308f815e77edb3850a5f5a8fd9a3d48b75";

pub const SEMANTIC_PROFILE_ENTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SEMANTIC-PROFILE-ENTRY-1.0");
pub const SEMANTIC_PROFILE_REGISTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SEMANTIC-PROFILE-REGISTRY-1.0");

pub const REGISTRY_CANONICAL_BYTES_MAX: u64 = 524_288;
pub const REGISTRY_TRANSPORT_BYTES_MAX: u64 = 524_289;
pub const SEMANTIC_REGISTRY_JSON_NESTING_MAX: u64 = 32;
pub const SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX: u64 = 128;
pub const SOURCE_LANGUAGE_BYTES_MAX: u64 = 64;
pub const SEMANTIC_REGISTRY_PROFILES_MAX: u64 = 256;
pub const SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX: u64 = 65_536;
pub const SELECTION_CANONICAL_BYTES_MAX: u64 = 65_536;
pub const COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX: u64 = 1_048_576;
pub const SEMANTIC_REGISTRY_REVISION_MAX: u64 = 9_007_199_254_740_991;

const ROOT_FIELDS: &[&str] = &["schema", "id", "revision", "profiles", "registry_sha256"];
const ENTRY_FIELDS: &[&str] = &[
    "schema",
    "source_language",
    "semantic_profile",
    "semantic_parameters_schema",
    "selection_schema",
    "contracts",
    "entry_sha256",
];
const CONTRACT_FIELDS: [ProfileContractField; 9] = [
    ProfileContractField::Ai,
    ProfileContractField::Evidence,
    ProfileContractField::Frontend,
    ProfileContractField::Manifest,
    ProfileContractField::Policy,
    ProfileContractField::Release,
    ProfileContractField::SourceMap,
    ProfileContractField::Vc,
    ProfileContractField::Vir,
];

const REGISTRY_TRANSPORT_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    REGISTRY_TRANSPORT_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX,
    SEMANTIC_REGISTRY_JSON_NESTING_MAX,
    SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryRevision {
    Revision1,
    Revision2,
}

impl RegistryRevision {
    pub const fn revision(self) -> u64 {
        match self {
            Self::Revision1 => 1,
            Self::Revision2 => 2,
        }
    }

    pub const fn registry_sha256(self) -> &'static str {
        match self {
            Self::Revision1 => REVISION_1_REGISTRY_SHA256,
            Self::Revision2 => REVISION_2_REGISTRY_SHA256,
        }
    }

    pub fn identity(self) -> ProfileRegistryIdentity {
        ProfileRegistryIdentity {
            schema: SEMANTIC_REGISTRY_SCHEMA.to_owned(),
            id: SEMANTIC_REGISTRY_SCHEMA.to_owned(),
            revision: self.revision(),
            registry_sha256: self.registry_sha256().to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompiledSemanticProfile {
    GoFixedV0,
    RustCheckedV0,
    CSharpScalarV0,
}

impl CompiledSemanticProfile {
    pub const fn source_language(self) -> &'static str {
        match self {
            Self::GoFixedV0 => "go",
            Self::RustCheckedV0 => "rust",
            Self::CSharpScalarV0 => "csharp",
        }
    }

    pub const fn semantic_profile(self) -> &'static str {
        match self {
            Self::GoFixedV0 => GO_FIXED_PROFILE,
            Self::RustCheckedV0 => RUST_CHECKED_PROFILE,
            Self::CSharpScalarV0 => CSHARP_SCALAR_PROFILE,
        }
    }

    pub const fn semantic_parameters_schema(self) -> &'static str {
        match self {
            Self::GoFixedV0 => "mpk.semantic_parameters.go_fixed.v0",
            Self::RustCheckedV0 => "mpk.semantic_parameters.rust_checked.v0",
            Self::CSharpScalarV0 => "mpk.semantic_parameters.csharp_scalar.v0",
        }
    }

    pub const fn selection_schema(self) -> &'static str {
        match self {
            Self::GoFixedV0 => "mpk.selection.go_function.v0",
            Self::RustCheckedV0 => "mpk.selection.rust_function.v0",
            Self::CSharpScalarV0 => "mpk.selection.csharp_methods.v0",
        }
    }

    pub const fn entry_sha256(self) -> &'static str {
        match self {
            Self::GoFixedV0 => GO_FIXED_ENTRY_SHA256,
            Self::RustCheckedV0 => RUST_CHECKED_ENTRY_SHA256,
            Self::CSharpScalarV0 => CSHARP_SCALAR_ENTRY_SHA256,
        }
    }

    pub fn from_profile(profile: &str) -> Option<Self> {
        match profile {
            GO_FIXED_PROFILE => Some(Self::GoFixedV0),
            RUST_CHECKED_PROFILE => Some(Self::RustCheckedV0),
            CSHARP_SCALAR_PROFILE => Some(Self::CSharpScalarV0),
            _ => None,
        }
    }

    pub fn from_identity(source_language: &str, semantic_profile: &str) -> Option<Self> {
        let profile = Self::from_profile(semantic_profile)?;
        (profile.source_language() == source_language).then_some(profile)
    }

    pub const fn parameter_contract(self) -> CompiledParameterContract {
        match self {
            Self::GoFixedV0 => CompiledParameterContract::GoFixedV0,
            Self::RustCheckedV0 => CompiledParameterContract::RustCheckedV0,
            Self::CSharpScalarV0 => CompiledParameterContract::CSharpScalarV0,
        }
    }

    pub const fn selection_contract(self) -> CompiledSelectionContract {
        match self {
            Self::GoFixedV0 => CompiledSelectionContract::GoFunctionV0,
            Self::RustCheckedV0 => CompiledSelectionContract::RustFunctionV0,
            Self::CSharpScalarV0 => CompiledSelectionContract::CSharpMethodsV0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompiledParameterContract {
    GoFixedV0,
    RustCheckedV0,
    CSharpScalarV0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompiledSelectionContract {
    GoFunctionV0,
    RustFunctionV0,
    CSharpMethodsV0,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProfileContractField {
    Ai,
    Evidence,
    Frontend,
    Manifest,
    Policy,
    Release,
    SourceMap,
    Vc,
    Vir,
}

impl ProfileContractField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ai => "ai",
            Self::Evidence => "evidence",
            Self::Frontend => "frontend",
            Self::Manifest => "manifest",
            Self::Policy => "policy",
            Self::Release => "release",
            Self::SourceMap => "source_map",
            Self::Vc => "vc",
            Self::Vir => "vir",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "ai" => Some(Self::Ai),
            "evidence" => Some(Self::Evidence),
            "frontend" => Some(Self::Frontend),
            "manifest" => Some(Self::Manifest),
            "policy" => Some(Self::Policy),
            "release" => Some(Self::Release),
            "source_map" => Some(Self::SourceMap),
            "vc" => Some(Self::Vc),
            "vir" => Some(Self::Vir),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompiledProfileContract {
    pub profile: CompiledSemanticProfile,
    pub field: ProfileContractField,
}

impl CompiledProfileContract {
    pub const fn contract_id(self) -> &'static str {
        use CompiledSemanticProfile::{CSharpScalarV0, GoFixedV0, RustCheckedV0};
        use ProfileContractField::{
            Ai, Evidence, Frontend, Manifest, Policy, Release, SourceMap, Vc, Vir,
        };
        match (self.profile, self.field) {
            (GoFixedV0, Ai) => "mpk.profile.ai.go_fixed.v0",
            (GoFixedV0, Evidence) => "mpk.profile.evidence.go_fixed.v0",
            (GoFixedV0, Frontend) => "mpk.profile.frontend.go_fixed.v0",
            (GoFixedV0, Manifest) => "mpk.profile.manifest.go_fixed.v0",
            (GoFixedV0, Policy) => "mpk.profile.policy.go_fixed.v0",
            (GoFixedV0, Release) => "mpk.profile.release.go_fixed.v0",
            (GoFixedV0, SourceMap) => "mpk.profile.source_map.go_fixed.v0",
            (GoFixedV0, Vc) => "mpk.profile.vc.go_fixed.v0",
            (GoFixedV0, Vir) => "mpk.profile.vir.go_fixed.v0",
            (RustCheckedV0, Ai) => "mpk.profile.ai.rust_checked.v0",
            (RustCheckedV0, Evidence) => "mpk.profile.evidence.rust_checked.v0",
            (RustCheckedV0, Frontend) => "mpk.profile.frontend.rust_checked.v0",
            (RustCheckedV0, Manifest) => "mpk.profile.manifest.rust_checked.v0",
            (RustCheckedV0, Policy) => "mpk.profile.policy.rust_checked.v0",
            (RustCheckedV0, Release) => "mpk.profile.release.rust_checked.v0",
            (RustCheckedV0, SourceMap) => "mpk.profile.source_map.rust_checked.v0",
            (RustCheckedV0, Vc) => "mpk.profile.vc.rust_checked.v0",
            (RustCheckedV0, Vir) => "mpk.profile.vir.rust_checked.v0",
            (CSharpScalarV0, Ai) => "mpk.profile.ai.csharp_scalar.v0",
            (CSharpScalarV0, Evidence) => "mpk.profile.evidence.csharp_scalar.v0",
            (CSharpScalarV0, Frontend) => "mpk.profile.frontend.csharp_scalar.v0",
            (CSharpScalarV0, Manifest) => "mpk.profile.manifest.csharp_scalar.v0",
            (CSharpScalarV0, Policy) => "mpk.profile.policy.csharp_scalar.v0",
            (CSharpScalarV0, Release) => "mpk.profile.release.csharp_scalar.v0",
            (CSharpScalarV0, SourceMap) => "mpk.profile.source_map.csharp_scalar.v0",
            (CSharpScalarV0, Vc) => "mpk.profile.vc.csharp_scalar.v0",
            (CSharpScalarV0, Vir) => "mpk.profile.vir.csharp_scalar.v0",
        }
    }

    pub fn from_contract_id(value: &str) -> Option<Self> {
        for profile in [
            CompiledSemanticProfile::GoFixedV0,
            CompiledSemanticProfile::RustCheckedV0,
            CompiledSemanticProfile::CSharpScalarV0,
        ] {
            for field in CONTRACT_FIELDS {
                let contract = Self { profile, field };
                if contract.contract_id() == value {
                    return Some(contract);
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProfileRegistryIdentity {
    schema: String,
    id: String,
    revision: u64,
    registry_sha256: String,
}

impl ProfileRegistryIdentity {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn registry_sha256(&self) -> &str {
        &self.registry_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledContracts {
    values: [String; 9],
}

impl CompiledContracts {
    pub fn contract_id(&self, field: ProfileContractField) -> &str {
        &self.values[contract_index(field)]
    }

    pub fn iter(&self) -> impl Iterator<Item = (ProfileContractField, &str)> {
        CONTRACT_FIELDS
            .into_iter()
            .map(|field| (field, self.contract_id(field)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticProfileEntry {
    source_language: String,
    semantic_profile: String,
    semantic_parameters_schema: String,
    selection_schema: String,
    contracts: CompiledContracts,
    entry_sha256: String,
    compiled_profile: CompiledSemanticProfile,
    canonical_json: Vec<u8>,
}

impl SemanticProfileEntry {
    pub fn source_language(&self) -> &str {
        &self.source_language
    }

    pub fn semantic_profile(&self) -> &str {
        &self.semantic_profile
    }

    pub fn semantic_parameters_schema(&self) -> &str {
        &self.semantic_parameters_schema
    }

    pub fn selection_schema(&self) -> &str {
        &self.selection_schema
    }

    pub fn contracts(&self) -> &CompiledContracts {
        &self.contracts
    }

    pub fn entry_sha256(&self) -> &str {
        &self.entry_sha256
    }

    pub const fn compiled_profile(&self) -> CompiledSemanticProfile {
        self.compiled_profile
    }

    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }

    pub fn compiled_contract(&self, field: ProfileContractField) -> CompiledProfileContract {
        CompiledProfileContract {
            profile: self.compiled_profile,
            field,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSemanticProfileRegistry {
    revision: RegistryRevision,
    identity: ProfileRegistryIdentity,
    entries: Vec<SemanticProfileEntry>,
}

impl ValidatedSemanticProfileRegistry {
    pub const fn revision(&self) -> RegistryRevision {
        self.revision
    }

    pub fn identity(&self) -> &ProfileRegistryIdentity {
        &self.identity
    }

    pub fn entries(&self) -> &[SemanticProfileEntry] {
        &self.entries
    }

    pub fn lookup(
        &self,
        source_language: &str,
        semantic_profile: &str,
    ) -> Option<&SemanticProfileEntry> {
        self.entries.iter().find(|entry| {
            entry.source_language == source_language && entry.semantic_profile == semantic_profile
        })
    }

    pub fn lookup_entry_hash(&self, entry_sha256: &str) -> Option<&SemanticProfileEntry> {
        self.entries
            .iter()
            .find(|entry| entry.entry_sha256 == entry_sha256)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SemanticParametersEnvelope {
    schema: String,
    value: Value,
}

impl SemanticParametersEnvelope {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SelectionEnvelope {
    schema: String,
    value: Value,
}

impl SelectionEnvelope {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SemanticContext {
    profile_registry: ProfileRegistryIdentity,
    profile_entry_sha256: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: SemanticParametersEnvelope,
}

impl SemanticContext {
    pub fn profile_registry(&self) -> &ProfileRegistryIdentity {
        &self.profile_registry
    }

    pub fn profile_entry_sha256(&self) -> &str {
        &self.profile_entry_sha256
    }

    pub fn source_language(&self) -> &str {
        &self.source_language
    }

    pub fn semantic_profile(&self) -> &str {
        &self.semantic_profile
    }

    pub fn semantic_parameters(&self) -> &SemanticParametersEnvelope {
        &self.semantic_parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidatedSemanticRequest {
    semantic_context: SemanticContext,
    selection: SelectionEnvelope,
    #[serde(skip)]
    compiled_profile: CompiledSemanticProfile,
}

impl ValidatedSemanticRequest {
    pub fn semantic_context(&self) -> &SemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &SelectionEnvelope {
        &self.selection
    }

    pub const fn compiled_profile(&self) -> CompiledSemanticProfile {
        self.compiled_profile
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CompiledProfileEnvelope {
    profile_entry_sha256: String,
    contract_id: String,
    value: Value,
}

impl CompiledProfileEnvelope {
    pub fn profile_entry_sha256(&self) -> &str {
        &self.profile_entry_sha256
    }

    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRegistryValidationPhase {
    Transport,
    Shape,
    Scalar,
    Limits,
    Order,
    EntryHash,
    ContractBinding,
    Invariant,
    RegistryHash,
    EmbeddedIdentity,
    CanonicalTransport,
    RegistryIdentity,
    ProfileLookup,
    ProfileEntry,
    ParametersSchema,
    ParametersValue,
    SelectionSchema,
    SelectionValue,
    ProfileEnvelope,
    ProfileContract,
    ProfilePayload,
    ContextLinkage,
}

impl SemanticRegistryValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Limits => "limits",
            Self::Order => "order",
            Self::EntryHash => "entry_hash",
            Self::ContractBinding => "contract_binding",
            Self::Invariant => "invariant",
            Self::RegistryHash => "registry_hash",
            Self::EmbeddedIdentity => "embedded_identity",
            Self::CanonicalTransport => "canonical_transport",
            Self::RegistryIdentity => "registry_identity",
            Self::ProfileLookup => "profile_lookup",
            Self::ProfileEntry => "profile_entry",
            Self::ParametersSchema => "parameters_schema",
            Self::ParametersValue => "parameters_value",
            Self::SelectionSchema => "selection_schema",
            Self::SelectionValue => "selection_value",
            Self::ProfileEnvelope => "profile_envelope",
            Self::ProfileContract => "profile_contract",
            Self::ProfilePayload => "profile_payload",
            Self::ContextLinkage => "context_linkage",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRegistryErrorCode {
    RegistryTransport,
    RegistryShape,
    RegistryScalar,
    RegistryLimit,
    RegistryOrder,
    RegistryEntryHash,
    RegistryContract,
    RegistryInvariant,
    RegistryHash,
    RegistryCanonical,
    RegistryAssertion,
    ProfileUnknown,
    ProfileEntry,
    ParametersSchema,
    ParametersInvalid,
    SelectionSchema,
    SelectionInvalid,
    ProfileEnvelope,
    ProfileContract,
    ProfilePayload,
    ContextLinkage,
}

impl SemanticRegistryErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryTransport => "SEMANTIC_REGISTRY_TRANSPORT",
            Self::RegistryShape => "SEMANTIC_REGISTRY_SHAPE",
            Self::RegistryScalar => "SEMANTIC_REGISTRY_SCALAR",
            Self::RegistryLimit => "SEMANTIC_REGISTRY_LIMIT",
            Self::RegistryOrder => "SEMANTIC_REGISTRY_ORDER",
            Self::RegistryEntryHash => "SEMANTIC_REGISTRY_ENTRY_HASH",
            Self::RegistryContract => "SEMANTIC_REGISTRY_CONTRACT",
            Self::RegistryInvariant => "SEMANTIC_REGISTRY_INVARIANT",
            Self::RegistryHash => "SEMANTIC_REGISTRY_HASH",
            Self::RegistryCanonical => "SEMANTIC_REGISTRY_CANONICAL",
            Self::RegistryAssertion => "SEMANTIC_REGISTRY_ASSERTION",
            Self::ProfileUnknown => "SEMANTIC_PROFILE_UNKNOWN",
            Self::ProfileEntry => "SEMANTIC_PROFILE_ENTRY",
            Self::ParametersSchema => "SEMANTIC_PARAMETERS_SCHEMA",
            Self::ParametersInvalid => "SEMANTIC_PARAMETERS_INVALID",
            Self::SelectionSchema => "SEMANTIC_SELECTION_SCHEMA",
            Self::SelectionInvalid => "SEMANTIC_SELECTION_INVALID",
            Self::ProfileEnvelope => "SEMANTIC_PROFILE_ENVELOPE",
            Self::ProfileContract => "SEMANTIC_PROFILE_CONTRACT",
            Self::ProfilePayload => "SEMANTIC_PROFILE_PAYLOAD",
            Self::ContextLinkage => "SEMANTIC_CONTEXT_LINKAGE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticRegistryValidationError {
    phase: SemanticRegistryValidationPhase,
    code: SemanticRegistryErrorCode,
}

impl SemanticRegistryValidationError {
    const fn new(phase: SemanticRegistryValidationPhase, code: SemanticRegistryErrorCode) -> Self {
        Self { phase, code }
    }

    pub const fn phase(&self) -> SemanticRegistryValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> SemanticRegistryErrorCode {
        self.code
    }
}

impl fmt::Display for SemanticRegistryValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at semantic-profile phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for SemanticRegistryValidationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRegistryFailureSurface {
    CallerConfiguration,
    InstalledRegistry,
    LaunchedChildContext,
    ImportedArtifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRegistryFailureDisposition {
    PrelaunchConfiguration,
    ReleaseFrontendError,
    ChildFrontendError,
    InvalidArtifact,
}

impl SemanticRegistryFailureDisposition {
    pub const fn exit_code(self) -> Option<u8> {
        match self {
            Self::PrelaunchConfiguration => Some(2),
            Self::ReleaseFrontendError | Self::ChildFrontendError | Self::InvalidArtifact => None,
        }
    }

    pub const fn status(self) -> Option<&'static str> {
        match self {
            Self::ReleaseFrontendError | Self::ChildFrontendError => Some("frontend-error"),
            Self::PrelaunchConfiguration | Self::InvalidArtifact => None,
        }
    }

    pub const fn child_started(self) -> bool {
        matches!(self, Self::ChildFrontendError)
    }

    pub const fn may_publish_artifact(self) -> bool {
        false
    }

    pub const fn may_become_ready_or_verified(self) -> bool {
        false
    }
}

/// Maps an already precedence-selected semantic-registry failure to the
/// frozen boundary disposition. The error is accepted explicitly so callers
/// cannot classify a successful path by supplying only a surface label.
pub const fn classify_semantic_registry_failure(
    _error: &SemanticRegistryValidationError,
    surface: SemanticRegistryFailureSurface,
) -> SemanticRegistryFailureDisposition {
    match surface {
        SemanticRegistryFailureSurface::CallerConfiguration => {
            SemanticRegistryFailureDisposition::PrelaunchConfiguration
        }
        SemanticRegistryFailureSurface::InstalledRegistry => {
            SemanticRegistryFailureDisposition::ReleaseFrontendError
        }
        SemanticRegistryFailureSurface::LaunchedChildContext => {
            SemanticRegistryFailureDisposition::ChildFrontendError
        }
        SemanticRegistryFailureSurface::ImportedArtifact => {
            SemanticRegistryFailureDisposition::InvalidArtifact
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRegistryLimit {
    RegistryCanonicalBytes,
    RegistryTransportBytes,
    JsonNesting,
    IdentifierBytes,
    SourceLanguageBytes,
    Profiles,
    SemanticParametersCanonicalBytes,
    SelectionCanonicalBytes,
    CompiledProfilePayloadCanonicalBytes,
    Revision,
}

impl SemanticRegistryLimit {
    pub fn from_id(value: &str) -> Option<Self> {
        match value {
            "registry_canonical_bytes" => Some(Self::RegistryCanonicalBytes),
            "registry_transport_bytes" => Some(Self::RegistryTransportBytes),
            "json_nesting" => Some(Self::JsonNesting),
            "identifier_bytes" => Some(Self::IdentifierBytes),
            "source_language_bytes" => Some(Self::SourceLanguageBytes),
            "profiles" => Some(Self::Profiles),
            "semantic_parameters_canonical_bytes" => Some(Self::SemanticParametersCanonicalBytes),
            "selection_canonical_bytes" => Some(Self::SelectionCanonicalBytes),
            "compiled_profile_payload_canonical_bytes" => {
                Some(Self::CompiledProfilePayloadCanonicalBytes)
            }
            "revision" => Some(Self::Revision),
            _ => None,
        }
    }

    pub const fn inclusive_maximum(self) -> u64 {
        match self {
            Self::RegistryCanonicalBytes => REGISTRY_CANONICAL_BYTES_MAX,
            Self::RegistryTransportBytes => REGISTRY_TRANSPORT_BYTES_MAX,
            Self::JsonNesting => SEMANTIC_REGISTRY_JSON_NESTING_MAX,
            Self::IdentifierBytes => SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX,
            Self::SourceLanguageBytes => SOURCE_LANGUAGE_BYTES_MAX,
            Self::Profiles => SEMANTIC_REGISTRY_PROFILES_MAX,
            Self::SemanticParametersCanonicalBytes => SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX,
            Self::SelectionCanonicalBytes => SELECTION_CANONICAL_BYTES_MAX,
            Self::CompiledProfilePayloadCanonicalBytes => {
                COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX
            }
            Self::Revision => SEMANTIC_REGISTRY_REVISION_MAX,
        }
    }
}

pub fn validate_semantic_registry_limit(
    limit: SemanticRegistryLimit,
    value: u64,
) -> Result<(), SemanticRegistryValidationError> {
    if value <= limit.inclusive_maximum() {
        Ok(())
    } else {
        Err(failure(
            SemanticRegistryValidationPhase::Limits,
            SemanticRegistryErrorCode::RegistryLimit,
        ))
    }
}

pub fn canonical_registry_transport(
    registry: &Value,
) -> Result<Vec<u8>, SemanticRegistryValidationError> {
    if !value_within_preallocation_limit(registry, REGISTRY_CANONICAL_BYTES_MAX) {
        return Err(failure(
            SemanticRegistryValidationPhase::Limits,
            SemanticRegistryErrorCode::RegistryLimit,
        ));
    }
    let strict = serde_to_strict(registry).ok_or_else(|| {
        failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        )
    })?;
    let mut bytes = canonical_json_bytes_bounded(
        &strict,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).expect("registry limit fits usize"),
    )
    .map_err(|_| {
        failure(
            SemanticRegistryValidationPhase::Limits,
            SemanticRegistryErrorCode::RegistryLimit,
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn semantic_profile_entry_hash(
    entry: &Value,
) -> Result<String, SemanticRegistryValidationError> {
    hash_without_field(
        entry,
        "entry_sha256",
        SEMANTIC_PROFILE_ENTRY_HASH_DOMAIN,
        SemanticRegistryValidationPhase::EntryHash,
        SemanticRegistryErrorCode::RegistryEntryHash,
    )
}

pub fn semantic_profile_registry_hash(
    registry: &Value,
) -> Result<String, SemanticRegistryValidationError> {
    hash_without_field(
        registry,
        "registry_sha256",
        SEMANTIC_PROFILE_REGISTRY_HASH_DOMAIN,
        SemanticRegistryValidationPhase::RegistryHash,
        SemanticRegistryErrorCode::RegistryHash,
    )
}

pub fn validate_semantic_profile_registry(
    transport: &[u8],
    expected_revision: RegistryRevision,
) -> Result<ValidatedSemanticProfileRegistry, SemanticRegistryValidationError> {
    let actual_transport_bytes = u64::try_from(transport.len()).unwrap_or(u64::MAX);
    validate_semantic_registry_limit(
        SemanticRegistryLimit::RegistryTransportBytes,
        actual_transport_bytes,
    )
    .map_err(|_| {
        failure(
            SemanticRegistryValidationPhase::Transport,
            SemanticRegistryErrorCode::RegistryTransport,
        )
    })?;

    let parsed = parse_strict_json(transport, REGISTRY_TRANSPORT_LIMITS).map_err(|_| {
        failure(
            SemanticRegistryValidationPhase::Transport,
            SemanticRegistryErrorCode::RegistryTransport,
        )
    })?;
    let registry = strict_to_serde(parsed);
    validate_registry_value(&registry, transport, expected_revision)
}

pub fn validate_semantic_request(
    registry: &ValidatedSemanticProfileRegistry,
    request: &Value,
) -> Result<ValidatedSemanticRequest, SemanticRegistryValidationError> {
    if !has_exact_fields(request, &["semantic_context", "selection"])
        || !semantic_context_shape_is_valid(get(request, "semantic_context"))
        || !selection_shape_is_valid(get(request, "selection"))
    {
        return Err(failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        ));
    }

    let (semantic_context, compiled_profile) =
        validate_semantic_context_after_shape(registry, get(request, "semantic_context"))?;
    let selection =
        validate_selection_after_shape(registry, &semantic_context, get(request, "selection"))?;

    Ok(ValidatedSemanticRequest {
        semantic_context,
        selection,
        compiled_profile,
    })
}

/// Validates one closed semantic context against an already validated
/// installed registry.
pub fn validate_registry_semantic_context(
    registry: &ValidatedSemanticProfileRegistry,
    context: &Value,
) -> Result<SemanticContext, SemanticRegistryValidationError> {
    if !semantic_context_shape_is_valid(context) {
        return Err(failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        ));
    }
    validate_semantic_context_after_shape(registry, context).map(|(context, _)| context)
}

/// Validates one closed selection envelope for a previously validated
/// semantic context. Detailed C# path/capture checks remain owned by T05.
pub fn validate_registry_selection_envelope(
    registry: &ValidatedSemanticProfileRegistry,
    context: &SemanticContext,
    selection: &Value,
) -> Result<SelectionEnvelope, SemanticRegistryValidationError> {
    if !selection_shape_is_valid(selection) {
        return Err(failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        ));
    }
    validate_selection_after_shape(registry, context, selection)
}

fn validate_semantic_context_after_shape(
    registry: &ValidatedSemanticProfileRegistry,
    context: &Value,
) -> Result<(SemanticContext, CompiledSemanticProfile), SemanticRegistryValidationError> {
    let identity = parse_registry_identity(get(context, "profile_registry")).ok_or_else(|| {
        failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        )
    })?;
    if &identity != registry.identity() {
        return Err(failure(
            SemanticRegistryValidationPhase::RegistryIdentity,
            SemanticRegistryErrorCode::RegistryAssertion,
        ));
    }

    let source_language = string(get(context, "source_language")).expect("shape checked");
    let semantic_profile = string(get(context, "semantic_profile")).expect("shape checked");
    let entry = registry
        .lookup(source_language, semantic_profile)
        .ok_or_else(|| {
            failure(
                SemanticRegistryValidationPhase::ProfileLookup,
                SemanticRegistryErrorCode::ProfileUnknown,
            )
        })?;

    let entry_hash = string(get(context, "profile_entry_sha256")).expect("shape checked");
    if entry_hash != entry.entry_sha256 {
        return Err(failure(
            SemanticRegistryValidationPhase::ProfileEntry,
            SemanticRegistryErrorCode::ProfileEntry,
        ));
    }

    let parameters = get(context, "semantic_parameters");
    let parameter_schema = string(get(parameters, "schema")).expect("shape checked");
    if parameter_schema != entry.semantic_parameters_schema {
        return Err(failure(
            SemanticRegistryValidationPhase::ParametersSchema,
            SemanticRegistryErrorCode::ParametersSchema,
        ));
    }
    if !canonical_size_within(parameters, SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX)
        || !validate_parameter_value(
            entry.compiled_profile.parameter_contract(),
            get(parameters, "value"),
        )
    {
        return Err(failure(
            SemanticRegistryValidationPhase::ParametersValue,
            SemanticRegistryErrorCode::ParametersInvalid,
        ));
    }

    Ok((
        SemanticContext {
            profile_registry: identity,
            profile_entry_sha256: entry_hash.to_owned(),
            source_language: source_language.to_owned(),
            semantic_profile: semantic_profile.to_owned(),
            semantic_parameters: SemanticParametersEnvelope {
                schema: parameter_schema.to_owned(),
                value: get(parameters, "value").clone(),
            },
        },
        entry.compiled_profile,
    ))
}

fn validate_selection_after_shape(
    registry: &ValidatedSemanticProfileRegistry,
    context: &SemanticContext,
    selection: &Value,
) -> Result<SelectionEnvelope, SemanticRegistryValidationError> {
    if &context.profile_registry != registry.identity() {
        return Err(failure(
            SemanticRegistryValidationPhase::RegistryIdentity,
            SemanticRegistryErrorCode::RegistryAssertion,
        ));
    }
    let entry = registry
        .lookup(&context.source_language, &context.semantic_profile)
        .ok_or_else(|| {
            failure(
                SemanticRegistryValidationPhase::ProfileLookup,
                SemanticRegistryErrorCode::ProfileUnknown,
            )
        })?;
    if context.profile_entry_sha256 != entry.entry_sha256 {
        return Err(failure(
            SemanticRegistryValidationPhase::ProfileEntry,
            SemanticRegistryErrorCode::ProfileEntry,
        ));
    }
    let selection_schema = string(get(selection, "schema")).expect("shape checked");
    if selection_schema != entry.selection_schema {
        return Err(failure(
            SemanticRegistryValidationPhase::SelectionSchema,
            SemanticRegistryErrorCode::SelectionSchema,
        ));
    }
    if !canonical_size_within(selection, SELECTION_CANONICAL_BYTES_MAX)
        || !validate_selection_value(
            entry.compiled_profile.selection_contract(),
            get(selection, "value"),
        )
    {
        return Err(failure(
            SemanticRegistryValidationPhase::SelectionValue,
            SemanticRegistryErrorCode::SelectionInvalid,
        ));
    }

    Ok(SelectionEnvelope {
        schema: selection_schema.to_owned(),
        value: get(selection, "value").clone(),
    })
}

pub fn validate_compiled_profile_envelope(
    registry: &ValidatedSemanticProfileRegistry,
    envelope: &Value,
    contract_field: ProfileContractField,
) -> Result<CompiledProfileEnvelope, SemanticRegistryValidationError> {
    if !has_exact_fields(envelope, &["profile_entry_sha256", "contract_id", "value"])
        || string(get(envelope, "profile_entry_sha256")).is_none()
        || string(get(envelope, "contract_id")).is_none()
        || get(envelope, "value").as_object().is_none()
    {
        return Err(failure(
            SemanticRegistryValidationPhase::ProfileEnvelope,
            SemanticRegistryErrorCode::ProfileEnvelope,
        ));
    }

    let entry_hash = string(get(envelope, "profile_entry_sha256")).expect("shape checked");
    let entry = registry.lookup_entry_hash(entry_hash).ok_or_else(|| {
        failure(
            SemanticRegistryValidationPhase::ProfileEntry,
            SemanticRegistryErrorCode::ProfileEntry,
        )
    })?;
    let contract_id = string(get(envelope, "contract_id")).expect("shape checked");
    let expected_contract = entry.compiled_contract(contract_field);
    if contract_id != entry.contracts.contract_id(contract_field)
        || CompiledProfileContract::from_contract_id(contract_id) != Some(expected_contract)
    {
        return Err(failure(
            SemanticRegistryValidationPhase::ProfileContract,
            SemanticRegistryErrorCode::ProfileContract,
        ));
    }
    if !canonical_size_within(envelope, COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX)
        || !validate_profile_payload(expected_contract, get(envelope, "value"))
    {
        return Err(failure(
            SemanticRegistryValidationPhase::ProfilePayload,
            SemanticRegistryErrorCode::ProfilePayload,
        ));
    }

    Ok(CompiledProfileEnvelope {
        profile_entry_sha256: entry_hash.to_owned(),
        contract_id: contract_id.to_owned(),
        value: get(envelope, "value").clone(),
    })
}

pub fn validate_semantic_context_linkage(
    left: &SemanticContext,
    right: &SemanticContext,
) -> Result<(), SemanticRegistryValidationError> {
    if left == right {
        Ok(())
    } else {
        Err(failure(
            SemanticRegistryValidationPhase::ContextLinkage,
            SemanticRegistryErrorCode::ContextLinkage,
        ))
    }
}

pub fn validate_revision_2_append_only(
    predecessor: &ValidatedSemanticProfileRegistry,
    successor: &ValidatedSemanticProfileRegistry,
) -> Result<(), SemanticRegistryValidationError> {
    let valid_revisions = predecessor.revision == RegistryRevision::Revision1
        && successor.revision == RegistryRevision::Revision2;
    let exact_count = predecessor.entries.len() == 2 && successor.entries.len() == 3;
    let csharp_first = successor.entries.first().is_some_and(|entry| {
        entry.compiled_profile == CompiledSemanticProfile::CSharpScalarV0
            && entry.entry_sha256 == CSHARP_SCALAR_ENTRY_SHA256
    });
    let retained = predecessor.entries.iter().all(|old| {
        successor
            .lookup(&old.source_language, &old.semantic_profile)
            .is_some_and(|new| new.canonical_json == old.canonical_json)
    });
    let exact_profiles = successor
        .entries
        .iter()
        .map(|entry| entry.compiled_profile)
        .collect::<BTreeSet<_>>()
        == BTreeSet::from([
            CompiledSemanticProfile::CSharpScalarV0,
            CompiledSemanticProfile::GoFixedV0,
            CompiledSemanticProfile::RustCheckedV0,
        ]);
    if valid_revisions && exact_count && csharp_first && retained && exact_profiles {
        Ok(())
    } else {
        Err(failure(
            SemanticRegistryValidationPhase::Invariant,
            SemanticRegistryErrorCode::RegistryInvariant,
        ))
    }
}

fn validate_registry_value(
    registry: &Value,
    transport: &[u8],
    expected_revision: RegistryRevision,
) -> Result<ValidatedSemanticProfileRegistry, SemanticRegistryValidationError> {
    if !has_exact_fields(registry, ROOT_FIELDS)
        || string(get(registry, "schema")) != Some(SEMANTIC_REGISTRY_SCHEMA)
        || get(registry, "profiles").as_array().is_none()
        || get(registry, "profiles")
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Err(failure(
            SemanticRegistryValidationPhase::Shape,
            SemanticRegistryErrorCode::RegistryShape,
        ));
    }
    let profiles = get(registry, "profiles").as_array().expect("shape checked");
    for entry in profiles {
        if !has_exact_fields(entry, ENTRY_FIELDS)
            || string(get(entry, "schema")) != Some(SEMANTIC_REGISTRY_ENTRY_SCHEMA)
            || get(entry, "contracts").as_object().is_none()
            || !has_exact_fields(
                get(entry, "contracts"),
                &CONTRACT_FIELDS
                    .iter()
                    .map(|field| field.as_str())
                    .collect::<Vec<_>>(),
            )
        {
            return Err(failure(
                SemanticRegistryValidationPhase::Shape,
                SemanticRegistryErrorCode::RegistryShape,
            ));
        }
    }

    let id = string(get(registry, "id"));
    let revision = unsigned_integer(get(registry, "revision"));
    let root_hash = string(get(registry, "registry_sha256"));
    if id != Some(SEMANTIC_REGISTRY_SCHEMA)
        || !id.is_some_and(|value| valid_identifier(value, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX))
        || !revision.is_some_and(|value| (1..=SEMANTIC_REGISTRY_REVISION_MAX).contains(&value))
        || !root_hash.is_some_and(valid_sha256)
    {
        return Err(failure(
            SemanticRegistryValidationPhase::Scalar,
            SemanticRegistryErrorCode::RegistryScalar,
        ));
    }
    for entry in profiles {
        let language = string(get(entry, "source_language"));
        let profile = string(get(entry, "semantic_profile"));
        let parameters = string(get(entry, "semantic_parameters_schema"));
        let selection = string(get(entry, "selection_schema"));
        let entry_hash = string(get(entry, "entry_sha256"));
        if !language.is_some_and(|value| valid_identifier(value, SOURCE_LANGUAGE_BYTES_MAX))
            || !profile.is_some_and(|value| {
                valid_identifier(value, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX)
            })
            || !parameters.is_some_and(|value| {
                valid_identifier(value, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX)
            })
            || !selection.is_some_and(|value| {
                valid_identifier(value, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX)
            })
            || !entry_hash.is_some_and(valid_sha256)
            || object(get(entry, "contracts")).values().any(|contract| {
                !string(contract).is_some_and(|value| {
                    valid_identifier(value, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX)
                })
            })
        {
            return Err(failure(
                SemanticRegistryValidationPhase::Scalar,
                SemanticRegistryErrorCode::RegistryScalar,
            ));
        }
    }

    let canonical = canonical_value(registry).ok_or_else(|| {
        failure(
            SemanticRegistryValidationPhase::Limits,
            SemanticRegistryErrorCode::RegistryLimit,
        )
    })?;
    let canonical_size = u64::try_from(canonical.len()).unwrap_or(u64::MAX);
    let transport_size = canonical_size.saturating_add(1);
    let profile_count = u64::try_from(profiles.len()).unwrap_or(u64::MAX);
    if validate_semantic_registry_limit(
        SemanticRegistryLimit::RegistryCanonicalBytes,
        canonical_size,
    )
    .is_err()
        || validate_semantic_registry_limit(
            SemanticRegistryLimit::RegistryTransportBytes,
            transport_size,
        )
        .is_err()
        || validate_semantic_registry_limit(SemanticRegistryLimit::Profiles, profile_count).is_err()
    {
        return Err(failure(
            SemanticRegistryValidationPhase::Limits,
            SemanticRegistryErrorCode::RegistryLimit,
        ));
    }

    let mut prior: Option<(&str, &str)> = None;
    let mut unique_profiles = BTreeSet::new();
    for entry in profiles {
        let pair = (
            string(get(entry, "source_language")).expect("scalar checked"),
            string(get(entry, "semantic_profile")).expect("scalar checked"),
        );
        if prior.is_some_and(|previous| previous >= pair) || !unique_profiles.insert(pair.1) {
            return Err(failure(
                SemanticRegistryValidationPhase::Order,
                SemanticRegistryErrorCode::RegistryOrder,
            ));
        }
        prior = Some(pair);
    }

    for entry in profiles {
        let expected = string(get(entry, "entry_sha256")).expect("scalar checked");
        if semantic_profile_entry_hash(entry).as_deref() != Ok(expected) {
            return Err(failure(
                SemanticRegistryValidationPhase::EntryHash,
                SemanticRegistryErrorCode::RegistryEntryHash,
            ));
        }
    }

    for entry in profiles {
        for contract in object(get(entry, "contracts")).values() {
            let id = string(contract).expect("scalar checked");
            if CompiledProfileContract::from_contract_id(id).is_none() {
                return Err(failure(
                    SemanticRegistryValidationPhase::ContractBinding,
                    SemanticRegistryErrorCode::RegistryContract,
                ));
            }
        }
    }

    for entry in profiles {
        if !entry_invariants_hold(entry) {
            return Err(failure(
                SemanticRegistryValidationPhase::Invariant,
                SemanticRegistryErrorCode::RegistryInvariant,
            ));
        }
    }

    let expected_root_hash = string(get(registry, "registry_sha256")).expect("scalar checked");
    if semantic_profile_registry_hash(registry).as_deref() != Ok(expected_root_hash) {
        return Err(failure(
            SemanticRegistryValidationPhase::RegistryHash,
            SemanticRegistryErrorCode::RegistryHash,
        ));
    }

    let identity = parse_registry_identity(registry).expect("validated root identity");
    if identity != expected_revision.identity() {
        return Err(failure(
            SemanticRegistryValidationPhase::EmbeddedIdentity,
            SemanticRegistryErrorCode::RegistryAssertion,
        ));
    }

    let mut expected_transport = canonical;
    expected_transport.push(b'\n');
    if transport != expected_transport {
        return Err(failure(
            SemanticRegistryValidationPhase::CanonicalTransport,
            SemanticRegistryErrorCode::RegistryCanonical,
        ));
    }

    let entries = profiles
        .iter()
        .map(parse_validated_entry)
        .collect::<Option<Vec<_>>>()
        .expect("entry invariants validated");
    Ok(ValidatedSemanticProfileRegistry {
        revision: expected_revision,
        identity,
        entries,
    })
}

fn parse_validated_entry(entry: &Value) -> Option<SemanticProfileEntry> {
    let profile = CompiledSemanticProfile::from_identity(
        string(get(entry, "source_language"))?,
        string(get(entry, "semantic_profile"))?,
    )?;
    let contracts = CONTRACT_FIELDS
        .map(|field| string(get(get(entry, "contracts"), field.as_str())).map(str::to_owned));
    Some(SemanticProfileEntry {
        source_language: profile.source_language().to_owned(),
        semantic_profile: profile.semantic_profile().to_owned(),
        semantic_parameters_schema: profile.semantic_parameters_schema().to_owned(),
        selection_schema: profile.selection_schema().to_owned(),
        contracts: CompiledContracts {
            values: contracts
                .into_iter()
                .collect::<Option<Vec<_>>>()?
                .try_into()
                .ok()?,
        },
        entry_sha256: string(get(entry, "entry_sha256"))?.to_owned(),
        compiled_profile: profile,
        canonical_json: canonical_value(entry)?,
    })
}

fn entry_invariants_hold(entry: &Value) -> bool {
    let Some(profile) = CompiledSemanticProfile::from_identity(
        string(get(entry, "source_language")).unwrap_or_default(),
        string(get(entry, "semantic_profile")).unwrap_or_default(),
    ) else {
        return false;
    };
    if string(get(entry, "semantic_parameters_schema"))
        != Some(profile.semantic_parameters_schema())
        || string(get(entry, "selection_schema")) != Some(profile.selection_schema())
        || string(get(entry, "entry_sha256")) != Some(profile.entry_sha256())
    {
        return false;
    }
    CONTRACT_FIELDS.into_iter().all(|field| {
        string(get(get(entry, "contracts"), field.as_str()))
            == Some(CompiledProfileContract { profile, field }.contract_id())
    })
}

fn validate_parameter_value(contract: CompiledParameterContract, value: &Value) -> bool {
    match contract {
        CompiledParameterContract::GoFixedV0 => {
            has_exact_fields(value, &["target_id", "pointer_width"])
                && string(get(value, "target_id")) == Some("linux/amd64")
                && unsigned_integer(get(value, "pointer_width")) == Some(64)
        }
        CompiledParameterContract::RustCheckedV0 => {
            has_exact_fields(
                value,
                &["target_id", "pointer_width", "overflow_mode", "panic_mode"],
            ) && matches!(
                (
                    string(get(value, "target_id")),
                    unsigned_integer(get(value, "pointer_width")),
                ),
                (Some("i686-unknown-linux-gnu"), Some(32))
                    | (Some("x86_64-unknown-linux-gnu"), Some(64))
            ) && string(get(value, "overflow_mode")) == Some("checked")
                && string(get(value, "panic_mode")) == Some("abort")
        }
        CompiledParameterContract::CSharpScalarV0 => {
            value
                == &serde_json::json!({
                    "check_overflow_default": false,
                    "documentation_mode": "none",
                    "language_version": "14.0",
                    "nullable_context": "disable",
                    "optimization": "release",
                    "platform": "x64",
                    "pointer_width": 64,
                    "preprocessor_symbols": [],
                    "source_kind": "regular",
                    "target_framework": "net10.0",
                    "target_id": "linux-x64",
                    "unsafe": false
                })
        }
    }
}

fn validate_selection_value(contract: CompiledSelectionContract, value: &Value) -> bool {
    match contract {
        CompiledSelectionContract::GoFunctionV0 => {
            if !has_exact_fields(value, &["package", "function"]) {
                return false;
            }
            let Some(package) = string(get(value, "package")) else {
                return false;
            };
            let Some(function) = string(get(value, "function")) else {
                return false;
            };
            !package.is_empty()
                && function
                    .strip_prefix(package)
                    .is_some_and(|suffix| suffix.starts_with('.') && suffix.len() > 1)
        }
        CompiledSelectionContract::RustFunctionV0 => {
            if !has_exact_fields(value, &["package", "crate", "kind", "function"]) {
                return false;
            }
            let Some(package) = string(get(value, "package")) else {
                return false;
            };
            let Some(crate_name) = string(get(value, "crate")) else {
                return false;
            };
            let Some(kind) = string(get(value, "kind")) else {
                return false;
            };
            let Some(function) = string(get(value, "function")) else {
                return false;
            };
            !package.is_empty()
                && !crate_name.is_empty()
                && matches!(kind, "lib" | "bin")
                && function
                    .strip_prefix(crate_name)
                    .is_some_and(|suffix| suffix.starts_with("::") && suffix.len() > 2)
        }
        CompiledSelectionContract::CSharpMethodsV0 => {
            has_exact_fields(value, &["compilation", "contracts", "methods", "sources"])
                && string(get(value, "compilation")).is_some_and(|item| !item.is_empty())
                && nonempty_string_array(get(value, "contracts"))
                && nonempty_string_array(get(value, "methods"))
                && nonempty_string_array(get(value, "sources"))
        }
    }
}

fn validate_profile_payload(contract: CompiledProfileContract, value: &Value) -> bool {
    use CompiledSemanticProfile::{CSharpScalarV0, GoFixedV0, RustCheckedV0};
    use ProfileContractField::{
        Ai, Evidence, Frontend, Manifest, Policy, Release, SourceMap, Vc, Vir,
    };
    match (contract.profile, contract.field) {
        (GoFixedV0, Frontend) => {
            value
                == &serde_json::json!({
                    "limit_profile_id": "mpk.vir.limits.v0",
                    "environment_profile_id": "mpk.go.frontend_environment.v0",
                    "argument_profile_id": "mpk.go.frontend_arguments.v0"
                })
        }
        (RustCheckedV0, Frontend) => {
            value
                == &serde_json::json!({
                    "limit_profile_id": "mpk.vir.limits.v0",
                    "environment_profile_id": "mpk.rust.frontend_environment.v0",
                    "argument_profile_id": "mpk.rust.frontend_arguments.v0"
                })
        }
        (CSharpScalarV0, Ai) => {
            value
                == &serde_json::json!({
                    "display_language": "C#",
                    "projection_profile_id": "mpk.csharp.ai_projection.v0",
                    "proof_authority": false,
                    "redaction_profile_id": "minimal-v1",
                    "source_access": false
                })
        }
        (GoFixedV0, Ai) => {
            value
                == &serde_json::json!({
                    "display_language": "Go",
                    "projection_profile_id": "mpk.go.ai_projection.v0",
                    "proof_authority": false,
                    "redaction_profile_id": "minimal-v1",
                    "source_access": false
                })
        }
        (RustCheckedV0, Ai) => {
            value
                == &serde_json::json!({
                    "display_language": "Rust",
                    "projection_profile_id": "mpk.rust.ai_projection.v0",
                    "proof_authority": false,
                    "redaction_profile_id": "minimal-v1",
                    "source_access": false
                })
        }
        (CSharpScalarV0, Evidence) => {
            value
                == &serde_json::json!({
                    "proof_authority": "certificate_only",
                    "recipe_profile_id": "mpk.csharp.evidence_recipe.v0",
                    "require_reference_checker": true,
                    "require_source_free_check": true
                })
        }
        (GoFixedV0, Evidence) => {
            value
                == &serde_json::json!({
                    "proof_authority": "certificate_only",
                    "recipe_profile_id": "mpk.go.evidence_recipe.v0",
                    "require_reference_checker": true,
                    "require_source_free_check": true
                })
        }
        (RustCheckedV0, Evidence) => {
            value
                == &serde_json::json!({
                    "proof_authority": "certificate_only",
                    "recipe_profile_id": "mpk.rust.evidence_recipe.v0",
                    "require_reference_checker": true,
                    "require_source_free_check": true
                })
        }
        (CSharpScalarV0, Frontend) => {
            value
                == &serde_json::json!({
                    "argument_profile_id": "mpk.csharp.frontend_arguments.v0",
                    "environment_profile_id": "mpk.csharp.frontend_environment.v0",
                    "launcher_profile_id": "mpk.csharp.dotnet_launcher.v0",
                    "limit_profile_id": "mpk.csharp.limits.v0",
                    "private_driver": "none"
                })
        }
        (CSharpScalarV0, Manifest) => {
            value
                == &serde_json::json!({
                    "input_kinds": ["contract", "source"],
                    "source_extension": ".cs",
                    "unit_kind": "compilation"
                })
        }
        (CSharpScalarV0, Policy) => {
            value
                == &serde_json::json!({
                    "axiom_profile": "mvp-theory",
                    "checker_profile": "mvp-strict",
                    "strategy_profile": "payment-policy-csharp-alpha"
                })
        }
        (GoFixedV0, Policy) => {
            value
                == &serde_json::json!({
                    "axiom_profile": "zero-axiom",
                    "checker_profile": "mvp-strict",
                    "strategy_profile": "payment-policy-alpha"
                })
        }
        (RustCheckedV0, Policy) => {
            value
                == &serde_json::json!({
                    "axiom_profile": "mvp-theory",
                    "checker_profile": "mvp-strict",
                    "strategy_profile": "payment-policy-rust-alpha"
                })
        }
        (CSharpScalarV0, Release) => {
            value
                == &serde_json::json!({
                    "compiler_profile_id": "mpk.csharp.roslyn_5_6_0.v0",
                    "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
                    "reference_profile_id": "mpk.dotnet.netcore_ref_10_0_11.v0",
                    "runtime_layout_profile_id": "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
                    "runtime_profile_id": "mpk.dotnet.runtime_10_0_11.linux_x64.v0",
                    "toolchain_inputs_sha256": "d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f"
                })
        }
        (CSharpScalarV0, SourceMap) => {
            value
                == &serde_json::json!({
                    "encoding": "utf-8",
                    "offset_unit": "utf8-byte",
                    "synthetic_reasons": []
                })
        }
        (CSharpScalarV0, Vc) => {
            value
                == &serde_json::json!({
                    "contract_profile_id": "mpk.csharp.contract.v0",
                    "required_check_profile_id": "mpk.csharp.required_checks.v0",
                    "verification_limit_profile_id": "mpk.verify.limits.v0"
                })
        }
        (GoFixedV0, Vc) => {
            value
                == &serde_json::json!({
                    "contract_profile_id": "mpk.go.contract.v0",
                    "required_check_profile_id": "mpk.go.fixed.v0",
                    "verification_limit_profile_id": "mpk.verify.limits.v0"
                })
        }
        (RustCheckedV0, Vc) => {
            value
                == &serde_json::json!({
                    "contract_profile_id": "mpk.rust.contract.v0",
                    "required_check_profile_id": "mpk.rust.checked.v0",
                    "verification_limit_profile_id": "mpk.verify.limits.v0"
                })
        }
        (CSharpScalarV0, Vir) => {
            value
                == &serde_json::json!({
                    "operation_profile_id": "mpk.csharp.vir_operations.v0",
                    "source_map_profile_id": "mpk.csharp.source_map.v0",
                    "vir_limit_profile_id": "mpk.vir.limits.v0"
                })
        }
        (GoFixedV0, Release) => {
            value
                == &serde_json::json!({
                    "compiler": {
                        "kind": "go",
                        "release": "go1.25.0"
                    },
                    "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.v0",
                    "native_runtime": {
                        "kind": "none"
                    },
                    "target_libraries": [{
                        "component_name": "go-target-linux-amd64",
                        "content_sha256": "5380dbbaf794293606958f98f1e0f2fdab25826eba775801becb9159119b6f50",
                        "pointer_width": 64,
                        "target_id": "linux/amd64"
                    }]
                })
        }
        (RustCheckedV0, Release) => {
            value
                == &serde_json::json!({
                    "compiler": {
                        "kind": "rust",
                        "release": "1.89.0-nightly",
                        "rustc_commit": "4d08223c054cf5a56d9761ca925fd46ffebe7115"
                    },
                    "execution_host_profile_id": "mpk.host.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0",
                    "native_runtime": {
                        "component_name": "native-runtime",
                        "component_root": "native-runtime",
                        "kind": "component",
                        "layout_profile_id": "mpk.runtime.linux-x86_64-gnu.glibc2_27.cgroup2_tmpfs.v0"
                    },
                    "target_libraries": [
                        {
                            "component_name": "rust-target-i686",
                            "content_sha256": "8f606996b669eb0f4314309d145d93c6eeaad8b261791584387bcff46ccafb0a",
                            "pointer_width": 32,
                            "target_id": "i686-unknown-linux-gnu"
                        },
                        {
                            "component_name": "rust-target-x86_64",
                            "content_sha256": "d8c45533753e17186cefde3e0830f7b358a8b4c818eb732d8814a31861335a15",
                            "pointer_width": 64,
                            "target_id": "x86_64-unknown-linux-gnu"
                        }
                    ]
                })
        }
        (GoFixedV0, Manifest | SourceMap | Vir) | (RustCheckedV0, Manifest | SourceMap | Vir) => {
            // Migration owners have bound the admitted frontend and release
            // payloads above. The remaining recognized Go/Rust IDs stay
            // unavailable until their successor field owners bind exact
            // meanings; they never fall back to an untagged or dynamically
            // selected validator.
            false
        }
    }
}

fn semantic_context_shape_is_valid(context: &Value) -> bool {
    if !has_exact_fields(
        context,
        &[
            "profile_registry",
            "profile_entry_sha256",
            "source_language",
            "semantic_profile",
            "semantic_parameters",
        ],
    ) || !has_exact_fields(
        get(context, "profile_registry"),
        &["schema", "id", "revision", "registry_sha256"],
    ) || !has_exact_fields(get(context, "semantic_parameters"), &["schema", "value"])
    {
        return false;
    }
    let identity = get(context, "profile_registry");
    let parameters = get(context, "semantic_parameters");
    string(get(identity, "schema")).is_some()
        && string(get(identity, "id")).is_some()
        && unsigned_integer(get(identity, "revision")).is_some()
        && string(get(identity, "registry_sha256")).is_some()
        && string(get(context, "profile_entry_sha256")).is_some()
        && string(get(context, "source_language")).is_some()
        && string(get(context, "semantic_profile")).is_some()
        && string(get(parameters, "schema")).is_some()
        && get(parameters, "value").as_object().is_some()
}

fn selection_shape_is_valid(selection: &Value) -> bool {
    has_exact_fields(selection, &["schema", "value"])
        && string(get(selection, "schema")).is_some()
        && get(selection, "value").as_object().is_some()
}

fn parse_registry_identity(value: &Value) -> Option<ProfileRegistryIdentity> {
    Some(ProfileRegistryIdentity {
        schema: string(get(value, "schema"))?.to_owned(),
        id: string(get(value, "id"))?.to_owned(),
        revision: unsigned_integer(get(value, "revision"))?,
        registry_sha256: string(get(value, "registry_sha256"))?.to_owned(),
    })
}

fn hash_without_field(
    value: &Value,
    excluded: &str,
    domain: HashDomain,
    phase: SemanticRegistryValidationPhase,
    code: SemanticRegistryErrorCode,
) -> Result<String, SemanticRegistryValidationError> {
    if !value_within_preallocation_limit(value, REGISTRY_CANONICAL_BYTES_MAX) {
        return Err(failure(phase, code));
    }
    let strict = serde_to_strict(value).ok_or_else(|| failure(phase, code))?;
    let payload = strict
        .clone_without_fields(&[excluded])
        .map_err(|_| failure(phase, code))?;
    let canonical = canonical_json_bytes_bounded(
        &payload,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).expect("registry limit fits usize"),
    )
    .map_err(|_| failure(phase, code))?;
    hash_domain_separated_raw(domain, &canonical)
        .map(|digest| digest.to_hex())
        .map_err(|_| failure(phase, code))
}

fn canonical_value(value: &Value) -> Option<Vec<u8>> {
    if !value_within_preallocation_limit(value, REGISTRY_CANONICAL_BYTES_MAX) {
        return None;
    }
    canonical_json_bytes_bounded(
        &serde_to_strict(value)?,
        usize::try_from(REGISTRY_CANONICAL_BYTES_MAX).ok()?,
    )
    .ok()
}

fn canonical_size_within(value: &Value, maximum: u64) -> bool {
    if !value_within_preallocation_limit(value, maximum) {
        return false;
    }
    let Some(strict) = serde_to_strict(value) else {
        return false;
    };
    usize::try_from(maximum)
        .ok()
        .is_some_and(|maximum| canonical_json_bytes_bounded(&strict, maximum).is_ok())
}

fn value_within_preallocation_limit(value: &Value, maximum: u64) -> bool {
    fn visit(value: &Value, depth: u64, maximum: u64, observed: &mut u64) -> bool {
        let add = |observed: &mut u64, amount: u64| {
            observed
                .checked_add(amount)
                .filter(|next| *next <= maximum)
                .map(|next| *observed = next)
                .is_some()
        };
        if !add(observed, 1) {
            return false;
        }
        match value {
            Value::Null | Value::Bool(_) | Value::Number(_) => true,
            Value::String(value) => u64::try_from(value.len())
                .ok()
                .is_some_and(|length| add(observed, length)),
            Value::Array(values) => {
                let Some(depth) = depth.checked_add(1) else {
                    return false;
                };
                depth <= SEMANTIC_REGISTRY_JSON_NESTING_MAX
                    && values
                        .iter()
                        .all(|value| visit(value, depth, maximum, observed))
            }
            Value::Object(fields) => {
                let Some(depth) = depth.checked_add(1) else {
                    return false;
                };
                depth <= SEMANTIC_REGISTRY_JSON_NESTING_MAX
                    && fields.iter().all(|(name, value)| {
                        u64::try_from(name.len())
                            .ok()
                            .is_some_and(|length| add(observed, length))
                            && visit(value, depth, maximum, observed)
                    })
            }
        }
    }

    let mut observed = 0;
    visit(value, 0, maximum, &mut observed)
}

fn strict_to_serde(value: StrictJsonValue) -> Value {
    match value {
        StrictJsonValue::Null => Value::Null,
        StrictJsonValue::Bool(value) => Value::Bool(value),
        StrictJsonValue::Integer(value) => Value::Number(value.into()),
        StrictJsonValue::String(value) => Value::String(value),
        StrictJsonValue::Array(values) => {
            Value::Array(values.into_iter().map(strict_to_serde).collect())
        }
        StrictJsonValue::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(name, value)| (name, strict_to_serde(value)))
                .collect(),
        ),
    }
}

fn serde_to_strict(value: &Value) -> Option<StrictJsonValue> {
    match value {
        Value::Null => Some(StrictJsonValue::Null),
        Value::Bool(value) => Some(StrictJsonValue::Bool(*value)),
        Value::Number(value) => value.as_i64().map(StrictJsonValue::Integer),
        Value::String(value) => Some(StrictJsonValue::String(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(serde_to_strict)
            .collect::<Option<Vec<_>>>()
            .map(StrictJsonValue::Array),
        Value::Object(entries) => entries
            .iter()
            .map(|(name, value)| Some((name.clone(), serde_to_strict(value)?)))
            .collect::<Option<Vec<_>>>()
            .map(StrictJsonValue::Object),
    }
}

fn valid_identifier(value: &str, maximum: u64) -> bool {
    let Ok(length) = u64::try_from(value.len()) else {
        return false;
    };
    if value.is_empty() || length > maximum || !value.is_ascii() {
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn nonempty_string_array(value: &Value) -> bool {
    value.as_array().is_some_and(|items| {
        !items.is_empty()
            && items
                .iter()
                .all(|item| string(item).is_some_and(|text| !text.is_empty()))
    })
}

fn has_exact_fields(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|fields| {
        fields.len() == expected.len() && expected.iter().all(|name| fields.contains_key(*name))
    })
}

fn get<'a>(value: &'a Value, name: &str) -> &'a Value {
    value.get(name).unwrap_or(&Value::Null)
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("shape checked")
}

fn string(value: &Value) -> Option<&str> {
    value.as_str()
}

fn unsigned_integer(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .filter(|value| *value <= SEMANTIC_REGISTRY_REVISION_MAX)
}

const fn contract_index(field: ProfileContractField) -> usize {
    match field {
        ProfileContractField::Ai => 0,
        ProfileContractField::Evidence => 1,
        ProfileContractField::Frontend => 2,
        ProfileContractField::Manifest => 3,
        ProfileContractField::Policy => 4,
        ProfileContractField::Release => 5,
        ProfileContractField::SourceMap => 6,
        ProfileContractField::Vc => 7,
        ProfileContractField::Vir => 8,
    }
}

const fn failure(
    phase: SemanticRegistryValidationPhase,
    code: SemanticRegistryErrorCode,
) -> SemanticRegistryValidationError {
    SemanticRegistryValidationError::new(phase, code)
}
