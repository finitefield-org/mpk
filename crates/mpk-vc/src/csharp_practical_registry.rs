//! Candidate-only semantic registry and context for the practical C# profile.
//!
//! The installed release continues to use `semantic_profile_registry` and its
//! revision-3 v1 root. This module validates the closed v2/revision-4 candidate
//! only when a caller explicitly injects its bytes. It performs no discovery,
//! installation, negotiation, fallback, or public-route selection.

use crate::canonical_json::{
    canonical_json_bytes_bounded, parse_strict_json, StrictJsonLimits, StrictJsonValue,
};
use crate::hash::{hash_domain_separated_raw, HashDomain};
use crate::semantic_profile_registry::{
    CompiledSemanticProfile, RegistryRevision, ValidatedSemanticProfileRegistry,
    COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX, REGISTRY_CANONICAL_BYTES_MAX,
    REGISTRY_TRANSPORT_BYTES_MAX, SELECTION_CANONICAL_BYTES_MAX,
    SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX, SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX,
    SEMANTIC_REGISTRY_JSON_NESTING_MAX, SEMANTIC_REGISTRY_PROFILES_MAX,
    SEMANTIC_REGISTRY_REVISION_MAX, SOURCE_LANGUAGE_BYTES_MAX,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub const SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA: &str = "mpk.semantic_profile.registry.v2";
pub const SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA: &str = "mpk.semantic_profile.entry.v2";
pub const SUCCESSOR_SEMANTIC_REGISTRY_LIMIT_PROFILE: &str =
    "mpk.semantic_profile.registry.limits.v2";
pub const SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA: &str = "mpk.semantic_context.v2";
pub const SUCCESSOR_VALIDATED_REQUEST_SCHEMA: &str = "mpk.validated_semantic_request.v2";
pub const CSHARP_PRACTICAL_PARAMETERS_SCHEMA: &str = "mpk.semantic_parameters.csharp_practical.v1";
pub const CSHARP_PRACTICAL_SELECTION_SCHEMA: &str = "mpk.selection.csharp_members.v1";
pub const CSHARP_PRACTICAL_PROFILE: &str = "mpk.csharp.practical.v1";
pub const FOUNDATION_DESCRIPTOR_SCHEMA: &str = "mpk.csharp.foundation_descriptor.v1";
pub const FOUNDATION_DESCRIPTOR_ID: &str = "mpk.csharp.practical.foundation.v1";
pub const FOUNDATION_DESCRIPTOR_CONTENT_SHA256: &str =
    "d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1";
pub const SUCCESSOR_CANDIDATE_REVISION: u64 = 4;

pub const SUCCESSOR_PROFILE_ENTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SEMANTIC-PROFILE-ENTRY-2.0");
pub const SUCCESSOR_PROFILE_REGISTRY_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SEMANTIC-PROFILE-REGISTRY-2.0");
pub const SUCCESSOR_SEMANTIC_CONTEXT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-SEMANTIC-CONTEXT-2.0");
pub const SUCCESSOR_VALIDATED_REQUEST_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-VALIDATED-SEMANTIC-REQUEST-2.0");
pub const CSHARP_PRACTICAL_PARAMETERS_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-PRACTICAL-PARAMETERS-1.0");
pub const CSHARP_PRACTICAL_SELECTION_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-CSHARP-SELECTION-1.0");
pub const SUCCESSOR_COMPILED_PROFILE_CONTRACT_HASH_DOMAIN: HashDomain =
    HashDomain::new("MPK-COMPILED-PROFILE-CONTRACT-1.0");

// Frozen from the canonical candidate asserted by the W01 production-owner
// tests; validation also recomputes every digest before constructing a value.
pub const CSHARP_PRACTICAL_ENTRY_SHA256: &str =
    "9a5b4737e928a93dfa07f71e72d49181d32a84200e3e786fc3a8914914676661";
pub const SUCCESSOR_CSHARP_SCALAR_ENTRY_SHA256: &str =
    "ff99f04464d3485f7239460da0562b8b812abbf25577bbb35ce05f07c5273bc3";
pub const SUCCESSOR_GO_FIXED_ENTRY_SHA256: &str =
    "8fa92fb20f37a0aef96f496d68b8d6d62370be0ea25fb4590aa4bba716d0d986";
pub const SUCCESSOR_JAVA_SCALAR_ENTRY_SHA256: &str =
    "cf6a4b2432a15f89196d0469ef67729d2d9d9a97dd5596ed48c43b905fa6fd51";
pub const SUCCESSOR_RUST_CHECKED_ENTRY_SHA256: &str =
    "a224764969f554caadf8b205a9a5f34db833dbb622d306ba048fc6d854725c75";
pub const SUCCESSOR_CANDIDATE_REGISTRY_SHA256: &str =
    "1cad5b32ce432eac39655240a84ec83ba6f347c335452b5e143fca3ba2cb78c8";

pub const SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX: u64 = REGISTRY_CANONICAL_BYTES_MAX;
pub const SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX: u64 = REGISTRY_TRANSPORT_BYTES_MAX;
pub const SUCCESSOR_REGISTRY_JSON_NESTING_MAX: u64 = SEMANTIC_REGISTRY_JSON_NESTING_MAX;
pub const SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX: u64 = SEMANTIC_REGISTRY_IDENTIFIER_BYTES_MAX;
pub const SUCCESSOR_SOURCE_LANGUAGE_BYTES_MAX: u64 = SOURCE_LANGUAGE_BYTES_MAX;
pub const SUCCESSOR_REGISTRY_PROFILES_MAX: u64 = SEMANTIC_REGISTRY_PROFILES_MAX;
pub const SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX: u64 = SEMANTIC_PARAMETERS_CANONICAL_BYTES_MAX;
pub const SUCCESSOR_SELECTION_CANONICAL_BYTES_MAX: u64 = SELECTION_CANONICAL_BYTES_MAX;
pub const SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX: u64 =
    COMPILED_PROFILE_PAYLOAD_CANONICAL_BYTES_MAX;
pub const SUCCESSOR_REGISTRY_REVISION_MAX: u64 = SEMANTIC_REGISTRY_REVISION_MAX;

const ROOT_FIELDS: &[&str] = &["schema", "id", "revision", "profiles", "registry_sha256"];
const ENTRY_FIELDS: &[&str] = &[
    "schema",
    "source_language",
    "semantic_profile",
    "semantic_parameters_schema",
    "selection_schema",
    "foundation_descriptor",
    "contracts",
    "entry_sha256",
];
const CONTRACT_FIELD_NAMES: &[&str] = &[
    "ai",
    "evidence",
    "frontend",
    "manifest",
    "policy",
    "release",
    "source_map",
    "vc",
    "vir",
];
const REGISTRY_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX,
    SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX,
    SUCCESSOR_REGISTRY_JSON_NESTING_MAX,
    SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX,
);
const DOCUMENT_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
    SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
    SUCCESSOR_REGISTRY_JSON_NESTING_MAX,
    SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
);

pub const SUCCESSOR_REGISTRY_IDENTITIES: &[&str] = &[
    CSHARP_PRACTICAL_PROFILE,
    SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA,
    SUCCESSOR_SEMANTIC_REGISTRY_LIMIT_PROFILE,
    SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA,
];
pub const SUCCESSOR_REGISTRY_HASH_DOMAINS: &[&str] = &[
    "MPK-SEMANTIC-PROFILE-ENTRY-2.0",
    "MPK-SEMANTIC-PROFILE-REGISTRY-2.0",
];
pub const SUCCESSOR_CONTEXT_IDENTITIES: &[&str] = &[
    SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA,
    SUCCESSOR_VALIDATED_REQUEST_SCHEMA,
];
pub const SUCCESSOR_CONTEXT_HASH_DOMAINS: &[&str] = &[
    "MPK-SEMANTIC-CONTEXT-2.0",
    "MPK-VALIDATED-SEMANTIC-REQUEST-2.0",
];
pub const SUCCESSOR_PARAMETER_IDENTITIES: &[&str] = &[CSHARP_PRACTICAL_PARAMETERS_SCHEMA];
pub const SUCCESSOR_PARAMETER_HASH_DOMAINS: &[&str] = &["MPK-CSHARP-PRACTICAL-PARAMETERS-1.0"];
pub const SUCCESSOR_SELECTION_IDENTITIES: &[&str] = &[CSHARP_PRACTICAL_SELECTION_SCHEMA];
pub const SUCCESSOR_SELECTION_HASH_DOMAINS: &[&str] = &["MPK-CSHARP-SELECTION-1.0"];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SuccessorCompiledSemanticProfile {
    CSharpPracticalV1,
    CSharpScalarV0,
    GoFixedV0,
    JavaScalarV0,
    RustCheckedV0,
}

pub const SUCCESSOR_PROFILE_ORDER: [SuccessorCompiledSemanticProfile; 5] = [
    SuccessorCompiledSemanticProfile::CSharpPracticalV1,
    SuccessorCompiledSemanticProfile::CSharpScalarV0,
    SuccessorCompiledSemanticProfile::GoFixedV0,
    SuccessorCompiledSemanticProfile::JavaScalarV0,
    SuccessorCompiledSemanticProfile::RustCheckedV0,
];

impl SuccessorCompiledSemanticProfile {
    pub const fn source_language(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 | Self::CSharpScalarV0 => "csharp",
            Self::GoFixedV0 => "go",
            Self::JavaScalarV0 => "java",
            Self::RustCheckedV0 => "rust",
        }
    }

    pub const fn semantic_profile(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 => CSHARP_PRACTICAL_PROFILE,
            Self::CSharpScalarV0 => "mpk.csharp.scalar.v0",
            Self::GoFixedV0 => "mpk.go.fixed.v0",
            Self::JavaScalarV0 => "mpk.java.scalar.v0",
            Self::RustCheckedV0 => "mpk.rust.checked.v0",
        }
    }

    pub const fn profile_contract_stem(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 => "csharp_practical",
            Self::CSharpScalarV0 => "csharp_scalar",
            Self::GoFixedV0 => "go_fixed",
            Self::JavaScalarV0 => "java_scalar",
            Self::RustCheckedV0 => "rust_checked",
        }
    }

    pub const fn semantic_parameters_schema(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 => CSHARP_PRACTICAL_PARAMETERS_SCHEMA,
            Self::CSharpScalarV0 => "mpk.semantic_parameters.csharp_scalar.v0",
            Self::GoFixedV0 => "mpk.semantic_parameters.go_fixed.v0",
            Self::JavaScalarV0 => "mpk.semantic_parameters.java_scalar.v0",
            Self::RustCheckedV0 => "mpk.semantic_parameters.rust_checked.v0",
        }
    }

    pub const fn selection_schema(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 => CSHARP_PRACTICAL_SELECTION_SCHEMA,
            Self::CSharpScalarV0 => "mpk.selection.csharp_methods.v0",
            Self::GoFixedV0 => "mpk.selection.go_function.v0",
            Self::JavaScalarV0 => "mpk.selection.java_methods.v0",
            Self::RustCheckedV0 => "mpk.selection.rust_function.v0",
        }
    }

    pub const fn expected_entry_sha256(self) -> &'static str {
        match self {
            Self::CSharpPracticalV1 => CSHARP_PRACTICAL_ENTRY_SHA256,
            Self::CSharpScalarV0 => SUCCESSOR_CSHARP_SCALAR_ENTRY_SHA256,
            Self::GoFixedV0 => SUCCESSOR_GO_FIXED_ENTRY_SHA256,
            Self::JavaScalarV0 => SUCCESSOR_JAVA_SCALAR_ENTRY_SHA256,
            Self::RustCheckedV0 => SUCCESSOR_RUST_CHECKED_ENTRY_SHA256,
        }
    }

    pub fn from_identity(source_language: &str, semantic_profile: &str) -> Option<Self> {
        SUCCESSOR_PROFILE_ORDER.into_iter().find(|profile| {
            profile.source_language() == source_language
                && profile.semantic_profile() == semantic_profile
        })
    }

    pub fn from_semantic_profile(semantic_profile: &str) -> Option<Self> {
        SUCCESSOR_PROFILE_ORDER
            .into_iter()
            .find(|profile| profile.semantic_profile() == semantic_profile)
    }

    pub fn from_parameters_schema(schema: &str) -> Option<Self> {
        SUCCESSOR_PROFILE_ORDER
            .into_iter()
            .find(|profile| profile.semantic_parameters_schema() == schema)
    }

    pub fn from_selection_schema(schema: &str) -> Option<Self> {
        SUCCESSOR_PROFILE_ORDER
            .into_iter()
            .find(|profile| profile.selection_schema() == schema)
    }

    pub const fn predecessor_profile(self) -> Option<CompiledSemanticProfile> {
        match self {
            Self::CSharpPracticalV1 => None,
            Self::CSharpScalarV0 => Some(CompiledSemanticProfile::CSharpScalarV0),
            Self::GoFixedV0 => Some(CompiledSemanticProfile::GoFixedV0),
            Self::JavaScalarV0 => Some(CompiledSemanticProfile::JavaScalarV0),
            Self::RustCheckedV0 => Some(CompiledSemanticProfile::RustCheckedV0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SuccessorProfileContractField {
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

pub const SUCCESSOR_CONTRACT_FIELDS: [SuccessorProfileContractField; 9] = [
    SuccessorProfileContractField::Ai,
    SuccessorProfileContractField::Evidence,
    SuccessorProfileContractField::Frontend,
    SuccessorProfileContractField::Manifest,
    SuccessorProfileContractField::Policy,
    SuccessorProfileContractField::Release,
    SuccessorProfileContractField::SourceMap,
    SuccessorProfileContractField::Vc,
    SuccessorProfileContractField::Vir,
];

impl SuccessorProfileContractField {
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
        SUCCESSOR_CONTRACT_FIELDS
            .into_iter()
            .find(|field| field.as_str() == value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SuccessorProfileContract {
    profile: SuccessorCompiledSemanticProfile,
    field: SuccessorProfileContractField,
}

impl SuccessorProfileContract {
    pub const fn new(
        profile: SuccessorCompiledSemanticProfile,
        field: SuccessorProfileContractField,
    ) -> Self {
        Self { profile, field }
    }

    pub const fn profile(self) -> SuccessorCompiledSemanticProfile {
        self.profile
    }

    pub const fn field(self) -> SuccessorProfileContractField {
        self.field
    }

    pub fn contract_id(self) -> String {
        format!(
            "mpk.profile.{}.{}.v1",
            self.field.as_str(),
            self.profile.profile_contract_stem()
        )
    }

    pub fn from_contract_id(value: &str) -> Option<Self> {
        successor_profile_contracts().find(|contract| contract.contract_id() == value)
    }
}

pub fn successor_profile_contracts() -> impl Iterator<Item = SuccessorProfileContract> {
    SUCCESSOR_CONTRACT_FIELDS.into_iter().flat_map(|field| {
        SUCCESSOR_PROFILE_ORDER
            .into_iter()
            .map(move |profile| SuccessorProfileContract::new(profile, field))
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CSharpFrontendDispatch {
    ScalarV0,
    PracticalV1,
}

pub const fn csharp_frontend_dispatch(
    profile: SuccessorCompiledSemanticProfile,
) -> Option<CSharpFrontendDispatch> {
    match profile {
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => Some(CSharpFrontendDispatch::ScalarV0),
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => {
            Some(CSharpFrontendDispatch::PracticalV1)
        }
        SuccessorCompiledSemanticProfile::GoFixedV0
        | SuccessorCompiledSemanticProfile::JavaScalarV0
        | SuccessorCompiledSemanticProfile::RustCheckedV0 => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FoundationDescriptorRef {
    schema: String,
    id: String,
    content_sha256: String,
}

impl FoundationDescriptorRef {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorRegistryIdentity {
    schema: String,
    id: String,
    revision: u64,
    registry_sha256: String,
}

impl SuccessorRegistryIdentity {
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
pub struct SuccessorCompiledContracts {
    values: [String; 9],
}

impl SuccessorCompiledContracts {
    pub fn contract_id(&self, field: SuccessorProfileContractField) -> &str {
        &self.values[contract_index(field)]
    }

    pub fn iter(&self) -> impl Iterator<Item = (SuccessorProfileContractField, &str)> {
        SUCCESSOR_CONTRACT_FIELDS
            .into_iter()
            .map(|field| (field, self.contract_id(field)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorProfileEntry {
    source_language: String,
    semantic_profile: String,
    semantic_parameters_schema: String,
    selection_schema: String,
    foundation_descriptor: FoundationDescriptorRef,
    contracts: SuccessorCompiledContracts,
    entry_sha256: String,
    compiled_profile: SuccessorCompiledSemanticProfile,
    canonical_json: Vec<u8>,
}

impl SuccessorProfileEntry {
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

    pub fn foundation_descriptor(&self) -> &FoundationDescriptorRef {
        &self.foundation_descriptor
    }

    pub fn contracts(&self) -> &SuccessorCompiledContracts {
        &self.contracts
    }

    pub fn entry_sha256(&self) -> &str {
        &self.entry_sha256
    }

    pub const fn compiled_profile(&self) -> SuccessorCompiledSemanticProfile {
        self.compiled_profile
    }

    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedSuccessorRegistry {
    identity: SuccessorRegistryIdentity,
    entries: Vec<SuccessorProfileEntry>,
}

impl ValidatedSuccessorRegistry {
    pub fn identity(&self) -> &SuccessorRegistryIdentity {
        &self.identity
    }

    pub fn entries(&self) -> &[SuccessorProfileEntry] {
        &self.entries
    }

    pub fn lookup(
        &self,
        source_language: &str,
        semantic_profile: &str,
    ) -> Option<&SuccessorProfileEntry> {
        self.entries.iter().find(|entry| {
            entry.source_language == source_language && entry.semantic_profile == semantic_profile
        })
    }

    pub fn lookup_entry_hash(&self, entry_sha256: &str) -> Option<&SuccessorProfileEntry> {
        self.entries
            .iter()
            .find(|entry| entry.entry_sha256 == entry_sha256)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorSemanticParameters {
    schema: String,
    value: Value,
}

impl SuccessorSemanticParameters {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SuccessorSemanticContext {
    schema: String,
    profile_registry: SuccessorRegistryIdentity,
    profile_entry_sha256: String,
    source_language: String,
    semantic_profile: String,
    semantic_parameters: SuccessorSemanticParameters,
    foundation_descriptor: FoundationDescriptorRef,
}

impl SuccessorSemanticContext {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn profile_registry(&self) -> &SuccessorRegistryIdentity {
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

    pub fn semantic_parameters(&self) -> &SuccessorSemanticParameters {
        &self.semantic_parameters
    }

    pub fn foundation_descriptor(&self) -> &FoundationDescriptorRef {
        &self.foundation_descriptor
    }

    pub fn compiled_profile(&self) -> SuccessorCompiledSemanticProfile {
        SuccessorCompiledSemanticProfile::from_identity(
            &self.source_language,
            &self.semantic_profile,
        )
        .expect("validated successor context has a compiled profile")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidatedSuccessorRequest {
    schema: String,
    semantic_context: SuccessorSemanticContext,
    selection: Value,
    request_sha256: String,
}

impl ValidatedSuccessorRequest {
    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn semantic_context(&self) -> &SuccessorSemanticContext {
        &self.semantic_context
    }

    pub fn selection(&self) -> &Value {
        &self.selection
    }

    pub fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub fn compiled_profile(&self) -> SuccessorCompiledSemanticProfile {
        self.semantic_context.compiled_profile()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundSuccessorProfileEnvelope {
    profile_entry_sha256: String,
    contract: SuccessorProfileContract,
    value: Value,
    envelope_sha256: String,
}

impl BoundSuccessorProfileEnvelope {
    pub fn profile_entry_sha256(&self) -> &str {
        &self.profile_entry_sha256
    }

    pub const fn contract(&self) -> SuccessorProfileContract {
        self.contract
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn envelope_sha256(&self) -> &str {
        &self.envelope_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorRegistryValidationPhase {
    Transport,
    Shape,
    Scalar,
    Limits,
    Order,
    EntryHash,
    ContractBinding,
    FoundationBinding,
    RegistryHash,
    CanonicalTransport,
    InstalledIdentity,
    ProfileLookup,
    ProfileEntry,
    ParametersSchema,
    ParametersValue,
    SelectionSchema,
    SelectionValue,
    ContextLinkage,
    RequestHash,
    ProfileEnvelope,
    PredecessorProjection,
}

impl SuccessorRegistryValidationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Shape => "shape",
            Self::Scalar => "scalar",
            Self::Limits => "limits",
            Self::Order => "order",
            Self::EntryHash => "entry_hash",
            Self::ContractBinding => "contract_binding",
            Self::FoundationBinding => "foundation_binding",
            Self::RegistryHash => "registry_hash",
            Self::CanonicalTransport => "canonical_transport",
            Self::InstalledIdentity => "installed_identity",
            Self::ProfileLookup => "profile_lookup",
            Self::ProfileEntry => "profile_entry",
            Self::ParametersSchema => "parameters_schema",
            Self::ParametersValue => "parameters_value",
            Self::SelectionSchema => "selection_schema",
            Self::SelectionValue => "selection_value",
            Self::ContextLinkage => "context_linkage",
            Self::RequestHash => "request_hash",
            Self::ProfileEnvelope => "profile_envelope",
            Self::PredecessorProjection => "predecessor_projection",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorRegistryErrorCode {
    Transport,
    Shape,
    Scalar,
    Limit,
    Order,
    EntryHash,
    Contract,
    Foundation,
    RegistryHash,
    Canonical,
    InstalledIdentity,
    ContextMismatch,
    ProfileEntryMismatch,
    ParametersMismatch,
    SelectionMismatch,
    RequestHash,
    ProfileEnvelope,
    ContextLinkage,
    PredecessorProjection,
}

impl SuccessorRegistryErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "CSHARP_PRACTICAL_REGISTRY_TRANSPORT",
            Self::Shape => "CSHARP_PRACTICAL_REGISTRY_SHAPE",
            Self::Scalar => "CSHARP_PRACTICAL_REGISTRY_SCALAR",
            Self::Limit => "CSHARP_PRACTICAL_REGISTRY_LIMIT",
            Self::Order => "CSHARP_PRACTICAL_REGISTRY_ORDER",
            Self::EntryHash => "CSHARP_PRACTICAL_REGISTRY_ENTRY_HASH",
            Self::Contract => "CSHARP_PRACTICAL_REGISTRY_CONTRACT",
            Self::Foundation => "CSHARP_PRACTICAL_FOUNDATION",
            Self::RegistryHash => "CSHARP_PRACTICAL_REGISTRY_HASH",
            Self::Canonical => "CSHARP_PRACTICAL_REGISTRY_CANONICAL",
            Self::InstalledIdentity => "CSHARP_PRACTICAL_INSTALLED_IDENTITY",
            Self::ContextMismatch => "CSHARP_PRACTICAL_CONTEXT_MISMATCH",
            Self::ProfileEntryMismatch => "CSHARP_PRACTICAL_PROFILE_ENTRY_MISMATCH",
            Self::ParametersMismatch => "CSHARP_PRACTICAL_PARAMETERS_MISMATCH",
            Self::SelectionMismatch => "CSHARP_PRACTICAL_SELECTION_MISMATCH",
            Self::RequestHash => "CSHARP_PRACTICAL_REQUEST_HASH",
            Self::ProfileEnvelope => "CSHARP_PRACTICAL_PROFILE_ENVELOPE",
            Self::ContextLinkage => "CSHARP_PRACTICAL_CONTEXT_LINKAGE",
            Self::PredecessorProjection => "CSHARP_PRACTICAL_PREDECESSOR_PROJECTION",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorRegistryValidationError {
    phase: SuccessorRegistryValidationPhase,
    code: SuccessorRegistryErrorCode,
}

impl SuccessorRegistryValidationError {
    pub const fn phase(&self) -> SuccessorRegistryValidationPhase {
        self.phase
    }

    pub const fn code(&self) -> SuccessorRegistryErrorCode {
        self.code
    }
}

impl fmt::Display for SuccessorRegistryValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at practical-registry phase {}",
            self.code.as_str(),
            self.phase.as_str()
        )
    }
}

impl Error for SuccessorRegistryValidationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorRegistryLimit {
    RegistryCanonicalBytes,
    RegistryTransportBytes,
    JsonNesting,
    IdentifierBytes,
    SourceLanguageBytes,
    Profiles,
    SemanticParametersCanonicalBytes,
    SelectionCanonicalBytes,
    CompiledProfileCanonicalBytes,
    Revision,
}

impl SuccessorRegistryLimit {
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
            "compiled_profile_payload_canonical_bytes" => Some(Self::CompiledProfileCanonicalBytes),
            "revision" => Some(Self::Revision),
            _ => None,
        }
    }

    pub const fn inclusive_maximum(self) -> u64 {
        match self {
            Self::RegistryCanonicalBytes => SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX,
            Self::RegistryTransportBytes => SUCCESSOR_REGISTRY_TRANSPORT_BYTES_MAX,
            Self::JsonNesting => SUCCESSOR_REGISTRY_JSON_NESTING_MAX,
            Self::IdentifierBytes => SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX,
            Self::SourceLanguageBytes => SUCCESSOR_SOURCE_LANGUAGE_BYTES_MAX,
            Self::Profiles => SUCCESSOR_REGISTRY_PROFILES_MAX,
            Self::SemanticParametersCanonicalBytes => SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX,
            Self::SelectionCanonicalBytes => SUCCESSOR_SELECTION_CANONICAL_BYTES_MAX,
            Self::CompiledProfileCanonicalBytes => SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
            Self::Revision => SUCCESSOR_REGISTRY_REVISION_MAX,
        }
    }
}

pub fn validate_successor_registry_limit(
    limit: SuccessorRegistryLimit,
    value: u64,
) -> Result<(), SuccessorRegistryValidationError> {
    if value <= limit.inclusive_maximum() {
        Ok(())
    } else {
        Err(failure(
            SuccessorRegistryValidationPhase::Limits,
            SuccessorRegistryErrorCode::Limit,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessorRegistryDocumentKind {
    PracticalParameterValues,
    FoundationDescriptorRef,
    SemanticParametersEnvelope,
    RegistryIdentity,
    PracticalSelection,
    SemanticContext,
    PracticalParameters,
    ValidatedRequest,
}

pub fn canonical_successor_registry_transport(
    registry: &Value,
) -> Result<Vec<u8>, SuccessorRegistryValidationError> {
    let mut bytes =
        canonical_value(registry, SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX).ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::Limits,
                SuccessorRegistryErrorCode::Limit,
            )
        })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn successor_profile_entry_hash(
    entry: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_without_field(
        entry,
        "entry_sha256",
        SUCCESSOR_PROFILE_ENTRY_HASH_DOMAIN,
        SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::EntryHash,
        SuccessorRegistryErrorCode::EntryHash,
    )
}

pub fn successor_profile_registry_hash(
    registry: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_without_field(
        registry,
        "registry_sha256",
        SUCCESSOR_PROFILE_REGISTRY_HASH_DOMAIN,
        SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::RegistryHash,
        SuccessorRegistryErrorCode::RegistryHash,
    )
}

pub fn csharp_practical_selection_hash(
    selection: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_without_field(
        selection,
        "selection_sha256",
        CSHARP_PRACTICAL_SELECTION_HASH_DOMAIN,
        SUCCESSOR_SELECTION_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::SelectionValue,
        SuccessorRegistryErrorCode::SelectionMismatch,
    )
}

pub fn successor_validated_request_hash(
    request: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_without_field(
        request,
        "request_sha256",
        SUCCESSOR_VALIDATED_REQUEST_HASH_DOMAIN,
        SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::RequestHash,
        SuccessorRegistryErrorCode::RequestHash,
    )
}

pub fn successor_semantic_context_hash(
    context: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_complete(
        context,
        SUCCESSOR_SEMANTIC_CONTEXT_HASH_DOMAIN,
        SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::ContextLinkage,
        SuccessorRegistryErrorCode::ContextLinkage,
    )
}

pub fn csharp_practical_parameters_hash(
    parameters: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_complete(
        parameters,
        CSHARP_PRACTICAL_PARAMETERS_HASH_DOMAIN,
        SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::ParametersValue,
        SuccessorRegistryErrorCode::ParametersMismatch,
    )
}

pub fn successor_compiled_profile_contract_hash(
    envelope: &Value,
) -> Result<String, SuccessorRegistryValidationError> {
    hash_complete(
        envelope,
        SUCCESSOR_COMPILED_PROFILE_CONTRACT_HASH_DOMAIN,
        SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX,
        SuccessorRegistryValidationPhase::ProfileEnvelope,
        SuccessorRegistryErrorCode::ProfileEnvelope,
    )
}

pub fn validate_candidate_successor_registry(
    transport: &[u8],
) -> Result<ValidatedSuccessorRegistry, SuccessorRegistryValidationError> {
    validate_successor_registry_limit(
        SuccessorRegistryLimit::RegistryTransportBytes,
        u64::try_from(transport.len()).unwrap_or(u64::MAX),
    )
    .map_err(|_| {
        failure(
            SuccessorRegistryValidationPhase::Transport,
            SuccessorRegistryErrorCode::Transport,
        )
    })?;
    let parsed = parse_strict_json(transport, REGISTRY_LIMITS).map_err(|_| {
        failure(
            SuccessorRegistryValidationPhase::Transport,
            SuccessorRegistryErrorCode::Transport,
        )
    })?;
    let registry = strict_to_serde(parsed);
    validate_candidate_registry_value(&registry, transport)
}

pub fn validate_successor_registry_document(
    kind: SuccessorRegistryDocumentKind,
    transport: &[u8],
) -> Result<(), SuccessorRegistryValidationError> {
    let value = parse_canonical_document(transport)?;
    let valid = match kind {
        SuccessorRegistryDocumentKind::PracticalParameterValues => {
            valid_practical_parameter_values(&value)
        }
        SuccessorRegistryDocumentKind::FoundationDescriptorRef => {
            parse_foundation_ref(&value).is_some()
        }
        SuccessorRegistryDocumentKind::SemanticParametersEnvelope => {
            semantic_parameters_document_is_valid(&value)
        }
        SuccessorRegistryDocumentKind::RegistryIdentity => {
            parse_registry_identity(&value).is_some()
        }
        SuccessorRegistryDocumentKind::PracticalSelection => {
            validate_practical_selection_value(&value).is_ok()
        }
        SuccessorRegistryDocumentKind::SemanticContext => semantic_context_shape_is_valid(&value),
        SuccessorRegistryDocumentKind::PracticalParameters => {
            practical_parameters_document_is_valid(&value)
        }
        SuccessorRegistryDocumentKind::ValidatedRequest => request_document_is_valid(&value),
    };
    if valid {
        Ok(())
    } else {
        Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ))
    }
}

pub fn validate_successor_semantic_context(
    registry: &ValidatedSuccessorRegistry,
    transport: &[u8],
) -> Result<SuccessorSemanticContext, SuccessorRegistryValidationError> {
    let value = parse_canonical_document(transport)?;
    validate_context_value(registry, &value)
}

pub fn validate_successor_semantic_request(
    registry: &ValidatedSuccessorRegistry,
    transport: &[u8],
) -> Result<ValidatedSuccessorRequest, SuccessorRegistryValidationError> {
    let request = parse_canonical_document(transport)?;
    if !request_shape_is_valid(&request)
        || string(get(&request, "schema")) != Some(SUCCESSOR_VALIDATED_REQUEST_SCHEMA)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }
    let semantic_context = validate_context_value(registry, get(&request, "semantic_context"))?;
    validate_selection_value(registry, &semantic_context, get(&request, "selection"))?;
    let actual_hash = string(get(&request, "request_sha256")).expect("shape checked");
    if !valid_sha256(actual_hash) || successor_validated_request_hash(&request)? != actual_hash {
        return Err(failure(
            SuccessorRegistryValidationPhase::RequestHash,
            SuccessorRegistryErrorCode::RequestHash,
        ));
    }
    Ok(ValidatedSuccessorRequest {
        schema: SUCCESSOR_VALIDATED_REQUEST_SCHEMA.to_owned(),
        semantic_context,
        selection: get(&request, "selection").clone(),
        request_sha256: actual_hash.to_owned(),
    })
}

/// Validates only the common, context-bound compiled-profile envelope.
///
/// The profile-field owner must separately validate `value` before use. W01
/// deliberately does not make an unimplemented downstream payload admissible.
pub fn bind_successor_compiled_profile_envelope(
    registry: &ValidatedSuccessorRegistry,
    context: &SuccessorSemanticContext,
    expected_field: SuccessorProfileContractField,
    transport: &[u8],
) -> Result<BoundSuccessorProfileEnvelope, SuccessorRegistryValidationError> {
    let envelope = parse_canonical_document(transport)?;
    if !has_exact_fields(&envelope, &["profile_entry_sha256", "contract_id", "value"])
        || string(get(&envelope, "profile_entry_sha256")).is_none()
        || string(get(&envelope, "contract_id")).is_none()
        || get(&envelope, "value").as_object().is_none()
        || !canonical_size_within(&envelope, SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::ProfileEnvelope,
            SuccessorRegistryErrorCode::ProfileEnvelope,
        ));
    }
    if context.profile_registry() != registry.identity() {
        return Err(failure(
            SuccessorRegistryValidationPhase::InstalledIdentity,
            SuccessorRegistryErrorCode::InstalledIdentity,
        ));
    }
    let entry_hash = string(get(&envelope, "profile_entry_sha256")).expect("shape checked");
    if entry_hash != context.profile_entry_sha256() {
        return Err(failure(
            SuccessorRegistryValidationPhase::ProfileEntry,
            SuccessorRegistryErrorCode::ProfileEntryMismatch,
        ));
    }
    let entry = registry.lookup_entry_hash(entry_hash).ok_or_else(|| {
        failure(
            SuccessorRegistryValidationPhase::ProfileEntry,
            SuccessorRegistryErrorCode::ProfileEntryMismatch,
        )
    })?;
    let contract = SuccessorProfileContract::new(entry.compiled_profile(), expected_field);
    let contract_id = string(get(&envelope, "contract_id")).expect("shape checked");
    if entry.contracts().contract_id(expected_field) != contract_id
        || contract.contract_id() != contract_id
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::ContractBinding,
            SuccessorRegistryErrorCode::Contract,
        ));
    }
    let envelope_sha256 = successor_compiled_profile_contract_hash(&envelope)?;
    Ok(BoundSuccessorProfileEnvelope {
        profile_entry_sha256: entry_hash.to_owned(),
        contract,
        value: get(&envelope, "value").clone(),
        envelope_sha256,
    })
}

pub fn validate_successor_context_linkage(
    left: &SuccessorSemanticContext,
    right: &SuccessorSemanticContext,
) -> Result<(), SuccessorRegistryValidationError> {
    if left == right {
        Ok(())
    } else {
        Err(failure(
            SuccessorRegistryValidationPhase::ContextLinkage,
            SuccessorRegistryErrorCode::ContextLinkage,
        ))
    }
}

pub fn validate_successor_predecessor_projection(
    predecessor: &ValidatedSemanticProfileRegistry,
    successor: &ValidatedSuccessorRegistry,
) -> Result<(), SuccessorRegistryValidationError> {
    let predecessor_is_active =
        predecessor.revision() == RegistryRevision::Revision3 && predecessor.entries().len() == 4;
    let successor_is_exact = successor.identity().revision() == SUCCESSOR_CANDIDATE_REVISION
        && successor.entries().len() == SUCCESSOR_PROFILE_ORDER.len();
    let retained = predecessor.entries().iter().all(|old| {
        successor
            .lookup(old.source_language(), old.semantic_profile())
            .is_some_and(|new| {
                new.semantic_parameters_schema() == old.semantic_parameters_schema()
                    && new.selection_schema() == old.selection_schema()
                    && new.compiled_profile().predecessor_profile() == Some(old.compiled_profile())
            })
    });
    let practical_is_new = predecessor
        .lookup("csharp", CSHARP_PRACTICAL_PROFILE)
        .is_none()
        && successor
            .lookup("csharp", CSHARP_PRACTICAL_PROFILE)
            .is_some();
    if predecessor_is_active && successor_is_exact && retained && practical_is_new {
        Ok(())
    } else {
        Err(failure(
            SuccessorRegistryValidationPhase::PredecessorProjection,
            SuccessorRegistryErrorCode::PredecessorProjection,
        ))
    }
}

fn validate_candidate_registry_value(
    registry: &Value,
    transport: &[u8],
) -> Result<ValidatedSuccessorRegistry, SuccessorRegistryValidationError> {
    if !has_exact_fields(registry, ROOT_FIELDS)
        || string(get(registry, "schema")).is_none()
        || string(get(registry, "id")).is_none()
        || unsigned_integer(get(registry, "revision")).is_none()
        || get(registry, "profiles").as_array().is_none()
        || string(get(registry, "registry_sha256")).is_none()
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }
    if string(get(registry, "schema")) != Some(SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA)
        || string(get(registry, "id")) != Some(SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA)
        || unsigned_integer(get(registry, "revision")) != Some(SUCCESSOR_CANDIDATE_REVISION)
        || !valid_sha256(string(get(registry, "registry_sha256")).expect("shape checked"))
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Scalar,
            SuccessorRegistryErrorCode::Scalar,
        ));
    }
    let profiles = array(get(registry, "profiles"));
    validate_successor_registry_limit(
        SuccessorRegistryLimit::Profiles,
        u64::try_from(profiles.len()).unwrap_or(u64::MAX),
    )?;
    if profiles.len() != SUCCESSOR_PROFILE_ORDER.len() {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }

    if !profiles.iter().all(candidate_entry_shape_is_valid) {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }

    let keys = profiles
        .iter()
        .map(|entry| {
            (
                string(get(entry, "source_language")).unwrap_or_default(),
                string(get(entry, "semantic_profile")).unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    if !keys.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(failure(
            SuccessorRegistryValidationPhase::Order,
            SuccessorRegistryErrorCode::Order,
        ));
    }

    let mut entries = Vec::with_capacity(profiles.len());
    for (index, value) in profiles.iter().enumerate() {
        entries.push(parse_candidate_entry(
            value,
            SUCCESSOR_PROFILE_ORDER[index],
        )?);
    }
    if entries
        .iter()
        .map(SuccessorProfileEntry::compiled_profile)
        .collect::<BTreeSet<_>>()
        != SUCCESSOR_PROFILE_ORDER.into_iter().collect::<BTreeSet<_>>()
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Order,
            SuccessorRegistryErrorCode::Order,
        ));
    }

    let registry_sha256 = string(get(registry, "registry_sha256")).expect("shape checked");
    if successor_profile_registry_hash(registry)? != registry_sha256
        || registry_sha256 != SUCCESSOR_CANDIDATE_REGISTRY_SHA256
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::RegistryHash,
            SuccessorRegistryErrorCode::RegistryHash,
        ));
    }
    if canonical_successor_registry_transport(registry)? != transport {
        return Err(failure(
            SuccessorRegistryValidationPhase::CanonicalTransport,
            SuccessorRegistryErrorCode::Canonical,
        ));
    }
    Ok(ValidatedSuccessorRegistry {
        identity: SuccessorRegistryIdentity {
            schema: SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA.to_owned(),
            id: SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA.to_owned(),
            revision: SUCCESSOR_CANDIDATE_REVISION,
            registry_sha256: registry_sha256.to_owned(),
        },
        entries,
    })
}

fn parse_candidate_entry(
    entry: &Value,
    expected_profile: SuccessorCompiledSemanticProfile,
) -> Result<SuccessorProfileEntry, SuccessorRegistryValidationError> {
    if !candidate_entry_shape_is_valid(entry) {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }
    let source_language = string(get(entry, "source_language")).expect("shape checked");
    let semantic_profile = string(get(entry, "semantic_profile")).expect("shape checked");
    let parameters_schema =
        string(get(entry, "semantic_parameters_schema")).expect("shape checked");
    let selection_schema = string(get(entry, "selection_schema")).expect("shape checked");
    let entry_sha256 = string(get(entry, "entry_sha256")).expect("shape checked");
    if string(get(entry, "schema")) != Some(SUCCESSOR_SEMANTIC_REGISTRY_ENTRY_SCHEMA)
        || !valid_identifier(source_language, SUCCESSOR_SOURCE_LANGUAGE_BYTES_MAX)
        || !valid_identifier(semantic_profile, SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX)
        || !valid_identifier(parameters_schema, SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX)
        || !valid_identifier(selection_schema, SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX)
        || !valid_sha256(entry_sha256)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Scalar,
            SuccessorRegistryErrorCode::Scalar,
        ));
    }
    if SuccessorCompiledSemanticProfile::from_identity(source_language, semantic_profile)
        != Some(expected_profile)
        || parameters_schema != expected_profile.semantic_parameters_schema()
        || selection_schema != expected_profile.selection_schema()
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::ContractBinding,
            SuccessorRegistryErrorCode::Contract,
        ));
    }
    let foundation_descriptor = parse_foundation_ref(get(entry, "foundation_descriptor"))
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::FoundationBinding,
                SuccessorRegistryErrorCode::Foundation,
            )
        })?;
    let contracts_value = get(entry, "contracts");
    if !has_exact_fields(contracts_value, CONTRACT_FIELD_NAMES) {
        return Err(failure(
            SuccessorRegistryValidationPhase::ContractBinding,
            SuccessorRegistryErrorCode::Contract,
        ));
    }
    let values = std::array::from_fn(|index| {
        string(get(contracts_value, CONTRACT_FIELD_NAMES[index]))
            .unwrap_or_default()
            .to_owned()
    });
    let contracts = SuccessorCompiledContracts { values };
    if !contracts.iter().all(|(field, contract_id)| {
        valid_identifier(contract_id, SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX)
            && contract_id == SuccessorProfileContract::new(expected_profile, field).contract_id()
            && SuccessorProfileContract::from_contract_id(contract_id)
                == Some(SuccessorProfileContract::new(expected_profile, field))
    }) {
        return Err(failure(
            SuccessorRegistryValidationPhase::ContractBinding,
            SuccessorRegistryErrorCode::Contract,
        ));
    }
    if successor_profile_entry_hash(entry)? != entry_sha256
        || entry_sha256 != expected_profile.expected_entry_sha256()
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::EntryHash,
            SuccessorRegistryErrorCode::EntryHash,
        ));
    }
    let canonical_json = canonical_value(entry, SUCCESSOR_REGISTRY_CANONICAL_BYTES_MAX)
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::Limits,
                SuccessorRegistryErrorCode::Limit,
            )
        })?;
    Ok(SuccessorProfileEntry {
        source_language: source_language.to_owned(),
        semantic_profile: semantic_profile.to_owned(),
        semantic_parameters_schema: parameters_schema.to_owned(),
        selection_schema: selection_schema.to_owned(),
        foundation_descriptor,
        contracts,
        entry_sha256: entry_sha256.to_owned(),
        compiled_profile: expected_profile,
        canonical_json,
    })
}

fn validate_context_value(
    registry: &ValidatedSuccessorRegistry,
    context: &Value,
) -> Result<SuccessorSemanticContext, SuccessorRegistryValidationError> {
    if !semantic_context_outer_shape_is_valid(context)
        || string(get(context, "schema")) != Some(SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        ));
    }
    let identity = parse_registry_identity(get(context, "profile_registry")).ok_or_else(|| {
        failure(
            SuccessorRegistryValidationPhase::Shape,
            SuccessorRegistryErrorCode::Shape,
        )
    })?;
    if &identity != registry.identity() {
        return Err(failure(
            SuccessorRegistryValidationPhase::InstalledIdentity,
            SuccessorRegistryErrorCode::InstalledIdentity,
        ));
    }
    let source_language = string(get(context, "source_language")).expect("shape checked");
    let semantic_profile = string(get(context, "semantic_profile")).expect("shape checked");
    let entry = registry
        .lookup(source_language, semantic_profile)
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::ProfileLookup,
                SuccessorRegistryErrorCode::ContextMismatch,
            )
        })?;
    let entry_hash = string(get(context, "profile_entry_sha256")).expect("shape checked");
    if entry_hash != entry.entry_sha256() {
        return Err(failure(
            SuccessorRegistryValidationPhase::ProfileEntry,
            SuccessorRegistryErrorCode::ProfileEntryMismatch,
        ));
    }
    let parameters = get(context, "semantic_parameters");
    if !semantic_parameters_shape_is_valid(parameters) {
        return Err(failure(
            SuccessorRegistryValidationPhase::ParametersSchema,
            SuccessorRegistryErrorCode::ParametersMismatch,
        ));
    }
    let parameter_schema = string(get(parameters, "schema")).expect("shape checked");
    if parameter_schema != entry.semantic_parameters_schema() {
        return Err(failure(
            SuccessorRegistryValidationPhase::ParametersSchema,
            SuccessorRegistryErrorCode::ParametersMismatch,
        ));
    }
    if !canonical_size_within(parameters, SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX)
        || !validate_parameter_value(entry.compiled_profile(), get(parameters, "value"))
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::ParametersValue,
            SuccessorRegistryErrorCode::ParametersMismatch,
        ));
    }
    let foundation_descriptor = parse_foundation_ref(get(context, "foundation_descriptor"))
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::FoundationBinding,
                SuccessorRegistryErrorCode::Foundation,
            )
        })?;
    if &foundation_descriptor != entry.foundation_descriptor() {
        return Err(failure(
            SuccessorRegistryValidationPhase::FoundationBinding,
            SuccessorRegistryErrorCode::Foundation,
        ));
    }
    Ok(SuccessorSemanticContext {
        schema: SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA.to_owned(),
        profile_registry: identity,
        profile_entry_sha256: entry_hash.to_owned(),
        source_language: source_language.to_owned(),
        semantic_profile: semantic_profile.to_owned(),
        semantic_parameters: SuccessorSemanticParameters {
            schema: parameter_schema.to_owned(),
            value: get(parameters, "value").clone(),
        },
        foundation_descriptor,
    })
}

fn validate_selection_value(
    registry: &ValidatedSuccessorRegistry,
    context: &SuccessorSemanticContext,
    selection: &Value,
) -> Result<(), SuccessorRegistryValidationError> {
    if context.profile_registry() != registry.identity() {
        return Err(failure(
            SuccessorRegistryValidationPhase::InstalledIdentity,
            SuccessorRegistryErrorCode::InstalledIdentity,
        ));
    }
    let entry = registry
        .lookup(context.source_language(), context.semantic_profile())
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::ProfileLookup,
                SuccessorRegistryErrorCode::ContextMismatch,
            )
        })?;
    let Some(selection_schema) = string(get(selection, "schema")) else {
        return Err(failure(
            SuccessorRegistryValidationPhase::SelectionSchema,
            SuccessorRegistryErrorCode::SelectionMismatch,
        ));
    };
    if selection_schema != entry.selection_schema() {
        return Err(failure(
            SuccessorRegistryValidationPhase::SelectionSchema,
            SuccessorRegistryErrorCode::SelectionMismatch,
        ));
    }
    if !canonical_size_within(selection, SUCCESSOR_SELECTION_CANONICAL_BYTES_MAX)
        || !validate_profile_selection(entry.compiled_profile(), selection)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::SelectionValue,
            SuccessorRegistryErrorCode::SelectionMismatch,
        ));
    }
    Ok(())
}

fn validate_profile_selection(
    profile: SuccessorCompiledSemanticProfile,
    selection: &Value,
) -> bool {
    if profile == SuccessorCompiledSemanticProfile::CSharpPracticalV1 {
        return validate_practical_selection_value(selection).is_ok();
    }
    if !has_exact_fields(selection, &["schema", "value"])
        || get(selection, "value").as_object().is_none()
    {
        return false;
    }
    let value = get(selection, "value");
    match profile {
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => unreachable!(),
        SuccessorCompiledSemanticProfile::JavaScalarV0 => {
            crate::java_profile::valid_selection(value)
        }
        SuccessorCompiledSemanticProfile::GoFixedV0 => {
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
        SuccessorCompiledSemanticProfile::RustCheckedV0 => {
            if !has_exact_fields(value, &["package", "crate", "kind", "function"]) {
                return false;
            }
            let Some(package) = string(get(value, "package")) else {
                return false;
            };
            let Some(crate_name) = string(get(value, "crate")) else {
                return false;
            };
            let Some(function) = string(get(value, "function")) else {
                return false;
            };
            !package.is_empty()
                && !crate_name.is_empty()
                && matches!(string(get(value, "kind")), Some("lib" | "bin"))
                && function
                    .strip_prefix(crate_name)
                    .is_some_and(|suffix| suffix.starts_with("::") && suffix.len() > 2)
        }
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => {
            has_exact_fields(value, &["compilation", "contracts", "methods", "sources"])
                && string(get(value, "compilation")).is_some_and(|item| !item.is_empty())
                && nonempty_string_array(get(value, "contracts"))
                && nonempty_string_array(get(value, "methods"))
                && nonempty_string_array(get(value, "sources"))
        }
    }
}

fn validate_practical_selection_value(
    selection: &Value,
) -> Result<(), SuccessorRegistryValidationError> {
    let fields = [
        "schema",
        "compilation_id",
        "source_paths",
        "selected_root_ids",
        "sidecar_paths",
        "selection_sha256",
    ];
    if !has_exact_fields(selection, &fields)
        || string(get(selection, "schema")) != Some(CSHARP_PRACTICAL_SELECTION_SCHEMA)
        || !valid_compilation_id(string(get(selection, "compilation_id")).unwrap_or_default())
        || !sorted_unique_string_array(get(selection, "source_paths"), true, |path| {
            valid_normalized_path(path, Some("src/"), ".cs")
        })
        || !sorted_unique_string_array(get(selection, "selected_root_ids"), true, |id| {
            valid_source_callable_id(id)
        })
        || !sorted_unique_string_array(get(selection, "sidecar_paths"), false, |path| {
            valid_normalized_path(path, None, ".json")
        })
        || !string(get(selection, "selection_sha256")).is_some_and(valid_sha256)
    {
        return Err(failure(
            SuccessorRegistryValidationPhase::SelectionValue,
            SuccessorRegistryErrorCode::SelectionMismatch,
        ));
    }
    let expected = string(get(selection, "selection_sha256")).expect("shape checked");
    if csharp_practical_selection_hash(selection)? != expected {
        return Err(failure(
            SuccessorRegistryValidationPhase::SelectionValue,
            SuccessorRegistryErrorCode::SelectionMismatch,
        ));
    }
    Ok(())
}

fn validate_parameter_value(profile: SuccessorCompiledSemanticProfile, value: &Value) -> bool {
    match profile {
        SuccessorCompiledSemanticProfile::CSharpPracticalV1 => {
            valid_practical_parameter_values(value)
        }
        SuccessorCompiledSemanticProfile::CSharpScalarV0 => {
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
        SuccessorCompiledSemanticProfile::GoFixedV0 => {
            has_exact_fields(value, &["target_id", "pointer_width"])
                && string(get(value, "target_id")) == Some("linux/amd64")
                && unsigned_integer(get(value, "pointer_width")) == Some(64)
        }
        SuccessorCompiledSemanticProfile::JavaScalarV0 => {
            value
                == &serde_json::json!({
                    "annotation_processing": "none",
                    "encoding": "UTF-8",
                    "language_version": "25",
                    "preview": false,
                    "release": "25",
                    "target_id": "linux-x64"
                })
        }
        SuccessorCompiledSemanticProfile::RustCheckedV0 => {
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
    }
}

fn valid_practical_parameter_values(value: &Value) -> bool {
    value
        == &serde_json::json!({
            "check_overflow_default": true,
            "documentation_mode": "none",
            "language_version": "14.0",
            "nullable_context": "enable",
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

fn practical_parameters_document_is_valid(value: &Value) -> bool {
    has_exact_fields(value, &["schema", "value"])
        && string(get(value, "schema")) == Some(CSHARP_PRACTICAL_PARAMETERS_SCHEMA)
        && valid_practical_parameter_values(get(value, "value"))
        && canonical_size_within(value, SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX)
}

fn semantic_parameters_shape_is_valid(value: &Value) -> bool {
    has_exact_fields(value, &["schema", "value"])
        && string(get(value, "schema")).is_some()
        && get(value, "value").as_object().is_some()
}

fn semantic_parameters_document_is_valid(value: &Value) -> bool {
    if !semantic_parameters_shape_is_valid(value)
        || !canonical_size_within(value, SUCCESSOR_PARAMETERS_CANONICAL_BYTES_MAX)
    {
        return false;
    }
    let Some(profile) = string(get(value, "schema"))
        .and_then(SuccessorCompiledSemanticProfile::from_parameters_schema)
    else {
        return false;
    };
    validate_parameter_value(profile, get(value, "value"))
}

fn semantic_context_shape_is_valid(context: &Value) -> bool {
    semantic_context_outer_shape_is_valid(context)
        && string(get(context, "schema")) == Some(SUCCESSOR_SEMANTIC_CONTEXT_SCHEMA)
        && parse_registry_identity(get(context, "profile_registry")).is_some()
        && string(get(context, "profile_entry_sha256")).is_some_and(valid_sha256)
        && matches!(
            string(get(context, "source_language")),
            Some("csharp" | "go" | "java" | "rust")
        )
        && string(get(context, "semantic_profile"))
            .is_some_and(|value| valid_identifier(value, SUCCESSOR_REGISTRY_IDENTIFIER_BYTES_MAX))
        && semantic_parameters_document_is_valid(get(context, "semantic_parameters"))
        && parse_foundation_ref(get(context, "foundation_descriptor")).is_some()
}

fn semantic_context_outer_shape_is_valid(context: &Value) -> bool {
    has_exact_fields(
        context,
        &[
            "schema",
            "profile_registry",
            "profile_entry_sha256",
            "source_language",
            "semantic_profile",
            "semantic_parameters",
            "foundation_descriptor",
        ],
    ) && string(get(context, "schema")).is_some()
        && get(context, "profile_registry").as_object().is_some()
        && string(get(context, "profile_entry_sha256")).is_some()
        && string(get(context, "source_language")).is_some()
        && string(get(context, "semantic_profile")).is_some()
        && get(context, "semantic_parameters").as_object().is_some()
        && get(context, "foundation_descriptor").as_object().is_some()
}

fn request_shape_is_valid(request: &Value) -> bool {
    has_exact_fields(
        request,
        &["schema", "semantic_context", "selection", "request_sha256"],
    ) && string(get(request, "schema")).is_some()
        && get(request, "semantic_context").as_object().is_some()
        && get(request, "selection").as_object().is_some()
        && string(get(request, "request_sha256")).is_some()
}

fn request_document_is_valid(request: &Value) -> bool {
    request_shape_is_valid(request)
        && string(get(request, "schema")) == Some(SUCCESSOR_VALIDATED_REQUEST_SCHEMA)
        && semantic_context_shape_is_valid(get(request, "semantic_context"))
        && selection_document_is_valid(get(request, "selection"))
        && string(get(request, "request_sha256")).is_some_and(valid_sha256)
        && successor_validated_request_hash(request)
            .is_ok_and(|expected| string(get(request, "request_sha256")) == Some(expected.as_str()))
}

fn selection_document_is_valid(selection: &Value) -> bool {
    let Some(profile) = string(get(selection, "schema"))
        .and_then(SuccessorCompiledSemanticProfile::from_selection_schema)
    else {
        return false;
    };
    canonical_size_within(selection, SUCCESSOR_SELECTION_CANONICAL_BYTES_MAX)
        && validate_profile_selection(profile, selection)
}

fn candidate_entry_shape_is_valid(entry: &Value) -> bool {
    has_exact_fields(entry, ENTRY_FIELDS)
        && string(get(entry, "schema")).is_some()
        && string(get(entry, "source_language")).is_some()
        && string(get(entry, "semantic_profile")).is_some()
        && string(get(entry, "semantic_parameters_schema")).is_some()
        && string(get(entry, "selection_schema")).is_some()
        && get(entry, "foundation_descriptor").as_object().is_some()
        && get(entry, "contracts").as_object().is_some()
        && string(get(entry, "entry_sha256")).is_some()
}

fn parse_foundation_ref(value: &Value) -> Option<FoundationDescriptorRef> {
    if !has_exact_fields(value, &["schema", "id", "content_sha256"])
        || string(get(value, "schema")) != Some(FOUNDATION_DESCRIPTOR_SCHEMA)
        || string(get(value, "id")) != Some(FOUNDATION_DESCRIPTOR_ID)
        || string(get(value, "content_sha256")) != Some(FOUNDATION_DESCRIPTOR_CONTENT_SHA256)
    {
        return None;
    }
    Some(FoundationDescriptorRef {
        schema: FOUNDATION_DESCRIPTOR_SCHEMA.to_owned(),
        id: FOUNDATION_DESCRIPTOR_ID.to_owned(),
        content_sha256: FOUNDATION_DESCRIPTOR_CONTENT_SHA256.to_owned(),
    })
}

fn parse_registry_identity(value: &Value) -> Option<SuccessorRegistryIdentity> {
    if !has_exact_fields(value, &["schema", "id", "revision", "registry_sha256"])
        || string(get(value, "schema")) != Some(SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA)
        || string(get(value, "id")) != Some(SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA)
        || unsigned_integer(get(value, "revision")) != Some(SUCCESSOR_CANDIDATE_REVISION)
        || !string(get(value, "registry_sha256")).is_some_and(valid_sha256)
    {
        return None;
    }
    Some(SuccessorRegistryIdentity {
        schema: SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA.to_owned(),
        id: SUCCESSOR_SEMANTIC_REGISTRY_SCHEMA.to_owned(),
        revision: SUCCESSOR_CANDIDATE_REVISION,
        registry_sha256: string(get(value, "registry_sha256"))?.to_owned(),
    })
}

fn parse_canonical_document(transport: &[u8]) -> Result<Value, SuccessorRegistryValidationError> {
    let parsed = parse_strict_json(transport, DOCUMENT_LIMITS).map_err(|_| {
        failure(
            SuccessorRegistryValidationPhase::Transport,
            SuccessorRegistryErrorCode::Transport,
        )
    })?;
    let value = strict_to_serde(parsed);
    let canonical = canonical_value(&value, SUCCESSOR_COMPILED_PROFILE_CANONICAL_BYTES_MAX)
        .ok_or_else(|| {
            failure(
                SuccessorRegistryValidationPhase::Limits,
                SuccessorRegistryErrorCode::Limit,
            )
        })?;
    if canonical != transport {
        return Err(failure(
            SuccessorRegistryValidationPhase::CanonicalTransport,
            SuccessorRegistryErrorCode::Canonical,
        ));
    }
    Ok(value)
}

fn hash_without_field(
    value: &Value,
    excluded: &str,
    domain: HashDomain,
    maximum: u64,
    phase: SuccessorRegistryValidationPhase,
    code: SuccessorRegistryErrorCode,
) -> Result<String, SuccessorRegistryValidationError> {
    let strict = serde_to_strict(value).ok_or_else(|| failure(phase, code))?;
    let payload = strict
        .clone_without_fields(&[excluded])
        .map_err(|_| failure(phase, code))?;
    let canonical = canonical_json_bytes_bounded(
        &payload,
        usize::try_from(maximum).expect("successor canonical limit fits usize"),
    )
    .map_err(|_| failure(phase, code))?;
    hash_domain_separated_raw(domain, &canonical)
        .map(|digest| digest.to_hex())
        .map_err(|_| failure(phase, code))
}

fn hash_complete(
    value: &Value,
    domain: HashDomain,
    maximum: u64,
    phase: SuccessorRegistryValidationPhase,
    code: SuccessorRegistryErrorCode,
) -> Result<String, SuccessorRegistryValidationError> {
    let canonical = canonical_value(value, maximum).ok_or_else(|| failure(phase, code))?;
    hash_domain_separated_raw(domain, &canonical)
        .map(|digest| digest.to_hex())
        .map_err(|_| failure(phase, code))
}

fn canonical_value(value: &Value, maximum: u64) -> Option<Vec<u8>> {
    canonical_json_bytes_bounded(&serde_to_strict(value)?, usize::try_from(maximum).ok()?).ok()
}

fn canonical_size_within(value: &Value, maximum: u64) -> bool {
    canonical_value(value, maximum).is_some()
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
                .map(|(key, value)| (key, strict_to_serde(value)))
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
            .map(|(key, value)| Some((key.clone(), serde_to_strict(value)?)))
            .collect::<Option<Vec<_>>>()
            .map(StrictJsonValue::Object),
    }
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

fn valid_normalized_path(value: &str, prefix: Option<&str>, suffix: &str) -> bool {
    crate::source_map::is_portable_normalized_path(value)
        && value.ends_with(suffix)
        && prefix.is_none_or(|prefix| value.starts_with(prefix))
}

fn sorted_unique_string_array(
    value: &Value,
    require_nonempty: bool,
    predicate: impl Fn(&str) -> bool,
) -> bool {
    let Some(values) = value.as_array() else {
        return false;
    };
    if require_nonempty && values.is_empty() {
        return false;
    }
    let Some(strings) = values.iter().map(Value::as_str).collect::<Option<Vec<_>>>() else {
        return false;
    };
    strings.iter().all(|item| predicate(item)) && strings.windows(2).all(|pair| pair[0] < pair[1])
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

fn valid_source_callable_id(value: &str) -> bool {
    value
        .strip_prefix("mpk.csharp.source.")
        .is_some_and(valid_sha256)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn nonempty_string_array(value: &Value) -> bool {
    value.as_array().is_some_and(|values| {
        !values.is_empty()
            && values
                .iter()
                .all(|value| value.as_str().is_some_and(|value| !value.is_empty()))
    })
}

fn has_exact_fields(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|entries| {
        entries.len() == expected.len() && expected.iter().all(|field| entries.contains_key(*field))
    })
}

fn contract_index(field: SuccessorProfileContractField) -> usize {
    SUCCESSOR_CONTRACT_FIELDS
        .iter()
        .position(|candidate| *candidate == field)
        .expect("successor contract field is closed")
}

fn get<'a>(value: &'a Value, name: &str) -> &'a Value {
    value.get(name).unwrap_or(&Value::Null)
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("shape checked")
}

fn string(value: &Value) -> Option<&str> {
    value.as_str()
}

fn unsigned_integer(value: &Value) -> Option<u64> {
    value.as_u64()
}

fn failure(
    phase: SuccessorRegistryValidationPhase,
    code: SuccessorRegistryErrorCode,
) -> SuccessorRegistryValidationError {
    SuccessorRegistryValidationError { phase, code }
}
